//! Reading and checking the `libs/game-data/` RON files.

mod common;

use std::collections::BTreeSet;
use std::fs;

use common::{copy_fixture, fixture_dir};
use rules::{
    Ability, AbilityBonus, Advancement, ArchetypeKey, ArchetypeLevelRow, ArmourType,
    BabProgression, BackgroundGroup, ClassKey, CountsAs, CountsToward, DataAt, FeatKey,
    FeatListKey, FeatSlot, FeatValue, FeatureKey, FeatureSelectionKey, GameData, LevelFrom,
    MythicAbilityGroup, MythicFeatGroup, MythicPathKey, OptionGroup, OptionLevelRow, OptionRow,
    Parameter, ParameterKind, Prerequisite, Repeat, Save, SaveException, SaveProgression,
    SelectionOptionKey, SkillKey, SpellLevels, Story, WeaponKey, WeaponProficiency,
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
    assert_eq!(data.archetypes.len(), 4);
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
    assert_eq!(
        names(data.deities.iter().map(|d| d.name.as_str()).collect()),
        "Achaekek, Iomedae"
    );
}

/// Ticket 86: a deity's alignments, favoured weapons and domains; a selection's limit by the deity's domains, a
/// blessing's domain, and a feature granting the favoured weapons.
#[test]
fn reads_deities_and_what_they_limit() {
    use rules::{Alignment, DeityDomains, DeityKey};
    let data = read_fixture();
    let iomedae = data
        .deities
        .iter()
        .find(|d| d.key == DeityKey::new("iomedae"))
        .unwrap();
    assert_eq!(
        iomedae.alignments,
        BTreeSet::from([
            Alignment::LawfulGood,
            Alignment::NeutralGood,
            Alignment::LawfulNeutral
        ])
    );
    assert_eq!(
        iomedae.favoured_weapons,
        BTreeSet::from([WeaponKey::new("longsword")])
    );
    assert_eq!(
        iomedae.domains,
        BTreeSet::from([
            SelectionOptionKey::new("cleric/domain/good"),
            SelectionOptionKey::new("cleric/domain/law")
        ])
    );
    let cleric = data.class(&ClassKey::new("cleric")).unwrap();
    assert_eq!(cleric.prerequisite, Some(Prerequisite::Worships));
    let limit = |key: &str| {
        data.selection(&FeatureSelectionKey::new(key))
            .unwrap()
            .deity_domains
    };
    assert_eq!(limit("cleric/domain"), Some(DeityDomains::Offered));
    assert_eq!(limit("cleric/heresy"), Some(DeityDomains::NotOffered));
    let blessing = data
        .selection_option(&SelectionOptionKey::new("cleric/blessings/good"))
        .unwrap();
    assert_eq!(
        blessing.domain,
        Some(SelectionOptionKey::new("cleric/domain/good"))
    );
    assert!(
        data.feature(&FeatureKey::new("cleric/cleric-proficiencies"))
            .unwrap()
            .favoured_weapon_proficiency
    );
    assert!(data.check().is_empty(), "{:?}", data.check());
}

/// A kind's folder may be missing, and reads as no entries of the kind: `deities/` as any other.
#[test]
fn a_missing_deities_folder_reads_as_no_deities() {
    let dir = copy_fixture();
    fs::remove_dir_all(dir.path().join("deities")).unwrap();
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    assert_eq!(data.deities, []);
    assert_eq!(data.feats.len(), read_fixture().feats.len());
}

