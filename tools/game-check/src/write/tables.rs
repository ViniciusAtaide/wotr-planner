//! The class-tables target (ticket 293g): each class's `bab`, `fort`, `reflex` and `will` as the formula that gives the
//! game's table, its `save_exceptions` kept only for a save no formula gives, and each archetype's overrides against
//! its class's target. Skill points and class skills are 293h's.
//!
//! A field is written only where the check would see a difference: the effective value of the planner's class
//! ([`Class::base_attack_bonus`], [`Class::save_bonus`]) against the game's table at each of its levels. A person's
//! order and an equal-but-redundant field stay. A table no formula gives is left, listed as unexpressed, and its
//! finding stays.

use std::fmt::Debug;

use rules::{Archetype, BabProgression, Class, DataAt, Save, SaveException, SaveProgression};
use serde::Serialize;

use super::span::{Value, ron_text};
use super::{Change, Unexpressed, entry_file};
use crate::areas::Ctx;
use crate::areas::class_tables::{SAVES, bonuses};
use crate::report::{Area, Out};

/// The BAB progression whose formula gives `table[1..=levels]` exactly.
pub fn bab_progression(table: &[i64], levels: u8) -> Option<BabProgression> {
    [
        BabProgression::Full,
        BabProgression::ThreeQuarters,
        BabProgression::Half,
    ]
    .into_iter()
    .find(|each| gives(table, levels, |level| each.bab(level)))
}

/// The save progression whose formula gives `table[1..=levels]` exactly (`prestige` picks the prestige formulas).
pub fn save_progression(table: &[i64], levels: u8, prestige: bool) -> Option<SaveProgression> {
    [SaveProgression::Good, SaveProgression::Poor]
        .into_iter()
        .find(|each| gives(table, levels, |level| each.save(level, prestige)))
}

fn gives(table: &[i64], levels: u8, formula: impl Fn(u8) -> u8) -> bool {
    (1..=levels).all(|level| table.get(usize::from(level)) == Some(&i64::from(formula(level))))
}

/// What one table needs.
enum Need<P> {
    /// The planner's effective value is the game's.
    Nothing,
    /// The game's table, as a formula.
    Formula(P),
    /// No table, or no formula gives it.
    Unexpressed(String),
}

fn need<P>(
    table: Option<&[i64]>,
    levels: u8,
    effective: impl Fn(u8) -> u8,
    formula: impl Fn(&[i64]) -> Option<P>,
    what: &str,
) -> Need<P> {
    let Some(table) = table else {
        return Need::Unexpressed(format!("{what}: the game names no table"));
    };
    if gives(table, levels, effective) {
        return Need::Nothing;
    }
    match formula(table) {
        Some(found) => Need::Formula(found),
        None => Need::Unexpressed(format!("{what}: no formula gives the game's table")),
    }
}

fn one<T: Serialize>(value: &T) -> Value {
    Value::One(ron_text(value))
}

fn exceptions_text(list: &[SaveException]) -> String {
    if list.is_empty() {
        return "none".to_owned();
    }
    let each: Vec<String> = list
        .iter()
        .map(|each| format!("{:?} {} at level {}", each.save, each.value, each.level))
        .collect();
    each.join("; ")
}

/// The changes that bring every class's and archetype's BAB and saves to the game's tables, classes first in key
/// order and then archetypes in key order, and what no formula gives.
pub(crate) fn changes(ctx: &Ctx) -> (Vec<Change>, Vec<Unexpressed>) {
    let data = ctx.planner.data;
    let mut changes = Vec::new();
    let mut archetype_changes: Vec<(String, Vec<Change>)> = Vec::new();
    let mut unexpressed = Vec::new();
    let mut classes: Vec<&Class> = data.all_classes().collect();
    classes.sort_by(|a, b| a.key.cmp(&b.key));
    for class in classes {
        // The check's skips are its own business.
        let mut skipped = Out::default();
        let Some((_, json)) = ctx.blueprint(
            &mut skipped,
            Area::ClassTables,
            class.blueprint.as_ref(),
            "class",
        ) else {
            continue;
        };
        let prestige = data
            .prestige_classes
            .iter()
            .any(|each| each.key == class.key);
        let levels = class.levels.len() as u8;
        let at = DataAt::Class(class.key.clone());
        let file = entry_file(data, &at);
        let mut target = class.clone();
        let mut push = |field: &'static str,
                        kind: &'static str,
                        value: Option<Value>,
                        old: String,
                        new: String| {
            changes.push(Change {
                at: at.clone(),
                file: file.clone(),
                field,
                kind,
                value,
                old,
                new,
            });
        };

        let table = bonuses(ctx.dump, json, "m_BaseAttackBonus");
        match need(
            table.as_deref(),
            levels,
            |level| class.base_attack_bonus(level, None),
            |table| bab_progression(table, levels),
            "BAB",
        ) {
            Need::Nothing => {}
            Need::Formula(found) => {
                if found != class.bab {
                    push(
                        "bab",
                        "BAB differs",
                        Some(one(&found)),
                        format!("{:?}", class.bab),
                        format!("{found:?}"),
                    );
                    target.bab = found;
                }
            }
            Need::Unexpressed(what) => unexpressed.push(Unexpressed {
                at: at.clone(),
                what,
            }),
        }

        for (save, field, kind) in SAVES {
            let table = bonuses(ctx.dump, json, field);
            let name = format!("{save:?}");
            match need(
                table.as_deref(),
                levels,
                |level| class.save_bonus(save, level, prestige, None),
                |table| save_progression(table, levels, prestige),
                &name,
            ) {
                Need::Nothing => {}
                Need::Formula(found) => {
                    let current = class.save_progression(save);
                    if found != current {
                        push(
                            save_field(save),
                            kind,
                            Some(one(&found)),
                            format!("{current:?}"),
                            format!("{found:?}"),
                        );
                        set_save(&mut target, save, found);
                    }
                    let kept: Vec<SaveException> = target
                        .save_exceptions
                        .iter()
                        .filter(|each| each.save != save)
                        .cloned()
                        .collect();
                    if kept != target.save_exceptions {
                        let value = (!kept.is_empty())
                            .then(|| Value::List(kept.iter().map(ron_text).collect()));
                        push(
                            "save_exceptions",
                            kind,
                            value,
                            exceptions_text(&target.save_exceptions),
                            exceptions_text(&kept),
                        );
                        target.save_exceptions = kept;
                    }
                }
                Need::Unexpressed(what) => unexpressed.push(Unexpressed {
                    at: at.clone(),
                    what,
                }),
            }
        }

        for archetype in data.archetypes_of(&class.key) {
            let mut skipped = Out::default();
            let Some((_, archetype_json)) = ctx.blueprint(
                &mut skipped,
                Area::ClassTables,
                archetype.blueprint.as_ref(),
                "archetype",
            ) else {
                continue;
            };
            let (own, left) = archetype_changes_of(
                ctx,
                &target,
                archetype,
                json,
                archetype_json,
                levels,
                entry_file(data, &DataAt::Archetype(archetype.key.clone())),
            );
            unexpressed.extend(left);
            archetype_changes.push((archetype.key.to_string(), own));
        }
    }
    archetype_changes.sort_by(|a, b| a.0.cmp(&b.0));
    changes.extend(archetype_changes.into_iter().flat_map(|(_, own)| own));
    (changes, unexpressed)
}

