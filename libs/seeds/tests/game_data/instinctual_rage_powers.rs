//! The Instinctual Warrior's monk rage powers (ticket 66).

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, Build, ClassKey, FeatKey, Prerequisite, Repeat, SelectionOptionKey, SlotId,
};

use crate::support::*;

const RAGE_POWER: &str = "barbarian/rage-power";
const INSTINCTUAL_WARRIOR: &str = "barbarian/instinctual-warrior";

/// The guide's monk abilities and ki powers an Instinctual Warrior may take as rage powers
/// (`barbarian#rage-power-instinctual-warrior`): each one's key in the rage power selection, its name, and the
/// barbarian level it needs (`None`: Deflect Arrows, which needs Improved Unarmed Strike instead).
const MONK_RAGE_POWERS: [(&str, &str, Option<u8>); 5] = [
    (
        "barbarian/rage-power/deflect-arrows-instinctual-warrior",
        "Deflect Arrows (Instinctual Warrior)",
        None,
    ),
    (
        "barbarian/rage-power/purity-of-body-instinctual-warrior",
        "Purity of Body (Instinctual Warrior)",
        Some(8),
    ),
    (
        "barbarian/rage-power/diamond-body-instinctual-warrior",
        "Diamond Body (Instinctual Warrior)",
        Some(12),
    ),
    (
        "barbarian/rage-power/diamond-soul-instinctual-warrior",
        "Diamond Soul (Instinctual Warrior)",
        Some(16),
    ),
    (
        "barbarian/rage-power/perfect-self-instinctual-warrior",
        "Perfect Self (Instinctual Warrior)",
        Some(20),
    ),
];

/// The Instinctual Warrior's rage power levels, by its table.
const INSTINCTUAL_RAGE_POWER_LEVELS: [u8; 7] = [4, 8, 10, 12, 16, 18, 20];

/// A chaotic neutral Instinctual Warrior of `levels` levels, with Improved Unarmed Strike at 1 when `unarmed`.
fn instinctual_warrior(levels: usize, unarmed: bool) -> Build {
    let mut build = chaotic(archetype_build("barbarian", INSTINCTUAL_WARRIOR, levels));
    if unarmed {
        build
            .picks
            .insert(general_slot(1), feat_pick_plain("improved-unarmed-strike"));
    }
    build
}

/// Whether a monk power needing barbarian level `needs` is open in a slot at `level`: Deflect Arrows (`None`) from its
/// first rage power, the others from the level they need.
fn opens_at(needs: Option<u8>, level: u8) -> bool {
    needs.is_none_or(|needs| needs <= level)
}

/// "Besides the normal Barbarian rage powers, an Instinctual Warrior can select a number of Monk abilities and Ki
/// powers": five options of the barbarian's rage power selection, each needing the archetype and the level (or, for
/// Deflect Arrows, the feat) the guide gives. Extra Rage Power's "Rage Power" is met by holding any of them too (`option_lists.rs` checks its list).
#[test]
fn the_instinctual_warriors_monk_powers_are_rage_power_options_for_it_alone() {
    let archetype = Prerequisite::Archetype(ArchetypeKey::new(INSTINCTUAL_WARRIOR));
    for (key, name, level) in MONK_RAGE_POWERS {
        let option = selection_option(key);
        assert_eq!(option.name, name);
        assert_eq!(option.guide.0, "barbarian#rage-power-instinctual-warrior");
        assert_eq!(option.repeat, Repeat::Once, "{name}");
        let needs = match level {
            Some(level) => Prerequisite::ClassLevel {
                class: ClassKey::new("barbarian"),
                level,
            },
            None => Prerequisite::Feat(FeatKey::new("improved-unarmed-strike")),
        };
        let mut expected = Prerequisite::All(vec![archetype.clone(), needs]);
        expected.canonicalize();
        assert_eq!(option.prerequisite, Some(expected), "{name}");
    }
    let selection = class("barbarian")
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == RAGE_POWER)
        .expect("the rage power selection");
    assert_eq!(selection.options.len(), 38);
}

/// The ticket's: an Instinctual Warrior's rage power picker offers each monk power from its level and not before,
/// with a reason; Deflect Arrows needs Improved Unarmed Strike. One taken at its level is possible, one taken early
/// isn't, and the sheet shows it picked.
#[test]
fn an_instinctual_warrior_is_offered_each_monk_power_from_its_level_and_not_before() {
    let engine = engine();
    let build = instinctual_warrior(20, true);
    assert!(possible(&build));
    assert_eq!(
        levels_with_slot_of(&build, RAGE_POWER),
        INSTINCTUAL_RAGE_POWER_LEVELS
    );
    for level in INSTINCTUAL_RAGE_POWER_LEVELS {
        let open = open_at(&build, level);
        for (key, name, needs) in MONK_RAGE_POWERS {
            assert_eq!(
                open.contains(key),
                opens_at(needs, level),
                "{name} at {level}"
            );
        }
    }
    assert_eq!(
        lock_reason(&build, 4, MONK_RAGE_POWERS[1].0),
        "Needs Barbarian level 8 (have 4)"
    );
    assert_eq!(
        lock_reason(&build, 10, MONK_RAGE_POWERS[2].0),
        "Needs Barbarian level 12 (have 10)"
    );
    assert_eq!(
        lock_reason(&build, 12, MONK_RAGE_POWERS[3].0),
        "Needs Barbarian level 16 (have 12)"
    );
    assert_eq!(
        lock_reason(&build, 18, MONK_RAGE_POWERS[4].0),
        "Needs Barbarian level 20 (have 18)"
    );
    // Without Improved Unarmed Strike, Deflect Arrows is locked.
    let bare = instinctual_warrior(4, false);
    assert_eq!(
        lock_reason(&bare, 4, MONK_RAGE_POWERS[0].0),
        "Needs Improved Unarmed Strike"
    );

    // Each taken at its level: possible, and the sheet shows it in its slot.
    let mut taken = build.clone();
    for (slot, (key, _, _)) in [4, 8, 12, 16, 20].into_iter().zip(MONK_RAGE_POWERS) {
        taken.picks.insert(selection_slot(slot), pick(key));
    }
    assert!(possible(&taken));
    let sheet = engine.evaluate(&taken);
    let slot = sheet.slot(&selection_slot(8)).unwrap();
    assert_eq!(engine.slot_label(slot), "Rage Power");
    assert_eq!(
        engine.option_name(&SelectionOptionKey::new(MONK_RAGE_POWERS[1].0)),
        "Purity of Body (Instinctual Warrior)"
    );
    // A power taken is "Already taken" in a later slot.
    assert_eq!(
        lock_reason(&taken, 10, MONK_RAGE_POWERS[0].0),
        "Already taken"
    );

    // Taken before its level: impossible, for that reason.
    let mut early = instinctual_warrior(8, true);
    early
        .picks
        .insert(selection_slot(8), pick(MONK_RAGE_POWERS[2].0));
    assert_eq!(
        why_not(&early, &selection_slot(8)).and_then(|issue| engine.issue_text(&issue)),
        Some("Needs Barbarian level 12 (have 8)".to_owned())
    );
}

