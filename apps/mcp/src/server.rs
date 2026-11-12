//! The MCP server: its tools, its instructions and its two resources, the glossary and the build guide.

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
use crate::rank::{ImproveArgs, ImproveResult, improve};
use crate::shapes::{
    BuildResult, ChooseArgs, ChooseResult, CompareArgs, CompareResult, DescribeArgs, Item,
    NewBuildArgs, OneOrMany, OptionsArgs, OptionsResult, SearchArgs, SearchResult, SheetArgs,
    SheetResult, ValidateArgs, ValidateResult, WithdrawArgs, WithdrawResult,
};
use crate::sheet::sheet;
use crate::validate::validate;
use crate::withdraw::withdraw;

/// What every client is told on connecting: where the build guide is, the loop in short, and the honesty boundary in
/// short (plan, ticket 104, section 1). Short on purpose: a client may cut long instructions, so the
/// rest lives in the guide, `wotr://build-guide`.
pub const INSTRUCTIONS: &str = "\
wotr-planner builds Pathfinder: Wrath of the Righteous characters with the rules engine of a build planner. Every \
Build its tools return is one the planner's rules allow.

Read the build guide before your first Build: the resource `wotr://build-guide` (in Claude Code, the `wotr-build` \
skill holds the same text). It gives what to know before you plan, the workflow, the choice addresses and their \
values, batching, branching, ranking by a number and the honesty boundary in full. In short:
1. Resolve each name the user gives with `search`; if a name could mean more than one item, ask the user which one.
2. `new_build`, then plan the whole Build as one map of address → value and check it with `validate`; `choose` what \
is still open, reading `rejected` when a step is refused.
3. Give the user the final `link`.

