//! Options with rows: an option taken through a selection with option levels, and the rows it gets as its option
//! level rises.

use std::collections::{BTreeMap, BTreeSet};

use super::{GrantSource, Held, Holding, Pending, Walker};
use crate::build::SlotId;
use crate::engine::{Slot, SlotKind};
use crate::game_data::{LevelFrom, MythicRankRow, OptionRow};
use crate::keys::{FeatureSelectionKey, SelectionOptionKey};
use crate::parameter::Parameter;

impl<'a> Walker<'a> {
    // ---- Options with rows --------------------------------------------------------------------------------

    /// Holds an option taken through a selection: the option, its feats (every value) and chosen feat (the pick's
    /// value), then its rows when the selection has option levels, and the row it replaces.
    pub(super) fn take_option(
        &mut self,
        selection: &FeatureSelectionKey,
        option: &SelectionOptionKey,
        parameter: Option<&Parameter>,
        source: Option<&GrantSource>,
    ) {
        let catalog = &self.engine.catalog;
        self.hold(
            Held::SelectionOption(option.clone(), selection.clone()),
            source.cloned(),
        );
        let Some(defined) = catalog.option(option) else {
            return;
        };
        for feat in &defined.feats {
            self.hold(
                Held::Feat(feat.clone(), catalog.every_value(feat)),
                source.cloned(),
            );
        }
        if let Some(feat) = &defined.chosen_feat {
            self.hold(
                Held::Feat(feat.clone(), parameter.iter().cloned().cloned().collect()),
                source.cloned(),
            );
        }
        if let Some(through) = catalog.selection(selection)
            && (!through.option_levels.is_empty() || through.level_from.is_some())
        {
            self.holdings.push(Holding {
                selection: selection.clone(),
                option: option.clone(),
                applied: BTreeMap::new(),
                skipped: through.skipped_levels.clone(),
                lists_only: through.feat_lists_only,
                unlocked: BTreeSet::new(),
                parameter: parameter.cloned(),
                source: source.cloned(),
                granted_by: self.granting.clone(),
            });
            // Every holding, not only the new one: one levelled by the other options held may reach a row now.
            self.advance_holdings();
        }
        if let Some(target) = &defined.replaces_row_of {
            self.replace_row(target);
        }
    }

    /// The option level of a holding: the levels of the classes its selection names, less the levels it lags
    /// behind; or, for a selection naming none, the level its source gives (for Mythic ranks, the rank it was
    /// picked at: 1 there, 2 at the next).
    fn holding_level(&self, holding: &Holding) -> u8 {
        let Some(selection) = self.engine.catalog.selection(&holding.selection) else {
            return 0;
        };
        let level = match selection.level_from {
            None => selection.option_levels.iter().fold(0u8, |sum, class| {
                sum.saturating_add(self.state.class_level(class))
            }),
            // The highest level of the other options held that it offers (themselves levelled by their classes),
            // and of the option that counts beside them (a Living Deity's way).
            Some(LevelFrom::OtherOptions) => self
                .holdings
                .iter()
                .filter(|other| {
                    other.option != holding.option
                        && (self.engine.catalog.offers(selection, &other.option)
                            || selection.level_also_from.as_ref() == Some(&other.option))
                })
                .filter(|other| {
                    self.engine
                        .catalog
                        .selection(&other.selection)
                        .is_some_and(|through| through.level_from.is_none())
                })
                .map(|other| self.holding_level(other))
                .max()
                .unwrap_or(0),
            Some(LevelFrom::GrantingOption) => holding
                .granted_by
                .as_ref()
                .map_or(0, |option| self.state.option_level(option)),
            Some(LevelFrom::MythicRanks) => holding.source.as_ref().map_or(0, |source| {
                self.state
                    .mythic_rank
                    .saturating_add(1)
                    .saturating_sub(source.rank)
            }),
        };
        level.saturating_sub(selection.levels_behind)
    }

