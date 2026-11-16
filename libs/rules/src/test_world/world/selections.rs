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
        ranks: 1,
        is_selection: false,
        prerequisite,
        levels: vec![],
        aliases: BTreeSet::new(),
        replaces_row_of: None,
        group: None,
        arcana: None,
        domain: None,
        bab_from_ranks: None,
        header: None,
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

/// A selection of feats that nests lists (ticket 304), named as the option-list slots were (`name`): each of `lists`
/// is `(option, list, opened by)`, an option whose pick is the list `list`, offered while one of the options
/// `opened by` is held (a sorcerer's bloodline feat: each bloodline's list, opened by the bloodline).
pub(super) fn nest(key: &str, name: &str, lists: &[(&str, &str, &[&str])]) -> FeatureSelection {
    let options = lists
        .iter()
        .map(|&(key, list, opened_by)| {
            let mut opened_by: Vec<Prerequisite> = opened_by
                .iter()
                .map(|key| Prerequisite::SelectionOption(option(key)))
                .collect();
            let prerequisite = if opened_by.len() == 1 {
                opened_by.remove(0)
            } else {
                Prerequisite::Any(opened_by)
            };
            SelectionOption {
                is_selection: true,
                levels: vec![OptionLevelRow {
                    selections: vec![FeatureSelectionKey::new(list)],
                    ..option_row(1, &[])
                }],
                ..selection_option(key, None, Some(prerequisite))
            }
        })
        .collect();
    FeatureSelection {
        name: name.into(),
        ..levelled_selection(key, &[], options, &[])
    }
}

pub(super) fn levelled_option(key: &str, levels: Vec<OptionLevelRow>) -> SelectionOption {
    SelectionOption {
        levels,
        ..selection_option(key, None, None)
    }
}
