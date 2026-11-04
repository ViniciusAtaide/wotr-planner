//! `GameData`'s own lookups: every class, every selection and feature, and a class or option by key.

use rules::{ClassKey, GameData, Prerequisite, SelectionOptionKey};

use crate::support::world;

fn class_name<'a>(data: &'a GameData, key: &str) -> Option<&'a str> {
    data.class(&ClassKey::new(key))
        .map(|class| class.name.as_str())
}

fn option_name<'a>(data: &'a GameData, key: &str) -> Option<&'a str> {
    data.selection_option(&SelectionOptionKey::new(key))
        .map(|option| option.name.as_str())
}

/// The class selections of the test world: each class's own, then its archetypes', base classes then prestige.
const CLASS_SELECTIONS: [&str; 30] = [
    "fighter/armiger/order",
    "rogue/rogue-talents",
    "slayer/slayer-talents",
    "slayer/combat-style",
    "wizard/school",
    "cleric/domain",
    "cleric/revelation",
    "cleric/fire-revelations",
    "cleric/water-revelations",
    "cleric/maneuver-mastery",
    "cleric/impossible-domain",
    "cleric/accursed/revelation",
    "cleric/separatist/domain",
    "monk/zen-archer/way-of-the-bow",
    "witch/hex",
    "witch/false-god/way",
    "sorcerer/bloodline",
    "sorcerer/school-power",
    "sorcerer/crossblooded/second-bloodline",
    "sorcerer/primalist/primal-choice",
    "sorcerer/sage/bloodline",
    "sorcerer/seeker/bloodline",
    "ranger/combat-style",
    "warpriest/blessings",
    "warpriest/champion/chosen-alignment",
    "druid/nature-bond",
    "kineticist/element",
    "kineticist/expanded-element",
    "kineticist/wild-talents",
    "dragon-disciple/bloodline",
];

#[test]
fn every_class_is_the_base_classes_then_the_prestige_classes() {
    let data = world();
    let keys: Vec<&str> = data.all_classes().map(|class| class.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "fighter",
            "rogue",
            "slayer",
            "wizard",
            "cleric",
            "monk",
            "witch",
            "sorcerer",
            "bard",
            "ranger",
            "warpriest",
            "druid",
            "kineticist",
            "arcane-trickster",
            "mystic-theurge",
            "eldritch-knight",
            "winter-witch",
            "dragon-disciple",
        ]
    );
}

#[test]
fn every_class_can_be_edited_in_place_base_and_prestige_alike() {
    let mut data = world();
    for class in data.all_classes_mut() {
        class.summary = "edited".into();
    }
    assert!(data.classes.iter().all(|class| class.summary == "edited"));
    assert!(
        data.prestige_classes
            .iter()
            .all(|class| class.summary == "edited")
    );
}

#[test]
fn a_class_is_found_by_key_whether_base_or_prestige() {
    let data = world();
    assert_eq!(class_name(&data, "rogue"), Some("Rogue"));
    assert_eq!(
        class_name(&data, "arcane-trickster"),
        Some("Arcane Trickster")
    );
    assert_eq!(class_name(&data, "rogue/eldritch-scoundrel"), None);
}

#[test]
fn class_selections_are_each_classs_own_then_its_archetypes_with_no_paths() {
    let data = world();
    let keys: Vec<&str> = data
        .class_selections()
        .map(|selection| selection.key.as_str())
        .collect();
    assert_eq!(keys, CLASS_SELECTIONS);
}

#[test]
fn every_selection_is_the_class_selections_then_each_paths() {
    let data = world();
    let keys: Vec<&str> = data
        .selections()
        .map(|selection| selection.key.as_str())
        .collect();
    let paths = [
        "trickster/tricks",
        "aeon/enforcing-gaze",
        "gold-dragon/breaths",
    ];
    assert_eq!(keys, [&CLASS_SELECTIONS[..], &paths].concat());
}

#[test]
fn class_features_are_each_classs_own_then_its_archetypes_with_no_paths() {
    let data = world();
    let keys: Vec<&str> = data
        .class_features()
        .map(|feature| feature.key.as_str())
        .collect();
    assert_eq!(
        keys,
        [
            "fighter/bravery",
            "rogue/sneak-attack",
            "wizard/hand-of-the-apprentice",
            "wizard/metamagic-mastery",
            "wizard/resistance",
            "wizard/protective-ward",
            "wizard/energy-absorption",
            "cleric/domain/war/battle-rage",
            "cleric/domain/fire/fire-bolt",
            "witch/major-hexes",
            "witch/grand-hexes",
            "sorcerer/bloodline/abyssal/arcana",
            "sorcerer/bloodline/abyssal/claws",
            "sorcerer/bloodline/abyssal/resistances",
            "sorcerer/bloodline/abyssal/added-summonings",
            "sorcerer/bloodline/abyssal/demonic-might",
            "sorcerer/bloodline/arcane/arcane-bond",
            "sorcerer/bloodline/arcane/combat-casting-adept",
            "sorcerer/bloodline/arcane/apotheosis",
            "sorcerer/bloodline/draconic/arcana",
            "sorcerer/bloodline/draconic/claws",
            "sorcerer/bloodline/draconic/resistances",
            "sorcerer/bloodline/draconic/power-of-wyrms",
            "sorcerer/bloodline/abyssal/strength-of-the-abyss",
            "sorcerer/bloodline-ascendance",
            "sorcerer/sage/arcane-bolt",
            "kineticist/element/air/defense",
            "kineticist/element/fire/defense",
            "kineticist/element/water/defense",
            "kineticist/wild-talents/expanded-defense/defensive-talents",
            "dragon-disciple/strength",
        ]
    );
}

#[test]
fn a_selection_option_is_found_by_key_in_any_selection() {
    let data = world();
    assert_eq!(
        option_name(&data, "fighter/armiger/order/dragon"),
        Some("dragon"),
        "an archetype's"
    );
    assert_eq!(
        data.selection_option(&SelectionOptionKey::new("gold-dragon/breaths/annihilating"))
            .and_then(|option| option.prerequisite.as_ref()),
        Some(&Prerequisite::All(vec![
            Prerequisite::MythicRank(9),
            Prerequisite::Story(rules::Story::VowOfDahak),
        ])),
        "a path's"
    );
    assert_eq!(option_name(&data, "fighter/armiger/order"), None);
}
