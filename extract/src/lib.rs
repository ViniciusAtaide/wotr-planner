//! The guide extractor: reads the saved guide pages in `guide-cache/` and builds the `game-data/` files.
//!
//! It runs once (`cargo run -p extract`); after that the data files are edited by hand. The seams with tests are
//! the prerequisite-line [`grammar`], progression [`tables`], class [`progressions`] and sparse archetype rows, and
//! the [`output`] folder rules.

pub mod build;
pub mod feat_lists;
pub mod grammar;
pub mod guide;
pub mod html;
pub mod names;
pub mod output;
pub mod parameters;
pub mod proficiencies;
pub mod progressions;
pub mod prose;
pub mod references;
pub mod report;
pub mod resolve;
pub mod ron_out;
pub mod shared_options;
pub mod tables;
