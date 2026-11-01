//! Reading and hashing the `game-data/` RON files.

mod common;

use std::collections::BTreeSet;
use std::fs;

use common::{copy_fixture, fixture_dir};
use rules::{
    Ability, AbilityBonus, ArchetypeLevelRow, BabProgression, ClassKey, FeatKey, FeatListKey,
    FeatSlot, FeatureKey, FeatureSelectionKey, GameData, MythicAbilityGroup, MythicFeatGroup,
    MythicPathKey, Prerequisite, Save, SaveException, SaveProgression, SelectionOptionKey,
    SpellLevels,
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
    assert_eq!(data.feats.len(), 16);
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
            feats: BTreeSet::new(),
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
fn reads_feat_lists_and_the_slots_that_name_them() {
    let data = read_fixture();
    let lists: Vec<(&str, &str, bool, Vec<&str>)> = data
        .feat_lists()
        .map(|list| {
            (
                list.key.as_str(),
                list.name.as_str(),
                list.ignores_prerequisites,
                list.feats.iter().map(FeatKey::as_str).collect(),
            )
        })
        .collect();
    // Each in its owner's file: the monk's, the Armiger's (inside the fighter's file), the Gold Dragon's and the
    // trickster's. Left out, `ignores_prerequisites` and `every_feat` are unset, and `feats` is empty.
    assert_eq!(
        lists,
        [
            (
                "fighter/armiger/bonus-feats",
                "Armiger Bonus Feats",
                false,
                vec!["combat-expertise", "power-attack"]
            ),
            (
                "monk/bonus-feats",
                "Monk Bonus Feats",
                true,
                vec!["crane-style", "dodge"]
            ),
            ("gold-dragon/dragon-feats", "Dragon Feats", true, vec![]),
            (
                "trickster/bonus-feats",
                "Trickster Bonus Feats",
                false,
                vec!["dirty-trick", "toughness"]
            ),
        ]
    );
    let monk = data.classes.iter().find(|c| c.key.as_str() == "monk");
    assert_eq!(
        monk.unwrap().levels[0].feat_slots,
        [FeatSlot::OneOf {
            list: FeatListKey::new("monk/bonus-feats")
        }]
    );
}

#[test]
fn reads_a_selections_shared_options_and_defaults_them_to_none() {
    let data = read_fixture();
    let rogue = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "rogue")
        .unwrap();
    // The rogue's talents share nothing: left out, `shared_options` is empty.
    assert_eq!(rogue.selections[0].shared_options, BTreeSet::new());
    let arcane_talents = &rogue.archetypes[0].selections[0];
    assert_eq!(
        arcane_talents.key,
        FeatureSelectionKey::new("rogue/eldritch-scoundrel/arcane-talents")
    );
    assert_eq!(
        arcane_talents.shared_options,
        BTreeSet::from([
            SelectionOptionKey::new("rogue/rogue-talents/combat-trick"),
            SelectionOptionKey::new("rogue/rogue-talents/crippling-strike"),
        ])
    );
    assert!(arcane_talents.offers(&SelectionOptionKey::new(
        "rogue/eldritch-scoundrel/arcane-talents/minor-magic"
    )));
    assert!(!arcane_talents.offers(&SelectionOptionKey::new("rogue/rogue-talents/fast-stealth")));
}

#[test]
fn reads_granted_feats_on_class_and_archetype_rows() {
    let data = read_fixture();
    let monk = data.classes.iter().find(|c| c.key.as_str() == "monk");
    assert_eq!(
        monk.unwrap().levels[0].feats,
        BTreeSet::from([FeatKey::new("improved-unarmed-strike")])
    );
    let armiger = data
        .archetypes()
        .find(|a| a.key.as_str() == "fighter/armiger")
        .unwrap();
    assert_eq!(
        armiger.levels[0].feats,
        BTreeSet::from([FeatKey::new("armiger-drill")])
    );
}

#[test]
fn reads_mythic_groups() {
    let data = read_fixture();
    let groups: Vec<MythicAbilityGroup> = data.mythic_abilities.iter().map(|a| a.group).collect();
    assert_eq!(
        groups,
        [MythicAbilityGroup::Class, MythicAbilityGroup::Melee]
    );
    assert!(
        data.mythic_feats
            .iter()
            .all(|f| f.group == MythicFeatGroup::Melee)
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

#[test]
fn reads_the_mythic_path_rules() {
    let data = read_fixture();
    let path = |key: &str| {
        data.mythic_paths
            .iter()
            .find(|p| p.key.as_str() == key)
            .unwrap()
    };
    let legend = path("legend");
    let switch = legend.switch.as_ref().unwrap();
    // Left out: no previous-path condition and nothing dropped by name.
    assert!(switch.from.is_empty());
    assert!(switch.dropped_features.is_empty() && switch.dropped_selections.is_empty());
    assert_eq!(
        (switch.kept_through_rank, switch.path_kept_through_rank),
        (2, 2)
    );
    assert_eq!(switch.rank_after_switch(), 3);
    assert_eq!(switch.features, [FeatureKey::new("legend/legend")].into());
    assert_eq!(
        legend.features[0].ability_bonuses,
        [AbilityBonus::Every { amount: 4 }]
    );

    let dragon = path("gold-dragon");
    let switch = dragon.switch.as_ref().unwrap();
    assert_eq!(switch.from, [MythicPathKey::new("trickster")].into());
    assert_eq!(
        (switch.kept_through_rank, switch.path_kept_through_rank),
        (7, 2)
    );
    assert_eq!(
        switch.dropped_selections,
        [FeatureSelectionKey::new("trickster/tricks")].into()
    );
    assert!(dragon.feat_lists[0].every_feat);
    let body = &dragon.features[2];
    assert_eq!(body.key.as_str(), "gold-dragon/perfect-body");
    assert_eq!(
        body.ability_bonuses,
        [
            AbilityBonus::Highest { amount: 4 },
            AbilityBonus::AtLeast { score: 14 }
        ]
    );
    assert_eq!(
        dragon.selections[0].options[0].prerequisite,
        Some(Prerequisite::MythicRank(9))
    );

    // A feature without bonuses reads as none; an option may grant several feats.
    assert!(path("trickster").features[0].ability_bonuses.is_empty());
    let rogue = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "rogue")
        .unwrap();
    let combat_trick = &rogue.selections[0].options[0];
    assert_eq!(
        combat_trick.feats,
        [
            FeatKey::new("combat-expertise"),
            FeatKey::new("dirty-trick")
        ]
        .into()
    );
}
