//! What a slot accepts and whether a pick fits it: the options a selection offers (its own, shared and pooled),
//! the lists a feat slot picks from, and the one check every pick goes through ([`check_pick`]).
//!
//! All of it reads only the game data and the character's state, so it's an `impl Catalog` block, and
//! [`check_pick`] takes the [`Catalog`], not the engine. The engine's two public answers here delegate to it.

use std::collections::BTreeSet;

use super::{Catalog, Engine, SlotKind};
use crate::build::Pick;
use crate::choice::{Issue, Unmet};
use crate::game_data::{DeityDomains, Feat, FeatList, FeatSlot, FeatureSelection, SelectionOption};
use crate::keys::{FeatKey, FeatListKey, FeatureSelectionKey, SelectionOptionKey};
use crate::parameter::{Parameter, Repeat};
use crate::prerequisite::Prerequisite;
use crate::state::CharacterState;

impl Catalog {
    /// Whether a selection's slot offers `option` to a character in `state`: its own and shared options, and for
    /// a pool selection, the pools of the options held that it draws on.
    fn offers_now(
        &self,
        selection: &FeatureSelection,
        option: &SelectionOptionKey,
        state: &CharacterState,
    ) -> bool {
        self.offers(selection, option)
            || self
                .held_pools(selection, state)
                .any(|pool| pool.contains(option))
    }

    /// The options a pool selection offers now: the pools of the options held that its `pool_of` selection
    /// offers. Empty for any other selection.
    pub(super) fn pools(
        &self,
        selection: &FeatureSelection,
        state: &CharacterState,
    ) -> BTreeSet<SelectionOptionKey> {
        self.held_pools(selection, state)
            .flatten()
            .cloned()
            .collect()
    }

