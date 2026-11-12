//! The `gear_item`, `gear_bonus`, `gear_always_on_effect` and `gear_other_effect` tables (ticket 165; migration
//! `20261111180000_165-gear.sql`, and `20261112092431_176-gear-always-on.sql` for the always-on lines): [`GearItem`] rows in and out, kept apart from `seed.rs` and `load.rs`.

use std::collections::BTreeMap;

use rules::{
    BonusType, GearBonus, GearItem, GearKey, GearSlot, GearTarget, GuideRef, SkillKey, SpellScope,
};
use sqlx::SqliteConnection;

use crate::bonuses::{subschool_from, subschool_text};
use crate::columns::{
    ability_from, ability_text, corrupt, energy_from, energy_text, save_from, save_text,
    spell_school_from, spell_school_text, to_i8,
};

fn slot_text(slot: GearSlot) -> &'static str {
    match slot {
        GearSlot::Belt => "belt",
        GearSlot::Boots => "boots",
        GearSlot::Bracers => "bracers",
        GearSlot::Cloak => "cloak",
        GearSlot::Glasses => "glasses",
        GearSlot::Gloves => "gloves",
        GearSlot::Headwear => "headwear",
        GearSlot::Necklace => "necklace",
        GearSlot::Ring => "ring",
        GearSlot::Robe => "robe",
        GearSlot::Armour => "armour",
        GearSlot::Weapon => "weapon",
        GearSlot::Usable => "usable",
    }
}

fn slot_from(text: &str) -> Result<GearSlot, sqlx::Error> {
    GearSlot::ALL
        .into_iter()
        .find(|slot| slot_text(*slot) == text)
        .ok_or_else(|| corrupt(format!("{text:?} is not a GearSlot")))
}

fn type_text(kind: BonusType) -> &'static str {
    match kind {
        BonusType::Alchemical => "alchemical",
        BonusType::Armour => "armour",
        BonusType::Circumstance => "circumstance",
        BonusType::Competence => "competence",
        BonusType::Deflection => "deflection",
        BonusType::Dodge => "dodge",
        BonusType::Enhancement => "enhancement",
        BonusType::Inherent => "inherent",
        BonusType::Insight => "insight",
        BonusType::Luck => "luck",
        BonusType::Morale => "morale",
        BonusType::NaturalArmour => "natural_armour",
        BonusType::Profane => "profane",
        BonusType::Racial => "racial",
        BonusType::Resistance => "resistance",
        BonusType::Sacred => "sacred",
        BonusType::Shield => "shield",
        BonusType::Size => "size",
        BonusType::Untyped => "untyped",
    }
}

fn type_from(text: &str) -> Result<BonusType, sqlx::Error> {
    BonusType::ALL
        .into_iter()
        .find(|kind| type_text(*kind) == text)
        .ok_or_else(|| corrupt(format!("{text:?} is not a BonusType")))
}

/// The target columns: `target`, `ability`, `save`, `skill_key`, `scope`, `spell_school`, `subschool`, `energy`.
#[derive(Default)]
struct TargetColumns<'a> {
    target: &'static str,
    ability: Option<&'static str>,
    save: Option<&'static str>,
    skill: Option<&'a str>,
    scope: Option<&'static str>,
    school: Option<&'static str>,
    subschool: Option<&'static str>,
    energy: Option<&'static str>,
}

fn target_columns(target: &GearTarget) -> Result<TargetColumns<'_>, sqlx::Error> {
    let plain = |target| TargetColumns {
        target,
        ..TargetColumns::default()
    };
    Ok(match target {
        GearTarget::Ability(ability) => TargetColumns {
            ability: Some(ability_text(*ability)),
            ..plain("ability")
        },
        GearTarget::AllSaves => plain("all_saves"),
        GearTarget::Save(save) => TargetColumns {
            save: Some(save_text(*save)),
            ..plain("save")
        },
        GearTarget::Initiative => plain("initiative"),
        GearTarget::Skill(skill) => TargetColumns {
            skill: Some(skill.as_str()),
            ..plain("skill")
        },
        GearTarget::SpellDc(scope) => {
            let mut columns = plain("spell_dc");
            match *scope {
                SpellScope::All => columns.scope = Some("all"),
                SpellScope::School(school) => {
                    columns.scope = Some("school");
                    columns.school = Some(spell_school_text(school));
                }
                SpellScope::Subschool(subschool) => {
                    columns.scope = Some("subschool");
                    columns.subschool = Some(subschool_text(subschool));
                }
                SpellScope::Energy(energy) => {
                    columns.scope = Some("energy");
                    columns.energy = Some(energy_text(energy));
                }
                SpellScope::Metamagic => columns.scope = Some("metamagic"),
                SpellScope::Value => return Err(corrupt("a gear spell DC for a value")),
            }
            columns
        }
        GearTarget::CasterLevel => plain("caster_level"),
        GearTarget::SpellPenetration => plain("spell_penetration"),
        GearTarget::Concentration => plain("concentration"),
    })
}

/// One `gear_bonus` row as read.
struct BonusRow {
    target: String,
    ability: Option<String>,
    save: Option<String>,
    skill_key: Option<String>,
    scope: Option<String>,
    spell_school: Option<String>,
    subschool: Option<String>,
    energy: Option<String>,
}

