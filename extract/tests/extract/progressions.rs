//! A class table's BAB and save columns become progressions (plus the cells they don't give), and an archetype's
//! full table becomes the rows where it differs from its base class.

use extract::progressions::{Numbers, progressions, sparse_archetype_rows};
use rules::{
    Archetype, ArchetypeKey, ArchetypeLevelRow, BabProgression, Class, ClassKey, ClassLevelRow,
    FeatureKey, GuideRef, Save, SaveException, SaveProgression, SpellLevels,
};

/// A rogue's columns as the guide prints them: ¾ BAB, poor Fortitude and Will, good Reflex.
fn rogue_numbers(levels: u8) -> Vec<(u8, Numbers)> {
    (1..=levels)
        .map(|l| {
            let numbers = Numbers {
                bab: l * 3 / 4,
                fort: l / 3,
                reflex: 2 + l / 2,
                will: l / 3,
            };
            (l, numbers)
        })
        .collect()
}

#[test]
fn a_base_class_table_gives_its_progressions() {
    let found = progressions(&rogue_numbers(20), false);
    assert_eq!(found.bab, BabProgression::ThreeQuarters);
    assert_eq!(
        [found.fort, found.reflex, found.will],
        [
            SaveProgression::Poor,
            SaveProgression::Good,
            SaveProgression::Poor
        ]
    );
    assert_eq!(found.save_exceptions, []);
    assert_eq!(found.bab_misfits, Vec::<u8>::new());
}

#[test]
fn a_save_cell_off_the_formula_is_kept_as_an_exception() {
    let mut numbers = rogue_numbers(20);
    // Like the guide's paladin Will +20 at 20, but on the rogue's poor Will.
    numbers[19].1.will = 20;
    let found = progressions(&numbers, false);
    assert_eq!(found.will, SaveProgression::Poor);
    assert_eq!(
        found.save_exceptions,
        [SaveException {
            level: 20,
            save: Save::Will,
            value: 20
        }]
    );
}

#[test]
fn a_prestige_table_uses_the_prestige_formula() {
    // ½ BAB, good Will ⌊(L+1)/2⌋, poor Fortitude and Reflex ⌊(L+1)/3⌋.
    let numbers: Vec<(u8, Numbers)> = (1..=10)
        .map(|l| {
            let numbers = Numbers {
                bab: l / 2,
                fort: (l + 1) / 3,
                reflex: (l + 1) / 3,
                will: l.div_ceil(2),
            };
            (l, numbers)
        })
        .collect();
    let found = progressions(&numbers, true);
    assert_eq!(found.bab, BabProgression::Half);
    assert_eq!(found.will, SaveProgression::Good);
    assert_eq!(found.fort, SaveProgression::Poor);
    assert_eq!(found.save_exceptions, []);
}

#[test]
fn bab_off_every_progression_is_a_misfit_the_closest_progression_replaces() {
    let mut numbers = rogue_numbers(20);
    numbers[4].1.bab = 9;
    let found = progressions(&numbers, false);
    assert_eq!(found.bab, BabProgression::ThreeQuarters);
    assert_eq!(found.bab_misfits, [5]);
}

fn row(level: u8, features: &[&str]) -> ClassLevelRow {
    ClassLevelRow {
        level,
        arcane_spell_level: 0,
        divine_spell_level: 0,
        advances_arcane: false,
        advances_divine: false,
        features: features.iter().map(|f| FeatureKey::new(*f)).collect(),
        feats: Default::default(),
        feat_slots: vec![],
        selections: vec![],
        feats_for: Default::default(),
    }
}

fn full_row(level: u8, arcane: u8, features: &[&str]) -> ArchetypeLevelRow {
    ArchetypeLevelRow {
        level,
        spell_levels: Some(SpellLevels { arcane, divine: 0 }),
        features: features.iter().map(|f| FeatureKey::new(*f)).collect(),
        feats: Default::default(),
        feat_slots: vec![],
        selections: vec![],
        feats_for: Default::default(),
    }
}

#[test]
fn an_archetype_keeps_only_the_rows_that_differ_from_its_base_class() {
    let mut class = Class {
        key: ClassKey::new("rogue"),
        name: "Rogue".into(),
        summary: String::new(),
        guide: GuideRef::new("rogue"),
        skill_points: 8,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: rules::Advancement::Each,
        counts_as: std::collections::BTreeSet::new(),
        class_skills: Default::default(),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels: vec![
            row(1, &["rogue/sneak-attack", "rogue/trapfinding"]),
            row(2, &["rogue/evasion"]),
            row(3, &["rogue/sneak-attack"]),
            row(4, &["rogue/talent"]),
            row(5, &["rogue/sneak-attack"]),
        ],
        archetypes: vec![Archetype {
            key: ArchetypeKey::new("rogue/x"),
            name: "X".into(),
            summary: String::new(),
            guide: GuideRef::new("rogue#x"),
            prerequisite: None,
            skill_points: None,
            class_skills: None,
            features: vec![],
            selections: vec![],
            feat_lists: vec![],
            levels: vec![
                // The base row, grants in another order: dropped.
                full_row(1, 0, &["rogue/trapfinding", "rogue/sneak-attack"]),
                // Other grants, same spell levels: kept without spell levels.
                full_row(2, 0, &["rogue/x/trick"]),
                // Same grants, other spell levels: kept with them.
                full_row(3, 1, &["rogue/sneak-attack"]),
                // The base row's grants taken away: kept, empty.
                full_row(4, 0, &[]),
                // The base row's features plus a granted feat: kept.
                ArchetypeLevelRow {
                    feats: [rules::FeatKey::new("endurance")].into(),
                    ..full_row(5, 0, &["rogue/sneak-attack"])
                },
            ],
            spontaneous: None,
        }],
        spontaneous: false,
    };
    sparse_archetype_rows(&mut class);

    let mut expected_2 = full_row(2, 0, &["rogue/x/trick"]);
    expected_2.spell_levels = None;
    let mut expected_4 = full_row(4, 0, &[]);
    expected_4.spell_levels = None;
    let expected_5 = ArchetypeLevelRow {
        spell_levels: None,
        feats: [rules::FeatKey::new("endurance")].into(),
        ..full_row(5, 0, &["rogue/sneak-attack"])
    };
    assert_eq!(
        class.archetypes[0].levels,
        [
            expected_2,
            full_row(3, 1, &["rogue/sneak-attack"]),
            expected_4,
            expected_5
        ]
    );
}
