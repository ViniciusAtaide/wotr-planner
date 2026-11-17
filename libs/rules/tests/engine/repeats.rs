//! Repeats from the header (ticket 303): how often a feat, an option, a mythic ability or a mythic feat is taken is
//! derived from its header, as the game's `BlueprintFeatureSelection.CanSelect` derives it (WrathDecompiled 6ce4a6b):
//! a feature that isn't a selection stops at its `ranks`; a parametrised one repeats once per value; a selection
//! repeats while it has an option left, once with a `Not` of itself, up to `n` times with a `Not` of its own rank `n`.
//! Each Build the walk holds is judged by the pickers too ([`rules::agreement::judge`]).

use rules::{
    Build, Engine, Fact, FeatKey, FeatureSelectionKey, Issue, MythicAbilityKey, MythicFeatKey,
    Prerequisite, Repeat, SelectionOptionKey, SlotId,
};

use crate::support::*;

/// A fighter 20 on the Trickster path: Mythic ranks 1 to 10.
fn mythic_fighter() -> Build {
    let mut build = build("human", &times("fighter", 20));
    build.mythic_path = Some(path("trickster"));
    build
}

fn general(rank: u8) -> SlotId {
    SlotId::Mythic { rank }
}

fn extra(rank: u8) -> SlotId {
    SlotId::MythicExtra { rank }
}

/// The picker's verdict on `slot`'s pick in `build` without it, where the walk refuses it for `issue`.
fn refused_for(engine: &Engine, build: &Build, slot: &SlotId, issue: &Issue) {
    let failures = engine.failures(build);
    assert!(
        failures.iter().any(|failure| failure.issue == *issue),
        "{failures:?}"
    );
    let rest = without(build, slot);
    assert!(engine.is_possible(&rest), "impossible without {slot:?} too");
    let pick = &build.picks[slot];
    let status = (engine.options(&rest, slot).expect("a slot").into_iter())
        .find(|option| &option.pick == pick)
        .expect("offered")
        .status;
    assert_eq!(status.as_ref().err(), Some(issue));
}

/// Asserts the pickers agree with the walk over a possible Build.
fn agrees(engine: &Engine, build: &Build) {
    assert!(engine.is_possible(build), "{:?}", engine.failures(build));
    let agreement = rules::agreement::judge(engine, build);
    assert!(
        agreement.disagreements.is_empty(),
        "{:?}",
        agreement.lines("build")
    );
}

#[test]
fn the_repeat_is_derived_from_the_header() {
    let data = world();
    let feat = |key: &str| {
        (data.feats.iter())
            .find(|feat| feat.key == FeatKey::new(key))
            .unwrap()
            .repeat()
    };
    let mythic_feat = |key: &str| {
        (data.mythic_feats.iter())
            .find(|feat| feat.key == MythicFeatKey::new(key))
            .unwrap()
            .repeat()
    };
    let option = |key: &str| {
        data.selection_option(&SelectionOptionKey::new(key))
            .unwrap()
            .repeat()
    };
    // One rank, and its ranks.
    assert_eq!(feat("toughness"), Repeat::Once);
    assert_eq!(feat("extra-ki"), Repeat::Times(20));
    assert_eq!(option("rogue/rogue-talents/resiliency"), Repeat::Times(2));
    // A parametrised feat.
    assert_eq!(feat("weapon-focus"), Repeat::PerValue);
    assert_eq!(mythic_feat("school-mastery"), Repeat::PerValue);
    // A selection: capped by its own rank, or while an option is left.
    assert_eq!(feat("extra-hex"), Repeat::Times(3));
    assert_eq!(feat("slayer-trick"), Repeat::WhileOptionLeft);
    assert_eq!(mythic_feat("extra-feat"), Repeat::WhileOptionLeft);
    // An option that takes a value but isn't a selection: once (Battle Master's weapon).
    assert_eq!(option("rogue/rogue-talents/weapon-focus"), Repeat::Once);
}

/// Extra Feat (mythic) is a selection: taken at two ranks, each a feat slot.
#[test]
fn a_selection_is_taken_again_while_it_has_an_option_left() {
    let engine = engine();
    let mut build = mythic_fighter();
    build.picks.extend([
        (general(2), pick_mythic_feat("extra-feat")),
        (extra(2), pick_feat("toughness")),
        (general(4), pick_mythic_feat("extra-feat")),
        (extra(4), pick_feat("dodge")),
    ]);
    agrees(&engine, &build);
    let state = engine.evaluate(&build).at(20).unwrap().clone();
    assert_eq!(
        state.mythic_feat_count(&MythicFeatKey::new("extra-feat")),
        2
    );
    assert_eq!(state.feat_count(&FeatKey::new("dodge")), 1);
}

