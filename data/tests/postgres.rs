//! Seeding Postgres from the `game-data/` files and reading it back.
//!
//! Each `#[sqlx::test]` gets its own throwaway database (created from `DATABASE_URL`, migrated from
//! `data/migrations`, the default), so these tests never touch the dev database.

mod common;

use std::fs;

use common::{copy_fixture, fixture_dir};
use data::{Database, GameDataStatus};
use rules::{
    Ability, AbilityModifier, Alignment, ArchetypeKey, ClassKey, DataAt, Feat, FeatCategory,
    FeatKey, FeatureKey, GuideRef, MythicAbilityKey, MythicFeatKey, Prerequisite, RaceKey,
    SelectionOptionKey, SkillKey, SpellType,
};
use sqlx::PgPool;

#[sqlx::test]
async fn files_to_postgres_to_game_data_equals_files_to_game_data(pool: PgPool) {
    let db = Database::from(pool);
    // Neither the files nor the tables carry an order, so both sides come back canonical.
    let from_files = data::read_game_data_files(&fixture_dir()).unwrap();

    data::seed(&db, &fixture_dir()).await.unwrap();
    let from_postgres = data::load_game_data(&db).await.unwrap();

    assert_eq!(from_postgres, from_files.clone().canonical());
    assert_eq!(from_postgres, from_files);
}

/// The fixture has no save exceptions, so add one, and the archetype rows that set spell levels and the ones that
/// don't both come back.
#[sqlx::test]
async fn save_exceptions_and_archetype_rows_round_trip(pool: PgPool) {
    let db = Database::from(pool);
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
    let from_files = data::read_game_data_files(dir.path()).unwrap();
    let fighter = from_files
        .classes
        .iter()
        .find(|c| c.key.as_str() == "fighter");
    assert_eq!(fighter.unwrap().save_exceptions.len(), 2);

    data::seed(&db, dir.path()).await.unwrap();

    assert_eq!(data::load_game_data(&db).await.unwrap(), from_files);
}

/// A feature or selection granted twice by one row is one `granted_*` row with a count of 2, and reads back as
/// two grants.
#[sqlx::test]
async fn repeated_grants_are_stored_as_counts_and_read_back_in_full(pool: PgPool) {
    let db = Database::from(pool.clone());
    let dir = copy_fixture();
    let rogue = dir.path().join("classes/rogue.ron");
    let text = fs::read_to_string(&rogue).unwrap();
    let text = text
        .replacen(
            r#"(level: 1, features: ["rogue/sneak-attack"]),"#,
            r#"(level: 1, features: ["rogue/sneak-attack", "rogue/sneak-attack"]),"#,
            1,
        )
        .replacen(
            r#"(level: 2, selections: ["rogue/rogue-talents"]),"#,
            r#"(level: 2, selections: ["rogue/rogue-talents", "rogue/rogue-talents", "rogue/rogue-talents"]),"#,
            1,
        );
    fs::write(&rogue, text).unwrap();
    let from_files = data::read_game_data_files(dir.path()).unwrap();

    data::seed(&db, dir.path()).await.unwrap();

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
    assert_eq!(data::load_game_data(&db).await.unwrap(), from_files);
}

#[sqlx::test]
async fn seeding_again_replaces_the_previous_data(pool: PgPool) {
    let db = Database::from(pool);
    data::seed(&db, &fixture_dir()).await.unwrap();
    let edited = copy_fixture();
    fs::remove_dir_all(edited.path().join("prestige")).unwrap();
    let skills = edited.path().join("skills.ron");
    let text = fs::read_to_string(&skills).unwrap();
    fs::write(
        &skills,
        text.replace("\"Athletics\"", "\"Athletics (edited)\""),
    )
    .unwrap();

    data::seed(&db, edited.path()).await.unwrap();
    let reloaded = data::load_game_data(&db).await.unwrap();

    assert!(reloaded.prestige_classes.is_empty());
    assert_eq!(reloaded.skills[0].name, "Athletics (edited)");
    assert_eq!(reloaded, data::read_game_data_files(edited.path()).unwrap());
}

