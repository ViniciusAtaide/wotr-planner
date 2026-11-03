//! The kineticist's wild talents (ticket 45).

use std::collections::BTreeSet;

use rules::{
    Build, ClassKey, FeatKey, FeatureKey, Issue, Pick, Prerequisite, SelectionOptionKey, SlotId,
    SlotKind,
};

use crate::support::*;

/// Each wild talent the kineticist picks (the four defensive ones are each element's 2nd-level grant), with its
/// level, its elements (none: Universal or All) and what else the guide's "Prerequisites" names.
const TALENTS: [(&str, u8, &[&str], Option<&str>); 22] = [
    ("aerial-adaptation", 1, &["air"], None),
    ("cold-adaptation", 1, &["fire", "water"], None),
    ("elemental-whispers", 1, &[], None),
    ("heat-adaptation", 1, &["fire", "water"], None),
    ("fires-fury", 1, &["fire"], None),
    (
        "flame-shield",
        1,
        &["fire"],
        Some("kineticist/element/fire/searing-flesh"),
    ),
    ("kinetic-healer", 1, &["water"], None),
    ("kineticist-bonus-feat", 1, &[], None),
    ("skill-focus", 1, &[], None),
    ("skilled-kineticist", 1, &[], None),
    ("slick", 1, &["water"], None),
    ("foxfire", 2, &["fire"], None),
    (
        "aerial-evasion",
        3,
        &["air"],
        Some("kineticist/element/air/enveloping-winds"),
    ),
    ("celerity", 3, &["air"], None),
    (
        "jagged-flesh",
        3,
        &["earth"],
        Some("kineticist/element/earth/flesh-of-stone"),
    ),
    ("kinetic-restoration", 3, &[], None),
    ("enduring-earth", 4, &["earth"], None),
    ("expanded-defense", 4, &[], None),
    (
        "healing-burst",
        4,
        &["water"],
        Some("kineticist/wild-talents/kinetic-healer"),
    ),
    (
        "kinetic-revivification",
        6,
        &["water"],
        Some("kineticist/wild-talents/kinetic-healer"),
    ),
    ("tremor-sense", 6, &["earth"], None),
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
    assert_eq!(selection.name, "Wild Talents");
    let keys: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    let expected: BTreeSet<String> = TALENTS.iter().map(|(name, ..)| talent(name)).collect();
    assert_eq!(keys, expected);
    for (name, level, elements, other) in TALENTS {
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
    assert_eq!(levels, [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
}

/// The ticket's: a kineticist 2 picks a talent of level 1 at most, from its element's and the Universal ones.
#[test]
fn a_fire_kineticist_2_is_offered_its_elements_and_universal_level_1_talents() {
    let build = kineticist("fire", 2);
    assert!(possible(&build));
    let open: BTreeSet<String> = [
        "cold-adaptation",
        "elemental-whispers",
        "heat-adaptation",
        "fires-fury",
        "flame-shield",
        "kineticist-bonus-feat",
        "skill-focus",
        "skilled-kineticist",
    ]
    .iter()
    .map(|name| talent(name))
    .collect();
    assert_eq!(open_talents(&build, 2), open);
    // A Water-only talent is locked, by its element.
    assert_eq!(
        talent_reason(&build, 2, "kinetic-healer"),
        "Needs Elemental Focus - Water"
    );
    // Foxfire is a level-2 Fire talent: from kineticist 4.
    assert_eq!(
        talent_reason(&build, 2, "foxfire"),
        "Needs Kineticist level 4 (have 2)"
    );
    let water = kineticist("water", 2);
    assert!(open_talents(&water, 2).contains(&talent("kinetic-healer")));
    assert!(!open_talents(&water, 2).contains(&talent("fires-fury")));
}

/// "The Kineticist's class level must be at least twice the level of the Wild Talent to unlock it."
#[test]
fn a_level_3_talent_opens_at_kineticist_6() {
    let build = kineticist("air", 6);
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

/// Flame Shield needs Searing Flesh, Fire's defensive talent: a Kinetic Knight has it from 4, not 2.
#[test]
fn a_talent_needing_a_defensive_talent_waits_for_it() {
    let knight = with_option(
        archetype_build("kineticist", "kineticist/kinetic-knight", 4),
        1,
        0,
        "kineticist/element/fire",
    );
    assert_eq!(
        talent_reason(&knight, 2, "flame-shield"),
        "Needs Searing Flesh"
    );
    assert!(open_talents(&knight, 4).contains(&talent("flame-shield")));
    // Healing Burst needs Kinetic Healer.
    let water = kineticist_with("water", 8, &[(2, &talent("kinetic-healer"))]);
    assert!(open_talents(&water, 8).contains(&talent("healing-burst")));
    assert_eq!(
        talent_reason(&kineticist("water", 8), 8, "healing-burst"),
        "Needs Kinetic Healer"
    );
}

/// "Select a bonus feat associated with an Elemental Focus. You do not need to meet the normal prerequisites for the
/// feat you select." One slot, from the held elements' lists, taken once.
#[test]
fn the_bonus_feat_talent_grants_one_slot_from_the_held_elements_lists() {
    let build = kineticist_with("fire", 2, &[(2, &talent("kineticist-bonus-feat"))]);
    assert!(possible(&build));
    assert_eq!(
        feats_offered(&build, &bonus_feat_slot(2)),
        element_feats(&["fire"])
    );
    // Improved Iron Will needs Iron Will, but the list ignores prerequisites.
    assert_eq!(
        feat_status(&build, &bonus_feat_slot(2), "improved-iron-will"),
        Ok(())
    );
    let mut with_feat = build.clone();
    with_feat.picks.insert(
        bonus_feat_slot(2),
        Pick::feat(FeatKey::new("improved-iron-will")),
    );
    assert!(possible(&with_feat));
    assert!(
        at(&with_feat, 2)
            .feats
            .contains_key(&FeatKey::new("improved-iron-will"))
    );
    // Not twice.
    let twice = kineticist_with("fire", 4, &[(2, &talent("kineticist-bonus-feat"))]);
    assert_eq!(
        offered(&twice, 4).remove(&talent("kineticist-bonus-feat")),
        Some(Err(Issue::AlreadyHeld))
    );
    // With Expanded Element (Water) at 7, the talent at 8 offers both elements' lists.
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
    // A feat off the lists isn't allowed.
    let mut off = build.clone();
    off.picks
        .insert(bonus_feat_slot(2), Pick::feat(FeatKey::new("power-attack")));
    assert!(!possible(&off));
}

/// "You unlock the Defensive Wild Talent for any element you selected with Expanded Element."
#[test]
fn expanded_defense_unlocks_the_expanded_elements_defensive_talent() {
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
    from_level(&with, 8, "kineticist/element/air/enveloping-winds");
    assert!(open_talents(&with, 10).contains(&talent("aerial-evasion")));
    // An element taken at 15, after the talent, gets its defensive talent too.
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
    from_level(&later, 15, "kineticist/element/water/shroud-of-water");
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
    assert!(at(&build, 2).has_feat_for(&FeatKey::new("skill-focus"), &skill("perception")));
}

/// The archetypes keep the wild talents at every even level: the Kinetic Knight's 4th and the Blood Kineticist's
/// 20th rows, which differ from the base's, pick one too.
#[test]
fn every_kineticist_archetype_picks_wild_talents_at_even_levels() {
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
