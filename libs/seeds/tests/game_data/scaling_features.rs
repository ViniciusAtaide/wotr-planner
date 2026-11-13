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
pub(crate) fn playable(
    make: impl Fn() -> Build,
    prepare: impl Fn(Build) -> Build,
) -> Option<Build> {
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

/// The levels at which the sheet has a bloodline feat slot (a Feat slot listing the held bloodline's options).
fn bloodline_feat_levels(build: &Build) -> Vec<u8> {
    engine()
        .evaluate(build)
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .filter(|slot| {
            matches!(
                &slot.kind,
                rules::SlotKind::Feat(rules::FeatSlot::OptionList { selection, .. })
                    if selection.as_str() == "bloodrager/bloodline"
            )
        })
        .map(|slot| match slot.id {
            rules::SlotId::Feat { level, .. } => level,
            ref other => panic!("a bloodline feat slot at {other:?}"),
        })
        .collect()
}

/// The Herald of the Horn's Horn Call (7, 13, 19) keeps the skald's Raging Song (Swift Action) at 13 and Damage
/// Reduction at 19 on the rows that now carry it (ticket 182).
#[test]
fn the_heralds_horn_call_steps_and_keeps_what_the_skalds_rows_give() {
    assert_gains(
        "skald",
        "skald/herald-of-the-horn/horn-call",
        &|_, build| build,
        &[],
        &[("skald/herald-of-the-horn", Some(&[7, 13, 19]))],
    );
    let skald = build("skald", 20);
    let herald = archetype_build("skald", "skald/herald-of-the-horn", 20);
    assert!(engine().is_possible(&herald));
    // The Herald's table keeps both, as the class's does: Damage Reduction at 9, 14 and 19, the swift-action song
    // at 13.
    for (feature, expected) in [
        ("skald/damage-reduction", &[9u8, 14, 19][..]),
        ("skald/raging-song-swift-action", &[13][..]),
        ("skald/raging-song-move-action", &[7][..]),
    ] {
        assert_eq!(gains(&skald, feature), expected, "{feature}");
        assert_eq!(gains(&herald, feature), expected, "{feature}");
    }
}

/// The Reformed Fiend's Hatred Against Evil (5, 10, 15, 20) and its Damage Reduction (7, 10, 13, 16, 19, already
/// stepped), with the bloodline feat slot at 15 and Mighty Bloodrage at 20 kept on the new rows.
#[test]
fn the_reformed_fiends_hatred_against_evil_steps_and_its_rows_keep_what_the_base_gives() {
    for bloodline in [
        "bloodrager/bloodline/abyssal",
        "bloodrager/bloodline/infernal",
    ] {
        let build = playable(
            || archetype_build("bloodrager", "bloodrager/reformed-fiend", 20),
            |build| with_option(build, 1, 0, bloodline),
        )
        .unwrap();
        assert_eq!(
            gains(&build, "bloodrager/reformed-fiend/hatred-against-evil"),
            [5, 10, 15, 20],
            "{bloodline}"
        );
        assert_eq!(
            gains(
                &build,
                "bloodrager/reformed-fiend/damage-reduction-reformed-fiend"
            ),
            [7, 10, 13, 16, 19],
            "{bloodline}"
        );
        assert_eq!(
            gains(&build, "bloodrager/mighty-bloodrage"),
            [20],
            "{bloodline}"
        );
        // The base row's bloodline feat slot at 15 stays, beside the ones at 6, 9, 12 and 18: those of the class.
        assert_eq!(
            bloodline_feat_levels(&build),
            [6, 9, 12, 15, 18],
            "{bloodline}"
        );
    }
    // Only the Reformed Fiend has it.
    assert_gains(
        "bloodrager",
        "bloodrager/reformed-fiend/hatred-against-evil",
        &|archetype, build| match archetype {
            Some("bloodrager/reformed-fiend") => {
                with_option(build, 1, 0, "bloodrager/bloodline/abyssal")
            }
            _ => build,
        },
        &[],
        &[("bloodrager/reformed-fiend", Some(&[5, 10, 15, 20]))],
    );
}

/// The Flesheater's Devour Essence (2, 8, 13, 18) and Power from Flesh (2, 9, 18): the row at 18 keeps the rage power
/// the Flesheater's table keeps there (4, 10, 12, 16, 18 and 20; none at 8).
#[test]
fn the_flesheaters_devour_essence_and_power_from_flesh_step() {
    assert_gains(
        "barbarian",
        "barbarian/flesheater/devour-essence",
        &|_, build| build,
        &[],
        &[("barbarian/flesheater", Some(&[2, 8, 13, 18]))],
    );
    assert_gains(
        "barbarian",
        "barbarian/flesheater/power-from-flesh",
        &|_, build| build,
        &[],
        &[("barbarian/flesheater", Some(&[2, 9, 18]))],
    );
    let flesheater = playable(
        || archetype_build("barbarian", "barbarian/flesheater", 20),
        |build| build,
    )
    .unwrap();
    assert_eq!(
        levels_with_slot_of(&flesheater, "barbarian/rage-power"),
        [4, 10, 12, 16, 18, 20]
    );
}

/// The Fiendflesh Shifter's Fiendish Resilience is one feature stepped at 2, 4, 8, 12, 16 and 20: the guide gives
/// it as one ability with two series (natural armour at 2, 4, 12 and 20; the resistance at 2, 8 and 16), and each
/// step improves one of them. Nothing reads its count.
#[test]
fn the_fiendflesh_shifters_fiendish_resilience_steps_at_both_series() {
    assert_gains(
        "shifter",
        "shifter/fiendflesh-shifter/fiendish-resilience",
        &|_, build| build,
        &[],
        &[("shifter/fiendflesh-shifter", Some(&[2, 4, 8, 12, 16, 20]))],
    );
}

/// The Fake Celebrity's Inhibit Greatness (12, 17) beside Inhibit Courage's.
#[test]
fn the_fake_celebritys_inhibit_greatness_steps_at_17() {
    const WAYS: &str = "arcanist/magic-deceiver/ways-of-the-deceiver";
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
            &[12, 17]
        } else {
            &[]
        };
        assert_gains_of_option(
            "arcanist",
            "arcanist/magic-deceiver/ways-of-the-deceiver/fake-celebrity/inhibit-greatness",
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

/// The Overwhelming Soul's Mental Prowess: its use at 6 and one more at 10, 14 and 18 (the feature itself is
/// granted at 1 in the class's table, so the count at 18 is 4: the first use and three more). The rows at 10, 14 and
/// 18 keep the wild talent.
#[test]
fn the_overwhelming_souls_mental_prowess_steps_at_10_14_and_18() {
    assert_gains(
        "kineticist",
        "kineticist/overwhelming-soul/mental-prowess",
        &|_, build| build,
        &[],
        &[("kineticist/overwhelming-soul", Some(&[1, 10, 14, 18]))],
    );
    let soul = archetype_build("kineticist", "kineticist/overwhelming-soul", 20);
    assert!(engine().is_possible(&soul));
    let base = build("kineticist", 20);
    assert_eq!(
        levels_with_slot_of(&soul, "kineticist/wild-talents"),
        levels_with_slot_of(&base, "kineticist/wild-talents")
    );
}

/// Ticket 182's sweep: each feature an archetype owns, with the levels the guide's sentence gives it at (the ticket's
/// Answer quotes each). `assert_gains` builds the class and every archetype of it: the owner gains the feature at
/// exactly these levels (a level once for each rise of its count) and no other archetype or the class gains it.
#[test]
fn the_barbarian_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "barbarian/armored-hulk/resilience-of-steel",
            &[3, 6, 9, 12, 15, 18][..],
        ),
        (
            "barbarian/invulnerable-rager/extreme-endurance",
            &[3, 6, 9, 12, 15, 18][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "barbarian",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_bard_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "bard/archaeologist/archaeologists-luck",
            &[1, 5, 11, 17][..],
        ),
        (
            "bard/archaeologist/danger-sense",
            &[3, 6, 9, 12, 15, 18][..],
        ),
        ("bard/chelaxian-diva/deadly-vibrato", &[6, 14, 20][..]),
        (
            "bard/chelaxian-diva/devastating-aria",
            &[2, 7, 11, 15, 19][..],
        ),
        ("bard/dirge-bard/haunting-refrain", &[5, 10, 15, 20][..]),
        ("bard/flame-dancer/fire-dance", &[3, 6, 11][..]),
        ("bard/thundercaller/thunder-call", &[3, 7, 11, 15, 19][..]),
        (
            "bard/tranquil-whisperer/inspire-tranquility",
            &[1, 5, 11, 17][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "bard",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_bloodrager_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("bloodrager/greenrager/furious-summoning", &[9, 11, 20][..]),
        (
            "bloodrager/hag-riven/scarred-hide",
            &[7, 10, 13, 16, 19][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "bloodrager",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_cavalier_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "cavalier/disciple-of-the-pike/bigger-they-are",
            &[1, 6, 12][..],
        ),
        (
            "cavalier/disciple-of-the-pike/weapon-training-spears-polearms",
            &[5, 9, 13, 17][..],
        ),
        (
            "cavalier/fearsome-leader/fearmonger",
            &[3, 6, 9, 12, 15, 18][..],
        ),
        (
            "cavalier/standard-bearer/banner-of-solace",
            &[11, 15, 19][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "cavalier",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_druid_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("druid/elemental-rampager/claws", &[1, 9, 15][..]),
        ("druid/elemental-rampager/rampage", &[1, 6, 11, 16][..]),
        ("druid/primal-druid/primal-size", &[1, 8, 16][..]),
        ("druid/winter-child/arctic-native", &[4, 8, 12, 16, 20][..]),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "druid",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_fighter_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "fighter/defender-of-the-hearth/shield-block",
            &[1, 5, 10, 15, 20][..],
        ),
        (
            "fighter/tower-shield-specialist/burst-barrier",
            &[2, 6, 10, 14, 18][..],
        ),
        (
            "fighter/two-handed-fighter/strong-grip",
            &[2, 6, 10, 14, 18][..],
        ),
        (
            "fighter/two-handed-fighter/two-handed-weapon-training",
            &[5, 9, 13, 17][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "fighter",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_hunter_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("hunter/colluding-scoundrel/backstabber", &[8, 15][..]),
        ("hunter/colluding-scoundrel/scapegoat", &[1, 8, 15][..]),
        ("hunter/forester/tactician", &[3, 7, 12, 17][..]),
        (
            "hunter/wandering-marksman/planned-distraction",
            &[3, 12, 18][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "hunter",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_inquisitor_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "inquisitor/faith-hunter/sworn-enemy-smite",
            &[6, 10, 14, 18][..],
        ),
        (
            "inquisitor/living-grimoire/blessed-script",
            &[5, 10, 16][..],
        ),
        (
            "inquisitor/tactical-leader/swift-tactician",
            &[3, 6, 9, 12, 15, 18][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "inquisitor",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_kineticist_archetypes_step_the_features_the_guide_gives_steps_to() {
    let feature = "kineticist/kinetic-knight/knights-resolve";
    let levels = &[3, 5, 7, 9, 11, 13, 15, 17, 19][..];
    assert_gains(
        "kineticist",
        feature,
        &|_, build| build,
        &[],
        &[(feature.rsplit_once('/').unwrap().0, Some(levels))],
    );
}

#[test]
fn the_magus_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("magus/bladebound/black-blade", &[1, 5, 9, 13, 17][..]),
        (
            "magus/bladebound/sharpening-the-black-blade",
            &[3, 6, 9, 12, 15, 18][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "magus",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_monk_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("monk/drunken-master/drunken-courage", &[7, 11][..]),
        (
            "monk/drunken-master/drunken-ki",
            &[3, 5, 7, 8, 9, 11, 13, 15, 17, 19][..],
        ),
        ("monk/drunken-master/drunken-resilience", &[9, 15, 19][..]),
        ("monk/sensei/mystic-powers", &[6, 10][..]),
        ("monk/sensei/mystic-wisdom", &[10, 14][..]),
        ("monk/sohei/ki-weapon", &[4, 8, 12, 16, 20][..]),
        ("monk/student-of-stone/bones-of-stone", &[7, 10, 15][..]),
        ("monk/student-of-stone/soul-of-stone", &[12, 16][..]),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "monk",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_oracle_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "oracle/enlightened-philosopher/mental-acuity",
            &[7, 10, 13, 16, 19][..],
        ),
        ("oracle/lone-strider/fade-from-memory", &[7, 14][..]),
        ("oracle/lone-strider/recluses-stride", &[1, 5, 10][..]),
        ("oracle/wind-whisperer/haste-of-the-wind", &[7, 13, 19][..]),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "oracle",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_paladin_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "paladin/hospitaler/smite-evil-hospitaler",
            &[1, 7, 13, 19][..],
        ),
        ("paladin/stonelord/stoneblood", &[3, 9, 15][..]),
        ("paladin/tortured-crusader/second-chance", &[2, 8, 15][..]),
        (
            "paladin/warrior-of-the-holy-light/power-of-faith",
            &[4, 8, 12, 16, 20][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "paladin",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_ranger_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("ranger/flamewarden/touch-of-flame", &[4, 12][..]),
        ("ranger/stormwalker/wind-treader", &[4, 12][..]),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "ranger",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_rogue_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("rogue/knife-master/blade-sense", &[3, 6, 9, 12, 15, 18][..]),
        (
            "rogue/sylvan-trickster/fey-resistance",
            &[8, 11, 14, 17, 20][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "rogue",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_shaman_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("shaman/shadow-shaman/shadows", &[4, 9, 14][..]),
        (
            "shaman/spirit-hunter/spirit-weapon-enchantment",
            &[1, 5, 9, 13, 17][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "shaman",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_shifter_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "shifter/child-of-the-manticore/manticore-aspect",
            &[1, 5, 10, 15, 20][..],
        ),
        (
            "shifter/child-of-the-manticore/manticore-shape",
            &[4, 9, 14][..],
        ),
        ("shifter/griffonheart-shifter/griffon-form", &[4, 9, 14][..]),
        ("shifter/wild-effigy/heart-of-earth", &[4, 9, 15][..]),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "shifter",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_skald_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "skald/court-poet/insightful-contemplation",
            &[1, 4, 8, 12, 16, 20][..],
        ),
        ("skald/inciter/caustic-ridicule", &[10, 15][..]),
        ("skald/inciter/prompted-by-backstabbing", &[8, 13, 18][..]),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "skald",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_slayer_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("slayer/executioner/assassinate", &[10, 14, 19][..]),
        ("slayer/stygian-slayer/invisibiliy", &[4, 8, 12, 16, 20][..]),
        ("slayer/spawn-slayer/studied-spawn", &[5, 10, 15, 20][..]),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "slayer",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_warpriest_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        (
            "warpriest/champion-of-the-faith/smite",
            &[4, 8, 12, 16, 20][..],
        ),
        (
            "warpriest/disenchanter/mystic-interference",
            &[4, 8, 12, 16, 20][..],
        ),
        ("warpriest/mantis-zealot/mantis-swarm", &[4, 11, 18][..]),
        ("warpriest/mantis-zealot/fading", &[7, 10, 13, 16, 19][..]),
        (
            "warpriest/proclaimer/cleanser-of-evil",
            &[2, 5, 7, 8, 11, 13, 14, 17, 19, 20][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "warpriest",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_witch_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("witch/hag-of-gyronna/sunder-hope", &[8, 14, 20][..]),
        ("witch/hagbound/hunched-muscle", &[2, 8, 14][..]),
        ("witch/witch-of-the-veil/shrouded-step", &[2, 8, 16][..]),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "witch",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

#[test]
fn the_wizard_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        ("wizard/shadowcaster/umbral-mind", &[10, 12, 14, 16, 18][..]),
        ("wizard/spell-master/focused-spells", &[1, 8, 16][..]),
        (
            "wizard/spell-master/item-bond-specialization",
            &[5, 10, 15, 20][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "wizard",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}

/// The owner's class, the feature, its first level, and the features that carry its steps with their levels.
type GrantedOnce = (
    &'static str,
    &'static str,
    u8,
    &'static [(&'static str, &'static [u8])],
);

/// Features whose steps the data already grants as features of their own, one per step, as the magus's Arcane
/// Weapon: each stays granted once, at its first level, and the step features carry the rises (ticket 182's review).
/// Stepping the feature as well would count every step twice.
#[test]
fn features_whose_steps_are_features_of_their_own_are_granted_once() {
    let cases: [GrantedOnce; 9] = [
        (
            "arcanist",
            "arcanist/phantasmal-mage/arcane-metamagic",
            1,
            &[
                ("arcanist/phantasmal-mage/arcane-metamagic-i", &[5]),
                ("arcanist/phantasmal-mage/arcane-metamagic-ii", &[11]),
                ("arcanist/phantasmal-mage/arcane-metamagic-iii", &[17]),
            ],
        ),
        (
            "bard",
            "bard/beast-tamer/beast-call",
            1,
            &[
                ("bard/beast-tamer/summon-natures-ally-ii", &[3]),
                ("bard/beast-tamer/summon-natures-ally-ix", &[17]),
            ],
        ),
        (
            "bloodrager",
            "bloodrager/hag-riven/claws-of-the-hag",
            1,
            &[
                ("bloodrager/hag-riven/claws-of-the-hag-magic", &[2]),
                ("bloodrager/hag-riven/claws-of-the-hag-1d6", &[5]),
                ("bloodrager/hag-riven/claws-of-the-hag-rend", &[9]),
                ("bloodrager/hag-riven/claws-of-the-hag-critical", &[13]),
                ("bloodrager/hag-riven/claws-of-the-hag-weakness", &[17]),
            ],
        ),
        (
            "druid",
            "druid/elemental-rampager/elemental-rampage",
            4,
            &[
                ("druid/elemental-rampager/elemental-damage", &[4, 11, 20]),
                ("druid/elemental-rampager/elemental-ability", &[11, 20]),
            ],
        ),
        (
            "druid",
            "druid/winter-child/call-a-feast",
            1,
            &[
                ("druid/winter-child/call-a-feast-cover-of-snow", &[10]),
                ("druid/winter-child/call-a-feast-biting-frost", &[15]),
            ],
        ),
        (
            "magus",
            "magus/armored-battlemage/arcane-pool-armored-battlemage",
            1,
            &[
                ("magus/armored-battlemage/arcane-armor-2", &[5]),
                ("magus/armored-battlemage/arcane-armor-5", &[17]),
            ],
        ),
        (
            "magus",
            "magus/spell-dancer/spell-dance",
            1,
            &[
                ("magus/spell-dancer/spell-dance-ii", &[5]),
                ("magus/spell-dancer/spell-dance-iii", &[9]),
                ("magus/spell-dancer/spell-dance-iv", &[13]),
            ],
        ),
        (
            "oracle",
            "oracle/possessed-oracle/powerless-prophecy",
            1,
            &[
                ("oracle/possessed-oracle/curse-2", &[5]),
                ("oracle/possessed-oracle/curse-3", &[10]),
                ("oracle/possessed-oracle/curse-4", &[15]),
            ],
        ),
        ("oracle", "oracle/purifier/celestial-armor", 7, &[]),
    ];
    for (class_key, feature, first, steps) in cases {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            class_key,
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(&[first]))],
        );
        let build = playable(|| archetype_build(class_key, archetype, 20), |build| build).unwrap();
        for (step, levels) in steps {
            assert_eq!(gains(&build, step), *levels, "{step}");
        }
    }
    // The Purifier's step at 11 is the bonus feat its row grants.
    let purifier = playable(
        || archetype_build("oracle", "oracle/purifier", 20),
        |build| build,
    )
    .unwrap();
    assert!(
        !at(&purifier, 10)
            .feats
            .contains_key(&rules::FeatKey::new("armor-proficiency-heavy-armor"))
    );
    assert!(
        at(&purifier, 11)
            .feats
            .contains_key(&rules::FeatKey::new("armor-proficiency-heavy-armor"))
    );
}

const INVULNERABLE_RAGER: &str = "barbarian/invulnerable-rager";
const RAGER_DAMAGE_REDUCTION: &str =
    "barbarian/invulnerable-rager/damage-reduction-invulnerable-rager";

/// Ticket 193: the Invulnerable Rager's table gives Damage Reduction at every even level from 2 to 20. It is a
/// feature of the archetype's own, on its rows at 10 and 16 as well (the archetype has no other row there to
/// take it from the base class's), and no other archetype or the class gains it.
#[test]
fn the_invulnerable_ragers_damage_reduction_is_its_own_at_every_even_level() {
    assert_gains(
        "barbarian",
        RAGER_DAMAGE_REDUCTION,
        &|_, build| build,
        &[],
        &[(
            INVULNERABLE_RAGER,
            Some(&[2, 4, 6, 8, 10, 12, 14, 16, 18, 20]),
        )],
    );
}

/// The barbarian's own Damage Reduction (7, 10, 13, 16, 19) is not the Invulnerable Rager's, and the other
/// archetypes keep what they had: the Armored Hulk and the Beastskin Berserker the class's rows, the Mad Dog its
/// delayed 10 and then the class's, the Flesheater, the Instinctual Warrior and the Pack Rager none.
#[test]
fn the_barbarians_damage_reduction_is_not_the_invulnerable_ragers() {
    assert_gains(
        "barbarian",
        "barbarian/damage-reduction",
        &|_, build| build,
        &[7, 10, 13, 16, 19],
        &[
            (INVULNERABLE_RAGER, Some(&[])),
            ("barbarian/flesheater", Some(&[])),
            ("barbarian/instinctual-warrior", Some(&[])),
            ("barbarian/pack-rager", Some(&[])),
            ("barbarian/mad-dog", Some(&[10, 13, 16, 19])),
        ],
    );
}

#[test]
fn the_two_damage_reductions_are_named_for_their_owners() {
    let engine = engine();
    assert_eq!(
        engine.feature_name_apart(&FeatureKey::new(RAGER_DAMAGE_REDUCTION)),
        "Damage Reduction (Invulnerable Rager)"
    );
    assert_eq!(
        engine.feature_name_apart(&FeatureKey::new("barbarian/damage-reduction")),
        "Damage Reduction (Barbarian)"
    );
}

/// The rows at 10 and 16 carry the rage power the class's rows give there.
#[test]
fn the_invulnerable_rager_has_a_rage_power_at_every_even_level() {
    let build = playable(
        || archetype_build("barbarian", INVULNERABLE_RAGER, 20),
        |build| build,
    )
    .unwrap();
    assert_eq!(
        levels_with_slot_of(&build, "barbarian/rage-power"),
        [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
    );
}

/// Ticket 188: the base classes' stepped features. The barbarian's Danger Sense (the guide: +1 every 3 levels, +6 at
/// 18) is granted at 3, 6, 9, 12, 15 and 18. The Armored Hulk, the Flesheater and the Invulnerable Rager lose it
/// (Resilience of Steel, nothing, Extreme Endurance), so none of them holds a step; every archetype that keeps it
/// gets it on its own rows at the steps (the Beastskin Berserker's 12, the Instinctual Warrior's 6, 12 and 18, the
/// Mad Dog's and the Pack Rager's 6 and 18).
#[test]
fn the_barbarians_danger_sense_steps_every_three_levels() {
    assert_gains(
        "barbarian",
        "barbarian/danger-sense",
        &|_, build| build,
        &[3, 6, 9, 12, 15, 18],
        &[
            ("barbarian/armored-hulk", Some(&[])),
            ("barbarian/flesheater", Some(&[])),
            (INVULNERABLE_RAGER, Some(&[])),
        ],
    );
}

/// Ticket 188: the rogue's Danger Sense (+1 every 3 levels, +6 at 18) at 3, 6, 9, 12, 15 and 18. The Knife Master
/// (Blade Sense) and the Thug (Brutal Beating) lose it, and the Thug, with no rows at the steps, has rows of its own
/// that carry the class's rogue talents and Sneak Attack without it.
#[test]
fn the_rogues_danger_sense_steps_every_three_levels() {
    assert_gains(
        "rogue",
        "rogue/danger-sense",
        &|_, build| build,
        &[3, 6, 9, 12, 15, 18],
        &[("rogue/knife-master", Some(&[])), ("rogue/thug", Some(&[]))],
    );
}

/// Ticket 188: the fighter's Bravery (+1 every four levels, +5 at 18), Armor Training (every four levels from 3, to
/// 15) and Weapon Training (every four levels from 5, to 17). An archetype whose table has no Bravery (the Armiger's
/// Ardent, the Dragonheir Scion's Fearful Might, the Tower Shield Specialist's Burst Barrier, the Two-Handed
/// Fighter's Strong Grip), no Armor Training (the Aldori Defender, the Dragonheir Scion, the Mutation Warrior, the
/// Titan Fighter, the Two-Handed Fighter) or no Weapon Training (the Titan Fighter, ticket 131; the Tower Shield
/// Specialist; the Two-Handed Fighter) holds none of its steps. The Defender of the Hearth's first Weapon Training
/// is its delayed one at 9 and it inherits the steps at 13 and 17 (an inference: the guide gives it none).
#[test]
fn the_fighters_bravery_armor_training_and_weapon_training_step() {
    assert_gains(
        "fighter",
        "fighter/bravery",
        &|_, build| build,
        &[2, 6, 10, 14, 18],
        &[
            ("fighter/armiger", Some(&[])),
            ("fighter/dragonheir-scion", Some(&[])),
            ("fighter/tower-shield-specialist", Some(&[])),
            ("fighter/two-handed-fighter", Some(&[])),
        ],
    );
    assert_gains(
        "fighter",
        "fighter/armor-training",
        &|_, build| build,
        &[3, 7, 11, 15],
        &[
            ("fighter/aldori-defender", Some(&[])),
            ("fighter/dragonheir-scion", Some(&[])),
            ("fighter/mutation-warrior", Some(&[])),
            ("fighter/titan-fighter", Some(&[])),
            ("fighter/two-handed-fighter", Some(&[])),
        ],
    );
    assert_gains(
        "fighter",
        "fighter/weapon-training",
        &|_, build| build,
        &[5, 9, 13, 17],
        &[
            ("fighter/defender-of-the-hearth", Some(&[9, 13, 17])),
            ("fighter/titan-fighter", Some(&[])),
            ("fighter/tower-shield-specialist", Some(&[])),
            ("fighter/two-handed-fighter", Some(&[])),
        ],
    );
}

/// Ticket 188: the two archetype features that replace a stepped base feature and rise themselves (the guide: the
/// Armiger's Ardent +1 every four levels from 2, +5 at 18; the Aldori Defender's Defensive Parry +1 every four levels
/// after 3, +5 at 19), on the rows the base features' steps force the two archetypes to have.
#[test]
fn the_armigers_ardent_and_the_aldoris_defensive_parry_step() {
    for (feature, levels) in [
        ("fighter/armiger/ardent", &[2, 6, 10, 14, 18][..]),
        (
            "fighter/aldori-defender/defensive-parry",
            &[3, 7, 11, 15, 19][..],
        ),
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "fighter",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(levels))],
        );
    }
}
