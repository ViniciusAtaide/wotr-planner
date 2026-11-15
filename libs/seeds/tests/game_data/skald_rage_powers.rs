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
    // Game: SkaldProgression 26418fed2bc153245972a5b54204ed75: SkaldRagePowerSelection
    // 2476514e31791394fa140f1a07941c96, the base rage powers (equivalent model: the barbarian's selection)
    (None, &[3, 6, 9, 12, 15, 18]),
    // Game: BattleScionArchetype 71d41dd6309ea994886908c34c87ccd9
    (Some(BATTLE_SCION), &[]),
    // Game: BattleSingerArchetype db942e7040084ea289c6be2bfeb3d0a8
    (Some("skald/battle-singer"), &[6, 12, 18]),
    // Game: CourtPoetArchetype 5e63586bebd229649bdafe0fde4caaec
    (Some("skald/court-poet"), &[]),
    // Game: DemonDancerArchetype 9c46c4e4303e5da40b63faa6df18e2c2
    (Some("skald/demon-dancer"), &[12, 15, 18]),
    // Game: HeraldOfTheHornArchetype 72fa7b1c7a6ede44eb490080bf4a2f90
    (Some("skald/herald-of-the-horn"), &[3, 6, 9, 12, 15, 18]),
    // Game: HuntCallerArchetype 67daa4f4f2b62f54cb3ec56bdc85dea6
    (Some("skald/hunt-caller"), &[3, 9, 12, 15]),
    // Game: ProvocateurArchetype a1f5ab7eb78942f6889304acbd92e945
    (Some("skald/inciter"), &[3, 6, 12, 18]),
];

/// The rage powers a level gates by barbarian level: (key, level), the Instinctual Warrior's five (which need the
/// archetype as well) not among them.
const LEVEL_GATED: [(&str, u8); 14] = [
    // Game: BeastTotemFeature c085888db293f2741b881cc989a2ab14
    ("beast-totem", 6),
    // Game: BeastTotemGreaterFeature 602d2684add008b4ba748af95c3f8b2e
    ("beast-totem-greater", 10),
    // Game: ClearMindFeature 4a236ae23d5a4f76b3616ddfa337e724
    ("clear-mind", 8),
    // Game: CripplingBlowsFeature 0eec6efbb7f66e148817c9f51b804f08
    ("crippling-blows", 8),
    // Game: DeadlyAccuracyFeature c841ffa13d39ce442a408f57feb3cb8e
    ("deadly-accuracy", 4),
    // Game: FearlessRageFeature 2f59c85adec695b418a66242f2ba08b8
    ("fearless-rage", 12),
    // Game: IncreasedDamageReductionFeature ddaee203ee4dcb24c880d633fbd77db6
    ("increased-damage-reduction", 8),
    // Game: InternalFortitudeFeature c6ceaa9bb284fae4c908658a05aa5f4d
    ("internal-fortitude", 8),
    // Game: LethalAccuracyFeature bb79cb9706379934c9460e46fe8cd04e
    ("lethal-accuracy", 16),
    // Game: ProtectVitalsFeature efd53fe2887c3a54d86b99f4bba61dd6
    ("protect-vitals", 8),
    // Game: ReflexiveDodgeFeature 6a9319f42f742024680af38af54f5d6f
    ("reflexive-dodge", 6),
    // Game: RenewedVigorFeature aad496a7c9bee224cbd01a3cf8e42061
    ("renewed-vigor", 4),
    // Game: RollingDodgeFeature abfaa9ad598093f45ac6ac1db58f9a40
    ("rolling-dodge", 6),
    // Game: WreckingBlowsFeature 5bccc86dd1f187a4a99f092dc054c755
    ("wrecking-blows", 12),
];

