//! Combat style feats through a slayer talent: one per window of levels and three at most; the first also picks one
//! of the ranger's styles (a ranger/slayer shares the ranger's), and each feat comes from the style's list at its
//! window's stage, whoever levels the style.

use rules::{Build, Issue, Pick, Prerequisite, SlotId, SlotKind, Unmet};

use crate::support::*;

fn option_feat(level: u8) -> SlotId {
    SlotId::OptionFeat { level, index: 0 }
}

fn offered(build: &Build, slot: &SlotId) -> Vec<String> {
    let mut keys: Vec<String> = engine()
        .options(build, slot)
        .unwrap()
        .into_iter()
        .map(|o| o.pick.key().to_owned())
        .collect();
    keys.dedup();
    keys
}

fn options(build: &Build, slot: &SlotId) -> Vec<String> {
    let mut keys: Vec<String> = engine()
        .options(build, slot)
        .unwrap()
        .into_iter()
        .map(|o| o.pick.key().to_owned())
        .collect();
    keys.dedup();
    keys
}

// ---- the slayer's combat style feats --------------------------------------------------------------------------

fn slayer(levels: usize) -> Build {
    build("elf", &times("slayer", levels))
}

fn combat_style(stage: u8) -> Pick {
    pick_option(&format!("slayer/slayer-talents/combat-style-feat-{stage}"))
}

/// The style picked with the first combat style feat (ticket 33).
fn style_slot(level: u8) -> SlotId {
    SlotId::OptionSelection { level, index: 0 }
}

#[test]
fn a_slayer_talent_at_2_can_be_a_combat_style_feat_from_the_first_list() {
    let mut build = slayer(2);
    build.picks.insert(selection_slot(2, 0), combat_style(2));
    build
        .picks
        .insert(style_slot(2), pick_option("ranger/combat-style/archery"));
    build.picks.insert(option_feat(2), pick_feat("toughness"));
    assert!(engine().is_possible(&build));
    let state = engine().evaluate(&build).at(2).unwrap().clone();
    assert_eq!(state.feat_count(&feat("toughness")), 1);
    // Great Cleave comes with the second list.
    build
        .picks
        .insert(option_feat(2), pick_feat("great-cleave"));
    assert_eq!(why_not(&build, &option_feat(2)), None);
}

#[test]
fn each_combat_style_feat_comes_in_its_own_window_of_levels() {
    // The first at 2 or 4, not from 6.
    let mut build = slayer(6);
    build.picks.insert(selection_slot(6, 0), combat_style(2));
    assert_eq!(
        why_not(&build, &selection_slot(6, 0)),
        Some(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::Not(Box::new(Prerequisite::ClassLevel {
                class: class("slayer"),
                level: 6,
            })),
            has: None,
        }))
    );
    // The second at 6 or 8, not before.
    let mut build = slayer(4);
    build.picks.insert(selection_slot(4, 0), combat_style(6));
    assert_eq!(
        why_not(&build, &selection_slot(4, 0)),
        Some(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::ClassLevel {
                class: class("slayer"),
                level: 6,
            },
            has: Some(4),
        }))
    );
}

#[test]
fn a_slayer_takes_three_combat_style_feats_at_most() {
    let mut build = slayer(12);
    for (level, stage) in [(2, 2), (6, 6), (10, 10)] {
        build
            .picks
            .insert(selection_slot(level, 0), combat_style(stage));
    }
    // The style with the first; a feat from its list each time (each held once).
    build
        .picks
        .insert(style_slot(2), pick_option("ranger/combat-style/archery"));
    build.picks.insert(option_feat(2), pick_feat("dodge"));
    build.picks.insert(option_feat(6), pick_feat("toughness"));
    build
        .picks
        .insert(option_feat(10), pick_feat("great-cleave"));
    assert!(engine().is_possible(&build));
    build.picks.insert(selection_slot(12, 0), combat_style(10));
    assert_eq!(
        why_not(&build, &selection_slot(12, 0)),
        Some(Issue::AlreadyHeld)
    );
}

// ---- the slayer's combat style (ticket 32's decision 2) ------------------------------------------------------

fn slayer_talent(stage: u8) -> rules::Pick {
    pick_option(&format!("slayer/slayer-talents/combat-style-feat-{stage}"))
}

#[test]
fn a_slayer_picks_one_ranger_style_with_his_first_combat_style_feat() {
    let mut build = build("elf", &times("slayer", 6));
    build.picks.insert(selection_slot(2, 0), slayer_talent(2));
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet.slot(&style_slot(2)).unwrap().kind,
        SlotKind::Selection(rules::FeatureSelectionKey::new("slayer/combat-style"))
    );
    assert_eq!(
        offered(&build, &style_slot(2)),
        ["ranger/combat-style/archery", "ranger/combat-style/crane"]
    );
    build
        .picks
        .insert(style_slot(2), pick_option("ranger/combat-style/archery"));
    // The feat comes from the style's list of the stage.
    assert_eq!(offered(&build, &option_feat(2)), ["dodge", "toughness"]);
    build.picks.insert(option_feat(2), pick_feat("crane-wing"));
    assert_eq!(why_not(&build, &option_feat(2)), None);
    build.picks.insert(option_feat(2), pick_feat("toughness"));
    assert!(engine().is_possible(&build));
    // The second talent picks no style: the feat comes from the same style's next list.
    build.picks.insert(selection_slot(6, 0), slayer_talent(6));
    let sheet = engine().evaluate(&build);
    assert!(sheet.slot(&style_slot(6)).is_none());
    assert_eq!(
        offered(&build, &option_feat(6)),
        ["dodge", "great-cleave", "toughness"]
    );
}

// ---- a ranger/slayer's combat style feats ---------------------------------------------------------------------

#[test]
fn a_ranger_slayers_combat_style_feat_comes_from_the_windows_stage() {
    // Ranger 6 holds Archery at its 6th-level list; the slayer's talent at slayer 2 picks from the 2nd-level one.
    let mut classes = times("ranger", 6);
    classes.extend(times("slayer", 2));
    let mut build = build("elf", &classes);
    build.picks.insert(
        selection_slot(2, 0),
        pick_option("ranger/combat-style/archery"),
    );
    assert_eq!(
        options(&build, &class_slot(6, 0)),
        ["dodge", "great-cleave", "toughness"]
    );
    build.picks.insert(
        selection_slot(8, 0),
        pick_option("slayer/slayer-talents/combat-style-feat-2"),
    );
    let style_slot = SlotId::OptionSelection { level: 8, index: 0 };
    assert!(engine().evaluate(&build).slot(&style_slot).is_none());
    let feat_slot = SlotId::OptionFeat { level: 8, index: 0 };
    assert_eq!(options(&build, &feat_slot), ["dodge", "toughness"]);
    build
        .picks
        .insert(feat_slot.clone(), pick_feat("great-cleave"));
    assert_eq!(why_not(&build, &feat_slot), None);
}
