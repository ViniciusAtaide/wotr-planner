//! Seeding the SQLite database from the `libs/game-data/` files and reading it back.
//!
//! A test that seeds gets a [`TestDatabase`], a database file in its own throwaway folder, which the seed
//! builds and renames into place as it does the dev database. One that only needs the empty, migrated
//! schema is a `#[sqlx::test]`, which makes its own file under `target/sqlx/test-dbs`. Neither touches the
//! dev database.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{copy_fixture, fixture_dir};
use rules::{
    Ability, AbilityModifier, Alignment, ArchetypeKey, ArmourType, ClassKey, DataAt, Energy, Feat,
    FeatCategory, FeatKey, FeatureKey, FeatureSelectionKey, GameData, GuideRef, MythicAbilityKey,
    MythicFeatKey, OptionGroup, Parameter, ParameterKind, Prerequisite, RaceKey, Repeat,
    SelectionOptionKey, SkillKey, SpellSchool, SpellType, Story, WeaponKey,
};
use seeds::{Database, GameDataStatus};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

/// A database file in a throwaway folder, which [`seed`](Self::seed) makes.
struct TestDatabase {
    path: PathBuf,
    _dir: tempfile::TempDir,
}

impl TestDatabase {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        Self {
            path: dir.path().join("game-data.db"),
            _dir: dir,
        }
    }

    async fn seed(&self, dir: &Path) -> Result<String, seeds::Error> {
        seeds::seed(&self.path, dir).await
    }

    async fn load(&self) -> Result<GameData, seeds::Error> {
        seeds::load_game_data(&Database::open(&self.path).await?).await
    }

    async fn status(&self, dir: &Path) -> Result<GameDataStatus, seeds::Error> {
        seeds::game_data_status(&Database::open(&self.path).await?, dir).await
    }

    /// A writable pool on the file, to corrupt it. A reseed replaces the file, so take a new one after it. One
    /// connection, so the rollback a failed commit leaves behind runs before the next statement.
    async fn writer(&self) -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(SqliteConnectOptions::new().filename(&self.path))
            .await
            .unwrap()
    }
}

/// Runs `sql` in a transaction and returns why it failed, lowercased (SQLite says `CHECK constraint failed`,
/// `FOREIGN KEY constraint failed`, `UNIQUE constraint failed`). Foreign keys are deferred: they fail at commit.
async fn rejected(pool: &SqlitePool, sql: &'static str) -> String {
    let mut tx = pool.begin().await.unwrap();
    let result = match sqlx::query(sql).execute(&mut *tx).await {
        Ok(_) => tx.commit().await.map(|_| ()),
        Err(error) => Err(error),
    };
    result.unwrap_err().to_string().to_lowercase()
}

#[tokio::test]
async fn files_to_database_to_game_data_equals_files_to_game_data() {
    let test = TestDatabase::new();
    // Neither the files nor the tables carry an order, so both sides come back canonical.
    let from_files = seeds::read_game_data_files(&fixture_dir()).unwrap();

    test.seed(&fixture_dir()).await.unwrap();
    let from_database = test.load().await.unwrap();

    assert_eq!(from_database, from_files.clone().canonical());
    assert_eq!(from_database, from_files);
}

/// The fixture has no save exceptions, so add one, and the archetype rows that set spell levels and the ones that
/// don't both come back.
#[tokio::test]
async fn save_exceptions_and_archetype_rows_round_trip() {
    let test = TestDatabase::new();
    let dir = copy_fixture();
    let fighter = dir.path().join("classes/fighter.ron");
    let text = fs::read_to_string(&fighter).unwrap();
    fs::write(
        &fighter,
        text.replacen(
            "    will: Poor,\n",
            "    will: Poor,\n    save_exceptions: [(level: 2, save: Will, value: 1), (level: 3, save: Reflex, value: 4)],\n",
            1,
        ),
    )
    .unwrap();
    let from_files = seeds::read_game_data_files(dir.path()).unwrap();
    let fighter = from_files
        .classes
        .iter()
        .find(|c| c.key.as_str() == "fighter");
    assert_eq!(fighter.unwrap().save_exceptions.len(), 2);

    test.seed(dir.path()).await.unwrap();

    assert_eq!(test.load().await.unwrap(), from_files);
}

/// An option taken once per value of its chosen feat and an option list slot's own label (ticket 47) are stored and
/// read back.
#[tokio::test]
async fn a_per_value_option_and_a_labelled_slot_round_trip() {
    let test = TestDatabase::new();
    let dir = copy_fixture();
    let edit = |file: &str, from: &str, to: &str| {
        let path = dir.path().join(file);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains(from), "{file}: {from}");
        fs::write(&path, text.replacen(from, to, 1)).unwrap();
    };
    edit(
        "classes/rogue.ron",
        "chosen_feat: \"weapon-focus\",",
        "chosen_feat: \"weapon-focus\", repeat: PerValue,",
    );
    edit(
        "classes/sorcerer.ron",
        "feat_slots: [OptionList(selection: \"sorcerer/bloodline\"),",
        "feat_slots: [OptionList(selection: \"sorcerer/bloodline\", label: \"Wild talent feat\"),",
    );
    let from_files = seeds::read_game_data_files(dir.path()).unwrap();
    assert_eq!(from_files.check(), []);
    let labelled = from_files
        .classes
        .iter()
        .flat_map(|class| &class.levels)
        .flat_map(|row| &row.feat_slots)
        .filter(|slot| matches!(slot, rules::FeatSlot::OptionList { label: Some(_), .. }))
        .count();
    assert_eq!(labelled, 1);

    test.seed(dir.path()).await.unwrap();

    assert_eq!(test.load().await.unwrap(), from_files);
}

/// A feat's extra pick (ticket 49: Extra Hex, a witch's or a shaman's hex) is stored as one row per selection and
/// read back; a selection the data lacks is refused.
#[tokio::test]
async fn a_feats_extra_pick_round_trips() {
    let test = TestDatabase::new();
    let dir = copy_fixture();
    let path = dir.path().join("feats.ron");
    let text = fs::read_to_string(&path).unwrap();
    let from = "        guide: \"feats#extra-rage\",\n        repeat: Times(3),\n";
    assert!(text.contains(from));
    fs::write(
        &path,
        text.replacen(
            from,
            "        guide: \"feats#extra-rage\",\n        repeat: Times(3),\n        extra_pick_of: [\"rogue/rogue-talents\", \"sorcerer/bloodline\"],\n",
            1,
        ),
    )
    .unwrap();
    let from_files = seeds::read_game_data_files(dir.path()).unwrap();
    assert_eq!(from_files.check(), []);
    let picks = &from_files
        .feats
        .iter()
        .find(|feat| feat.key.as_str() == "extra-rage")
        .unwrap()
        .extra_pick_of;
    assert_eq!(picks.len(), 2);

    test.seed(dir.path()).await.unwrap();
    let pool = test.writer().await;
    assert_eq!(test.load().await.unwrap(), from_files);

    let error = sqlx::query(
        "INSERT INTO feat_extra_pick (feat_key, selection_key) VALUES ('extra-ki', 'rogue/no-such-selection')",
    )
    .execute(&pool)
    .await
    .unwrap_err();
    assert!(
        error.to_string().to_lowercase().contains("foreign key"),
        "{error}"
    );
}

