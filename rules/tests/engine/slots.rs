//! Slots at each level: general slots, bonus slots from race, subrace, class and archetype, Feature selection
//! slots, and the order a level's slots are checked in.

use rules::{
    ArchetypeKey, Build, FeatSlot, FeatureSelectionKey, LevelSheet, SlotId, SlotKind, SubraceKey,
};

use crate::support::*;

fn slot_ids(level: &LevelSheet) -> Vec<SlotId> {
    level.slots.iter().map(|slot| slot.id.clone()).collect()
}

fn kind_of(level: &LevelSheet, id: &SlotId) -> SlotKind {
    level
        .slots
        .iter()
        .find(|slot| &slot.id == id)
        .unwrap_or_else(|| panic!("no slot {id:?} at level {}", level.character_level))
        .kind
        .clone()
}

#[test]
fn a_general_slot_comes_at_every_odd_character_level() {
    let sheet = engine().evaluate(&build("elf", &times("bard", 4)));
    assert_eq!(slot_ids(&sheet.levels[0]), [general_slot(1)]);
    assert_eq!(slot_ids(&sheet.levels[1]), []);
    assert_eq!(slot_ids(&sheet.levels[2]), [general_slot(3)]);
    assert_eq!(slot_ids(&sheet.levels[3]), []);
    assert_eq!(
        kind_of(&sheet.levels[2], &general_slot(3)),
        SlotKind::Feat(FeatSlot::General)
    );
}

#[test]
fn a_human_gets_a_bonus_general_slot_at_level_1_only() {
    let sheet = engine().evaluate(&build("human", &times("bard", 3)));
    // Race slots come first, then class slots, then the general slot.
    assert_eq!(slot_ids(&sheet.levels[0]), [race_slot(0), general_slot(1)]);
    assert_eq!(
        kind_of(&sheet.levels[0], &race_slot(0)),
        SlotKind::Feat(FeatSlot::General)
    );
    assert_eq!(slot_ids(&sheet.levels[2]), [general_slot(3)]);

    let elf = engine().evaluate(&build("elf", &["bard"]));
    assert_eq!(slot_ids(&elf.levels[0]), [general_slot(1)]);
}

#[test]
fn fighter_levels_give_combat_feat_bonus_slots_by_class_level() {
    // Wizard 1, then Fighter 1-3: fighter bonus slots at fighter levels 1 and 2, whatever the character level.
    let sheet = engine().evaluate(&build("elf", &["wizard", "fighter", "fighter", "fighter"]));
    assert_eq!(slot_ids(&sheet.levels[1]), [class_slot(2, 0)]);
    assert_eq!(
        kind_of(&sheet.levels[1], &class_slot(2, 0)),
        SlotKind::Feat(FeatSlot::CombatFeat)
    );
    assert_eq!(
        slot_ids(&sheet.levels[2]),
        [class_slot(3, 0), general_slot(3)]
    );
    assert_eq!(slot_ids(&sheet.levels[3]), []);
    assert_eq!(sheet.levels[3].class_level, 3);
}

#[test]
fn rogue_talents_are_feature_selection_slots() {
    let sheet = engine().evaluate(&build("elf", &times("rogue", 2)));
    assert_eq!(slot_ids(&sheet.levels[1]), [selection_slot(2, 0)]);
    assert_eq!(
        kind_of(&sheet.levels[1], &selection_slot(2, 0)),
        SlotKind::Selection(FeatureSelectionKey::new("rogue/rogue-talents"))
    );
}

#[test]
fn an_archetype_chosen_on_entry_replaces_the_class_rows() {
    let mut build = build("elf", &times("fighter", 2));
    build.class_choices.insert(
        class("fighter"),
        rules::ClassChoice {
            archetype: Some(ArchetypeKey::new("fighter/armiger")),
            ..rules::ClassChoice::default()
        },
    );
    let sheet = engine().evaluate(&build);

    assert_eq!(
        slot_ids(&sheet.levels[0]),
        [selection_slot(1, 0), general_slot(1)]
    );
    assert_eq!(
        kind_of(&sheet.levels[1], &class_slot(2, 0)),
        SlotKind::Feat(FeatSlot::OneOf {
            list: feat_list("fighter/armiger/bonus-feats"),
        })
    );
}

#[test]
fn ability_increases_come_at_every_4th_level() {
    let sheet = engine().evaluate(&build("elf", &times("fighter", 8)));
    let increases: Vec<_> = sheet
        .levels
        .iter()
        .filter(|level| level.ability_increase)
        .map(|level| level.character_level)
        .collect();
    assert_eq!(increases, [4, 8]);
}

// ---- pick order within a level --------------------------------------------------------------------------------

#[test]
fn a_levels_slots_are_checked_class_selections_first_then_class_slots_race_slots_and_the_general_slot()
 {
    let mut druid = build("human", &["druid"]);
    druid.picks.insert(
        selection_slot(1, 0),
        pick_option("druid/nature-bond/animal-companion"),
    );
    let ids: Vec<SlotId> = engine().evaluate(&druid).levels[0]
        .slots
        .iter()
        .map(|slot| slot.id.clone())
        .collect();
    assert_eq!(ids, [selection_slot(1, 0), race_slot(0), general_slot(1)]);

    let warpriest = build("human", &["warpriest"]);
    let ids: Vec<SlotId> = engine().evaluate(&warpriest).levels[0]
        .slots
        .iter()
        .map(|slot| slot.id.clone())
        .collect();
    assert_eq!(ids, [class_slot(1, 0), race_slot(0), general_slot(1)]);
}

#[test]
fn a_human_warpriests_race_slot_takes_dazzling_display_on_the_class_slots_weapon_focus() {
    let mut build = build("human", &["warpriest"]);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build
        .picks
        .insert(race_slot(0), pick_feat("dazzling-display"));
    assert!(engine().is_possible(&build));
}

#[test]
fn a_human_druid_with_a_companion_takes_boon_companion_at_level_1() {
    let mut build = build("human", &["druid"]);
    build.picks.insert(
        selection_slot(1, 0),
        pick_option("druid/nature-bond/animal-companion"),
    );
    build
        .picks
        .insert(race_slot(0), pick_feat("boon-companion"));
    assert!(engine().is_possible(&build));
}

// ---- a subrace's feat slots -----------------------------------------------------------------------------------

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
