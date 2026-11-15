//! A feature's steps (ticket 254): a feature granted once rises at the levels its steps name, on the level the game
//! reads (a class's levels, several classes' summed, or the character level), whatever rows are traded away after the
//! grant. `Engine::feature_step` counts a grant and each step reached; `feature_count` stays the grants.

use std::collections::BTreeSet;

use rules::{
    Build, ClassChoice, DataAt, DataIssue, DataProblem, Engine, FeatureSteps, GameData, StepLevel,
};

use crate::support::*;

const RESISTANCES: &str = "sorcerer/bloodline/abyssal/resistances";
const STRENGTH: &str = "sorcerer/bloodline/abyssal/strength-of-the-abyss";

/// The test world with the Abyssal bloodline's resistances (granted once, at bloodline level 3) stepping at `at` on
/// `on`.
fn stepping(on: StepLevel, at: &[u8]) -> GameData {
    let mut data = world();
    let feature = class_mut(&mut data, "sorcerer")
        .features
        .iter_mut()
        .find(|feature| feature.key.as_str() == RESISTANCES)
        .unwrap();
    feature.steps = Some(FeatureSteps {
        on,
        at: at.iter().copied().collect(),
    });
    data
}

fn on_sorcerer() -> StepLevel {
    StepLevel::Classes(BTreeSet::from([class("sorcerer")]))
}

/// `classes` with the Abyssal bloodline at 1, and the invented Primalist's choice at 3 and 9 when given.
fn abyssal(classes: &[&str], primal: Option<[&str; 2]>) -> Build {
    let mut build = build("elf", classes);
    build.picks.insert(
        selection_slot(1, 0),
        pick_option("sorcerer/bloodline/abyssal"),
    );
    if let Some(choices) = primal {
        build.class_choices.insert(
            class("sorcerer"),
            ClassChoice {
                archetype: Some(rules::ArchetypeKey::new("sorcerer/primalist")),
                ..ClassChoice::default()
            },
        );
        for (level, choice) in [3, 9].into_iter().zip(choices) {
            build.picks.insert(
                selection_slot(level, 0),
                pick_option(&format!("sorcerer/primalist/primal-choice/{choice}")),
            );
        }
    }
    build
}

/// `feature`'s step at each of `levels` (a grant and each step reached), on a Build that holds up.
fn steps(engine: &Engine, build: &Build, feature_key: &str, levels: &[u8]) -> Vec<u8> {
    let sheet = walked(engine, build);
    let key = feature(feature_key);
    levels
        .iter()
        .map(|&level| engine.feature_step(sheet.at(level).unwrap(), &key))
        .collect()
}

/// `feature`'s grants at `level` (its feature rank), on a Build that holds up.
fn grants(engine: &Engine, build: &Build, feature_key: &str, level: u8) -> u8 {
    walked(engine, build)
        .at(level)
        .unwrap()
        .feature_count(&feature(feature_key))
}

#[test]
fn a_feature_granted_once_rises_at_its_steps_with_the_class_level() {
    let engine = engine_on(stepping(on_sorcerer(), &[9, 15]));
    let sorcerer = abyssal(&times("sorcerer", 15), None);
    assert_eq!(
        steps(&engine, &sorcerer, RESISTANCES, &[2, 3, 8, 9, 14, 15]),
        [0, 1, 1, 2, 2, 3]
    );
    // One grant: its feature rank stays 1.
    assert_eq!(grants(&engine, &sorcerer, RESISTANCES, 15), 1);
}

/// The Primalist's choice at 9 trades the bloodline's row at 9 (Strength of the Abyss), not the step the resistances
/// take at sorcerer 9: the step reads the class level, not a row. Traded at 3, the resistances are never held, so no
/// step comes.
#[test]
fn a_step_survives_a_traded_away_row_and_never_comes_without_the_grant() {
    let engine = engine_on(stepping(on_sorcerer(), &[9]));
    let kept = abyssal(&times("sorcerer", 10), Some(["bloodline-power", "talents"]));
    assert_eq!(
        steps(&engine, &kept, RESISTANCES, &[3, 8, 9, 10]),
        [1, 1, 2, 2]
    );
    assert_eq!(grants(&engine, &kept, STRENGTH, 9), 0);
    let traded = abyssal(&times("sorcerer", 10), Some(["talents", "bloodline-power"]));
    assert_eq!(steps(&engine, &traded, RESISTANCES, &[3, 8, 9, 10]), [0; 4]);
    assert_eq!(grants(&engine, &traded, STRENGTH, 9), 1);
}

/// Levels of another class don't count toward a step on the sorcerer's level; on the character level, or on both
/// classes summed, they do.
#[test]
fn a_step_reads_the_level_its_steps_name() {
    let classes = ["sorcerer", "sorcerer", "sorcerer", "fighter", "fighter"];
    for (on, expected) in [
        (on_sorcerer(), [1, 1, 1]),
        (StepLevel::CharacterLevel, [1, 1, 2]),
        (
            StepLevel::Classes(BTreeSet::from([class("sorcerer"), class("fighter")])),
            [1, 1, 2],
        ),
    ] {
        let engine = engine_on(stepping(on.clone(), &[5]));
        assert_eq!(
            steps(&engine, &abyssal(&classes, None), RESISTANCES, &[3, 4, 5]),
            expected,
            "{on:?}"
        );
    }
}

/// A step level reached before the grant counts from the grant on: the class level is read whenever the feature is
/// held.
#[test]
fn a_feature_granted_after_a_step_level_holds_that_step_at_once() {
    let engine = engine_on(stepping(StepLevel::CharacterLevel, &[2]));
    let late = abyssal(&times("sorcerer", 3), None);
    assert_eq!(steps(&engine, &late, RESISTANCES, &[2, 3]), [0, 2]);
}

#[test]
fn steps_name_a_known_class_and_a_level_to_rise_at() {
    let at = DataAt::Feature(feature(RESISTANCES));
    let issues = |data: &GameData| -> Vec<(DataAt, DataIssue)> {
        data.check()
            .into_iter()
            .map(|DataProblem { at, issue }| (at, issue))
            .collect()
    };
    assert_eq!(issues(&stepping(on_sorcerer(), &[9])), []);
    assert_eq!(
        issues(&stepping(on_sorcerer(), &[])),
        [(at.clone(), DataIssue::StepsWithoutLevels)]
    );
    assert_eq!(
        issues(&stepping(StepLevel::Classes(BTreeSet::new()), &[9])),
        [(at.clone(), DataIssue::StepsWithoutLevels)]
    );
    assert_eq!(
        issues(&stepping(on_sorcerer(), &[41])),
        [(at.clone(), DataIssue::LevelOutOfRange(41))]
    );
    let unknown = stepping(StepLevel::Classes(BTreeSet::from([class("nobody")])), &[9]);
    assert!(matches!(
        issues(&unknown).as_slice(),
        [(issue_at, DataIssue::UnknownKey(_))] if *issue_at == at
    ));
}
