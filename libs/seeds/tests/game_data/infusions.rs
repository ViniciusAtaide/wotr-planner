//! The kineticist's infusions (ticket 59).

use std::collections::BTreeSet;

use rules::{ArchetypeKey, Build, ClassKey, FeatureKey, Prerequisite, SelectionOptionKey};

use crate::support::*;

/// Each infusion of the guide's list (`kineticist#infusions`), with the kineticist level its blueprint's
/// `PrerequisiteClassLevel` asks (0: none), its elements (none: Universal), the infusions its "Prerequisites" names
/// (one of them) and who may take it. The elements stand for the game's blasts (equivalent model: an element gives its
/// blasts) but in `BLASTS`; the Sharpshooter column stands for the game's `PrerequisiteArchetypeLevel` and
/// `PrerequisiteNoArchetype` (equivalent model: Eruption, Deadly Earth and Fragmentation carry no archetype check but
/// need Extended Range, which the Sharpshooter can't take).
#[allow(clippy::type_complexity)]
const INFUSIONS: [(&str, u8, &[&str], &[&str], Sharpshooter); 32] = [
    // Game: BurningInfusionFeature 228ec37705a5e5240be3ff5622c4476f
    ("burning-infusion", 0, &["fire"], &[], Sharpshooter::Any),
    // Game: DazzlingInfusionFeature 037460f7ae3e21943b237007f2b1a5d5
    ("dazzling-infusion", 0, &["fire"], &[], Sharpshooter::Any),
    // Game: ExplodingArrowsInfusion 7c8ded7015114e4086b3dbdd16388b29
    ("exploding-arrows", 3, &[], &[], Sharpshooter::Only),
    // Game: ExtendedRangeInfusion cb2d9e6355dd33940b2bef49e544b0bf
    ("extended-range", 0, &[], &[], Sharpshooter::Not),
    // Game: FanOfFlamesInfusion fde466d2c24705641bcd97d04a323566
    ("fan-of-flames", 0, &["fire"], &[], Sharpshooter::Any),
    // Game: KineticBladeInfusion 9ff81732daddb174aa8138ad1297c787
    ("kinetic-blade", 0, &[], &[], Sharpshooter::Not),
    // Game: PushingInfusionFeature fbb97f35a41b71c4cbc36c5f3995b892
    (
        "pushing-infusion",
        0,
        &["air", "earth", "water"],
        &[],
        Sharpshooter::Any,
    ),
    // Game: BowlingInfusionFeature b3bd080eed83a9940abd97e4aa2a7341
    ("bowling-infusion", 4, &["earth"], &[], Sharpshooter::Any),
    // Game: EntanglingInfusionFeature 607539d018d03454aaac0a2c1522f7ac
    (
        "entangling-infusion",
        4,
        &["earth", "water"],
        &[],
        Sharpshooter::Any,
    ),
    // Game: MagneticInfusionFeature d6b95ac99e3004b499d750835864e053
    (
        "magnetic-infusion",
        4,
        &["air", "earth"],
        &[],
        Sharpshooter::Any,
    ),
    // Game: SpindleInfusion c4f4a62a325f7c14dbcace3ce34782b5
    ("spindle", 4, &[], &[], Sharpshooter::Any),
    // Game: EruptionInfusion 00f8e4b846c367141afcd133f4a1c816
    (
        "eruption",
        6,
        &["fire"],
        &["extended-range"],
        Sharpshooter::Not,
    ),
    // Game: FoxfireInfusionFeature ae21c5369252ec74aa1fee89f1bc1b21
    (
        "foxfire-infusion",
        6,
        &["fire"],
        &["kineticist/wild-talents/foxfire"],
        Sharpshooter::Any,
    ),
    // Game: RareMetalInfusionFeature 6700ce7f56ddc30488e45b049f4ee475
    ("rare-metal-infusion", 6, &["earth"], &[], Sharpshooter::Any),
    // Game: SynapticInfusionFeature 53c80f136f2bf65409d358f28b0c5bb4
    ("synaptic-infusion", 6, &["air"], &[], Sharpshooter::Any),
    // Game: TorrentInfusionFeature 2aad85320d0751340a0786de073ee3d5
    (
        "torrent",
        6,
        &["air", "fire", "water"],
        &[],
        Sharpshooter::Any,
    ),
    // Game: CycloneInfusion f2fa7541f18b8af4896fbaf9f2a21dfe
    ("cyclone", 8, &["air"], &[], Sharpshooter::Any),
    // Game: DetonationInfusion 77c24cc95ce319c44a0e5fc6ff466d5b
    ("detonation", 8, &["fire"], &[], Sharpshooter::Any),
    // Game: FlashInfusionFeature 37f3cfca29073e142a80c3b8e7c54b05
    ("flash-infusion", 8, &["fire"], &[], Sharpshooter::Any),
    // Game: SprayInfusion b5852e8287f12d34ca6f84fcc7019f07
    ("spray", 8, &["water"], &[], Sharpshooter::Any),
    // Game: BladeWhirlwindInfusion 80fdf049d396c33408a805d9e21a42e1
    (
        "blade-whirlwind",
        10,
        &[],
        &["kinetic-blade"],
        Sharpshooter::Not,
    ),
    // Game: ChillingInfusionFeature 6ac87a3af9ccf014787c49745df75e6a
    ("chilling-infusion", 10, &["water"], &[], Sharpshooter::Any),
    // Game: GrapplingInfusionFeature e2d70b95e80549b439d30df29a79cb58
    (
        "grappling-infusion",
        10,
        &[],
        &["cloud", "deadly-earth", "wall"],
        Sharpshooter::Any,
    ),
    // Game: ClusterArrowsInfusion 8896ac251d394dceaf650793aa85e177
    ("rending-arrows", 11, &[], &[], Sharpshooter::Only),
    // Game: UnravelingInfusionFeature 79339d57d491d824ba0aa4ed0c114b2f
    ("unraveling-infusion", 10, &["fire"], &[], Sharpshooter::Any),
    // Game: WallInfusion c684335918896ce4ab13e96cec929796
    ("wall", 10, &[], &[], Sharpshooter::Any),
    // Game: DeadlyEarthInfusion 061f5d7e659432b478668b70f6d4caae
    (
        "deadly-earth",
        12,
        &["earth"],
        &["extended-range"],
        Sharpshooter::Not,
    ),
    // Game: ArmorPiercingInfusion cf531860c9fb44c09bda9d8904657314
    ("honed-infusion", 12, &[], &[], Sharpshooter::Only),
    // Game: CloudInfusion 3c53ee4965a13d74e81b37ae34f0861b
    (
        "cloud",
        14,
        &["air", "water"],
        &["extended-range"],
        Sharpshooter::Any,
    ),
    // Game: FragmentationInfusion 88ae936abf296894695a282f49214718
    (
        "fragmentation",
        14,
        &["earth", "water"],
        &["extended-range"],
        Sharpshooter::Not,
    ),
    // Game: PureFlameInfusionFeature 1dfd9b8e1439e4a4ab4b6b11f5ea676a
    ("pure-flame-infusion", 14, &["fire"], &[], Sharpshooter::Any),
    // Game: ChainArrowsInfusion 0ff81c5691c64c4aa8b2ecc35c885901
    ("chain-arrows", 16, &[], &[], Sharpshooter::Only),
];

