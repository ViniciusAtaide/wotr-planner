//! Bloodline Ascendance (ticket 180) grants the 20th-level row of every bloodline option held that
//! `sorcerer/bloodline` offers, except one the holding's own selection skips: the Dragonheir Scion's Draconic
//! bloodline gets nothing from it, since its table gives Might of Wyrms at 20 instead.

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
