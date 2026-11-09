//! The `item_bonus` table (ticket 119, `migrations/20261109033141_119-caster-line.sql`): [`ItemBonus`] rows in and
//! out, kept apart from `seed.rs` and `load.rs` so a new kind of bonus touches only this file and its migration.

use rules::{
    BonusAmount, BonusItem, BonusTarget, FeatKey, FeatureKey, ItemBonus, MythicFeatKey, RaceKey,
    SelectionOptionKey, SpellScope, SubraceKey, Subschool,
};
use sqlx::SqliteConnection;

use crate::columns::{
    corrupt, energy_from, energy_text, spell_school_from, spell_school_text, to_i8,
};
use crate::prerequisites::{self, Trees};

/// The item columns of a row: race, subrace, feat, mythic feat, feature, selection option.
type ItemColumns<'a> = [Option<&'a str>; 6];

fn item_columns(item: &BonusItem) -> ItemColumns<'_> {
    let mut columns = [None; 6];
    let (index, key) = match item {
        BonusItem::Race(key) => (0, key.as_str()),
        BonusItem::Subrace(key) => (1, key.as_str()),
        BonusItem::Feat(key) => (2, key.as_str()),
        BonusItem::MythicFeat(key) => (3, key.as_str()),
        BonusItem::Feature(key) => (4, key.as_str()),
        BonusItem::SelectionOption(key) => (5, key.as_str()),
    };
    columns[index] = Some(key);
    columns
}

fn item_from(columns: [Option<String>; 6]) -> Result<BonusItem, sqlx::Error> {
    let mut set = columns
        .into_iter()
        .enumerate()
        .filter_map(|(i, key)| Some((i, key?)));
    let (Some((index, key)), None) = (set.next(), set.next()) else {
        return Err(corrupt("item_bonus without exactly one item"));
    };
    Ok(match index {
        0 => BonusItem::Race(RaceKey(key)),
        1 => BonusItem::Subrace(SubraceKey(key)),
        2 => BonusItem::Feat(FeatKey(key)),
        3 => BonusItem::MythicFeat(MythicFeatKey(key)),
        4 => BonusItem::Feature(FeatureKey(key)),
        _ => BonusItem::SelectionOption(SelectionOptionKey(key)),
    })
}

/// The target columns: `target`, `scope`, `spell_school`, `subschool`, `energy`.
type TargetColumns = (
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
);

fn target_columns(target: &BonusTarget) -> TargetColumns {
    match target {
        BonusTarget::SpellPenetration => ("spell_penetration", None, None, None, None),
        BonusTarget::Concentration => ("concentration", None, None, None, None),
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
            ("spell_dc", Some(scope), school, subschool, energy)
        }
    }
}

fn target_from(
    target: &str,
    scope: Option<&str>,
    school: Option<&str>,
    subschool: Option<&str>,
    energy: Option<&str>,
) -> Result<BonusTarget, sqlx::Error> {
    let scope = match (target, scope, school, subschool, energy) {
        ("spell_penetration", None, None, None, None) => return Ok(BonusTarget::SpellPenetration),
        ("concentration", None, None, None, None) => return Ok(BonusTarget::Concentration),
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

fn amount_columns(amount: BonusAmount) -> (&'static str, Option<i16>) {
    match amount {
        BonusAmount::Flat(value) => ("flat", Some(value.into())),
        BonusAmount::HalfMythicRank => ("half_mythic_rank", None),
        BonusAmount::MythicRank => ("mythic_rank", None),
        BonusAmount::SpellFocusOfAnotherSchool => ("spell_focus_of_another_school", None),
    }
}

fn amount_from(kind: &str, amount: Option<i64>) -> Result<BonusAmount, sqlx::Error> {
    Ok(match (kind, amount) {
        ("flat", Some(value)) => BonusAmount::Flat(to_i8(value, "item_bonus.amount")?),
        ("half_mythic_rank", None) => BonusAmount::HalfMythicRank,
        ("mythic_rank", None) => BonusAmount::MythicRank,
        ("spell_focus_of_another_school", None) => BonusAmount::SpellFocusOfAnotherSchool,
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
        let [race, subrace, feat, mythic_feat, feature, option] = item_columns(&bonus.item);
        let (target, scope, school, subschool, energy) = target_columns(&bonus.target);
        let (amount_kind, amount) = amount_columns(bonus.amount);
        sqlx::query!(
            "INSERT INTO item_bonus (race_key, subrace_key, feat_key, mythic_feat_key, feature_key,
                selection_option_key, target, scope, spell_school, subschool, energy, amount_kind, amount,
                prerequisite_root_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
            race,
            subrace,
            feat,
            mythic_feat,
            feature,
            option,
            target,
            scope,
            school,
            subschool,
            energy,
            amount_kind,
            amount,
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
        "SELECT race_key, subrace_key, feat_key, mythic_feat_key, feature_key, selection_option_key, target,
            scope, spell_school, subschool, energy, amount_kind, amount, prerequisite_root_id
        FROM item_bonus ORDER BY id"
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ItemBonus {
            item: item_from([
                row.race_key,
                row.subrace_key,
                row.feat_key,
                row.mythic_feat_key,
                row.feature_key,
                row.selection_option_key,
            ])?,
            target: target_from(
                &row.target,
                row.scope.as_deref(),
                row.spell_school.as_deref(),
                row.subschool.as_deref(),
                row.energy.as_deref(),
            )?,
            amount: amount_from(&row.amount_kind, row.amount)?,
            requires: trees.get(row.prerequisite_root_id)?,
        })
    })
    .collect()
}
