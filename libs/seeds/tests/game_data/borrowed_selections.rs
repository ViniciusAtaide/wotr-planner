//! Talents, hexes and discoveries taken through another class (ticket 60).

use std::collections::BTreeSet;

use rules::{Build, Prerequisite, SelectionOptionKey, SlotId, SlotKind};

use crate::support::*;

/// The guide's Advanced Rogue Talents (`rogue#advanced-rogue-talents`: "These unlock at 10th level").
const ADVANCED_TALENTS: [&str; 16] = [
    "blinding-strike",
    "confounding-blades",
    "crippling-strike",
    "dispelling-attack",
    "double-debilitation",
    "familiar",
    "hunters-surprise",
    "improved-evasion",
    "opportunist",
    "petrifying-strike",
    "skill-mastery-knowledge",
    "skill-mastery-lore",
    "skill-mastery-stealth-and-trickery",
    "slippery-mind",
    "weapon-snatcher",
    "wearying-strike",
];

/// The features the new slots replace: each was a label for a pick.
const BORROWED_FEATURES: [(&str, &str); 7] = [
    ("bard", "bard/bard-talents"),
    ("skald", "skald/skald-talents"),
    ("shifter", FEYFORM_TALENT),
    ("alchemist", MEDICAL_DISCOVERY),
    ("inquisitor", "inquisitor/sanctified-slayer/slayer-talents"),
    ("magus", "magus/hexcrafter/hex-magus"),
    ("magus", HEX_ARCANA),
];

/// Every selection of the data, by key.
fn any_selection(key: &str) -> &'static rules::FeatureSelection {
    DATA.selections()
        .find(|selection| selection.key.as_str() == key)
        .unwrap_or_else(|| panic!("no selection {key}"))
}

fn option_prerequisite(key: &str) -> &'static Prerequisite {
    DATA.selection_option(&SelectionOptionKey::new(key))
        .unwrap_or_else(|| panic!("no option {key}"))
        .prerequisite
        .as_ref()
        .unwrap_or_else(|| panic!("{key}: no prerequisite"))
}

fn names_feature(prerequisite: &Prerequisite, feature: &str) -> bool {
    match prerequisite {
        Prerequisite::All(children) | Prerequisite::Any(children) => {
            children.iter().any(|child| names_feature(child, feature))
        }
        Prerequisite::Feature { feature: f, .. } => f.as_str() == feature,
        _ => false,
    }
}

/// The one selection slot of the Build at character level `level`.
fn the_slot_at(build: &Build, level: u8) -> SlotId {
    let found: Vec<SlotId> = engine()
        .evaluate(build)
        .levels
        .iter()
        .flat_map(|l| &l.slots)
        .filter(|slot| {
            matches!(slot.kind, SlotKind::Selection(_))
                && matches!(slot.id, SlotId::Selection { level: at, .. } if at == level)
        })
        .map(|slot| slot.id.clone())
        .collect();
    let [slot] = found.as_slice() else {
        panic!("{} selection slots at {level}", found.len())
    };
    slot.clone()
}

/// The one selection slot of the Build at character level `level` whose selection offers `option`.
fn slot_offering(build: &Build, level: u8, option: &str) -> SlotId {
    let found: Vec<SlotId> = engine()
        .evaluate(build)
        .levels
        .iter()
        .flat_map(|l| &l.slots)
        .filter(|slot| {
            matches!(&slot.kind, SlotKind::Selection(selection)
                if any_selection(selection.as_str()).offers(&SelectionOptionKey::new(option)))
                && matches!(slot.id, SlotId::Selection { level: at, .. } if at == level)
        })
        .map(|slot| slot.id.clone())
        .collect();
    let [slot] = found.as_slice() else {
        panic!("{} slots offering {option} at {level}", found.len())
    };
    slot.clone()
}

