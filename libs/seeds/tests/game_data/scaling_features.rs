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

const STUDIED_TARGET: &str = "slayer/studied-target";

#[test]
fn a_slayers_studied_target_rises_at_5_10_15_and_20() {
    let build = build("slayer", 20);
    assert!(engine().is_possible(&build));
    assert_eq!(
        counts(&build, STUDIED_TARGET, &[1, 4, 5, 9, 10, 14, 15, 19, 20]),
        [1, 1, 2, 2, 3, 3, 4, 4, 5]
    );
}

#[test]
fn every_slayer_archetype_that_keeps_studied_target_steps_it_too() {
    // The Imitator gives it up; every other archetype keeps it (the Bloodseeker needs an evil alignment).
    let mut kept = Vec::new();
    for archetype in &class("slayer").archetypes {
        let plain = archetype_build("slayer", archetype.key.as_str(), 20);
        let build = if engine().is_possible(&plain) {
            plain
        } else {
            realigned(plain, rules::Alignment::ChaoticEvil)
        };
        if !engine().is_possible(&build) {
            // The Imitator needs a human.
            assert_eq!(archetype.key.as_str(), "slayer/imitator");
            continue;
        }
        kept.push(archetype.key.as_str());
        assert_eq!(
            counts(&build, STUDIED_TARGET, &[1, 5, 10, 15, 20]),
            [1, 2, 3, 4, 5],
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
        let build = archetype_build(class_key, archetype, 20);
        assert_eq!(
            counts(&build, feature, &[1, 4, 5, 10, 15, 20]),
            [1, 1, 2, 3, 4, 5],
            "{archetype}"
        );
    }
}

#[test]
fn a_draconic_sorcerers_resistances_rise_at_bloodline_levels_9_and_15() {
    for colour in [
        "black", "blue", "brass", "bronze", "copper", "gold", "green", "red", "silver", "white",
    ] {
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
    let build = with_option(
        build("bloodrager", 20),
        1,
        0,
        "bloodrager/bloodline/draconic-red",
    );
    assert!(engine().is_possible(&build));
    assert_eq!(
        counts(
            &build,
            "bloodrager/bloodline/draconic/draconic-resistance",
            &[3, 4, 7, 8, 15, 16, 20]
        ),
        [0, 1, 1, 2, 2, 3, 3]
    );
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
