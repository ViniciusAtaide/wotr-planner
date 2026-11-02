//! What a slot accepts and whether a pick fits it: the options a selection offers (its own, shared and pooled),
//! the lists a feat slot picks from, and the one check every pick goes through ([`check_pick`]).

use std::collections::BTreeSet;

use super::{Engine, SlotKind};
use crate::build::Pick;
use crate::choice::Issue;
use crate::game_data::{Feat, FeatList, FeatSlot, FeatureSelection};
use crate::keys::{FeatKey, FeatureSelectionKey, SelectionOptionKey};
use crate::parameter::{Parameter, Repeat};
use crate::state::CharacterState;

impl Engine {
    /// Whether a selection's slot offers `option` to a character in `state`: its own and shared options, and for
    /// a pool selection, the pools of the options held that it draws on.
    fn offers(
        &self,
        selection: &FeatureSelection,
        option: &SelectionOptionKey,
        state: &CharacterState,
    ) -> bool {
        selection.offers(option)
            || (selection.pool_of.is_some() && self.pools(selection, state).contains(option))
    }

    /// The options a pool selection offers now: the pools of the options held that its `pool_of` selection
    /// offers. Empty for any other selection.
    pub(super) fn pools(
        &self,
        selection: &FeatureSelection,
        state: &CharacterState,
    ) -> BTreeSet<SelectionOptionKey> {
        let Some(source) = selection
            .pool_of
            .as_ref()
            .and_then(|key| self.selection(key))
        else {
            return BTreeSet::new();
        };
        state
            .selection_options
            .keys()
            .filter(|held| source.offers(held))
            .filter_map(|held| self.option(held)?.pool.as_ref())
            .filter_map(|pool| self.selection(pool))
            .flat_map(|pool| pool.offered())
            .filter(|key| self.indexes.options.contains_key(*key))
            .cloned()
            .collect()
    }

