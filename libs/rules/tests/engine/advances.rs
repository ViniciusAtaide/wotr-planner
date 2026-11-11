//! Prestige spellcasting advancement: which classes a prestige class may advance, and what that gives.

use std::collections::BTreeSet;

use rules::{Build, ClassChoice, ClassKey, ClassOption, Edit, FeatSource, Issue, Pick, SlotId};

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

/// Each advancement option's classes.
fn advances(options: &[rules::AdvancesOption]) -> Vec<BTreeSet<ClassKey>> {
    options
        .iter()
        .map(|option| option.advances.clone())
        .collect()
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
        advances(&engine().advances_options(&build, 8, &theurge)),
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
        advances(&engine().advances_options(&build, 7, &winter_witch)),
        sets(&[&["wizard"], &["cleric"], &[]])
    );
}

/// Ticket 150, disagreement 1: the class picker judged a prestige class that asks what it advances by the
/// advancement the Build holds for it (none, for a class it doesn't take), so it locked one that advancing the class
/// it replaces would keep, and the advancement picker that offers that advancement was out of reach.
#[test]
fn the_class_picker_opens_a_prestige_class_that_some_advancement_keeps() {
    // Cleric 5 with Spell Penetration (caster level 5) in the general slot at level 5.
    let mut build = build("elf", &["cleric"; 5]);
    let spell_penetration = SlotId::Feat {
        level: 5,
        source: FeatSource::General,
        index: 0,
    };
    build.picks.insert(
        spell_penetration.clone(),
        Pick::feat(feat("spell-penetration")),
    );
    let engine = engine();
    assert!(engine.is_possible(&build));

    // Winter Witch in place of the 4th cleric level keeps it only advancing the cleric.
    let winter_witch = class_option(&build, 4, "winter-witch");
    assert_eq!(winter_witch.status, Ok(()));
    assert_eq!(
        advances(&engine.advances_options(&build, 4, &winter_witch)),
        sets(&[&["cleric"]])
    );
    let without = Edit::Class {
        level: 4,
        class: class("winter-witch"),
        choice: Some(ClassChoice::default()),
    };
    assert!(!engine.allows(&build, &without));

    // Without the feat, every advancement keeps the Build.
    build.picks.remove(&spell_penetration);
    assert_eq!(
        advances(&engine.advances_options(&build, 4, &winter_witch)),
        sets(&[&["cleric"], &[]])
    );
}

