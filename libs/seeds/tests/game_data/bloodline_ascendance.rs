//! Bloodline Ascendance (ticket 180) grants the 20th-level row of every bloodline option held that
//! `sorcerer/bloodline` offers, except one the holding's own selection skips: the Dragonheir Scion's Draconic
//! bloodline gets nothing from it, since its table gives Might of Wyrms at 20 instead. Where the selection says what
//! it gives in place of the skipped row (`replaced_by`: the Empyreal Sorcerer's Apotheosis, ticket 181), the ability
//! grants that, early, and it is held once.
//!
//! The game's Bloodline Ascendance (BloodlineAscendance ce85aee1726900641ab53ede61ac5c19) is a selection of one power:
//! one held bloodline's 20th-level power, each option gated by its bloodline's requisite feature. Where a Build holds
//! one bloodline, the data's grant of every row agrees; where it holds two, the game gives one
//! (`…_grants_one_power_of_the_two`, waiting on a ticket).

use rules::{Build, Pick, SlotId};

use crate::support::*;

const ABYSSAL: &str = "sorcerer/bloodline/abyssal";
const DRACONIC_RED: &str = "sorcerer/bloodline/draconic-red";
const DEMONIC_MIGHT: &str = "sorcerer/bloodline/abyssal/demonic-might";
const POWER_OF_WYRMS: &str = "sorcerer/bloodline/draconic/power-of-wyrms";
const ARCANE_APOTHEOSIS: &str = "sorcerer/bloodline/arcane/arcane-apotheosis";

/// `build` with Bloodline Ascendance as MR 1's ability (MR 1 comes at level 6).
fn ascending(build: Build) -> Build {
    with_mr_1_ability(build, "bloodline-ascendance")
}

/// A sorcerer 1 of the Abyssal bloodline who then takes `levels` levels of `class` (with `archetype`), whose
/// bloodline selection holds `option`.
fn sorcerer_then(class: &str, archetype: &str, levels: usize, option: &str) -> Build {
    let build = multiclass(&[("sorcerer", None, 1), (class, Some(archetype), levels)]);
    let build = with_option(build, 1, 0, ABYSSAL);
    let selection = format!(
        "{class}/{}/bloodline",
        archetype.rsplit('/').next().unwrap()
    );
    let slot = slot_of(&build, 2, &selection);
    with_picks_at(build, slot, option)
}

fn with_picks_at(mut build: Build, slot: SlotId, option: &str) -> Build {
    build.picks.insert(slot, pick(option));
    faithful(build)
}

/// The repro: a sorcerer 1 (Abyssal) / Dragonheir Scion 19 (Draconic Red) with Bloodline Ascendance holds the
/// Abyssal's 20th-level power and not the Draconic bloodline's, which its selection skips.
#[test]
fn a_dragonheir_scions_bloodline_gets_nothing_from_bloodline_ascendance() {
    let build = ascending(sorcerer_then(
        "fighter",
        "fighter/dragonheir-scion",
        19,
        DRACONIC_RED,
    ));
    assert!(engine().is_possible(&build));
    // Game: DragonheirDragonSelection 729411185291d704696e58316420fe38 (the Dragonheir's dragon progressions, e.g.
    // DragonheirRedProgression 267dbd4789fa4b75a44294c7d1625bba, grant no Draconic requisite feature, so Power of Wyrms
    // isn't an option of BloodlineAscendance ce85aee1726900641ab53ede61ac5c19); BloodlineAbyssalDemonicMight
    // 5c1c2ed7fe5f99649ab00605610b775b is
    for level in [5, 6, 7, 20] {
        let state = at(&build, level);
        assert!(!holds(&state, POWER_OF_WYRMS), "level {level}");
        assert_eq!(holds(&state, DEMONIC_MIGHT), level >= 6, "level {level}");
    }
}

/// A plain sorcerer's Ascendance grants its own bloodline's 20th-level power, as before.
#[test]
fn a_sorcerers_bloodline_ascendance_grants_its_20th_level_power() {
    let build = ascending(with_option(build("sorcerer", 20), 1, 0, ABYSSAL));
    assert!(engine().is_possible(&build));
    // Game: BloodlineAscendance ce85aee1726900641ab53ede61ac5c19 (its one open option here is
    // BloodlineAbyssalDemonicMight 5c1c2ed7fe5f99649ab00605610b775b); BloodlineAbyssalProgression
    // d3a4cb7be97a6694290f0dcfbd147113 (the same feature at 20, held once)
    assert!(!holds(&at(&build, 5), DEMONIC_MIGHT));
    assert!(holds(&at(&build, 6), DEMONIC_MIGHT));
    assert_eq!(
        at(&build, 20).feature_count(&rules::FeatureKey::new(DEMONIC_MIGHT)),
        1
    );
}