    /// The lists a feat slot picks from for a character in `state`: a `OneOf`'s list, or the lists in force of
    /// the options held that an `OptionList`'s selection offers. Empty for the other slots.
    fn slot_lists(&self, slot: &FeatSlot, state: &CharacterState) -> Vec<&FeatList> {
        match slot {
            FeatSlot::OneOf { list } => self.feat_list(list).into_iter().collect(),
            FeatSlot::OptionList {
                selection,
                at_level: None,
                ..
            } => {
                let Some(selection) = self.selection(selection) else {
                    return Vec::new();
                };
                state
                    .option_feat_lists
                    .iter()
                    .filter(|(option, _)| selection.offers(option))
                    .filter_map(|(_, list)| self.feat_list(list))
                    .collect()
            }
            // The list in force at that option level of each option held that the selection offers.
            FeatSlot::OptionList {
                selection,
                at_level: Some(level),
                ..
            } => {
                let Some(selection) = self.selection(selection) else {
                    return Vec::new();
                };
                state
                    .selection_options
                    .keys()
                    .filter(|option| selection.offers(option))
                    .filter_map(|option| self.option(option))
                    .filter_map(|option| {
                        option
                            .levels
                            .iter()
                            .filter(|row| row.level <= *level)
                            .filter_map(|row| row.feat_list.as_ref())
                            .next_back()
                    })
                    .filter_map(|list| self.feat_list(list))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// [`slot_accepts`](Self::slot_accepts) for a character in `state`, which an `OptionList` slot needs.
    pub(super) fn slot_accepts_in(
        &self,
        slot: &FeatSlot,
        feat: &Feat,
        state: &CharacterState,
    ) -> bool {
        match slot {
            FeatSlot::OneOf { .. } | FeatSlot::OptionList { .. } => self
                .slot_lists(slot, state)
                .iter()
                .any(|list| list.contains(&feat.key)),
            _ => self.slot_accepts(slot, feat),
        }
    }

    /// Whether a pick of `feat` in this slot ignores its prerequisites, for a character in `state`: when a bonus
    /// list the slot offers it from ignores them.
    fn slot_ignores_in(&self, slot: &FeatSlot, feat: &Feat, state: &CharacterState) -> bool {
        self.slot_lists(slot, state)
            .iter()
            .any(|list| list.ignores_prerequisites && list.contains(&feat.key))
    }

    /// The one option a selection offers, when it offers exactly one: then it's granted with no pick.
    pub(super) fn single_option(
        &self,
        selection: &FeatureSelectionKey,
    ) -> Option<SelectionOptionKey> {
        let selection = self.selection(selection)?;
        if selection.pool_of.is_some() {
            return None;
        }
        let offered = selection.offered();
        match <[&SelectionOptionKey; 1]>::try_from(offered.into_iter().collect::<Vec<_>>()) {
            // An option whose pick names a value still needs the pick (the Zen Archer's Way of the Bow).
            Ok([option])
                if self
                    .option(option)
                    .is_some_and(|o| self.option_parameter(o).is_some()) =>
            {
                None
            }
            Ok([option]) => Some(option.clone()),
            Err(_) => None,
        }
    }

    /// Whether a feat slot accepts a feat, ignoring prerequisites. A list the data lacks accepts none, and so does
    /// an `OptionList` slot here: its lists depend on the options held, which [`options`](Self::options) sees.
    pub fn slot_accepts(&self, slot: &FeatSlot, feat: &Feat) -> bool {
        match slot {
            FeatSlot::General => true,
            FeatSlot::CombatFeat => feat.category.is_combat(),
            FeatSlot::Category(category) => feat.category == *category,
            FeatSlot::OneOf { list } => self
                .feat_list(list)
                .is_some_and(|list| list.contains(&feat.key)),
            FeatSlot::OptionList { .. } => false,
        }
    }

    /// Whether a pick in this slot ignores the feat's prerequisites: only a bonus-list slot's does, one whose list
    /// ignores them. Tests only.
    #[cfg(any(test, feature = "testing"))]
    pub fn slot_ignores_prerequisites(&self, slot: &FeatSlot) -> bool {
        match slot {
            FeatSlot::OneOf { list } => self
                .feat_list(list)
                .is_some_and(|list| list.ignores_prerequisites),
            _ => false,
        }
    }
}

/// Whether `pick` may fill a slot of `kind` for a character in `state`: a known key, accepted by the slot, a value
/// that fits what it takes, not already held (as often as it repeats), prerequisites met for the pick's value
/// (unless the slot is a bonus-list slot). Checked in that order.
pub(crate) fn check_pick(
    engine: &Engine,
    state: &CharacterState,
    kind: &SlotKind,
    pick: &Pick,
) -> Result<(), Issue> {
    let value = pick.parameter();
    let prerequisite = match (kind, pick) {
        (_, Pick::Feat { feat: key, .. }) => {
            let feat = engine.feat(key).ok_or(Issue::NotAllowed)?;
            let SlotKind::Feat(slot) = kind else {
                return Err(Issue::NotAllowed);
            };
            if !engine.slot_accepts_in(slot, feat, state)
                || !engine.value_fits(feat.parameter, value)
            {
                return Err(Issue::NotAllowed);
            }
            check_repeat(state, key, feat.repeat, value)?;
            if engine.slot_ignores_in(slot, feat, state) {
                return Ok(());
            }
            feat.prerequisite.as_ref()
        }
        (_, Pick::SelectionOption { option: key, .. }) => {
            let option = engine.option(key).ok_or(Issue::NotAllowed)?;
            let SlotKind::Selection(selection) = kind else {
                return Err(Issue::NotAllowed);
            };
            // Its own option or one it shares (or one of a held option's pool); a shared option keeps its one key, so
            // it's held once.
            let Some(through) = engine.selection(selection) else {
                return Err(Issue::NotAllowed);
            };
            if !engine.offers(through, key, state) {
                return Err(Issue::NotAllowed);
            }
            let listed =
                value.is_none_or(|value| option.values.is_empty() || option.values.contains(value));
            if !engine.value_fits(engine.option_parameter(option), value) || !listed {
                return Err(Issue::NotAllowed);
            }
            match option.repeat {
                Repeat::Times(times) if state.option_count(key) >= times => {
                    return Err(Issue::MaxTimes { times });
                }
                Repeat::Once if state.option_count(key) > 0 => {
                    return Err(Issue::AlreadyHeld);
                }
                // Once for each value of its chosen feat (the Skill Focus talents: once per skill), so not for one
                // the feat is held for.
                Repeat::PerValue => {
                    let held = match (&option.chosen_feat, value) {
                        (Some(feat), Some(value)) => state.has_feat_for(feat, value),
                        _ => state.option_count(key) > 0,
                    };
                    if held {
                        return Err(Issue::AlreadyHeld);
                    }
                }
                _ => {}
            }
            // Once through this selection, however often the option repeats (Expanded Element).
            if through.once_per_option && state.holds_through(selection, key) {
                return Err(Issue::AlreadyHeld);
            }
            // The chosen feat for a value it's held for already would add nothing.
            if let Some(feat) = option.chosen_feat.as_ref()
                && let Some(chosen) = engine.feat(feat)
            {
                check_repeat(state, feat, chosen.repeat, value)?;
            }
            option.prerequisite.as_ref()
        }
        (_, Pick::MythicAbility(key)) => {
            let ability = engine.mythic_ability(key).ok_or(Issue::NotAllowed)?;
            if kind != &SlotKind::MythicAbility {
                return Err(Issue::NotAllowed);
            }
            if state.mythic_abilities.contains(key) {
                return Err(Issue::AlreadyHeld);
            }
            ability.prerequisite.as_ref()
        }
        (_, Pick::MythicFeat { feat: key, .. }) => {
            let feat = engine.mythic_feat(key).ok_or(Issue::NotAllowed)?;
            if kind != &SlotKind::MythicFeat || !engine.value_fits(feat.parameter, value) {
                return Err(Issue::NotAllowed);
            }
            let held = match (feat.repeat, value) {
                (Repeat::PerValue, Some(value)) => state.has_mythic_feat_for(key, value),
                _ => state.mythic_feats.contains(key),
            };
            if held {
                return Err(Issue::AlreadyHeld);
            }
            feat.prerequisite.as_ref()
        }
    };
    match (prerequisite, pick, kind) {
        (None, _, _) => Ok(()),
        // Feats see the class levels other classes count toward (Diverse Training).
        (Some(prerequisite), Pick::Feat { .. }, _) => state
            .check_feat_for(prerequisite, value)
            .map_err(Issue::Unmet),
        // An option's may look at the pick: the option (its group) and the selection it's picked through.
        (
            Some(prerequisite),
            Pick::SelectionOption { option, .. },
            SlotKind::Selection(selection),
        ) => state
            .check_option(prerequisite, option, selection, value)
            .map_err(Issue::Unmet),
        (Some(prerequisite), _, _) => state.check_for(prerequisite, value).map_err(Issue::Unmet),
    }
}

/// Whether a feat can be held once more, for `value`: once at all, once per value, or up to its number of times.
fn check_repeat(
    state: &CharacterState,
    feat: &FeatKey,
    repeat: Repeat,
    value: Option<&Parameter>,
) -> Result<(), Issue> {
    let held = match repeat {
        Repeat::Once => state.feat_count(feat) > 0,
        Repeat::PerValue => value.is_some_and(|value| state.has_feat_for(feat, value)),
        Repeat::Times(times) => {
            if state.feat_count(feat) >= times {
                return Err(Issue::MaxTimes { times });
            }
            false
        }
        Repeat::Unlimited => false,
    };
    if held {
        Err(Issue::AlreadyHeld)
    } else {
        Ok(())
    }
}
