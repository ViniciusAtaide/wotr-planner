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
//!   A walk adds a parent only where the chain is held (a grant path from it leads up to a mapped blueprint, an
//!   archetype or a class's progression: the composite blasts' orphan progressions are not on one) and it is no pick of
//!   any selection's list, but for the four basic blast progressions the research counts.
//!
//! Never a candidate: a member of any `m_AllFeatures` (a pick: the selections' stage), or a blueprint an entry maps
//! (a second run finds a carrier it made by its blueprint).

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArchetypeKey, BlueprintGuid, ClassKey, Component, DataAt, Fact, FeatureKey, OptionLevelRow,
};
use serde::Serialize;
use serde_json::Value;

use super::span::{Value as Field, ron_text};
use super::{Change, NewFile, Unexpressed};
use crate::areas::Ctx;
use crate::dump::{
    component_kind, components, flag, int, is_feature_kind, level_entries, reference, references,
};

/// The four basic blast progressions a chain reaches through a selection nothing models: `AirBlastProgression`,
/// `ColdBlastProgression`, `ElectricBlastProgression`, `WaterBlastProgression` (ticket 307's research, `final-cited.tsv`).
const BASIC_BLASTS: [&str; 4] = [
    "6f1d86ae43adf1049834457ce5264003",
    "dbb1159b0e8137c4ea20434a854ae6a8",
    "ba7767cb03f7f3949ad08bd3ff8a646f",
    "e4027e0fec48e8048a172c6627d4eba9",
];