/// The rage powers the game also gates by level that the data doesn't yet: the celestial, daemon and fiend totems and
/// their greater forms, and Come and Get Me.
const LEVEL_GATED_TOO: [(&str, u8); 7] = [
    // Game: CelestialTotemFeature 5156331dc888e9347ae6fc81ad3f3cec
    ("celestial-totem", 6),
    // Game: CelestialTotemGreaterFeature 774f79845d1683a43aa42ebd2a549497
    ("celestial-totem-greater", 10),
    // Game: DaemonTotemFeature d673c30720e8e7c4bb0903dc3c9ab649
    ("daemon-totem", 6),
    // Game: DaemonTotemGreaterFeature 9a2f0ffe517d221459640a4ad85710d7
    ("daemon-totem-greater", 10),
    // Game: FiendTotemFeature ce449404eeb4a7c499fbe0248056174f
    ("fiend-totem", 6),
    // Game: FiendTotemGreaterFeature 1105632657d94d940a43707a3a57b006
    ("fiend-totem-greater", 10),
    // Game: ComeAndGetMeFeature 032402bde3fb47e39e6ebb6724ae9c22
    ("come-and-get-me", 12),
];

/// Whether `prerequisite` holds `wanted` among its children, at any depth.
fn mentions(prerequisite: &Prerequisite, wanted: &Prerequisite) -> bool {
    prerequisite == wanted
        || match prerequisite {
            Prerequisite::All(children) | Prerequisite::Any(children) => {
                children.iter().any(|c| mentions(c, wanted))
            }
            _ => false,
        }
}

fn class_level(class: &str, level: u8) -> Prerequisite {
    Prerequisite::ClassLevel {
        class: ClassKey::new(class),
        level,
    }
}

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
        // Game: SkaldTalentSelection d2a8fde8985691045b90e1ec57e3cc57 at 2, 7, 12, 17; ProvocateurArchetype
        // a1f5ab7eb78942f6889304acbd92e945 trades them for ProvocateurTalentSelection
        // aa88cb5263a54791b8d1e471bc192ad4 at 4, 9, 14, 19.
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
        // Game: BattleScionArchetype 71d41dd6309ea994886908c34c87ccd9: BattleProwessSelection
        // 29b480a26a88f9e47a10d8c9fab84ee6 at 3, 6, 9, 12, 15, 18.
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
}

/// The game's Battle Scion holds the Battle Prowess feature from its 3rd-level row only; the later rows hold only the
/// selection.
#[test]
#[ignore = "waits on ticket 286: the game gives the Battle Prowess feature on row 3 only"]
fn the_battle_scions_battle_prowess_feature_is_on_row_3_only() {
    // Game: BattleScionArchetype 71d41dd6309ea994886908c34c87ccd9 adds BattleProwessFeature
    // 225940b509289224e83cd900f15b852e at 3 only.
    let prowess = class("skald")
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
    assert_eq!(prowess, [3]);
}

/// A rage power's level prerequisite reads skald levels as well as barbarian levels, in each of the 14 options the
/// guide gates by "Barbarian level" (the game's other two ways and seven more gated powers are
/// `a_rage_powers_level_gate_reads_barbarian_skald_primalist_and_shifter_levels`'s).
#[test]
fn a_rage_powers_level_prerequisite_reads_skald_levels_too() {
    // Game: each row's blueprint in LEVEL_GATED: PrerequisiteClassLevel BarbarianClass and SkaldClass at the level, in
    // its Any group.
    for (name, needs) in LEVEL_GATED {
        let key = rage_option(name);
        let prerequisite = selection_option(&key).prerequisite.clone().unwrap();
        assert!(
            mentions(&prerequisite, &class_level("barbarian", needs))
                && mentions(&prerequisite, &class_level("skald", needs)),
            "{key}: {prerequisite:?}"
        );
    }
}

/// The game's level gates are one of barbarian, skald, Primalist bloodrager and shifter levels (no shifter for Protect
/// Vitals and Increased Damage Reduction), on 21 rage powers: the 14 and the celestial, daemon and fiend totems (6),
/// their greater forms (10) and Come and Get Me (12), each totem needing the one before it.
#[test]
#[ignore = "waits on ticket 269: the game gives 21 level gates, each read from barbarian, skald, \
            Primalist or shifter levels"]