/// The infusions whose kineticist level the data still holds as twice the guide's infusion level (2, and 2 for
/// Exploding Arrows, 10 for Rending Arrows), where the game's blueprint asks for none, 3 or 11. No slot shows it today:
/// the base class's first slot is at 3 and the Sharpshooter's at odd levels.
const LEVEL_DIFFERS: [&str; 8] = [
    "burning-infusion",
    "dazzling-infusion",
    "exploding-arrows",
    "extended-range",
    "fan-of-flames",
    "kinetic-blade",
    "pushing-infusion",
    "rending-arrows",
];

/// The infusions whose game prerequisite is a list of blasts that no set of the planner's elements matches: the game's
/// air element gives the Air or the Electric blast, water the Water or the Cold blast, and the composites need two
/// elements (`infusions_need_a_blast_not_an_element`).
const BLASTS: [&str; 12] = [
    "pushing-infusion",
    "entangling-infusion",
    "magnetic-infusion",
    "rare-metal-infusion",
    "synaptic-infusion",
    "torrent",
    "cyclone",
    "spray",
    "chilling-infusion",
    "cloud",
    "fragmentation",
    "pure-flame-infusion",
];

/// What an infusion needs: the kineticist level its blueprint names (0: none), one of its elements, one of the options
/// its "Prerequisites" names (an infusion, or the wild talent the key names), and the Sharpshooter's archetype or its
/// absence.
fn infusion_prerequisite(
    kineticist_level: u8,
    elements: &[&str],
    needs: &[&str],
    sharpshooter: Sharpshooter,
) -> Prerequisite {
    let one_of = |keys: Vec<Prerequisite>| match keys.len() {
        1 => keys.into_iter().next().unwrap(),
        _ => Prerequisite::Any(keys),
    };
    let mut all = Vec::new();
    if kineticist_level > 0 {
        all.push(Prerequisite::ClassLevel {
            class: ClassKey::new("kineticist"),
            level: kineticist_level,
        });
    }
    if !elements.is_empty() {
        all.push(one_of(
            elements
                .iter()
                .map(|e| {
                    Prerequisite::SelectionOption(SelectionOptionKey::new(format!(
                        "kineticist/element/{e}"
                    )))
                })
                .collect(),
        ));
    }
    if !needs.is_empty() {
        all.push(one_of(
            needs
                .iter()
                .map(|key| {
                    let key = match key.contains('/') {
                        true => (*key).to_owned(),
                        false => infusion(key),
                    };
                    Prerequisite::SelectionOption(SelectionOptionKey::new(key))
                })
                .collect(),
        ));
    }
    let archetype = || Prerequisite::Archetype(ArchetypeKey::new(SHARPSHOOTER));
    match sharpshooter {
        Sharpshooter::Any => {}
        Sharpshooter::Only => all.push(archetype()),
        Sharpshooter::Not => all.push(Prerequisite::Not(Box::new(archetype()))),
    }
    let mut prerequisite = match all.len() {
        1 => all.pop().unwrap(),
        _ => Prerequisite::All(all),
    };
    prerequisite.canonicalize();
    prerequisite
}