/// A mythic feat's extra pick (ticket 55: Extra Feat, a feat; Extra Mythic Ability, a mythic ability) is stored in
/// its row and read back; any other kind of pick is refused.
#[tokio::test]
async fn a_mythic_feats_extra_pick_round_trips() {
    let test = TestDatabase::new();
    let dir = copy_fixture();
    let path = dir.path().join("mythic/feats.ron");
    let text = fs::read_to_string(&path).unwrap();
    let end = text.rfind(']').unwrap();
    let added = r#"    (key: "extra-feat", name: "Extra Feat", group: Miscellaneous, guide: "mythic-paths#extra-feat", extra_pick: Feat),
    (key: "extra-mythic-ability", name: "Extra Mythic Ability", group: Miscellaneous, guide: "mythic-paths#extra-mythic-ability", extra_pick: MythicAbility),
"#;
    fs::write(&path, format!("{}{added}{}", &text[..end], &text[end..])).unwrap();
    let from_files = seeds::read_game_data_files(dir.path()).unwrap();
    assert_eq!(from_files.check(), []);
    let picks: Vec<_> = from_files
        .mythic_feats
        .iter()
        .filter_map(|feat| feat.extra_pick)
        .collect();
    assert_eq!(
        picks,
        [
            rules::MythicExtraPick::Feat,
            rules::MythicExtraPick::MythicAbility
        ]
    );

    test.seed(dir.path()).await.unwrap();
    let pool = test.writer().await;
    assert_eq!(test.load().await.unwrap(), from_files);

    let error =
        sqlx::query("UPDATE mythic_feat SET extra_pick = 'selection' WHERE key = 'extra-feat'")
            .execute(&pool)
            .await
            .unwrap_err();
    assert!(error.to_string().contains("mythic_extra_pick"), "{error}");
}

/// A feature or selection granted twice by one row is one `granted_*` row with a count of 2, and reads back as
/// two grants.
#[tokio::test]
async fn repeated_grants_are_stored_as_counts_and_read_back_in_full() {
    let test = TestDatabase::new();
    let dir = copy_fixture();
    let rogue = dir.path().join("classes/rogue.ron");
    let text = fs::read_to_string(&rogue).unwrap();
    let text = text
        .replacen(
            r#"(level: 1, features: ["rogue/sneak-attack"],"#,
            r#"(level: 1, features: ["rogue/sneak-attack", "rogue/sneak-attack"],"#,
            1,
        )
        .replacen(
            r#"(level: 2, selections: ["rogue/rogue-talents"]),"#,
            r#"(level: 2, selections: ["rogue/rogue-talents", "rogue/rogue-talents", "rogue/rogue-talents"]),"#,
            1,
        );
    fs::write(&rogue, text).unwrap();
    let from_files = seeds::read_game_data_files(dir.path()).unwrap();

    test.seed(dir.path()).await.unwrap();
    let pool = test.writer().await;

    let counts: Vec<(String, i16)> = sqlx::query_as(
        "SELECT g.feature_key, g.count FROM granted_feature g
        JOIN class_level l ON l.id = g.class_level_id
        WHERE l.class_key = 'rogue' AND l.archetype_key IS NULL AND l.level = 1",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(counts, [("rogue/sneak-attack".to_owned(), 2)]);
    let counts: Vec<(String, i16)> = sqlx::query_as(
        "SELECT g.selection_key, g.count FROM granted_selection g
        JOIN class_level l ON l.id = g.class_level_id
        WHERE l.class_key = 'rogue' AND l.archetype_key IS NULL AND l.level = 2",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(counts, [("rogue/rogue-talents".to_owned(), 3)]);
    assert_eq!(test.load().await.unwrap(), from_files);
}

#[tokio::test]
async fn seeding_again_replaces_the_previous_data() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let edited = copy_fixture();
    fs::remove_dir_all(edited.path().join("prestige")).unwrap();
    let skills = edited.path().join("skills.ron");
    let text = fs::read_to_string(&skills).unwrap();
    fs::write(
        &skills,
        text.replace("\"Athletics\"", "\"Athletics (edited)\""),
    )
    .unwrap();

    test.seed(edited.path()).await.unwrap();
    let reloaded = test.load().await.unwrap();

    assert!(reloaded.prestige_classes.is_empty());
    assert_eq!(reloaded.skills[0].name, "Athletics (edited)");
    assert_eq!(
        reloaded,
        seeds::read_game_data_files(edited.path()).unwrap()
    );
}

#[tokio::test]
async fn a_seed_that_fails_leaves_the_previous_data_in_place() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let broken = copy_fixture();
    let feats = broken.path().join("feats.ron");
    let text = fs::read_to_string(&feats).unwrap();
    fs::write(
        &feats,
        text.replace("Feat(\"power-attack\")", "Feat(\"no-such-feat\")"),
    )
    .unwrap();

    let error = test.seed(broken.path()).await.unwrap_err().to_string();

    assert!(error.contains("foreign key"), "{error}");
    assert_eq!(
        test.load().await.unwrap(),
        seeds::read_game_data_files(&fixture_dir()).unwrap()
    );
    assert_eq!(
        test.status(&fixture_dir()).await.unwrap(),
        GameDataStatus::Current
    );
}

/// The foreign key check names every key the data lacks, not just that one is missing, and the new file it was
/// building is gone.
#[tokio::test]
async fn a_failed_seed_names_every_missing_key_and_leaves_no_new_file() {
    let test = TestDatabase::new();
    let broken = copy_fixture();
    let feats = broken.path().join("feats.ron");
    let text = fs::read_to_string(&feats).unwrap();
    fs::write(
        &feats,
        text.replace("Feat(\"power-attack\")", "Feat(\"no-such-feat\")")
            .replacen(
                "        repeat: Times(3),\n",
                "        repeat: Times(3),\n        extra_pick_of: [\"rogue/no-such-selection\"],\n",
                1,
            ),
    )
    .unwrap();

    let error = test.seed(broken.path()).await.unwrap_err();

    let seeds::Error::MissingReferences(missing) = &error else {
        panic!("{error}");
    };
    let mut named: Vec<_> = missing
        .iter()
        .map(|reference| {
            (
                reference.table.as_str(),
                reference.columns.clone(),
                reference.parent.as_str(),
            )
        })
        .collect();
    named.sort();
    named.dedup();
    assert_eq!(
        named,
        [
            (
                "feat_extra_pick",
                vec![(
                    "selection_key".to_owned(),
                    "'rogue/no-such-selection'".to_owned()
                )],
                "feature_selection"
            ),
            (
                "prerequisite_node",
                vec![("feat_key".to_owned(), "'no-such-feat'".to_owned())],
                "feat"
            ),
        ]
    );
    assert!(
        error
            .to_string()
            .contains("feat_key 'no-such-feat' is not in feat key"),
        "{error}"
    );
    assert!(!test.path.exists());
    assert!(!test.path.with_extension("db.new").exists());
}

