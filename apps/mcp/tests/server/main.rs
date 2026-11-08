//! The MCP server through its own interface: an rmcp client talking to it over an in-memory pipe, on the real
//! game data in `libs/game-data/` or on `rules`' small invented test world (`test_world`).

mod address;
mod builds;
mod describe;
mod listing;
mod search;
mod sheet;
mod support;
mod test_world;
