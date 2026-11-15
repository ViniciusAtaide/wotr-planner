//! The kineticist's wild talents (ticket 45).

use std::collections::BTreeSet;

use rules::{
    Build, ClassKey, FeatKey, FeatureKey, Pick, Prerequisite, SelectionOptionKey, SlotId, SlotKind,
};

use crate::support::*;

/// Each wild talent the kineticist picks (the four defensive ones are each element's 2nd-level grant), with its
/// level, its elements (none: Universal or All) and what else the guide's "Prerequisites" names. A level-1 talent's
/// kineticist 2 stands for the game's no level (equivalent model: the first slot is at 2); a talent's elements stand
/// for the game's element features or blast (equivalent model).
const TALENTS: [(&str, u8, &[&str], Option<&str>); 22] = [
    // Game: AerialAdaptationFeature c8719b3c5c0d4694cb13abcc3b7e893b (equivalent model)
    ("aerial-adaptation", 1, &["air"], None),
    // Game: ColdAdaptationFeature 1ff5d6e76b7c2fa48be555b77d1ad8b2 (equivalent model)
    ("cold-adaptation", 1, &["fire", "water"], None),
    // Game: ElementalWhispersSelection f02525006521bee4eb90ab26b7b9db24 (equivalent model)
    ("elemental-whispers", 1, &[], None),
    // Game: HeatAdaptationFeature 2825e3a53c76ad944a47c5c44fb6109f (equivalent model)
    ("heat-adaptation", 1, &["fire", "water"], None),
    // Game: FiresFuryFeature 641d15b6c3d3017409e352c558fb0090 (equivalent model: the Fire blast)
    ("fires-fury", 1, &["fire"], None),
    // Game: FlameShieldFeature 733bbf836ec575b44b3c88b61a26282e: kineticist 10 (level 5), and SearingFlesh
    // 8ad77685e64842c45a6f5b19f9086c6c (`flame_shield_needs_kineticist_10_and_searing_flesh`)
    (
        "flame-shield",
        5,
        &["fire"],
        Some("kineticist/element/fire/searing-flesh"),
    ),
    // Game: KineticHealerFeature 3ef666973adfa8f40af6c0679bd98ba5 (equivalent model)
    ("kinetic-healer", 1, &["water"], None),
    // Game: WildTalentBonusFeatFire 4d14baf0ee4da2a4cb05fb4312921ee4 and the other 15 WildTalentBonusFeat selections,
    // one talent (equivalent model: each needs its element)
    ("kineticist-bonus-feat", 1, &[], None),
    // Game: SkillFocusSelection c9629ef9eebb88b479b2fbc5e836656a (equivalent model)
    ("skill-focus", 1, &[], None),
    // Game: SkilledKineticistFeature fd13e9efee08db448974fe0263eb96c8 (equivalent model)
    ("skilled-kineticist", 1, &[], None),
    // Game: SlickFeature 1d42456e6113739499e1bda025e0ba03 (equivalent model)
    ("slick", 1, &["water"], None),
    // Game: FoxfireFeature 14c699ccd0564e04a9587b1845d16014
    ("foxfire", 2, &["fire"], None),
    // Game: AerialEvasionFeature 871e6b0bb9d050743b98b78990ae1cff
    (
        "aerial-evasion",
        3,
        &["air"],
        Some("kineticist/element/air/enveloping-winds"),
    ),
    // Game: CelerityFeature b182f559f572da342b54bece4404e4e7
    ("celerity", 3, &["air"], None),
    // Game: JaggedFleshFeature 94064ed53b1020247941ac70313b439d (equivalent model: Flesh of Stone implies earth)
    (
        "jagged-flesh",
        3,
        &["earth"],
        Some("kineticist/element/earth/flesh-of-stone"),
    ),
    // Game: KineticRestorationFeature ed01d50910ae67b4dadc050f16d93bdf
    ("kinetic-restoration", 3, &[], None),
    // Game: EnduringEarthFeature 14359e9c35a42ee48b8c31b4424a8d3f
    ("enduring-earth", 4, &["earth"], None),
    // Game: ExpandedDefense d741f298dfae8fc40b4615aaf83b6548
    ("expanded-defense", 4, &[], None),
    // Game: HealingBUrstFeature c73b37aaa2b82b44686c56db8ce14e7f (equivalent model: Kinetic Healer implies water)
    (
        "healing-burst",
        4,
        &["water"],
        Some("kineticist/wild-talents/kinetic-healer"),
    ),
    // Game: KineticRevificationFeature 0377fcf4c10871f4187809d273af7f5d (equivalent model: Kinetic Healer implies
    // water)
    (
        "kinetic-revivification",
        6,
        &["water"],
        Some("kineticist/wild-talents/kinetic-healer"),
    ),
    // Game: Tremorsense 6e668702fdc53c343a0363813683346e
    ("tremor-sense", 6, &["earth"], None),
    // Game: TidalWaveFeature 7c4bbfe3b089a8a4ebcd2401995230a4
    ("tidal-wave", 9, &["water"], None),
];

