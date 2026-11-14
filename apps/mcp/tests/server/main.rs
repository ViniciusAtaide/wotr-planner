//! The MCP server through its own interface: an rmcp client talking to it over an in-memory pipe, on the real
//! game data in `libs/game-data/` or on `rules`' small invented test world (`test_world`).

mod attack_line;
mod builds;
mod caster_line;
mod choose;
mod compact;
mod compare;
mod describe;
mod end_to_end;
mod finesse_training;
mod gained;
mod gear;
mod interactions;
mod listing;
mod options;
mod pick_values;
mod rank_levels;
mod ranking;
mod search;
mod sheet;
mod spells;
mod summaries;
mod support;
mod test_world;
mod totals;
mod validate;
mod walks;
mod withdraw;
