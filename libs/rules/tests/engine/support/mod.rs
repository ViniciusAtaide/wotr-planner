//! A small hand-built game world for the rules engine tests, plus helpers for writing Builds.
//!
//! Invented and simplified: the numbers follow Pathfinder's usual tables, but nothing here is checked against the
//! guide. Full 20-level tables are generated so mythic ascensions and Legend can be exercised.

#![allow(dead_code)]

mod builds;
mod world;

pub use builds::*;
pub use world::world;

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