fn save_field(save: Save) -> &'static str {
    match save {
        Save::Fortitude => "fort",
        Save::Reflex => "reflex",
        Save::Will => "will",
    }
}

fn set_save(class: &mut Class, save: Save, value: SaveProgression) {
    match save {
        Save::Fortitude => class.fort = value,
        Save::Reflex => class.reflex = value,
        Save::Will => class.will = value,
    }
}

fn show<P: Debug>(value: Option<P>) -> String {
    value.map_or_else(|| "the class's".to_owned(), |each| format!("{each:?}"))
}

/// One archetype's changes against `class` as it will be written. An override is `Some` only where the game's
/// table is not the class's, and a field that holds the class's own value is removed (the data check refuses it).
fn archetype_changes_of(
    ctx: &Ctx,
    class: &Class,
    archetype: &Archetype,
    class_json: &serde_json::Value,
    json: &serde_json::Value,
    levels: u8,
    file: std::path::PathBuf,
) -> (Vec<Change>, Vec<Unexpressed>) {
    let at = DataAt::Archetype(archetype.key.clone());
    let mut changes = Vec::new();
    let mut unexpressed = Vec::new();
    let table_json = |field: &str| {
        if crate::dump::reference(&json[field]).is_some() {
            json
        } else {
            class_json
        }
    };
    let mut push = |field: &'static str,
                    kind: &'static str,
                    value: Option<Value>,
                    old: String,
                    new: String| {
        changes.push(Change {
            at: at.clone(),
            file: file.clone(),
            field,
            kind,
            value,
            old,
            new,
        });
    };

    let table = bonuses(
        ctx.dump,
        table_json("m_BaseAttackBonus"),
        "m_BaseAttackBonus",
    );
    let wanted = match need(
        table.as_deref(),
        levels,
        |level| class.base_attack_bonus(level, Some(archetype)),
        |table| bab_progression(table, levels),
        "BAB",
    ) {
        Need::Nothing => archetype.bab,
        Need::Formula(found) => Some(found),
        Need::Unexpressed(what) => {
            unexpressed.push(Unexpressed {
                at: at.clone(),
                what,
            });
            archetype.bab
        }
    };
    let wanted = wanted.filter(|found| *found != class.bab);
    if wanted != archetype.bab {
        push(
            "bab",
            "BAB differs",
            wanted.as_ref().map(one),
            show(archetype.bab),
            show(wanted),
        );
    }

    for (save, field, kind) in SAVES {
        let table = bonuses(ctx.dump, table_json(field), field);
        let current = archetype.save_progression(save);
        let wanted = match need(
            table.as_deref(),
            levels,
            |level| class.save_bonus(save, level, false, Some(archetype)),
            |table| save_progression(table, levels, false),
            &format!("{save:?}"),
        ) {
            Need::Nothing => current,
            Need::Formula(found) => Some(found),
            Need::Unexpressed(what) => {
                unexpressed.push(Unexpressed {
                    at: at.clone(),
                    what,
                });
                current
            }
        };
        let wanted = wanted.filter(|found| *found != class.save_progression(save));
        if wanted != current {
            push(
                save_field(save),
                kind,
                wanted.as_ref().map(one),
                show(current),
                show(wanted),
            );
        }
    }
    (changes, unexpressed)
}