/// The kineticist level a prerequisite asks for, if any.
fn kineticist_level_needed(prerequisite: &Prerequisite) -> Option<u8> {
    match prerequisite {
        Prerequisite::ClassLevel { class, level } if class.as_str() == "kineticist" => Some(*level),
        Prerequisite::All(all) => all.iter().find_map(kineticist_level_needed),
        _ => None,
    }
}

/// The infusions a slot leaves open, by name.
fn open_infusions(build: &Build, level: u8) -> BTreeSet<String> {
    open_at(build, level)
        .into_iter()
        .map(|key| {
            key.strip_prefix(&format!("{INFUSIONS_SELECTION}/"))
                .unwrap_or_else(|| panic!("{key}: not an infusion"))
                .to_owned()
        })
        .collect()
}

/// "The Kineticist may select any Infusion for which she qualifies." The guide's 32 infusions, each with its level,
/// elements and prerequisites; no feat gives an infusion. The levels of the slots and the 1st-level "Infusion"
/// feature are `the_kineticist_picks_an_infusion_at_1_and_at_each_later_infusion_level`'s, which waits on the data.
#[test]
fn infusions_are_a_selection_of_32_with_their_level_and_elements() {
    let kineticist = class("kineticist");
    let selection = kineticist
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == INFUSIONS_SELECTION)
        .expect("the infusion selection");
    // Game: InfusionSelection 58d6f8e9eea63f6418b107ce64f315ea: its 32 options (Honed Infusion is
    // ArmorPiercingInfusion, Rending Arrows ClusterArrowsInfusion).
    assert_eq!(selection.name, "Infusion");
    let keys: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    let expected: BTreeSet<String> = INFUSIONS.iter().map(|(name, ..)| infusion(name)).collect();
    assert_eq!(keys, expected);
    // Game: each row's blueprint, cited beside it in INFUSIONS (the rows of LEVEL_DIFFERS and BLASTS wait on the data).
    for (name, level, elements, needs, sharpshooter) in INFUSIONS {
        if LEVEL_DIFFERS.contains(&name) || BLASTS.contains(&name) {
            continue;
        }
        assert_eq!(
            selection_option(&infusion(name)).prerequisite,
            Some(infusion_prerequisite(level, elements, needs, sharpshooter)),
            "{name}"
        );
    }
    // The guide's "Dazziing Infusion" is named as the game names it; its link is the guide's anchor.
    // Game: DazzlingInfusionFeature 037460f7ae3e21943b237007f2b1a5d5.
    let dazzling = selection_option(&infusion("dazzling-infusion"));
    assert_eq!(dazzling.name, "Dazzling Infusion");
    assert_eq!(
        dazzling.guide.as_ref().unwrap().0,
        "kineticist#dazziing-infusion"
    );

    // The Sharpshooter's "Infusion (Kinetic Sharpshooter)" was its pick; it's gone.
    // Game: KineticSharpshooterArchetype e0684d0a6ef44cae9d095f5318618a85 removes the level-1 InfusionSelection and
    // adds none.
    assert!(kineticist.archetypes.iter().all(|a| {
        a.features.iter().all(|f| {
            f.key.as_str() != "kineticist/kinetic-sharpshooter/infusion-kinetic-sharpshooter"
        })
    }));
    // Game: InfusionSelection 58d6f8e9eea63f6418b107ce64f315ea is held by the kineticist's progression and archetypes
    // alone (InfusionProgression 908286aeefa6be54e8c39e144cb87fdd is named by nothing).
    assert!(DATA.feats.iter().all(|feat| {
        feat.extra_pick_of
            .iter()
            .all(|s| s.as_str() != INFUSIONS_SELECTION)
    }));
}

