//! Mythic ascension: a rank's general pick and path row (the first ascension's at MR 1 and 2, the path's from MR 3),
//! choosing the path at MR 3, and what a Path switch keeps and takes away.

use std::collections::BTreeSet;

use super::{GrantSource, Held, Holding, Walker};
use crate::build::{Pick, SlotId};
use crate::engine::{
    MythicPickKind, PATH_CHOICE_RANK, PATH_SWITCH_RANK, RankStatus, SlotKind, mythic_pick_kind,
};
use crate::game_data::{MythicPath, PathSwitch};
use crate::keys::{FeatureKey, MythicAbilityKey};

impl GrantSource {
    /// What a rank's general pick, or a pick nested in it, grants: not the path's, of no selection or slot.
    fn general(rank: u8) -> Self {
        Self {
            rank,
            from_path: false,
            selection: None,
            slot: None,
        }
    }

    /// Whether a Path switch takes away what this source granted ([`PathSwitch::drops`]); `feature` when the grant
    /// is one.
    fn dropped_by(&self, switch: &PathSwitch, feature: Option<&FeatureKey>) -> bool {
        switch.drops(self.rank, self.from_path, feature, self.selection.as_ref())
    }
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
        }
        self.set_mythic_rank(rank);
        // A Mythic rank is a mythic class's level gained (`UnitProgressionData.AddClassLevel`, `OnGainClassLevel`).
        self.level_components();
        self.advance_by_mythic_rank();

        // The general pick first, then the slot its mythic feat's selection opens (Extra Feat, Extra Mythic Ability):
        // a mythic ability either holds may grant a selection, whose slot comes next (Second Bloodline; ticket 302).
        let general_id = SlotId::Mythic { rank };
        let general_pick = self.build.picks.get(&general_id);
        let mut ability = self.picked_ability(general_pick);
        let feat = match general_pick {
            Some(Pick::MythicFeat { feat, .. }) => Some((feat, self.state.mythic_feat_count(feat))),
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
        if let Some(kind) = feat
            .filter(|(key, before)| self.state.mythic_feat_count(key) > *before)
            .and_then(|(key, _)| catalog.mythic_feat(key)?.selection.as_ref())
            .and_then(|selection| self.granted_slot(selection, Some(&GrantSource::general(rank))))
        {
            let id = SlotId::MythicExtra { rank };
            ability = self.picked_ability(self.build.picks.get(&id));
            slots.push(self.fill(id, kind, Some(rank)));
        }
        let mut slot_kinds = Vec::new();
        if let Some(kind) = ability
            .filter(|(key, before)| self.state.mythic_ability_count(key) > *before)
            .map(|(key, _)| key)
            .and_then(|key| catalog.mythic_ability(key)?.selection.as_ref())
            .and_then(|selection| self.granted_slot(selection, Some(&GrantSource::general(rank))))
        {
            slot_kinds.push((kind, SlotId::MythicSelection { rank, index: 0 }));
        }
        // MR 1 and 2 give the first ascension's rows (the MR 3 path's, without one of its own); from MR 3 on, the
        // path in force gives its own.
        let path = if rank < PATH_CHOICE_RANK {
            engine.first_ascension_path(self.build)
        } else {
            self.state
                .mythic_path
                .as_ref()
                .and_then(|key| catalog.path(key))
        };
        let row = path.and_then(|path| path.row(rank));
        if let Some(row) = row {
            for feature in &row.features {
                self.hold(Held::Feature(feature.clone()), Some(GrantSource::row(rank)));
            }
            // Its selections of feats, then its other selections, each numbered in row order (ticket 301).
            let (feat_selections, selections) = catalog.split_selections(&row.selections);
            for (index, selection) in feat_selections.into_iter().enumerate() {
                // Held as the game holds a selection a progression grants (`AddFeaturesFromProgression`; ticket 305),
                // named as the selection as the others are.
                let source = GrantSource {
                    selection: Some(selection.clone()),
                    ..GrantSource::row(rank)
                };
                self.hold(Held::Selection(selection.clone()), Some(source));
                slot_kinds.push((
                    SlotKind::Feat(selection.clone()),
                    SlotId::PathFeat {
                        rank,
                        index: u8::try_from(index).unwrap_or(u8::MAX),
                    },
                ));
            }
            for (index, selection) in selections.into_iter().enumerate() {
                // Named as the selection, so a switch that drops the selection's picks drops it too.
                let source = GrantSource {
                    selection: Some(selection.clone()),
                    ..GrantSource::row(rank)
                };
                self.hold(Held::Selection(selection.clone()), Some(source));
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
        // Then the slots of the rows this rank reaches of options levelled by Mythic rank (Life-Bonding
        // Friendship).
        slots.extend(self.fill_rank_rows(rank, row));

        let path = path.map(|path| path.key.clone());
        if let Some(sheet) = self.ranks.iter_mut().find(|sheet| sheet.rank == rank) {
            sheet.status = RankStatus::Reached;
            sheet.path = path;
            sheet.slots = slots;
        }
        true
    }

    /// The mythic ability a pick names, with how many picks hold it before it: whether its slot held it is whether
    /// that number grew (a second Impossible Domain grants its selection again; ticket 303). `None` for another kind
    /// of pick.
    fn picked_ability(&self, pick: Option<&'a Pick>) -> Option<(&'a MythicAbilityKey, u8)> {
        match pick {
            Some(Pick::MythicAbility { ability, .. }) => {
                Some((ability, self.state.mythic_ability_count(ability)))
            }
            _ => None,
        }
    }

    /// Switches path at the switch rank. Returns whether the ascensions end here (a switch without further
    /// ascensions, such as Legend).
    fn switch_path(&mut self, rank: u8, path: &'a MythicPath, switch: &'a PathSwitch) -> bool {
        let (dropped, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut self.rank_grants)
            .into_iter()
            .partition(|grant| {
                let feature = match &grant.item {
                    Held::Feature(key) => Some(key),
                    _ => None,
                };
                grant.source.dropped_by(switch, feature)
            });
        self.rank_grants = kept;
        for grant in &dropped {
            self.release(&grant.item);
        }
        // An option a dropped pick took goes with it; what its rows gave was dropped with the pick's grants.
        let (gone, holdings): (Vec<Holding>, Vec<Holding>) = std::mem::take(&mut self.holdings)
            .into_iter()
            .partition(|holding| {
                holding
                    .source
                    .as_ref()
                    .is_some_and(|source| source.dropped_by(switch, None))
            });
        self.holdings = holdings;
        self.take_back_early_rows(|source| source.dropped_by(switch, None));
        for holding in gone {
            self.state.option_levels.remove(&holding.option);
        }
        for sheet in &mut self.ranks {
            if sheet.rank >= rank || sheet.status != RankStatus::Reached {
                continue;
            }
            let general_lost = !switch.keeps(sheet.rank, false);
            let path_lost = !switch.keeps(sheet.rank, true);
            // The selection a dropped path row held goes too, but alone it takes nothing the rank shows.
            let named_lost = dropped.iter().any(|grant| {
                grant.source.rank == sheet.rank
                    && grant.source.from_path
                    && !matches!(grant.item, Held::Selection(_))
            });
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
        if switch.ends_ascensions_at().is_some_and(|end| rank >= end) {
            self.set_mythic_rank(switch.rank_after_switch());
            self.ascensions_over = true;
            true
        } else {
            false
        }
    }

    /// Sets the Mythic rank, and the BAB, which may follow it (Perfect Mind).
    fn set_mythic_rank(&mut self, rank: u8) {
        self.state.mythic_rank = rank;
        self.refresh_bab();
    }
}
