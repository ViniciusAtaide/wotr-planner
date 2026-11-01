//! The guide extractor: reads the saved guide pages in `guide-cache/` and builds the `game-data/` files.
//!
//! It runs once (`cargo run -p extract`); after that the data files are edited by hand. The seams with tests are
//! the prerequisite-line [`grammar`], progression [`tables`] and the [`output`] folder rules.

pub mod build;
pub mod grammar;
pub mod guide;
pub mod html;
pub mod names;
pub mod output;
pub mod prose;
pub mod references;
pub mod report;
pub mod resolve;
pub mod ron_out;
pub mod tables;
