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
    assert_eq!(skilled, [("half-orc/human-raised", 1), ("human", 1)]);
}

#[test]
fn skilled_adds_a_skill_point_every_level() {
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