/// Ticket 150, decision 5, the other way: a prestige class that asks what it advances stays locked when no
/// advancement keeps the Build, here because the level it replaces is one a later pick needs whatever it casts.
#[test]
fn the_class_picker_locks_a_prestige_class_that_no_advancement_keeps() {
    // Wizard 3 / Fighter 4, with Weapon Specialization (fighter level 4) in the general slot at level 7.
    let mut build = build(
        "elf",
        &[
            "wizard", "wizard", "wizard", "fighter", "fighter", "fighter", "fighter",
        ],
    );
    build.picks.insert(
        general_slot(5),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build.picks.insert(
        general_slot(7),
        pick_feat_for("weapon-specialization", weapon("longsword")),
    );
    let engine = engine();
    assert!(engine.is_possible(&build));

    // Winter Witch in place of the 3rd fighter level leaves fighter 3 at level 7, however it advances.
    let winter_witch = class_option(&build, 6, "winter-witch");
    assert_eq!(winter_witch.status, Err(Issue::BreaksLater));
    assert!(engine.advances_options(&build, 6, &winter_witch).is_empty());

    // Without the feat, it's open.
    build.picks.remove(&general_slot(7));
    let winter_witch = class_option(&build, 6, "winter-witch");
    assert_eq!(winter_witch.status, Ok(()));
    assert!(!engine.advances_options(&build, 6, &winter_witch).is_empty());
}

/// Ticket 150, decision 5, for a prestige class that advances one class, arcane or divine (`Either`): it's open
/// when advancing the class it replaces keeps the Build, whichever spell type that class casts, and the
/// advancement picker offers only that class.
#[test]
fn a_prestige_class_advancing_either_spell_type_is_open_through_the_class_it_must_advance() {
    // Mystic Theurge at level 7 needs 2nd-level arcane and divine spells. Winter Witch in place of the 3rd level of
    // one of its two classes keeps that only by advancing that class.
    for (classes, replaced) in [
        (
            [
                "wizard",
                "wizard",
                "wizard",
                "cleric",
                "cleric",
                "cleric",
                "mystic-theurge",
            ],
            "cleric",
        ),
        (
            [
                "cleric",
                "cleric",
                "cleric",
                "wizard",
                "wizard",
                "wizard",
                "mystic-theurge",
            ],
            "wizard",
        ),
    ] {
        let build = with_advances(&classes, "mystic-theurge", &["wizard", "cleric"]);
        let engine = engine();
        assert!(engine.is_possible(&build), "{replaced}");

        let winter_witch = class_option(&build, 6, "winter-witch");
        assert_eq!(winter_witch.status, Ok(()), "{replaced}");
        assert_eq!(
            advances(&engine.advances_options(&build, 6, &winter_witch)),
            sets(&[&[replaced]]),
        );
    }
}

/// Ticket 150's review: the class picker's status for a prestige class that asks what it advances and the
/// advancement picker come from one routine, so they agree even on a Build that isn't possible (which no picker is
/// asked about): the Winter Witch the Build holds is closed there, as no advancement makes the Build possible.
#[test]
fn the_class_and_advancement_pickers_agree_on_a_build_that_is_not_possible() {
    let mut build = with_advances(&["cleric"; 5], "winter-witch", &["cleric"]);
    build.levels[3] = rules::LevelUp::new(class("winter-witch"));
    let engine = engine();
    assert!(engine.is_possible(&build));
    let winter_witch = class_option(&build, 4, "winter-witch");
    assert_eq!(winter_witch.status, Ok(()));
    assert_eq!(
        advances(&engine.advances_options(&build, 4, &winter_witch)),
        sets(&[&["cleric"], &[]])
    );

    // Weapon Specialization in the general slot at level 5 fails: no fighter levels.
    build.picks.insert(
        general_slot(5),
        pick_feat_for("weapon-specialization", weapon("longsword")),
    );
    assert!(!engine.is_possible(&build));
    let winter_witch = class_option(&build, 4, "winter-witch");
    assert!(engine.advances_options(&build, 4, &winter_witch).is_empty());
    assert_eq!(winter_witch.status, Err(Issue::BreaksLater));
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
fn a_class_option_sets_the_choice_only_where_the_class_is_first_taken() {
    let mut build = with_advances(&THEURGE, "mystic-theurge", &["wizard", "cleric"]);
    let engine = engine();
    let theurge = class_option(&build, 7, "mystic-theurge");
    let choice = |advances: &[&str]| {
        Some(ClassChoice {
            archetype: None,
            advances: advances.iter().map(|key| class(key)).collect(),
        })
    };
    // First taken at 7: the option keeps the advancement the Build has, and each advancement option sets its own.
    assert_eq!(
        theurge.edit,
        Edit::Class {
            level: 7,
            class: class("mystic-theurge"),
            choice: choice(&["wizard", "cleric"]),
        }
    );
    let options = engine.advances_options(&build, 7, &theurge);
    assert!(!options.is_empty());
    for option in options {
        let classes: Vec<&str> = option.advances.iter().map(ClassKey::as_str).collect();
        assert_eq!(
            option.edit,
            Edit::Class {
                level: 7,
                class: class("mystic-theurge"),
                choice: choice(&classes),
            }
        );
    }
    // Taken before level 9: no choice.
    build
        .levels
        .push(rules::LevelUp::new(class("mystic-theurge")));
    assert_eq!(
        class_option(&build, 9, "mystic-theurge").edit,
        Edit::Class {
            level: 9,
            class: class("mystic-theurge"),
            choice: None,
        }
    );
}

#[test]
fn a_class_option_gives_a_class_first_taken_its_archetype() {
    let build = build("elf", &[]);
    let options = engine().class_options(&build, 1).unwrap();
    let zen_archer = options
        .iter()
        .find(|option| option.archetype == Some(rules::ArchetypeKey::new("monk/zen-archer")))
        .unwrap();
    assert_eq!(
        zen_archer.edit,
        Edit::Class {
            level: 1,
            class: class("monk"),
            choice: Some(ClassChoice {
                archetype: Some(rules::ArchetypeKey::new("monk/zen-archer")),
                advances: BTreeSet::new(),
            }),
        }
    );
}