#[test]
fn reads_class_progressions_and_only_the_archetype_rows_that_differ() {
    let data = read_fixture();
    let class = |key: &str| data.class(&ClassKey::new(key)).unwrap();
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
    let herald = data.archetypes_of(&class("cleric").key).next().unwrap();
    assert_eq!(herald.key.as_str(), "cleric/herald");
    assert_eq!(herald.levels, []);

    // The Eldritch Scoundrel casts arcane spells the rogue doesn't.
    let scoundrel = data.archetypes_of(&class("rogue").key).next().unwrap();
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
    let lists: BTreeSet<(&str, &str, bool, Vec<&str>)> = data
        .feat_lists
        .iter()
        .map(|list| {
            (
                list.key.as_str(),
                list.name.as_str(),
                list.ignores_prerequisites,
                list.feats.iter().map(FeatKey::as_str).collect(),
            )
        })
        .collect();
    // Each in its own file under `feat-lists/`: the half-elf's, the monk's, the Armiger's, the Gold Dragon's and the
    // trickster's among them. Left out, `ignores_prerequisites` and `every_feat` are unset, and `feats` is empty.
    assert_eq!(
        lists,
        BTreeSet::from([
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
        ])
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
    let selection = |key: &str| data.selection(&FeatureSelectionKey::new(key)).unwrap();
    // The rogue's talents share nothing: left out, `shared_options` is empty.
    assert_eq!(
        selection("rogue/rogue-talents").shared_options,
        BTreeSet::new()
    );
    let arcane_talents = selection("rogue/eldritch-scoundrel/arcane-talents");
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
        .archetypes
        .iter()
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
    let feat = dir.path().join("feats/power-attack.ron");
    let text = fs::read_to_string(&feat).unwrap();
    fs::write(&feat, text.replacen("prerequisite:", "prerequisites:", 1)).unwrap();

    let error = seeds::read_game_data_files(dir.path())
        .unwrap_err()
        .to_string();

    assert!(error.contains("power-attack.ron"), "{error}");
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
    fs::write(dir.path().join("fireball.ron"), "()").unwrap();

    let error = seeds::read_game_data_files(dir.path())
        .unwrap_err()
        .to_string();

    assert!(error.contains("fireball.ron"), "{error}");
}

/// Ticket 244, review: a file of another extension under a kind's folder, or under `planner/`'s, fails naming it
/// (a feature saved as `.txt` would otherwise be skipped without a word). A `README.md` there, a hidden file (Finder's `.DS_Store`, an editor's swap file),
/// and any file outside the kinds' folders (the data folder's own README, `notes/`) are no data and read as nothing.
#[test]
fn a_file_of_another_extension_under_a_kinds_folder_fails() {
    for stray in [
        "classes/stray.txt",
        "features/rogue/sneak-attack.ron.bak",
        "planner/features/note.md",
    ] {
        let dir = copy_fixture();
        let path = dir.path().join(stray);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "()").unwrap();

        let error = seeds::read_game_data_files(dir.path()).unwrap_err();

        assert!(
            matches!(&error, seeds::FilesError::NotRon { path: at } if *at == path),
            "{stray}: {error}"
        );
        assert!(error.to_string().contains(stray), "{stray}: {error}");
    }

    let dir = copy_fixture();
    for harmless in [
        "features/README.md",
        "planner/features/README.md",
        "features/rogue/.DS_Store",
        "features/rogue/.sneak-attack.ron.swp",
        "README.md",
        "notes/244-a-note.md",
    ] {
        let path = dir.path().join(harmless);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "no data").unwrap();
    }
    assert_eq!(
        seeds::read_game_data_files(dir.path()).unwrap(),
        read_fixture()
    );
}

/// Data from before ticket 121 has no `spells/` and no `spell-lists/` folder, and reads as no spells and no lists;
/// the spells read without their lists.
#[test]
fn a_folder_without_spells_has_none_and_spells_read_without_their_lists() {
    let dir = copy_fixture();
    fs::remove_dir_all(dir.path().join("spell-lists")).unwrap();
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    assert_eq!(data.spells.len(), 3);
    assert_eq!(data.spell_lists, []);

    fs::remove_dir_all(dir.path().join("spells")).unwrap();
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    assert_eq!(data.spells, []);
    assert_eq!(data.spell_lists, []);
}

/// A spell's file is `spells/<key>.ron`: the guide page and heading of the old `spells/<page>/level-<n>.ron` files
/// are no part of its path, and a file there fails, naming it.
#[test]
fn a_spell_file_in_the_old_page_and_level_layout_fails() {
    let spell = fs::read_to_string(fixture_dir().join("spells/dismiss-spell.ron")).unwrap();
    for stray in [
        "spells/wizard/level-0.ron",
        "spells/wizard/dismiss-spell.ron",
        "spells/level-0.ron",
    ] {
        let dir = copy_fixture();
        fs::remove_file(dir.path().join("spells/dismiss-spell.ron")).unwrap();
        let path = dir.path().join(stray);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &spell).unwrap();

        let error = seeds::read_game_data_files(dir.path()).unwrap_err();

        assert!(
            matches!(&error, seeds::FilesError::Misnamed { key, .. } if key == "dismiss-spell"),
            "{stray}: {error}"
        );
        assert!(error.to_string().contains(stray), "{stray}: {error}");
    }
}

