//! Which choices the sheet may remove or change now: those no later choice depends on, so a chain comes off from
//! the leaves inward. The pickers lock what would break a later choice.

use rules::{Ability, ArchetypeKey, Build, Choice, ClassChoice, Edit, Engine, Issue, SlotId};

use crate::support::*;

/// The picks among the choices that may be removed now.
fn removable_picks(engine: &Engine, build: &Build) -> Vec<SlotId> {
    engine
        .removable(build)
        .into_iter()
        .filter_map(|choice| match choice {
            Choice::Slot(slot) => Some(slot),
            _ => None,
        })
        .collect()
}

/// Elf fighter 5: Power Attack (F1 bonus), Cleave (L1), Great Cleave (F4 bonus), Cleaving Finish (L5).
fn cleave_chain() -> Build {
    let mut build = build("elf", &times("fighter", 5));
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    build.picks.insert(general_slot(1), pick_feat("cleave"));
    build
        .picks
        .insert(class_slot(4, 0), pick_feat("great-cleave"));
    build
        .picks
        .insert(general_slot(5), pick_feat("cleaving-finish"));
    assert!(engine().is_possible(&build));
    build
}

#[test]
fn a_chain_comes_off_from_the_leaves_inward() {
    let engine = engine();
    let mut build = cleave_chain();
    // Each step: only the last link may be removed; removing it frees the one before.
    for leaf in [
        general_slot(5),
        class_slot(4, 0),
        general_slot(1),
        class_slot(1, 0),
    ] {
        assert_eq!(
            removable_picks(&engine, &build),
            std::slice::from_ref(&leaf)
        );
        build.apply(&Edit::Clear(leaf));
        assert!(engine.is_possible(&build));
    }
    assert_eq!(removable_picks(&engine, &build), []);
}

#[test]
fn a_class_level_a_later_pick_depends_on_isnt_changeable() {
    let engine = engine();
    let build = cleave_chain();
    let removable = engine.removable(&build);
    // Fighter 1 and 4 hold Power Attack and Great Cleave in their bonus slots; changing fighter 2 or 3 would move
    // fighter 4's bonus slot to level 5.
    for level in 1..=4 {
        assert!(
            !removable.contains(&Choice::Class { level }),
            "level {level}"
        );
    }
    // Cleaving Finish sits in level 5's general slot, which any class has.
    assert!(removable.contains(&Choice::Class { level: 5 }));

    // With nothing in them, every level may change.
    let empty = build_with_no_picks(&build);
    let removable = engine.removable(&empty);
    assert!((1..=5).all(|level| removable.contains(&Choice::Class { level })));
}

fn build_with_no_picks(build: &Build) -> Build {
    Build {
        picks: Default::default(),
        ..build.clone()
    }
}

#[test]
fn an_archetype_a_later_pick_depends_on_isnt_changeable() {
    let engine = engine();
    let mut build = build("elf", &times("fighter", 2));
    build.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    let archetype = Choice::Archetype {
        class: class("fighter"),
    };
    assert!(engine.removable(&build).contains(&archetype));

    // The armiger's order fills a selection only its row has.
    build.picks.insert(
        selection_slot(1, 0),
        pick_option("fighter/armiger/order/cockatrice"),
    );
    assert!(engine.is_possible(&build));
    let removable = engine.removable(&build);
    assert!(!removable.contains(&archetype));
    assert!(removable.contains(&Choice::Slot(selection_slot(1, 0))));
}

#[test]
fn a_point_buy_score_a_later_pick_depends_on_isnt_changeable() {
    let engine = engine();
    let mut build = build("elf", &["fighter"]);
    let all_scores = |build: &Build| {
        let removable = engine.removable(build);
        rules::AbilityScores::ABILITIES
            .into_iter()
            .filter(|ability| removable.contains(&Choice::AbilityScore { ability: *ability }))
            .collect::<Vec<_>>()
    };
    assert_eq!(all_scores(&build).len(), 6);

    // Power Attack needs Str 13.
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    assert!(!all_scores(&build).contains(&Ability::Strength));
    assert_eq!(all_scores(&build).len(), 5);
}

