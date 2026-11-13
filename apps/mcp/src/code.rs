//! Build codes, the handle every Build-taking tool takes: opening one only through the engine's checked decode, and
//! the envelope every such tool returns (the code, its link and the open choices).

use rules::{
    Address, Build, Engine, FIRST_ASCENSION_RANK, PATH_CHOICE_RANK, RankStatus, Sheet, Slot,
    SlotId, encode_build, has_ability_increase,
};

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
    // Open at an address, labelled as the engine labels it ([`Engine::placed_label`]) unless a label is given.
    let mut open = |address: Address, label: Option<String>| {
        todo.push(Todo {
            label: label.unwrap_or_else(|| engine.placed_label(&address, None)),
            choice: address.to_string(),
        })
    };
    let start = &build.start;
    if start.race.is_none() {
        open(Address::Race, None);
    }
    if engine.chooses_racial_ability(start) && start.racial_ability.is_none() {
        open(Address::RacialAbility, None);
    }
    if start.deity.is_none() {
        open(Address::Faith, None);
    }
    if start.alignment.is_none() {
        open(Address::Alignment, None);
    }
    if start.background.is_none() {
        open(Address::Background, None);
    }
    for (level_up, level) in build.levels.iter().zip(&sheet.levels) {
        let n = level.character_level;
        if has_ability_increase(n) && level_up.ability_increase.is_none() {
            open(Address::Increase(n), None);
        }
        let points = level.skill_points;
        let room = engine
            .skills()
            .iter()
            .any(|skill| level.state.skill_ranks(&skill.key) < n);
        if points.spent < points.available && room {
            let address = Address::SkillPoints(n);
            let label = address.placed(&format!(
                "{} of {} skill point{} unspent",
                points.available - points.spent,
                points.available,
                if points.available == 1 { "" } else { "s" },
            ));
            open(address, Some(label));
        }
        for slot in &level.slots {
            open_slot(engine, slot, &mut open);
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
            open(Address::FirstAscension, None);
        }
        if r == PATH_CHOICE_RANK && build.mythic_path.is_none() {
            open(Address::Path, None);
        }
        let dropped = |id: &SlotId| rank.dropped_slots.contains(id);
        for slot in rank.slots.iter().filter(|slot| !dropped(&slot.id)) {
            open_slot(engine, slot, &mut open);
        }
    }
    todo
}

/// What's open in a slot, if anything: the slot itself while empty ("Level 1: General feat"), or its pick's value
/// while the pick names none ("MR 3: Favorite Metamagic: choose a metamagic", which `choose` fills at the slot's
/// address with `{key, parameter}`).
fn open_slot(engine: &Engine, slot: &Slot, open: &mut impl FnMut(Address, Option<String>)) {
    let address = Address::Slot(slot.id.clone());
    match &slot.pick {
        None => {
            let label = engine.placed_label(&address, Some(slot));
            open(address, Some(label));
        }
        Some(pick) => {
            if let Some(kind) = engine.value_left_open(pick) {
                let what = format!("{}: choose {}", engine.pick_name(pick), kind.with_article());
                let label = address.placed(&what);
                open(address, Some(label));
            }
        }
    }
}
