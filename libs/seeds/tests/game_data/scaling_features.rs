//! Features that improve in steps are granted once for each step (ticket 170), where the game's level-up names it:
//! the slayer's Studied Target at 1, 5, 10, 15 and 20, the Draconic bloodline's resistances at bloodline levels 3,
//! 9 and 15 and the bloodrager's at 4, 8 and 16. A feature's count is how many steps it has reached. A selection that
//! shares a bloodline gets the steps its rows give, and one that skips a feature's first row skips its steps too
//! (ticket 179: the Dragonheir Scion).

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
    // The fixed bloodlines (Empyreal: Celestial; Sage: Arcane; Sylvan: Fey) have no Draconic option to pick, so none
    // holds the feature. (The Seeker shares every bloodline, Draconic too, and skips the rows that carry the steps:
    // `every_sharer_of_a_stepped_bloodline_holds_the_steps_its_rows_give`.)
    for archetype in [
        "sorcerer/empyreal-sorcerer",
        "sorcerer/sage-sorcerer",
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

/// The Dragonheir Scion (ticket 179) takes a Draconic bloodline for its energy type and none of the bloodline's
/// powers: its own table gives Draconic Defences at 3 in place of the resistances, Wings at 15 and Might of Wyrms at
/// 20 (the guide: "Gains a Draconic Bloodline", and its level progression lists no bloodline power). Its selection
/// skips every row of the bloodline, the resistances' steps at 9 and 15 among them.
#[test]
fn a_dragonheir_scion_of_each_colour_never_holds_the_bloodlines_powers() {
    const ALL_LEVELS: [u8; 20] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    for colour in COLOURS {
        let build = with_option(
            archetype_build("fighter", "fighter/dragonheir-scion", 20),
            1,
            0,
            &format!("sorcerer/bloodline/draconic-{colour}"),
        );
        assert!(engine().is_possible(&build), "{colour}");
        for feature in [
            "sorcerer/bloodline/draconic/draconic-resistances",
            "sorcerer/bloodline/draconic/draconic-arcana",
            "sorcerer/bloodline/draconic/grow-claws",
            "sorcerer/bloodline/draconic/power-of-wyrms",
        ] {
            assert_eq!(
                counts(&build, feature, &ALL_LEVELS),
                [0; 20],
                "{colour}: {feature}"
            );
        }
        // What its own table gives, at the level it gives it and not before.
        for (feature, level) in [
            ("fighter/dragonheir-scion/fearful-might", 2),
            ("fighter/dragonheir-scion/draconic-defences", 3),
            ("fighter/dragonheir-scion/draconic-strike", 4),
            ("fighter/dragonheir-scion/draconic-presence", 6),
            ("fighter/dragonheir-scion/might-of-wyrms", 20),
        ] {
            assert_eq!(
                counts(&build, feature, &[level - 1, level, 20]),
                [0, 1, 1],
                "{colour}: {feature}"
            );
        }
        let feat = |key: &str, level: u8| at(&build, level).feat_count(&rules::FeatKey::new(key));
        assert_eq!(
            (
                feat("arcane-strike", 1),
                feat("wings", 14),
                feat("wings", 15)
            ),
            (1, 0, 1),
            "{colour}"
        );
    }
}

/// Every other selection that takes a bloodline carrying a stepped feature (ticket 179's sweep), by what its rows
/// give: the steps, at the option level its classes make, or none where it skips the rows or gives only the lists.
#[test]
fn every_sharer_of_a_stepped_bloodline_holds_the_steps_its_rows_give() {
    const DRACONIC: &str = "sorcerer/bloodline/draconic/draconic-resistances";
    const RED: &str = "sorcerer/bloodline/draconic-red";
    // The Eldritch Scion (magus) "gains spells and abilities related to his bloodline (for example he might grow Wings
    // at level 15)": the bloodline's rows by magus level.
    let scion = with_option(
        archetype_build("magus", "magus/eldritch-scion", 20),
        1,
        0,
        RED,
    );
    assert!(engine().is_possible(&scion));
    assert_eq!(
        counts(&scion, DRACONIC, &[2, 3, 8, 9, 14, 15, 20]),
        [0, 1, 1, 2, 2, 3, 3]
    );
    // The Seeker keeps a bloodline's arcana, spells and feats and skips the rows at 3, 9 and 15 (its own replace
    // them), so a Draconic Seeker holds no resistance, and an Abyssal one no Strength of the Abyss (9, 13, 17).
    let seeker = |option: &str| {
        with_option(
            archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 20),
            1,
            0,
            option,
        )
    };
    assert!(engine().is_possible(&seeker(RED)));
    assert_eq!(counts(&seeker(RED), DRACONIC, &[3, 9, 15, 20]), [0; 4]);
    assert_eq!(
        counts(
            &seeker(ABYSSAL),
            "sorcerer/bloodline/abyssal/strength-of-the-abyss",
            &[9, 13, 17, 20]
        ),
        [0; 4]
    );
    // A Dragon Disciple who had no bloodline picks a Draconic one: "your bloodline level is your class level", so
    // the steps come at Dragon Disciple 3 and 9 (Build levels 9 and 15 after six bard levels).
    let mut disciple = build("bard", 16);
    for level in &mut disciple.levels[..6] {
        level
            .skill_ranks
            .insert(rules::SkillKey::new("knowledge-arcana"), 1);
    }
    for level in &mut disciple.levels[6..] {
        level.class = rules::ClassKey::new("dragon-disciple");
    }
    let disciple = with_option(disciple, 7, 0, RED);
    assert!(engine().is_possible(&disciple));
    assert_eq!(
        counts(&disciple, DRACONIC, &[8, 9, 14, 15, 16]),
        [0, 1, 1, 2, 2]
    );
    // The Mythic Second Bloodline is held through the sorcerer's own selection, by sorcerer levels: a Draconic one
    // taken at MR 1 (level 6) has its first resistance at once and the steps at 9 and 15 (the gold Build Jack's).
    let second = |class: &str, first: &str, ability: &str, option: &str| {
        let mut build = with_option(build(class, 20), 1, 0, first);
        build.picks.insert(
            rules::SlotId::Mythic { rank: 1 },
            rules::Pick::mythic_ability(rules::MythicAbilityKey::new(ability)),
        );
        build.picks.insert(
            rules::SlotId::MythicSelection { rank: 1, index: 0 },
            pick(option),
        );
        build
    };
    let sorcerer = second("sorcerer", ABYSSAL, "second-bloodline", RED);
    assert!(engine().is_possible(&sorcerer));
    assert_eq!(
        counts(&sorcerer, DRACONIC, &[5, 6, 8, 9, 14, 15, 20]),
        [0, 1, 1, 2, 2, 3, 3]
    );
    // The bloodrager's Second Bloodline, the same by bloodrager levels.
    let bloodrager = second(
        "bloodrager",
        "bloodrager/bloodline/abyssal",
        "second-bloodrager-bloodline",
        "bloodrager/bloodline/draconic-red",
    );
    assert!(engine().is_possible(&bloodrager));
    assert_eq!(
        counts(
            &bloodrager,
            "bloodrager/bloodline/draconic/draconic-resistance",
            &[5, 6, 7, 8, 15, 16, 20]
        ),
        [0, 1, 1, 2, 2, 3, 3]
    );
    // Bloodline Ascendance grants a bloodline's 20th-level row, which carries no step of any bloodline.
    let ascendance = DATA
        .class_features()
        .find(|feature| feature.key.as_str() == "sorcerer/bloodline-ascendance")
        .and_then(|feature| feature.option_row.as_ref())
        .unwrap();
    assert_eq!(ascendance.level, 20);
    let bloodlines = DATA
        .selections()
        .find(|selection| selection.key == ascendance.selection)
        .unwrap();
    for option in &bloodlines.options {
        let earlier: Vec<_> = option
            .levels
            .iter()
            .filter(|row| row.level < 20)
            .flat_map(|row| &row.features)
            .collect();
        for row in option.levels.iter().filter(|row| row.level == 20) {
            assert!(
                row.features
                    .iter()
                    .all(|feature| !earlier.contains(&feature)),
                "{}",
                option.key
            );
        }
    }
}

/// A skipped level drops the option's whole row, so a selection that skips the row where a feature is first granted
/// must skip every later row that grants it again (a step), or it gains a step of a feature it never had: the
/// Dragonheir Scion's resistances at 9 before ticket 179. Every selection, over every option it offers.
#[test]
fn no_selection_gains_a_step_of_a_feature_whose_first_row_it_skips() {
    let mut leaks = Vec::new();
    for selection in DATA.selections() {
        if selection.skipped_levels.is_empty() || selection.feat_lists_only {
            continue;
        }
        for key in selection.offered() {
            let Some(option) = DATA.selection_option(key) else {
                continue;
            };
            for row in &option.levels {
                for feature in &row.features {
                    let first = option
                        .levels
                        .iter()
                        .find(|earlier| earlier.features.contains(feature))
                        .map(|earlier| earlier.level)
                        .unwrap();
                    if first != row.level
                        && selection.skipped_levels.contains(&first)
                        && !selection.skipped_levels.contains(&row.level)
                    {
                        leaks.push(format!(
                            "{} through {}: {} at {} (first at {first})",
                            key, selection.key, feature, row.level
                        ));
                    }
                }
            }
        }
    }
    assert!(leaks.is_empty(), "{leaks:#?}");
}
