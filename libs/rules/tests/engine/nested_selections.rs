//! Nested selections (ticket 302): a picked feat, mythic feat or mythic ability that names a selection opens a slot of
//! it, as the game adds a picked selection to the level-up's selections (`SelectFeature.Apply`, WrathDecompiled
//! 6ce4a6b). The slot is of the selection itself when it lists its own picks; one naming only the selections a
//! character must have had a slot of first gives a slot of the first of those it had (the planner's form until 293).

use std::collections::BTreeSet;

use rules::{Build, GameData, Issue, MythicAbilityKey, Pick, SlotId, SlotKind};

use crate::support::*;

fn extra_slot(level: u8) -> SlotId {
    SlotId::FeatSelection { level, index: 0 }
}

fn offered(engine: &rules::Engine, build: &Build, slot: &SlotId) -> BTreeSet<Pick> {
    engine
        .options(build, slot)
        .expect("a slot")
        .into_iter()
        .map(|option| option.pick)
        .collect()
}

/// Extra Rogue Talent as a selection listing the rogue talents itself, after a rogue talent slot.
fn with_own_list(data: &mut GameData) {
    let talents = selection_mut(data, "rogue/rogue-talents").clone();
    let own = selection_mut(data, "rogue/extra-rogue-talent");
    own.shared_options = talents.offered().into_iter().cloned().collect();
    own.name = talents.name.clone();
    own.needs_slot_of = BTreeSet::from([feat_list("rogue/rogue-talents")]);
}

#[test]
fn a_feat_granting_a_selection_with_its_own_list_gives_a_slot_of_that_selection() {
    let engine = engine_with(with_own_list);
    let mut rogue = build("human", &times("rogue", 3));
    rogue
        .picks
        .insert(general_slot(3), pick_feat("extra-rogue-talent"));
    let sheet = engine.evaluate(&rogue);
    let slot = sheet.slot(&extra_slot(3)).expect("the nested slot");
    assert_eq!(
        slot.kind,
        SlotKind::Selection(feat_list("rogue/extra-rogue-talent"))
    );
    assert_eq!(engine.slot_label(slot), "Rogue Talents");
    assert_eq!(
        offered(&engine, &rogue, &extra_slot(3)),
        offered(&engine, &rogue, &selection_slot(2, 0))
    );
    rogue.picks.insert(
        extra_slot(3),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    assert!(engine.is_possible(&rogue));
}

#[test]
fn a_selection_needing_a_slot_not_had_gives_none() {
    // A slayer 3 had no rogue talent slot: the own-list selection that needs one gives no slot, as before.
    let engine = engine_with(with_own_list);
    let mut slayer = build("human", &times("slayer", 3));
    slayer
        .picks
        .insert(general_slot(3), pick_feat("extra-rogue-talent"));
    assert!(engine.evaluate(&slayer).slot(&extra_slot(3)).is_none());
    // Without the condition, the selection is the slot.
    let engine = engine_with(|data| {
        with_own_list(data);
        selection_mut(data, "rogue/extra-rogue-talent")
            .needs_slot_of
            .clear();
    });
    let sheet = engine.evaluate(&slayer);
    assert_eq!(
        sheet.slot(&extra_slot(3)).map(|slot| slot.kind.clone()),
        Some(SlotKind::Selection(feat_list("rogue/extra-rogue-talent")))
    );
}

#[test]
fn a_first_had_selection_gives_a_slot_of_the_first_selection_had() {
    // The test world's Extra Rogue Talent lists none of its own: a rogue's slot is a rogue talent's, a slayer's a
    // slayer talent's.
    let engine = engine();
    for (class, selection) in [
        ("rogue", "rogue/rogue-talents"),
        ("slayer", "slayer/slayer-talents"),
    ] {
        let mut build = build("human", &times(class, 3));
        build
            .picks
            .insert(general_slot(3), pick_feat("extra-rogue-talent"));
        assert_eq!(
            engine
                .evaluate(&build)
                .slot(&extra_slot(3))
                .map(|slot| slot.kind.clone()),
            Some(SlotKind::Selection(feat_list(selection))),
            "{class}"
        );
    }
}

/// A human fighter 7 with Extra Mythic Ability as MR 2's general pick.
fn with_extra_mythic_ability() -> Build {
    let mut build = build("human", &times("fighter", 7));
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        pick_mythic_feat("extra-mythic-ability"),
    );
    build
}

