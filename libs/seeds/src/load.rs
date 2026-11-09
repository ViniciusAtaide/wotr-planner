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
    AbilityBonus, AbilityModifier, Archetype, ArchetypeKey, ArchetypeLevelRow, Background,
    BackgroundKey, Class, ClassKey, ClassLevelRow, CountsAs, CountsToward, Deity, DeityKey, Dice,
    Feat, FeatKey, FeatList, FeatListKey, FeatSlot, FeatValue, Feature, FeatureKey,
    FeatureSelection, FeatureSelectionKey, GameData, GuideRef, MythicAbility, MythicAbilityKey,
    MythicAscension, MythicFeat, MythicFeatKey, MythicPath, MythicPathKey, MythicRankRow,
    OptionLevelRow, OptionRow, PathSwitch, Progression, Race, RaceKey, SaveException,
    SelectionOption, SelectionOptionKey, Skill, SkillKey, SpellLevels, Subrace, SubraceKey, Weapon,
    WeaponKey, WeaponStats,
};
use sqlx::SqliteConnection;

use crate::columns::{
    AbilityBonusKind, FeatSlotKind, ability_bonus_kind_from, ability_from, advancement_from,
    alignment_from, bab_progression_from, background_group_from, corrupt, deity_domains_from,
    feat_category_from, feat_slot_kind_from, handedness_from, level_from_from,
    mythic_ability_group_from, mythic_extra_pick_from, mythic_feat_group_from, option_group_from,
    parameter_kind_from, ranged_from, repeat_from, save_from, save_progression_from, to_i8, to_u8,
    value_from, weapon_proficiency_from,
};
use crate::prerequisites::Trees;
use crate::{Database, Error};

/// Reads the seeded game data, in canonical order. Unseeded tables read as empty game data.
pub async fn load_game_data(db: &Database) -> Result<GameData, Error> {
    Ok(load(db).await?)
}

