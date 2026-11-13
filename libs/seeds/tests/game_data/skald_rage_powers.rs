//! The skald's rage powers (ticket 189): a slot of the barbarian's `barbarian/rage-power` at each cell of the skald's
//! and its archetypes' tables, whose level prerequisites read skald levels too, and the Battle Scion's teamwork feat
//! in place of one.

use std::collections::BTreeSet;

use rules::{ArchetypeKey, ClassKey, Prerequisite, Repeat, SlotId};

use crate::support::*;

const RAGE_POWER: &str = "barbarian/rage-power";
const TEAMWORK_FEAT: &str = "barbarian/rage-power/teamwork-feat";
const BATTLE_SCION: &str = "skald/battle-scion";

/// The skald's and each archetype's Rage Power cells (`skald`: each table's "Rage Power" column; the Battle Scion's
/// "Battle Prowess" stands in the same cells, "whenever a Battle Scion can add a Rage Power to her song, she can
/// instead take any teamwork feat").
const TABLES: [(Option<&str>, &[u8]); 8] = [
    (None, &[3, 6, 9, 12, 15, 18]),
    (Some("skald/battle-scion"), &[3, 6, 9, 12, 15, 18]),
    (Some("skald/battle-singer"), &[6, 12, 18]),
    (Some("skald/court-poet"), &[]),
    (Some("skald/demon-dancer"), &[12, 15, 18]),
    (Some("skald/herald-of-the-horn"), &[3, 6, 9, 12, 15, 18]),
    (Some("skald/hunt-caller"), &[3, 9, 12, 15]),
    (Some("skald/inciter"), &[3, 6, 12, 18]),
];

/// The rage powers a level gates by barbarian level: (key, level), the Instinctual Warrior's five (which need the
/// archetype as well) not among them.
const LEVEL_GATED: [(&str, u8); 14] = [
    ("beast-totem", 6),
    ("beast-totem-greater", 10),
    ("clear-mind", 8),
    ("crippling-blows", 8),
    ("deadly-accuracy", 4),
    ("fearless-rage", 12),
    ("increased-damage-reduction", 8),
    ("internal-fortitude", 8),
    ("lethal-accuracy", 16),
    ("protect-vitals", 8),
    ("reflexive-dodge", 6),
    ("renewed-vigor", 4),
    ("rolling-dodge", 6),
    ("wrecking-blows", 12),
];

fn skald(archetype: Option<&str>, levels: usize) -> rules::Build {
    match archetype {
        Some(archetype) => archetype_build("skald", archetype, levels),
        None => build("skald", levels),
    }
}

fn rage_option(name: &str) -> String {
    format!("{RAGE_POWER}/{name}")
}

fn rage_power_selection() -> &'static rules::FeatureSelection {
    class("barbarian")
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == RAGE_POWER)
        .expect("the rage power selection")
}

/// Every cell of the skald's table, and of each archetype's, is a slot of the barbarian's rage powers; the rogue
/// talents stay where they were.
#[test]
fn the_skald_and_each_archetype_have_a_rage_power_slot_at_their_tables_levels() {
    for (archetype, levels) in TABLES {
        let build = skald(archetype, 20);
        assert!(possible(&build), "{archetype:?}");
        assert_eq!(
            levels_with_slot_of(&build, RAGE_POWER),
            levels,
            "{archetype:?}"
        );
        let talents: &[u8] = if archetype == Some("skald/inciter") {
            &[4, 9, 14, 19]
        } else {
            &[2, 7, 12, 17]
        };
        assert_eq!(
            levels_with_slot_of(&build, ROGUE_TALENTS),
            talents,
            "{archetype:?}"
        );
    }
}

/// The feature that stood for the picks goes, from the class, its archetypes and every row.
#[test]
fn the_skald_rage_powers_feature_is_gone() {
    let skald = class("skald");
    let feature = "skald/rage-powers";
    assert!(
        !skald
            .features
            .iter()
            .chain(skald.archetypes.iter().flat_map(|a| &a.features))
            .any(|f| f.key.as_str() == feature)
    );
    let on_a_row = skald
        .levels
        .iter()
        .map(|row| &row.features)
        .chain(
            skald
                .archetypes
                .iter()
                .flat_map(|a| a.levels.iter().map(|row| &row.features)),
        )
        .flatten()
        .any(|f| f.as_str() == feature);
    assert!(!on_a_row);
    // The Battle Scion's Battle Prowess stays on its six rows.
    let prowess = skald
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == BATTLE_SCION)
        .unwrap()
        .levels
        .iter()
        .filter(|row| {
            row.features
                .iter()
                .any(|f| f.as_str() == "skald/battle-scion/battle-prowess")
        })
        .map(|row| row.level)
        .collect::<Vec<_>>();
    assert_eq!(prowess, [3, 6, 9, 12, 15, 18]);
}

