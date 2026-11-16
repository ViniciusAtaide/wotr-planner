//! Options with rows by option level (ticket 30): bloodlines, schools, orders, spirits, ways and combat styles.

use std::collections::BTreeSet;

use rules::{Build, ClassKey, FeatKey, Issue, LevelUp, SelectionOptionKey, SkillKey, SlotKind};

use crate::support::*;

#[test]
fn a_sorcerer_holds_the_bloodlines_powers_at_1_3_9_15_and_20() {
    let build = with_option(build("sorcerer", 20), 1, 0, ABYSSAL);
    assert!(engine().is_possible(&build));
    // Game: BloodlineAbyssalProgression d3a4cb7be97a6694290f0dcfbd147113: claws at 1, resistances at 3, strength at
    // 9, summonings at 15, Demonic Might at 20.
    let powers = [
        (1, "sorcerer/bloodline/abyssal/grow-claws"),
        (3, "sorcerer/bloodline/abyssal/demon-resistances"),
        (9, "sorcerer/bloodline/abyssal/strength-of-the-abyss"),
        (15, "sorcerer/bloodline/abyssal/added-summonings"),
        (20, "sorcerer/bloodline/abyssal/demonic-might"),
    ];
    for (level, power) in powers {
        if level > 1 {
            assert!(
                !holds(&at(&build, level - 1), power),
                "{power} before {level}"
            );
        }
        assert!(holds(&at(&build, level), power), "{power} at {level}");
    }
    // Wings at 15, and Strength of the Abyss +2 at 9, 13 and 17.
    // Game: BloodlineAbyssalProgression d3a4cb7be97a6694290f0dcfbd147113's wings at 15; the strength chain
    // BloodlineAbyssalStrengthAbilityAddLevel1 489c8c4a53a111d4094d239054b26e32 (+2, then +4 at 13 by
    // AddFeatureOnClassLevel) and BloodlineAbyssalStrengthAbilityAddLevel2 48d81d94ddf0cc84abbde97718cb25f2 (+6 at 17).
    assert!(at(&build, 15).has_feat(&FeatKey::new("wings")));
    assert_eq!(at(&build, 8).ability_scores.strength, 10);
    assert_eq!(at(&build, 9).ability_scores.strength, 12);
    assert_eq!(at(&build, 17).ability_scores.strength, 16);
    assert_eq!(
        at(&build, 20).option_level(&SelectionOptionKey::new(ABYSSAL)),
        20
    );
}

#[test]
fn a_sorcerer_picks_the_bloodlines_feats_at_7_13_and_19() {
    let build = with_option(build("sorcerer", 19), 1, 0, ABYSSAL);
    // Game: SorcererProgression 997665565ca80a649aedd72455c4df1f gives the bloodline's feat selection at 7, 13 and 19.
    let sheet = engine().evaluate(&build);
    for level in [7, 13, 19] {
        let slot = sheet
            .slot(&class_slot(level))
            .expect("a bloodline feat slot");
        assert_eq!(slot.kind, feat_kind("sorcerer/bloodline-feats"));
    }
    // "Augment Summoning, Bull Rush, Cleave, Empower Spell, Great Fortitude, Power Attack, Skill Focus (Lore:
    // Religion), Sunder Armor."
    // Game: BloodlineAbyssalFeatSelection 7781831170d794a4d8f147a1326c35fd: Augment Summoning, Cleave, Empower Spell,
    // Great Fortitude, Power Attack, Skill Focus (Lore Religion), Improved Bull Rush, Improved Sunder.
    let feats = feats_offered(&build, &class_slot(7));
    assert!(feats.contains("augment-summoning") && feats.contains("metamagic-empower-spell"));
    assert!(
        !feats.contains("combat-casting"),
        "an Arcane bloodline feat"
    );
}

#[test]
fn a_seeker_skips_the_bloodline_rows_at_3_9_and_15() {
    let build = with_option(
        archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 20),
        1,
        0,
        ABYSSAL,
    );
    assert!(engine().is_possible(&build));
    let at_20 = at(&build, 20);
    // Game: SeekerArchetype 7229db6fc0b07af4180e783eed43c4d9's SeekerBloodlineAbyssalProgression
    // 17b752be1e0f4a34e8914df52eebeb75: the claws at 1 and Demonic Might at 20, no resistances at 3, no summonings at
    // 15. Strength of the Abyss: `a_seekers_abyssal_bloodline_keeps_strength_of_the_abyss`.
    assert!(holds(&at_20, "sorcerer/bloodline/abyssal/grow-claws"));
    assert!(holds(&at_20, "sorcerer/bloodline/abyssal/demonic-might"));
    for skipped in [
        "sorcerer/bloodline/abyssal/demon-resistances",
        "sorcerer/bloodline/abyssal/added-summonings",
    ] {
        assert!(!holds(&at_20, skipped), "{skipped}");
    }
}