fn basic_blast(guid: &BlueprintGuid) -> bool {
    BASIC_BLASTS.contains(&guid.as_str())
}

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
    // The features the entries map count too, so a run in pieces keys as one run does: `FooFeature` written first
    // as `foo-feature` still makes `FooProgression` `foo-progression`.
    let mapped = ctx.planner.data.features.iter().filter_map(|feature| {
        feature
            .blueprint
            .as_ref()
            .filter(|guid| ctx.dump.kind(guid).is_some())
    });
    for guid in mapped {
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
pub(super) fn words(name: &str) -> Vec<String> {
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
    let parents = granters(dump);
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
    // A chain never climbs to a pick of any selection's list, modelled or not, but for the four basic blasts
    // the research counts (their selections, `AirBlastSelection` and `WaterBlastSelection`, nothing models).
    let listed: BTreeSet<&BlueprintGuid> = lists.iter().flat_map(|(_, members)| members).collect();
    let mut out: BTreeSet<BlueprintGuid> = BTreeSet::new();
    let mut walk: Vec<BlueprintGuid> = Vec::new();
    for guid in prerequisite_references(ctx) {
        if wanted(&guid) && !picks.contains(&guid) && !split(&guid) && out.insert(guid.clone()) {
            walk.push(guid);
        }
    }
    // A chain exists only where it is held: the blueprints from which a grant path leads up to a mapped blueprint,
    // an archetype or a class's progression. A blueprint nothing grants (the composite blasts' progressions, which
    // only the game's actions give) is not on a chain.
    let mut children: BTreeMap<&BlueprintGuid, Vec<&BlueprintGuid>> = BTreeMap::new();
    let mut held: BTreeSet<&BlueprintGuid> = BTreeSet::new();
    let mut up: Vec<&BlueprintGuid> = Vec::new();
    for (child, above) in &parents {
        for parent in above {
            children.entry(parent).or_default().push(child);
            if (planner.maps(parent) || stops.contains(parent))
                && wanted(child)
                && held.insert(child)
            {
                up.push(child);
            }
        }
    }
    while let Some(held_guid) = up.pop() {
        for child in children.get(held_guid).into_iter().flatten() {
            if wanted(child) && held.insert(child) {
                up.push(child);
            }
        }
    }
    while let Some(child) = walk.pop() {
        for parent in parents.get(&child).into_iter().flatten() {
            if wanted(parent)
                && !stops.contains(parent)
                && (basic_blast(parent) || (held.contains(parent) && !listed.contains(parent)))
                && out.insert(parent.clone())
            {
                walk.push(parent.clone());
            }
        }
    }
    out
}

/// Each blueprint's granters: the features, archetypes and classes whose [`grants`] name it, in GUID order (a granter
/// naming it twice is listed twice).
pub(super) fn granters(dump: &crate::Dump) -> BTreeMap<BlueprintGuid, Vec<BlueprintGuid>> {
    let mut parents: BTreeMap<BlueprintGuid, Vec<BlueprintGuid>> = BTreeMap::new();
    for (guid, blueprint) in dump.iter() {
        let kind = blueprint.kind.as_str();
        if !is_feature_kind(kind)
            && kind != "BlueprintArchetype"
            && kind != "BlueprintCharacterClass"
        {
            continue;
        }
        let Some(json) = dump.json(guid) else {
            continue;
        };
        for child in grants(json) {
            parents.entry(child).or_default().push(guid.clone());
        }
    }
    parents
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

/// The finding kind a carrier grant answers.
pub const GRANT_KIND: &str = "carrier grant";

/// The grants (ticket 307m): to each existing feature, feat or option whose blueprint grants a carrier, the grant
/// components that make it hold the carrier, and for an option whose blueprint is a progression, its row at the
/// level the progression grants it. It only adds: a grant the entry already names (any grant component, or the
/// option's row at that level, naming the carrier) is not written again, and no component or row is removed.
///
/// A carrier is a `features/` entry of the carriers' template (`name` the key's words capitalised, no `summary` or
/// `guide`, only grant components: [`is_carrier`]), or one made in this run (`made`). With `prospective` (a dry
/// run) the carriers `only` names (GUIDs, as for the carriers area) count too, so the plan shows what a run after
/// the carriers area would write; with no `only` none does, so a grant toward a carrier no file holds is never planned.
/// A carrier's own grants are its own file's (ticket 307c), so a carrier is never a source here.
///
/// A grant whose condition or class has no single entry is left unwritten and listed.
pub(crate) fn attachments(
    ctx: &Ctx,
    made: &[NewFile],
    prospective: bool,
    only: Option<&BTreeSet<String>>,
) -> (Vec<Change>, Vec<Unexpressed>) {
    let data = ctx.planner.data;
    let mut targets: BTreeMap<BlueprintGuid, FeatureKey> = data
        .features
        .iter()
        .filter(|feature| is_carrier(feature))
        .filter_map(|feature| Some((feature.blueprint.clone()?, feature.key.clone())))
        .collect();
    let mut planned: Vec<NewFile> = made.to_vec();
    // Only the carriers `only` names, as the carriers area does: no run plans a grant toward one another ticket owns.
    if let (true, Some(only)) = (prospective, only) {
        planned.extend(changes(ctx, Some(only)).0);
    }
    for file in &planned {
        if let Ok(feature) = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str::<rules::Feature>(&file.text)
            && let Some(guid) = feature.blueprint
        {
            targets.insert(guid, feature.key);
        }
    }
    let mut sources: Vec<Source> = Vec::new();
    for feature in data.features.iter().filter(|f| !is_carrier(f)) {
        sources.push(Source {
            at: DataAt::Feature(feature.key.clone()),
            blueprint: feature.blueprint.as_ref(),
            components: &feature.components,
            rows: None,
        });
    }
    for feat in &data.feats {
        sources.push(Source {
            at: DataAt::Feat(feat.key.clone()),
            blueprint: feat.blueprint.as_ref(),
            components: &feat.components,
            rows: None,
        });
    }
    for option in &data.options {
        sources.push(Source {
            at: DataAt::SelectionOption(option.key.clone()),
            blueprint: option.blueprint.as_ref(),
            components: &option.components,
            rows: Some(&option.levels),
        });
    }
    let (mut out, mut left) = (Vec::new(), Vec::new());
    for source in sources {
        let Some(guid) = source.blueprint else {
            continue;
        };
        let Some(json) = ctx.dump.json(guid) else {
            continue;
        };
        let (added, rows, unread) = grants_of(ctx, &targets, &source, json);
        left.extend(unread.into_iter().map(|what| Unexpressed {
            at: source.at.clone(),
            what,
        }));
        let file = super::entry_file(data, &source.at);
        if !added.is_empty() {
            let all: Vec<String> = source
                .components
                .iter()
                .chain(&added)
                .map(ron_text)
                .collect();
            out.push(Change {
                at: source.at.clone(),
                file: file.clone(),
                field: "components",
                kind: GRANT_KIND,
                value: Some(Field::List(all)),
                old: format!("{} components", source.components.len()),
                new: added.iter().map(ron_text).collect::<Vec<_>>().join("; "),
            });
        }
        if let (Some(held), false) = (source.rows, rows.is_empty()) {
            let mut all: Vec<OptionLevelRow> = held.to_vec();
            for (level, key) in &rows {
                match all.iter_mut().find(|row| row.level == *level) {
                    Some(row) => row.features.push(key.clone()),
                    None => all.push(OptionLevelRow {
                        level: *level,
                        features: vec![key.clone()],
                        ..OptionLevelRow::default()
                    }),
                }
            }
            all.sort_by_key(|row| row.level);
            out.push(Change {
                at: source.at.clone(),
                file,
                field: "levels",
                kind: GRANT_KIND,
                value: Some(Field::List(all.iter().map(row_text).collect())),
                old: format!("{} rows", held.len()),
                new: rows
                    .iter()
                    .map(|(level, key)| format!("level {level}: {key}"))
                    .collect::<Vec<_>>()
                    .join("; "),
            });
        }
    }
    (out, left)
}

/// An entry that can hold a grant: what it has now, and its blueprint.
struct Source<'a> {
    at: DataAt,
    blueprint: Option<&'a BlueprintGuid>,
    components: &'a [Component],
    /// An option's rows; `None` for an entry that has none.
    rows: Option<&'a [OptionLevelRow]>,
}

/// Whether a feature is a carrier as the carriers area writes it: the key's words as the name, nothing else of its
/// own, and only grants of its own.
fn is_carrier(feature: &rules::Feature) -> bool {
    let name: Vec<String> = feature.key.as_str().split('-').map(capitalised).collect();
    feature.blueprint.is_some()
        && feature.name == name.join(" ")
        && feature.summary.is_empty()
        && feature.guide.is_none()
        && feature.option_row.is_none()
        && !feature.daily_choice
        && !feature.favoured_weapon_proficiency
        && feature.weapons.is_empty()
        && feature.components.iter().all(|part| {
            matches!(
                part,
                Component::Grant(_)
                    | Component::GrantAtClassLevel { .. }
                    | Component::GrantIf { .. }
                    | Component::GrantOnApply(_)
            )
        })
}

/// The facts a component grants or names in a grant.
pub(super) fn named(component: &Component) -> Vec<&Fact> {
    match component {
        Component::Grant(facts) => facts.iter().collect(),
        Component::GrantAtClassLevel { fact, .. } | Component::GrantOnApply(fact) => vec![fact],
        Component::GrantIf { fact, .. } => vec![fact],
        _ => Vec::new(),
    }
}

/// The components an entry's blueprint adds for the carriers it grants, the rows (option level, carrier) its
/// progression grants them at, and what could not be written.
fn grants_of(
    ctx: &Ctx,
    targets: &BTreeMap<BlueprintGuid, FeatureKey>,
    source: &Source,
    json: &Value,
) -> (Vec<Component>, Vec<(u8, FeatureKey)>, Vec<String>) {
    let mut held: BTreeSet<Fact> = source.components.iter().flat_map(named).cloned().collect();
    let carrier = |guid: &BlueprintGuid| targets.get(guid).map(|key| Fact::Feature(key.clone()));
    let (mut added, mut rows, mut left) = (Vec::new(), Vec::new(), Vec::new());
    for part in components(json) {
        match component_kind(part) {
            "AddFacts" => {
                let new: BTreeSet<Fact> = references(part.get("m_Facts"))
                    .iter()
                    .filter_map(carrier)
                    .filter(|fact| held.insert(fact.clone()))
                    .collect();
                if !new.is_empty() {
                    added.push(Component::Grant(new));
                }
            }
            "AddFeatureOnApply" => {
                if let Some(fact) = reference(&part["m_Feature"])
                    .as_ref()
                    .and_then(carrier)
                    .filter(|fact| held.insert(fact.clone()))
                {
                    added.push(Component::GrantOnApply(fact));
                }
            }
            "AddFeatureIfHasFact" => {
                let Some(feature) = reference(&part["m_Feature"]) else {
                    continue;
                };
                let Some(fact) = carrier(&feature).filter(|fact| !held.contains(fact)) else {
                    continue;
                };
                let checked = reference(&part["m_CheckedFact"]);
                let Some(has) = checked
                    .as_ref()
                    .and_then(|guid| fact_of(ctx, targets, guid))
                else {
                    left.push(format!(
                        "a grant of {} if it has {}: no single entry for the checked fact",
                        ctx.name(&feature),
                        checked.map_or_else(String::new, |guid| ctx.name(&guid))
                    ));
                    continue;
                };
                held.insert(fact.clone());
                added.push(Component::GrantIf {
                    has,
                    not: flag(&part["Not"]),
                    fact,
                });
            }
            "AddFeatureOnClassLevel" => {
                let Some(feature) = reference(&part["m_Feature"]) else {
                    continue;
                };
                let Some(fact) = carrier(&feature).filter(|fact| !held.contains(fact)) else {
                    continue;
                };
                let classes: Vec<BlueprintGuid> = reference(&part["m_Class"])
                    .into_iter()
                    .chain(references(part.get("m_AdditionalClasses")))
                    .collect();
                let archetypes = references(part.get("m_Archetypes"));
                let class_keys: Vec<ClassKey> = classes
                    .iter()
                    .filter_map(|guid| match one_entry(ctx, guid) {
                        Some(DataAt::Class(key)) => Some(key.clone()),
                        _ => None,
                    })
                    .collect();
                let archetype_keys: Vec<ArchetypeKey> = archetypes
                    .iter()
                    .filter_map(|guid| match one_entry(ctx, guid) {
                        Some(DataAt::Archetype(key)) => Some(key.clone()),
                        _ => None,
                    })
                    .collect();
                let Ok(level) = u8::try_from(int(&part["Level"])) else {
                    continue;
                };
                if class_keys.len() != classes.len() || archetype_keys.len() != archetypes.len() {
                    left.push(format!(
                        "a grant of {} at class level {level}: a class or archetype has no single entry",
                        ctx.name(&feature)
                    ));
                    continue;
                }
                held.insert(fact.clone());
                added.push(Component::GrantAtClassLevel {
                    fact,
                    classes: class_keys.into_iter().collect(),
                    archetypes: archetype_keys.into_iter().collect(),
                    level,
                    before: flag(&part["BeforeThisLevel"]),
                });
            }
            _ => {}
        }
    }
    if let Some(have) = source.rows {
        for (level, guids) in level_entries(&json["LevelEntries"]) {
            let Ok(level) = u8::try_from(level) else {
                continue;
            };
            for guid in guids {
                let Some(key) = targets.get(&guid) else {
                    continue;
                };
                let present = have
                    .iter()
                    .any(|row| row.level == level && row.features.contains(key));
                if !present && !rows.contains(&(level, key.clone())) {
                    rows.push((level, key.clone()));
                }
            }
        }
    }
    (added, rows, left)
}

/// The one entry that names a blueprint.
fn one_entry<'a>(ctx: &'a Ctx, guid: &BlueprintGuid) -> Option<&'a DataAt> {
    match ctx.planner.entries_of(guid)[..] {
        [at] => Some(at),
        _ => None,
    }
}