/// A rage power's level prerequisite reads skald levels as well as barbarian levels, in each of the 14 options the
/// guide gates by "Barbarian level", and in no other.
#[test]
fn a_rage_powers_level_prerequisite_reads_skald_levels_too() {
    let level = |class: &str, level: u8| Prerequisite::ClassLevel {
        class: ClassKey::new(class),
        level,
    };
    fn any_level(prerequisite: &Prerequisite, wanted: &Prerequisite) -> bool {
        match prerequisite {
            Prerequisite::All(children) => children.iter().any(|c| any_level(c, wanted)),
            other => other == wanted,
        }
    }
    for (name, needs) in LEVEL_GATED {
        let key = rage_option(name);
        let prerequisite = selection_option(&key).prerequisite.clone().unwrap();
        let mut either = Prerequisite::Any(vec![level("barbarian", needs), level("skald", needs)]);
        either.canonicalize();
        assert!(any_level(&prerequisite, &either), "{key}: {prerequisite:?}");
    }
    // Only those: the Instinctual Warrior's monk powers need the archetype as well and stay as they were.
    let skald_gated = rage_power_selection()
        .options
        .iter()
        .filter(|option| format!("{:?}", option.prerequisite).contains("\"skald\""))
        .count();
    assert_eq!(skald_gated, LEVEL_GATED.len());
}

/// A skald's picker opens each level-gated power at its skald level, a barbarian's at its barbarian level, and the
/// Instinctual Warrior's monk powers and the Battle Scion's teamwork feat stay locked.
#[test]
fn a_skalds_rage_powers_open_at_skald_levels() {
    let build = skald(None, 20);
    for (name, needs) in LEVEL_GATED {
        let key = rage_option(name);
        for slot in [3, 6, 9, 12, 15, 18] {
            let open = open_at(&build, slot);
            let totem = matches!(name, "beast-totem" | "beast-totem-greater");
            let stance = matches!(
                name,
                "deadly-accuracy"
                    | "lethal-accuracy"
                    | "crippling-blows"
                    | "wrecking-blows"
                    | "protect-vitals"
                    | "reflexive-dodge"
                    | "rolling-dodge"
            );
            if totem || stance {
                // These also need the power they follow, held by nobody here.
                assert!(!open.contains(&key), "{name} at {slot}");
            } else {
                assert_eq!(open.contains(&key), slot >= needs, "{name} at {slot}");
            }
        }
    }
    let reason = lock_reason(&build, 3, &rage_option("renewed-vigor"));
    assert!(reason.contains("Skald level 4"), "{reason}");
    for slot in [3, 6, 9, 12, 15, 18] {
        for monk in [
            "deflect-arrows-instinctual-warrior",
            "purity-of-body-instinctual-warrior",
            "perfect-self-instinctual-warrior",
        ] {
            assert!(!open_at(&build, slot).contains(&rage_option(monk)));
        }
        let reason = lock_reason(&build, slot, TEAMWORK_FEAT);
        assert!(reason.contains("Battle Scion"), "{reason}");
    }
    // A chain through the levels is possible: Lethal Stance, Deadly Accuracy, then Lethal Accuracy at 16.
    let chain = with_picks(
        build,
        &[
            (3, pick(&rage_option("lethal-stance"))),
            (6, pick(&rage_option("deadly-accuracy"))),
            (18, pick(&rage_option("lethal-accuracy"))),
        ],
    );
    assert!(possible(&chain));
    // Lethal Accuracy needs skald level 16: locked in the slot at 15.
    let reason = lock_reason(&chain, 15, &rage_option("lethal-accuracy"));
    assert!(reason.contains("Skald level 16"), "{reason}");
}

/// The barbarian's own picker is as before: a level-gated power opens at the barbarian's level, and the teamwork feat
/// is locked for the barbarian.
#[test]
fn a_barbarians_rage_powers_are_as_before() {
    let build = barbarian(12);
    let fearless = rage_option("fearless-rage");
    assert!(!open_at(&build, 10).contains(&fearless));
    assert!(open_at(&build, 12).contains(&fearless));
    for level in levels_with_slot_of(&build, RAGE_POWER) {
        assert!(!open_at(&build, level).contains(TEAMWORK_FEAT), "{level}");
    }
    // Other classes have no such slot.
    for class in ["bard", "rogue", "fighter"] {
        assert!(levels_with_slot_of(&self::build(class, 20), RAGE_POWER).is_empty());
    }
}