/// The ticket's: a barbarian without the archetype (plain, or another archetype) has them locked in every rage power
/// slot, needing the archetype (and the level, before it), and can't take one.
#[test]
fn a_barbarian_without_the_archetype_is_not_offered_the_monk_powers() {
    for archetype in [None, Some("barbarian/armored-hulk")] {
        let mut build = match archetype {
            Some(archetype) => chaotic(archetype_build("barbarian", archetype, 20)),
            None => barbarian(20),
        };
        build
            .picks
            .insert(general_slot(1), feat_pick_plain("improved-unarmed-strike"));
        assert!(possible(&build), "{archetype:?}");
        let levels = levels_with_slot_of(&build, RAGE_POWER);
        assert!(!levels.is_empty());
        for level in levels {
            let open = open_at(&build, level);
            for (key, name, needs) in MONK_RAGE_POWERS {
                assert!(!open.contains(key), "{name} at {level} ({archetype:?})");
                let reason = match needs {
                    Some(needs) if needs > level => format!(
                        "Needs Barbarian level {needs} (have {level}), the Instinctual Warrior archetype"
                    ),
                    _ => "Needs the Instinctual Warrior archetype".to_owned(),
                };
                assert_eq!(
                    lock_reason(&build, level, key),
                    reason,
                    "{name} at {level} ({archetype:?})"
                );
            }
        }
        let mut taken = build.clone();
        taken
            .picks
            .insert(selection_slot(20), pick(MONK_RAGE_POWERS[4].0));
        assert!(!possible(&taken), "{archetype:?}");
    }
}

/// The ticket's: Extra Rage Power gives an Instinctual Warrior a rage power slot offering the monk powers as its own
/// slots do, from their levels. Holding only a monk power meets the feat's "Rage Power".
#[test]
fn extra_rage_power_offers_an_instinctual_warrior_its_monk_powers() {
    let engine = engine();
    let mut build = instinctual_warrior(9, true);
    build
        .picks
        .insert(selection_slot(4), pick(MONK_RAGE_POWERS[0].0));
    build
        .picks
        .insert(general_slot(5), feat_pick_plain("extra-rage-power"));
    assert!(possible(&build));
    build
        .picks
        .insert(general_slot(9), feat_pick_plain("extra-rage-power"));
    assert!(possible(&build));
    for level in [5, 9] {
        assert_eq!(
            kind_at(&build, &extra_slot(level, 0)),
            Some(selection_kind(RAGE_POWER)),
            "at {level}"
        );
    }
    let keys = |slot: &SlotId| -> BTreeSet<String> {
        engine
            .options(&build, slot)
            .unwrap()
            .into_iter()
            .map(|option| option.pick.key().to_owned())
            .collect()
    };
    assert_eq!(keys(&extra_slot(9, 0)), keys(&selection_slot(8)));
    let open = |slot: &SlotId| -> BTreeSet<String> {
        engine
            .options(&build, slot)
            .unwrap()
            .into_iter()
            .filter(|option| option.status.is_ok())
            .map(|option| option.pick.key().to_owned())
            .collect()
    };
    // Deflect Arrows is taken at 4; Purity of Body opens at 8, so in the slot at 9 but not at 5.
    for (slot, level) in [(extra_slot(5, 0), 5), (extra_slot(9, 0), 9)] {
        let open = open(&slot);
        assert!(!open.contains(MONK_RAGE_POWERS[0].0), "at {level}");
        for (key, name, needs) in &MONK_RAGE_POWERS[1..] {
            assert_eq!(
                open.contains(*key),
                opens_at(*needs, level),
                "{name} at {level}"
            );
        }
    }
    let mut filled = build.clone();
    filled
        .picks
        .insert(extra_slot(9, 0), pick(MONK_RAGE_POWERS[1].0));
    let filled = with_selections_filled(filled);
    assert!(possible(&filled));

    // A plain barbarian's Extra Rage Power slot has them locked.
    let mut plain = with_selections_filled(barbarian(3));
    plain
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    let locked: BTreeSet<String> = engine
        .options(&plain, &extra_slot(3, 0))
        .unwrap()
        .into_iter()
        .filter(|option| option.status.is_err())
        .map(|option| option.pick.key().to_owned())
        .collect();
    for (key, name, _) in MONK_RAGE_POWERS {
        assert!(locked.contains(key), "{name}");
    }
}
