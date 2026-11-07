//! An archetype's save progressions (ticket 90): the Arcane Enforcer has good Reflex and Will and poor Fortitude
//! ("High saving throws for Reflex and Will, low saving throws for Fortitude"), where the slayer has good
//! Fortitude and Reflex and poor Will.

use rules::{CharacterState, ClassKey, SaveProgression};

use crate::support::*;

fn saves(state: &CharacterState) -> (u8, u8, u8) {
    (state.fort, state.reflex, state.will)
}

#[test]
fn the_arcane_enforcer_has_good_reflex_and_will_and_poor_fortitude() {
    let enforcer = enforcer(13);
    // Poor ⌊L/3⌋, good 2 + ⌊L/2⌋.
    assert_eq!(saves(&at(&enforcer, 1)), (0, 2, 2));
    assert_eq!(saves(&at(&enforcer, 13)), (4, 8, 8));
}

#[test]
fn a_plain_slayer_keeps_good_fortitude_and_reflex_and_poor_will() {
    let slayer = build("slayer", 13);
    assert_eq!(saves(&at(&slayer, 1)), (2, 2, 0));
    assert_eq!(saves(&at(&slayer, 13)), (8, 8, 4));
}

/// The Enforcer Build from ticket 88's QA, at level 1 (the ticket's repro link), which read Fort +2 / Ref +2 /
/// Will +0.
#[test]
fn the_qa_enforcer_build_has_the_arcane_enforcers_saves() {
    let build = engine()
        .open_link(
            "AQFQAK__DFwlwpHwUQ-GdkLAnXq5_aYx6Q-Gdo0MN_9IBaYx6WEsyN0dGAABAAEDAQEQEA4KCAwBAAMBAQEBAQAAAAIAAgEAAAABAAIAAAABAQEAAQA",
        )
        .expect("the QA link opens");
    let at_1 = at(&build, 1);
    assert_eq!(at_1.class_level(&ClassKey::new("slayer")), 1);
    assert_eq!(saves(&at_1), (0, 2, 2));
}

/// The guide gives no other archetype saves of its own (a sweep of every "saving throws for" sentence in
/// `guide-cache/`).
#[test]
fn only_the_arcane_enforcer_replaces_a_save_progression() {
    let replacing: Vec<_> = DATA
        .classes
        .iter()
        .flat_map(|class| &class.archetypes)
        .filter(|archetype| {
            archetype.fort.is_some() || archetype.reflex.is_some() || archetype.will.is_some()
        })
        .map(|archetype| {
            (
                archetype.key.as_str(),
                archetype.fort,
                archetype.reflex,
                archetype.will,
            )
        })
        .collect();
    assert_eq!(
        replacing,
        [(
            "slayer/arcane-enforcer",
            Some(SaveProgression::Poor),
            None,
            Some(SaveProgression::Good)
        )]
    );
}
