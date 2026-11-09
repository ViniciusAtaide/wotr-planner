//! The MCP server: its tools, its instructions and its one resource, the glossary.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    ErrorData, Implementation, ListResourcesResult, PaginatedRequestParams,
    ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult, Resource,
    ResourceContents, ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{Json, RoleServer, ServerHandler, tool, tool_handler, tool_router};

use rules::{Build, Edit};

use crate::attack::weapon_argument as attack_weapon;
use crate::caster::caster_argument;
use crate::choose::choose;
use crate::code::{envelope, open};
use crate::compare::compare;
use crate::describe::describe;
use crate::options::options;
use crate::planner::Planner;
use crate::shapes::{
    BuildResult, ChooseArgs, ChooseResult, CompareArgs, CompareResult, DescribeArgs, Item,
    NewBuildArgs, OneOrMany, OptionsArgs, OptionsResult, SearchArgs, SearchResult, SheetArgs,
    SheetResult, WithdrawArgs,
};
use crate::sheet::sheet;
use crate::withdraw::withdraw;

/// What every client is told on connecting: what to know before planning (ticket 117; the pick order within a level,
/// ticket 126), the workflow, how to batch,
/// value and branch, and the honesty boundary (plan, ticket 104, section 1). Short and imperative; the tools'
/// descriptions hold the details.
pub const INSTRUCTIONS: &str = "\
wotr-planner builds Pathfinder: Wrath of the Righteous characters with the rules engine of a build planner. Every \
Build its tools return is one the planner's rules allow.

Before you plan:
- Point-buy scores start at 10 and run from 7 to 18 on a 25-point budget; `sheet` gives `point_buy_spent` and \
`point_buy_budget`.
- Choose the faith (a deity, or atheism) before the alignment: no alignment is open until the faith is chosen, and a \
deity allows only some.
- `sheet` gives each Mythic rank's `character_level`, the character level its ascension comes at.
- Within a level, picks are checked in order: class selections and class feat slots (`slot.selection.N.I`, \
`slot.feat.N.class.I`), race slots, the general slot (`slot.feat.N.general.0`), the slots options and feats give, \
then each Mythic rank that comes at that level (`slot.mythic.R`, `slot.path-feat.R.I`, `slot.path-selection.R.I`). \
A pick that needs another taken at the same level goes in the later slot: Spell Focus in the sorcerer's level-1 \
class slot and Greater Spell Focus in the general slot, not the reverse. A refusal that says the prerequisite's pick \
\"comes after this slot\" names that pick's slot: swap the two picks, or take this one at a later level.
- A class split decides what the last levels can take: Fighter 5 / Barbarian 15 gives up Barbarian levels 16 to 20 \
and whatever needs them. Before you commit to a class split, branch the candidates from one code (Barbarian 20, and \
Fighter 5 / Barbarian 15), add their levels, and `compare` the branches: it lists the feats and features each gains \
that the other doesn't. `options` with `only_open: false` says what a closed option still needs (\"Needs Barbarian \
level 16 (have 14)\").

