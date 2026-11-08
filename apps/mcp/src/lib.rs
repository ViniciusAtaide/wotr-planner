//! An MCP server over the rules engine (plan: ticket 104), so an agent can search the game data and, in later
//! slices, build a possible Build. It calls `rules` directly, the same engine the browser runs as wasm.
//!
//! - [`Planner`] (`planner.rs`): the checked game data, the [`rules::Engine`] and the search index, loaded once.
//! - [`Server`] (`server.rs`): the tools and the instructions; each tool call runs on a blocking thread.
//! - `shapes.rs`: every JSON shape a tool takes or returns (the `rules` types stay serde-free for this).
//! - One module per tool's work: `search.rs`, `describe.rs`.

mod describe;
mod planner;
mod search;
mod server;
mod shapes;

pub use planner::Planner;
pub use server::{INSTRUCTIONS, Server};
pub use shapes::Kind;
