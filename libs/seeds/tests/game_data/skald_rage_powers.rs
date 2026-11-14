//! The skald's rage powers (ticket 189): a slot of the barbarian's `barbarian/rage-power` at each cell of the skald's
//! and its archetypes' tables, whose level prerequisites read skald levels too, and the Battle Scion's own selection,
//! which offers a teamwork feat in place of a rage power.

use std::collections::BTreeSet;

use rules::{ClassKey, Prerequisite, Repeat, SlotId};

use crate::support::*;

const RAGE_POWER: &str = "barbarian/rage-power";
const BATTLE_SCION: &str = "skald/battle-scion";
/// The Battle Scion's slot in the Rage Power cells: a rage power or a teamwork feat.
const SCION_SLOT: &str = "skald/battle-scion/rage-power-or-teamwork-feat";
const TEAMWORK_FEAT: &str = "skald/battle-scion/rage-power-or-teamwork-feat/teamwork-feat";

/// The Battle Scion's Rage Power cells: "Battle Prowess" stands in the skald's six.
const SCION_LEVELS: [u8; 6] = [3, 6, 9, 12, 15, 18];

/// The slots of `barbarian/rage-power` each archetype has, by each table's "Rage Power" column. The Battle Scion has
/// none: its "Battle Prowess" cells are slots of its own selection, `SCION_SLOT`.
const TABLES: [(Option<&str>, &[u8]); 8] = [
    (None, &[3, 6, 9, 12, 15, 18]),
    (Some(BATTLE_SCION), &[]),
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
        let scion: &[u8] = if archetype == Some(BATTLE_SCION) {
            &SCION_LEVELS
        } else {
            &[]
        };
        assert_eq!(
            levels_with_slot_of(&build, SCION_SLOT),
            scion,
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
        assert!(!offered(&build, slot).contains_key(TEAMWORK_FEAT), "{slot}");
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
    // A plain barbarian's reason names the barbarian only.
    assert_eq!(
        lock_reason(&barbarian(4), 4, &rage_option("clear-mind")),
        "Needs Barbarian level 8 (have 4)"
    );
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

/// The rage power selection is the barbarian's alone: no option level, no teamwork feat, 38 options. The Battle
/// Scion's selection shares every one of them (read from the data, so a new rage power joins it or fails here) and
/// has one option of its own, the repeatable Teamwork Feat, whose row gives a Teamwork feat slot.
#[test]
fn the_battle_scions_selection_shares_every_rage_power_and_adds_a_teamwork_feat() {
    let rage = rage_power_selection();
    assert!(rage.option_levels.is_empty());
    assert_eq!(rage.options.len(), 38);
    let rage_keys: BTreeSet<String> = rage.options.iter().map(|o| o.key.to_string()).collect();
    let selection = any_selection(SCION_SLOT);
    assert_eq!(selection.name, "Rage Power or Teamwork Feat");
    let shared: BTreeSet<String> = selection
        .shared_options
        .iter()
        .map(|k| k.to_string())
        .collect();
    assert_eq!(shared, rage_keys);
    let [option] = selection.options.as_slice() else {
        panic!("the Battle Scion's selection has one option of its own");
    };
    assert_eq!(option.key.as_str(), TEAMWORK_FEAT);
    assert_eq!(option.name, "Teamwork Feat");
    assert_eq!(option.repeat, Repeat::Unlimited);
    assert_eq!(option.prerequisite, None);
    assert_eq!(
        selection
            .option_levels
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>(),
        ["skald"]
    );
    // Extra Rage Power gives a Battle Scion a slot of this selection (that its prerequisite leaves out
    // the Teamwork Feat is option_lists.rs's exception table).
    let extra = &feat("extra-rage-power").extra_pick_of;
    assert!(extra.iter().any(|s| s.as_str() == SCION_SLOT));
}

fn any_selection(key: &str) -> &'static rules::FeatureSelection {
    DATA.selections()
        .find(|selection| selection.key.as_str() == key)
        .unwrap_or_else(|| panic!("no selection {key}"))
}

/// A Battle Scion takes a teamwork feat in place of a rage power at each of the six slots: it gets a Teamwork feat
/// slot at the level, and no second rage power slot.
#[test]
fn a_battle_scion_takes_a_teamwork_feat_in_place_of_a_rage_power() {
    let scion = skald(Some(BATTLE_SCION), 20);
    for level in SCION_LEVELS {
        let slot = slot_of(&scion, level, SCION_SLOT);
        let open: BTreeSet<String> = offered_in(&scion, &slot)
            .into_iter()
            .filter(|(_, status)| status.is_ok())
            .map(|(key, _)| key)
            .collect();
        assert!(open.contains(TEAMWORK_FEAT), "{level}");
        assert!(open.contains(&rage_option("animal-fury")), "{level}");
    }
    let picks: Vec<(SlotId, rules::Pick)> = SCION_LEVELS
        .into_iter()
        .map(|level| (slot_of(&scion, level, SCION_SLOT), pick(TEAMWORK_FEAT)))
        .collect();
    let mut taken = scion.clone();
    taken.picks.extend(picks);
    assert!(possible(&taken));
    for level in SCION_LEVELS {
        let slot = SlotId::OptionFeat { level, index: 0 };
        let teamwork = feats_offered(&taken, &slot);
        assert!(teamwork.contains("outflank"), "{level}");
        assert!(!teamwork.contains("power-attack"), "{level}");
    }
    // One slot at each level: the teamwork feat fills the rage power's.
    assert_eq!(levels_with_slot_of(&taken, SCION_SLOT), SCION_LEVELS);
    // A rage power in its place gives no feat slot.
    let mut power = scion.clone();
    power.picks.insert(
        slot_of(&scion, 3, SCION_SLOT),
        pick(&rage_option("animal-fury")),
    );
    assert!(possible(&power));
    assert_eq!(
        kind_at(&power, &SlotId::OptionFeat { level: 3, index: 0 }),
        None
    );
    // Extra Rage Power gives a Battle Scion with a rage power one more slot of its own selection, which offers the
    // teamwork feat too.
    let mut extra = skald(Some(BATTLE_SCION), 3);
    extra.picks.insert(
        slot_of(&extra, 3, SCION_SLOT),
        pick(&rage_option("animal-fury")),
    );
    extra
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    assert!(possible(&extra));
    let slot = extra_slot(3, 0);
    assert_eq!(kind_at(&extra, &slot), Some(selection_kind(SCION_SLOT)));
    assert!(offered_in(&extra, &slot)[TEAMWORK_FEAT].is_ok());
}

/// The teamwork feat is offered in the Battle Scion's own slots only: not in a barbarian's slot, even for a Battle Scion
/// with barbarian levels, and a rage power is taken once across both.
#[test]
fn a_battle_scion_with_barbarian_levels_is_offered_no_teamwork_feat_in_a_barbarian_slot() {
    let animal_fury = rage_option("animal-fury");
    let build = multiclass(&[("barbarian", None, 2), ("skald", Some(BATTLE_SCION), 3)]);
    let barbarian_slot = selection_slot(2);
    assert_eq!(
        kind_at(&build, &barbarian_slot),
        Some(selection_kind(RAGE_POWER))
    );
    assert!(!offered_in(&build, &barbarian_slot).contains_key(TEAMWORK_FEAT));
    let scion_slot = slot_of(&build, 5, SCION_SLOT);
    assert!(offered_in(&build, &scion_slot)[TEAMWORK_FEAT].is_ok());
    // A rage power taken in the barbarian's slot is "Already taken" in the Battle Scion's.
    let build = with_picks(build, &[(2, pick(&animal_fury))]);
    assert_eq!(
        reason_in(&build, &scion_slot, &animal_fury),
        "Already taken"
    );
    // No other archetype has the option in any slot.
    for (archetype, levels) in TABLES {
        if let Some(&level) = levels.first() {
            assert!(!offered(&skald(archetype, 20), level).contains_key(TEAMWORK_FEAT));
        }
    }
}

/// The rogue talent slot of a skald 12 follows the rage power slot: a row's selections are sorted by key, so the
/// rage power slot is index 0 and the rogue talents' index 1 (the user accepted the move from 0). The Battle Scion's
/// own selection sorts after the rogue talents, so its slot at 12 is index 1 and the talents stay at 0.
#[test]
fn the_rogue_talent_slot_at_skald_12_follows_the_rage_power_slot() {
    for archetype in [None, Some("skald/demon-dancer")] {
        let build = skald(archetype, 12);
        let kind = |index| kind_at(&build, &SlotId::Selection { level: 12, index });
        assert_eq!(kind(0), Some(selection_kind(RAGE_POWER)), "{archetype:?}");
        assert_eq!(
            kind(1),
            Some(selection_kind(ROGUE_TALENTS)),
            "{archetype:?}"
        );
    }
    let scion = skald(Some(BATTLE_SCION), 12);
    let kind = |index| kind_at(&scion, &SlotId::Selection { level: 12, index });
    assert_eq!(kind(0), Some(selection_kind(ROGUE_TALENTS)));
    assert_eq!(kind(1), Some(selection_kind(SCION_SLOT)));
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

/// What a locked level-gated power says: a barbarian's picker reads the barbarian's level only, and so does a
/// skald's its own. A Primalist, who holds neither class's levels, is told of both, as is the prerequisite as the
/// data states it, as ticket 25's shared gates do.
#[test]
fn a_locked_level_gated_powers_texts_name_the_classes_that_can_open_it() {
    let clear_mind = rage_option("clear-mind");
    let raw = |build: &rules::Build, slot: &SlotId| {
        let status = offered_in(build, slot).remove(&clear_mind).unwrap();
        let Err(rules::Issue::Unmet(unmet)) = status else {
            panic!("not an unmet prerequisite: {status:?}");
        };
        engine().unmet_text(&unmet)
    };
    let barbarian = barbarian(4);
    assert_eq!(
        raw(&barbarian, &selection_slot(4)),
        "Barbarian level 8 (have 4)"
    );
    let skald = skald(None, 3);
    assert_eq!(raw(&skald, &selection_slot(3)), "Skald level 8 (have 3)");
    assert_eq!(
        reason_in(&skald, &selection_slot(3), &clear_mind),
        "Needs Skald level 8 (have 3)"
    );
    // The prerequisite as the data states it (the MCP server's search and describe) names both.
    assert_eq!(
        engine().prerequisite_text(
            selection_option(&clear_mind).prerequisite.as_ref().unwrap(),
            None
        ),
        "Barbarian level 8 or Skald level 8"
    );
    // A Primalist takes rage powers through its primal choices, with neither class's levels.
    let mut primalist = with_option(
        archetype_build("bloodrager", "bloodrager/primalist", 4),
        1,
        0,
        "bloodrager/bloodline/infernal",
    );
    let choice = slot_of(&primalist, 4, "bloodrager/primalist/primal-choices");
    primalist = with_pick(
        primalist,
        choice,
        "bloodrager/primalist/primal-choices/rage-powers",
    );
    let slot = engine()
        .evaluate(&primalist)
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .find(|slot| slot.kind == selection_kind(RAGE_POWER))
        .expect("a rage power slot")
        .id
        .clone();
    assert_eq!(
        reason_in(&primalist, &slot, &clear_mind),
        "Needs Barbarian level 8 (have 0) or Skald level 8 (have 0)"
    );
}