async fn load(db: &Database) -> Result<GameData, sqlx::Error> {
    // One snapshot. The seed renames a new file over the database rather than writing to it, so nothing
    // changes under the reads below anyway, but the transaction keeps them consistent if something does.
    let mut tx = db.pool().begin().await?;
    let conn: &mut SqliteConnection = &mut tx;

    let mut trees = Trees::load(conn).await?;
    let progression = Progression {
        mythic_ascensions: load_mythic_ascensions(conn).await?,
    };
    let skills = load_skills(conn).await?;
    let weapons = load_weapons(conn).await?;
    let mut feat_slots = load_feat_slots(conn).await?;
    let mut feat_lists = load_feat_lists(conn).await?;
    let races = load_races(conn, &mut feat_slots, &mut feat_lists).await?;
    let feats = load_feats(conn, &mut trees).await?;
    let mut grants = load_grants(conn).await?;
    let mut levels = load_levels(conn, &mut grants, &mut feat_slots).await?;
    let mut features = load_features(conn).await?;
    let mut class_skills = load_class_skills(conn).await?;
    let mut selections = load_selections(
        conn,
        &mut trees,
        &mut grants,
        &mut feat_slots,
        &mut class_skills,
    )
    .await?;
    let (classes, prestige_classes) = load_classes(
        conn,
        &mut trees,
        &mut Parts {
            features: &mut features,
            selections: &mut selections,
            feat_lists: &mut feat_lists,
            class_skills: &mut class_skills,
            levels: &mut levels,
        },
    )
    .await?;
    let mythic_abilities = load_mythic_abilities(conn, &mut trees, &mut grants).await?;
    let mythic_feats = load_mythic_feats(conn, &mut trees).await?;
    let backgrounds = load_backgrounds(conn, &mut grants, &mut class_skills).await?;
    let deities = load_deities(conn).await?;
    let mythic_paths = load_mythic_paths(
        conn,
        &mut Parts {
            features: &mut features,
            selections: &mut selections,
            feat_lists: &mut feat_lists,
            class_skills: &mut class_skills,
            levels: &mut levels,
        },
        &mut grants,
        &mut feat_slots,
    )
    .await?;
    feat_slots.finish("feat_slot")?;
    grants.features.finish("granted_feature")?;
    grants.selections.finish("granted_selection")?;
    grants.feats.finish("granted_feat")?;
    grants.feats_for.finish("granted_feat")?;
    levels.class_rows.finish("class_level")?;
    levels.archetype_rows.finish("class_level")?;
    features.finish("feature")?;
    selections.finish("feature_selection")?;
    feat_lists.finish("feat_list")?;
    class_skills.finish("class_skill")?;
    trees.finish()?;

    tx.commit().await?;
    let data = GameData {
        progression,
        races,
        skills,
        weapons,
        feats,
        classes,
        prestige_classes,
        mythic_abilities,
        mythic_feats,
        mythic_paths,
        backgrounds,
        deities,
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
    OptionLevel(i64),
    MythicAbility(String),
    Background(String),
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
    conn: &mut SqliteConnection,
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

async fn load_weapons(conn: &mut SqliteConnection) -> Result<Vec<Weapon>, sqlx::Error> {
    sqlx::query!(
        "SELECT key, name, proficiency, damage_count, damage_sides, threat, multiplier, handedness, ranged,
            finesse AS \"finesse: bool\"
        FROM weapon ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        // The migration keeps the stats all set or all NULL.
        let stats = match (
            row.damage_count,
            row.damage_sides,
            row.threat,
            row.multiplier,
            row.handedness,
            row.finesse,
        ) {
            (Some(count), Some(sides), Some(threat), Some(multiplier), Some(handedness), Some(finesse)) => {
                Some(WeaponStats {
                    damage: Dice {
                        count: to_u8(count, "weapon.damage_count")?,
                        sides: to_u8(sides, "weapon.damage_sides")?,
                    },
                    threat: to_u8(threat, "weapon.threat")?,
                    multiplier: to_u8(multiplier, "weapon.multiplier")?,
                    handedness: handedness_from(&handedness)?,
                    ranged: row.ranged.as_deref().map(ranged_from).transpose()?,
                    finesse,
                })
            }
            _ => None,
        };
        Ok(Weapon {
            key: WeaponKey(row.key),
            name: row.name,
            proficiency: row
                .proficiency
                .as_deref()
                .map(weapon_proficiency_from)
                .transpose()?,
            stats,
        })
    })
    .collect()
}

async fn load_skills(conn: &mut SqliteConnection) -> Result<Vec<Skill>, sqlx::Error> {
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

async fn load_feat_slots(
    conn: &mut SqliteConnection,
) -> Result<Grouped<Owner, FeatSlot>, sqlx::Error> {
    let mut slots = Grouped::new();
    for row in sqlx::query!(
        "SELECT id, race_key, subrace_key, class_level_id, mythic_rank_id, option_level_id, kind,
            category, feat_list_key, selection_key, at_level, label
        FROM feat_slot ORDER BY id"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let columns = (
            feat_slot_kind_from(&row.kind)?,
            row.category,
            row.feat_list_key,
            row.selection_key,
        );
        let slot = match columns {
            (FeatSlotKind::General, None, None, None) => FeatSlot::General,
            (FeatSlotKind::CombatFeat, None, None, None) => FeatSlot::CombatFeat,
            (FeatSlotKind::Category, Some(category), None, None) => {
                FeatSlot::Category(feat_category_from(&category)?)
            }
            (FeatSlotKind::OneOf, None, Some(list), None) => FeatSlot::OneOf {
                list: FeatListKey(list),
            },
            (FeatSlotKind::OptionList, None, None, Some(selection)) => FeatSlot::OptionList {
                selection: FeatureSelectionKey(selection),
                at_level: row
                    .at_level
                    .map(|level| to_u8(level, "feat_slot.at_level"))
                    .transpose()?,
                label: row.label,
            },
            (kind, _, _, _) => {
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
                row.subrace_key.map(Owner::Subrace),
                row.class_level_id.map(Owner::ClassLevel),
                row.mythic_rank_id.map(Owner::MythicRank),
                row.option_level_id.map(Owner::OptionLevel),
            ],
        )?;
        slots.push(owner, slot);
    }
    Ok(slots)
}

async fn load_races(
    conn: &mut SqliteConnection,
    feat_slots: &mut Grouped<Owner, FeatSlot>,
    feat_lists: &mut Grouped<Owner, FeatList>,
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
        let owner = Owner::Subrace(row.key.clone());
        subraces.push(
            row.race_key,
            Subrace {
                key: SubraceKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
                ability_modifiers: modifiers.take(&owner),
                feat_slots: feat_slots.take(&owner),
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
                feat_lists: feat_lists.take(&owner),
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

async fn load_feats(
    conn: &mut SqliteConnection,
    trees: &mut Trees,
) -> Result<Vec<Feat>, sqlx::Error> {
    let mut extra_picks = Grouped::new();
    for row in sqlx::query!(
        "SELECT feat_key, selection_key FROM feat_extra_pick ORDER BY feat_key, selection_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        extra_picks.push(row.feat_key, FeatureSelectionKey(row.selection_key));
    }
    let feats = sqlx::query!(
        "SELECT key, name, category, summary, guide, prerequisite_root_id, parameter, repeat,
            repeat_times
        FROM feat ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(Feat {
            extra_pick_of: taken_set(&mut extra_picks, &row.key),
            key: FeatKey(row.key),
            name: row.name,
            category: feat_category_from(&row.category)?,
            summary: row.summary,
            guide: GuideRef(row.guide),
            parameter: row
                .parameter
                .as_deref()
                .map(parameter_kind_from)
                .transpose()?,
            repeat: repeat_from(&row.repeat, row.repeat_times)?,
            prerequisite: trees.get(row.prerequisite_root_id)?,
        })
    })
    .collect::<Result<Vec<Feat>, sqlx::Error>>()?;
    extra_picks.finish("feat_extra_pick")?;
    Ok(feats)
}

/// Features, selections and feats granted by class levels, mythic ranks and backgrounds. A counted grant comes back as
/// that many entries. A feat granted for one value is in `feats_for`.
struct Grants {
    features: Grouped<Owner, FeatureKey>,
    selections: Grouped<Owner, FeatureSelectionKey>,
    feats: Grouped<Owner, FeatKey>,
    feats_for: Grouped<Owner, FeatValue>,
}

async fn load_grants(conn: &mut SqliteConnection) -> Result<Grants, sqlx::Error> {
    let mut grants = Grants {
        features: Grouped::new(),
        selections: Grouped::new(),
        feats: Grouped::new(),
        feats_for: Grouped::new(),
    };
    for row in sqlx::query!(
        "SELECT class_level_id, mythic_rank_id, option_level_id, mythic_ability_key, feature_key,
            count
        FROM granted_feature ORDER BY feature_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = owner(
            "granted_feature",
            [
                row.class_level_id.map(Owner::ClassLevel),
                row.mythic_rank_id.map(Owner::MythicRank),
                row.option_level_id.map(Owner::OptionLevel),
                row.mythic_ability_key.map(Owner::MythicAbility),
            ],
        )?;
        for _ in 0..to_u8(row.count, "granted_feature.count")? {
            grants
                .features
                .push(owner.clone(), FeatureKey(row.feature_key.clone()));
        }
    }
    for row in sqlx::query!(
        "SELECT class_level_id, mythic_rank_id, option_level_id, mythic_ability_key, selection_key,
            count
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
                row.option_level_id.map(Owner::OptionLevel),
                row.mythic_ability_key.map(Owner::MythicAbility),
            ],
        )?;
        for _ in 0..to_u8(row.count, "granted_selection.count")? {
            grants.selections.push(
                owner.clone(),
                FeatureSelectionKey(row.selection_key.clone()),
            );
        }
    }
    for row in sqlx::query!(
        "SELECT class_level_id, option_level_id, background_key, feat_key, skill_key, weapon_key,
            spell_school, energy, armour_type
        FROM granted_feat ORDER BY feat_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = owner(
            "granted_feat",
            [
                row.class_level_id.map(Owner::ClassLevel),
                row.option_level_id.map(Owner::OptionLevel),
                row.background_key.map(Owner::Background),
            ],
        )?;
        let feat = FeatKey(row.feat_key);
        match value_from(
            row.skill_key,
            row.weapon_key,
            row.spell_school,
            row.energy,
            row.armour_type,
        )? {
            None => grants.feats.push(owner, feat),
            Some(value) => grants.feats_for.push(owner, FeatValue { feat, value }),
        }
    }
    Ok(grants)
}

/// The level rows of classes (by class) and of archetypes (by archetype).
struct Levels {
    class_rows: Grouped<Owner, ClassLevelRow>,
    archetype_rows: Grouped<Owner, ArchetypeLevelRow>,
}

async fn load_levels(
    conn: &mut SqliteConnection,
    grants: &mut Grants,
    feat_slots: &mut Grouped<Owner, FeatSlot>,
) -> Result<Levels, sqlx::Error> {
    let mut levels = Levels {
        class_rows: Grouped::new(),
        archetype_rows: Grouped::new(),
    };
    for row in sqlx::query!(
        "SELECT id, class_key, archetype_key, level, arcane_spell_level, divine_spell_level,
            advances_arcane AS \"advances_arcane: bool\", advances_divine AS \"advances_divine: bool\"
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
        let (features, feats, feats_for, feat_slots, selections) = (
            grants.features.take(&id),
            grants.feats.take(&id).into_iter().collect(),
            grants.feats_for.take(&id).into_iter().collect(),
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
                        feats,
                        feats_for,
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
                        feats,
                        feats_for,
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

async fn load_features(
    conn: &mut SqliteConnection,
) -> Result<Grouped<Owner, Feature>, sqlx::Error> {
    let mut bonuses = Grouped::new();
    for row in sqlx::query!(
        "SELECT feature_key, kind, ability, amount
        FROM feature_ability_bonus ORDER BY feature_key, kind, ability"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let amount = to_u8(row.amount, "feature_ability_bonus.amount")?;
        let bonus = match (ability_bonus_kind_from(&row.kind)?, row.ability) {
            (AbilityBonusKind::Fixed, Some(ability)) => AbilityBonus::Fixed {
                ability: ability_from(&ability)?,
                amount,
            },
            (AbilityBonusKind::Every, None) => AbilityBonus::Every { amount },
            (AbilityBonusKind::Highest, None) => AbilityBonus::Highest { amount },
            (AbilityBonusKind::AtLeast, None) => AbilityBonus::AtLeast { score: amount },
            _ => {
                return Err(corrupt(
                    "a feature_ability_bonus row sets an ability on a kind other than fixed",
                ));
            }
        };
        bonuses.push(row.feature_key, bonus);
    }

    let mut features = Grouped::new();
    for row in sqlx::query!(
        "SELECT key, class_key, archetype_key, path_key, name, summary, guide,
            option_row_selection_key, option_row_level, daily_choice AS \"daily_choice: bool\", bab_floor,
            skill_ranks_at_least_mythic_rank AS \"skill_ranks_at_least_mythic_rank: bool\",
            favoured_weapon_proficiency AS \"favoured_weapon_proficiency: bool\"
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
                ability_bonuses: bonuses.take(&row.key),
                key: FeatureKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
                option_row: match (row.option_row_selection_key, row.option_row_level) {
                    (Some(selection), Some(level)) => Some(OptionRow {
                        selection: FeatureSelectionKey(selection),
                        level: to_u8(level, "feature.option_row_level")?,
                    }),
                    (None, None) => None,
                    _ => return Err(corrupt("a feature with half an option row")),
                },
                daily_choice: row.daily_choice,
                bab_floor: row
                    .bab_floor
                    .map(|floor| to_u8(floor, "feature.bab_floor"))
                    .transpose()?,
                skill_ranks_at_least_mythic_rank: row.skill_ranks_at_least_mythic_rank,
                favoured_weapon_proficiency: row.favoured_weapon_proficiency,
            },
        );
    }
    bonuses.finish("feature_ability_bonus")?;
    Ok(features)
}

async fn load_selections(
    conn: &mut SqliteConnection,
    trees: &mut Trees,
    grants: &mut Grants,
    feat_slots: &mut Grouped<Owner, FeatSlot>,
    class_skills: &mut Grouped<Owner, SkillKey>,
) -> Result<Grouped<Owner, FeatureSelection>, sqlx::Error> {
    let mut chosen_feats = Grouped::new();
    for row in sqlx::query!(
        "SELECT option_level_id, feat_key FROM option_level_chosen_feat
        ORDER BY option_level_id, feat_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        chosen_feats.push(
            Owner::OptionLevel(row.option_level_id),
            FeatKey(row.feat_key),
        );
    }
    let mut option_rows = Grouped::new();
    for row in sqlx::query!(
        "SELECT id, option_key, level, feat_list_key FROM option_level ORDER BY option_key, level"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let id = Owner::OptionLevel(row.id);
        option_rows.push(
            row.option_key,
            OptionLevelRow {
                level: to_u8(row.level, "option_level.level")?,
                features: grants.features.take(&id),
                feats: grants.feats.take(&id).into_iter().collect(),
                chosen_feats: chosen_feats.take(&id).into_iter().collect(),
                feat_slots: feat_slots.take(&id),
                selections: grants.selections.take(&id),
                feat_list: row.feat_list_key.map(FeatListKey),
                class_skills: class_skills.take(&id).into_iter().collect(),
            },
        );
    }

    let mut option_feats = Grouped::new();
    for row in sqlx::query!(
        "SELECT option_key, feat_key FROM selection_option_feat ORDER BY option_key, feat_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        option_feats.push(row.option_key, FeatKey(row.feat_key));
    }

    let mut option_values = Grouped::new();
    for row in sqlx::query!(
        "SELECT option_key, skill_key, weapon_key, spell_school, energy, armour_type
        FROM selection_option_value ORDER BY option_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let value = value_from(
            row.skill_key,
            row.weapon_key,
            row.spell_school,
            row.energy,
            row.armour_type,
        )?
        .ok_or_else(|| corrupt("a selection_option_value row with no value"))?;
        option_values.push(row.option_key, value);
    }

    let mut options = Grouped::new();
    for row in sqlx::query!(
        "SELECT key, selection_key, name, summary, guide, prerequisite_root_id, chosen_feat_key,
            parameter, repeat, repeat_times, pool_key, replaces_row_of_key, option_group, arcana_key,
            domain_key
        FROM selection_option ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        options.push(
            row.selection_key,
            SelectionOption {
                feats: option_feats.take(&row.key).into_iter().collect(),
                chosen_feat: row.chosen_feat_key.map(FeatKey),
                parameter: row
                    .parameter
                    .as_deref()
                    .map(parameter_kind_from)
                    .transpose()?,
                values: option_values.take(&row.key).into_iter().collect(),
                repeat: repeat_from(&row.repeat, row.repeat_times)?,
                levels: option_levels_of(&mut option_rows, &row.key),
                key: SelectionOptionKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
                prerequisite: trees.get(row.prerequisite_root_id)?,
                pool: row.pool_key.map(FeatureSelectionKey),
                replaces_row_of: row.replaces_row_of_key.map(FeatureSelectionKey),
                group: row
                    .option_group
                    .as_deref()
                    .map(option_group_from)
                    .transpose()?,
                arcana: row.arcana_key.map(FeatureKey),
                domain: row.domain_key.map(SelectionOptionKey),
            },
        );
    }

    let mut shared = Grouped::new();
    for row in sqlx::query!(
        "SELECT selection_key, option_key FROM selection_shared_option
        ORDER BY selection_key, option_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        shared.push(row.selection_key, SelectionOptionKey(row.option_key));
    }

    let mut level_classes = Grouped::new();
    for row in sqlx::query!(
        "SELECT selection_key, class_key FROM selection_level_class ORDER BY selection_key, class_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        level_classes.push(row.selection_key, ClassKey(row.class_key));
    }
    let mut skipped = Grouped::new();
    for row in sqlx::query!(
        "SELECT selection_key, level FROM selection_skipped_level ORDER BY selection_key, level"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        skipped.push(
            row.selection_key,
            to_u8(row.level, "selection_skipped_level.level")?,
        );
    }

    let mut selections = Grouped::new();
    for row in sqlx::query!(
        "SELECT key, class_key, archetype_key, path_key, name, summary, guide,
            feat_lists_only AS \"feat_lists_only: bool\", only_without_option AS \"only_without_option: bool\",
            pool_of_key, levels_behind, level_from, level_also_from_key,
            once_per_option AS \"once_per_option: bool\", deity_domains
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
                shared_options: shared.take(&row.key).into_iter().collect(),
                option_levels: taken_set(&mut level_classes, &row.key),
                skipped_levels: taken_set(&mut skipped, &row.key),
                key: FeatureSelectionKey(row.key),
                name: row.name,
                summary: row.summary,
                guide: GuideRef(row.guide),
                feat_lists_only: row.feat_lists_only,
                only_without_option: row.only_without_option,
                pool_of: row.pool_of_key.map(FeatureSelectionKey),
                levels_behind: to_u8(row.levels_behind, "feature_selection.levels_behind")?,
                level_from: row.level_from.as_deref().map(level_from_from).transpose()?,
                once_per_option: row.once_per_option,
                level_also_from: row.level_also_from_key.map(SelectionOptionKey),
                deity_domains: row
                    .deity_domains
                    .as_deref()
                    .map(deity_domains_from)
                    .transpose()?,
            },
        );
    }
    options.finish("selection_option")?;
    option_values.finish("selection_option_value")?;
    option_feats.finish("selection_option_feat")?;
    option_rows.finish("option_level")?;
    chosen_feats.finish("option_level_chosen_feat")?;
    level_classes.finish("selection_level_class")?;
    skipped.finish("selection_skipped_level")?;
    shared.finish("selection_shared_option")?;
    Ok(selections)
}

