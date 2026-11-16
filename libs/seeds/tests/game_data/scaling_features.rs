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

const STUDIED_TARGET: &str = "studied-target";

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
    // Game: SlayerProgression 1f35d11937a06624b8a60db68c7869a1 grants SlayerStudyTargetFeature
    // 09bdd9445ac38044389476689ae8d5a1 at 1, 5, 10, 15 and 20.
    assert_eq!(counts(&build, STUDIED_TARGET, &STEP_LEVELS), STEP_COUNTS);
}

#[test]
fn every_slayer_archetype_that_keeps_studied_target_steps_it_too() {
    // The Imitator gives it up (below); every other archetype keeps it (the Bloodseeker needs an evil alignment).
    // Game: BloodseekerArchetype 11e473bfb29c42f580b9a801b6771788 has a PrerequisiteAlignment of Evil; of the
    // archetypes only ImitatorArchetype 5947427f10bc497dbc24f66129b43e5d removes SlayerStudyTargetFeature
    // 09bdd9445ac38044389476689ae8d5a1.
    let mut kept = Vec::new();
    for archetype in DATA.archetypes_of(&class("slayer").key) {
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
    // Game: SlayerClass c75e0971973957d4dbad24bc7957e4fb lists these seven archetypes and the Imitator.
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
    // Game: ImitatorArchetype 5947427f10bc497dbc24f66129b43e5d has PrerequisiteFeature HumanRace
    // 0a5d473ead98b0646b94495af250fdc4.
    let build = human(archetype_build("slayer", "slayer/imitator", 20));
    assert!(engine().is_possible(&build));
    // Game: ImitatorArchetype 5947427f10bc497dbc24f66129b43e5d removes SlayerStudyTargetFeature
    // 09bdd9445ac38044389476689ae8d5a1 at 1, 5, 10, 15 and 20 and SlayerSwiftStudyTargetFeature
    // 40d4f55a5ac0e4f469d67d36c0dfc40b at 7.
    assert_eq!(counts(&build, STUDIED_TARGET, &STEP_LEVELS), [0; 9]);
    assert_eq!(counts(&build, "studied-target-swift", &[7, 20]), [0, 0]);
    // Game: DefensiveStudy ec041312142ba1f48a0044521f68fc0a has PrerequisiteFeature SlayerStudyTargetFeature
    // 09bdd9445ac38044389476689ae8d5a1.
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

/// The Imitator's row at 20 trades the slayer's Master Slayer for one of its own (it keeps the slayer talent).
#[test]
#[ignore = "waits on ticket 276: the game gives the Imitator its own Master Slayer at 20, not the slayer's"]
fn a_human_imitator_holds_its_own_master_slayer_at_20_not_the_slayers() {
    let build = human(archetype_build("slayer", "slayer/imitator", 20));
    assert!(engine().is_possible(&build));
    // Game: ImitatorArchetype 5947427f10bc497dbc24f66129b43e5d removes MasterSlayerFeature
    // a26c0279a423fc94cabeea898f4d9f8a at 20 and adds ImitatorMasterSlayerFeature 97c7054442504513b0324c00db0290fc.
    assert_eq!(counts(&build, "slayer/master-slayer", &[19, 20]), [0, 0]);
}

#[test]
fn the_other_classes_copies_step_at_the_same_levels() {
    // Game: DivineScionArchetype 5ce3a2f8eae64cf2ba83865711e53e27 adds SlayerStudyTargetFeature
    // 09bdd9445ac38044389476689ae8d5a1 at 1, 5, 10, 15 and 20; SanctifiedSlayerStudyTargetFeature
    // 6d9d3fbc30564e64d966dba27cd6357a and TandemExecutionerStudyTargetFeature 3dac0ae65dec4b619c035df07751b59d grant
    // it once at 1, its bonus 1 + level / 5 (SlayerStudyTargetBuff 45548967b714e254aa83f23354f174b0) (equivalent model:
    // the planner counts the rises as grants).
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
        ("paladin", "paladin/divine-scion", "studied-target"),
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
        // Game: BloodlineDraconicRedProgression 8c6e5b3cf12f71e43949f52c41ae70a8 grants
        // BloodlineDraconicRedResistancesAbilityAddLevel1 e3f7c6de540a3ed478fa32be32c7fc02 at 3 and
        // BloodlineDraconicRedResistancesAbilityAddLevel2 915c31bf91a8d8e4f8009fc487ac1786 at 9, whose
        // AddFeatureOnClassLevel switches at sorcerer level 15 (equivalent model; the other colours have the same
        // shape).
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
        // Game: granted at 4; BloodragerDraconicResistanceBuffFire 54d7b792a931f20459bb983a063aa534 (and the other
        // elements') rank rises at 8, BloodragerDraconicResistanceACBuff 89ac2964ef12ddb468d5c3ade461eb68's at 8 and
        // 16 (equivalent model: the planner counts the rises as grants).
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
    // Each granted once and rising by its buff's rank on bloodrager level (equivalent model: the planner counts the
    // rises as grants).
    let steps: [(&str, &str, &[u8], &[u8]); 3] = [
        // Game: BloodragerAbyssalBloodline 55d5bbf4b5ae1744ab26c71be98067f9 grants BloodragerAbyssalDemonResistances
        // cbe641b9f6e57d94280ef05eb8aa7121 at 8; BloodragerAbyssalResistanceBuff f2ad59f72a9544738a20dfb4e3d34ddd
        // rises at 16.
        (
            "bloodrager/bloodline/abyssal",
            "bloodrager/bloodline/abyssal/demon-resistances",
            &[7, 8, 15, 16],
            &[0, 1, 1, 2],
        ),
        // Game: BloodragerCelestialResistances 16b632b134b2c27458fb2559e72815f4 at 4;
        // BloodragerCelestialResistancesBuff f1ceaeea9311e0f4884f069ebfa00b3b rises at 12.
        (
            "bloodrager/bloodline/celestial",
            "bloodrager/bloodline/celestial/celestial-resistances",
            &[3, 4, 11, 12],
            &[0, 1, 1, 2],
        ),
        // Game: BloodragerSerpentineResistances4 6ef7c801217421647a68e74dd4678fe9 at 4;
        // BloodragerSerpentineResistanesBuff ef2d9784fcd404d4f9bd97840541b2e5 rises at 8, 12, 16 and 20.
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
    // Game: CrossbloodedArchetype e66a112ab57f69e418f1f11d5d2f3248 and GeomancerArchetype
    // a0e56a59ad0b44b8add84185da6bb845 keep SorcererBloodlineSelection.
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
    // Game: CrossbloodedArchetype e66a112ab57f69e418f1f11d5d2f3248's second bloodline is a
    // CrossbloodedSecondaryBloodline*Progression, which grants no power (equivalent model).
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
    // Game: EmpyrealSorcererArchetype aa00d945f7cf6c34c909a29a25f2df38, SageSorcererArchetype
    // 00b990c8be2117e45ae6514ee4ef561c and SylvanSorcererArchetype 711d5024ecc75f346b9cda609c3a1f83 replace
    // SorcererBloodlineSelection with a fixed progression (Empyreal, Sage, Sylvan).
    for archetype in [
        "sorcerer/empyreal-sorcerer",
        "sorcerer/sage-sorcerer",
        "sorcerer/sylvan-sorcerer",
    ] {
        let option = format!("{archetype}/bloodline");
        let draconic = DATA
            .selections
            .iter()
            .filter(|selection| selection.key.as_str() == option)
            .flat_map(|selection| &selection.options)
            .any(|option| option.as_str().contains("draconic"));
        assert!(!draconic, "{archetype}");
    }
    // The bloodrager's archetypes keep the bloodline's rows; the Primalist's rage-power choice is a known
    // consequence (see the ticket), not asserted here.
    // Game: BloodriderArchetype 9a8b323e5559dab45a23e0dc0b8e9c52, GreenragerArchetype
    // 5648585af75596f4a9fa3ae385127f57, SpelleaterArchetype afe79636228fdd64ea8d9bf3644838a3 and SteelbloodArchetype
    // 32a5dff92373a9641b43e97d453b9369 keep BloodragerBloodlineSelection 62b33ac8ceb18dd47ad4c8f06849bc01.
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
    // Game: ReformedFiendBloodlineSelection dd62cb5011f64cd38b8b08abb19ba2cc offers BloodragerAbyssalBloodline
    // 55d5bbf4b5ae1744ab26c71be98067f9 and the Infernal one.
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
        // Game: DragonheirScionArchetype 8dff97413c63c1147be8a5ca229abefc takes DragonheirDragonSelection
        // 729411185291d704696e58316420fe38 of the Dragonheir*Progressions (DragonheirRedProgression
        // 267dbd4789fa4b75a44294c7d1625bba), never the sorcerer's bloodline (equivalent model: the planner's
        // selection takes the Draconic option and skips every row).
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
        // Game: DragonheirScionArchetype 8dff97413c63c1147be8a5ca229abefc adds FearfulMightFeature
        // 6ba2b2178dfc4331a34cc2a8a2b68d6a at 2 (its rank rises 6, 10, 14, 18) and DraconicPresenceFeature
        // f6b7f213fc0e428d9a01ed4b02b50cce at 6; DragonheirRedProgression 267dbd4789fa4b75a44294c7d1625bba
        // gives DragonicDefencesFire d2a31af5bd02f9a4a87d6015fdba56bb at 3 (its rank rises 7, 13), the Strike at 4 and
        // Might of Wyrms at 20 (equivalent model: the planner counts the rises as grants).
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
        // Game: DragonheirScionArchetype 8dff97413c63c1147be8a5ca229abefc adds ArcaneStrikeFeature
        // 0ab2f21a922feee4dab116238e3150b4 at 1; DragonheirRedProgression 267dbd4789fa4b75a44294c7d1625bba gives
        // FeatureWingsDraconicRed df86a121e5d1bd04b8afdbaf8d24a1ce at 15 (equivalent model: Wings is a feat here).
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
    // Game: BloodlineDraconicRedProgression 8c6e5b3cf12f71e43949f52c41ae70a8 lists MagusClass with
    // EldritchScionArchetype d078b2ef073f2814c9e338a789d97b73, and BloodlineDraconicRedResistancesAbilityAddLevel2
    // 915c31bf91a8d8e4f8009fc487ac1786's switch at 15 counts magus levels.
    assert_eq!(
        counts(&scion, DRACONIC, &[2, 3, 8, 9, 14, 15, 20]),
        [0, 1, 1, 2, 2, 3, 3]
    );
    // The Seeker takes a bloodline of its own (its `SeekerBloodline*Progression`), with no resistance at 3. What its
    // rows give from 9 is `a_seeker_holds_the_bloodline_steps_its_own_progressions_give`.
    // Game: SeekerBloodlineDraconicRedProgression d69a7785de1959c4497e4ff1e9490509 has no row before 9 with a
    // resistance.
    assert!(engine().is_possible(&seeker(RED)));
    assert_eq!(counts(&seeker(RED), DRACONIC, &[3, 8]), [0, 0]);
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
    // Game: BloodlineDraconicRedProgression 8c6e5b3cf12f71e43949f52c41ae70a8 lists DragonDiscipleClass
    // 72051275b1dbb2d42ba9118237794f7c, and BloodlineDraconicRedResistancesAbilityAddLevel2
    // 915c31bf91a8d8e4f8009fc487ac1786's switch at 15 counts its levels.
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
        .feature(&FeatureKey::new("sorcerer/bloodline-ascendance"))
        .and_then(|feature| feature.option_row.as_ref())
        .unwrap();
    assert_eq!(ascendance.level, 20);
    let bloodlines = DATA.selection(&ascendance.selection).unwrap();
    for option in DATA.options_of(bloodlines) {
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

/// A level 20 Seeker (sorcerer) with `option` taken at its level-1 bloodline selection.
fn seeker(option: &str) -> Build {
    with_option(
        archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 20),
        1,
        0,
        option,
    )
}

/// The Seeker's bloodline is the game's own `SeekerBloodline*Progression`, which keeps some of the bloodline's stepped
/// powers on its rows from 9: the counts are those the plain sorcerer has with the same blueprints.
#[test]
#[ignore = "waits on ticket 281: the game gives the Seeker the Draconic resistances from 9, Death's Gift \
            at 9, Snakeskin and Strength of the Abyss at 9, 13 and 17"]
fn a_seeker_holds_the_bloodline_steps_its_own_progressions_give() {
    const DRACONIC: &str = "sorcerer/bloodline/draconic/draconic-resistances";
    // Game: SeekerBloodlineDraconicRedProgression d69a7785de1959c4497e4ff1e9490509 grants
    // BloodlineDraconicRedResistancesAbilityAddLevel2 915c31bf91a8d8e4f8009fc487ac1786 at 9 (the plain sorcerer's
    // second step), whose AddFeatureOnClassLevel switches at sorcerer level 15.
    assert_eq!(
        counts(
            &seeker("sorcerer/bloodline/draconic-red"),
            DRACONIC,
            &[8, 9, 14, 15]
        ),
        [0, 2, 2, 3]
    );
    // Game: SeekerBloodlineAbyssalProgression 17b752be1e0f4a34e8914df52eebeb75 grants
    // BloodlineAbyssalStrengthAbilityAddLevel1 489c8c4a53a111d4094d239054b26e32 at 9 and
    // BloodlineAbyssalStrengthAbilityAddLevel2 48d81d94ddf0cc84abbde97718cb25f2 at 13, which switches at 17.
    assert_eq!(
        gains(
            &seeker(ABYSSAL),
            "sorcerer/bloodline/abyssal/strength-of-the-abyss"
        ),
        [9, 13, 17]
    );
    // Game: SeekerBloodlineUndeadProgression 5bc63fdb68b539f4fa500cfb2d0fe0f6 grants BloodlineUndeadDeathsGift
    // fd5f14d44e82f464196fdf0ea82347cc once, at 9.
    assert_eq!(
        gains(
            &seeker("sorcerer/bloodline/undead"),
            "sorcerer/bloodline/undead/deaths-gift"
        ),
        [9]
    );
    // Game: SeekerBloodlineSerpentineProgression 59904bf6cc50a52489ebc648fb35f36f grants
    // BloodlineSerpentineSnakeskinFeatureAddLevel1 19d2ea3d212681946920c7b70bb44492 at 9 and
    // BloodlineSerpentineSnakeskinFeatureAddLevel2 dc3f843c9c9e5564080d35f102c84eed at 13, which switches at 17.
    assert_eq!(
        gains(
            &seeker("sorcerer/bloodline/serpentine"),
            "sorcerer/bloodline/serpentine/snakeskin"
        ),
        [9, 13, 17]
    );
}

/// A skipped level drops the option's whole row, so a selection that skips the row where a feature is first granted
/// must skip every later row that grants it again (a step), or it gains a step of a feature it never had: the
/// Dragonheir Scion's resistances at 9 before ticket 179. Every selection, over every option it offers.
#[test]
fn no_selection_gains_a_step_of_a_feature_whose_first_row_it_skips() {
    let mut leaks = Vec::new();
    for selection in &DATA.selections {
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

/// The levels at which `build`'s step of `feature` rises (a grant, or a step its level reaches: ticket 254), a level
/// once for each rise: the steps the sheet gains it at.
fn gains(build: &Build, feature: &str) -> Vec<u8> {
    let key = FeatureKey::new(feature);
    let sheet = engine().evaluate(build);
    let mut previous = 0;
    let mut gained = Vec::new();
    for level in 1..=build.levels.len() as u8 {
        let count = engine().feature_step(sheet.at(level).unwrap(), &key);
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
            DATA.selections
                .iter()
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
        DATA.archetypes_of(&class.key)
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
        DATA.selection(key).is_some_and(|selection| {
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
/// it (checked from the selections' options, not from a failed Build) and holds the feature never; one [`WAITING`]
/// has every check but the levels it gains.
fn assert_gains_of_option(
    class_key: &str,
    feature: &str,
    option: &str,
    takes: &dyn Fn(Option<&str>) -> bool,
    prepare: &dyn Fn(Option<&str>, Build) -> Build,
    default: &[u8],
    exceptions: &[(&str, Option<&[u8]>)],
) {
    let archetypes = DATA
        .archetypes_of(&class(class_key).key)
        .map(|a| Some(a.key.as_str()));
    for (key, _) in exceptions {
        assert!(
            DATA.archetypes_of(&class(class_key).key)
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
        if expected != WAITING {
            assert_eq!(gains(&build, feature), expected, "{label}");
        }
    }
}

/// An exception's levels that [`assert_gains_of_option`] leaves unasserted: the game gives that archetype others, and
/// a test of its own, named for what the game does, waits on the ticket that brings the data to them.
const WAITING: &[u8] = &[u8::MAX];

/// [`assert_gains`] for a feature of one archetype (its key's parent) that it gains at `levels`, every other archetype
/// and the class never.
fn assert_archetype_steps(class_key: &str, feature: &str, levels: &[u8]) {
    let archetype = feature.rsplit_once('/').expect("an archetype's feature").0;
    assert_gains(
        class_key,
        feature,
        &|_, build| build,
        &[],
        &[(archetype, Some(levels))],
    );
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

/// The sorcerer's bloodlines whose power steps at bloodline level 9 (ticket 177), with the guide's sentence for each in
/// the ticket's Answer, and what every archetype of the sorcerer gains it at: the keeping archetypes at the bloodline's
/// own levels (a Crossblooded's first bloodline, the Geomancer, the Overwhelming Mage, the Nine-Tailed Heir), the
/// Seeker no resistance (its own Death's Gift and Snakeskin wait on a ticket), the Empyreal Sorcerer only for its fixed
/// Celestial bloodline, and the Sage and the Sylvan not at all (their bloodlines are fixed: nothing to pick, so nothing
/// held).
#[test]
fn every_sorcerer_archetype_holds_the_steps_of_the_bloodline_resistances() {
    // Each resistance is granted at 3 and switches at sorcerer level 9 by its AddFeatureOnClassLevel (equivalent
    // model: the planner counts the switch as a grant).
    let steps: [(&str, &str, &[u8]); 8] = [
        // Game: BloodlineAbyssalProgression d3a4cb7be97a6694290f0dcfbd147113, BloodlineAbyssalResistancesAbilityAdd
        // 8cbf303d479cf0d42a8e36092c76fa7c.
        (
            "sorcerer/bloodline/abyssal",
            "sorcerer/bloodline/abyssal/demon-resistances",
            &[3, 9],
        ),
        // Game: BloodlineCelestialProgression aa79c65fa0e11464d9d100b038c50796,
        // BloodlineCelestialResistancesAbilityAdd b46ef711b4825e04a9380bd0b88d5b06.
        (
            "sorcerer/bloodline/celestial",
            "sorcerer/bloodline/celestial/celestial-resistances",
            &[3, 9],
        ),
        // Game: BloodlineInfernalProgression e76a774cacfb092498177e6ca706064d,
        // BloodlineInfernalResistancesAbilityAdd c8b51c7112077384da68c48359419046.
        (
            "sorcerer/bloodline/infernal",
            "sorcerer/bloodline/infernal/infernal-resistances",
            &[3, 9],
        ),
        // Game: BloodlineUndeadProgression a1a8bf61cadaa4143b2d4966f2d1142e grants BloodlineUndeadDeathsGift
        // fd5f14d44e82f464196fdf0ea82347cc at 3 and 9.
        (
            "sorcerer/bloodline/undead",
            "sorcerer/bloodline/undead/deaths-gift",
            &[3, 9],
        ),
        // Game: BloodlineSerpentineProgression 739c1e842bf77994baf963f4ad964379 grants
        // BloodlineSerpentineSnakeskinFeatureAddLevel1 19d2ea3d212681946920c7b70bb44492 at 9 and
        // BloodlineSerpentineSnakeskinFeatureAddLevel2 dc3f843c9c9e5564080d35f102c84eed at 13, which switches at 17.
        (
            "sorcerer/bloodline/serpentine",
            "sorcerer/bloodline/serpentine/snakeskin",
            &[9, 13, 17],
        ),
        // Game: BloodlineElementalAirProgression cd788df497c6f10439c7025e87864ee4,
        // BloodlineElementalAirResistanceAbilityAdd 6472c51065d734e4b99ac56694925920.
        (
            "sorcerer/bloodline/elemental-air",
            "sorcerer/bloodline/elemental/elemental-resistances",
            &[3, 9],
        ),
        // Game: BloodlineElementalEarthProgression 32393034410fb2f4d9c8beaa5c8c8ab7,
        // BloodlineElementalEarthResistanceAbilityAdd 3dc0ba141d7aeb647ac428cdaf05b42a.
        (
            "sorcerer/bloodline/elemental-earth",
            "sorcerer/bloodline/elemental/elemental-resistances",
            &[3, 9],
        ),
        // Game: BloodlineElementalFireProgression 17cc794d47408bc4986c55265475c06f,
        // BloodlineElementalFireResistanceAbilityAdd 24980315c1bdcc4478ebb717e9b81961.
        (
            "sorcerer/bloodline/elemental-fire",
            "sorcerer/bloodline/elemental/elemental-resistances",
            &[3, 9],
        ),
    ];
    for (option, feature, levels) in steps {
        let celestial = option == "sorcerer/bloodline/celestial";
        // Game: EmpyrealBloodlineProgression 8a95d80a3162d274896d50c2f18bb6b1 grants
        // BloodlineCelestialResistancesAbilityAdd b46ef711b4825e04a9380bd0b88d5b06 at 3.
        let empyreal: Option<&[u8]> = celestial.then_some(levels);
        // Game: the Seeker's own SeekerBloodline*Progressions grant no resistance, but Death's Gift and Snakeskin
        // from 9 (`a_seeker_holds_the_bloodline_steps_its_own_progressions_give`, ticket 281).
        let seeker: Option<&[u8]> = match option {
            "sorcerer/bloodline/undead" | "sorcerer/bloodline/serpentine" => Some(WAITING),
            _ => Some(&[]),
        };
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
                // The Seeker is offered every bloodline and holds the one it picks, but none of its resistances.
                ("sorcerer/seeker-sorcerer", seeker),
                // The Empyreal Sorcerer's fixed bloodline is Celestial; the Sage's Arcane and the Sylvan's Fey.
                ("sorcerer/empyreal-sorcerer", empyreal),
                ("sorcerer/sage-sorcerer", None),
                ("sorcerer/sylvan-sorcerer", None),
            ],
        );
    }
    // The fourth Elemental element, and every one of the four through the class itself.
    // Game: BloodlineElementalWaterProgression 7c692e90592257a4e901d12ae6ec1e41,
    // BloodlineElementalWaterResistanceAbilityAdd 30ba5298a6246184cb034aa7ef0c6b12 (equivalent model, as above).
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
    // Infernal Resistance 4, 5 and 8; Abyssal Bloodrage 12, 16 and 20. The Hag-Riven and the Mixed Blood Rager are not
    // offered either bloodline; the Primalist's rage-power choice is the known open question (ticket 177's Answer),
    // and a Primalist taking the bloodline's ability at every level holds them all.
    // Game: BloodragerAbyssalBloodline 55d5bbf4b5ae1744ab26c71be98067f9 and BloodragerInfernalBloodline
    // 9aef64b53406f114cb43f898a3aec01e each have a PrerequisiteNoArchetype of HagRivenArchetype
    // 61edfe08d9e04a11844e5fcc01da3602; MixedBloodRagerArchetype 10d92d54d19e4a59b5bbd4259ec0974f removes
    // BloodragerBloodlineSelection 62b33ac8ceb18dd47ad4c8f06849bc01 at 1.
    for (option, feature, levels) in [
        // Game: BloodragerInfernalBloodline 9aef64b53406f114cb43f898a3aec01e grants BloodragerInfernalResistance
        // ac77b22f74aed034ca2b16ba6a4bfde9 at 4; BloodragerInfernalResistanceBuff 66c973ccad1293345ae6c5bfeced4a7b has
        // two ContextRankConfigs on BloodragerClass level: the resistance's table rises at 8, the poison-save bonus's
        // (DamageDice, custom {4 -> 2, 8 -> 4}; ApplyProgression takes the first entry with value <= base) at 5.
        (
            "bloodrager/bloodline/infernal",
            "bloodrager/bloodline/infernal/infernal-resistance",
            &[4u8, 5, 8][..],
        ),
        // BloodragerAbyssalBloodrage 29fc83f824111bb41829957486752c93 is granted once at 12; its steps at 16 and 20
        // are not found in the blueprints (unsettled).
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
    // Game: InstinctualWarriorArchetype adffdd8a99094a89823a79292a503ee9 adds CunningElusionUnlockFeature
    // 91c8b2e3abdb4e2e807fddb668b619f8 at 2, its bonus level / 4 (equivalent model: the planner counts the rises).
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
    // Game: CrushAndTearFeature 68684e91025d4ebf892e8c0469d14b03's AddFeatureOnClassLevel steps at 1, 5, 11 and 17
    // (equivalent model).
    assert_archetype_steps(
        "barbarian",
        "barbarian/flesheater/crush-and-tear",
        &[1, 5, 11, 17],
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
        // Game: DragonheirRedProgression 267dbd4789fa4b75a44294c7d1625bba gives DragonicDefencesFire
        // d2a31af5bd02f9a4a87d6015fdba56bb at 3, its rank rising at 7 and 13; DragonheirScionArchetype
        // 8dff97413c63c1147be8a5ca229abefc adds FearfulMightFeature 6ba2b2178dfc4331a34cc2a8a2b68d6a at 2, its rank
        // rising at 6, 10, 14 and 18 (equivalent model: the planner counts the rises as grants).
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
fn the_alchemists_reanimator_steps() {
    // Game: ReanimatorArchetype 5faf2921393d422d94467174d029e4cd adds ReanimatorResearchOfCorpsesFeature
    // a3b73e40ce9c48bcb6dc866a541e8bb4 at 1, its rank 1 + (level + 1) / 6 rising at 5, 11 and 17 (equivalent model).
    assert_archetype_steps(
        "alchemist",
        "alchemist/reanimator/corpse-studying",
        &[1, 5, 11, 17],
    );
}

/// The Metamorph's Adaptive Physiology is granted once, at 3, with a fixed fortification and nothing that rises.
#[test]
#[ignore = "waits on ticket 280: the game gives Adaptive Physiology once, at 3"]
fn the_metamorphs_adaptive_physiology_is_granted_once_at_3() {
    // Game: MetamorphArchetype ac242ddeaa05af449bfc5230231fab84 adds AdaptivePhysiologyFeature
    // cec88d5f7ebda294c93b95cd7360f504 at 3 only (an AddFortification of a fixed 25), and no other blueprint grants it.
    assert_archetype_steps("alchemist", "alchemist/metamorph/adaptive-physiology", &[3]);
}

#[test]
fn the_fake_celebritys_inhibit_courage_steps() {
    const WAYS: &str = "arcanist/magic-deceiver/ways-of-the-deceiver";
    // Each of the Magic Deceiver's six ways: only the Fake Celebrity holds Inhibit Courage.
    // Game: MagicDeceiverWayOfFakeCelebrityProgression fc120e8f61a84694adfd5ff5298be820 grants
    // FakeInspireCourageFeature d754e431ef504adaaccdfbb90174657a at 1; its steps at 5, 11 and 17 are not found in the
    // blueprints (unsettled).
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
    // Game: DefenderOfTheTrueWorld 782c46afacd88d448afba6178471a744 adds FeyStalkerMasterFeature
    // 02357ba2802b8654bb3e824bae68f5c0 at 3, 8, 13 and 18.
    assert_archetype_steps(
        "druid",
        "druid/defender-of-the-true-world/fey-stalker",
        &[3, 8, 13, 18],
    );
    // Game: OverwhelmingSoulArchetype aa11888104d17f7459851e8d559ffa98 adds OverwhelmingSoulOverwhelmingPowerFeature
    // 6c24bf7cc2266f24f9b0c652799f1d91 at 3, 6, 9, 12, 15 and 18.
    assert_archetype_steps(
        "kineticist",
        "kineticist/overwhelming-soul/overwhelming-power",
        &[3, 6, 9, 12, 15, 18],
    );
    // Game: DemonDancerArchetype 9c46c4e4303e5da40b63faa6df18e2c2 adds FiendishMaw1d6Feature
    // a5f55fc653c389e4ea1c032342a507bd at 3, FiendishMaw1d8Feature b78ddb104afa442408c520f84b0c0281 at 7,
    // FiendishMaw2d6Feature 594ac4d1c4067d74a881a1a4a7cfcb31 at 12 and FiendishMaw3d6Feature
    // 486c52f5ef06ec54f82683ac221b2a6a at 17 (equivalent model: one feature stepped).
    assert_archetype_steps("skald", "skald/demon-dancer/fiendish-maw", &[3, 7, 12, 17]);
}

#[test]
fn the_shifters_aspects_and_defences_step() {
    // Game: RageshaperShifterArchetype 41d5b44e421048d09d32623689173e11 adds InvulnerableDefensesShifterFeature
    // 44ad5d0af1474c74a616723bfb5c13f5 at 2, its rank rising at 10, 15 and 20 (equivalent model).
    assert_archetype_steps(
        "shifter",
        "shifter/rageshaper/invulnerable-defenses",
        &[2, 10, 15, 20],
    );
    // Game: FiendfleshShifterShifterArchetype f5d22b6b5a7940d49a4d494f67a91efa adds ShifterAspectFiend
    // a52705ab3a0644e6922cffe6bbe2b002 at 1, wings at 5, its rank rising at 5, 10, 15 and 20 (equivalent model).
    assert_archetype_steps(
        "shifter",
        "shifter/fiendflesh-shifter/fiendish-aspect",
        &[1, 5, 10, 15, 20],
    );
}

/// The Weretouched's Lycanthropic Aspect: granted at 1, its dice rising every seven levels after.
#[test]
#[ignore = "waits on ticket 277: the game gives Lycanthropic Aspect at 1, 8 and 15"]
fn the_weretouched_lycanthropic_aspect_steps_at_1_8_and_15() {
    // Game: WereTouchedShifter a30313b4b01a4049ae2f50884917e443 adds ShifterAspectWereTouchedFeature
    // d6f5aa6fba3149c28a25036fceed03c9 at 1; ShifterAspectWereTouchedBuff 20433fe89d654779a3eb44cdc9feacee's dice are
    // 1 + (level - 1) / 7, rising at 8 and 15 (equivalent model: the planner counts the rises as grants).
    assert_archetype_steps(
        "shifter",
        "shifter/weretouched/lycanthropic-aspect",
        &[1, 8, 15],
    );
}

#[test]
fn the_battle_spirit_steps_at_8_and_16_for_every_shaman_that_takes_it() {
    // Game: ShamanBattleSpiritFeature 5e1161a3a7e5a83458de8a5f412c8d0c; ShamanBattleSpiritEffectBuff
    // 7a76add83f3397c4eacc59ebd8b804d8's rank is 1 + shaman level / 8, rising at 8 and 16 (equivalent model).
    // ProphetOfPestilenceArchetype bcd758a75fb54651a7f668fe2661a307 and UnswornShamanArchetype
    // 556590a43467a27459ac1a80324c9f9f remove the class's spirit selection at 1.
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
    // Game: AssassinProgression a02e1f0e13baa8c43b758425eda9e973 grants AssassinColdMindedFeature
    // dc71af4fbb014aae958c11fa10ec6ab3 at 2, 5 and 8.
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
    // Game: WinterWitchProgression 2703dc14f69421a44866e9046a95f348 grants WinterWitchColdSpecialization
    // 9b827777e51ec0540ba6a0cbea6ea84a at 1, which adds WinterWitchColdSpecialization1
    // 0a98787328ad435ea00c4b2a193ea7c2 before Winter Witch 5, and from 5 WinterWitchColdSpecializationUpgreader
    // 03c3d24d15954791a07b938d9434dd19, which adds WinterWitchColdSpecialization5 fa40f1218b824073b7759dc0d4ca5a55
    // before 10 and WinterWitchColdSpecialization10 d2926dfd0c3d4918bff84d8c292e732a from 10 (equivalent model).
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
    // Game: the sorcerer's bloodline progressions list MagusClass with EldritchScionArchetype
    // d078b2ef073f2814c9e338a789d97b73, and their switches count magus levels (BloodlineAbyssalResistancesAbilityAdd
    // 8cbf303d479cf0d42a8e36092c76fa7c) (equivalent model, as in
    // `every_sorcerer_archetype_holds_the_steps_of_the_bloodline_resistances`).
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
    // The Second Bloodrager Bloodline: Infernal Resistance granted at 6, with the step at 5 already passed (the game
    // reads bloodrager level whenever the feature is held: the grant and that step at 6), then the step at 8.
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
        [6, 6, 8]
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

/// A level 20 Primalist with `option` for its bloodline, taking the two rage powers at each of `rage_powers_at`.
fn primalist(option: &str, rage_powers_at: &[u8]) -> Build {
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
}

const PRIMALIST_INFERNAL: &str = "bloodrager/bloodline/infernal";
const PRIMALIST_RESISTANCE: &str = "bloodrager/bloodline/infernal/infernal-resistance";
const PRIMALIST_ABYSSAL: &str = "bloodrager/bloodline/abyssal";
const PRIMALIST_BLOODRAGE: &str = "bloodrager/bloodline/abyssal/abyssal-bloodrage";

/// The Primalist (ticket 177): "Two Rage Powers" replaces the bloodline's whole row at 4, 8, 12, 16 or 20. A power
/// kept rises with the bloodrager level whatever is traded later, since its steps are its rank on the class level, not
/// rows (ticket 254). What a traded power does is
/// `a_primalists_traded_power_never_steps_and_a_kept_one_steps_with_the_class_level`.
#[test]
fn a_primalists_rage_powers_replace_the_row_and_its_step_with_it() {
    // Game: BloodragerInfernalResistance ac77b22f74aed034ca2b16ba6a4bfde9 has a PrerequisiteNoFeature of
    // PrimalistProgression d8b8d1dd83393484cbacf6c8830080ae; PrimalistTakeBloodlinePower4
    // 2140040bf367e8b4a9c6a632820becbe adds it; BloodragerInfernalResistanceBuff 66c973ccad1293345ae6c5bfeced4a7b's
    // resistance table rises at 8 and its second ContextRankConfig (DamageDice, the poison-save bonus: custom
    // {4 -> 2, 8 -> 4}, ApplyProgression: first entry with value <= base) at 5.
    assert_eq!(
        gains(&primalist(PRIMALIST_INFERNAL, &[]), PRIMALIST_RESISTANCE),
        [4, 5, 8]
    );
    assert_eq!(
        gains(
            &primalist(PRIMALIST_INFERNAL, &[4, 8]),
            PRIMALIST_RESISTANCE
        ),
        Vec::<u8>::new()
    );
    // Game: BloodragerAbyssalBloodrage 29fc83f824111bb41829957486752c93 is granted at 12 only
    // (BloodragerAbyssalBloodline 55d5bbf4b5ae1744ab26c71be98067f9); BloodragerAbyssalBloodrageBonusProperty
    // 415f71e5a47f4cccb3dbd10bd7a0f8f8 multiplies a ClassLevelGetter on BloodragerClass (1 at 12-15, 2 at 16-19, 3 at
    // 20) by the feature's rank (`ClassLevelGetter`, `FactRankGetter`, `PropertySettings.ApplyProgression`,
    // WrathDecompiled 6ce4a6b), so trading the row at 16 or 20 takes no step with it.
    assert_eq!(
        gains(&primalist(PRIMALIST_ABYSSAL, &[]), PRIMALIST_BLOODRAGE),
        [12, 16, 20]
    );
    assert_eq!(
        gains(&primalist(PRIMALIST_ABYSSAL, &[16]), PRIMALIST_BLOODRAGE),
        [12, 16, 20]
    );
    assert_eq!(
        gains(&primalist(PRIMALIST_ABYSSAL, &[20]), PRIMALIST_BLOODRAGE),
        [12, 16, 20]
    );
    // Granted once: its feature rank stays 1 while its step reaches 3.
    let kept = at(&primalist(PRIMALIST_ABYSSAL, &[16]), 20);
    assert_eq!(kept.feature_count(&FeatureKey::new(PRIMALIST_BLOODRAGE)), 1);
    assert_eq!(
        engine().feature_step(&kept, &FeatureKey::new(PRIMALIST_BLOODRAGE)),
        3
    );
}

/// A Primalist's bloodline power is granted only by the level's choice: one traded for rage powers is never held, so
/// none of its steps come, and one kept steps with the bloodrager level whatever is traded later.
#[test]
fn a_primalists_traded_power_never_steps_and_a_kept_one_steps_with_the_class_level() {
    // Game: PrimalistTakeBloodlinePower4 2140040bf367e8b4a9c6a632820becbe adds BloodragerInfernalResistance
    // ac77b22f74aed034ca2b16ba6a4bfde9 (which a Primalist holds no other way: PrerequisiteNoFeature
    // PrimalistProgression d8b8d1dd83393484cbacf6c8830080ae), and its step at 8 is BloodragerInfernalResistanceBuff
    // 66c973ccad1293345ae6c5bfeced4a7b's rank on bloodrager level, not a grant on the row at 8.
    assert_eq!(
        gains(&primalist(PRIMALIST_INFERNAL, &[4]), PRIMALIST_RESISTANCE),
        Vec::<u8>::new()
    );
    assert_eq!(
        gains(&primalist(PRIMALIST_INFERNAL, &[8]), PRIMALIST_RESISTANCE),
        [4, 5, 8]
    );
    // Game: PrimalistTakeBloodlinePower12 57bb4dc36611c7444817c13135ec58b4 is the Primalist's only way to
    // BloodragerAbyssalBloodrage 29fc83f824111bb41829957486752c93 (PrerequisiteNoFeature PrimalistProgression).
    assert_eq!(
        gains(&primalist(PRIMALIST_ABYSSAL, &[12]), PRIMALIST_BLOODRAGE),
        Vec::<u8>::new()
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
                rules::SlotKind::Feat(selection)
                    if selection.as_str() == "bloodrager/bloodline-feats"
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
    // Game: HeraldOfTheHornArchetype 72fa7b1c7a6ede44eb490080bf4a2f90 adds HornCallFeature
    // 7ab19a006c4abb0489d6d1a99e54fb99 at 7, its rank rising at 13 and 19 (equivalent model).
    assert_archetype_steps("skald", "skald/herald-of-the-horn/horn-call", &[7, 13, 19]);
    let skald = build("skald", 20);
    let herald = archetype_build("skald", "skald/herald-of-the-horn", 20);
    assert!(engine().is_possible(&herald));
    // The Herald's table keeps both, as the class's does: Damage Reduction at 9, 14 and 19, the swift-action song
    // at 13.
    // Game: SkaldProgression 26418fed2bc153245972a5b54204ed75's rows, of which HeraldOfTheHornArchetype
    // 72fa7b1c7a6ede44eb490080bf4a2f90 removes none.
    for (feature, expected) in [
        ("skald/damage-reduction", &[9u8, 14, 19][..]), // Game: SkaldDR d9446a35d1401cf418bb9b5e0e199d57
        ("skald/raging-song-swift-action", &[13][..]), // Game: SkaldSwiftPerformance 0c12540bf7e45f04faa84730136cb726
        ("skald/raging-song-move-action", &[7][..]), // Game: SKaldMovePerformance f24dc98de96671d4288cba09584e249b
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
        // Game: ReformedFiendArchetype d55163eed9214367820654f0ebe0ff69 adds ReformedFiendBloodrageFeature
        // fbc27acd44ab4f1cb0e2cbc0b573add7 at 5, its rank 1 + level / 5 rising at 10, 15 and 20 (equivalent model).
        assert_eq!(
            gains(&build, "bloodrager/reformed-fiend/hatred-against-evil"),
            [5, 10, 15, 20],
            "{bloodline}"
        );
        // Game: ReformedFiendArchetype d55163eed9214367820654f0ebe0ff69 adds ReformedFiendDamageReductionFeature
        // 2a3243ad1ccf43d5a5d69de3f9d0420e at 7, 10, 13, 16 and 19, where it removes the class's.
        assert_eq!(
            gains(
                &build,
                "bloodrager/reformed-fiend/damage-reduction-reformed-fiend"
            ),
            [7, 10, 13, 16, 19],
            "{bloodline}"
        );
        // Game: BloodragerProgression 69dec3bda9a94914bb159cba128300b9 grants BloodragerMightyBloodrage
        // a6cd3eca05ee24840ab159ca47b4cd88 at 20, which the archetype keeps.
        assert_eq!(
            gains(&build, "bloodrager/mighty-bloodrage"),
            [20],
            "{bloodline}"
        );
        // The base row's bloodline feat slot at 15 stays, beside the ones at 6, 9, 12 and 18: those of the class.
        // Game: BloodragerAbyssalBloodline 55d5bbf4b5ae1744ab26c71be98067f9 has its feat selections at 6, 9, 12, 15
        // and 18.
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

/// The Flesheater's Power from Flesh (2, 9, 18): the row at 18 keeps the rage power the Flesheater's table keeps
/// there (4, 10, 12, 16, 18 and 20; none at 8).
#[test]
fn the_flesheaters_power_from_flesh_steps() {
    // Game: FleshEaterBarbarianArchetype 85c9d72cb20040a79542289dd5126694 adds PowerInTheFleshFeature
    // c53ac2508a4a4f29a22c22b108135200 at 2, its rank on barbarian level rising at 9 and 18 (equivalent model).
    assert_archetype_steps(
        "barbarian",
        "barbarian/flesheater/power-from-flesh",
        &[2, 9, 18],
    );
    let flesheater = playable(
        || archetype_build("barbarian", "barbarian/flesheater", 20),
        |build| build,
    )
    .unwrap();
    // Game: FleshEaterBarbarianArchetype 85c9d72cb20040a79542289dd5126694 removes RagePowerSelection
    // 28710502f46848d48b3f0d6132817c4e at 2, 6, 8 and 14.
    assert_eq!(
        levels_with_slot_of(&flesheater, "barbarian/rage-power"),
        [4, 10, 12, 16, 18, 20]
    );
}

/// The Flesheater's Devour Essence: granted at 2, its ability's rank rising every six levels after 1.
#[test]
#[ignore = "waits on ticket 269: the game gives Devour Essence at 2, 7, 13 and 19"]
fn the_flesheaters_devour_essence_steps_at_2_7_13_and_19() {
    // Game: FleshEaterBarbarianArchetype 85c9d72cb20040a79542289dd5126694 adds DevourEssenceFeature
    // 492abe84bb2c4e4ebf5fbc83f054ef78 at 2; DevourEssenceAbility 427f49e3eb644e269140774c62725c97's rank is
    // 1 + (level - 1) / 6, rising at 7, 13 and 19 (equivalent model: the planner counts the rises as grants).
    assert_archetype_steps(
        "barbarian",
        "barbarian/flesheater/devour-essence",
        &[2, 7, 13, 19],
    );
}

/// The Fiendflesh Shifter's Fiendish Resilience is one feature stepped at 2, 4, 8, 12, 16 and 20: the guide gives
/// it as one ability with two series (natural armour at 2, 4, 12 and 20; the resistance at 2, 8 and 16), and each
/// step improves one of them. Nothing reads its count.
#[test]
fn the_fiendflesh_shifters_fiendish_resilience_steps_at_both_series() {
    // Game: FiendflashShifteACBonusUnlock cb30a00a269b4d3e9542da1c3a18fab0, granted at 2: its armour rank rises at 4,
    // 12 and 20, its resistance rank at 8 and 16 (equivalent model).
    assert_archetype_steps(
        "shifter",
        "shifter/fiendflesh-shifter/fiendish-resilience",
        &[2, 4, 8, 12, 16, 20],
    );
}

/// The Fake Celebrity's Inhibit Greatness (12, 17) beside Inhibit Courage's.
#[test]
fn the_fake_celebritys_inhibit_greatness_steps_at_17() {
    // Game: MagicDeceiverWayOfFakeCelebrityProgression fc120e8f61a84694adfd5ff5298be820 grants
    // FakeInspireGreatnessFeature bc4cac39686b44ab8030edc4b60634ce at 12; its step at 17 is not found in the
    // blueprints (unsettled).
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
    // Game: OverwhelmingSoulArchetype aa11888104d17f7459851e8d559ffa98 adds OverwhelmingSoulBurnFeature
    // a3051f965d971ed44b9c6c63bf240b79 at 1, OverwhelmingSoulMentalProwessFeature c07916b1fe7400c4c97472337b719e64 at 6
    // and OverwhelmingSoulMentalProwessAdditionalUseFeature ec41ec200f734c04baea35e27a103fcd at 10, 14 and 18
    // (equivalent model: one feature counted from 1).
    assert_archetype_steps(
        "kineticist",
        "kineticist/overwhelming-soul/mental-prowess",
        &[1, 10, 14, 18],
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
        // Game: ArmoredHulkArchetype 48277b221b6ba6e40affb1e9b49faf36 adds ArmoredHulkResilienceOfSteel
        // ac8f1fcec10a9584c9e40a5cd8100830 at each.
        (
            "barbarian/armored-hulk/resilience-of-steel",
            &[3, 6, 9, 12, 15, 18][..],
        ),
        // Game: InvulnerableRagerArchetype a2ccb759dc6f1f94d9aae8061509bf87 adds InvulnerableRagerExtremeEndurance
        // 427b4a34432389042861b8db4cbe3d99 at each.
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
        // Game: ArchaeologistArchetype 38384e0c1e99c2e42ac6ed70a04aca46 adds ArchaeologistLuckFeature
        // 03bf87dd753cd4f48a47eaf0ea6da9fe at each.
        (
            "bard/archaeologist/archaeologists-luck",
            &[1, 5, 11, 17][..],
        ),
        // Game: ArchaeologistArchetype 38384e0c1e99c2e42ac6ed70a04aca46 adds ArchaelogistDangerSense
        // f3ce82243623f414a94fd83cb22d5528 at each.
        (
            "bard/archaeologist/danger-sense",
            &[3, 6, 9, 12, 15, 18][..],
        ),
        // Game: ChelishDivaArchetype 985e74f9becb4ea492eb27726f5df872 adds DeadlyVibratoFeature
        // 2d301c453db84865b162403d34ad1f17 at 6; BansheesRequiemAreaEffect 141d633de63c477885463d4e836ae1a6's rank
        // rises at 14 and 20 (equivalent model).
        ("bard/chelaxian-diva/deadly-vibrato", &[6, 14, 20][..]),
        // Game: DirgeBardArchetype f128f6cd9c140284caa244da069f250c adds DirgeBardHauntingRefrainFeature
        // d90802b1a74ae004aac439036263825a at 5, its dice level / 5 rising at 10, 15 and 20 (equivalent model).
        ("bard/dirge-bard/haunting-refrain", &[5, 10, 15, 20][..]),
        // Game: FlameDancerArchetype e7914f2adcdb8fc46af5b65d1e06c539 adds FlameDancerPerformanceFeature
        // 3c10a0069e7f110499d2e810f4861a6e at each.
        ("bard/flame-dancer/fire-dance", &[3, 6, 11][..]),
        // Game: ThundercallerArchetype 3c67fb6458752a1419fb8cd4efaf8eaa adds ThunderCall
        // 5ebb5d1f76b602d44818a21e9b6e31b8 at each.
        ("bard/thundercaller/thunder-call", &[3, 7, 11, 15, 19][..]),
        // Game: TranquilWhispererArchetype bed077eaa12834b4495cf481ee721810 adds InspireTranquilityFeature
        // 6ef23b06bafd8f544b783979e01e3505 at each.
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

/// The Chelaxian Diva's Devastating Aria: granted at 2, its ability's rank rising at bard levels 4, 8, 12, 16 and 20.
#[test]
#[ignore = "waits on ticket 286: the game gives Devastating Aria at 2, 4, 8, 12, 16 and 20"]
fn the_chelaxian_divas_devastating_aria_steps_at_2_4_8_12_16_and_20() {
    // Game: "Devastating AriaFeature" f7dcab1cd75140488bacb73b04e29b5c, added by ChelishDivaArchetype
    // 985e74f9becb4ea492eb27726f5df872 at 2; "Devastating AriaAbility" 40e997669e6b4dbaaee8b9a093cfb506's custom rank
    // on bard level is 1 to 3, 2 to 7, 3 to 11, 4 to 15, 5 to 19 and 6 at 20 (equivalent model: the planner counts the
    // rises as grants).
    assert_archetype_steps(
        "bard",
        "bard/chelaxian-diva/devastating-aria",
        &[2, 4, 8, 12, 16, 20],
    );
}

#[test]
fn the_bloodrager_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        // GreenragerFuriousSummoning eabdfba5d63bf164aa7f847eff8ca1af is granted once at 9; its buff's rank is a custom
        // table on the caster level (4 to 9, 6 to 11, 8 to 20), whose bloodrager mapping is unsettled.
        ("bloodrager/greenrager/furious-summoning", &[9, 11, 20][..]),
        // Game: HagRivenArchetype 61edfe08d9e04a11844e5fcc01da3602 adds ScarredHideFeature
        // e05f729b68d14ea0b034b9ee84630ec3 at 7, its custom rank rising at 10, 13, 16 and 19 (equivalent model).
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
        // Game: DiscipleOfThePikeArchetype 4c4c3f9df00a5e04680d172a290111c4 adds DiscipleOfThePikeBiggerTheyAre
        // a42d67b00e0bd4744bf47b3618955b09 at 1, DiscipleOfThePikeBiggerTheyAre6 ab91dc81bf2a77646bd167b45db4fa26 at 6
        // and DiscipleOfThePikeBiggerTheyAre12 811ec31e270db874bb67b52a60ed1a39 at 12 (equivalent model).
        (
            "cavalier/disciple-of-the-pike/bigger-they-are",
            &[1, 6, 12][..],
        ),
        // Game: DiscipleOfThePikeArchetype 4c4c3f9df00a5e04680d172a290111c4 adds
        // DIscipleOfThePikeWeaponTrainingSelection bc050eac79b34d4abf63e2f1152badad at each.
        (
            "cavalier/disciple-of-the-pike/weapon-training-spears-polearms",
            &[5, 9, 13, 17][..],
        ),
        // Game: FearsomeLeaderArchetype 13cc5ccd5cef4c98a5806f033e19a8bb adds FearsomeLeaderFeature
        // d9a592a5eb96484985bd2200b47055f4 at 3, its rank level / 3 rising at 6, 9, 12, 15 and 18 (equivalent model).
        (
            "cavalier/fearsome-leader/fearmonger",
            &[3, 6, 9, 12, 15, 18][..],
        ),
        // StandardBearerArchetype 07db3fec753209546a792a7942288998 adds StandardBearerBannerOfSolaceFeature
        // 859ec9d911cecf5438dc93ca3f88fa5d once, at 11; its buff's ranks step on the caster level (level / 2, and from
        // 7 every 4), whose cavalier mapping is unsettled.
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
        // Game: ElementalRampagerArchetype f815594bd1e3454182022d375bf70fd1 adds ElementalRampagerClaws1d6
        // 1c592a4f288a46889aa70a0eac774c4f at 1, ElementalRampagerClaws1d8 c88eb191d1dd4f5dafc9988822ae24fd at 9 and
        // ElementalRampagerClaws1d10 bd273aafc6104aacb0e0bf2727a2ce4d at 15 (equivalent model).
        ("druid/elemental-rampager/claws", &[1, 9, 15][..]),
        // Game: ElementalRampagerRampageFeature 64c5dfe0ba664dd38b7e914ef0912a1c at 1, its buff's rank rising at 6,
        // 11 and 16 (equivalent model).
        ("druid/elemental-rampager/rampage", &[1, 6, 11, 16][..]),
        // PrimalDruidEnlargeFeature 9b18d052ac2040f9a68f0c36bf390ce4 is granted once at 1, with no step found in the
        // blueprints (unsettled).
        ("druid/primal-druid/primal-size", &[1, 8, 16][..]),
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

/// The Winter Child's Arctic Native: granted at 4, its resistance rising with the character level at 8 and 12.
#[test]
#[ignore = "waits on ticket 277: the game gives Arctic Native at 4, 8 and 12, on character level"]
fn the_winter_childs_arctic_native_steps_at_4_8_and_12() {
    // Game: WinterChildArchetype 9f835c539f6e4a96a7b64461f4267d06 adds ArcticNativeFeature
    // 776190c771e2487886ab1de9dfa8f2f1 at 4; its custom rank on character level is 5 to 7, 10 to 11 and 15 from 12
    // (equivalent model: the planner counts the rises as grants).
    assert_archetype_steps("druid", "druid/winter-child/arctic-native", &[4, 8, 12]);
}

#[test]
fn the_fighter_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        // DefenderOfTheHearthArchetype 6c96dc1f5573487399336cd351ef8f2c adds ShieldBlockFeature
        // 08ef60cbb8324d13a8524ba552fa579e once, at 1, with no step found in the blueprints (unsettled).
        (
            "fighter/defender-of-the-hearth/shield-block",
            &[1, 5, 10, 15, 20][..],
        ),
        // Game: TowerShieldSpecialistArchetype a599da9a8a6b9e54083b0a4d2a25db59 adds BurstBarrier
        // 6c97b3fd5d354454c9f69fea5348a7e8 at each.
        (
            "fighter/tower-shield-specialist/burst-barrier",
            &[2, 6, 10, 14, 18][..],
        ),
        // Game: TwoHandedFighterArchetype 84643e02a764bff4a9c1aba333a53c89 adds TwoHandedFighterStrongGrip
        // ab9d45de98fe56441a4c012f70dad6ee at each.
        (
            "fighter/two-handed-fighter/strong-grip",
            &[2, 6, 10, 14, 18][..],
        ),
        // Game: TwoHandedFighterArchetype 84643e02a764bff4a9c1aba333a53c89 adds TwoHandedFighterWeaponTraining
        // 88da2a5dfc505054f933bb81014e864f at each.
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
    // Each granted once and rising by a rank or a use (equivalent model: the planner counts the rises as grants).
    for (feature, levels) in [
        // Game: ColludingScoundrelArchetype 34963fc6d87354c4fa85eb76f122c41c adds BackstabberFeature
        // f1ca6cf566e473e47bb0f3aee435ded0 at 8, its custom rank rising at 15.
        ("hunter/colluding-scoundrel/backstabber", &[8, 15][..]),
        // Game: ScapegoatFeature cffe3f12c8d00ba4fa15d0c6fcd375c8 at 1; ScapegoatAbilityEnemyBuff
        // 9261f8040c97eab458010defa54cb594's penalty changes at 8 and 15.
        ("hunter/colluding-scoundrel/scapegoat", &[1, 8, 15][..]),
        // Game: ForesterArchetype 74ee8172df6782f4792ea0f8647dea34 adds ForesterTacticianFeature
        // 994db4abfa0d6194eb3c847605e6f148 at 3 and ForesterTacticianExtraUse 6299947e288a4e75996cfe39862249ec at 7, 12
        // and 17.
        ("hunter/forester/tactician", &[3, 7, 12, 17][..]),
        // Game: WanderingMarksmanArchetype 0be343c549776574286730b33ae5c676 adds PlannedDistractionFeature
        // d4300e18cb39a0845934a36ea8366c5f at 3, its custom rank rising at 12 and 18.
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
    // Each granted once and rising by a rank or a use (equivalent model: the planner counts the rises as grants).
    for (feature, levels) in [
        // Game: FaithHunterArchetype ac87f40932cfd324c90b441f93857c40 adds FaithHunterSwornEnemySmiteFeature
        // 3ffb55877d0a7764e8f15c3ce84d3ae3 at 6 and FaithHunterSwornEnemySmiteExtraUse
        // 2fab74210ab73ba47b8bad213becce4f at 10, 14 and 18.
        (
            "inquisitor/faith-hunter/sworn-enemy-smite",
            &[6, 10, 14, 18][..],
        ),
        // Game: LivingGrimoireArchetype b68b52e5e802426d989a3d4f8c986761 adds BlessedScriptFeature
        // 42559d1923644932bea28da3795a0db5 at 5, its rank 1 + (level - 4) / 6 rising at 10 and 16.
        (
            "inquisitor/living-grimoire/blessed-script",
            &[5, 10, 16][..],
        ),
        // Game: TacticalLeaderArchetype 639b74fd2f48d474e965c596b1649095 adds TacticalLeaderFeatShareFeature
        // 93e78cad499b1b54c859a970cbe4f585 at 3; TacticalLeaderFeatShareResource 8b4a056d78c656849a415f7efc1a9d03
        // rises at 6, 9, 12, 15 and 18.
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
    // Game: KineticKnightArchetype 7d61d9b2250260a45b18c5634524a8fb adds KnightsResolve
    // e65a711b3fd6e604b8edd03c08902f7e at 3; KnightsResolveResource f50ac8d0d17ea9a4bbcc7bc39d6cf6c3 rises at 5, 7,
    // ..., 19 (equivalent model).
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
        // Game: BladeboundArchetype 9b3915d38c7d4b6495415b38571e9d53 adds BladeboundBlackBladeFeature
        // 1e7c04a3151043d792d6a5153c84d779 at 1, the blade's rank 1 + (level - 1) / 4 rising at 5, 9, 13 and 17
        // (equivalent model).
        ("magus/bladebound/black-blade", &[1, 5, 9, 13, 17][..]),
        // Game: BladeboundBlackBladeDamageFeature a5f8b3acaa0e4df49a7cb1eb8456d09a at 3;
        // BladeboundBlackBladeDamageBuff 599e7eedaff3466cb40ec3b5af79140f's rank rises at 6, 9, 12, 15 and 18
        // (equivalent model).
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
        // Game: DrunkenMasterArchetype a8532f88b704413ba68f62e4e7ff84c1 adds DrunkenCourageFeature
        // 479dd8ad30eb433b9ad8a7fbaedd2fd9 at 7 and DrunkenCourage11thFeature 246d01faa176445a9aaf49bda965dbc6 at 11
        // (equivalent model).
        ("monk/drunken-master/drunken-courage", &[7, 11][..]),
        // Game: SenseiArchetype f8767821ec805bf479706392fcc3394c adds SenseiMysticPowersFeature
        // d5f7bcde6e7e5ed498f430ebf5c29837 at 6 and SenseiMysticPowersMass a316044187ec61344ba33535f42f6a4d at 10
        // (equivalent model).
        ("monk/sensei/mystic-powers", &[6, 10][..]),
        // Game: SenseiArchetype f8767821ec805bf479706392fcc3394c adds SenseiMysticWisdom
        // 4356b5d6d34489747bba68d43924a857 at 10 and SenseiMysticWisdomMass 1c02ec3a153954b459becc0d40d3cf60 at 14
        // (equivalent model).
        ("monk/sensei/mystic-wisdom", &[10, 14][..]),
        // StudentOfStoneArchetype 3b81e57a75299b74ab6b144e830864e9 adds BonesOfStoneFeature
        // 5d0b1fc437130424da8eb69f6913dadc once, at 7; its buff's rank is a custom table on the caster level (2 to 14,
        // 5 to 20), whose monk mapping is unsettled.
        ("monk/student-of-stone/bones-of-stone", &[7, 10, 15][..]),
        // StudentOfStoneArchetype 3b81e57a75299b74ab6b144e830864e9 adds SoulOfStoneFeature
        // cbfbc3b230813ad4d81433ac9fb40ae7 once, at 12, with no step found in the blueprints (unsettled).
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

/// The Drunken Master's Drunken Ki: granted at 3, its pool monk level / 2, so rising at every even level after.
#[test]
#[ignore = "waits on ticket 279: the game gives Drunken Ki at 3, its pool rising at 4, 6, ..., 20"]
fn the_drunken_masters_drunken_ki_pool_rises_at_every_even_level() {
    // Game: DrunkenMasterArchetype a8532f88b704413ba68f62e4e7ff84c1 adds KiPowerDrunkenFeature
    // 814ddce615b84e488fb7a196acebd408 at 3 only; DrunkenKiPowerResource fd01f3f969a04febab7877a17aebb812 grows by 1
    // per two monk levels (Blueprints/BlueprintAbilityResource.GetMaxAmount, WrathDecompiled 6ce4a6b) (equivalent
    // model: the planner counts the rises as grants).
    assert_gains(
        "monk",
        "monk/drunken-master/drunken-ki",
        &|_, build| build,
        &[],
        &[(
            "monk/drunken-master",
            Some(&[3, 4, 6, 8, 10, 12, 14, 16, 18, 20]),
        )],
    );
}

/// The Drunken Master's Drunken Resilience is granted once, at 9, and its damage reduction counts only its own rank.
#[test]
#[ignore = "waits on ticket 279: the game gives Drunken Resilience once, at 9"]
fn the_drunken_masters_drunken_resilience_is_granted_once_at_9() {
    // Game: DrunkenMasterArchetype a8532f88b704413ba68f62e4e7ff84c1 adds DrunkenResilienceFeature
    // 4db82a5a352543b3842f63e2583a85f1 at 9 only; it adds DamageReductionDrunkenMaster
    // 187ddd9028a1413fa256fdc9f46bf7d5, whose rank counts its own ranks, and nothing else grants it.
    assert_archetype_steps("monk", "monk/drunken-master/drunken-resilience", &[9]);
}

/// The Sohei's Ki Weapon: granted at 4, its bonus rising at monk levels 5, 9, 13 and 17.
#[test]
#[ignore = "waits on ticket 279: the game gives Ki Weapon at 4, 5, 9, 13 and 17"]
fn the_soheis_ki_weapon_steps_at_4_5_9_13_and_17() {
    // Game: SoheiArchetype fad7c56737ed12e42aacc330acc86428 adds SoheiKiWeaponFeature
    // 52b62805b9925ca498ae886cd1a41ffd at 4; SoheiKiWeaponBuff 093777ec5327c8449a09941f293b4692's custom rank on monk
    // level is 1 at 4, 2 to 8, 3 to 12, 4 to 16 and 5 to 20 (equivalent model: the planner counts the rises as grants).
    assert_archetype_steps("monk", "monk/sohei/ki-weapon", &[4, 5, 9, 13, 17]);
}

#[test]
fn the_oracle_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        // Game: EnlightenedPhilosopherArchetype 4f2698f04fa63e34b8153fa3a9795649 adds
        // EnlightenedPhilosopherRevelationMentalAcuity 736d3d5d940736b4e87792e94eaff96b at each.
        (
            "oracle/enlightened-philosopher/mental-acuity",
            &[7, 10, 13, 16, 19][..],
        ),
        // Game: HermitArchetype 8f205adfab73b3c4c8c93d251a3cef02 adds HermitRevelationFadeFromMemoryBase
        // 996c51938bc82f1408d868ddb7bf2a4f at 7 and HermitRevelationFadeFromMemory14 7bb526fcbbc826048a2a65c1b60fa36d
        // at 14 (equivalent model).
        ("oracle/lone-strider/fade-from-memory", &[7, 14][..]),
        // Game: HermitArchetype 8f205adfab73b3c4c8c93d251a3cef02 adds HermitRevelationReclusesStride
        // 1c50d942a6017df47bee95ea169d638f at 1, whose AddFeatureOnClassLevel steps at oracle 5 and 10 (equivalent
        // model).
        ("oracle/lone-strider/recluses-stride", &[1, 5, 10][..]),
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

/// The Wind Whisperer's Haste of the Wind: granted at 7, one use more at oracle 12 and 18.
#[test]
#[ignore = "waits on ticket 272: the game gives Haste of the Wind at 7, 12 and 18"]
fn the_wind_whisperers_haste_of_the_wind_steps_at_7_12_and_18() {
    // Game: WIndWhispererArchetype 77769e43e63647c2b32298efcc577aca adds WindWhispererHasteFeature
    // 83bb307b3ee74090b7b86c1a8c0a7612 at 7; WindWhispererHasteResource 1e222665df1f471f9e2e461c3cde67ad is 1, plus 1
    // from oracle 6 and 1 more every six levels (equivalent model: the planner counts the rises as grants).
    assert_archetype_steps(
        "oracle",
        "oracle/wind-whisperer/haste-of-the-wind",
        &[7, 12, 18],
    );
}

#[test]
fn the_paladin_archetypes_step_the_features_the_guide_gives_steps_to() {
    // The Hospitaler's smite is the class's own in the game:
    // `the_hospitaler_holds_the_paladins_own_smite_evil_at_1_7_13_and_19`.
    for (feature, levels) in [
        // Game: StonelordArchetype cf0f99b3cd7444a48681b1c1c4aa2a41 adds StonebloodFeature
        // 33a6791617cd483b9d5794d40bec6c7d at 3, StonebloodFeature2 ea9a6c2f9edb484e9d960c514dc74236 at 9 and
        // StonebloodFeature3 684ca1b38c4a437aa6f903cee0f838aa at 15 (equivalent model).
        ("paladin/stonelord/stoneblood", &[3, 9, 15][..]),
        // Game: TorturedCrusader f91ea9596e6f42f8a23803c1b3e976b4 adds SecondChanceProgression
        // 2936d727551c4e8796dc18f717e9bf07, whose entries are at 2, 8 and 15 (equivalent model).
        ("paladin/tortured-crusader/second-chance", &[2, 8, 15][..]),
        // Game: WarriorOfTheHolyLightArchetype cc70a9fdcd8781a4ca2f0e594f066964 adds a Power of Faith tier feature at
        // each (PowerOfFaithTier1Feature 6a585dce1d94c9b4c8cf95dede6b0568 first) (equivalent model).
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
    // FlamewardenWeaponFeature 0718a2218ce5b3745bee20d8a7846946 and StormwalkerWeaponFeature
    // 8686a5e28ae18f7429f37ee6792a63ee are each granted once, at 4; their abilities' ranks follow ranger level as it
    // is, so where the step at 12 comes from is unsettled.
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
        // Game: KnifeMasterArchetype 9e94d1847e6f331478e5a714659220ce adds KnifeMasterBladeSense
        // 112bf4c6943097942b24eadfa750215f at each.
        ("rogue/knife-master/blade-sense", &[3, 6, 9, 12, 15, 18][..]),
        // Game: SylvanTricksterArchetype 490394869f666c141bf8647b1a365220 adds FeyResistanceFeature
        // ae0ef197e1904464e9afe115334d4bca at each.
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
        // ShadowShamanArchetype 5086aef2b76c433584160f7b49aef95b adds ShadowShamanShadowsFeature
        // b38e129e760940378d369a79a5c81b30 once, at 4; its ability's rank follows shaman level as it is, so where the
        // steps at 9 and 14 come from is unsettled.
        ("shaman/shadow-shaman/shadows", &[4, 9, 14][..]),
        // Game: SpiritHunterArchetype 39b2fd5000e5eb142bfde2afa6c83f76 adds ShamanWeaponPoolFeature
        // ef7e19661304e124f95c49637f931429 at 1 and ShamanWeaponPlus2 ecdee76fb34d45f40988d06fe524129e to
        // ShamanWeaponPlus5 17527cfc59388bc4fbb959c5408db459 at 5, 9, 13 and 17 (equivalent model).
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
        // Game: ChildOfManticoreShifterArchetype 1a51b5856d3b48d78b3e967dc5f20bd6 adds ShifterAspectManticore
        // 50b0bb6a4c824a06adc5e56c079a0a1d at 1, a level-up feature at 5, ShifterAspectManticoreBuff
        // 5a21b25be7bb43e0b94646511afadfc6's dice rising at 10, 15 and 20 (equivalent model).
        (
            "shifter/child-of-the-manticore/manticore-aspect",
            &[1, 5, 10, 15, 20][..],
        ),
        // Game: GriffonheartShifterArchetype aed5b306ad734a6da5d5638edcb667c9; ShifterWildShapeGriffonFeatureLevelUp
        // 84e9b89787854c37b2412d1a71229e53 switches at 9, ShifterWildShapeGriffonFeatureLevelUp1
        // 1f99fbb3b1b445d6bfbaf3588af171cf at 14 (equivalent model).
        ("shifter/griffonheart-shifter/griffon-form", &[4, 9, 14][..]),
        // Game: WildEffigyShifterArchetype c42d338616fa46ab9bcc67c94bca7609 adds HeartOfEarthShifterFeature
        // 0b09ce94b5fe48dfa81b9a6f9d55a351 at 4; HeartOfEarthShifterBuff c3ad4aca75764e82ab047533a8d5aea6 switches at
        // 9 and 15 (equivalent model).
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

/// The Child of the Manticore's Manticore Shape: granted at 4, its form switching at shifter 8 and 14.
#[test]
#[ignore = "waits on ticket 277: the game gives Manticore Shape at 4, 8 and 14"]
fn the_child_of_the_manticores_manticore_shape_steps_at_4_8_and_14() {
    // Game: ChildOfManticoreShifterArchetype 1a51b5856d3b48d78b3e967dc5f20bd6 adds
    // ShifterWildShapeManticoreFeatureLevelUp 719be33c87f94ed58414ba3eb5a4b664 at 4, whose AddFeatureOnClassLevel
    // switches to ShifterWildShapeManticoreFeatureLevelUp1 4f9c07f21ec742b6882d3d1453a9c2df at 8, which switches at 14
    // (equivalent model: the planner counts the switches as grants).
    assert_archetype_steps(
        "shifter",
        "shifter/child-of-the-manticore/manticore-shape",
        &[4, 8, 14],
    );
}

#[test]
fn the_skald_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        // Game: CourtPoetArchetype 5e63586bebd229649bdafe0fde4caaec adds InsightfulContemplationSongFeature
        // 7549a18f454afe24c8f3b3ec7efffee2 at 1 and CourtPoetInsightfulDamageFeature 04da9870c35d48efb109838cb2bb8d82
        // at 4, 8, 12, 16 and 20 (equivalent model).
        (
            "skald/court-poet/insightful-contemplation",
            &[1, 4, 8, 12, 16, 20][..],
        ),
        // ProvocateurArchetype a1f5ab7eb78942f6889304acbd92e945 adds CausticRidiculeFeature
        // 47619c4de860458186b9e5b968a4f1c1 once, at 10, and RagingSongSneakAttackFeature
        // 1ae6618b3eb2450b92d6ae6e0aa64c03 once, at 8; neither has a component that steps (unsettled).
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
    // Each a feature of its own per step in the game (equivalent model: one feature stepped).
    for (feature, levels) in [
        // Game: ExecutionerArchetype 8040e66d2724c754f85dd3026a1f0cee adds ExecutionerAssassinateFeature
        // bd7e70e98f9036f4ba27ef3e29a177c2 at 10, ExecutionerAssassinate14 c81f15dbbadbb3a48b37e95a5a7ef759 at 14 and
        // ExecutionerAssassinate19 a10abd69551134341a6f771d23535165 at 19.
        ("slayer/executioner/assassinate", &[10, 14, 19][..]),
        // Game: StygianSlayerArchetype 598f461cd7885304e8bae040122cb249 adds StygianSlayerInvisibilityFeature
        // c38720e9140d5b34f8f22b25c23168a9 at 4 and StygianSlayerInvisibilityExtraUse
        // e8f9da8ae21a2f9488d81ee59698229d at 8, 12, 16 and 20.
        ("slayer/stygian-slayer/invisibiliy", &[4, 8, 12, 16, 20][..]),
        // Game: SpawnSlayerArchetype f6e304de008939d46960a2ebb075e150 adds StudiedSpawnLargeSizeBonus
        // c2e2b3b4bd9601c43b8891957e830534 at 5, StudiedSpawnHugeSizeBonus bd9f8b8e2217ab74ab5d6a81eec34691 at 10,
        // StudiedSpawnGargantuanSizeBonus 04c9063177cce8044bb7997252a6d29f at 15 and StudiedSpawnColossalSizeBonus
        // 596bf484e187cfd4b9f0a22d1727dbf3 at 20.
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
    // Each granted once and rising by a rank or a use (equivalent model: the planner counts the rises as grants).
    for (feature, levels) in [
        // Game: ChampionOfTheFaithArchetype ad56b5819c4072345bff9aff6f1df13c adds ChampionOfTheFaithSmiteFeature
        // 24fc98788181e6e4880041bee2b66938 at 4, its uses rising at 8, 12, 16 and 20.
        (
            "warpriest/champion-of-the-faith/smite",
            &[4, 8, 12, 16, 20][..],
        ),
        // Game: DisenchanterArchetype 384a9d0c3f1b0634ba183b77d75618c0 adds DisenchanterMysticInterferenceFeature
        // eb796b57b9ad58a4d8b3c386d530c429 at 4, its buff's rank level / 4 rising at 8, 12, 16 and 20.
        (
            "warpriest/disenchanter/mystic-interference",
            &[4, 8, 12, 16, 20][..],
        ),
        // Game: MantisZealotArchetype fc403240fdca4a52a413578169ea7117 adds MantisZealotMantisSwarmFeature
        // aba4b42c3cad4bcdaaf79b1a52778591 at 4, its rank 1 + (level - 4) / 7 rising at 11 and 18.
        ("warpriest/mantis-zealot/mantis-swarm", &[4, 11, 18][..]),
        // Game: ProclaimerArchetype aaf7ace9642192d4389eab95e6cd04fd adds ProclaimerCleanserOfEvil5FeetFeatureLimit
        // e23f6e72498cb224b939ee5ea2c5d4e4 at 2, the 10-foot f236ddd506258794c9f1a18d0a1548d9 at 7, the 15-foot
        // f0f07c512fa3bc44f8593154bf6b2a11 at 13 and the 20-foot 0bb21b9818eac054a979be0fa82aade4 at 19, its damage
        // rank rising at 5, 8, 11, 14, 17 and 20: the union of both series.
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

/// The Mantis Zealot's Fading: granted at 7, its miss chance rising at 10, 13 and 16 and no more after.
#[test]
#[ignore = "waits on ticket 268: the game gives Fading at 7, 10, 13 and 16"]
fn the_mantis_zealots_fading_steps_at_7_10_13_and_16() {
    // Game: MantisZealotArchetype fc403240fdca4a52a413578169ea7117 adds MantisZealotFadingFeature
    // 724a017c96d643729f65d2d1eac54673 at 7; MantisZealotFadingBuff 5fda36eeec94433cbfda5571c42888a2's custom rank on
    // warpriest level is 20 to 9, 30 to 12, 40 to 15 and 50 from 16 (equivalent model: the planner counts the rises as
    // grants).
    assert_archetype_steps(
        "warpriest",
        "warpriest/mantis-zealot/fading",
        &[7, 10, 13, 16],
    );
}

#[test]
fn the_witch_archetypes_step_the_features_the_guide_gives_steps_to() {
    for (feature, levels) in [
        // Game: DarkSisterArchetype 411fa458481e44f0855d47a19358874b adds SunderHopeFeature
        // be778d85e96d4c21860a301f4153ba3f at 8 and SunderHopeIncreaseResourceFeature e45eab72dba5406db9f4ea3cde8b1a4a
        // at 14 and 20 (equivalent model).
        ("witch/hag-of-gyronna/sunder-hope", &[8, 14, 20][..]),
        // Game: HagboundWitchArchetype 96e4c06b6add0a64a8ad8976f9c38c37 adds HagboundWitchHunchedMusclesFeature
        // 53a90334e15796d48a45a3d22cdb147b at each.
        ("witch/hagbound/hunched-muscle", &[2, 8, 14][..]),
        // Game: WitchOfTheVeilArchetype 08045a390a764d71b2150fd122e05e19 adds WitchOfTheVeilShroudedStepFeature
        // 855a1e132cfa444eb8eed4c784ee8ec9 at each.
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
        // Game: ShadowCasterArchetype 9e1211f2c3db4d0c8313013df72d7187 adds ShadowySpecialisationProgression
        // 28222114243e496bafa3b4b9f83d9036 at 10, whose entries are at 10, 12, 14, 16 and 18 (equivalent model).
        ("wizard/shadowcaster/umbral-mind", &[10, 12, 14, 16, 18][..]),
        // Game: SpellMasterArchetype bb571f1b6cea462e99db7893685a0982 adds SpellMasterFocusedSpellsFeature
        // ee18ad78a7b046e789a763ab860cd50a at 1; SpellMasterFocusedSpellsResource 32947324b3054fdea55092735de9c82c
        // rises at 8 and 16 (equivalent model).
        ("wizard/spell-master/focused-spells", &[1, 8, 16][..]),
        // Game: SpellMasterArchetype bb571f1b6cea462e99db7893685a0982 adds SpellMasterItemBondSpecializationFeature
        // 88fe74c940ce41a0ac62b9b22774be3f at each.
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
    let cases: [GrantedOnce; 7] = [
        // Game: PhantasmalMageArchetype e9d0ee69305049fe8400a066010dbcd1 adds ArcaneMetamagicFeature
        // 842528c733db422a915b7b06e9a71178 at 1, ArcaneMetamagicFeature5 9905e8d1afa644afaa7d61ba979b7dec at 5,
        // ArcaneMetamagicFeature11 402d0054da7e45dcb6b554ed1db6bd1f at 11 and ArcaneMetamagicFeature17
        // bbb28051942243cf81dc90ec40b646ba at 17.
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
        // Game: BeastTamerArchetype 912b0495d9485124bb2c0e4aa6ac2efd adds BeastCallSummonFeature1
        // fb6000f054cec7847a6e793069e90040 at 1, BeastCallSummonFeature2 ef9409a4908f30a4392832af62f9a64d at 3 and
        // BeastCallSummonAddition9 0e38ad0c8b28e914eaea5327ebad9315 at 17.
        (
            "bard",
            "bard/beast-tamer/beast-call",
            1,
            &[
                ("bard/beast-tamer/summon-natures-ally-ii", &[3]),
                ("bard/beast-tamer/summon-natures-ally-ix", &[17]),
            ],
        ),
        // Game: HagRivenArchetype 61edfe08d9e04a11844e5fcc01da3602 adds HagRivenClawsFeatureAddLevel
        // e0065edfef8e46e3a175d5c9694b8ff6 at 1, HagRivenClawsFeatureAddLevel1 dc5f3f8886084f5e948e00503e7f1b73 at 2,
        // HagRivenClawsFeatureAddLevel2 f349d26212084f4980a731c2e04bd811 at 5, HagRivenClawsFeatureAddLevel3
        // dcbf2ee286c540cebfba87671c108d80 at 9, HagRivenClawsFeatureAddLevel4 3e54e4f0db1947cea29b1d6a2f8e5a5c at 13
        // and HagRivenClawsFeatureAddLevel5 cecfbb4d624e4afcaf19ee0761426a68 at 17.
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
        // Game: WinterChildArchetype 9f835c539f6e4a96a7b64461f4267d06 adds CallingForAFeastFeature
        // bfe548bd90d4476c8afde9a0667384cd at 1, CallingForAFeastImprovedFeature 4a64c3265d5e490eadb33c4f33382992 at
        // 10 and CallingForAFeastGreaterFeature a78be8dba1274689b8a97b08a3069087 at 15.
        (
            "druid",
            "druid/winter-child/call-a-feast",
            1,
            &[
                ("druid/winter-child/call-a-feast-cover-of-snow", &[10]),
                ("druid/winter-child/call-a-feast-biting-frost", &[15]),
            ],
        ),
        // Game: ArmoredBattlemageArchetype 67ec8dcae6fb3d3439e5ae874ddc7b9b adds ArmoredBattlemageArcanePoolFeature
        // 466c40aba50096341bf6532b1e53e8bd at 1, ArcaneArmorEnchantPlus2 9157c414fd5f3b346aaa67474a40dd16 at 5 and
        // ArcaneArmorEnchantPlus5 08a480ca96c7a834f86813f959a69fa9 at 17.
        (
            "magus",
            "magus/armored-battlemage/arcane-pool-armored-battlemage",
            1,
            &[
                ("magus/armored-battlemage/arcane-armor-2", &[5]),
                ("magus/armored-battlemage/arcane-armor-5", &[17]),
            ],
        ),
        // Game: SpellDancerArchetype 1125145639129cf45b6b9b674cbd62b1 adds SpellDanceFeature
        // 330d055badafd5e45a5f5624dd005756 at 1, SpellDanceBlurFeature 959467aaad7d8564babafd05f41a3973 at 5,
        // SpellDanceDoorFeature 7a3879b1f34ede3469fd32de34eb4c3d at 9 and SpellDanceFreedomFeature
        // 586977ef129c6af4a9b94dc05c4e91bf at 13 (equivalent model).
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
    // The Purifier's Celestial Armor gives heavy armour proficiency at 11.
    let purifier = playable(
        || archetype_build("oracle", "oracle/purifier", 20),
        |build| build,
    )
    .unwrap();
    // Game: PurifierCelestialArmorFeature 7dc8d7dede2704640956f7bc4102760a's AddFeatureOnClassLevel adds
    // HeavyArmorProficiency 1b0f68188dcc435429fb87a022239681 from oracle 11.
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

/// The Purifier's Celestial Armor is granted again at 11, 15 and 19, each grant a rank of its armour bonuses.
#[test]
#[ignore = "waits on ticket 272: the game gives Celestial Armor at 7, 11, 15 and 19"]
fn the_purifiers_celestial_armor_steps_at_7_11_15_and_19() {
    // Game: PurifierAasimarArchetype c9df67160a77ecd4a97928f2455545d7 adds PurifierCelestialArmorFeature
    // 7dc8d7dede2704640956f7bc4102760a (Ranks 4, its maximum Dexterity and armour check penalty per rank) at 7, 11, 15
    // and 19.
    assert_archetype_steps(
        "oracle",
        "oracle/purifier/celestial-armor",
        &[7, 11, 15, 19],
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
    // Game: InvulnerableRagerArchetype a2ccb759dc6f1f94d9aae8061509bf87 adds InvulnerableRagerDamageReduction
    // e71bd204a2579b1438ebdfbf75aeefae at every even level.
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
/// delayed 10 and then the class's, the Flesheater and the Instinctual Warrior none (the Pack Rager's keeps the class's
/// at 10 in the game: `the_pack_rager_keeps_the_barbarians_damage_reduction_at_10`).
#[test]
fn the_barbarians_damage_reduction_is_not_the_invulnerable_ragers() {
    // Game: BarbarianProgression 74ad9fbd930a1d84aaf9a230f29f8f12 grants DamageReduction
    // cffb5cddefab30140ac133699d52a8f8 at 7, 10, 13, 16 and 19; the Invulnerable Rager, the Flesheater and the
    // Instinctual Warrior remove it at each; MadDogArchetype 063da4c77fa59454a9828c9ece407ea1 adds
    // MadDogDamageReductionMaster a0d4a3295224b8f4387464a4447c31d5 at 10, 13, 16 and 19 (equivalent model).
    assert_gains(
        "barbarian",
        "barbarian/damage-reduction",
        &|_, build| build,
        &[7, 10, 13, 16, 19],
        &[
            (INVULNERABLE_RAGER, Some(&[])),
            ("barbarian/flesheater", Some(&[])),
            ("barbarian/instinctual-warrior", Some(&[])),
            // Ticket 269: the_pack_rager_keeps_the_barbarians_damage_reduction_at_10.
            ("barbarian/pack-rager", Some(WAITING)),
            ("barbarian/mad-dog", Some(&[10, 13, 16, 19])),
        ],
    );
}

/// The Pack Rager trades the barbarian's Damage Reduction at 7, 13, 16 and 19 but not at 10.
#[test]
#[ignore = "waits on ticket 269: the game gives the Pack Rager the barbarian's Damage Reduction at 10"]
fn the_pack_rager_keeps_the_barbarians_damage_reduction_at_10() {
    // Game: PackRagerArchetype 3f0a548044dbad04b82535237584772b removes DamageReduction
    // cffb5cddefab30140ac133699d52a8f8 at 7, 13, 16 and 19 only.
    assert_eq!(
        gains(
            &archetype_20("barbarian", "barbarian/pack-rager"),
            "barbarian/damage-reduction"
        ),
        [10]
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
    // Game: InvulnerableRagerArchetype a2ccb759dc6f1f94d9aae8061509bf87 removes no RagePowerSelection
    // 28710502f46848d48b3f0d6132817c4e, which BarbarianProgression 74ad9fbd930a1d84aaf9a230f29f8f12 grants at every
    // even level.
    assert_eq!(
        levels_with_slot_of(&build, "barbarian/rage-power"),
        [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
    );
}

/// How many slots (feat and selection) each level of `build` offers.
fn slot_counts(build: &Build) -> Vec<usize> {
    engine()
        .evaluate(build)
        .levels
        .iter()
        .map(|level| level.slots.len())
        .collect()
}

/// A possible level 20 Build of `archetype` of `class_key`.
fn archetype_20(class_key: &str, archetype: &str) -> Build {
    playable(|| archetype_build(class_key, archetype, 20), |build| build)
        .unwrap_or_else(|| panic!("{archetype}: no possible Build"))
}

/// Ticket 188: the base classes' stepped features. The barbarian's Danger Sense (the guide: +1 every 3 levels, +6 at
/// 18) is granted at 3, 6, 9, 12, 15 and 18. The Armored Hulk, the Flesheater and the Invulnerable Rager lose it
/// (Resilience of Steel, nothing, Extreme Endurance), so none of them holds a step; every archetype that keeps it
/// gets it on its own rows at the steps (the Beastskin Berserker's 12, the Instinctual Warrior's 6, 12 and 18, the
/// Mad Dog's and the Pack Rager's 6 and 18).
#[test]
fn the_barbarians_danger_sense_steps_every_three_levels() {
    // Game: BarbarianProgression 74ad9fbd930a1d84aaf9a230f29f8f12 grants DangerSenseBarbarian
    // fdd591c1fbf1c0b41a359d59756f2888 at 3, 6, 9, 12, 15 and 18; ArmoredHulkArchetype
    // 48277b221b6ba6e40affb1e9b49faf36, FleshEaterBarbarianArchetype 85c9d72cb20040a79542289dd5126694 and
    // InvulnerableRagerArchetype a2ccb759dc6f1f94d9aae8061509bf87 remove it at each.
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
    // Game: RogueProgression b57b2a75a5abcaf47a01cf84672b50e9 grants DangerSenseRogue 0bcbe9e450b0e7b428f08f66c53c5136
    // at 3, 6, 9, 12, 15 and 18; KnifeMasterArchetype 9e94d1847e6f331478e5a714659220ce and ThugArchetype
    // b5f0b25e1519d27479018353b51fd349 remove it at each.
    assert_gains(
        "rogue",
        "rogue/danger-sense",
        &|_, build| build,
        &[3, 6, 9, 12, 15, 18],
        &[("rogue/knife-master", Some(&[])), ("rogue/thug", Some(&[]))],
    );
    // The Thug's rows at the steps keep the class's rogue talents and Sneak Attack.
    // Game: ThugArchetype b5f0b25e1519d27479018353b51fd349 removes no sneak attack or talent selection.
    let (thug, rogue) = (archetype_20("rogue", "rogue/thug"), build("rogue", 20));
    let levels = [6, 9, 12, 15, 18];
    assert_eq!(
        counts(&thug, "sneak-attack", &levels),
        counts(&rogue, "sneak-attack", &levels)
    );
    assert_eq!(
        levels_with_slot_of(&thug, "rogue/rogue-talents"),
        levels_with_slot_of(&rogue, "rogue/rogue-talents")
    );
}

/// Ticket 188: the fighter's Bravery (+1 every four levels, +5 at 18), Armor Training (every four levels from 3, to
/// 15) and Weapon Training (every four levels from 5, to 17). An archetype whose table has no Bravery (the Armiger's
/// Ardent, the Dragonheir Scion's Fearful Might, the Tower Shield Specialist's Burst Barrier, the Two-Handed
/// Fighter's Strong Grip), no Armor Training (the Aldori Defender, the Dragonheir Scion, the Mutation Warrior, the
/// Titan Fighter, the Two-Handed Fighter) or no Weapon Training (the Titan Fighter, ticket 131; the Tower Shield
/// Specialist; the Two-Handed Fighter) holds none of its steps. A count of Weapon Training stands for the weapon
/// groups (a new one every four levels, four at 17). The Defender of the Hearth pays one for its abilities (the
/// fighter page's overview), so it has three: its first at 9 (its table), then 13 and 17 with the class's rows (an
/// inference: the guide gives it no steps). The table's note, two ranks in the same weapon type at 9, is how strong
/// that one group is, not a second grant, and is not modelled.
#[test]
fn the_fighters_bravery_armor_training_and_weapon_training_step() {
    // Game: FighterProgression b50e94b57be32f74892f381ae2a8905a grants Bravery f6388946f9f472f4585591b80e9f2452 at 2,
    // 6, 10, 14 and 18, ArmorTraining 3c380607706f209499d951b29d3c44f3 at 3, 7, 11 and 15 and WeaponTrainingSelection
    // b8cecf4e5e464ad41b79d5b42b76b399 at 5, 9, 13 and 17; the archetypes below remove them as listed (the Aldori
    // Defender is SwordlordArchetype d80a67a264f206e4b8d2fcf7e560d48f; DefenderOfTheHearthArchetype
    // 6c96dc1f5573487399336cd351ef8f2c removes only the selection at 5).
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
        "armor-training",
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
        // Game: ArmigerArchetype fdee5f25a5d0fe0448d093b462435c0c adds ArmigerArdentFeature
        // fcffd9b743a98784fb9e293c22ee6aff at each.
        ("fighter/armiger/ardent", &[2, 6, 10, 14, 18][..]),
        // Game: SwordlordArchetype d80a67a264f206e4b8d2fcf7e560d48f adds SwordlordDefensiveParryFeature
        // faa56353e193e5940a305c60b4412ea5 at each.
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

/// Ticket 188: the cavalier's Banner (+1 at 10 and every five levels after, to 20), Tactician (an extra use at 5 and
/// every five levels after, to 20) and Challenge (an extra use every three levels beyond 1st, to 19). The Disciple of
/// the Pike and the Ghost Rider lose the Banner (the Standard Bearer moves it to 1, below); the Fearsome Leader, the
/// Gendarme, the Ghost Rider and the Knight of the Wall lose Tactician with all their teamwork abilities. Each
/// holds none of a step it lost, on rows of its own where it has none, which keep what the class's rows give there
/// (the Supreme Charge at 20).
#[test]
fn the_cavaliers_banner_tactician_and_challenge_step() {
    // Game: CavalierProgression aa70326bdaa7015438df585cf2ab93b9 grants CavalierBanner 2d957edad0adb3d49991cfcd3ac4cbf8
    // at 5, CavalierBannerBuff 4d3b79e464282af4897c1d860bf9e9b3's rank rising at 10, 15 and 20 (equivalent model); the
    // Disciple of the Pike and the Ghost Rider remove it.
    assert_gains(
        "cavalier",
        "banner",
        &|_, build| build,
        &[5, 10, 15, 20],
        &[
            ("cavalier/disciple-of-the-pike", Some(&[])),
            ("cavalier/ghost-rider", Some(&[])),
            // Ticket 273: the_standard_bearer_holds_the_cavaliers_own_banner_from_1.
            ("cavalier/standard-bearer", Some(WAITING)),
        ],
    );
    // Game: CavalierTacticianFeature 404f6e3da48b87f4e9fca21150d47f71 at 1, CavalierTacticianResource
    // 7a5fd780d1a866444ad893382d41ec22 rising at 5, 10, 15 and 20 (equivalent model); FearsomeLeaderArchetype
    // 13cc5ccd5cef4c98a5806f033e19a8bb, GendarmeArchetype 48fd69254e9e5c540bd65101d3252184, GhostRiderArchetype
    // c0503d9864b94134ab1237a2bf2d38a0 and KnightOfTheWallArchetype 112dd0e61f95c3a459ac0a565472e685 remove it.
    assert_gains(
        "cavalier",
        "cavalier/tactician",
        &|_, build| build,
        &[1, 5, 10, 15, 20],
        &[
            ("cavalier/fearsome-leader", Some(&[])),
            ("cavalier/gendarme", Some(&[])),
            ("cavalier/ghost-rider", Some(&[])),
            ("cavalier/knight-of-the-wall", Some(&[])),
        ],
    );
    // Game: CavalierChallengeFeature dc77cd2ad52cb0e43bb88b264d7af648 at 1, CavalierChallengeResource
    // 672e8c9c98db1df4aa66676a66036e71 rising every three levels to 19; no archetype removes it (equivalent model).
    assert_gains(
        "cavalier",
        "cavalier/challenge",
        &|_, build| build,
        &[1, 4, 7, 10, 13, 16, 19],
        &[],
    );
    // The rows the archetypes that keep the Supreme Charge take at 20 keep it.
    // Game: none of the three removes CavalierSupremeCharge 77af3c58e71118d4481c50694bd99e77.
    for archetype in [
        "cavalier/fearsome-leader",
        "cavalier/ghost-rider",
        "cavalier/knight-of-the-wall",
    ] {
        assert!(
            holds(
                &at(&archetype_20("cavalier", archetype), 20),
                "cavalier/supreme-charge"
            ),
            "{archetype}"
        );
    }
}

/// The Standard Bearer holds the cavalier's own Banner from 1, its ranks rising with the cavalier level as the
/// class's do: the damage dice at 5, the bonus at 10, 15 and 20.
#[test]
#[ignore = "waits on ticket 273: the game gives the Standard Bearer the cavalier's own Banner at 1, 5, 10, 15 \
            and 20"]
fn the_standard_bearer_holds_the_cavaliers_own_banner_from_1() {
    // Game: StandardBearerArchetype 07db3fec753209546a792a7942288998 removes CavalierBanner
    // 2d957edad0adb3d49991cfcd3ac4cbf8 at 5 and adds it at 1; CavalierBannerBuff 4d3b79e464282af4897c1d860bf9e9b3's
    // damage dice are 1 + level / 5 and its bonus level / 5 (equivalent model: the planner counts the rises as grants).
    let bearer = archetype_20("cavalier", "cavalier/standard-bearer");
    assert_eq!(gains(&bearer, "banner"), [1, 5, 10, 15, 20]);
}

/// Ticket 188: the paladin's Smite Evil (the guide: one a day, another at 4 and every three levels after, 7 at 19)
/// at 1, 4, 7, 10, 13, 16 and 19. The Divine Scion, the Martyr, the Stonelord and the Tortured Crusader have none
/// (their tables give none); the Hospitaler keeps some of the class's (the test after this one). The rows they take
/// at the steps keep the class's Channel Positive Energy at 4 (the Divine Scion, the Martyr).
#[test]
fn the_paladins_smite_evil_steps_every_three_levels_after_4() {
    // Game: PaladinProgression fd325cbba872e5f40b618970678db002 grants SmiteEvilFeature
    // 3a6db57fce75b0244a6a5819528ddf26 at 1 and SmiteEvilAdditionalUse 0f5c99ffb9c084545bbbe960b825d137 at 4, 7, 10,
    // 13, 16 and 19; DivineScionArchetype 5ce3a2f8eae64cf2ba83865711e53e27, MartyrArchetype
    // 4d9a864eae278744aafa7ff23f2a6466, StonelordArchetype cf0f99b3cd7444a48681b1c1c4aa2a41 and TorturedCrusader
    // f91ea9596e6f42f8a23803c1b3e976b4 remove them all.
    assert_gains(
        "paladin",
        "smite-evil",
        &|_, build| build,
        &[1, 4, 7, 10, 13, 16, 19],
        &[
            ("paladin/divine-scion", Some(&[])),
            // Ticket 274: the_hospitaler_holds_the_paladins_own_smite_evil_at_1_7_13_and_19.
            ("paladin/hospitaler", Some(WAITING)),
            ("paladin/martyr", Some(&[])),
            ("paladin/stonelord", Some(&[])),
            ("paladin/tortured-crusader", Some(&[])),
        ],
    );
    // Game: neither archetype removes ChannelEnergyPaladinFeature cb6d55dda5ab906459d18a435994a760 at 4.
    for archetype in ["paladin/divine-scion", "paladin/martyr"] {
        assert!(
            holds(
                &at(&archetype_20("paladin", archetype), 4),
                "paladin/channel-positive-energy"
            ),
            "{archetype}"
        );
    }
}

/// The Hospitaler trades only the paladin's smite uses at 4, 10 and 16, so it holds the class's own Smite Evil at 1,
/// 7, 13 and 19.
#[test]
#[ignore = "waits on ticket 274: the game gives the Hospitaler the paladin's own Smite Evil at 1, 7, 13 and 19"]
fn the_hospitaler_holds_the_paladins_own_smite_evil_at_1_7_13_and_19() {
    // Game: HospitalerArchetype fcdb27e1a6cef834ea1d7fab7c6e2b61 removes SmiteEvilAdditionalUse
    // 0f5c99ffb9c084545bbbe960b825d137 at 4, 10 and 16 only, keeping PaladinProgression
    // fd325cbba872e5f40b618970678db002's SmiteEvilFeature 3a6db57fce75b0244a6a5819528ddf26 at 1 and its uses at 7, 13
    // and 19.
    let hospitaler = archetype_20("paladin", "paladin/hospitaler");
    assert_eq!(gains(&hospitaler, "smite-evil"), [1, 7, 13, 19]);
}

/// Ticket 188: the druid's Wild Shape (Wolf) (the guide: once a day, one more use every two levels above 4th, 8 at
/// 18) at 4, 6, 8, ..., 18. The Drovier and the Winter Child have no Wild Shape; the five others hold the class's own
/// in the game (`the_five_druid_archetypes_with_wild_shape_hold_the_classs_own_wolf`).
#[test]
fn the_druids_wild_shape_wolf_steps_every_two_levels() {
    // Game: DruidProgression 01006f2ac8866764fb7af135e73be81c grants WildShapeIWolfFeature
    // 19bb148cb92db224abb431642d10efeb at 4 and WildShapeExtraUse f78260b9a089ccc44b55f0fed08b1752 at 6, 8, ..., 18
    // (equivalent model); DrovierArchetype a6bf603aa4d666f4f86dab0289e2048c and WinterChildArchetype
    // 9f835c539f6e4a96a7b64461f4267d06 remove them all.
    let class = class("druid");
    let holders = [
        "druid/blight-druid",
        "druid/defender-of-the-true-world",
        "druid/elemental-rampager",
        "druid/feyspeaker",
        "druid/primal-druid",
    ];
    assert_gains(
        "druid",
        "wild-shape-wolf",
        &|_, build| build,
        &[4, 6, 8, 10, 12, 14, 16, 18],
        &DATA
            .archetypes_of(&class.key)
            .map(|archetype| {
                let key = archetype.key.as_str();
                (
                    key,
                    Some(if holders.contains(&key) {
                        WAITING
                    } else {
                        &[][..]
                    }),
                )
            })
            .collect::<Vec<_>>(),
    );
}

/// The Blight Druid, the Defender of the True World, the Elemental Rampager and the Primal Druid keep the druid's own
/// Wild Shape (Wolf) and its uses; the Feyspeaker takes the class's own two levels later.
#[test]
#[ignore = "waits on ticket 277: the game gives these five the druid's own Wild Shape (Wolf), the Feyspeaker's \
            from 6"]
fn the_five_druid_archetypes_with_wild_shape_hold_the_classs_own_wolf() {
    // Game: BlightDruid 35a3b7bfc663ac74aa8bb50adfe70813, DefenderOfTheTrueWorld 782c46afacd88d448afba6178471a744,
    // ElementalRampagerArchetype f815594bd1e3454182022d375bf70fd1 and PrimalDruidArchetype
    // c1c86e2997fd4257a13ef5601b5dc6dd remove neither WildShapeIWolfFeature 19bb148cb92db224abb431642d10efeb nor
    // WildShapeExtraUse f78260b9a089ccc44b55f0fed08b1752; FeyspeakerArchetype da69747aa3dd0044ebff5f3d701cdde3 removes
    // the wolf at 4 and the extra use at 6 and adds the wolf at 6.
    for (archetype, levels) in [
        ("druid/blight-druid", &[4, 6, 8, 10, 12, 14, 16, 18][..]),
        (
            "druid/defender-of-the-true-world",
            &[4, 6, 8, 10, 12, 14, 16, 18][..],
        ),
        (
            "druid/elemental-rampager",
            &[4, 6, 8, 10, 12, 14, 16, 18][..],
        ),
        ("druid/feyspeaker", &[6, 8, 10, 12, 14, 16, 18][..]),
        ("druid/primal-druid", &[4, 6, 8, 10, 12, 14, 16, 18][..]),
    ] {
        assert_eq!(
            gains(&archetype_20("druid", archetype), "wild-shape-wolf"),
            levels,
            "{archetype}"
        );
    }
}

/// Ticket 188: the warpriest's Channel Energy (the guide: 1d6 plus 1d6 per three levels above 2nd, 7d6 at 20) at 4, 5,
/// 8, 11, 14, 17 and 20, and Fervor (1d6 plus 1d6 per three more levels, 7d6 at 20) at 2, 5, 8, 11, 14, 17 and 20.
/// Only the Feral Champion keeps the class's Channel Energy: the Champion of the Faith, the Cult Leader, the
/// Disenchanter, the Mantis Zealot and the Proclaimer replace it, and the Shieldbearer has its own, which takes the
/// same steps (an inference: its sentence gives none). Every archetype but the Proclaimer keeps Fervor whole.
#[test]
fn the_warpriests_channel_energy_and_fervor_step() {
    // Game: WarpriestProgression 062b215705430ad47b7174c1e75d285e grants WarpriestChannelEnergySelection
    // 1a871294410971745bba12ef11e41f6e at 4; ChampionOfTheFaithArchetype ad56b5819c4072345bff9aff6f1df13c,
    // CultLeaderArchetype cd2d419961bb019438b77d4aecc2fc04, DisenchanterArchetype 384a9d0c3f1b0634ba183b77d75618c0,
    // MantisZealotArchetype fc403240fdca4a52a413578169ea7117 and ProclaimerArchetype aaf7ace9642192d4389eab95e6cd04fd
    // remove it, and ShieldbearerArchetype d34dcc10e74961f44a81f72b54377b24 replaces it with
    // WarpriestShieldbearerChannelEnergySelection 4d4f00bb16467144780538107387af78. Its dice's steps are not traced.
    assert_gains(
        "warpriest",
        "warpriest/channel-energy",
        &|_, build| build,
        &[4, 5, 8, 11, 14, 17, 20],
        &[
            ("warpriest/champion-of-the-faith", Some(&[])),
            ("warpriest/cult-leader", Some(&[])),
            ("warpriest/disenchanter", Some(&[])),
            ("warpriest/mantis-zealot", Some(&[])),
            ("warpriest/proclaimer", Some(&[])),
            ("warpriest/shieldbearer", Some(&[])),
        ],
    );
    assert_archetype_steps(
        "warpriest",
        "warpriest/shieldbearer/channel-energy-shieldbearer",
        &[4, 5, 8, 11, 14, 17, 20],
    );
    // Game: WarpriestProgression 062b215705430ad47b7174c1e75d285e grants WarpriestFervorBase
    // 2d5140fd9f19a8e41be32d300eea2e18 at 2 and WarpriestFervorHealDamageRank bfcafb19f397da542be30d77126f1097 at 5, 8,
    // 11, 14, 17 and 20 (equivalent model).
    assert_gains(
        "warpriest",
        "warpriest/fervor",
        &|_, build| build,
        &[2, 5, 8, 11, 14, 17, 20],
        // Ticket 268: the_proclaimers_fervor_keeps_its_rank_at_8_only.
        &[("warpriest/proclaimer", Some(WAITING))],
    );
}

/// The Proclaimer keeps Fervor's base at 2 but only one of its damage ranks, at 8.
#[test]
#[ignore = "waits on ticket 268: the game gives the Proclaimer Fervor at 2 and 8 only"]
fn the_proclaimers_fervor_keeps_its_rank_at_8_only() {
    // Game: ProclaimerArchetype aaf7ace9642192d4389eab95e6cd04fd removes WarpriestFervorHealDamageSelection
    // b0dc39bfbc6f85646adbfc6413f843b0 at 2 and WarpriestFervorHealDamageRank bfcafb19f397da542be30d77126f1097 at 5,
    // 11, 14, 17 and 20, keeping WarpriestFervorBase 2d5140fd9f19a8e41be32d300eea2e18 at 2 and the rank at 8.
    let proclaimer = archetype_20("warpriest", "warpriest/proclaimer");
    assert_eq!(gains(&proclaimer, "warpriest/fervor"), [2, 8]);
}

/// Ticket 188: the Feral Champion's Wild Shape (Wolf) (the guide: once a day at 7, one more use every four levels,
/// 4 at 19) at 7, 11, 15 and 19, on rows that keep the bear form at 11 and the combat feat at 15.
#[test]
fn the_feral_champions_wild_shape_wolf_steps_every_four_levels() {
    // Game: FeralChampionArchetype f68ca492c9c15e241ab73735fbd0fb9f adds FeralChampnionWildShapeIWolfFeature
    // 1b60050091002ad458bd49788e84f13a at 7, an extra use at 11, 15 and 19 and the bear form at 11 (equivalent model).
    assert_archetype_steps(
        "warpriest",
        "warpriest/feral-champion/wild-shape-wolf",
        &[7, 11, 15, 19],
    );
    let feral = archetype_20("warpriest", "warpriest/feral-champion");
    assert!(holds(
        &at(&feral, 11),
        "warpriest/feral-champion/wild-shape-bear"
    ));
    // The row at 15 keeps the class's combat feat slot.
    assert_eq!(
        slot_counts(&feral)[14],
        slot_counts(&build("warpriest", 20))[14]
    );
}

/// Ticket 188 (review): the two features that replace the paladin's Smite Evil and rise themselves, each from 1st
/// level, at 5th and every five levels after, to 20th (the guide: the Stonelord's Stonestrike, +5 at 20; the
/// Tortured Crusader's All is Darkness, 5 a day at 20). No other archetype or the class holds either. The
/// Tortured Crusader's Unbroken Paragon at 20 adds three smite uses; it is not modelled.
#[test]
fn the_stonelords_stonestrike_and_the_tortured_crusaders_all_is_darkness_step() {
    // Game: StonelordArchetype cf0f99b3cd7444a48681b1c1c4aa2a41 adds StonestrikeFeature
    // 402c7820f7804cb7b56ab13cc7a9022b and TorturedCrusader f91ea9596e6f42f8a23803c1b3e976b4 adds
    // AllIsdarknessLvlUpFeature a1cf87d341064810b032780ecd8df079 at 1, 5, 10, 15 and 20.
    for feature in [
        "paladin/stonelord/stonestrike",
        "paladin/tortured-crusader/all-is-darkness",
    ] {
        let archetype = feature.rsplit_once('/').unwrap().0;
        assert_gains(
            "paladin",
            feature,
            &|_, build| build,
            &[],
            &[(archetype, Some(&[1, 5, 10, 15, 20]))],
        );
    }
}
