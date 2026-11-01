//! Whole-Build validation, as when a link is opened: picks that no longer qualify, and keys the game data lacks.

use rules::{Ability, Choice, Engine, Issue, LevelUp, Prerequisite, Problem, Unmet};

use crate::support::*;

fn at(problems: &[Problem]) -> Vec<(Choice, Issue)> {
    problems
        .iter()
        .map(|p| (p.at.clone(), p.issue.clone()))
        .collect()
}

#[test]
fn keys_missing_from_the_game_data_are_reported_not_fatal() {
    let mut build = build("race:elf", &["class:fighter", "class:monk"]);
    build
        .picks
        .insert(general_slot(1), pick_feat("feat:removed-feat"));
    build.levels[0]
        .skill_ranks
        .insert(skill("skill:removed-skill"), 1);
    build.mythic_path = Some(path("path:removed"));
    let problems = engine().validate(&build);

    assert!(problems.iter().all(Problem::is_unknown_key), "{problems:?}");
    assert_eq!(
        at(&problems),
        [
            (
                Choice::SkillRanks {
                    level: 1,
                    skill: skill("skill:removed-skill")
                },
                Issue::UnknownKey("skill:removed-skill".into())
            ),
            (
                Choice::Slot(general_slot(1)),
                Issue::UnknownKey("feat:removed-feat".into())
            ),
            (
                Choice::Class { level: 2 },
                Issue::UnknownKey("class:monk".into())
            ),
        ]
    );
    // The path isn't reached in a 2-level Build, so it isn't checked yet.
}

#[test]
fn picks_that_no_longer_qualify_are_reported_with_their_reason() {
    let mut build = build("race:elf", &["class:fighter"]);
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("feat:power-attack"));
    build
        .picks
        .insert(general_slot(1), pick_feat("feat:cleave"));
    assert_eq!(engine().validate(&build), []);

    // The game data changes: Power Attack now needs Str 18.
    let mut data = world();
    let power_attack = data
        .feats
        .iter_mut()
        .find(|f| f.key == feat("feat:power-attack"))
        .unwrap();
    power_attack.prerequisite = Some(Prerequisite::AbilityScore {
        ability: Ability::Strength,
        score: 18,
    });
    let problems = Engine::new(data).validate(&build);
    assert_eq!(
        at(&problems),
        [
            (
                Choice::Slot(class_slot(1, 0)),
                Issue::Unmet(Unmet::Condition {
                    needs: Prerequisite::AbilityScore {
                        ability: Ability::Strength,
                        score: 18
                    },
                    has: Some(16),
                })
            ),
            // Cleave goes down with it.
            (
                Choice::Slot(general_slot(1)),
                Issue::Unmet(Unmet::Condition {
                    needs: Prerequisite::Feat(feat("feat:power-attack")),
                    has: None,
                })
            ),
        ]
    );
}

#[test]
fn a_pick_for_a_slot_that_no_longer_exists_is_reported() {
    let mut build = build("race:elf", &times("class:wizard", 2));
    build
        .picks
        .insert(general_slot(2), pick_feat("feat:toughness"));
    assert_eq!(
        at(&engine().validate(&build)),
        [(Choice::Slot(general_slot(2)), Issue::NoSuchSlot)]
    );
}

#[test]
fn skill_ranks_are_capped_at_the_character_level() {
    let mut build = build("race:elf", &times("class:rogue", 3));
    build.levels[0]
        .skill_ranks
        .insert(skill("skill:mobility"), 1);
    build.levels[1]
        .skill_ranks
        .insert(skill("skill:mobility"), 1);
    build.levels[2]
        .skill_ranks
        .insert(skill("skill:mobility"), 1);
    assert_eq!(engine().validate(&build), []);

    build.levels[0]
        .skill_ranks
        .insert(skill("skill:athletics"), 2);
    assert_eq!(
        at(&engine().validate(&build)),
        [(
            Choice::SkillRanks {
                level: 1,
                skill: skill("skill:athletics")
            },
            Issue::SkillRankCap { cap: 1, ranks: 2 }
        )]
    );
}

