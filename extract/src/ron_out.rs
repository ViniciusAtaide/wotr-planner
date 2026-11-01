//! Writing [`GameData`] as the `game-data/` RON files: one small item per line, fields left at their default
//! left out, so the files read and diff well when edited by hand.

use std::path::PathBuf;

use ron::ser::PrettyConfig;
use rules::{
    Archetype, ArchetypeLevelRow, Class, ClassLevelRow, Feat, Feature, FeatureSelection, GameData,
    MythicAbility, MythicFeat, MythicPath, MythicRankRow, Prerequisite, Race, SelectionOption,
    Skill,
};
use serde::Serialize;

/// Every data file, as paths relative to the data folder and their text.
pub fn files(data: &GameData) -> Vec<(PathBuf, String)> {
    let mut files = vec![
        ("progression.ron".into(), progression(data)),
        ("races.ron".into(), list(&data.races, race)),
        ("skills.ron".into(), list(&data.skills, skill)),
        ("feats.ron".into(), list(&data.feats, feat)),
        (
            "mythic/abilities.ron".into(),
            list(&data.mythic_abilities, mythic_ability),
        ),
        (
            "mythic/feats.ron".into(),
            list(&data.mythic_feats, mythic_feat),
        ),
    ];
    for class in &data.classes {
        files.push((
            format!("classes/{}.ron", class.key).into(),
            class_file(class),
        ));
    }
    for class in &data.prestige_classes {
        files.push((
            format!("prestige/{}.ron", class.key).into(),
            class_file(class),
        ));
    }
    for path in &data.mythic_paths {
        files.push((
            format!("mythic/paths/{}.ron", path.key).into(),
            path_file(path),
        ));
    }
    files
}

/// A value on one line.
fn inline<T: Serialize + ?Sized>(value: &T) -> String {
    let config = PrettyConfig::new()
        .new_line(" ".to_owned())
        .indentor(String::new())
        .compact_arrays(true)
        .compact_structs(true)
        .compact_maps(true);
    let text = ron::ser::to_string_pretty(value, config).expect("game data serialises");
    tidy(&text)
}

