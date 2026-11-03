//! The kineticist's infusions (ticket 59).

use std::collections::BTreeSet;

use rules::{ArchetypeKey, Build, ClassKey, FeatureKey, Prerequisite, SelectionOptionKey};

use crate::support::*;

/// Each infusion of the guide's list (`kineticist#infusions`), with its level, its elements (none: Universal), the
/// infusions its "Prerequisites" names (one of them) and who may take it.
#[allow(clippy::type_complexity)]
const INFUSIONS: [(&str, u8, &[&str], &[&str], Sharpshooter); 32] = [
    ("burning-infusion", 1, &["fire"], &[], Sharpshooter::Any),
    ("dazzling-infusion", 1, &["fire"], &[], Sharpshooter::Any),
    ("exploding-arrows", 1, &[], &[], Sharpshooter::Only),
    ("extended-range", 1, &[], &[], Sharpshooter::Not),
    ("fan-of-flames", 1, &["fire"], &[], Sharpshooter::Any),
    ("kinetic-blade", 1, &[], &[], Sharpshooter::Not),
    (
        "pushing-infusion",
        1,
        &["air", "earth", "water"],
        &[],
        Sharpshooter::Any,
    ),
    ("bowling-infusion", 2, &["earth"], &[], Sharpshooter::Any),
    (
        "entangling-infusion",
        2,
        &["earth", "water"],
        &[],
        Sharpshooter::Any,
    ),
    (
        "magnetic-infusion",
        2,
        &["air", "earth"],
        &[],
        Sharpshooter::Any,
    ),
    ("spindle", 2, &[], &[], Sharpshooter::Any),
    (
        "eruption",
        3,
        &["fire"],
        &["extended-range"],
        Sharpshooter::Not,
    ),
    (
        "foxfire-infusion",
        3,
        &["fire"],
        &["kineticist/wild-talents/foxfire"],
        Sharpshooter::Any,
    ),
    ("rare-metal-infusion", 3, &["earth"], &[], Sharpshooter::Any),
    ("synaptic-infusion", 3, &["air"], &[], Sharpshooter::Any),
    (
        "torrent",
        3,
        &["air", "fire", "water"],
        &[],
        Sharpshooter::Any,
    ),
    ("cyclone", 4, &["air"], &[], Sharpshooter::Any),
    ("detonation", 4, &["fire"], &[], Sharpshooter::Any),
    ("flash-infusion", 4, &["fire"], &[], Sharpshooter::Any),
    ("spray", 4, &["water"], &[], Sharpshooter::Any),
    (
        "blade-whirlwind",
        5,
        &[],
        &["kinetic-blade"],
        Sharpshooter::Not,
    ),
    ("chilling-infusion", 5, &["water"], &[], Sharpshooter::Any),
    (
        "grappling-infusion",
        5,
        &[],
        &["cloud", "deadly-earth", "wall"],
        Sharpshooter::Any,
    ),
    ("rending-arrows", 5, &[], &[], Sharpshooter::Only),
    ("unraveling-infusion", 5, &["fire"], &[], Sharpshooter::Any),
    ("wall", 5, &[], &[], Sharpshooter::Any),
    (
        "deadly-earth",
        6,
        &["earth"],
        &["extended-range"],
        Sharpshooter::Not,
    ),
    ("honed-infusion", 6, &[], &[], Sharpshooter::Only),
    (
        "cloud",
        7,
        &["air", "water"],
        &["extended-range"],
        Sharpshooter::Any,
    ),
    (
        "fragmentation",
        7,
        &["earth", "water"],
        &["extended-range"],
        Sharpshooter::Not,
    ),
    ("pure-flame-infusion", 7, &["fire"], &[], Sharpshooter::Any),
    ("chain-arrows", 8, &[], &[], Sharpshooter::Only),
];