/// A sorcerer 20 (Abyssal) with the Mythic Second Bloodline (Draconic Red) at MR 1 and Bloodline Ascendance at MR 2.
fn two_bloodlines_ascending() -> Build {
    let mut build = with_option(build("sorcerer", 20), 1, 0, ABYSSAL);
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::mythic_ability(rules::MythicAbilityKey::new("second-bloodline")),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        pick(DRACONIC_RED),
    );
    // Extra Mythic Ability at MR 2 gives the second ability pick.
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new("extra-mythic-ability")),
    );
    build.picks.insert(
        SlotId::MythicExtra { rank: 2 },
        Pick::mythic_ability(rules::MythicAbilityKey::new("bloodline-ascendance")),
    );
    build
}

/// How many of `powers` `state` holds.
fn held(state: &rules::CharacterState, powers: &[&str]) -> usize {
    powers.iter().filter(|power| holds(state, power)).count()
}

/// With the Mythic Second Bloodline too, neither bloodline's 20th-level power comes before the rank that takes
/// Ascendance, and one at least comes at it.
#[test]
fn with_a_second_bloodline_a_20th_level_power_comes_at_ascendance() {
    let build = two_bloodlines_ascending();
    assert!(engine().is_possible(&build));
    let rank_2 = engine().rank_level(&build, 2).expect("MR 2 has a level");
    let before = at(&build, rank_2 - 1);
    // Game: BloodlineAscendance ce85aee1726900641ab53ede61ac5c19 (taken at MR 2); SecondBloodline
    // 3cf2ab2c320b73347a7c21cf0d0995bd
    assert_eq!(held(&before, &[DEMONIC_MIGHT, POWER_OF_WYRMS]), 0);
    assert!(held(&at(&build, rank_2), &[DEMONIC_MIGHT, POWER_OF_WYRMS]) >= 1);
}

/// The game's Ascendance is a pick of one: with the Abyssal and the Draconic Red bloodlines, one of Demonic Might and
/// Power of Wyrms, not both.
// Game: BloodlineAscendance ce85aee1726900641ab53ede61ac5c19 (a BlueprintFeatureSelection, one pick, options
// BloodlineAbyssalDemonicMight 5c1c2ed7fe5f99649ab00605610b775b and BloodlineDraconicRedPowerOfWyrms
// a18ab74c10933e84daf76afdaabc28dd both open)
#[test]
#[ignore = "waits on ticket 257: the game gives one 20th-level power of the two bloodlines held"]
fn a_second_bloodlines_ascendance_grants_one_power_of_the_two() {
    let build = two_bloodlines_ascending();
    let rank_2 = engine().rank_level(&build, 2).expect("MR 2 has a level");
    assert_eq!(
        held(&at(&build, rank_2), &[DEMONIC_MIGHT, POWER_OF_WYRMS]),
        1
    );
}

/// A sorcerer 1 (Abyssal) / Eldritch Scion 19 (Arcane) with Bloodline Ascendance.
fn abyssal_sorcerer_arcane_scion() -> Build {
    ascending(sorcerer_then(
        "magus",
        "magus/eldritch-scion",
        19,
        "sorcerer/bloodline/arcane",
    ))
}

/// An Eldritch Scion's selection skips no row, so its bloodline's 20th-level power is an option of Ascendance: none
/// comes before the ability's rank, and one at least at it.
#[test]
fn an_eldritch_scions_bloodline_still_gets_its_20th_level_power() {
    let build = abyssal_sorcerer_arcane_scion();
    assert!(engine().is_possible(&build));
    // Game: BloodlineAscendance ce85aee1726900641ab53ede61ac5c19 (open to an Eldritch Scion's bloodline);
    // BloodlineArcaneArcaneApotheosisFeature 2086d8c0d40e35b40b86d47e47fb17e4
    assert_eq!(held(&at(&build, 5), &[ARCANE_APOTHEOSIS, DEMONIC_MIGHT]), 0);
    assert!(held(&at(&build, 6), &[ARCANE_APOTHEOSIS, DEMONIC_MIGHT]) >= 1);
}