#[sqlx::test]
async fn a_seed_that_fails_leaves_the_previous_data_in_place(pool: PgPool) {
    let db = Database::from(pool);
    data::seed(&db, &fixture_dir()).await.unwrap();
    let broken = copy_fixture();
    let feats = broken.path().join("feats.ron");
    let text = fs::read_to_string(&feats).unwrap();
    fs::write(
        &feats,
        text.replace("Feat(\"power-attack\")", "Feat(\"no-such-feat\")"),
    )
    .unwrap();

    let error = data::seed(&db, broken.path())
        .await
        .unwrap_err()
        .to_string();

    assert!(error.contains("foreign key"), "{error}");
    assert_eq!(
        data::load_game_data(&db).await.unwrap(),
        data::read_game_data_files(&fixture_dir()).unwrap()
    );
    assert_eq!(
        data::game_data_status(&db, &fixture_dir()).await.unwrap(),
        GameDataStatus::Current
    );
}

#[sqlx::test]
async fn the_seed_refuses_data_that_fails_the_data_check(pool: PgPool) {
    let db = Database::from(pool);
    let broken = copy_fixture();
    let fighter = broken.path().join("classes/fighter.ron");
    let text = fs::read_to_string(&fighter).unwrap();
    // The fighter's own level 2 row goes missing, so the Armiger's row there is outside its class too.
    let row = text
        .lines()
        .find(|line| line.starts_with("        (level: 2,"))
        .unwrap();
    fs::write(&fighter, text.replacen(&format!("{row}\n"), "", 1)).unwrap();

    let error = data::seed(&db, broken.path()).await.unwrap_err();

    let data::Error::Invalid(problems) = &error else {
        panic!("{error}");
    };
    assert_eq!(
        problems.iter().map(|p| &p.at).collect::<Vec<_>>(),
        [
            &DataAt::Class(ClassKey::new("fighter")),
            &DataAt::Archetype(ArchetypeKey::new("fighter/armiger"))
        ]
    );
    assert!(error.to_string().contains("class fighter"), "{error}");
    assert_eq!(
        data::game_data_status(&db, &fixture_dir()).await.unwrap(),
        GameDataStatus::NeverSeeded
    );
}

#[sqlx::test]
async fn status_is_never_seeded_before_the_first_seed(pool: PgPool) {
    let db = Database::from(pool);
    assert_eq!(
        data::game_data_status(&db, &fixture_dir()).await.unwrap(),
        GameDataStatus::NeverSeeded
    );
}