#[tokio::test]
async fn the_seed_refuses_data_that_fails_the_data_check() {
    let test = TestDatabase::new();
    let broken = copy_fixture();
    let fighter = broken.path().join("classes/fighter.ron");
    let text = fs::read_to_string(&fighter).unwrap();
    // The fighter's own level 2 row goes missing, so the Armiger's row there is outside its class too, and the
    // Bravery that row granted, which the Eldritch Knight counts toward, is granted nowhere.
    let row = text
        .lines()
        .find(|line| line.starts_with("        (level: 2,"))
        .unwrap();
    fs::write(&fighter, text.replacen(&format!("{row}\n"), "", 1)).unwrap();

    let error = test.seed(broken.path()).await.unwrap_err();

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
    // Nothing was written: there is still no database.
    assert!(!test.path.exists());
}

#[sqlx::test]
async fn status_is_never_seeded_before_the_first_seed(pool: SqlitePool) {
    let db = Database::from(pool);
    assert_eq!(
        seeds::game_data_status(&db, &fixture_dir()).await.unwrap(),
        GameDataStatus::NeverSeeded
    );
}

#[tokio::test]
async fn status_is_stale_once_a_data_file_changes_after_seeding() {
    let test = TestDatabase::new();
    let dir = copy_fixture();
    let seeded = test.seed(dir.path()).await.unwrap();
    assert_eq!(
        test.status(dir.path()).await.unwrap(),
        GameDataStatus::Current
    );

    let races = dir.path().join("races.ron");
    let text = fs::read_to_string(&races).unwrap();
    fs::write(&races, text.replace("\"Dwarf\"", "\"Mountain Dwarf\"")).unwrap();

    assert_eq!(
        test.status(dir.path()).await.unwrap(),
        GameDataStatus::Stale {
            seeded,
            files: seeds::game_data_hash(dir.path()).unwrap(),
        }
    );
}

/// One of each leaf condition, naming fixture keys, plus each kind of group. The `match` fails to compile when a
/// variant is added, so this list can't fall behind.
fn every_prerequisite_variant() -> Vec<Prerequisite> {
    use Prerequisite as P;
    let leaves = vec![
        P::Feat(FeatKey::new("power-attack")),
        P::MythicFeat(MythicFeatKey::new("power-attack-mythic")),
        P::MythicAbility(MythicAbilityKey::new("mythic-charge")),
        P::ClassLevel {
            class: ClassKey::new("fighter"),
            level: 2,
        },
        P::Archetype(ArchetypeKey::new("rogue/eldritch-scoundrel")),
        P::Feature {
            feature: FeatureKey::new("rogue/sneak-attack"),
            count: 2,
        },
        P::SelectionOption(SelectionOptionKey::new("rogue/rogue-talents/combat-trick")),
        P::SkillRanks {
            skill: SkillKey::new("mobility"),
            ranks: 4,
        },
        P::AbilityScore {
            ability: Ability::Wisdom,
            score: 13,
        },
        P::BaseAttackBonus(6),
        P::SpellLevel {
            spell_type: SpellType::Divine,
            level: 3,
        },
        P::CasterLevel(5),
        P::CharacterLevel(7),
        P::MythicRank(9),
        P::Alignment(Alignment::LawfulNeutral),
        P::Race(RaceKey::new("dwarf")),
        // One per value column.
        P::FeatFor {
            feat: FeatKey::new("skill-focus"),
            value: Parameter::Skill(SkillKey::new("mobility")),
        },
        P::FeatFor {
            feat: FeatKey::new("weapon-focus"),
            value: Parameter::Weapon(WeaponKey::new("dueling-sword")),
        },
        P::FeatFor {
            feat: FeatKey::new("spell-focus"),
            value: Parameter::SpellSchool(SpellSchool::Necromancy),
        },
        P::FeatFor {
            feat: FeatKey::new("elemental-focus"),
            value: Parameter::Energy(Energy::Cold),
        },
        P::FeatFor {
            feat: FeatKey::new("armor-focus"),
            value: Parameter::ArmourType(ArmourType::Medium),
        },
        P::FeatForSameValue(FeatKey::new("weapon-focus")),
        // Ticket 32: the pick's value asked about (stored like `FeatFor`; the fixture's Armor Focus has two more).
        P::FeatIfValue {
            value: Parameter::Weapon(WeaponKey::new("dagger")),
            feat: FeatKey::new("dodge"),
        },
        P::Story(Story::VowOfDahak),
        P::Story(Story::Uncorrupted),
        P::NoOptionFrom(FeatureSelectionKey::new("sorcerer/bloodline")),
        P::SpontaneousSpellLevel {
            spell_type: SpellType::Arcane,
            level: 2,
        },
        // Ticket 33: the pick's group and the selection it's picked through.
        P::NoOtherOptionOfGroup(OptionGroup::Draconic),
        P::NoOtherOptionOfGroup(OptionGroup::Elemental),
        // Ticket 35: the bloodrager's groups.
        P::NoOtherOptionOfGroup(OptionGroup::BloodragerDraconic),
        P::NoOtherOptionOfGroup(OptionGroup::BloodragerElemental),
        P::PickedThrough(FeatureSelectionKey::new("sorcerer/school")),
        // Ticket 86: the faith.
        P::Deity(rules::DeityKey::new("achaekek")),
        P::Atheist,
        P::Worships,
        P::DeityOffers(SelectionOptionKey::new("cleric/domain/good")),
    ];
    let groups = vec![
        P::Any(vec![P::BaseAttackBonus(1), P::CasterLevel(1)]),
        P::AtLeast(1, vec![P::CharacterLevel(2), P::CasterLevel(2)]),
        // A `not` of a leaf and of a group.
        P::Not(Box::new(P::CharacterLevel(12))),
        P::Not(Box::new(P::All(vec![
            P::BaseAttackBonus(3),
            P::Not(Box::new(P::CasterLevel(3))),
        ]))),
    ];
    let all = [leaves, groups].concat();
    for prerequisite in &all {
        match prerequisite {
            P::All(_)
            | P::Any(_)
            | P::AtLeast(..)
            | P::Not(_)
            | P::Feat(_)
            | P::MythicFeat(_)
            | P::MythicAbility(_)
            | P::ClassLevel { .. }
            | P::Archetype(_)
            | P::Feature { .. }
            | P::SelectionOption(_)
            | P::SkillRanks { .. }
            | P::AbilityScore { .. }
            | P::BaseAttackBonus(_)
            | P::SpellLevel { .. }
            | P::CasterLevel(_)
            | P::CharacterLevel(_)
            | P::MythicRank(_)
            | P::Alignment(_)
            | P::Race(_)
            | P::FeatFor { .. }
            | P::FeatForSameValue(_)
            | P::FeatIfValue { .. }
            | P::Story(_)
            | P::NoOptionFrom(_)
            | P::NoOtherOptionOfGroup(_)
            | P::PickedThrough(_)
            | P::SpontaneousSpellLevel { .. }
            | P::Deity(_)
            | P::Atheist
            | P::Worships
            | P::DeityOffers(_) => {}
        }
    }
    all
}

