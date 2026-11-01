//! Reading every game table back into one [`GameData`].
//!
//! Each table is read whole and the rows are grouped under their owners in memory. That's a fixed
//! number of queries however much data there is. A row whose owner never claims it is an error,
//! not silently dropped: that covers class skills under an archetype that keeps its class's list, and
//! prerequisite nodes outside exactly one owner's tree. The tables store no order, so the result is made
//! [canonical](GameData::canonicalize), the same as the files reader's.

use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;

use rules::{
    AbilityModifier, Archetype, ArchetypeKey, ArchetypeLevelRow, Class, ClassKey, ClassLevelRow,
    Feat, FeatKey, FeatSlot, Feature, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData,
    GuideRef, MythicAbility, MythicAbilityKey, MythicAscension, MythicFeat, MythicFeatKey,
    MythicPath, MythicPathKey, MythicRankRow, PathSwitch, Progression, Race, RaceKey,
    SaveException, SelectionOption, SelectionOptionKey, Skill, SkillKey, SpellLevels, Subrace,
    SubraceKey,
};
use sqlx::PgConnection;

use crate::columns::{
    FeatSlotKind, ability_from, bab_progression_from, corrupt, feat_category_from,
    feat_slot_kind_from, save_from, save_progression_from, to_i8, to_u8,
};
use crate::prerequisites::Trees;
use crate::{Database, Error};

/// Reads the seeded game data, in canonical order. Unseeded tables read as empty game data.
pub async fn load_game_data(db: &Database) -> Result<GameData, Error> {
    Ok(load(db).await?)
}

async fn load(db: &Database) -> Result<GameData, sqlx::Error> {
    // One snapshot: the seed may replace everything between two of the reads below.
    let mut tx = db.pool().begin().await?;
    sqlx::query!("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;
    let conn: &mut PgConnection = &mut tx;

    let mut trees = Trees::load(conn).await?;
    let progression = Progression {
        mythic_ascensions: load_mythic_ascensions(conn).await?,
    };
    let skills = load_skills(conn).await?;
    let mut feat_slots = load_feat_slots(conn).await?;
    let races = load_races(conn, &mut feat_slots).await?;
    let feats = load_feats(conn, &mut trees).await?;
    let mut grants = load_grants(conn).await?;
    let mut levels = load_levels(conn, &mut grants, &mut feat_slots).await?;
    let mut features = load_features(conn).await?;
    let mut selections = load_selections(conn, &mut trees).await?;
    let mut class_skills = load_class_skills(conn).await?;
    let (classes, prestige_classes) = load_classes(
        conn,
        &mut trees,
        &mut Parts {
            features: &mut features,
            selections: &mut selections,
            class_skills: &mut class_skills,
            levels: &mut levels,
        },
    )
    .await?;
    let mythic_abilities = load_mythic_abilities(conn, &mut trees).await?;
    let mythic_feats = load_mythic_feats(conn, &mut trees).await?;
    let mythic_paths = load_mythic_paths(
        conn,
        &mut features,
        &mut selections,
        &mut grants,
        &mut feat_slots,
    )
    .await?;
    feat_slots.finish("feat_slot")?;
    grants.features.finish("granted_feature")?;
    grants.selections.finish("granted_selection")?;
    levels.class_rows.finish("class_level")?;
    levels.archetype_rows.finish("class_level")?;
    features.finish("feature")?;
    selections.finish("feature_selection")?;
    class_skills.finish("class_skill")?;
    trees.finish()?;

    tx.commit().await?;
    let data = GameData {
        progression,
        races,
        skills,
        feats,
        classes,
        prestige_classes,
        mythic_abilities,
        mythic_feats,
        mythic_paths,
    };
    Ok(data.canonical())
}

/// Who a row belongs to, for grouping rows under their owner.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Owner {
    Race(String),
    Subrace(String),
    Class(String),
    Archetype(String),
    Path(String),
    ClassLevel(i64),
    MythicRank(i64),
}

