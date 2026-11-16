//! What a slot accepts: the options a selection offers (its own and shared; of a filtered pool, the picker lists those
//! its held options admit), the feats a selection of feats offers, from its own list or a held option's, and the
//! mythic abilities a selection of them lists. Whether a pick fits its slot is the walk's one check,
//! `walk::judge::check_pick`, which reads these.
//!
//! All of it reads only the game data and the character's state, so it's an `impl Catalog` block. The engine's
//! public answer here delegates to it.

use super::{Catalog, Engine, SlotKind};
use crate::game_data::{Feat, FeatureSelection, OptionLists};
use crate::keys::{FeatureSelectionKey, MythicAbilityKey, SelectionOptionKey};
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

    /// The selections of feats a feat slot of `key` picks from for a character in `state`: the selection itself, or
    /// for one whose lists come from an option ([`FeatureSelection::option_lists`]), the lists in force of the options
    /// held that the option's selection offers. Empty for a selection the data lacks. A picker works them out once and
    /// asks them about every feat (the walk's pick check).
    pub(super) fn slot_lists(
        &self,
        key: &FeatureSelectionKey,
        state: &CharacterState,
    ) -> Vec<&FeatureSelection> {
        let Some(selection) = self.selection(key) else {
            return Vec::new();
        };
        let Some(OptionLists {
            selection: of,
            at_level,
        }) = &selection.option_lists
        else {
            return vec![selection];
        };
        let of = self.named_selection(of);
        match at_level {
            None => state
                .option_feat_lists
                .iter()
                .filter(|(option, _)| self.offers(of, option))
                .map(|(_, list)| self.named_selection(list))
                .collect(),
            // The list in force at that option level of each option held that the selection offers.
            Some(level) => state
                .selection_options
                .keys()
                .filter(|option| self.offers(of, option))
                .filter_map(|option| self.option(option))
                .filter_map(|option| {
                    option
                        .levels
                        .iter()
                        .filter(|row| row.level <= *level)
                        .filter_map(|row| row.feat_list.as_ref())
                        .next_back()
                })
                .map(|list| self.named_selection(list))
                .collect(),
        }
    }

    /// Whether a mythic ability slot of `kind` offers `ability`: a rank's general pick, any; a selection of mythic
    /// abilities, those it lists (ticket 302). `false` for any other slot.
    pub(super) fn lists_mythic_ability(&self, kind: &SlotKind, ability: &MythicAbilityKey) -> bool {
        match kind {
            SlotKind::MythicAbility => true,
            SlotKind::MythicAbilities(key) => self
                .selection(key)
                .is_some_and(|selection| selection.mythic_abilities.contains(ability)),
            _ => false,
        }
    }

    /// A selection that an option row's feat list or a selection's option lists name: one of the data's, which
    /// [`CheckedGameData`](crate::CheckedGameData) guarantees.
    fn named_selection(&self, key: &FeatureSelectionKey) -> &FeatureSelection {
        self.selection(key).expect(
            "an option row's feat list or a selection's option lists name a selection of the data",
        )
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
}

impl Engine {
    /// Whether a feat slot of the selection `key` accepts a feat, ignoring prerequisites: the selection lists it. A
    /// selection whose lists come from a held option ([`FeatureSelection::option_lists`]) accepts none here: its
    /// lists depend on the options held, which [`options`](Self::options) sees. `false` for a selection the data
    /// lacks.
    pub fn slot_accepts(&self, key: &FeatureSelectionKey, feat: &Feat) -> bool {
        (self.catalog.selection(key)).is_some_and(|selection| lists_feat(&[selection], feat))
    }
}

/// Whether a feat slot whose lists for the character are `lists` ([`Catalog::slot_lists`]) accepts a feat, ignoring
/// prerequisites: one of them lists it.
pub(super) fn lists_feat(lists: &[&FeatureSelection], feat: &Feat) -> bool {
    lists.iter().any(|list| list.feats.contains(&feat.key))
}
