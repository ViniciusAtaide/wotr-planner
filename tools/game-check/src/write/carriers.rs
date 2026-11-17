//! The carriers target (ticket 307b): a `features/` entry for each hidden feature a prerequisite names, and for each
//! one on the chain that holds it, keyed and named from its asset name.
//!
//! A carrier is an entry of the planner with no game text: `(key, blueprint, name)`, the template of importer design
//! §2.7. Nothing names it yet, so no Build holds one (ticket 307, decision 2).
//!
//! The candidates (slices.md, 307b):
//! - **(a)** every blueprint a `Prerequisite*` component of an entry the prerequisites area compares references, at any
//!   depth (a `PrerequisiteCondition`'s tree too), that is a feature type and has no entry;
//! - **(c)** every such blueprint on a chain up from (a): the reverse of the grants of 295's vocabulary (`AddFacts`,
//!   `AddFeatureOnClassLevel`, `AddFeatureIfHasFact`, `AddFeatureOnApply`, a progression's `LevelEntries`, an
//!   archetype's `AddFeatures`), walked until a blueprint an entry maps, an archetype, or a class's own progression.
//!   The last two end a walk and are not candidates: the class's rows hold them (the writer's inference).
//!
//! Never a candidate: a member of any `m_AllFeatures` (a pick: the selections' stage), or a blueprint an entry maps
//! (a second run finds a carrier it made by its blueprint).

use std::collections::{BTreeMap, BTreeSet};

use rules::{BlueprintGuid, DataAt, FeatureKey};
use serde::Serialize;
use serde_json::Value;

use super::span::ron_text;
use super::{NewFile, Unexpressed};
use crate::areas::Ctx;
use crate::dump::{
    component_kind, components, is_feature_kind, level_entries, reference, references,
};

/// The finding kind a carrier answers.
pub const KIND: &str = "carrier";

/// The suffixes an asset name drops when no other would share the key.
const SUFFIXES: [&str; 3] = ["Feature", "Selection", "Progression"];

/// The new files of the carriers, in GUID order, and the candidates left unwritten. `only` limits the files to the
/// GUIDs it names; the keys are decided over the whole set first, so a key doesn't depend on it.
pub(crate) fn changes(
    ctx: &Ctx,
    only: Option<&BTreeSet<String>>,
) -> (Vec<NewFile>, Vec<Unexpressed>) {
    let candidates = candidates(ctx);
    let existing: BTreeSet<&str> = ctx
        .planner
        .data
        .features
        .iter()
        .map(|feature| feature.key.as_str())
        .collect();
    let mut by_short: BTreeMap<String, usize> = BTreeMap::new();
    for guid in &candidates {
        *by_short.entry(short_key(&ctx.name(guid))).or_default() += 1;
    }
    let keyed: Vec<(&BlueprintGuid, String)> = candidates
        .iter()
        .map(|guid| {
            let name = ctx.name(guid);
            let short = short_key(&name);
            let shared = by_short[&short] > 1 || existing.contains(short.as_str());
            (
                guid,
                if shared {
                    words(&name).join("-")
                } else {
                    short
                },
            )
        })
        .collect();
    let mut taken: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, key) in &keyed {
        *taken.entry(key.as_str()).or_default() += 1;
    }
    let (mut files, mut left) = (Vec::new(), Vec::new());
    for (guid, key) in &keyed {
        if only.is_some_and(|only| !only.contains(guid.as_str())) {
            continue;
        }
        let at = DataAt::Feature(FeatureKey::new(key.as_str()));
        if taken[key.as_str()] > 1 || existing.contains(key.as_str()) {
            left.push(Unexpressed {
                at,
                what: format!("{guid} ({}): its key is taken", ctx.name(guid)),
            });
            continue;
        }
        files.push(NewFile {
            at,
            file: format!("features/{key}.ron").into(),
            text: text(key, guid),
            kind: KIND,
        });
    }
    (files, left)
}

/// The carriers' file as the files hold an entry: one line.
#[derive(Serialize)]
struct Line<'a> {
    key: &'a str,
    blueprint: &'a str,
    name: String,
}

fn text(key: &str, guid: &BlueprintGuid) -> String {
    let name: Vec<String> = key.split('-').map(capitalised).collect();
    let line = Line {
        key,
        blueprint: guid.as_str(),
        name: name.join(" "),
    };
    format!("{}\n", ron_text(&line))
}

fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// An asset name's words, lowercased: a capital with its lowercase letters, a run of capitals (an acronym) up to the
/// capital that starts the next word, a run of digits.
fn words(name: &str) -> Vec<String> {
    let chars: Vec<char> = name.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut word = String::new();
    for (at, c) in chars.iter().enumerate() {
        if !c.is_alphanumeric() {
            if !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            continue;
        }
        let before = at
            .checked_sub(1)
            .map(|i| chars[i])
            .filter(|p| p.is_alphanumeric());
        let after = chars.get(at + 1);
        let breaks = before.is_some_and(|p| {
            (c.is_uppercase() && (p.is_lowercase() || p.is_ascii_digit()))
                || (c.is_uppercase() && p.is_uppercase() && after.is_some_and(|n| n.is_lowercase()))
                || (c.is_ascii_digit() != p.is_ascii_digit())
        });
        if breaks && !word.is_empty() {
            out.push(std::mem::take(&mut word));
        }
        word.extend(c.to_lowercase());
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

/// The key of an asset name less one trailing suffix, kebab-case; the whole name when nothing is left.
fn short_key(name: &str) -> String {
    let stripped = SUFFIXES
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .filter(|rest| !rest.is_empty())
        .unwrap_or(name);
    words(stripped).join("-")
}

/// The candidates, in GUID order.
fn candidates(ctx: &Ctx) -> BTreeSet<BlueprintGuid> {
    let dump = ctx.dump;
    let planner = ctx.planner;
    let wanted =
        |guid: &BlueprintGuid| !planner.maps(guid) && dump.kind(guid).is_some_and(is_feature_kind);
    let mut parents: BTreeMap<BlueprintGuid, Vec<BlueprintGuid>> = BTreeMap::new();
    // Each selection's list, and every blueprint a progression's or an archetype's rows grant.
    let mut lists: Vec<(BlueprintGuid, Vec<BlueprintGuid>)> = Vec::new();
    let mut in_rows: BTreeSet<BlueprintGuid> = BTreeSet::new();
    let mut stops: BTreeSet<BlueprintGuid> = BTreeSet::new();
    for (guid, blueprint) in dump.iter() {
        let kind = blueprint.kind.as_str();
        let feature = is_feature_kind(kind);
        if !feature && kind != "BlueprintArchetype" && kind != "BlueprintCharacterClass" {
            continue;
        }
        let Some(json) = dump.json(guid) else {
            continue;
        };
        if kind == "BlueprintArchetype" {
            stops.insert(guid.clone());
        }
        if kind == "BlueprintCharacterClass" {
            stops.extend(reference(&json["m_Progression"]));
        }
        if feature {
            lists.push((guid.clone(), references(json.get("m_AllFeatures"))));
        }
        for rows in [&json["LevelEntries"], &json["AddFeatures"]] {
            in_rows.extend(level_entries(rows).into_values().flatten());
        }
        for child in grants(json) {
            parents.entry(child).or_default().push(guid.clone());
        }
    }
    // A pick is the selections' stage's: a member of the list of a selection that has an entry or that the rows
    // grant, and a selection the rows grant (the writer's reading of "a pick"; ticket 307's research classed seeds
    // the same way). A pick is never a candidate by itself and nothing is walked up from it; a chain can still reach
    // one (the kineticist's blast progressions, listed by selections nothing models).
    let picks: BTreeSet<&BlueprintGuid> = lists
        .iter()
        .filter(|(selection, _)| planner.maps(selection) || in_rows.contains(selection))
        .flat_map(|(_, members)| members)
        .collect();
    let split = |guid: &BlueprintGuid| {
        in_rows.contains(guid)
            && dump
                .kind(guid)
                .is_some_and(|kind| kind.starts_with("BlueprintFeatureSelection"))
    };
    let mut out: BTreeSet<BlueprintGuid> = BTreeSet::new();
    let mut walk: Vec<BlueprintGuid> = Vec::new();
    for guid in prerequisite_references(ctx) {
        if wanted(&guid) && !picks.contains(&guid) && !split(&guid) && out.insert(guid.clone()) {
            walk.push(guid);
        }
    }
    while let Some(child) = walk.pop() {
        for parent in parents.get(&child).into_iter().flatten() {
            if wanted(parent) && !stops.contains(parent) && out.insert(parent.clone()) {
                walk.push(parent.clone());
            }
        }
    }
    out
}

/// What a blueprint grants through the vocabulary's components and rows.
fn grants(json: &Value) -> Vec<BlueprintGuid> {
    let mut out = Vec::new();
    for part in components(json) {
        match component_kind(part) {
            "AddFacts" => out.extend(references(part.get("m_Facts"))),
            "AddFeatureOnClassLevel" | "AddFeatureIfHasFact" | "AddFeatureOnApply" => {
                out.extend(reference(&part["m_Feature"]));
            }
            _ => {}
        }
    }
    for rows in [&json["LevelEntries"], &json["AddFeatures"]] {
        out.extend(level_entries(rows).into_values().flatten());
    }
    out
}

/// Every reference inside a `Prerequisite*` component of an entry the prerequisites area compares.
fn prerequisite_references(ctx: &Ctx) -> BTreeSet<BlueprintGuid> {
    let data = ctx.planner.data;
    let mut blueprints: Vec<&BlueprintGuid> = Vec::new();
    blueprints.extend(data.feats.iter().filter_map(|e| e.blueprint.as_ref()));
    blueprints.extend(data.options.iter().filter_map(|e| e.blueprint.as_ref()));
    blueprints.extend(
        data.mythic_abilities
            .iter()
            .filter_map(|e| e.blueprint.as_ref()),
    );
    blueprints.extend(
        data.mythic_feats
            .iter()
            .filter_map(|e| e.blueprint.as_ref()),
    );
    for class in data.all_classes() {
        blueprints.extend(class.blueprint.as_ref());
        blueprints.extend(
            data.archetypes_of(&class.key)
                .filter_map(|a| a.blueprint.as_ref()),
        );
    }
    let mut out = BTreeSet::new();
    for guid in blueprints {
        let Some(json) = ctx.dump.json(guid) else {
            continue;
        };
        for part in components(json) {
            if component_kind(part).starts_with("Prerequisite") {
                collect(part, &mut out);
            }
        }
    }
    out
}

/// Every blueprint reference in a JSON value, at any depth.
fn collect(value: &Value, out: &mut BTreeSet<BlueprintGuid>) {
    match value {
        Value::String(_) => out.extend(reference(value)),
        Value::Array(items) => items.iter().for_each(|item| collect(item, out)),
        Value::Object(fields) => fields.values().for_each(|item| collect(item, out)),
        _ => {}
    }
}