async fn load_feat_lists(
    conn: &mut SqliteConnection,
) -> Result<Grouped<Owner, FeatList>, sqlx::Error> {
    let mut members = Grouped::new();
    for row in sqlx::query!(
        "SELECT feat_list_key, feat_key FROM feat_list_member ORDER BY feat_list_key, feat_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        members.push(row.feat_list_key, FeatKey(row.feat_key));
    }

    let mut lists = Grouped::new();
    for row in sqlx::query!(
        "SELECT key, class_key, archetype_key, path_key, race_key, name, guide,
            ignores_prerequisites AS \"ignores_prerequisites: bool\", every_feat AS \"every_feat: bool\"
        FROM feat_list ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = owner(
            "feat_list",
            [
                row.class_key
                    .map(|class| class_owner(class, row.archetype_key)),
                row.path_key.map(Owner::Path),
                row.race_key.map(Owner::Race),
            ],
        )?;
        lists.push(
            owner,
            FeatList {
                feats: members.take(&row.key).into_iter().collect(),
                key: FeatListKey(row.key),
                name: row.name,
                guide: GuideRef(row.guide),
                ignores_prerequisites: row.ignores_prerequisites,
                every_feat: row.every_feat,
            },
        );
    }
    members.finish("feat_list_member")?;
    Ok(lists)
}

