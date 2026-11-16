//! The importer's writer (ticket 293, stage S1): it brings the game data's text to the game's values and keeps
//! everything else (importer design §1.2).
//!
//! The data's text is the planner's: keys, names, summaries, guide links, comments, field order and line style. The
//! writer never re-serialises a file. It finds one top-level field of an entry by its span and replaces, inserts or
//! removes only that, so the text outside the span is byte-identical and the comments survive ([`span`]).

pub mod span;
