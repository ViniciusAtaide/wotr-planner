//! Named feat lists: a row names a selection of feats as it names any selection (ticket 301), and whether its picks
//! ignore prerequisites is the selection's. A race's slot can name a list of its own (the half-elf's Skill Focus).

use rules::{
    Build, Engine, FeatureSelectionKey, MythicAbilityGroup, MythicFeatGroup, Pick, SlotId,
    SlotOption,
};

use crate::support::*;

/// Whether a pick in a slot of this selection ignores the feat's prerequisites: only a bonus list's does.
fn slot_ignores_prerequisites(engine: &Engine, key: &FeatureSelectionKey) -> bool {
    engine
        .selection(key)
        .is_some_and(|selection| selection.ignores_prerequisites)
}

#[test]
fn a_bonus_list_slot_takes_the_feats_of_its_selection() {
    let engine = engine();
    let monk = engine.selection(&feat_list("monk/bonus-feats")).unwrap();
    assert_eq!(monk.name, "monk/bonus-feats");
    assert!(monk.ignores_prerequisites);
    assert!(slot_ignores_prerequisites(
        &engine,
        &feat_list("monk/bonus-feats")
    ));

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
    assert!(!slot_ignores_prerequisites(
        &engine,
        &feat_list("fighter/armiger/bonus-feats")
    ));
    // Nor is the general slot's or a category's.
    assert!(!slot_ignores_prerequisites(&engine, &feat_list("feats")));
    assert!(!slot_ignores_prerequisites(
        &engine,
        &feat_list("combat-feats")
    ));
}

/// A slot naming a selection the data lacks is a data mistake that `GameData::check` reports (ticket 98); the engine
/// offers nothing there.
#[test]
fn a_slot_naming_a_selection_the_data_lacks_accepts_no_feat() {
    assert!(!engine().slot_accepts(&feat_list("no-such-list"), &world().feats[0]));
}

#[test]
fn every_feat_list_of_the_data_is_listed_once() {
    let keys: Vec<String> = world()
        .selections
        .iter()
        .filter(|selection| !selection.feats.is_empty())
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
            "gold-dragon/dragon-feats",
            "feats",
            "combat-feats",
            "magic-feats",
            "teamwork-feats"
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
