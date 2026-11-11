//! An MCP server over the rules engine (plan: ticket 104), so an agent can search the game data and build a
//! possible Build. It calls `rules` directly, the same engine the browser runs as wasm.
//!
//! - [`Planner`] (`planner.rs`): the checked game data, the [`rules::Engine`] and the search index, loaded once.
//! - [`Server`] (`server.rs`): the tools, the instructions and the two resources (`wotr://glossary` and
//!   `wotr://build-guide`, the `wotr-build` skill's text); each tool call
//!   runs on a blocking thread.
//! - `shapes.rs`: every JSON shape a tool takes or returns (the `rules` types stay serde-free for this).
//! - One module per tool's work: `search.rs`, `describe.rs`, `sheet.rs`, `options.rs`, `choose.rs`, `withdraw.rs`,
//!   `compare.rs`, `rank.rs` (`improve`, and the number `compare`'s `by` orders by), `validate/` (a whole declared
//!   Build's failures as a tree); `attack.rs` is the attack line `sheet` and `compare` add for a `weapon`, and `caster.rs` the caster line they
//!   add for a `caster`, each with its own shapes; `totals.rs` is the initiative, save and skill totals `sheet` gives; `assume.rs` is the
//!   gear an agent assumes on top of a Build (`assume`) and those numbers with it.
//! - Builds: `code.rs` opens a Build code (only through the engine's checked decode) and writes the envelope every
//!   Build-taking tool returns; `address.rs` writes and reads choice addresses; `values.rs` writes choices' values;
//!   `error.rs` is the server's refusals (`stale_link`, `no_such_choice`, `not_an_option`, `needs_parameter`,
//!   `not_removable`, `invalid_arguments`, `internal`).
//! - `spells.rs`: the spell catalogue (ticket 121) as `search` and `describe` give it.
//! - `similar.rs`: how close a given name is to the planner's, for the hints after a miss, above one floor.
//! - [`http`]: the server over streamable HTTP on a loopback port (`WOTR_MCP_PORT`), which `wotr-mcp` and the app's
//!   server both serve; over stdio, `wotr-mcp` uses rmcp's own transport.

mod address;
mod assume;
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
mod rank;
mod search;
mod server;
mod shapes;
mod sheet;
mod similar;
mod spells;
mod totals;
mod validate;
mod values;
mod withdraw;

pub use address::{Address, NotAnAddress};
pub use planner::Planner;
pub use server::{BUILD_GUIDE_URI, GLOSSARY, GLOSSARY_URI, INSTRUCTIONS, Server, build_guide};
pub use shapes::{ChoiceValue, Kind};
