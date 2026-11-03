//! Building an option's rows and definitions: the names a row grants, rows from cells, and the empty row, feature,
//! option, selection and feat list constructors the families share.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ClassKey, FeatKey, FeatList, FeatListKey, Feature, FeatureKey, FeatureSelection,
    FeatureSelectionKey, GuideRef, OptionLevelRow, Repeat, SelectionOption, SelectionOptionKey,
};

use crate::names::slug;

use super::reading::FeatNames;
use super::{Family, bare};

/// The names an option row grants, for comparing with an archetype's table: its features' and its selections'.
pub(super) fn row_names(family: &Family, row: &OptionLevelRow) -> Vec<String> {
    row.features
        .iter()
        .map(|f| bare(family.feature_name(f).unwrap_or_default()))
        .chain(row.selections.iter().map(|s| {
            bare(
                family
                    .nested
                    .iter()
                    .find(|nested| &nested.key == s)
                    .map_or("", |nested| nested.name.as_str()),
            )
        }))
        .collect()
}

/// Builds an option's rows from its cells: a cell naming a feat is a granted feat, anything else a feature keyed
/// under `features_under` (shared by the options of one group, such as the ten dragons).
pub(super) struct RowBuilder<'a> {
    pub(super) feats: &'a FeatNames,
    pub(super) features: &'a mut Vec<Feature>,
}

impl RowBuilder<'_> {
    pub(super) fn rows(
        &mut self,
        cells: &BTreeMap<u8, Vec<String>>,
        features_under: &str,
        guide_for: &dyn Fn(&str) -> String,
    ) -> Vec<OptionLevelRow> {
        cells
            .iter()
            .map(|(level, cells)| {
                let mut row = empty_row(*level);
                for cell in cells {
                    // Only Wings is a feat among these cells.
                    if cell == "Wings"
                        && let Some(feat) = self.feats.get(cell)
                    {
                        row.feats.insert(feat);
                        continue;
                    }
                    let key = FeatureKey::new(format!("{features_under}/{}", slug(cell)));
                    if !self.features.iter().any(|feature| feature.key == key) {
                        self.features.push(feature(&key, cell, guide_for(cell)));
                    }
                    row.features.push(key);
                }
                row
            })
            .collect()
    }
}

pub(crate) fn empty_row(level: u8) -> OptionLevelRow {
    OptionLevelRow {
        level,
        ..OptionLevelRow::default()
    }
}

pub(crate) fn feature(key: &FeatureKey, name: &str, guide: String) -> Feature {
    Feature {
        key: key.clone(),
        name: name.to_owned(),
        summary: String::new(),
        guide: GuideRef::new(guide),
        ability_bonuses: Vec::new(),
        option_row: None,
        daily_choice: false,
        bab_floor: None,
        skill_ranks_at_least_mythic_rank: false,
    }
}

pub(crate) fn option(
    key: &str,
    name: &str,
    guide: String,
    levels: Vec<OptionLevelRow>,
) -> SelectionOption {
    SelectionOption {
        key: SelectionOptionKey::new(key),
        name: name.to_owned(),
        summary: String::new(),
        guide: GuideRef::new(guide),
        feats: BTreeSet::new(),
        chosen_feat: None,
        repeat: Repeat::Once,
        prerequisite: None,
        levels,
        pool: None,
        replaces_row_of: None,
        group: None,
        arcana: None,
        parameter: None,
        values: Default::default(),
    }
}

pub(crate) fn selection(key: &str, name: &str, guide: String, levels: &[&str]) -> FeatureSelection {
    FeatureSelection {
        key: FeatureSelectionKey::new(key),
        name: name.to_owned(),
        summary: String::new(),
        guide: GuideRef::new(guide),
        options: Vec::new(),
        shared_options: BTreeSet::new(),
        option_levels: levels.iter().map(|class| ClassKey::new(*class)).collect(),
        skipped_levels: BTreeSet::new(),
        feat_lists_only: false,
        only_without_option: false,
        pool_of: None,
        levels_behind: 0,
        level_from: None,
        once_per_option: false,
        level_also_from: None,
    }
}

pub(crate) fn list(
    key: &str,
    name: &str,
    guide: String,
    ignores: bool,
    feats: BTreeSet<FeatKey>,
) -> FeatList {
    FeatList {
        key: FeatListKey::new(key),
        name: name.to_owned(),
        guide: GuideRef::new(guide),
        ignores_prerequisites: ignores,
        every_feat: false,
        feats,
    }
}

/// Puts the list on the option's first row (creating it if it has none at that level).
pub(super) fn set_list(option: &mut SelectionOption, level: u8, list: &FeatListKey) {
    match option.levels.iter_mut().find(|row| row.level == level) {
        Some(row) => row.feat_list = Some(list.clone()),
        None => {
            option.levels.push(OptionLevelRow {
                feat_list: Some(list.clone()),
                ..empty_row(level)
            });
            option.levels.sort_by_key(|row| row.level);
        }
    }
}
