//! Bloodline Ascendance (ticket 180) grants the 20th-level row of every bloodline option held that
//! `sorcerer/bloodline` offers, except one the holding's own selection skips: the Dragonheir Scion's Draconic
//! bloodline gets nothing from it, since its table gives Might of Wyrms at 20 instead. Where the selection says what
//! it gives in place of the skipped row (`replaced_by`: the Empyreal Sorcerer's Apotheosis, ticket 181), the ability
//! grants that, early, and it is held once.

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
    assert!(!holds(&at(&build, 5), DEMONIC_MIGHT));
    assert!(holds(&at(&build, 6), DEMONIC_MIGHT));
    assert_eq!(
        at(&build, 20).feature_count(&rules::FeatureKey::new(DEMONIC_MIGHT)),
        1
    );
}

/// With the Mythic Second Bloodline too, both bloodlines' 20th-level powers come at the rank that takes Ascendance.
#[test]
fn a_second_bloodline_gets_its_20th_level_power_too() {
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
    assert!(engine().is_possible(&build));
    let rank_2 = engine().rank_level(&build, 2).expect("MR 2 has a level");
    let before = at(&build, rank_2 - 1);
    assert!(!holds(&before, DEMONIC_MIGHT));
    assert!(!holds(&before, POWER_OF_WYRMS));
    let after = at(&build, rank_2);
    assert!(holds(&after, DEMONIC_MIGHT));
    assert!(holds(&after, POWER_OF_WYRMS));
}

/// An Eldritch Scion's selection skips no row, so its bloodline still gets its 20th-level power.
#[test]
fn an_eldritch_scions_bloodline_still_gets_its_20th_level_power() {
    let build = ascending(sorcerer_then(
        "magus",
        "magus/eldritch-scion",
        19,
        "sorcerer/bloodline/arcane",
    ));
    assert!(engine().is_possible(&build));
    assert!(!holds(&at(&build, 5), ARCANE_APOTHEOSIS));
    assert!(holds(&at(&build, 6), ARCANE_APOTHEOSIS));
    assert!(holds(&at(&build, 6), DEMONIC_MIGHT));
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
    assert!(!holds(&at(&build, 5), DEMONIC_MIGHT));
    assert!(holds(&at(&build, 6), DEMONIC_MIGHT));
}

/// An Empyreal Sorcerer's selection skips level 20 (its table gives Apotheosis there), so Ascendance adds nothing
/// to its Celestial bloodline: before ticket 180 it held the Celestial's Ascension from MR 1.
#[test]
fn an_empyreal_sorcerers_bloodline_gets_nothing_from_bloodline_ascendance() {
    let build = ascending(archetype_build(
        "sorcerer",
        "sorcerer/empyreal-sorcerer",
        20,
    ));
    assert!(engine().is_possible(&build));
    assert!(!holds(
        &at(&build, 20),
        "sorcerer/bloodline/celestial/ascension"
    ));
    assert!(holds(
        &at(&build, 20),
        "sorcerer/empyreal-sorcerer/apotheosis"
    ));
}

const APOTHEOSIS: &str = "sorcerer/empyreal-sorcerer/apotheosis";
const ASCENSION: &str = "sorcerer/bloodline/celestial/ascension";

fn empyreal(levels: usize) -> Build {
    archetype_build("sorcerer", "sorcerer/empyreal-sorcerer", levels)
}

fn count(state: &rules::CharacterState, feature: &str) -> u32 {
    u32::from(state.feature_count(&rules::FeatureKey::new(feature)))
}

/// Without the ability an Empyreal Sorcerer gains Apotheosis at 20, once, and never the Celestial's Ascension.
#[test]
fn an_empyreal_sorcerer_gains_apotheosis_at_20_without_the_ability() {
    let build = empyreal(20);
    assert!(engine().is_possible(&build));
    for level in 1..=20 {
        let state = at(&build, level);
        assert_eq!(count(&state, APOTHEOSIS), u32::from(level == 20), "{level}");
        assert_eq!(count(&state, ASCENSION), 0, "{level}");
    }
}

/// With the ability (MR 1, level 6) it holds Apotheosis from the rank's level, once at every level up to 20 (the
/// class's 20th level does not give it again), and never the Celestial's Ascension.
#[test]
fn an_empyreal_sorcerer_holds_apotheosis_early_through_the_ability_once() {
    let build = ascending(empyreal(20));
    assert!(engine().is_possible(&build));
    for level in 1..=20 {
        let state = at(&build, level);
        assert_eq!(count(&state, APOTHEOSIS), u32::from(level >= 6), "{level}");
        assert_eq!(count(&state, ASCENSION), 0, "{level}");
    }
}

/// The ability taken at the level the class reaches 20 (the latest a rank can come: a Build has 20 levels, and the
/// class's 20th is the Empyreal Sorcerer's last): the class row's level comes first in a level and the rank after,
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
    assert!(!holds(&state, APOTHEOSIS));
    assert!(!holds(&state, "fighter/dragonheir-scion/might-of-wyrms"));
}