async fn load_backgrounds(
    conn: &mut SqliteConnection,
    grants: &mut Grants,
    class_skills: &mut Grouped<Owner, SkillKey>,
) -> Result<Vec<Background>, sqlx::Error> {
    sqlx::query!("SELECT key, name, background_group, note FROM background ORDER BY key")
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|row| {
            let owner = Owner::Background(row.key.clone());
            Ok(Background {
                key: BackgroundKey(row.key),
                name: row.name,
                group: row
                    .background_group
                    .as_deref()
                    .map(background_group_from)
                    .transpose()?,
                note: row.note,
                class_skills: class_skills.take(&owner).into_iter().collect(),
                feats: grants.feats.take(&owner).into_iter().collect(),
                feats_for: grants.feats_for.take(&owner).into_iter().collect(),
            })
        })
        .collect()
}

async fn load_deities(conn: &mut SqliteConnection) -> Result<Vec<Deity>, sqlx::Error> {
    let mut alignments = Grouped::new();
    for row in sqlx::query!(
        "SELECT deity_key, alignment FROM deity_alignment ORDER BY deity_key, alignment"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        alignments.push(row.deity_key, alignment_from(&row.alignment)?);
    }
    let mut weapons = Grouped::new();
    for row in sqlx::query!(
        "SELECT deity_key, weapon_key FROM deity_weapon ORDER BY deity_key, weapon_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        weapons.push(row.deity_key, WeaponKey(row.weapon_key));
    }
    let mut domains = Grouped::new();
    for row in sqlx::query!(
        "SELECT deity_key, domain_key FROM deity_domain ORDER BY deity_key, domain_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        domains.push(row.deity_key, SelectionOptionKey(row.domain_key));
    }
    let deities = sqlx::query!("SELECT key, name, guide FROM deity ORDER BY key")
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|row| Deity {
            alignments: taken_set(&mut alignments, &row.key),
            favoured_weapons: taken_set(&mut weapons, &row.key),
            domains: taken_set(&mut domains, &row.key),
            key: DeityKey(row.key),
            name: row.name,
            guide: GuideRef(row.guide),
        })
        .collect();
    alignments.finish("deity_alignment")?;
    weapons.finish("deity_weapon")?;
    domains.finish("deity_domain")?;
    Ok(deities)
}