fn a_rage_powers_level_gate_reads_barbarian_skald_primalist_and_shifter_levels() {
    // Game: each row's blueprint in LEVEL_GATED and LEVEL_GATED_TOO (PrerequisiteClassLevel BarbarianClass, SkaldClass,
    // ShifterClass, and PrerequisiteArchetypeLevel BloodragerClass PrimalistArchetype, one of them).
    for (name, needs) in LEVEL_GATED.into_iter().chain(LEVEL_GATED_TOO) {
        let key = rage_option(name);
        let prerequisite = selection_option(&key).prerequisite.clone().unwrap();
        let mut ways = vec![
            class_level("barbarian", needs),
            class_level("skald", needs),
            Prerequisite::All(vec![
                Prerequisite::Archetype(rules::ArchetypeKey::new("bloodrager/primalist")),
                class_level("bloodrager", needs),
            ]),
        ];
        if !matches!(name, "protect-vitals" | "increased-damage-reduction") {
            ways.push(class_level("shifter", needs));
        }
        let mut gate = Prerequisite::Any(ways);
        gate.canonicalize();
        assert!(mentions(&prerequisite, &gate), "{key}: {prerequisite:?}");
    }
    // Game: CelestialTotemFeature 5156331dc888e9347ae6fc81ad3f3cec needs CelestialTotemLesserFeature, and the greater
    // CelestialTotemFeature; so too the daemon and fiend totems.
    for totem in ["celestial", "daemon", "fiend"] {
        for (needs, power) in [("lesser", ""), ("", "-greater")] {
            let before = match needs {
                "" => rage_option(&format!("{totem}-totem")),
                _ => rage_option(&format!("{totem}-totem-{needs}")),
            };
            let key = rage_option(&format!("{totem}-totem{power}"));
            let prerequisite = selection_option(&key).prerequisite.clone().unwrap();
            assert!(
                mentions(
                    &prerequisite,
                    &Prerequisite::SelectionOption(rules::SelectionOptionKey::new(before))
                ),
                "{key}: {prerequisite:?}"
            );
        }
    }
    let gated = rage_power_selection()
        .options
        .iter()
        .filter(|option| format!("{:?}", option.prerequisite).contains("\"skald\""))
        .count();
    assert_eq!(gated, 21);
    // A Primalist 8 with rage powers at 4 and 8 is offered Clear Mind in the slot of 8.
    let mut primalist = with_option(
        archetype_build("bloodrager", "bloodrager/primalist", 8),
        1,
        0,
        "bloodrager/bloodline/infernal",
    );
    for level in [4, 8] {
        let choice = slot_of(&primalist, level, "bloodrager/primalist/primal-choices");
        primalist = with_pick(
            primalist,
            choice,
            "bloodrager/primalist/primal-choices/rage-powers",
        );
    }
    let slot = engine()
        .evaluate(&primalist)
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .rfind(|slot| slot.kind == selection_kind(RAGE_POWER))
        .expect("a rage power slot")
        .id
        .clone();
    assert!(offered_in(&primalist, &slot)[&rage_option("clear-mind")].is_ok());
}

/// A skald's picker opens each level-gated power at its skald level, a barbarian's at its barbarian level, and the
/// Instinctual Warrior's monk powers and the Battle Scion's teamwork feat stay locked.
#[test]
fn a_skalds_rage_powers_open_at_skald_levels() {
    let build = skald(None, 20);
    // Game: each row's blueprint in LEVEL_GATED; the totems and stances also need the power before them
    // (BeastTotemFeature c085888db293f2741b881cc989a2ab14 needs BeastTotemLesserFeature, DeadlyAccuracyFeature
    // c841ffa13d39ce442a408f57feb3cb8e LethalStanceFeature e4450dd9c06dc034fb7c0c08abcc202b, …).
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
    // Game: RenewedVigorFeature aad496a7c9bee224cbd01a3cf8e42061 (skald 4).
    let reason = lock_reason(&build, 3, &rage_option("renewed-vigor"));
    assert!(reason.contains("Skald level 4"), "{reason}");
    // Game: SkaldRagePowerSelection 2476514e31791394fa140f1a07941c96 holds neither the monk powers nor a teamwork feat.
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
    // A chain through the levels is possible: Lethal Stance, Deadly Accuracy, then Lethal Accuracy at 16. Game:
    // LethalStanceFeature e4450dd9c06dc034fb7c0c08abcc202b, DeadlyAccuracyFeature c841ffa13d39ce442a408f57feb3cb8e,
    // LethalAccuracyFeature bb79cb9706379934c9460e46fe8cd04e.
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
    // Game: FearlessRageFeature 2f59c85adec695b418a66242f2ba08b8 (12); ClearMindFeature
    // 4a236ae23d5a4f76b3616ddfa337e724 (8).
    let build = barbarian(12);
    let fearless = rage_option("fearless-rage");
    assert!(!open_at(&build, 10).contains(&fearless));
    assert!(open_at(&build, 12).contains(&fearless));
    // A plain barbarian's reason names the barbarian only.
    assert_eq!(
        lock_reason(&barbarian(4), 4, &rage_option("clear-mind")),
        "Needs Barbarian level 8 (have 4)"
    );
    // Other classes have no such slot. Game: BardProgression 8127f5ff40f5b484b8be98609358b9d2, RogueProgression
    // b57b2a75a5abcaf47a01cf84672b50e9 and the fighter's hold no rage power selection.
    for class in ["bard", "rogue", "fighter"] {
        assert!(levels_with_slot_of(&self::build(class, 20), RAGE_POWER).is_empty());
    }
}

