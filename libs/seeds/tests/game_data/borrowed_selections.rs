//! Talents, hexes and discoveries taken through another class (ticket 60).

use std::collections::BTreeSet;

use rules::{Build, Prerequisite, SelectionOptionKey, SlotId, SlotKind};

use crate::support::*;

/// The guide's Advanced Rogue Talents (`rogue#advanced-rogue-talents`: "These unlock at 10th level"). The game's
/// are these and Emboldening Strike (`emboldening_strike_is_an_advanced_talent`); AdvanceTalents is
/// a33b99f95322d6741af83e9381b2391c.
const ADVANCED_TALENTS: [&str; 16] = [
    "blinding-strike", // Game: BlindingStrike fd01c15d9e74cd247b1fdbd6eb4d4713 (needs AdvanceTalents)
    "confounding-blades", // Game: ConfoundingBlades ce72662a812b1f242849417b2c784b5e (needs AdvanceTalents)
    "crippling-strike", // Game: CripplingStrike b696bd7cb38da194fa3404032483d1db (needs AdvanceTalents)
    "dispelling-attack", // Game: DispellingAttack 1b92146b8a9830d4bb97ab694335fa7c (needs AdvanceTalents)
    "double-debilitation", // Game: DoubleDebilitation dd699394df0ef8847abba26038333f02 (needs AdvanceTalents)
    "familiar",            // Game: Familiar 363cab72f77c47745bf3a8807074d183 (needs AdvanceTalents)
    "hunters-surprise", // Game: HuntersSurprise 6209197fed619cf41910f9aaceaf7d51 (needs AdvanceTalents)
    "improved-evasion", // Game: ImprovedEvasion 0d35d6c4d5eef8d4790d09bd9a874e57 (needs AdvanceTalents)
    "opportunist",      // Game: Opportunist 5bb6dc5ce00550441880a6ff8ad4c968 (needs AdvanceTalents)
    "petrifying-strike", // Game: PetrifyingStrike 719d4539f7824eb99b8c6668e77820d2 (needs AdvanceTalents)
    "skill-mastery-knowledge", // Game: RogueSkillMasteryKnowledge 4b104a7319393494ca8d449a3b38a45f (needs AdvanceTalents)
    "skill-mastery-lore", // Game: RogueSkillMasteryLore 31b5f3e5093c8244abb32f6c7c775de1 (needs AdvanceTalents)
    "skill-mastery-stealth-and-trickery", // Game: RogueSkillMasteryStealth 84ba86c9cc1ef9e489dab1ad03e3ba17 (needs AdvanceTalents)
    "slippery-mind", // Game: SlipperyMind a14e8c1801911334f96d410f10eab7bf (needs AdvanceTalents)
    "weapon-snatcher", // Game: WeaponSnatcher ffced614e1f95e34d99df77debbc4207 (needs AdvanceTalents)
    "wearying-strike", // Game: WearyingStrike b4befb0a9b58e0e4687942661c55198d (needs AdvanceTalents)
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
    DATA.selections
        .iter()
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
        let defined = DATA.features.iter().any(|f| f.key.as_str() == feature);
        assert!(!defined, "{feature} is still a feature");
        let on_a_row = class
            .levels
            .iter()
            .map(|row| &row.features)
            .chain(
                DATA.archetypes_of(&class.key)
                    .flat_map(|a| a.levels.iter().map(|row| &row.features)),
            )
            .flatten()
            .any(|f| f.as_str() == feature);
        assert!(!on_a_row, "{feature} is still on a row");
    }
    // Game: ArchaeologistArchetype 38384e0c1e99c2e42ac6ed70a04aca46 and VivisectionistArchetype
    // 68cbcd9fbf1fb1d489562f829bb97e38 add AdvanceTalents a33b99f95322d6741af83e9381b2391c at 10
    for (class_key, archetype, feature) in [
        ("bard", ARCHAEOLOGIST, "advanced-talents"),
        ("alchemist", VIVISECTIONIST, "advanced-talents"),
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
    // Game: RogueTalentSelection c074a5d615200494b8f2a9c845799d93 (38, UncannyDodgeTalent
    // ca5274d057152fa45b7527cad0927840 and ImprovedUncannyDodgeTalent e821c61b2711cea4cb993725b910e7e8 among them)
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
    // Not a game fact: the classes whose levels give Combat Trick's row. The game's bard and skald pick from lists of
    // their own (`a_bard_and_a_skald_pick_from_their_own_talent_lists`).
    assert_eq!(classes(rogue), set(&["bard", "rogue", "skald"]));
    // Game: SanctifiedSlayerArchetype 19b4a243d9a4bd34887d03c3dea03ced adds the slayer's own SlayerTalentSelection6
    // 43d1b15873e926848be2abf0ea3ad9a8 and SlayerTalentSelection10 913b9cf25c9536949b43a2651b7ffb66 (equivalent model)
    assert_eq!(
        classes(any_selection(SLAYER_TALENTS)),
        set(&["inquisitor", "slayer"])
    );

    // The Feyform Shifter's list: the game's is the rogue's own (`a_feyform_shifters_rogue_talent_is_the_rogues_selection`).
    let feyform = any_selection(FEYFORM_TALENT);
    assert_eq!(feyform.name, "Rogue Talent");
    assert!(feyform.options.is_empty());
    assert_eq!(classes(feyform), set(&["shifter"]));

    let medical = any_selection(MEDICAL_DISCOVERY);
    assert_eq!(medical.name, "Medical Discovery");
    assert!(medical.options.is_empty());
    // Game: VivsectionistDiscoverySelection 67f499218a0e22944abab6fe1c9eaeee (the same 25)
    assert_eq!(keys(medical), set(&MEDICAL_DISCOVERIES));
    assert_eq!(classes(medical), set(&["alchemist"]));

    let hex_arcana = any_selection(HEX_ARCANA);
    assert_eq!(hex_arcana.name, "Hex Arcana");
    assert!(hex_arcana.options.is_empty());
    let arcana_and_hexes: BTreeSet<String> =
        &keys(any_selection("magus/magus-arcana")) | &keys(any_selection(WITCH_HEX));
    // Game: HexcrafterMagusHexArcanaSelection ad6b9cecb5286d841a66e23cea3ef7bf lists MagusArcanaSelection
    // e9dc4dfc73eaaf94aae27e0ed6cc9ada's 15 and WitchHexSelection 9846043cf51251a4897728ed6e24e76f's 27, of which
    // HexChannelerChannelFeature b40316f05d4772e4894688e6743602bd needs a Hex Channeler witch (equivalent model: 41 a
    // magus can take)
    assert_eq!(arcana_and_hexes.len(), 15 + 26);
    assert_eq!(keys(hex_arcana), arcana_and_hexes);
    assert!(hex_arcana.option_levels.is_empty());

    // "At 10th level, the Archaeologist may select from the Advanced Rogue Talents". The game's Vivisectionist holds
    // the same AdvanceTalents (`the_vivisectionists_advanced_talents_meet_every_advanced_talent`).
    // Game: ArchaeologistArchetype 38384e0c1e99c2e42ac6ed70a04aca46 adds AdvanceTalents a33b99f95322d6741af83e9381b2391c
    // at 10, which each advanced talent needs (equivalent model: the planner names the archetype's own feature)
    for talent in &advanced {
        let prerequisite = option_prerequisite(talent);
        assert!(names_feature(prerequisite, "advanced-talents"), "{talent}");
    }
}