/// The game's kineticist picks an infusion at 1 as well as at 3, 5, 9, 11, 13, 17 and 19: the 1st-level "Infusion" of
/// the table is that pick, not a feature.
#[test]
#[ignore = "waits on ticket 270: the game gives an infusion pick at kineticist 1"]
fn the_kineticist_picks_an_infusion_at_1_and_at_each_later_infusion_level() {
    let table = class("kineticist");
    // Game: KineticistProgression b79e92dd495edd64e90fb483c504b8df: InfusionSelection
    // 58d6f8e9eea63f6418b107ce64f315ea at 1, 3, 5, 9, 11, 13, 17 and 19.
    let levels: Vec<u8> = table
        .levels
        .iter()
        .filter(|row| {
            row.selections
                .iter()
                .any(|s| s.as_str() == INFUSIONS_SELECTION)
        })
        .map(|row| row.level)
        .collect();
    assert_eq!(levels, [1, 3, 5, 9, 11, 13, 17, 19]);
    let feature = FeatureKey::new("kineticist/infusion");
    assert!(
        table
            .levels
            .iter()
            .all(|row| !row.features.contains(&feature))
    );
    assert!(
        table
            .archetypes
            .iter()
            .flat_map(|a| &a.levels)
            .all(|row| !row.features.contains(&feature))
    );
    assert_eq!(
        levels_with_slot_of(&kineticist("fire", 2), INFUSIONS_SELECTION),
        [1]
    );
    assert_eq!(
        levels_with_slot_of(&kineticist("fire", 3), INFUSIONS_SELECTION),
        [1, 3]
    );
    assert!(!holds(
        &at(&kineticist("fire", 3), 1),
        "kineticist/infusion"
    ));
}

/// The game's infusions ask for the kineticist level their blueprint names, not twice the guide's infusion level: none
/// for the level-1 infusions, 3 for Exploding Arrows and 11 for Rending Arrows.
#[test]
#[ignore = "waits on ticket 270: the game gives no kineticist level for a level-1 infusion, 3 for \
            Exploding Arrows and 11 for Rending Arrows"]
fn each_infusion_needs_the_kineticist_level_its_blueprint_names() {
    // Game: each row's blueprint in INFUSIONS (its PrerequisiteClassLevel on KineticistClass, or none).
    for (name, level, ..) in INFUSIONS {
        if !LEVEL_DIFFERS.contains(&name) {
            continue;
        }
        let prerequisite = selection_option(&infusion(name))
            .prerequisite
            .as_ref()
            .expect("a prerequisite");
        assert_eq!(
            kineticist_level_needed(prerequisite),
            (level > 0).then_some(level),
            "{name}"
        );
    }
}

