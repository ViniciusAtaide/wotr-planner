//! Feats that give one more slot of a Feature selection each time they're taken (Extra Rage Power, Extra Hex,
//! ticket 49): the slot is at the level the feat is taken, of the feat's own selection (ticket 306aa).

use std::collections::BTreeSet;

use rules::{
    Build, Choice, Edit, FeatureSelectionKey, Issue, Pick, SelectionOptionKey, SlotId, SlotKind,
};

use crate::support::*;

fn extra_slot(level: u8, index: u8) -> SlotId {
    SlotId::FeatSelection { level, index }
}

fn selection(key: &str) -> SlotKind {
    SlotKind::Selection(FeatureSelectionKey::new(key))
}

/// What the Build's sheet has in a slot, `None` when it has no such slot.
fn kind_of(build: &Build, slot: &SlotId) -> Option<SlotKind> {
    engine()
        .evaluate(build)
        .slot(slot)
        .map(|slot| slot.kind.clone())
}

/// Human rogue 3 with Extra Rogue Talent in the general slot at 3 (rogue talents at 2).
fn rogue_with_extra_talent() -> Build {
    let mut build = build("human", &times("rogue", 3));
    build
        .picks
        .insert(general_slot(3), pick_feat("extra-rogue-talent"));
    build
}

