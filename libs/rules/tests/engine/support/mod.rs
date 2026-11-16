//! The engine tests' helpers: the test world and its Build helpers (`rules::test_world`, behind the `test-world`
//! feature), plus what only these tests use.

#![allow(dead_code)]

pub use rules::test_world::*;

use rules::{Build, Engine, GameData, LevelSheet, Prerequisite, Sheet, SlotId, Unmet};

/// The sheet of a Build every choice of which holds up, so its states hold all the Build names.
pub fn walked(engine: &Engine, build: &Build) -> Sheet {
    let failures = engine.failures(build);
    assert!(failures.is_empty(), "the Build holds up: {failures:?}");
    engine.evaluate(build)
}

/// The last level of the sheet of a Build that holds up ([`walked`]), which the totals and the caster line read.
pub fn last_level(engine: &Engine, build: &Build) -> LevelSheet {
    let mut sheet = walked(engine, build);
    sheet.levels.pop().expect("a level")
}

/// Every failing single condition in an `Unmet`, as a flat list.
pub fn conditions(unmet: &Unmet) -> Vec<(&Prerequisite, Option<u8>)> {
    match unmet {
        Unmet::Condition { needs, has } => vec![(needs, *has)],
        Unmet::All(children) | Unmet::Any(children) => {
            children.iter().flat_map(conditions).collect()
        }
        Unmet::AtLeast { failed, .. } => failed.iter().flat_map(conditions).collect(),
        Unmet::MetLater { unmet, .. } => conditions(unmet),
    }
}

/// A class of the data, base or prestige, to change for a test.
pub fn class_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Class {
    data.all_classes_mut()
        .find(|c| c.key.as_str() == key)
        .unwrap()
}

/// A feat of the data, to change for a test.
pub fn feat_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Feat {
    data.feats
        .iter_mut()
        .find(|f| f.key.as_str() == key)
        .unwrap()
}

/// A selection of the sorcerer or one of its archetypes, to change for a test.
pub fn sorcerer_selection<'a>(
    data: &'a mut GameData,
    key: &str,
) -> &'a mut rules::FeatureSelection {
    class_selection(data, "sorcerer", key)
}

/// A selection of a class or one of its archetypes, to change for a test.
pub fn class_selection<'a>(
    data: &'a mut GameData,
    class: &str,
    key: &str,
) -> &'a mut rules::FeatureSelection {
    let class = class_mut(data, class);
    class
        .selections
        .iter_mut()
        .chain(
            class
                .archetypes
                .iter_mut()
                .flat_map(|archetype| &mut archetype.selections),
        )
        .find(|selection| selection.key.as_str() == key)
        .unwrap()
}

/// The keys a slot offers on `engine`, each once (an option with a chosen feat is offered once per value).
pub fn offered_on(engine: &Engine, build: &Build, slot: &SlotId) -> Vec<String> {
    let mut keys: Vec<String> = engine
        .options(build, slot)
        .unwrap()
        .into_iter()
        .map(|option| option.pick.key().to_owned())
        .collect();
    keys.dedup();
    keys
}
