//! Blueprint lookups (ticket 299): `show`, `grep` and `refs` over every blueprint of the pinned dump, whatever its
//! kind, so a ticket's research reads a blueprint with one command. Ticket 311 adds `granted-by`, and a banner on
//! `show` and `refs` when a name is shared by several blueprints: a research agent once took the first of two
//! `BardTalentSelection`s, the one only units use, for the bard's.
//!
//! The output is safe to paste into a ticket: a blueprint is named by asset name and GUID, and every field that
//! holds the game's display text is left out ([`PLACEHOLDER`] stands in its place) and never matched by `grep`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write;
use std::sync::OnceLock;

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

/// The blueprint kinds [`Library::granted_by`] stops at: what gives a character the thing.
const GRANTING_KINDS: [&str; 5] = [
    "BlueprintProgression",
    "BlueprintArchetype",
    "BlueprintCharacterClass",
    "BlueprintRace",
    "BlueprintUnit",
];

/// How many chains `granted-by` prints per blueprint, and how many granters a banner names.
const CHAINS_SHOWN: usize = 30;
const GRANTERS_IN_BANNER: usize = 5;

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
    /// A blueprint's referrers (the blueprints whose JSON holds `!bp_<guid>`), built when first asked.
    referrers: OnceLock<BTreeMap<String, Vec<String>>>,
}

