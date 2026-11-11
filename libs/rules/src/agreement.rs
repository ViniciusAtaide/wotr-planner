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
    Build, ChoiceOption, Edit, Engine, FIRST_ASCENSION_RANK, Issue, PATH_CHOICE_RANK,
    PATH_SWITCH_RANK, PathOption, RankStatus, Stepper,
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

impl Agreement {
    /// Its disagreements as the tests pin them, each after its Build's name: `name: disagreement`.
    pub fn lines(&self, name: &str) -> Vec<String> {
        self.disagreements
            .iter()
            .map(|disagreement| format!("{name}: {disagreement}"))
            .collect()
    }
}

/// Judges every Build of a corpus on every core, each thread taking the next Build as it finishes one: how many
/// options, and the disagreements' [`lines`](Agreement::lines), sorted.
///
/// # Panics
///
/// When a Build isn't possible, or one has no option to judge.
pub fn judge_all(engine: &Engine, builds: &[(String, Build)]) -> (usize, Vec<String>) {
    let threads = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: Vec<(usize, Vec<String>)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut judged = 0;
                    let mut found = Vec::new();
                    while let Some((name, build)) =
                        builds.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                    {
                        let agreement = judge(engine, build);
                        assert!(agreement.judged > 0, "{name}");
                        judged += agreement.judged;
                        found.extend(agreement.lines(name));
                    }
                    (judged, found)
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("a judging thread"))
            .collect()
    });
    let judged = results.iter().map(|(judged, _)| judged).sum();
    let mut found: Vec<String> = results.into_iter().flat_map(|(_, found)| found).collect();
    found.sort();
    (judged, found)
}

/// The lines of `known` about the Builds of `builds`: what a part of a corpus finds of its pinned disagreements.
pub fn known_for<'a>(known: &[&'a str], builds: &[(String, Build)]) -> Vec<&'a str> {
    known
        .iter()
        .copied()
        .filter(|line| {
            builds
                .iter()
                .any(|(name, _)| line.starts_with(&format!("{name}: ")))
        })
        .collect()
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
    let mut judge = Judge {
        engine,
        build,
        agreement: Agreement::default(),
    };

    // The Build start.
    judge.choices("race_options", engine.race_options(build), |v| {
        format!("{v:?}")
    });
    judge.choices("faith_options", engine.faith_options(build), |v| {
        format!("{v:?}")
    });
    judge.choices("alignment_options", engine.alignment_options(build), |v| {
        format!("{v:?}")
    });
    judge.choices(
        "background_options",
        engine.background_options(build),
        ToString::to_string,
    );
    judge.choices(
        "racial_ability_options",
        engine.racial_ability_options(build),
        |v| format!("{v:?}"),
    );
    judge.steps("point_buy_steps", engine.point_buy_steps(build), |v| {
        format!("{v:?}")
    });

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
                judge.check(&picker, &option, &o.status, &o.edit);
                continue;
            }
            // A prestige class that asks what it advances is taken through its advancement picker, never with the
            // option's own edit (the sheet and the MCP server ask `advances_options` once it's chosen): the walk
            // keeps it when it keeps some advancement, and each advancement offered must be one the walk keeps.
            let answers = engine.advances_options(build, level, &o);
            judge.agreement.add(verdict(
                &picker,
                &format!("{option} (asks advances)"),
                &o.status,
                !answers.is_empty(),
                || engine.allows(build, &o.edit),
            ));
            let picker = format!("advances_options {level}");
            for a in answers {
                let advances = format!("{option} advancing {:?}", a.advances);
                judge.check(&picker, &advances, &Ok(()), &a.edit);
            }
        }
        judge.choices(
            &format!("ability_increase_options {level}"),
            engine
                .ability_increase_options(build, level)
                .unwrap_or_default(),
            |v| format!("{v:?}"),
        );
        judge.steps(
            &format!("skill_steps {level}"),
            engine.skill_steps(build, level).unwrap_or_default(),
            ToString::to_string,
        );
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
            judge.check(&picker, &format!("{:?}", o.pick), &o.status, &o.edit);
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
                judge.check(picker, &o.path.to_string(), &o.status, &o.edit);
            }
        }
    }
    judge.agreement
}

/// The check of one Build under way: the Build, and what it has found so far.
struct Judge<'a> {
    engine: &'a Engine,
    build: &'a Build,
    agreement: Agreement,
}

impl Judge<'_> {
    /// Judges one option ([`judge_one`]).
    fn check(&mut self, picker: &str, option: &str, status: &Result<(), Issue>, edit: &Edit) {
        let disagreement = judge_one(self.engine, self.build, picker, option, status, edit);
        self.agreement.add(disagreement);
    }

    /// Judges a start or level picker's options, each named by its value.
    fn choices<T>(
        &mut self,
        picker: &str,
        options: Vec<ChoiceOption<T>>,
        name: impl Fn(&T) -> String,
    ) {
        for o in options {
            self.check(picker, &name(&o.value), &o.status, &o.edit);
        }
    }

    /// Judges a picker's steppers, each step named by its value and its way (`down`, `up`).
    fn steps<T>(&mut self, picker: &str, steppers: Vec<Stepper<T>>, name: impl Fn(&T) -> String) {
        for stepper in steppers {
            for (way, step) in [("down", stepper.down), ("up", stepper.up)] {
                if let Some(step) = step {
                    let option = format!("{} {way}", name(&stepper.value));
                    self.check(picker, &option, &step.status, &step.edit);
                }
            }
        }
    }
}
