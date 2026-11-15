//! The engine's self-checks on the corpus (ticket 235: the checks the behaviour proof's record made, kept when the
//! proof went). Each answers one question two ways, a shortcut and the full answer it stands for, and asserts they
//! agree:
//!
//! - **Withdrawals:** for each choice the Build holds ([`Engine::held_choices`]), [`Engine::is_removable`] and
//!   [`Engine::withdrawal`] agree with [`Engine::removable`]; for some choices it doesn't hold, both say so;
//!   `removable` names only held choices; and [`Engine::removable_classes`] is `removable`'s class levels.
//! - **Statuses against `allows`:** each option and step of the Build start's and the levels' pickers ([`controls`])
//!   is open exactly when [`Engine::allows`] keeps the Build with its edit (ticket 96: the pickers share one walk where
//!   `allows` walks each), unless a faith and an alignment that don't fit lock it.
//! - **Ticket 48's picker shortcut,** on the Builds [`SHORTCUT_CHECKED`] names: every feat a feat slot accepts is
//!   open exactly when `allows` keeps the Build with it picked there.
//!
//! They check the engine against itself, never against the game: when one disagrees with the game, the engine is
//! fixed (`CLAUDE.md`, "Behaviour checks"). The suite runs them on [`corpus::builds::sample`]; the whole corpus is
//! ignored for its time. Run it in release:
//!
//! ```text
//! cargo test --release -p corpus --test self_checks -- --ignored --nocapture
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use corpus::Named;
use rules::{Build, Choice, ChoiceOption, Edit, Engine, GameData, Issue, Pick, SlotKind, Stepper};

/// The name prefixes of the Builds whose feat pickers are checked against full walks (ticket 48's shortcut), each also
/// with `full ` before it: the Builds added from ticket 55 on (the checks are slow, so not on every Build).
const SHORTCUT_CHECKED: [&str; 13] = [
    "t55 ", "t57 ", "t58 ", "t59 ", "t60 ", "t61 ", "t62 ", "t63 ", "t64 ", "t65 ", "t66 ", "t67 ",
    "t86 ",
];

/// What the checks covered, for the log and the sample's own assertions.
#[derive(Debug, Default)]
struct Covered {
    /// The held choices checked, by their kind (`Deity`, `Slot`…).
    held: BTreeMap<String, usize>,
    /// The start and level options and steps checked against `allows`.
    controls: usize,
    /// The Builds and feat options ticket 48's shortcut was checked on.
    shortcut_builds: usize,
    shortcut_options: usize,
}

impl Covered {
    fn add(&mut self, other: Covered) {
        for (kind, count) in other.held {
            *self.held.entry(kind).or_default() += count;
        }
        self.controls += other.controls;
        self.shortcut_builds += other.shortcut_builds;
        self.shortcut_options += other.shortcut_options;
    }
}

fn engine() -> (Engine, GameData) {
    corpus::engine().expect("the game data reads and passes `check`")
}

/// Runs every check on each Build, on `threads` threads, and says what they covered. Panics on a disagreement.
fn check_all(engine: &Engine, builds: &[Named], threads: usize) -> Covered {
    let mut covered = Covered::default();
    for one in corpus::in_parallel(builds, threads, |(name, build)| check(engine, name, build)) {
        covered.add(one);
    }
    covered
}

/// The checks on one Build (see the module docs).
fn check(engine: &Engine, name: &str, build: &Build) -> Covered {
    let mut covered = Covered::default();
    withdrawals(engine, name, build, &mut covered);
    statuses(engine, name, build, &mut covered);
    let unfull = name.strip_prefix("full ").unwrap_or(name);
    if SHORTCUT_CHECKED
        .iter()
        .any(|prefix| unfull.starts_with(prefix))
    {
        shortcut(engine, name, build, &mut covered);
    }
    covered
}

