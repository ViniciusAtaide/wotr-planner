//! The MCP server through its own interface: an rmcp client talking to it over an in-memory pipe, on the real
//! game data in `libs/game-data/` or on `rules`' small invented test world (`test_world`).

mod address;
mod attack_line;
mod builds;
mod caster_line;
mod choose;
mod compare;
mod describe;
mod end_to_end;
mod listing;
mod options;
mod pick_values;
mod search;
mod sheet;
mod summaries;
mod support;
mod test_world;
mod walks;
mod withdraw;