/// The one owner a row names, out of the owner columns it has.
fn owner(
    table: &str,
    candidates: impl IntoIterator<Item = Option<Owner>>,
) -> Result<Owner, sqlx::Error> {
    let mut set = candidates.into_iter().flatten();
    match (set.next(), set.next()) {
        (Some(owner), None) => Ok(owner),
        _ => Err(corrupt(format!(
            "a {table} row doesn't name exactly one owner"
        ))),
    }
}

/// Rows grouped by owner, each group in query order.
struct Grouped<K, T>(HashMap<K, Vec<T>>);

impl<K: Eq + Hash, T> Grouped<K, T> {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn push(&mut self, owner: K, item: T) {
        self.0.entry(owner).or_default().push(item);
    }

    /// The owner's rows, or none.
    fn take(&mut self, owner: &K) -> Vec<T> {
        self.0.remove(owner).unwrap_or_default()
    }

    /// Every row has been taken by its owner; a row left over belongs to nothing the loader read.
    fn finish(self, table: &str) -> Result<(), sqlx::Error> {
        let left: usize = self.0.values().map(Vec::len).sum();
        if left == 0 {
            Ok(())
        } else {
            Err(corrupt(format!(
                "{left} {table} row(s) belong to nothing the game data has"
            )))
        }
    }
}

/// The owner of a row with `class_key` and a nullable `archetype_key`: the archetype when set.
fn class_owner(class_key: String, archetype_key: Option<String>) -> Owner {
    archetype_key.map_or(Owner::Class(class_key), Owner::Archetype)
}

async fn load_mythic_ascensions(
    conn: &mut PgConnection,
) -> Result<Vec<MythicAscension>, sqlx::Error> {
    sqlx::query!("SELECT rank, character_level FROM mythic_ascension ORDER BY rank")
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|row| {
            Ok(MythicAscension {
                rank: to_u8(row.rank, "mythic_ascension.rank")?,
                character_level: to_u8(row.character_level, "mythic_ascension.character_level")?,
            })
        })
        .collect()
}

async fn load_skills(conn: &mut PgConnection) -> Result<Vec<Skill>, sqlx::Error> {
    sqlx::query!("SELECT key, name, ability FROM skill ORDER BY key")
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Skill {
                key: SkillKey(row.key),
                name: row.name,
                ability: ability_from(&row.ability)?,
            })
        })
        .collect()
}

async fn load_feat_slots(conn: &mut PgConnection) -> Result<Grouped<Owner, FeatSlot>, sqlx::Error> {
    let mut options: Grouped<i64, FeatKey> = Grouped::new();
    for row in sqlx::query!(
        "SELECT feat_slot_id, feat_key FROM feat_slot_option ORDER BY feat_slot_id, feat_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        options.push(row.feat_slot_id, FeatKey(row.feat_key));
    }

    let mut slots = Grouped::new();
    for row in sqlx::query!(
        "SELECT id, race_key, class_level_id, mythic_rank_id, kind, category, ignores_prerequisites
        FROM feat_slot ORDER BY id"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let columns = (
            feat_slot_kind_from(&row.kind)?,
            row.category,
            row.ignores_prerequisites,
        );
        let slot = match columns {
            (FeatSlotKind::General, None, false) => FeatSlot::General,
            (FeatSlotKind::CombatFeat, None, false) => FeatSlot::CombatFeat,
            (FeatSlotKind::Category, Some(category), false) => {
                FeatSlot::Category(feat_category_from(&category)?)
            }
            (FeatSlotKind::OneOf, None, ignores_prerequisites) => FeatSlot::OneOf {
                feats: options.take(&row.id).into_iter().collect(),
                ignores_prerequisites,
            },
            (kind, _, _) => {
                return Err(corrupt(format!(
                    "feat_slot {}: a {kind:?} slot with the wrong columns",
                    row.id
                )));
            }
        };
        let owner = owner(
            "feat_slot",
            [
                row.race_key.map(Owner::Race),
                row.class_level_id.map(Owner::ClassLevel),
                row.mythic_rank_id.map(Owner::MythicRank),
            ],
        )?;
        slots.push(owner, slot);
    }
    options.finish("feat_slot_option")?;
    Ok(slots)
}