Workflow:
1. Resolve every name the user gives with `search` (names, keys and initials: \"BFT\" finds the Brown-Fur \
Transmuter; words of what an item does, such as \"DC\", also find it by its summary). If a name could mean more \
than one item, ask the user which one before you build.
2. Start with `new_build`. Every Build tool takes the Build's code (`build`) and returns a new code, the planner \
`link` and `todo`, the choices still open, each by its address (`options` only with `envelope: true`).
3. Fill the Build start: race, racial ability, faith, alignment, background, point-buy scores.
4. Add levels with `level.N.class` (N one past the last level adds one) and fill each level's open choices: feats, \
Feature selections, ability increases, skill points.
5. Fill the Mythic ranks once their levels are reached: the first ascension at MR 1, the path at MR 3, and each \
rank's picks.
6. Give the user the final `link`.

Batching: send many steps in one `choose`. It stops at the first refused step: read `rejected` (its issue and \
reason), fix that step, and resend the rest from the code it returned.

Values: take them from `options`, which lists what a choice can take now and why the rest is closed. A \
parameterised feat or mythic ability (Favorite Metamagic: a metamagic) needs `{key, parameter}`: list them with \
`options {values: true}`. A `todo` entry on a filled slot is a pick whose value is still open: choose the value \
there. A spellcasting prestige class needs `advances`, the classes it advances (`[]` for none). A point-buy score \
or a skill's ranks take the number to reach; `level.N.skill` takes a map `{skill: ranks}` to spend a level's skill \
points in one step. \
`compact: true` on `options` and `search` leaves out summaries (and `search`'s prerequisites and guide links) when \
you need only the names.

Branching: for alternatives (Lich or Aeon), build the shared part once, `choose` each alternative from that same \
code, fill each branch, then `compare` the codes. Report its differences as facts, without ranking them.

The honesty boundary:
- State as fact only what the tools return: every choice in the Build is legal under the planner's rules at the \
level it is taken; the link opens in the planner; and the features, numbers (BAB, saves, scores, skill ranks, \
caster level from class levels, spell-level access) and path features the sheet lists.
- The attack line (`sheet` or `compare` with a `weapon`) is fact too: each attack's bonus, damage per hit, Power \
Attack's trade and the critical range, from BAB, the character's own scores and the weapon feats held. It leaves \
out rage, stances, mythic feats, class features, gear and buffs (`attack_leaves_out` lists all it leaves out): \
label anything you add for them as your own game knowledge.
- The caster line (`sheet` or `compare` with a `caster`: a school, and optionally a subschool, an energy type and \
metamagic) is fact too: the save DC of each spell level the casting class can cast, with its breakdown by source \
(the casting ability's modifier, Spell Focus and its kin, racial DCs, bloodline arcana, School Power), caster \
level against spell resistance and the concentration bonus, from the class levels, the character's own scores and \
the items held. It leaves out gear, buffs, abilities switched on in a fight (the Aeon's gaze), bonuses that grow \
with class level and what changes the target's roll instead of the DC (`caster_leaves_out` lists all it leaves \
out): label anything you add for them as your own game knowledge. Which spells exist, and their schools and \
subschools, stay your own knowledge.
- The totals (`sheet`'s `totals`) are fact too: initiative, each save and each skill's total, with what each \
counts, from the character's own scores, the class progressions, the skill ranks and class skills and the flat \
bonuses of the items held (Improved Initiative, Iron Will, Skill Focus, racial and background bonuses). They \
leave out gear, buffs, bonuses against one kind of effect or for one use of a skill (Persuasion to intimidate) and \
bonuses that grow with class level (`totals_leave_out` lists all they leave out): label anything you add for \
them as your own game knowledge.
- Most items carry a `summary`: what the guide says the item does, in one sentence in the planner's own words, \
checked against the guide. Cite it as the planner's summary of the guide, not as a rule the planner enforces: the \
engine checks prerequisites, not effects.
- A few items carry `interactions` in `describe`: what the guide says about how the item combines with others \
(or that it doesn't say), in the planner's own words. Cite it as the planner's note on the guide; where it says \
the guide doesn't say, any answer is your own game knowledge.
- Label everything else as your own game knowledge: what an item does beyond its summary, how items interact, \
spells and spell availability, durations, caster level arithmetic beyond the caster line, item and buff effects, summons' stats, and which \
choice is better. The planner checks none of it.
- Guide links (`guide_url`) are for the user to read: never fetch them yourself.

The resource `wotr://glossary` defines the planner's terms (Build start, Slot, Withdrawal, Path switch, …).";

/// The resource that serves the glossary.
pub const GLOSSARY_URI: &str = "wotr://glossary";

/// The project's glossary (`GLOSSARY.md`), embedded at build time so the binary needs no file at run time.
pub const GLOSSARY: &str = include_str!("../../../GLOSSARY.md");

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
        description = "Find game items by name, key or initials (\"BFT\" finds the Brown-Fur Transmuter), or by \
            what they do: an item whose summary holds a query word as a whole word (\"DC\" finds Spell Focus and \
            the Kitsune) matches too, after every name match, marked `matched: \"summary\"`, those holding every \
            query word first; `note` says when `limit` left some out. Kinds: \
            class, archetype, path (Mythic path), feat, mythic_ability, mythic_feat, option (an option of a Feature \
            selection: an arcanist exploit, a rogue talent, a bloodline, a Lich power), race, deity, background. \
            Each match has the key `describe` takes, its short summary when the data has one, its parent (an \
            archetype's class, an option's selection), its category, its prerequisite in the planner's words (a long \
            one shortened: \"one of 49 hexes (Air Barrier, …)\"; `describe` gives it whole), and a guide link for \
            the user (never fetch it). `compact: true` gives each match as kind, key, name and category only. Best \
            match first. `category` filters by a feat's category (\"Weapon\", \"Magic\", …), a mythic pick's group \
            (\"Melee\", \"Ranged\", …) or an option's Feature selection (\"Rage Power\"), one or a list; with \
            `kinds` or `category`, `query` may be empty to list every item they allow (\"every Melee mythic \
            feat\"). `limit` is at most 100 (a larger one returns 100, with a `note`); `total` counts every match, \
            beyond `limit`. When nothing matches, `note` says so and names \
            only near names: the data holds what the guide lists, so a name it lacks isn't in the planner.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<Json<SearchResult>, String> {
        self.blocking(move |planner| {
            let categories = args.category.map(OneOrMany::into_vec);
            planner
                .index
                .search(
                    &args.query,
                    args.kinds.as_deref(),
                    categories.as_deref(),
                    args.limit,
                    args.compact.unwrap_or(false),
                )
                .map(Json)
        })
        .await
    }

    #[tool(
        description = "One item by kind and key (both from `search`): its name, parent, category, prerequisite, \
            guide link, and what its kind has. A class or archetype: its rows by class level (BAB, saves, spell \
            levels, features, granted feats, feat slots, Feature selections; an archetype's rows merged over its \
            class's), plus its archetypes and its selections with their options. A Mythic path: its features, feat \
            slots and selections by Mythic rank, and its selections with their options. An option: its rows by \
            level, when it has any. A deity: alignments and domains. The item, and each feature and selection option \
            it lists, carries its short summary when the data has one; beyond those, what an item does is not in the \
            planner's data.",
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
            caster level (class levels only) and spell-level access at the last level, class skills and notes; \
            `totals` at the last level: initiative, each save and each skill's total, each with what it counts \
            (ability modifiers, base saves, ranks, the class skill bonus, the flat bonuses of the items held). \
            Every choice has its address (`start.race`, `level.4.class`, `slot.feat.3.general.0`, …) for `options`, \
            its value and its name. `removable: true` also marks which held choices can be withdrawn now. \
            `weapon` (a weapon key) adds each level's attack line for that base weapon. `caster` ({school, \
            subschool?, energy?, metamagic?}) adds each level's caster line for those spells: the save DC of each \
            spell level castable (and heightened, with Heighten Spell) broken down by source, caster level against \
            spell resistance and the concentration bonus. Plus the envelope \
            (`build`, `link`, `todo`).",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn sheet(
        &self,
        Parameters(args): Parameters<SheetArgs>,
    ) -> Result<Json<SheetResult>, String> {
        self.blocking(move |planner| {
            let build = open(planner, &args.build)?;
            let weapon = attack_weapon(&planner.engine, args.weapon.as_deref())?;
            let spell = caster_argument(args.caster.as_ref())?;
            Ok(Json(sheet(
                planner,
                &build,
                args.removable,
                weapon.as_ref(),
                spell,
            )))
        })
        .await
    }

    #[tool(
        description = "What one choice of a Build can take, from the planner's own picker or stepper for it: each \
            option's `value` (what `choose` takes), its name, its short summary when the data has one, and its \
            status: \"ok\", or {issue, reason} with the \
            engine's issue (`unmet`, `already_held`, `max_times`, `class_level_cap`, `not_from_path`, \
            `deity_alignment`, `needs_faith`, `deity_domain`, `breaks_later`, …) and the planner's reason. `choice` \
            is an address from `sheet` or `todo`: a slot (`slot.feat.3.general.0`), `level.N.class` (N one past the \
            last level adds a level), `level.N.increase`, `level.N.skill` (one rank more in a skill), \
            `level.N.skill.<skill>` and `start.score.<ability>` (a step down or up), the Build start's \
            `start.race|faith|alignment|background|racial-ability`, or `mythic.first-ascension|path|switch` once \
            the rank is reached. By default only the options open now are listed and `closed` counts the rest; \
            `only_open: false` lists them with their reasons. `values: true` lists a parameterised feat once per \
            value. `query` filters by name or key. `compact: true` leaves out the summaries. It changes nothing, so \
            the envelope (`build`, `link`, `todo`) comes only with `envelope: true`. Errors: `no_such_choice` for an \
            address the Build lacks, `stale_link` for a code that doesn't open.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn options(
        &self,
        Parameters(args): Parameters<OptionsArgs>,
    ) -> Result<Json<OptionsResult>, String> {
        self.blocking(move |planner| Ok(Json(options(planner, args)?)))
            .await
    }

    #[tool(
        description = "Make choices in a Build, in order: `steps` is a list of {choice, value}, each `choice` an \
            address (from `todo`, `sheet` or `options`) and each `value` one `options` lists for it (with `values: \
            true` for a parameterised feat's {key, parameter}; a prestige class that asks needs `advances`, `[]` for \
            none). Each step takes only an option the planner offers and finds open, by the planner's own edit, and \
            the result is checked again, so the Build stays possible. A point-buy score (`start.score.int`) or a \
            skill's ranks at a level (`level.N.skill.<skill>`) take the number to reach, reached one step at a time, \
            each checked; the current value is a no-op. `level.N.skill` takes a map {skill: ranks} (`{\"athletics\": \
            1, \"perception\": 1}`), each skill's ranks at that level reached the same way, all in one step. The first \
            step refused stops the rest (a refused rank refuses its whole step): `applied` counts the steps applied, \
            and `rejected` {step, choice, issue, reason} gives the engine's issue (`unmet`, `already_held`, \
            `breaks_later`, …) or the server's (`not_an_option`, with the closest values; `needs_parameter`, a \
            parameterised feat given by its key alone, with what it takes and its open values; `no_such_choice`). \
            Returns the envelope (`build`, `link`, `todo`) of the Build as the applied steps left \
            it. A code that doesn't open is `stale_link`.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn choose(
        &self,
        Parameters(args): Parameters<ChooseArgs>,
    ) -> Result<Json<ChooseResult>, String> {
        self.blocking(move |planner| Ok(Json(choose(planner, args)?)))
            .await
    }

    #[tool(
        description = "Take one choice back, by the planner's own withdrawal: a pick cleared, the last level \
            removed (`level.N`), a level's ability increase or a skill's ranks gone, a class's archetype or advanced \
            classes unchosen (`class.<class>.archetype|advances`), a point-buy score back to 7, a Build start choice, \
            or the first ascension, path or Path switch unchosen. Only a choice nothing later depends on comes off: \
            otherwise `not_removable` (`sheet` with `removable: true` lists what can come off now, so chains come \
            off from the leaves inward). A level's class isn't withdrawn (`choose` another class, or withdraw the \
            last level), nor the racial ability (change the race, or choose another ability). Errors: \
            `not_removable`, `no_such_choice` (nothing held there), `stale_link`. Returns the envelope (`build`, \
            `link`, `todo`).",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn withdraw(
        &self,
        Parameters(args): Parameters<WithdrawArgs>,
    ) -> Result<Json<BuildResult>, String> {
        self.blocking(move |planner| Ok(Json(withdraw(planner, args)?)))
            .await
    }

    #[tool(
        description = "Compare 1 to 4 Builds (`builds`: their codes, for example two branches of one Build), from \
            the planner's sheets. For each Build, a summary at its last level: classes and levels, BAB, saves, \
            scores, caster level (class levels only), spell-level access, Mythic rank, first ascension, path, Path \
            switch, level cap, link, with `weapon` (a weapon key) its attack line, and with `caster` ({school, \
            subschool?, energy?, metamagic?}) its caster line. Then only what differs: the Build start's choices; \
            by character level, the \
            numbers (class level, BAB, saves, scores), the choices by address and the feats and features each Build \
            gains that another doesn't; by Mythic rank, the status, path and choices; and `path_features`, by rank, \
            the path features each Build has that another lacks. Every per-Build list follows the order of \
            `builds`. Facts only: it doesn't say which Build is better. One code gives its summary alone, with \
            no differences. Errors: `invalid_arguments` (not 1 to 4 codes), `stale_link` (naming the code that \
            doesn't open).",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn compare(
        &self,
        Parameters(args): Parameters<CompareArgs>,
    ) -> Result<Json<CompareResult>, String> {
        self.blocking(move |planner| Ok(Json(compare(planner, args)?)))
            .await
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(
            Implementation::new("wotr-planner", env!("CARGO_PKG_VERSION"))
                .with_title("WotR Build Planner"),
        )
        .with_instructions(INSTRUCTIONS)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult::with_all_items(vec![glossary()]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        if request.uri != GLOSSARY_URI {
            return Err(ErrorData::resource_not_found(
                format!(
                    "No resource `{}`: the one resource is `{GLOSSARY_URI}`.",
                    request.uri
                ),
                None,
            ));
        }
        let text = ResourceContents::text(GLOSSARY, GLOSSARY_URI).with_mime_type("text/markdown");
        Ok(ReadResourceResult::new(vec![text]).into())
    }
}

/// The glossary resource, as `resources/list` lists it.
fn glossary() -> Resource {
    Resource::new(GLOSSARY_URI, "glossary")
        .with_title("WotR Build Planner glossary")
        .with_description(
            "The planner's terms: Build, Build start, Level-up, Mythic ascension, Slot, Pick, Withdrawal, Path \
             switch, Option level, … The tools speak in these words.",
        )
        .with_mime_type("text/markdown")
        .with_size(GLOSSARY.len() as u64)
}
