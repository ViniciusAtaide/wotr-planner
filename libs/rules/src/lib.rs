//! Game rules for the WotR build planner. Pure Rust + serde, compiled for both the browser and the server.

mod bonus;
mod build;
mod check;
mod choice;
mod codec;
mod engine;
mod game_data;
mod gear;
mod keys;
mod parameter;
mod prerequisite;
mod spells;
mod state;
#[cfg(feature = "test-world")]
pub mod test_world;

pub use bonus::*;
pub use build::*;
pub use check::{CheckedGameData, DataAt, DataIssue, DataProblem};
pub use choice::*;
pub use codec::*;
pub use engine::*;
pub use game_data::*;
pub use gear::*;
pub use keys::*;
pub use parameter::*;
pub use prerequisite::{Prerequisite, Story};
pub use spells::*;
pub use state::*;
