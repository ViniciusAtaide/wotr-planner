//! Two records compared Build by Build, by name.
//!
//! One tab-separated line per difference, so a ticket filters them with `grep` or `awk`:
//!
//! - `only in A<TAB><build>` / `only in B<TAB><build>`: a Build one record lacks;
//! - `differs<TAB><build><TAB><field>, <field>…`: the fields of a Build that differ;
//! - `slot<TAB><build><TAB><slot id><TAB><what>`: for a Build whose `slots` differ, each slot that differs: `only in
//!   A`, `only in B`, or the slot's fields that differ.
//!
//! Nothing when the records agree. A light record (`record --light`, ticket 162: its lines say `light`) and a full
//! one aren't compared: the comparison is refused with a message.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

/// What a comparison found.
pub struct Comparison {
    /// The difference lines (see the module docs).
    pub lines: Vec<String>,
    /// The counts, for the log.
    pub summary: String,
}

impl Comparison {
    /// Whether the records agree.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// A record: its Builds' fields by name, and the names in its order.
struct Record {
    order: Vec<String>,
    builds: BTreeMap<String, Map<String, Value>>,
}

impl Record {
    /// Whether the record is a light one (its Builds say so; see [`crate::fingerprint::Mode`]).
    fn is_light(&self) -> bool {
        self.builds
            .values()
            .any(|fields| fields.contains_key("light"))
    }

    fn parse(record: &str, side: &str) -> Result<Self, String> {
        let mut order = Vec::new();
        let mut builds = BTreeMap::new();
        for (index, line) in record.lines().enumerate() {
            if line.is_empty() {
                continue;
            }
            let at = || format!("{side}, line {}", index + 1);
            let Value::Object(fields) =
                serde_json::from_str(line).map_err(|error| format!("{}: {error}", at()))?
            else {
                return Err(format!("{}: not a JSON object", at()));
            };
            let name = fields
                .get("build")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{}: no `build` name", at()))?
                .to_owned();
            if builds.insert(name.clone(), fields).is_some() {
                return Err(format!("{}: Build {name:?} twice", at()));
            }
            order.push(name);
        }
        Ok(Record { order, builds })
    }
}

/// A Build's slots, by id (none when it has no `slots` object).
fn slots(fields: &Map<String, Value>) -> Map<String, Value> {
    match fields.get("slots") {
        Some(Value::Object(slots)) => slots.clone(),
        _ => Map::new(),
    }
}

/// The keys whose values differ between two objects, a key on one side only included.
fn differing<'a>(a: &'a Map<String, Value>, b: &'a Map<String, Value>) -> Vec<&'a str> {
    let keys: BTreeSet<&str> = a.keys().chain(b.keys()).map(String::as_str).collect();
    keys.into_iter()
        .filter(|key| a.get(*key) != b.get(*key))
        .collect()
}

/// Compares record `a` with record `b` (each one JSON line per Build).
pub fn compare(a: &str, b: &str) -> Result<Comparison, String> {
    let a = Record::parse(a, "A")?;
    let b = Record::parse(b, "B")?;
    if a.is_light() != b.is_light() {
        let kind = |light| if light { "light" } else { "full" };
        return Err(format!(
            "A is a {} record and B a {} one: a light record leaves out `allows`, so it only compares with another \
             light one (record both with `--light`, or neither)",
            kind(a.is_light()),
            kind(b.is_light())
        ));
    }
    let mut lines = Vec::new();
    let mut differ = 0;
    for name in &a.order {
        let Some(b_fields) = b.builds.get(name) else {
            lines.push(format!("only in A\t{name}"));
            continue;
        };
        let a_fields = &a.builds[name];
        let fields = differing(a_fields, b_fields);
        if fields.is_empty() {
            continue;
        }
        differ += 1;
        lines.push(format!("differs\t{name}\t{}", fields.join(", ")));
        if fields.contains(&"slots") {
            let (a_slots, b_slots) = (slots(a_fields), slots(b_fields));
            let ids: BTreeSet<&String> = a_slots.keys().chain(b_slots.keys()).collect();
            for id in ids {
                let what = match (a_slots.get(id), b_slots.get(id)) {
                    (Some(_), None) => "only in A".to_owned(),
                    (None, Some(_)) => "only in B".to_owned(),
                    (Some(a_slot), Some(b_slot)) if a_slot == b_slot => continue,
                    (Some(Value::Object(a_slot)), Some(Value::Object(b_slot))) => {
                        differing(a_slot, b_slot).join(", ")
                    }
                    _ => "value".to_owned(),
                };
                lines.push(format!("slot\t{name}\t{id}\t{what}"));
            }
        }
    }
    let only_b: Vec<&String> = b
        .order
        .iter()
        .filter(|name| !a.builds.contains_key(*name))
        .collect();
    for name in &only_b {
        lines.push(format!("only in B\t{name}"));
    }
    let only_a = a.order.len() - (b.order.len() - only_b.len());
    let summary = format!(
        "A: {} Builds, B: {}; only in A: {only_a}, only in B: {}, differing: {differ}",
        a.order.len(),
        b.order.len(),
        only_b.len()
    );
    Ok(Comparison { lines, summary })
}