    /// The pool of each option held that a pool selection's `pool_of` selection offers ([`Catalog::pool`]). None
    /// for any other selection.
    fn held_pools<'s>(
        &'s self,
        selection: &FeatureSelection,
        state: &'s CharacterState,
    ) -> impl Iterator<Item = &'s BTreeSet<SelectionOptionKey>> {
        let source = selection
            .pool_of
            .as_ref()
            .and_then(|key| self.selection(key));
        source
            .into_iter()
            .flat_map(move |source| {
                state
                    .selection_options
                    .keys()
                    .filter(move |held| self.offers(source, held))
            })
            .filter_map(|held| self.pool(held))
    }

    /// The lists a feat slot picks from for a character in `state`: a `OneOf`'s list, or the lists in force of
    /// the options held that an `OptionList`'s selection offers. Empty for the other slots. A picker works them out
    /// once and asks them about every feat ([`check_pick_with`]).
    pub(super) fn slot_lists(&self, slot: &FeatSlot, state: &CharacterState) -> Vec<&FeatList> {
        match slot {
            FeatSlot::OneOf { list } => vec![self.slot_list(list)],
            FeatSlot::OptionList {
                selection,
                at_level: None,
                ..
            } => {
                let selection = self.slot_selection(selection);
                state
                    .option_feat_lists
                    .iter()
                    .filter(|(option, _)| self.offers(selection, option))
                    .map(|(_, list)| self.slot_list(list))
                    .collect()
            }
            // The list in force at that option level of each option held that the selection offers.
            FeatSlot::OptionList {
                selection,
                at_level: Some(level),
                ..
            } => {
                let selection = self.slot_selection(selection);
                state
                    .selection_options
                    .keys()
                    .filter(|option| self.offers(selection, option))
                    .filter_map(|option| self.option(option))
                    .filter_map(|option| {
                        option
                            .levels
                            .iter()
                            .filter(|row| row.level <= *level)
                            .filter_map(|row| row.feat_list.as_ref())
                            .next_back()
                    })
                    .map(|list| self.slot_list(list))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// A feat list a slot or an option row names: one of the data's, which
    /// [`GameData::check`](crate::GameData::check) makes sure of.
    fn slot_list(&self, key: &FeatListKey) -> &FeatList {
        self.feat_list(key)
            .expect("a slot or an option row names a feat list of the data (GameData::check)")
    }

    /// The selection an `OptionList` slot names: one of the data's, which
    /// [`GameData::check`](crate::GameData::check) makes sure of.
    fn slot_selection(&self, key: &FeatureSelectionKey) -> &FeatureSelection {
        self.selection(key)
            .expect("a slot names a selection of the data (GameData::check)")
    }

    /// [`slot_accepts`](Self::slot_accepts) for a character whose lists for the slot are `lists`
    /// ([`slot_lists`](Self::slot_lists)), which an `OptionList` slot needs.
    pub(super) fn slot_accepts_with(
        &self,
        slot: &FeatSlot,
        feat: &Feat,
        lists: &[&FeatList],
    ) -> bool {
        match slot {
            FeatSlot::OneOf { .. } | FeatSlot::OptionList { .. } => {
                lists.iter().any(|list| list.contains(&feat.key))
            }
            _ => self.slot_accepts(slot, feat),
        }
    }

    /// The one option a selection offers, when it offers exactly one: then it's granted with no pick. Worked out
    /// once per selection by [`Catalog::new`] ([`single_option`](Catalog::single_option) reads it).
    pub(super) fn find_single_option(
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

    /// Whether a feat slot accepts a feat, ignoring prerequisites. An `OptionList` slot accepts none here: its lists
    /// depend on the options held, which [`Engine::options`] sees.
    ///
    /// # Panics
    ///
    /// For a `OneOf` naming a list the data lacks: a slot is the data's, and
    /// [`GameData::check`](crate::GameData::check) makes sure of its list.
    pub(super) fn slot_accepts(&self, slot: &FeatSlot, feat: &Feat) -> bool {
        match slot {
            FeatSlot::General => true,
            FeatSlot::CombatFeat => feat.category.is_combat(),
            FeatSlot::Category(category) => feat.category == *category,
            FeatSlot::OneOf { list } => self.slot_list(list).contains(&feat.key),
            FeatSlot::OptionList { .. } => false,
        }
    }
}

impl Engine {
    /// Whether a feat slot accepts a feat, ignoring prerequisites. An `OptionList` slot accepts none here: its lists
    /// depend on the options held, which [`options`](Self::options) sees.
    ///
    /// # Panics
    ///
    /// For a `OneOf` naming a list the data lacks: a slot is the data's, and
    /// [`GameData::check`](crate::GameData::check) makes sure of its list.
    pub fn slot_accepts(&self, slot: &FeatSlot, feat: &Feat) -> bool {
        self.catalog.slot_accepts(slot, feat)
    }
}

/// Whether `pick` may fill a slot of `kind` for a character in `state`: a known key, accepted by the slot, a value
/// that fits what it takes, not already held (as often as it repeats), prerequisites met for the pick's value
/// (unless the slot is a bonus-list slot). Checked in that order.
pub(crate) fn check_pick(
    catalog: &Catalog,
    state: &CharacterState,
    kind: &SlotKind,
    pick: &Pick,
) -> Result<(), Issue> {
    let lists = match (kind, pick) {
        (SlotKind::Feat(slot), Pick::Feat { .. }) => catalog.slot_lists(slot, state),
        _ => Vec::new(),
    };
    check_pick_with(catalog, state, kind, &lists, pick)
}

/// [`check_pick`] with the slot's lists for `state` worked out already ([`Catalog::slot_lists`]; read only for a
/// feat pick in a feat slot), as a picker does once for all its feats.
pub(crate) fn check_pick_with(
    catalog: &Catalog,
    state: &CharacterState,
    kind: &SlotKind,
    lists: &[&FeatList],
    pick: &Pick,
) -> Result<(), Issue> {
    let value = pick.parameter();
    let prerequisite = match (kind, pick) {
        (_, Pick::Feat { feat: key, .. }) => {
            let feat = catalog.feat(key).ok_or(Issue::NotAllowed)?;
            let SlotKind::Feat(slot) = kind else {
                return Err(Issue::NotAllowed);
            };
            if !catalog.slot_accepts_with(slot, feat, lists)
                || !catalog.value_fits(feat.parameter, value)
            {
                return Err(Issue::NotAllowed);
            }
            check_repeat(state, key, feat.repeat, value)?;
            // A bonus list the slot offers it from that ignores them.
            if lists
                .iter()
                .any(|list| list.ignores_prerequisites && list.contains(&feat.key))
            {
                return Ok(());
            }
            feat.prerequisite.as_ref()
        }
        (_, Pick::SelectionOption { option: key, .. }) => {
            let option = catalog.option(key).ok_or(Issue::NotAllowed)?;
            let SlotKind::Selection(selection) = kind else {
                return Err(Issue::NotAllowed);
            };
            // Its own option or one it shares (or one of a held option's pool); a shared option keeps its one key, so
            // it's held once.
            let Some(through) = catalog.selection(selection) else {
                return Err(Issue::NotAllowed);
            };
            if !catalog.offers_now(through, key, state) {
                return Err(Issue::NotAllowed);
            }
            let listed =
                value.is_none_or(|value| option.values.is_empty() || option.values.contains(value));
            if !catalog.value_fits(catalog.option_parameter(option), value) || !listed {
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
                && let Some(chosen) = catalog.feat(feat)
            {
                check_repeat(state, feat, chosen.repeat, value)?;
            }
            if let Some(limit) = through.deity_domains {
                check_deity_domain(state, limit, option)?;
            }
            option.prerequisite.as_ref()
        }
        (_, Pick::MythicAbility { ability: key, .. }) => {
            let ability = catalog.mythic_ability(key).ok_or(Issue::NotAllowed)?;
            if kind != &SlotKind::MythicAbility || !catalog.value_fits(ability.parameter, value) {
                return Err(Issue::NotAllowed);
            }
            // Held once, whatever its value.
            if state.mythic_abilities.contains(key) {
                return Err(Issue::AlreadyHeld);
            }
            ability.prerequisite.as_ref()
        }
        (_, Pick::MythicFeat { feat: key, .. }) => {
            let feat = catalog.mythic_feat(key).ok_or(Issue::NotAllowed)?;
            if kind != &SlotKind::MythicFeat || !catalog.value_fits(feat.parameter, value) {
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

/// Whether a deity-limited selection takes an option, by its domain: one the deity offers (`Offered`) or one it
/// doesn't (`NotOffered`). Without a deity it takes none, and the reason is the deity it needs.
fn check_deity_domain(
    state: &CharacterState,
    limit: DeityDomains,
    option: &SelectionOption,
) -> Result<(), Issue> {
    let Some(deity) = state.deity() else {
        return Err(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::Worships,
            has: None,
        }));
    };
    let offered = state.deity_domains.contains(option.deity_domain());
    if offered == (limit == DeityDomains::Offered) {
        Ok(())
    } else {
        Err(Issue::DeityDomain {
            deity: deity.clone(),
            limit,
        })
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
