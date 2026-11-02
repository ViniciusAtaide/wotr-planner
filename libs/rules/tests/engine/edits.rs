//! Edits to a Build, as the sheet makes them: class changes keep each class's archetype and spellcasting choice,
//! and levels come off the end.

use rules::{Ability, ArchetypeKey, Build, ClassChoice, Edit};

use crate::support::*;

fn armiger() -> ArchetypeKey {
    ArchetypeKey::new("fighter/armiger")
}

fn archetype_of(build: &Build, class_key: &str) -> Option<ArchetypeKey> {
    let sheet = engine().evaluate(build);
    let state = &sheet.levels.last().unwrap().state;
    state
        .classes
        .iter()
        .find(|c| c.class == class(class_key))
        .and_then(|c| c.archetype.clone())
}

#[test]
fn changing_a_levels_class_keeps_the_archetype_of_the_class_it_replaced() {
    let mut build = build("elf", &times("fighter", 3));
    build.apply(&Edit::Class {
        level: 1,
        class: class("fighter"),
        choice: Some(ClassChoice {
            archetype: Some(armiger()),
            ..ClassChoice::default()
        }),
    });

    // Level 1 becomes Rogue; Fighter now starts at level 2 and is still an armiger.
    build.apply(&Edit::Class {
        level: 1,
        class: class("rogue"),
        choice: None,
    });

    assert_eq!(archetype_of(&build, "fighter"), Some(armiger()));
    assert_eq!(archetype_of(&build, "rogue"), None);
}

#[test]
fn setting_an_archetype_on_a_later_level_applies_to_the_class() {
    let mut build = build("elf", &times("fighter", 3));

    build.apply(&Edit::Class {
        level: 3,
        class: class("fighter"),
        choice: Some(ClassChoice {
            archetype: Some(armiger()),
            ..ClassChoice::default()
        }),
    });

    assert_eq!(archetype_of(&build, "fighter"), Some(armiger()));
    // The armiger's rows apply from Fighter 1: an order selection instead of a Combat feat slot.
    let sheet = engine().evaluate(&build);
    let ids: Vec<_> = sheet.levels[0].slots.iter().map(|s| s.id.clone()).collect();
    assert_eq!(ids, [selection_slot(1, 0), general_slot(1)]);
}

#[test]
fn a_class_change_without_a_choice_leaves_the_new_classes_choice_alone() {
    // Rogue 1 is an Eldritch Scoundrel; changing level 2 from Fighter to Rogue keeps that.
    let mut build = build("elf", &["rogue", "fighter"]);
    let scoundrel = ArchetypeKey::new("rogue/eldritch-scoundrel");
    build.apply(&Edit::Class {
        level: 1,
        class: class("rogue"),
        choice: Some(ClassChoice {
            archetype: Some(scoundrel.clone()),
            ..ClassChoice::default()
        }),
    });

    build.apply(&Edit::Class {
        level: 2,
        class: class("rogue"),
        choice: None,
    });

    assert_eq!(archetype_of(&build, "rogue"), Some(scoundrel));
}

#[test]
fn a_choice_for_a_class_no_level_takes_any_more_is_dropped() {
    let mut build = build("elf", &["fighter"]);
    build.apply(&Edit::Class {
        level: 1,
        class: class("fighter"),
        choice: Some(ClassChoice {
            archetype: Some(armiger()),
            ..ClassChoice::default()
        }),
    });

    build.apply(&Edit::Class {
        level: 1,
        class: class("rogue"),
        choice: None,
    });

    assert!(build.class_choices.is_empty(), "{:?}", build.class_choices);
}

#[test]
fn remove_level_takes_off_the_last_level_with_its_picks_and_unused_class_choices() {
    let mut build = build("elf", &["fighter", "fighter", "rogue"]);
    build.picks.insert(general_slot(1), pick_feat("toughness"));
    build.picks.insert(general_slot(3), pick_feat("dodge"));
    build.picks.insert(
        selection_slot(3, 0),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    build
        .class_choices
        .insert(class("rogue"), ClassChoice::default());

    build.apply(&Edit::RemoveLevel);

    assert_eq!(build.levels.len(), 2);
    assert_eq!(
        build.picks.keys().cloned().collect::<Vec<_>>(),
        [general_slot(1)]
    );
    assert!(build.class_choices.is_empty());
    assert!(engine().is_possible(&build));
}

#[test]
fn remove_level_on_an_empty_build_does_nothing() {
    let mut build = Build::default();
    build.apply(&Edit::RemoveLevel);
    assert_eq!(build, Build::default());
}

#[test]
fn set_name_names_and_unnames_the_build() {
    let mut build = Build::default();
    build.apply(&Edit::SetName(Some("Cleaver".into())));
    assert_eq!(build.name.as_deref(), Some("Cleaver"));
    build.apply(&Edit::SetName(None));
    assert_eq!(build.name, None);
}

#[test]
fn an_ability_increase_goes_only_on_a_level_that_has_one() {
    let mut build = build("elf", &times("fighter", 4));
    let before = build.clone();
    build.apply(&Edit::AbilityIncrease {
        level: 3,
        ability: Some(Ability::Strength),
    });
    assert_eq!(build, before);
    build.apply(&Edit::AbilityIncrease {
        level: 4,
        ability: Some(Ability::Strength),
    });
    assert_eq!(build.levels[3].ability_increase, Some(Ability::Strength));
}

#[test]
fn skill_ranks_never_go_above_the_character_level() {
    let mut build = build("elf", &times("rogue", 3));
    let ranks = |level, ranks| Edit::SkillRanks {
        level,
        skill: skill("mobility"),
        ranks,
    };
    // Two ranks at level 1 is one too many.
    build.apply(&ranks(1, 2));
    assert!(build.levels[0].skill_ranks.is_empty());
    // None at 1 and two at 2 fits; then one at 1 would make three by level 2.
    build.apply(&ranks(2, 2));
    build.apply(&ranks(1, 1));
    assert!(build.levels[0].skill_ranks.is_empty());
    assert_eq!(build.levels[1].skill_ranks[&skill("mobility")], 2);
    // Fewer ranks always fit.
    build.apply(&ranks(2, 1));
    build.apply(&ranks(1, 1));
    assert_eq!(build.levels[0].skill_ranks[&skill("mobility")], 1);
}
