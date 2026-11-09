//! A level's controls (ticket 96): the ability increase's picker and the skill ranks' steppers, each option and
//! step with its status and the edit that takes it.

use rules::{Ability, AbilityScores, Build, Edit, Issue, Step};

use crate::support::*;

// ---- the ability increase -------------------------------------------------------------------------------------

#[test]
fn every_fourth_level_offers_each_ability_its_increase() {
    let build = build("human", &times("fighter", 4));
    let options = engine().ability_increase_options(&build, 4).unwrap();
    assert_eq!(options.len(), 6);
    for (option, ability) in options.iter().zip(AbilityScores::ABILITIES) {
        assert_eq!(option.value, ability);
        assert_eq!(option.status, Ok(()));
        assert_eq!(
            option.edit,
            Edit::AbilityIncrease {
                level: 4,
                ability: Some(ability),
            }
        );
    }
    assert_eq!(engine().ability_increase_options(&build, 3), None);
    assert_eq!(engine().ability_increase_options(&build, 8), None);
}

#[test]
fn an_increase_a_later_pick_needs_stays_where_it_is() {
    // Str 12, +1 at level 4, then Power Attack (Str 13) at level 5.
    let mut build = build("human", &times("fighter", 5));
    build.start.ability_scores = point_buy([12, 14, 12, 13, 10, 8]);
    build.levels[3].ability_increase = Some(Ability::Strength);
    build
        .picks
        .insert(general_slot(5), pick_feat("power-attack"));
    assert!(engine().is_possible(&build));
    for option in engine().ability_increase_options(&build, 4).unwrap() {
        let expected = if option.value == Ability::Strength {
            Ok(())
        } else {
            Err(Issue::BreaksLater)
        };
        assert_eq!(option.status, expected, "{:?}", option.value);
    }
}

// ---- the skill ranks ------------------------------------------------------------------------------------------

/// The skills in key order.
const SKILLS: [&str; 4] = ["athletics", "knowledge-arcana", "mobility", "persuasion"];

/// A step's status, or `None` past the end of the range.
type StepStatus = Option<Result<(), Issue>>;

/// Each skill's steps at a level: whether it has a step down and up, and their statuses.
fn skill_statuses(build: &Build, level: u8) -> Vec<(String, StepStatus, StepStatus)> {
    let status = |step: &Option<Step>| step.as_ref().map(|step| step.status.clone());
    engine()
        .skill_steps(build, level)
        .unwrap()
        .iter()
        .map(|stepper| {
            (
                stepper.value.to_string(),
                status(&stepper.down),
                status(&stepper.up),
            )
        })
        .collect()
}

fn ranks(build: &mut Build, level: u8, skill_key: &str, ranks: u8) {
    build.levels[usize::from(level) - 1]
        .skill_ranks
        .insert(skill(skill_key), ranks);
}

#[test]
fn each_skill_steps_up_from_no_ranks() {
    // A fighter's 2 skill points, +1 for Int 13.
    let build = build("human", &["fighter"]);
    let steppers = engine().skill_steps(&build, 1).unwrap();
    assert_eq!(
        skill_statuses(&build, 1),
        SKILLS.map(|key| (key.to_owned(), None, Some(Ok(()))))
    );
    for (stepper, key) in steppers.iter().zip(SKILLS) {
        assert_eq!(stepper.value, skill(key));
        assert_eq!(
            stepper.up.as_ref().unwrap().edit,
            Edit::SkillRanks {
                level: 1,
                skill: skill(key),
                ranks: 1,
            }
        );
    }
    assert_eq!(engine().skill_steps(&build, 2), None);
}

#[test]
fn ranks_step_no_higher_than_the_character_level_here_or_later() {
    let mut build = build("human", &["fighter", "fighter"]);
    ranks(&mut build, 1, "athletics", 1);
    ranks(&mut build, 2, "mobility", 2);
    assert!(engine().is_possible(&build));
    let level_1 = skill_statuses(&build, 1);
    // Athletics has its one rank at level 1; a second mobility rank at 1 would make three by level 2.
    assert_eq!(
        level_1[0],
        ("athletics".to_owned(), Some(Ok(())), None),
        "at the cap"
    );
    assert_eq!(
        level_1[2],
        ("mobility".to_owned(), None, None),
        "capped later"
    );
    assert_eq!(
        engine().skill_steps(&build, 1).unwrap()[0].down,
        Some(Step {
            status: Ok(()),
            edit: Edit::SkillRanks {
                level: 1,
                skill: skill("athletics"),
                ranks: 0,
            },
        })
    );
    // At level 2 athletics may take its second rank.
    assert_eq!(
        skill_statuses(&build, 2)[0],
        ("athletics".to_owned(), None, Some(Ok(())))
    );
}

#[test]
fn no_skill_steps_up_once_the_levels_points_are_spent() {
    // A dwarf fighter with Int 13 has 2 points at level 2 (Intelligence's half point comes at odd levels; a human
    // has one more).
    let mut build = build("dwarf", &["fighter", "fighter"]);
    for key in ["athletics", "mobility"] {
        ranks(&mut build, 2, key, 1);
    }
    assert!(engine().is_possible(&build));
    assert_eq!(
        skill_statuses(&build, 2),
        [
            ("athletics".to_owned(), Some(Ok(())), None),
            ("knowledge-arcana".to_owned(), None, None),
            ("mobility".to_owned(), Some(Ok(())), None),
            ("persuasion".to_owned(), None, None),
        ]
    );
}

#[test]
fn a_rank_a_later_pick_needs_does_not_step_down() {
    // Deft Maneuvers needs 3 ranks in Mobility: one at each of levels 1 to 3.
    let mut build = build("human", &times("fighter", 3));
    for level in 1..=3 {
        ranks(&mut build, level, "mobility", 1);
    }
    build
        .picks
        .insert(general_slot(3), pick_feat("deft-maneuvers"));
    assert!(engine().is_possible(&build));
    for level in 1..=3 {
        let mobility = &skill_statuses(&build, level)[2];
        assert_eq!(mobility.0, "mobility");
        assert_eq!(mobility.1, Some(Err(Issue::BreaksLater)), "level {level}");
    }
}
