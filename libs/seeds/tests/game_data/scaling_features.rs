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
        // The steps of ticket 177: Fearful Might 2, 6, 10, 14 and 18, Draconic Defences 3, 7 and 13.
        for (feature, level, last) in [
            ("fighter/dragonheir-scion/fearful-might", 2, 5),
            ("fighter/dragonheir-scion/draconic-defences", 3, 3),
            ("fighter/dragonheir-scion/draconic-strike", 4, 1),
            ("fighter/dragonheir-scion/draconic-presence", 6, 1),
            ("fighter/dragonheir-scion/might-of-wyrms", 20, 1),
        ] {
            assert_eq!(
                counts(&build, feature, &[level - 1, level, 20]),
                [0, 1, last],
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

// ---- ticket 177: the other stepped features ----------------------------------------------------------------------

const ALIGNMENTS: [rules::Alignment; 9] = [
    rules::Alignment::LawfulGood,
    rules::Alignment::NeutralGood,
    rules::Alignment::ChaoticGood,
    rules::Alignment::LawfulNeutral,
    rules::Alignment::TrueNeutral,
    rules::Alignment::ChaoticNeutral,
    rules::Alignment::LawfulEvil,
    rules::Alignment::NeutralEvil,
    rules::Alignment::ChaoticEvil,
];

/// The levels at which `build`'s count of `feature` rises, a level once for each rise: the steps the sheet gains it
/// at.
fn gains(build: &Build, feature: &str) -> Vec<u8> {
    let key = FeatureKey::new(feature);
    let sheet = engine().evaluate(build);
    let mut previous = 0;
    let mut gained = Vec::new();
    for level in 1..=build.levels.len() as u8 {
        let count = sheet.at(level).unwrap().feature_count(&key);
        gained.extend((previous..count).map(|_| level));
        previous = count;
    }
    gained
}

/// A possible Build of `make`, then `prepare`d. The race `make` needs is the first, over the races and alignments in
/// turn (the default first), that makes it possible as it is (the Reformed Fiend: a good tiefling); the prepared
/// Build then tries every alignment with it (the Razmiri Infiltrator is neutral or evil). `None` when none is
/// possible (an option the Build's class does not offer).
fn playable(make: impl Fn() -> Build, prepare: impl Fn(Build) -> Build) -> Option<Build> {
    let engine = engine();
    let races = std::iter::once(None).chain(DATA.races.iter().map(|race| Some(race.key.clone())));
    let variant = |race: &Option<rules::RaceKey>, alignment: Option<rules::Alignment>| {
        let mut build = make();
        build.start.race = race.clone().or(build.start.race);
        if let Some(alignment) = alignment {
            build.start.alignment = Some(alignment);
            build.start.deity = None;
        }
        faithful(build)
    };
    let alignments = || std::iter::once(None).chain(ALIGNMENTS.map(Some));
    let race = races
        .into_iter()
        .find(|race| alignments().any(|alignment| engine.is_possible(&variant(race, alignment))))
        .expect("a race and an alignment make the Build possible");
    alignments().find_map(|alignment| {
        let prepared = prepare(variant(&race, alignment));
        engine.is_possible(&prepared).then_some(prepared)
    })
}

/// `build` with `option` taken at level 1 in the archetype's own selection (`<archetype>/<own>`) when it has one, else
/// in the class's (`base`): the Seeker's, Empyreal Sorcerer's and Wildland Shaman's are their own. The Build is left
/// unchanged when it has no such slot.
fn with_own_or_base(
    build: Build,
    archetype: Option<&str>,
    own: &str,
    base: &str,
    option: &str,
) -> Build {
    let own = archetype
        .map(|archetype| format!("{archetype}/{own}"))
        .filter(|key| {
            DATA.selections()
                .any(|selection| selection.key.as_str() == key)
        });
    let key = own.as_deref().unwrap_or(base);
    let slot = engine()
        .evaluate(&build)
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .find(|slot| {
            slot.kind == selection_kind(key)
                && matches!(slot.id, rules::SlotId::Selection { level: 1, .. })
        })
        .map(|slot| slot.id.clone());
    match slot {
        Some(slot) => with_pick(build, slot, option),
        None => build,
    }
}

/// Whether the archetype's level 1 (the class's own when the archetype has no row there) has a selection that offers
/// `option`, own or shared.
fn offers(class_key: &str, archetype: Option<&str>, option: &str) -> bool {
    let class = class(class_key);
    let own = archetype.and_then(|key| {
        class
            .archetypes
            .iter()
            .find(|archetype| archetype.key.as_str() == key)
            .and_then(|archetype| archetype.levels.iter().find(|row| row.level == 1))
            .map(|row| &row.selections)
    });
    let selections = own.unwrap_or_else(|| {
        &class
            .levels
            .iter()
            .find(|row| row.level == 1)
            .unwrap()
            .selections
    });
    selections.iter().any(|key| {
        DATA.selections()
            .find(|selection| selection.key == *key)
            .is_some_and(|selection| {
                selection
                    .offered()
                    .iter()
                    .any(|offered| offered.as_str() == option)
            })
    })
}

/// Asserts what each archetype of `class_key` (and the class itself) gains `feature` at, at class level 20: the class
/// and every archetype not named in `exceptions` gain it at `default`, each exception at its own levels. Every
/// archetype is built as a possible Build.
///
/// `option` is the option that carries the feature (`""` for a feature that is the archetype's own). An archetype that
/// `takes` it must be offered it and hold it on its sheet; an exception `None` names an archetype that is not offered
/// it (checked from the selections' options, not from a failed Build) and holds the feature never.
fn assert_gains_of_option(
    class_key: &str,
    feature: &str,
    option: &str,
    takes: &dyn Fn(Option<&str>) -> bool,
    prepare: &dyn Fn(Option<&str>, Build) -> Build,
    default: &[u8],
    exceptions: &[(&str, Option<&[u8]>)],
) {
    let archetypes = class(class_key)
        .archetypes
        .iter()
        .map(|a| Some(a.key.as_str()));
    for (key, _) in exceptions {
        assert!(
            class(class_key)
                .archetypes
                .iter()
                .any(|a| a.key.as_str() == *key),
            "{class_key}: {key}"
        );
    }
    for archetype in std::iter::once(None).chain(archetypes) {
        let make = || match archetype {
            Some(key) => archetype_build(class_key, key, 20),
            None => build(class_key, 20),
        };
        let label = format!("{class_key} {archetype:?}: {feature}");
        let expected = exceptions
            .iter()
            .find(|(key, _)| Some(*key) == archetype)
            .map_or(Some(default), |(_, levels)| *levels);
        let Some(expected) = expected else {
            assert!(!option.is_empty(), "{label}: no option to be offered");
            assert!(
                !offers(class_key, archetype, option),
                "{label}: offered {option}"
            );
            let build = playable(make, |build| build).unwrap();
            assert_eq!(gains(&build, feature), Vec::<u8>::new(), "{label}");
            continue;
        };
        let taking = !option.is_empty() && takes(archetype);
        if taking {
            assert!(
                offers(class_key, archetype, option),
                "{label}: not offered {option}"
            );
        }
        let build = playable(make, |build| prepare(archetype, build))
            .unwrap_or_else(|| panic!("{label}: no possible Build"));
        if taking {
            let held = at(&build, 20).option_count(&rules::SelectionOptionKey::new(option));
            assert_eq!(held, 1, "{label}: holds {option}");
        }
        assert_eq!(gains(&build, feature), expected, "{label}");
    }
}

/// [`assert_gains_of_option`] for a feature that is the archetype's own: no option to take.
fn assert_gains(
    class_key: &str,
    feature: &str,
    prepare: &dyn Fn(Option<&str>, Build) -> Build,
    default: &[u8],
    exceptions: &[(&str, Option<&[u8]>)],
) {
    assert_gains_of_option(
        class_key,
        feature,
        "",
        &|_| true,
        prepare,
        default,
        exceptions,
    );
}

/// The sorcerer's bloodlines whose power steps at bloodline level 9 (ticket 177), with the guide's sentence for each
/// in the ticket's Answer, and what every archetype of the sorcerer gains it at: the keeping archetypes at the
/// bloodline's own levels (a Crossblooded's first bloodline, the Geomancer, the Overwhelming Mage, the Nine-Tailed
/// Heir), the Seeker never (it skips the rows of 3, 9, 13, 15 and 17), the Empyreal Sorcerer only for its fixed
/// Celestial bloodline, and the Sage and the Sylvan not at all (their bloodlines are fixed: nothing to pick, so
/// nothing held).
#[test]
fn every_sorcerer_archetype_holds_the_steps_of_the_bloodline_resistances() {
    let steps: [(&str, &str, &[u8]); 8] = [
        (
            "sorcerer/bloodline/abyssal",
            "sorcerer/bloodline/abyssal/demon-resistances",
            &[3, 9],
        ),
        (
            "sorcerer/bloodline/celestial",
            "sorcerer/bloodline/celestial/celestial-resistances",
            &[3, 9],
        ),
        (
            "sorcerer/bloodline/infernal",
            "sorcerer/bloodline/infernal/infernal-resistances",
            &[3, 9],
        ),
        (
            "sorcerer/bloodline/undead",
            "sorcerer/bloodline/undead/deaths-gift",
            &[3, 9],
        ),
        (
            "sorcerer/bloodline/serpentine",
            "sorcerer/bloodline/serpentine/snakeskin",
            &[9, 13, 17],
        ),
        (
            "sorcerer/bloodline/elemental-air",
            "sorcerer/bloodline/elemental/elemental-resistances",
            &[3, 9],
        ),
        (
            "sorcerer/bloodline/elemental-earth",
            "sorcerer/bloodline/elemental/elemental-resistances",
            &[3, 9],
        ),
        (
            "sorcerer/bloodline/elemental-fire",
            "sorcerer/bloodline/elemental/elemental-resistances",
            &[3, 9],
        ),
    ];
    for (option, feature, levels) in steps {
        let celestial = option == "sorcerer/bloodline/celestial";
        let empyreal: Option<&[u8]> = celestial.then_some(levels);
        assert_gains_of_option(
            "sorcerer",
            feature,
            option,
            &|_| true,
            &|archetype, build| {
                with_own_or_base(build, archetype, "bloodline", "sorcerer/bloodline", option)
            },
            levels,
            &[
                // The Seeker is offered every bloodline and holds the one it picks, but none of its steps.
                ("sorcerer/seeker-sorcerer", Some(&[])),
                // The Empyreal Sorcerer's fixed bloodline is Celestial; the Sage's Arcane and the Sylvan's Fey.
                ("sorcerer/empyreal-sorcerer", empyreal),
                ("sorcerer/sage-sorcerer", None),
                ("sorcerer/sylvan-sorcerer", None),
            ],
        );
    }
    // The fourth Elemental element, and every one of the four through the class itself.
    assert_gains_of_option(
        "sorcerer",
        "sorcerer/bloodline/elemental/elemental-resistances",
        "sorcerer/bloodline/elemental-water",
        &|_| true,
        &|archetype, build| {
            with_own_or_base(
                build,
                archetype,
                "bloodline",
                "sorcerer/bloodline",
                "sorcerer/bloodline/elemental-water",
            )
        },
        &[3, 9],
        &[
            ("sorcerer/seeker-sorcerer", Some(&[])),
            ("sorcerer/empyreal-sorcerer", None),
            ("sorcerer/sage-sorcerer", None),
            ("sorcerer/sylvan-sorcerer", None),
        ],
    );
}

#[test]
fn the_bloodragers_infernal_resistance_and_abyssal_bloodrage_step() {
    // Infernal Resistance 4 and 8; Abyssal Bloodrage 12, 16 and 20. The Hag-Riven and the Mixed Blood Rager are not
    // offered either bloodline; the Primalist's rage-power choice is the known open question (ticket 177's Answer),
    // and a Primalist taking the bloodline's ability at every level holds them all.
    for (option, feature, levels) in [
        (
            "bloodrager/bloodline/infernal",
            "bloodrager/bloodline/infernal/infernal-resistance",
            &[4u8, 8][..],
        ),
        (
            "bloodrager/bloodline/abyssal",
            "bloodrager/bloodline/abyssal/abyssal-bloodrage",
            &[12, 16, 20][..],
        ),
    ] {
        assert_gains_of_option(
            "bloodrager",
            feature,
            option,
            &|_| true,
            &|_, build| with_option(build, 1, 0, option),
            levels,
            &[
                ("bloodrager/hag-riven", None),
                ("bloodrager/mixed-blood-rager", None),
            ],
        );
    }
}

#[test]
fn the_instinctual_warriors_cunning_elusion_steps_every_four_levels() {
    assert_gains(
        "barbarian",
        "barbarian/instinctual-warrior/cunning-elusion",
        &|_, build| build,
        &[],
        &[(
            "barbarian/instinctual-warrior",
            Some(&[2, 4, 8, 12, 16, 20]),
        )],
    );
    // The Flesheater's Crush and Tear: the bite at 1, 5, 11 and 17.
    assert_gains(
        "barbarian",
        "barbarian/flesheater/crush-and-tear",
        &|_, build| build,
        &[],
        &[("barbarian/flesheater", Some(&[1, 5, 11, 17]))],
    );
}

#[test]
fn the_dragonheir_scions_defences_and_fearful_might_step() {
    for colour in COLOURS {
        let option = format!("sorcerer/bloodline/draconic-{colour}");
        let prepare = |archetype: Option<&str>, build: Build| match archetype {
            Some("fighter/dragonheir-scion") => with_option(build, 1, 0, &option),
            _ => build,
        };
        assert_gains_of_option(
            "fighter",
            "fighter/dragonheir-scion/draconic-defences",
            &option,
            &|archetype| archetype == Some("fighter/dragonheir-scion"),
            &prepare,
            &[],
            &[("fighter/dragonheir-scion", Some(&[3, 7, 13]))],
        );
        assert_gains_of_option(
            "fighter",
            "fighter/dragonheir-scion/fearful-might",
            &option,
            &|archetype| archetype == Some("fighter/dragonheir-scion"),
            &prepare,
            &[],
            &[("fighter/dragonheir-scion", Some(&[2, 6, 10, 14, 18]))],
        );
    }
}

#[test]
fn the_alchemists_reanimator_and_metamorph_step() {
    assert_gains(
        "alchemist",
        "alchemist/reanimator/corpse-studying",
        &|_, build| build,
        &[],
        &[("alchemist/reanimator", Some(&[1, 5, 11, 17]))],
    );
    assert_gains(
        "alchemist",
        "alchemist/metamorph/adaptive-physiology",
        &|_, build| build,
        &[],
        &[("alchemist/metamorph", Some(&[3, 6, 18]))],
    );
}

#[test]
fn the_fake_celebritys_inhibit_courage_steps() {
    const WAYS: &str = "arcanist/magic-deceiver/ways-of-the-deceiver";
    // Each of the Magic Deceiver's six ways: only the Fake Celebrity holds Inhibit Courage.
    for way in [
        "fake-celebrity",
        "living-deity",
        "magic-inventor",
        "razmiri-infiltrator",
        "time-manipulator",
        "way-of-the-charlatan",
    ] {
        let option = format!("{WAYS}/{way}");
        let expected: &[u8] = if way == "fake-celebrity" {
            &[1, 5, 11, 17]
        } else {
            &[]
        };
        assert_gains_of_option(
            "arcanist",
            "arcanist/magic-deceiver/ways-of-the-deceiver/fake-celebrity/inhibit-courage",
            &option,
            &|archetype| archetype == Some("arcanist/magic-deceiver"),
            &|archetype, build| match archetype {
                Some("arcanist/magic-deceiver") => {
                    let slot = slot_of(&build, 1, WAYS);
                    with_pick(build, slot, &option)
                }
                _ => build,
            },
            &[],
            &[("arcanist/magic-deceiver", Some(expected))],
        );
    }
}

#[test]
fn the_defender_of_the_true_worlds_fey_stalker_the_overwhelming_souls_power_and_the_demon_dancers_maw_step()
 {
    assert_gains(
        "druid",
        "druid/defender-of-the-true-world/fey-stalker",
        &|_, build| build,
        &[],
        &[("druid/defender-of-the-true-world", Some(&[3, 8, 13, 18]))],
    );
    assert_gains(
        "kineticist",
        "kineticist/overwhelming-soul/overwhelming-power",
        &|_, build| build,
        &[],
        &[("kineticist/overwhelming-soul", Some(&[3, 6, 9, 12, 15, 18]))],
    );
    assert_gains(
        "skald",
        "skald/demon-dancer/fiendish-maw",
        &|_, build| build,
        &[],
        &[("skald/demon-dancer", Some(&[3, 7, 12, 17]))],
    );
}

#[test]
fn the_shifters_aspects_and_defences_step() {
    assert_gains(
        "shifter",
        "shifter/weretouched/lycanthropic-aspect",
        &|_, build| build,
        &[],
        &[("shifter/weretouched", Some(&[1, 8, 16]))],
    );
    assert_gains(
        "shifter",
        "shifter/rageshaper/invulnerable-defenses",
        &|_, build| build,
        &[],
        &[("shifter/rageshaper", Some(&[2, 10, 15, 20]))],
    );
    assert_gains(
        "shifter",
        "shifter/fiendflesh-shifter/fiendish-aspect",
        &|_, build| build,
        &[],
        &[("shifter/fiendflesh-shifter", Some(&[1, 5, 10, 15, 20]))],
    );
}

#[test]
fn the_battle_spirit_steps_at_8_and_16_for_every_shaman_that_takes_it() {
    assert_gains_of_option(
        "shaman",
        "shaman/spirit/battle/battle-spirit",
        "shaman/spirit/battle",
        &|_| true,
        &|archetype, build| {
            with_own_or_base(
                build,
                archetype,
                "spirit",
                "shaman/spirit",
                "shaman/spirit/battle",
            )
        },
        &[1, 8, 16],
        &[
            // A fixed Pestilence spirit; wandering spirits only: the Battle spirit is not offered to them.
            ("shaman/prophet-of-pestilence", None),
            ("shaman/unsworn-shaman", None),
        ],
    );
}

/// The assassin's Cold Minded (2, 5 and 8) and the Winter Witch's Cold Specialization (1, 5 and 10), on the prestige
/// class's own levels: the Assassin after six rogue levels with five ranks of Stealth, the Winter Witch after six
/// shaman levels with the Frost spirit.
#[test]
fn the_assassins_cold_minded_and_the_winter_witchs_cold_specialization_step() {
    let mut assassin = build("rogue", 14);
    for level in &mut assassin.levels[..5] {
        level.skill_ranks.insert(rules::SkillKey::new("stealth"), 1);
    }
    for level in &mut assassin.levels[6..] {
        level.class = rules::ClassKey::new("assassin");
    }
    let assassin = realigned(assassin, rules::Alignment::NeutralEvil);
    assert!(engine().is_possible(&assassin));
    // Assassin levels 1 to 8 are Build levels 7 to 14.
    assert_eq!(
        gains(&assassin, "assassin/cold-minded"),
        [8, 11, 14],
        "Assassin 2, 5 and 8"
    );
    let mut witch = build("shaman", 16);
    for level in &mut witch.levels[..6] {
        level
            .skill_ranks
            .insert(rules::SkillKey::new("knowledge-arcana"), 1);
        level
            .skill_ranks
            .insert(rules::SkillKey::new("lore-nature"), 1);
    }
    for level in &mut witch.levels[6..] {
        level.class = rules::ClassKey::new("winter-witch");
    }
    let witch = with_option(witch, 1, 0, "shaman/spirit/frost");
    assert!(engine().is_possible(&witch));
    // Winter Witch levels 1 to 10 are Build levels 7 to 16.
    assert_eq!(
        gains(&witch, "winter-witch/cold-specialization"),
        [7, 11, 16],
        "Winter Witch 1, 5 and 10"
    );
}

/// The selections that share an option whose feature steps (ticket 177's sweep, as 179's): the Eldritch Scion, the
/// Mythic Second Bloodline and Second Bloodrager Bloodline and the Second Spirit; the Reformed Fiend, the Wildland
/// Shaman, the Seeker and the Empyreal Sorcerer are in the class loops above. The Dragon Disciple takes only a
/// Draconic bloodline (its prerequisite refuses any other), so it shares none of these features.
#[test]
fn the_selections_sharing_a_stepped_option_hold_the_steps_its_rows_give() {
    // The Eldritch Scion takes the bloodline by magus level.
    for (option, feature, levels) in [
        (
            "sorcerer/bloodline/abyssal",
            "sorcerer/bloodline/abyssal/demon-resistances",
            &[3u8, 9][..],
        ),
        (
            "sorcerer/bloodline/undead",
            "sorcerer/bloodline/undead/deaths-gift",
            &[3, 9][..],
        ),
        (
            "sorcerer/bloodline/serpentine",
            "sorcerer/bloodline/serpentine/snakeskin",
            &[9, 13, 17][..],
        ),
        (
            "sorcerer/bloodline/elemental-fire",
            "sorcerer/bloodline/elemental/elemental-resistances",
            &[3, 9][..],
        ),
    ] {
        let scion = with_option(
            archetype_build("magus", "magus/eldritch-scion", 20),
            1,
            0,
            option,
        );
        assert!(engine().is_possible(&scion), "{option}");
        assert_eq!(gains(&scion, feature), levels, "Eldritch Scion {option}");
    }
    // The Mythic Second Bloodline, by sorcerer levels, taken at MR 1 (level 6): the first grant at once, the step at
    // 9 (Snakeskin: 9, 13, 17, so its first grant is at 9).
    let second = |first: &str, option: &str| {
        let mut build = with_option(build("sorcerer", 20), 1, 0, first);
        build.picks.insert(
            rules::SlotId::Mythic { rank: 1 },
            rules::Pick::mythic_ability(rules::MythicAbilityKey::new("second-bloodline")),
        );
        build.picks.insert(
            rules::SlotId::MythicSelection { rank: 1, index: 0 },
            pick(option),
        );
        build
    };
    let build_ = second("sorcerer/bloodline/abyssal", "sorcerer/bloodline/celestial");
    assert!(engine().is_possible(&build_));
    assert_eq!(
        gains(
            &build_,
            "sorcerer/bloodline/celestial/celestial-resistances"
        ),
        [6, 9]
    );
    // The Second Bloodrager Bloodline: Infernal Resistance by bloodrager levels, 6 and then 8.
    let mut bloodrager = with_option(
        build("bloodrager", 20),
        1,
        0,
        "bloodrager/bloodline/abyssal",
    );
    bloodrager.picks.insert(
        rules::SlotId::Mythic { rank: 1 },
        rules::Pick::mythic_ability(rules::MythicAbilityKey::new("second-bloodrager-bloodline")),
    );
    bloodrager.picks.insert(
        rules::SlotId::MythicSelection { rank: 1, index: 0 },
        pick("bloodrager/bloodline/infernal"),
    );
    assert!(engine().is_possible(&bloodrager));
    assert_eq!(
        gains(
            &bloodrager,
            "bloodrager/bloodline/infernal/infernal-resistance"
        ),
        [6, 8]
    );
    // The Second Spirit: a Battle spirit taken at MR 1 (level 6) after the Frost spirit holds Battle Spirit at 6, 8
    // and 16.
    let mut shaman = with_option(build("shaman", 20), 1, 0, "shaman/spirit/frost");
    shaman.picks.insert(
        rules::SlotId::Mythic { rank: 1 },
        rules::Pick::mythic_ability(rules::MythicAbilityKey::new("second-spirit")),
    );
    shaman.picks.insert(
        rules::SlotId::MythicSelection { rank: 1, index: 0 },
        pick("shaman/spirit/battle"),
    );
    assert!(engine().is_possible(&shaman));
    assert_eq!(
        gains(&shaman, "shaman/spirit/battle/battle-spirit"),
        [6, 8, 16]
    );
}

/// The Primalist (ticket 177): "Two Rage Powers" replaces the bloodline's whole row at 4, 8, 12, 16 or 20, so the
/// bloodrager's steps on those rows go with it. The guide says nothing of what the game does with a step after the
/// first grant is traded away (the open question in the ticket's Answer), so this records what the data does: a
/// Primalist taking the rage powers at 4 and the bloodline at 8 gains Infernal Resistance at 8 (a step of a feature it
/// traded away), and one taking them at 8 loses the step.
#[test]
fn a_primalists_rage_powers_replace_the_row_and_its_step_with_it() {
    let primalist = |option: &str, rage_powers_at: &[u8]| {
        let mut build = with_option(
            archetype_build("bloodrager", "bloodrager/primalist", 20),
            1,
            0,
            option,
        );
        for &level in rage_powers_at {
            let slot = slot_of(&build, level, "bloodrager/primalist/primal-choices");
            build = with_pick(
                build,
                slot,
                "bloodrager/primalist/primal-choices/rage-powers",
            );
        }
        assert!(engine().is_possible(&build), "{rage_powers_at:?}");
        build
    };
    const INFERNAL: &str = "bloodrager/bloodline/infernal";
    const RESISTANCE: &str = "bloodrager/bloodline/infernal/infernal-resistance";
    assert_eq!(gains(&primalist(INFERNAL, &[]), RESISTANCE), [4, 8]);
    assert_eq!(gains(&primalist(INFERNAL, &[4]), RESISTANCE), [8]);
    assert_eq!(gains(&primalist(INFERNAL, &[8]), RESISTANCE), [4]);
    assert_eq!(
        gains(&primalist(INFERNAL, &[4, 8]), RESISTANCE),
        Vec::<u8>::new()
    );
    // The Abyssal bloodline's Abyssal Bloodrage (first at 12, steps at 16 and 20) leaks the same way: trading the
    // row at 12 never grants it, yet the steps still come; trading a later row drops that step.
    const ABYSSAL: &str = "bloodrager/bloodline/abyssal";
    const BLOODRAGE: &str = "bloodrager/bloodline/abyssal/abyssal-bloodrage";
    assert_eq!(gains(&primalist(ABYSSAL, &[]), BLOODRAGE), [12, 16, 20]);
    assert_eq!(gains(&primalist(ABYSSAL, &[12]), BLOODRAGE), [16, 20]);
    assert_eq!(gains(&primalist(ABYSSAL, &[16]), BLOODRAGE), [12, 20]);
    assert_eq!(gains(&primalist(ABYSSAL, &[20]), BLOODRAGE), [12, 16]);
    assert_eq!(
        at(&primalist(ABYSSAL, &[12]), 20).feature_count(&FeatureKey::new(BLOODRAGE)),
        2
    );
}
