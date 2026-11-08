//! Monk ki powers (ticket 58).

use std::collections::BTreeSet;

use rules::{Build, ClassKey, FeatureKey, Prerequisite};

use crate::support::*;

const KI_POWER: &str = "monk/ki-power";

/// The guide's ki powers (`monk#ki-powers`): each one's key and the monk level it unlocks at.
const KI_POWERS: [(&str, u8); 14] = [
    ("barkskin", 4),
    ("scorching-ray", 4),
    ("sudden-speed", 4),
    ("true-strike", 4),
    ("abundant-step", 8),
    ("poison", 8),
    ("restoration", 8),
    ("wholeness-of-body", 8),
    ("spit-venom", 10),
    ("diamond-body", 12),
    ("cold-ice-strike", 14),
    ("diamond-soul", 14),
    ("shout", 14),
    ("quivering-palm", 16),
];

fn ki_power(name: &str) -> String {
    format!("{KI_POWER}/{name}")
}

/// A monk of `archetype`, `levels` levels, with no picks: lawful neutral, but chaotic neutral for the Drunken Master
/// ("Cannot be Lawful"), and an oread for the Student of Stone ("Race must be Oread").
fn monk_of(archetype: &str, levels: usize) -> Build {
    let mut build = archetype_build("monk", archetype, levels);
    let alignment = match archetype {
        "monk/drunken-master" => rules::Alignment::ChaoticNeutral,
        _ => rules::Alignment::LawfulNeutral,
    };
    align(&mut build, alignment);
    if archetype == "monk/student-of-stone" {
        build.start.race = Some(rules::RaceKey::new("oread"));
    }
    build
}

/// "At 4th level and every 2 levels thereafter a Monk may select a ki power": the guide's 14 ki powers, each needing
/// the monk level its table gives; no feature stands for the pick any more, and no feat gives one.
#[test]
fn ki_powers_are_a_selection_of_14_with_their_level() {
    let monk = class("monk");
    let selection = monk
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == KI_POWER)
        .expect("the ki power selection");
    assert_eq!(selection.name, "Ki Power");
    let keys: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    let expected: BTreeSet<String> = KI_POWERS.iter().map(|(name, _)| ki_power(name)).collect();
    assert_eq!(keys, expected);
    for (name, level) in KI_POWERS {
        assert_eq!(
            selection_option(&ki_power(name)).prerequisite,
            Some(Prerequisite::ClassLevel {
                class: ClassKey::new("monk"),
                level,
            }),
            "{name}"
        );
    }
    assert!(
        monk.features
            .iter()
            .all(|feature| feature.key.as_str() != KI_POWER)
    );
    let feature = FeatureKey::new(KI_POWER);
    assert!(
        monk.levels
            .iter()
            .all(|row| !row.features.contains(&feature))
            && monk
                .archetypes
                .iter()
                .flat_map(|a| &a.levels)
                .all(|row| !row.features.contains(&feature))
    );
    let levels: Vec<u8> = monk
        .levels
        .iter()
        .filter(|row| row.selections.iter().any(|s| s.as_str() == KI_POWER))
        .map(|row| row.level)
        .collect();
    assert_eq!(levels, [4, 6, 8, 10, 12, 14, 16, 18, 20]);
    // Extra Ki and Abundant Ki Pool grow the pool; no feat gives a ki power.
    assert!(
        DATA.feats
            .iter()
            .all(|feat| feat.extra_pick_of.iter().all(|s| s.as_str() != KI_POWER))
    );
}

/// The ticket's: each monk archetype picks a ki power where its table has "Ki Power": the Scaled Fist not at 12
/// (Dragon Breath), the Sohei from 6, the Student of Stone not at 6 or 12, the Zen Archer not at 10, and the
/// Traditional Monk never (its ki powers are fixed). Filled, each Build is possible, the Drunken Master's too since
/// its "Cannot be Lawful" replaces the monk's "Lawful" (ticket 63).
#[test]
fn every_monk_archetype_picks_a_ki_power_at_its_tables_levels() {
    let every = vec![4, 6, 8, 10, 12, 14, 16, 18, 20];
    assert_eq!(levels_with_slot_of(&monk(20), KI_POWER), every);
    for (archetype, levels) in [
        ("monk/drunken-master", every.clone()),
        ("monk/quarterstaff-master", every.clone()),
        ("monk/scaled-fist", vec![4, 6, 8, 10, 14, 16, 18, 20]),
        ("monk/sensei", every.clone()),
        ("monk/sohei", vec![6, 8, 10, 12, 14, 16, 18, 20]),
        ("monk/student-of-stone", vec![4, 8, 10, 14, 16, 18, 20]),
        ("monk/traditional-monk", vec![]),
        ("monk/zen-archer", vec![4, 6, 8, 12, 14, 16, 18, 20]),
    ] {
        let build = monk_of(archetype, 20);
        assert_eq!(levels_with_slot_of(&build, KI_POWER), levels, "{archetype}");
        assert!(possible(&build), "{archetype}");
        let filled = with_selections_filled(build);
        assert!(possible(&filled), "{archetype} filled");
    }
    assert_eq!(class("monk").archetypes.len(), 8);
}

