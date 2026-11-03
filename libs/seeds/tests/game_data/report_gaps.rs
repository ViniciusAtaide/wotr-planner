//! The extraction report's gaps (ticket 43), and the sorcerer's level-1 feat and Extra Slayer Talent (ticket 44).

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, Build, ClassKey, FeatKey, FeatSlot, FeatureKey, Issue, Pick, Prerequisite,
    SelectionOptionKey, SlotKind, Unmet,
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

// ---- the sorcerer's level-1 feat and Extra Slayer Talent (ticket 44) ------------------------------------------

/// The slot a sorcerer's bloodline feats take: the held bloodline's list.
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

/// Every sorcerer table has "Bonus Feat" at 1, in the column of the "Bloodline Feat" at 7, 13 and 19 (ticket 43's
/// decision 1): the base sorcerer's level-1 bonus feat is from the held bloodline's list.
#[test]
fn a_sorcerer_1_picks_a_bonus_feat_from_the_held_bloodlines_list() {
    let draconic = with_option(
        build("sorcerer", 1),
        1,
        0,
        "sorcerer/bloodline/draconic-red",
    );
    assert!(possible(&draconic));
    assert_eq!(class_slot_kind(&draconic, 1), Some(bloodline_feat_slot()));
    // "Blind-Fight, Great Fortitude, Improved Initiative, Quicken Spell, Power Attack, Skill Focus, Toughness": the
    // Draconic list and nothing else. Its Power Attack is offered but locked at sorcerer 1 (BAB +1).
    let offered = feats_offered(&draconic, &class_slot(1));
    assert_eq!(offered, bloodline_feats(&["draconic"]));
    assert!(!offered.contains("dodge") && !offered.contains("combat-casting"));
    assert!(feat_status(&draconic, &class_slot(1), "power-attack").is_err());
    assert_eq!(feat_status(&draconic, &class_slot(1), "toughness"), Ok(()));
    let mut toughness = draconic.clone();
    toughness
        .picks
        .insert(class_slot(1), Pick::feat(FeatKey::new("toughness")));
    assert!(possible(&toughness));
    // An Abyssal sorcerer 1 is offered the Abyssal list, Power Attack among them, and not Toughness.
    let abyssal = with_option(build("sorcerer", 1), 1, 0, ABYSSAL);
    let offered = feats_offered(&abyssal, &class_slot(1));
    assert_eq!(offered, bloodline_feats(&["abyssal"]));
    assert!(!offered.contains("toughness"));
}

/// "Note that at the bonus feat levels, you may choose a feat from one bloodline or the other, not both."
/// (sorcerer#crossblooded)
#[test]
fn a_crossblooded_sorcerer_1_picks_from_either_bloodlines_list() {
    let build = crossblooded("arcane", "abyssal");
    assert_eq!(class_slot_kind(&build, 1), Some(bloodline_feat_slot()));
    let offered = feats_offered(&build, &class_slot(1));
    assert_eq!(offered, bloodline_feats(&["arcane", "abyssal"]));
    assert!(offered.contains("combat-casting") && offered.contains("cleave"));
    assert!(!offered.contains("toughness"));
}

/// The Empyreal, Sage and Sylvan Sorcerers' fixed bloodlines (Celestial, Arcane, Fey) give their level-1 list.
#[test]
fn a_fixed_bloodline_sorcerer_1_picks_from_its_bloodlines_list() {
    for (archetype, bloodline) in [
        ("sorcerer/empyreal-sorcerer", "celestial"),
        ("sorcerer/sage-sorcerer", "arcane"),
        ("sorcerer/sylvan-sorcerer", "fey"),
    ] {
        let build = archetype_build("sorcerer", archetype, 1);
        assert!(possible(&build), "{archetype}");
        assert_eq!(
            class_slot_kind(&build, 1),
            Some(bloodline_feat_slot()),
            "{archetype}"
        );
        let offered = feats_offered(&build, &class_slot(1));
        assert_eq!(offered, bloodline_feats(&[bloodline]), "{archetype}");
        assert!(!offered.contains("power-attack"), "{archetype}");
    }
}

