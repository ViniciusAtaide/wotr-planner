//! Reading and hashing the `game-data/` RON files.

mod common;

use std::fs;

use common::{copy_fixture, fixture_dir};
use rules::{
    Ability, ClassKey, FeatKey, FeatSlot, FeatureKey, GameData, MythicPathKey, Prerequisite,
};

fn read_fixture() -> GameData {
    data::read_game_data_files(&fixture_dir()).expect("fixture reads")
}

#[test]
fn reads_every_fixture_file() {
    let data = read_fixture();

    // Lists come back in canonical order: catalogs by key, whatever order the file has.
    let names = |items: Vec<&str>| items.join(", ");
    assert_eq!(
        names(data.races.iter().map(|r| r.name.as_str()).collect()),
        "Dwarf, Elf, Human"
    );
    assert_eq!(data.skills.len(), 4);
    assert_eq!(data.feats.len(), 15);
    assert_eq!(
        data.classes
            .iter()
            .map(|c| c.key.clone())
            .collect::<Vec<_>>(),
        [
            ClassKey::new("class:cleric"),
            ClassKey::new("class:fighter"),
            ClassKey::new("class:monk"),
            ClassKey::new("class:rogue")
        ]
    );
    assert_eq!(data.archetypes().count(), 3);
    assert_eq!(
        data.prestige_classes
            .iter()
            .map(|c| c.key.clone())
            .collect::<Vec<_>>(),
        [
            ClassKey::new("class:arcane-trickster"),
            ClassKey::new("class:mystic-theurge")
        ]
    );
    assert_eq!(data.mythic_abilities.len(), 2);
    assert_eq!(data.mythic_feats.len(), 2);
    assert_eq!(
        data.mythic_paths
            .iter()
            .map(|p| p.key.clone())
            .collect::<Vec<_>>(),
        [
            MythicPathKey::new("path:gold-dragon"),
            MythicPathKey::new("path:legend"),
            MythicPathKey::new("path:trickster")
        ]
    );
    assert_eq!(data.progression.mythic_ascensions.len(), 10);
}

#[test]
fn reads_whether_a_fixed_list_slot_ignores_prerequisites() {
    let data = read_fixture();
    let slot_at = |class: &str, level: usize| {
        let class = data.classes.iter().find(|c| c.key.as_str() == class);
        class.unwrap().levels[level - 1].feat_slots.clone()
    };
    let monk_list = FeatSlot::OneOf {
        feats: [FeatKey::new("feat:crane-style"), FeatKey::new("feat:dodge")].into(),
        ignores_prerequisites: true,
    };
    assert_eq!(slot_at("class:monk", 1), [monk_list]);

    // Left out, it's unset: the Armiger's list checks prerequisites.
    let armiger = data
        .archetypes()
        .find(|a| a.key.as_str() == "archetype:fighter/armiger")
        .unwrap();
    let armiger = &armiger.levels[1].feat_slots;
    assert!(
        matches!(
            armiger.as_slice(),
            [FeatSlot::OneOf {
                ignores_prerequisites: false,
                ..
            }]
        ),
        "{armiger:?}"
    );
}

#[test]
fn reads_nested_prerequisites_as_trees() {
    let data = read_fixture();
    let dirty_trick = data
        .feats
        .iter()
        .find(|f| f.key == FeatKey::new("feat:dirty-trick"))
        .unwrap();

    // As written in the file; group children are a set, so compare canonical forms.
    let mut expected = Prerequisite::Any(vec![
        Prerequisite::Feat(FeatKey::new("feat:combat-expertise")),
        Prerequisite::All(vec![
            Prerequisite::Feature {
                feature: FeatureKey::new("feature:rogue/sneak-attack"),
                count: 1,
            },
            Prerequisite::AbilityScore {
                ability: Ability::Intelligence,
                score: 13,
            },
        ]),
    ]);
    expected.canonicalize();
    assert_eq!(dirty_trick.prerequisite, Some(expected));
}

#[test]
fn a_misspelt_field_fails_naming_the_file() {
    let dir = copy_fixture();
    let feats = dir.path().join("feats.ron");
    let text = fs::read_to_string(&feats).unwrap();
    fs::write(&feats, text.replacen("prerequisite:", "prerequisites:", 1)).unwrap();

    let error = data::read_game_data_files(dir.path())
        .unwrap_err()
        .to_string();

    assert!(error.contains("feats.ron"), "{error}");
    assert!(error.contains("prerequisites"), "{error}");
}

#[test]
fn a_missing_required_file_fails() {
    let dir = copy_fixture();
    fs::remove_file(dir.path().join("progression.ron")).unwrap();

    let error = data::read_game_data_files(dir.path())
        .unwrap_err()
        .to_string();

    assert!(error.contains("progression.ron"), "{error}");
}

