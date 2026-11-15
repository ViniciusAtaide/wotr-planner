//! The game check's tests, on a small invented dump and game data (`blueprints.json`, `planner.ron`): each comparison
//! finds its planted difference, a shuffled run gives the same bytes, and a dump that isn't the pinned one is refused.

mod areas;
mod determinism;
mod known;
mod pins;
mod refusal;
mod support;
