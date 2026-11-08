//! An MCP server over the rules engine (plan: ticket 104), so an agent can search the game data and, in later
//! slices, build a possible Build. It calls `rules` directly, the same engine the browser runs as wasm.
//!
//! - [`Planner`] (`planner.rs`): the checked game data, the [`rules::Engine`] and the search index, loaded once.
//! - [`Server`] (`server.rs`): the tools and the instructions; each tool call runs on a blocking thread.
//! - `shapes.rs`: every JSON shape a tool takes or returns (the `rules` types stay serde-free for this).
//! - One module per tool's work: `search.rs`, `describe.rs`, `sheet.rs`.
//! - Builds: `code.rs` opens a Build code (only through the engine's checked decode) and writes the envelope every
//!   Build-taking tool returns; `address.rs` writes and reads choice addresses; `values.rs` writes choices' values;
//!   `error.rs` is the server's refusals (`stale_link`, `no_such_choice`).

mod address;
mod code;
mod describe;
mod error;
mod planner;
mod search;
mod server;
mod shapes;
mod sheet;
mod values;

pub use address::{Address, NotAnAddress};
pub use planner::Planner;
pub use server::{INSTRUCTIONS, Server};
pub use shapes::{ChoiceValue, Kind};