/// The Seeker's Abyssal bloodline keeps Strength of the Abyss, at 9 and 13.
#[test]
#[ignore = "waits on ticket 281: the game gives the Seeker's Abyssal bloodline Strength of the Abyss"]
fn a_seekers_abyssal_bloodline_keeps_strength_of_the_abyss() {
    let build = with_option(
        archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 20),
        1,
        0,
        ABYSSAL,
    );
    // Game: SeekerBloodlineAbyssalProgression 17b752be1e0f4a34e8914df52eebeb75 at 9:
    // BloodlineAbyssalStrengthAbilityAddLevel1 489c8c4a53a111d4094d239054b26e32 (and AddLevel2 at 13).
    from_level(
        &build,
        9,
        "sorcerer/bloodline/abyssal/strength-of-the-abyss",
    );
}

#[test]
fn a_sorcerer_5_dragon_disciple_5_has_bloodline_level_10() {
    let red = "sorcerer/bloodline/draconic-red";
    let mut build = build("sorcerer", 5);
    for _ in 0..5 {
        build
            .levels
            .push(LevelUp::new(ClassKey::new("dragon-disciple")));
    }
    for level in &mut build.levels[..5] {
        level
            .skill_ranks
            .insert(SkillKey::new("knowledge-arcana"), 1);
    }
    let build = with_option(build, 1, 0, red);
    // Entering at 6: `a_sorcerer_5_enters_dragon_disciple_at_6`.
    let at_10 = at(&build, 10);
    // Game: BloodlineDraconicRedProgression 8c6e5b3cf12f71e43949f52c41ae70a8 lists DragonDiscipleClass
    // 72051275b1dbb2d42ba9118237794f7c (Blueprints/Classes/BlueprintProgression.CalcLevel, WrathDecompiled 6ce4a6b);
    // DragonDiscipleProgression 69fc2bad2eb331346a6c777423e0d0f7's DragonDiscipleStrength
    // 270259f59c6ae6040babd5797feef2e2 at 2 and 4.
    assert_eq!(at_10.option_level(&SelectionOptionKey::new(red)), 10);
    assert!(holds(&at_10, "sorcerer/bloodline/draconic/grow-claws"));
    // The Dragon Disciple's Strength +2 at its 2nd and 4th levels.
    assert_eq!(at_10.ability_scores.strength, 14);
}

/// A sorcerer 5 enters Dragon Disciple at character level 6: it has no character-level prerequisite.
#[test]
#[ignore = "waits on ticket 287: the game gives Dragon Disciple no character-level prerequisite"]
fn a_sorcerer_5_enters_dragon_disciple_at_6() {
    let mut build = build("sorcerer", 5);
    build
        .levels
        .push(LevelUp::new(ClassKey::new("dragon-disciple")));
    for level in &mut build.levels[..5] {
        level
            .skill_ranks
            .insert(SkillKey::new("knowledge-arcana"), 1);
    }
    let build = with_option(build, 1, 0, "sorcerer/bloodline/draconic-red");
    // Game: DragonDiscipleClass 72051275b1dbb2d42ba9118237794f7c needs 1st-level arcane spontaneous spells, Knowledge
    // (Arcana) 5 and a draconic bloodline or none; no character level (Blueprints/Classes/
    // BlueprintCharacterClass.MeetsPrerequisites, WrathDecompiled 6ce4a6b).
    let entry = engine()
        .class_options(&build, 6)
        .unwrap()
        .into_iter()
        .find(|option| option.class == ClassKey::new("dragon-disciple"))
        .unwrap()
        .status;
    assert_eq!(entry, Ok(()));
}

#[test]
fn a_wizard_holds_the_schools_first_level_power_at_1() {
    // Game: SpecialisationSchoolUniversalistProgression 0933849149cfc9244ac05d6a5b57fd80 at 1:
    // UniversalistSchoolBaseFeature 38aab7423d96de84d8e6ab2cdbccce63.
    let build = with_option(build("wizard", 1), 1, 0, "wizard/school/universalist");
    assert!(engine().is_possible(&build));
    assert!(holds(
        &at(&build, 1),
        "wizard/school/universalist/hand-of-the-apprentice"
    ));
}