/// Ticket 121: the spells, a file each under `spells/`, and the spell lists, a file each under `spell-lists/`.
#[test]
fn reads_spells_and_spell_lists_and_defaults_them() {
    use rules::{
        CastingTime, SpellCaster, SpellDescriptor, SpellKey, SpellListEntry, SpellListOnly,
        SpellSchool, Story,
    };
    let data = read_fixture();
    let keys: Vec<&str> = data.spells.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(keys, ["bless", "dismiss-spell", "hold-monster"]);
    let [bless, dismiss, hold] = [0, 1, 2].map(|i| &data.spells[i]);
    assert_eq!(dismiss.school, None);
    assert_eq!(dismiss.summary, "");
    assert_eq!(dismiss.duration, None);
    assert_eq!(dismiss.components, None);
    assert_eq!(dismiss.descriptors, BTreeSet::new());
    assert_eq!(hold.school, Some(SpellSchool::Enchantment));
    assert_eq!(
        hold.descriptors,
        BTreeSet::from([SpellDescriptor::MindAffecting, SpellDescriptor::Compulsion])
    );
    assert_eq!(hold.saving_throw, "Will negates");
    assert!(hold.spell_resistance);
    assert_eq!(hold.duration.as_deref(), Some("1 round / level"));
    assert_eq!(bless.components.as_deref(), Some("Holy water"));
    assert_eq!(bless.casting_time, CastingTime::FullRound);

    let lists: Vec<&str> = data.spell_lists.iter().map(|l| l.key.as_str()).collect();
    assert_eq!(lists, ["cleric", "gold-dragon", "sorcerer"]);
    let sorcerer = &data.spell_lists[2];
    assert_eq!(
        sorcerer.owner,
        SpellCaster::Class(ClassKey::new("sorcerer"))
    );
    assert_eq!(
        sorcerer.used_by,
        BTreeSet::from([SpellCaster::Archetype(rules::ArchetypeKey::new(
            "rogue/eldritch-scoundrel"
        ))])
    );
    assert_eq!(data.spell_lists[0].used_by, BTreeSet::new());
    let gold = &data.spell_lists[1];
    assert_eq!(
        gold.owner,
        SpellCaster::Path(MythicPathKey::new("gold-dragon"))
    );
    assert_eq!(
        gold.entries,
        BTreeSet::from([
            SpellListEntry {
                level: 1,
                spell: SpellKey::new("bless"),
                only: Some(SpellListOnly::Story(Story::Uncorrupted)),
            },
            SpellListEntry {
                level: 10,
                spell: SpellKey::new("hold-monster"),
                only: None,
            },
        ])
    );
}

#[test]
fn the_fixture_passes_the_data_check() {
    assert_eq!(read_fixture().check(), []);
}

#[test]
fn checking_the_files_passes_the_fixture() {
    let data = seeds::check_game_data_files(&fixture_dir()).unwrap();

    assert_eq!(*data, read_fixture());
}

#[test]
fn checking_the_files_lists_the_data_problems() {
    let dir = copy_fixture();
    let skills = dir.path().join("skills.ron");
    let text = fs::read_to_string(&skills).unwrap();
    // Two skills with one key, and none with the key the data names elsewhere.
    fs::write(&skills, text.replacen("mobility", "athletics", 1)).unwrap();

    let error = seeds::check_game_data_files(dir.path()).unwrap_err();

    assert!(matches!(&error, seeds::Error::Invalid(_)), "{error}");
    let error = error.to_string();
    assert!(
        error.contains("skill athletics: the key is used twice"),
        "{error}"
    );
    assert!(
        error.contains("names skill mobility, which the data doesn't have"),
        "{error}"
    );
}

/// The check names every key the data lacks, each where it is named, not just that one is missing.
#[test]
fn checking_the_files_names_every_missing_key() {
    let dir = copy_fixture();
    for file in fs::read_dir(dir.path().join("feats")).unwrap() {
        let path = file.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        fs::write(
            &path,
            text.replace("Feat(\"power-attack\")", "Feat(\"no-such-feat\")")
                .replacen(
                    "    repeat: Times(3),\n",
                    "    repeat: Times(3),\n    extra_pick_of: [\"rogue/no-such-selection\"],\n",
                    1,
                ),
        )
        .unwrap();
    }

    let error = seeds::check_game_data_files(dir.path()).unwrap_err();

    let seeds::Error::Invalid(problems) = &error else {
        panic!("{error}");
    };
    assert_eq!(
        problems
            .iter()
            .map(|problem| problem.to_string())
            .collect::<Vec<_>>(),
        [
            "feat cornugon-smash: names feat no-such-feat, which the data doesn't have",
            "feat critical-mastery: names feat no-such-feat, which the data doesn't have",
            "feat extra-rage: names selection rogue/no-such-selection, which the data doesn't have",
        ]
    );
}

