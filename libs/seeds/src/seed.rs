//! Replacing the game data database with a new file holding a [`GameData`].

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use rules::{
    AbilityBonus, AbilityModifier, Archetype, ArchetypeLevelRow, Background, Class, ClassLevelRow,
    CountsToward, Deity, FeatKey, FeatList, FeatListKey, FeatSlot, FeatValue, Feature, FeatureKey,
    FeatureSelection, FeatureSelectionKey, GameData, MythicPath, Race, SelectionOptionKey,
    SkillKey,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{AssertSqlSafe, Row, SqliteConnection};

use crate::columns::{
    AbilityBonusKind, FeatSlotKind, ValueColumns, ability_bonus_kind_text, ability_text,
    advancement_text, alignment_text, bab_progression_text, background_group_text,
    deity_domains_text, feat_category_text, feat_slot_kind_text, level_from_text,
    mythic_ability_group_text, mythic_extra_pick_text, mythic_feat_group_text, option_group_text,
    parameter_kind_text, repeat_columns, save_progression_text, save_text, weapon_proficiency_text,
};
use crate::{Error, MIGRATOR, prerequisites};

/// Checks `data` ([`GameData::check`]), then builds a new database next to `database` (its name plus `.new`):
/// runs the migration, loads `data` and records `hash` as the data the tables came from, in one transaction
/// that commits only when no foreign key fails (`PRAGMA foreign_key_check`, every failure reported), and
/// renames the new file over `database`. Either all of it happens or none of it does: data with problems
/// writes nothing, and the old database stays as it was. A running server keeps reading the file it opened.
pub async fn replace_game_data(database: &Path, data: &GameData, hash: &str) -> Result<(), Error> {
    let problems = data.check();
    if !problems.is_empty() {
        return Err(Error::Invalid(problems));
    }
    let mut name = database.as_os_str().to_owned();
    name.push(".new");
    let new = PathBuf::from(name);
    // A seed that was killed may have left one behind.
    remove_database_file(&new)?;
    let replaced = match build(&new, data, hash).await {
        Ok(()) => std::fs::rename(&new, database).map_err(|error| sqlx::Error::Io(error).into()),
        Err(error) => Err(error),
    };
    if replaced.is_err() {
        // Best effort: the error to report is the seed's.
        let _ = remove_database_file(&new);
    }
    replaced
}

/// Removes a database file and the journal SQLite may have left beside it, if they exist.
fn remove_database_file(path: &Path) -> Result<(), sqlx::Error> {
    for suffix in ["", "-journal", "-wal", "-shm"] {
        let mut name = path.as_os_str().to_owned();
        name.push(suffix);
        match std::fs::remove_file(PathBuf::from(name)) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                return Err(sqlx::Error::Io(error));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Creates the database at `path`, migrates it and loads `data` into it.
async fn build(path: &Path, data: &GameData, hash: &str) -> Result<(), Error> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    let built = async {
        MIGRATOR.run(&pool).await.map_err(sqlx::Error::from)?;
        let mut tx = pool.begin().await?;
        insert_all(&mut tx, data, hash).await?;
        // The foreign keys are deferred, so a failing one would fail the commit with no more than
        // "FOREIGN KEY constraint failed". Name every one instead, and roll back.
        let missing = missing_references(&mut tx).await?;
        if !missing.is_empty() {
            return Err(Error::MissingReferences(missing));
        }
        tx.commit().await?;
        Ok(())
    }
    .await;
    pool.close().await;
    built
}

/// A row whose foreign key names a key its parent table doesn't have.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingReference {
    pub table: String,
    pub rowid: i64,
    /// The foreign key's columns and their values, as SQL literals (`'power-attack'`, `12`).
    pub columns: Vec<(String, String)>,
    /// The table and columns it refers to.
    pub parent: String,
    pub parent_columns: Vec<String>,
}

impl fmt::Display for MissingReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list = |items: Vec<&str>| match items.as_slice() {
            [one] => (*one).to_owned(),
            many => format!("({})", many.join(", ")),
        };
        write!(
            f,
            "{} row {}: {} {} is not in {} {}",
            self.table,
            self.rowid,
            list(self.columns.iter().map(|(name, _)| name.as_str()).collect()),
            list(
                self.columns
                    .iter()
                    .map(|(_, value)| value.as_str())
                    .collect()
            ),
            self.parent,
            list(self.parent_columns.iter().map(String::as_str).collect()),
        )
    }
}

