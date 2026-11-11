//! Ticket 150's agreement check: every picker against the walk. Tests only (the `testing` feature), and written
//! against the engine's public interface alone, so it judges the pickers from outside, as the sheet uses them.
//!
//! An option's **verdict** is the walk's: whether the Build with the option's edit applied ([`Engine::apply`])
//! [is possible](Engine::is_possible). A picker agrees when it marks open exactly the options whose verdict is
//! possible, and marks [`Issue::BreaksLater`] only on edits [`Engine::allows`] refuses.
//!
//! The verdict isn't `allows` alone: `allows` excuses the edit's own choices ([`Edit::choices`]: "the choices it sets
//! are the pickers' business"), so it accepts a feat whose prerequisites aren't met. For a possible Build, the edited
//! Build is possible exactly when `allows` accepts the edit and the edit's own choices hold.
//!
//! The pickers judged, for a possible Build, at every slot and level it has: the Build start's (race, faith,
//! alignment, background, racial ability, the point buy's steps), each level's class (and one past the last, while
//! the level cap allows another) with a prestige class's advancement, ability increase and skill steps, every slot of
//! the sheet's levels and reached Mythic ranks, and the first ascension, path and switch at the ranks that choose
//! them, once the Build reaches them (as the MCP server offers them).

use std::fmt;

use crate::{
    Build, Edit, Engine, FIRST_ASCENSION_RANK, Issue, PATH_CHOICE_RANK, PATH_SWITCH_RANK,
    PathOption, RankStatus,
};

/// One option on which a picker and the walk disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Disagreement {
    /// The picker and where: `options Feat { level: 3, … }`, `class_options 4`, `race_options`.
    pub picker: String,
    /// The option: its value or pick.
    pub option: String,
    /// What the picker said.
    pub status: Result<(), Issue>,
    /// The walk's verdict: whether the Build with the edit is possible.
    pub possible: bool,
    /// [`Engine::allows`] of the edit.
    pub allows: bool,
}

impl fmt::Display for Disagreement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let picker = match &self.status {
            Ok(()) => "Ok".to_owned(),
            Err(issue) => format!("Err({issue:?})"),
        };
        let walk = if self.possible { "keeps" } else { "refuses" };
        write!(
            f,
            "{} {}: picker {picker}, walk {walk}, allows {}",
            self.picker, self.option, self.allows
        )
    }
}

/// What the check found on one Build: how many options it judged, and the disagreements among them.
#[derive(Clone, Debug, Default)]
pub struct Agreement {
    pub judged: usize,
    pub disagreements: Vec<Disagreement>,
}

impl Agreement {
    fn add(&mut self, disagreement: Option<Disagreement>) {
        self.judged += 1;
        self.disagreements.extend(disagreement);
    }
}

/// Judges one option: a disagreement when the picker's `status` is open but the walk refuses the edit, closed but
/// the walk keeps it, or [`Issue::BreaksLater`] on an edit `allows` accepts. `build` must be possible.
pub fn judge_one(
    engine: &Engine,
    build: &Build,
    picker: &str,
    option: &str,
    status: &Result<(), Issue>,
    edit: &Edit,
) -> Option<Disagreement> {
    let mut edited = build.clone();
    engine.apply(&mut edited, edit);
    let possible = engine.is_possible(&edited);
    verdict(picker, option, status, possible, || {
        engine.allows(build, edit)
    })
}

/// A disagreement between a picker's `status` and the walk's verdict, `possible`; `allows` is asked only when the
/// two disagree or the status is [`Issue::BreaksLater`].
fn verdict(
    picker: &str,
    option: &str,
    status: &Result<(), Issue>,
    possible: bool,
    allows: impl FnOnce() -> bool,
) -> Option<Disagreement> {
    let breaks_later = status == &Err(Issue::BreaksLater);
    if status.is_ok() == possible && !breaks_later {
        return None;
    }
    let allows = allows();
    (status.is_ok() != possible || (breaks_later && allows)).then(|| Disagreement {
        picker: picker.to_owned(),
        option: option.to_owned(),
        status: status.clone(),
        possible,
        allows,
    })
}