    /// Whether `option` is held through a selection levelled by Mythic rank.
    fn levelled_by_rank(&self, option: &SelectionOptionKey) -> bool {
        self.holdings.iter().any(|holding| {
            &holding.option == option
                && self
                    .engine
                    .catalog
                    .selection(&holding.selection)
                    .is_some_and(|selection| selection.level_from == Some(LevelFrom::MythicRanks))
        })
    }

    /// At a Mythic rank, after the path row's slots: the rows the rank reaches of the options levelled by Mythic
    /// rank (Life-Bonding Friendship's teamwork feat at the rank it's picked and the next), whose slots are the
    /// rank's path slots after `row`'s own, filled until none are left (a pick can give more). The other options'
    /// row slots stay pending for the level.
    pub(super) fn fill_rank_rows(&mut self, rank: u8, row: Option<&MythicRankRow>) -> Vec<Slot> {
        let mut feat_index = row.map_or(0, |row| row.feat_slots.len());
        let mut selection_index = row.map_or(0, |row| row.selections.len());
        let mut slots = Vec::new();
        loop {
            for index in 0..self.holdings.len() {
                if self.levelled_by_rank(&self.holdings[index].option) {
                    self.advance(index);
                }
            }
            let (due, others): (Vec<Pending>, Vec<Pending>) = std::mem::take(&mut self.pending)
                .into_iter()
                .partition(|pending| {
                    matches!(pending, Pending::Row(_, option) if self.levelled_by_rank(option))
                });
            self.pending = others;
            if due.is_empty() {
                return slots;
            }
            for pending in due {
                let Pending::Row(kind, granting) = pending else {
                    continue;
                };
                let index = match &kind {
                    SlotKind::Selection(_) => &mut selection_index,
                    _ => &mut feat_index,
                };
                let at = u8::try_from(*index).unwrap_or(u8::MAX);
                *index += 1;
                let id = match &kind {
                    SlotKind::Selection(_) => SlotId::PathSelection { rank, index: at },
                    _ => SlotId::PathFeat { rank, index: at },
                };
                self.granting = Some(granting);
                slots.push(self.fill(id, kind, Some(rank)));
                self.granting = None;
            }
        }
    }

    pub(super) fn advance_holdings(&mut self) {
        for index in 0..self.holdings.len() {
            self.advance(index);
        }
    }

    /// Gives a holding every row up to its option level that it hasn't had or skipped, and sets its level and
    /// feat list in the state.
    fn advance(&mut self, index: usize) {
        let catalog = &self.engine.catalog;
        let holding = &self.holdings[index];
        let Some(option) = catalog.option(&holding.option) else {
            return;
        };
        let level = self.holding_level(holding);
        let key = holding.option.clone();
        let source = holding.source.clone();
        let parameter = holding.parameter.clone();
        let list = option
            .levels
            .iter()
            .filter(|row| row.level <= level)
            .filter_map(|row| row.feat_list.clone())
            .next_back();
        self.state.option_levels.insert(key.clone(), level);
        match list {
            Some(list) => {
                self.state.option_feat_lists.insert(key.clone(), list);
            }
            None => {
                self.state.option_feat_lists.remove(&key);
            }
        }
        for row in option.levels.iter().filter(|row| row.level <= level) {
            let holding = &self.holdings[index];
            if holding.applied.contains_key(&row.level) || holding.skipped.contains(&row.level) {
                continue;
            }
            // A lists-only holding gets its arcana from the row that has it (Crossblooded's second bloodline), or
            // the whole row when a held feature unlocks it (Expanded Defense).
            let unlocked = holding.lists_only && self.unlocks(&holding.selection, row.level);
            let held = if holding.lists_only && !unlocked {
                let arcana: Vec<Held> = option
                    .arcana
                    .iter()
                    .filter(|arcana| row.features.contains(arcana))
                    .map(|arcana| Held::Feature(arcana.clone()))
                    .collect();
                for item in &arcana {
                    self.hold(item.clone(), source.clone());
                }
                arcana
            } else {
                self.grant_row(row, &key, parameter.as_ref(), source.as_ref())
            };
            if unlocked {
                self.holdings[index].unlocked.insert(row.level);
            }
            self.holdings[index].applied.insert(row.level, held);
        }
    }