/// Every foreign key that fails (`PRAGMA foreign_key_check`), with the values it names.
async fn missing_references(
    conn: &mut SqliteConnection,
) -> Result<Vec<MissingReference>, sqlx::Error> {
    // (table, rowid, parent, the foreign key's id in `pragma_foreign_key_list(table)`).
    let failures: Vec<(String, i64, String, i64)> =
        sqlx::query_as("SELECT \"table\", rowid, parent, fkid FROM pragma_foreign_key_check")
            .fetch_all(&mut *conn)
            .await?;
    let mut missing = Vec::with_capacity(failures.len());
    for (table, rowid, parent, fkid) in failures {
        let pairs: Vec<(String, String)> = sqlx::query_as(
            "SELECT \"from\", \"to\" FROM pragma_foreign_key_list(?1) WHERE id = ?2 ORDER BY seq",
        )
        .bind(&table)
        .bind(fkid)
        .fetch_all(&mut *conn)
        .await?;
        // The names come from SQLite's own catalogue, quoted as identifiers.
        let quoted = |name: &str| format!("\"{}\"", name.replace('"', "\"\""));
        let select = format!(
            "SELECT {} FROM {} WHERE rowid = ?1",
            pairs
                .iter()
                .map(|(from, _)| format!("quote({})", quoted(from)))
                .collect::<Vec<_>>()
                .join(", "),
            quoted(&table),
        );
        let row = sqlx::query(AssertSqlSafe(select))
            .bind(rowid)
            .fetch_one(&mut *conn)
            .await?;
        let values = (0..pairs.len())
            .map(|i| row.try_get::<String, _>(i))
            .collect::<Result<Vec<_>, _>>()?;
        missing.push(MissingReference {
            columns: pairs
                .iter()
                .map(|(from, _)| from.clone())
                .zip(values)
                .collect(),
            parent_columns: pairs.into_iter().map(|(_, to)| to).collect(),
            table,
            rowid,
            parent,
        });
    }
    Ok(missing)
}

/// Inserts every row into the new, empty tables. Every foreign key is deferred, so the order of the inserts
/// doesn't matter to them.
async fn insert_all(
    conn: &mut SqliteConnection,
    data: &GameData,
    hash: &str,
) -> Result<(), sqlx::Error> {
    insert_mythic_ascensions(conn, data).await?;
    for skill in data.skills.iter() {
        sqlx::query!(
            "INSERT INTO skill (key, name, ability) VALUES ($1, $2, $3)",
            skill.key.as_str(),
            skill.name,
            ability_text(skill.ability),
        )
        .execute(&mut *conn)
        .await?;
    }
    for weapon in data.weapons.iter() {
        sqlx::query!(
            "INSERT INTO weapon (key, name, proficiency) VALUES ($1, $2, $3)",
            weapon.key.as_str(),
            weapon.name,
            weapon.proficiency.map(weapon_proficiency_text),
        )
        .execute(&mut *conn)
        .await?;
    }
    for race in data.races.iter() {
        insert_race(conn, race).await?;
    }
    for feat in data.feats.iter() {
        let root = prerequisites::insert(conn, feat.prerequisite.as_ref()).await?;
        let (repeat, repeat_times) = repeat_columns(feat.repeat);
        sqlx::query!(
            "INSERT INTO feat (key, name, category, summary, guide, prerequisite_root_id, parameter,
                repeat, repeat_times)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            feat.key.as_str(),
            feat.name,
            feat_category_text(feat.category),
            feat.summary,
            feat.guide.0,
            root,
            feat.parameter.map(parameter_kind_text),
            repeat,
            repeat_times,
        )
        .execute(&mut *conn)
        .await?;
        for selection in feat.extra_pick_of.iter() {
            sqlx::query!(
                "INSERT INTO feat_extra_pick (feat_key, selection_key) VALUES ($1, $2)",
                feat.key.as_str(),
                selection.as_str(),
            )
            .execute(&mut *conn)
            .await?;
        }
    }
    for background in data.backgrounds.iter() {
        insert_background(conn, background).await?;
    }
    for deity in data.deities.iter() {
        insert_deity(conn, deity).await?;
    }
    for class in data.classes.iter() {
        insert_class(conn, class, false).await?;
    }
    for class in data.prestige_classes.iter() {
        insert_class(conn, class, true).await?;
    }
    for ability in data.mythic_abilities.iter() {
        let root = prerequisites::insert(conn, ability.prerequisite.as_ref()).await?;
        sqlx::query!(
            "INSERT INTO mythic_ability
                (key, name, group_name, summary, guide, prerequisite_root_id)
            VALUES ($1, $2, $3, $4, $5, $6)",
            ability.key.as_str(),
            ability.name,
            mythic_ability_group_text(ability.group),
            ability.summary,
            ability.guide.0,
            root,
        )
        .execute(&mut *conn)
        .await?;
        let owner = SlotOwner::MythicAbility(ability.key.as_str());
        insert_grants(conn, owner, &ability.features, &ability.selections).await?;
    }
    for feat in data.mythic_feats.iter() {
        let root = prerequisites::insert(conn, feat.prerequisite.as_ref()).await?;
        let (repeat, repeat_times) = repeat_columns(feat.repeat);
        sqlx::query!(
            "INSERT INTO mythic_feat
                (key, name, group_name, summary, guide, prerequisite_root_id, parameter, repeat,
                repeat_times, removes_racial_penalty, extra_pick)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            feat.key.as_str(),
            feat.name,
            mythic_feat_group_text(feat.group),
            feat.summary,
            feat.guide.0,
            root,
            feat.parameter.map(parameter_kind_text),
            repeat,
            repeat_times,
            feat.removes_racial_penalty,
            feat.extra_pick.map(mythic_extra_pick_text),
        )
        .execute(&mut *conn)
        .await?;
    }
    for path in data.mythic_paths.iter() {
        insert_mythic_path(conn, path).await?;
    }

    sqlx::query!("INSERT INTO game_data_seed (hash) VALUES ($1)", hash)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

