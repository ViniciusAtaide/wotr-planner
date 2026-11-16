//! The order of a level's picks (tickets 126 and 248). The game applies a level-up's picks band by band (the
//! selections of the Path band, a domain or a bloodline, before the feats and talents of the Features band), and the
//! picks of one band in the order the player makes them, each checked with the earlier ones applied. So a pick may
//! lean on another pick of its band at its level, whichever slot holds it, but not on itself, on a later band's pick,
//! on a Mythic rank's pick or on a later level's. A refusal names a later band's or a Mythic rank's pick that would
//! meet it, and says to swap the two picks when each slot takes the other's.

use rules::{Band, Issue, Pick, Prerequisite, SlotId, Unmet, agreement::judge};

use crate::support::*;

/// A slot's option's status for `pick`, on `engine`.
fn status_on(
    engine: &rules::Engine,
    build: &rules::Build,
    slot: &SlotId,
    pick: &Pick,
) -> Result<(), Issue> {
    engine
        .options(build, slot)
        .expect("the slot")
        .into_iter()
        .find(|option| &option.pick == pick)
        .expect("offered")
        .status
}

fn status(build: &rules::Build, slot: &SlotId, pick: &Pick) -> Result<(), Issue> {
    status_on(&engine(), build, slot, pick)
}

fn reason(issue: &Issue) -> String {
    engine().issue_text(issue).expect("a reason")
}

fn needs_feat(key: &str) -> Unmet {
    Unmet::Condition {
        needs: Prerequisite::Feat(feat(key)),
        has: None,
    }
}

/// The pickers agree with the walk on every slot and level of the Build.
fn agrees(engine: &rules::Engine, build: &rules::Build, name: &str) {
    let agreement = judge(engine, build);
    assert!(
        agreement.disagreements.is_empty(),
        "{:?}",
        agreement.lines(name)
    );
}

#[test]
fn two_picks_of_one_level_lean_on_each_other_in_either_slot_order() {
    // The fighter's level-1 Combat feat slot comes before the general slot in the planner's slots; the game has no
    // order between them (both are the Features band: SelectFeature.CalculatePriority, LevelUpController.AddAction,
    // WrathDecompiled 6ce4a6b).
    let fighter = build("human", &["fighter"]);
    let mut later = fighter.clone();
    later
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    assert_eq!(
        status(&later, &class_slot(1, 0), &pick_feat("cleave")),
        Ok(())
    );
    later.picks.insert(class_slot(1, 0), pick_feat("cleave"));
    assert!(engine().failures(&later).is_empty());
    let held = &walked(&engine(), &later).levels[0].state;
    assert!(held.has_feat(&feat("cleave")) && held.has_feat(&feat("power-attack")));
    agrees(&engine(), &later, "fighter 1, Cleave first");

    // In the earlier slot it holds for the later one, as before.
    let mut swapped = fighter;
    swapped
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    assert_eq!(
        status(&swapped, &general_slot(1), &pick_feat("cleave")),
        Ok(())
    );
    swapped.picks.insert(general_slot(1), pick_feat("cleave"));
    assert!(engine().is_possible(&swapped));
    agrees(&engine(), &swapped, "fighter 1, Power Attack first");
}

#[test]
fn a_chain_of_three_picks_holds_whatever_slots_they_are_in() {
    // Crane Wing needs Crane Style and Dodge; Crane Style needs Dodge. At level 1 of a human fighter the class, race
    // and general slots take them last first.
    let mut build = build("human", &["fighter"]);
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("crane-wing"));
    build.picks.insert(race_slot(0), pick_feat("crane-style"));
    build.picks.insert(general_slot(1), pick_feat("dodge"));
    assert!(engine().failures(&build).is_empty());
    agrees(&engine(), &build, "fighter 1, a chain");
}