/// The game's infusions name blasts, not elements: one element alone never opens Cloud (composites only), and earth
/// once opens neither Rare Metal (the Metal blast: earth twice) nor Magnetic; water once never opens Fragmentation
/// (Earth, Ice or Metal), fire once never Pure Flame (Blue Flame: fire twice). Pushing, Entangling, Synaptic, Torrent,
/// Cyclone, Spray and Chilling also turn on the blast an air or water kineticist chooses (Air or Electric, Water or
/// Cold), which the planner doesn't model yet: an Expanded Element water opens Spray and Pushing only with the Water
/// blast.
#[test]
#[ignore = "waits on ticket 270: the game gives infusion prerequisites by blast, not by element"]
fn infusions_need_a_blast_not_an_element() {
    // Game: RareMetalInfusionFeature 6700ce7f56ddc30488e45b049f4ee475 (the Metal blast only);
    // MagneticInfusionFeature d6b95ac99e3004b499d750835864e053 (Charged Water, Electric, Metal, Plasma or
    // Thunderstorm).
    let earth = kineticist("earth", 9);
    assert!(!open_infusions(&earth, 9).contains("rare-metal-infusion"));
    assert!(!open_infusions(&earth, 9).contains("magnetic-infusion"));
    // Game: CloudInfusion 3c53ee4965a13d74e81b37ae34f0861b (Blizzard, Sandstorm, Steam or Thunderstorm).
    let air = kineticist_with("air", 17, &[(3, &infusion("extended-range"))]);
    assert!(!open_infusions(&air, 17).contains("cloud"));
    // Game: FragmentationInfusion 88ae936abf296894695a282f49214718 (Earth, Ice or Metal).
    let water = kineticist_with("water", 17, &[(3, &infusion("extended-range"))]);
    assert!(!open_infusions(&water, 17).contains("fragmentation"));
    // Game: PureFlameInfusionFeature 1dfd9b8e1439e4a4ab4b6b11f5ea676a (Blue Flame only).
    let fire = kineticist("fire", 19);
    assert!(!open_infusions(&fire, 17).contains("pure-flame-infusion"));
}

/// The game's Kinetic Knight trades the Elemental Focus selection for its own, so it never holds that selection, which
/// 13 infusions need: Spindle among them.
#[test]
#[ignore = "waits on ticket 270: the game gives the Kinetic Knight no infusion that needs the Elemental \
            Focus selection"]
fn a_kinetic_knight_is_refused_the_infusions_that_need_the_elemental_focus_selection() {
    // Game: SpindleInfusion c4f4a62a325f7c14dbcace3ce34782b5 (PrerequisiteFeature ElementalFocusSelection);
    // KineticKnightArchetype 7d61d9b2250260a45b18c5634524a8fb removes ElementalFocusSelection at 1.
    let knight = with_option(
        archetype_build("kineticist", "kineticist/kinetic-knight", 5),
        1,
        0,
        "kineticist/element/fire",
    );
    assert!(possible(&knight));
    assert!(!open_infusions(&knight, 5).contains("spindle"));
}

/// The ticket's: a kineticist 3's infusion slot at 3, "Infusion", offers every infusion and leaves open only its
/// element's and the Universal ones of level 1 (but the Sharpshooter's own). Its slot at 1 is
/// `the_kineticist_picks_an_infusion_at_1_and_at_each_later_infusion_level`'s.
#[test]
fn a_kineticist_3s_infusion_slot_at_3_offers_its_elements_and_universal_infusions_of_level_1() {
    let engine = engine();
    let build = kineticist("fire", 3);
    assert!(possible(&build));
    // Game: InfusionSelection 58d6f8e9eea63f6418b107ce64f315ea, 32 options, at 3 in KineticistProgression
    // b79e92dd495edd64e90fb483c504b8df.
    assert!(levels_with_slot_of(&build, INFUSIONS_SELECTION).contains(&3));
    assert_eq!(offered(&build, 3).len(), 32);
    // Game: BurningInfusionFeature 228ec37705a5e5240be3ff5622c4476f, DazzlingInfusionFeature
    // 037460f7ae3e21943b237007f2b1a5d5, ExtendedRangeInfusion cb2d9e6355dd33940b2bef49e544b0bf, FanOfFlamesInfusion
    // fde466d2c24705641bcd97d04a323566, KineticBladeInfusion 9ff81732daddb174aa8138ad1297c787 (equivalent model: the
    // fire element gives the Fire blast).
    let fire: BTreeSet<String> = [
        "burning-infusion",
        "dazzling-infusion",
        "extended-range",
        "fan-of-flames",
        "kinetic-blade",
    ]
    .iter()
    .map(|name| (*name).to_owned())
    .collect();
    assert_eq!(open_infusions(&build, 3), fire);
    // Game: PushingInfusionFeature fbb97f35a41b71c4cbc36c5f3995b892 lists the Earth blast (equivalent model).
    let earth: BTreeSet<String> = ["extended-range", "kinetic-blade", "pushing-infusion"]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    assert_eq!(open_infusions(&kineticist("earth", 3), 3), earth);
    let sheet = engine.evaluate(&build);
    // Game: InfusionSelection 58d6f8e9eea63f6418b107ce64f315ea.
    assert_eq!(
        engine.slot_label(sheet.slot(&selection_slot(3)).unwrap()),
        "Infusion"
    );
    let picked = kineticist_with("fire", 3, &[(3, &infusion("extended-range"))]);
    assert!(possible(&picked));
}

