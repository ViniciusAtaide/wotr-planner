//! Named feat lists: a `OneOf` slot names a list, and whether its picks ignore prerequisites is the list's. A race's
//! slot can name a list of its own (the half-elf's Skill Focus).

use rules::{
    Build, Engine, FeatSlot, MythicAbilityGroup, MythicFeatGroup, Pick, SlotId, SlotOption,
};

use crate::support::*;

/// Whether a pick in this slot ignores the feat's prerequisites: only a bonus-list slot's does, one whose list
/// ignores them.
fn slot_ignores_prerequisites(engine: &Engine, slot: &FeatSlot) -> bool {
    match slot {
        FeatSlot::OneOf { list } => engine
            .feat_list(list)
            .is_some_and(|list| list.ignores_prerequisites),
        _ => false,
    }
}

#[test]
fn a_one_of_slot_takes_the_feats_of_the_list_it_names() {
    let engine = engine();
    let monk = engine.feat_list(&feat_list("monk/bonus-feats")).unwrap();
    assert_eq!(monk.name, "monk/bonus-feats");
    assert!(monk.ignores_prerequisites);
    let slot = FeatSlot::OneOf {
        list: feat_list("monk/bonus-feats"),
    };
    assert!(slot_ignores_prerequisites(&engine, &slot));

    let offered: Vec<Pick> = engine
        .options(&build("dwarf", &["monk"]), &class_slot(1, 0))
        .unwrap()
        .into_iter()
        .map(|option| option.pick)
        .collect();
    assert_eq!(offered, [pick_feat("crane-style"), pick_feat("dodge")]);
}

#[test]
fn a_list_that_checks_prerequisites_is_not_a_bonus_list() {
    let engine = engine();
    let armiger = FeatSlot::OneOf {
        list: feat_list("fighter/armiger/bonus-feats"),
    };
    assert!(!slot_ignores_prerequisites(&engine, &armiger));
    // Nor is any other kind of slot.
    assert!(!slot_ignores_prerequisites(&engine, &FeatSlot::General));
}

/// A slot naming a list the data lacks is a data mistake that `GameData::check` reports (ticket 98), so the engine
/// takes the list to be there.
#[test]
#[should_panic(expected = "names a feat list of the data")]
fn a_slot_naming_a_list_the_data_lacks_is_a_data_mistake() {
    let slot = FeatSlot::OneOf {
        list: feat_list("no-such-list"),
    };
    engine().slot_accepts(&slot, &world().feats[0]);
}

#[test]
fn every_feat_list_of_the_data_is_listed_once() {
    let keys: Vec<String> = world()
        .feat_lists()
        .map(|list| list.key.to_string())
        .collect();
    assert_eq!(
        keys,
        [
            "half-elf/adaptability",
            "fighter/armiger/bonus-feats",
            "rogue/skill-focus",
            "monk/bonus-feats",
            "sorcerer/bloodline/abyssal-feats",
            "sorcerer/bloodline/arcane-feats",
            "sorcerer/bloodline/draconic-feats",
            "ranger/combat-style/archery-2",
            "ranger/combat-style/archery-6",
            "ranger/combat-style/crane-2",
            "warpriest/weapon-focus",
            "trickster/bonus-feats",
            "gold-dragon/dragon-feats"
        ]
    );
}

#[test]
fn mythic_groups_are_named_as_the_guide_heads_them() {
    assert_eq!(MythicAbilityGroup::Miscellaneous.name(), "Miscellaneous");
    assert_eq!(MythicFeatGroup::ArmourAndShield.name(), "Armour and Shield");
    assert_eq!(MythicFeatGroup::CombatManoeuvre.name(), "Combat Manoeuvre");
}

// ---- a race's own feat list -----------------------------------------------------------------------------------

fn options(build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("the Build has no slot {slot:?}"))
}

#[test]
fn a_half_elfs_race_slot_is_skill_focus_in_a_skill_of_choice() {
    let build = build("half-elf", &["fighter"]);
    let offered: Vec<Pick> = options(&build, &race_slot(0))
        .into_iter()
        .map(|option| option.pick)
        .collect();
    let skills = ["athletics", "knowledge-arcana", "mobility", "persuasion"];
    let expected: Vec<Pick> = skills
        .into_iter()
        .map(|key| pick_feat_for("skill-focus", skill_value(key)))
        .collect();
    assert_eq!(offered, expected);
}
