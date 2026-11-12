//! Features that improve in steps are granted once for each step (ticket 170), where the game's level-up names it:
//! the slayer's Studied Target at 1, 5, 10, 15 and 20, the Draconic bloodline's resistances at bloodline levels 3,
//! 9 and 15 and the bloodrager's at 4, 8 and 16. A feature's count is how many steps it has reached.

use rules::{Build, FeatureKey};

use crate::support::*;

/// `feature`'s count at each of `levels`, after `build`'s level.
fn counts(build: &Build, feature: &str, levels: &[u8]) -> Vec<u8> {
    levels
        .iter()
        .map(|&level| at(build, level).feature_count(&FeatureKey::new(feature)))
        .collect()
}

const COLOURS: [&str; 10] = [
    "black", "blue", "brass", "bronze", "copper", "gold", "green", "red", "silver", "white",
];

const STUDIED_TARGET: &str = "slayer/studied-target";

/// The levels before and at each of the slayer's steps, and the count a keeping slayer holds at each.
const STEP_LEVELS: [u8; 9] = [1, 4, 5, 9, 10, 14, 15, 19, 20];
const STEP_COUNTS: [u8; 9] = [1, 1, 2, 2, 3, 3, 4, 4, 5];

fn human(mut build: Build) -> Build {
    build.start.race = Some(rules::RaceKey::new("human"));
    faithful(build)
}

#[test]
fn a_slayers_studied_target_rises_at_5_10_15_and_20() {
    let build = build("slayer", 20);
    assert!(engine().is_possible(&build));
    assert_eq!(counts(&build, STUDIED_TARGET, &STEP_LEVELS), STEP_COUNTS);
}

#[test]
fn every_slayer_archetype_that_keeps_studied_target_steps_it_too() {
    // The Imitator gives it up (below); every other archetype keeps it (the Bloodseeker needs an evil alignment).
    let mut kept = Vec::new();
    for archetype in &class("slayer").archetypes {
        let plain = human(archetype_build("slayer", archetype.key.as_str(), 20));
        let build = if engine().is_possible(&plain) {
            plain
        } else {
            human(realigned(plain, rules::Alignment::ChaoticEvil))
        };
        assert!(engine().is_possible(&build), "{}", archetype.key);
        if archetype.key.as_str() == "slayer/imitator" {
            continue;
        }
        kept.push(archetype.key.as_str());
        assert_eq!(
            counts(&build, STUDIED_TARGET, &STEP_LEVELS),
            STEP_COUNTS,
            "{}",
            archetype.key
        );
    }
    assert_eq!(
        kept,
        [
            "slayer/arcane-enforcer",
            "slayer/bloodseeker",
            "slayer/deliverer",
            "slayer/executioner",
            "slayer/spawn-slayer",
            "slayer/stygian-slayer",
            "slayer/vanguard",
        ]
    );
}

/// The Imitator "Loses Studied Target" (the guide): its own rows at 5 and 20 keep the base class's rows from granting
/// it back there, and Defensive Study, which needs the feature, stays closed to it.
#[test]
fn a_human_imitator_never_holds_studied_target() {
    let build = human(archetype_build("slayer", "slayer/imitator", 20));
    assert!(engine().is_possible(&build));
    assert_eq!(counts(&build, STUDIED_TARGET, &STEP_LEVELS), [0; 9]);
    assert_eq!(
        counts(&build, "slayer/studied-target-swift", &[7, 20]),
        [0, 0]
    );
    // What the guide's table gives at 20 stays: Master Slayer and a slayer talent.
    assert_eq!(counts(&build, "slayer/master-slayer", &[19, 20]), [0, 1]);
    let defensive_study = DATA
        .mythic_abilities
        .iter()
        .find(|ability| ability.key.as_str() == "defensive-study")
        .unwrap();
    let prerequisite = defensive_study.prerequisite.as_ref().unwrap();
    assert!(at(&build, 20).check(prerequisite).is_err());
    assert!(
        at(&human(build_of_slayer()), 20)
            .check(prerequisite)
            .is_ok()
    );
}

fn build_of_slayer() -> Build {
    build("slayer", 20)
}

#[test]
fn the_other_classes_copies_step_at_the_same_levels() {
    for (class_key, archetype, feature) in [
        (
            "inquisitor",
            "inquisitor/sanctified-slayer",
            "inquisitor/sanctified-slayer/studied-target",
        ),
        (
            "hunter",
            "hunter/tandem-executioner",
            "hunter/tandem-executioner/studied-target",
        ),
        (
            "paladin",
            "paladin/divine-scion",
            "paladin/divine-scion/studied-target",
        ),
    ] {
        let build = realigned(
            archetype_build(class_key, archetype, 20),
            rules::Alignment::LawfulGood,
        );
        assert!(engine().is_possible(&build), "{archetype}");
        assert_eq!(
            counts(&build, feature, &[1, 4, 5, 10, 15, 20]),
            [1, 1, 2, 3, 4, 5],
            "{archetype}"
        );
    }
}

#[test]
fn a_draconic_sorcerers_resistances_rise_at_bloodline_levels_9_and_15() {
    for colour in COLOURS {
        let option = format!("sorcerer/bloodline/draconic-{colour}");
        let build = with_option(build("sorcerer", 20), 1, 0, &option);
        assert!(engine().is_possible(&build), "{colour}");
        assert_eq!(
            counts(
                &build,
                "sorcerer/bloodline/draconic/draconic-resistances",
                &[2, 3, 8, 9, 14, 15, 20]
            ),
            [0, 1, 1, 2, 2, 3, 3],
            "{colour}"
        );
    }
}

