//! Races' traits the rules count (ticket 130).

use rules::{RaceKey, SubraceKey};

use crate::support::*;

/// The skill points level 1 gives a fighter of `race` (and `subrace`).
fn fighter_points(race: &str, subrace: Option<&str>) -> u8 {
    let mut build = build("fighter", 1);
    build.start.race = Some(RaceKey::new(race));
    build.start.subrace = subrace.map(SubraceKey::new);
    engine().evaluate(&build).levels[0].skill_points.available
}

#[test]
fn only_the_human_and_the_human_raised_half_orc_are_skilled() {
    let skilled: Vec<(&str, u8)> = DATA
        .races
        .iter()
        .flat_map(|race| {
            std::iter::once((race.key.as_str(), race.skill_points)).chain(
                race.subraces
                    .iter()
                    .map(|subrace| (subrace.key.as_str(), subrace.skill_points)),
            )
        })
        .filter(|&(_, points)| points > 0)
        .collect();
    // Game: only HumanSkilled 3adf9274a210b164cb68f472dc1e4544 (HumanRace) and HalfOrcSkilled
    // 4e8fe10f42f314e4fa7c7918bcfadbd5 (HumanRaisedHalfOrc 289d95c2c26f4e638c1f26d5ea7c594b) carry
    // AddSkillPointPerCharacterLevel among what a race grants.
    assert_eq!(skilled, [("half-orc/human-raised", 1), ("human", 1)]);
}

#[test]
fn skilled_adds_a_skill_point_every_level() {
    // Game: UnitLogic/FactLogic/AddSkillPointPerCharacterLevel.HandleUnitCalculateSkillPointsOnLevelup, WrathDecompiled
    // 6ce4a6b: one point a level; TribalHalfOrc eb20cb0326444e348fc18f9b4d1644da has no HalfOrcSkilled.
    let half_orc = fighter_points("half-orc", None);
    assert_eq!(fighter_points("human", None), half_orc + 1);
    assert_eq!(
        fighter_points("half-orc", Some("half-orc/human-raised")),
        half_orc + 1
    );
    assert_eq!(
        fighter_points("half-orc", Some("half-orc/tribal")),
        half_orc
    );
}