/// The ticket's: "The minimum Kineticist level to use a given Infusion is twice the Infusion's level": one of a higher
/// level is locked with a reason and opens at its level; one taken isn't offered again.
#[test]
fn a_higher_level_infusion_is_locked_with_a_reason_and_opens_at_its_level() {
    let build = kineticist("fire", 19);
    // Game: DetonationInfusion 77c24cc95ce319c44a0e5fc6ff466d5b: kineticist 8.
    assert_eq!(
        infusion_reason(&build, 3, "detonation"),
        "Needs Kineticist level 8 (have 3)"
    );
    assert_eq!(
        infusion_reason(&build, 5, "detonation"),
        "Needs Kineticist level 8 (have 5)"
    );
    assert!(open_infusions(&build, 9).contains("detonation"));
    // Game: PureFlameInfusionFeature 1dfd9b8e1439e4a4ab4b6b11f5ea676a: kineticist 14 (its blast is
    // `infusions_need_a_blast_not_an_element`'s).
    assert_eq!(
        infusion_reason(&build, 13, "pure-flame-infusion"),
        "Needs Kineticist level 14 (have 13)"
    );
    // Each Fire or Universal infusion with nothing else to need opens at its own level. Game: each row's blueprint in
    // INFUSIONS (Pure Flame needs the Blue Flame blast: `infusions_need_a_blast_not_an_element`).
    for (name, level, elements, needs, sharpshooter) in INFUSIONS {
        if !needs.is_empty()
            || !matches!(sharpshooter, Sharpshooter::Any)
            || !(elements.is_empty() || elements.contains(&"fire"))
            || name == "pure-flame-infusion"
        {
            continue;
        }
        for at in [3, 5, 9, 11, 13, 17, 19] {
            assert_eq!(
                open_infusions(&build, at).contains(name),
                level <= at,
                "{name} at {at}"
            );
        }
    }

    let early = kineticist_with("fire", 3, &[(3, &infusion("detonation"))]);
    assert_eq!(
        why_not(&early, &selection_slot(3)).and_then(|issue| engine().issue_text(&issue)),
        Some("Needs Kineticist level 8 (have 3)".to_owned())
    );
    // Game: ExtendedRangeInfusion cb2d9e6355dd33940b2bef49e544b0bf (Ranks 1);
    // Blueprints/Classes/BlueprintFeature.MeetsPrerequisites, WrathDecompiled 6ce4a6b (a feature at its Ranks isn't
    // offered again).
    let taken = kineticist_with("fire", 5, &[(3, &infusion("extended-range"))]);
    assert_eq!(
        infusion_reason(&taken, 5, "extended-range"),
        "Already taken"
    );
    let twice = kineticist_with(
        "fire",
        5,
        &[
            (3, &infusion("extended-range")),
            (5, &infusion("extended-range")),
        ],
    );
    assert!(!possible(&twice));
}

/// The ticket's: a Fire kineticist isn't offered a Water-only infusion, until Expanded Element gives it Water.
#[test]
fn a_fire_kineticist_isnt_offered_a_water_only_infusion() {
    let build = kineticist("fire", 9);
    // Game: SprayInfusion b5852e8287f12d34ca6f84fcc7019f07 (Water or Charged Water blast); PushingInfusionFeature
    // fbb97f35a41b71c4cbc36c5f3995b892 (no fire blast but Magma and Steam); TorrentInfusionFeature
    // 2aad85320d0751340a0786de073ee3d5 (the Fire blast among 13) (equivalent model).
    assert_eq!(
        infusion_reason(&build, 9, "spray"),
        "Needs Elemental Focus - Water"
    );
    assert!(!open_infusions(&build, 9).contains("pushing-infusion"));
    assert!(open_infusions(&build, 9).contains("torrent"));
    // Game: SecondaryElementWater faa5f1233600d864fa998bc0afe351ab gives the Water or the Cold blast; Spray and Pushing
    // open with the Water blast only (the blast choice is `infusions_need_a_blast_not_an_element`'s).
    let expanded = kineticist_with("fire", 9, &[(7, "kineticist/element/water")]);
    assert!(possible(&expanded));
    assert!(open_infusions(&expanded, 9).contains("spray"));
    assert!(open_infusions(&expanded, 9).contains("pushing-infusion"));
}

