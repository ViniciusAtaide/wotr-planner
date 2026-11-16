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

/// An archetype of the data, to change for a test.
pub fn archetype_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Archetype {
    data.archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == key)
        .unwrap()
}

/// A class's first archetype in the data's order (the test world's, before it is put in canonical order).
pub fn first_archetype_mut<'a>(data: &'a mut GameData, class: &str) -> &'a mut rules::Archetype {
    data.archetypes
        .iter_mut()
        .find(|a| a.class.as_str() == class)
        .unwrap()
}

/// A Feature selection of the data, to change for a test.
pub fn selection_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::FeatureSelection {
    data.selections
        .iter_mut()
        .find(|s| s.key.as_str() == key)
        .unwrap()
}

/// A feature of the data, to change for a test.
pub fn feature_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Feature {
    data.features
        .iter_mut()
        .find(|f| f.key.as_str() == key)
        .unwrap()
}

/// A Feature selection's own options, in the data's order (the test world's, before it is put in canonical order),
/// to change for a test.
pub fn own_options_mut<'a>(
    data: &'a mut GameData,
    selection: &str,
) -> Vec<&'a mut rules::SelectionOption> {
    let keys = data
        .selections
        .iter()
        .find(|s| s.key.as_str() == selection)
        .unwrap()
        .options
        .clone();
    data.options
        .iter_mut()
        .filter(|o| keys.contains(&o.key))
        .collect()
}

/// A Feature selection's `n`th own option in the data's order ([`own_options_mut`]), to change for a test.
pub fn own_option_mut<'a>(
    data: &'a mut GameData,
    selection: &str,
    n: usize,
) -> &'a mut rules::SelectionOption {
    own_options_mut(data, selection).swap_remove(n)
}

/// A feat list of the data, to change for a test.
pub fn feat_list_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::FeatList {
    data.feat_lists
        .iter_mut()
        .find(|l| l.key.as_str() == key)
        .unwrap()
}

/// A selection option of the data, to change for a test.
pub fn option_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::SelectionOption {
    data.options
        .iter_mut()
        .find(|o| o.key.as_str() == key)
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