/// Removes the trailing commas and doubled spaces the one-line pretty printer leaves.
fn tidy(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escaped = false;
    for c in text.chars() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            ' ' if out.ends_with(' ') || out.ends_with('(') || out.ends_with('[') => {}
            ')' | ']' => {
                while out.ends_with(' ') || out.ends_with(',') {
                    out.pop();
                }
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Fields of a one-line struct, skipping the empty ones.
struct Fields(Vec<String>);

impl Fields {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn field<T: Serialize + ?Sized>(&mut self, name: &str, value: &T) -> &mut Self {
        self.0.push(format!("{name}: {}", inline(value)));
        self
    }

    fn text(&mut self, name: &str, value: &str) -> &mut Self {
        if !value.is_empty() {
            self.field(name, value);
        }
        self
    }

    fn number(&mut self, name: &str, value: u8) -> &mut Self {
        if value != 0 {
            self.0.push(format!("{name}: {value}"));
        }
        self
    }

    fn flag(&mut self, name: &str, value: bool) -> &mut Self {
        if value {
            self.0.push(format!("{name}: true"));
        }
        self
    }

    fn items<T: Serialize>(&mut self, name: &str, values: &[T]) -> &mut Self {
        if !values.is_empty() {
            self.field(name, values);
        }
        self
    }

    fn prerequisite(&mut self, value: &Option<Prerequisite>) -> &mut Self {
        if let Some(prerequisite) = value {
            self.field("prerequisite", prerequisite);
        }
        self
    }

    fn line(&self) -> String {
        format!("({})", self.0.join(", "))
    }

    /// The fields one per line at `indent`, with `extra` blocks (already indented) after them.
    fn block(&self, name: &str, indent: usize, extra: &[String]) -> String {
        let pad = " ".repeat(indent + 4);
        let mut out = format!("{name}(\n");
        for field in &self.0 {
            out.push_str(&format!("{pad}{field},\n"));
        }
        for block in extra {
            out.push_str(block);
        }
        out.push_str(&format!("{}){}", " ".repeat(indent), ""));
        out
    }
}

fn list<T>(items: &[T], one: fn(&T) -> String) -> String {
    let mut out = String::from("[\n");
    for item in items {
        out.push_str("    ");
        out.push_str(&one(item));
        out.push_str(",\n");
    }
    out.push_str("]\n");
    out
}

/// `name: [` items, one per line, `],` at `indent`.
fn nested(name: &str, indent: usize, items: Vec<String>) -> String {
    if items.is_empty() {
        return String::new();
    }
    let pad = " ".repeat(indent);
    let mut out = format!("{pad}{name}: [\n");
    for item in items {
        out.push_str(&format!("{pad}    {item},\n"));
    }
    out.push_str(&format!("{pad}],\n"));
    out
}

fn progression(data: &GameData) -> String {
    let rows = data
        .progression
        .mythic_ascensions
        .iter()
        .map(|a| format!("(rank: {}, character_level: {})", a.rank, a.character_level))
        .collect();
    format!("Progression(\n{})\n", nested("mythic_ascensions", 4, rows))
}

fn skill(skill: &Skill) -> String {
    Fields::new()
        .field("key", &skill.key)
        .field("name", &skill.name)
        .field("ability", &skill.ability)
        .line()
}

fn race(race: &Race) -> String {
    let subraces: Vec<String> = race
        .subraces
        .iter()
        .map(|sub| {
            Fields::new()
                .field("key", &sub.key)
                .field("name", &sub.name)
                .text("summary", &sub.summary)
                .field("guide", &sub.guide)
                .items("ability_modifiers", &sub.ability_modifiers)
                .line()
        })
        .collect();
    let mut fields = Fields::new();
    fields
        .field("key", &race.key)
        .field("name", &race.name)
        .text("summary", &race.summary)
        .field("guide", &race.guide)
        .items("ability_modifiers", &race.ability_modifiers)
        .items("feat_slots", &race.feat_slots);
    fields.block("", 4, &[nested("subraces", 8, subraces)])
}

fn feat(feat: &Feat) -> String {
    Fields::new()
        .field("key", &feat.key)
        .field("name", &feat.name)
        .field("category", &feat.category)
        .text("summary", &feat.summary)
        .field("guide", &feat.guide)
        .prerequisite(&feat.prerequisite)
        .line()
}

fn mythic_ability(ability: &MythicAbility) -> String {
    Fields::new()
        .field("key", &ability.key)
        .field("name", &ability.name)
        .text("group", &ability.group)
        .text("summary", &ability.summary)
        .field("guide", &ability.guide)
        .prerequisite(&ability.prerequisite)
        .line()
}

fn mythic_feat(feat: &MythicFeat) -> String {
    Fields::new()
        .field("key", &feat.key)
        .field("name", &feat.name)
        .text("group", &feat.group)
        .text("summary", &feat.summary)
        .field("guide", &feat.guide)
        .prerequisite(&feat.prerequisite)
        .line()
}

fn feature(feature: &Feature) -> String {
    Fields::new()
        .field("key", &feature.key)
        .field("name", &feature.name)
        .text("summary", &feature.summary)
        .field("guide", &feature.guide)
        .line()
}

fn option(option: &SelectionOption) -> String {
    let mut fields = Fields::new();
    fields
        .field("key", &option.key)
        .field("name", &option.name)
        .text("summary", &option.summary)
        .field("guide", &option.guide);
    if let Some(feat) = &option.feat {
        fields.field("feat", feat);
    }
    fields.prerequisite(&option.prerequisite).line()
}

fn selections(indent: usize, selections: &[FeatureSelection]) -> String {
    if selections.is_empty() {
        return String::new();
    }
    let pad = " ".repeat(indent);
    let mut out = format!("{pad}selections: [\n");
    for selection in selections {
        let mut fields = Fields::new();
        fields
            .field("key", &selection.key)
            .field("name", &selection.name)
            .text("summary", &selection.summary)
            .field("guide", &selection.guide);
        let options = selection.options.iter().map(option).collect();
        let options = if selection.options.is_empty() {
            format!("{pad}        options: [],\n")
        } else {
            nested("options", indent + 8, options)
        };
        out.push_str(&format!(
            "{pad}    {},\n",
            fields.block("", indent + 4, &[options])
        ));
    }
    out.push_str(&format!("{pad}],\n"));
    out
}

fn level_row(row: &ClassLevelRow) -> String {
    let mut fields = Fields::new();
    fields.0.push(format!("level: {}", row.level));
    fields
        .number("arcane_spell_level", row.arcane_spell_level)
        .number("divine_spell_level", row.divine_spell_level)
        .flag("advances_arcane", row.advances_arcane)
        .flag("advances_divine", row.advances_divine)
        .items("features", &row.features)
        .items("feat_slots", &row.feat_slots)
        .items("selections", &row.selections)
        .line()
}

/// An archetype's row: its spell levels, when it sets them, right after the level.
fn archetype_level_row(row: &ArchetypeLevelRow) -> String {
    let mut fields = Fields::new();
    fields.0.push(format!("level: {}", row.level));
    if let Some(spell_levels) = &row.spell_levels {
        fields.field("spell_levels", spell_levels);
    }
    fields
        .items("features", &row.features)
        .items("feat_slots", &row.feat_slots)
        .items("selections", &row.selections)
        .line()
}

fn rank_row(row: &MythicRankRow) -> String {
    let mut fields = Fields::new();
    fields.0.push(format!("rank: {}", row.rank));
    fields
        .items("features", &row.features)
        .items("feat_slots", &row.feat_slots)
        .items("selections", &row.selections)
        .line()
}

fn archetype(archetype: &Archetype) -> String {
    let mut fields = Fields::new();
    fields
        .field("key", &archetype.key)
        .field("name", &archetype.name)
        .text("summary", &archetype.summary)
        .field("guide", &archetype.guide)
        .prerequisite(&archetype.prerequisite);
    if let Some(points) = archetype.skill_points {
        fields.0.push(format!("skill_points: {points}"));
    }
    if let Some(skills) = &archetype.class_skills {
        fields.field("class_skills", skills);
    }
    let blocks = [
        nested(
            "features",
            12,
            archetype.features.iter().map(feature).collect(),
        ),
        selections(12, &archetype.selections),
        nested(
            "levels",
            12,
            archetype.levels.iter().map(archetype_level_row).collect(),
        ),
    ];
    fields.block("", 8, &blocks)
}

fn class_file(class: &Class) -> String {
    let mut fields = Fields::new();
    fields
        .field("key", &class.key)
        .field("name", &class.name)
        .text("summary", &class.summary)
        .field("guide", &class.guide);
    fields
        .0
        .push(format!("skill_points: {}", class.skill_points));
    fields
        .field("bab", &class.bab)
        .field("fort", &class.fort)
        .field("reflex", &class.reflex)
        .field("will", &class.will)
        .items("save_exceptions", &class.save_exceptions);
    if !class.class_skills.is_empty() {
        fields.field("class_skills", &class.class_skills);
    }
    fields.prerequisite(&class.prerequisite);
    let blocks = [
        nested("features", 4, class.features.iter().map(feature).collect()),
        selections(4, &class.selections),
        nested("levels", 4, class.levels.iter().map(level_row).collect()),
        nested(
            "archetypes",
            4,
            class.archetypes.iter().map(archetype).collect(),
        ),
    ];
    format!("{}\n", fields.block("Class", 0, &blocks))
}

fn path_file(path: &MythicPath) -> String {
    let mut fields = Fields::new();
    fields
        .field("key", &path.key)
        .field("name", &path.name)
        .text("summary", &path.summary)
        .field("guide", &path.guide);
    if let Some(switch) = &path.switch {
        fields.field("switch", switch);
    }
    let blocks = [
        nested("features", 4, path.features.iter().map(feature).collect()),
        selections(4, &path.selections),
        nested("ranks", 4, path.ranks.iter().map(rank_row).collect()),
    ];
    format!("{}\n", fields.block("MythicPath", 0, &blocks))
}