async fn load_races(
    conn: &mut PgConnection,
    feat_slots: &mut Grouped<Owner, FeatSlot>,
) -> Result<Vec<Race>, sqlx::Error> {
    let mut modifiers = Grouped::new();
    for row in sqlx::query!(
        "SELECT race_key, subrace_key, ability, amount FROM ability_modifier ORDER BY id"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let amount = to_i8(row.amount, "ability_modifier.amount")?;
        // No ability means the chosen kind.
        let modifier = match row.ability {
            Some(ability) => AbilityModifier::Fixed {
                ability: ability_from(&ability)?,
                amount,
            },
            None => AbilityModifier::Chosen { amount },
        };
        let owner = owner(
            "ability_modifier",
            [
                row.race_key.map(Owner::Race),
                row.subrace_key.map(Owner::Subrace),
            ],
        )?;
        modifiers.push(owner, modifier);
    }

    let mut subraces = Grouped::new();
    for row in sqlx::query!("SELECT key, race_key, name, summary, guide FROM subrace ORDER BY key")
        .fetch_all(&mut *conn)
        .await?
    {
        let ability_modifiers = modifiers.take(&Owner::Subrace(row.key.clone()));
        subraces.push(
            row.race_key,
            Subrace {
                key: SubraceKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
                ability_modifiers,
            },
        );
    }

    let rows = sqlx::query!("SELECT key, name, summary, guide FROM race ORDER BY key")
        .fetch_all(&mut *conn)
        .await?;
    let races = rows
        .into_iter()
        .map(|row| {
            let owner = Owner::Race(row.key.clone());
            Race {
                ability_modifiers: modifiers.take(&owner),
                feat_slots: feat_slots.take(&owner),
                subraces: subraces.take(&row.key),
                key: RaceKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
            }
        })
        .collect();
    modifiers.finish("ability_modifier")?;
    subraces.finish("subrace")?;
    Ok(races)
}

async fn load_feats(conn: &mut PgConnection, trees: &mut Trees) -> Result<Vec<Feat>, sqlx::Error> {
    sqlx::query!(
        "SELECT key, name, category, summary, guide, prerequisite_root_id
        FROM feat ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(Feat {
            key: FeatKey(row.key),
            name: row.name,
            category: feat_category_from(&row.category)?,
            summary: row.summary,
            guide: GuideRef(row.guide),
            prerequisite: trees.get(row.prerequisite_root_id)?,
        })
    })
    .collect()
}

/// Features and selections granted by class levels and mythic ranks.
struct Grants {
    features: Grouped<Owner, FeatureKey>,
    selections: Grouped<Owner, FeatureSelectionKey>,
}

async fn load_grants(conn: &mut PgConnection) -> Result<Grants, sqlx::Error> {
    let mut grants = Grants {
        features: Grouped::new(),
        selections: Grouped::new(),
    };
    for row in sqlx::query!(
        "SELECT class_level_id, mythic_rank_id, feature_key FROM granted_feature ORDER BY feature_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = owner(
            "granted_feature",
            [
                row.class_level_id.map(Owner::ClassLevel),
                row.mythic_rank_id.map(Owner::MythicRank),
            ],
        )?;
        grants.features.push(owner, FeatureKey(row.feature_key));
    }
    for row in sqlx::query!(
        "SELECT class_level_id, mythic_rank_id, selection_key
        FROM granted_selection ORDER BY selection_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = owner(
            "granted_selection",
            [
                row.class_level_id.map(Owner::ClassLevel),
                row.mythic_rank_id.map(Owner::MythicRank),
            ],
        )?;
        grants
            .selections
            .push(owner, FeatureSelectionKey(row.selection_key));
    }
    Ok(grants)
}