/// What an infusion of `level` needs: the kineticist level twice its own, one of its elements, one of the options its
/// "Prerequisites" names (an infusion, or the wild talent the key names), and the Sharpshooter's archetype or its
/// absence.
fn infusion_prerequisite(
    level: u8,
    elements: &[&str],
    needs: &[&str],
    sharpshooter: Sharpshooter,
) -> Prerequisite {
    let one_of = |keys: Vec<Prerequisite>| match keys.len() {
        1 => keys.into_iter().next().unwrap(),
        _ => Prerequisite::Any(keys),
    };
    let mut all = vec![Prerequisite::ClassLevel {
        class: ClassKey::new("kineticist"),
        level: 2 * level,
    }];
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

/// "The Kineticist may select any Infusion for which she qualifies. She gains new Infusions at 3rd, 5th, 9th, 11th,
/// 13th, 17th and 19th levels." The guide's 32 infusions, each with its level, elements and prerequisites; the
/// "Infusion" feature stays only on the 1st-level rows whose table has it, and no feat gives an infusion.
#[test]
fn infusions_are_a_selection_of_32_with_their_level_and_elements() {
    let kineticist = class("kineticist");
    let selection = kineticist
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == INFUSIONS_SELECTION)
        .expect("the infusion selection");
    assert_eq!(selection.name, "Infusion");
    let keys: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    let expected: BTreeSet<String> = INFUSIONS.iter().map(|(name, ..)| infusion(name)).collect();
    assert_eq!(keys, expected);
    for (name, level, elements, needs, sharpshooter) in INFUSIONS {
        assert_eq!(
            selection_option(&infusion(name)).prerequisite,
            Some(infusion_prerequisite(level, elements, needs, sharpshooter)),
            "{name}"
        );
    }
    // The guide's "Dazziing Infusion" is named as the game names it; its link is the guide's anchor.
    let dazzling = selection_option(&infusion("dazzling-infusion"));
    assert_eq!(dazzling.name, "Dazzling Infusion");
    assert_eq!(dazzling.guide.0, "kineticist#dazziing-infusion");

    let levels: Vec<u8> = kineticist
        .levels
        .iter()
        .filter(|row| {
            row.selections
                .iter()
                .any(|s| s.as_str() == INFUSIONS_SELECTION)
        })
        .map(|row| row.level)
        .collect();
    assert_eq!(levels, [3, 5, 9, 11, 13, 17, 19]);
    let feature = FeatureKey::new("kineticist/infusion");
    let rows_with_feature: Vec<u8> = kineticist
        .levels
        .iter()
        .filter(|row| row.features.contains(&feature))
        .map(|row| row.level)
        .collect();
    assert_eq!(rows_with_feature, [1]);
    assert!(
        kineticist
            .archetypes
            .iter()
            .flat_map(|a| &a.levels)
            .all(|row| row.level == 1 || !row.features.contains(&feature))
    );
    // The Sharpshooter's "Infusion (Kinetic Sharpshooter)" was its pick; it's gone.
    assert!(kineticist.archetypes.iter().all(|a| {
        a.features.iter().all(|f| {
            f.key.as_str() != "kineticist/kinetic-sharpshooter/infusion-kinetic-sharpshooter"
        })
    }));
    assert!(DATA.feats.iter().all(|feat| {
        feat.extra_pick_of
            .iter()
            .all(|s| s.as_str() != INFUSIONS_SELECTION)
    }));
}

/// The ticket's: a kineticist 3 has one infusion slot, "Infusion", offering every infusion and leaving open only its
/// element's and the Universal ones of level 1 (but the Sharpshooter's own). A kineticist 2 has none.
#[test]
fn a_kineticist_3_has_one_infusion_slot_offering_its_elements_and_universal_infusions_of_level_1() {
    let engine = engine();
    let build = kineticist("fire", 3);
    assert!(possible(&build));
    assert_eq!(levels_with_slot_of(&build, INFUSIONS_SELECTION), [3]);
    assert_eq!(offered(&build, 3).len(), 32);
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
    let earth: BTreeSet<String> = ["extended-range", "kinetic-blade", "pushing-infusion"]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    assert_eq!(open_infusions(&kineticist("earth", 3), 3), earth);
    let sheet = engine.evaluate(&build);
    assert_eq!(
        engine.slot_label(sheet.slot(&selection_slot(3)).unwrap()),
        "Infusion"
    );
    let picked = kineticist_with("fire", 3, &[(3, &infusion("extended-range"))]);
    assert!(possible(&picked));
    assert_eq!(
        levels_with_slot_of(&kineticist("fire", 2), INFUSIONS_SELECTION),
        Vec::<u8>::new()
    );
    // The 1st-level "Infusion" stays a feature.
    assert!(holds(&at(&build, 1), "kineticist/infusion"));
}