/// "A Seeker chooses a standard bloodline for the purpose of bonus feats and spells." and "At 5th, 10th and 15th
/// levels, a Seeker gains a bonus magic feat." (sorcerer#seeker-sorcerer): its level-1 feat is the bloodline's, its
/// magic feats come at 5, 10 and 15. A Nine Tailed Heir "chooses a standard bloodline for the purpose of bonus feats".
#[test]
fn a_seeker_and_a_nine_tailed_heir_1_pick_from_the_chosen_bloodlines_list() {
    let seeker = with_option(
        archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 15),
        1,
        0,
        ABYSSAL,
    );
    assert!(possible(&seeker));
    assert_eq!(class_slot_kind(&seeker, 1), Some(bloodline_feat_slot()));
    let offered = feats_offered(&seeker, &class_slot(1));
    assert_eq!(offered, bloodline_feats(&["abyssal"]));
    // No longer any Magic feat: Spell Focus is one, and not on the Abyssal list.
    assert!(!offered.contains("spell-focus"));
    for level in [5, 10, 15] {
        assert_eq!(
            class_slot_kind(&seeker, level),
            Some(SlotKind::Feat(FeatSlot::Category(
                rules::FeatCategory::Magic
            ))),
            "{level}"
        );
    }
    // The Nine Tailed Heir is the kitsune's.
    let mut heir = with_option(
        archetype_build("sorcerer", "sorcerer/nine-tailed-heir", 1),
        1,
        0,
        "sorcerer/bloodline/fey",
    );
    heir.start.race = Some(rules::RaceKey::new("kitsune"));
    assert!(possible(&heir));
    assert_eq!(class_slot_kind(&heir, 1), Some(bloodline_feat_slot()));
    assert_eq!(
        feats_offered(&heir, &class_slot(1)),
        bloodline_feats(&["fey"])
    );
}

/// "Does not gain bloodline spells or bloodline feats." (sorcerer#geomancer): its level-1 bonus feat stays any feat.
/// "Does not receive bonus feats." (sorcerer#overwhelming-mage): no slot at 1.
#[test]
fn a_geomancer_1_still_takes_any_feat_and_an_overwhelming_mage_none() {
    let geomancer = with_option(
        archetype_build("sorcerer", "sorcerer/geomancer", 1),
        1,
        0,
        ABYSSAL,
    );
    assert_eq!(
        class_slot_kind(&geomancer, 1),
        Some(SlotKind::Feat(FeatSlot::General))
    );
    let offered = feats_offered(&geomancer, &class_slot(1));
    assert!(offered.contains("power-attack") && offered.contains("dodge"));
    let mage = with_option(
        archetype_build("sorcerer", "sorcerer/overwhelming-mage", 1),
        1,
        0,
        ABYSSAL,
    );
    assert_eq!(class_slot_kind(&mage, 1), None);
}

/// The sheet names the level-1 slot as it names the 7th's: "Bloodline feat" (a Geomancer's: "Bonus feat").
#[test]
fn the_sheet_labels_a_sorcerers_level_1_feat_bloodline_feat() {
    let engine = engine();
    let label = |build: &Build, level: u8| {
        let sheet = engine.evaluate(build);
        engine.slot_label(sheet.slot(&class_slot(level)).expect("a class feat slot"))
    };
    let base = with_option(build("sorcerer", 20), 1, 0, ABYSSAL);
    assert_eq!(label(&base, 1), "Bloodline feat");
    assert_eq!(label(&base, 7), "Bloodline feat");
    for archetype in [
        "sorcerer/empyreal-sorcerer",
        "sorcerer/sage-sorcerer",
        "sorcerer/sylvan-sorcerer",
    ] {
        assert_eq!(
            label(&archetype_build("sorcerer", archetype, 1), 1),
            "Bloodline feat",
            "{archetype}"
        );
    }
    assert_eq!(
        label(&crossblooded("arcane", "abyssal"), 1),
        "Bloodline feat"
    );
    let geomancer = archetype_build("sorcerer", "sorcerer/geomancer", 1);
    assert_eq!(label(&geomancer, 1), "Bonus feat");
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
