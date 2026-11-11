//! The walk's own checks of a choice against the character just before it: a pick in its slot ([`check_pick`]) and
//! a class's first level ([`class_entry`]). The walk asks them as it goes, and the cursor asks the same ones of a
//! candidate at the paused walk, so a picker and the walk can't judge a choice two ways.

use crate::build::Pick;
use crate::choice::{Issue, Unmet};
use crate::engine::SlotKind;
use crate::engine::catalog::Catalog;
use crate::game_data::{Class, DeityDomains, FeatList, SelectionOption};
use crate::keys::{ArchetypeKey, FeatKey};
use crate::parameter::{Parameter, Repeat};
use crate::prerequisite::Prerequisite;
use crate::state::CharacterState;

/// Whether `pick` may fill a slot of `kind` for a character in `state`: a known key, accepted by the slot, a value
/// that fits what it takes, not already held (as often as it repeats), prerequisites met for the pick's value
/// (unless the slot is a bonus-list slot). Checked in that order.
pub(super) fn check_pick(
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
pub(super) fn check_pick_with(
    catalog: &Catalog,
    state: &CharacterState,
    kind: &SlotKind,
    lists: &[&FeatList],
    pick: &Pick,
) -> Result<(), Issue> {
    let value = pick.parameter();
    let prerequisite = match (kind, pick) {
        (_, Pick::Feat { feat: key, .. }) => {
            let feat = catalog.feat(key).ok_or(Issue::UnknownKey)?;
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
            let option = catalog.option(key).ok_or(Issue::UnknownKey)?;
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
            let ability = catalog.mythic_ability(key).ok_or(Issue::UnknownKey)?;
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
            let feat = catalog.mythic_feat(key).ok_or(Issue::UnknownKey)?;
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

/// What keeps a character in `state` from taking a class's first level with an archetype key (`None`: the class's
/// own): the class's unmet entry prerequisites, and the archetype's issue, which is a key the data lacks
/// ([`Issue::UnknownKey`]), an archetype of another class ([`Issue::NotAllowed`]) or its unmet prerequisites. An
/// archetype stating an alignment requirement replaces the class's ([`Catalog::entry_prerequisites`]).
///
/// The one class-entry check: the walk fails the class and the archetype with its two parts, and the cursor joins
/// them into a class option's issue ([`Entry::issue`]). A prestige class's advancement belongs to its entry too: the
/// walk checks it on the class's first level, and the cursor judges it by resuming the walk there.
pub(super) struct Entry {
    pub(super) class: Option<Unmet>,
    pub(super) archetype: Option<Issue>,
}

pub(super) fn class_entry(
    catalog: &Catalog,
    state: &CharacterState,
    class: &Class,
    key: Option<&ArchetypeKey>,
) -> Entry {
    let archetype = catalog.archetype_of(class, key);
    let [class_needs, archetype_needs] = catalog.entry_prerequisites(class, archetype);
    let unmet = |needs: Option<&Prerequisite>| needs.and_then(|needs| state.check(needs).err());
    let archetype = match (key, archetype) {
        (None, _) => None,
        (Some(key), None) if catalog.archetype(key).is_none() => Some(Issue::UnknownKey),
        (Some(_), None) => Some(Issue::NotAllowed),
        (Some(_), Some(_)) => unmet(archetype_needs).map(Issue::Unmet),
    };
    Entry {
        class: unmet(class_needs),
        archetype,
    }
}

impl Entry {
    /// The entry as one issue, as a class option shows it: the archetype's key problem, else every unmet
    /// prerequisite, the class's then the archetype's (one alone, or all of them).
    pub(super) fn issue(self) -> Result<(), Issue> {
        let archetype = match self.archetype {
            Some(Issue::Unmet(unmet)) => Some(unmet),
            Some(issue) => return Err(issue),
            None => None,
        };
        let unmet: Vec<Unmet> = self.class.into_iter().chain(archetype).collect();
        match <[Unmet; 1]>::try_from(unmet) {
            Ok([unmet]) => Err(Issue::Unmet(unmet)),
            Err(unmet) if unmet.is_empty() => Ok(()),
            Err(unmet) => Err(Issue::Unmet(Unmet::All(unmet))),
        }
    }
}
