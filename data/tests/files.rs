//! Reading and hashing the `game-data/` RON files.

mod common;

use std::fs;

use common::{copy_fixture, fixture_dir};
use rules::{
    Ability, ArchetypeLevelRow, BabProgression, ClassKey, FeatKey, FeatSlot, FeatureKey, GameData,
    MythicPathKey, Prerequisite, Save, SaveException, SaveProgression, SpellLevels,
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
            ClassKey::new("cleric"),
            ClassKey::new("fighter"),
            ClassKey::new("monk"),
            ClassKey::new("rogue")
        ]
    );
    assert_eq!(data.archetypes().count(), 3);
    assert_eq!(
        data.prestige_classes
            .iter()
            .map(|c| c.key.clone())
            .collect::<Vec<_>>(),
        [
            ClassKey::new("arcane-trickster"),
            ClassKey::new("mystic-theurge")
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
            MythicPathKey::new("gold-dragon"),
            MythicPathKey::new("legend"),
            MythicPathKey::new("trickster")
        ]
    );
    assert_eq!(data.progression.mythic_ascensions.len(), 10);
}

#[test]
fn reads_class_progressions_and_only_the_archetype_rows_that_differ() {
    let data = read_fixture();
    let class = |key: &str| {
        data.classes
            .iter()
            .chain(&data.prestige_classes)
            .find(|c| c.key.as_str() == key)
            .unwrap()
    };
    let fighter = class("fighter");
    assert_eq!(
        (fighter.bab, fighter.fort, fighter.reflex, fighter.will),
        (
            BabProgression::Full,
            SaveProgression::Good,
            SaveProgression::Poor,
            SaveProgression::Poor
        )
    );
    assert_eq!(fighter.save_exceptions, []);
    assert_eq!(class("mystic-theurge").bab, BabProgression::Half);

    // The herald's rows were all the cleric's, so it stores none.
    let herald = &class("cleric").archetypes[0];
    assert_eq!(herald.key.as_str(), "cleric/herald");
    assert_eq!(herald.levels, []);

    // The Eldritch Scoundrel casts arcane spells the rogue doesn't.
    let scoundrel = &class("rogue").archetypes[0];
    assert_eq!(
        scoundrel.levels[0],
        ArchetypeLevelRow {
            level: 1,
            spell_levels: Some(SpellLevels {
                arcane: 1,
                divine: 0
            }),
            features: vec![FeatureKey::new("rogue/eldritch-scoundrel/spells")],
            feat_slots: vec![],
            selections: vec![],
        }
    );
}

#[test]
fn reads_save_exceptions() {
    let dir = copy_fixture();
    let fighter = dir.path().join("classes/fighter.ron");
    let text = fs::read_to_string(&fighter).unwrap();
    fs::write(
        &fighter,
        text.replacen(
            "    will: Poor,\n",
            "    will: Poor,\n    save_exceptions: [(level: 2, save: Will, value: 1)],\n",
            1,
        ),
    )
    .unwrap();

    let data = data::read_game_data_files(dir.path()).unwrap();
    let fighter = data.classes.iter().find(|c| c.key.as_str() == "fighter");
    assert_eq!(
        fighter.unwrap().save_exceptions,
        [SaveException {
            level: 2,
            save: Save::Will,
            value: 1
        }]
    );
    assert_eq!(data.check(), []);
}

#[test]
fn reads_whether_a_fixed_list_slot_ignores_prerequisites() {
    let data = read_fixture();
    let slot_at = |class: &str, level: usize| {
        let class = data.classes.iter().find(|c| c.key.as_str() == class);
        class.unwrap().levels[level - 1].feat_slots.clone()
    };
    let monk_list = FeatSlot::OneOf {
        feats: [FeatKey::new("crane-style"), FeatKey::new("dodge")].into(),
        ignores_prerequisites: true,
    };
    assert_eq!(slot_at("monk", 1), [monk_list]);

    // Left out, it's unset: the Armiger's list checks prerequisites.
    let armiger = data
        .archetypes()
        .find(|a| a.key.as_str() == "fighter/armiger")
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
        .find(|f| f.key == FeatKey::new("dirty-trick"))
        .unwrap();

    // As written in the file; group children are a set, so compare canonical forms.
    let mut expected = Prerequisite::Any(vec![
        Prerequisite::Feat(FeatKey::new("combat-expertise")),
        Prerequisite::All(vec![
            Prerequisite::Feature {
                feature: FeatureKey::new("rogue/sneak-attack"),
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
    fs::write(&skills, text.replacen("mobility", "athletics", 1)).unwrap();

    let error = data::check_game_data_files(dir.path()).unwrap_err();

    assert!(
        matches!(&error, data::Error::Invalid(problems) if problems.len() == 1),
        "{error}"
    );
    assert!(error.to_string().contains("athletics"), "{error}");
}