/// The game's Feyform Shifter takes the rogue's own talent selection at 8 and 14, all 38, the advanced ones locked
/// only by Advanced Talents.
// Game: FeyformShifterShifterArchetype 1cdfc7d306d1430eac19427539b62091 adds RogueTalentSelection
// c074a5d615200494b8f2a9c845799d93 at 8 and 14
#[test]
#[ignore = "waits on ticket 276: the game gives the Feyform Shifter the rogue's own talent selection, 38 options"]
fn a_feyform_shifters_rogue_talent_is_the_rogues_selection() {
    assert_eq!(
        keys(any_selection(FEYFORM_TALENT)),
        keys(any_selection(ROGUE_TALENTS))
    );
    assert_eq!(offered(&borrower("shifter", Some(FEYFORM), 8), 8).len(), 38);
}

/// Advanced Talents is one feature in the game, which the Vivisectionist gets at 10 as the Archaeologist does: it meets
/// every advanced talent's prerequisite, not only the three its list names.
// Game: VivisectionistArchetype 68cbcd9fbf1fb1d489562f829bb97e38 adds AdvanceTalents a33b99f95322d6741af83e9381b2391c
// at 10; e.g. BlindingStrike fd01c15d9e74cd247b1fdbd6eb4d4713 needs AdvanceTalents
#[test]
#[ignore = "waits on ticket 276: the game gives one Advanced Talents feature, which the Vivisectionist's meets for every advanced talent"]
fn the_vivisectionists_advanced_talents_meet_every_advanced_talent() {
    for talent in ADVANCED_TALENTS.map(rogue_talent) {
        assert!(
            names_feature(option_prerequisite(&talent), "advanced-talents"),
            "{talent}"
        );
    }
}

/// Emboldening Strike is an advanced talent in the game: it needs Advanced Talents (and Sneak Attack).
// Game: EmboldeningStrike ba6bea8c93c64764ab7e5bbdb88fb9a6 (PrerequisiteFeaturesFromList AdvanceTalents
// a33b99f95322d6741af83e9381b2391c or SneakyTricksFeature; PrerequisiteFeature SneakAttack
// 9b9eac6709e1c084cb18c3a366e0ec87)
#[test]
#[ignore = "waits on ticket 276: the game gives Emboldening Strike the Advanced Talents and Sneak Attack prerequisites"]
fn emboldening_strike_is_an_advanced_talent() {
    let prerequisite = option_prerequisite(&rogue_talent("emboldening-strike"));
    assert!(names_feature(prerequisite, "advanced-talents"));
}