/// The fact a blueprint stands for: a carrier, or the one entry that maps it.
fn fact_of(
    ctx: &Ctx,
    targets: &BTreeMap<BlueprintGuid, FeatureKey>,
    guid: &BlueprintGuid,
) -> Option<Fact> {
    if let Some(key) = targets.get(guid) {
        return Some(Fact::Feature(key.clone()));
    }
    fact_at(one_entry(ctx, guid)?)
}

/// The facts of every entry that maps a blueprint, in entry order: several for a blueprint entries share (ticket
/// 305's 90), none for one no entry maps or whose entries are no fact a character holds (a weapon, a spell).
pub(super) fn facts_of(ctx: &Ctx, guid: &BlueprintGuid) -> Vec<Fact> {
    ctx.planner
        .entries_of(guid)
        .into_iter()
        .filter_map(fact_at)
        .collect()
}

/// The fact an entry is, when it is one a character holds.
pub(super) fn fact_at(at: &DataAt) -> Option<Fact> {
    Some(match at {
        DataAt::Feat(key) => Fact::Feat(key.clone()),
        DataAt::Feature(key) => Fact::Feature(key.clone()),
        DataAt::SelectionOption(key) => Fact::Option(key.clone()),
        DataAt::FeatureSelection(key) => Fact::Selection(key.clone()),
        DataAt::MythicAbility(key) => Fact::MythicAbility(key.clone()),
        DataAt::MythicFeat(key) => Fact::MythicFeat(key.clone()),
        DataAt::Class(key) => Fact::Class(key.clone()),
        DataAt::Archetype(key) => Fact::Archetype(key.clone()),
        DataAt::Race(key) => Fact::Race(key.clone()),
        DataAt::Deity(key) => Fact::Deity(key.clone()),
        _ => return None,
    })
}

/// An option's row as the files hold it: the level, then the lists it has.
fn row_text(row: &OptionLevelRow) -> String {
    let mut parts = vec![format!("level: {}", row.level)];
    let mut list = |name: &str, empty: bool, text: String| {
        if !empty {
            parts.push(format!("{name}: {text}"));
        }
    };
    list("features", row.features.is_empty(), ron_text(&row.features));
    list("feats", row.feats.is_empty(), ron_text(&row.feats));
    list(
        "chosen_feats",
        row.chosen_feats.is_empty(),
        ron_text(&row.chosen_feats),
    );
    list(
        "selections",
        row.selections.is_empty(),
        ron_text(&row.selections),
    );
    list(
        "class_skills",
        row.class_skills.is_empty(),
        ron_text(&row.class_skills),
    );
    format!("({})", parts.join(", "))
}
