//! The `item_bonus` and `item_interactions` tables (tickets 119, 120 and 124; migrations
//! `20261109033141_119-caster-line.sql` and `20261109075346_120-124-totals-and-interactions.sql`): [`ItemBonus`] and
//! [`ItemInteractions`] rows in and out, kept apart from `seed.rs` and `load.rs` so a new kind of bonus touches only
//! this file and its migration.

use rules::{
    BackgroundKey, BonusAmount, BonusItem, BonusTarget, FeatKey, FeatureKey, ItemBonus,
    ItemInteractions, MythicAbilityKey, MythicFeatKey, RaceKey, SelectionOptionKey, SkillKey,
    SpellScope, SubraceKey, Subschool,
};
use sqlx::SqliteConnection;

use crate::columns::{
    ability_from, ability_text, corrupt, energy_from, energy_text, save_from, save_text,
    spell_school_from, spell_school_text, to_i8,
};
use crate::prerequisites::{self, Trees};

/// The item columns of a row: race, subrace, background, feat, mythic feat, mythic ability, feature, selection
/// option.
type ItemColumns<'a> = [Option<&'a str>; 8];

fn item_columns(item: &BonusItem) -> ItemColumns<'_> {
    let mut columns = [None; 8];
    let (index, key) = match item {
        BonusItem::Race(key) => (0, key.as_str()),
        BonusItem::Subrace(key) => (1, key.as_str()),
        BonusItem::Background(key) => (2, key.as_str()),
        BonusItem::Feat(key) => (3, key.as_str()),
        BonusItem::MythicFeat(key) => (4, key.as_str()),
        BonusItem::MythicAbility(key) => (5, key.as_str()),
        BonusItem::Feature(key) => (6, key.as_str()),
        BonusItem::SelectionOption(key) => (7, key.as_str()),
    };
    columns[index] = Some(key);
    columns
}

fn item_from(columns: [Option<String>; 8], table: &str) -> Result<BonusItem, sqlx::Error> {
    let mut set = columns
        .into_iter()
        .enumerate()
        .filter_map(|(i, key)| Some((i, key?)));
    let (Some((index, key)), None) = (set.next(), set.next()) else {
        return Err(corrupt(format!("{table} without exactly one item")));
    };
    Ok(match index {
        0 => BonusItem::Race(RaceKey(key)),
        1 => BonusItem::Subrace(SubraceKey(key)),
        2 => BonusItem::Background(BackgroundKey(key)),
        3 => BonusItem::Feat(FeatKey(key)),
        4 => BonusItem::MythicFeat(MythicFeatKey(key)),
        5 => BonusItem::MythicAbility(MythicAbilityKey(key)),
        6 => BonusItem::Feature(FeatureKey(key)),
        _ => BonusItem::SelectionOption(SelectionOptionKey(key)),
    })
}

/// The target columns: `target`, `scope`, `spell_school`, `subschool`, `energy`, `save`, `skill_key`.
struct TargetColumns<'a> {
    target: &'static str,
    scope: Option<&'static str>,
    school: Option<&'static str>,
    subschool: Option<&'static str>,
    energy: Option<&'static str>,
    save: Option<&'static str>,
    skill: Option<&'a str>,
}

fn target_columns(target: &BonusTarget) -> TargetColumns<'_> {
    let plain = |target| TargetColumns {
        target,
        scope: None,
        school: None,
        subschool: None,
        energy: None,
        save: None,
        skill: None,
    };
    match target {
        BonusTarget::SpellPenetration => plain("spell_penetration"),
        BonusTarget::Concentration => plain("concentration"),
        BonusTarget::Initiative => plain("initiative"),
        BonusTarget::AllSaves => plain("all_saves"),
        BonusTarget::AllSkills => plain("all_skills"),
        BonusTarget::Held => plain("held"),
        BonusTarget::Save(save) => TargetColumns {
            save: Some(save_text(*save)),
            ..plain("save")
        },
        BonusTarget::Skill(skill) => TargetColumns {
            skill: Some(skill.as_str()),
            ..plain("skill")
        },
        BonusTarget::SpellDc(scope) => {
            let (scope, school, subschool, energy) = match *scope {
                SpellScope::All => ("all", None, None, None),
                SpellScope::School(school) => {
                    ("school", Some(spell_school_text(school)), None, None)
                }
                SpellScope::Subschool(subschool) => {
                    ("subschool", None, Some(subschool_text(subschool)), None)
                }
                SpellScope::Energy(energy) => ("energy", None, None, Some(energy_text(energy))),
                SpellScope::Metamagic => ("metamagic", None, None, None),
                SpellScope::Value => ("value", None, None, None),
            };
            TargetColumns {
                scope: Some(scope),
                school,
                subschool,
                energy,
                ..plain("spell_dc")
            }
        }
    }
}