fn target_from(row: BonusRow) -> Result<GearTarget, sqlx::Error> {
    let BonusRow {
        target,
        ability,
        save,
        skill_key,
        scope,
        spell_school,
        subschool,
        energy,
    } = row;
    let (ability, save, skill, scope, school, subschool, energy) = (
        ability.as_deref(),
        save.as_deref(),
        skill_key,
        scope.as_deref(),
        spell_school.as_deref(),
        subschool.as_deref(),
        energy.as_deref(),
    );
    Ok(match (target.as_str(), ability, save, skill, scope) {
        ("ability", Some(ability), None, None, None) => GearTarget::Ability(ability_from(ability)?),
        ("save", None, Some(save), None, None) => GearTarget::Save(save_from(save)?),
        ("skill", None, None, Some(skill), None) => GearTarget::Skill(SkillKey(skill)),
        ("spell_dc", None, None, None, Some(scope)) => {
            GearTarget::SpellDc(match (scope, school, subschool, energy) {
                ("all", None, None, None) => SpellScope::All,
                ("school", Some(school), None, None) => {
                    SpellScope::School(spell_school_from(school)?)
                }
                ("subschool", None, Some(subschool), None) => {
                    SpellScope::Subschool(subschool_from(subschool)?)
                }
                ("energy", None, None, Some(energy)) => SpellScope::Energy(energy_from(energy)?),
                ("metamagic", None, None, None) => SpellScope::Metamagic,
                other => return Err(corrupt(format!("gear_bonus spell_dc {other:?}"))),
            })
        }
        (plain, None, None, None, None) => match plain {
            "all_saves" => GearTarget::AllSaves,
            "initiative" => GearTarget::Initiative,
            "caster_level" => GearTarget::CasterLevel,
            "spell_penetration" => GearTarget::SpellPenetration,
            "concentration" => GearTarget::Concentration,
            other => return Err(corrupt(format!("gear_bonus target {other:?}"))),
        },
        other => return Err(corrupt(format!("gear_bonus target {other:?}"))),
    })
}

/// Writes every item, its bonuses, its always-on lines and its other-effect lines.
pub(crate) async fn insert(
    conn: &mut SqliteConnection,
    items: &[GearItem],
) -> Result<(), sqlx::Error> {
    for item in items {
        sqlx::query!(
            "INSERT INTO gear_item (key, name, slot, guide) VALUES ($1, $2, $3, $4)",
            item.key.as_str(),
            item.name,
            slot_text(item.slot),
            item.guide.as_ref().map(|guide| guide.0.as_str()),
        )
        .execute(&mut *conn)
        .await?;
        for bonus in &item.bonuses {
            let target = target_columns(&bonus.target)?;
            sqlx::query!(
                "INSERT INTO gear_bonus (gear_item_key, target, ability, save, skill_key, scope, spell_school,
                    subschool, energy, amount, bonus_type)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                item.key.as_str(),
                target.target,
                target.ability,
                target.save,
                target.skill,
                target.scope,
                target.school,
                target.subschool,
                target.energy,
                bonus.amount,
                type_text(bonus.kind),
            )
            .execute(&mut *conn)
            .await?;
        }
        for line in &item.always_on {
            sqlx::query!(
                "INSERT INTO gear_always_on_effect (gear_item_key, text) VALUES ($1, $2)",
                item.key.as_str(),
                line,
            )
            .execute(&mut *conn)
            .await?;
        }
        for line in &item.other_effects {
            sqlx::query!(
                "INSERT INTO gear_other_effect (gear_item_key, text) VALUES ($1, $2)",
                item.key.as_str(),
                line,
            )
            .execute(&mut *conn)
            .await?;
        }
    }
    Ok(())
}

/// Reads every item back, in no particular order (the loader makes the data canonical).
pub(crate) async fn load(conn: &mut SqliteConnection) -> Result<Vec<GearItem>, sqlx::Error> {
    let mut items: BTreeMap<String, GearItem> = BTreeMap::new();
    for row in sqlx::query!("SELECT key, name, slot, guide FROM gear_item ORDER BY key")
        .fetch_all(&mut *conn)
        .await?
    {
        let item = GearItem {
            key: GearKey(row.key.clone()),
            name: row.name,
            slot: slot_from(&row.slot)?,
            guide: row.guide.map(GuideRef),
            bonuses: Vec::new(),
            always_on: Vec::new(),
            other_effects: Vec::new(),
        };
        items.insert(row.key, item);
    }
    for row in sqlx::query!(
        "SELECT gear_item_key, target, ability, save, skill_key, scope, spell_school, subschool, energy, amount,
            bonus_type
        FROM gear_bonus ORDER BY id"
    )
    .fetch_all(&mut *conn)
    .await?
    {
        let item = items
            .get_mut(&row.gear_item_key)
            .ok_or_else(|| corrupt("gear_bonus of no item"))?;
        item.bonuses.push(GearBonus {
            target: target_from(BonusRow {
                target: row.target,
                ability: row.ability,
                save: row.save,
                skill_key: row.skill_key,
                scope: row.scope,
                spell_school: row.spell_school,
                subschool: row.subschool,
                energy: row.energy,
            })?,
            amount: to_i8(row.amount, "gear_bonus.amount")?,
            kind: type_from(&row.bonus_type)?,
        });
    }
    for row in sqlx::query!("SELECT gear_item_key, text FROM gear_always_on_effect ORDER BY id")
        .fetch_all(&mut *conn)
        .await?
    {
        items
            .get_mut(&row.gear_item_key)
            .ok_or_else(|| corrupt("gear_always_on_effect of no item"))?
            .always_on
            .push(row.text);
    }
    for row in sqlx::query!("SELECT gear_item_key, text FROM gear_other_effect ORDER BY id")
        .fetch_all(&mut *conn)
        .await?
    {
        items
            .get_mut(&row.gear_item_key)
            .ok_or_else(|| corrupt("gear_other_effect of no item"))?
            .other_effects
            .push(row.text);
    }
    Ok(items.into_values().collect())
}