/// An infusion's "Prerequisites": Eruption needs Extended Range, Foxfire Infusion the Foxfire wild talent, Blade
/// Whirlwind Kinetic Blade, and Grappling Infusion one of Cloud, Deadly Earth and Wall.
#[test]
fn an_infusion_waits_for_the_infusion_or_talent_it_needs() {
    let build = kineticist("fire", 11);
    // Game: EruptionInfusion 00f8e4b846c367141afcd133f4a1c816 needs ExtendedRangeInfusion
    // cb2d9e6355dd33940b2bef49e544b0bf.
    assert_eq!(
        infusion_reason(&build, 9, "eruption"),
        "Needs Extended Range"
    );
    let ranged = kineticist_with("fire", 11, &[(3, &infusion("extended-range"))]);
    assert!(open_infusions(&ranged, 9).contains("eruption"));
    // Game: FoxfireInfusionFeature ae21c5369252ec74aa1fee89f1bc1b21 needs FoxfireFeature
    // 14c699ccd0564e04a9587b1845d16014.
    assert_eq!(
        infusion_reason(&build, 9, "foxfire-infusion"),
        "Needs Foxfire"
    );
    let foxfire = kineticist_with("fire", 11, &[(4, &talent("foxfire"))]);
    assert!(possible(&foxfire));
    assert!(open_infusions(&foxfire, 9).contains("foxfire-infusion"));
    // Game: BladeWhirlwindInfusion 80fdf049d396c33408a805d9e21a42e1 needs KineticBladeInfusion
    // 9ff81732daddb174aa8138ad1297c787 and kineticist 10.
    assert!(!open_infusions(&build, 11).contains("blade-whirlwind"));
    let blade = kineticist_with("fire", 11, &[(3, &infusion("kinetic-blade"))]);
    assert!(open_infusions(&blade, 11).contains("blade-whirlwind"));
    // Game: GrapplingInfusionFeature e2d70b95e80549b439d30df29a79cb58 needs one of CloudInfusion, DeadlyEarthInfusion
    // and WallInfusion c684335918896ce4ab13e96cec929796, and kineticist 10.
    assert!(!open_infusions(&build, 11).contains("grappling-infusion"));
    let wall = kineticist_with("fire", 13, &[(11, &infusion("wall"))]);
    assert!(possible(&wall));
    assert!(open_infusions(&wall, 13).contains("grappling-infusion"));
}

/// The Kinetic Sharpshooter's section: "A number of infusions are exclusive to the Sharpshooter" (Exploding Arrows,
/// Rending Arrows, Honed Infusion, Chain Arrows) and "a number of infusions that the Sharpshooter cannot use"
/// (Extended Range, Kinetic Blade, Eruption, Blade Whirlwind, Deadly Earth, Fragmentation).
#[test]
fn the_kinetic_sharpshooter_has_its_own_infusions_and_not_the_ones_it_cannot_use() {
    let sharpshooter = with_option(
        archetype_build("kineticist", SHARPSHOOTER, 3),
        1,
        0,
        "kineticist/element/fire",
    );
    assert!(possible(&sharpshooter));
    // Game: ExplodingArrowsInfusion 7c8ded7015114e4086b3dbdd16388b29 (PrerequisiteArchetypeLevel
    // KineticSharpshooterArchetype e0684d0a6ef44cae9d095f5318618a85 at 3); ExtendedRangeInfusion
    // cb2d9e6355dd33940b2bef49e544b0bf and KineticBladeInfusion 9ff81732daddb174aa8138ad1297c787
    // (PrerequisiteNoArchetype); BurningInfusionFeature 228ec37705a5e5240be3ff5622c4476f (no archetype check).
    let open = open_infusions(&sharpshooter, 3);
    assert!(open.contains("exploding-arrows"));
    assert!(!open.contains("extended-range"));
    assert!(!open.contains("kinetic-blade"));
    assert!(open.contains("burning-infusion"));
    let base = kineticist("fire", 3);
    assert!(!open_infusions(&base, 3).contains("exploding-arrows"));
    assert_eq!(
        infusion_reason(&base, 3, "exploding-arrows"),
        "Needs the Kinetic Sharpshooter archetype"
    );
    let picked = with_option(sharpshooter, 3, 0, &infusion("exploding-arrows"));
    assert!(possible(&picked));
}