/// Skald levels and barbarian levels do not stack (the user's decision): each class's level alone is read.
#[test]
fn barbarian_and_skald_levels_do_not_stack() {
    // Barbarian 4 / skald 6: ten levels in all, but neither class has 8.
    let build = multiclass(&[("barbarian", None, 4), ("skald", None, 6)]);
    assert!(possible(&build));
    let skald_6 = SlotId::Selection {
        level: 10,
        index: 0,
    };
    assert_eq!(kind_at(&build, &skald_6), Some(selection_kind(RAGE_POWER)));
    let open = offered_in(&build, &skald_6);
    assert!(open[&rage_option("clear-mind")].is_err());
    let reason = reason_in(&build, &skald_6, &rage_option("clear-mind"));
    assert!(reason.contains("Skald level 8"), "{reason}");
    // The barbarian's four levels do open Renewed Vigor in a skald's slot: either class's level will do.
    assert!(open[&rage_option("renewed-vigor")].is_ok());
    // A skald 8 reads its own levels: Renewed Vigor (4) is open at 6, Clear Mind (8) is not.
    let skald_8 = multiclass(&[("skald", None, 8)]);
    assert!(open_at(&skald_8, 6).contains(&rage_option("renewed-vigor")));
    assert!(!open_at(&skald_8, 6).contains(&rage_option("clear-mind")));
}

/// A rage power is taken once across the barbarian's and the skald's slots.
#[test]
fn a_rage_power_is_taken_once_across_barbarian_and_skald() {
    let animal_fury = rage_option("animal-fury");
    let build = multiclass(&[("barbarian", None, 2), ("skald", None, 3)]);
    let barbarian_slot = selection_slot(2);
    let skald_slot = SlotId::Selection { level: 5, index: 0 };
    let build = with_picks(build, &[(2, pick(&animal_fury))]);
    assert_eq!(
        kind_at(&build, &skald_slot),
        Some(selection_kind(RAGE_POWER))
    );
    assert_eq!(
        reason_in(&build, &skald_slot, &animal_fury),
        "Already taken"
    );
    assert!(offered_in(&build, &barbarian_slot).contains_key(&animal_fury));
}

/// Ticket 49's rule: Extra Rage Power gives a skald, once it holds a rage power, one more slot of the barbarian's
/// rage powers.
#[test]
fn extra_rage_power_reaches_a_skald() {
    let animal_fury = rage_option("animal-fury");
    let empty = skald(None, 3);
    assert!(feat_status(&empty, &general_slot(3), "extra-rage-power").is_err());
    let mut build = with_picks(skald(None, 3), &[(3, pick(&animal_fury))]);
    assert_eq!(
        feat_status(&build, &general_slot(3), "extra-rage-power"),
        Ok(())
    );
    build
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    assert!(possible(&build));
    let slot = extra_slot(3, 0);
    assert_eq!(kind_at(&build, &slot), Some(selection_kind(RAGE_POWER)));
    assert_eq!(
        offered_in(&build, &slot).keys().collect::<BTreeSet<_>>(),
        offered_in(&build, &selection_slot(3))
            .keys()
            .collect::<BTreeSet<_>>()
    );
}

/// The Battle Scion's teamwork feat is one option of the rage power selection that only the Battle Scion may take,
/// once per slot and any number of times, whose row gives a Teamwork feat slot; it isn't a rage power.
#[test]
fn the_battle_scions_teamwork_feat_is_an_option_of_the_rage_power_selection() {
    let selection = rage_power_selection();
    let option = selection
        .options
        .iter()
        .find(|o| o.key.as_str() == TEAMWORK_FEAT)
        .expect("the teamwork feat option");
    assert_eq!(option.name, "Teamwork Feat");
    assert_eq!(option.repeat, Repeat::Unlimited);
    assert_eq!(
        option.prerequisite,
        Some(Prerequisite::Archetype(ArchetypeKey::new(BATTLE_SCION)))
    );
    assert_eq!(selection.options.len(), 39);
    assert_eq!(
        selection
            .option_levels
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>(),
        ["skald"]
    );
    // A teamwork feat isn't a rage power: Extra Rage Power's prerequisite doesn't name it.
    let Some(Prerequisite::Any(children)) = &feat("extra-rage-power").prerequisite else {
        panic!("Extra Rage Power's prerequisite isn't an Any");
    };
    assert!(children.iter().all(|child| match child {
        Prerequisite::SelectionOption(key) => key.as_str() != TEAMWORK_FEAT,
        _ => false,
    }));
}

