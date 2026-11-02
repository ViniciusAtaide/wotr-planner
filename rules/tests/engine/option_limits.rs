//! How often an option may be held: one option of a group (one Draconic bloodline, whichever slot takes it), an
//! element at most twice, and Expanded Element never the element it gave at 7 again.

use rules::{Build, CharacterState, ClassChoice, Issue, OptionGroup, Prerequisite, SlotId, Unmet};

use crate::support::*;

/// `classes` in order, with `archetype` on `of` and option `picks` at their slots.
fn with(classes: &[&str], archetype: Option<(&str, &str)>, picks: &[(SlotId, &str)]) -> Build {
    let mut build = build("elf", classes);
    if let Some((of, archetype)) = archetype {
        build.class_choices.insert(
            class(of),
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

fn at(build: &Build, level: u8) -> CharacterState {
    engine().evaluate(build).at(level).cloned().unwrap()
}

fn status(build: &Build, slot: &SlotId, key: &str) -> Result<(), Issue> {
    engine()
        .options(build, slot)
        .unwrap()
        .into_iter()
        .find(|o| o.pick.key() == key)
        .unwrap_or_else(|| panic!("no option {key}"))
        .status
}

const ABYSSAL: &str = "sorcerer/bloodline/abyssal";
const GOLD: &str = "sorcerer/bloodline/draconic-gold";
const RED: &str = "sorcerer/bloodline/draconic-red";

fn one_draconic() -> Issue {
    Issue::Unmet(Unmet::Condition {
        needs: Prerequisite::NoOtherOptionOfGroup(OptionGroup::Draconic),
        has: None,
    })
}

// ---- option groups ------------------------------------------------------------------------------------------

fn crossblooded(first: &str, second: &str, levels: usize) -> Build {
    with(
        &times("sorcerer", levels),
        Some(("sorcerer", "sorcerer/crossblooded")),
        &[
            (selection_slot(1, 0), first),
            (selection_slot(1, 1), second),
        ],
    )
}

#[test]
fn a_crossblooded_sorcerer_cant_take_a_second_draconic_bloodline() {
    let build = crossblooded(GOLD, RED, 1);
    assert_eq!(
        status(&build, &selection_slot(1, 1), RED),
        Err(one_draconic())
    );
    assert_eq!(status(&build, &selection_slot(1, 1), ABYSSAL), Ok(()));
    // The first pick saw no other dragon.
    assert_eq!(status(&build, &selection_slot(1, 0), GOLD), Ok(()));
    assert_eq!(why_not(&build, &selection_slot(1, 1)), Some(one_draconic()));
}

#[test]
fn second_bloodline_cant_take_a_second_draconic_bloodline() {
    let mut build = with(&times("sorcerer", 6), None, &[(selection_slot(1, 0), RED)]);
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("second-bloodline"),
    );
    let slot = SlotId::MythicSelection { rank: 1, index: 0 };
    assert_eq!(status(&build, &slot, GOLD), Err(one_draconic()));
    assert_eq!(status(&build, &slot, ABYSSAL), Ok(()));
}

#[test]
fn an_option_of_a_group_is_held_in_the_states_groups() {
    let state = at(&crossblooded(RED, ABYSSAL, 1), 1);
    assert_eq!(
        state.option_groups,
        [(option(RED), OptionGroup::Draconic)].into()
    );
    assert!(state.holds_other_option_of(OptionGroup::Draconic, Some(&option(GOLD))));
    assert!(!state.holds_other_option_of(OptionGroup::Draconic, Some(&option(RED))));
}

// ---- Expanded Element -----------------------------------------------------------------------------------------

const AIR: &str = "kineticist/element/air";
const FIRE: &str = "kineticist/element/fire";
const WATER: &str = "kineticist/element/water";

fn kineticist(first: &str, seventh: &str) -> Build {
    with(
        &times("kineticist", 15),
        None,
        &[
            (selection_slot(1, 0), first),
            (selection_slot(7, 0), seventh),
        ],
    )
}

#[test]
fn expanded_element_at_15_cant_take_the_7th_level_element_again() {
    let build = kineticist(AIR, FIRE);
    assert!(engine().is_possible(&build));
    let fifteenth = selection_slot(15, 0);
    assert_eq!(status(&build, &fifteenth, FIRE), Err(Issue::AlreadyHeld));
    // The 1st-level element (a second time) or a third.
    assert_eq!(status(&build, &fifteenth, AIR), Ok(()));
    assert_eq!(status(&build, &fifteenth, WATER), Ok(()));
    assert!(at(&build, 7).holds_through(
        &rules::FeatureSelectionKey::new("kineticist/expanded-element"),
        &option(FIRE)
    ));
}

#[test]
fn an_element_doubled_at_7_isnt_taken_a_third_time() {
    let build = kineticist(AIR, AIR);
    assert!(engine().is_possible(&build));
    let fifteenth = selection_slot(15, 0);
    assert_eq!(
        status(&build, &fifteenth, AIR),
        Err(Issue::MaxTimes { times: 2 })
    );
    assert_eq!(status(&build, &fifteenth, FIRE), Ok(()));
}
