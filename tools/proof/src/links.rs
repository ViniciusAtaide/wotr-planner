//! Builds as share links, one `name<TAB>link` line each: the benchmark's links (`bench-links.tsv`), and the files
//! `export` writes and `record --links` reads, so two revisions record the same Builds.
//!
//! Or as JSON, one `name<TAB>json` line each (`export --json`, `record --builds`), for two revisions whose link layouts
//! differ (ticket 86 added the faith to version 1's start): serde reads a Build whatever the layout, and a side whose
//! Build start has no faith drops it.

use std::collections::BTreeMap;

use rules::{Build, Engine, GameData, LinkKeys, decode_build, encode_build};
use serde_json::{Value, json};

use crate::Named;

/// The benchmark's Builds (ticket 56's `links.json`), by name and link; an empty link is the blank Build.
const BENCH: &str = include_str!("../bench-links.tsv");

/// The benchmark's Builds, named `bench <name>`, opened as the planner opens a link.
pub fn bench_builds(engine: &Engine) -> Vec<Named> {
    parse(BENCH)
        .map(|(name, link)| {
            let build = if link.is_empty() {
                Build::default()
            } else {
                engine.open_link(link).expect("the benchmark's links open")
            };
            (format!("bench {name}"), build)
        })
        .collect()
}

/// A Build as JSON: its fields, with the picks as `[slot, pick]` pairs (JSON maps need text keys).
fn build_json(build: &Build) -> Value {
    let Build {
        name,
        start,
        levels,
        class_choices,
        mythic_path,
        path_switch,
        picks,
    } = build;
    json!({
        "name": name,
        "start": start,
        "levels": levels,
        "class_choices": class_choices,
        "mythic_path": mythic_path,
        "path_switch": path_switch,
        "picks": picks.iter().collect::<Vec<_>>(),
    })
}

fn build_from_json(value: Value) -> Result<Build, serde_json::Error> {
    let field = |key: &str| value.get(key).cloned().unwrap_or(Value::Null);
    let picks: Vec<(rules::SlotId, rules::Pick)> = serde_json::from_value(field("picks"))?;
    let class_choices: BTreeMap<rules::ClassKey, rules::ClassChoice> =
        serde_json::from_value(field("class_choices"))?;
    Ok(Build {
        name: serde_json::from_value(field("name"))?,
        start: serde_json::from_value(field("start"))?,
        levels: serde_json::from_value(field("levels"))?,
        class_choices,
        mythic_path: serde_json::from_value(field("mythic_path"))?,
        path_switch: serde_json::from_value(field("path_switch"))?,
        picks: picks.into_iter().collect(),
    })
}

/// The Builds as a JSON file, one `name<TAB>json` line each.
pub fn export_json(builds: &[Named]) -> String {
    let mut text = String::new();
    for (name, build) in builds {
        assert!(
            !name.contains(['\t', '\n']),
            "a Build name holds no tab or line break: {name:?}"
        );
        text.push_str(name);
        text.push('\t');
        text.push_str(&build_json(build).to_string());
        text.push('\n');
    }
    text
}

/// The Builds of a JSON file, in its order. A side whose Build start holds no faith (before ticket 86) leaves out
/// the deity Builds (`t86 …` and their full Builds), which only the other side records, and reads every other Build
/// without its faith. Returns the names of the lines it can't read, or leaves out, apart.
pub fn read_json(text: &str) -> (Vec<Named>, Vec<String>) {
    let faith = crate::generate::holds_faith();
    let mut builds = Vec::new();
    let mut failed = Vec::new();
    for (name, json) in parse(text) {
        let unfull = name.strip_prefix("full ").unwrap_or(name);
        if !faith && unfull.starts_with("t86 ") {
            failed.push(format!(
                "{name}: a deity Build, recorded on the side with deities only"
            ));
            continue;
        }
        match serde_json::from_str(json).and_then(build_from_json) {
            Ok(build) => builds.push((name.to_owned(), build)),
            Err(error) => failed.push(format!("{name}: {error}")),
        }
    }
    (builds, failed)
}

/// The `name<TAB>link` lines of a file, skipping blank ones.
fn parse(text: &str) -> impl Iterator<Item = (&str, &str)> {
    text.lines().filter(|line| !line.is_empty()).map(|line| {
        line.split_once('\t')
            .unwrap_or_else(|| panic!("a links line is `name<TAB>link`: {line:?}"))
    })
}

/// The Builds of a links file, in its order, decoded with the game data's keys whether or not they're possible (a
/// Build no sheet holds is recorded too). Returns the names of the links that don't decode apart: those Builds
/// can't be recorded on this side, so a comparison lists them as only in the other.
pub fn read(data: &GameData, text: &str) -> (Vec<Named>, Vec<String>) {
    let keys = LinkKeys::new(data);
    let mut builds = Vec::new();
    let mut failed = Vec::new();
    for (name, link) in parse(text) {
        if link.is_empty() {
            builds.push((name.to_owned(), Build::default()));
            continue;
        }
        match decode_build(link, &keys) {
            Ok(build) => builds.push((name.to_owned(), build)),
            Err(error) => failed.push(format!("{name}: {error:?}")),
        }
    }
    (builds, failed)
}

/// The Builds as a links file. Returns the names of the Builds whose link doesn't decode back to the same Build with
/// this game data's keys; they're left out.
pub fn export(data: &GameData, builds: &[Named]) -> (String, Vec<String>) {
    let keys = LinkKeys::new(data);
    let mut text = String::new();
    let mut skipped = Vec::new();
    for (name, build) in builds {
        assert!(
            !name.contains(['\t', '\n']),
            "a Build name holds no tab or line break: {name:?}"
        );
        let link = encode_build(build);
        if decode_build(&link, &keys).is_ok_and(|decoded| decoded == *build) {
            text.push_str(name);
            text.push('\t');
            text.push_str(&link);
            text.push('\n');
        } else {
            skipped.push(name.clone());
        }
    }
    (text, skipped)
}
