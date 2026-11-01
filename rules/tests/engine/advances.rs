//! Prestige spellcasting advancement: which classes a prestige class may advance, and what that gives.

use rules::{Build, Choice, ClassChoice, Issue};

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

/// The problems at the prestige class's advancement choice.
fn advances_problems(build: &Build, prestige: &str) -> Vec<Issue> {
    engine()
        .validate(build)
        .into_iter()
        .filter(|p| {
            p.at == Choice::Advances {
                class: class(prestige),
            }
        })
        .map(|p| p.issue)
        .collect()
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
    assert_eq!(engine().validate(&build), []);

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
    assert_eq!(
        advances_problems(&build, "mystic-theurge"),
        [Issue::NotAllowed]
    );
}

#[test]
fn an_advanced_class_must_already_be_taken() {
    // The cleric levels come after the Mystic Theurge entry.
    let build = with_advances(
        &["wizard", "wizard", "wizard", "mystic-theurge", "cleric"],
        "mystic-theurge",
        &["wizard", "cleric"],
    );
    assert_eq!(
        advances_problems(&build, "mystic-theurge"),
        [Issue::NotAllowed]
    );
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
    for advances in ["rogue", "cleric"] {
        let build = with_advances(&rogue_wizard, "arcane-trickster", &[advances]);
        assert_eq!(
            advances_problems(&build, "arcane-trickster"),
            [Issue::NotAllowed],
            "{advances}"
        );
    }
    let build = with_advances(&rogue_wizard, "arcane-trickster", &["wizard"]);
    assert_eq!(advances_problems(&build, "arcane-trickster"), []);
}

#[test]
fn only_a_class_that_advances_spellcasting_takes_advances() {
    let build = with_advances(&["wizard", "fighter"], "fighter", &["wizard"]);
    assert_eq!(advances_problems(&build, "fighter"), [Issue::NotAllowed]);
}