fn target_from(
    target: &str,
    scope: Option<&str>,
    school: Option<&str>,
    subschool: Option<&str>,
    energy: Option<&str>,
    save: Option<&str>,
    skill: Option<String>,
) -> Result<BonusTarget, sqlx::Error> {
    if target != "spell_dc" {
        if let (None, None, None, None) = (scope, school, subschool, energy) {
            match (target, save, skill) {
                ("spell_penetration", None, None) => return Ok(BonusTarget::SpellPenetration),
                ("concentration", None, None) => return Ok(BonusTarget::Concentration),
                ("initiative", None, None) => return Ok(BonusTarget::Initiative),
                ("all_saves", None, None) => return Ok(BonusTarget::AllSaves),
                ("all_skills", None, None) => return Ok(BonusTarget::AllSkills),
                ("held", None, None) => return Ok(BonusTarget::Held),
                ("save", Some(save), None) => return Ok(BonusTarget::Save(save_from(save)?)),
                ("skill", None, Some(skill)) => return Ok(BonusTarget::Skill(SkillKey(skill))),
                _ => {}
            }
        }
        return Err(corrupt(format!("item_bonus target {target:?}")));
    }
    if save.is_some() || skill.is_some() {
        return Err(corrupt("item_bonus spell_dc with a save or a skill"));
    }
    let scope = match (target, scope, school, subschool, energy) {
        ("spell_dc", Some("all"), None, None, None) => SpellScope::All,
        ("spell_dc", Some("school"), Some(school), None, None) => {
            SpellScope::School(spell_school_from(school)?)
        }
        ("spell_dc", Some("subschool"), None, Some(subschool), None) => {
            SpellScope::Subschool(subschool_from(subschool)?)
        }
        ("spell_dc", Some("energy"), None, None, Some(energy)) => {
            SpellScope::Energy(energy_from(energy)?)
        }
        ("spell_dc", Some("metamagic"), None, None, None) => SpellScope::Metamagic,
        ("spell_dc", Some("value"), None, None, None) => SpellScope::Value,
        other => return Err(corrupt(format!("item_bonus target {other:?}"))),
    };
    Ok(BonusTarget::SpellDc(scope))
}

/// The amount columns: `amount_kind`, `amount`, `amount_with_ten_ranks`, `ability`.
type AmountColumns = (&'static str, Option<i16>, Option<i16>, Option<&'static str>);

fn amount_columns(amount: BonusAmount) -> AmountColumns {
    match amount {
        BonusAmount::Flat(value) => ("flat", Some(value.into()), None, None),
        BonusAmount::HalfMythicRank => ("half_mythic_rank", None, None, None),
        BonusAmount::MythicRank => ("mythic_rank", None, None, None),
        BonusAmount::SpellFocusOfAnotherSchool => {
            ("spell_focus_of_another_school", None, None, None)
        }
        BonusAmount::TenRanks {
            amount,
            with_ten_ranks,
        } => (
            "ten_ranks",
            Some(amount.into()),
            Some(with_ten_ranks.into()),
            None,
        ),
        BonusAmount::AbilityModifier(ability) => {
            ("ability_modifier", None, None, Some(ability_text(ability)))
        }
    }
}

fn amount_from(
    kind: &str,
    amount: Option<i64>,
    with_ten_ranks: Option<i64>,
    ability: Option<&str>,
) -> Result<BonusAmount, sqlx::Error> {
    let byte = |value| to_i8(value, "item_bonus.amount");
    Ok(match (kind, amount, with_ten_ranks, ability) {
        ("flat", Some(value), None, None) => BonusAmount::Flat(byte(value)?),
        ("half_mythic_rank", None, None, None) => BonusAmount::HalfMythicRank,
        ("mythic_rank", None, None, None) => BonusAmount::MythicRank,
        ("spell_focus_of_another_school", None, None, None) => {
            BonusAmount::SpellFocusOfAnotherSchool
        }
        ("ten_ranks", Some(amount), Some(with_ten_ranks), None) => BonusAmount::TenRanks {
            amount: byte(amount)?,
            with_ten_ranks: byte(with_ten_ranks)?,
        },
        ("ability_modifier", None, None, Some(ability)) => {
            BonusAmount::AbilityModifier(ability_from(ability)?)
        }
        other => return Err(corrupt(format!("item_bonus amount {other:?}"))),
    })
}

fn subschool_text(subschool: Subschool) -> &'static str {
    match subschool {
        Subschool::Calling => "calling",
        Subschool::Creation => "creation",
        Subschool::Healing => "healing",
        Subschool::Summoning => "summoning",
        Subschool::Teleportation => "teleportation",
        Subschool::Scrying => "scrying",
        Subschool::Charm => "charm",
        Subschool::Compulsion => "compulsion",
        Subschool::Figment => "figment",
        Subschool::Glamer => "glamer",
        Subschool::Pattern => "pattern",
        Subschool::Phantasm => "phantasm",
        Subschool::Shadow => "shadow",
        Subschool::Polymorph => "polymorph",
    }
}