#[tokio::test]
async fn every_prerequisite_variant_round_trips() {
    let test = TestDatabase::new();
    let mut game_data = seeds::read_game_data_files(&fixture_dir()).unwrap();
    game_data.feats.push(Feat {
        key: FeatKey::new("everything"),
        name: "Everything".into(),
        category: FeatCategory::Miscellaneous,
        summary: String::new(),
        guide: GuideRef::new("feats#everything"),
        // Taken for a weapon, so "the same weapon" means something.
        parameter: Some(ParameterKind::Weapon),
        repeat: Repeat::Times(2),
        extra_pick_of: std::collections::BTreeSet::new(),
        prerequisite: Some(Prerequisite::All(every_prerequisite_variant())),
    });
    // The leaves about the pick belong in an option's prerequisite.
    let Some(Prerequisite::All(every)) = &mut game_data.feats.last_mut().unwrap().prerequisite
    else {
        unreachable!()
    };
    let (about_the_pick, rest): (Vec<_>, Vec<_>) = every.drain(..).partition(|leaf| {
        matches!(
            leaf,
            Prerequisite::NoOtherOptionOfGroup(_) | Prerequisite::PickedThrough(_)
        )
    });
    *every = rest;
    let evocation = game_data
        .classes
        .iter_mut()
        .flat_map(|class| &mut class.selections)
        .flat_map(|selection| &mut selection.options)
        .find(|option| option.key.as_str() == "sorcerer/school/evocation")
        .unwrap();
    evocation.prerequisite = Some(Prerequisite::All(about_the_pick));
    let game_data = game_data.canonical();

    seeds::replace_game_data(&test.path, &game_data, &"0".repeat(64))
        .await
        .unwrap();

    assert_eq!(test.load().await.unwrap(), game_data);
}

#[tokio::test]
async fn class_skills_under_an_archetype_that_keeps_the_base_list_are_an_error() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    // Eldritch Scoundrel keeps the rogue's class skills, so it can't list its own.
    sqlx::query(
        "INSERT INTO class_skill (class_key, archetype_key, skill_key)
        VALUES ('rogue', 'rogue/eldritch-scoundrel', 'athletics')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let error = test.load().await.unwrap_err().to_string();

    assert!(error.contains("class_skill"), "{error}");
}

#[tokio::test]
async fn a_prerequisite_leaf_with_children_is_an_error() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    sqlx::query(
        "INSERT INTO prerequisite_node (parent_id, kind, amount)
        SELECT id, 'base_attack_bonus', 1 FROM prerequisite_node WHERE kind = 'feat' LIMIT 1",
    )
    .execute(&pool)
    .await
    .unwrap();

    let error = test.load().await.unwrap_err().to_string();

    assert!(error.contains("children"), "{error}");
}

#[tokio::test]
async fn a_not_with_two_children_is_an_error() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    sqlx::query(
        "INSERT INTO prerequisite_node (parent_id, kind, amount)
        SELECT id, 'base_attack_bonus', 1 FROM prerequisite_node WHERE kind = 'not' LIMIT 1",
    )
    .execute(&pool)
    .await
    .unwrap();

    let error = test.load().await.unwrap_err().to_string();

    assert!(error.contains("a not has 2 children"), "{error}");
}

#[tokio::test]
async fn every_prerequisite_node_belongs_to_exactly_one_owner() {
    let test = TestDatabase::new();
    for (corruption, expected) in [
        // A tree no entity points at.
        (
            "INSERT INTO prerequisite_node (kind, amount) VALUES ('base_attack_bonus', 1)",
            "prerequisite_node",
        ),
        // One tree shared by a feat and a class.
        (
            "UPDATE class SET prerequisite_root_id =
                (SELECT prerequisite_root_id FROM feat WHERE key = 'power-attack')
            WHERE key = 'fighter'",
            "prerequisite_node",
        ),
        // An owner pointing into the middle of another tree.
        (
            "UPDATE class SET prerequisite_root_id =
                (SELECT id FROM prerequisite_node WHERE parent_id IS NOT NULL LIMIT 1)
            WHERE key = 'fighter'",
            "parent",
        ),
    ] {
        test.seed(&fixture_dir()).await.unwrap();
        let pool = test.writer().await;
        sqlx::query(corruption).execute(&pool).await.unwrap();

        let error = test.load().await.unwrap_err().to_string();

        assert!(error.contains(expected), "{corruption}: {error}");
    }
}

#[sqlx::test]
async fn a_prerequisite_node_must_fill_exactly_its_own_columns(pool: SqlitePool) {
    let insert = |sql: &'static str| sqlx::query(sql).execute(&pool);

    insert("INSERT INTO prerequisite_node (kind, amount) VALUES ('base_attack_bonus', 6)")
        .await
        .expect("a BAB condition with its amount is valid");
    insert("INSERT INTO prerequisite_node (kind) VALUES ('any')")
        .await
        .expect("a group with no values is valid");
    for invalid in [
        // A BAB condition without its amount.
        "INSERT INTO prerequisite_node (kind) VALUES ('base_attack_bonus')",
        // A skill-ranks condition without its skill.
        "INSERT INTO prerequisite_node (kind, amount) VALUES ('skill_ranks', 4)",
        // A Mythic rank condition without its rank.
        "INSERT INTO prerequisite_node (kind) VALUES ('mythic_rank')",
        // A class-level condition without its level.
        "INSERT INTO prerequisite_node (kind, class_key) VALUES ('class_level', 'fighter')",
        // An at-least group without its n.
        "INSERT INTO prerequisite_node (kind) VALUES ('at_least')",
        // A group carrying a condition's value.
        "INSERT INTO prerequisite_node (kind, ability) VALUES ('all', 'strength')",
        // A feat condition with a stray amount.
        "INSERT INTO prerequisite_node (kind, feat_key, amount) VALUES ('feat', 'power-attack', 1)",
        // An ability score condition with a stray spell type.
        "INSERT INTO prerequisite_node (kind, ability, spell_type, amount)
            VALUES ('ability_score', 'strength', 'arcane', 13)",
        // A feat for a value, without the value or with two.
        "INSERT INTO prerequisite_node (kind, feat_key) VALUES ('feat_for', 'weapon-focus')",
        "INSERT INTO prerequisite_node (kind, feat_key, weapon_key, energy)
            VALUES ('feat_for', 'weapon-focus', 'dagger', 'fire')",
        // A feat for a value with a stray amount.
        "INSERT INTO prerequisite_node (kind, feat_key, spell_school, amount)
            VALUES ('feat_for', 'spell-focus', 'evocation', 1)",
        // "The same value" names no value of its own.
        "INSERT INTO prerequisite_node (kind, feat_key, skill_key)
            VALUES ('feat_for_same_value', 'skill-focus', 'mobility')",
    ] {
        let error = insert(invalid)
            .await
            .unwrap_err()
            .to_string()
            .to_lowercase();
        assert!(error.contains("check constraint"), "{invalid}: {error}");
    }
}

#[tokio::test]
async fn an_ability_modifier_without_an_ability_is_the_chosen_kind() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    sqlx::query("INSERT INTO ability_modifier (race_key, amount) VALUES ('dwarf', 2)")
        .execute(&pool)
        .await
        .unwrap();

    let loaded = test.load().await.unwrap();

    let dwarf = loaded
        .races
        .iter()
        .find(|race| race.key.as_str() == "dwarf")
        .unwrap();
    assert!(
        dwarf
            .ability_modifiers
            .contains(&AbilityModifier::Chosen { amount: 2 }),
        "{:?}",
        dwarf.ability_modifiers
    );
}

