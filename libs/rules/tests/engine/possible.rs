//! Whether a Build is possible: the rules the walk checks over the whole Build. The sheet only ever holds a possible
//! Build, and a link to one that isn't opens nothing (`stale_links.rs`); the pickers say why an option can't be
//! taken.

use rules::{Ability, Build, Issue, LevelUp, PointBuy};

use crate::support::*;

#[test]
fn skill_points_cannot_be_overspent() {
    // Wizard: 2 + Int 13 (+1) = 3 points.
    let mut build = build("dwarf", &["wizard"]);
    for key in ["athletics", "mobility", "persuasion"] {
        build.levels[0].skill_ranks.insert(skill(key), 1);
    }
    assert!(engine().is_possible(&build));
    build.levels[0]
        .skill_ranks
        .insert(skill("knowledge-arcana"), 1);
    assert!(!engine().is_possible(&build));
}

#[test]
fn levels_above_20_need_the_legend_switch() {
    let mut build = build("elf", &[times("fighter", 20), vec!["rogue"]].concat());
    assert!(!engine().is_possible(&build));
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    assert!(engine().is_possible(&build));
}

#[test]
fn class_entry_prerequisites_are_checked_on_the_first_level() {
    // Arcane Trickster straight away: no Mobility, no spells, no Sneak Attack.
    let build = build("elf", &["arcane-trickster"]);
    assert!(!engine().is_possible(&build));
    let status = |build: &Build, key: &str, archetype: Option<&str>| {
        engine()
            .class_options(build, 1)
            .unwrap()
            .into_iter()
            .find(|option| {
                option.class == class(key)
                    && option.archetype.as_ref().map(|a| a.as_str()) == archetype
            })
            .unwrap()
            .status
    };
    let blank = Build {
        levels: vec![],
        ..build.clone()
    };
    let Err(Issue::Unmet(unmet)) = status(&blank, "arcane-trickster", None) else {
        panic!("Arcane Trickster is open at level 1");
    };
    assert_eq!(conditions(&unmet).len(), 3);

    // An archetype whose race prerequisite fails: Eldritch Scoundrel is for humans and elves.
    let mut dwarf = Build {
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
    assert!(!engine().is_possible(&dwarf));
    let blank = Build {
        levels: vec![],
        class_choices: Default::default(),
        ..dwarf.clone()
    };
    assert!(matches!(
        status(&blank, "rogue", Some("rogue/eldritch-scoundrel")),
        Err(Issue::Unmet(_))
    ));
    assert_eq!(status(&blank, "rogue", None), Ok(()));
}

/// A point buy that doesn't fit can't be made, so no Build holds one: the type holds only scores from 7 to 18
/// costing at most 25 points (and a link with another doesn't decode).
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
    assert!(!engine().is_possible(&build));
    let offered = |options: Vec<rules::PathOption>| -> Vec<_> {
        options.into_iter().map(|option| option.path).collect()
    };
    assert!(!offered(engine().path_options(&build)).contains(&path("legend")));

    // And a path without a switch rule cannot be switched to.
    build.mythic_path = Some(path("trickster"));
    build
        .levels
        .extend((0..8).map(|_| LevelUp::new(class("fighter"))));
    assert!(engine().is_possible(&build));
    build.path_switch = Some(path("trickster"));
    assert!(!engine().is_possible(&build));
    assert!(!offered(engine().switch_options(&build)).contains(&path("trickster")));
}

#[test]
fn a_duplicate_pick_is_already_held() {
    let mut build = build("elf", &times("fighter", 3));
    build.picks.insert(class_slot(1, 0), pick_feat("toughness"));
    build.picks.insert(general_slot(3), pick_feat("toughness"));
    assert_eq!(why_not(&build, &general_slot(3)), Some(Issue::AlreadyHeld));
}