/// With the sorcerer's Abyssal and the Eldritch Scion's Arcane bloodline, Ascendance gives one of Arcane Apotheosis
/// and Demonic Might.
// Game: BloodlineAscendance ce85aee1726900641ab53ede61ac5c19 (one pick; BloodlineArcaneArcaneApotheosisFeature
// 2086d8c0d40e35b40b86d47e47fb17e4 and BloodlineAbyssalDemonicMight 5c1c2ed7fe5f99649ab00605610b775b both open)
#[test]
#[ignore = "waits on ticket 257: the game gives an Abyssal sorcerer / Arcane Eldritch Scion one of Arcane Apotheosis and Demonic Might"]
fn an_eldritch_scions_ascendance_grants_one_power_of_the_two() {
    let build = abyssal_sorcerer_arcane_scion();
    assert_eq!(held(&at(&build, 6), &[ARCANE_APOTHEOSIS, DEMONIC_MIGHT]), 1);
}

/// A Seeker's selection skips levels 3, 9, 13, 15 and 17 but not 20, so Ascendance grants its bloodline's 20th-level
/// row as it does a plain sorcerer's.
#[test]
fn a_seekers_bloodline_still_gets_its_20th_level_power() {
    let build = ascending(with_option(
        archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 20),
        1,
        0,
        ABYSSAL,
    ));
    assert!(engine().is_possible(&build));
    // Game: BloodlineAscendance ce85aee1726900641ab53ede61ac5c19 (SeekerBloodlineSelection
    // 7bda7cdb0ccda664c9eb8978cf512dbc qualifies; the Abyssal requisite is held)
    assert!(!holds(&at(&build, 5), DEMONIC_MIGHT));
    assert!(holds(&at(&build, 6), DEMONIC_MIGHT));
}

/// An Empyreal Sorcerer's selection skips level 20 (its table gives Apotheosis there, the game's Celestial Ascension:
/// one entry since ticket 244), so Ascendance adds nothing to its Celestial bloodline but that one feature: before
/// ticket 180 it held the Celestial's row 20 from MR 1 besides.
#[test]
fn an_empyreal_sorcerers_bloodline_gets_nothing_from_bloodline_ascendance() {
    let build = ascending(archetype_build(
        "sorcerer",
        "sorcerer/empyreal-sorcerer",
        20,
    ));
    assert!(engine().is_possible(&build));
    // Game: EmpyrealBloodlineProgression 8a95d80a3162d274896d50c2f18bb6b1 (row 20: BloodlineCelestialAscensionAbility
    // d85e0396d8f68e047b7b67a1290f8dbc, the Empyreal's Apotheosis and the Celestial's Ascension, held once)
    assert_eq!(count(&at(&build, 20), APOTHEOSIS), 1);
}

/// The Empyreal's Apotheosis: the game's Celestial Ascension, one entry since ticket 244.
const APOTHEOSIS: &str = "celestial-ascension";

fn empyreal(levels: usize) -> Build {
    archetype_build("sorcerer", "sorcerer/empyreal-sorcerer", levels)
}

fn count(state: &rules::CharacterState, feature: &str) -> u32 {
    u32::from(state.feature_count(&rules::FeatureKey::new(feature)))
}

/// Without the ability an Empyreal Sorcerer gains Apotheosis at 20, once.
#[test]
fn an_empyreal_sorcerer_gains_apotheosis_at_20_without_the_ability() {
    let build = empyreal(20);
    assert!(engine().is_possible(&build));
    // Game: EmpyrealBloodlineProgression 8a95d80a3162d274896d50c2f18bb6b1 (BloodlineCelestialAscensionAbility
    // d85e0396d8f68e047b7b67a1290f8dbc at sorcerer 20, once)
    for level in 1..=20 {
        let state = at(&build, level);
        assert_eq!(count(&state, APOTHEOSIS), u32::from(level == 20), "{level}");
    }
}