#[tokio::test]
async fn the_schema_rejects_values_outside_their_domain_and_misplaced_keys() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;

    // A u8 above 255, and an ability that isn't one.
    let error = rejected(
        &pool,
        "UPDATE class SET skill_points = 256 WHERE key = 'fighter'",
    )
    .await;
    assert!(error.contains("u8"), "{error}");
    let error = rejected(
        &pool,
        "UPDATE skill SET ability = 'luck' WHERE key = 'mobility'",
    )
    .await;
    assert!(error.contains("ability"), "{error}");

    // A key is a lowercase slug, `/`-separated after an owner, with no type prefix.
    for key in ["skill:stealth", "Stealth", "stealth/", "two  words", ""] {
        let sql = format!(
            "INSERT INTO skill (key, name, ability) VALUES ('{key}', 'Stealth', 'dexterity')"
        );
        let error = rejected(&pool, sql.leak()).await;
        assert!(error.contains("text_key"), "{key:?}: {error}");
    }
    sqlx::query("INSERT INTO skill (key, name, ability) VALUES ('stealth-2/x', 'X', 'dexterity')")
        .execute(&pool)
        .await
        .expect("a slug with an owner is a valid key");

    // An option whose key is outside its selection's.
    let error = rejected(
        &pool,
        "INSERT INTO selection_option (key, selection_key, name, summary, guide, repeat)
        VALUES ('fighter/stray', 'rogue/rogue-talents', 'Stray', '', 'x', 'once')",
    )
    .await;
    assert!(error.contains("check constraint"), "{error}");

    // A level row for the fighter that names the rogue's archetype.
    let error = rejected(
        &pool,
        "INSERT INTO class_level (class_key, archetype_key, level)
        VALUES ('fighter', 'rogue/eldritch-scoundrel', 4)",
    )
    .await;
    assert!(error.contains("foreign key"), "{error}");

    // A class row has spell levels and advances flags; an archetype row has no advances flags, and
    // sets both spell levels or neither.
    for sql in [
        "INSERT INTO class_level (class_key, level, advances_arcane, advances_divine)
        VALUES ('fighter', 4, false, false)",
        "INSERT INTO class_level (class_key, level, arcane_spell_level, divine_spell_level)
        VALUES ('fighter', 4, 0, 0)",
        "INSERT INTO class_level (class_key, archetype_key, level, advances_arcane, advances_divine)
        VALUES ('fighter', 'fighter/armiger', 4, false, false)",
        "INSERT INTO class_level (class_key, archetype_key, level, arcane_spell_level)
        VALUES ('fighter', 'fighter/armiger', 4, 1)",
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains("check constraint"), "{sql}: {error}");
    }

    // Progressions and saves are closed sets.
    let error = rejected(
        &pool,
        "UPDATE class SET bab = 'quarter' WHERE key = 'fighter'",
    )
    .await;
    assert!(error.contains("bab_progression"), "{error}");
    let error = rejected(
        &pool,
        "UPDATE class SET will = 'great' WHERE key = 'fighter'",
    )
    .await;
    assert!(error.contains("save_progression"), "{error}");
    let error = rejected(
        &pool,
        "INSERT INTO save_exception (class_key, level, save, value) VALUES ('fighter', 2, 'luck', 1)",
    )
    .await;
    assert!(error.contains("check constraint"), "{error}");

    // A switch's previous paths, drops and granted features belong to a path's switch rule, so a path without
    // one can't have them.
    for sql in [
        "INSERT INTO path_switch_from (path_key, from_path_key) VALUES ('trickster', 'legend')",
        "INSERT INTO path_switch_drop (path_key, feature_key)
        VALUES ('trickster', 'trickster/trickster-tricks')",
        "INSERT INTO path_switch_feature (path_key, feature_key)
        VALUES ('trickster', 'trickster/trickster-tricks')",
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains("foreign key"), "{sql}: {error}");
    }
    // A switch isn't open from its own path, and a drop names exactly one feature or selection.
    for sql in [
        "INSERT INTO path_switch_from (path_key, from_path_key) VALUES ('legend', 'legend')",
        "INSERT INTO path_switch_drop (path_key) VALUES ('legend')",
        "INSERT INTO path_switch_drop (path_key, feature_key, selection_key)
        VALUES ('legend', 'trickster/trickster-tricks', 'trickster/tricks')",
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains("check constraint"), "{sql}: {error}");
    }

    // A switch's level cap runs 20-40.
    let error = rejected(
        &pool,
        "UPDATE path_switch SET level_cap = 41 WHERE path_key = 'legend'",
    )
    .await;
    assert!(error.contains("check constraint"), "{error}");

    // A path's rank row and a switch's kept ranks name a Mythic rank the ascension table has.
    let error = rejected(
        &pool,
        "INSERT INTO mythic_rank (path_key, rank) VALUES ('legend', 11)",
    )
    .await;
    assert!(error.contains("foreign key"), "{error}");
    for sql in [
        "UPDATE path_switch SET kept_through_rank = 11 WHERE path_key = 'legend'",
        "UPDATE path_switch SET path_kept_through_rank = 11 WHERE path_key = 'legend'",
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains("foreign key"), "{sql}: {error}");
    }

    // An ability bonus names an ability exactly when it's the fixed kind, and its kind is a closed set.
    for sql in [
        "INSERT INTO feature_ability_bonus (feature_key, kind, amount)
        VALUES ('legend/legend', 'fixed', 4)",
        "INSERT INTO feature_ability_bonus (feature_key, kind, ability, amount)
        VALUES ('legend/legend', 'highest', 'strength', 4)",
        "INSERT INTO feature_ability_bonus (feature_key, kind, amount)
        VALUES ('legend/legend', 'double', 4)",
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains("check constraint"), "{sql}: {error}");
    }
    // One bonus of a kind (and ability) per feature.
    let error = rejected(
        &pool,
        "INSERT INTO feature_ability_bonus (feature_key, kind, amount) VALUES ('legend/legend', 'every', 2)",
    )
    .await;
    assert!(error.contains("unique constraint"), "{error}");

    // An option's feats are feats.
    let error = rejected(
        &pool,
        "INSERT INTO selection_option_feat (option_key, feat_key)
        VALUES ('rogue/rogue-talents/combat-trick', 'no-such-feat')",
    )
    .await;
    assert!(error.contains("foreign key"), "{error}");

    // Only a fixed-list slot names a list, and it always does.
    let error = rejected(
        &pool,
        "UPDATE feat_slot SET feat_list_key = 'monk/bonus-feats' WHERE kind = 'combat_feat'",
    )
    .await;
    assert!(error.contains("check constraint"), "{error}");
    let error = rejected(
        &pool,
        "UPDATE feat_slot SET feat_list_key = NULL WHERE kind = 'one_of'",
    )
    .await;
    assert!(error.contains("check constraint"), "{error}");

    // Features, selections and feat lists are keyed under their owner: the archetype when there is one, else
    // the class or the path.
    for sql in [
        "INSERT INTO feature (key, class_key, name, summary, guide, daily_choice,
            skill_ranks_at_least_mythic_rank, favoured_weapon_proficiency)
        VALUES ('rogue/stray', 'fighter', 'Stray', '', 'x', false, false, false)",
        "INSERT INTO feature (key, class_key, archetype_key, name, summary, guide, daily_choice,
            skill_ranks_at_least_mythic_rank, favoured_weapon_proficiency)
        VALUES ('fighter/stray', 'fighter', 'fighter/armiger', 'Stray', '', 'x', false, false, false)",
        "INSERT INTO feature (key, path_key, name, summary, guide, daily_choice,
            skill_ranks_at_least_mythic_rank, favoured_weapon_proficiency)
        VALUES ('mythic/stray', 'trickster', 'Stray', '', 'x', false, false, false)",
        "INSERT INTO feature_selection (key, path_key, name, summary, guide, feat_lists_only,
            only_without_option, levels_behind, once_per_option)
        VALUES ('legend/stray', 'trickster', 'Stray', '', 'x', false, false, 0, false)",
        "INSERT INTO feat_list (key, class_key, name, guide, ignores_prerequisites, every_feat)
        VALUES ('fighter-list', 'fighter', 'Stray', 'x', false, false)",
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains("check constraint"), "{sql}: {error}");
    }
    sqlx::query(
        "INSERT INTO feature (key, class_key, archetype_key, name, summary, guide, daily_choice,
            skill_ranks_at_least_mythic_rank, favoured_weapon_proficiency)
        VALUES ('fighter/armiger/x', 'fighter', 'fighter/armiger', 'X', '', 'x', false, false, false)",
    )
    .execute(&pool)
    .await
    .expect("an archetype's feature under the archetype's key");

    // A grant is counted at least once, and each row grants a feature or a selection in one row.
    let error = rejected(&pool, "UPDATE granted_feature SET count = 0").await;
    assert!(error.contains("check constraint"), "{error}");
    let error = rejected(
        &pool,
        "INSERT INTO granted_feature (class_level_id, feature_key, count)
        SELECT class_level_id, feature_key, 1 FROM granted_feature WHERE class_level_id IS NOT NULL LIMIT 1",
    )
    .await;
    assert!(error.contains("unique"), "{error}");
    let error = rejected(
        &pool,
        "INSERT INTO granted_feat (class_level_id, feat_key)
        SELECT class_level_id, feat_key FROM granted_feat WHERE class_level_id IS NOT NULL LIMIT 1",
    )
    .await;
    assert!(
        error.contains("unique") || error.contains("duplicate"),
        "{error}"
    );

    // A selection shares another selection's existing options, each once, and never one keyed under itself.
    let error = rejected(
        &pool,
        "INSERT INTO selection_shared_option (selection_key, option_key)
        VALUES ('rogue/rogue-talents', 'rogue/rogue-talents/combat-trick')",
    )
    .await;
    assert!(error.contains("check constraint"), "{error}");
    let error = rejected(
        &pool,
        "INSERT INTO selection_shared_option (selection_key, option_key)
        VALUES ('rogue/eldritch-scoundrel/arcane-talents', 'rogue/rogue-talents/ghost')",
    )
    .await;
    assert!(error.contains("foreign key"), "{error}");
    let error = rejected(
        &pool,
        "INSERT INTO selection_shared_option (selection_key, option_key)
        VALUES ('rogue/eldritch-scoundrel/arcane-talents', 'rogue/rogue-talents/combat-trick')",
    )
    .await;
    assert!(
        error.contains("unique") || error.contains("duplicate"),
        "{error}"
    );

    // Mythic groups are the guide's headings.
    let error = rejected(
        &pool,
        "UPDATE mythic_ability SET group_name = 'armour_and_shield'",
    )
    .await;
    assert!(error.contains("mythic_ability_group"), "{error}");
    let error = rejected(&pool, "UPDATE mythic_feat SET group_name = 'healing'").await;
    assert!(error.contains("mythic_feat_group"), "{error}");
}

