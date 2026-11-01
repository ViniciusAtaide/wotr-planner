//! Second follow-ups (ticket 35): a subrace's own feat slots, a selection levelled by an option held elsewhere
//! (Impossible Domain for a Living Deity), and an option list slot fixed at its window's stage (a ranger/slayer).

use rules::{Build, ClassChoice, SlotId, SubraceKey};

use crate::support::*;

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

fn slots_at(build: &Build, level: u8) -> Vec<SlotId> {
    engine().evaluate(build).levels[usize::from(level) - 1]
        .slots
        .iter()
        .map(|slot| slot.id.clone())
        .collect()
}

// ---- a subrace's feat slots -----------------------------------------------------------------------------------

fn half_elf(subrace: Option<&str>) -> Build {
    let mut build = build("half-elf", &["fighter"]);
    build.start.subrace = subrace.map(SubraceKey::new);
    build
}

#[test]
fn a_subrace_without_the_races_slot_gets_none() {
    assert!(slots_at(&half_elf(None), 1).contains(&race_slot(0)));
    let dual_heritage = half_elf(Some("half-elf/dual-heritage"));
    assert!(!slots_at(&dual_heritage, 1).contains(&race_slot(0)));
    let mut picked = dual_heritage;
    picked.picks.insert(
        race_slot(0),
        pick_feat_for("skill-focus", skill_value("mobility")),
    );
    assert_eq!(why_not(&picked, &race_slot(0)), None);
}

#[test]
fn a_subraces_own_slot_may_name_the_races_list() {
    let mut build = half_elf(Some("half-elf/adaptable"));
    assert_eq!(options(&build, &race_slot(0)), ["skill-focus"]);
    build.picks.insert(
        race_slot(0),
        pick_feat_for("skill-focus", skill_value("mobility")),
    );
    assert!(engine().is_possible(&build));
}

// ---- Impossible Domain for a Living Deity ---------------------------------------------------------------------

const WAR_DOMAIN: &str = "cleric/domain/war";

/// A False God witch (the world's Living Deity) of `levels` levels, granted the way at 1 (the selection's one
/// option), who picked Impossible Domain's War domain at MR 1.
fn false_god(levels: usize) -> Build {
    let mut build = build("elf", &times("witch", levels));
    build.class_choices.insert(
        class("witch"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("witch/false-god")),
            ..ClassChoice::default()
        },
    );
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("impossible-domain"),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        pick_option(WAR_DOMAIN),
    );
    build
}

#[test]
fn a_living_deitys_impossible_domain_runs_on_the_ways_levels_from_the_pick() {
    let build = false_god(9);
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let at_6 = sheet.at(6).unwrap();
    assert_eq!(at_6.option_level(&option(WAR_DOMAIN)), 6);
    assert_eq!(
        at_6.feature_count(&feature("cleric/domain/war/battle-rage")),
        1
    );
    assert_eq!(sheet.at(8).unwrap().option_level(&option(WAR_DOMAIN)), 8);
    // War's bonus Combat feat (its row 8) at witch 8, long before the way's own domain at 20.
    assert!(
        sheet
            .slot(&SlotId::OptionFeat { level: 8, index: 0 })
            .is_some()
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