/// The ticket's: the Blood Kineticist, the Kinetic Knight and the Kinetic Sharpshooter pick an infusion where their
/// table has "Infusion" after 1st level (the Blood Kineticist's own infusions at 5, 9 and 11 are fixed; the Kinetic
/// Knight has none at 3 or 9), and none at 1. Filled, each Build is possible.
#[test]
fn the_kineticist_archetypes_without_the_level_1_infusion_pick_infusions_at_their_tables_levels() {
    for (archetype, levels) in [
        // Game: BloodKineticistArchetype 365b50dba54efb74fa24c07e9b7a838c removes InfusionSelection at 1, 5, 9, 11.
        ("kineticist/blood-kineticist", vec![3, 13, 17, 19]),
        // Game: KineticKnightArchetype 7d61d9b2250260a45b18c5634524a8fb removes it at 1, 3, 9.
        ("kineticist/kinetic-knight", vec![5, 11, 13, 17, 19]),
        // Game: KineticSharpshooterArchetype e0684d0a6ef44cae9d095f5318618a85 removes it at 1.
        (SHARPSHOOTER, vec![3, 5, 9, 11, 13, 17, 19]),
    ] {
        let build = archetype_build("kineticist", archetype, 20);
        assert_eq!(
            levels_with_slot_of(&build, INFUSIONS_SELECTION),
            levels,
            "{archetype}"
        );
        assert!(!holds(&at(&build, 1), "kineticist/infusion"), "{archetype}");
        let filled = with_selections_filled(build);
        assert!(possible(&filled), "{archetype} filled");
        assert_eq!(
            levels.len(),
            levels
                .iter()
                .filter(|level| filled.picks.contains_key(&selection_slot(**level)))
                .count(),
            "{archetype}: every slot filled"
        );
    }
    // Game: KineticistClass 42a455d9ec1ad924d889272429eb8391 lists 7 archetypes.
    assert_eq!(class("kineticist").archetypes.len(), 7);
}

/// The game's base kineticist, Dark Elementalist, Elemental Engine, Overwhelming Soul and Psychokineticist pick an
/// infusion at 1 as well as at 3, 5, 9, 11, 13, 17 and 19, with no "Infusion" feature at 1. Filled, each Build is
/// possible.
#[test]
#[ignore = "waits on ticket 270: the game gives an infusion pick at kineticist 1"]
fn the_kineticist_and_the_archetypes_that_keep_it_pick_an_infusion_at_1() {
    let every = vec![1, 3, 5, 9, 11, 13, 17, 19];
    // Game: KineticistProgression b79e92dd495edd64e90fb483c504b8df.
    assert_eq!(
        levels_with_slot_of(&build("kineticist", 20), INFUSIONS_SELECTION),
        every
    );
    // Game: DarkElementalistArchetype f12f18ae8842425489d29f302e69134c, ElementalEngineArchetype
    // 4a8324d676d642c99edcdda6988ca3b1, OverwhelmingSoulArchetype aa11888104d17f7459851e8d559ffa98,
    // PsychokineticistArchetype f2847dd4b12fffd41beaa3d7120d27ad: none removes InfusionSelection.
    for archetype in [
        "kineticist/dark-elementalist",
        "kineticist/elemental-engine",
        "kineticist/overwhelming-soul",
        "kineticist/psychokineticist",
    ] {
        let build = archetype_build("kineticist", archetype, 20);
        assert_eq!(
            levels_with_slot_of(&build, INFUSIONS_SELECTION),
            every,
            "{archetype}"
        );
        assert!(!holds(&at(&build, 1), "kineticist/infusion"), "{archetype}");
        let filled = with_selections_filled(build);
        assert!(possible(&filled), "{archetype} filled");
    }
}
