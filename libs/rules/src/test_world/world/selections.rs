//! Building Feature selections and their options, with rows by option level or without.

use std::collections::{BTreeMap, BTreeSet};

use crate::test_world::world::nested::FeatureSelection;
use crate::*;

use crate::test_world::builds::{class, feature, option, skill};
use crate::test_world::world::guide;

pub(super) fn selection_option(
    key: &str,
    feat: Option<FeatKey>,
    prerequisite: Option<Prerequisite>,
) -> SelectionOption {
    SelectionOption {
        components: Vec::new(),
        key: option(key),
        blueprint: None,
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
        aliases: BTreeSet::new(),
        replaces_row_of: None,
        group: None,
        arcana: None,
        domain: None,
        bab_from_ranks: None,
    }
}

// ---- options with rows by option level ------------------------------------------------------------------------

pub(super) fn option_row(level: u8, features: &[&str]) -> OptionLevelRow {
    OptionLevelRow {
        level,
        features: features.iter().map(|key| feature(key)).collect(),
        feats: BTreeSet::new(),
        chosen_feats: BTreeSet::new(),
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
        blueprint: None,
        name: key.into(),
        summary: String::new(),
        guide: guide(key),
        options,
        shared_options: shared.iter().map(|key| option(key)).collect(),
        option_levels: levels.iter().map(|key| class(key)).collect(),
        option_level_archetypes: BTreeMap::new(),
        half_other_classes: false,
        skipped_levels: BTreeSet::new(),
        replaced_by: BTreeMap::new(),
        feat_lists_only: false,
        only_without_option: false,
        filtered_by: None,
        later_rows_behind: 0,
        level_from: None,
        once_per_option: false,
        level_also_from: None,
        deity_domains: None,
        band: Band::Features,
    }
}

pub(super) fn levelled_option(key: &str, levels: Vec<OptionLevelRow>) -> SelectionOption {
    SelectionOption {
        levels,
        ..selection_option(key, None, None)
    }
}
