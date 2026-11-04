//! Prestige spellcasting advancement: which classes a prestige class may advance, and what that gives.

use std::collections::BTreeSet;

use rules::{Build, ClassChoice, ClassKey, ClassOption};

use crate::support::*;

/// `classes` in order, with `prestige` advancing `advances`, and Mobility 1 at each of the first four levels.
fn with_advances(classes: &[&str], prestige: &str, advances: &[&str]) -> Build {
    let mut build = build("elf", classes);
    for level in build.levels.iter_mut().take(4) {
        level.skill_ranks.insert(skill("mobility"), 1);
    }
    build.class_choices.insert(
        class(prestige),
        ClassChoice {
            archetype: None,
            advances: advances.iter().map(|key| class(key)).collect(),
        },
    );
    build
}

/// Whether the Build is possible with `prestige` advancing `advances` instead.
fn possible_advancing(build: &Build, prestige: &str, advances: &[&str]) -> bool {
    let mut build = build.clone();
    build
        .class_choices
        .get_mut(&class(prestige))
        .unwrap()
        .advances = advances.iter().map(|key| class(key)).collect();
    engine().is_possible(&build)
}

const THEURGE: [&str; 8] = [
    "wizard",
    "wizard",
    "wizard",
    "cleric",
    "cleric",
    "cleric",
    "mystic-theurge",
    "mystic-theurge",
];

#[test]
fn a_mystic_theurge_advances_one_arcane_and_one_divine_class() {
    let build = with_advances(&THEURGE, "mystic-theurge", &["wizard", "cleric"]);
    assert!(engine().is_possible(&build));

    // Wizard 3 / Cleric 3 / Mystic Theurge 2 casts as a wizard 5 and a cleric 5.
    let sheet = engine().evaluate(&build);
    let at_8 = sheet.at(8).unwrap();
    assert_eq!(at_8.arcane_spell_level, 3);
    assert_eq!(at_8.divine_spell_level, 3);
    assert_eq!(sheet.at(6).unwrap().divine_spell_level, 2);
}

#[test]
fn a_mystic_theurge_cannot_advance_two_arcane_classes() {
    let mut build = with_advances(
        &[
            "wizard",
            "wizard",
            "wizard",
            "cleric",
            "cleric",
            "cleric",
            "rogue",
            "rogue",
            "rogue",
            "mystic-theurge",
        ],
        "mystic-theurge",
        &["wizard", "rogue"],
    );
    build.class_choices.insert(
        class("rogue"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("rogue/eldritch-scoundrel")),
            ..ClassChoice::default()
        },
    );
    assert!(!engine().is_possible(&build));
    assert!(possible_advancing(
        &build,
        "mystic-theurge",
        &["wizard", "cleric"]
    ));
}

#[test]
fn an_advanced_class_must_already_be_taken() {
    // The witch levels come after the Arcane Trickster entry.
    let build = with_advances(
        &[
            "rogue",
            "rogue",
            "rogue",
            "wizard",
            "wizard",
            "wizard",
            "arcane-trickster",
            "witch",
        ],
        "arcane-trickster",
        &["witch"],
    );
    assert!(!engine().is_possible(&build));
    assert!(possible_advancing(&build, "arcane-trickster", &["wizard"]));
}

#[test]
fn an_advanced_class_must_cast_what_the_prestige_class_advances() {
    // A plain rogue casts nothing, and Arcane Trickster advances only arcane spellcasting.
    let rogue_wizard = [
        "rogue",
        "rogue",
        "rogue",
        "wizard",
        "wizard",
        "wizard",
        "cleric",
        "arcane-trickster",
    ];
    let build = with_advances(&rogue_wizard, "arcane-trickster", &["wizard"]);
    assert!(engine().is_possible(&build));
    for advances in ["rogue", "cleric"] {
        assert!(
            !possible_advancing(&build, "arcane-trickster", &[advances]),
            "{advances}"
        );
    }
}

#[test]
fn only_a_class_that_advances_spellcasting_takes_advances() {
    let build = with_advances(&["wizard", "fighter"], "fighter", &["wizard"]);
    assert!(!engine().is_possible(&build));
    assert!(possible_advancing(&build, "fighter", &[]));
}

/// The class picker's option for `class` at `level`.
fn class_option(build: &Build, level: u8, key: &str) -> ClassOption {
    engine()
        .class_options(build, level)
        .unwrap()
        .into_iter()
        .find(|option| option.class == class(key) && option.archetype.is_none())
        .unwrap()
}