/// `is_removable`, `withdrawal` and `removable_classes` against `removable`.
fn withdrawals(engine: &Engine, name: &str, build: &Build, covered: &mut Covered) {
    let removable = engine.removable(build);
    let held = engine.held_choices(build);
    for withdrawal in &held {
        let choice = &withdrawal.choice;
        let removable = removable.contains(choice);
        assert_eq!(
            engine.is_removable(build, choice),
            removable,
            "{name}: {choice:?}"
        );
        assert_eq!(
            engine.withdrawal(build, choice),
            Some((withdrawal.edit.clone(), removable)),
            "{name}: {choice:?}"
        );
        let debug = format!("{choice:?}");
        let kind: String = debug
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect();
        *covered.held.entry(kind).or_default() += 1;
    }
    for choice in unheld_choices(build)
        .iter()
        .filter(|choice| !held.iter().any(|held| &held.choice == *choice))
    {
        assert!(!engine.is_removable(build, choice), "{name}: {choice:?}");
        assert_eq!(engine.withdrawal(build, choice), None, "{name}: {choice:?}");
    }
    assert!(
        removable
            .iter()
            .all(|choice| held.iter().any(|held| &held.choice == choice)),
        "{name}: a removable choice the Build doesn't hold"
    );
    let levels: BTreeSet<u8> = removable
        .iter()
        .filter_map(|choice| match choice {
            Choice::Class { level } => Some(*level),
            _ => None,
        })
        .collect();
    assert_eq!(engine.removable_classes(build), levels, "{name}");
}

/// Some choices a Build may not hold, for `is_removable` and `withdrawal` beside the held ones: every Build start
/// choice and Mythic one, each level's ability increase, skill points, archetype and advanced classes, and the class
/// and the character level one past the end.
fn unheld_choices(build: &Build) -> Vec<Choice> {
    let mut choices = vec![
        Choice::Race,
        Choice::Subrace,
        Choice::Alignment,
        Choice::Background,
        Choice::Deity,
        Choice::FirstAscension,
        Choice::MythicPath,
        Choice::PathSwitch,
    ];
    for (index, level_up) in build.levels.iter().enumerate() {
        let level = u8::try_from(index + 1).expect("at most 255 levels");
        choices.push(Choice::AbilityIncrease { level });
        choices.push(Choice::SkillPoints { level });
        choices.push(Choice::CharacterLevel { level });
        choices.push(Choice::Archetype {
            class: level_up.class.clone(),
        });
        choices.push(Choice::Advances {
            class: level_up.class.clone(),
        });
    }
    let past = u8::try_from(build.levels.len() + 1).expect("at most 254 levels");
    choices.push(Choice::Class { level: past });
    choices.push(Choice::CharacterLevel { level: past });
    choices
}

/// Every option and step of the Build start's and the levels' pickers and steppers, in order, each with its status
/// and edit: race and subrace, faith, alignment, background, the racial ability, the point buy's steps, then each
/// level's ability increase and skill steps (ticket 96: the engine's, so a new start choice is covered here without
/// editing the check).
fn controls(engine: &Engine, build: &Build) -> Vec<(Result<(), Issue>, Edit)> {
    fn options<T>(
        options: Vec<ChoiceOption<T>>,
    ) -> impl Iterator<Item = (Result<(), Issue>, Edit)> {
        options
            .into_iter()
            .map(|option| (option.status, option.edit))
    }
    fn steps<T>(steppers: Vec<Stepper<T>>) -> impl Iterator<Item = (Result<(), Issue>, Edit)> {
        steppers
            .into_iter()
            .flat_map(|stepper| [stepper.down, stepper.up])
            .flatten()
            .map(|step| (step.status, step.edit))
    }
    let mut controls: Vec<(Result<(), Issue>, Edit)> = Vec::new();
    controls.extend(options(engine.race_options(build)));
    controls.extend(options(engine.faith_options(build)));
    controls.extend(options(engine.alignment_options(build)));
    controls.extend(options(engine.background_options(build)));
    controls.extend(options(engine.racial_ability_options(build)));
    controls.extend(steps(engine.point_buy_steps(build)));
    for index in 0..build.levels.len() {
        let level = u8::try_from(index + 1).expect("at most 255 levels");
        let increases = engine.ability_increase_options(build, level);
        controls.extend(options(increases.unwrap_or_default()));
        controls.extend(steps(engine.skill_steps(build, level).unwrap_or_default()));
    }
    controls
}