#[test]
fn a_draconic_bloodragers_resistance_rises_at_8_and_16() {
    for colour in COLOURS {
        let option = format!("bloodrager/bloodline/draconic-{colour}");
        let build = with_option(build("bloodrager", 20), 1, 0, &option);
        assert!(engine().is_possible(&build), "{colour}");
        assert_eq!(
            counts(
                &build,
                "bloodrager/bloodline/draconic/draconic-resistance",
                &[3, 4, 7, 8, 15, 16, 20]
            ),
            [0, 1, 1, 2, 2, 3, 3],
            "{colour}"
        );
    }
}

#[test]
fn the_bloodragers_other_resistances_rise_where_the_guide_says() {
    let steps: [(&str, &str, &[u8], &[u8]); 3] = [
        (
            "bloodrager/bloodline/abyssal",
            "bloodrager/bloodline/abyssal/demon-resistances",
            &[7, 8, 15, 16],
            &[0, 1, 1, 2],
        ),
        (
            "bloodrager/bloodline/celestial",
            "bloodrager/bloodline/celestial/celestial-resistances",
            &[3, 4, 11, 12],
            &[0, 1, 1, 2],
        ),
        (
            "bloodrager/bloodline/serpentine",
            "bloodrager/bloodline/serpentine/serpentine-resistances",
            &[3, 4, 7, 8, 11, 12, 15, 16, 19, 20],
            &[0, 1, 1, 2, 2, 3, 3, 4, 4, 5],
        ),
    ];
    for (option, feature, levels, expected) in steps {
        let build = with_option(build("bloodrager", 20), 1, 0, option);
        assert!(engine().is_possible(&build), "{option}");
        assert_eq!(counts(&build, feature, levels), expected, "{option}");
    }
}

/// Every archetype of the sorcerer and the bloodrager, by what it does to the stepped resistances (ticket 170's
/// general check): the steps sit in the bloodline option's own rows, so an archetype that keeps the bloodline's powers
/// keeps the steps, and one that swaps the bloodline for a fixed one or keeps only its arcana, spells and feats has
/// none of the Draconic ones.
#[test]
fn the_sorcerer_and_bloodrager_archetypes_hold_the_steps_their_bloodline_gives() {
    const DRACONIC: &str = "sorcerer/bloodline/draconic/draconic-resistances";
    // Keeping archetypes: the bloodline picked at the first selection.
    for archetype in [
        "sorcerer/crossblooded",
        "sorcerer/geomancer",
        "sorcerer/overwhelming-mage",
    ] {
        let build = with_option(
            archetype_build("sorcerer", archetype, 20),
            1,
            0,
            "sorcerer/bloodline/draconic-red",
        );
        assert!(engine().is_possible(&build), "{archetype}");
        assert_eq!(
            counts(&build, DRACONIC, &[2, 3, 8, 9, 14, 15]),
            [0, 1, 1, 2, 2, 3],
            "{archetype}"
        );
    }
    // The Crossblooded's second bloodline gives no powers: a Draconic one second holds none.
    let crossblooded = with_option(
        with_option(
            archetype_build("sorcerer", "sorcerer/crossblooded", 20),
            1,
            0,
            "sorcerer/bloodline/abyssal",
        ),
        1,
        1,
        "sorcerer/bloodline/draconic-red",
    );
    assert!(engine().is_possible(&crossblooded));
    assert_eq!(counts(&crossblooded, DRACONIC, &[3, 9, 15, 20]), [0; 4]);
    // The fixed or reduced bloodlines (Empyreal: Celestial; Sage: Arcane; Sylvan: Fey; Seeker: arcana, spells,
    // feats and the 20th-level power) have no Draconic option to pick, so none holds the feature.
    for archetype in [
        "sorcerer/empyreal-sorcerer",
        "sorcerer/sage-sorcerer",
        "sorcerer/seeker-sorcerer",
        "sorcerer/sylvan-sorcerer",
    ] {
        let option = format!("{archetype}/bloodline");
        let draconic = DATA
            .selections()
            .filter(|selection| selection.key.as_str() == option)
            .flat_map(|selection| &selection.options)
            .any(|option| option.key.as_str().contains("draconic"));
        assert!(!draconic, "{archetype}");
    }
    // The bloodrager's archetypes keep the bloodline's rows; the Primalist's rage-power choice is a known
    // consequence (see the ticket), not asserted here.
    for archetype in [
        "bloodrager/bloodrider",
        "bloodrager/greenrager",
        "bloodrager/primalist",
        "bloodrager/spelleater",
        "bloodrager/steelblood",
    ] {
        let build = with_option(
            archetype_build("bloodrager", archetype, 20),
            1,
            0,
            "bloodrager/bloodline/draconic-red",
        );
        assert!(engine().is_possible(&build), "{archetype}");
        assert_eq!(
            counts(
                &build,
                "bloodrager/bloodline/draconic/draconic-resistance",
                &[3, 4, 7, 8, 15, 16]
            ),
            [0, 1, 1, 2, 2, 3],
            "{archetype}"
        );
    }
    // The Reformed Fiend shares the Abyssal bloodline's rows.
    let mut fiend = archetype_build("bloodrager", "bloodrager/reformed-fiend", 20);
    fiend.start.race = Some(rules::RaceKey::new("tiefling"));
    let fiend = with_option(
        realigned(fiend, rules::Alignment::NeutralGood),
        1,
        0,
        "bloodrager/bloodline/abyssal",
    );
    assert!(engine().is_possible(&fiend));
    assert_eq!(
        counts(
            &fiend,
            "bloodrager/bloodline/abyssal/demon-resistances",
            &[7, 8, 15, 16]
        ),
        [0, 1, 1, 2]
    );
}