#[test]
fn a_ron_file_outside_the_layout_fails() {
    let dir = copy_fixture();
    fs::write(dir.path().join("classes").join("stray.txt"), "ignored").unwrap();
    fs::create_dir(dir.path().join("spells")).unwrap();
    fs::write(dir.path().join("spells").join("fireball.ron"), "()").unwrap();

    let error = data::read_game_data_files(dir.path())
        .unwrap_err()
        .to_string();

    assert!(error.contains("fireball.ron"), "{error}");
}

#[test]
fn the_hash_changes_only_when_a_ron_file_changes() {
    let dir = copy_fixture();
    let original = data::game_data_hash(dir.path()).unwrap();
    assert_eq!(original.len(), 64, "hex sha-256");

    fs::write(dir.path().join("README.md"), "not data").unwrap();
    assert_eq!(data::game_data_hash(dir.path()).unwrap(), original);

    let skills = dir.path().join("skills.ron");
    let text = fs::read_to_string(&skills).unwrap();
    fs::write(&skills, text.replace("Athletics", "Athletics!")).unwrap();
    assert_ne!(data::game_data_hash(dir.path()).unwrap(), original);
}

#[test]
fn the_hash_does_not_depend_on_where_the_folder_is() {
    assert_eq!(
        data::game_data_hash(copy_fixture().path()).unwrap(),
        data::game_data_hash(&fixture_dir()).unwrap()
    );
}

#[test]
fn the_fixture_passes_the_data_check() {
    assert_eq!(read_fixture().check(), []);
}

#[test]
fn checking_the_files_needs_no_database_and_passes_the_fixture() {
    let data = data::check_game_data_files(&fixture_dir()).unwrap();

    assert_eq!(data, read_fixture());
}

#[test]
fn checking_the_files_lists_the_data_problems() {
    let dir = copy_fixture();
    let skills = dir.path().join("skills.ron");
    let text = fs::read_to_string(&skills).unwrap();
    // Two skills with one key.
    fs::write(
        &skills,
        text.replacen("skill:mobility", "skill:athletics", 1),
    )
    .unwrap();

    let error = data::check_game_data_files(dir.path()).unwrap_err();

    assert!(
        matches!(&error, data::Error::Invalid(problems) if problems.len() == 1),
        "{error}"
    );
    assert!(error.to_string().contains("skill:athletics"), "{error}");
}

/// Archetypes whose BAB, saves or spell levels are allowed to differ from their base class's at the same level,
/// with the column that differs. Everything else must match: an archetype's full rows repeat those numbers, and
/// this catches a copy that drifted.
const ARCHETYPE_NUMBERS_ALLOWED_TO_DIFFER: &[(&str, &str)] = &[
    // Eldritch Scoundrel casts arcane spells; the rogue doesn't.
    ("archetype:rogue/eldritch-scoundrel", "arcane_spell_level"),
];

/// Every archetype row number that differs from the base class's at the same level and isn't allow-listed.
fn archetype_differences(data: &GameData) -> Vec<String> {
    let mut differences = Vec::new();
    for class in &data.classes {
        for archetype in &class.archetypes {
            for row in &archetype.levels {
                let Some(base) = class.levels.iter().find(|r| r.level == row.level) else {
                    continue;
                };
                let columns = [
                    ("bab", row.bab, base.bab),
                    ("fort", row.fort, base.fort),
                    ("reflex", row.reflex, base.reflex),
                    ("will", row.will, base.will),
                    (
                        "arcane_spell_level",
                        row.arcane_spell_level,
                        base.arcane_spell_level,
                    ),
                    (
                        "divine_spell_level",
                        row.divine_spell_level,
                        base.divine_spell_level,
                    ),
                ];
                for (column, archetype_value, base_value) in columns {
                    let allowed = ARCHETYPE_NUMBERS_ALLOWED_TO_DIFFER
                        .contains(&(archetype.key.as_str(), column));
                    if archetype_value != base_value && !allowed {
                        differences.push(format!(
                            "{} level {}: {column} {archetype_value}, base class {base_value}",
                            archetype.key, row.level
                        ));
                    }
                }
            }
        }
    }
    differences
}

#[test]
fn archetype_numbers_match_the_base_class_unless_allow_listed() {
    let differences = archetype_differences(&read_fixture());
    assert!(differences.is_empty(), "{differences:#?}");
}

#[test]
fn the_archetype_check_catches_a_drifted_copy() {
    let dir = copy_fixture();
    let fighter = dir.path().join("classes/fighter.ron");
    let text = fs::read_to_string(&fighter).unwrap();
    // The armiger's level 3 row is the last `(level: 3, bab: 3` in the file.
    let at = text.rfind("(level: 3, bab: 3").unwrap();
    let mut edited = text.clone();
    edited.replace_range(at..at + "(level: 3, bab: 3".len(), "(level: 3, bab: 2");
    fs::write(&fighter, edited).unwrap();

    let data = data::read_game_data_files(dir.path()).unwrap();
    assert_eq!(
        archetype_differences(&data),
        ["archetype:fighter/armiger level 3: bab 2, base class 3"]
    );
}