/// What a talent of `level` with these elements and that other prerequisite needs: the kineticist level twice the
/// talent's, one of its elements, and the other (a Universal talent: the level alone).
fn talent_prerequisite(level: u8, elements: &[&str], other: Option<&str>) -> Prerequisite {
    let element = |e: &&str| {
        Prerequisite::SelectionOption(SelectionOptionKey::new(format!("kineticist/element/{e}")))
    };
    let mut all = vec![Prerequisite::ClassLevel {
        class: ClassKey::new("kineticist"),
        level: 2 * level,
    }];
    match elements {
        [] => {}
        [one] => all.push(element(one)),
        several => all.push(Prerequisite::Any(several.iter().map(element).collect())),
    }
    if let Some(other) = other {
        all.push(match other.strip_prefix("kineticist/wild-talents/") {
            Some(_) => Prerequisite::SelectionOption(SelectionOptionKey::new(other)),
            None => Prerequisite::Feature {
                feature: FeatureKey::new(other),
                count: 1,
            },
        });
    }
    let mut prerequisite = match all.len() {
        1 => all.pop().unwrap(),
        _ => Prerequisite::All(all),
    };
    prerequisite.canonicalize();
    prerequisite
}

/// The wild talents a slot leaves open, by key.
fn open_talents(build: &Build, level: u8) -> BTreeSet<String> {
    offered(build, level)
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .collect()
}

fn talent_reason(build: &Build, level: u8, name: &str) -> String {
    let status = offered(build, level)
        .remove(&talent(name))
        .unwrap_or_else(|| panic!("{name} isn't offered at {level}"));
    engine().issue_text(&status.expect_err("locked")).unwrap()
}

fn bonus_feat_slot(level: u8) -> SlotId {
    SlotId::OptionFeat { level, index: 0 }
}

