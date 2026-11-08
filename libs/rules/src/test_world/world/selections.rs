//! Building Feature selections and their options, with rows by option level or without.

use std::collections::BTreeSet;

use crate::*;

use crate::test_world::builds::{class, feature, option, skill};
use crate::test_world::world::guide;

pub(super) fn selection_option(
    key: &str,
    feat: Option<FeatKey>,
    prerequisite: Option<Prerequisite>,
) -> SelectionOption {
    SelectionOption {
        key: option(key),
        name: key.rsplit('/').next().unwrap_or(key).into(),
        summary: String::new(),
        guide: guide(key),
        feats: feat.into_iter().collect(),
        chosen_feat: None,
        parameter: None,
        values: BTreeSet::new(),
        repeat: Repeat::Once,
        prerequisite,
        levels: vec![],
        pool: None,
        replaces_row_of: None,
        group: None,
        arcana: None,
        domain: None,
    }
}

// ---- options with rows by option level ------------------------------------------------------------------------

pub(super) fn option_row(level: u8, features: &[&str]) -> OptionLevelRow {
    OptionLevelRow {
        level,
        features: features.iter().map(|key| feature(key)).collect(),
        feats: BTreeSet::new(),
        chosen_feats: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
        feat_list: None,
        class_skills: BTreeSet::new(),
    }
}

/// `row` with `skills` as its class skills.
pub(super) fn with_class_skills(row: OptionLevelRow, skills: &[&str]) -> OptionLevelRow {
    OptionLevelRow {
        class_skills: skills.iter().map(|key| skill(key)).collect(),
        ..row
    }
}

/// A selection whose options have rows: `levels` are the classes whose levels make the option level.
pub(super) fn levelled_selection(
    key: &str,
    levels: &[&str],
    options: Vec<SelectionOption>,
    shared: &[&str],
) -> FeatureSelection {
    FeatureSelection {
        key: FeatureSelectionKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: guide(key),
        options,
        shared_options: shared.iter().map(|key| option(key)).collect(),
        option_levels: levels.iter().map(|key| class(key)).collect(),
        skipped_levels: BTreeSet::new(),
        feat_lists_only: false,
        only_without_option: false,
        pool_of: None,
        levels_behind: 0,
        level_from: None,
        once_per_option: false,
        level_also_from: None,
        deity_domains: None,
    }
}

pub(super) fn levelled_option(key: &str, levels: Vec<OptionLevelRow>) -> SelectionOption {
    SelectionOption {
        levels,
        ..selection_option(key, None, None)
    }
}