/// Six advanced talents need Sneak Attack besides Advanced Talents in the game.
// Game: CripplingStrike b696bd7cb38da194fa3404032483d1db, ConfoundingBlades ce72662a812b1f242849417b2c784b5e,
// DispellingAttack 1b92146b8a9830d4bb97ab694335fa7c, WearyingStrike b4befb0a9b58e0e4687942661c55198d,
// EmboldeningStrike ba6bea8c93c64764ab7e5bbdb88fb9a6, PetrifyingStrike 719d4539f7824eb99b8c6668e77820d2: each
// PrerequisiteFeature SneakAttack 9b9eac6709e1c084cb18c3a366e0ec87 (Group All)
#[test]
#[ignore = "waits on ticket 276: the game gives six advanced talents a Sneak Attack prerequisite"]
fn six_advanced_talents_need_sneak_attack() {
    let names_sneak_attack = |prerequisite: &Prerequisite| {
        prerequisite.leaves().any(|leaf| {
            matches!(leaf, Prerequisite::Feature { feature, .. } if feature.as_str().ends_with("/sneak-attack"))
        })
    };
    for talent in [
        "crippling-strike",
        "confounding-blades",
        "dispelling-attack",
        "wearying-strike",
        "emboldening-strike",
        "petrifying-strike",
    ] {
        let key = rogue_talent(talent);
        assert!(
            DATA.selection_option(&SelectionOptionKey::new(&key))
                .and_then(|option| option.prerequisite.as_ref())
                .is_some_and(names_sneak_attack),
            "{talent}"
        );
    }
}