fn subschool_from(text: &str) -> Result<Subschool, sqlx::Error> {
    Subschool::ALL
        .into_iter()
        .find(|subschool| subschool_text(*subschool) == text)
        .ok_or_else(|| corrupt(format!("{text:?} is not a Subschool")))
}

/// Writes every bonus, its `requires` as a prerequisite tree.
pub(crate) async fn insert(
    conn: &mut SqliteConnection,
    bonuses: &[ItemBonus],
) -> Result<(), sqlx::Error> {
    for bonus in bonuses {
        let root = prerequisites::insert(conn, bonus.requires.as_ref()).await?;
        let [
            race,
            subrace,
            background,
            feat,
            mythic_feat,
            mythic_ability,
            feature,
            option,
        ] = item_columns(&bonus.item);
        let target = target_columns(&bonus.target);
        let (amount_kind, amount, with_ten_ranks, ability) = amount_columns(bonus.amount);
        sqlx::query!(
            "INSERT INTO item_bonus (race_key, subrace_key, background_key, feat_key, mythic_feat_key,
                mythic_ability_key, feature_key, selection_option_key, target, scope, spell_school, subschool, energy,
                save, skill_key, amount_kind, amount, amount_with_ten_ranks, ability, prerequisite_root_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)",
            race,
            subrace,
            background,
            feat,
            mythic_feat,
            mythic_ability,
            feature,
            option,
            target.target,
            target.scope,
            target.school,
            target.subschool,
            target.energy,
            target.save,
            target.skill,
            amount_kind,
            amount,
            with_ten_ranks,
            ability,
            root,
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Reads every bonus back, in no particular order (the loader makes the data canonical).
pub(crate) async fn load(
    conn: &mut SqliteConnection,
    trees: &mut Trees,
) -> Result<Vec<ItemBonus>, sqlx::Error> {
    sqlx::query!(
        "SELECT race_key, subrace_key, background_key, feat_key, mythic_feat_key, mythic_ability_key, feature_key,
            selection_option_key, target, scope, spell_school, subschool, energy, save, skill_key, amount_kind, amount,
            amount_with_ten_ranks, ability, prerequisite_root_id
        FROM item_bonus ORDER BY id"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ItemBonus {
            item: item_from(
                [
                    row.race_key,
                    row.subrace_key,
                    row.background_key,
                    row.feat_key,
                    row.mythic_feat_key,
                    row.mythic_ability_key,
                    row.feature_key,
                    row.selection_option_key,
                ],
                "item_bonus",
            )?,
            target: target_from(
                &row.target,
                row.scope.as_deref(),
                row.spell_school.as_deref(),
                row.subschool.as_deref(),
                row.energy.as_deref(),
                row.save.as_deref(),
                row.skill_key,
            )?,
            amount: amount_from(
                &row.amount_kind,
                row.amount,
                row.amount_with_ten_ranks,
                row.ability.as_deref(),
            )?,
            requires: trees.get(row.prerequisite_root_id)?,
        })
    })
    .collect()
}

/// Writes every interactions note.
pub(crate) async fn insert_interactions(
    conn: &mut SqliteConnection,
    notes: &[ItemInteractions],
) -> Result<(), sqlx::Error> {
    for note in notes {
        let [
            race,
            subrace,
            background,
            feat,
            mythic_feat,
            mythic_ability,
            feature,
            option,
        ] = item_columns(&note.item);
        sqlx::query!(
            "INSERT INTO item_interactions (race_key, subrace_key, background_key, feat_key, mythic_feat_key,
                mythic_ability_key, feature_key, selection_option_key, interactions)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            race,
            subrace,
            background,
            feat,
            mythic_feat,
            mythic_ability,
            feature,
            option,
            note.interactions,
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Reads every interactions note back, in no particular order.
pub(crate) async fn load_interactions(
    conn: &mut SqliteConnection,
) -> Result<Vec<ItemInteractions>, sqlx::Error> {
    sqlx::query!(
        "SELECT race_key, subrace_key, background_key, feat_key, mythic_feat_key, mythic_ability_key, feature_key,
            selection_option_key, interactions
        FROM item_interactions ORDER BY id"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ItemInteractions {
            item: item_from(
                [
                    row.race_key,
                    row.subrace_key,
                    row.background_key,
                    row.feat_key,
                    row.mythic_feat_key,
                    row.mythic_ability_key,
                    row.feature_key,
                    row.selection_option_key,
                ],
                "item_interactions",
            )?,
            interactions: row.interactions,
        })
    })
    .collect()
}
