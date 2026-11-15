//! What a slot accepts: the options a selection offers (its own and shared; of a filtered pool, the picker lists those
//! its held options admit) and the lists a feat slot picks from. Whether a pick fits its slot is the walk's one check,
//! `walk::judge::check_pick`, which reads these.
//!
//! All of it reads only the game data and the character's state, so it's an `impl Catalog` block. The engine's
//! public answer here delegates to it.

use super::{Catalog, Engine};
use crate::game_data::{Feat, FeatList, FeatSlot, FeatureSelection};
use crate::keys::{FeatListKey, FeatureSelectionKey, SelectionOptionKey};
use crate::prerequisite::Prerequisite;
use crate::state::CharacterState;

impl Catalog {
    /// The options a selection's picker lists for a character in `state`, in key order: its own and shared ones,
    /// those of a filtered pool its held options admit ([`admits`](Self::admits)). The walk takes any option the
    /// selection offers, whose prerequisite then refuses one nothing admits, with the reason.
    pub(super) fn offered_now<'s>(
        &self,
        selection: &'s FeatureSelection,
        state: &CharacterState,
    ) -> Vec<&'s SelectionOptionKey> {
        let by = selection
            .filtered_by
            .as_ref()
            .and_then(|key| self.selection(key));
        selection
            .offered()
            .into_iter()
            .filter(|option| by.is_none_or(|by| self.admits(by, option, state)))
            .collect()
    }

    /// Whether the selection `by` that filters a pool admits `option` for a character in `state`: unless the option's
    /// prerequisite can't be met without an option of `by` that isn't held (a revelation's mystery).
    fn admits(
        &self,
        by: &FeatureSelection,
        option: &SelectionOptionKey,
        state: &CharacterState,
    ) -> bool {
        self.option(option)
            .and_then(|o| o.prerequisite.as_ref())
            .is_none_or(|prerequisite| !self.needs_unheld(by, prerequisite, state))
    }

    /// Whether `prerequisite` fails for want of an option of `by` that isn't held, whatever else is met: one it needs
    /// (under `All`), or every way it has (under `Any`), or more than it can spare (under `AtLeast`). A condition
    /// under a `Not`, or beside another way in, never makes it so, so the picker never leaves out what may be met.
    fn needs_unheld(
        &self,
        by: &FeatureSelection,
        prerequisite: &Prerequisite,
        state: &CharacterState,
    ) -> bool {
        let fails = |child: &Prerequisite| self.needs_unheld(by, child, state);
        match prerequisite {
            Prerequisite::SelectionOption(key) => {
                self.offers(by, key) && state.option_count(key) == 0
            }
            Prerequisite::All(children) => children.iter().any(fails),
            Prerequisite::Any(children) => !children.is_empty() && children.iter().all(fails),
            Prerequisite::AtLeast(needed, children) => {
                children.iter().filter(|child| !fails(child)).count() < usize::from(*needed)
            }
            _ => false,
        }
    }

    /// The lists a feat slot picks from for a character in `state`: a `OneOf`'s list, or the lists in force of
    /// the options held that an `OptionList`'s selection offers. Empty for the other slots. A picker works them out
    /// once and asks them about every feat (the walk's pick check).
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
    /// [`CheckedGameData`](crate::CheckedGameData) guarantees.
    fn slot_list(&self, key: &FeatListKey) -> &FeatList {
        self.feat_list(key)
            .expect("a slot or an option row names a feat list of the data (CheckedGameData)")
    }

    /// The selection an `OptionList` slot names: one of the data's, which
    /// [`CheckedGameData`](crate::CheckedGameData) guarantees.
    fn slot_selection(&self, key: &FeatureSelectionKey) -> &FeatureSelection {
        self.selection(key)
            .expect("a slot names a selection of the data (CheckedGameData)")
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
        // A filtered pool's options depend on what's held.
        if selection.filtered_by.is_some() {
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
    /// [`CheckedGameData`](crate::CheckedGameData) guarantees its list.
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
    /// [`CheckedGameData`](crate::CheckedGameData) guarantees its list.
    pub fn slot_accepts(&self, slot: &FeatSlot, feat: &Feat) -> bool {
        self.catalog.slot_accepts(slot, feat)
    }
}