/// A Battle Scion takes a teamwork feat in place of a rage power at each of the six slots: it gets a Teamwork feat
/// slot at the level, and no second rage power slot.
#[test]
fn a_battle_scion_takes_a_teamwork_feat_in_place_of_a_rage_power() {
    let scion = skald(Some(BATTLE_SCION), 20);
    for level in [3, 6, 9, 12, 15, 18] {
        let open = open_at(&scion, level);
        assert!(open.contains(TEAMWORK_FEAT), "{level}");
        assert!(open.contains(&rage_option("animal-fury")), "{level}");
    }
    let picks: Vec<_> = [3u8, 6, 9, 12, 15, 18]
        .into_iter()
        .map(|level| (level, pick(TEAMWORK_FEAT)))
        .collect();
    let taken = with_picks(scion.clone(), &picks);
    assert!(possible(&taken));
    for level in [3, 6, 9, 12, 15, 18] {
        let slot = SlotId::OptionFeat { level, index: 0 };
        let teamwork = feats_offered(&taken, &slot);
        assert!(teamwork.contains("outflank"), "{level}");
        assert!(!teamwork.contains("power-attack"), "{level}");
    }
    // One slot of the selection at each level: the teamwork feat fills the rage power's.
    assert_eq!(
        levels_with_slot_of(&taken, RAGE_POWER),
        [3, 6, 9, 12, 15, 18]
    );
    // A rage power in its place gives no feat slot.
    let power = with_picks(scion, &[(3, pick(&rage_option("animal-fury")))]);
    assert!(possible(&power));
    assert_eq!(
        kind_at(&power, &SlotId::OptionFeat { level: 3, index: 0 }),
        None
    );
    // Extra Rage Power reaches a Battle Scion with a rage power, and its slot offers the teamwork feat too.
    let mut extra = with_picks(
        skald(Some(BATTLE_SCION), 3),
        &[(3, pick(&rage_option("animal-fury")))],
    );
    extra
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    assert!(possible(&extra));
    assert!(offered_in(&extra, &extra_slot(3, 0)).contains_key(TEAMWORK_FEAT));
    assert!(offered_in(&extra, &extra_slot(3, 0))[TEAMWORK_FEAT].is_ok());
}

/// Nobody else is offered it, and an archetype's slot is the same selection's: the Battle Scion's slot only differs
/// by that option.
#[test]
fn only_the_battle_scion_may_take_the_teamwork_feat() {
    for (archetype, levels) in TABLES {
        let Some(&level) = levels.first() else {
            continue;
        };
        let build = skald(archetype, 20);
        let reason = lock_reason_or_none(&build, level);
        if archetype == Some(BATTLE_SCION) {
            assert_eq!(reason, None);
        } else {
            assert!(reason.unwrap().contains("Battle Scion"), "{archetype:?}");
        }
    }
}

fn lock_reason_or_none(build: &rules::Build, level: u8) -> Option<String> {
    let status = offered(build, level).remove(TEAMWORK_FEAT)?;
    status
        .err()
        .map(|issue| engine().issue_text(&issue).unwrap())
}

/// The rogue talent slot of a skald 12 follows the rage power slot: the row's selections are sorted by key
/// (the user accepted the move from index 0 to 1).
#[test]
fn the_rogue_talent_slot_at_skald_12_follows_the_rage_power_slot() {
    for archetype in [None, Some("skald/demon-dancer"), Some(BATTLE_SCION)] {
        let build = skald(archetype, 12);
        assert_eq!(
            kind_at(
                &build,
                &SlotId::Selection {
                    level: 12,
                    index: 0
                }
            ),
            Some(selection_kind(RAGE_POWER)),
            "{archetype:?}"
        );
        assert_eq!(
            kind_at(
                &build,
                &SlotId::Selection {
                    level: 12,
                    index: 1
                }
            ),
            Some(selection_kind(ROGUE_TALENTS)),
            "{archetype:?}"
        );
    }
    // The Inciter's 12 has a rage power and no rogue talent; its talents are at 4, 9, 14 and 19.
    let inciter = skald(Some("skald/inciter"), 12);
    assert_eq!(
        kind_at(
            &inciter,
            &SlotId::Selection {
                level: 12,
                index: 0
            }
        ),
        Some(selection_kind(RAGE_POWER))
    );
    assert_eq!(
        kind_at(
            &inciter,
            &SlotId::Selection {
                level: 12,
                index: 1
            }
        ),
        None
    );
}