/// The guide's lists are features in the data no more: each borrowed pick is a slot, and the feature that stood for
/// it is gone from its class's (or archetype's) features and from every row. The Advanced Talents features of the
/// Archaeologist and the Vivisectionist stay: they open the advanced rogue talents.
#[test]
fn the_borrowed_picks_replace_their_features() {
    for (class_key, feature) in BORROWED_FEATURES {
        let class = class(class_key);
        let defined = class
            .features
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.features))
            .any(|f| f.key.as_str() == feature);
        assert!(!defined, "{feature} is still a feature");
        let on_a_row = class
            .levels
            .iter()
            .map(|row| &row.features)
            .chain(
                class
                    .archetypes
                    .iter()
                    .flat_map(|a| a.levels.iter().map(|row| &row.features)),
            )
            .flatten()
            .any(|f| f.as_str() == feature);
        assert!(!on_a_row, "{feature} is still on a row");
    }
    for (class_key, archetype, feature) in [
        ("bard", ARCHAEOLOGIST, "bard/archaeologist/advanced-talents"),
        (
            "alchemist",
            VIVISECTIONIST,
            "alchemist/vivisectionist/advanced-talents",
        ),
    ] {
        assert!(
            archetype_features(class_key, archetype).contains(feature),
            "{feature}"
        );
    }
}

/// Each list is an existing selection where the guide's list is one ("The Bard can select from the Rogue Talent
/// list", "a Sanctified Slayer can select a Slayer talent", "a Hexcrafter may select a Witch hex"), or a selection
/// sharing the options of the lists it mixes, as ticket 25 made the slayer's (the Feyform Shifter's rogue talents but
/// the advanced ones; the Vivisectionist's discoveries and talents; the Hexcrafter's arcana and hexes). Combat Trick's
/// row needs an option level, so the classes that pick through each name it.
#[test]
fn the_borrowed_selections_offer_the_guides_lists() {
    let rogue = any_selection(ROGUE_TALENTS);
    let advanced: BTreeSet<String> = ADVANCED_TALENTS.iter().map(|t| rogue_talent(t)).collect();
    let all_talents = keys(rogue);
    // The guide's 36 talents and the Eldritch Scoundrel's two (ticket 190), which no borrower shares.
    assert_eq!(all_talents.len(), 38);
    let scoundrels = BTreeSet::from(SCOUNDRELS_TALENTS.map(rogue_talent));
    let all_talents = &all_talents - &scoundrels;
    assert_eq!(all_talents.len(), 36);
    assert!(advanced.is_subset(&all_talents));
    let classes = |selection: &rules::FeatureSelection| -> BTreeSet<String> {
        selection
            .option_levels
            .iter()
            .map(|c| c.to_string())
            .collect()
    };
    let set =
        |items: &[&str]| -> BTreeSet<String> { items.iter().map(|s| s.to_string()).collect() };
    assert_eq!(classes(rogue), set(&["bard", "rogue", "skald"]));
    assert_eq!(
        classes(any_selection(SLAYER_TALENTS)),
        set(&["inquisitor", "slayer"])
    );

    let feyform = any_selection(FEYFORM_TALENT);
    assert_eq!(feyform.name, "Rogue Talent");
    assert!(feyform.options.is_empty());
    assert_eq!(keys(feyform), &all_talents - &advanced);
    assert_eq!(keys(feyform).len(), 20);
    assert_eq!(classes(feyform), set(&["shifter"]));

    let medical = any_selection(MEDICAL_DISCOVERY);
    assert_eq!(medical.name, "Medical Discovery");
    assert!(medical.options.is_empty());
    assert_eq!(keys(medical), set(&MEDICAL_DISCOVERIES));
    assert_eq!(classes(medical), set(&["alchemist"]));

    let hex_arcana = any_selection(HEX_ARCANA);
    assert_eq!(hex_arcana.name, "Hex Arcana");
    assert!(hex_arcana.options.is_empty());
    let arcana_and_hexes: BTreeSet<String> =
        &keys(any_selection("magus/magus-arcana")) | &keys(any_selection(WITCH_HEX));
    assert_eq!(arcana_and_hexes.len(), 15 + 26);
    assert_eq!(keys(hex_arcana), arcana_and_hexes);
    assert!(hex_arcana.option_levels.is_empty());

    // "At 10th level, the Archaeologist may select from the Advanced Rogue Talents"; the Vivisectionist only its
    // three, "including Advanced Talents from level 10".
    for talent in &advanced {
        let prerequisite = option_prerequisite(talent);
        assert!(
            names_feature(prerequisite, "bard/archaeologist/advanced-talents"),
            "{talent}"
        );
        assert_eq!(
            names_feature(prerequisite, "alchemist/vivisectionist/advanced-talents"),
            MEDICAL_DISCOVERIES.contains(&talent.as_str()),
            "{talent}"
        );
    }
}

