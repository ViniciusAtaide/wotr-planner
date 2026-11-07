//! The extraction report's gaps (ticket 43), the sorcerer's level-1 feat (tickets 44 and 92) and Extra Slayer Talent
//! (ticket 44).

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, Build, ClassKey, FeatCategory, FeatKey, FeatSlot, FeatureKey, Issue, Pick,
    Prerequisite, SelectionOptionKey, SkillKey, SlotKind, Unmet,
};

use crate::support::*;

// ---- the extraction report's gaps (ticket 43) -----------------------------------------------------------------

/// The barbarian's four totem lines, each Lesser, plain and Greater.
const TOTEM_LINES: [&str; 4] = ["beast", "celestial", "daemon", "fiend"];

fn totem_options(line: &str) -> [String; 3] {
    [
        format!("barbarian/rage-power/{line}-totem-lesser"),
        format!("barbarian/rage-power/{line}-totem"),
        format!("barbarian/rage-power/{line}-totem-greater"),
    ]
}

/// The options a prerequisite rules out with a `Not(SelectionOption(..))` in its top-level `All` (or as itself).
fn ruled_out(prerequisite: Option<&Prerequisite>) -> BTreeSet<String> {
    let children = match prerequisite {
        Some(Prerequisite::All(children)) => children.iter().collect(),
        Some(other) => vec![other],
        None => Vec::new(),
    };
    children
        .into_iter()
        .filter_map(|child| match child {
            Prerequisite::Not(inner) => match inner.as_ref() {
                Prerequisite::SelectionOption(option) => Some(option.to_string()),
                other => panic!("an unexpected Not: {other:?}"),
            },
            _ => None,
        })
        .collect()
}

fn not_option(key: &str) -> Prerequisite {
    Prerequisite::Not(Box::new(Prerequisite::SelectionOption(
        SelectionOptionKey::new(key),
    )))
}

/// '"Totem" powers are exclusive. Selecting one "Totem" power closes off the other categories.' (barbarian#rage-power)
#[test]
fn the_totem_rage_powers_are_exclusive_one_line_per_character() {
    // Each totem power rules out the other three lines' nine powers; no other rage power rules anything out.
    for line in TOTEM_LINES {
        let others: BTreeSet<String> = TOTEM_LINES
            .iter()
            .filter(|other| **other != line)
            .flat_map(|other| totem_options(other))
            .collect();
        for key in totem_options(line) {
            assert_eq!(
                ruled_out(selection_option(&key).prerequisite.as_ref()),
                others,
                "{key}"
            );
        }
    }
    let rage_power = DATA
        .classes
        .iter()
        .find(|class| class.key.as_str() == "barbarian")
        .unwrap()
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == "barbarian/rage-power")
        .unwrap();
    for option in &rage_power.options {
        if !option.key.as_str().contains("-totem") {
            assert_eq!(ruled_out(option.prerequisite.as_ref()), BTreeSet::new());
        }
    }

    // A barbarian with Beast Totem, Lesser at 2 is offered no other line's power, and his own line's chain.
    let mut build = barbarian(10);
    build.picks.insert(
        selection_slot(2),
        pick("barbarian/rage-power/beast-totem-lesser"),
    );
    let at_4 = offered(&build, 4);
    let beast_lesser = not_option("barbarian/rage-power/beast-totem-lesser");
    for line in ["celestial", "daemon", "fiend"] {
        for key in totem_options(line) {
            let status = at_4[&key].clone();
            match status {
                Err(Issue::Unmet(Unmet::Condition { needs, .. })) => {
                    assert_eq!(needs, beast_lesser)
                }
                Err(Issue::Unmet(Unmet::All(children))) => assert!(
                    children.contains(&Unmet::Condition {
                        needs: beast_lesser.clone(),
                        has: None
                    }),
                    "{key}: {children:?}"
                ),
                other => panic!("{key}: {other:?}"),
            }
        }
    }
    build
        .picks
        .insert(selection_slot(6), pick("barbarian/rage-power/beast-totem"));
    assert!(possible(&build));
    assert_eq!(
        offered(&build, 10)["barbarian/rage-power/beast-totem-greater"],
        Ok(())
    );

    // Two lines can't be held: Fiend Totem, Lesser after Beast Totem, Lesser, and the reason says why.
    let mut two = barbarian(4);
    two.picks.insert(
        selection_slot(2),
        pick("barbarian/rage-power/beast-totem-lesser"),
    );
    two.picks.insert(
        selection_slot(4),
        pick("barbarian/rage-power/fiend-totem-lesser"),
    );
    let issue = why_not(&two, &selection_slot(4)).expect("offered, locked");
    assert_eq!(issue, unmet(beast_lesser, None).unwrap());
    assert_eq!(
        engine().issue_text(&issue).as_deref(),
        Some("Not with Beast Totem, Lesser")
    );
    // Lines other than the one held stay open to a barbarian with none.
    let none = offered(&barbarian(2), 2);
    for line in TOTEM_LINES {
        assert_eq!(
            none[&format!("barbarian/rage-power/{line}-totem-lesser")],
            Ok(())
        );
    }
}

