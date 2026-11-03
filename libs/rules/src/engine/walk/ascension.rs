//! Mythic ascension: a rank's general pick and path row, choosing the path at MR 3, and what a Path switch keeps
//! and takes away.

use std::collections::BTreeSet;

use super::{GrantSource, Held, Holding, RankGrant, Walker};
use crate::build::{Pick, SlotId};
use crate::engine::{
    MythicPickKind, PATH_CHOICE_RANK, PATH_SWITCH_RANK, RankStatus, SlotKind, mythic_pick_kind,
};
use crate::game_data::{FeatSlot, MythicExtraPick, MythicPath, PathSwitch};
use crate::keys::MythicAbilityKey;

/// Whether a Path switch takes a grant away: a general pick past the general cut-off, the old path's grant past
/// the path cut-off, or one the switch drops by name whatever its rank.
fn switch_drops(switch: &PathSwitch, grant: &RankGrant) -> bool {
    let source = &grant.source;
    if !source.from_path {
        return source.rank > switch.kept_through_rank;
    }
    source.rank > switch.path_kept_through_rank
        || matches!(&grant.item, Held::Feature(key) if switch.dropped_features.contains(key))
        || source
            .selection
            .as_ref()
            .is_some_and(|selection| switch.dropped_selections.contains(selection))
}

impl<'a> Walker<'a> {
    // ---- Mythic ascension ---------------------------------------------------------------------------------

    /// Applies one Mythic rank. Returns whether the rank was reached.
    pub(super) fn ascend(&mut self, rank: u8) -> bool {
        if self.ascensions_over {
            return false;
        }
        let engine = self.engine;
        let catalog = &engine.catalog;
        if rank == PATH_SWITCH_RANK
            && let Some((path, switch)) = engine.chosen_switch(self.build)
            && self.switch_path(rank, path, switch)
        {
            return false;
        }
        // A path or switch that can't be taken doesn't happen (`check_unreached` fails it).
        if rank == PATH_CHOICE_RANK
            && let Some(path) = engine.chosen_path(self.build)
        {
            self.state.mythic_path = Some(path.key.clone());
            self.grant_rows_before_the_choice(path);
        }
        self.state.mythic_rank = rank;

        // The general pick first, then the slot its mythic feat gives (Extra Feat, Extra Mythic Ability): a mythic
        // ability either holds may grant selection slots (Second Bloodline).
        let general_id = SlotId::Mythic { rank };
        let general_pick = self.build.picks.get(&general_id);
        let mut ability = self.newly_held_ability(general_pick);
        let feat = match general_pick {
            Some(Pick::MythicFeat { feat, .. }) if !self.state.mythic_feats.contains(feat) => {
                Some(feat)
            }
            _ => None,
        };
        let general = self.fill(
            general_id,
            match mythic_pick_kind(rank) {
                MythicPickKind::Ability => SlotKind::MythicAbility,
                MythicPickKind::Feat => SlotKind::MythicFeat,
            },
            Some(rank),
        );
        let mut slots = vec![general];
        if let Some(extra) = feat
            .filter(|key| self.state.mythic_feats.contains(*key))
            .and_then(|key| catalog.mythic_feat(key))
            .and_then(|feat| feat.extra_pick)
        {
            let id = SlotId::MythicExtra { rank };
            ability = self.newly_held_ability(self.build.picks.get(&id));
            let kind = match extra {
                MythicExtraPick::Feat => SlotKind::Feat(FeatSlot::General),
                MythicExtraPick::MythicAbility => SlotKind::MythicAbility,
            };
            slots.push(self.fill(id, kind, Some(rank)));
        }
        let mut slot_kinds = Vec::new();
        if let Some(ability) = ability
            .filter(|key| self.state.mythic_abilities.contains(*key))
            .and_then(|key| catalog.mythic_ability(key))
        {
            let source = GrantSource {
                rank,
                from_path: false,
                selection: None,
                slot: None,
            };
            for (index, selection) in ability.selections.iter().enumerate() {
                if self.grant_without_slot(selection, Some(&source)) {
                    continue;
                }
                slot_kinds.push((
                    SlotKind::Selection(selection.clone()),
                    SlotId::MythicSelection {
                        rank,
                        index: u8::try_from(index).unwrap_or(u8::MAX),
                    },
                ));
            }
        }
        let path = self
            .state
            .mythic_path
            .as_ref()
            .and_then(|key| catalog.path(key));
        if let Some(row) = path.and_then(|path| path.ranks.iter().find(|row| row.rank == rank)) {
            for feature in &row.features {
                self.hold(Held::Feature(feature.clone()), Some(GrantSource::row(rank)));
            }
            for (index, slot) in row.feat_slots.iter().enumerate() {
                slot_kinds.push((
                    SlotKind::Feat(slot.clone()),
                    SlotId::PathFeat {
                        rank,
                        index: u8::try_from(index).unwrap_or(u8::MAX),
                    },
                ));
            }
            for (index, selection) in row.selections.iter().enumerate() {
                slot_kinds.push((
                    SlotKind::Selection(selection.clone()),
                    SlotId::PathSelection {
                        rank,
                        index: u8::try_from(index).unwrap_or(u8::MAX),
                    },
                ));
            }
        }
        slots.extend(self.fill_all(slot_kinds, Some(rank)));

        let path = self.state.mythic_path.clone();
        if let Some(sheet) = self.ranks.iter_mut().find(|sheet| sheet.rank == rank) {
            sheet.status = RankStatus::Reached;
            sheet.path = path;
            sheet.slots = slots;
        }
        true
    }