/// The level rows of classes (by class) and of archetypes (by archetype).
struct Levels {
    class_rows: Grouped<Owner, ClassLevelRow>,
    archetype_rows: Grouped<Owner, ArchetypeLevelRow>,
}

async fn load_levels(
    conn: &mut PgConnection,
    grants: &mut Grants,
    feat_slots: &mut Grouped<Owner, FeatSlot>,
) -> Result<Levels, sqlx::Error> {
    let mut levels = Levels {
        class_rows: Grouped::new(),
        archetype_rows: Grouped::new(),
    };
    for row in sqlx::query!(
        "SELECT id, class_key, archetype_key, level, arcane_spell_level, divine_spell_level,
            advances_arcane, advances_divine
        FROM class_level ORDER BY level"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let id = Owner::ClassLevel(row.id);
        let level = to_u8(row.level, "class_level.level")?;
        let spell_levels = match (row.arcane_spell_level, row.divine_spell_level) {
            (Some(arcane), Some(divine)) => Some(SpellLevels {
                arcane: to_u8(arcane, "class_level.arcane_spell_level")?,
                divine: to_u8(divine, "class_level.divine_spell_level")?,
            }),
            (None, None) => None,
            _ => {
                return Err(corrupt(format!(
                    "class_level {}: one spell level set",
                    row.id
                )));
            }
        };
        let (features, feat_slots, selections) = (
            grants.features.take(&id),
            feat_slots.take(&id),
            grants.selections.take(&id),
        );
        match (
            row.archetype_key,
            spell_levels,
            row.advances_arcane,
            row.advances_divine,
        ) {
            (None, Some(spells), Some(advances_arcane), Some(advances_divine)) => {
                levels.class_rows.push(
                    Owner::Class(row.class_key),
                    ClassLevelRow {
                        level,
                        arcane_spell_level: spells.arcane,
                        divine_spell_level: spells.divine,
                        advances_arcane,
                        advances_divine,
                        features,
                        feat_slots,
                        selections,
                    },
                );
            }
            (Some(archetype), spell_levels, None, None) => {
                levels.archetype_rows.push(
                    Owner::Archetype(archetype),
                    ArchetypeLevelRow {
                        level,
                        spell_levels,
                        features,
                        feat_slots,
                        selections,
                    },
                );
            }
            _ => {
                return Err(corrupt(format!(
                    "class_level {}: the wrong columns for a class or an archetype row",
                    row.id
                )));
            }
        }
    }
    Ok(levels)
}

async fn load_features(conn: &mut PgConnection) -> Result<Grouped<Owner, Feature>, sqlx::Error> {
    let mut features = Grouped::new();
    for row in sqlx::query!(
        "SELECT key, class_key, archetype_key, path_key, name, summary, guide
        FROM feature ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = owner(
            "feature",
            [
                row.class_key
                    .map(|class| class_owner(class, row.archetype_key)),
                row.path_key.map(Owner::Path),
            ],
        )?;
        features.push(
            owner,
            Feature {
                key: FeatureKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
            },
        );
    }
    Ok(features)
}

async fn load_selections(
    conn: &mut PgConnection,
    trees: &mut Trees,
) -> Result<Grouped<Owner, FeatureSelection>, sqlx::Error> {
    let mut options = Grouped::new();
    for row in sqlx::query!(
        "SELECT key, selection_key, name, summary, guide, feat_key, prerequisite_root_id
        FROM selection_option ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        options.push(
            row.selection_key,
            SelectionOption {
                key: SelectionOptionKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
                feat: row.feat_key.map(FeatKey),
                prerequisite: trees.get(row.prerequisite_root_id)?,
            },
        );
    }

    let mut selections = Grouped::new();
    for row in sqlx::query!(
        "SELECT key, class_key, archetype_key, path_key, name, summary, guide
        FROM feature_selection ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = owner(
            "feature_selection",
            [
                row.class_key
                    .map(|class| class_owner(class, row.archetype_key)),
                row.path_key.map(Owner::Path),
            ],
        )?;
        selections.push(
            owner,
            FeatureSelection {
                options: options.take(&row.key),
                key: FeatureSelectionKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
            },
        );
    }
    options.finish("selection_option")?;
    Ok(selections)
}