/// Each class or archetype picks where its table says: Bard Talent, Skald Talent, the Feyform Shifter's Rogue
/// Talent, Medical Discovery, the Sanctified Slayer's Slayer Talent, Hex Magus and Hex Arcana. Filled, each is possible.
#[test]
fn each_borrowing_class_picks_at_its_tables_levels() {
    let bard = vec![2, 6, 10, 14, 18];
    let skald = vec![2, 7, 12, 17];
    let mut cases: Vec<(Build, &str, Vec<u8>)> = vec![
        (borrower("bard", None, 20), ROGUE_TALENTS, bard.clone()),
        (borrower("skald", None, 20), ROGUE_TALENTS, skald.clone()),
        (
            borrower("bard", Some("bard/beast-tamer"), 20),
            ROGUE_TALENTS,
            vec![6, 14, 18],
        ),
        (
            borrower("skald", Some("skald/inciter"), 20),
            ROGUE_TALENTS,
            vec![4, 9, 14, 19],
        ),
        (
            borrower("shifter", Some(FEYFORM), 20),
            FEYFORM_TALENT,
            vec![8, 14],
        ),
        (
            borrower("alchemist", Some(VIVISECTIONIST), 20),
            MEDICAL_DISCOVERY,
            vec![2, 4, 6, 8, 10, 12, 14, 16, 18],
        ),
        (
            borrower("inquisitor", Some(SANCTIFIED), 20),
            SLAYER_TALENTS,
            vec![8, 16, 17, 20],
        ),
        (borrower("magus", Some(HEXCRAFTER), 20), WITCH_HEX, vec![4]),
        (
            borrower("magus", Some(HEXCRAFTER), 20),
            HEX_ARCANA,
            vec![6, 9, 12, 15, 18],
        ),
    ];
    for archetype in [
        ARCHAEOLOGIST,
        "bard/chelaxian-diva",
        "bard/dirge-bard",
        "bard/flame-dancer",
        "bard/thundercaller",
        "bard/tranquil-whisperer",
    ] {
        cases.push((
            borrower("bard", Some(archetype), 20),
            ROGUE_TALENTS,
            bard.clone(),
        ));
    }
    for archetype in [
        "skald/battle-scion",
        "skald/battle-singer",
        "skald/court-poet",
        "skald/demon-dancer",
        "skald/herald-of-the-horn",
        "skald/hunt-caller",
    ] {
        cases.push((
            borrower("skald", Some(archetype), 20),
            ROGUE_TALENTS,
            skald.clone(),
        ));
    }
    assert_eq!(class("bard").archetypes.len(), 7);
    assert_eq!(class("skald").archetypes.len(), 7);
    for (build, selection, levels) in cases {
        let who = format!("{:?} {selection}", build.class_choices);
        assert_eq!(levels_with_slot_of(&build, selection), levels, "{who}");
        let filled = with_selections_filled(build);
        assert!(possible(&filled), "{who}: filled");
        assert!(
            levels.iter().all(|level| filled
                .picks
                .contains_key(&slot_of(&filled, *level, selection))),
            "{who}: every slot filled"
        );
    }
    // The Vivisectionist's Medical Discovery is its discovery: no alchemist discovery slot is left, and the Hexcrafter
    // has no magus arcana slot of its own.
    assert_eq!(
        levels_with_slot_of(
            &borrower("alchemist", Some(VIVISECTIONIST), 20),
            "alchemist/discovery"
        ),
        Vec::<u8>::new()
    );
    assert_eq!(
        levels_with_slot_of(
            &borrower("magus", Some(HEXCRAFTER), 20),
            "magus/magus-arcana"
        ),
        Vec::<u8>::new()
    );
    // Other archetypes pick none of them.
    for (class_key, selection) in [
        ("inquisitor", SLAYER_TALENTS),
        ("magus", WITCH_HEX),
        ("shifter", FEYFORM_TALENT),
        ("alchemist", MEDICAL_DISCOVERY),
    ] {
        assert_eq!(
            levels_with_slot_of(&borrower(class_key, None, 20), selection),
            Vec::<u8>::new(),
            "{class_key}"
        );
    }
}