Gear: a Build has no gear unless the user asks for it; never add an item yourself. To add one the user names, \
`search` with `kinds: [\"gear\"]` (only that finds gear; a name, or a bonus: \"Dex\"), then `choose` its key at \
its place: `gear.belt`, \
`gear.headwear`, `gear.ring.0`, `gear.ring.1`, `gear.body` (robe or armour), … or `gear.usable.<key>` for a \
manual or tome (`options` on a place lists its items; in `validate`'s map, the same addresses). An item's \
always-on bonuses count at every level, for prerequisites and every number (never for skill points), so it can \
open a feat (`choose` the item before that feat): `options` marks such an option `{\"with_gear\": …}`, and \
`sheet`'s `gear_dependent` lists each choice only gear allows with its item; a `choose` or `withdraw` step that \
leaves one so names it in `leans_on_gear`. Report them, naming each pick and its item. An item a pick needs can't be withdrawn, or replaced by one that doesn't keep the pick, until the pick is. \
The planner's items are those that \
give an ability score bonus, with manuals and tomes; weapons' and armour's enhancements, armour class and other \
items aren't modelled yet: what they add is your own game knowledge.

The honesty boundary, in short:
- State as fact only what the tools return: every choice in the Build is legal under the planner's rules, the link \
opens, and the sheet, the attack line, the caster line and the totals are the planner's numbers.
- The numbers count the Build's gear, and only it; `sheet`'s `naked` gives them without it. `compare` and \
`improve` rank by naked numbers unless you pass `with_gear: true` (`numbers` says which). The attack line's weapon \
is the Build's Weapon Focus weapon unless you give one. Each line says what it leaves out (`attack_leaves_out`: \
rage, stances, mythic feats, class features, buffs, gear but for its ability score bonuses; `caster_leaves_out`; \
`totals_leave_out`), and an item's `other_effects` count in no number: label anything you add for them as your \
own game knowledge.
- When the planner ranks Builds or options by a number (`compare` or `improve` with `by`), that ranking is a fact. \
Report `compare`'s other differences as facts, without ranking them by your own judgement, and label which choice \
is better overall as your judgement.
- Summaries and `interactions` (`describe`) are the planner's notes on the guide, not rules it enforces.
- Guide links (`guide_url`) are for the user to read: never fetch them yourself.

The resource `wotr://glossary` defines the planner's terms (Build start, Slot, Withdrawal, Path switch, …).";

/// The resource that serves the glossary.
pub const GLOSSARY_URI: &str = "wotr://glossary";

/// The project's glossary (`GLOSSARY.md`), embedded at build time so the binary needs no file at run time.
pub const GLOSSARY: &str = include_str!("../../../GLOSSARY.md");

/// The resource that serves the build guide.
pub const BUILD_GUIDE_URI: &str = "wotr://build-guide";

/// The `wotr-build` skill's file, embedded at build time: one text for Claude Code's skill and the MCP resource.
const BUILD_GUIDE_FILE: &str = include_str!("../../../.claude/skills/wotr-build/SKILL.md");

/// The build guide: the skill's text without its frontmatter (the skill's name and description).
pub fn build_guide() -> &'static str {
    BUILD_GUIDE_FILE
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map_or(BUILD_GUIDE_FILE, |(_, body)| body.trim_start())
}

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
            selection: an arcanist exploit, a rogue talent, a bloodline, a Lich power), race, deity, background, \
            spell (the spell catalogue: a spell's category is its school, and it carries its descriptors), \
            spell_list (a class's, an archetype's or a Mythic path's spells by level), gear (an item a Build holds \
            only when the user asks, so found only when `kinds` names `gear`, never by an unfiltered search: \
            matched by name, by its place (\"belt\") and by its bonuses' words (\"Dex\", \
            \"enhancement\"); its `category` is its slot (\"Belt\", \"Ring\", \"Usable\"), and it carries \
            `other_effects` (text only, in no number) and `places`, the addresses `choose` takes its key at). \
            Each match has the key `describe` takes, its short summary when the data has one, its parent (an \
            archetype's class, an option's selection), its category, its prerequisite in the planner's words (a long \
            one shortened: \"one of 49 hexes (Air Barrier, …)\"; `describe` gives it whole), and a guide link for \
            the user (never fetch it). `compact: true` gives each match as kind, key, name and category only (no \
            `other_effects` or `places`). Best \
            match first. `category` filters by a feat's category (\"Weapon\", \"Magic\", …), a mythic pick's group \
            (\"Melee\", \"Ranged\", …), an option's Feature selection (\"Rage Power\"), or a spell's school \
            (\"Enchantment\") or descriptor, its subschool among them (\"Compulsion\", \"Mind-Affecting\"), one or \
            a list; with \
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
            level, when it has any. A deity: alignments and domains. A class, an archetype or a path also names the \
            spell lists it casts from (`spell_lists`). A spell: its `facts` (school, descriptors, target, duration, \
            saving throw, spell resistance, casting time, material component) and every spell list it's on \
            (`lists`), each with the spell's level there, its owner, the classes and archetypes that use it, and \
            any footnote (\"Flame Dancer only\"). A spell list: its owner, `used_by` and its spells by level, \
            each with its school and descriptors. A gear item: its summary (its always-on bonuses), `other_effects` \
            and `places`. The item, and each feature and selection option \
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
            Each level's attack line (`attack`, naming its weapon): for `weapon` (a weapon key) when given, else for \
            the weapon of the Build's earliest Weapon Focus (by level; within a level, the first weapon key), and \
            none without one. `caster` ({school, subschool?, \
            energy?, metamagic?}) adds each level's caster line for those spells: the save DC of each spell level \
            castable (and heightened, with Heighten Spell) broken down by source, caster level against spell \
            resistance and the concentration bonus. A Build with gear (only what the user chose) also has `gear`, \
            each item at its address with its summary and `other_effects`; `gear_dependent`, each choice only the \
            gear allows, with the items it needs (also `with_gear` on its line), to report as such; and `naked`, \
            the numbers its gear changes (scores, attack and caster lines by level, totals) without the gear. Every \
            other number counts the gear's always-on bonuses (a level's skill points never do). Plus the envelope \
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
            Ok(Json(sheet(planner, &build, args.removable, weapon, spell)))
        })
        .await
    }

    #[tool(
        description = "What one choice of a Build can take, from the planner's own picker or stepper for it: each \
            option's `value` (what `choose` takes), its name, its short summary when the data has one, and its \
            status: \"ok\"; `{\"with_gear\": \"With <item>\"}`, open only because of the Build's gear; \
            `{\"leans_on_gear\": …}`, open but leaving other choices allowed only because of the gear; or \
            {issue, reason} with the \
            engine's issue (`unmet`, `already_held`, `max_times`, `class_level_cap`, `not_from_path`, \
            `deity_alignment`, `needs_faith`, `deity_domain`, `breaks_later`, …) and the planner's reason. `choice` \
            is an address from `sheet` or `todo`: a slot (`slot.feat.3.general.0`), `level.N.class` (N one past the \
            last level adds a level), `level.N.increase`, `level.N.skill` (one rank more in a skill), \
            `level.N.skill.<skill>` and `start.score.<ability>` (a step down or up), the Build start's \
            `start.race|faith|alignment|background|racial-ability`, `mythic.first-ascension|path|switch` once \
            the rank is reached, or a gear place (`gear.belt`, `gear.boots`, `gear.bracers`, `gear.cloak`, \
            `gear.glasses`, `gear.gloves`, `gear.headwear`, `gear.necklace`, `gear.ring.0`, `gear.ring.1`, \
            `gear.body`, `gear.weapon`: \"None\" (`none`) and each item it takes, with its summary and \
            `other_effects`) or `gear.usable.<item key>` (a manual, tome or other usable item: `none` or its key). \
            An item a choice needs can't be taken off (\"None\" is locked with the choice's name), nor replaced by \
            an item that would refuse the choice (locked the same way); an item that keeps it is open. By default only the options open now are listed and `closed` counts the rest; \
            `only_open: false` lists them with their reasons. `values: true` lists a parameterised feat once per \
            value. `query` filters by name or key. `compact: true` leaves out the summaries and other effects. It \
            changes nothing, so \
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
            none; a gear place an item's key, or `none`). Add gear only when the user asks for it. Each step takes \
            only an option the planner offers and finds open, by the planner's own edit, and \
            the result is checked again, so the Build stays possible. A point-buy score (`start.score.int`) or a \
            skill's ranks at a level (`level.N.skill.<skill>`) take the number to reach, reached one step at a time, \
            each checked; the current value is a no-op. `level.N.skill` takes a map {skill: ranks} (`{\"athletics\": \
            1, \"perception\": 1}`), each skill's ranks at that level reached the same way, all in one step. The first \
            step refused stops the rest (a refused rank refuses its whole step): `applied` counts the steps applied, \
            and `rejected` {step, choice, issue, reason} gives the engine's issue (`unmet`, `already_held`, \
            `breaks_later`, …) or the server's (`not_an_option`, with the closest values; `needs_parameter`, a \
            parameterised feat given by its key alone, with what it takes and its open values; `no_such_choice`). \
            Returns the envelope (`build`, `link`, `todo`) of the Build as the applied steps left \
            it; `moved` {step, choice, item, name, from} for a ring a step took from the other ring place (that \
            place is then empty); and `leans_on_gear` {step, choice, leaning} for each applied step that left \
            choices allowed only because of the Build's gear (a Dexterity lowered under a belt, with a feat that \
            needs it), each as `sheet`'s `gear_dependent` lists it: the step is applied all the same. A code that \
            doesn't open is `stale_link`.",
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
            the first ascension, path or Path switch unchosen, or a gear item taken off (`gear.belt`, \
            `gear.usable.<item key>`, …). Only a choice nothing later depends on comes off (an item a pick needs is \
            refused naming the pick: withdraw the pick first, or `choose` an item that keeps it): \
            otherwise `not_removable` (`sheet` with `removable: true` lists what can come off now, so chains come \
            off from the leaves inward). A level's class isn't withdrawn (`choose` another class, or withdraw the \
            last level), nor the racial ability (change the race, or choose another ability). Errors: \
            `not_removable`, `no_such_choice` (nothing held there), `stale_link`. Returns the envelope (`build`, \
            `link`, `todo`), and `leans_on_gear`, the choices the withdrawal left allowed only because of the \
            Build's gear, as `sheet`'s `gear_dependent` lists them (the withdrawal is made all the same).",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn withdraw(
        &self,
        Parameters(args): Parameters<WithdrawArgs>,
    ) -> Result<Json<WithdrawResult>, String> {
        self.blocking(move |planner| Ok(Json(withdraw(planner, args)?)))
            .await
    }

    #[tool(
        description = "Compare 1 to 4 Builds (`builds`: their codes, for example two branches of one Build), from \
            the planner's sheets. For each Build, a summary at its last level: classes and levels, BAB, saves, \
            scores, caster level (class levels only), spell-level access, Mythic rank, first ascension, path, Path \
            switch, level cap, link, its attack line (`attack`, naming its weapon: `weapon`, a weapon key, when \
            given, else the weapon of that Build's earliest Weapon Focus (by level; within a level, the first \
            weapon key), and none without one), and with `caster` \
            ({school, subschool?, energy?, metamagic?}) its caster line, and the gear it holds (`gear`) and the \
            choices only its gear allows (`gear_dependent`). Its numbers (scores, lines, `by`'s number) are naked, \
            each Build's with no gear at all, unless `with_gear: true`; `numbers` in the result says which, and a \
            Build with gear also gets the other ones its gear changes (`with_gear`, or `naked`). It never adds gear. \
            Then only what differs: the Build start's choices; the gear (`gear`, by address); \
            by character level, the \
            numbers (class level, BAB, saves, scores), the choices by address and the feats and features each Build \
            gains that another doesn't; by Mythic rank, the status, path and choices; and `path_features`, by rank, \
            the path features each Build has that another lacks. Every per-Build list follows the order of \
            `builds`. Facts only: it doesn't say which Build is better. `by` (`{\"caster_line\": {school, \
            subschool?, energy?, metamagic?, number?}}`, `number` one of `dc` (the default: the DC of the highest \
            spell level castable), `spell_penetration`, `concentration`; or `{\"attack_line\": {weapon?, number?}}`, \
            `weapon` left out for the line shown (`weapon`'s when given, else each Build's own Weapon Focus \
            weapon), `number` `attack` (the default: the first \
            attack's bonus) or `damage` (the bonus per hit)) orders the \
            Builds by that number at their last level, highest first, and gives each summary its `number` and its \
            place in `builds` (`given`), and `by` in the result says what the number leaves out: a ranking by a \
            number, still a fact (by the naked number unless `with_gear`). One code gives its summary alone, with no differences. Errors: `invalid_arguments` (not 1 to 4 codes), `stale_link` (naming the code that \
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

    #[tool(
        description = "What each option of one choice does to a number: every option `options` finds open at \
            `choice` (an address: `slot.mythic.4`, `slot.feat.9.general.0`, `level.12.increase`, `level.N.class`, \
            …; a parameterised feat once per value) is taken by the planner's own edit, and the number `by` names \
            is read at the resulting Build's last level: `{\"caster_line\": {school, subschool?, energy?, \
            metamagic?, number?}}` (`number`: `dc`, the default, the DC of the highest spell level castable; \
            `spell_penetration`; `concentration`) or `{\"attack_line\": {weapon?, number?}}` (`number`: `attack`, \
            the default, the first attack's bonus; `damage`, the bonus per hit; `weapon` left out: the Build's \
            Weapon Focus weapon, which every option must keep: refused with `invalid_arguments` when the Build has \
            none or an option would change it, such as on the slot holding that Weapon Focus or an earlier one \
            open to another; give `weapon` there). Returns the Build's number now \
            (`current`), `options`: those that change it, highest number first, each with its `value` (what \
            `choose` takes), `number` and `change`; `no_change`: those that leave it as it is (the one held now \
            among them; conditional effects such as rolling twice stay in their summaries); `no_line`: those whose \
            Build has no such line; `closed`, how many can't be taken now; and `leaves_out`, what the number \
            doesn't count. The number is naked (the Build's with no gear at all, so an option open only because of \
            gear changes nothing) unless `with_gear: true` counts the gear the Build holds; `numbers` says which. It \
            never proposes gear: a gear address is `invalid_arguments`. `query` filters the options by name or key. \
            The order is by the number, a fact; it changes nothing. Errors: `no_such_choice`, \
            `invalid_arguments`, `stale_link`.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn improve(
        &self,
        Parameters(args): Parameters<ImproveArgs>,
    ) -> Result<Json<ImproveResult>, String> {
        self.blocking(move |planner| Ok(Json(improve(planner, args)?)))
            .await
    }

    #[tool(
        description = "Check a whole planned Build at once: `choices` maps each choice's address to its value, as \
            `choose` takes them (`{\"start.race\": \"human\", \"start.faith\": \"atheism\", \"start.alignment\": \
            \"true-neutral\", \"start.score.str\": 16, \"level.1.class\": {\"class\": \"fighter\"}, \
            \"level.1.skill\": {\"athletics\": 1}, \"slot.feat.1.class.0\": \"power-attack\", …}`), in any order. \
            Every level from 1 is declared by its class (`level.N.class`); a class's archetype and `advances` come \
            with it, the same wherever given (or the archetype at `class.<class>.archetype`); a skill's ranks at a \
            level go in `level.N.skill`'s map or at `level.N.skill.<skill>`; gear the user asked for goes at its \
            place (`\"gear.belt\": \"<item key>\"`, `\"gear.usable.<item key>\": \"<item key>\"`); choices left out \
            stay open. The planner \
            walks the declared Build once and returns `status: \"valid\"` with the envelope (`build`, `link`, `todo`: \
            the same code `choose` would make), or `status: \"invalid\"` with `count` and `failures`: every failed \
            choice, each {choice, label, issue, reason, needs, also_needs, children}, placed under the earliest \
            failure it needs (a refused Power Attack holds the Cleave that needs it; a refused class holds the picks \
            in slots its level doesn't give, `not_reached`). `issue` is the engine's (`unmet`, `already_held`, \
            `not_allowed` (not taken there), `unknown_key` (not in the game data, with the closest keys), \
            `not_reached`, `class_level_cap`, `not_from_path`, `deity_alignment`, `needs_faith`, …) or the \
            declaration's: `no_such_choice`, `not_an_option`, `needs_parameter`, `declared_twice` (one thing \
            declared two ways with different values), `fixed_rule` (a point-buy score outside 7 to 18 or over the budget, an ability \
            increase off every 4th level, a skill's ranks above the character level), `missing_level`. `needs` lists \
            an `unmet` failure's prerequisites in short; `also_needs` names other failed choices it needs too. An \
            invalid Build gets no code. It changes nothing.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn validate(
        &self,
        Parameters(args): Parameters<ValidateArgs>,
    ) -> Result<Json<ValidateResult>, String> {
        self.blocking(move |planner| Ok(Json(validate(planner, args)?)))
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
        Ok(ListResourcesResult::with_all_items(vec![
            glossary(),
            guide(),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let text = match request.uri.as_str() {
            GLOSSARY_URI => GLOSSARY,
            BUILD_GUIDE_URI => build_guide(),
            uri => {
                return Err(ErrorData::resource_not_found(
                    format!(
                        "No resource `{uri}`: the resources are `{GLOSSARY_URI}` and `{BUILD_GUIDE_URI}`."
                    ),
                    None,
                ));
            }
        };
        let text = ResourceContents::text(text, request.uri).with_mime_type("text/markdown");
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

/// The build guide resource, as `resources/list` lists it.
fn guide() -> Resource {
    Resource::new(BUILD_GUIDE_URI, "build-guide")
        .with_title("WotR Build Planner build guide")
        .with_description(
            "How to plan a Build with these tools: what to know before you plan, the workflow, addresses and values, \
             `validate`, branching, ranking by a number, and the honesty boundary in full. Read it before your \
             first Build.",
        )
        .with_mime_type("text/markdown")
        .with_size(build_guide().len() as u64)
}