async fn load_class_skills(
    conn: &mut PgConnection,
) -> Result<Grouped<Owner, SkillKey>, sqlx::Error> {
    let mut skills = Grouped::new();
    for row in sqlx::query!(
        "SELECT class_key, archetype_key, skill_key FROM class_skill ORDER BY skill_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        skills.push(
            class_owner(row.class_key, row.archetype_key),
            SkillKey(row.skill_key),
        );
    }
    Ok(skills)
}

/// The rows that classes and archetypes own.
struct Parts<'a> {
    features: &'a mut Grouped<Owner, Feature>,
    selections: &'a mut Grouped<Owner, FeatureSelection>,
    class_skills: &'a mut Grouped<Owner, SkillKey>,
    levels: &'a mut Levels,
}

/// Base classes and prestige classes.
async fn load_classes(
    conn: &mut PgConnection,
    trees: &mut Trees,
    parts: &mut Parts<'_>,
) -> Result<(Vec<Class>, Vec<Class>), sqlx::Error> {
    let mut archetypes = Grouped::new();
    for row in sqlx::query!(
        "SELECT key, class_key, name, summary, guide, prerequisite_root_id, skill_points,
            overrides_class_skills
        FROM archetype ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = Owner::Archetype(row.key.clone());
        let class_skills: BTreeSet<SkillKey> =
            parts.class_skills.take(&owner).into_iter().collect();
        if !row.overrides_class_skills && !class_skills.is_empty() {
            return Err(corrupt(format!(
                "{}: class_skill rows under an archetype that keeps its class's class skills",
                row.key
            )));
        }
        archetypes.push(
            row.class_key,
            Archetype {
                key: ArchetypeKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
                prerequisite: trees.get(row.prerequisite_root_id)?,
                skill_points: row
                    .skill_points
                    .map(|points| to_u8(points, "archetype.skill_points"))
                    .transpose()?,
                class_skills: row.overrides_class_skills.then_some(class_skills),
                features: parts.features.take(&owner),
                selections: parts.selections.take(&owner),
                levels: parts.levels.archetype_rows.take(&owner),
            },
        );
    }

    let mut save_exceptions = Grouped::new();
    for row in sqlx::query!(
        "SELECT class_key, level, save, value FROM save_exception ORDER BY class_key, level, save"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        save_exceptions.push(
            row.class_key,
            SaveException {
                level: to_u8(row.level, "save_exception.level")?,
                save: save_from(&row.save)?,
                value: to_u8(row.value, "save_exception.value")?,
            },
        );
    }

    let (mut classes, mut prestige_classes) = (Vec::new(), Vec::new());
    for row in sqlx::query!(
        "SELECT key, is_prestige, name, summary, guide, skill_points, bab, fort, reflex, will,
            prerequisite_root_id
        FROM class ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = Owner::Class(row.key.clone());
        let class = Class {
            archetypes: archetypes.take(&row.key),
            save_exceptions: save_exceptions.take(&row.key),
            bab: bab_progression_from(&row.bab)?,
            fort: save_progression_from(&row.fort)?,
            reflex: save_progression_from(&row.reflex)?,
            will: save_progression_from(&row.will)?,
            key: ClassKey(row.key),
            name: row.name,
            summary: row.summary,
            guide: GuideRef(row.guide),
            skill_points: to_u8(row.skill_points, "class.skill_points")?,
            class_skills: parts.class_skills.take(&owner).into_iter().collect(),
            prerequisite: trees.get(row.prerequisite_root_id)?,
            features: parts.features.take(&owner),
            selections: parts.selections.take(&owner),
            levels: parts.levels.class_rows.take(&owner),
        };
        if row.is_prestige {
            prestige_classes.push(class);
        } else {
            classes.push(class);
        }
    }
    archetypes.finish("archetype")?;
    save_exceptions.finish("save_exception")?;
    Ok((classes, prestige_classes))
}