/// A bard and a skald pick from the rogue talents; the advanced ones need Advanced Talents, which an Archaeologist
/// has from 10.
#[test]
fn a_bard_takes_rogue_talents_and_an_archaeologist_advanced_ones_from_10() {
    let crippling = rogue_talent("crippling-strike");
    for build in [borrower("bard", None, 2), borrower("skald", None, 2)] {
        let offered = offered(&build, 2);
        assert_eq!(offered.len(), 38, "{:?}", build.levels[0].class);
        assert_eq!(offered[&rogue_talent("fast-stealth")], Ok(()));
        assert!(offered[&crippling].is_err());
    }
    assert!(offered(&borrower("bard", None, 10), 10)[&crippling].is_err());
    assert_eq!(
        lock_reason(&borrower("bard", Some(ARCHAEOLOGIST), 6), 6, &crippling),
        "Needs Advanced Talents (Archaeologist)"
    );
    let archaeologist = borrower("bard", Some(ARCHAEOLOGIST), 10);
    assert_eq!(offered(&archaeologist, 10)[&crippling], Ok(()));
    let with = with_picks(archaeologist, &[(10, pick(&crippling))]);
    assert!(possible(&with));
}

/// "At 8th and 14th levels, the shifter can choose a Rogue Talent. Note that you cannot choose Advanced Talents":
/// the advanced ones aren't offered, even to a rogue 10.
#[test]
fn a_feyform_shifter_picks_a_rogue_talent_at_8_and_14_but_no_advanced_one() {
    let feyform = borrower("shifter", Some(FEYFORM), 8);
    let offered = offered(&feyform, 8);
    assert_eq!(offered.len(), 20);
    assert_eq!(offered[&rogue_talent("combat-trick")], Ok(()));
    let rogue_then_feyform = multiclass(&[("rogue", None, 10), ("shifter", Some(FEYFORM), 8)]);
    let slot = the_slot_at(&rogue_then_feyform, 18);
    let offered = offered_in(&rogue_then_feyform, &slot);
    assert!(
        ADVANCED_TALENTS
            .iter()
            .all(|t| !offered.contains_key(&rogue_talent(t)))
    );
    let with = with_picks(feyform, &[(8, pick(&rogue_talent("fast-stealth")))]);
    assert!(possible(&with));
}

/// "The Vivisectionist is able to select certain Rogue Talents (including Advanced Talents from level 10) and
/// Alchemist discoveries": its lists, with Crippling Strike open from 10.
#[test]
fn a_vivisectionist_picks_its_discoveries_and_talents_and_advanced_ones_from_10() {
    let at_2 = borrower("alchemist", Some(VIVISECTIONIST), 2);
    let offered_2 = offered(&at_2, 2);
    assert_eq!(
        offered_2.keys().cloned().collect::<BTreeSet<_>>(),
        MEDICAL_DISCOVERIES.iter().map(|k| k.to_string()).collect()
    );
    for open in [
        "alchemist/discovery/cognatogen",
        "rogue/rogue-talents/fast-stealth",
        "rogue/rogue-talents/combat-trick",
    ] {
        assert_eq!(offered_2[open], Ok(()), "{open}");
    }
    let crippling = rogue_talent("crippling-strike");
    assert_eq!(
        lock_reason(&at_2, 2, &crippling),
        "Needs Advanced Talents (Vivisectionist)"
    );
    assert!(offered_2["alchemist/discovery/greater-mutagen"].is_err());
    let at_10 = borrower("alchemist", Some(VIVISECTIONIST), 10);
    assert_eq!(offered(&at_10, 10)[&crippling], Ok(()));
    // Blinding Strike is an advanced talent the list doesn't name.
    assert!(!offered(&at_10, 10).contains_key(&rogue_talent("blinding-strike")));
    let with = with_picks(
        at_10,
        &[
            (2, pick("alchemist/discovery/feral-mutagen")),
            (6, pick("alchemist/discovery/feral-wings")),
            (10, pick(&crippling)),
        ],
    );
    assert!(possible(&with));
}