#[test]
fn a_feat_with_an_extra_pick_gives_a_slot_of_its_selection_at_its_level() {
    let engine = engine();
    let build = rogue_with_extra_talent();
    let sheet = engine.evaluate(&build);
    // After the level's other slots.
    let last = sheet.levels[2].slots.last().expect("slots at 3");
    assert_eq!(last.id, extra_slot(3, 0));
    assert_eq!(last.kind, selection("rogue/extra-rogue-talent"));
    assert_eq!(engine.slot_label(last), "Rogue Talents");

    // It offers the rogue talents, as the rogue's own slot at 2 does.
    let offered = |slot: &SlotId| -> BTreeSet<Pick> {
        engine
            .options(&build, slot)
            .expect("a slot")
            .into_iter()
            .map(|option| option.pick)
            .collect()
    };
    assert_eq!(offered(&extra_slot(3, 0)), offered(&selection_slot(2, 0)));

    let mut build = build;
    build.picks.insert(
        extra_slot(3, 0),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    assert!(engine.is_possible(&build));
    let state = engine.evaluate(&build).at(3).unwrap().clone();
    assert_eq!(
        state.option_count(&SelectionOptionKey::new("rogue/rogue-talents/fast-stealth")),
        1
    );
}

#[test]
fn each_time_its_taken_gives_another_slot() {
    // At 3 and 5.
    let mut rogue = build("human", &times("rogue", 5));
    for level in [3, 5] {
        rogue
            .picks
            .insert(general_slot(level), pick_feat("extra-rogue-talent"));
    }
    rogue.picks.insert(
        extra_slot(3, 0),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    rogue.picks.insert(
        extra_slot(5, 0),
        pick_option("rogue/rogue-talents/resiliency"),
    );
    assert!(engine().is_possible(&rogue));

    // Twice at one level: a human witch's racial bonus feat and general feat at 1, after her hex slot.
    let mut witch = build("human", &["witch"]);
    witch.picks.insert(race_slot(0), pick_feat("extra-hex"));
    witch.picks.insert(general_slot(1), pick_feat("extra-hex"));
    assert_eq!(
        kind_of(&witch, &extra_slot(1, 0)),
        Some(selection("witch/extra-hex"))
    );
    assert_eq!(
        kind_of(&witch, &extra_slot(1, 1)),
        Some(selection("witch/extra-hex"))
    );
    witch
        .picks
        .insert(extra_slot(1, 0), pick_option("witch/hex/slumber"));
    witch
        .picks
        .insert(extra_slot(1, 1), pick_option("witch/hex/misfortune"));
    assert!(engine().is_possible(&witch));
}

#[test]
fn the_slot_is_the_feats_own_whatever_slots_the_character_had_first() {
    // Slayer talents at 2, rogue talents at 4, or the other way round: Extra Rogue Talent at 5 gives a slot of its
    // own selection either way.
    for classes in [
        ["slayer", "slayer", "rogue", "rogue", "rogue"],
        ["rogue", "rogue", "slayer", "slayer", "slayer"],
    ] {
        let mut build = build("human", &classes);
        build
            .picks
            .insert(general_slot(5), pick_feat("extra-rogue-talent"));
        assert_eq!(
            kind_of(&build, &extra_slot(5, 0)),
            Some(selection("rogue/extra-rogue-talent")),
            "{classes:?}"
        );
    }
}

#[test]
fn a_character_with_no_slot_of_a_class_selection_still_gets_the_feats_slot() {
    // A rogue 1 has had no rogue talent slot yet; a wizard never has a hex slot: the game opens the selection's
    // slot whenever it is taken (`SelectFeature.Apply`, WrathDecompiled 6ce4a6b).
    let mut rogue = build("human", &["rogue"]);
    rogue
        .picks
        .insert(general_slot(1), pick_feat("extra-rogue-talent"));
    let mut wizard = build("human", &["wizard"]);
    wizard.picks.insert(general_slot(1), pick_feat("extra-hex"));
    for (build, selection_key, option) in [
        (
            rogue,
            "rogue/extra-rogue-talent",
            "rogue/rogue-talents/fast-stealth",
        ),
        (wizard, "witch/extra-hex", "witch/hex/slumber"),
    ] {
        assert!(engine().is_possible(&build));
        assert_eq!(
            kind_of(&build, &extra_slot(1, 0)),
            Some(selection(selection_key))
        );
        let mut with_pick = build;
        with_pick
            .picks
            .insert(extra_slot(1, 0), pick_option(option));
        assert!(engine().is_possible(&with_pick), "{selection_key}");
    }
}

#[test]
fn removing_the_feat_removes_its_slot_and_isnt_offered_while_the_slots_pick_depends_on_it() {
    let engine = engine();
    let mut build = rogue_with_extra_talent();
    build.picks.insert(
        extra_slot(3, 0),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    assert!(engine.is_possible(&build));
    let feat = Choice::Slot(general_slot(3));
    let extra = Choice::Slot(extra_slot(3, 0));
    let removable = engine.removable(&build);
    assert!(removable.contains(&extra));
    assert!(!removable.contains(&feat));
    assert!(!engine.is_removable(&build, &feat));
    assert!(!engine.allows(&build, &Edit::Clear(general_slot(3))));

    // The feat's picker locks every other feat while the slot holds a pick.
    let options = engine.options(&build, &general_slot(3)).unwrap();
    let status = |key: &str| {
        options
            .iter()
            .find(|option| option.pick == pick_feat(key))
            .map(|option| option.status.clone())
    };
    assert_eq!(status("extra-rogue-talent"), Some(Ok(())));
    assert_eq!(status("toughness"), Some(Err(Issue::BreaksLater)));

    // With the slot empty, the feat comes off, and its slot with it.
    build.apply(&Edit::Clear(extra_slot(3, 0)));
    assert!(engine.is_removable(&build, &feat));
    build.apply(&Edit::Clear(general_slot(3)));
    assert!(engine.is_possible(&build));
    assert_eq!(kind_of(&build, &extra_slot(3, 0)), None);
}

#[test]
fn the_pickers_walk_a_feat_with_an_extra_pick_even_when_no_check_names_it() {
    // Rogue talents at 2, slayer talents at 4; at 5 a fighter's bonus Combat feat slot comes before the general slot.
    // Extra Rogue Talent in the general slot gives the rogue talent Resiliency the level's first extra slot. Slayer
    // Trick in the bonus slot would take that slot (a slayer talent) and push Extra Rogue Talent's to the second, so
    // it breaks Resiliency, though no check names it.
    let mut build = build("human", &["rogue", "rogue", "slayer", "slayer", "fighter"]);
    build
        .picks
        .insert(general_slot(5), pick_feat("extra-rogue-talent"));
    build.picks.insert(
        extra_slot(5, 0),
        pick_option("rogue/rogue-talents/resiliency"),
    );
    let engine = engine();
    assert!(engine.is_possible(&build));
    let trick = pick_feat("slayer-trick");
    assert!(!engine.allows(
        &build,
        &Edit::Pick {
            slot: class_slot(5, 0),
            pick: trick.clone(),
        }
    ));
    let options = engine.options(&build, &class_slot(5, 0)).unwrap();
    let status = |pick: &Pick| {
        options
            .iter()
            .find(|option| &option.pick == pick)
            .map(|option| option.status.clone())
    };
    assert_eq!(status(&trick), Some(Err(Issue::BreaksLater)));
    assert_eq!(status(&pick_feat("toughness")), Some(Ok(())));
}