/// Each class or archetype picks where its table says: Bard Talent, Skald Talent, the Feyform Shifter's Rogue
/// Talent, Medical Discovery, the Sanctified Slayer's Slayer Talent, Hex Magus and Hex Arcana. Filled, each is possible.
#[test]
fn each_borrowing_class_picks_at_its_tables_levels() {
    // Game: BardProgression 8127f5ff40f5b484b8be98609358b9d2 (BardTalentSelection 94e2cd84bf3a8e04f8609fe502892f4f)
    // and SkaldProgression 26418fed2bc153245972a5b54204ed75 (SkaldTalentSelection d2a8fde8985691045b90e1ec57e3cc57) at
    // these levels; the lists are their own (`a_bard_and_a_skald_pick_from_their_own_talent_lists`)
    let bard = vec![2, 6, 10, 14, 18];
    let skald = vec![2, 7, 12, 17];
    let mut cases: Vec<(Build, &str, Vec<u8>)> = vec![
        (borrower("bard", None, 20), ROGUE_TALENTS, bard.clone()),
        (borrower("skald", None, 20), ROGUE_TALENTS, skald.clone()),
        // Game: BeastTamerArchetype 912b0495d9485124bb2c0e4aa6ac2efd removes the talents at 2 and 10
        (
            borrower("bard", Some("bard/beast-tamer"), 20),
            ROGUE_TALENTS,
            vec![6, 14, 18],
        ),
        // Game: ProvocateurArchetype a1f5ab7eb78942f6889304acbd92e945 (ProvocateurTalentSelection
        // aa88cb5263a54791b8d1e471bc192ad4 at these levels; its list: see the own-lists test)
        (
            borrower("skald", Some("skald/inciter"), 20),
            ROGUE_TALENTS,
            vec![4, 9, 14, 19],
        ),
        // Game: FeyformShifterShifterArchetype 1cdfc7d306d1430eac19427539b62091
        (
            borrower("shifter", Some(FEYFORM), 20),
            FEYFORM_TALENT,
            vec![8, 14],
        ),
        // Game: VivisectionistArchetype 68cbcd9fbf1fb1d489562f829bb97e38
        (
            borrower("alchemist", Some(VIVISECTIONIST), 20),
            MEDICAL_DISCOVERY,
            vec![2, 4, 6, 8, 10, 12, 14, 16, 18],
        ),
        // Game: SanctifiedSlayerArchetype 19b4a243d9a4bd34887d03c3dea03ced
        (
            borrower("inquisitor", Some(SANCTIFIED), 20),
            SLAYER_TALENTS,
            vec![8, 16, 17, 20],
        ),
        // Game: HexcrafterArchetype 79ccf7a306a5d5547bebd97299f6fc89 (HexcrafterMagusHexMagusSelection
        // a18b8c3d6251d8641a8094e5c2a7bc78 at 4, HexcrafterMagusHexArcanaSelection ad6b9cecb5286d841a66e23cea3ef7bf
        // at 6 to 18)
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
    // Game: BardClass 772c83a25e2268e448e841dcd548235f (8 archetypes, one a companion's, Penta_UniqueBard; none of
    // the other seven touches the talents) and SkaldClass 6afa347d804838b48bda16acb0573dc0 (7)
    assert_eq!(DATA.archetypes_of(&class("bard").key).count(), 7);
    assert_eq!(DATA.archetypes_of(&class("skald").key).count(), 7);
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
    // The Vivisectionist's and the Hexcrafter's other slots: `a_vivisectionist_keeps_one_alchemist_discovery_at_20`,
    // `a_hexcrafter_keeps_its_magus_arcana_at_3`.
    // Other archetypes pick none of them.
    // Game: InquisitorProgression 4e945c2fe5e252f4ea61eee7fb560017, MagusProgression 1b912721a7e075d4f9cfe8dafa39414c,
    // ShifterProgression da560d3be3c943d8b1fa23e77dd7efa4, AlchemistProgression efd55ff9be2fda34981f5b9c83afe4f1
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

/// The alchemist's 20th level gives two discoveries; the Vivisectionist removes one, so one is left at 20.
// Game: AlchemistProgression efd55ff9be2fda34981f5b9c83afe4f1 (DiscoverySelection twice at 20);
// VivisectionistArchetype 68cbcd9fbf1fb1d489562f829bb97e38 (removes one at 20);
// ProgressionData.CalculateLevelEntries, WrathDecompiled 6ce4a6b (one instance removed per entry)
#[test]
#[ignore = "waits on ticket 280: the game gives the Vivisectionist one alchemist discovery at 20"]
fn a_vivisectionist_keeps_one_alchemist_discovery_at_20() {
    assert_eq!(
        levels_with_slot_of(
            &borrower("alchemist", Some(VIVISECTIONIST), 20),
            "alchemist/discovery"
        ),
        [20]
    );
}

/// The Hexcrafter keeps the magus arcana at 3 and loses only those of 6 to 18.
// Game: HexcrafterArchetype 79ccf7a306a5d5547bebd97299f6fc89 (removes MagusArcanaSelection
// e9dc4dfc73eaaf94aae27e0ed6cc9ada at 6, 9, 12, 15 and 18)
#[test]
#[ignore = "waits on ticket 283: the game gives the Hexcrafter its magus arcana at 3"]
fn a_hexcrafter_keeps_its_magus_arcana_at_3() {
    assert_eq!(
        levels_with_slot_of(
            &borrower("magus", Some(HEXCRAFTER), 20),
            "magus/magus-arcana"
        ),
        [3]
    );
}

/// A bard, a skald and an Inciter each pick from a list of their own: 24, 21 and 24 talents.
// Game: BardTalentSelection 94e2cd84bf3a8e04f8609fe502892f4f (24; the one BardProgression
// 8127f5ff40f5b484b8be98609358b9d2 uses), SkaldTalentSelection d2a8fde8985691045b90e1ec57e3cc57 (21),
// ProvocateurTalentSelection aa88cb5263a54791b8d1e471bc192ad4 (24)
#[test]
#[ignore = "waits on ticket 276: the game gives the bard 24 talents, the skald 21 and the Inciter 24 of their own lists"]
fn a_bard_and_a_skald_pick_from_their_own_talent_lists() {
    assert_eq!(offered(&borrower("bard", None, 2), 2).len(), 24);
    assert_eq!(offered(&borrower("skald", None, 2), 2).len(), 21);
    assert_eq!(
        offered(&borrower("skald", Some("skald/inciter"), 4), 4).len(),
        24
    );
}

/// A bard and a skald pick from the rogue talents; the advanced ones need Advanced Talents, which an Archaeologist
/// has from 10.
#[test]
fn a_bard_takes_rogue_talents_and_an_archaeologist_advanced_ones_from_10() {
    let crippling = rogue_talent("crippling-strike");
    let opportunist = rogue_talent("opportunist");
    // The lists' sizes: `a_bard_and_a_skald_pick_from_their_own_talent_lists`.
    // Game: FastStealth 97a6aa2b64dd21a4fac67658a91067d7 (no prerequisite) and CripplingStrike
    // b696bd7cb38da194fa3404032483d1db (SneakAttack, AdvanceTalents) are in BardTalentSelection
    // 94e2cd84bf3a8e04f8609fe502892f4f and SkaldTalentSelection d2a8fde8985691045b90e1ec57e3cc57
    for build in [borrower("bard", None, 2), borrower("skald", None, 2)] {
        let offered = offered(&build, 2);
        assert_eq!(offered[&rogue_talent("fast-stealth")], Ok(()));
        assert!(offered[&crippling].is_err());
    }
    assert!(offered(&borrower("bard", None, 10), 10)[&crippling].is_err());
    // Game: ArchaeologistArchetype 38384e0c1e99c2e42ac6ed70a04aca46 adds AdvanceTalents
    // a33b99f95322d6741af83e9381b2391c only at 10
    assert_eq!(
        lock_reason(&borrower("bard", Some(ARCHAEOLOGIST), 6), 6, &crippling),
        "Needs Advanced Talents"
    );
    // Game: Opportunist 5bb6dc5ce00550441880a6ff8ad4c968 needs AdvanceTalents alone
    let archaeologist = borrower("bard", Some(ARCHAEOLOGIST), 10);
    assert_eq!(offered(&archaeologist, 10)[&opportunist], Ok(()));
    let with = with_picks(archaeologist, &[(10, pick(&opportunist))]);
    assert!(possible(&with));
}

/// The Archaeologist never gains Sneak Attack, so Crippling Strike stays locked at 10.
// Game: CripplingStrike b696bd7cb38da194fa3404032483d1db (PrerequisiteFeature SneakAttack
// 9b9eac6709e1c084cb18c3a366e0ec87); ArchaeologistArchetype 38384e0c1e99c2e42ac6ed70a04aca46 grants no SneakAttack
#[test]
#[ignore = "waits on ticket 276: the game locks Crippling Strike for an Archaeologist 10, who has no Sneak Attack"]
fn an_archaeologist_10_has_no_sneak_attack_for_crippling_strike() {
    let archaeologist = borrower("bard", Some(ARCHAEOLOGIST), 10);
    assert!(offered(&archaeologist, 10)[&rogue_talent("crippling-strike")].is_err());
}

/// "At 8th and 14th levels, the shifter can choose a Rogue Talent. Note that you cannot choose Advanced Talents":
/// the advanced ones aren't offered, even to a rogue 10.
#[test]
fn a_feyform_shifter_picks_a_rogue_talent_at_8_and_14_but_no_advanced_one() {
    let feyform = borrower("shifter", Some(FEYFORM), 8);
    let offered = offered(&feyform, 8);
    // Game: CombatTrick c5158a6622d0b694a99efb1d0025d2c1 and FastStealth 97a6aa2b64dd21a4fac67658a91067d7 are in
    // RogueTalentSelection c074a5d615200494b8f2a9c845799d93, which FeyformShifterShifterArchetype
    // 1cdfc7d306d1430eac19427539b62091 adds at 8; the list's size: `a_feyform_shifters_rogue_talent_is_the_rogues_selection`
    assert_eq!(offered[&rogue_talent("combat-trick")], Ok(()));
    let with = with_picks(feyform, &[(8, pick(&rogue_talent("fast-stealth")))]);
    assert!(possible(&with));
}

/// A rogue 10 who takes Feyform Shifter levels holds Advanced Talents, which opens the advanced talents at the
/// Feyform's slot: the game offers them all.
// Game: RogueProgression b57b2a75a5abcaf47a01cf84672b50e9 (AdvanceTalents a33b99f95322d6741af83e9381b2391c at 10);
// FeyformShifterShifterArchetype 1cdfc7d306d1430eac19427539b62091 (RogueTalentSelection
// c074a5d615200494b8f2a9c845799d93 at 8); BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b
#[test]
#[ignore = "waits on ticket 276: the game offers a rogue 10 the advanced talents at the Feyform Shifter's slot"]
fn a_rogue_10_feyform_shifter_is_offered_the_advanced_talents() {
    let rogue_then_feyform = multiclass(&[("rogue", None, 10), ("shifter", Some(FEYFORM), 8)]);
    let slot = the_slot_at(&rogue_then_feyform, 18);
    let offered = offered_in(&rogue_then_feyform, &slot);
    for talent in ADVANCED_TALENTS.map(rogue_talent) {
        assert!(offered.contains_key(&talent), "{talent}");
    }
}

/// "The Vivisectionist is able to select certain Rogue Talents (including Advanced Talents from level 10) and
/// Alchemist discoveries": its lists, with Crippling Strike open from 10.
#[test]
fn a_vivisectionist_picks_its_discoveries_and_talents_and_advanced_ones_from_10() {
    let at_2 = borrower("alchemist", Some(VIVISECTIONIST), 2);
    let offered_2 = offered(&at_2, 2);
    // Game: VivsectionistDiscoverySelection 67f499218a0e22944abab6fe1c9eaeee (the 25, Blinding Strike not among them);
    // VivisectionistArchetype 68cbcd9fbf1fb1d489562f829bb97e38 (SneakAttack from 1, AdvanceTalents
    // a33b99f95322d6741af83e9381b2391c at 10); GreaterMutagenFeature 76c61966afdd82048911f3d63c6fe0bc needs alchemist 12
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
    assert_eq!(lock_reason(&at_2, 2, &crippling), "Needs Advanced Talents");
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
    // Game: SanctifiedSlayerArchetype 19b4a243d9a4bd34887d03c3dea03ced adds SlayerTalentSelection6
    // 43d1b15873e926848be2abf0ea3ad9a8 at 8; SlayerTrapfinding e3c12938c2f93544da89824fbe0933a5, CombatTrick
    // c5158a6622d0b694a99efb1d0025d2c1 and FastStealth 97a6aa2b64dd21a4fac67658a91067d7 have no prerequisite
    assert_eq!(offered_8.len(), keys(any_selection(SLAYER_TALENTS)).len());
    for open in [
        "slayer/slayer-talents/trapfinding",
        "rogue/rogue-talents/combat-trick",
        "rogue/rogue-talents/fast-stealth",
    ] {
        assert_eq!(offered_8[open], Ok(()), "{open}");
    }
    // The first combat style feat at 8: `a_sanctified_slayer_8_may_take_the_first_combat_style_feat`.
    // Game: CripplingStrike b696bd7cb38da194fa3404032483d1db needs AdvanceTalents, which the Sanctified Slayer lacks
    assert!(offered_8[&rogue_talent("crippling-strike")].is_err());
    // A slayer 2 still takes its 2nd-or-4th combat style feat.
    // Game: SlayerTalentSelection2 04430ad24988baa4daa0bcd4f1c7d118 lists RangerStyleSelection2
    // c6d0da9124735a44f93ac31df803b9a9
    assert_eq!(
        offered(&build("slayer", 2), 2)["slayer/slayer-talents/combat-style-feat-2"],
        Ok(())
    );
    // "Prerequisites: Slayer Talents": the Sanctified Slayer's Slayer Talents from 8.
    // Game: ExtraSlayerTalentSelection 53f7237f5b1447bb851ba68045a00e41 needs SlayerTalentSelection2 or
    // SanctifiedSlayerSlayerTalentFeature db3767ba22ce84942af56525cb9f9ce6 (granted at 8)
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
    // The slot is Extra Slayer Talent's own selection (ticket 302), standing for the slayer talents.
    // Game: ExtraSlayerTalentSelection 53f7237f5b1447bb851ba68045a00e41 (its own list of 41)
    assert_eq!(
        kind_at(&at_9, &extra_slot(9, 0)),
        Some(selection_kind("slayer/extra-slayer-talent"))
    );
}

/// The first combat style feat has no class-level prerequisite in the game: a Sanctified Slayer 8 takes it.
// Game: RangerStyleSelection2 c6d0da9124735a44f93ac31df803b9a9 (only PrerequisiteNoFeature of itself), listed by
// SlayerTalentSelection6 43d1b15873e926848be2abf0ea3ad9a8, which SanctifiedSlayerArchetype
// 19b4a243d9a4bd34887d03c3dea03ced adds at 8
#[test]
#[ignore = "waits on ticket 276: the game opens the first combat style feat to a Sanctified Slayer 8"]
fn a_sanctified_slayer_8_may_take_the_first_combat_style_feat() {
    let at_8 = borrower("inquisitor", Some(SANCTIFIED), 8);
    assert_eq!(
        offered(&at_8, 8)["slayer/slayer-talents/combat-style-feat-2"],
        Ok(())
    );
}

/// "At 4th level, a Hexcrafter may select a Witch hex"; "Besides the usual Magus Arcana abilities, a Hexcrafter can
/// select a hex. From 12th level, a Hexcrafter can select major hexes and at 18th level, he can select a grand hex."
#[test]
fn a_hexcrafter_picks_a_witch_hex_at_4_and_an_arcana_or_hex_from_6() {
    let at_4 = borrower("magus", Some(HEXCRAFTER), 4);
    let hex_magus = offered(&at_4, 4);
    // Game: HexcrafterMagusHexMagusSelection a18b8c3d6251d8641a8094e5c2a7bc78 lists 27, HexChannelerChannelFeature
    // b40316f05d4772e4894688e6743602bd needing a Hex Channeler witch (equivalent model: 26 a magus can take);
    // WitchHexCackleFeature 36f2467103d4635459d412fb418276f4 has no prerequisite, WitchHexAgonyFeature
    // de5f107ec5a340d4dbc333fd86a3cc06 needs WitchMajorHex 8ac781b33e380c84aa578f1b006dd6c5
    assert_eq!(hex_magus.len(), 26);
    assert_eq!(hex_magus["witch/hex/cackle"], Ok(()));
    assert!(hex_magus["witch/hex/agony"].is_err());

    let at_6 = borrower("magus", Some(HEXCRAFTER), 6);
    let hex_arcana = offered(&at_6, 6);
    // Game: HexcrafterMagusHexArcanaSelection ad6b9cecb5286d841a66e23cea3ef7bf (equivalent model: 41 a magus can take)
    assert_eq!(hex_arcana.len(), 41);
    for open in ["magus/magus-arcana/arcane-accuracy", "witch/hex/cackle"] {
        assert_eq!(hex_arcana[open], Ok(()), "{open}");
    }
    // Game: HexcrafterArchetype 79ccf7a306a5d5547bebd97299f6fc89 adds WitchMajorHex 8ac781b33e380c84aa578f1b006dd6c5 at
    // 12 and WitchGrandHex d24c2467804ce0e4497d9978bafec1f9 at 18, which Agony and WitchHexDeathCurseFeature
    // b7cd7ba71151e354d9c8a1bff959d789 need (equivalent model: the planner reads the Hexcrafter's magus level)
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
    // Game: WitchHexRestlessSlumberFeature 0a31b4aba2b3bc24c85636cb8bdba1d9 needs Slumber and WitchMajorHex
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
    // Game: BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b (an option held at its ranks, 1 here, can't
    // be selected again); each pair's option is in both lists (FastStealth 97a6aa2b64dd21a4fac67658a91067d7 in
    // RogueTalentSelection, BardTalentSelection 94e2cd84bf3a8e04f8609fe502892f4f, SkaldTalentSelection
    // d2a8fde8985691045b90e1ec57e3cc57 and VivsectionistDiscoverySelection 67f499218a0e22944abab6fe1c9eaeee)
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
/// its table has talents at 2, 8, 12, 16 and 20 and no Uncanny Dodge of its own. The talent needs Eldritch Scoundrel
/// 4 (`an_eldritch_scoundrel_2_isnt_offered_uncanny_dodge`).
#[test]
fn an_eldritch_scoundrel_is_offered_uncanny_dodge_at_each_talent_level() {
    let uncanny_dodge = rogue_talent(UNCANNY_DODGE);
    let build = scoundrel(20);
    // Game: UncannyDodgeTalent ca5274d057152fa45b7527cad0927840 (PrerequisiteArchetypeLevel EldritchScoundrelArchetype
    // 57f93dd8423c97c49989501281296c4a 4, in an Any group)
    for level in [8, 12, 16, 20] {
        assert!(open_at(&build, level, &uncanny_dodge), "level {level}");
    }
    // Finesse Training's slot at 3 is another selection.
    assert!(!offered(&build, 3).contains_key(&uncanny_dodge));
    // The archetype has neither feature of its own.
    // Game: EldritchScoundrelArchetype 57f93dd8423c97c49989501281296c4a removes UncannyDodgeChecker
    // 8f800ed6ce8c42e8a01fd8f3e990c459 at 4 and ImprovedUncannyDodge 485a18c05792521459c7d06c63128c79 at 8
    assert!(!holds(&at(&build, 20), "uncanny-dodge"));
    assert!(!holds(&at(&build, 20), "improved-uncanny-dodge"));
}

/// The Uncanny Dodge talent needs Eldritch Scoundrel level 4: not offered at its talent slot of 2.
// Game: UncannyDodgeTalent ca5274d057152fa45b7527cad0927840 (PrerequisiteArchetypeLevel EldritchScoundrelArchetype
// 57f93dd8423c97c49989501281296c4a 4)
#[test]
#[ignore = "waits on ticket 276: the game gives the Uncanny Dodge talent from Eldritch Scoundrel 4, not at 2"]
fn an_eldritch_scoundrel_2_isnt_offered_uncanny_dodge() {
    assert!(!open_at(&scoundrel(20), 2, &rogue_talent(UNCANNY_DODGE)));
}

/// The guide's sentence doesn't say that Improved Uncanny Dodge needs Uncanny Dodge first; the rogue's own order
/// (4, then 8) is taken (user, 2026-11-13: the recommended option). The game's rule is Eldritch Scoundrel 12, with no
/// Uncanny Dodge first (`an_eldritch_scoundrel_takes_improved_uncanny_dodge_from_rogue_12`, to the user); what both
/// rules agree on stays here: no Improved Uncanny Dodge at 2 or 8, and none before Uncanny Dodge at 8.
#[test]
fn an_eldritch_scoundrel_has_no_improved_uncanny_dodge_at_2_or_8() {
    // Game: ImprovedUncannyDodgeTalent e821c61b2711cea4cb993725b910e7e8 (PrerequisiteArchetypeLevel
    // EldritchScoundrelArchetype 57f93dd8423c97c49989501281296c4a 12)
    let improved = rogue_talent(IMPROVED_UNCANNY_DODGE);
    let build = scoundrel(20);
    for level in [2, 8] {
        assert!(!open_at(&build, level, &improved), "level {level}");
    }
    let reversed = with_picks(
        scoundrel(8),
        &[
            (2, pick(&improved)),
            (8, pick(&rogue_talent(UNCANNY_DODGE))),
        ],
    );
    assert!(!possible(&reversed));
}

/// The game opens Improved Uncanny Dodge to an Eldritch Scoundrel 12 whether or not she holds Uncanny Dodge.
// Game: ImprovedUncannyDodgeTalent e821c61b2711cea4cb993725b910e7e8 (PrerequisiteArchetypeLevel
// EldritchScoundrelArchetype 57f93dd8423c97c49989501281296c4a 12; no Uncanny Dodge prerequisite)
#[test]
#[ignore = "waits on ticket 276: the game gives Improved Uncanny Dodge from Eldritch Scoundrel 12, with no Uncanny Dodge first"]
fn an_eldritch_scoundrel_takes_improved_uncanny_dodge_from_rogue_12() {
    let improved = rogue_talent(IMPROVED_UNCANNY_DODGE);
    let build = scoundrel(20);
    for level in [12, 16, 20] {
        assert!(open_at(&build, level, &improved), "level {level}");
    }
}

#[test]
fn the_scoundrels_talents_grant_the_rogues_features_once() {
    let uncanny_dodge = rogue_talent(UNCANNY_DODGE);
    let improved = rogue_talent(IMPROVED_UNCANNY_DODGE);
    // Game: UncannyDodgeTalent ca5274d057152fa45b7527cad0927840 grants UncannyDodgeChecker
    // 8f800ed6ce8c42e8a01fd8f3e990c459 (from Eldritch Scoundrel 4); ImprovedUncannyDodgeTalent
    // e821c61b2711cea4cb993725b910e7e8 grants ImprovedUncannyDodge 485a18c05792521459c7d06c63128c79 (from 12); each
    // Ranks 1 (BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b)
    let build = with_picks(
        scoundrel(12),
        &[(8, pick(&uncanny_dodge)), (12, pick(&improved))],
    );
    assert!(possible(&build));
    let count = |level: u8, feature: &str| {
        at(&build, level).feature_count(&rules::FeatureKey::new(feature))
    };
    assert_eq!(count(8, "uncanny-dodge"), 1);
    assert_eq!(count(8, "improved-uncanny-dodge"), 0);
    assert_eq!(count(12, "uncanny-dodge"), 1);
    assert_eq!(count(12, "improved-uncanny-dodge"), 1);
    // Once each.
    let taken = with_picks(scoundrel(12), &[(8, pick(&uncanny_dodge))]);
    assert_eq!(
        offered(&taken, 12)[&uncanny_dodge],
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
    // Game: UncannyDodgeTalent ca5274d057152fa45b7527cad0927840 and ImprovedUncannyDodgeTalent
    // e821c61b2711cea4cb993725b910e7e8 need Eldritch Scoundrel, bard, slayer or skald levels (and the first, Feyform
    // Shifter levels); RogueProgression b57b2a75a5abcaf47a01cf84672b50e9; SkaldTalentSelection
    // d2a8fde8985691045b90e1ec57e3cc57 lists neither. The bard's, the slayer's and the Feyform's:
    // `the_bard_slayer_and_feyform_shifter_are_offered_the_uncanny_dodge_talents`.
    let builds = [build("rogue", 20), borrower("skald", None, 20)]
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
    // The Vivisectionist's list never gained them.
    // Game: VivsectionistDiscoverySelection 67f499218a0e22944abab6fe1c9eaeee
    let keys = keys(any_selection(MEDICAL_DISCOVERY));
    assert!(options.iter().all(|key| !keys.contains(key)));
}

/// The bard's and the slayer's lists hold both talents, the Feyform Shifter's (the rogue's own) the first: a bard is
/// offered Uncanny Dodge from bard 4 and Improved from 12, a slayer the same, a Feyform Shifter Uncanny Dodge from 4.
// Game: UncannyDodgeTalent ca5274d057152fa45b7527cad0927840 (Any: bard, slayer, skald, Feyform Shifter level 4);
// ImprovedUncannyDodgeTalent e821c61b2711cea4cb993725b910e7e8 (Any: bard, slayer, skald level 12);
// BardTalentSelection 94e2cd84bf3a8e04f8609fe502892f4f, SlayerTalentSelection2 04430ad24988baa4daa0bcd4f1c7d118,
// SlayerTalentSelection6 43d1b15873e926848be2abf0ea3ad9a8, SlayerTalentSelection10 913b9cf25c9536949b43a2651b7ffb66,
// RogueTalentSelection c074a5d615200494b8f2a9c845799d93 (the Feyform's)
#[test]
#[ignore = "waits on ticket 276: the game offers Uncanny Dodge to a bard, a slayer and a Feyform Shifter from their level 4, Improved to a bard and a slayer from 12"]
fn the_bard_slayer_and_feyform_shifter_are_offered_the_uncanny_dodge_talents() {
    let [uncanny_dodge, improved] = SCOUNDRELS_TALENTS.map(rogue_talent);
    for bard in [
        borrower("bard", None, 20),
        borrower("bard", Some(ARCHAEOLOGIST), 20),
    ] {
        assert!(open_at(&bard, 6, &uncanny_dodge));
        assert!(open_at(&bard, 14, &improved));
    }
    let slayer = build("slayer", 20);
    assert!(open_at(&slayer, 4, &uncanny_dodge));
    assert!(open_at(&slayer, 12, &improved));
    assert!(open_at(
        &borrower("shifter", Some(FEYFORM), 20),
        8,
        &uncanny_dodge
    ));
}

/// An Eldritch Scoundrel 2 with bard 2 meets neither of the talent's level-4 prerequisites, so the bard's slot at
/// character level 4 (the bard's level 2) doesn't offer it.
// Game: UncannyDodgeTalent ca5274d057152fa45b7527cad0927840 (Any: Eldritch Scoundrel 4, bard 4, slayer 4, …)
#[test]
#[ignore = "waits on ticket 276: the game refuses Uncanny Dodge to an Eldritch Scoundrel 2 / bard 2"]
fn a_scoundrel_2_with_bard_2_isnt_offered_uncanny_dodge() {
    let build = multiclass(&[("rogue", Some(ELDRITCH_SCOUNDREL), 2), ("bard", None, 2)]);
    assert!(!open_at(&build, 4, &rogue_talent(UNCANNY_DODGE)));
}

/// An Eldritch Scoundrel whose talents are Uncanny Dodge (at 8, from her level 4) meets Extra Rogue Talent's
/// prerequisite; so with Improved Uncanny Dodge (at 12) held too.
#[test]
fn a_scoundrel_with_only_uncanny_dodge_may_take_extra_rogue_talent() {
    // Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 needs RogueTalentSelection
    // c074a5d615200494b8f2a9c845799d93, held from rogue 2 (LevelUpHelper.AddFeaturesFromProgression, WrathDecompiled
    // 6ce4a6b); UncannyDodgeTalent ca5274d057152fa45b7527cad0927840 from Eldritch Scoundrel 4,
    // ImprovedUncannyDodgeTalent e821c61b2711cea4cb993725b910e7e8 from 12
    let extra = feat_pick_plain("extra-rogue-talent");
    let first = with_picks(scoundrel(9), &[(8, pick(&rogue_talent(UNCANNY_DODGE)))]);
    let with_extra = |mut build: Build, level: u8| {
        build.picks.insert(general_slot(level), extra.clone());
        build
    };
    assert_eq!(
        feat_status(&first, &general_slot(9), "extra-rogue-talent"),
        Ok(())
    );
    let build = with_extra(first, 9);
    assert!(possible(&build));

    let mut both = with_picks(
        scoundrel(13),
        &[
            (8, pick(&rogue_talent(UNCANNY_DODGE))),
            (12, pick(&rogue_talent(IMPROVED_UNCANNY_DODGE))),
        ],
    );
    both = with_extra(both, 13);
    assert!(possible(&both));
}

/// Extra Rogue Talent's pick is its own list: the rogue's 38 talents and Evasion, one more than a rogue's talent slot.
// Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 (39: RogueTalentSelection
// c074a5d615200494b8f2a9c845799d93's 38 and Evasion 576933720c440aa4d8d42b0c54b77e80)
#[test]
#[ignore = "waits on ticket 264: the game gives Extra Rogue Talent its own list of 39, the rogue talents and Evasion"]
fn extra_rogue_talent_offers_the_rogue_talents_and_evasion() {
    let mut build = with_selections_filled(build("rogue", 3));
    build
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rogue-talent"));
    let extra = offered_at(&build, &extra_slot(3, 0));
    let slot = offered_at(&build, &selection_slot(2));
    assert!(slot.is_subset(&extra));
    assert_eq!(extra.len(), slot.len() + 1);
}
