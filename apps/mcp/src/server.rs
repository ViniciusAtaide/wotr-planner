//! The MCP server: its tools and its instructions.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{Json, ServerHandler, tool, tool_handler, tool_router};

use rules::{Build, Edit};

use crate::code::{envelope, open};
use crate::describe::describe;
use crate::options::options;
use crate::planner::Planner;
use crate::shapes::{
    BuildResult, DescribeArgs, Item, NewBuildArgs, OptionsArgs, OptionsResult, SearchArgs,
    SearchResult, SheetArgs, SheetResult,
};
use crate::sheet::sheet;

/// What every client is told on connecting: the workflow and the honesty boundary (plan, ticket 104, section 1).
/// A first version; ticket 110 finishes it.
pub const INSTRUCTIONS: &str = "\
wotr-planner answers from the rules engine and game data of a Pathfinder: Wrath of the Righteous build planner.

Workflow: resolve every name the user gives with `search` (it matches names, keys and initials: \"BFT\" finds the \
Brown-Fur Transmuter), and ask the user whenever a name matches more than one item. Use `describe` for an item's \
details: a class's or archetype's rows by level, a Mythic path's features by rank.

Building: `new_build` starts a Build and returns its code (`build`), its planner link and its open choices \
(`todo`), each by address. Every Build tool takes the code and returns a new one; keep an old code to branch. \
`sheet` shows the Build level by level with every choice's address, and `options` what a choice can take now, \
each option with the engine's status and reason.

What you may state as fact: what these tools return from the planner's data (names, keys, prerequisites, rows by \
level, features by rank) and, once the building tools exist, that every choice in a Build is legal under the \
planner's rules at the level it is taken, and the numbers its sheet lists.

What you must label as your own game knowledge, outside the planner's checks: what a feature, feat or spell does; \
spell availability and durations; caster level arithmetic; item and buff effects; summons' stats; and which choice \
is better. The data holds names, prerequisites and guide links, and almost no descriptions.

Guide links (`guide_url`) are for the user to read: never fetch them yourself.";

/// The server: the loaded data and the tools over it. Each tool call runs on a blocking thread.
#[derive(Clone, Debug)]
pub struct Server {
    planner: Planner,
    tool_router: ToolRouter<Self>,
}

impl Server {
    pub fn new(planner: Planner) -> Self {
        Self {
            planner,
            tool_router: Self::tool_router(),
        }
    }

    /// Runs `work` on a blocking thread with a copy of the planner.
    async fn blocking<T: Send + 'static>(
        &self,
        work: impl FnOnce(&Planner) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        let planner = self.planner.clone();
        tokio::task::spawn_blocking(move || work(&planner))
            .await
            .map_err(|error| format!("the tool failed: {error}"))?
    }
}

