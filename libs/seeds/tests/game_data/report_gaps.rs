//! The extraction report's gaps (ticket 43), the sorcerer's level-1 feat (tickets 44 and 92) and Extra Slayer Talent
//! (ticket 44).

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, Build, ClassKey, FeatKey, FeatSlot, FeatureKey, Issue, Pick, Prerequisite,
    SelectionOptionKey, SkillKey, SlotKind, Unmet,
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
    // Game: each Lesser totem (BeastTotemLesserFeature d99dfc9238a8a6646b32be09057c1729, CelestialTotemLesserFeature
    // aba61e0b0e66bf3439cc247ee89fddae, DaemonTotemLesserFeature 45102fd7aab96f94d81ec80768549e12,
    // FiendTotemLesserFeature 76437492f801f054ba536473ad2fde79) rules out the other Lessers, and each line's plain and
    // Greater power needs its own Lesser (equivalent model: the same legal picks).
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
/// The game's Eldritch Scoundrel keeps the rogue's own Sneak Attack at those levels.
#[test]
#[ignore = "waits on ticket 276: the game gives the Eldritch Scoundrel the rogue's own Sneak Attack"]
fn an_eldritch_scoundrel_holds_the_rogues_own_sneak_attack_at_1_3_7_11_15_and_19() {
    // Game: EldritchScoundrelArchetype 57f93dd8423c97c49989501281296c4a removes SneakAttack
    // 9b9eac6709e1c084cb18c3a366e0ec87 at 5, 9, 13 and 17 and keeps RogueProgression
    // b57b2a75a5abcaf47a01cf84672b50e9's others (1, 3, 7, 11, 15, 19).
    let sheet = engine().evaluate(&archetype_build("rogue", "rogue/eldritch-scoundrel", 20));
    let dice: Vec<u8> = (1..=20)
        .map(|level| {
            sheet
                .at(level)
                .unwrap()
                .feature_count(&FeatureKey::new("rogue/sneak-attack"))
        })
        .collect();
    assert_eq!(
        dice,
        [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6]
    );
}

