//! Options' class skills (ticket 37): an option row's skills are class skills while the row is held, so whatever
//! governs the row (its level, an archetype skipping it, a lists-only holding, a replaced row) governs them.

use std::collections::BTreeSet;

use rules::{Build, ClassChoice, Engine, GameData, SlotId};

use crate::support::*;

const ABYSSAL: &str = "sorcerer/bloodline/abyssal";
const ARCANE: &str = "sorcerer/bloodline/arcane";
const RED: &str = "sorcerer/bloodline/draconic-red";

/// `classes` in order, with `archetype` on the sorcerer and `picks` at their slots.
fn with(classes: &[&str], archetype: Option<&str>, picks: &[(SlotId, &str)]) -> Build {
    let mut build = build("elf", classes);
    if let Some(archetype) = archetype {
        build.class_choices.insert(
            class("sorcerer"),
            ClassChoice {
                archetype: Some(rules::ArchetypeKey::new(archetype)),
                ..ClassChoice::default()
            },
        );
    }
    for (slot, key) in picks {
        build.picks.insert(slot.clone(), pick_option(key));
    }
    build
}

fn sorcerer(bloodline: &str, levels: usize) -> Build {
    with(
        &times("sorcerer", levels),
        None,
        &[(selection_slot(1, 0), bloodline)],
    )
}

fn class_skills_with(engine: &Engine, build: &Build, level: u8) -> BTreeSet<String> {
    engine
        .evaluate(build)
        .at(level)
        .unwrap()
        .class_skills
        .iter()
        .map(|skill| skill.to_string())
        .collect()
}

fn class_skills(build: &Build, level: u8) -> BTreeSet<String> {
    class_skills_with(&engine(), build, level)
}

fn strings(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|key| (*key).to_owned()).collect()
}

/// The world with `skills` as the class skills of `bloodline`'s row at `level` (added if it has none there).
fn world_with_row_skills(bloodline: &str, level: u8, skills: &[&str]) -> GameData {
    let mut data = world();
    let sorcerer = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == "sorcerer")
        .unwrap();
    let option = sorcerer
        .selections
        .iter_mut()
        .flat_map(|selection| selection.options.iter_mut())
        .find(|option| option.key.as_str() == bloodline)
        .unwrap();
    let row = match option.levels.iter().position(|row| row.level == level) {
        Some(index) => &mut option.levels[index],
        None => {
            option.levels.push(rules::OptionLevelRow {
                level,
                ..rules::OptionLevelRow::default()
            });
            option.levels.last_mut().unwrap()
        }
    };
    row.class_skills = skills.iter().map(|key| skill(key)).collect();
    data
}

// ---- held with the row ----------------------------------------------------------------------------------------

#[test]
fn an_options_row_makes_its_skills_class_skills() {
    assert_eq!(
        class_skills(&sorcerer(ABYSSAL, 1), 1),
        strings(&["athletics", "knowledge-arcana"])
    );
    assert_eq!(
        class_skills(&sorcerer(RED, 1), 1),
        strings(&["knowledge-arcana", "persuasion"])
    );
}

#[test]
fn a_character_without_the_option_lacks_its_class_skills() {
    assert_eq!(
        class_skills(&sorcerer(ARCANE, 1), 1),
        strings(&["knowledge-arcana"])
    );
    let unpicked = build("elf", &["sorcerer"]);
    assert_eq!(class_skills(&unpicked, 1), strings(&["knowledge-arcana"]));
}

#[test]
fn an_option_rows_class_skills_join_when_its_level_is_reached() {
    let engine = Engine::new(world_with_row_skills(ABYSSAL, 3, &["mobility"]));
    let build = sorcerer(ABYSSAL, 3);
    assert_eq!(
        class_skills_with(&engine, &build, 2),
        strings(&["athletics", "knowledge-arcana"])
    );
    assert_eq!(
        class_skills_with(&engine, &build, 3),
        strings(&["athletics", "knowledge-arcana", "mobility"])
    );
}

#[test]
fn an_option_taken_late_gives_its_class_skills_from_the_pick() {
    // Second Bloodline at MR 1 (character level 6) takes the Abyssal bloodline.
    let mut build = sorcerer(ARCANE, 6);
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("second-bloodline"),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        pick_option(ABYSSAL),
    );
    assert!(engine().is_possible(&build));
    assert_eq!(class_skills(&build, 5), strings(&["knowledge-arcana"]));
    assert_eq!(
        class_skills(&build, 6),
        strings(&["athletics", "knowledge-arcana"])
    );
}

// ---- what governs the row governs its class skills ------------------------------------------------------------

#[test]
fn an_archetype_skipping_the_row_gets_none_of_its_class_skills() {
    // The Sage's bloodline skips the Arcane bloodline's 1st-level row.
    let engine = Engine::new(world_with_row_skills(ARCANE, 1, &["mobility"]));
    assert_eq!(
        class_skills_with(&engine, &sorcerer(ARCANE, 1), 1),
        strings(&["knowledge-arcana", "mobility"])
    );
    let sage = with(&["sorcerer"], Some("sorcerer/sage"), &[]);
    assert!(engine.is_possible(&sage));
    assert_eq!(
        class_skills_with(&engine, &sage, 1),
        strings(&["knowledge-arcana"])
    );
}

#[test]
fn a_second_bloodline_giving_only_its_lists_gives_no_class_skills() {
    let crossblooded = |first: &str, second: &str| {
        with(
            &["sorcerer"],
            Some("sorcerer/crossblooded"),
            &[
                (selection_slot(1, 0), first),
                (selection_slot(1, 1), second),
            ],
        )
    };
    let second = crossblooded(ARCANE, ABYSSAL);
    assert!(engine().is_possible(&second));
    assert_eq!(class_skills(&second, 1), strings(&["knowledge-arcana"]));
    let first = crossblooded(ABYSSAL, ARCANE);
    assert_eq!(
        class_skills(&first, 1),
        strings(&["athletics", "knowledge-arcana"])
    );
}

#[test]
fn a_replaced_row_takes_back_its_class_skills_but_not_those_held_otherwise() {
    // The Abyssal row at 3 gives Mobility and the sorcerer's own Knowledge (Arcana); a Primalist's talents replace it.
    let engine = Engine::new(world_with_row_skills(
        ABYSSAL,
        3,
        &["knowledge-arcana", "mobility"],
    ));
    let primalist = |choice: &str| {
        with(
            &times("sorcerer", 3),
            Some("sorcerer/primalist"),
            &[
                (selection_slot(1, 0), ABYSSAL),
                (
                    selection_slot(3, 0),
                    &format!("sorcerer/primalist/primal-choice/{choice}"),
                ),
            ],
        )
    };
    assert_eq!(
        class_skills_with(&engine, &primalist("bloodline-power"), 3),
        strings(&["athletics", "knowledge-arcana", "mobility"])
    );
    assert_eq!(
        class_skills_with(&engine, &primalist("talents"), 3),
        strings(&["athletics", "knowledge-arcana"])
    );
}
