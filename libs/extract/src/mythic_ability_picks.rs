//! Ticket 62: ticket 55's decisions 1 and 2 (user, 2026-11-02). The two mythic abilities whose entry reads like a pick
//! but that gave none ([`LEFT`]) give one slot of a class selection when they're picked (`MythicAbility::selections`),
//! as Second Spirit gives a spirit and Second Mystery a mystery, set where the mythic abilities are built
//! ([`provide`]):
//!
//! - **Witch Wandering Hex** ("You can select any hex for which you meet the prerequisites"): one slot of the witch's
//!   hexes (`witch/hex`), its options' prerequisites checked as in any hex slot. Its "Prequisites: Hex" (so spelt on
//!   the mythic page, which the prerequisite reader passes over) is Extra Hex's "Prerequisites: Hex": a hex held, the
//!   witch's or the shaman's, so any option of the selections named "Hex".
//! - **Beneficial Curse** ("You gain an additional Curse with no penalties"): one slot of the oracle's curses
//!   (`oracle/oracles-curse`). A curse held is "Already taken" there, as in any once-only option's slot. Its
//!   "Prerequisites: Oracle's Curse" is already in the data: any option of that selection.
//!
//! Each rule asserts the guide phrase it rests on where it's read, and [`check`] what it finds in the finished data, so
//! it fails loudly on data it doesn't expect.

use std::collections::BTreeSet;

use rules::{DataAt, FeatureSelectionKey, GameData, MythicAbilityKey, Prerequisite};

use crate::extra_picks::{named_selections, selections as all_selections};
use crate::follow_ups::says;
use crate::mythic_extra_picks::LEFT;
use crate::picks::Picks;

/// Each mythic ability that gives a pick: its anchor on the mythic page, its prerequisite line as the guide writes it,
/// the selection names that line names, the class whose selection the slot is, and that selection.
const PICKS: [(&str, &str, &str, &str, &str); 2] = [
    (
        "beneficial-curse",
        "Prerequisites: Oracle's Curse",
        "Oracle's Curse",
        "oracle",
        "oracle/oracles-curse",
    ),
    (
        "witch-wandering-hex",
        "Prequisites: Hex",
        "Hex",
        "witch",
        "witch/hex",
    ),
];

/// "Holds any option of these selections": an `Any` of every option they offer, own and shared, once each, in key
/// order.
pub fn holding_one_of(data: &GameData, selections: &[FeatureSelectionKey]) -> Prerequisite {
    let offered: BTreeSet<_> = all_selections(data)
        .filter(|selection| selections.contains(&selection.key))
        .flat_map(|selection| selection.offered())
        .cloned()
        .collect();
    Prerequisite::Any(
        offered
            .into_iter()
            .map(Prerequisite::SelectionOption)
            .collect(),
    )
}

/// Witch Wandering Hex and Beneficial Curse give their slot, and Witch Wandering Hex's misspelt line is read, where
/// the mythic abilities are built.
pub fn provide(guide: &crate::guide::Guide, picks: &mut Picks) {
    let anchors: BTreeSet<&str> = PICKS.iter().map(|(anchor, ..)| *anchor).collect();
    let left: BTreeSet<&str> = LEFT.iter().map(|(anchor, _)| *anchor).collect();
    assert_eq!(anchors, left, "the mythic abilities ticket 55 left");
    for (key, line, name, _, selection) in PICKS {
        says(guide, "mythic-paths", key, line);
        picks.mythic_selection(key, FeatureSelectionKey::new(selection));
        // "Prequisites: Hex", so spelt, which the prerequisite reader passes over.
        if !line.starts_with("Prerequisites: ") {
            picks.line(DataAt::MythicAbility(MythicAbilityKey::new(key)), name);
        }
    }
}

/// Each slot is of its class's selection named as the line names it, each needs one of that name's options held, and
/// Witch Wandering Hex's "Hex" is Extra Hex's.
pub fn check(_guide: &crate::guide::Guide, data: &GameData) {
    for (key, _, name, class, selection) in PICKS {
        let named = named_selections(data, name);
        let selection = FeatureSelectionKey::new(selection);
        let owner = data
            .classes
            .iter()
            .find(|c| c.key.as_str() == class)
            .unwrap_or_else(|| panic!("no class {class}"));
        assert!(
            owner.selections.iter().any(|s| s.key == selection) && named.contains(&selection),
            "{key}: {selection} isn't the {class}'s selection named {name:?} (named: {named:?})"
        );
        let mut needs = holding_one_of(data, &named);
        needs.canonicalize();
        let ability = data
            .mythic_abilities
            .iter()
            .find(|ability| ability.key.as_str() == key)
            .unwrap_or_else(|| panic!("no mythic ability {key}"));
        assert!(
            ability.selections == [selection] && ability.features.is_empty(),
            "{key}: grants something else"
        );
        let mut held = ability
            .prerequisite
            .clone()
            .unwrap_or_else(|| panic!("{key}: no prerequisite"));
        held.canonicalize();
        assert_eq!(held, needs, "{key}: its prerequisite isn't {name:?} held");
    }

    // Witch Wandering Hex's "Hex" is Extra Hex's.
    let canonical = |mut prerequisite: Prerequisite| {
        prerequisite.canonicalize();
        prerequisite
    };
    let extra_hex = data
        .feats
        .iter()
        .find(|feat| feat.key.as_str() == "extra-hex")
        .and_then(|feat| feat.prerequisite.clone())
        .map(canonical);
    let hex = data
        .mythic_abilities
        .iter()
        .find(|ability| ability.key.as_str() == "witch-wandering-hex")
        .and_then(|ability| ability.prerequisite.clone())
        .map(canonical);
    assert_eq!(
        hex, extra_hex,
        "Witch Wandering Hex's Hex isn't Extra Hex's"
    );
}