async fn load_class_skills(
    conn: &mut SqliteConnection,
) -> Result<Grouped<Owner, SkillKey>, sqlx::Error> {
    let mut skills = Grouped::new();
    for row in sqlx::query!(
        "SELECT class_key, archetype_key, background_key, option_level_id, skill_key FROM class_skill
        ORDER BY skill_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let owner = owner(
            "class_skill",
            [
                row.class_key
                    .map(|class_key| class_owner(class_key, row.archetype_key)),
                row.background_key.map(Owner::Background),
                row.option_level_id.map(Owner::OptionLevel),
            ],
        )?;
        skills.push(owner, SkillKey(row.skill_key));
    }
    Ok(skills)
}

/// The rows that classes, archetypes and paths own.
struct Parts<'a> {
    features: &'a mut Grouped<Owner, Feature>,
    selections: &'a mut Grouped<Owner, FeatureSelection>,
    feat_lists: &'a mut Grouped<Owner, FeatList>,
    class_skills: &'a mut Grouped<Owner, SkillKey>,
    levels: &'a mut Levels,
}

/// Base classes and prestige classes.
async fn load_classes(
    conn: &mut SqliteConnection,
    trees: &mut Trees,
    parts: &mut Parts<'_>,
) -> Result<(Vec<Class>, Vec<Class>), sqlx::Error> {
    let mut archetypes = Grouped::new();
    for row in sqlx::query!(
        "SELECT key, class_key, name, summary, guide, prerequisite_root_id, skill_points,
            overrides_class_skills AS \"overrides_class_skills: bool\", spontaneous AS \"spontaneous: bool\",
            fort, reflex, will
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
                spontaneous: row.spontaneous,
                fort: row.fort.as_deref().map(save_progression_from).transpose()?,
                reflex: row.reflex.as_deref().map(save_progression_from).transpose()?,
                will: row.will.as_deref().map(save_progression_from).transpose()?,
                features: parts.features.take(&owner),
                selections: parts.selections.take(&owner),
                feat_lists: parts.feat_lists.take(&owner),
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

    let mut counts_as = Grouped::new();
    for row in sqlx::query!(
        "SELECT class_key, as_class_key, toward, feature_key FROM class_counts_as
        ORDER BY class_key, as_class_key, toward, feature_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let toward = match (row.toward.as_str(), row.feature_key) {
            ("feat_prerequisites", None) => CountsToward::FeatPrerequisites,
            ("feature", Some(feature)) => CountsToward::Feature(FeatureKey(feature)),
            (toward, feature) => {
                return Err(corrupt(format!(
                    "class_counts_as toward {toward:?} with feature {feature:?}"
                )));
            }
        };
        counts_as.push(
            row.class_key,
            CountsAs {
                class: ClassKey(row.as_class_key),
                toward,
            },
        );
    }

    let (mut classes, mut prestige_classes) = (Vec::new(), Vec::new());
    for row in sqlx::query!(
        "SELECT key, is_prestige AS \"is_prestige: bool\", name, summary, guide, skill_points, bab, fort,
            reflex, will, advances, spontaneous AS \"spontaneous: bool\", prerequisite_root_id
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
            advances: advancement_from(&row.advances)?,
            spontaneous: row.spontaneous,
            counts_as: counts_as.take(&row.key).into_iter().collect(),
            key: ClassKey(row.key),
            name: row.name,
            summary: row.summary,
            guide: GuideRef(row.guide),
            skill_points: to_u8(row.skill_points, "class.skill_points")?,
            class_skills: parts.class_skills.take(&owner).into_iter().collect(),
            prerequisite: trees.get(row.prerequisite_root_id)?,
            features: parts.features.take(&owner),
            selections: parts.selections.take(&owner),
            feat_lists: parts.feat_lists.take(&owner),
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
    counts_as.finish("class_counts_as")?;
    Ok((classes, prestige_classes))
}

async fn load_mythic_abilities(
    conn: &mut SqliteConnection,
    trees: &mut Trees,
    grants: &mut Grants,
) -> Result<Vec<MythicAbility>, sqlx::Error> {
    sqlx::query!(
        "SELECT key, name, group_name, summary, guide, prerequisite_root_id
        FROM mythic_ability ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        let owner = Owner::MythicAbility(row.key.clone());
        Ok(MythicAbility {
            features: grants.features.take(&owner),
            selections: grants.selections.take(&owner),
            key: MythicAbilityKey(row.key),
            name: row.name,
            group: mythic_ability_group_from(&row.group_name)?,
            summary: row.summary,
            guide: GuideRef(row.guide),
            prerequisite: trees.get(row.prerequisite_root_id)?,
        })
    })
    .collect()
}

async fn load_mythic_feats(
    conn: &mut SqliteConnection,
    trees: &mut Trees,
) -> Result<Vec<MythicFeat>, sqlx::Error> {
    sqlx::query!(
        "SELECT key, name, group_name, summary, guide, prerequisite_root_id, parameter, repeat,
            repeat_times, removes_racial_penalty AS \"removes_racial_penalty: bool\", extra_pick
        FROM mythic_feat ORDER BY key"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(MythicFeat {
            key: MythicFeatKey(row.key),
            name: row.name,
            group: mythic_feat_group_from(&row.group_name)?,
            summary: row.summary,
            guide: GuideRef(row.guide),
            parameter: row
                .parameter
                .as_deref()
                .map(parameter_kind_from)
                .transpose()?,
            repeat: repeat_from(&row.repeat, row.repeat_times)?,
            removes_racial_penalty: row.removes_racial_penalty,
            extra_pick: row
                .extra_pick
                .as_deref()
                .map(mythic_extra_pick_from)
                .transpose()?,
            prerequisite: trees.get(row.prerequisite_root_id)?,
        })
    })
    .collect()
}

