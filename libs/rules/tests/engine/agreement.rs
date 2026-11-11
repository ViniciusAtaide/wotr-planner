//! Ticket 150, step 1: every picker agrees with the walk. For each Build of a corpus grown through the pickers, at
//! every slot and level it has, an option a picker marks open is an edit whose Build is possible, and one it marks
//! closed is an edit whose Build isn't ([`rules::agreement`]). This is the oracle the cursor refactor keeps green.

use rules::agreement::{judge_all, known_for};
use rules::{Build, Edit, Engine};

use crate::support::*;

/// The disagreements the test world's corpus finds, as `agreement` writes them, each pinned by its content: the plan
/// its Build grows from, the picker with its level or slot, the option and the verdicts. None: disagreement 1 of
/// ticket 150's Findings (the class picker locking a prestige class an advancement would keep) is fixed by the
/// cursor, which judges such a class by every advancement it can take.
const KNOWN: &[&str] = &[];

/// The levels the suite's sample judges each plan at, filled and just after the level's class: the first levels,
/// MR 1 (level 6) and MR 3 (level 9), the first ascension and path choices.
const SAMPLE: &[u8] = &[1, 2, 6, 9];

/// The suite's sample: every plan grown with one seed and judged at the Build start and the [`SAMPLE`] levels. Each
/// disagreement it finds is pinned in [`KNOWN`]; the whole corpus finds them all.
#[test]
fn every_picker_agrees_with_the_walk_on_a_sample_of_the_test_world() {
    let engine = engine();
    let corpus = corpus(&engine, 1..=1, Some(SAMPLE));
    assert!(corpus.len() > 150, "a sample of {} Builds", corpus.len());
    let sweep = judge_all(&engine, &corpus);
    assert!(sweep.judged > 50_000, "{} options judged", sweep.judged);
    let known = known_for(KNOWN, &corpus);
    let unknown: Vec<&String> = sweep
        .lines
        .iter()
        .filter(|line| !known.contains(&line.as_str()))
        .collect();
    assert!(unknown.is_empty(), "{unknown:#?}");
}

/// The whole corpus, which holds the sample: every plan grown with three seeds and judged at every level (and just
/// after some levels' class). Ignored for its time (about 30 s in release, on every core):
/// `cargo test --release -p rules --test engine agreement -- --ignored --nocapture`.
#[test]
#[ignore = "about 2,400 Builds: about 30 s in release"]
fn every_picker_agrees_with_the_walk_on_the_whole_test_world_corpus() {
    let engine = engine();
    let start = std::time::Instant::now();
    let corpus = corpus(&engine, 1..=3, None);
    let sweep = judge_all(&engine, &corpus);
    eprintln!(
        "{} Builds, {} options judged, {} disagreements in {:?}",
        corpus.len(),
        sweep.judged,
        sweep.disagreements,
        start.elapsed()
    );
    assert_eq!(sweep.lines, KNOWN, "{} disagreements", sweep.lines.len());
}

/// The check itself, so an empty list of disagreements means something: an option said open whose edit the walk
/// refuses is named, with both verdicts.
#[test]
fn the_check_names_an_open_option_the_walk_refuses() {
    let engine = engine();
    let build = build("human", &["fighter"]);
    // No picker offers a class the data lacks: the status is made up, and the walk refuses the edit.
    let edit = Edit::Class {
        level: 1,
        class: class("no-such-class"),
        choice: None,
    };
    let found = rules::agreement::judge_one(
        &engine,
        &build,
        "class at 1",
        "no-such-class",
        &Ok(()),
        &edit,
        false,
    );
    let found = found.expect("a disagreement").to_string();
    assert!(
        found.starts_with("class at 1 no-such-class: picker Ok, walk refuses"),
        "{found}"
    );
}

/// An open option whose edit changes nothing (a level the Build lacks, which `apply` drops) is named, unless it is
/// the pick the Build holds there.
#[test]
fn the_check_names_an_open_option_whose_edit_changes_nothing() {
    let engine = engine();
    let build = build("human", &["fighter"]);
    let edit = Edit::AbilityIncrease {
        level: 4,
        ability: Some(rules::Ability::Strength),
    };
    let judge = |held| {
        rules::agreement::judge_one(
            &engine,
            &build,
            "increase at 4",
            "Strength",
            &Ok(()),
            &edit,
            held,
        )
    };
    let found = judge(false).expect("a disagreement").to_string();
    assert!(
        found.starts_with("increase at 4 Strength: picker Ok, walk keeps")
            && found.ends_with("the edit changes nothing"),
        "{found}"
    );
    assert_eq!(judge(true), None);
}

// ---- the corpus -----------------------------------------------------------------------------------------------

/// A Build to grow: its classes level by level (a planned class the picker doesn't open is replaced by one it does)
/// and whether it takes a Path switch at MR 8.
struct Plan {
    name: &'static str,
    classes: Vec<(&'static str, usize)>,
    switch: bool,
}

fn plan(name: &'static str, classes: &[(&'static str, usize)], switch: bool) -> Plan {
    Plan {
        name,
        classes: classes.to_vec(),
        switch,
    }
}

