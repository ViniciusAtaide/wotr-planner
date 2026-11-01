//! Named feat lists: a `OneOf` slot names a list, and whether its picks ignore prerequisites is the list's.

use rules::{FeatSlot, MythicAbilityGroup, MythicFeatGroup, Pick};

use crate::support::*;

#[test]
fn a_one_of_slot_takes_the_feats_of_the_list_it_names() {
    let engine = engine();
    let monk = engine.feat_list(&feat_list("monk/bonus-feats")).unwrap();
    assert_eq!(monk.name, "monk/bonus-feats");
    assert!(monk.ignores_prerequisites);
    let slot = FeatSlot::OneOf {
        list: feat_list("monk/bonus-feats"),
    };
    assert!(engine.slot_ignores_prerequisites(&slot));

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
    assert!(!engine.slot_ignores_prerequisites(&armiger));
    // Nor is any other kind of slot.
    assert!(!engine.slot_ignores_prerequisites(&FeatSlot::General));
}

#[test]
fn a_slot_naming_a_list_the_data_lacks_takes_nothing() {
    let slot = FeatSlot::OneOf {
        list: feat_list("no-such-list"),
    };
    let engine = engine();
    assert!(engine.feat_list(&feat_list("no-such-list")).is_none());
    assert!(
        world()
            .feats
            .iter()
            .all(|feat| !engine.slot_accepts(&slot, feat))
    );
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
