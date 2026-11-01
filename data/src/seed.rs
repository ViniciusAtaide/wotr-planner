//! Replacing every game table with the contents of a [`GameData`], in one transaction.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    AbilityModifier, Archetype, ArchetypeLevelRow, Class, ClassLevelRow, FeatKey, FeatList,
    FeatSlot, Feature, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData, MythicPath,
    Race, SkillKey,
};
use sqlx::PgConnection;

use crate::columns::{
    FeatSlotKind, ability_text, bab_progression_text, feat_category_text, feat_slot_kind_text,
    mythic_ability_group_text, mythic_feat_group_text, save_progression_text, save_text,
};
use crate::{Database, Error, prerequisites};

/// Checks `data` ([`GameData::check`]), then deletes every game table's rows and loads `data` in
/// their place, and records `hash` as the data the tables came from. Either all of it happens or
/// none of it does; data with problems writes nothing.
pub async fn replace_game_data(db: &Database, data: &GameData, hash: &str) -> Result<(), Error> {
    let problems = data.check();
    if !problems.is_empty() {
        return Err(Error::Invalid(problems));
    }
    insert_all(db, data, hash).await?;
    Ok(())
}

async fn insert_all(db: &Database, data: &GameData, hash: &str) -> Result<(), sqlx::Error> {
    let mut tx = db.pool().begin().await?;
    let conn: &mut PgConnection = &mut tx;

    // Every foreign key is deferred, so the order of the inserts below doesn't matter to them.
    sqlx::query!(
        "TRUNCATE game_data_seed, mythic_ascension, prerequisite_node, skill, race, subrace,
            ability_modifier, feat, class, save_exception, archetype, class_skill, mythic_ability,
            mythic_feat,
            mythic_path, path_switch, path_switch_kept_rank, feature, feature_selection,
            selection_option, selection_shared_option, feat_list, feat_list_member, class_level,
            mythic_rank, granted_feature, granted_selection, granted_feat, feat_slot
        RESTART IDENTITY"
    )
    .execute(&mut *conn)
    .await?;

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
    for race in data.races.iter() {
        insert_race(conn, race).await?;
    }
    for feat in data.feats.iter() {
        let root = prerequisites::insert(conn, feat.prerequisite.as_ref()).await?;
        sqlx::query!(
            "INSERT INTO feat (key, name, category, summary, guide, prerequisite_root_id)
            VALUES ($1, $2, $3, $4, $5, $6)",
            feat.key.as_str(),
            feat.name,
            feat_category_text(feat.category),
            feat.summary,
            feat.guide.0,
            root,
        )
        .execute(&mut *conn)
        .await?;
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
    }
    for feat in data.mythic_feats.iter() {
        let root = prerequisites::insert(conn, feat.prerequisite.as_ref()).await?;
        sqlx::query!(
            "INSERT INTO mythic_feat
                (key, name, group_name, summary, guide, prerequisite_root_id)
            VALUES ($1, $2, $3, $4, $5, $6)",
            feat.key.as_str(),
            feat.name,
            mythic_feat_group_text(feat.group),
            feat.summary,
            feat.guide.0,
            root,
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
    tx.commit().await
}

async fn insert_mythic_ascensions(
    conn: &mut PgConnection,
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

async fn insert_race(conn: &mut PgConnection, race: &Race) -> Result<(), sqlx::Error> {
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
    }
    Ok(())
}

async fn insert_ability_modifiers(
    conn: &mut PgConnection,
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

/// Who a feature, a feature selection or a feat list belongs to.
#[derive(Clone, Copy)]
enum Owner<'a> {
    Class(ClassOwner<'a>),
    Path(&'a str),
}

impl<'a> Owner<'a> {
    /// `(class_key, archetype_key, path_key)` columns.
    fn columns(self) -> (Option<&'a str>, Option<&'a str>, Option<&'a str>) {
        match self {
            Self::Class(owner) => (Some(owner.class), owner.archetype, None),
            Self::Path(key) => (None, None, Some(key)),
        }
    }
}

async fn insert_class(
    conn: &mut PgConnection,
    class: &Class,
    is_prestige: bool,
) -> Result<(), sqlx::Error> {
    let root = prerequisites::insert(conn, class.prerequisite.as_ref()).await?;
    sqlx::query!(
        "INSERT INTO class (key, is_prestige, name, summary, guide, skill_points, bab, fort, reflex,
            will, prerequisite_root_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
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
        root,
    )
    .execute(&mut *conn)
    .await?;
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
    conn: &mut PgConnection,
    class: &Class,
    archetype: &Archetype,
) -> Result<(), sqlx::Error> {
    let root = prerequisites::insert(conn, archetype.prerequisite.as_ref()).await?;
    sqlx::query!(
        "INSERT INTO archetype (key, class_key, name, summary, guide,
            prerequisite_root_id, skill_points, overrides_class_skills)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        archetype.key.as_str(),
        class.key.as_str(),
        archetype.name,
        archetype.summary,
        archetype.guide.0,
        root,
        archetype.skill_points.map(i16::from),
        archetype.class_skills.is_some(),
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

async fn insert_class_skills(
    conn: &mut PgConnection,
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
    conn: &mut PgConnection,
    owner: Owner<'_>,
    features: &[Feature],
) -> Result<(), sqlx::Error> {
    let (class_key, archetype_key, path_key) = owner.columns();
    for feature in features.iter() {
        sqlx::query!(
            "INSERT INTO feature
                (key, class_key, archetype_key, path_key, name, summary, guide)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
            feature.key.as_str(),
            class_key,
            archetype_key,
            path_key,
            feature.name,
            feature.summary,
            feature.guide.0,
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_selections(
    conn: &mut PgConnection,
    owner: Owner<'_>,
    selections: &[FeatureSelection],
) -> Result<(), sqlx::Error> {
    let (class_key, archetype_key, path_key) = owner.columns();
    for selection in selections.iter() {
        sqlx::query!(
            "INSERT INTO feature_selection
                (key, class_key, archetype_key, path_key, name, summary, guide)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
            selection.key.as_str(),
            class_key,
            archetype_key,
            path_key,
            selection.name,
            selection.summary,
            selection.guide.0,
        )
        .execute(&mut *conn)
        .await?;
        for option in selection.options.iter() {
            let root = prerequisites::insert(conn, option.prerequisite.as_ref()).await?;
            sqlx::query!(
                "INSERT INTO selection_option (key, selection_key, name, summary, guide,
                    feat_key, prerequisite_root_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)",
                option.key.as_str(),
                selection.key.as_str(),
                option.name,
                option.summary,
                option.guide.0,
                option.feat.as_ref().map(|feat| feat.as_str()),
                root,
            )
            .execute(&mut *conn)
            .await?;
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
    conn: &mut PgConnection,
    owner: Owner<'_>,
    lists: &[FeatList],
) -> Result<(), sqlx::Error> {
    let (class_key, archetype_key, path_key) = owner.columns();
    for list in lists.iter() {
        sqlx::query!(
            "INSERT INTO feat_list
                (key, class_key, archetype_key, path_key, name, guide, ignores_prerequisites)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
            list.key.as_str(),
            class_key,
            archetype_key,
            path_key,
            list.name,
            list.guide.0,
            list.ignores_prerequisites,
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
    conn: &mut PgConnection,
    owner: ClassOwner<'_>,
    levels: &[ClassLevelRow],
) -> Result<(), sqlx::Error> {
    for row in levels.iter() {
        let id = sqlx::query_scalar!(
            "INSERT INTO class_level (class_key, archetype_key, level, arcane_spell_level,
                divine_spell_level, advances_arcane, advances_divine)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id",
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
        insert_granted_feats(conn, id, &row.feats).await?;
        insert_feat_slots(conn, grantor, &row.feat_slots).await?;
    }
    Ok(())
}

async fn insert_granted_feats(
    conn: &mut PgConnection,
    class_level_id: i64,
    feats: &BTreeSet<FeatKey>,
) -> Result<(), sqlx::Error> {
    for feat in feats.iter() {
        sqlx::query!(
            "INSERT INTO granted_feat (class_level_id, feat_key) VALUES ($1, $2)",
            class_level_id,
            feat.as_str(),
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// An archetype's rows: spell levels only where it sets its own, and no advances flags.
async fn insert_archetype_levels(
    conn: &mut PgConnection,
    owner: ClassOwner<'_>,
    levels: &[ArchetypeLevelRow],
) -> Result<(), sqlx::Error> {
    for row in levels.iter() {
        let id = sqlx::query_scalar!(
            "INSERT INTO class_level (class_key, archetype_key, level, arcane_spell_level,
                divine_spell_level)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id",
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
        insert_granted_feats(conn, id, &row.feats).await?;
        insert_feat_slots(conn, grantor, &row.feat_slots).await?;
    }
    Ok(())
}

async fn insert_mythic_path(conn: &mut PgConnection, path: &MythicPath) -> Result<(), sqlx::Error> {
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
            "INSERT INTO path_switch (path_key, further_ascensions, level_cap) VALUES ($1, $2, $3)",
            path.key.as_str(),
            switch.further_ascensions,
            switch.level_cap.map(i16::from),
        )
        .execute(&mut *conn)
        .await?;
        for rank in switch.kept_ranks.iter() {
            sqlx::query!(
                "INSERT INTO path_switch_kept_rank (path_key, rank) VALUES ($1, $2)",
                path.key.as_str(),
                i16::from(*rank),
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
            "INSERT INTO mythic_rank (path_key, rank) VALUES ($1, $2) RETURNING id",
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

/// Who a feat slot, granted feature or granted selection belongs to.
#[derive(Clone, Copy)]
enum SlotOwner<'a> {
    Race(&'a str),
    ClassLevel(i64),
    MythicRank(i64),
}

impl<'a> SlotOwner<'a> {
    /// `(race_key, class_level_id, mythic_rank_id)` columns.
    fn columns(self) -> (Option<&'a str>, Option<i64>, Option<i64>) {
        match self {
            Self::Race(key) => (Some(key), None, None),
            Self::ClassLevel(id) => (None, Some(id), None),
            Self::MythicRank(id) => (None, None, Some(id)),
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
    conn: &mut PgConnection,
    owner: SlotOwner<'_>,
    features: &[FeatureKey],
    selections: &[FeatureSelectionKey],
) -> Result<(), sqlx::Error> {
    let (_, class_level_id, mythic_rank_id) = owner.columns();
    for (feature, count) in counted(features.iter().map(FeatureKey::as_str)) {
        sqlx::query!(
            "INSERT INTO granted_feature (class_level_id, mythic_rank_id, feature_key, count)
            VALUES ($1, $2, $3, $4)",
            class_level_id,
            mythic_rank_id,
            feature,
            count,
        )
        .execute(&mut *conn)
        .await?;
    }
    for (selection, count) in counted(selections.iter().map(FeatureSelectionKey::as_str)) {
        sqlx::query!(
            "INSERT INTO granted_selection (class_level_id, mythic_rank_id, selection_key, count)
            VALUES ($1, $2, $3, $4)",
            class_level_id,
            mythic_rank_id,
            selection,
            count,
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_feat_slots(
    conn: &mut PgConnection,
    owner: SlotOwner<'_>,
    slots: &[FeatSlot],
) -> Result<(), sqlx::Error> {
    let (race_key, class_level_id, mythic_rank_id) = owner.columns();
    for slot in slots.iter() {
        let (kind, category, list) = match slot {
            FeatSlot::General => (FeatSlotKind::General, None, None),
            FeatSlot::CombatFeat => (FeatSlotKind::CombatFeat, None, None),
            FeatSlot::Category(category) => (
                FeatSlotKind::Category,
                Some(feat_category_text(*category)),
                None,
            ),
            FeatSlot::OneOf { list } => (FeatSlotKind::OneOf, None, Some(list.as_str())),
        };
        sqlx::query!(
            "INSERT INTO feat_slot
                (race_key, class_level_id, mythic_rank_id, kind, category, feat_list_key)
            VALUES ($1, $2, $3, $4, $5, $6)",
            race_key,
            class_level_id,
            mythic_rank_id,
            feat_slot_kind_text(kind),
            category,
            list,
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}
