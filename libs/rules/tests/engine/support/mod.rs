//! The engine tests' helpers: the test world and its Build helpers (`rules::test_world`, behind the `test-world`
//! feature), plus what only these tests use.

#![allow(dead_code)]

pub use rules::test_world::*;

use rules::{Prerequisite, Unmet};

/// Every failing single condition in an `Unmet`, as a flat list.
pub fn conditions(unmet: &Unmet) -> Vec<(&Prerequisite, Option<u8>)> {
    match unmet {
        Unmet::Condition { needs, has } => vec![(needs, *has)],
        Unmet::All(children) | Unmet::Any(children) => {
            children.iter().flat_map(conditions).collect()
        }
        Unmet::AtLeast { failed, .. } => failed.iter().flat_map(conditions).collect(),
    }
}