fn sets(sets: &[&[&str]]) -> Vec<BTreeSet<ClassKey>> {
    sets.iter()
        .map(|set| set.iter().map(|key| class(key)).collect())
        .collect()
}

#[test]
fn the_advancement_choices_are_each_held_class_alone_then_each_pair_then_none_that_the_rules_allow()
{
    // Wizard 3 / Cleric 3 / Rogue 1, then Mystic Theurge: the rogue casts nothing, so it's in no choice.
    let build = build(
        "elf",
        &[
            "wizard", "wizard", "wizard", "cleric", "cleric", "cleric", "rogue",
        ],
    );
    let theurge = class_option(&build, 8, "mystic-theurge");
    assert_eq!(
        engine().advances_options(&build, 8, &theurge),
        sets(&[&["wizard"], &["cleric"], &["cleric", "wizard"], &[]])
    );
}

#[test]
fn a_prestige_class_advancing_either_spell_type_offers_one_class_at_a_time() {
    let build = build(
        "elf",
        &["wizard", "wizard", "wizard", "cleric", "cleric", "cleric"],
    );
    let winter_witch = class_option(&build, 7, "winter-witch");
    assert_eq!(
        engine().advances_options(&build, 7, &winter_witch),
        sets(&[&["wizard"], &["cleric"], &[]])
    );
}

#[test]
fn only_a_prestige_class_first_taken_asks_which_spellcasting_it_advances() {
    let mut build = build(
        "elf",
        &["wizard", "wizard", "wizard", "cleric", "cleric", "cleric"],
    );
    let engine = engine();
    assert!(engine.asks_advances(&build, 7, &class("mystic-theurge")));
    assert!(!engine.asks_advances(&build, 7, &class("wizard")));
    assert!(!engine.asks_advances(&build, 7, &class("fighter")));

    // Once taken, its advancement is fixed: another level of it, or the level that first takes it, asks nothing.
    build
        .levels
        .push(rules::LevelUp::new(class("mystic-theurge")));
    assert!(!engine.asks_advances(&build, 8, &class("mystic-theurge")));
    assert!(engine.asks_advances(&build, 7, &class("mystic-theurge")));
}

#[test]
fn a_class_edit_sets_the_choice_only_where_the_class_is_first_taken() {
    let mut build = with_advances(&THEURGE, "mystic-theurge", &["wizard", "cleric"]);
    let engine = engine();
    let theurge = class_option(&build, 7, "mystic-theurge");
    let edit = |build: &Build, level, advances: Option<&[&str]>| {
        engine.class_edit(
            build,
            level,
            &theurge,
            advances.map(|set| set.iter().map(|key| class(key)).collect()),
        )
    };
    // First taken at 7: the advancement given, else the one the Build has.
    let choice = |advances: &[&str]| {
        Some(ClassChoice {
            archetype: None,
            advances: advances.iter().map(|key| class(key)).collect(),
        })
    };
    assert_eq!(
        edit(&build, 7, Some(&["wizard"])),
        rules::Edit::Class {
            level: 7,
            class: class("mystic-theurge"),
            choice: choice(&["wizard"]),
        }
    );
    assert_eq!(
        edit(&build, 7, None),
        rules::Edit::Class {
            level: 7,
            class: class("mystic-theurge"),
            choice: choice(&["wizard", "cleric"]),
        }
    );
    // Taken before level 9: no choice.
    build
        .levels
        .push(rules::LevelUp::new(class("mystic-theurge")));
    assert_eq!(
        edit(&build, 9, Some(&["wizard"])),
        rules::Edit::Class {
            level: 9,
            class: class("mystic-theurge"),
            choice: None,
        }
    );
}

#[test]
fn a_class_edit_gives_a_class_first_taken_its_archetype() {
    let build = build("elf", &[]);
    let options = engine().class_options(&build, 1).unwrap();
    let zen_archer = options
        .iter()
        .find(|option| option.archetype == Some(rules::ArchetypeKey::new("monk/zen-archer")))
        .unwrap();
    assert_eq!(
        engine().class_edit(&build, 1, zen_archer, None),
        rules::Edit::Class {
            level: 1,
            class: class("monk"),
            choice: Some(ClassChoice {
                archetype: Some(rules::ArchetypeKey::new("monk/zen-archer")),
                advances: BTreeSet::new(),
            }),
        }
    );
}
