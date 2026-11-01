//! Reading and hashing the `game-data/` RON files.

mod common;

use std::collections::BTreeSet;
use std::fs;

use common::{copy_fixture, fixture_dir};
use rules::{
    Ability, AbilityBonus, Advancement, ArchetypeLevelRow, BabProgression, ClassKey, CountsAs,
    CountsToward, FeatKey, FeatListKey, FeatSlot, FeatureKey, FeatureSelectionKey, GameData,
    MythicAbilityGroup, MythicFeatGroup, MythicPathKey, OptionLevelRow, OptionRow, Parameter,
    ParameterKind, Prerequisite, Repeat, Save, SaveException, SaveProgression, SelectionOptionKey,
    SpellLevels, Story, WeaponKey, WeaponProficiency,
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
    assert_eq!(data.weapons.len(), 4);
    assert_eq!(data.feats.len(), 29);
    assert_eq!(
        data.classes
            .iter()
            .map(|c| c.key.clone())
            .collect::<Vec<_>>(),
        [
            ClassKey::new("cleric"),
            ClassKey::new("fighter"),
            ClassKey::new("monk"),
            ClassKey::new("rogue"),
            ClassKey::new("sorcerer")
        ]
    );
    assert_eq!(data.archetypes().count(), 4);
    assert_eq!(
        data.prestige_classes
            .iter()
            .map(|c| c.key.clone())
            .collect::<Vec<_>>(),
        [
            ClassKey::new("arcane-trickster"),
            ClassKey::new("eldritch-knight"),
            ClassKey::new("loremaster"),
            ClassKey::new("mystic-theurge")
        ]
    );
    assert_eq!(data.mythic_abilities.len(), 4);
    assert_eq!(data.mythic_feats.len(), 3);
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
            (
                "rogue/skill-focus",
                "Skill Focus",
                true,
                vec!["skill-focus"]
            ),
            (
                "sorcerer/bloodline/abyssal-feats",
                "Abyssal Bloodline Feats",
                false,
                vec!["power-attack", "toughness"]
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
        [
            MythicAbilityGroup::Class,
            MythicAbilityGroup::Class,
            MythicAbilityGroup::Melee,
            MythicAbilityGroup::Class
        ]
    );
    let groups: Vec<MythicFeatGroup> = data.mythic_feats.iter().map(|f| f.group).collect();
    assert_eq!(
        groups,
        [
            MythicFeatGroup::ArmourAndShield,
            MythicFeatGroup::Melee,
            MythicFeatGroup::Melee
        ]
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
        Some(Prerequisite::All(vec![
            Prerequisite::MythicRank(9),
            Prerequisite::Story(Story::VowOfDahak),
        ]))
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

#[test]
fn reads_weapons_and_what_feats_are_taken_for() {
    let data = read_fixture();
    let weapons: Vec<(&str, Option<WeaponProficiency>)> = data
        .weapons
        .iter()
        .map(|w| (w.key.as_str(), w.proficiency))
        .collect();
    assert_eq!(
        weapons,
        [
            ("dagger", Some(WeaponProficiency::Simple)),
            ("dueling-sword", Some(WeaponProficiency::Exotic)),
            ("longsword", Some(WeaponProficiency::Martial)),
            ("unarmed-strike", None),
        ]
    );
    let feat = |key: &str| data.feats.iter().find(|f| f.key.as_str() == key).unwrap();
    assert_eq!(feat("weapon-focus").parameter, Some(ParameterKind::Weapon));
    assert_eq!(feat("weapon-focus").repeat, Repeat::PerValue);
    assert_eq!(
        feat("spell-focus").parameter,
        Some(ParameterKind::SpellSchool)
    );
    assert_eq!(feat("extra-rage").repeat, Repeat::Times(3));
    assert_eq!(feat("extra-ki").repeat, Repeat::Unlimited);
    // Left out: nothing to take it for, taken once.
    assert_eq!(feat("toughness").parameter, None);
    assert_eq!(feat("toughness").repeat, Repeat::Once);
    assert_eq!(
        feat("weapon-specialization").prerequisite,
        Some(Prerequisite::All(vec![
            Prerequisite::FeatForSameValue(FeatKey::new("weapon-focus")),
            Prerequisite::ClassLevel {
                class: ClassKey::new("fighter"),
                level: 4
            },
        ]))
    );
    assert_eq!(
        feat("aldori-dueling-mastery").prerequisite,
        Some(Prerequisite::FeatFor {
            feat: FeatKey::new("weapon-focus"),
            value: Parameter::Weapon(WeaponKey::new("dueling-sword")),
        })
    );
}

#[test]
fn reads_an_options_chosen_feat_and_repeat_and_defaults_them() {
    let data = read_fixture();
    let rogue = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "rogue")
        .unwrap();
    let option = |key: &str| {
        rogue.selections[0]
            .options
            .iter()
            .find(|o| o.key.as_str() == key)
            .unwrap()
    };
    assert_eq!(
        option("rogue/rogue-talents/weapon-focus").chosen_feat,
        Some(FeatKey::new("weapon-focus"))
    );
    assert_eq!(
        option("rogue/rogue-talents/resiliency").repeat,
        Repeat::Times(2)
    );
    let trick = option("rogue/rogue-talents/combat-trick");
    assert_eq!(
        (trick.chosen_feat.clone(), trick.repeat),
        (None, Repeat::Once)
    );
    // A bonus feat of the player's choosing: a one-feat list of a parameterised feat.
    assert_eq!(
        rogue.feat_lists[0].feats,
        BTreeSet::from([FeatKey::new("skill-focus")])
    );
}

#[test]
fn reads_advancement_counts_as_and_story_conditions_and_defaults_them() {
    let data = read_fixture();
    let class = |key: &str| {
        data.classes
            .iter()
            .chain(&data.prestige_classes)
            .find(|c| c.key.as_str() == key)
            .unwrap()
    };
    assert_eq!(class("loremaster").advances, Advancement::Either);
    assert_eq!(class("mystic-theurge").advances, Advancement::Each);
    assert_eq!(
        class("eldritch-knight").counts_as,
        BTreeSet::from([
            CountsAs {
                class: ClassKey::new("fighter"),
                toward: CountsToward::FeatPrerequisites,
            },
            CountsAs {
                class: ClassKey::new("fighter"),
                toward: CountsToward::Feature(FeatureKey::new("fighter/bravery")),
            },
        ])
    );
    assert_eq!(class("fighter").counts_as, BTreeSet::new());

    let breath = data
        .mythic_paths
        .iter()
        .flat_map(|path| &path.selections)
        .flat_map(|selection| &selection.options)
        .find(|option| option.key.as_str() == "gold-dragon/breaths/annihilating")
        .unwrap();
    assert_eq!(
        breath.prerequisite,
        Some(Prerequisite::All(vec![
            Prerequisite::MythicRank(9),
            Prerequisite::Story(Story::VowOfDahak),
        ]))
    );
}

#[test]
fn reads_options_with_rows_by_option_level_and_defaults_them() {
    let data = read_fixture();
    let sorcerer = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "sorcerer")
        .unwrap();
    assert!(sorcerer.spontaneous);
    let selection = |key: &str| {
        sorcerer
            .selections
            .iter()
            .find(|s| s.key.as_str() == key)
            .unwrap()
    };
    let bloodline = selection("sorcerer/bloodline");
    assert_eq!(
        bloodline
            .option_levels
            .iter()
            .map(ClassKey::as_str)
            .collect::<Vec<_>>(),
        ["fighter", "sorcerer"]
    );
    let abyssal = &bloodline.options[0];
    assert_eq!(
        abyssal.pool.as_ref().map(FeatureSelectionKey::as_str),
        Some("sorcerer/abyssal-powers")
    );
    assert_eq!(
        abyssal.levels[0],
        OptionLevelRow {
            level: 1,
            features: vec![FeatureKey::new("sorcerer/bloodline/abyssal/claws")],
            feats: BTreeSet::new(),
            selections: vec![],
            feat_list: Some(FeatListKey::new("sorcerer/bloodline/abyssal-feats")),
        }
    );
    assert_eq!(
        abyssal.levels[1].feats,
        BTreeSet::from([FeatKey::new("toughness")])
    );
    assert_eq!(
        abyssal.levels[1].selections,
        [FeatureSelectionKey::new("sorcerer/school")]
    );
    assert_eq!(
        sorcerer.levels[2].feat_slots,
        [FeatSlot::OptionList {
            selection: FeatureSelectionKey::new("sorcerer/bloodline")
        }]
    );
    let ascendance = sorcerer
        .features
        .iter()
        .find(|f| f.key.as_str() == "sorcerer/bloodline-ascendance")
        .unwrap();
    assert_eq!(
        ascendance.option_row,
        Some(OptionRow {
            selection: FeatureSelectionKey::new("sorcerer/bloodline"),
            level: 3,
        })
    );
    assert!(
        sorcerer
            .features
            .iter()
            .any(|f| f.key.as_str() == "sorcerer/mixed-bloodline" && f.daily_choice)
    );
    let power = sorcerer
        .selections
        .iter()
        .find(|s| s.key.as_str() == "sorcerer/power")
        .unwrap();
    assert_eq!(
        power.pool_of.as_ref().map(FeatureSelectionKey::as_str),
        Some("sorcerer/bloodline")
    );

    let seeker = &sorcerer.archetypes[0];
    assert_eq!(seeker.spontaneous, Some(false));
    let [own, choice, second] = &seeker.selections[..] else {
        panic!("three selections");
    };
    assert_eq!(own.skipped_levels, BTreeSet::from([3]));
    assert!(second.feat_lists_only && second.only_without_option);
    assert!(!own.feat_lists_only && !own.only_without_option);
    assert_eq!(
        choice
            .options
            .iter()
            .find(|o| o.key.as_str() == "sorcerer/seeker/choice/talents")
            .unwrap()
            .replaces_row_of
            .as_ref()
            .map(FeatureSelectionKey::as_str),
        Some("sorcerer/bloodline")
    );

    let ascendance = data
        .mythic_abilities
        .iter()
        .find(|a| a.key.as_str() == "bloodline-ascendance")
        .unwrap();
    assert_eq!(
        ascendance.features,
        [FeatureKey::new("sorcerer/bloodline-ascendance")]
    );
    let second = data
        .mythic_abilities
        .iter()
        .find(|a| a.key.as_str() == "second-bloodline")
        .unwrap();
    assert_eq!(
        second.selections,
        [FeatureSelectionKey::new("sorcerer/bloodline")]
    );

    // Everything else defaults to none.
    let fighter = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "fighter")
        .unwrap();
    assert!(!fighter.spontaneous);
    let rogue = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "rogue")
        .unwrap();
    let talents = &rogue.selections[0];
    assert!(talents.option_levels.is_empty() && talents.skipped_levels.is_empty());
    assert!(talents.pool_of.is_none() && !talents.feat_lists_only);
    assert!(
        talents
            .options
            .iter()
            .all(|o| o.levels.is_empty() && o.pool.is_none())
    );
    assert!(
        rogue
            .features
            .iter()
            .all(|f| f.option_row.is_none() && !f.daily_choice)
    );
    assert!(rogue.archetypes.iter().all(|a| a.spontaneous.is_none()));
}