async fn load_mythic_paths(
    conn: &mut SqliteConnection,
    parts: &mut Parts<'_>,
    grants: &mut Grants,
    feat_slots: &mut Grouped<Owner, FeatSlot>,
) -> Result<Vec<MythicPath>, sqlx::Error> {
    let mut from = Grouped::new();
    for row in sqlx::query!(
        "SELECT path_key, from_path_key FROM path_switch_from ORDER BY path_key, from_path_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        from.push(row.path_key, MythicPathKey(row.from_path_key));
    }
    let (mut dropped_features, mut dropped_selections) = (Grouped::new(), Grouped::new());
    for row in sqlx::query!(
        "SELECT path_key, feature_key, selection_key FROM path_switch_drop
        ORDER BY path_key, feature_key, selection_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        match (row.feature_key, row.selection_key) {
            (Some(feature), None) => dropped_features.push(row.path_key, FeatureKey(feature)),
            (None, Some(selection)) => {
                dropped_selections.push(row.path_key, FeatureSelectionKey(selection));
            }
            _ => {
                return Err(corrupt(
                    "a path_switch_drop row doesn't name exactly one feature or selection",
                ));
            }
        }
    }
    let mut switch_features = Grouped::new();
    for row in sqlx::query!(
        "SELECT path_key, feature_key FROM path_switch_feature ORDER BY path_key, feature_key"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        switch_features.push(row.path_key, FeatureKey(row.feature_key));
    }
    let mut switches = Grouped::new();
    for row in sqlx::query!(
        "SELECT path_key, kept_through_rank, path_kept_through_rank,
            further_ascensions AS \"further_ascensions: bool\", level_cap
        FROM path_switch"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let switch = PathSwitch {
            from: from.take(&row.path_key).into_iter().collect(),
            kept_through_rank: to_u8(row.kept_through_rank, "path_switch.kept_through_rank")?,
            path_kept_through_rank: to_u8(
                row.path_kept_through_rank,
                "path_switch.path_kept_through_rank",
            )?,
            dropped_features: dropped_features.take(&row.path_key).into_iter().collect(),
            dropped_selections: dropped_selections.take(&row.path_key).into_iter().collect(),
            features: switch_features.take(&row.path_key).into_iter().collect(),
            further_ascensions: row.further_ascensions,
            level_cap: row
                .level_cap
                .map(|cap| to_u8(cap, "path_switch.level_cap"))
                .transpose()?,
        };
        switches.push(row.path_key, switch);
    }
    from.finish("path_switch_from")?;
    dropped_features.finish("path_switch_drop")?;
    dropped_selections.finish("path_switch_drop")?;
    switch_features.finish("path_switch_feature")?;

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
                features: parts.features.take(&owner),
                selections: parts.selections.take(&owner),
                feat_lists: parts.feat_lists.take(&owner),
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

/// An option's rows, taken from the rows grouped by option.
fn option_levels_of(
    rows: &mut Grouped<String, OptionLevelRow>,
    option: &str,
) -> Vec<OptionLevelRow> {
    rows.take(&option.to_owned())
}

/// An owner's set-like rows (a selection's, a feat's), taken from the rows grouped by its key.
fn taken_set<T: Ord>(rows: &mut Grouped<String, T>, owner: &str) -> BTreeSet<T> {
    rows.take(&owner.to_owned()).into_iter().collect()
}