#[tokio::test]
async fn the_schema_keeps_repeats_and_parameters_consistent() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    for (sql, expected) in [
        // `times` carries its number, and only `times` does; a number is 2 or more.
        (
            "UPDATE feat SET repeat = 'times' WHERE key = 'toughness'",
            "check constraint",
        ),
        (
            "UPDATE feat SET repeat_times = 3 WHERE key = 'toughness'",
            "check constraint",
        ),
        (
            "UPDATE feat SET repeat_times = 1 WHERE key = 'extra-rage'",
            "check constraint",
        ),
        // Once per value needs a value.
        (
            "UPDATE feat SET repeat = 'per_value' WHERE key = 'toughness'",
            "check constraint",
        ),
        (
            "UPDATE feat SET parameter = 'colour' WHERE key = 'weapon-focus'",
            "parameter_kind",
        ),
        // An option is taken once per value only of its chosen feat (ticket 47).
        (
            "UPDATE selection_option SET repeat = 'per_value'
            WHERE key = 'rogue/rogue-talents/combat-trick'",
            "check constraint",
        ),
        // A chosen feat and a weapon in a prerequisite are real.
        (
            "UPDATE selection_option SET chosen_feat_key = 'no-such-feat'
            WHERE key = 'rogue/rogue-talents/combat-trick'",
            "foreign key",
        ),
        (
            "UPDATE prerequisite_node SET weapon_key = 'spork' WHERE weapon_key IS NOT NULL",
            "foreign key",
        ),
        (
            "UPDATE weapon SET proficiency = 'legendary' WHERE key = 'dagger'",
            "check constraint",
        ),
        (
            "UPDATE prerequisite_node SET spell_school = 'chronomancy'
            WHERE spell_school IS NOT NULL",
            "spell_school",
        ),
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains(expected), "{sql}: {error}");
    }
    // With a chosen feat it is (the Skill Focus talents, once per skill).
    let updated = sqlx::query(
        "UPDATE selection_option SET repeat = 'per_value' WHERE chosen_feat_key IS NOT NULL",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert!(updated.rows_affected() > 0);
}

#[tokio::test]
async fn the_schema_keeps_advancement_and_counts_as_consistent() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    for (sql, expected) in [
        (
            "UPDATE class SET advances = 'both' WHERE key = 'loremaster'",
            "check constraint",
        ),
        // A feature only with `feature`, and always with it.
        (
            "UPDATE class_counts_as SET feature_key = 'fighter/bravery'
            WHERE toward = 'feat_prerequisites'",
            "check constraint",
        ),
        (
            "UPDATE class_counts_as SET feature_key = NULL WHERE toward = 'feature'",
            "check constraint",
        ),
        (
            "UPDATE class_counts_as SET feature_key = 'no-such/feature' WHERE toward = 'feature'",
            "foreign key",
        ),
        (
            "UPDATE class_counts_as SET as_class_key = class_key",
            "check constraint",
        ),
        (
            "UPDATE class_counts_as SET as_class_key = 'no-such-class'",
            "foreign key",
        ),
        (
            "UPDATE prerequisite_node SET story = 'chosen_one' WHERE story IS NOT NULL",
            "check constraint",
        ),
        // A story node fills its own column only.
        (
            "UPDATE prerequisite_node SET amount = 1 WHERE kind = 'story'",
            "check constraint",
        ),
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains(expected), "{sql}: {error}");
    }
}

