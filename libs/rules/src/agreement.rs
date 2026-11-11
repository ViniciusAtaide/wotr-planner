//! Ticket 150's agreement check: every picker against the walk. Tests only (the `testing` feature), and written
//! against the engine's public interface alone, so it judges the pickers from outside, as the sheet uses them.
//!
//! An option's **verdict** is the walk's: whether the Build with the option's edit applied ([`Engine::apply`])
//! [is possible](Engine::is_possible). A picker agrees when it marks open exactly the options whose verdict is
//! possible, marks [`Issue::BreaksLater`] only on edits [`Engine::allows`] refuses, and marks open no edit that
//! leaves the Build as it is, but for the pick the Build holds there (a wrong level or slot, which `apply` drops).
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
//!
//! A prestige class that asks what it advances is judged against advancements the check lists itself, not the
//! advancement picker's: every set of at most two of the classes the Build takes (the game's rule: one arcane and one
//! divine class at most), each judged by the walk. The class is open when one of them keeps the Build, and the
//! advancement picker offers exactly those that do.
//!
//! The tests pin each known disagreement by its content: a line `build: disagreement`, where `build` is the key the
//! corpus gives its Builds (a test-world plan's name, which every Build grown from it shares, or a real Build's name),
//! and the disagreement names the picker with its level or slot, the option and the verdicts.

use std::collections::BTreeSet;
use std::fmt;

use crate::{
    Build, ChoiceOption, ClassChoice, ClassKey, ClassOption, Edit, Engine, FIRST_ASCENSION_RANK,
    Issue, MythicPathKey, PATH_CHOICE_RANK, PATH_SWITCH_RANK, PathOption, RankStatus, Stepper,
};

/// One option on which a picker and the walk disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Disagreement {
    /// The picker and where: `options Feat { level: 3, … }`, `class_options 4`, `race_options`.
    pub picker: String,
    /// The option: its value or pick.
    pub option: String,
    /// What the picker said; `None` when it doesn't offer the option (an advancement the check lists itself).
    pub status: Option<Result<(), Issue>>,
    /// The walk's verdict: whether the Build with the edit is possible.
    pub possible: bool,
    /// [`Engine::allows`] of the edit.
    pub allows: bool,
    /// An open option whose edit leaves the Build as it is, though it isn't the pick the Build holds there.
    pub unchanged: bool,
}

impl fmt::Display for Disagreement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let picker = match &self.status {
            None => "doesn't offer it".to_owned(),
            Some(Ok(())) => "Ok".to_owned(),
            Some(Err(issue)) => format!("Err({issue:?})"),
        };
        let walk = if self.possible { "keeps" } else { "refuses" };
        write!(
            f,
            "{} {}: picker {picker}, walk {walk}, allows {}",
            self.picker, self.option, self.allows
        )?;
        if self.unchanged {
            write!(f, ", the edit changes nothing")?;
        }
        Ok(())
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

    /// Its disagreements as the tests pin them, each after its Build's key: `build: disagreement`.
    pub fn lines(&self, build: &str) -> Vec<String> {
        self.disagreements
            .iter()
            .map(|disagreement| format!("{build}: {disagreement}"))
            .collect()
    }
}

/// What the check found on a corpus: how many options it judged, how many disagreements, and their
/// [`lines`](Agreement::lines), sorted, each once (the Builds of one plan share their key).
#[derive(Clone, Debug, Default)]
pub struct Sweep {
    pub judged: usize,
    pub disagreements: usize,
    pub lines: Vec<String>,
}

/// Judges every Build of a corpus (each after its key, see the module docs) on every core, each thread taking the
/// next Build as it finishes one.
///
/// # Panics
///
/// When a Build isn't possible, or one has no option to judge.
pub fn judge_all(engine: &Engine, builds: &[(String, Build)]) -> Sweep {
    let threads = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: Vec<Sweep> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut sweep = Sweep::default();
                    while let Some((key, build)) =
                        builds.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                    {
                        let agreement = judge(engine, build);
                        assert!(agreement.judged > 0, "{key}");
                        sweep.judged += agreement.judged;
                        sweep.disagreements += agreement.disagreements.len();
                        sweep.lines.extend(agreement.lines(key));
                    }
                    sweep
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("a judging thread"))
            .collect()
    });
    let mut sweep = Sweep::default();
    for result in results {
        sweep.judged += result.judged;
        sweep.disagreements += result.disagreements;
        sweep.lines.extend(result.lines);
    }
    sweep.lines.sort();
    sweep.lines.dedup();
    sweep
}

/// The lines of `known` about the Builds of `builds`, by their keys: what a part of a corpus finds of its pinned
/// disagreements.
pub fn known_for<'a>(known: &[&'a str], builds: &[(String, Build)]) -> Vec<&'a str> {
    known
        .iter()
        .copied()
        .filter(|line| {
            builds
                .iter()
                .any(|(key, _)| line.starts_with(&format!("{key}: ")))
        })
        .collect()
}