fn plans() -> Vec<Plan> {
    let mut plans: Vec<Plan> = [
        "fighter",
        "rogue",
        "slayer",
        "wizard",
        "cleric",
        "monk",
        "witch",
        "sorcerer",
        "bard",
        "ranger",
        "warpriest",
        "druid",
        "kineticist",
        "priest",
        "seeker",
    ]
    .into_iter()
    .map(|class| plan(class, &[(class, 20)], false))
    .collect();
    plans.extend([
        plan(
            "eldritch knight",
            &[
                ("fighter", 1),
                ("wizard", 3),
                ("eldritch-knight", 10),
                ("fighter", 6),
            ],
            true,
        ),
        plan(
            "mystic theurge",
            &[
                ("wizard", 3),
                ("cleric", 3),
                ("mystic-theurge", 10),
                ("wizard", 4),
            ],
            false,
        ),
        plan(
            "arcane trickster",
            &[
                ("rogue", 4),
                ("wizard", 3),
                ("arcane-trickster", 10),
                ("rogue", 3),
            ],
            true,
        ),
        plan(
            "winter witch",
            &[("witch", 3), ("winter-witch", 10), ("witch", 7)],
            false,
        ),
        plan(
            "dragon disciple",
            &[("sorcerer", 1), ("dragon-disciple", 10), ("sorcerer", 9)],
            true,
        ),
        plan(
            "fighter and rogue",
            &[("fighter", 2), ("rogue", 2), ("fighter", 2), ("slayer", 14)],
            true,
        ),
        plan("legend", &[("fighter", 30)], true),
        plan(
            "monk and cleric",
            &[("monk", 5), ("cleric", 5), ("monk", 10)],
            true,
        ),
    ]);
    plans
}

/// Every plan grown with each seed, each Build taken at several points of its growth: its start, then just after a
/// level's class (its slots empty) and once the level is filled, at the levels `at` names (every level when `None`;
/// then just after the class only at the [`SAMPLE`] levels, every third level and from 17 on, so the whole corpus
/// holds the sample). Each Build is keyed by its plan's name, which pins its disagreements ([`KNOWN`]) whatever the
/// seeds draw.
fn corpus(
    engine: &Engine,
    seeds: std::ops::RangeInclusive<u64>,
    at: Option<&[u8]>,
) -> Vec<(String, Build)> {
    // Each plan and seed grows on a thread of its own (each with its own chooser), joined in order.
    let plans = plans();
    let grown: Vec<Vec<(String, Build)>> = std::thread::scope(|scope| {
        let growers: Vec<_> = plans
            .iter()
            .flat_map(|plan| seeds.clone().map(move |seed| (plan, seed)))
            .map(|(plan, seed)| {
                scope.spawn(move || {
                    grow(engine, plan, seed, at)
                        .into_iter()
                        .map(|build| (plan.name.to_owned(), build))
                        .collect()
                })
            })
            .collect();
        growers
            .into_iter()
            .map(|grower| grower.join().expect("a growing thread"))
            .collect()
    });
    std::iter::once(("blank".to_owned(), Build::default()))
        .chain(grown.into_iter().flatten())
        .collect()
}

/// A small deterministic chooser.
struct Chooser(u64);

impl Chooser {
    fn pick(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(self.0 >> 33).expect("a 31-bit number") % n
    }
}

/// Applies one of the edits, chosen, among those that change the Build and keep it possible; `false` when none does.
fn take(engine: &Engine, build: &mut Build, chooser: &mut Chooser, edits: Vec<Edit>) -> bool {
    let mut kept: Vec<Build> = edits
        .iter()
        .filter_map(|edit| {
            let mut after = build.clone();
            engine.apply(&mut after, edit);
            (after != *build && engine.is_possible(&after)).then_some(after)
        })
        .collect();
    if kept.is_empty() {
        return false;
    }
    *build = kept.swap_remove(chooser.pick(kept.len()));
    true
}

/// The edits of a picker's open options.
fn open<T>(options: impl IntoIterator<Item = (Result<(), rules::Issue>, T)>) -> Vec<T> {
    options
        .into_iter()
        .filter(|(status, _)| status.is_ok())
        .map(|(_, edit)| edit)
        .collect()
}

/// A start or level picker's options with their statuses.
fn options<T>(options: Vec<rules::ChoiceOption<T>>) -> Vec<(Result<(), rules::Issue>, Edit)> {
    options.into_iter().map(|o| (o.status, o.edit)).collect()
}

