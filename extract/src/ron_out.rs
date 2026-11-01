//! Writing [`GameData`] as the `game-data/` RON files: one small item per line, fields left at their default
//! left out, so the files read and diff well when edited by hand.

use std::path::PathBuf;

use ron::ser::PrettyConfig;
use std::collections::BTreeSet;

use rules::{
    Advancement, Archetype, ArchetypeLevelRow, Class, ClassLevelRow, Feat, FeatKey, FeatList,
    FeatSlot, FeatValue, Feature, FeatureSelection, GameData, MythicAbility, MythicFeat,
    MythicPath, MythicRankRow, OptionLevelRow, PathSwitch, Prerequisite, Race, Repeat,
    SelectionOption, Skill, Weapon,
};
use serde::Serialize;

/// Every data file, as paths relative to the data folder and their text.
pub fn files(data: &GameData) -> Vec<(PathBuf, String)> {
    let mut files = vec![
        ("progression.ron".into(), progression(data)),
        ("races.ron".into(), list(&data.races, race)),
        ("skills.ron".into(), list(&data.skills, skill)),
        ("weapons.ron".into(), list(&data.weapons, weapon)),
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

    fn set(&mut self, name: &str, values: &BTreeSet<FeatKey>) -> &mut Self {
        if !values.is_empty() {
            self.field(name, values);
        }
        self
    }

    /// Feat slots, when there are any; an option list slot's fixed option level is a plain number, as the files
    /// read an option (`at_level: 2`).
    fn slots(&mut self, name: &str, slots: &[FeatSlot]) -> &mut Self {
        if !slots.is_empty() {
            let slots: Vec<String> = slots
                .iter()
                .map(|slot| match slot {
                    FeatSlot::OptionList {
                        selection,
                        at_level: Some(level),
                    } => format!(
                        "OptionList(selection: {}, at_level: {level})",
                        inline(selection)
                    ),
                    _ => inline(slot),
                })
                .collect();
            self.0.push(format!("{name}: [{}]", slots.join(", ")));
        }
        self
    }

    /// Feats granted for a value, when there are any.
    fn values(&mut self, name: &str, values: &BTreeSet<FeatValue>) -> &mut Self {
        if !values.is_empty() {
            self.field(name, values);
        }
        self
    }

    /// A repeat other than once.
    fn repeat(&mut self, repeat: Repeat) -> &mut Self {
        if repeat != Repeat::Once {
            self.field("repeat", &repeat);
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

/// A block with a comment line above it, at `indent`, when it has one.
fn noted(note: Option<&str>, indent: usize, block: String) -> String {
    match note {
        Some(note) if !block.is_empty() => format!("{}{note}\n{block}", " ".repeat(indent)),
        _ => block,
    }
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

/// A weapon, with its proficiency list when one names it.
fn weapon(weapon: &Weapon) -> String {
    let mut fields = Fields::new();
    fields.field("key", &weapon.key).field("name", &weapon.name);
    if let Some(proficiency) = &weapon.proficiency {
        fields.field("proficiency", proficiency);
    }
    fields.line()
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
                .slots("feat_slots", &sub.feat_slots)
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
        .slots("feat_slots", &race.feat_slots);
    let lists = nested(
        "feat_lists",
        8,
        race.feat_lists.iter().map(feat_list).collect(),
    );
    fields.block("", 4, &[lists, nested("subraces", 8, subraces)])
}

/// A feat on one line: what it's taken for and how often only when the guide says.
fn feat(feat: &Feat) -> String {
    let mut fields = Fields::new();
    fields
        .field("key", &feat.key)
        .field("name", &feat.name)
        .field("category", &feat.category)
        .text("summary", &feat.summary)
        .field("guide", &feat.guide);
    if let Some(parameter) = &feat.parameter {
        fields.field("parameter", parameter);
    }
    fields
        .repeat(feat.repeat)
        .prerequisite(&feat.prerequisite)
        .line()
}

fn mythic_ability(ability: &MythicAbility) -> String {
    Fields::new()
        .field("key", &ability.key)
        .field("name", &ability.name)
        .field("group", &ability.group)
        .text("summary", &ability.summary)
        .field("guide", &ability.guide)
        .prerequisite(&ability.prerequisite)
        .items("features", &ability.features)
        .items("selections", &ability.selections)
        .line()
}

/// A mythic feat on one line: what it's taken for, how often and whether it removes the racial penalty only when
/// set.
fn mythic_feat(feat: &MythicFeat) -> String {
    let mut fields = Fields::new();
    fields
        .field("key", &feat.key)
        .field("name", &feat.name)
        .field("group", &feat.group)
        .text("summary", &feat.summary)
        .field("guide", &feat.guide);
    if let Some(parameter) = &feat.parameter {
        fields.field("parameter", parameter);
    }
    fields
        .repeat(feat.repeat)
        .flag("removes_racial_penalty", feat.removes_racial_penalty)
        .prerequisite(&feat.prerequisite)
        .line()
}

fn feature(feature: &Feature) -> String {
    let mut fields = Fields::new();
    fields
        .field("key", &feature.key)
        .field("name", &feature.name)
        .text("summary", &feature.summary)
        .field("guide", &feature.guide)
        .items("ability_bonuses", &feature.ability_bonuses);
    if let Some(row) = &feature.option_row {
        fields.field("option_row", row);
    }
    fields.flag("daily_choice", feature.daily_choice);
    if let Some(floor) = feature.bab_floor {
        fields.0.push(format!("bab_floor: Some({floor})"));
    }
    fields
        .flag(
            "skill_ranks_at_least_mythic_rank",
            feature.skill_ranks_at_least_mythic_rank,
        )
        .line()
}

/// A switch rule on one line: the previous paths, drops and granted features only when set, the level cap only
/// when the switch raises it.
fn switch(switch: &PathSwitch) -> String {
    let mut fields = Fields::new();
    if !switch.from.is_empty() {
        fields.field("from", &switch.from);
    }
    fields
        .0
        .push(format!("kept_through_rank: {}", switch.kept_through_rank));
    fields.0.push(format!(
        "path_kept_through_rank: {}",
        switch.path_kept_through_rank
    ));
    if !switch.dropped_features.is_empty() {
        fields.field("dropped_features", &switch.dropped_features);
    }
    if !switch.dropped_selections.is_empty() {
        fields.field("dropped_selections", &switch.dropped_selections);
    }
    if !switch.features.is_empty() {
        fields.field("features", &switch.features);
    }
    fields
        .0
        .push(format!("further_ascensions: {}", switch.further_ascensions));
    if let Some(cap) = switch.level_cap {
        fields.0.push(format!("level_cap: Some({cap})"));
    }
    fields.line()
}

/// An option on one line, or, with rows by option level, a block with a row per line at `indent`.
fn option(option: &SelectionOption, indent: usize) -> String {
    let mut fields = Fields::new();
    fields
        .field("key", &option.key)
        .field("name", &option.name)
        .text("summary", &option.summary)
        .field("guide", &option.guide);
    if !option.feats.is_empty() {
        fields.field("feats", &option.feats);
    }
    if let Some(feat) = &option.chosen_feat {
        fields.field("chosen_feat", feat);
    }
    if let Some(parameter) = &option.parameter {
        fields.field("parameter", parameter);
    }
    if !option.values.is_empty() {
        fields.field("values", &option.values);
    }
    fields
        .repeat(option.repeat)
        .prerequisite(&option.prerequisite);
    if let Some(pool) = &option.pool {
        fields.field("pool", pool);
    }
    if let Some(selection) = &option.replaces_row_of {
        fields.field("replaces_row_of", selection);
    }
    if let Some(group) = &option.group {
        fields.field("group", group);
    }
    if let Some(arcana) = &option.arcana {
        fields.field("arcana", arcana);
    }
    if option.levels.is_empty() {
        return fields.line();
    }
    let rows = nested(
        "levels",
        indent + 4,
        option.levels.iter().map(option_level_row).collect(),
    );
    fields.block("", indent, &[rows])
}

fn option_level_row(row: &OptionLevelRow) -> String {
    let mut fields = Fields::new();
    fields.0.push(format!("level: {}", row.level));
    fields
        .items("features", &row.features)
        .set("feats", &row.feats)
        .set("chosen_feats", &row.chosen_feats)
        .slots("feat_slots", &row.feat_slots)
        .items("selections", &row.selections);
    if let Some(list) = &row.feat_list {
        fields.field("feat_list", list);
    }
    fields.line()
}

/// A feat list on one line: key, name, guide, the bonus-list and every-feat flags when set, then its feats (none on
/// a list of every feat).
fn feat_list(list: &FeatList) -> String {
    Fields::new()
        .field("key", &list.key)
        .field("name", &list.name)
        .field("guide", &list.guide)
        .flag("ignores_prerequisites", list.ignores_prerequisites)
        .flag("every_feat", list.every_feat)
        .set("feats", &list.feats)
        .line()
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
        if !selection.option_levels.is_empty() {
            fields.field("option_levels", &selection.option_levels);
        }
        if !selection.skipped_levels.is_empty() {
            fields.field("skipped_levels", &selection.skipped_levels);
        }
        fields
            .flag("feat_lists_only", selection.feat_lists_only)
            .flag("only_without_option", selection.only_without_option);
        if let Some(pool_of) = &selection.pool_of {
            fields.field("pool_of", pool_of);
        }
        fields.number("levels_behind", selection.levels_behind);
        if let Some(level_from) = &selection.level_from {
            fields.field("level_from", level_from);
        }
        if let Some(option) = &selection.level_also_from {
            fields.field("level_also_from", option);
        }
        fields.flag("once_per_option", selection.once_per_option);
        let options = selection
            .options
            .iter()
            .map(|o| option(o, indent + 12))
            .collect();
        let options = if selection.options.is_empty() {
            format!("{pad}        options: [],\n")
        } else {
            nested("options", indent + 8, options)
        };
        let shared = nested(
            "shared_options",
            indent + 8,
            selection.shared_options.iter().map(inline).collect(),
        );
        out.push_str(&format!(
            "{pad}    {},\n",
            fields.block("", indent + 4, &[options, shared])
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
        .set("feats", &row.feats)
        .values("feats_for", &row.feats_for)
        .slots("feat_slots", &row.feat_slots)
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
        .set("feats", &row.feats)
        .values("feats_for", &row.feats_for)
        .slots("feat_slots", &row.feat_slots)
        .items("selections", &row.selections)
        .line()
}

fn rank_row(row: &MythicRankRow) -> String {
    let mut fields = Fields::new();
    fields.0.push(format!("rank: {}", row.rank));
    fields
        .items("features", &row.features)
        .slots("feat_slots", &row.feat_slots)
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
    if let Some(spontaneous) = archetype.spontaneous {
        fields.0.push(format!("spontaneous: Some({spontaneous})"));
    }
    let blocks = [
        nested(
            "features",
            12,
            archetype.features.iter().map(feature).collect(),
        ),
        selections(12, &archetype.selections),
        noted(
            crate::class_rules::feat_lists_note(archetype.key.as_str()),
            12,
            nested(
                "feat_lists",
                12,
                archetype.feat_lists.iter().map(feat_list).collect(),
            ),
        ),
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
    if class.advances != Advancement::Each {
        fields.field("advances", &class.advances);
    }
    fields.flag("spontaneous", class.spontaneous);
    if !class.counts_as.is_empty() {
        fields.field("counts_as", &class.counts_as);
    }
    let blocks = [
        nested("features", 4, class.features.iter().map(feature).collect()),
        selections(4, &class.selections),
        nested(
            "feat_lists",
            4,
            class.feat_lists.iter().map(feat_list).collect(),
        ),
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
    if let Some(rule) = &path.switch {
        fields.0.push(format!("switch: {}", switch(rule)));
    }
    let blocks = [
        nested("features", 4, path.features.iter().map(feature).collect()),
        selections(4, &path.selections),
        nested(
            "feat_lists",
            4,
            path.feat_lists.iter().map(feat_list).collect(),
        ),
        nested("ranks", 4, path.ranks.iter().map(rank_row).collect()),
    ];
    format!("{}\n", fields.block("MythicPath", 0, &blocks))
}