/// "At 8th, 16th, 17th and 20th levels, a Sanctified Slayer can select a Slayer talent": the slayer's talents, its
/// combat style feats the slayer's own (they need slayer levels), and Extra Slayer Talent from its first talent.
#[test]
fn a_sanctified_slayer_picks_slayer_talents_at_8_16_17_and_20() {
    let at_8 = borrower("inquisitor", Some(SANCTIFIED), 8);
    let offered_8 = offered(&at_8, 8);
    assert_eq!(offered_8.len(), keys(any_selection(SLAYER_TALENTS)).len());
    for open in [
        "slayer/slayer-talents/trapfinding",
        "rogue/rogue-talents/combat-trick",
        "rogue/rogue-talents/fast-stealth",
    ] {
        assert_eq!(offered_8[open], Ok(()), "{open}");
    }
    assert_eq!(
        lock_reason(&at_8, 8, "slayer/slayer-talents/combat-style-feat-2"),
        "Needs Slayer level 2 (have 0)"
    );
    assert!(offered_8[&rogue_talent("crippling-strike")].is_err());
    // A slayer 2 still takes its 2nd-or-4th combat style feat.
    assert_eq!(
        offered(&build("slayer", 2), 2)["slayer/slayer-talents/combat-style-feat-2"],
        Ok(())
    );
    // "Prerequisites: Slayer Talents": the Sanctified Slayer's Slayer Talents from 8.
    assert!(
        feat_status(
            &borrower("inquisitor", Some(SANCTIFIED), 7),
            &general_slot(7),
            "extra-slayer-talent"
        )
        .is_err()
    );
    let mut at_9 = with_selections_filled(borrower("inquisitor", Some(SANCTIFIED), 9));
    assert_eq!(
        feat_status(&at_9, &general_slot(9), "extra-slayer-talent"),
        Ok(())
    );
    at_9.picks
        .insert(general_slot(9), feat_pick_plain("extra-slayer-talent"));
    assert!(possible(&at_9));
    assert_eq!(
        kind_at(&at_9, &extra_slot(9, 0)),
        Some(selection_kind(SLAYER_TALENTS))
    );
}

/// "At 4th level, a Hexcrafter may select a Witch hex"; "Besides the usual Magus Arcana abilities, a Hexcrafter can
/// select a hex. From 12th level, a Hexcrafter can select major hexes and at 18th level, he can select a grand hex."
#[test]
fn a_hexcrafter_picks_a_witch_hex_at_4_and_an_arcana_or_hex_from_6() {
    let at_4 = borrower("magus", Some(HEXCRAFTER), 4);
    let hex_magus = offered(&at_4, 4);
    assert_eq!(hex_magus.len(), 26);
    assert_eq!(hex_magus["witch/hex/cackle"], Ok(()));
    assert!(hex_magus["witch/hex/agony"].is_err());

    let at_6 = borrower("magus", Some(HEXCRAFTER), 6);
    let hex_arcana = offered(&at_6, 6);
    assert_eq!(hex_arcana.len(), 41);
    for open in ["magus/magus-arcana/arcane-accuracy", "witch/hex/cackle"] {
        assert_eq!(hex_arcana[open], Ok(()), "{open}");
    }
    let at_9 = borrower("magus", Some(HEXCRAFTER), 9);
    assert_eq!(
        lock_reason(&at_9, 9, "witch/hex/agony"),
        "Needs Magus level 12 (have 9)"
    );
    assert_eq!(
        offered(&borrower("magus", Some(HEXCRAFTER), 12), 12)["witch/hex/agony"],
        Ok(())
    );
    let at_15 = borrower("magus", Some(HEXCRAFTER), 15);
    assert_eq!(
        lock_reason(&at_15, 15, "witch/hex/death-curse"),
        "Needs Magus level 18 (have 15)"
    );
    let at_18 = borrower("magus", Some(HEXCRAFTER), 18);
    assert_eq!(offered(&at_18, 18)["witch/hex/death-curse"], Ok(()));
    let with = with_picks(
        at_18,
        &[
            (4, pick("witch/hex/slumber")),
            (6, pick("magus/magus-arcana/arcane-accuracy")),
            (12, pick("witch/hex/restless-slumber")),
            (18, pick("witch/hex/death-curse")),
        ],
    );
    assert!(possible(&with));
    // A magus of no archetype still can't.
    assert!(!offered(&borrower("magus", None, 12), 12).contains_key("witch/hex/agony"));
}