#[tokio::test]
async fn the_schema_keeps_option_levels_and_their_grants_consistent() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    for (sql, expected) in [
        // An option row's selection and level are set together.
        (
            "UPDATE feature SET option_row_level = NULL WHERE option_row_level IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE feature SET option_row_selection_key = 'no-such/selection'
            WHERE option_row_level IS NOT NULL",
            "foreign key",
        ),
        // One row per option level.
        (
            "UPDATE option_level SET level = 1 WHERE level = 3",
            "unique constraint",
        ),
        (
            "UPDATE option_level SET level = 41 WHERE level = 3",
            "check constraint",
        ),
        // A grant has exactly one owner, now one of four.
        (
            "UPDATE granted_feature SET class_level_id = (SELECT min(id) FROM class_level)
            WHERE option_level_id IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE granted_selection SET mythic_ability_key = 'mythic-charge'
            WHERE option_level_id IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE granted_feat SET class_level_id = NULL WHERE class_level_id IS NOT NULL",
            "check constraint",
        ),
        // Ticket 31: an option level owns feat slots (one owner each) and chosen feats.
        (
            "UPDATE feat_slot SET class_level_id = (SELECT min(id) FROM class_level)
            WHERE option_level_id IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE option_level_chosen_feat SET feat_key = 'no-such-feat'",
            "foreign key",
        ),
        (
            "UPDATE option_level_chosen_feat SET option_level_id = -1",
            "foreign key",
        ),
        // An option-list slot names a selection, and only it does.
        (
            "UPDATE feat_slot SET selection_key = NULL WHERE kind = 'option_list'",
            "check constraint",
        ),
        (
            "UPDATE feat_slot SET selection_key = 'sorcerer/bloodline' WHERE kind = 'general'",
            "check constraint",
        ),
        (
            "UPDATE feature_selection SET pool_of_key = key WHERE pool_of_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE selection_skipped_level SET level = 0",
            "check constraint",
        ),
        (
            "UPDATE selection_level_class SET class_key = 'no-such-class' WHERE class_key = 'fighter'",
            "foreign key",
        ),
        // The new prerequisite leaves fill their own columns only.
        (
            "UPDATE prerequisite_node SET selection_key = NULL WHERE kind = 'no_option_from'",
            "check constraint",
        ),
        (
            "UPDATE prerequisite_node SET selection_key = 'sorcerer/bloodline'
            WHERE kind = 'spontaneous_spell_level'",
            "check constraint",
        ),
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains(expected), "{sql}: {error}");
    }
}

#[tokio::test]
async fn the_schema_keeps_values_granted_offered_and_asked_about_consistent() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    for (sql, expected) in [
        // A feat granted for a value: one value column at most, a known value, on a class level only.
        (
            "UPDATE granted_feat SET skill_key = 'mobility' WHERE weapon_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE granted_feat SET weapon_key = 'no-such-weapon' WHERE weapon_key IS NOT NULL",
            "foreign key",
        ),
        (
            "UPDATE granted_feat SET energy = 'sonic' WHERE weapon_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE granted_feat SET armour_type = 'light' WHERE option_level_id IS NOT NULL",
            "check constraint",
        ),
        // An option's values: exactly one column each, known values; its own parameter never beside a chosen
        // feat.
        (
            "UPDATE selection_option_value SET weapon_key = NULL",
            "check constraint",
        ),
        (
            "UPDATE selection_option_value SET spell_school = 'evocation'",
            "check constraint",
        ),
        (
            "UPDATE selection_option_value SET option_key = 'no-such/option'",
            "foreign key",
        ),
        (
            "UPDATE selection_option SET parameter = 'weapon' WHERE chosen_feat_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE selection_option SET parameter = 'colour' WHERE parameter IS NOT NULL",
            "check constraint",
        ),
        // A mythic feat's repeat is as consistent as a feat's.
        (
            "UPDATE mythic_feat SET parameter = NULL WHERE repeat = 'per_value'",
            "check constraint",
        ),
        (
            "UPDATE mythic_feat SET repeat_times = NULL WHERE repeat = 'times'",
            "check constraint",
        ),
        // A race's feat list: one owner, keyed under it.
        (
            "UPDATE feat_list SET class_key = 'fighter' WHERE race_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE feat_list SET race_key = 'elf' WHERE race_key IS NOT NULL",
            "check constraint",
        ),
        // A feature's BAB floor is a game number.
        (
            "UPDATE feature SET bab_floor = 300 WHERE bab_floor IS NOT NULL",
            "check constraint",
        ),
        // `not` fills no column, `feat_if_value` a feat and one value.
        (
            "UPDATE prerequisite_node SET amount = 1 WHERE kind = 'not'",
            "check constraint",
        ),
        (
            "UPDATE prerequisite_node SET armour_type = NULL WHERE kind = 'feat_if_value'",
            "check constraint",
        ),
        (
            "UPDATE prerequisite_node SET weapon_key = 'dagger' WHERE kind = 'feat_if_value'",
            "check constraint",
        ),
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains(expected), "{sql}: {error}");
    }
}

#[tokio::test]
async fn the_schema_keeps_option_groups_arcanas_and_level_sources_consistent() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    for (sql, expected) in [
        // An option's group is a known one, its arcana a feature.
        (
            "UPDATE selection_option SET option_group = 'gnomish' WHERE option_group IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE selection_option SET arcana_key = 'no-such/feature' WHERE arcana_key IS NOT NULL",
            "foreign key",
        ),
        // A selection's level source is a known one; it lags by a game number of levels.
        (
            "UPDATE feature_selection SET level_from = 'elsewhere' WHERE level_from IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE feature_selection SET levels_behind = -1 WHERE levels_behind > 0",
            "check constraint",
        ),
        // The two new leaves fill their own columns only.
        (
            "UPDATE prerequisite_node SET option_group = NULL WHERE kind = 'no_other_option_of_group'",
            "check constraint",
        ),
        (
            "UPDATE prerequisite_node SET selection_key = 'sorcerer/school'
            WHERE kind = 'no_other_option_of_group'",
            "check constraint",
        ),
        (
            "UPDATE prerequisite_node SET selection_key = NULL WHERE kind = 'picked_through'",
            "check constraint",
        ),
        (
            "UPDATE prerequisite_node SET option_group = 'draconic' WHERE kind = 'picked_through'",
            "check constraint",
        ),
        (
            "UPDATE prerequisite_node SET selection_key = 'no-such/selection' WHERE kind = 'picked_through'",
            "foreign key",
        ),
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains(expected), "{sql}: {error}");
    }
}

