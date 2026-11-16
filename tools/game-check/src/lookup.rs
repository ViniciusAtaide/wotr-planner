//! Blueprint lookups (ticket 299): `show`, `grep` and `refs` over every blueprint of the pinned dump, whatever its
//! kind, so a ticket's research reads a blueprint with one command.
//!
//! The output is safe to paste into a ticket: a blueprint is named by asset name and GUID, and every field that
//! holds the game's display text is left out ([`PLACEHOLDER`] stands in its place) and never matched by `grep`.

use std::collections::BTreeMap;
use std::fmt::Write;

use regex::Regex;
use serde_json::Value;

use crate::{Error, binz};

/// What stands in the output for a localized text.
pub const PLACEHOLDER: &str = "<localized text>";

/// The fields that hold the game's display text, whatever shape their value has.
const TEXT_FIELDS: [&str; 5] = [
    "m_DisplayName",
    "m_Description",
    "m_DescriptionShort",
    "m_Flavor",
    "m_ShortDescription",
];

/// The keys of a localized string's object: a reference into the strings chunk.
const TEXT_KEYS: [&str; 3] = ["m_Key", "Shared", "stringkey"];

struct Entry {
    name: String,
    kind: String,
    raw: Box<str>,
}

/// Every blueprint of a dump, with its JSON.
pub struct Library {
    /// By GUID.
    entries: BTreeMap<String, Entry>,
    /// A name's GUIDs, in GUID order.
    by_name: BTreeMap<String, Vec<String>>,
}

impl Library {
    /// The blueprints of a `.binz` file's bytes, unchecked.
    pub fn parse(bytes: &[u8]) -> Result<Library, Error> {
        let mut library = Library {
            entries: BTreeMap::new(),
            by_name: BTreeMap::new(),
        };
        binz::scan(bytes, |guid, name, type_name, raw| {
            let entry = Entry {
                name: name.to_owned(),
                kind: type_name.rsplit('.').next().unwrap_or(type_name).to_owned(),
                raw: binz::text(raw)?.into(),
            };
            if library.entries.insert(guid.to_owned(), entry).is_some() {
                return Err(Error::Format(format!("the GUID {guid} twice")));
            }
            Ok(())
        })?;
        for (guid, entry) in &library.entries {
            library
                .by_name
                .entry(entry.name.clone())
                .or_default()
                .push(guid.clone());
        }
        Ok(library)
    }

    /// The GUIDs a query names: the GUID itself (32 hex digits, any case, an optional `bp_` or `!bp_` before it),
    /// or every blueprint of that asset name.
    fn resolve(&self, query: &str) -> Vec<&str> {
        let bare = query
            .trim_start_matches("!bp_")
            .trim_start_matches("bp_")
            .to_ascii_lowercase();
        if let Some((guid, _)) = self.entries.get_key_value(&bare) {
            return vec![guid.as_str()];
        }
        self.by_name
            .get(query)
            .map(|guids| guids.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// `<name> <guid>`, or `(not in the dump) <guid>`.
    fn label(&self, guid: &str) -> String {
        match self.entries.get(guid) {
            Some(entry) => format!("{} {guid}", entry.name),
            None => format!("(not in the dump) {guid}"),
        }
    }

    /// Each named blueprint: a header line (`name guid type`), then its JSON with the references resolved and the
    /// text left out. Blueprints are separated by a blank line.
    pub fn show(&self, queries: &[String]) -> Result<String, String> {
        let mut out = String::new();
        for query in queries {
            let guids = self.resolve(query);
            if guids.is_empty() {
                return Err(format!("no blueprint is named or numbered {query}"));
            }
            for guid in guids {
                let entry = &self.entries[guid];
                let json = parse_json(entry, guid)?;
                let json = self.render(&json, None);
                let _ = writeln!(out, "{} {guid} {}", entry.name, entry.kind);
                out.push_str(&serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?);
                out.push_str("\n\n");
            }
        }
        Ok(out)
    }

    /// `guid name type` for each blueprint whose name or JSON matches, in name order. The JSON is matched with the
    /// text left out, on one line, references as GUIDs (`"Stat":"Strength"`).
    pub fn grep(&self, pattern: &str) -> Result<String, String> {
        let regex = Regex::new(pattern).map_err(|error| format!("the pattern: {error}"))?;
        let mut found = Vec::new();
        for (guid, entry) in &self.entries {
            if regex.is_match(&entry.name) {
                found.push((guid, entry));
                continue;
            }
            let json = parse_json(entry, guid)?;
            // The whole JSON matches before the text is left out, never after: the first match is a filter.
            if regex.is_match(&json.to_string())
                && regex.is_match(&without_text(&json, None).to_string())
            {
                found.push((guid, entry));
            }
        }
        Ok(self.listing(found))
    }

    /// `guid name type` for each blueprint that references the named one, in name order.
    pub fn refs(&self, query: &str) -> Result<String, String> {
        let guids = self.resolve(query);
        if guids.is_empty() {
            return Err(format!("no blueprint is named or numbered {query}"));
        }
        let needles: Vec<String> = guids.iter().map(|guid| format!("!bp_{guid}")).collect();
        let found = self
            .entries
            .iter()
            .filter(|(guid, entry)| {
                !guids.contains(&guid.as_str())
                    && needles.iter().any(|needle| entry.raw.contains(needle))
            })
            .collect();
        Ok(self.listing(found))
    }

    fn listing(&self, mut found: Vec<(&String, &Entry)>) -> String {
        found.sort_by(|a, b| (&a.1.name, a.0).cmp(&(&b.1.name, b.0)));
        found.iter().fold(String::new(), |mut out, (guid, entry)| {
            let _ = writeln!(out, "{guid} {} {}", entry.name, entry.kind);
            out
        })
    }

    /// The JSON without its text, each `!bp_<guid>` reference as `<name> <guid>`.
    fn render(&self, value: &Value, key: Option<&str>) -> Value {
        if is_text(value, key) {
            return Value::String(PLACEHOLDER.to_owned());
        }
        match value {
            Value::String(text) => match text.strip_prefix("!bp_") {
                Some(guid) if !guid.is_empty() => Value::String(self.label(guid)),
                _ => value.clone(),
            },
            Value::Array(items) => {
                Value::Array(items.iter().map(|v| self.render(v, None)).collect())
            }
            Value::Object(fields) => Value::Object(
                fields
                    .iter()
                    .map(|(k, v)| (k.clone(), self.render(v, Some(k))))
                    .collect(),
            ),
            _ => value.clone(),
        }
    }
}

fn parse_json(entry: &Entry, guid: &str) -> Result<Value, String> {
    serde_json::from_str(&entry.raw).map_err(|error| format!("{} {guid}: {error}", entry.name))
}

/// Whether a value holds the game's text: under a text field's key, or a localized string's object.
fn is_text(value: &Value, key: Option<&str>) -> bool {
    key.is_some_and(|key| TEXT_FIELDS.contains(&key))
        || value
            .as_object()
            .is_some_and(|fields| fields.keys().any(|key| TEXT_KEYS.contains(&key.as_str())))
}

/// The JSON with each text replaced by [`PLACEHOLDER`], all the way down.
fn without_text(value: &Value, key: Option<&str>) -> Value {
    if is_text(value, key) {
        return Value::String(PLACEHOLDER.to_owned());
    }
    match value {
        Value::Array(items) => Value::Array(items.iter().map(|v| without_text(v, None)).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(k, v)| (k.clone(), without_text(v, Some(k))))
                .collect(),
        ),
        other => other.clone(),
    }
}