/// Each start and level option and step's status against `allows` of its edit: open exactly when allowed, unless a
/// faith and an alignment that don't fit lock it.
fn statuses(engine: &Engine, name: &str, build: &Build, covered: &mut Covered) {
    for (status, edit) in controls(engine, build) {
        let allowed = engine.allows(build, &edit);
        match status {
            Ok(()) | Err(Issue::BreaksLater) => {
                assert_eq!(status.is_ok(), allowed, "{name}: {edit:?}");
            }
            Err(Issue::NeedsFaith | Issue::DeityAlignment { .. }) => {}
            Err(issue) => panic!("{name}: {issue:?} for {edit:?}"),
        }
        covered.controls += 1;
    }
}

/// Ticket 48's picker shortcut: every feat a feat slot accepts is open, not `BreaksLater`, exactly when `allows`
/// keeps the Build with it picked there (a full walk of the later choices; the shortcut gives a feat no later check
/// names the slot-left-empty answer).
fn shortcut(engine: &Engine, name: &str, build: &Build, covered: &mut Covered) {
    let sheet = engine.evaluate(build);
    for slot in sheet
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
        .filter(|slot| matches!(slot.kind, SlotKind::Feat(_)))
    {
        for option in engine.options(build, &slot.id).unwrap_or_default() {
            // Only a feat the slot accepts: the shortcut answers whether it breaks a later choice.
            if !matches!(option.pick, Pick::Feat { .. })
                || !matches!(option.status, Ok(()) | Err(Issue::BreaksLater))
            {
                continue;
            }
            let allowed = engine.allows(
                build,
                &Edit::Pick {
                    slot: slot.id.clone(),
                    pick: option.pick.clone(),
                },
            );
            assert_eq!(
                option.status.is_ok(),
                allowed,
                "{name}: {:?} {:?}: {:?}",
                slot.id,
                option.pick,
                option.status
            );
            covered.shortcut_options += 1;
        }
    }
    covered.shortcut_builds += 1;
}

#[test]
fn the_self_checks_hold_on_a_sample_of_the_corpus() {
    let (engine, data) = engine();
    let sample = corpus::builds::sample(&engine, &data);
    let covered = check_all(&engine, &sample, 4);
    // The sample reaches every check: held choices of several kinds, the controls, and the shortcut.
    assert!(covered.held.len() > 3, "{covered:?}");
    assert!(covered.controls > 0, "{covered:?}");
    assert!(covered.shortcut_options > 0, "{covered:?}");
}

#[test]
#[ignore = "about 2,000 real-data Builds: minutes in release"]
fn the_self_checks_hold_on_the_whole_corpus() {
    let threads = corpus::threads("SELF_CHECK_THREADS");
    let (engine, data) = engine();
    let start = Instant::now();
    let all = corpus::builds::all(&engine, &data, threads);
    eprintln!("{} Builds in {:?}", all.len(), start.elapsed());
    let covered = check_all(&engine, &all, threads);
    eprintln!("{covered:?}, checked in {:?}", start.elapsed());
    // Every check ran, and every prefix the shortcut check names still names a Build.
    assert!(
        covered.held.len() > 3 && covered.controls > 0,
        "{covered:?}"
    );
    for prefix in SHORTCUT_CHECKED {
        assert!(
            all.iter().any(|(name, _)| name.starts_with(prefix)),
            "no Build of the corpus is named {prefix:?}…"
        );
    }
}