#[tool_router]
impl Server {
    #[tool(
        description = "Find game items by name, key or initials (\"BFT\" finds the Brown-Fur Transmuter). Kinds: \
            class, archetype, path (Mythic path), feat, mythic_ability, mythic_feat, option (an option of a Feature \
            selection: an arcanist exploit, a rogue talent, a bloodline, a Lich power), race, deity, background. \
            Each match has the key `describe` takes, its parent (an archetype's class, an option's selection), its \
            category, its prerequisite in the planner's words, and a guide link for the user (never fetch it). Best \
            match first.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<Json<SearchResult>, String> {
        self.blocking(move |planner| {
            let matches = planner
                .index
                .search(&args.query, args.kinds.as_deref(), args.limit)
                .ok_or("The query is empty: give a name, a key or initials.")?;
            Ok(Json(SearchResult { matches }))
        })
        .await
    }

    #[tool(
        description = "One item by kind and key (both from `search`): its name, parent, category, prerequisite, \
            guide link, and what its kind has. A class or archetype: its rows by class level (BAB, saves, spell \
            levels, features, granted feats, feat slots, Feature selections; an archetype's rows merged over its \
            class's), plus its archetypes and its selections with their options. A Mythic path: its features, feat \
            slots and selections by Mythic rank, and its selections with their options. An option: its rows by \
            level, when it has any. A deity: alignments and domains. Facts from the planner's data; what a feature \
            does is not in it.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn describe(
        &self,
        Parameters(args): Parameters<DescribeArgs>,
    ) -> Result<Json<Item>, String> {
        self.blocking(move |planner| describe(planner, args.kind, args.key.trim()).map(Json))
            .await
    }

    #[tool(
        description = "Start a Build: an empty one, or the Build of a planner link or code (`link`, opened only if \
            the planner's rules find it possible; otherwise `stale_link`). Returns the envelope every Build tool \
            returns: `build`, the code the other tools take (a new one after every change; keep an old one to \
            branch); `link`, the planner link for the user; and `todo`, the open choices by address.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn new_build(
        &self,
        Parameters(args): Parameters<NewBuildArgs>,
    ) -> Result<Json<BuildResult>, String> {
        self.blocking(move |planner| {
            let mut build = match args.link.as_deref() {
                Some(link) => open(planner, link)?,
                None => Build::default(),
            };
            if let Some(name) = args.name {
                let name = name.trim();
                let name = (!name.is_empty()).then(|| name.to_owned());
                planner.engine.apply(&mut build, &Edit::SetName(name));
            }
            let sheet = planner.engine.evaluate(&build);
            Ok(Json(BuildResult {
                envelope: envelope(&planner.engine, &build, &sheet),
            }))
        })
        .await
    }

    #[tool(
        description = "A Build's sheet, from the planner's rules engine: the Build start's choices; per character \
            level its class and class level, BAB, saves, ability scores, skill points and ranks, the feats and \
            features it adds, and its choices; per Mythic rank its status, path, choices and path features; the \
            caster level (class levels only) and spell-level access at the last level, class skills and notes. \
            Every choice has its address (`start.race`, `level.4.class`, `slot.feat.3.general.0`, …) for `options`, \
            its value and its name. `removable: true` also marks which held choices can be withdrawn now. Plus the \
            envelope (`build`, `link`, `todo`).",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn sheet(
        &self,
        Parameters(args): Parameters<SheetArgs>,
    ) -> Result<Json<SheetResult>, String> {
        self.blocking(move |planner| {
            let build = open(planner, &args.build)?;
            Ok(Json(sheet(planner, &build, args.removable)))
        })
        .await
    }

    #[tool(
        description = "What one choice of a Build can take, from the planner's own picker or stepper for it: each \
            option's `value` (what `choose` takes), its name, and its status: \"ok\", or {issue, reason} with the \
            engine's issue (`unmet`, `already_held`, `max_times`, `class_level_cap`, `not_from_path`, \
            `deity_alignment`, `needs_faith`, `deity_domain`, `breaks_later`, …) and the planner's reason. `choice` \
            is an address from `sheet` or `todo`: a slot (`slot.feat.3.general.0`), `level.N.class` (N one past the \
            last level adds a level), `level.N.increase`, `level.N.skill` (one rank more in a skill), \
            `level.N.skill.<skill>` and `start.score.<ability>` (a step down or up), the Build start's \
            `start.race|faith|alignment|background|racial-ability`, or `mythic.first-ascension|path|switch` once \
            the rank is reached. By default only the options open now are listed and `closed` counts the rest; \
            `only_open: false` lists them with their reasons. `values: true` lists a parameterised feat once per \
            value. `query` filters by name or key. Errors: `no_such_choice` for an address the Build lacks, \
            `stale_link` for a code that doesn't open. Plus the envelope (`build`, `link`, `todo`).",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn options(
        &self,
        Parameters(args): Parameters<OptionsArgs>,
    ) -> Result<Json<OptionsResult>, String> {
        self.blocking(move |planner| Ok(Json(options(planner, args)?)))
            .await
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("wotr-planner", env!("CARGO_PKG_VERSION"))
                    .with_title("WotR Build Planner"),
            )
            .with_instructions(INSTRUCTIONS)
    }
}