/// "At 1st level, the Eldritch Scoundrel deals extra damage … She does an additional 1d6 damage at 3rd level,
/// increasing by 1d6 every four additional class levels (7, 11, 15, 19)." (rogue#sneak-attack-eldritch-scoundrel)
#[test]
fn an_eldritch_scoundrels_sneak_attack_dice_come_at_1_3_7_11_15_and_19() {
    let sheet = engine().evaluate(&archetype_build("rogue", "rogue/eldritch-scoundrel", 20));
    let dice: Vec<u8> = (1..=20)
        .map(|level| {
            sheet.at(level).unwrap().feature_count(&FeatureKey::new(
                "rogue/eldritch-scoundrel/sneak-attack-eldritch-scoundrel",
            ))
        })
        .collect();
    assert_eq!(
        dice,
        [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6]
    );
    // It is the archetype's only sneak attack.
    assert_eq!(
        sheet
            .at(20)
            .unwrap()
            .feature_count(&FeatureKey::new("rogue/sneak-attack")),
        0
    );
}

/// "At 6th and 14th levels, a Shaman gains a hex slot that can be temporarily filled with a hex granted by either
/// his main spirit or his current Wandering Spirit." (shaman#wandering-hex): a daily choice, as the Wandering Spirit
/// it draws on.
#[test]
fn a_shamans_wandering_hex_is_a_daily_choice_from_6() {
    let wandering_hex = FeatureKey::new("shaman/wandering-hex");
    let daily = |build: &Build, level: u8| engine().daily_choices(&at(build, level));
    let shaman = build("shaman", 14);
    assert!(!daily(&shaman, 5).contains(&wandering_hex));
    assert!(daily(&shaman, 6).contains(&wandering_hex));
    assert!(daily(&shaman, 6).contains(&FeatureKey::new("shaman/wandering-spirit")));
    // The Possessed Shaman has Wandering Skills at 6 and the Wandering Hex at 14 only.
    let possessed = archetype_build("shaman", "shaman/possessed-shaman", 14);
    assert!(!daily(&possessed, 13).contains(&wandering_hex));
    assert!(daily(&possessed, 14).contains(&wandering_hex));
}

// ---- the sorcerer's level-1 feat (tickets 44 and 92) and Extra Slayer Talent (ticket 44) ----------------------

/// The slot a sorcerer's bloodline feats take at 7, 13 and 19: the held bloodline's list.
fn bloodline_feat_slot() -> SlotKind {
    SlotKind::Feat(FeatSlot::OptionList {
        selection: rules::FeatureSelectionKey::new("sorcerer/bloodline"),
        at_level: None,
        label: None,
    })
}

/// A sorcerer bloodline's bonus-feat list, by key: `bloodline_feats(&["abyssal"])`; several lists together.
fn bloodline_feats(bloodlines: &[&str]) -> BTreeSet<String> {
    bloodlines
        .iter()
        .flat_map(|bloodline| {
            let key = format!("sorcerer/bloodline/{bloodline}/feats");
            class("sorcerer")
                .feat_lists
                .iter()
                .find(|list| list.key.as_str() == key)
                .unwrap_or_else(|| panic!("no list {key}"))
                .feats
                .iter()
                .map(|feat| feat.to_string())
        })
        .collect()
}

/// The kind of a Build's class feat slot at `level`, if it has one.
fn class_slot_kind(build: &Build, level: u8) -> Option<SlotKind> {
    engine()
        .evaluate(build)
        .slot(&class_slot(level))
        .map(|slot| slot.kind.clone())
}

/// The slot a sorcerer's level-1 bonus feat takes (ticket 92): any Magic feat.
fn magic_feat_slot() -> SlotKind {
    SlotKind::Feat(FeatSlot::Category(FeatCategory::Magic))
}

/// Every Magic feat, by key.
fn magic_feats() -> BTreeSet<String> {
    DATA.feats
        .iter()
        .filter(|feat| feat.category == FeatCategory::Magic)
        .map(|feat| feat.key.to_string())
        .collect()
}