#[test]
fn a_fixed_bloodline_is_held_with_no_pick() {
    // Game: EmpyrealSorcererArchetype aa00d945f7cf6c34c909a29a25f2df38 removes the bloodline selection and adds
    // EmpyrealBloodlineProgression 8a95d80a3162d274896d50c2f18bb6b1: its own arcana at 1, the resistances at 3.
    let build = archetype_build("sorcerer", "sorcerer/empyreal-sorcerer", 3);
    let sheet = engine().evaluate(&build);
    assert!(
        sheet.levels[0]
            .slots
            .iter()
            .all(|slot| !matches!(slot.kind, SlotKind::Selection(_)))
    );
    let at_3 = at(&build, 3);
    assert_eq!(
        at_3.option_count(&SelectionOptionKey::new("sorcerer/bloodline/celestial")),
        1
    );
    assert!(holds(
        &at_3,
        "sorcerer/bloodline/celestial/celestial-resistances"
    ));
    // Its own Empyreal Arcana stands in for the Celestial's arcana.
    assert!(!holds(&at_3, "sorcerer/bloodline/celestial/angelic-arcana"));
}

#[test]
fn the_winter_witch_needs_a_shamans_frost_spirit() {
    let shaman = |spirit: &str| {
        let mut build = build("shaman", 6);
        for level in &mut build.levels {
            level
                .skill_ranks
                .insert(SkillKey::new("knowledge-arcana"), 1);
            level.skill_ranks.insert(SkillKey::new("lore-nature"), 1);
        }
        with_option(build, 1, 0, spirit)
    };
    let witch = |build: &Build| -> Result<(), Issue> {
        engine()
            .class_options(build, 7)
            .unwrap()
            .into_iter()
            .find(|o| o.class == ClassKey::new("winter-witch"))
            .unwrap()
            .status
    };
    // Game: WinterWitchClass eb24ca44debf6714aabe1af1fd905a07 needs (Any) ShamanFrostSpiritProgression,
    // WitchWinterPatronProgression or WinterChildDomainSelection, 3rd-level spells and Knowledge (Arcana) 5.
    assert_eq!(witch(&shaman("shaman/spirit/frost")), Ok(()));
    assert!(witch(&shaman("shaman/spirit/battle")).is_err());
}

#[test]
fn a_rangers_style_slot_picks_from_the_held_styles_list() {
    let build = with_option(
        build("ranger", 6),
        2,
        0,
        "ranger/combat-style-feat/archery-feats",
    );
    // Game: RangerStyleArcherySelection2 3cf94df3f1cf9a443a880bc1ae9be3c6 (Rapid Shot, Dodge; no Manyshot, no Power
    // Attack); RangerStyleArcherySelection6 6c799d09d5b93f344b9ade0e0c765c2d (Manyshot).
    let at_2 = feats_offered(&build, &class_slot(2));
    assert!(at_2.contains("rapid-shot") && at_2.contains("dodge"));
    assert!(!at_2.contains("manyshot") && !at_2.contains("power-attack"));
    assert!(feats_offered(&build, &class_slot(6)).contains("manyshot"));
}

#[test]
fn a_hag_riven_picks_from_its_four_bloodlines() {
    let build = archetype_build("bloodrager", "bloodrager/hag-riven", 1);
    // Game: BloodragerBloodlineSelection 62b33ac8ceb18dd47ad4c8f06849bc01: every bloodline but Hag
    // (BloodragerHagBloodline c5d57f425eec48beb1f43719e92db1c1), Arcane, Fey and the four Elemental ones has
    // PrerequisiteNoArchetype HagRivenArchetype 61edfe08d9e04a11844e5fcc01da3602.
    let offered: BTreeSet<String> = offered(&build, 1).into_keys().collect();
    assert!(offered.contains("bloodrager/bloodline/hag"));
    assert!(offered.contains("bloodrager/bloodline/elemental-fire"));
    assert!(!offered.contains("bloodrager/bloodline/abyssal"));
    assert_eq!(offered.len(), 7);
}

#[test]
fn a_cavalier_of_the_cockatrice_gets_dazzling_display_at_2() {
    let build = with_option(
        build("cavalier", 2),
        1,
        0,
        "cavalier/order/order-of-the-cockatrice",
    );
    // Game: CavalierOrderOfTheCockatriceProgression 4e2fb051c9c4c144dbad08d994906f99 at 2:
    // DazzlingDisplayIgnoreWeaponNeed 4d2b28040c8d4552aa401cc627881b09 (equivalent model: the feat).
    assert!(at(&build, 2).has_feat(&FeatKey::new("dazzling-display")));
}