/// An option is one key whichever class takes it: taken through one class's slot, it's "Already taken" through
/// another's, and twice is impossible.
#[test]
fn an_option_taken_through_one_class_isnt_offered_again_through_another() {
    let fast_stealth = rogue_talent("fast-stealth");
    let cases: [(Build, u8, &str, u8); 7] = [
        (
            multiclass(&[("rogue", None, 2), ("bard", None, 2)]),
            2,
            &fast_stealth,
            4,
        ),
        (
            multiclass(&[("bard", None, 2), ("skald", None, 2)]),
            2,
            &fast_stealth,
            4,
        ),
        (
            multiclass(&[("rogue", None, 2), ("shifter", Some(FEYFORM), 8)]),
            2,
            &fast_stealth,
            10,
        ),
        (
            multiclass(&[("rogue", None, 2), ("alchemist", Some(VIVISECTIONIST), 2)]),
            2,
            &fast_stealth,
            4,
        ),
        (
            multiclass(&[("slayer", None, 2), ("inquisitor", Some(SANCTIFIED), 8)]),
            2,
            "slayer/slayer-talents/trapfinding",
            10,
        ),
        (
            multiclass(&[("witch", None, 1), ("magus", Some(HEXCRAFTER), 4)]),
            1,
            "witch/hex/cackle",
            5,
        ),
        (
            multiclass(&[("witch", None, 1), ("magus", Some(HEXCRAFTER), 6)]),
            1,
            "witch/hex/cackle",
            7,
        ),
    ];
    for (build, first, option, then) in cases {
        let who = format!("{:?}", build.class_choices);
        let first_slot = slot_offering(&build, first, option);
        let mut build = build.clone();
        build.picks.insert(first_slot, pick(option));
        assert!(possible(&build), "{who}");
        let slot = slot_offering(&build, then, option);
        assert_eq!(reason_in(&build, &slot, option), "Already taken", "{who}");
        build.picks.insert(slot, pick(option));
        assert!(!possible(&build), "{who}: twice");
    }
}

const ELDRITCH_SCOUNDREL: &str = "rogue/eldritch-scoundrel";
const UNCANNY_DODGE: &str = "uncanny-dodge";
const IMPROVED_UNCANNY_DODGE: &str = "improved-uncanny-dodge";
/// The Eldritch Scoundrel's two options of the rogue talents.
const SCOUNDRELS_TALENTS: [&str; 2] = [UNCANNY_DODGE, IMPROVED_UNCANNY_DODGE];

fn scoundrel(levels: usize) -> Build {
    archetype_build("rogue", ELDRITCH_SCOUNDREL, levels)
}

/// Whether the slot of `level` offers `option` to the Build, open to pick.
fn open_at(build: &Build, level: u8, option: &str) -> bool {
    matches!(offered(build, level).get(option), Some(Ok(())))
}

/// "An Eldritch Scoundrel can select (Improved) Uncanny Dodge as Rogue Talents" (`rogue#rogue-talents`, ticket 190);
/// its table has talents at 2, 8, 12, 16 and 20 and no Uncanny Dodge of its own.
#[test]
fn an_eldritch_scoundrel_is_offered_uncanny_dodge_at_each_talent_level() {
    let uncanny_dodge = rogue_talent(UNCANNY_DODGE);
    let build = scoundrel(20);
    for level in [2, 8, 12, 16, 20] {
        assert!(open_at(&build, level, &uncanny_dodge), "level {level}");
    }
    // Finesse Training's slot at 3 is another selection.
    assert!(!offered(&build, 3).contains_key(&uncanny_dodge));
    // The archetype has neither feature of its own.
    assert!(!holds(&at(&build, 20), "rogue/uncanny-dodge"));
    assert!(!holds(&at(&build, 20), "rogue/improved-uncanny-dodge"));
}