    /// The mythic ability a pick would hold anew: `None` for another kind of pick, or one held already.
    fn newly_held_ability(&self, pick: Option<&'a Pick>) -> Option<&'a MythicAbilityKey> {
        match pick {
            Some(Pick::MythicAbility(key)) if !self.state.mythic_abilities.contains(key) => {
                Some(key)
            }
            _ => None,
        }
    }

    /// Choosing the path at MR 3 grants its MR 1 and 2 rows: held from now on, as those ranks' own grants (so a
    /// switch that keeps MR 1-2 keeps them), and shown at those ranks. Only their features: `GameData::check`
    /// gives those rows no slots.
    fn grant_rows_before_the_choice(&mut self, path: &'a MythicPath) {
        for row in path.ranks.iter().filter(|row| row.rank < PATH_CHOICE_RANK) {
            for feature in &row.features {
                self.hold(
                    Held::Feature(feature.clone()),
                    Some(GrantSource::row(row.rank)),
                );
            }
        }
        for sheet in &mut self.ranks {
            if sheet.rank < PATH_CHOICE_RANK && sheet.status == RankStatus::Reached {
                sheet.path = Some(path.key.clone());
            }
        }
    }

    /// Switches path at the switch rank. Returns whether the ascensions end here (a switch without further
    /// ascensions, such as Legend).
    fn switch_path(&mut self, rank: u8, path: &'a MythicPath, switch: &'a PathSwitch) -> bool {
        let (dropped, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut self.rank_grants)
            .into_iter()
            .partition(|grant| switch_drops(switch, grant));
        self.rank_grants = kept;
        for grant in &dropped {
            self.release(&grant.item);
        }
        // An option a dropped pick took goes with it; what its rows gave was dropped with the pick's grants.
        let (gone, holdings): (Vec<Holding>, Vec<Holding>) = std::mem::take(&mut self.holdings)
            .into_iter()
            .partition(|holding| {
                holding.source.as_ref().is_some_and(|source| {
                    switch_drops(
                        switch,
                        &RankGrant {
                            source: source.clone(),
                            item: Held::SelectionOption(
                                holding.option.clone(),
                                holding.selection.clone(),
                            ),
                        },
                    )
                })
            });
        self.holdings = holdings;
        for holding in gone {
            self.state.option_levels.remove(&holding.option);
            self.state.option_feat_lists.remove(&holding.option);
        }
        for sheet in &mut self.ranks {
            if sheet.rank >= rank || sheet.status != RankStatus::Reached {
                continue;
            }
            let general_lost = sheet.rank > switch.kept_through_rank;
            let path_lost = sheet.rank > switch.path_kept_through_rank;
            let named_lost = dropped
                .iter()
                .any(|grant| grant.source.rank == sheet.rank && grant.source.from_path);
            sheet.status = if general_lost && path_lost {
                RankStatus::Dropped
            } else if general_lost || path_lost || named_lost {
                RankStatus::PartlyDropped
            } else {
                RankStatus::Reached
            };
            let lost: BTreeSet<&SlotId> = dropped
                .iter()
                .filter(|grant| grant.source.rank == sheet.rank)
                .filter_map(|grant| grant.source.slot.as_ref())
                .collect();
            sheet.dropped_slots = sheet
                .slots
                .iter()
                .map(|slot| &slot.id)
                .filter(|id| lost.contains(id))
                .cloned()
                .collect();
        }

        self.state.mythic_path = Some(path.key.clone());
        for feature in &switch.features {
            self.hold(Held::Feature(feature.clone()), None);
        }
        if let Some(cap) = switch.level_cap {
            self.level_cap = cap;
        }
        if switch.further_ascensions {
            false
        } else {
            self.state.mythic_rank = switch.rank_after_switch();
            self.ascensions_over = true;
            true
        }
    }
}