/// The ticket's: a monk 4 has one ki power slot, "Ki Power", offering the 14 and leaving open only the four of level
/// 4. A monk 3 has none.
#[test]
fn a_monk_4_has_one_ki_power_slot_offering_only_the_powers_of_level_4() {
    let engine = engine();
    let build = monk(4);
    assert_eq!(levels_with_slot_of(&build, KI_POWER), [4]);
    let open: BTreeSet<String> = KI_POWERS
        .iter()
        .filter(|(_, level)| *level <= 4)
        .map(|(name, _)| ki_power(name))
        .collect();
    assert_eq!(open.len(), 4);
    assert_eq!(open_at(&build, 4), open);
    assert_eq!(offered(&build, 4).len(), 14);
    let sheet = engine.evaluate(&build);
    assert_eq!(
        engine.slot_label(sheet.slot(&selection_slot(4)).unwrap()),
        "Ki Power"
    );
    let mut picked = build.clone();
    picked
        .picks
        .insert(selection_slot(4), pick(&ki_power("sudden-speed")));
    assert!(possible(&picked));
    assert_eq!(levels_with_slot_of(&monk(3), KI_POWER), Vec::<u8>::new());
}

/// The ticket's: a ki power of a higher level is locked with a reason and opens at its level; one taken isn't offered
/// again.
#[test]
fn a_higher_level_ki_power_is_locked_with_a_reason_and_opens_at_its_level() {
    let build = monk(16);
    assert_eq!(
        lock_reason(&build, 4, &ki_power("abundant-step")),
        "Needs Monk level 8 (have 4)"
    );
    assert_eq!(
        lock_reason(&build, 6, &ki_power("abundant-step")),
        "Needs Monk level 8 (have 6)"
    );
    assert!(open_at(&build, 8).contains(&ki_power("abundant-step")));
    assert_eq!(
        lock_reason(&build, 14, &ki_power("quivering-palm")),
        "Needs Monk level 16 (have 14)"
    );
    assert!(open_at(&build, 16).contains(&ki_power("quivering-palm")));
    // Each opens at its own level.
    for (name, level) in KI_POWERS {
        for at in (4..=16).step_by(2) {
            assert_eq!(
                open_at(&build, at).contains(&ki_power(name)),
                level <= at,
                "{name} at {at}"
            );
        }
    }

    let mut early = monk(4);
    early
        .picks
        .insert(selection_slot(4), pick(&ki_power("diamond-body")));
    assert_eq!(
        why_not(&early, &selection_slot(4)).and_then(|issue| engine().issue_text(&issue)),
        Some("Needs Monk level 12 (have 4)".to_owned())
    );

    let mut taken = monk(6);
    taken
        .picks
        .insert(selection_slot(4), pick(&ki_power("barkskin")));
    assert_eq!(
        lock_reason(&taken, 6, &ki_power("barkskin")),
        "Already taken"
    );
    let mut twice = taken.clone();
    twice
        .picks
        .insert(selection_slot(6), pick(&ki_power("barkskin")));
    assert!(!possible(&twice));
}

/// "Gains fixed Ki Powers": a Traditional Monk picks no ki power, and holds the five its table names from their
/// levels (Wholeness of Body at 6, Abundant Step at 8, Diamond Body at 10, Diamond Soul at 12, Quivering Palm at
/// 14), each a feature of its own.
#[test]
fn a_traditional_monk_gains_its_fixed_ki_powers_and_picks_none() {
    let build = monk_of("monk/traditional-monk", 20);
    assert!(possible(&build));
    assert_eq!(levels_with_slot_of(&build, KI_POWER), Vec::<u8>::new());
    let sheet = engine().evaluate(&build);
    for (feature, from) in [
        ("monk/traditional-monk/ki-power-wholeness-of-body", 6),
        ("monk/traditional-monk/ki-power-abundant-step", 8),
        ("monk/traditional-monk/ki-power-diamond-body", 10),
        ("monk/traditional-monk/ki-power-diamond-soul", 12),
        ("monk/traditional-monk/ki-power-quivering-palm", 14),
    ] {
        assert!(!holds(sheet.at(from - 1).unwrap(), feature), "{feature}");
        assert!(holds(sheet.at(from).unwrap(), feature), "{feature}");
        assert!(holds(sheet.at(20).unwrap(), feature), "{feature}");
    }
    // The rest of those rows is the base class's: the bonus feats at 6, 10 and 14, Stunning Fist: Sicken at 8 and
    // Ki Strike - Lawful at 10.
    assert!(holds(sheet.at(8).unwrap(), "monk/stunning-fist-sicken"));
    assert!(holds(sheet.at(10).unwrap(), "monk/ki-strike-lawful"));
    let traditional = class("monk")
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == "monk/traditional-monk")
        .unwrap();
    for level in [6, 10, 14] {
        let row = rules::effective_row(class("monk"), Some(traditional), level).unwrap();
        assert_eq!(row.feat_slots.len(), 1, "{level}");
    }
}