/// The ticket's: "The minimum Kineticist level to use a given Infusion is twice the Infusion's level": one of a higher
/// level is locked with a reason and opens at its level; one taken isn't offered again.
#[test]
fn a_higher_level_infusion_is_locked_with_a_reason_and_opens_at_its_level() {
    let build = kineticist("fire", 19);
    assert_eq!(
        infusion_reason(&build, 3, "detonation"),
        "Needs Kineticist level 8 (have 3)"
    );
    assert_eq!(
        infusion_reason(&build, 5, "detonation"),
        "Needs Kineticist level 8 (have 5)"
    );
    assert!(open_infusions(&build, 9).contains("detonation"));
    assert_eq!(
        infusion_reason(&build, 13, "pure-flame-infusion"),
        "Needs Kineticist level 14 (have 13)"
    );
    assert!(open_infusions(&build, 17).contains("pure-flame-infusion"));
    // Each Fire or Universal infusion with nothing else to need opens at its own level.
    for (name, level, elements, needs, sharpshooter) in INFUSIONS {
        if !needs.is_empty()
            || !matches!(sharpshooter, Sharpshooter::Any)
            || !(elements.is_empty() || elements.contains(&"fire"))
        {
            continue;
        }
        for at in [3, 5, 9, 11, 13, 17, 19] {
            assert_eq!(
                open_infusions(&build, at).contains(name),
                2 * level <= at,
                "{name} at {at}"
            );
        }
    }

    let early = kineticist_with("fire", 3, &[(3, &infusion("detonation"))]);
    assert_eq!(
        why_not(&early, &selection_slot(3)).and_then(|issue| engine().issue_text(&issue)),
        Some("Needs Kineticist level 8 (have 3)".to_owned())
    );
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
    assert_eq!(
        infusion_reason(&build, 9, "spray"),
        "Needs Elemental Focus - Water"
    );
    assert!(!open_infusions(&build, 9).contains("pushing-infusion"));
    assert!(open_infusions(&build, 9).contains("torrent"));
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
    assert_eq!(
        infusion_reason(&build, 9, "eruption"),
        "Needs Extended Range"
    );
    let ranged = kineticist_with("fire", 11, &[(3, &infusion("extended-range"))]);
    assert!(open_infusions(&ranged, 9).contains("eruption"));
    assert_eq!(
        infusion_reason(&build, 9, "foxfire-infusion"),
        "Needs Foxfire"
    );
    let foxfire = kineticist_with("fire", 11, &[(4, &talent("foxfire"))]);
    assert!(possible(&foxfire));
    assert!(open_infusions(&foxfire, 9).contains("foxfire-infusion"));
    assert!(!open_infusions(&build, 11).contains("blade-whirlwind"));
    let blade = kineticist_with("fire", 11, &[(3, &infusion("kinetic-blade"))]);
    assert!(open_infusions(&blade, 11).contains("blade-whirlwind"));
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

/// The ticket's: each kineticist archetype picks an infusion where its table has "Infusion" after 1st level (the
/// Blood Kineticist's own infusions at 5, 9 and 11 are fixed; the Kinetic Knight has none at 3 or 9), and holds the
/// "Infusion" feature at 1 where its table has it there. Filled, each Build is possible.
#[test]
fn every_kineticist_archetype_picks_infusions_at_its_tables_levels() {
    let every = vec![3, 5, 9, 11, 13, 17, 19];
    assert_eq!(
        levels_with_slot_of(&build("kineticist", 20), INFUSIONS_SELECTION),
        every
    );
    for (archetype, levels, feature_at_1) in [
        ("kineticist/blood-kineticist", vec![3, 13, 17, 19], false),
        ("kineticist/dark-elementalist", every.clone(), true),
        ("kineticist/elemental-engine", every.clone(), true),
        ("kineticist/kinetic-knight", vec![5, 11, 13, 17, 19], false),
        (SHARPSHOOTER, every.clone(), false),
        ("kineticist/overwhelming-soul", every.clone(), true),
        ("kineticist/psychokineticist", every.clone(), true),
    ] {
        let build = archetype_build("kineticist", archetype, 20);
        assert_eq!(
            levels_with_slot_of(&build, INFUSIONS_SELECTION),
            levels,
            "{archetype}"
        );
        assert_eq!(
            holds(&at(&build, 1), "kineticist/infusion"),
            feature_at_1,
            "{archetype}"
        );
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
    assert_eq!(class("kineticist").archetypes.len(), 7);
}