/// An element's five wild-talent bonus feats, by key.
fn element_feats(elements: &[&str]) -> BTreeSet<String> {
    elements
        .iter()
        .flat_map(|element| {
            let key = format!("kineticist/element/{element}/wild-talent-feats");
            class("kineticist")
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

/// "At 2nd level and every subsequent even-numbered level, the Kineticist selects a Utility Wild Talent." The guide's
/// 26 talents less the four defensive ones, each with its level and element conditions.
#[test]
fn wild_talents_are_a_selection_of_22_talents_with_their_level_and_elements() {
    let kineticist = class("kineticist");
    let selection = kineticist
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == WILD_TALENTS)
        .expect("the wild talents selection");
    // Game: WildTalentSelection 5c883ae0cd6d7d5448b7a420f51f8459: 20 talents, Skill Focus and 16 bonus-feat
    // selections (equivalent model: one bonus-feat talent).
    assert_eq!(selection.name, "Wild Talents");
    let keys: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    let expected: BTreeSet<String> = TALENTS.iter().map(|(name, ..)| talent(name)).collect();
    assert_eq!(keys, expected);
    // Game: each row's blueprint in TALENTS (Flame Shield's is `flame_shield_needs_kineticist_10_and_searing_flesh`'s).
    for (name, level, elements, other) in TALENTS {
        if name == "flame-shield" {
            continue;
        }
        assert_eq!(
            selection_option(&talent(name)).prerequisite,
            Some(talent_prerequisite(level, elements, other)),
            "{name}"
        );
    }
    // No feature stands for the pick any more.
    assert!(
        kineticist
            .features
            .iter()
            .all(|feature| feature.key.as_str() != "kineticist/wild-talents")
    );
    let levels: Vec<u8> = kineticist
        .levels
        .iter()
        .filter(|row| row.selections.iter().any(|s| s.as_str() == WILD_TALENTS))
        .map(|row| row.level)
        .collect();
    // Game: KineticistProgression b79e92dd495edd64e90fb483c504b8df: WildTalentSelection
    // 5c883ae0cd6d7d5448b7a420f51f8459 at every even level.
    assert_eq!(levels, [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
}

/// The game's Flame Shield needs kineticist 10 as well as Searing Flesh: a Kinetic Knight, who holds Searing Flesh from
/// 4, takes it from 10, and a fire kineticist 2 can't.
#[test]
#[ignore = "waits on ticket 271: the game gives Flame Shield kineticist 10"]
fn flame_shield_needs_kineticist_10_and_searing_flesh() {
    let (name, level, elements, other) = TALENTS[5];
    assert_eq!(name, "flame-shield");
    // Game: FlameShieldFeature 733bbf836ec575b44b3c88b61a26282e: PrerequisiteClassLevel KineticistClass 10 and
    // SearingFlesh 8ad77685e64842c45a6f5b19f9086c6c.
    assert_eq!(
        selection_option(&talent(name)).prerequisite,
        Some(talent_prerequisite(level, elements, other))
    );
    assert!(!open_talents(&kineticist("fire", 2), 2).contains(&talent("flame-shield")));
    // Game: KineticKnightElementalFocusFire d4a2a75d01d1e77489ff692636a538bf gives SearingFlesh at 4.
    let knight = with_option(
        archetype_build("kineticist", "kineticist/kinetic-knight", 10),
        1,
        0,
        "kineticist/element/fire",
    );
    assert!(!open_talents(&knight, 8).contains(&talent("flame-shield")));
    assert!(open_talents(&knight, 10).contains(&talent("flame-shield")));
}

/// The ticket's: a kineticist 2 picks a talent of level 1 at most, from its element's and the Universal ones (Flame
/// Shield, which the game gives kineticist 10, is `flame_shield_needs_kineticist_10_and_searing_flesh`'s).
#[test]
fn a_fire_kineticist_2_is_offered_its_elements_and_universal_level_1_talents() {
    let build = kineticist("fire", 2);
    assert!(possible(&build));
    // Game: each talent's blueprint in TALENTS (equivalent model: the fire element).
    let open: BTreeSet<String> = [
        "cold-adaptation",
        "elemental-whispers",
        "heat-adaptation",
        "fires-fury",
        "kineticist-bonus-feat",
        "skill-focus",
        "skilled-kineticist",
    ]
    .iter()
    .map(|name| talent(name))
    .collect();
    let mut open_now = open_talents(&build, 2);
    open_now.remove(&talent("flame-shield"));
    assert_eq!(open_now, open);
    // A Water-only talent is locked, by its element. Game: KineticHealerFeature 3ef666973adfa8f40af6c0679bd98ba5.
    assert_eq!(
        talent_reason(&build, 2, "kinetic-healer"),
        "Needs Elemental Focus - Water"
    );
    // Foxfire is a level-2 Fire talent: from kineticist 4. Game: FoxfireFeature 14c699ccd0564e04a9587b1845d16014.
    assert_eq!(
        talent_reason(&build, 2, "foxfire"),
        "Needs Kineticist level 4 (have 2)"
    );
    // Game: KineticHealerFeature 3ef666973adfa8f40af6c0679bd98ba5; FiresFuryFeature 641d15b6c3d3017409e352c558fb0090.
    let water = kineticist("water", 2);
    assert!(open_talents(&water, 2).contains(&talent("kinetic-healer")));
    assert!(!open_talents(&water, 2).contains(&talent("fires-fury")));
}

/// "The Kineticist's class level must be at least twice the level of the Wild Talent to unlock it."
#[test]
fn a_level_3_talent_opens_at_kineticist_6() {
    let build = kineticist("air", 6);
    // Game: CelerityFeature b182f559f572da342b54bece4404e4e7: kineticist 6.
    assert_eq!(
        talent_reason(&build, 4, "celerity"),
        "Needs Kineticist level 6 (have 4)"
    );
    assert!(open_talents(&build, 6).contains(&talent("celerity")));
    let celerity = kineticist_with("air", 6, &[(6, &talent("celerity"))]);
    assert!(possible(&celerity));
    let early = kineticist_with("air", 6, &[(4, &talent("celerity"))]);
    assert!(!possible(&early));
}

/// Flame Shield needs Searing Flesh, Fire's defensive talent: a Kinetic Knight has it from 4, not 2 (the game's
/// kineticist 10 is `flame_shield_needs_kineticist_10_and_searing_flesh`'s).
#[test]
fn a_talent_needing_a_defensive_talent_waits_for_it() {
    let knight = with_option(
        archetype_build("kineticist", "kineticist/kinetic-knight", 4),
        1,
        0,
        "kineticist/element/fire",
    );
    // Game: FlameShieldFeature 733bbf836ec575b44b3c88b61a26282e needs SearingFlesh 8ad77685e64842c45a6f5b19f9086c6c,
    // which KineticKnightElementalFocusFire d4a2a75d01d1e77489ff692636a538bf gives at 4.
    assert_eq!(
        talent_reason(&knight, 2, "flame-shield"),
        "Needs Searing Flesh"
    );
    // Healing Burst needs Kinetic Healer. Game: HealingBUrstFeature c73b37aaa2b82b44686c56db8ce14e7f.
    let water = kineticist_with("water", 8, &[(2, &talent("kinetic-healer"))]);
    assert!(open_talents(&water, 8).contains(&talent("healing-burst")));
    assert_eq!(
        talent_reason(&kineticist("water", 8), 8, "healing-burst"),
        "Needs Kinetic Healer"
    );
}

/// "Select a bonus feat associated with an Elemental Focus. You do not need to meet the normal prerequisites for the
/// feat you select." One slot, from the held elements' lists (the game's chain of lists is
/// `the_first_bonus_feat_talent_offers_its_elements_entry_feats_and_the_talent_repeats_down_the_chain`'s).
#[test]
fn the_bonus_feat_talent_grants_one_slot_from_the_held_elements_lists() {
    let build = kineticist_with("fire", 2, &[(2, &talent("kineticist-bonus-feat"))]);
    assert!(possible(&build));
    // Game: WildTalentBonusFeatFire 4d14baf0ee4da2a4cb05fb4312921ee4, WildTalentBonusFeatFire1
    // 0e0207491a09a9e428409c4a1b2871a3, WildTalentBonusFeatFire2 cd107bf355f84b64f9f472ca288c208b,
    // WildTalentBonusFeatFire3 0213c0c9062203540bd0365cbde44b99: their lists' union.
    assert_eq!(
        feats_offered(&build, &bonus_feat_slot(2)),
        element_feats(&["fire"])
    );
    // With Expanded Element (Water) at 7, the talent at 8 offers both elements' lists. Game: WildTalentBonusFeatWater
    // 31c18eae013b09c4f9ee51da71a2d61c needs SecondaryElementWater faa5f1233600d864fa998bc0afe351ab.
    let expanded = kineticist_with(
        "fire",
        8,
        &[
            (7, "kineticist/element/water"),
            (8, &talent("kineticist-bonus-feat")),
        ],
    );
    assert!(possible(&expanded));
    assert_eq!(
        feats_offered(&expanded, &bonus_feat_slot(8)),
        element_feats(&["fire", "water"])
    );
    // A feat off the lists isn't allowed. Game: the WildTalentBonusFeat selections hold only their lists.
    let mut off = build.clone();
    off.picks
        .insert(bonus_feat_slot(2), Pick::feat(FeatKey::new("power-attack")));
    assert!(!possible(&off));
}

/// The game's bonus-feat talent is a chain of selections per element (four for air, earth and fire, six for water),
/// each open by the feats held: the first pick offers only the entry feats (fire: Iron Will, Persuasive, Dazzling
/// Display, not Improved Iron Will), and the talent is taken again while a next list is open.
#[test]
#[ignore = "waits on ticket 271: the game gives the bonus-feat talent as a per-element chain of lists, \
            repeatable"]
fn the_first_bonus_feat_talent_offers_its_elements_entry_feats_and_the_talent_repeats_down_the_chain()
 {
    let build = kineticist_with("fire", 2, &[(2, &talent("kineticist-bonus-feat"))]);
    // Game: WildTalentBonusFeatFire 4d14baf0ee4da2a4cb05fb4312921ee4 offers IronWill 175d1577bb6c9a04baf88eec99c66334,
    // Persuasive 86d93a5891d299d4983bdc6ef3987afd and DazzlingDisplayFeature bcbd674ec70ff6f4894bb5f07b6f4095; only
    // WildTalentBonusFeatFire2 cd107bf355f84b64f9f472ca288c208b and Fire3 0213c0c9062203540bd0365cbde44b99, which need
    // IronWill held, offer IronWillImproved 3ea2215150a1c8a4a9bfed9d9023903e.
    assert!(feat_status(&build, &bonus_feat_slot(2), "improved-iron-will").is_err());
    assert_eq!(
        feat_status(&build, &bonus_feat_slot(2), "iron-will"),
        Ok(())
    );
    // Game: each WildTalentBonusFeat selection has Ranks 1, but WildTalentBonusFeatFire2 opens once IronWill is held.
    let mut twice = kineticist_with("fire", 4, &[(2, &talent("kineticist-bonus-feat"))]);
    twice
        .picks
        .insert(bonus_feat_slot(2), Pick::feat(FeatKey::new("iron-will")));
    assert_eq!(
        offered(&twice, 4).remove(&talent("kineticist-bonus-feat")),
        Some(Ok(()))
    );
}

/// "You unlock the Defensive Wild Talent for any element you selected with Expanded Element."
#[test]
fn expanded_defense_unlocks_the_expanded_elements_defensive_talent() {
    // Game: SecondaryElementAir 659c39542b728c04b83e969c834782a9: its level-2 row is empty; AerialEvasionFeature
    // 871e6b0bb9d050743b98b78990ae1cff needs EnvelopingWinds bb0de2047c448bd46aff120be3b39b7a.
    let without = kineticist_with("fire", 10, &[(7, "kineticist/element/air")]);
    assert!(!holds(
        &at(&without, 10),
        "kineticist/element/air/enveloping-winds"
    ));
    assert_eq!(
        talent_reason(&without, 10, "aerial-evasion"),
        "Needs Enveloping Winds"
    );
    let with = kineticist_with(
        "fire",
        10,
        &[
            (7, "kineticist/element/air"),
            (8, &talent("expanded-defense")),
        ],
    );
    assert!(possible(&with));
    // Game: ExpandedDefense d741f298dfae8fc40b4615aaf83b6548, a pick of one defensive talent whose element is held:
    // here only EnvelopingWinds bb0de2047c448bd46aff120be3b39b7a (equivalent model).
    from_level(&with, 8, "kineticist/element/air/enveloping-winds");
    assert!(open_talents(&with, 10).contains(&talent("aerial-evasion")));
}

/// The game's Expanded Defense picks one defensive talent when taken: an element taken at 15, after the talent, gets
/// no defensive talent.
#[test]
#[ignore = "waits on ticket 271: the game gives Expanded Defense one defensive talent, none for a later \
            element"]
fn expanded_defense_gives_no_defensive_talent_to_an_element_taken_after_it() {
    let later = kineticist_with(
        "fire",
        15,
        &[
            (7, "kineticist/element/air"),
            (8, &talent("expanded-defense")),
            (15, "kineticist/element/water"),
        ],
    );
    assert!(possible(&later));
    // Game: ExpandedDefense d741f298dfae8fc40b4615aaf83b6548 (Ranks 1, a selection of one defensive talent);
    // SecondaryElementWater faa5f1233600d864fa998bc0afe351ab grants no ShroudOfWater 29ec36fa2a5b8b94ebce170bd369083a.
    assert!(!holds(
        &at(&later, 15),
        "kineticist/element/water/shroud-of-water"
    ));
}

/// The Skill Focus talent: "Identical to the feat", for a skill of the player's choosing.
#[test]
fn the_skill_focus_talent_is_skill_focus_in_a_skill() {
    let mut build = kineticist("earth", 2);
    build.picks.insert(
        selection_slot(2),
        Pick::option_for(
            SelectionOptionKey::new(talent("skill-focus")),
            skill("perception"),
        ),
    );
    assert!(possible(&build));
    // Game: WildTalentSelection 5c883ae0cd6d7d5448b7a420f51f8459 offers SkillFocusSelection
    // c9629ef9eebb88b479b2fbc5e836656a itself.
    assert!(at(&build, 2).has_feat_for(&FeatKey::new("skill-focus"), &skill("perception")));
}

/// The archetypes keep the wild talents at every even level: the Kinetic Knight's 4th and the Blood Kineticist's
/// 20th rows, which differ from the base's, pick one too.
#[test]
fn every_kineticist_archetype_picks_wild_talents_at_even_levels() {
    // Game: KineticistProgression b79e92dd495edd64e90fb483c504b8df; no archetype of KineticistClass
    // 42a455d9ec1ad924d889272429eb8391 removes WildTalentSelection 5c883ae0cd6d7d5448b7a420f51f8459.
    for archetype in &class("kineticist").archetypes {
        let build = archetype_build("kineticist", archetype.key.as_str(), 20);
        let sheet = engine().evaluate(&build);
        let levels: Vec<u8> = (1..=20)
            .filter(|&level| {
                sheet.slot(&selection_slot(level)).is_some_and(|slot| {
                    slot.kind == SlotKind::Selection(rules::FeatureSelectionKey::new(WILD_TALENTS))
                })
            })
            .collect();
        assert_eq!(
            levels,
            [2, 4, 6, 8, 10, 12, 14, 16, 18, 20],
            "{}",
            archetype.key
        );
    }
}

/// The sheet names the wild talent slot after the selection, "Wild Talents", and the bonus-feat talent's slot "Wild
/// talent feat" (ticket 47; the user's name for it, not the Elemental Focus's whose lists it offers).
#[test]
fn the_sheet_labels_the_wild_talent_and_its_bonus_feat() {
    let engine = engine();
    let build = kineticist_with(
        "fire",
        20,
        &[
            (2, &talent("kineticist-bonus-feat")),
            (7, "kineticist/element/water"),
        ],
    );
    assert!(possible(&build));
    let sheet = engine.evaluate(&build);
    let label = |slot: &SlotId| engine.slot_label(sheet.slot(slot).expect("a slot"));
    // Game: WildTalentSelection 5c883ae0cd6d7d5448b7a420f51f8459.
    assert_eq!(label(&selection_slot(2)), "Wild Talents");
    assert_eq!(label(&selection_slot(20)), "Wild Talents");
    assert_eq!(label(&bonus_feat_slot(2)), "Wild talent feat");
    for slot in sheet.levels.iter().flat_map(|level| &level.slots) {
        let label = engine.slot_label(slot);
        assert!(!label.is_empty() && !label.contains('/'), "{label}");
    }
}

/// "Prerequisites: Wild Talents" (feats#extra-wild-talent): the feature the even rows held is a pick now, so the
/// feat needs the level of the first one, kineticist 2, as Extra Slayer Talent needs slayer 2.
#[test]
fn extra_wild_talent_needs_kineticist_level_2() {
    // Game: ExtraWildTalentSelection 0f73730cf1b44ee882671e55d5f6e471 needs WildTalentSelection, first held at
    // kineticist 2 (equivalent model).
    assert_eq!(
        feat("extra-wild-talent").prerequisite,
        Some(Prerequisite::ClassLevel {
            class: ClassKey::new("kineticist"),
            level: 2,
        })
    );
    let status = feat_status(
        &kineticist("fire", 1),
        &general_slot(1),
        "extra-wild-talent",
    );
    assert_eq!(
        engine().issue_text(&status.unwrap_err()).as_deref(),
        Some("Needs Kineticist level 2 (have 1)")
    );
    assert_eq!(
        feat_status(
            &kineticist("fire", 3),
            &general_slot(3),
            "extra-wild-talent"
        ),
        Ok(())
    );
}