#[test]
fn skill_ranks_a_prestige_class_needs_are_not_removable() {
    // Rogue 3 / Wizard 3, Mobility 4 by level 4, then Arcane Trickster.
    let mut build = build(
        "elf",
        &[
            "rogue",
            "rogue",
            "rogue",
            "wizard",
            "wizard",
            "wizard",
            "arcane-trickster",
        ],
    );
    for level in &mut build.levels[..4] {
        level.skill_ranks.insert(skill("mobility"), 1);
    }
    assert!(engine().is_possible(&build));
    let removable = engine().removable(&build);
    for level in 1..=4 {
        assert!(
            !removable.contains(&Choice::SkillRanks {
                level,
                skill: skill("mobility")
            }),
            "level {level}"
        );
    }
    // The prestige class is the last level: nothing depends on it.
    assert!(removable.contains(&Choice::Class { level: 7 }));
}

#[test]
fn a_switch_the_levels_past_20_need_is_not_removable() {
    let engine = engine();
    let mut build = build("elf", &times("fighter", 20));
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    assert!(engine.is_possible(&build));
    assert!(engine.removable(&build).contains(&Choice::PathSwitch));

    build.apply(&Edit::Class {
        level: 21,
        class: class("rogue"),
        choice: None,
    });
    assert!(engine.is_possible(&build));
    assert!(!engine.removable(&build).contains(&Choice::PathSwitch));
}

#[test]
fn an_option_a_later_pick_conflicts_with_is_locked() {
    let engine = engine();
    let mut build = build("elf", &times("fighter", 3));
    build.picks.insert(general_slot(3), pick_feat("toughness"));
    let options = engine.options(&build, &general_slot(1)).unwrap();
    let status = |key: &str| {
        options
            .iter()
            .find(|option| option.pick.key() == key)
            .unwrap()
            .status
            .clone()
    };
    // Toughness at 1 would make level 3's Toughness held twice.
    assert_eq!(status("toughness"), Err(Issue::BreaksLater));
    assert_eq!(status("dodge"), Ok(()));
}

#[test]
fn replacing_a_pick_with_dependants_is_locked() {
    let engine = engine();
    let build = cleave_chain();
    let options = engine.options(&build, &general_slot(1)).unwrap();
    let status = |key: &str| {
        options
            .iter()
            .find(|option| option.pick.key() == key)
            .unwrap()
            .status
            .clone()
    };
    assert_eq!(status("cleave"), Ok(()));
    assert_eq!(status("toughness"), Err(Issue::BreaksLater));
}

#[test]
fn the_class_picker_locks_a_class_that_would_break_a_later_pick() {
    let engine = engine();
    let build = cleave_chain();
    let status = |level: u8, key: &str| {
        engine
            .class_options(&build, level)
            .unwrap()
            .into_iter()
            .find(|option| option.class == class(key) && option.archetype.is_none())
            .unwrap()
            .status
    };
    assert_eq!(status(4, "fighter"), Ok(()));
    assert_eq!(status(4, "wizard"), Err(Issue::BreaksLater));
    assert_eq!(status(5, "wizard"), Ok(()));
    // A level after the last breaks nothing.
    assert_eq!(status(6, "wizard"), Ok(()));
}

#[test]
fn the_switch_picker_locks_a_switch_the_levels_past_20_need() {
    let engine = engine();
    let mut build = build("elf", &[times("fighter", 20), vec!["rogue"]].concat());
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    assert!(engine.is_possible(&build));
    let status = |key: &str| {
        engine
            .switch_options(&build)
            .into_iter()
            .find(|option| option.path == path(key))
            .unwrap()
            .status
    };
    assert_eq!(status("legend"), Ok(()));
    // Gold Dragon keeps the cap at 20.
    assert_eq!(status("gold-dragon"), Err(Issue::BreaksLater));
}

#[test]
fn an_edit_is_allowed_when_every_other_choice_still_holds() {
    let engine = engine();
    // Wizard: 2 + Int 13 (+1) = 3 points, all spent.
    let mut build = build("dwarf", &["wizard"]);
    for key in ["athletics", "mobility", "persuasion"] {
        build.levels[0].skill_ranks.insert(skill(key), 1);
    }
    assert!(engine.is_possible(&build));
    let lower = |ability: Ability, score: u8| {
        let mut start = build.start.clone();
        start.ability_scores = start.ability_scores.with(ability, score).unwrap();
        Edit::Start(start)
    };
    // Int 11 leaves 2 points for the 3 ranks.
    assert!(!engine.allows(&build, &lower(Ability::Intelligence, 11)));
    assert!(engine.allows(&build, &lower(Ability::Charisma, 7)));
    assert!(engine.allows(&build, &Edit::Clear(general_slot(1))));
    let removable = engine.removable(&build);
    assert!(!removable.contains(&Choice::AbilityScore {
        ability: Ability::Intelligence
    }));
}