#[sqlx::test]
async fn status_is_stale_once_a_data_file_changes_after_seeding(pool: PgPool) {
    let db = Database::from(pool);
    let dir = copy_fixture();
    let seeded = data::seed(&db, dir.path()).await.unwrap();
    assert_eq!(
        data::game_data_status(&db, dir.path()).await.unwrap(),
        GameDataStatus::Current
    );

    let races = dir.path().join("races.ron");
    let text = fs::read_to_string(&races).unwrap();
    fs::write(&races, text.replace("\"Dwarf\"", "\"Mountain Dwarf\"")).unwrap();

    assert_eq!(
        data::game_data_status(&db, dir.path()).await.unwrap(),
        GameDataStatus::Stale {
            seeded,
            files: data::game_data_hash(dir.path()).unwrap(),
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
    ];
    let groups = vec![
        P::Any(vec![P::BaseAttackBonus(1), P::CasterLevel(1)]),
        P::AtLeast(1, vec![P::CharacterLevel(2), P::CasterLevel(2)]),
    ];
    let all = [leaves, groups].concat();
    for prerequisite in &all {
        match prerequisite {
            P::All(_)
            | P::Any(_)
            | P::AtLeast(..)
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
            | P::Race(_) => {}
        }
    }
    all
}

#[sqlx::test]
async fn every_prerequisite_variant_round_trips(pool: PgPool) {
    let db = Database::from(pool);
    let mut game_data = data::read_game_data_files(&fixture_dir()).unwrap();
    game_data.feats.push(Feat {
        key: FeatKey::new("everything"),
        name: "Everything".into(),
        category: FeatCategory::Miscellaneous,
        summary: String::new(),
        guide: GuideRef::new("feats#everything"),
        prerequisite: Some(Prerequisite::All(every_prerequisite_variant())),
    });
    let game_data = game_data.canonical();

    data::replace_game_data(&db, &game_data, &"0".repeat(64))
        .await
        .unwrap();

    assert_eq!(data::load_game_data(&db).await.unwrap(), game_data);
}

#[sqlx::test]
async fn class_skills_under_an_archetype_that_keeps_the_base_list_are_an_error(pool: PgPool) {
    let db = Database::from(pool.clone());
    data::seed(&db, &fixture_dir()).await.unwrap();
    // Eldritch Scoundrel keeps the rogue's class skills, so it can't list its own.
    sqlx::query(
        "INSERT INTO class_skill (class_key, archetype_key, skill_key)
        VALUES ('rogue', 'rogue/eldritch-scoundrel', 'athletics')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let error = data::load_game_data(&db).await.unwrap_err().to_string();

    assert!(error.contains("class_skill"), "{error}");
}

#[sqlx::test]
async fn a_prerequisite_leaf_with_children_is_an_error(pool: PgPool) {
    let db = Database::from(pool.clone());
    data::seed(&db, &fixture_dir()).await.unwrap();
    sqlx::query(
        "INSERT INTO prerequisite_node (parent_id, kind, amount)
        SELECT id, 'base_attack_bonus', 1 FROM prerequisite_node WHERE kind = 'feat' LIMIT 1",
    )
    .execute(&pool)
    .await
    .unwrap();

    let error = data::load_game_data(&db).await.unwrap_err().to_string();

    assert!(error.contains("children"), "{error}");
}

#[sqlx::test]
async fn every_prerequisite_node_belongs_to_exactly_one_owner(pool: PgPool) {
    let db = Database::from(pool.clone());
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
        data::seed(&db, &fixture_dir()).await.unwrap();
        sqlx::query(corruption).execute(&pool).await.unwrap();

        let error = data::load_game_data(&db).await.unwrap_err().to_string();

        assert!(error.contains(expected), "{corruption}: {error}");
    }
}

#[sqlx::test]
async fn a_prerequisite_node_must_fill_exactly_its_own_columns(pool: PgPool) {
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
    ] {
        let error = insert(invalid).await.unwrap_err().to_string();
        assert!(error.contains("check constraint"), "{invalid}: {error}");
    }
}