/// With the ability (MR 1, level 6) it holds Apotheosis from the rank's level, once at every level up to 20 (the
/// class's 20th level does not give it again).
#[test]
fn an_empyreal_sorcerer_holds_apotheosis_early_through_the_ability_once() {
    let build = ascending(empyreal(20));
    assert!(engine().is_possible(&build));
    // Game: BloodlineAscendance ce85aee1726900641ab53ede61ac5c19 (its option for the Empyreal is
    // BloodlineCelestialAscensionAbility d85e0396d8f68e047b7b67a1290f8dbc, the same feature as row 20, held once:
    // one entry)
    for level in 1..=20 {
        let state = at(&build, level);
        assert_eq!(count(&state, APOTHEOSIS), u32::from(level >= 6), "{level}");
    }
}

/// The ability taken at the level the class reaches 20 (the latest a rank can come in a Build that reaches it first:
/// levels past 20 need the Legend switch, and ranks never come before the ones before them, so a rank placed past
/// 20 pushes the switch past 20 and the Build is refused): the class row's level comes first in a level and the rank after,
/// and Apotheosis is held once at 20, not twice.
#[test]
fn apotheosis_is_held_once_when_the_ability_comes_at_level_20() {
    let mut build = empyreal(20);
    build.rank_levels.insert(1, 20);
    let build = ascending(build);
    assert!(
        engine().is_possible(&build),
        "{:?}",
        engine().failures(&build)
    );
    assert_eq!(engine().rank_level(&build, 1), Some(20));
    // Game: BloodlineCelestialAscensionAbility d85e0396d8f68e047b7b67a1290f8dbc, granted by both and held once (a
    // feature granted again gains a rank: Feature.AddRank, WrathDecompiled 6ce4a6b); the order of a class row and a
    // rank in one level is the planner's model
    for level in 1..=20 {
        assert_eq!(
            count(&at(&build, level), APOTHEOSIS),
            u32::from(level == 20),
            "level {level}"
        );
    }
}

/// Withdrawing the ability (a Build without the pick) returns Apotheosis to level 20.
#[test]
fn withdrawing_the_ability_returns_apotheosis_to_level_20() {
    let with = ascending(empyreal(20));
    let mut without = with.clone();
    without.picks.remove(&SlotId::Mythic { rank: 1 });
    assert!(holds(&at(&with, 6), APOTHEOSIS));
    assert!(!holds(&at(&without, 6), APOTHEOSIS));
    assert!(!holds(&at(&without, 19), APOTHEOSIS));
    assert_eq!(count(&at(&without, 20), APOTHEOSIS), 1);
}

/// The Dragonheir Scion's Might of Wyrms stays out of the
/// ability's reach (ticket 181 leaves it for the user to decide).
#[test]
fn the_dragonheir_scion_gets_no_replacement_from_the_ability() {
    let build = ascending(sorcerer_then(
        "fighter",
        "fighter/dragonheir-scion",
        19,
        DRACONIC_RED,
    ));
    let state = at(&build, 20);
    // Game: BloodlineAscendance ce85aee1726900641ab53ede61ac5c19 (no option for the Dragonheir's dragon)
    assert!(!holds(&state, APOTHEOSIS));
    assert!(!holds(&state, "fighter/dragonheir-scion/might-of-wyrms"));
}

/// The Build `parts` on the Aeon path with the Legend switch, and Bloodline Ascendance at MR `rank` (the ability
/// slot of every rank is `slot.mythic.R`).
fn legend_ascending(parts: &[(&str, Option<&str>, usize)], rank: u8) -> Build {
    let mut build = multiclass(parts);
    build.mythic_path = Some(rules::MythicPathKey::new("aeon"));
    build.path_switch = Some(rules::MythicPathKey::new("legend"));
    build.picks.insert(
        SlotId::Mythic { rank },
        Pick::mythic_ability(rules::MythicAbilityKey::new("bloodline-ascendance")),
    );
    faithful(build)
}

/// What `feature` should count at each level of a Legend Build with Ascendance at `rank`: held from the rank's
/// level while the rank is, and from the level the sorcerer reaches 20 whatever happens to the rank. Asserts it
/// at every level from the rank's level to the last, and that the switch drops the rank when `drops_rank`.
fn assert_legend_counts(
    build: &Build,
    rank: u8,
    feature: &str,
    sorcerer_before: usize,
    drops: bool,
) {
    assert!(
        engine().is_possible(build),
        "{:?}",
        engine().failures(build)
    );
    let rank_level = engine().rank_level(build, rank).unwrap();
    let last = build.levels.len() as u8;
    let switch = (1..=last)
        .find(|&level| at(build, level).mythic_path.as_ref().map(|p| p.as_str()) == Some("legend"))
        .expect("the switch is reached");
    for level in rank_level..=last {
        let sorcerer_level = (usize::from(level)).saturating_sub(sorcerer_before);
        let early = level >= rank_level && !(drops && level >= switch);
        let at_20 = sorcerer_level >= 20;
        assert_eq!(
            count(&at(build, level), feature),
            u32::from(early || at_20),
            "{feature}, ability at MR {rank} (level {rank_level}), switch at {switch}, level {level}"
        );
    }
}