/// Every sorcerer with a level-1 bonus feat, `levels` levels, holding a bloodline where it picks one, with the
/// bloodline lists its feats at 7, 13 and 19 come from (none for the Geomancer): the base sorcerer (Abyssal),
/// Crossblooded (Arcane and Abyssal), the Empyreal, Sage and Sylvan (fixed: Celestial, Arcane, Fey), a kitsune Nine
/// Tailed Heir (Fey; the archetype is the kitsune's), the Seeker (Abyssal) and the Geomancer (Abyssal).
fn sorcerers_with_a_bonus_feat(
    levels: usize,
) -> Vec<(&'static str, Build, &'static [&'static str])> {
    let crossblooded = with_option(
        with_option(
            archetype_build("sorcerer", "sorcerer/crossblooded", levels),
            1,
            0,
            "sorcerer/bloodline/arcane",
        ),
        1,
        1,
        ABYSSAL,
    );
    let mut heir = with_option(
        archetype_build("sorcerer", "sorcerer/nine-tailed-heir", levels),
        1,
        0,
        "sorcerer/bloodline/fey",
    );
    heir.start.race = Some(rules::RaceKey::new("kitsune"));
    let chosen = |archetype: &str| {
        with_option(
            archetype_build("sorcerer", archetype, levels),
            1,
            0,
            ABYSSAL,
        )
    };
    vec![
        (
            "sorcerer",
            with_option(build("sorcerer", levels), 1, 0, ABYSSAL),
            &["abyssal"],
        ),
        ("crossblooded", crossblooded, &["arcane", "abyssal"]),
        (
            "empyreal",
            archetype_build("sorcerer", "sorcerer/empyreal-sorcerer", levels),
            &["celestial"],
        ),
        (
            "sage",
            archetype_build("sorcerer", "sorcerer/sage-sorcerer", levels),
            &["arcane"],
        ),
        (
            "sylvan",
            archetype_build("sorcerer", "sorcerer/sylvan-sorcerer", levels),
            &["fey"],
        ),
        ("nine tailed heir", heir, &["fey"]),
        ("seeker", chosen("sorcerer/seeker-sorcerer"), &["abyssal"]),
        ("geomancer", chosen("sorcerer/geomancer"), &[]),
    ]
}

/// Ticket 92 (user, 2026-11-07): every sorcerer's level-1 "Bonus Feat" is a Magic feat, in place of ticket 44's
/// bloodline list (the Jack-in-the-Green write-up takes Spell Penetration there, off the Fey list) and of the
/// Geomancer's General slot. Dodge, Power Attack and Toughness, on some bloodline lists, aren't Magic feats.
#[test]
fn every_sorcerers_level_1_bonus_feat_is_a_magic_feat() {
    let magic = magic_feats();
    assert!(magic.contains("spell-penetration") && magic.contains("spell-focus"));
    for (name, build, _) in sorcerers_with_a_bonus_feat(1) {
        assert!(possible(&build), "{name}");
        assert_eq!(
            class_slot_kind(&build, 1),
            Some(magic_feat_slot()),
            "{name}"
        );
        let offered = feats_offered(&build, &class_slot(1));
        assert_eq!(offered, magic, "{name}");
        for feat in ["dodge", "power-attack", "toughness", "improved-initiative"] {
            assert!(!offered.contains(feat), "{name}: {feat}");
        }
        assert_eq!(
            feat_status(&build, &class_slot(1), "spell-penetration"),
            Ok(()),
            "{name}"
        );
        let mut dodge = build;
        dodge
            .picks
            .insert(class_slot(1), Pick::feat(FeatKey::new("dodge")));
        assert!(!possible(&dodge), "{name}");
    }
}

/// The Jack-in-the-Green write-up's level 1 (ticket 92): "Bonus Feat - Spell Penetration; Mounted Combat", a Sylvan
/// Sorcerer 1 with Spell Penetration in the bonus slot and Mounted Combat (Mobility 1 rank) in its general slot.
#[test]
fn a_sylvan_sorcerer_1_takes_spell_penetration_and_mounted_combat() {
    let mut build = archetype_build("sorcerer", "sorcerer/sylvan-sorcerer", 1);
    build.levels[0]
        .skill_ranks
        .insert(SkillKey::new("mobility"), 1);
    assert_eq!(
        feat_status(&build, &class_slot(1), "spell-penetration"),
        Ok(())
    );
    assert_eq!(
        feat_status(&build, &general_slot(1), "mounted-combat"),
        Ok(())
    );
    build
        .picks
        .insert(class_slot(1), Pick::feat(FeatKey::new("spell-penetration")));
    build
        .picks
        .insert(general_slot(1), Pick::feat(FeatKey::new("mounted-combat")));
    assert!(possible(&build));
}

/// "Does not receive bonus feats." (sorcerer#overwhelming-mage): Overwhelming Penetration at 1 and no class feat
/// slot, at 1 or later.
#[test]
fn an_overwhelming_mage_still_has_no_bonus_feat() {
    let mage = with_option(
        archetype_build("sorcerer", "sorcerer/overwhelming-mage", 20),
        1,
        0,
        ABYSSAL,
    );
    assert!(possible(&mage));
    assert!(holds(
        &at(&mage, 1),
        "sorcerer/overwhelming-mage/overwhelming-penetration"
    ));
    for level in 1..=20 {
        assert_eq!(class_slot_kind(&mage, level), None, "{level}");
    }
}

/// The bloodline feats stay at 7, 13 and 19, from the held bloodline's list ("at the bonus feat levels, you may
/// choose a feat from one bloodline or the other", sorcerer#crossblooded); the Geomancer "does not gain … bloodline
/// feats" (sorcerer#geomancer); the Seeker's bonus magic feats stay at 5, 10 and 15.
#[test]
fn the_bloodline_feats_stay_at_7_13_and_19() {
    for (name, build, lists) in sorcerers_with_a_bonus_feat(19) {
        for level in [7, 13, 19] {
            if lists.is_empty() {
                assert_eq!(class_slot_kind(&build, level), None, "{name} {level}");
                continue;
            }
            assert_eq!(
                class_slot_kind(&build, level),
                Some(bloodline_feat_slot()),
                "{name} {level}"
            );
            assert_eq!(
                feats_offered(&build, &class_slot(level)),
                bloodline_feats(lists),
                "{name} {level}"
            );
        }
    }
    let seeker = with_option(
        archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 15),
        1,
        0,
        ABYSSAL,
    );
    for level in [5, 10, 15] {
        assert_eq!(
            class_slot_kind(&seeker, level),
            Some(magic_feat_slot()),
            "{level}"
        );
    }
}

