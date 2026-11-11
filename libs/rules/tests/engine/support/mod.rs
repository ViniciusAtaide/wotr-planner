//! The engine tests' helpers: the test world and its Build helpers (`rules::test_world`, behind the `test-world`
//! feature), plus what only these tests use.

#![allow(dead_code)]

pub use rules::test_world::*;

use rules::{Build, CharacterState, Engine, Prerequisite, Sheet, Unmet};

/// The sheet of a Build every choice of which holds up, so its states hold all the Build names.
pub fn walked(engine: &Engine, build: &Build) -> Sheet {
    let failures = engine.failures(build);
    assert!(failures.is_empty(), "the Build holds up: {failures:?}");
    engine.evaluate(build)
}

/// The character after the last level of a Build that holds up ([`walked`]).
pub fn last_state(engine: &Engine, build: &Build) -> CharacterState {
    let sheet = walked(engine, build);
    sheet.levels.last().expect("a level").state.clone()
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
