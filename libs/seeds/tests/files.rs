//! Reading and hashing the `libs/game-data/` RON files.

mod common;

use std::collections::BTreeSet;
use std::fs;

use common::{copy_fixture, fixture_dir};
use rules::{
    Ability, AbilityBonus, Advancement, ArchetypeLevelRow, ArmourType, BabProgression,
    BackgroundGroup, ClassKey, CountsAs, CountsToward, FeatKey, FeatListKey, FeatSlot, FeatValue,
    FeatureKey, FeatureSelectionKey, GameData, LevelFrom, MythicAbilityGroup, MythicFeatGroup,
    MythicPathKey, OptionGroup, OptionLevelRow, OptionRow, Parameter, ParameterKind, Prerequisite,
    Repeat, Save, SaveException, SaveProgression, SelectionOptionKey, SkillKey, SpellLevels, Story,
    WeaponKey, WeaponProficiency,
};

fn read_fixture() -> GameData {
    seeds::read_game_data_files(&fixture_dir()).expect("fixture reads")
}

#[test]
fn reads_every_fixture_file() {
    let data = read_fixture();

    // Lists come back in canonical order: catalogs by key, whatever order the file has.
    let names = |items: Vec<&str>| items.join(", ");
    assert_eq!(
        names(data.races.iter().map(|r| r.name.as_str()).collect()),
        "Dwarf, Elf, Half-Elf, Human"
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
    assert_eq!(data.mythic_feats.len(), 6);
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
    assert_eq!(
        names(data.backgrounds.iter().map(|b| b.name.as_str()).collect()),
        "Aldori Swordsman, Martial Disciple, None, Shackles Corsair"
    );
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
            feats_for: BTreeSet::from([FeatValue {
                feat: FeatKey::new("skill-focus"),
                value: Parameter::Skill(SkillKey::new("mobility")),
            }]),
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

    let data = seeds::read_game_data_files(dir.path()).unwrap();
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
    // Each in its owner's file: the half-elf's (in the races file), the monk's, the Armiger's (inside the fighter's
    // file), the Gold Dragon's and the trickster's. Left out, `ignores_prerequisites` and `every_feat` are unset,
    // and `feats` is empty.
    assert_eq!(
        lists,
        [
            (
                "half-elf/adaptability",
                "Adaptability",
                true,
                vec!["skill-focus"]
            ),
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
            MythicFeatGroup::Miscellaneous,
            MythicFeatGroup::Miscellaneous,
            MythicFeatGroup::ArmourAndShield,
            MythicFeatGroup::Melee,
            MythicFeatGroup::Magic,
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

    let error = seeds::read_game_data_files(dir.path())
        .unwrap_err()
        .to_string();

    assert!(error.contains("feats.ron"), "{error}");
    assert!(error.contains("prerequisites"), "{error}");
}

#[test]
fn a_missing_required_file_fails() {
    let dir = copy_fixture();
    fs::remove_file(dir.path().join("progression.ron")).unwrap();

    let error = seeds::read_game_data_files(dir.path())
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

    let error = seeds::read_game_data_files(dir.path())
        .unwrap_err()
        .to_string();

    assert!(error.contains("fireball.ron"), "{error}");
}

#[test]
fn the_hash_changes_only_when_a_ron_file_changes() {
    let dir = copy_fixture();
    let original = seeds::game_data_hash(dir.path()).unwrap();
    assert_eq!(original.len(), 64, "hex sha-256");

    fs::write(dir.path().join("README.md"), "not data").unwrap();
    assert_eq!(seeds::game_data_hash(dir.path()).unwrap(), original);

    let skills = dir.path().join("skills.ron");
    let text = fs::read_to_string(&skills).unwrap();
    fs::write(&skills, text.replace("Athletics", "Athletics!")).unwrap();
    assert_ne!(seeds::game_data_hash(dir.path()).unwrap(), original);
}

#[test]
fn the_hash_does_not_depend_on_where_the_folder_is() {
    assert_eq!(
        seeds::game_data_hash(copy_fixture().path()).unwrap(),
        seeds::game_data_hash(&fixture_dir()).unwrap()
    );
}

#[test]
fn the_fixture_passes_the_data_check() {
    assert_eq!(read_fixture().check(), []);
}

#[test]
fn checking_the_files_needs_no_database_and_passes_the_fixture() {
    let data = seeds::check_game_data_files(&fixture_dir()).unwrap();

    assert_eq!(data, read_fixture());
}

#[test]
fn checking_the_files_lists_the_data_problems() {
    let dir = copy_fixture();
    let skills = dir.path().join("skills.ron");
    let text = fs::read_to_string(&skills).unwrap();
    // Two skills with one key.
    fs::write(&skills, text.replacen("mobility", "athletics", 1)).unwrap();

    let error = seeds::check_game_data_files(dir.path()).unwrap_err();

    assert!(
        matches!(&error, seeds::Error::Invalid(problems) if problems.len() == 1),
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
            feat_list: Some(FeatListKey::new("sorcerer/bloodline/abyssal-feats")),
            class_skills: BTreeSet::from([SkillKey::new("athletics")]),
            ..OptionLevelRow::default()
        }
    );
    // Ticket 37: an option row's class skills, and a row holding nothing else.
    assert_eq!(
        bloodline.options[1].levels[2],
        OptionLevelRow {
            level: 9,
            class_skills: BTreeSet::from([SkillKey::new("mobility"), SkillKey::new("persuasion")]),
            ..OptionLevelRow::default()
        }
    );
    // Ticket 31: an option row's feat slots, and its feats for the value the option's pick named.
    assert_eq!(
        bloodline.options[1].levels[1].feat_slots,
        [
            FeatSlot::CombatFeat,
            FeatSlot::OneOf {
                list: FeatListKey::new("sorcerer/bloodline/abyssal-feats")
            }
        ]
    );
    let mastery = selection("sorcerer/abyssal-powers")
        .options
        .iter()
        .find(|o| o.key.as_str() == "sorcerer/abyssal-powers/mastery")
        .unwrap();
    assert_eq!(mastery.chosen_feat, Some(FeatKey::new("weapon-focus")));
    assert_eq!(
        mastery.levels[0].chosen_feats,
        BTreeSet::from([FeatKey::new("weapon-specialization")])
    );
    assert_eq!(
        mastery.levels[0].feats,
        BTreeSet::from([FeatKey::new("toughness")])
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
        sorcerer.levels[2].feat_slots[0],
        FeatSlot::OptionList {
            selection: FeatureSelectionKey::new("sorcerer/bloodline"),
            at_level: None,
            label: None,
        }
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

#[test]
fn reads_values_granted_asked_about_and_offered_and_defaults_them() {
    let data = read_fixture();
    // A race's own list.
    let half_elf = data
        .races
        .iter()
        .find(|r| r.key.as_str() == "half-elf")
        .unwrap();
    assert_eq!(
        half_elf.feat_slots,
        [FeatSlot::OneOf {
            list: FeatListKey::new("half-elf/adaptability")
        }]
    );
    assert_eq!(
        half_elf.feat_lists[0].feats,
        BTreeSet::from([FeatKey::new("skill-focus")])
    );
    assert!(
        data.races
            .iter()
            .filter(|r| r.key.as_str() != "half-elf")
            .all(|r| r.feat_lists.is_empty())
    );

    // A feat granted for one value on a class row; none elsewhere.
    let rogue = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "rogue")
        .unwrap();
    assert_eq!(
        rogue.levels[0].feats_for,
        BTreeSet::from([FeatValue {
            feat: FeatKey::new("weapon-focus"),
            value: Parameter::Weapon(WeaponKey::new("dagger")),
        }])
    );
    assert!(rogue.levels[1].feats_for.is_empty());

    // An option's own value from a short list; others take none.
    let talents = &rogue.selections[0].options;
    let training = talents
        .iter()
        .find(|o| o.key.as_str() == "rogue/rogue-talents/weapon-training")
        .unwrap();
    assert_eq!(training.parameter, Some(ParameterKind::Weapon));
    assert_eq!(
        training.values,
        BTreeSet::from([
            Parameter::Weapon(WeaponKey::new("dagger")),
            Parameter::Weapon(WeaponKey::new("longsword"))
        ])
    );
    assert!(
        talents
            .iter()
            .filter(|o| o.key != training.key)
            .all(|o| o.parameter.is_none() && o.values.is_empty())
    );

    // Mythic feats: a value, a repeat, the racial penalty; defaults elsewhere.
    let mythic = |key: &str| {
        data.mythic_feats
            .iter()
            .find(|f| f.key.as_str() == key)
            .unwrap()
    };
    let spell_focus = mythic("spell-focus-mythic");
    assert_eq!(spell_focus.parameter, Some(ParameterKind::SpellSchool));
    assert_eq!(spell_focus.repeat, Repeat::PerValue);
    assert_eq!(mythic("extra-mythic-power").repeat, Repeat::Times(2));
    assert!(mythic("destiny-beyond-birth").removes_racial_penalty);
    let power_attack = mythic("power-attack-mythic");
    assert_eq!(
        (
            power_attack.parameter,
            power_attack.repeat,
            power_attack.removes_racial_penalty
        ),
        (None, Repeat::Once, false)
    );
    assert_eq!(
        mythic("destiny-beyond-birth").prerequisite,
        Some(Prerequisite::Not(Box::new(Prerequisite::CharacterLevel(
            11
        ))))
    );

    // Armor Focus asks about the pick's value.
    let armor_focus = data
        .feats
        .iter()
        .find(|f| f.key.as_str() == "armor-focus")
        .unwrap();
    assert!(matches!(
        &armor_focus.prerequisite,
        Some(Prerequisite::All(children)) if children.contains(&Prerequisite::FeatIfValue {
            value: Parameter::ArmourType(ArmourType::Heavy),
            feat: FeatKey::new("toughness"),
        })
    ));

    // A feature's floors; none elsewhere.
    let features: Vec<&rules::Feature> =
        data.mythic_paths.iter().flat_map(|p| &p.features).collect();
    let might = features
        .iter()
        .find(|f| f.key.as_str() == "gold-dragon/dragon-might")
        .unwrap();
    assert_eq!(
        (might.bab_floor, might.skill_ranks_at_least_mythic_rank),
        (Some(15), true)
    );
    assert!(
        features
            .iter()
            .filter(|f| f.key != might.key)
            .all(|f| f.bab_floor.is_none() && !f.skill_ranks_at_least_mythic_rank)
    );
}

#[test]
fn reads_option_groups_arcanas_level_sources_and_picks_through_and_defaults_them() {
    let data = read_fixture();
    let sorcerer = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "sorcerer")
        .unwrap();
    let selection = |key: &str| {
        sorcerer
            .selections
            .iter()
            .chain(sorcerer.archetypes.iter().flat_map(|a| &a.selections))
            .find(|s| s.key.as_str() == key)
            .unwrap()
    };
    // An option's group and arcana, and the prerequisite that looks at the group.
    let bloodline = selection("sorcerer/bloodline");
    let red = &bloodline.options[1];
    assert_eq!(red.group, Some(OptionGroup::Draconic));
    assert_eq!(
        red.arcana,
        Some(FeatureKey::new("sorcerer/bloodline/draconic/arcana"))
    );
    assert_eq!(
        red.prerequisite,
        Some(Prerequisite::All(vec![
            Prerequisite::NoOptionFrom(FeatureSelectionKey::new("sorcerer/bloodline")),
            Prerequisite::NoOtherOptionOfGroup(OptionGroup::Draconic),
        ]))
    );
    let abyssal = &bloodline.options[0];
    assert_eq!((abyssal.group, &abyssal.arcana), (None, &None));
    // The selection picked through.
    let necromancy = &selection("sorcerer/school").options[1];
    assert_eq!(
        necromancy.prerequisite,
        Some(Prerequisite::Any(vec![
            Prerequisite::Not(Box::new(Prerequisite::PickedThrough(
                FeatureSelectionKey::new("sorcerer/school")
            ))),
            Prerequisite::Alignment(rules::Alignment::NeutralEvil),
        ]))
    );
    // Where option levels come from, and how they lag.
    assert_eq!(
        selection("sorcerer/school").level_from,
        Some(LevelFrom::GrantingOption)
    );
    assert_eq!(
        selection("sorcerer/abyssal-powers").level_from,
        Some(LevelFrom::OtherOptions)
    );
    assert_eq!(bloodline.level_from, None);
    assert_eq!(selection("sorcerer/seeker/bloodline").levels_behind, 2);
    assert_eq!(bloodline.levels_behind, 0);
    // Once per option through a selection.
    assert!(selection("sorcerer/seeker/second-bloodline").once_per_option);
    assert!(!bloodline.once_per_option);
    // A pool selection with an option of its own.
    let power = selection("sorcerer/power");
    assert!(power.pool_of.is_some());
    assert_eq!(
        power
            .options
            .iter()
            .map(|o| o.key.as_str())
            .collect::<Vec<_>>(),
        ["sorcerer/power/surge"]
    );
}

#[test]
fn reads_subrace_slots_option_list_stages_and_options_levelled_alongside_and_defaults_them() {
    let data = read_fixture();
    // A subrace's own slots, naming its race's list; another with none.
    let half_elf = data
        .races
        .iter()
        .find(|r| r.key.as_str() == "half-elf")
        .unwrap();
    let subrace = |key: &str| {
        half_elf
            .subraces
            .iter()
            .find(|s| s.key.as_str() == key)
            .unwrap()
    };
    assert_eq!(
        subrace("half-elf/adaptable").feat_slots,
        [FeatSlot::OneOf {
            list: FeatListKey::new("half-elf/adaptability")
        }]
    );
    assert_eq!(subrace("half-elf/dual-heritage").feat_slots, []);
    assert!(
        data.races
            .iter()
            .flat_map(|r| &r.subraces)
            .filter(|s| s.key.as_str() != "half-elf/adaptable")
            .all(|s| s.feat_slots.is_empty())
    );
    // An option list slot fixed at an option level, beside one that follows the option's level.
    let sorcerer = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "sorcerer")
        .unwrap();
    let bloodline = FeatureSelectionKey::new("sorcerer/bloodline");
    assert_eq!(
        sorcerer.levels[2].feat_slots,
        [
            FeatSlot::OptionList {
                selection: bloodline.clone(),
                at_level: None,
                label: None,
            },
            FeatSlot::OptionList {
                selection: bloodline,
                at_level: Some(1),
                label: None,
            },
        ]
    );
    // An option counting beside the other options held.
    let selection = |key: &str| {
        sorcerer
            .selections
            .iter()
            .chain(sorcerer.archetypes.iter().flat_map(|a| &a.selections))
            .find(|s| s.key.as_str() == key)
            .unwrap()
    };
    assert_eq!(
        selection("sorcerer/abyssal-powers").level_also_from,
        Some(SelectionOptionKey::new("sorcerer/seeker/choice/power"))
    );
    assert_eq!(selection("sorcerer/bloodline").level_also_from, None);
}

#[test]
fn reads_backgrounds_and_defaults_them() {
    let data = read_fixture();
    let background = |key: &str| {
        data.backgrounds
            .iter()
            .find(|b| b.key.as_str() == key)
            .unwrap()
    };
    let skills =
        |keys: &[&str]| -> BTreeSet<SkillKey> { keys.iter().map(|k| SkillKey::new(*k)).collect() };

    let corsair = background("shackles-corsair");
    assert_eq!(corsair.group, Some(BackgroundGroup::Regional));
    assert_eq!(corsair.note, "scimitar; +2 CMD against trip");
    assert_eq!(corsair.class_skills, skills(&["athletics", "mobility"]));
    assert!(corsair.feats.is_empty() && corsair.feats_for.is_empty());

    assert_eq!(
        background("martial-disciple").feats,
        BTreeSet::from([FeatKey::new("improved-unarmed-strike")])
    );
    assert_eq!(
        background("aldori-swordsman").feats_for,
        BTreeSet::from([FeatValue {
            feat: FeatKey::new("exotic-weapon-proficiency"),
            value: Parameter::Weapon(WeaponKey::new("dueling-sword")),
        }])
    );

    let none = background("none");
    assert_eq!(none.group, None);
    assert_eq!(none.note, "");
    assert!(none.class_skills.is_empty() && none.feats.is_empty() && none.feats_for.is_empty());
}