/// Skald levels and barbarian levels do not stack (the user's decision): each class's level alone is read.
// Game: Blueprints/Classes/Prerequisites/PrerequisiteClassLevel.CheckInternal, WrathDecompiled 6ce4a6b (one class's
// level); ClearMindFeature 4a236ae23d5a4f76b3616ddfa337e724 (8), RenewedVigorFeature aad496a7c9bee224cbd01a3cf8e42061
// (4).
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
    // Game: AnimalFuryFeature 25954b1652bebc2409f9cb9d5728bceb (Ranks 1), in both RagePowerSelection
    // 28710502f46848d48b3f0d6132817c4e and SkaldRagePowerSelection 2476514e31791394fa140f1a07941c96.
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
/// rage powers (the game's "once it has the slot" is `a_skald_takes_extra_rage_power_once_it_has_a_rage_power_slot`'s).
#[test]
fn extra_rage_power_reaches_a_skald() {
    let animal_fury = rage_option("animal-fury");
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a needs SkaldRagePowerSelection
    // 2476514e31791394fa140f1a07941c96 among others, and offers the same rage powers.
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
    // Game: RagePowerSelection 28710502f46848d48b3f0d6132817c4e (33) and InstinctualWarriorRagePowerSelection
    // 609f0e5336084442a0dafa3abd4d31c5 (38) (equivalent model: one list, the five gated by the archetype).
    assert_eq!(rage.options.len(), 38);
    let selection = any_selection(SCION_SLOT);
    assert_eq!(selection.name, "Rage Power or Teamwork Feat");
    // The shared rage powers are `the_battle_scions_selection_offers_21_rage_powers`'s.
    // Game: BattleProwessSelection 29b480a26a88f9e47a10d8c9fab84ee6 lists the 12 teamwork feats of TeamworkFeat
    // d87e2f6a9278ac04caeb0f93eff95fcb, each once (equivalent model: a repeatable option giving a teamwork feat slot).
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
}

/// The game's Extra Rage Power needs a rage power selection held, which a skald holds from its slot at 3, picked or
/// not.
#[test]
#[ignore = "waits on ticket 293: the importer writes Extra Rage Power's prerequisite as the game's \
            selections, in ticket 247's form, so a skald 3 with no rage power picked meets it"]