    /// Whether a held feature's option row is this row of a lists-only selection's options (Expanded Defense's,
    /// the expanded elements' 2nd).
    fn unlocks(&self, selection: &FeatureSelectionKey, level: u8) -> bool {
        self.engine
            .catalog
            .features_granting_row(selection, level)
            .any(|feature| self.state.feature_count(feature) > 0)
    }

    /// Holds an option row's features, feats, chosen feats (for `parameter`, the value the option's pick named) and
    /// class skills (from `source`) and queues its selections and feat slots as `option`'s. Returns what it holds.
    fn grant_row(
        &mut self,
        row: &'a crate::game_data::OptionLevelRow,
        option: &SelectionOptionKey,
        parameter: Option<&Parameter>,
        source: Option<&GrantSource>,
    ) -> Vec<Held> {
        let catalog = &self.engine.catalog;
        let held: Vec<Held> = row
            .features
            .iter()
            .map(|feature| Held::Feature(feature.clone()))
            .chain(
                row.feats
                    .iter()
                    .map(|feat| Held::Feat(feat.clone(), catalog.every_value(feat))),
            )
            .chain(
                row.chosen_feats
                    .iter()
                    .map(|feat| Held::Feat(feat.clone(), parameter.into_iter().cloned().collect())),
            )
            .chain(
                row.class_skills
                    .iter()
                    .map(|skill| Held::ClassSkill(skill.clone())),
            )
            .collect();
        for item in &held {
            self.hold(item.clone(), source.cloned());
        }
        self.pending.extend(
            row.selections
                .iter()
                .cloned()
                .map(|selection| Pending::Row(SlotKind::Selection(selection), option.clone())),
        );
        self.pending.extend(
            row.feat_slots
                .iter()
                .cloned()
                .map(|slot| Pending::Row(SlotKind::Feat(slot), option.clone())),
        );
        held
    }

    /// Skips, in each option held that `target` offers, the row at the level it has reached, taking back what it
    /// gave (a Primalist's rage powers instead of that level's bloodline power).
    fn replace_row(&mut self, target: &FeatureSelectionKey) {
        let Some(target) = self.engine.catalog.selection(target) else {
            return;
        };
        for index in 0..self.holdings.len() {
            let holding = &self.holdings[index];
            if holding.lists_only || !self.engine.catalog.offers(target, &holding.option) {
                continue;
            }
            let level = self.holding_level(holding);
            let holding = &mut self.holdings[index];
            holding.skipped.insert(level);
            if let Some(held) = holding.applied.remove(&level) {
                for item in &held {
                    self.release(item);
                }
            }
        }
    }

    /// A feature's option row (Bloodline Ascendance): the row at its level of each option held that its selection
    /// offers, unless already had. A lists-only holding has it only when held through that very selection
    /// (Expanded Defense: an expanded element's defensive talent), in place of the arcana it had of the row.
    pub(super) fn grant_option_row(&mut self, row: &OptionRow, source: Option<&GrantSource>) {
        let catalog = &self.engine.catalog;
        let Some(selection) = catalog.selection(&row.selection) else {
            return;
        };
        for index in 0..self.holdings.len() {
            let holding = &self.holdings[index];
            let passed_over = if holding.lists_only {
                holding.selection != row.selection || holding.unlocked.contains(&row.level)
            } else {
                holding.applied.contains_key(&row.level)
            };
            if passed_over || !catalog.offers(selection, &holding.option) {
                continue;
            }
            let Some(option_row) = catalog
                .option(&holding.option)
                .and_then(|option| option.levels.iter().find(|r| r.level == row.level))
            else {
                continue;
            };
            if self.holdings[index].lists_only {
                if let Some(arcana) = self.holdings[index].applied.remove(&row.level) {
                    for item in &arcana {
                        self.release(item);
                    }
                }
                self.holdings[index].unlocked.insert(row.level);
            }
            let parameter = self.holdings[index].parameter.clone();
            let key = self.holdings[index].option.clone();
            let held = self.grant_row(option_row, &key, parameter.as_ref(), source);
            self.holdings[index].applied.insert(row.level, held);
        }
    }
}