/// A selection with nothing left to pick is refused: Extra Mythic Ability granting a selection of one ability, held.
#[test]
fn a_selection_with_no_option_left_is_refused() {
    let one = FeatureSelectionKey::new("mythic/one-ability");
    let engine = engine_with(|data| {
        let mut selection = selection_mut(data, "mythic/extra-mythic-ability").clone();
        selection.key = one.clone();
        selection.mythic_abilities = [MythicAbilityKey::new("mythic-charge")].into();
        data.selections.push(selection);
        mythic_feat_mut(data, "extra-mythic-ability").selection = Some(one.clone());
    });
    let mut build = mythic_fighter();
    build.picks.extend([
        (general(2), pick_mythic_feat("extra-mythic-ability")),
        (extra(2), pick_mythic_ability("mythic-charge")),
    ]);
    agrees(&engine, &build);
    build
        .picks
        .insert(general(4), pick_mythic_feat("extra-mythic-ability"));
    refused_for(&engine, &build, &general(4), &Issue::NoOptionLeft(one));
}

/// A selection whose prerequisite is that it isn't held is taken once (the game's own `PrerequisiteNoFeature`:
/// Second Bloodline).
#[test]
fn a_selection_not_of_itself_is_taken_once() {
    let engine = engine_with(|data| {
        let feat = mythic_feat_mut(data, "extra-feat");
        feat.prerequisite = Some(Prerequisite::Not(Box::new(Prerequisite::MythicFeat(
            feat.key.clone(),
        ))));
        assert_eq!(feat.repeat(), Repeat::Once);
    });
    let mut build = mythic_fighter();
    build.picks.extend([
        (general(2), pick_mythic_feat("extra-feat")),
        (extra(2), pick_feat("toughness")),
    ]);
    agrees(&engine, &build);
    build
        .picks
        .insert(general(4), pick_mythic_feat("extra-feat"));
    refused_for(&engine, &build, &general(4), &Issue::AlreadyHeld);
}

/// A selection whose own `PrerequisiteNoFeature` names a blueprint it shares with another entry holds a `Not` of an
/// `Any` naming both, and is taken once all the same (ticket 308d: Second Mystery and the oracle's second mystery
/// selection share SecondMystery 277b0164740b97945a3f8022bd572f48).
#[test]
fn a_selection_not_of_an_any_naming_itself_is_taken_once() {
    let engine = engine_with(|data| {
        let feat = mythic_feat_mut(data, "extra-feat");
        feat.prerequisite = Some(Prerequisite::Not(Box::new(Prerequisite::Any(vec![
            Prerequisite::MythicFeat(feat.key.clone()),
            Prerequisite::Feat(FeatKey::new("dodge")),
        ]))));
        assert_eq!(feat.repeat(), Repeat::Once);
    });
    let mut build = mythic_fighter();
    build.picks.extend([
        (general(2), pick_mythic_feat("extra-feat")),
        (extra(2), pick_feat("toughness")),
    ]);
    agrees(&engine, &build);
    build
        .picks
        .insert(general(4), pick_mythic_feat("extra-feat"));
    refused_for(&engine, &build, &general(4), &Issue::AlreadyHeld);
}

/// A selection that caps its own rank stops there (the game's `Not` of `ContextConditionBuffRank` on itself: Extra
/// Hex at 3).
#[test]
fn a_selection_capped_at_its_rank_stops_there() {
    let engine = engine_with(|data| {
        let feat = mythic_feat_mut(data, "extra-feat");
        feat.prerequisite = Some(Prerequisite::Not(Box::new(Prerequisite::Rank {
            fact: Fact::MythicFeat(feat.key.clone()),
            rank: 2,
        })));
        assert_eq!(feat.repeat(), Repeat::Times(2));
    });
    let mut build = mythic_fighter();
    build.picks.extend([
        (general(2), pick_mythic_feat("extra-feat")),
        (extra(2), pick_feat("toughness")),
        (general(4), pick_mythic_feat("extra-feat")),
        (extra(4), pick_feat("dodge")),
    ]);
    agrees(&engine, &build);
    build
        .picks
        .insert(general(6), pick_mythic_feat("extra-feat"));
    refused_for(&engine, &build, &general(6), &Issue::MaxTimes { times: 2 });
}

/// A feature that isn't a selection stops at its ranks: the invented rogue talent Resiliency at 2.
#[test]
fn a_feature_stops_at_its_ranks() {
    let engine = engine();
    let talent = "rogue/rogue-talents/resiliency";
    let mut build = build("human", &times("rogue", 6));
    build.picks.extend([
        (selection_slot(2, 0), pick_option(talent)),
        (selection_slot(4, 0), pick_option(talent)),
    ]);
    agrees(&engine, &build);
    build
        .picks
        .insert(selection_slot(6, 0), pick_option(talent));
    refused_for(
        &engine,
        &build,
        &selection_slot(6, 0),
        &Issue::MaxTimes { times: 2 },
    );
}