#[test]
fn extra_mythic_abilitys_slot_offers_its_selections_list() {
    let engine = engine();
    let build = with_extra_mythic_ability();
    let extra = SlotId::MythicExtra { rank: 2 };
    let sheet = engine.evaluate(&build);
    let slot = sheet.slot(&extra).expect("the extra slot");
    assert_eq!(
        slot.kind,
        SlotKind::MythicAbilities(feat_list("mythic/extra-mythic-ability"))
    );
    assert_eq!(engine.slot_label(slot), "Mythic ability");
    // Every mythic ability, as a rank's general pick offers.
    assert_eq!(
        offered(&engine, &build, &extra),
        offered(&engine, &build, &SlotId::Mythic { rank: 1 })
    );

    // A shorter list offers only its own, and refuses the others.
    let engine = engine_with(|data| {
        let short = rules::FeatureSelection {
            key: feat_list("mythic/short-list"),
            mythic_abilities: BTreeSet::from([MythicAbilityKey::new("unstoppable")]),
            ..selection_mut(data, "mythic/extra-mythic-ability").clone()
        };
        data.selections.push(short);
        (data.mythic_feats.iter_mut())
            .find(|feat| feat.key.as_str() == "extra-mythic-ability")
            .expect("Extra Mythic Ability")
            .selection = Some(feat_list("mythic/short-list"));
    });
    assert_eq!(
        offered(&engine, &build, &extra),
        BTreeSet::from([pick_mythic_ability("unstoppable")])
    );
    let mut refused = build.clone();
    refused
        .picks
        .insert(extra.clone(), pick_mythic_ability("mythic-charge"));
    assert!(!engine.is_possible(&refused));
    assert_eq!(
        engine.failures(&refused),
        [rules::Failure {
            choice: rules::Choice::Slot(extra),
            issue: Issue::NotAllowed,
        }]
    );
}

#[test]
fn a_mythic_ability_granting_a_selection_of_feats_gives_a_feat_slot() {
    // Second Bloodline made to grant the general feats: its slot at the rank is a Feat slot of that selection.
    let engine = engine_with(|data| {
        data.mythic_abilities
            .iter_mut()
            .find(|ability| ability.key.as_str() == "second-bloodline")
            .expect("Second Bloodline")
            .selection = Some(feat_list(rules::BASIC_FEAT_SELECTION));
        for ability in &mut data.mythic_abilities {
            ability.prerequisite = None;
        }
    });
    let mut build = build("human", &times("fighter", 6));
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("second-bloodline"),
    );
    let nested = SlotId::MythicSelection { rank: 1, index: 0 };
    assert_eq!(
        engine
            .evaluate(&build)
            .slot(&nested)
            .map(|slot| slot.kind.clone()),
        Some(SlotKind::Feat(feat_list(rules::BASIC_FEAT_SELECTION)))
    );
    build.picks.insert(nested, pick_feat("toughness"));
    assert!(engine.is_possible(&build));
}

/// Slayer Trick made to grant the teamwork feats (outflank: base attack 4 and lawful good; shield wall: base attack
/// 20), for a human fighter 3 with it as the level 3 general pick; `open` drops shield wall's prerequisite.
fn trick_granting(open: bool) -> (rules::Engine, Build) {
    let engine = engine_with(|data| {
        let trick = (data.feats.iter_mut())
            .find(|feat| feat.key.as_str() == "slayer-trick")
            .expect("Slayer Trick");
        trick.selection = Some(feat_list("teamwork-feats"));
        if open {
            (data.feats.iter_mut())
                .find(|feat| feat.key.as_str() == "shield-wall")
                .expect("Shield Wall")
                .prerequisite = None;
        }
    });
    let mut build = build("human", &times("fighter", 3));
    build
        .picks
        .insert(general_slot(3), pick_feat("slayer-trick"));
    (engine, build)
}

/// A selection none of whose options passes its prerequisite is refused, as the game's `CanSelectAny` runs each
/// option's `CanSelect` (`FeatureSelectionExtensions.CanSelectAny`, WrathDecompiled 6ce4a6b): a fighter 3 has the base
/// attack bonus of neither teamwork feat.
#[test]
fn a_selection_whose_options_all_fail_their_prerequisites_is_no_option_left() {
    let (engine, build) = trick_granting(false);
    assert!(!engine.is_possible(&build));
    let failures = engine.failures(&build);
    assert!(
        failures
            .iter()
            .any(|failure| failure.issue == Issue::NoOptionLeft(feat_list("teamwork-feats"))),
        "{failures:?}"
    );
}

#[test]
fn one_takeable_option_keeps_the_pick_open() {
    let (engine, build) = trick_granting(true);
    assert!(engine.is_possible(&build), "{:?}", engine.failures(&build));
    assert!(engine.evaluate(&build).slot(&extra_slot(3)).is_some());
}
