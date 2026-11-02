//! Prestige spellcasting advancement: which classes a prestige class may advance, and what that gives.

use rules::{Build, ClassChoice};

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