#[sqlx::test]
async fn an_ability_modifier_without_an_ability_is_the_chosen_kind(pool: PgPool) {
    let db = Database::from(pool.clone());
    data::seed(&db, &fixture_dir()).await.unwrap();
    sqlx::query("INSERT INTO ability_modifier (race_key, amount) VALUES ('dwarf', 2)")
        .execute(&pool)
        .await
        .unwrap();

    let loaded = data::load_game_data(&db).await.unwrap();

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

#[sqlx::test]
async fn the_schema_rejects_values_outside_their_domain_and_misplaced_keys(pool: PgPool) {
    let db = Database::from(pool.clone());
    data::seed(&db, &fixture_dir()).await.unwrap();
    let rejected = |sql: &'static str| {
        let pool = pool.clone();
        async move {
            let mut tx = pool.begin().await.unwrap();
            let insert = sqlx::query(sql).execute(&mut *tx).await;
            // Foreign keys are deferred: they fail at commit.
            let result = match insert {
                Ok(_) => tx.commit().await.map(|_| ()),
                Err(error) => Err(error),
            };
            result.unwrap_err().to_string()
        }
    };

    // A u8 above 255, and an ability that isn't one.
    let error = rejected("UPDATE class SET skill_points = 256 WHERE key = 'fighter'").await;
    assert!(error.contains("u8"), "{error}");
    let error = rejected("UPDATE skill SET ability = 'luck' WHERE key = 'mobility'").await;
    assert!(error.contains("ability"), "{error}");

    // A key is a lowercase slug, `/`-separated after an owner, with no type prefix.
    for key in ["skill:stealth", "Stealth", "stealth/", "two  words", ""] {
        let sql = format!(
            "INSERT INTO skill (key, name, ability) VALUES ('{key}', 'Stealth', 'dexterity')"
        );
        let error = rejected(sql.leak()).await;
        assert!(error.contains("text_key"), "{key:?}: {error}");
    }
    sqlx::query("INSERT INTO skill (key, name, ability) VALUES ('stealth-2/x', 'X', 'dexterity')")
        .execute(&pool)
        .await
        .expect("a slug with an owner is a valid key");

    // An option whose key is outside its selection's.
    let error = rejected(
        "INSERT INTO selection_option (key, selection_key, name, summary, guide)
        VALUES ('fighter/stray', 'rogue/rogue-talents', 'Stray', '', 'x')",
    )
    .await;
    assert!(error.contains("check constraint"), "{error}");

    // A level row for the fighter that names the rogue's archetype.
    let error = rejected(
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
        let error = rejected(sql).await;
        assert!(error.contains("check constraint"), "{sql}: {error}");
    }

    // Progressions and saves are closed sets.
    let error = rejected("UPDATE class SET bab = 'quarter' WHERE key = 'fighter'").await;
    assert!(error.contains("bab_progression"), "{error}");
    let error = rejected("UPDATE class SET will = 'great' WHERE key = 'fighter'").await;
    assert!(error.contains("save_progression"), "{error}");
    let error = rejected(
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
        let error = rejected(sql).await;
        assert!(error.contains("foreign key"), "{sql}: {error}");
    }
    // A switch isn't open from its own path, and a drop names exactly one feature or selection.
    for sql in [
        "INSERT INTO path_switch_from (path_key, from_path_key) VALUES ('legend', 'legend')",
        "INSERT INTO path_switch_drop (path_key) VALUES ('legend')",
        "INSERT INTO path_switch_drop (path_key, feature_key, selection_key)
        VALUES ('legend', 'trickster/trickster-tricks', 'trickster/tricks')",
    ] {
        let error = rejected(sql).await;
        assert!(error.contains("check constraint"), "{sql}: {error}");
    }

    // A switch's level cap runs 20-40.
    let error = rejected("UPDATE path_switch SET level_cap = 41 WHERE path_key = 'legend'").await;
    assert!(error.contains("check constraint"), "{error}");

    // A path's rank row and a switch's kept ranks name a Mythic rank the ascension table has.
    let error = rejected("INSERT INTO mythic_rank (path_key, rank) VALUES ('legend', 11)").await;
    assert!(error.contains("foreign key"), "{error}");
    for sql in [
        "UPDATE path_switch SET kept_through_rank = 11 WHERE path_key = 'legend'",
        "UPDATE path_switch SET path_kept_through_rank = 11 WHERE path_key = 'legend'",
    ] {
        let error = rejected(sql).await;
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
        let error = rejected(sql).await;
        assert!(error.contains("check constraint"), "{sql}: {error}");
    }
    // One bonus of a kind (and ability) per feature.
    let error = rejected(
        "INSERT INTO feature_ability_bonus (feature_key, kind, amount) VALUES ('legend/legend', 'every', 2)",
    )
    .await;
    assert!(error.contains("unique constraint"), "{error}");

    // An option's feats are feats.
    let error = rejected(
        "INSERT INTO selection_option_feat (option_key, feat_key)
        VALUES ('rogue/rogue-talents/combat-trick', 'no-such-feat')",
    )
    .await;
    assert!(error.contains("foreign key"), "{error}");

    // Only a fixed-list slot names a list, and it always does.
    let error = rejected(
        "UPDATE feat_slot SET feat_list_key = 'monk/bonus-feats' WHERE kind = 'combat_feat'",
    )
    .await;
    assert!(error.contains("check constraint"), "{error}");
    let error = rejected("UPDATE feat_slot SET feat_list_key = NULL WHERE kind = 'one_of'").await;
    assert!(error.contains("check constraint"), "{error}");

    // Features, selections and feat lists are keyed under their owner: the archetype when there is one, else
    // the class or the path.
    for sql in [
        "INSERT INTO feature (key, class_key, name, summary, guide)
        VALUES ('rogue/stray', 'fighter', 'Stray', '', 'x')",
        "INSERT INTO feature (key, class_key, archetype_key, name, summary, guide)
        VALUES ('fighter/stray', 'fighter', 'fighter/armiger', 'Stray', '', 'x')",
        "INSERT INTO feature (key, path_key, name, summary, guide)
        VALUES ('mythic/stray', 'trickster', 'Stray', '', 'x')",
        "INSERT INTO feature_selection (key, path_key, name, summary, guide)
        VALUES ('legend/stray', 'trickster', 'Stray', '', 'x')",
        "INSERT INTO feat_list (key, class_key, name, guide, ignores_prerequisites, every_feat)
        VALUES ('fighter-list', 'fighter', 'Stray', 'x', false, false)",
    ] {
        let error = rejected(sql).await;
        assert!(error.contains("check constraint"), "{sql}: {error}");
    }
    sqlx::query(
        "INSERT INTO feature (key, class_key, archetype_key, name, summary, guide)
        VALUES ('fighter/armiger/x', 'fighter', 'fighter/armiger', 'X', '', 'x')",
    )
    .execute(&pool)
    .await
    .expect("an archetype's feature under the archetype's key");

    // A grant is counted at least once, and each row grants a feature or a selection in one row.
    let error = rejected("UPDATE granted_feature SET count = 0").await;
    assert!(error.contains("check constraint"), "{error}");
    let error = rejected(
        "INSERT INTO granted_feature (class_level_id, feature_key, count)
        SELECT class_level_id, feature_key, 1 FROM granted_feature WHERE class_level_id IS NOT NULL LIMIT 1",
    )
    .await;
    assert!(error.contains("unique"), "{error}");
    let error = rejected(
        "INSERT INTO granted_feat (class_level_id, feat_key)
        SELECT class_level_id, feat_key FROM granted_feat LIMIT 1",
    )
    .await;
    assert!(
        error.contains("unique") || error.contains("duplicate"),
        "{error}"
    );

    // A selection shares another selection's existing options, each once, and never one keyed under itself.
    let error = rejected(
        "INSERT INTO selection_shared_option (selection_key, option_key)
        VALUES ('rogue/rogue-talents', 'rogue/rogue-talents/combat-trick')",
    )
    .await;
    assert!(error.contains("check constraint"), "{error}");
    let error = rejected(
        "INSERT INTO selection_shared_option (selection_key, option_key)
        VALUES ('rogue/eldritch-scoundrel/arcane-talents', 'rogue/rogue-talents/ghost')",
    )
    .await;
    assert!(error.contains("foreign key"), "{error}");
    let error = rejected(
        "INSERT INTO selection_shared_option (selection_key, option_key)
        VALUES ('rogue/eldritch-scoundrel/arcane-talents', 'rogue/rogue-talents/combat-trick')",
    )
    .await;
    assert!(
        error.contains("unique") || error.contains("duplicate"),
        "{error}"
    );

    // Mythic groups are the guide's headings.
    let error = rejected("UPDATE mythic_ability SET group_name = 'armour_and_shield'").await;
    assert!(error.contains("mythic_ability_group"), "{error}");
    let error = rejected("UPDATE mythic_feat SET group_name = 'healing'").await;
    assert!(error.contains("mythic_feat_group"), "{error}");
}
