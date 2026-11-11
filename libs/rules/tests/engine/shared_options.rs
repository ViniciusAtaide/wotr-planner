//! Shared selection options: a selection offers its own options and the ones it shares from other selections (a
//! slayer's talents include the rogue's). A shared option keeps the one key of the selection that defines it, so
//! a character holds it once, whichever class's slot took it.

use rules::{Build, Engine, Issue, Pick, Prerequisite, SlotId, SlotOption, Unmet};

use crate::support::*;

fn options(engine: &Engine, build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine
        .options(build, slot)
        .unwrap_or_else(|| panic!("the Build has no slot {slot:?}"))
}

fn status_of(options: &[SlotOption], pick: &Pick) -> Result<(), Issue> {
    options
        .iter()
        .find(|option| &option.pick == pick)
        .unwrap_or_else(|| panic!("{pick:?} isn't offered"))
        .status
        .clone()
}

/// Rogue 2, then slayer 2: a rogue talent slot at character level 2, a slayer talent slot at 4.
fn rogue_then_slayer() -> Build {
    build("elf", &["rogue", "rogue", "slayer", "slayer"])
}

#[test]
fn a_slayer_talent_slot_offers_its_own_options_and_the_shared_rogue_talents_in_key_order() {
    let build = build("elf", &times("slayer", 2));
    let offered = options(&engine(), &build, &selection_slot(2, 0));
    let keys: Vec<_> = offered.iter().map(|o| o.pick.key()).collect();
    assert_eq!(
        keys,
        [
            "rogue/rogue-talents/combat-trick",
            "rogue/rogue-talents/crippling-strike",
            "rogue/rogue-talents/fast-stealth",
            // Battle Master once per weapon.
            "slayer/slayer-talents/battle-master",
            "slayer/slayer-talents/battle-master",
            "slayer/slayer-talents/battle-master",
            "slayer/slayer-talents/battle-master",
            "slayer/slayer-talents/battle-master",
            "slayer/slayer-talents/combat-style-feat-10",
            "slayer/slayer-talents/combat-style-feat-2",
            "slayer/slayer-talents/combat-style-feat-6",
            "slayer/slayer-talents/slayer-camouflage",
        ]
    );
}

#[test]
fn a_shared_talent_taken_in_a_slayer_slot_is_held_under_the_rogue_talents_key() {
    let mut build = build("elf", &times("slayer", 2));
    build.picks.insert(
        selection_slot(2, 0),
        pick_option("rogue/rogue-talents/combat-trick"),
    );
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let state = sheet.at(2).unwrap();
    assert!(
        state
            .selection_options
            .contains_key(&option("rogue/rogue-talents/combat-trick"))
    );
    // The option's feat comes with it, as in the rogue's own slot.
    assert!(state.feats.contains_key(&feat("combat-expertise")));
}

#[test]
fn a_rogue_slayer_cannot_take_one_talent_twice() {
    let fast_stealth = pick_option("rogue/rogue-talents/fast-stealth");

    // Rogue first, then slayer.
    let mut build = rogue_then_slayer();
    build
        .picks
        .insert(selection_slot(2, 0), fast_stealth.clone());
    assert_eq!(
        status_of(
            &options(&engine(), &build, &selection_slot(4, 0)),
            &fast_stealth
        ),
        Err(Issue::AlreadyHeld)
    );
    build
        .picks
        .insert(selection_slot(4, 0), fast_stealth.clone());
    assert_eq!(
        why_not(&build, &selection_slot(4, 0)),
        Some(Issue::AlreadyHeld)
    );

    // Slayer first, then rogue.
    let mut build = build_of(&["slayer", "slayer", "rogue", "rogue"]);
    build
        .picks
        .insert(selection_slot(2, 0), fast_stealth.clone());
    build
        .picks
        .insert(selection_slot(4, 0), fast_stealth.clone());
    assert_eq!(
        why_not(&build, &selection_slot(4, 0)),
        Some(Issue::AlreadyHeld)
    );
}

#[test]
fn an_own_option_that_needs_a_shared_one_sees_it_taken_in_either_classs_slot() {
    let mut build = rogue_then_slayer();
    build.picks.insert(
        selection_slot(2, 0),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    build.picks.insert(
        selection_slot(4, 0),
        pick_option("slayer/slayer-talents/slayer-camouflage"),
    );
    assert!(engine().is_possible(&build));

    // Without Fast Stealth it's unmet, and dropping the rogue's pick would break it.
    build.picks.remove(&selection_slot(2, 0));
    assert_eq!(
        why_not(&build, &selection_slot(4, 0)),
        Some(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::SelectionOption(option("rogue/rogue-talents/fast-stealth")),
            has: None,
        }))
    );
}

#[test]
fn a_selection_refuses_an_option_it_neither_defines_nor_shares() {
    let mut build = build("elf", &times("rogue", 2));
    let camouflage = pick_option("slayer/slayer-talents/slayer-camouflage");
    let offered = options(&engine(), &build, &selection_slot(2, 0));
    assert!(offered.iter().all(|o| o.pick != camouflage));
    build.picks.insert(selection_slot(2, 0), camouflage);
    assert_eq!(why_not(&build, &selection_slot(2, 0)), None);
}

/// The data makes a class-dependent prerequisite an `Any` over each class's version (decision S4, option (a)).
/// The accepted trade-off: a rogue 10 / slayer 2 can then take the option through the slayer's slot.
#[test]
fn a_shared_options_any_over_each_classs_level_lets_either_class_qualify() {
    let mut data = world();
    let rogue = data
        .classes
        .iter_mut()
        .find(|c| c.key == class("rogue"))
        .unwrap();
    let crippling_strike = rogue.selections[0]
        .options
        .iter_mut()
        .find(|o| o.key == option("rogue/rogue-talents/crippling-strike"))
        .unwrap();
    crippling_strike.prerequisite = Some(Prerequisite::Any(vec![
        Prerequisite::ClassLevel {
            class: class("rogue"),
            level: 10,
        },
        Prerequisite::ClassLevel {
            class: class("slayer"),
            level: 10,
        },
    ]));
    let engine = engine_on(data);
    let crippling_strike = pick_option("rogue/rogue-talents/crippling-strike");

    // A slayer 10 qualifies through the slayer's level.
    let slayer = build("elf", &times("slayer", 10));
    assert_eq!(
        status_of(
            &options(&engine, &slayer, &selection_slot(10, 0)),
            &crippling_strike
        ),
        Ok(())
    );

    // A rogue 10 / slayer 2 qualifies in the slayer's slot through the rogue's level.
    let mut classes = times("rogue", 10);
    classes.extend(times("slayer", 2));
    let multiclass = build("elf", &classes);
    assert_eq!(
        status_of(
            &options(&engine, &multiclass, &selection_slot(12, 0)),
            &crippling_strike
        ),
        Ok(())
    );

    // A slayer 8 doesn't.
    let short = build("elf", &times("slayer", 8));
    assert!(matches!(
        status_of(
            &options(&engine, &short, &selection_slot(8, 0)),
            &crippling_strike
        ),
        Err(Issue::Unmet(_))
    ));
}

fn build_of(classes: &[&str]) -> Build {
    build("elf", classes)
}
