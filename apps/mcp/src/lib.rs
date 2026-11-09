//! An MCP server over the rules engine (plan: ticket 104), so an agent can search the game data and build a
//! possible Build. It calls `rules` directly, the same engine the browser runs as wasm.
//!
//! - [`Planner`] (`planner.rs`): the checked game data, the [`rules::Engine`] and the search index, loaded once.
//! - [`Server`] (`server.rs`): the tools, the instructions and the one resource (`wotr://glossary`); each tool call
//!   runs on a blocking thread.
//! - `shapes.rs`: every JSON shape a tool takes or returns (the `rules` types stay serde-free for this).
//! - One module per tool's work: `search.rs`, `describe.rs`, `sheet.rs`, `options.rs`, `choose.rs`, `withdraw.rs`,
//!   `compare.rs`; `attack.rs` is the attack line `sheet` and `compare` add for a `weapon`, and `caster.rs` the caster line they
//!   add for a `caster`, each with its own shapes; `totals.rs` is the initiative, save and skill totals `sheet` gives.
//! - Builds: `code.rs` opens a Build code (only through the engine's checked decode) and writes the envelope every
//!   Build-taking tool returns; `address.rs` writes and reads choice addresses; `values.rs` writes choices' values;
//!   `error.rs` is the server's refusals (`stale_link`, `no_such_choice`, `not_an_option`, `needs_parameter`,
//!   `not_removable`, `invalid_arguments`, `internal`).
//! - `similar.rs`: how close a given name is to the planner's, for the hints after a miss, above one floor.
//! - [`http`]: the server over streamable HTTP on a loopback port (`WOTR_MCP_PORT`), which `wotr-mcp` and the app's
//!   server both serve; over stdio, `wotr-mcp` uses rmcp's own transport.

mod address;
mod attack;
mod caster;
mod choose;
mod code;
mod compare;
mod describe;
mod error;
pub mod http;
mod options;
mod planner;
mod search;
mod server;
mod shapes;
mod sheet;
mod similar;
mod totals;
mod values;
mod withdraw;

pub use address::{Address, NotAnAddress};
pub use planner::Planner;
pub use server::{GLOSSARY, GLOSSARY_URI, INSTRUCTIONS, Server};
pub use shapes::{ChoiceValue, Kind};