#[tokio::test]
async fn the_schema_keeps_subrace_slots_option_list_stages_and_levels_alongside_consistent() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    for (sql, expected) in [
        // A subrace owns a slot as one owner; it is a subrace.
        (
            "UPDATE feat_slot SET race_key = 'half-elf' WHERE subrace_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE feat_slot SET subrace_key = 'no-such/subrace' WHERE subrace_key IS NOT NULL",
            "foreign key",
        ),
        // Only an option list slot names the option level of its list, a game number.
        (
            "UPDATE feat_slot SET at_level = 1 WHERE kind <> 'option_list'",
            "check constraint",
        ),
        (
            "UPDATE feat_slot SET at_level = -1 WHERE at_level IS NOT NULL",
            "check constraint",
        ),
        // Only an option list slot has its own label (ticket 47), and a label says something.
        (
            "UPDATE feat_slot SET label = 'Bonus feat' WHERE kind <> 'option_list'",
            "check constraint",
        ),
        (
            "UPDATE feat_slot SET label = '' WHERE kind = 'option_list'",
            "check constraint",
        ),
        // An option counted beside the others is an option, on a selection levelled by the other options.
        (
            "UPDATE feature_selection SET level_also_from_key = 'no-such/option'
            WHERE level_also_from_key IS NOT NULL",
            "foreign key",
        ),
        (
            "UPDATE feature_selection SET level_from = 'granting_option'
            WHERE level_also_from_key IS NOT NULL",
            "check constraint",
        ),
        // Only a path's selection is levelled by Mythic rank (ticket 89: the Azata's superpowers).
        (
            "UPDATE feature_selection SET level_from = 'mythic_ranks' WHERE class_key IS NOT NULL",
            "check constraint",
        ),
        // The bloodrager's groups are known; another isn't.
        (
            "UPDATE selection_option SET option_group = 'bloodrager_gnomish' WHERE option_group IS NOT NULL",
            "check constraint",
        ),
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains(expected), "{sql}: {error}");
    }
}

#[tokio::test]
async fn the_schema_keeps_backgrounds_and_their_skills_and_feats_consistent() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    for (sql, expected) in [
        // The groups are the game's; None has none.
        (
            "UPDATE background SET background_group = 'pirate' WHERE key = 'shackles-corsair'",
            "check constraint",
        ),
        (
            "UPDATE background SET key = 'Shackles Corsair' WHERE key = 'shackles-corsair'",
            "check constraint",
        ),
        // A class skill belongs to a class (or its archetype) or to a background, never both or neither.
        (
            "UPDATE class_skill SET class_key = 'fighter' WHERE background_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE class_skill SET background_key = NULL WHERE background_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE class_skill SET archetype_key = 'fighter/armiger' WHERE background_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE class_skill SET background_key = 'no-such-background'
            WHERE background_key IS NOT NULL AND skill_key = 'athletics'",
            "foreign key",
        ),
        // A granted feat belongs to one owner; a background may grant one for a value, as a class level does.
        (
            "UPDATE granted_feat SET class_level_id = (SELECT min(id) FROM class_level)
            WHERE background_key IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE granted_feat SET background_key = 'no-such-background' WHERE background_key IS NOT NULL",
            "foreign key",
        ),
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains(expected), "{sql}: {error}");
    }
}

#[tokio::test]
async fn the_schema_keeps_option_levels_class_skills_consistent() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    for (sql, expected) in [
        // An option level's class skill has that one owner: not a background or a class besides, not none.
        (
            "UPDATE class_skill SET background_key = 'shackles-corsair' WHERE option_level_id IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE class_skill SET class_key = 'sorcerer' WHERE option_level_id IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE class_skill SET option_level_id = NULL WHERE option_level_id IS NOT NULL",
            "check constraint",
        ),
        (
            "UPDATE class_skill SET option_level_id = -1
            WHERE option_level_id IS NOT NULL AND skill_key = 'athletics'",
            "foreign key",
        ),
        // Each skill once per option level.
        (
            "INSERT INTO class_skill (option_level_id, skill_key)
            SELECT option_level_id, skill_key FROM class_skill WHERE option_level_id IS NOT NULL",
            "unique constraint",
        ),
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains(expected), "{sql}: {error}");
    }
}

/// Ticket 86: a deity's rows name a known alignment, weapon and domain; a deity node names its deity; a
/// selection's limit and an option's domain are what the types allow.
#[tokio::test]
async fn the_schema_keeps_deities_and_their_limits_consistent() {
    let test = TestDatabase::new();
    test.seed(&fixture_dir()).await.unwrap();
    let pool = test.writer().await;
    for (sql, expected) in [
        (
            "UPDATE deity_alignment SET alignment = 'lawful' WHERE deity_key = 'iomedae'",
            "check constraint",
        ),
        (
            "UPDATE deity SET key = 'Iomedae' WHERE key = 'iomedae'",
            "check constraint",
        ),
        (
            "UPDATE deity_domain SET domain_key = 'cleric/domain/no-such-domain' WHERE deity_key = 'achaekek'",
            "foreign key",
        ),
        (
            "UPDATE deity_weapon SET weapon_key = 'no-such-weapon' WHERE deity_key = 'achaekek'",
            "foreign key",
        ),
        (
            "UPDATE deity_alignment SET deity_key = 'no-such-deity' WHERE deity_key = 'achaekek'",
            "foreign key",
        ),
        // Each alignment, weapon and domain once per deity.
        (
            "INSERT INTO deity_domain (deity_key, domain_key) VALUES ('achaekek', 'cleric/domain/law')",
            "unique constraint",
        ),
        // A `deity` node fills `deity_key` and nothing else; `worships` and `atheist` fill nothing.
        (
            "INSERT INTO prerequisite_node (kind) VALUES ('deity')",
            "check constraint",
        ),
        (
            "INSERT INTO prerequisite_node (kind, deity_key) VALUES ('worships', 'iomedae')",
            "check constraint",
        ),
        (
            "INSERT INTO prerequisite_node (kind, selection_option_key) VALUES ('deity_offers', NULL)",
            "check constraint",
        ),
        (
            "INSERT INTO prerequisite_node (kind, deity_key) VALUES ('deity', 'no-such-deity')",
            "foreign key",
        ),
        (
            "UPDATE feature_selection SET deity_domains = 'some' WHERE key = 'cleric/domain'",
            "check constraint",
        ),
        (
            "UPDATE selection_option SET domain_key = 'cleric/domain/no-such-domain'
            WHERE key = 'cleric/blessings/good'",
            "foreign key",
        ),
        (
            "UPDATE feature SET favoured_weapon_proficiency = 2 WHERE key = 'cleric/cleric-proficiencies'",
            "check constraint",
        ),
    ] {
        let error = rejected(&pool, sql).await;
        assert!(error.contains(expected), "{sql}: {error}");
    }
}

#[tokio::test]
async fn every_background_group_round_trips() {
    let test = TestDatabase::new();
    let mut game_data = seeds::read_game_data_files(&fixture_dir()).unwrap();
    for group in rules::BackgroundGroup::ALL {
        let key = group.name().to_lowercase().replace(' ', "-");
        game_data.backgrounds.push(rules::Background {
            key: rules::BackgroundKey::new(format!("every-{key}")),
            name: group.name().into(),
            group: Some(group),
            note: String::new(),
            class_skills: [SkillKey::new("athletics")].into(),
            feats: [FeatKey::new("toughness")].into(),
            feats_for: [rules::FeatValue {
                feat: FeatKey::new("skill-focus"),
                value: Parameter::Skill(SkillKey::new("mobility")),
            }]
            .into(),
        });
    }
    let game_data = game_data.canonical();

    seeds::replace_game_data(&test.path, &game_data, &"0".repeat(64))
        .await
        .unwrap();

    assert_eq!(test.load().await.unwrap(), game_data);
}