/// Judges one option: a disagreement when the picker's `status` is open but the walk refuses the edit, closed but
/// the walk keeps it, [`Issue::BreaksLater`] on an edit `allows` accepts, or open on an edit that leaves the Build as
/// it is when the option isn't the pick the Build `held` there. `build` must be possible.
pub fn judge_one(
    engine: &Engine,
    build: &Build,
    picker: &str,
    option: &str,
    status: &Result<(), Issue>,
    edit: &Edit,
    held: bool,
) -> Option<Disagreement> {
    let mut edited = build.clone();
    engine.apply(&mut edited, edit);
    // The Build is possible, so an edit that leaves it as it is keeps it.
    let unchanged = edited == *build;
    let possible = unchanged || engine.is_possible(&edited);
    verdict(
        picker,
        option,
        Some(status),
        possible,
        status.is_ok() && unchanged && !held,
        || engine.allows(build, edit),
    )
}

/// A disagreement between a picker's `status` (`None`: not offered, so closed) and the walk's verdict, `possible`,
/// or an open option whose edit leaves the Build as it is though it isn't the held pick (`unchanged`); `allows` is
/// asked only when there is one or the status is [`Issue::BreaksLater`].
fn verdict(
    picker: &str,
    option: &str,
    status: Option<&Result<(), Issue>>,
    possible: bool,
    unchanged: bool,
    allows: impl FnOnce() -> bool,
) -> Option<Disagreement> {
    let open = status.is_some_and(Result::is_ok);
    let breaks_later = status == Some(&Err(Issue::BreaksLater));
    if open == possible && !breaks_later && !unchanged {
        return None;
    }
    let allows = allows();
    (open != possible || (breaks_later && allows) || unchanged).then(|| Disagreement {
        picker: picker.to_owned(),
        option: option.to_owned(),
        status: status.cloned(),
        possible,
        allows,
        unchanged,
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
    let start = &build.start;

    // The Build start: each picker's held value is the start's.
    judge.choices(
        "race_options",
        engine.race_options(build),
        |v| format!("{v:?}"),
        |v| start.race.as_ref() == Some(&v.race) && start.subrace == v.subrace,
    );
    judge.choices(
        "faith_options",
        engine.faith_options(build),
        |v| format!("{v:?}"),
        |v| start.deity.as_ref() == Some(v),
    );
    judge.choices(
        "alignment_options",
        engine.alignment_options(build),
        |v| format!("{v:?}"),
        |v| start.alignment == Some(*v),
    );
    judge.choices(
        "background_options",
        engine.background_options(build),
        ToString::to_string,
        |v| start.background.as_ref() == Some(v),
    );
    judge.choices(
        "racial_ability_options",
        engine.racial_ability_options(build),
        |v| format!("{v:?}"),
        |v| start.racial_ability == Some(*v),
    );
    judge.steps("point_buy_steps", engine.point_buy_steps(build), |v| {
        format!("{v:?}")
    });

    // Each level, and the class one past the last while the cap allows another.
    let sheet = engine.evaluate(build);
    let levels = u8::try_from(build.levels.len()).expect("at most 255 levels");
    let past_end = (levels < sheet.level_cap).then_some(levels + 1);
    for level in (1..=levels).chain(past_end) {
        let level_up = build.levels.get(usize::from(level) - 1);
        let picker = format!("class_options {level}");
        for o in engine.class_options(build, level).unwrap_or_default() {
            let option = match &o.archetype {
                Some(archetype) => format!("{} ({archetype})", o.class),
                None => o.class.to_string(),
            };
            if engine.asks_advances(build, level, &o.class) {
                judge.advances(level, &picker, &option, &o);
            } else {
                // The class the level takes, with the archetype the Build holds for it.
                let held = level_up.is_some_and(|l| l.class == o.class)
                    && build
                        .class_choices
                        .get(&o.class)
                        .and_then(|choice| choice.archetype.as_ref())
                        == o.archetype.as_ref();
                judge.check(&picker, &option, &o.status, &o.edit, held);
            }
        }
        judge.choices(
            &format!("ability_increase_options {level}"),
            engine
                .ability_increase_options(build, level)
                .unwrap_or_default(),
            |v| format!("{v:?}"),
            |v| level_up.is_some_and(|l| l.ability_increase == Some(*v)),
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
            let held = slot.pick.as_ref() == Some(&o.pick);
            judge.check(&picker, &format!("{:?}", o.pick), &o.status, &o.edit, held);
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
    let mythic: [(u8, &str, Picker, &Option<MythicPathKey>); 3] = [
        (
            FIRST_ASCENSION_RANK,
            "first_ascension_options",
            Engine::first_ascension_options,
            &build.first_ascension,
        ),
        (
            PATH_CHOICE_RANK,
            "path_options",
            Engine::path_options,
            &build.mythic_path,
        ),
        (
            PATH_SWITCH_RANK,
            "switch_options",
            Engine::switch_options,
            &build.path_switch,
        ),
    ];
    for (rank, picker, options, held) in mythic {
        if reached(rank) {
            for o in options(engine, build) {
                let held = held.as_ref() == Some(&o.path);
                judge.check(picker, &o.path.to_string(), &o.status, &o.edit, held);
            }
        }
    }
    judge.agreement
}

/// The advancements the check judges a prestige class with: every set of at most two of the classes the Build
/// takes, none first (a prestige class advances at most one arcane and one divine class, each taken before it; the
/// walk refuses the rest).
fn advancements(build: &Build) -> Vec<BTreeSet<ClassKey>> {
    let classes: BTreeSet<&ClassKey> = build.levels.iter().map(|level| &level.class).collect();
    let mut sets = vec![BTreeSet::new()];
    for (index, &first) in classes.iter().enumerate() {
        sets.push(BTreeSet::from([first.clone()]));
        for &second in classes.iter().skip(index + 1) {
            sets.push(BTreeSet::from([first.clone(), second.clone()]));
        }
    }
    sets
}

/// The check of one Build under way: the Build, and what it has found so far.
struct Judge<'a> {
    engine: &'a Engine,
    build: &'a Build,
    agreement: Agreement,
}

impl Judge<'_> {
    /// Judges one option ([`judge_one`]).
    fn check(
        &mut self,
        picker: &str,
        option: &str,
        status: &Result<(), Issue>,
        edit: &Edit,
        held: bool,
    ) {
        let disagreement = judge_one(self.engine, self.build, picker, option, status, edit, held);
        self.agreement.add(disagreement);
    }

    /// Judges a start or level picker's options, each named by its value; `held` says which value the Build holds.
    fn choices<T>(
        &mut self,
        picker: &str,
        options: Vec<ChoiceOption<T>>,
        name: impl Fn(&T) -> String,
        held: impl Fn(&T) -> bool,
    ) {
        for o in options {
            self.check(picker, &name(&o.value), &o.status, &o.edit, held(&o.value));
        }
    }

    /// Judges a picker's steppers, each step named by its value and its way (`down`, `up`). A step always changes
    /// the Build.
    fn steps<T>(&mut self, picker: &str, steppers: Vec<Stepper<T>>, name: impl Fn(&T) -> String) {
        for stepper in steppers {
            for (way, step) in [("down", stepper.down), ("up", stepper.up)] {
                if let Some(step) = step {
                    let option = format!("{} {way}", name(&stepper.value));
                    self.check(picker, &option, &step.status, &step.edit, false);
                }
            }
        }
    }

    /// Judges a prestige class option that asks what it advances, as the sheet and the MCP server take it: never with
    /// the option's own edit, but through the advancement picker ([`Engine::advances_options`]). Each of the check's
    /// own [`advancements`] (and any other the picker offers) is judged by the walk: the picker must offer exactly
    /// those the walk keeps, and the class is open exactly when one is kept.
    fn advances(&mut self, level: u8, picker: &str, option: &str, o: &ClassOption) {
        let (engine, build) = (self.engine, self.build);
        let answers = engine.advances_options(build, level, o);
        let mut candidates = advancements(build);
        for answer in &answers {
            if !candidates.contains(&answer.advances) {
                candidates.push(answer.advances.clone());
            }
        }
        let advances_picker = format!("advances_options {level}");
        let mut kept = false;
        for advances in candidates {
            // The class is first taken here (it asks advances), so its edit carries the option's archetype.
            let edit = Edit::Class {
                level,
                class: o.class.clone(),
                choice: Some(ClassChoice {
                    archetype: o.archetype.clone(),
                    advances: advances.clone(),
                }),
            };
            let mut edited = build.clone();
            engine.apply(&mut edited, &edit);
            let possible = edited == *build || engine.is_possible(&edited);
            kept |= possible;
            let name = format!("{option} advancing {advances:?}");
            let answer = answers.iter().find(|a| a.advances == advances);
            // An answer whose edit makes another Build than the check's is judged on its own.
            if let Some(answer) = answer.filter(|a| a.edit != edit) {
                let mut answered = build.clone();
                engine.apply(&mut answered, &answer.edit);
                if answered != edited {
                    self.check(&advances_picker, &name, &Ok(()), &answer.edit, false);
                }
            }
            let status = answer.map(|_| Ok(()));
            self.agreement.add(verdict(
                &advances_picker,
                &name,
                status.as_ref(),
                possible,
                false,
                || engine.allows(build, &edit),
            ));
        }
        self.agreement.add(verdict(
            picker,
            &format!("{option} (asks advances)"),
            Some(&o.status),
            kept,
            false,
            || engine.allows(build, &o.edit),
        ));
    }
}
