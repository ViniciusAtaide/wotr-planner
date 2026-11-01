//! Slots at each level: general slots, bonus slots from race, class and archetype, Feature selection slots.

use rules::{ArchetypeKey, FeatSlot, FeatureSelectionKey, LevelSheet, SlotId, SlotKind};

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
        [general_slot(1), selection_slot(1, 0)]
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
