//! The engine tests' helpers: the test world and its Build helpers (`rules::test_world`, behind the `test-world`
//! feature), plus what only these tests use.

#![allow(dead_code)]

pub use rules::test_world::*;

use rules::{Build, Engine, LevelSheet, Prerequisite, Sheet, Unmet};

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
