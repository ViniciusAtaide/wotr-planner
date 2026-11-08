//! A small hand-built game world for tests, plus helpers for writing Builds against it.
//!
//! Invented and simplified: the numbers follow Pathfinder's usual tables, but nothing here is checked against the
//! guide. Full 20-level tables are generated so mythic ascensions and Legend can be exercised.
//!
//! Behind the `test-world` feature, which only test targets turn on (`rules`' own engine tests and `apps/mcp`'s
//! tests); no app builds it.

mod builds;
mod world;

pub use builds::*;
pub use world::world;