#[test]
fn skill_points_cannot_be_overspent() {
    // Wizard: 2 + Int 13 (+1) = 3 points.
    let mut build = build("race:dwarf", &["class:wizard"]);
    for key in [
        "skill:athletics",
        "skill:mobility",
        "skill:persuasion",
        "skill:knowledge-arcana",
    ] {
        build.levels[0].skill_ranks.insert(skill(key), 1);
    }
    assert_eq!(
        at(&engine().validate(&build)),
        [(
            Choice::SkillPoints { level: 1 },
            Issue::SkillPointsOverspent {
                available: 3,
                spent: 4
            }
        )]
    );
}

#[test]
fn levels_above_20_need_the_legend_switch() {
    let build = build(
        "race:elf",
        &[times("class:fighter", 20), vec!["class:rogue"]].concat(),
    );
    assert_eq!(
        at(&engine().validate(&build)),
        [(
            Choice::CharacterLevel { level: 21 },
            Issue::LevelCap { cap: 20 }
        )]
    );
}

#[test]
fn class_entry_prerequisites_are_checked_on_the_first_level() {
    // Arcane Trickster straight away: no Mobility, no spells, no Sneak Attack.
    let build = build("race:elf", &["class:arcane-trickster"]);
    let problems = engine().validate(&build);
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].at, Choice::Class { level: 1 });
    let Issue::Unmet(unmet) = &problems[0].issue else {
        panic!("{problems:?}");
    };
    assert_eq!(unmet.conditions().len(), 3);

    // An archetype whose race prerequisite fails: Eldritch Scoundrel is for humans and elves.
    let mut dwarf = rules::Build {
        levels: vec![LevelUp::new(class("class:rogue"))],
        ..build.clone()
    };
    dwarf.start.race = Some(rules::RaceKey::new("race:dwarf"));
    dwarf.class_choices.insert(
        class("class:rogue"),
        rules::ClassChoice {
            archetype: Some(rules::ArchetypeKey::new(
                "archetype:rogue/eldritch-scoundrel",
            )),
            ..rules::ClassChoice::default()
        },
    );
    assert_eq!(
        engine()
            .validate(&dwarf)
            .into_iter()
            .map(|p| p.at)
            .collect::<Vec<_>>(),
        [Choice::Archetype {
            class: class("class:rogue")
        }]
    );
}

#[test]
fn the_point_buy_must_fit_the_budget() {
    let mut build = build("race:elf", &["class:fighter"]);
    build.start.ability_scores = scores([18, 18, 10, 10, 10, 10]);
    assert_eq!(
        at(&engine().validate(&build)),
        [(
            Choice::AbilityScores,
            Issue::PointBuy {
                cost: 34,
                budget: 25
            }
        )]
    );
    build.start.ability_scores = scores([18, 14, 14, 7, 10, 7]);
    assert_eq!(engine().validate(&build), []);
}

#[test]
fn a_switch_only_path_cannot_be_chosen_at_mr_3() {
    let mut build = build("race:elf", &times("class:fighter", 9));
    build.mythic_path = Some(path("path:legend"));
    assert_eq!(
        at(&engine().validate(&build)),
        [(Choice::MythicPath, Issue::NotChoosable)]
    );
    // And a path without a switch rule cannot be switched to.
    build.mythic_path = Some(path("path:trickster"));
    build
        .levels
        .extend((0..8).map(|_| LevelUp::new(class("class:fighter"))));
    build.path_switch = Some(path("path:trickster"));
    assert_eq!(
        at(&engine().validate(&build)),
        [(Choice::PathSwitch, Issue::NotChoosable)]
    );
}

#[test]
fn a_duplicate_pick_is_already_held() {
    let mut build = build("race:elf", &times("class:fighter", 3));
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("feat:toughness"));
    build
        .picks
        .insert(general_slot(3), pick_feat("feat:toughness"));
    assert_eq!(
        at(&engine().validate(&build)),
        [(Choice::Slot(general_slot(3)), Issue::AlreadyHeld)]
    );
}
