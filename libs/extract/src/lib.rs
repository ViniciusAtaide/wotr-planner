//! The guide extractor: reads the saved guide pages in `guide-cache/` and builds the `libs/game-data/` files.
//!
//! It runs once (`cargo run -p extract`); after that the data files are edited by hand. The seams with tests are
//! the prerequisite-line [`grammar`], progression [`tables`], class [`progressions`] and sparse archetype rows, and
//! the [`output`] folder rules.

pub mod backgrounds;
pub mod build;
pub mod class_rules;
pub mod exploits;
pub mod extra_picks;
pub mod feat_lists;
pub mod follow_ups;
pub mod grammar;
pub mod guide;
pub mod html;
pub mod mercies_and_exploits;
pub mod mythic_extra_picks;
pub mod names;
pub mod option_class_skills;
pub mod option_families;
pub mod option_follow_ups;
pub mod option_levels;
pub mod output;
pub mod parameters;
pub mod proficiencies;
pub mod progressions;
pub mod prose;
pub mod references;
pub mod report;
pub mod report_gaps;
pub mod resolve;
pub mod ron_out;
pub mod second_follow_ups;
pub mod shared_options;
pub mod tables;
pub mod talent_follow_ups;
pub mod wild_talents;
