//! Proficiency feats: a class's proficiency text, read from its page, becomes the real Armor, Shield and Weapon
//! Proficiency feats on the row that grants its proficiency feature; a feature those feats cover in full goes.

use std::collections::BTreeSet;

use extract::proficiencies::apply;
use rules::{
    Archetype, ArchetypeKey, ArchetypeLevelRow, BabProgression, Class, ClassKey, ClassLevelRow,
    FeatKey, Feature, FeatureKey, GameData, GuideRef, Progression, SaveProgression, SpellLevels,
};

fn feats(keys: &[&str]) -> BTreeSet<FeatKey> {
    keys.iter().map(|key| FeatKey::new(*key)).collect()
}

fn features(keys: &[&str]) -> Vec<FeatureKey> {
    keys.iter().map(|key| FeatureKey::new(*key)).collect()
}

fn feature(key: &str) -> Feature {
    Feature {
        key: FeatureKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        ability_bonuses: vec![],
        option_row: None,
        daily_choice: false,
        bab_floor: None,
        skill_ranks_at_least_mythic_rank: false,
    }
}

fn row(level: u8, granted: &[&str]) -> ClassLevelRow {
    ClassLevelRow {
        level,
        arcane_spell_level: 0,
        divine_spell_level: 0,
        advances_arcane: false,
        advances_divine: false,
        features: features(granted),
        feats: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
        feats_for: Default::default(),
    }
}

/// An archetype's full row, as the extractor has it before the rows are made sparse.
fn full_row(level: u8, granted: &[&str]) -> ArchetypeLevelRow {
    ArchetypeLevelRow {
        level,
        spell_levels: Some(SpellLevels::default()),
        features: features(granted),
        feats: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
        feats_for: Default::default(),
    }
}

fn class(
    key: &str,
    defined: &[&str],
    levels: Vec<ClassLevelRow>,
    archetypes: Vec<Archetype>,
) -> Class {
    Class {
        key: ClassKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        skill_points: 2,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: rules::Advancement::Each,
        counts_as: std::collections::BTreeSet::new(),
        class_skills: Default::default(),
        prerequisite: None,
        features: defined.iter().map(|key| feature(key)).collect(),
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes,
        spontaneous: false,
    }
}

fn archetype(key: &str, levels: Vec<ArchetypeLevelRow>) -> Archetype {
    Archetype {
        key: ArchetypeKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        prerequisite: None,
        skill_points: None,
        class_skills: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        spontaneous: None,
    }
}

fn data(classes: Vec<Class>) -> GameData {
    GameData {
        progression: Progression {
            mythic_ascensions: vec![],
        },
        races: vec![],
        skills: vec![],
        weapons: vec![],
        feats: vec![],
        classes,
        prestige_classes: vec![],
        mythic_abilities: vec![],
        mythic_feats: vec![],
        mythic_paths: vec![],
        backgrounds: vec![],
    }
}

#[test]
fn a_proficiency_feature_the_feats_cover_gives_way_to_them() {
    // "Fighters are proficient with all simple and martial weapons, all types of armour and shields (including
    // tower shields)."
    let mut data = data(vec![class(
        "fighter",
        &["fighter/fighter-proficiencies", "fighter/bravery"],
        vec![
            row(1, &["fighter/fighter-proficiencies"]),
            row(2, &["fighter/bravery"]),
        ],
        vec![],
    )]);

    apply(&mut data);

    let fighter = &data.classes[0];
    assert_eq!(
        fighter.levels[0].feats,
        feats(&[
            "armor-proficiency-heavy-armor",
            "armor-proficiency-light-armor",
            "armor-proficiency-medium-armor",
            "martial-weapons-proficiency",
            "shield-proficiency",
            "simple-weapon-proficiency",
            "tower-shield-proficiency",
        ])
    );
    assert_eq!(fighter.levels[0].features, []);
    assert_eq!(fighter.levels[1].features, features(&["fighter/bravery"]));
    let defined: Vec<&str> = fighter.features.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(defined, ["fighter/bravery"]);
}

#[test]
fn a_feature_that_names_more_than_the_feats_stays() {
    // "Rogues are proficient with all simple weapons, rapier, short sword and short bow. They are proficient with
    // light armour but not with shields." The named weapons have no feat.
    let mut data = data(vec![class(
        "rogue",
        &["rogue/rogue-proficiencies"],
        vec![row(1, &["rogue/rogue-proficiencies"])],
        // "The Eldritch Scoundrel is not proficient with light armour."
        vec![archetype(
            "rogue/eldritch-scoundrel",
            vec![full_row(1, &["rogue/rogue-proficiencies"])],
        )],
    )]);

    apply(&mut data);

    let rogue = &data.classes[0];
    assert_eq!(
        rogue.levels[0].feats,
        feats(&["armor-proficiency-light-armor", "simple-weapon-proficiency"])
    );
    assert_eq!(
        rogue.levels[0].features,
        features(&["rogue/rogue-proficiencies"])
    );
    assert_eq!(
        rogue.archetypes[0].levels[0].feats,
        feats(&["simple-weapon-proficiency"])
    );
}

#[test]
fn a_class_whose_table_names_no_proficiency_feature_gets_its_feats_at_level_1() {
    // "Kineticists are proficient with all simple weapons and light armour", and so are its archetypes; the
    // Kinetic Knight's Elemental Bastion adds medium and heavy armour and shields.
    let mut data = data(vec![class(
        "kineticist",
        &[],
        vec![row(1, &[]), row(2, &[])],
        vec![archetype(
            "kineticist/kinetic-knight",
            vec![
                full_row(1, &["kineticist/kinetic-knight/elemental-bastion"]),
                full_row(2, &[]),
            ],
        )],
    )]);

    apply(&mut data);

    let kineticist = &data.classes[0];
    let base = feats(&["armor-proficiency-light-armor", "simple-weapon-proficiency"]);
    assert_eq!(kineticist.levels[0].feats, base);
    assert_eq!(kineticist.levels[1].feats, BTreeSet::new());
    assert_eq!(
        kineticist.archetypes[0].levels[0].feats,
        feats(&[
            "armor-proficiency-heavy-armor",
            "armor-proficiency-light-armor",
            "armor-proficiency-medium-armor",
            "shield-proficiency",
            "simple-weapon-proficiency",
        ])
    );
}
