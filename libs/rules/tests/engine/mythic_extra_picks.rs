//! Mythic feats that give one more pick at the Mythic rank they're taken (ticket 55): Extra Feat a feat slot that
//! offers what the general slot does, Extra Mythic Ability a mythic ability slot that offers what a rank's general
//! pick does. The slot is [`SlotId::MythicExtra`], after the rank's general pick.

use std::collections::BTreeSet;

use rules::{
    Build, Choice, Edit, FeatSlot, FeatureSelectionKey, Issue, Pick, RankStatus, SlotId, SlotKind,
};

use crate::support::*;

fn general_mythic(rank: u8) -> SlotId {
    SlotId::Mythic { rank }
}

fn extra(rank: u8) -> SlotId {
    SlotId::MythicExtra { rank }
}

/// What the Build's sheet has in a slot, `None` when it has no such slot.
fn kind_of(build: &Build, slot: &SlotId) -> Option<SlotKind> {
    engine()
        .evaluate(build)
        .slot(slot)
        .map(|slot| slot.kind.clone())
}

/// The picks a slot's picker offers, open or not.
fn offered(build: &Build, slot: &SlotId) -> BTreeSet<Pick> {
    engine()
        .options(build, slot)
        .expect("a slot")
        .into_iter()
        .map(|option| option.pick)
        .collect()
}

/// A human `class` 7 (MR 1 at 6, MR 2 at 7) with `mythic_feat` as MR 2's general pick.
fn with_mythic_feat(class: &'static str, mythic_feat: &str) -> Build {
    let mut build = build("human", &times(class, 7));
    build
        .picks
        .insert(general_mythic(2), pick_mythic_feat(mythic_feat));
    build
}

#[test]
fn extra_feat_gives_a_feat_slot_at_its_rank_offering_what_the_general_slot_does() {
    let engine = engine();
    let build = with_mythic_feat("fighter", "extra-feat");
    let sheet = engine.evaluate(&build);
    let slots: Vec<(SlotId, SlotKind)> = sheet.ranks[1]
        .slots
        .iter()
        .map(|slot| (slot.id.clone(), slot.kind.clone()))
        .collect();
    assert_eq!(
        slots,
        [
            (general_mythic(2), SlotKind::MythicFeat),
            (extra(2), SlotKind::Feat(FeatSlot::General)),
        ]
    );
    assert_eq!(engine.slot_label(&sheet.ranks[1].slots[1]), "Feat");
    // The feats the level's general slot offers.
    assert_eq!(
        offered(&build, &extra(2)),
        offered(&build, &general_slot(7))
    );

    // A pick there is held, its prerequisites checked.
    let mut with_pick = build.clone();
    with_pick.picks.insert(extra(2), pick_feat("power-attack"));
    assert!(engine.is_possible(&with_pick));
    assert_eq!(
        engine.evaluate(&with_pick).at(7).unwrap().feats[&feat("power-attack")],
        1
    );
    let mut unmet = build;
    unmet.picks.insert(extra(2), pick_feat("great-cleave"));
    assert!(matches!(why_not(&unmet, &extra(2)), Some(Issue::Unmet(_))));
}

#[test]
fn extra_mythic_ability_gives_a_mythic_ability_slot_offering_what_a_ranks_does() {
    let engine = engine();
    let build = with_mythic_feat("fighter", "extra-mythic-ability");
    let sheet = engine.evaluate(&build);
    let slot = sheet.slot(&extra(2)).expect("the extra slot");
    assert_eq!(slot.kind, SlotKind::MythicAbility);
    assert_eq!(engine.slot_label(slot), "Mythic ability");
    assert_eq!(
        offered(&build, &extra(2)),
        offered(&build, &general_mythic(1))
    );

    let mut with_pick = build;
    with_pick
        .picks
        .insert(extra(2), pick_mythic_ability("mythic-charge"));
    assert!(engine.is_possible(&with_pick));
    // Not the one MR 1 took.
    with_pick
        .picks
        .insert(general_mythic(1), pick_mythic_ability("mythic-charge"));
    assert_eq!(why_not(&with_pick, &extra(2)), Some(Issue::AlreadyHeld));
}