fn a_skald_takes_extra_rage_power_once_it_has_a_rage_power_slot() {
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a (PrerequisiteFeature SkaldRagePowerSelection
    // 2476514e31791394fa140f1a07941c96, Group Any);
    // UnitLogic/Class/LevelUp/Actions/LevelUpHelper.AddFeaturesFromProgression, WrathDecompiled 6ce4a6b (the
    // selection is held once its level is reached).
    let empty = skald(None, 3);
    assert_eq!(
        feat_status(&empty, &general_slot(3), "extra-rage-power"),
        Ok(())
    );
}

/// The game's Battle Prowess offers 21 rage powers, not every one: neither the celestial nor the daemon totems (but
/// their lesser forms neither), Rolling Dodge, Crippling Blows, Powerful Stance, Wrecking Blows, Clear Mind nor Come
/// and Get Me.
#[test]
#[ignore = "waits on ticket 286: the game gives Battle Prowess 21 rage powers"]
fn the_battle_scions_selection_offers_21_rage_powers() {
    // Game: BattleProwessSelection 29b480a26a88f9e47a10d8c9fab84ee6: 21 rage powers and 12 teamwork feats.
    let selection = any_selection(SCION_SLOT);
    let shared: BTreeSet<String> = selection
        .shared_options
        .iter()
        .map(|k| k.to_string())
        .collect();
    let expected: BTreeSet<String> = [
        "animal-fury",
        "beast-totem",
        "beast-totem-greater",
        "beast-totem-lesser",
        "fearless-rage",
        "fiend-totem",
        "fiend-totem-greater",
        "fiend-totem-lesser",
        "guarded-stance",
        "protect-vitals",
        "reflexive-dodge",
        "increased-damage-reduction",
        "internal-fortitude",
        "deadly-accuracy",
        "lethal-accuracy",
        "lethal-stance",
        "inspire-ferocity",
        "reckless-stance",
        "renewed-vigor",
        "scent",
        "swift-foot",
    ]
    .iter()
    .map(|name| rage_option(name))
    .collect();
    assert_eq!(shared, expected);
}

/// The game's Extra Rage Power needs the barbarian's, the Instinctual Warrior's or the skald's rage power selection:
/// a Battle Scion with none of them can't take it, and the feat's slot is never Battle Prowess.
#[test]
#[ignore = "waits on ticket 264: the game gives Extra Rage Power no Battle Prowess slot and closes it \
            to a Battle Scion"]
fn extra_rage_power_is_closed_to_a_battle_scion_with_no_other_rage_power_selection() {
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a (RagePowerSelection,
    // InstinctualWarriorRagePowerSelection or SkaldRagePowerSelection, Group Any); BattleScionArchetype
    // 71d41dd6309ea994886908c34c87ccd9 removes SkaldRagePowerSelection.
    let extra = &feat("extra-rage-power").extra_pick_of;
    assert!(extra.iter().all(|s| s.as_str() != SCION_SLOT));
    let mut scion = skald(Some(BATTLE_SCION), 3);
    scion.picks.insert(
        slot_of(&scion, 3, SCION_SLOT),
        pick(&rage_option("animal-fury")),
    );
    assert!(feat_status(&scion, &general_slot(3), "extra-rage-power").is_err());
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
    // Game: BattleProwessSelection 29b480a26a88f9e47a10d8c9fab84ee6 offers Outflank 422dab7309e1ad343935f33a4d6e9f11
    // and AnimalFuryFeature 25954b1652bebc2409f9cb9d5728bceb, not Power Attack (equivalent model: the teamwork feat
    // through its own slot).
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
    // Extra Rage Power and a Battle Scion are
    // `extra_rage_power_is_closed_to_a_battle_scion_with_no_other_rage_power_selection`'s.
}

/// The teamwork feat is offered in the Battle Scion's own slots only: not in a barbarian's slot, even for a Battle Scion
/// with barbarian levels, and a rage power is taken once across both.
#[test]
fn a_battle_scion_with_barbarian_levels_is_offered_no_teamwork_feat_in_a_barbarian_slot() {
    // Game: RagePowerSelection 28710502f46848d48b3f0d6132817c4e holds no teamwork feat; AnimalFuryFeature
    // 25954b1652bebc2409f9cb9d5728bceb (Ranks 1) in both selections.
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
    // The Inciter's 12 has a rage power and no rogue talent; its talents are at 4, 9, 14 and 19. Game:
    // ProvocateurArchetype a1f5ab7eb78942f6889304acbd92e945 keeps SkaldRagePowerSelection at 12 and removes
    // SkaldTalentSelection there.
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
/// skald's its own. (The game's gate also reads Primalist and shifter levels: the prerequisite's text and a
/// Primalist's reason are `a_rage_powers_level_gate_reads_barbarian_skald_primalist_and_shifter_levels`'s.)
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
    // Game: ClearMindFeature 4a236ae23d5a4f76b3616ddfa337e724: barbarian 8 or skald 8 among its ways.
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
}
