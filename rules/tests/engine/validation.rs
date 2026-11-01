//! Whole-Build validation, as when a link is opened: picks that no longer qualify, and keys the game data lacks.

use rules::{Ability, Choice, Engine, Issue, LevelUp, PointBuy, Prerequisite, Problem, Unmet};

use crate::support::*;

fn at(problems: &[Problem]) -> Vec<(Choice, Issue)> {
    problems
        .iter()
        .map(|p| (p.at.clone(), p.issue.clone()))
        .collect()
}

#[test]
fn keys_missing_from_the_game_data_are_reported_not_fatal() {
    let mut build = build("elf", &["fighter", "removed-class"]);
    build
        .picks
        .insert(general_slot(1), pick_feat("removed-feat"));
    build.levels[0]
        .skill_ranks
        .insert(skill("removed-skill"), 1);
    build.mythic_path = Some(path("removed"));
    let problems = engine().validate(&build);

    assert!(problems.iter().all(Problem::is_unknown_key), "{problems:?}");
    assert_eq!(
        at(&problems),
        [
            (
                Choice::SkillRanks {
                    level: 1,
                    skill: skill("removed-skill")
                },
                Issue::UnknownKey("removed-skill".into())
            ),
            (
                Choice::Slot(general_slot(1)),
                Issue::UnknownKey("removed-feat".into())
            ),
            (
                Choice::Class { level: 2 },
                Issue::UnknownKey("removed-class".into())
            ),
        ]
    );
    // The path isn't reached in a 2-level Build, so it isn't checked yet.
}

#[test]
fn picks_that_no_longer_qualify_are_reported_with_their_reason() {
    let mut build = build("elf", &["fighter"]);
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    build.picks.insert(general_slot(1), pick_feat("cleave"));
    assert_eq!(engine().validate(&build), []);

    // The game data changes: Power Attack now needs Str 18.
    let mut data = world();
    let power_attack = data
        .feats
        .iter_mut()
        .find(|f| f.key == feat("power-attack"))
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
                    needs: Prerequisite::Feat(feat("power-attack")),
                    has: None,
                })
            ),
        ]
    );
}

#[test]
fn a_pick_for_a_slot_that_no_longer_exists_is_reported() {
    let mut build = build("elf", &times("wizard", 2));
    build.picks.insert(general_slot(2), pick_feat("toughness"));
    assert_eq!(
        at(&engine().validate(&build)),
        [(Choice::Slot(general_slot(2)), Issue::NoSuchSlot)]
    );
}

#[test]
fn skill_points_cannot_be_overspent() {
    // Wizard: 2 + Int 13 (+1) = 3 points.
    let mut build = build("dwarf", &["wizard"]);
    for key in ["athletics", "mobility", "persuasion", "knowledge-arcana"] {
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
    let build = build("elf", &[times("fighter", 20), vec!["rogue"]].concat());
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
    let build = build("elf", &["arcane-trickster"]);
    let problems = engine().validate(&build);
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].at, Choice::Class { level: 1 });
    let Issue::Unmet(unmet) = &problems[0].issue else {
        panic!("{problems:?}");
    };
    assert_eq!(unmet.conditions().len(), 3);

    // An archetype whose race prerequisite fails: Eldritch Scoundrel is for humans and elves.
    let mut dwarf = rules::Build {
        levels: vec![LevelUp::new(class("rogue"))],
        ..build.clone()
    };
    dwarf.start.race = Some(rules::RaceKey::new("dwarf"));
    dwarf.class_choices.insert(
        class("rogue"),
        rules::ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("rogue/eldritch-scoundrel")),
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
            class: class("rogue")
        }]
    );
}

/// A point buy that doesn't fit can't be made, so it's no problem of a Build's: the type holds only scores from 7 to
/// 18 costing at most 25 points (and a link with another doesn't decode).
#[test]
fn a_point_buy_holds_only_scores_from_7_to_18_within_the_budget() {
    // 17 + 5 + 5 - 4 + 0 - 4 = 19 points.
    assert!(PointBuy::try_from(scores([18, 14, 14, 7, 10, 7])).is_ok());
    // 17 + 17 = 34 points.
    assert!(PointBuy::try_from(scores([18, 18, 10, 10, 10, 10])).is_err());
    assert!(PointBuy::try_from(scores([19, 10, 10, 10, 10, 10])).is_err());
    assert!(PointBuy::try_from(scores([6, 10, 10, 10, 10, 10])).is_err());

    let point_buy = PointBuy::default();
    assert_eq!(point_buy.get(Ability::Strength), 10);
    let strong = point_buy.with(Ability::Strength, 18).unwrap();
    assert_eq!(strong.get(Ability::Strength), 18);
    assert_eq!(strong.cost(), 17);
    assert_eq!(point_buy.with(Ability::Strength, 19), None);
    assert_eq!(strong.with(Ability::Dexterity, 18), None);
}

#[test]
fn a_switch_only_path_cannot_be_chosen_at_mr_3() {
    let mut build = build("elf", &times("fighter", 9));
    build.mythic_path = Some(path("legend"));
    assert_eq!(
        at(&engine().validate(&build)),
        [(Choice::MythicPath, Issue::NotChoosable)]
    );
    // And a path without a switch rule cannot be switched to.
    build.mythic_path = Some(path("trickster"));
    build
        .levels
        .extend((0..8).map(|_| LevelUp::new(class("fighter"))));
    build.path_switch = Some(path("trickster"));
    assert_eq!(
        at(&engine().validate(&build)),
        [(Choice::PathSwitch, Issue::NotChoosable)]
    );
}

#[test]
fn a_duplicate_pick_is_already_held() {
    let mut build = build("elf", &times("fighter", 3));
    build.picks.insert(class_slot(1, 0), pick_feat("toughness"));
    build.picks.insert(general_slot(3), pick_feat("toughness"));
    assert_eq!(
        at(&engine().validate(&build)),
        [(Choice::Slot(general_slot(3)), Issue::AlreadyHeld)]
    );
}