async fn insert_mythic_ascensions(
    conn: &mut SqliteConnection,
    data: &GameData,
) -> Result<(), sqlx::Error> {
    for ascension in data.progression.mythic_ascensions.iter() {
        sqlx::query!(
            "INSERT INTO mythic_ascension (rank, character_level) VALUES ($1, $2)",
            i16::from(ascension.rank),
            i16::from(ascension.character_level),
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_race(conn: &mut SqliteConnection, race: &Race) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO race (key, name, summary, guide) VALUES ($1, $2, $3, $4)",
        race.key.as_str(),
        race.name,
        race.summary,
        race.guide.0,
    )
    .execute(&mut *conn)
    .await?;
    insert_ability_modifiers(conn, Some(race.key.as_str()), None, &race.ability_modifiers).await?;
    insert_feat_lists(conn, Owner::Race(race.key.as_str()), &race.feat_lists).await?;
    insert_feat_slots(conn, SlotOwner::Race(race.key.as_str()), &race.feat_slots).await?;

    for subrace in race.subraces.iter() {
        sqlx::query!(
            "INSERT INTO subrace (key, race_key, name, summary, guide)
            VALUES ($1, $2, $3, $4, $5)",
            subrace.key.as_str(),
            race.key.as_str(),
            subrace.name,
            subrace.summary,
            subrace.guide.0,
        )
        .execute(&mut *conn)
        .await?;
        insert_ability_modifiers(
            conn,
            None,
            Some(subrace.key.as_str()),
            &subrace.ability_modifiers,
        )
        .await?;
        insert_feat_slots(
            conn,
            SlotOwner::Subrace(subrace.key.as_str()),
            &subrace.feat_slots,
        )
        .await?;
    }
    Ok(())
}

async fn insert_ability_modifiers(
    conn: &mut SqliteConnection,
    race_key: Option<&str>,
    subrace_key: Option<&str>,
    modifiers: &[AbilityModifier],
) -> Result<(), sqlx::Error> {
    for modifier in modifiers.iter() {
        // No ability means the chosen kind.
        let (ability, amount) = match modifier {
            AbilityModifier::Fixed { ability, amount } => (Some(ability_text(*ability)), *amount),
            AbilityModifier::Chosen { amount } => (None, *amount),
        };
        sqlx::query!(
            "INSERT INTO ability_modifier (race_key, subrace_key, ability, amount)
            VALUES ($1, $2, $3, $4)",
            race_key,
            subrace_key,
            ability,
            i16::from(amount),
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// A class, or one of its archetypes: who a class skill list or a level table belongs to. An
/// archetype's rows name its class too (`class_key`, `archetype_key`).
#[derive(Clone, Copy)]
struct ClassOwner<'a> {
    class: &'a str,
    archetype: Option<&'a str>,
}

/// Who a feature, a feature selection or a feat list belongs to (a race owns feat lists only).
#[derive(Clone, Copy)]
enum Owner<'a> {
    Class(ClassOwner<'a>),
    Path(&'a str),
    Race(&'a str),
}

impl<'a> Owner<'a> {
    /// `(class_key, archetype_key, path_key)` columns.
    fn columns(self) -> (Option<&'a str>, Option<&'a str>, Option<&'a str>) {
        match self {
            Self::Class(owner) => (Some(owner.class), owner.archetype, None),
            Self::Path(key) => (None, None, Some(key)),
            Self::Race(_) => (None, None, None),
        }
    }

    /// The `race_key` column of a race's feat list.
    fn race(self) -> Option<&'a str> {
        match self {
            Self::Race(key) => Some(key),
            _ => None,
        }
    }
}

async fn insert_class(
    conn: &mut SqliteConnection,
    class: &Class,
    is_prestige: bool,
) -> Result<(), sqlx::Error> {
    let root = prerequisites::insert(conn, class.prerequisite.as_ref()).await?;
    sqlx::query!(
        "INSERT INTO class (key, is_prestige, name, summary, guide, skill_points, bab, fort, reflex,
            will, advances, spontaneous, prerequisite_root_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        class.key.as_str(),
        is_prestige,
        class.name,
        class.summary,
        class.guide.0,
        i16::from(class.skill_points),
        bab_progression_text(class.bab),
        save_progression_text(class.fort),
        save_progression_text(class.reflex),
        save_progression_text(class.will),
        advancement_text(class.advances),
        class.spontaneous,
        root,
    )
    .execute(&mut *conn)
    .await?;
    for counts in class.counts_as.iter() {
        let (toward, feature) = match &counts.toward {
            CountsToward::FeatPrerequisites => ("feat_prerequisites", None),
            CountsToward::Feature(feature) => ("feature", Some(feature.as_str())),
        };
        sqlx::query!(
            "INSERT INTO class_counts_as (class_key, as_class_key, toward, feature_key)
            VALUES ($1, $2, $3, $4)",
            class.key.as_str(),
            counts.class.as_str(),
            toward,
            feature,
        )
        .execute(&mut *conn)
        .await?;
    }
    for exception in class.save_exceptions.iter() {
        sqlx::query!(
            "INSERT INTO save_exception (class_key, level, save, value) VALUES ($1, $2, $3, $4)",
            class.key.as_str(),
            i16::from(exception.level),
            save_text(exception.save),
            i16::from(exception.value),
        )
        .execute(&mut *conn)
        .await?;
    }
    let owner = ClassOwner {
        class: class.key.as_str(),
        archetype: None,
    };
    insert_class_skills(conn, owner, &class.class_skills).await?;
    insert_features(conn, Owner::Class(owner), &class.features).await?;
    insert_selections(conn, Owner::Class(owner), &class.selections).await?;
    insert_feat_lists(conn, Owner::Class(owner), &class.feat_lists).await?;
    insert_levels(conn, owner, &class.levels).await?;
    for archetype in class.archetypes.iter() {
        insert_archetype(conn, class, archetype).await?;
    }
    Ok(())
}

async fn insert_archetype(
    conn: &mut SqliteConnection,
    class: &Class,
    archetype: &Archetype,
) -> Result<(), sqlx::Error> {
    let root = prerequisites::insert(conn, archetype.prerequisite.as_ref()).await?;
    sqlx::query!(
        "INSERT INTO archetype (key, class_key, name, summary, guide,
            prerequisite_root_id, skill_points, overrides_class_skills, spontaneous)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        archetype.key.as_str(),
        class.key.as_str(),
        archetype.name,
        archetype.summary,
        archetype.guide.0,
        root,
        archetype.skill_points.map(i16::from),
        archetype.class_skills.is_some(),
        archetype.spontaneous,
    )
    .execute(&mut *conn)
    .await?;
    let owner = ClassOwner {
        class: class.key.as_str(),
        archetype: Some(archetype.key.as_str()),
    };
    if let Some(class_skills) = &archetype.class_skills {
        insert_class_skills(conn, owner, class_skills).await?;
    }
    insert_features(conn, Owner::Class(owner), &archetype.features).await?;
    insert_selections(conn, Owner::Class(owner), &archetype.selections).await?;
    insert_feat_lists(conn, Owner::Class(owner), &archetype.feat_lists).await?;
    insert_archetype_levels(conn, owner, &archetype.levels).await
}

/// A background, its class skills and its granted feats.
async fn insert_background(
    conn: &mut SqliteConnection,
    background: &Background,
) -> Result<(), sqlx::Error> {
    let key = background.key.as_str();
    sqlx::query!(
        "INSERT INTO background (key, name, background_group, note) VALUES ($1, $2, $3, $4)",
        key,
        background.name,
        background.group.map(background_group_text),
        background.note,
    )
    .execute(&mut *conn)
    .await?;
    for skill in background.class_skills.iter() {
        sqlx::query!(
            "INSERT INTO class_skill (background_key, skill_key) VALUES ($1, $2)",
            key,
            skill.as_str(),
        )
        .execute(&mut *conn)
        .await?;
    }
    let owner = SlotOwner::Background(key);
    insert_granted_feats(conn, owner, &background.feats).await?;
    insert_granted_feats_for(conn, owner, &background.feats_for).await
}

/// A deity, with its alignments, favoured weapons and domains.
async fn insert_deity(conn: &mut SqliteConnection, deity: &Deity) -> Result<(), sqlx::Error> {
    let key = deity.key.as_str();
    sqlx::query!(
        "INSERT INTO deity (key, name, guide) VALUES ($1, $2, $3)",
        key,
        deity.name,
        deity.guide.0,
    )
    .execute(&mut *conn)
    .await?;
    for alignment in deity.alignments.iter() {
        sqlx::query!(
            "INSERT INTO deity_alignment (deity_key, alignment) VALUES ($1, $2)",
            key,
            alignment_text(*alignment),
        )
        .execute(&mut *conn)
        .await?;
    }
    for weapon in deity.favoured_weapons.iter() {
        sqlx::query!(
            "INSERT INTO deity_weapon (deity_key, weapon_key) VALUES ($1, $2)",
            key,
            weapon.as_str(),
        )
        .execute(&mut *conn)
        .await?;
    }
    for domain in deity.domains.iter() {
        sqlx::query!(
            "INSERT INTO deity_domain (deity_key, domain_key) VALUES ($1, $2)",
            key,
            domain.as_str(),
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_class_skills(
    conn: &mut SqliteConnection,
    owner: ClassOwner<'_>,
    skills: &BTreeSet<SkillKey>,
) -> Result<(), sqlx::Error> {
    for skill in skills.iter() {
        sqlx::query!(
            "INSERT INTO class_skill (class_key, archetype_key, skill_key)
            VALUES ($1, $2, $3)",
            owner.class,
            owner.archetype,
            skill.as_str(),
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_features(
    conn: &mut SqliteConnection,
    owner: Owner<'_>,
    features: &[Feature],
) -> Result<(), sqlx::Error> {
    let (class_key, archetype_key, path_key) = owner.columns();
    for feature in features.iter() {
        sqlx::query!(
            "INSERT INTO feature
                (key, class_key, archetype_key, path_key, name, summary, guide,
                option_row_selection_key, option_row_level, daily_choice, bab_floor,
                skill_ranks_at_least_mythic_rank, favoured_weapon_proficiency)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            feature.key.as_str(),
            class_key,
            archetype_key,
            path_key,
            feature.name,
            feature.summary,
            feature.guide.0,
            feature
                .option_row
                .as_ref()
                .map(|row| row.selection.as_str()),
            feature.option_row.as_ref().map(|row| i16::from(row.level)),
            feature.daily_choice,
            feature.bab_floor.map(i16::from),
            feature.skill_ranks_at_least_mythic_rank,
            feature.favoured_weapon_proficiency,
        )
        .execute(&mut *conn)
        .await?;
        for bonus in feature.ability_bonuses.iter() {
            let (kind, ability, amount) = match *bonus {
                AbilityBonus::Fixed { ability, amount } => {
                    (AbilityBonusKind::Fixed, Some(ability_text(ability)), amount)
                }
                AbilityBonus::Every { amount } => (AbilityBonusKind::Every, None, amount),
                AbilityBonus::Highest { amount } => (AbilityBonusKind::Highest, None, amount),
                AbilityBonus::AtLeast { score } => (AbilityBonusKind::AtLeast, None, score),
            };
            sqlx::query!(
                "INSERT INTO feature_ability_bonus (feature_key, kind, ability, amount)
                VALUES ($1, $2, $3, $4)",
                feature.key.as_str(),
                ability_bonus_kind_text(kind),
                ability,
                i16::from(amount),
            )
            .execute(&mut *conn)
            .await?;
        }
    }
    Ok(())
}

async fn insert_selections(
    conn: &mut SqliteConnection,
    owner: Owner<'_>,
    selections: &[FeatureSelection],
) -> Result<(), sqlx::Error> {
    let (class_key, archetype_key, path_key) = owner.columns();
    for selection in selections.iter() {
        sqlx::query!(
            "INSERT INTO feature_selection
                (key, class_key, archetype_key, path_key, name, summary, guide, feat_lists_only,
                only_without_option, pool_of_key, levels_behind, level_from, level_also_from_key,
                once_per_option, deity_domains)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
            selection.key.as_str(),
            class_key,
            archetype_key,
            path_key,
            selection.name,
            selection.summary,
            selection.guide.0,
            selection.feat_lists_only,
            selection.only_without_option,
            selection.pool_of.as_ref().map(FeatureSelectionKey::as_str),
            i16::from(selection.levels_behind),
            selection.level_from.map(level_from_text),
            selection
                .level_also_from
                .as_ref()
                .map(SelectionOptionKey::as_str),
            selection.once_per_option,
            selection.deity_domains.map(deity_domains_text),
        )
        .execute(&mut *conn)
        .await?;
        for class in selection.option_levels.iter() {
            sqlx::query!(
                "INSERT INTO selection_level_class (selection_key, class_key) VALUES ($1, $2)",
                selection.key.as_str(),
                class.as_str(),
            )
            .execute(&mut *conn)
            .await?;
        }
        for level in selection.skipped_levels.iter() {
            sqlx::query!(
                "INSERT INTO selection_skipped_level (selection_key, level) VALUES ($1, $2)",
                selection.key.as_str(),
                i16::from(*level),
            )
            .execute(&mut *conn)
            .await?;
        }
        for option in selection.options.iter() {
            let root = prerequisites::insert(conn, option.prerequisite.as_ref()).await?;
            let (repeat, repeat_times) = repeat_columns(option.repeat);
            sqlx::query!(
                "INSERT INTO selection_option (key, selection_key, name, summary, guide,
                    prerequisite_root_id, chosen_feat_key, parameter, repeat, repeat_times, pool_key,
                    replaces_row_of_key, option_group, arcana_key, domain_key)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
                option.key.as_str(),
                selection.key.as_str(),
                option.name,
                option.summary,
                option.guide.0,
                root,
                option.chosen_feat.as_ref().map(FeatKey::as_str),
                option.parameter.map(parameter_kind_text),
                repeat,
                repeat_times,
                option.pool.as_ref().map(FeatureSelectionKey::as_str),
                option
                    .replaces_row_of
                    .as_ref()
                    .map(FeatureSelectionKey::as_str),
                option.group.map(option_group_text),
                option.arcana.as_ref().map(FeatureKey::as_str),
                option.domain.as_ref().map(SelectionOptionKey::as_str),
            )
            .execute(&mut *conn)
            .await?;
            for row in option.levels.iter() {
                let id = sqlx::query_scalar!(
                    "INSERT INTO option_level (option_key, level, feat_list_key)
                    VALUES ($1, $2, $3)
                    RETURNING id AS \"id!\"",
                    option.key.as_str(),
                    i16::from(row.level),
                    row.feat_list.as_ref().map(FeatListKey::as_str),
                )
                .fetch_one(&mut *conn)
                .await?;
                let owner = SlotOwner::OptionLevel(id);
                insert_grants(conn, owner, &row.features, &row.selections).await?;
                insert_granted_feats(conn, owner, &row.feats).await?;
                insert_feat_slots(conn, owner, &row.feat_slots).await?;
                for skill in row.class_skills.iter() {
                    sqlx::query!(
                        "INSERT INTO class_skill (option_level_id, skill_key) VALUES ($1, $2)",
                        id,
                        skill.as_str(),
                    )
                    .execute(&mut *conn)
                    .await?;
                }
                for feat in row.chosen_feats.iter() {
                    sqlx::query!(
                        "INSERT INTO option_level_chosen_feat (option_level_id, feat_key) VALUES ($1, $2)",
                        id,
                        feat.as_str(),
                    )
                    .execute(&mut *conn)
                    .await?;
                }
            }
            for value in option.values.iter() {
                let value = ValueColumns::of(value);
                sqlx::query!(
                    "INSERT INTO selection_option_value
                        (option_key, skill_key, weapon_key, spell_school, energy, armour_type)
                    VALUES ($1, $2, $3, $4, $5, $6)",
                    option.key.as_str(),
                    value.skill_key,
                    value.weapon_key,
                    value.spell_school,
                    value.energy,
                    value.armour_type,
                )
                .execute(&mut *conn)
                .await?;
            }
            for feat in option.feats.iter() {
                sqlx::query!(
                    "INSERT INTO selection_option_feat (option_key, feat_key) VALUES ($1, $2)",
                    option.key.as_str(),
                    feat.as_str(),
                )
                .execute(&mut *conn)
                .await?;
            }
        }
        for option in selection.shared_options.iter() {
            sqlx::query!(
                "INSERT INTO selection_shared_option (selection_key, option_key) VALUES ($1, $2)",
                selection.key.as_str(),
                option.as_str(),
            )
            .execute(&mut *conn)
            .await?;
        }
    }
    Ok(())
}

async fn insert_feat_lists(
    conn: &mut SqliteConnection,
    owner: Owner<'_>,
    lists: &[FeatList],
) -> Result<(), sqlx::Error> {
    let (class_key, archetype_key, path_key) = owner.columns();
    for list in lists.iter() {
        sqlx::query!(
            "INSERT INTO feat_list
                (key, class_key, archetype_key, path_key, race_key, name, guide, ignores_prerequisites,
                every_feat)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            list.key.as_str(),
            class_key,
            archetype_key,
            path_key,
            owner.race(),
            list.name,
            list.guide.0,
            list.ignores_prerequisites,
            list.every_feat,
        )
        .execute(&mut *conn)
        .await?;
        for feat in list.feats.iter() {
            sqlx::query!(
                "INSERT INTO feat_list_member (feat_list_key, feat_key) VALUES ($1, $2)",
                list.key.as_str(),
                feat.as_str(),
            )
            .execute(&mut *conn)
            .await?;
        }
    }
    Ok(())
}

async fn insert_levels(
    conn: &mut SqliteConnection,
    owner: ClassOwner<'_>,
    levels: &[ClassLevelRow],
) -> Result<(), sqlx::Error> {
    for row in levels.iter() {
        let id = sqlx::query_scalar!(
            "INSERT INTO class_level (class_key, archetype_key, level, arcane_spell_level,
                divine_spell_level, advances_arcane, advances_divine)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id AS \"id!\"",
            owner.class,
            owner.archetype,
            i16::from(row.level),
            i16::from(row.arcane_spell_level),
            i16::from(row.divine_spell_level),
            row.advances_arcane,
            row.advances_divine,
        )
        .fetch_one(&mut *conn)
        .await?;
        let grantor = SlotOwner::ClassLevel(id);
        insert_grants(conn, grantor, &row.features, &row.selections).await?;
        insert_granted_feats(conn, grantor, &row.feats).await?;
        insert_granted_feats_for(conn, grantor, &row.feats_for).await?;
        insert_feat_slots(conn, grantor, &row.feat_slots).await?;
    }
    Ok(())
}

/// A class level's or a background's feats granted for one value: `granted_feat` rows with their value column
/// set.
async fn insert_granted_feats_for(
    conn: &mut SqliteConnection,
    owner: SlotOwner<'_>,
    feats: &BTreeSet<FeatValue>,
) -> Result<(), sqlx::Error> {
    let columns = owner.columns();
    for granted in feats.iter() {
        let value = ValueColumns::of(&granted.value);
        sqlx::query!(
            "INSERT INTO granted_feat (class_level_id, background_key, feat_key, skill_key, weapon_key,
                spell_school, energy, armour_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            columns.class_level_id,
            columns.background_key,
            granted.feat.as_str(),
            value.skill_key,
            value.weapon_key,
            value.spell_school,
            value.energy,
            value.armour_type,
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// A class level's, an option level's or a background's granted feats.
async fn insert_granted_feats(
    conn: &mut SqliteConnection,
    owner: SlotOwner<'_>,
    feats: &BTreeSet<FeatKey>,
) -> Result<(), sqlx::Error> {
    let columns = owner.columns();
    for feat in feats.iter() {
        sqlx::query!(
            "INSERT INTO granted_feat (class_level_id, option_level_id, background_key, feat_key)
            VALUES ($1, $2, $3, $4)",
            columns.class_level_id,
            columns.option_level_id,
            columns.background_key,
            feat.as_str(),
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// An archetype's rows: spell levels only where it sets its own, and no advances flags.
async fn insert_archetype_levels(
    conn: &mut SqliteConnection,
    owner: ClassOwner<'_>,
    levels: &[ArchetypeLevelRow],
) -> Result<(), sqlx::Error> {
    for row in levels.iter() {
        let id = sqlx::query_scalar!(
            "INSERT INTO class_level (class_key, archetype_key, level, arcane_spell_level,
                divine_spell_level)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id AS \"id!\"",
            owner.class,
            owner.archetype,
            i16::from(row.level),
            row.spell_levels.map(|spells| i16::from(spells.arcane)),
            row.spell_levels.map(|spells| i16::from(spells.divine)),
        )
        .fetch_one(&mut *conn)
        .await?;
        let grantor = SlotOwner::ClassLevel(id);
        insert_grants(conn, grantor, &row.features, &row.selections).await?;
        insert_granted_feats(conn, grantor, &row.feats).await?;
        insert_granted_feats_for(conn, grantor, &row.feats_for).await?;
        insert_feat_slots(conn, grantor, &row.feat_slots).await?;
    }
    Ok(())
}

async fn insert_mythic_path(
    conn: &mut SqliteConnection,
    path: &MythicPath,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO mythic_path (key, name, summary, guide) VALUES ($1, $2, $3, $4)",
        path.key.as_str(),
        path.name,
        path.summary,
        path.guide.0,
    )
    .execute(&mut *conn)
    .await?;
    if let Some(switch) = &path.switch {
        sqlx::query!(
            "INSERT INTO path_switch (path_key, kept_through_rank, path_kept_through_rank,
                further_ascensions, level_cap)
            VALUES ($1, $2, $3, $4, $5)",
            path.key.as_str(),
            i16::from(switch.kept_through_rank),
            i16::from(switch.path_kept_through_rank),
            switch.further_ascensions,
            switch.level_cap.map(i16::from),
        )
        .execute(&mut *conn)
        .await?;
        for from in switch.from.iter() {
            sqlx::query!(
                "INSERT INTO path_switch_from (path_key, from_path_key) VALUES ($1, $2)",
                path.key.as_str(),
                from.as_str(),
            )
            .execute(&mut *conn)
            .await?;
        }
        let drops = switch
            .dropped_features
            .iter()
            .map(|feature| (Some(feature.as_str()), None))
            .chain(
                switch
                    .dropped_selections
                    .iter()
                    .map(|selection| (None, Some(selection.as_str()))),
            );
        for (feature, selection) in drops {
            sqlx::query!(
                "INSERT INTO path_switch_drop (path_key, feature_key, selection_key)
                VALUES ($1, $2, $3)",
                path.key.as_str(),
                feature,
                selection,
            )
            .execute(&mut *conn)
            .await?;
        }
        for feature in switch.features.iter() {
            sqlx::query!(
                "INSERT INTO path_switch_feature (path_key, feature_key) VALUES ($1, $2)",
                path.key.as_str(),
                feature.as_str(),
            )
            .execute(&mut *conn)
            .await?;
        }
    }

    let owner = Owner::Path(path.key.as_str());
    insert_features(conn, owner, &path.features).await?;
    insert_selections(conn, owner, &path.selections).await?;
    insert_feat_lists(conn, owner, &path.feat_lists).await?;
    for row in path.ranks.iter() {
        let id = sqlx::query_scalar!(
            "INSERT INTO mythic_rank (path_key, rank) VALUES ($1, $2) RETURNING id AS \"id!\"",
            path.key.as_str(),
            i16::from(row.rank),
        )
        .fetch_one(&mut *conn)
        .await?;
        let owner = SlotOwner::MythicRank(id);
        insert_grants(conn, owner, &row.features, &row.selections).await?;
        insert_feat_slots(conn, owner, &row.feat_slots).await?;
    }
    Ok(())
}

/// Who a feat slot, granted feature, granted selection or granted feat belongs to.
#[derive(Clone, Copy)]
enum SlotOwner<'a> {
    Race(&'a str),
    Subrace(&'a str),
    ClassLevel(i64),
    MythicRank(i64),
    OptionLevel(i64),
    MythicAbility(&'a str),
    Background(&'a str),
}

/// The owner columns of a grant: exactly one is set.
#[derive(Default)]
struct OwnerColumns<'a> {
    race_key: Option<&'a str>,
    subrace_key: Option<&'a str>,
    class_level_id: Option<i64>,
    mythic_rank_id: Option<i64>,
    option_level_id: Option<i64>,
    mythic_ability_key: Option<&'a str>,
    background_key: Option<&'a str>,
}

impl<'a> SlotOwner<'a> {
    fn columns(self) -> OwnerColumns<'a> {
        match self {
            Self::Race(key) => OwnerColumns {
                race_key: Some(key),
                ..OwnerColumns::default()
            },
            Self::Subrace(key) => OwnerColumns {
                subrace_key: Some(key),
                ..OwnerColumns::default()
            },
            Self::ClassLevel(id) => OwnerColumns {
                class_level_id: Some(id),
                ..OwnerColumns::default()
            },
            Self::MythicRank(id) => OwnerColumns {
                mythic_rank_id: Some(id),
                ..OwnerColumns::default()
            },
            Self::OptionLevel(id) => OwnerColumns {
                option_level_id: Some(id),
                ..OwnerColumns::default()
            },
            Self::MythicAbility(key) => OwnerColumns {
                mythic_ability_key: Some(key),
                ..OwnerColumns::default()
            },
            Self::Background(key) => OwnerColumns {
                background_key: Some(key),
                ..OwnerColumns::default()
            },
        }
    }
}

/// Each distinct key of a multiset of grants, with how many times it appears.
fn counted<'a>(keys: impl IntoIterator<Item = &'a str>) -> BTreeMap<&'a str, i16> {
    let mut counts = BTreeMap::new();
    for key in keys {
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

/// A row's granted features and selections, one table row per distinct key with its count.
async fn insert_grants(
    conn: &mut SqliteConnection,
    owner: SlotOwner<'_>,
    features: &[FeatureKey],
    selections: &[FeatureSelectionKey],
) -> Result<(), sqlx::Error> {
    let columns = owner.columns();
    for (feature, count) in counted(features.iter().map(FeatureKey::as_str)) {
        sqlx::query!(
            "INSERT INTO granted_feature (class_level_id, mythic_rank_id, option_level_id,
                mythic_ability_key, feature_key, count)
            VALUES ($1, $2, $3, $4, $5, $6)",
            columns.class_level_id,
            columns.mythic_rank_id,
            columns.option_level_id,
            columns.mythic_ability_key,
            feature,
            count,
        )
        .execute(&mut *conn)
        .await?;
    }
    for (selection, count) in counted(selections.iter().map(FeatureSelectionKey::as_str)) {
        sqlx::query!(
            "INSERT INTO granted_selection (class_level_id, mythic_rank_id, option_level_id,
                mythic_ability_key, selection_key, count)
            VALUES ($1, $2, $3, $4, $5, $6)",
            columns.class_level_id,
            columns.mythic_rank_id,
            columns.option_level_id,
            columns.mythic_ability_key,
            selection,
            count,
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_feat_slots(
    conn: &mut SqliteConnection,
    owner: SlotOwner<'_>,
    slots: &[FeatSlot],
) -> Result<(), sqlx::Error> {
    let columns = owner.columns();
    for slot in slots.iter() {
        let (kind, category, list, selection, at_level, label) = match slot {
            FeatSlot::General => (FeatSlotKind::General, None, None, None, None, None),
            FeatSlot::CombatFeat => (FeatSlotKind::CombatFeat, None, None, None, None, None),
            FeatSlot::Category(category) => (
                FeatSlotKind::Category,
                Some(feat_category_text(*category)),
                None,
                None,
                None,
                None,
            ),
            FeatSlot::OneOf { list } => (
                FeatSlotKind::OneOf,
                None,
                Some(list.as_str()),
                None,
                None,
                None,
            ),
            FeatSlot::OptionList {
                selection,
                at_level,
                label,
            } => (
                FeatSlotKind::OptionList,
                None,
                None,
                Some(selection.as_str()),
                at_level.map(i16::from),
                label.as_deref(),
            ),
        };
        sqlx::query!(
            "INSERT INTO feat_slot
                (race_key, subrace_key, class_level_id, mythic_rank_id, option_level_id, kind,
                category, feat_list_key, selection_key, at_level, label)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            columns.race_key,
            columns.subrace_key,
            columns.class_level_id,
            columns.mythic_rank_id,
            columns.option_level_id,
            feat_slot_kind_text(kind),
            category,
            list,
            selection,
            at_level,
            label,
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}
