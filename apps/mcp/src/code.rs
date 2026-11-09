//! Build codes, the handle every Build-taking tool takes: opening one only through the engine's checked decode, and
//! the envelope every such tool returns (the code, its link and the open choices).

use rules::{
    Build, Engine, FIRST_ASCENSION_RANK, PATH_CHOICE_RANK, RankStatus, Sheet, Slot, SlotId,
    encode_build, has_ability_increase,
};

use crate::address::Address;
use crate::error::ToolError;
use crate::planner::Planner;
use crate::shapes::{Envelope, Todo};

/// Where the planner opens a Build: the link is this and the code.
pub(crate) const LINK_BASE: &str = "https://viniciusataide.github.io/wotr-planner/#b=";

/// The code in a planner link (`…#b=<code>`), or a bare code.
pub(crate) fn code_of(text: &str) -> &str {
    let text = text.trim();
    match text.rfind("#b=") {
        Some(at) => &text[at + 3..],
        None => text.strip_prefix("b=").unwrap_or(text),
    }
}

/// The Build a code (or link) holds, through [`Engine::open_link`]: decoded with this game data and possible, or
/// `stale_link`.
pub(crate) fn open(planner: &Planner, code: &str) -> Result<Build, ToolError> {
    planner
        .engine
        .open_link(code_of(code))
        .map_err(|_| ToolError::stale_link())
}

/// The final gate on a Build an edit made: its code opened again through [`Engine::open_link`], so what comes back
/// is exactly what the code holds and the engine finds it possible. `None` when it doesn't open.
pub(crate) fn reopen(engine: &Engine, build: &Build) -> Option<Build> {
    engine.open_link(&encode_build(build)).ok()
}

/// The envelope for a Build, from its sheet.
pub(crate) fn envelope(engine: &Engine, build: &Build, sheet: &Sheet) -> Envelope {
    let code = encode_build(build);
    Envelope {
        link: format!("{LINK_BASE}{code}"),
        build: code,
        todo: todo(engine, build, sheet),
    }
}

/// The open choices, in Build order: the Build start's unchosen choices; per level its unchosen ability increase,
/// unspent skill points (while some skill can still take a rank), empty slots and picks whose value is still open
/// (a link made before Stat Focus took its stat); then the first ascension (while neither it nor a path is chosen)
/// and the path once their ranks are reached, and each reached rank's empty slots and picks whose value is still
/// open (Favorite Metamagic's metamagic), not those a Path switch dropped.
fn todo(engine: &Engine, build: &Build, sheet: &Sheet) -> Vec<Todo> {
    let mut todo = Vec::new();
    let mut open = |address: Address, label: String| {
        todo.push(Todo {
            choice: address.to_string(),
            label,
        })
    };
    let start = &build.start;
    if start.race.is_none() {
        open(Address::Race, "Race".to_owned());
    }
    if engine.chooses_racial_ability(start) && start.racial_ability.is_none() {
        open(Address::RacialAbility, "Racial ability bonus".to_owned());
    }
    if start.deity.is_none() {
        open(Address::Faith, "Faith".to_owned());
    }
    if start.alignment.is_none() {
        open(Address::Alignment, "Alignment".to_owned());
    }
    if start.background.is_none() {
        open(Address::Background, "Background".to_owned());
    }
    for (level_up, level) in build.levels.iter().zip(&sheet.levels) {
        let n = level.character_level;
        if has_ability_increase(n) && level_up.ability_increase.is_none() {
            open(Address::Increase(n), format!("Level {n}: ability increase"));
        }
        let points = level.skill_points;
        let room = engine
            .skills()
            .iter()
            .any(|skill| level.state.skill_ranks(&skill.key) < n);
        if points.spent < points.available && room {
            open(
                Address::SkillPoints(n),
                format!(
                    "Level {n}: {} of {} skill points unspent",
                    points.available - points.spent,
                    points.available
                ),
            );
        }
        for slot in &level.slots {
            if let Some(what) = open_in(engine, slot) {
                open(Address::Slot(slot.id.clone()), format!("Level {n}: {what}"));
            }
        }
    }
    for rank in &sheet.ranks {
        if rank.status == RankStatus::NotReached {
            continue;
        }
        let r = rank.rank;
        if r == FIRST_ASCENSION_RANK
            && build.first_ascension.is_none()
            && build.mythic_path.is_none()
        {
            open(Address::FirstAscension, format!("MR {r}: first ascension"));
        }
        if r == PATH_CHOICE_RANK && build.mythic_path.is_none() {
            open(Address::Path, format!("MR {r}: Mythic path"));
        }
        let dropped = |id: &SlotId| rank.dropped_slots.contains(id);
        for slot in rank.slots.iter().filter(|slot| !dropped(&slot.id)) {
            if let Some(what) = open_in(engine, slot) {
                open(Address::Slot(slot.id.clone()), format!("MR {r}: {what}"));
            }
        }
    }
    todo
}

/// What's open in a slot: the slot itself while empty ("General feat"), or its pick's value while the pick names
/// none ("Favorite Metamagic: choose a metamagic", which `choose` fills at the slot's address with
/// `{key, parameter}`). `None` when nothing is.
fn open_in(engine: &Engine, slot: &Slot) -> Option<String> {
    match &slot.pick {
        None => Some(engine.slot_label(slot)),
        Some(pick) => engine
            .value_left_open(pick)
            .map(|kind| format!("{}: choose {}", engine.pick_name(pick), kind.with_article())),
    }
}