/// PROVISIONAL (ticket 190, decided by the main session, for the user to confirm): the guide's sentence doesn't say
/// that Improved Uncanny Dodge needs Uncanny Dodge first; the rogue's own order (4, then 8) is taken. To flip it,
/// drop the `Feature("rogue/uncanny-dodge")` from the option's prerequisite and rewrite this test.
#[test]
fn an_eldritch_scoundrel_takes_improved_uncanny_dodge_only_after_uncanny_dodge() {
    let improved = rogue_talent(IMPROVED_UNCANNY_DODGE);
    let build = scoundrel(20);
    for level in [2, 8, 12, 16, 20] {
        assert!(!open_at(&build, level, &improved), "level {level}");
    }
    let first = with_picks(build, &[(2, pick(&rogue_talent(UNCANNY_DODGE)))]);
    for level in [8, 12, 16, 20] {
        assert!(open_at(&first, level, &improved), "level {level}");
    }
    let both = with_picks(first, &[(8, pick(&improved))]);
    assert!(possible(&both));
    // Improved at 2 with Uncanny Dodge only at 8 is out of order.
    let reversed = with_picks(
        scoundrel(8),
        &[
            (2, pick(&improved)),
            (8, pick(&rogue_talent(UNCANNY_DODGE))),
        ],
    );
    assert!(!possible(&reversed));
}

#[test]
fn the_scoundrels_talents_grant_the_rogues_features_once() {
    let uncanny_dodge = rogue_talent(UNCANNY_DODGE);
    let improved = rogue_talent(IMPROVED_UNCANNY_DODGE);
    let build = with_picks(
        scoundrel(8),
        &[(2, pick(&uncanny_dodge)), (8, pick(&improved))],
    );
    assert!(possible(&build));
    let count = |level: u8, feature: &str| {
        at(&build, level).feature_count(&rules::FeatureKey::new(feature))
    };
    assert_eq!(count(2, "rogue/uncanny-dodge"), 1);
    assert_eq!(count(2, "rogue/improved-uncanny-dodge"), 0);
    assert_eq!(count(8, "rogue/uncanny-dodge"), 1);
    assert_eq!(count(8, "rogue/improved-uncanny-dodge"), 1);
    // Once each.
    let taken = with_picks(scoundrel(12), &[(2, pick(&uncanny_dodge))]);
    assert_eq!(
        offered(&taken, 8)[&uncanny_dodge],
        Err(rules::Issue::AlreadyHeld)
    );
}

#[test]
fn no_other_class_or_archetype_is_offered_the_scoundrels_talents() {
    let options = SCOUNDRELS_TALENTS.map(rogue_talent);
    let archetypes = [
        "rogue/dark-lurker",
        "rogue/knife-master",
        "rogue/master-of-all",
        "rogue/rowdy",
        "rogue/sylvan-trickster",
        "rogue/thug",
        "rogue/underground-chemist",
    ];
    let builds = [
        build("rogue", 20),
        borrower("bard", None, 20),
        borrower("skald", None, 20),
        build("slayer", 20),
        borrower("bard", Some(ARCHAEOLOGIST), 20),
    ]
    .into_iter()
    .chain(archetypes.map(|archetype| archetype_build("rogue", archetype, 20)));
    for (who, build) in builds.enumerate() {
        for level in
            (1..=20).filter(|level| engine().options(&build, &selection_slot(*level)).is_some())
        {
            for option in &options {
                assert!(
                    !open_at(&build, level, option),
                    "build {who} level {level}: {option}"
                );
            }
        }
    }
    // The shared lists never gained them.
    for selection in [SLAYER_TALENTS, FEYFORM_TALENT, MEDICAL_DISCOVERY] {
        let keys = keys(any_selection(selection));
        assert!(options.iter().all(|key| !keys.contains(key)), "{selection}");
    }
}

/// An Eldritch Scoundrel who also has bard levels meets the prerequisite there too (`Archetype` looks at the whole
/// Build): accepted for ticket 190, as for the Sanctified Slayer's talents.
#[test]
fn a_scoundrel_with_bard_levels_may_take_them_through_the_bards_slots() {
    let build = multiclass(&[("rogue", Some(ELDRITCH_SCOUNDREL), 2), ("bard", None, 2)]);
    // Character level 4 is the bard's level 2: its Rogue Talent slot.
    assert!(open_at(&build, 4, &rogue_talent(UNCANNY_DODGE)));
}
