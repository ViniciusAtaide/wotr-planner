//! Options with rows: an option taken through a selection with option levels, and the rows it gets as its option
//! level rises.

use std::collections::{BTreeMap, BTreeSet};

use super::{GrantSource, Held, Holding, Pending, Walker};
use crate::build::SlotId;
use crate::engine::{Slot, SlotKind};
use crate::game_data::{LevelFrom, MythicRankRow, OptionRow, ProgressionHeader};
use crate::keys::{ClassKey, FeatureKey, FeatureSelectionKey, SelectionOptionKey};
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
        match (&defined.chosen_feat, parameter) {
            (Some(feat), _) => self.hold(
                Held::Feat(feat.clone(), parameter.iter().cloned().cloned().collect()),
                source.cloned(),
            ),
            // A value of the option's own (Finesse Training's weapon), which its repeat counts.
            (None, Some(value)) => self.hold(
                Held::OptionValue(option.clone(), value.clone()),
                source.cloned(),
            ),
            (None, None) => {}
        }
        // Held with its rows when its level is made: by a header (its own, or the one its selection's classes make), or
        // by the selection's other source.
        if let Some(through) = catalog.selection(selection)
            && let header = catalog.level_header(through, defined)
            && (header.is_some() || through.level_from.is_some())
        {
            self.holdings.push(Holding {
                selection: selection.clone(),
                option: option.clone(),
                applied: BTreeMap::new(),
                early: BTreeMap::new(),
                skipped: through.skipped_levels.clone(),
                previous_levels: header.is_none_or(|header| header.previous_levels),
                reached: 0,
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

    /// The option level of a holding: the class levels its header counts ([`Catalog::level_header`]), less the levels
    /// it lags behind; or, for a selection with another source, the level that source gives (for Mythic ranks, the
    /// rank it was picked at: 1 there, 2 at the next).
    ///
    /// [`Catalog::level_header`]: crate::engine::catalog::Catalog::level_header
    fn holding_level(&self, holding: &Holding) -> u8 {
        let catalog = &self.engine.catalog;
        let Some(selection) = catalog.selection(&holding.selection) else {
            return 0;
        };
        let level = match selection.level_from {
            None => catalog
                .option(&holding.option)
                .and_then(|option| catalog.level_header(selection, option))
                .map_or(0, |header| self.class_levels_of(header)),
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
        lagging(level, selection.later_rows_behind)
    }

    /// The class levels a progression header counts, as the game's `BlueprintProgression.CalcLevel` adds them
    /// (WrathDecompiled 6ce4a6b): with no classes and no archetypes, the character level; else each class it names
    /// plus its offset, unless that class has one of its archetypes; each class taken with one of its archetypes, plus
    /// that archetype's offset (a sum below 0 adds 0, a class not taken counts 0 levels); and half the other classes'
    /// levels, when it counts them, rounded down. The other classes include the mythic ones, whose levels are the
    /// Mythic rank (`UnitProgressionData.SetupLevelsIfNecessary`).
    fn class_levels_of(&self, header: &ProgressionHeader) -> u8 {
        let classes = &self.state.classes;
        if header.classes.is_empty() && header.archetypes.is_empty() {
            return classes
                .iter()
                .fold(0u8, |sum, taken| sum.saturating_add(taken.level));
        }
        let catalog = &self.engine.catalog;
        let archetypes = &header.archetypes;
        let has_listed_archetype = |class: &ClassKey| {
            !archetypes.is_empty()
                && (catalog.archetypes_of(class))
                    .any(|archetype| archetypes.contains_key(&archetype.key))
        };
        let level_of = |class: &ClassKey| {
            (classes.iter())
                .find(|taken| &taken.class == class)
                .map_or(0, |taken| taken.level)
        };
        let mut own = 0u8;
        for (class, &offset) in &header.classes {
            if !has_listed_archetype(class) {
                own = own.saturating_add(level_of(class).saturating_add_signed(offset));
            }
        }
        let mut others = self.state.mythic_rank;
        for taken in classes {
            match (taken.archetype.as_ref()).and_then(|archetype| archetypes.get(archetype)) {
                Some(&offset) => {
                    own = own.saturating_add(taken.level.saturating_add_signed(offset))
                }
                None if !header.classes.contains_key(&taken.class) => {
                    others = others.saturating_add(taken.level);
                }
                None => {}
            }
        }
        if header.half_other_classes {
            own.saturating_add(others / 2)
        } else {
            own
        }
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
        let (mut feat_index, mut selection_index) = row.map_or((0, 0), |row| {
            let (feats, others) = self.engine.catalog.split_selections(&row.selections);
            (feats.len(), others.len())
        });
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
        self.advance_where(|_| true);
    }

    /// At a Mythic rank, before its slots: the holdings whose option level counts half the other classes, the mythic
    /// ones among them, as every level-up recomputes each progression's level (the game's
    /// `ApplyClassMechanics.ApplyProgressions`).
    pub(super) fn advance_by_mythic_rank(&mut self) {
        self.advance_where(|header| header.is_some_and(|header| header.half_other_classes));
    }

    /// Advances each holding whose level header (`None`: its selection's other source) `by` keeps.
    fn advance_where(&mut self, by: impl Fn(Option<&ProgressionHeader>) -> bool) {
        let catalog = &self.engine.catalog;
        for index in 0..self.holdings.len() {
            let holding = &self.holdings[index];
            let header = catalog
                .selection(&holding.selection)
                .zip(catalog.option(&holding.option))
                .and_then(|(selection, option)| catalog.level_header(selection, option));
            if by(header) {
                self.advance(index);
            }
        }
    }

    /// Gives a holding every row up to its option level that it hasn't had or skipped, and sets its level in the
    /// state. A holding without its header's `previous_levels` skips the rows of the levels it passes over, as the
    /// game gives only the reached level's row (`LevelUpHelper.UpdateProgression`, WrathDecompiled 6ce4a6b).
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
        self.state.option_levels.insert(key.clone(), level);
        let holding = &mut self.holdings[index];
        if !holding.previous_levels && level > holding.reached {
            let passed = holding.reached.saturating_add(1)..level;
            (holding.skipped).extend(
                option
                    .levels
                    .iter()
                    .map(|row| row.level)
                    .filter(|at| passed.contains(at)),
            );
        }
        holding.reached = holding.reached.max(level);
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
        // What its selection gives in place of the rows it skips, as each level is reached.
        let holding = &self.holdings[index];
        if !holding.lists_only
            && let Some(through) = catalog.selection(&holding.selection)
        {
            for (at, feature) in through.replaced_by.range(..=level) {
                self.grant_replacement(index, *at, feature, source.as_ref());
            }
        }
    }

    /// After a Path switch took back what a rank's grants gave: each row a feature granted early from a dropped
    /// source is no longer applied (its items went with the grants), so the holding's option level gives it again,
    /// at once when it has reached it.
    pub(super) fn take_back_early_rows(&mut self, drops: impl Fn(&GrantSource) -> bool) {
        for holding in &mut self.holdings {
            let levels: Vec<u8> = holding
                .early
                .iter()
                .filter(|(_, source)| drops(source))
                .map(|(level, _)| *level)
                .collect();
            for level in levels {
                holding.early.remove(&level);
                holding.applied.remove(&level);
                holding.unlocked.remove(&level);
            }
        }
        self.advance_holdings();
    }

    /// Gives a holding the feature its selection has in place of the row it skips at `level`, unless it has it
    /// already (the row's `applied` entry: so once, whether it came at the option level or early).
    fn grant_replacement(
        &mut self,
        index: usize,
        level: u8,
        feature: &FeatureKey,
        source: Option<&GrantSource>,
    ) {
        if self.holdings[index].applied.contains_key(&level) {
            return;
        }
        let held = Held::Feature(feature.clone());
        self.hold(held.clone(), source.cloned());
        self.holdings[index].applied.insert(level, vec![held]);
        if let Some(source) = source {
            self.holdings[index].early.insert(level, source.clone());
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

    /// Holds an option row's features, feats, chosen feats (for `parameter`, the value the option's pick named),
    /// class skills and selections (from `source`) and queues its selections' and feat slots as `option`'s. Returns
    /// what it holds.
    fn grant_row(
        &mut self,
        row: &'a crate::game_data::OptionLevelRow,
        option: &SelectionOptionKey,
        parameter: Option<&Parameter>,
        source: Option<&GrantSource>,
    ) -> Vec<Held> {
        let catalog = &self.engine.catalog;
        let (feat_selections, selections) = catalog.split_selections(&row.selections);
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
            // Its selections, of feats too, in row order (ticket 305).
            .chain((row.selections.iter()).map(|selection| Held::Selection(selection.clone())))
            .collect();
        for item in &held {
            self.hold(item.clone(), source.cloned());
        }
        // Its other selections' slots, then its selections of feats' (ticket 301: the order the feat slots came in).
        let slots = (selections.into_iter())
            .map(|selection| SlotKind::Selection(selection.clone()))
            .chain(
                feat_selections
                    .into_iter()
                    .map(|selection| SlotKind::Feat(selection.clone())),
            );
        self.pending
            .extend(slots.map(|kind| Pending::Row(kind, option.clone())));
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
            holding.early.remove(&level);
            if let Some(held) = holding.applied.remove(&level) {
                for item in &held {
                    self.release(item);
                }
            }
        }
    }

    /// A feature's option row (Bloodline Ascendance): the row at its level of each option held that its selection
    /// offers, unless already had or skipped by the selection it is held through (`skipped_levels`, unless the row is of that very
    /// selection). A lists-only holding has it only when held through that very selection
    /// (Expanded Defense: an expanded element's defensive talent), in place of the arcana it had of the row.
    pub(super) fn grant_option_row(&mut self, row: &OptionRow, source: Option<&GrantSource>) {
        let catalog = &self.engine.catalog;
        let Some(selection) = catalog.selection(&row.selection) else {
            return;
        };
        for index in 0..self.holdings.len() {
            let holding = &self.holdings[index];
            // Where its selection skips the row and gives a feature of its own in its place, that is what it gets.
            let replacement = (!holding.lists_only && holding.selection != row.selection)
                .then(|| catalog.selection(&holding.selection))
                .flatten()
                .and_then(|held| held.replaced_by.get(&row.level))
                .filter(|_| catalog.offers(selection, &holding.option));
            if let Some(feature) = replacement {
                self.grant_replacement(index, row.level, feature, source);
                continue;
            }
            let passed_over = if holding.lists_only {
                holding.selection != row.selection || holding.unlocked.contains(&row.level)
            } else {
                // Nor a row the holding's own selection skips (the Dragonheir Scion's table gives Might of Wyrms
                // at 20 in place of its bloodline's row), unless the feature's row is of that very selection
                // (the Kinetic Knight's Defensive Wild Talent is the row its selection skips).
                holding.applied.contains_key(&row.level)
                    || (holding.selection != row.selection
                        && catalog
                            .selection(&holding.selection)
                            .is_some_and(|held| held.skipped_levels.contains(&row.level)))
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
            if let Some(source) = source {
                self.holdings[index].early.insert(row.level, source.clone());
            }
        }
    }
}

/// An option level that lags `behind` the classes' `level` from its second row on
/// ([`FeatureSelection::later_rows_behind`]).
fn lagging(level: u8, behind: u8) -> u8 {
    if level == 0 {
        0
    } else {
        level.saturating_sub(behind).max(1)
    }
}