/// A plan grown through the pickers, as a player would: the Build start, then level by level the class, the Mythic
/// choices its ascensions reach, the ability increase, the skill ranks and every slot, each choice one the pickers
/// open and the walk keeps. The Builds along the way.
fn grow(engine: &Engine, plan: &Plan, seed: u64, at: Option<&[u8]>) -> Vec<Build> {
    let mut chooser = Chooser(seed);
    let mut build = Build::default();
    let mut out = Vec::new();
    type StartPicker = fn(&Engine, &Build) -> Vec<(Result<(), rules::Issue>, Edit)>;
    let start: [StartPicker; 5] = [
        |engine, build| options(engine.race_options(build)),
        |engine, build| options(engine.faith_options(build)),
        |engine, build| options(engine.alignment_options(build)),
        |engine, build| options(engine.background_options(build)),
        |engine, build| options(engine.racial_ability_options(build)),
    ];
    for picker in start {
        let edits = open(picker(engine, &build));
        take(engine, &mut build, &mut chooser, edits);
    }
    for _ in 0..8 {
        let ups = engine
            .point_buy_steps(&build)
            .into_iter()
            .filter_map(|stepper| stepper.up)
            .map(|step| (step.status, step.edit));
        take(engine, &mut build, &mut chooser, open(ups));
    }
    out.push((format!("{} #{seed} start", plan.name), build.clone()));

    let planned: Vec<&str> = plan
        .classes
        .iter()
        .flat_map(|(class, n)| std::iter::repeat_n(*class, *n))
        .take(
            at.and_then(|at| at.iter().max())
                .map_or(usize::MAX, |&last| usize::from(last)),
        )
        .collect();
    let judged = |level: u8| at.is_none_or(|at| at.contains(&level));
    for (index, class) in planned.iter().enumerate() {
        let level = u8::try_from(index + 1).expect("a level");
        let Some(classes) = engine.class_options(&build, level) else {
            break;
        };
        let classes: Vec<rules::ClassOption> =
            classes.into_iter().filter(|o| o.status.is_ok()).collect();
        let wanted: Vec<&rules::ClassOption> = classes
            .iter()
            .filter(|o| o.class.as_str() == *class)
            .collect();
        let chosen: Vec<&rules::ClassOption> = if wanted.is_empty() {
            classes.iter().collect()
        } else {
            wanted
        };
        let mut edits = Vec::new();
        for option in chosen {
            if engine.asks_advances(&build, level, &option.class) {
                edits.extend(
                    engine
                        .advances_options(&build, level, option)
                        .into_iter()
                        .map(|a| a.edit),
                );
            } else {
                edits.push(option.edit.clone());
            }
        }
        if !take(engine, &mut build, &mut chooser, edits) {
            break;
        }
        let reached = |build: &Build, rank: u8| {
            engine
                .evaluate(build)
                .ranks
                .iter()
                .any(|r| r.rank == rank && r.status != rules::RankStatus::NotReached)
        };
        let paths = |o: Vec<rules::PathOption>| o.into_iter().map(|o| (o.status, o.edit));
        if build.first_ascension.is_none() && reached(&build, rules::FIRST_ASCENSION_RANK) {
            let edits = open(paths(engine.first_ascension_options(&build)));
            take(engine, &mut build, &mut chooser, edits);
        }
        if build.mythic_path.is_none() && reached(&build, rules::PATH_CHOICE_RANK) {
            let edits = open(paths(engine.path_options(&build)));
            take(engine, &mut build, &mut chooser, edits);
        }
        if plan.switch && build.path_switch.is_none() && reached(&build, rules::PATH_SWITCH_RANK) {
            let edits = open(paths(engine.switch_options(&build)));
            take(engine, &mut build, &mut chooser, edits);
        }
        if judged(level)
            && (at.is_some() || SAMPLE.contains(&level) || level % 3 == 1 || level >= 17)
        {
            out.push((
                format!("{} #{seed} level {level} unfilled", plan.name),
                build.clone(),
            ));
        }
        if let Some(increases) = engine.ability_increase_options(&build, level) {
            take(engine, &mut build, &mut chooser, open(options(increases)));
        }
        loop {
            let ups = engine
                .skill_steps(&build, level)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|stepper| stepper.up)
                .map(|step| (step.status, step.edit));
            if !take(engine, &mut build, &mut chooser, open(ups)) {
                break;
            }
        }
        fill_slots(engine, &mut build, &mut chooser, level);
        if judged(level) {
            out.push((
                format!("{} #{seed} level {level}", plan.name),
                build.clone(),
            ));
        }
    }
    for (name, build) in &out {
        assert!(engine.is_possible(build), "{name} is possible");
    }
    out.into_iter().map(|(_, build)| build).collect()
}

/// Fills every empty slot of a level and of the Mythic ranks it reaches, with options the pickers open, until no
/// empty slot takes one (a pick may grant another slot).
fn fill_slots(engine: &Engine, build: &mut Build, chooser: &mut Chooser, level: u8) {
    loop {
        let sheet = engine.evaluate(build);
        let empty: Vec<rules::SlotId> = sheet
            .levels
            .get(usize::from(level) - 1)
            .into_iter()
            .flat_map(|row| &row.slots)
            .chain(
                sheet
                    .ranks
                    .iter()
                    .filter(|rank| rank.character_level == level)
                    .flat_map(|rank| &rank.slots),
            )
            .filter(|slot| slot.pick.is_none())
            .map(|slot| slot.id.clone())
            .collect();
        let mut filled = false;
        for slot in empty {
            let edits = engine
                .options(build, &slot)
                .unwrap_or_default()
                .into_iter()
                .map(|o| (o.status, o.edit));
            filled |= take(engine, build, chooser, open(edits));
        }
        if !filled {
            return;
        }
    }
}