#[test]
fn a_pick_is_never_met_by_itself_nor_by_a_later_levels_pick() {
    // Cleave at 1 with Power Attack in the fighter's level-2 slot: refused at 1 with the plain reason.
    let mut build = build("human", &["fighter", "fighter"]);
    build
        .picks
        .insert(class_slot(2, 0), pick_feat("power-attack"));
    let refused = status(&build, &general_slot(1), &pick_feat("cleave"));
    assert_eq!(refused, Err(Issue::Unmet(needs_feat("power-attack"))));
    build.picks.insert(general_slot(1), pick_feat("cleave"));
    let failures = engine().failures(&build);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].issue, Issue::Unmet(needs_feat("power-attack")));

    // Two picks that each need the other hold in no order.
    let mut data = world();
    feat_mut(&mut data, "power-attack").prerequisite = Some(Prerequisite::Feat(feat("cleave")));
    let engine = engine_on(data);
    let mut circle = rules::Build {
        picks: Default::default(),
        ..build
    };
    circle.levels.truncate(1);
    circle.picks.insert(class_slot(1, 0), pick_feat("cleave"));
    circle
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    let failures = engine.failures(&circle);
    assert_eq!(failures.len(), 2, "{failures:?}");
}

#[test]
fn a_path_bands_pick_still_comes_before_its_levels_feats() {
    // The sorcerer's bloodline is a Path-band selection (the game's SorcererBloodlineSelection, group BloodLine), so
    // its pick comes before the level's feats: one needing a feat picked at its level is refused, naming that slot.
    let mut data = world();
    sorcerer_selection(&mut data, "sorcerer/bloodline").band = Band::Path;
    let arcane = option_mut(&mut data, "sorcerer/bloodline/arcane");
    arcane.prerequisite = Some(Prerequisite::Feat(feat("toughness")));
    let engine = engine_on(data);
    let mut build = build("human", &["sorcerer"]);
    build.picks.insert(general_slot(1), pick_feat("toughness"));
    let refused = status_on(
        &engine,
        &build,
        &selection_slot(1, 0),
        &pick_option("sorcerer/bloodline/arcane"),
    );
    let Err(Issue::Unmet(Unmet::MetLater { unmet, later })) = &refused else {
        panic!("{refused:?}");
    };
    assert_eq!(**unmet, needs_feat("toughness"));
    assert_eq!((&later.slot, later.level), (&general_slot(1), 1));
    build.picks.insert(
        selection_slot(1, 0),
        pick_option("sorcerer/bloodline/arcane"),
    );
    assert!(!engine.is_possible(&build));

    // The pickers agree: the bloodline's picker refuses it beside the general slot's Toughness.
    agrees(
        &engine,
        &without(&build, &selection_slot(1, 0)),
        "sorcerer 1, Toughness",
    );
}

#[test]
fn a_mythic_ranks_pick_comes_after_its_levels_slots() {
    // MR 4 comes at level 11, after the level's own slots: Dodge in the Trickster's MR 4 bonus feat slot is too late
    // for Crane Style in the level-11 general slot. The bonus slot takes only Dodge and Toughness, so no swap.
    let mut build = build("human", &times("fighter", 11));
    build.mythic_path = Some(path("trickster"));
    let path_feat = SlotId::PathFeat { rank: 4, index: 0 };
    build.picks.insert(path_feat.clone(), pick_feat("dodge"));
    let status = status(&build, &general_slot(11), &pick_feat("crane-style"));
    let Err(Issue::Unmet(Unmet::MetLater { unmet, later })) = &status else {
        panic!("{status:?}");
    };
    assert_eq!(**unmet, needs_feat("dodge"));
    assert_eq!(
        (&later.slot, later.level, later.swap),
        (&path_feat, 11, false)
    );
    assert_eq!(
        reason(&status.clone().unwrap_err()),
        "Needs dodge: dodge is in Mythic rank 4's trickster/bonus-feats slot, which comes after this slot in level \
         11's pick order"
    );
}
