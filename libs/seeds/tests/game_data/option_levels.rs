//! Options with rows by option level (ticket 30): bloodlines, schools, orders, spirits, ways and combat styles.

use std::collections::BTreeSet;

use rules::{
    Build, ClassKey, FeatKey, FeatSlot, Issue, LevelUp, Prerequisite, SelectionOptionKey, SkillKey,
    SlotKind, Unmet,
};

use crate::support::*;

#[test]
fn a_sorcerer_holds_the_bloodlines_powers_at_1_3_9_15_and_20() {
    let build = with_option(build("sorcerer", 20), 1, 0, ABYSSAL);
    assert!(engine().is_possible(&build));
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
    assert!(at(&build, 15).feats.contains_key(&FeatKey::new("wings")));
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
    let sheet = engine().evaluate(&build);
    for level in [7, 13, 19] {
        let slot = sheet
            .slot(&class_slot(level))
            .expect("a bloodline feat slot");
        assert_eq!(
            slot.kind,
            SlotKind::Feat(FeatSlot::OptionList {
                selection: rules::FeatureSelectionKey::new("sorcerer/bloodline"),
                at_level: None,
                label: None,
            })
        );
    }
    // "Augment Summoning, Bull Rush, Cleave, Empower Spell, Great Fortitude, Power Attack, Skill Focus (Lore:
    // Religion), Sunder Armor."
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
    assert!(holds(&at_20, "sorcerer/bloodline/abyssal/grow-claws"));
    assert!(holds(&at_20, "sorcerer/bloodline/abyssal/demonic-might"));
    for skipped in [
        "sorcerer/bloodline/abyssal/demon-resistances",
        "sorcerer/bloodline/abyssal/strength-of-the-abyss",
        "sorcerer/bloodline/abyssal/added-summonings",
    ] {
        assert!(!holds(&at_20, skipped), "{skipped}");
    }
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
    // Only the minimum character level stands in the way of entering at 6 (it's checked before the level).
    let entry = engine()
        .class_options(&build, 6)
        .unwrap()
        .into_iter()
        .find(|option| option.class == ClassKey::new("dragon-disciple"))
        .unwrap()
        .status;
    assert_eq!(
        entry,
        Err(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::CharacterLevel(6),
            has: Some(5),
        }))
    );
    let at_10 = at(&build, 10);
    assert_eq!(at_10.option_level(&SelectionOptionKey::new(red)), 10);
    assert!(holds(&at_10, "sorcerer/bloodline/draconic/grow-claws"));
    // The Dragon Disciple's Strength +2 at its 2nd and 4th levels.
    assert_eq!(at_10.ability_scores.strength, 14);
}

#[test]
fn a_wizard_holds_the_schools_first_level_power_at_1() {
    let build = with_option(build("wizard", 1), 1, 0, "wizard/school/universalist");
    assert!(engine().is_possible(&build));
    assert!(holds(
        &at(&build, 1),
        "wizard/school/universalist/hand-of-the-apprentice"
    ));
}

#[test]
fn a_fixed_bloodline_is_held_with_no_pick() {
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
    let at_2 = feats_offered(&build, &class_slot(2));
    assert!(at_2.contains("rapid-shot") && at_2.contains("dodge"));
    assert!(!at_2.contains("manyshot") && !at_2.contains("power-attack"));
    assert!(feats_offered(&build, &class_slot(6)).contains("manyshot"));
}

#[test]
fn a_hag_riven_picks_from_its_four_bloodlines() {
    let build = archetype_build("bloodrager", "bloodrager/hag-riven", 1);
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
    assert!(
        at(&build, 2)
            .feats
            .contains_key(&FeatKey::new("dazzling-display"))
    );
}
