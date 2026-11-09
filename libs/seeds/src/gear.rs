//! The `gear_item` and `gear_bonus` tables (ticket 122; migration `20261109091222_122-assumed-gear.sql`): the guide's
//! caster level and spell DC items ([`GearItem`]) in and out, kept apart from `seed.rs` and `load.rs` as
//! `bonuses.rs` is.

use std::collections::BTreeMap;

use rules::{ClassKey, GearBonus, GearCircumstance, GearItem, GearKey, GuideRef, SpellScope};
use sqlx::SqliteConnection;

use crate::columns::{
    corrupt, energy_from, energy_text, gear_slot_from, gear_slot_text, gear_target_from,
    gear_target_text, save_from, save_text, spell_school_from, spell_school_text, to_i8,
};

/// The circumstance columns: `circumstance`, `scope`, `spell_school`, `subschool`, `energy`, `save`, `class_key`.
#[derive(Default)]
struct CircumstanceColumns<'a> {
    circumstance: &'static str,
    scope: Option<&'static str>,
    school: Option<&'static str>,
    subschool: Option<&'static str>,
    energy: Option<&'static str>,
    save: Option<&'static str>,
    class: Option<&'a str>,
}

fn circumstance_columns(circumstance: &GearCircumstance) -> CircumstanceColumns<'_> {
    match circumstance {
        GearCircumstance::Spells(scope) => {
            let (scope, school, subschool, energy) = match *scope {
                SpellScope::All => ("all", None, None, None),
                SpellScope::School(school) => {
                    ("school", Some(spell_school_text(school)), None, None)
                }
                SpellScope::Subschool(subschool) => (
                    "subschool",
                    None,
                    Some(crate::bonuses::subschool_text(subschool)),
                    None,
                ),
                SpellScope::Energy(energy) => ("energy", None, None, Some(energy_text(energy))),
                SpellScope::Metamagic => ("metamagic", None, None, None),
                // `GameData::check` refuses it; the table's CHECK too.
                SpellScope::Value => ("value", None, None, None),
            };
            CircumstanceColumns {
                circumstance: "spells",
                scope: Some(scope),
                school,
                subschool,
                energy,
                ..CircumstanceColumns::default()
            }
        }
        GearCircumstance::Save(save) => CircumstanceColumns {
            circumstance: "save",
            save: Some(save_text(*save)),
            ..CircumstanceColumns::default()
        },
        GearCircumstance::MindAffecting => CircumstanceColumns {
            circumstance: "mind_affecting",
            ..CircumstanceColumns::default()
        },
        GearCircumstance::Class(class) => CircumstanceColumns {
            circumstance: "class",
            class: Some(class.as_str()),
            ..CircumstanceColumns::default()
        },
    }
}

fn circumstance_from(
    circumstance: &str,
    scope: Option<&str>,
    school: Option<&str>,
    subschool: Option<&str>,
    energy: Option<&str>,
    save: Option<&str>,
    class: Option<String>,
) -> Result<GearCircumstance, sqlx::Error> {
    let spell = (scope, school, subschool, energy);
    Ok(match (circumstance, spell, save, class) {
        ("save", (None, None, None, None), Some(save), None) => {
            GearCircumstance::Save(save_from(save)?)
        }
        ("mind_affecting", (None, None, None, None), None, None) => GearCircumstance::MindAffecting,
        ("class", (None, None, None, None), None, Some(class)) => {
            GearCircumstance::Class(ClassKey(class))
        }
        ("spells", spell, None, None) => GearCircumstance::Spells(match spell {
            (Some("all"), None, None, None) => SpellScope::All,
            (Some("school"), Some(school), None, None) => {
                SpellScope::School(spell_school_from(school)?)
            }
            (Some("subschool"), None, Some(subschool), None) => {
                SpellScope::Subschool(crate::bonuses::subschool_from(subschool)?)
            }
            (Some("energy"), None, None, Some(energy)) => SpellScope::Energy(energy_from(energy)?),
            (Some("metamagic"), None, None, None) => SpellScope::Metamagic,
            other => return Err(corrupt(format!("gear_bonus scope {other:?}"))),
        }),
        (other, ..) => return Err(corrupt(format!("gear_bonus circumstance {other:?}"))),
    })
}

/// Writes every item and its bonuses.
pub(crate) async fn insert(
    conn: &mut SqliteConnection,
    items: &[GearItem],
) -> Result<(), sqlx::Error> {
    for item in items {
        let slot = gear_slot_text(item.slot);
        sqlx::query!(
            "INSERT INTO gear_item (key, name, slot, guide) VALUES ($1, $2, $3, $4)",
            item.key.0,
            item.name,
            slot,
            item.guide.0,
        )
        .execute(&mut *conn)
        .await?;
        for bonus in &item.bonuses {
            let target = gear_target_text(bonus.target);
            let amount = i16::from(bonus.amount);
            let columns = circumstance_columns(&bonus.circumstance);
            sqlx::query!(
                "INSERT INTO gear_bonus (gear_item_key, target, amount, circumstance, scope, spell_school, subschool,
                    energy, save, class_key)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                item.key.0,
                target,
                amount,
                columns.circumstance,
                columns.scope,
                columns.school,
                columns.subschool,
                columns.energy,
                columns.save,
                columns.class,
            )
            .execute(&mut *conn)
            .await?;
        }
    }
    Ok(())
}

/// Reads every item back with its bonuses, in no particular order (the loader makes the data canonical).
pub(crate) async fn load(conn: &mut SqliteConnection) -> Result<Vec<GearItem>, sqlx::Error> {
    let mut bonuses: BTreeMap<String, Vec<GearBonus>> = BTreeMap::new();
    for row in sqlx::query!(
        "SELECT gear_item_key, target, amount, circumstance, scope, spell_school, subschool, energy, save, class_key
        FROM gear_bonus ORDER BY id"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        bonuses
            .entry(row.gear_item_key)
            .or_default()
            .push(GearBonus {
                target: gear_target_from(&row.target)?,
                amount: to_i8(row.amount, "gear_bonus.amount")?,
                circumstance: circumstance_from(
                    &row.circumstance,
                    row.scope.as_deref(),
                    row.spell_school.as_deref(),
                    row.subschool.as_deref(),
                    row.energy.as_deref(),
                    row.save.as_deref(),
                    row.class_key,
                )?,
            });
    }
    let items = sqlx::query!("SELECT key, name, slot, guide FROM gear_item ORDER BY key")
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|row| {
            Ok(GearItem {
                bonuses: bonuses.remove(&row.key).unwrap_or_default(),
                key: GearKey(row.key),
                name: row.name,
                slot: gear_slot_from(&row.slot)?,
                guide: GuideRef(row.guide),
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    if let Some(key) = bonuses.keys().next() {
        return Err(corrupt(format!("gear_bonus for no gear_item: {key}")));
    }
    Ok(items)
}
