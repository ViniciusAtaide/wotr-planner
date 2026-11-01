//! Sparse archetype rows: an archetype stores only the levels where it differs from its base class, and
//! `effective_row` is the one place they meet the base class's rows.

use rules::{
    ArchetypeKey, ArchetypeLevelRow, Build, ClassChoice, Engine, FeatSlot, GameData, SpellLevels,
    effective_row,
};

use crate::support::*;

fn as_archetype(mut build: Build, class_key: &str, archetype: &str) -> Build {
    build.class_choices.insert(
        class(class_key),
        ClassChoice {
            archetype: Some(ArchetypeKey::new(archetype)),
            advances: Default::default(),
        },
    );
    build
}

fn class_in<'a>(data: &'a GameData, key: &str) -> &'a rules::Class {
    data.classes.iter().find(|c| c.key.as_str() == key).unwrap()
}

fn class_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Class {
    data.classes
        .iter_mut()
        .find(|c| c.key.as_str() == key)
        .unwrap()
}

#[test]
fn a_level_the_archetype_leaves_out_is_the_base_class_row() {
    let mut data = world();
    // The armiger has no row at 3, so a Combat feat slot the fighter gains at 3 is the armiger's too.
    class_mut(&mut data, "fighter").levels[2]
        .feat_slots
        .push(FeatSlot::CombatFeat);
    let build = as_archetype(
        build("elf", &times("fighter", 3)),
        "fighter",
        "fighter/armiger",
    );
    let sheet = Engine::new(data).evaluate(&build);

    let kinds_at_3: Vec<_> = sheet.levels[2].slots.iter().map(|s| s.id.clone()).collect();
    assert_eq!(kinds_at_3, [class_slot(3, 0), general_slot(3)]);
}

#[test]
fn an_archetype_row_replaces_the_base_rows_grants_in_full() {
    let build = as_archetype(
        build("elf", &times("fighter", 4)),
        "fighter",
        "fighter/armiger",
    );
    let sheet = engine().evaluate(&build);

    // The fighter gets a Combat feat slot at 4; the armiger's empty row at 4 takes it away.
    assert_eq!(sheet.levels[3].slots, []);
    // And Bravery (fighter 2) with it, since the armiger's row at 2 doesn't list it.
    assert_eq!(
        sheet
            .at(4)
            .unwrap()
            .feature_count(&feature("fighter/bravery")),
        0
    );
}

#[test]
fn an_archetype_row_keeps_the_base_spell_levels_unless_it_sets_its_own() {
    let mut data = world();
    let cleric = class_mut(&mut data, "cleric");
    let mut archetype = class_in(&world(), "fighter").archetypes[0].clone();
    archetype.key = ArchetypeKey::new("cleric/herald");
    archetype.selections.clear();
    archetype.feat_lists.clear();
    archetype.levels = vec![
        // Grants something but leaves the spell levels alone.
        ArchetypeLevelRow {
            feats_for: Default::default(),
            level: 3,
            spell_levels: None,
            features: vec![feature("fighter/bravery")],
            feats: Default::default(),
            feat_slots: vec![],
            selections: vec![],
        },
        // Loses spellcasting from 5.
        ArchetypeLevelRow {
            feats_for: Default::default(),
            level: 5,
            spell_levels: Some(SpellLevels::default()),
            features: vec![],
            feats: Default::default(),
            feat_slots: vec![],
            selections: vec![],
        },
    ];
    cleric.archetypes.push(archetype);
    let build = as_archetype(build("elf", &times("cleric", 5)), "cleric", "cleric/herald");
    let sheet = Engine::new(data).evaluate(&build);

    let divine: Vec<_> = (1..=5)
        .map(|level| sheet.at(level).unwrap().divine_spell_level)
        .collect();
    assert_eq!(divine, [1, 1, 2, 2, 0]);
}

#[test]
fn effective_row_is_the_archetypes_row_where_it_has_one_and_the_base_row_elsewhere() {
    let data = world();
    let fighter = class_in(&data, "fighter");
    let armiger = &fighter.archetypes[0];

    let at_1 = effective_row(fighter, Some(armiger), 1).unwrap();
    assert_eq!(at_1.feat_slots, []);
    assert_eq!(
        at_1.selections,
        [rules::FeatureSelectionKey::new("fighter/armiger/order")]
    );

    let at_3 = effective_row(fighter, Some(armiger), 3).unwrap();
    assert_eq!(at_3, effective_row(fighter, None, 3).unwrap());

    let base_1 = effective_row(fighter, None, 1).unwrap();
    assert_eq!(base_1.feat_slots, [FeatSlot::CombatFeat]);

    assert_eq!(effective_row(fighter, Some(armiger), 21), None);
}

#[test]
fn an_archetype_has_its_base_class_level_cap_however_few_rows_it_stores() {
    let build = as_archetype(
        build("elf", &times("fighter", 20)),
        "fighter",
        "fighter/armiger",
    );
    assert!(engine().is_possible(&build));
}