/// "At 6th and 14th levels, a Shaman gains a hex slot that can be temporarily filled with a hex granted by either
/// his main spirit or his current Wandering Spirit." (shaman#wandering-hex): a daily choice, as the Wandering Spirit
/// it draws on.
#[test]
fn a_shamans_wandering_hex_is_a_daily_choice_from_6() {
    // Game: ShamanProgression d04a543c82b45694c805301bce9dac24 gives ShamanWanderingHexFeature
    // 2674f98616188814fbba9f8fccf43564 at 6 and ShamanWanderingHexFeature2 f15d95a394e950a42b1d6afbb9f781d9 at 14;
    // PossessedShamanArchetype 7939ff57cc2ce104693ea2f5afa595b6 removes the 6th's.
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
/// Geomancer's General slot. Dodge, Power Attack and Toughness, on some bloodline lists, aren't Magic feats. Which
/// feats the slot offers waits in `every_sorcerers_level_1_bonus_feat_is_one_of_the_games_21`.
#[test]
fn every_sorcerers_level_1_bonus_feat_is_a_magic_feat() {
    // Game: SorcererProgression 997665565ca80a649aedd72455c4df1f gives SorcererBonusFeat
    // d6dd06f454b34014ab0903cb1ed2ade3 at 1, which holds the 21 below (SpellPenetration
    // ee7dc126939e4d9438357fbd5980d459 among them) and no Dodge, Power Attack, Toughness or Improved Initiative.
    for (name, build, _) in sorcerers_with_a_bonus_feat(1) {
        assert!(possible(&build), "{name}");
        let offered = feats_offered(&build, &class_slot(1));
        assert!(
            SORCERER_BONUS_FEATS
                .iter()
                .all(|feat| offered.contains(*feat)),
            "{name}"
        );
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

/// The game's sorcerer bonus feat at 1 (the user approved the game's list over ticket 92's, 2026-11-15).
/// Game: SorcererBonusFeat d6dd06f454b34014ab0903cb1ed2ade3's 21 feats.
const SORCERER_BONUS_FEATS: [&str; 21] = [
    "augment-summoning",
    "combat-casting",
    "elemental-focus",
    "greater-elemental-focus",
    "greater-spell-focus",
    "greater-spell-penetration",
    "metamagic-bolster-spell",
    "metamagic-empower-spell",
    "metamagic-extend-spell",
    "metamagic-heighten-spell",
    "metamagic-intensified-spell",
    "metamagic-maximise-spell",
    "metamagic-persistent-spell",
    "metamagic-piercing-spell",
    "metamagic-quicken-spell",
    "metamagic-reach-spell",
    "metamagic-selective-spell",
    "spell-focus",
    "spell-penetration",
    "spell-specialization",
    "superior-summoning",
];

/// The game's sorcerer level-1 bonus feat offers the 21 feats of its own selection, not every Magic feat.
#[test]
#[ignore = "waits on ticket 281: the game gives the sorcerer's level-1 bonus feat 21 feats"]
fn every_sorcerers_level_1_bonus_feat_is_one_of_the_games_21() {
    for (name, build, _) in sorcerers_with_a_bonus_feat(1) {
        let offered = feats_offered(&build, &class_slot(1));
        assert_eq!(
            offered,
            SORCERER_BONUS_FEATS.map(String::from).into(),
            "{name}"
        );
    }
}

/// The game's Seeker picks its feats at 5, 10 and 15 from its own selection of 26.
#[test]
#[ignore = "waits on ticket 281: the game gives the Seeker's feats at 5, 10 and 15 a list of 26"]
fn the_seekers_feats_at_5_10_and_15_are_its_own_26() {
    // Game: SeekerArchetype 7229db6fc0b07af4180e783eed43c4d9 adds SeekerFeatSelection c6b609279cc3174478624182ac1ad812
    // at 5, 10 and 15: the 21 above with Ambuscading Spells, Arcane Spirit, Dispel Focus, Greater Dispel Focus and
    // Spell Chain.
    let mut expected: BTreeSet<String> = SORCERER_BONUS_FEATS.map(String::from).into();
    expected.extend(
        [
            "ambuscading-spells",
            "arcane-spirit",
            "dispel-focus",
            "greater-dispel-focus",
            "spell-chain",
        ]
        .map(String::from),
    );
    let seeker = with_option(
        archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 15),
        1,
        0,
        ABYSSAL,
    );
    for level in [5, 10, 15] {
        assert_eq!(
            feats_offered(&seeker, &class_slot(level)),
            expected,
            "{level}"
        );
    }
}

/// The Jack-in-the-Green write-up's level 1 (ticket 92): "Bonus Feat - Spell Penetration; Mounted Combat", a Sylvan
/// Sorcerer 1 with Spell Penetration in the bonus slot and Mounted Combat (Mobility 1 rank) in its general slot.
#[test]
fn a_sylvan_sorcerer_1_takes_spell_penetration_and_mounted_combat() {
    // Game: SorcererBonusFeat d6dd06f454b34014ab0903cb1ed2ade3 holds SpellPenetration
    // ee7dc126939e4d9438357fbd5980d459 (SylvanSorcererArchetype 711d5024ecc75f346b9cda609c3a1f83 keeps it);
    // MountedCombat f308a03bea0d69843a8ed0af003d47a9 needs 1 rank of Mobility.
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
    // Game: OverwhelimgMageArchetype 734917ec8b8542378c1d35519439d67f removes SorcererBonusFeat and the feats at 7, 13
    // and 19, and adds Overwhelming Penetration at 1.
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
    // Game: SorcererProgression 997665565ca80a649aedd72455c4df1f gives SorcererFeatSelection
    // 3a60f0c0442acfb419b0c03b584e1394 at 7, 13 and 19; GeomancerArchetype a0e56a59ad0b44b8add84185da6bb845 removes
    // them. Each bloodline's list was not compared with the game here.
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
    // Game: SeekerArchetype 7229db6fc0b07af4180e783eed43c4d9 adds a feat at 5, 10 and 15; which feats waits in
    // the_seekers_feats_at_5_10_and_15_are_its_own_26.
    for level in [5, 10, 15] {
        assert!(class_slot_kind(&seeker, level).is_some(), "{level}");
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
    // Game: ExtraSlayerTalentSelection 53f7237f5b1447bb851ba68045a00e41 needs SlayerTalentSelection2
    // 04430ad24988baa4daa0bcd4f1c7d118 (SlayerProgression 1f35d11937a06624b8a60db68c7869a1 at 2) or
    // SanctifiedSlayerSlayerTalentFeature db3767ba22ce84942af56525cb9f9ce6 (SanctifiedSlayerArchetype
    // 19b4a243d9a4bd34887d03c3dea03ced at inquisitor 8) (equivalent model: the levels that grant them).
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

// ---- ticket 192: labels and prerequisite texts ----------------------------------------------------------------

/// The ranger's selection is "Combat Style" (it read "Combat Style Feat", and its feat slots "Combat Style Feat
/// feat"): the level-2 slot that picks the style says so, and the feat slots it gives read "Combat Style feat".
#[test]
fn the_rangers_combat_style_slots_read_combat_style_and_combat_style_feat() {
    let engine = engine();
    let build = build("ranger", 20);
    let sheet = engine.evaluate(&build);
    let label = |slot: &rules::SlotId| engine.slot_label(sheet.slot(slot).expect("a ranger slot"));
    assert_eq!(label(&selection_slot(2)), "Combat Style");
    for level in [2, 6, 10, 14, 18] {
        assert_eq!(label(&class_slot(level)), "Combat Style feat", "{level}");
    }
}

/// The Dragon Disciple's first alternative is "no Bloodline yet" once, not once for each of the Sorcerer's and the
/// Bloodrager's selections (which share a name): in `search`'s one line, in `describe`, and in a refusal.
#[test]
fn the_dragon_disciples_no_bloodline_yet_reads_once() {
    let engine = engine();
    let prerequisite = class("dragon-disciple")
        .prerequisite
        .as_ref()
        .expect("a prerequisite");
    let whole = engine.prerequisite_text(prerequisite, None);
    assert!(
        whole.starts_with("(no Bloodline yet or Draconic (Black) or Draconic (Blue) or "),
        "{whole}"
    );
    assert_eq!(whole.matches("no Bloodline yet").count(), 1, "{whole}");
    // A sorcerer with an Abyssal bloodline is refused the class with it said once too.
    let sorcerer = with_option(build("sorcerer", 6), 1, 0, ABYSSAL);
    let option = engine
        .class_options(&sorcerer, 7)
        .expect("the next level")
        .into_iter()
        .find(|option| option.class == ClassKey::new("dragon-disciple"))
        .expect("the class is offered");
    let reason = engine
        .issue_text(&option.status.expect_err("a locked class"))
        .expect("a reason");
    assert!(
        reason.starts_with("Needs (no Bloodline yet or Draconic (Black) or "),
        "{reason}"
    );
    assert_eq!(reason.matches("no Bloodline yet").count(), 1, "{reason}");
}

/// The game's Dragon Disciple asks for no character level: its prerequisite reads without one.
#[test]
#[ignore = "waits on ticket 287: the game gives the Dragon Disciple no character-level prerequisite"]
fn the_dragon_disciples_prerequisite_reads_without_a_character_level() {
    let prerequisite = class("dragon-disciple")
        .prerequisite
        .as_ref()
        .expect("a prerequisite");
    // `search` shortens a group of more than three alternatives (`MOST_ALTERNATIVES` in the MCP server).
    // Game: DragonDiscipleClass 72051275b1dbb2d42ba9118237794f7c: 5 ranks of Knowledge (Arcana) and 1st-level arcane
    // spells cast spontaneously, no character level.
    assert_eq!(
        engine().short_prerequisite_text(prerequisite, 3),
        "(one of 11 alternatives (no Bloodline yet, Draconic (Black), Draconic (Blue), …)), 5 ranks in Knowledge \
         (Arcana), to cast 1st-level arcane spells spontaneously"
    );
}