#[test]
fn a_mythic_ability_in_the_extra_slot_grants_its_selection_slots() {
    // Second Bloodline through Extra Mythic Ability: another bloodline at MR 2.
    let mut build = with_mythic_feat("sorcerer", "extra-mythic-ability");
    build
        .picks
        .insert(extra(2), pick_mythic_ability("second-bloodline"));
    let second = SlotId::MythicSelection { rank: 2, index: 0 };
    assert_eq!(
        kind_of(&build, &second),
        Some(SlotKind::Selection(FeatureSelectionKey::new(
            "sorcerer/bloodline"
        )))
    );
    build
        .picks
        .insert(second, pick_option("sorcerer/bloodline/abyssal"));
    assert!(engine().is_possible(&build));
}

#[test]
fn a_feat_with_an_extra_pick_in_the_extra_slot_gives_its_slot_at_the_ranks_level() {
    // A rogue 7 (talents at 2, 4, 6) takes Extra Rogue Talent through Extra Feat at MR 2: a rogue talent at 7.
    let mut build = with_mythic_feat("rogue", "extra-feat");
    build
        .picks
        .insert(extra(2), pick_feat("extra-rogue-talent"));
    let talent = SlotId::FeatSelection { level: 7, index: 0 };
    assert_eq!(
        kind_of(&build, &talent),
        Some(SlotKind::Selection(FeatureSelectionKey::new(
            "rogue/rogue-talents"
        )))
    );
    build
        .picks
        .insert(talent, pick_option("rogue/rogue-talents/fast-stealth"));
    assert!(engine().is_possible(&build));
}

#[test]
fn other_mythic_feats_give_no_slot() {
    let build = with_mythic_feat("fighter", "dodge-mythic");
    assert_eq!(kind_of(&build, &extra(2)), None);
    let mut with_pick = build;
    with_pick.picks.insert(extra(2), pick_feat("dodge"));
    assert!(!engine().is_possible(&with_pick));
}

#[test]
fn removing_the_mythic_feat_is_refused_while_its_slot_holds_a_pick_and_takes_the_slot_with_it() {
    let engine = engine();
    for (mythic_feat, pick) in [
        ("extra-feat", pick_feat("dodge")),
        ("extra-mythic-ability", pick_mythic_ability("mythic-charge")),
    ] {
        let mut build = with_mythic_feat("fighter", mythic_feat);
        build.picks.insert(extra(2), pick);
        assert!(engine.is_possible(&build));
        let feat = Choice::Slot(general_mythic(2));
        let removable = engine.removable(&build);
        assert!(removable.contains(&Choice::Slot(extra(2))));
        assert!(!removable.contains(&feat), "{mythic_feat}");
        assert!(!engine.is_removable(&build, &feat));
        assert!(!engine.allows(&build, &Edit::Clear(general_mythic(2))));
        // The rank's picker locks every other mythic feat while the slot holds a pick.
        let options = engine.options(&build, &general_mythic(2)).unwrap();
        let status = |key: &str| {
            options
                .iter()
                .find(|option| option.pick == pick_mythic_feat(key))
                .map(|option| option.status.clone())
        };
        assert_eq!(status(mythic_feat), Some(Ok(())));
        assert_eq!(status("dodge-mythic"), Some(Err(Issue::BreaksLater)));

        // With the slot empty, the mythic feat comes off, and its slot with it.
        build.apply(&Edit::Clear(extra(2)));
        assert!(engine.is_removable(&build, &feat));
        build.apply(&Edit::Clear(general_mythic(2)));
        assert!(engine.is_possible(&build));
        assert_eq!(kind_of(&build, &extra(2)), None);
    }
}

#[test]
fn a_switch_that_drops_the_ranks_general_pick_drops_the_extra_slots_pick_too() {
    // Extra Feat at MR 4 (level 11) with Dodge; a Legend keeps only MR 1-2's general picks.
    let mut build = build("human", &times("fighter", 20));
    build.mythic_path = Some(path("trickster"));
    build
        .picks
        .insert(general_mythic(4), pick_mythic_feat("extra-feat"));
    build.picks.insert(extra(4), pick_feat("dodge"));
    engine().apply(&mut build, &Edit::PathSwitch(Some(path("legend"))));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    assert!(sheet.at(16).unwrap().feats.contains_key(&feat("dodge")));
    assert!(!sheet.at(17).unwrap().feats.contains_key(&feat("dodge")));
    let rank_4 = &sheet.ranks[3];
    assert_eq!(rank_4.status, RankStatus::Dropped);
    assert!(rank_4.dropped_slots.contains(&extra(4)));
}