/// The sheet labels the level-1 slot "Magic feat" and the 7th's still "Bloodline feat"; every slot of a sorcerer 20
/// has a label.
#[test]
fn the_sheet_labels_a_sorcerers_level_1_feat_magic_feat() {
    let engine = engine();
    let label = |build: &Build, level: u8| {
        let sheet = engine.evaluate(build);
        engine.slot_label(sheet.slot(&class_slot(level)).expect("a class feat slot"))
    };
    for (name, build, lists) in sorcerers_with_a_bonus_feat(7) {
        assert_eq!(label(&build, 1), "Magic feat", "{name}");
        if !lists.is_empty() {
            assert_eq!(label(&build, 7), "Bloodline feat", "{name}");
        }
    }
    let base = with_option(build("sorcerer", 20), 1, 0, ABYSSAL);
    let sheet = engine.evaluate(&base);
    for slot in sheet.levels.iter().flat_map(|level| &level.slots) {
        let label = engine.slot_label(slot);
        assert!(!label.is_empty() && !label.contains('/'), "{label}");
    }
}

/// "Prerequisites: Slayer Talents or Talented Slayer" (feats#extra-slayer-talent), read as slayer level 2, the level
/// the slayer's table first gives Slayer Talents (ticket 43's decision 3; Talented Slayer is on no saved page), or
/// the Sanctified Slayer's own "Slayer Talents" from inquisitor 8 (ticket 60).
#[test]
fn extra_slayer_talent_needs_slayer_level_2() {
    assert_eq!(
        feat("extra-slayer-talent").prerequisite,
        Some(Prerequisite::Any(vec![
            Prerequisite::All(vec![
                Prerequisite::ClassLevel {
                    class: ClassKey::new("inquisitor"),
                    level: 8,
                },
                Prerequisite::Archetype(ArchetypeKey::new("inquisitor/sanctified-slayer")),
            ]),
            Prerequisite::ClassLevel {
                class: ClassKey::new("slayer"),
                level: 2,
            },
        ]))
    );
    let status = feat_status(&build("slayer", 1), &general_slot(1), "extra-slayer-talent");
    assert_eq!(
        engine().issue_text(&status.unwrap_err()).as_deref(),
        Some("Needs Slayer level 2 (have 1)")
    );
    // A fighter 1 / slayer 2 takes it at character level 3.
    let slayer_2 = classes(&["fighter", "slayer", "slayer"]);
    assert_eq!(
        feat_status(&slayer_2, &general_slot(3), "extra-slayer-talent"),
        Ok(())
    );
    let rogue = feat_status(&build("rogue", 1), &general_slot(1), "extra-slayer-talent");
    assert!(
        engine()
            .issue_text(&rogue.unwrap_err())
            .unwrap()
            .ends_with("or Slayer level 2 (have 0)"),
    );
}