impl Library {
    /// The blueprints of a `.binz` file's bytes, unchecked.
    pub fn parse(bytes: &[u8]) -> Result<Library, Error> {
        let mut library = Library {
            entries: BTreeMap::new(),
            by_name: BTreeMap::new(),
            referrers: OnceLock::new(),
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
            out.push_str(&self.banner(&guids));
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
        let banner = self.banner(&guids);
        let needles: Vec<String> = guids.iter().map(|guid| format!("!bp_{guid}")).collect();
        let found = self
            .entries
            .iter()
            .filter(|(guid, entry)| {
                !guids.contains(&guid.as_str())
                    && needles.iter().any(|needle| entry.raw.contains(needle))
            })
            .collect();
        Ok(banner + &self.listing(found))
    }

    /// For each named blueprint, the chains of referrers from it up to the first progression, archetype, class,
    /// race or unit (`name guid <- name guid <- … [type]`): what grants it, found by following [`Library::refs`]
    /// upward. A name several blueprints share gets one block per GUID.
    pub fn granted_by(&self, query: &str) -> Result<String, String> {
        let guids = self.resolve(query);
        if guids.is_empty() {
            return Err(format!("no blueprint is named or numbered {query}"));
        }
        let mut out = self.banner(&guids);
        for guid in guids {
            let chains = self.grant_chains(guid);
            let _ = writeln!(out, "{}", self.label_with_kind(guid));
            if chains.is_empty() {
                out.push_str("  (no progression, archetype, class, race or unit reaches it)\n");
            }
            for chain in chains.iter().take(CHAINS_SHOWN) {
                let links: Vec<String> = chain.iter().map(|g| self.label(g)).collect();
                let last = chain
                    .last()
                    .map_or("", |g| self.entries[g.as_str()].kind.as_str());
                let _ = writeln!(out, "  {} [{last}]", links.join(" <- "));
            }
            if chains.len() > CHAINS_SHOWN {
                let _ = writeln!(out, "  … and {} more", chains.len() - CHAINS_SHOWN);
            }
        }
        Ok(out)
    }

    /// A line per GUID of a shared name, and a blank line; nothing for a name one blueprint holds.
    fn banner(&self, guids: &[&str]) -> String {
        let [first, _, ..] = guids else {
            return String::new();
        };
        let mut out = format!(
            "shared name: {} is {} blueprints; pick one by what grants it (`granted-by <guid>`):\n",
            self.entries[*first].name,
            guids.len()
        );
        for guid in guids {
            // Units last: a unit holding a feature says little about who gets it from a class or progression.
            let granters: BTreeSet<(bool, String)> = (self.grant_chains(guid).iter())
                .filter_map(|chain| chain.last())
                .map(|last| {
                    let entry = &self.entries[last.as_str()];
                    (
                        entry.kind == "BlueprintUnit",
                        format!("{} ({})", entry.name, entry.kind),
                    )
                })
                .collect();
            let shown: Vec<&str> = (granters.iter())
                .take(GRANTERS_IN_BANNER)
                .map(|(_, text)| text.as_str())
                .collect();
            let more = granters.len().saturating_sub(GRANTERS_IN_BANNER);
            let grant = match (shown.is_empty(), more) {
                (true, _) => "nothing grants it".to_owned(),
                (false, 0) => format!("granted by {}", shown.join(", ")),
                (false, more) => format!("granted by {}, and {more} more", shown.join(", ")),
            };
            let _ = writeln!(out, "  {}: {grant}", self.label_with_kind(guid));
        }
        out.push('\n');
        out
    }

    /// `<name> <guid> <type>`.
    fn label_with_kind(&self, guid: &str) -> String {
        format!("{} {}", self.label(guid), self.entries[guid].kind)
    }

    /// The paths from a blueprint up its referrers to each granting blueprint it reaches first (breadth first, so
    /// each is by a shortest path; a granting blueprint is not climbed past). A granting blueprint's own chain is
    /// itself.
    fn grant_chains(&self, start: &str) -> Vec<Vec<String>> {
        let granting = |guid: &String| GRANTING_KINDS.contains(&self.entries[guid].kind.as_str());
        let Some((start, _)) = self.entries.get_key_value(start) else {
            return Vec::new();
        };
        if granting(start) {
            return vec![vec![start.clone()]];
        }
        let referrers = self.referrers();
        let above = |guid: &String| referrers.get(guid).map_or(&[][..], Vec::as_slice);
        let mut chains: Vec<Vec<String>> = chains_up(start, above, granting)
            .into_iter()
            .map(|chain| chain.into_iter().cloned().collect())
            .collect();
        chains.sort_by_key(|chain| {
            let top = chain.last().map(|g| &self.entries[g.as_str()]);
            (chain.len(), top.map(|e| e.name.clone()), chain.clone())
        });
        chains
    }

    /// Each blueprint's referrers, in GUID order.
    fn referrers(&self) -> &BTreeMap<String, Vec<String>> {
        self.referrers.get_or_init(|| {
            let mut index: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
            for (guid, entry) in &self.entries {
                let mut rest: &str = &entry.raw;
                while let Some(at) = rest.find("!bp_") {
                    rest = &rest[at + 4..];
                    let named = rest.get(..32).filter(|id| {
                        id.bytes()
                            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                    });
                    if let Some(named) = named
                        && named != guid
                        && self.entries.contains_key(named)
                    {
                        index.entry(named.to_owned()).or_default().insert(guid);
                    }
                }
            }
            index
                .into_iter()
                .map(|(guid, above)| (guid, above.into_iter().map(str::to_owned).collect()))
                .collect()
        })
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

/// The paths from `start` up `above` to each node `stops` holds that it reaches first, each `[start, …, top]`: breadth
/// first, so each is by a shortest path, and a stop is not climbed past. `start` is climbed from whether it stops or
/// not. `granted-by` walks it up every reference to a progression, archetype, class, race or unit; the prerequisites
/// writer's carrier guard (308h) up the game's grants to a blueprint the planner maps.
pub(crate) fn chains_up<'a, T: Ord>(
    start: &'a T,
    above: impl Fn(&T) -> &'a [T],
    stops: impl Fn(&T) -> bool,
) -> Vec<Vec<&'a T>> {
    let mut parent: BTreeMap<&T, &T> = BTreeMap::new();
    let mut seen: BTreeSet<&T> = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    let mut found = Vec::new();
    while let Some(at) = queue.pop_front() {
        for up in above(at) {
            if !seen.insert(up) {
                continue;
            }
            parent.insert(up, at);
            if stops(up) {
                found.push(up);
            } else {
                queue.push_back(up);
            }
        }
    }
    found
        .into_iter()
        .map(|top| {
            let mut chain = vec![top];
            let mut at = top;
            while let Some(below) = parent.get(at) {
                chain.push(below);
                at = below;
            }
            chain.reverse();
            chain
        })
        .collect()
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