#[test]
fn checking_the_files_names_where_each_problem_is() {
    let dir = copy_fixture();
    let fighter = dir.path().join("classes/fighter.ron");
    let text = fs::read_to_string(&fighter).unwrap();
    // The fighter's own level 2 row goes missing, so the Armiger's row there is outside its class too, and the
    // Bravery that row granted, which the Eldritch Knight counts toward, is granted nowhere.
    let row = text
        .lines()
        .find(|line| line.starts_with("        (level: 2,"))
        .unwrap();
    fs::write(&fighter, text.replacen(&format!("{row}\n"), "", 1)).unwrap();

    let error = seeds::check_game_data_files(dir.path()).unwrap_err();

    let seeds::Error::Invalid(problems) = &error else {
        panic!("{error}");
    };
    assert_eq!(
        problems.iter().map(|p| &p.at).collect::<Vec<_>>(),
        [
            &DataAt::Class(ClassKey::new("eldritch-knight")),
            &DataAt::Class(ClassKey::new("fighter")),
            &DataAt::Archetype(ArchetypeKey::new("fighter/armiger"))
        ]
    );
    assert!(error.to_string().contains("class fighter"), "{error}");
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
    let feature = |key: &str| data.feature(&FeatureKey::new(key)).unwrap();
    assert_eq!(
        feature("legend/legend").ability_bonuses,
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
    let dragon_feats = FeatListKey::new("gold-dragon/dragon-feats");
    assert!(dragon.ranks[1].feat_slots.contains(&FeatSlot::OneOf {
        list: dragon_feats.clone()
    }));
    let list = data.feat_lists.iter().find(|l| l.key == dragon_feats);
    assert!(list.unwrap().every_feat);
    let body = feature(dragon.ranks[0].features[1].as_str());
    assert_eq!(body.key.as_str(), "gold-dragon/perfect-body");
    assert_eq!(
        body.ability_bonuses,
        [
            AbilityBonus::Highest { amount: 4 },
            AbilityBonus::AtLeast { score: 14 }
        ]
    );
    let breaths = data.selection(&dragon.ranks[0].selections[0]).unwrap();
    assert_eq!(
        data.options_of(breaths).next().unwrap().prerequisite,
        Some(Prerequisite::All(vec![
            Prerequisite::MythicRank(9),
            Prerequisite::Story(Story::VowOfDahak),
        ]))
    );

    // A feature without bonuses reads as none; an option may grant several feats.
    assert!(
        feature("trickster/trickster-tricks")
            .ability_bonuses
            .is_empty()
    );
    let combat_trick = data
        .selection_option(&SelectionOptionKey::new("rogue/rogue-talents/combat-trick"))
        .unwrap();
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
    let talents = data
        .selection(&FeatureSelectionKey::new("rogue/rogue-talents"))
        .unwrap();
    let option = |key: &str| {
        data.options_of(talents)
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
    let list = data
        .feat_lists
        .iter()
        .find(|l| l.key.as_str() == "rogue/skill-focus")
        .unwrap();
    assert_eq!(list.feats, BTreeSet::from([FeatKey::new("skill-focus")]));
}

#[test]
fn reads_advancement_counts_as_and_story_conditions_and_defaults_them() {
    let data = read_fixture();
    let class = |key: &str| data.class(&ClassKey::new(key)).unwrap();
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
        .selection_option(&SelectionOptionKey::new("gold-dragon/breaths/annihilating"))
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
    let selection = |key: &str| data.selection(&FeatureSelectionKey::new(key)).unwrap();
    let options = |key: &str| data.options_of(selection(key)).collect::<Vec<_>>();
    let bloodline = selection("sorcerer/bloodline");
    let bloodlines = options("sorcerer/bloodline");
    assert_eq!(
        bloodline
            .option_levels
            .iter()
            .map(ClassKey::as_str)
            .collect::<Vec<_>>(),
        ["fighter", "sorcerer"]
    );
    let abyssal = bloodlines[0];
    assert_eq!(
        abyssal
            .aliases
            .iter()
            .map(SelectionOptionKey::as_str)
            .collect::<Vec<_>>(),
        ["sorcerer/bloodline/demonic"]
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
        bloodlines[1].levels[2],
        OptionLevelRow {
            level: 9,
            class_skills: BTreeSet::from([SkillKey::new("mobility"), SkillKey::new("persuasion")]),
            ..OptionLevelRow::default()
        }
    );
    // Ticket 31: an option row's feat slots, and its feats for the value the option's pick named.
    assert_eq!(
        bloodlines[1].levels[1].feat_slots,
        [
            FeatSlot::CombatFeat,
            FeatSlot::OneOf {
                list: FeatListKey::new("sorcerer/bloodline/abyssal-feats")
            }
        ]
    );
    let mastery = options("sorcerer/abyssal-powers")
        .into_iter()
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
    let feature = |key: &str| data.feature(&FeatureKey::new(key)).unwrap();
    let ascendance = feature("sorcerer/bloodline-ascendance");
    assert_eq!(
        ascendance.option_row,
        Some(OptionRow {
            selection: FeatureSelectionKey::new("sorcerer/bloodline"),
            level: 3,
        })
    );
    assert!(feature("sorcerer/mixed-bloodline").daily_choice);
    let power = selection("sorcerer/power");
    assert_eq!(
        power.filtered_by.as_ref().map(FeatureSelectionKey::as_str),
        Some("sorcerer/bloodline")
    );

    let seeker = data.archetypes_of(&sorcerer.key).next().unwrap();
    assert_eq!(seeker.spontaneous, Some(false));
    let seekers: Vec<_> = seeker
        .levels
        .iter()
        .flat_map(|row| &row.selections)
        .map(|key| data.selection(key).unwrap())
        .collect();
    let [own, second, choice] = seekers[..] else {
        panic!("three selections");
    };
    assert_eq!(own.skipped_levels, BTreeSet::from([3]));
    assert_eq!(own.replaced_by.len(), 1);
    assert!(second.feat_lists_only && second.only_without_option);
    assert!(!own.feat_lists_only && !own.only_without_option);
    assert_eq!(
        data.options_of(choice)
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
    let talents = selection("rogue/rogue-talents");
    assert!(talents.option_levels.is_empty() && talents.skipped_levels.is_empty());
    assert!(talents.filtered_by.is_none() && !talents.feat_lists_only);
    assert!(talents.option_level_archetypes.is_empty() && !talents.half_other_classes);
    assert!(
        data.options_of(talents)
            .all(|o| o.levels.is_empty() && o.aliases.is_empty())
    );
    assert!(
        rogue
            .levels
            .iter()
            .flat_map(|row| &row.features)
            .map(|key| feature(key.as_str()))
            .all(|f| f.option_row.is_none() && !f.daily_choice)
    );
    assert!(
        data.archetypes_of(&rogue.key)
            .all(|a| a.spontaneous.is_none())
    );
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
    let adaptability = data
        .feat_lists
        .iter()
        .find(|l| l.key.as_str() == "half-elf/adaptability")
        .unwrap();
    assert_eq!(
        adaptability.feats,
        BTreeSet::from([FeatKey::new("skill-focus")])
    );
    assert!(
        data.races
            .iter()
            .filter(|r| r.key.as_str() != "half-elf")
            .all(|r| !r
                .feat_slots
                .iter()
                .any(|slot| matches!(slot, FeatSlot::OneOf { .. })))
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
    let talents: Vec<_> = data
        .options_of(
            data.selection(&FeatureSelectionKey::new("rogue/rogue-talents"))
                .unwrap(),
        )
        .collect();
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
    let features: Vec<&rules::Feature> = data.features.iter().collect();
    let might = features
        .iter()
        .find(|f| f.key.as_str() == "gold-dragon/dragon-might")
        .unwrap();
    assert_eq!(
        (
            might.bab_at_least_mythic_rank_plus,
            might.skill_ranks_at_least_mythic_rank
        ),
        (Some(15), true)
    );
    assert!(
        features
            .iter()
            .filter(|f| f.key != might.key)
            .all(|f| f.bab_at_least_mythic_rank_plus.is_none()
                && !f.skill_ranks_at_least_mythic_rank)
    );
}

#[test]
fn reads_option_groups_arcanas_level_sources_and_picks_through_and_defaults_them() {
    let data = read_fixture();
    let selection = |key: &str| data.selection(&FeatureSelectionKey::new(key)).unwrap();
    let options = |key: &str| data.options_of(selection(key)).collect::<Vec<_>>();
    // An option's group and arcana, and the prerequisite that looks at the group.
    let bloodline = selection("sorcerer/bloodline");
    let red = options("sorcerer/bloodline")[1];
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
    let abyssal = options("sorcerer/bloodline")[0];
    assert_eq!((abyssal.group, &abyssal.arcana), (None, &None));
    // The selection picked through.
    let necromancy = options("sorcerer/school")[1];
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
    // A filtered pool with an option of its own, levelled by an archetype and half the other classes.
    let power = selection("sorcerer/power");
    assert!(power.filtered_by.is_some() && power.half_other_classes);
    assert_eq!(
        power
            .option_level_archetypes
            .iter()
            .map(ArchetypeKey::as_str)
            .collect::<Vec<_>>(),
        ["sorcerer/seeker"]
    );
    assert_eq!(
        power
            .options
            .iter()
            .map(SelectionOptionKey::as_str)
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
        data.subraces_of(&half_elf.key)
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
        data.subraces
            .iter()
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
    let selection = |key: &str| data.selection(&FeatureSelectionKey::new(key)).unwrap();
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

/// Ticket 239: an entry with no `guide` line loads and checks.
#[test]
fn reads_an_entry_without_a_guide_link() {
    let dir = copy_fixture();
    let deities = dir.path().join("deities/iomedae.ron");
    let text = fs::read_to_string(&deities).unwrap();
    let without = text.replacen(r#"guide: "character-creation#deities", "#, "", 1);
    assert_ne!(without, text, "the fixture's Iomedae had a guide link");
    fs::write(&deities, without).unwrap();

    let data = seeds::read_game_data_files(dir.path()).unwrap();
    let guides: Vec<_> = data.deities.iter().map(|d| d.guide.is_some()).collect();
    assert_eq!(guides.iter().filter(|g| !**g).count(), 1, "{guides:?}");
    assert_eq!(data.check(), []);
}

/// Ticket 244: an entry's file is named by its key, under its kind's folder; a file whose path says another key
/// fails, naming the file and the key it holds.
#[test]
fn a_file_whose_path_isnt_its_entrys_key_fails() {
    let dir = copy_fixture();
    let features = dir.path().join("features");
    fs::rename(
        features.join("rogue/sneak-attack.ron"),
        features.join("fighter/sneak-attack.ron"),
    )
    .unwrap();

    let error = seeds::read_game_data_files(dir.path()).unwrap_err();

    assert!(
        matches!(&error, seeds::FilesError::Misnamed { key, .. } if key == "rogue/sneak-attack"),
        "{error}"
    );
    let error = error.to_string();
    assert!(
        error.contains("features/fighter/sneak-attack.ron"),
        "{error}"
    );
    assert!(error.contains("rogue/sneak-attack"), "{error}");
}

/// Ticket 244: an entry the planner made, with no game blueprint behind it, sits under `planner/<kind folder>/`,
/// wrapped with the reason no blueprint stands behind it; it reads as the entry, and one with an empty reason fails.
#[test]
fn a_planner_entry_reads_with_its_reason_and_fails_without_one() {
    let dir = copy_fixture();
    let own = dir.path().join("features/rogue/sneak-attack.ron");
    let entry = fs::read_to_string(&own).unwrap();
    fs::remove_file(&own).unwrap();
    let planner = dir.path().join("planner/features/rogue/sneak-attack.ron");
    fs::create_dir_all(planner.parent().unwrap()).unwrap();
    let wrapped = |why: &str| format!("(\n    why: \"{why}\",\n    entry: {entry})\n");

    fs::write(
        &planner,
        wrapped("The guide's feature, which no blueprint holds alone."),
    )
    .unwrap();
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    assert_eq!(data, read_fixture());

    fs::write(&planner, wrapped(" ")).unwrap();
    let error = seeds::read_game_data_files(dir.path()).unwrap_err();
    assert!(
        matches!(&error, seeds::FilesError::NoReason { path } if *path == planner),
        "{error}"
    );
}