/// Legend drops the ranks 3 to 7: an ability taken there goes at the switch, and what it granted early goes with
/// it, so sorcerer level 20 gives the power at its own level (before ticket 181's review fix it never did).
#[test]
fn a_legend_empyreal_sorcerer_gets_apotheosis_at_20_whatever_happens_to_the_rank() {
    for rank in [1, 3, 5, 7] {
        let drops = rank >= 3;
        // Game: EmpyrealBloodlineProgression 8a95d80a3162d274896d50c2f18bb6b1 (row 20 at sorcerer 20); the Legend
        // switch's dropped ranks are the planner's model of the story, not a game fact
        let build = legend_ascending(
            &[
                ("sorcerer", Some("sorcerer/empyreal-sorcerer"), 20),
                ("fighter", None, 5),
            ],
            rank,
        );
        assert_legend_counts(&build, rank, APOTHEOSIS, 0, drops);
        let build = legend_ascending(
            &[
                ("fighter", None, 5),
                ("sorcerer", Some("sorcerer/empyreal-sorcerer"), 20),
            ],
            rank,
        );
        assert_legend_counts(&build, rank, APOTHEOSIS, 5, drops);
    }
}

/// The same for an ordinary bloodline row the ability granted (it was lost on `main` too): an Abyssal sorcerer 20 /
/// fighter 5 with Ascendance at MR 3 holds Demonic Might again from sorcerer level 20.
#[test]
fn a_legend_sorcerer_gets_its_20th_level_power_at_20_whatever_happens_to_the_rank() {
    for rank in [1, 3, 5, 7] {
        let build = legend_ascending(&[("sorcerer", None, 20), ("fighter", None, 5)], rank);
        let build = with_option(build, 1, 0, ABYSSAL);
        // Game: BloodlineAbyssalProgression d3a4cb7be97a6694290f0dcfbd147113 (Demonic Might at sorcerer 20)
        assert_legend_counts(&build, rank, DEMONIC_MIGHT, 0, rank >= 3);
    }
}

/// The game answers ticket 181's question: the Empyreal bloodline counts sorcerer levels only, so an Empyreal
/// Sorcerer 6 / Dragon Disciple 10 / Empyreal Sorcerer 4 (the Dragon Disciple reached through the Mythic Second
/// Bloodline, Draconic) has 10 sorcerer levels and no Apotheosis by character level 20.
// Game: EmpyrealBloodlineProgression 8a95d80a3162d274896d50c2f18bb6b1 (classes: SorcererClass alone; the Draconic
// ones, e.g. BloodlineDraconicRedProgression 8c6e5b3cf12f71e43949f52c41ae70a8, also list DragonDiscipleClass)
#[test]
#[ignore = "waits on ticket 257: the game counts no Dragon Disciple level towards the Empyreal's 20th-level row"]
fn dragon_disciple_levels_dont_count_towards_an_empyreal_sorcerers_apotheosis() {
    let mut build = multiclass(&[
        ("sorcerer", Some("sorcerer/empyreal-sorcerer"), 6),
        ("dragon-disciple", None, 10),
        ("sorcerer", Some("sorcerer/empyreal-sorcerer"), 4),
    ]);
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::mythic_ability(rules::MythicAbilityKey::new("second-bloodline")),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        pick(DRACONIC_RED),
    );
    // Dragon Disciple needs 5 ranks in Knowledge (Arcana).
    for level_up in build.levels.iter_mut().take(5) {
        level_up
            .skill_ranks
            .insert(rules::SkillKey::new("knowledge-arcana"), 1);
    }
    let build = faithful(build);
    assert!(
        engine().is_possible(&build),
        "{:?}",
        engine().failures(&build)
    );
    for level in 1..=20 {
        assert_eq!(count(&at(&build, level), APOTHEOSIS), 0, "level {level}");
    }
}