/// Every picker of a possible Build against the walk (see the module docs).
///
/// # Panics
///
/// When the Build isn't possible: the pickers answer only for possible Builds.
pub fn judge(engine: &Engine, build: &Build) -> Agreement {
    assert!(
        engine.is_possible(build),
        "the pickers answer for possible Builds"
    );
    let mut agreement = Agreement::default();
    let check = |agreement: &mut Agreement,
                 picker: &str,
                 option: String,
                 status: &Result<(), Issue>,
                 edit: &Edit| {
        agreement.add(judge_one(engine, build, picker, &option, status, edit));
    };

    // The Build start.
    for o in engine.race_options(build) {
        let option = format!("{:?}", o.value);
        check(&mut agreement, "race_options", option, &o.status, &o.edit);
    }
    for o in engine.faith_options(build) {
        let option = format!("{:?}", o.value);
        check(&mut agreement, "faith_options", option, &o.status, &o.edit);
    }
    for o in engine.alignment_options(build) {
        let option = format!("{:?}", o.value);
        check(
            &mut agreement,
            "alignment_options",
            option,
            &o.status,
            &o.edit,
        );
    }
    for o in engine.background_options(build) {
        let option = o.value.to_string();
        check(
            &mut agreement,
            "background_options",
            option,
            &o.status,
            &o.edit,
        );
    }
    for o in engine.racial_ability_options(build) {
        let option = format!("{:?}", o.value);
        check(
            &mut agreement,
            "racial_ability_options",
            option,
            &o.status,
            &o.edit,
        );
    }
    for stepper in engine.point_buy_steps(build) {
        for (way, step) in [("down", stepper.down), ("up", stepper.up)] {
            if let Some(step) = step {
                let option = format!("{:?} {way}", stepper.value);
                check(
                    &mut agreement,
                    "point_buy_steps",
                    option,
                    &step.status,
                    &step.edit,
                );
            }
        }
    }

    // Each level, and the class one past the last while the cap allows another.
    let sheet = engine.evaluate(build);
    let levels = u8::try_from(build.levels.len()).expect("at most 255 levels");
    let past_end = (levels < sheet.level_cap).then_some(levels + 1);
    for level in (1..=levels).chain(past_end) {
        let picker = format!("class_options {level}");
        for o in engine.class_options(build, level).unwrap_or_default() {
            let option = match &o.archetype {
                Some(archetype) => format!("{} ({archetype})", o.class),
                None => o.class.to_string(),
            };
            if !engine.asks_advances(build, level, &o.class) {
                check(&mut agreement, &picker, option, &o.status, &o.edit);
                continue;
            }
            // A prestige class that asks what it advances is taken through its advancement picker, never with the
            // option's own edit (the sheet and the MCP server ask `advances_options` once it's chosen): the walk
            // keeps it when it keeps some advancement, and each advancement offered must be one the walk keeps.
            let answers = engine.advances_options(build, level, &o);
            agreement.add(verdict(
                &picker,
                &format!("{option} (asks advances)"),
                &o.status,
                !answers.is_empty(),
                || engine.allows(build, &o.edit),
            ));
            let picker = format!("advances_options {level}");
            for a in answers {
                let advances = format!("{option} advancing {:?}", a.advances);
                check(&mut agreement, &picker, advances, &Ok(()), &a.edit);
            }
        }
        let picker = format!("ability_increase_options {level}");
        let increases = engine.ability_increase_options(build, level);
        for o in increases.unwrap_or_default() {
            let option = format!("{:?}", o.value);
            check(&mut agreement, &picker, option, &o.status, &o.edit);
        }
        let picker = format!("skill_steps {level}");
        for stepper in engine.skill_steps(build, level).unwrap_or_default() {
            for (way, step) in [("down", stepper.down), ("up", stepper.up)] {
                if let Some(step) = step {
                    let option = format!("{} {way}", stepper.value);
                    check(&mut agreement, &picker, option, &step.status, &step.edit);
                }
            }
        }
    }

    // Every slot of the sheet's levels and reached ranks.
    for slot in sheet
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
    {
        let picker = format!("options {:?}", slot.id);
        for o in engine.options(build, &slot.id).unwrap_or_default() {
            let option = format!("{:?}", o.pick);
            check(&mut agreement, &picker, option, &o.status, &o.edit);
        }
    }

    // The Mythic choices, at ranks the Build reaches.
    let reached = |rank: u8| {
        sheet
            .ranks
            .iter()
            .any(|r| r.rank == rank && r.status != RankStatus::NotReached)
    };
    type Picker = fn(&Engine, &Build) -> Vec<PathOption>;
    let mythic: [(u8, &str, Picker); 3] = [
        (
            FIRST_ASCENSION_RANK,
            "first_ascension_options",
            Engine::first_ascension_options,
        ),
        (PATH_CHOICE_RANK, "path_options", Engine::path_options),
        (PATH_SWITCH_RANK, "switch_options", Engine::switch_options),
    ];
    for (rank, picker, options) in mythic {
        if reached(rank) {
            for o in options(engine, build) {
                let option = o.path.to_string();
                check(&mut agreement, picker, option, &o.status, &o.edit);
            }
        }
    }
    agreement
}