async fn load_mythic_abilities(
    conn: &mut PgConnection,
    trees: &mut Trees,
) -> Result<Vec<MythicAbility>, sqlx::Error> {
    sqlx::query!(
        "SELECT key, name, group_name, summary, guide, prerequisite_root_id
        FROM mythic_ability ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(MythicAbility {
            key: MythicAbilityKey(row.key),
            name: row.name,
            group: row.group_name,
            summary: row.summary,
            guide: GuideRef(row.guide),
            prerequisite: trees.get(row.prerequisite_root_id)?,
        })
    })
    .collect()
}

async fn load_mythic_feats(
    conn: &mut PgConnection,
    trees: &mut Trees,
) -> Result<Vec<MythicFeat>, sqlx::Error> {
    sqlx::query!(
        "SELECT key, name, group_name, summary, guide, prerequisite_root_id
        FROM mythic_feat ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(MythicFeat {
            key: MythicFeatKey(row.key),
            name: row.name,
            group: row.group_name,
            summary: row.summary,
            guide: GuideRef(row.guide),
            prerequisite: trees.get(row.prerequisite_root_id)?,
        })
    })
    .collect()
}

async fn load_mythic_paths(
    conn: &mut PgConnection,
    features: &mut Grouped<Owner, Feature>,
    selections: &mut Grouped<Owner, FeatureSelection>,
    grants: &mut Grants,
    feat_slots: &mut Grouped<Owner, FeatSlot>,
) -> Result<Vec<MythicPath>, sqlx::Error> {
    let mut kept_ranks = Grouped::new();
    for row in
        sqlx::query!("SELECT path_key, rank FROM path_switch_kept_rank ORDER BY path_key, rank")
            .fetch_all(&mut *conn)
            .await?
    {
        kept_ranks.push(row.path_key, to_u8(row.rank, "path_switch_kept_rank.rank")?);
    }
    let mut switches = Grouped::new();
    for row in sqlx::query!("SELECT path_key, further_ascensions, level_cap FROM path_switch")
        .fetch_all(&mut *conn)
        .await?
    {
        let switch = PathSwitch {
            kept_ranks: kept_ranks.take(&row.path_key).into_iter().collect(),
            further_ascensions: row.further_ascensions,
            level_cap: row
                .level_cap
                .map(|cap| to_u8(cap, "path_switch.level_cap"))
                .transpose()?,
        };
        switches.push(row.path_key, switch);
    }
    kept_ranks.finish("path_switch_kept_rank")?;

    let mut ranks = Grouped::new();
    for row in sqlx::query!("SELECT id, path_key, rank FROM mythic_rank ORDER BY path_key, rank")
        .fetch_all(&mut *conn)
        .await?
    {
        let id = Owner::MythicRank(row.id);
        ranks.push(
            row.path_key,
            MythicRankRow {
                rank: to_u8(row.rank, "mythic_rank.rank")?,
                features: grants.features.take(&id),
                feat_slots: feat_slots.take(&id),
                selections: grants.selections.take(&id),
            },
        );
    }

    let paths = sqlx::query!("SELECT key, name, summary, guide FROM mythic_path ORDER BY key")
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|row| {
            let owner = Owner::Path(row.key.clone());
            MythicPath {
                features: features.take(&owner),
                selections: selections.take(&owner),
                ranks: ranks.take(&row.key),
                switch: switches.take(&row.key).pop(),
                key: MythicPathKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
            }
        })
        .collect();
    switches.finish("path_switch")?;
    ranks.finish("mythic_rank")?;
    Ok(paths)
}
