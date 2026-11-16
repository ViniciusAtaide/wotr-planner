//! Class tables: skill points, class skills, and the BAB and saves at each level, for classes, prestige classes and
//! archetypes. BAB and saves are compared by value (`BlueprintStatProgression.Bonuses[level]`), so a table the planner
//! doesn't name still compares. The hit die isn't: the planner has none.
//!
//! Skill points and class skills are among the user's facts (the map's Notes): their kinds say so, since a difference
//! there goes to the user before the data changes.

use std::collections::BTreeSet;

use rules::{Archetype, BlueprintGuid, Class, DataAt, Save};
use serde_json::Value;

use super::{Ctx, skill_stat};
use crate::dump::{flag, int, reference};
use crate::report::{Area, Out};

const AREA: Area = Area::ClassTables;

pub fn compare(ctx: &Ctx, out: &mut Out) {
    let data = ctx.planner.data;
    for class in data.all_classes() {
        let prestige = data
            .prestige_classes
            .iter()
            .any(|each| each.key == class.key);
        let Some((guid, json)) = ctx.blueprint(out, AREA, class.blueprint.as_ref(), "class") else {
            continue;
        };
        let mark = out.mark();
        let at = DataAt::Class(class.key.clone());
        let row = Row {
            ctx,
            at: &at,
            guid,
            name: ctx.name(guid),
        };
        let skill_points = int(&json["SkillPoints"]);
        row.number(
            out,
            "skill points differ (the user's facts)",
            skill_points,
            i64::from(class.skill_points),
        );
        let skills = game_skills(json);
        row.skills(out, &skills, &planner_skills(class.class_skills.iter()));
        let levels = class.levels.len() as u8;
        let bab = row.progression(json, "m_BaseAttackBonus");
        row.table(out, "BAB differs", bab.as_deref(), levels, |level| {
            class.base_attack_bonus(level)
        });
        for (save, field, kind) in SAVES {
            let game = row.progression(json, field);
            row.table(out, kind, game.as_deref(), levels, |level| {
                class.save_bonus(save, level, prestige, None)
            });
        }
        out.compared(AREA, mark);

        for archetype in ctx.planner.data.archetypes_of(&class.key) {
            archetype_tables(
                ctx,
                out,
                class,
                archetype,
                json,
                skill_points,
                &skills,
                levels,
            );
        }
    }
}

const SAVES: [(Save, &str, &str); 3] = [
    (Save::Fortitude, "m_FortitudeSave", "Fortitude differs"),
    (Save::Reflex, "m_ReflexSave", "Reflex differs"),
    (Save::Will, "m_WillSave", "Will differs"),
];

#[allow(clippy::too_many_arguments)]
fn archetype_tables(
    ctx: &Ctx,
    out: &mut Out,
    class: &Class,
    archetype: &Archetype,
    class_json: &Value,
    class_skill_points: i64,
    class_skills: &BTreeSet<String>,
    levels: u8,
) {
    let Some((guid, json)) = ctx.blueprint(out, AREA, archetype.blueprint.as_ref(), "archetype")
    else {
        return;
    };
    let mark = out.mark();
    let at = DataAt::Archetype(archetype.key.clone());
    let row = Row {
        ctx,
        at: &at,
        guid,
        name: ctx.name(guid),
    };
    row.number(
        out,
        "skill points differ (the user's facts)",
        class_skill_points + int(&json["AddSkillPoints"]),
        i64::from(archetype.skill_points.unwrap_or(class.skill_points)),
    );
    let game_skills = if flag(&json["ReplaceClassSkills"]) {
        self::game_skills(json)
    } else {
        class_skills.clone()
    };
    let planner = planner_skills(
        archetype
            .class_skills
            .as_ref()
            .unwrap_or(&class.class_skills)
            .iter(),
    );
    row.skills(out, &game_skills, &planner);
    if reference(&json["m_BaseAttackBonus"]).is_some() {
        let bab = row.progression(json, "m_BaseAttackBonus");
        row.table(out, "BAB differs", bab.as_deref(), levels, |level| {
            class.base_attack_bonus(level)
        });
    }
    for (save, field, kind) in SAVES {
        let table = if reference(&json[field]).is_some() {
            json
        } else {
            class_json
        };
        let game = row.progression(table, field);
        row.table(out, kind, game.as_deref(), levels, |level| {
            class.save_bonus(save, level, false, Some(archetype))
        });
    }
    out.compared(AREA, mark);
}

fn game_skills(json: &Value) -> BTreeSet<String> {
    json["ClassSkills"]
        .as_array()
        .map(|skills| {
            skills
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn planner_skills<'a>(skills: impl Iterator<Item = &'a rules::SkillKey>) -> BTreeSet<String> {
    skills.map(skill_stat).collect()
}

struct Row<'a> {
    ctx: &'a Ctx<'a>,
    at: &'a DataAt,
    guid: &'a BlueprintGuid,
    name: String,
}

impl Row<'_> {
    fn finding(&self, out: &mut Out, kind: &'static str, game: String, planner: String) {
        out.finding(AREA, kind, self.at, self.guid, &self.name, game, planner);
    }

    fn number(&self, out: &mut Out, kind: &'static str, game: i64, planner: i64) {
        if game != planner {
            self.finding(out, kind, game.to_string(), planner.to_string());
        }
    }

    fn skills(&self, out: &mut Out, game: &BTreeSet<String>, planner: &BTreeSet<String>) {
        if game != planner {
            let list =
                |skills: &BTreeSet<String>| skills.iter().cloned().collect::<Vec<_>>().join(", ");
            self.finding(
                out,
                "class skills differ (the user's facts)",
                list(game),
                list(planner),
            );
        }
    }

    /// A stat progression's bonuses by level, through the reference in `field`.
    fn progression(&self, json: &Value, field: &str) -> Option<Vec<i64>> {
        let guid = reference(&json[field])?;
        let table = self.ctx.dump.json(&guid)?;
        Some(table["Bonuses"].as_array()?.iter().map(int).collect())
    }

    /// The table at each level from 1 to `levels`: the first level that differs, and how many do.
    fn table(
        &self,
        out: &mut Out,
        kind: &'static str,
        game: Option<&[i64]>,
        levels: u8,
        planner: impl Fn(u8) -> u8,
    ) {
        let Some(game) = game else {
            self.finding(out, kind, "no table".to_owned(), "a table".to_owned());
            return;
        };
        let differing: Vec<(u8, i64, i64)> = (1..=levels)
            .map(|level| {
                let game = game.get(usize::from(level)).copied().unwrap_or(-1);
                (level, game, i64::from(planner(level)))
            })
            .filter(|(_, game, planner)| game != planner)
            .collect();
        if let Some((level, game, planner)) = differing.first() {
            let count = differing.len();
            self.finding(
                out,
                kind,
                format!("+{game} at level {level}"),
                format!("+{planner} at level {level}; {count} of {levels} levels differ"),
            );
        }
    }
}
