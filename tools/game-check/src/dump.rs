//! The game's blueprints, by GUID: each one's asset name and type, and the JSON of the kinds the check compares.

use std::cell::OnceCell;
use std::collections::BTreeMap;

use rules::BlueprintGuid;
use serde_json::Value;

use crate::Error;

/// One blueprint of the dump.
#[derive(Debug)]
pub struct Blueprint {
    /// The asset name (`PowerAttackFeature`): an identifier, never the game's display text.
    pub name: String,
    /// The type's own name, without its namespace (`BlueprintFeature`).
    pub kind: String,
    /// The JSON, kept only for the kinds the check reads ([`keeps_json`]); parsed on first use.
    raw: Option<Box<str>>,
    /// The parsed JSON, or the parser's message when it doesn't parse.
    json: OnceCell<Result<Value, String>>,
}

/// Every blueprint of the dump, by GUID.
#[derive(Debug, Default)]
pub struct Dump {
    blueprints: BTreeMap<BlueprintGuid, Blueprint>,
}

impl Dump {
    /// Adds one record. `type_name` is the full type name (`Kingmaker.Blueprints.Classes.BlueprintFeature`). A GUID
    /// already present is an error: the pinned dump has none, and keeping either would depend on the order.
    pub(crate) fn insert(
        &mut self,
        guid: &str,
        name: &str,
        type_name: &str,
        raw: &[u8],
    ) -> Result<(), Error> {
        let kind = type_name.rsplit('.').next().unwrap_or(type_name).to_owned();
        let raw = if keeps_json(&kind) {
            Some(crate::binz::text(raw)?.into())
        } else {
            None
        };
        let blueprint = Blueprint {
            name: name.to_owned(),
            kind,
            raw,
            json: OnceCell::new(),
        };
        if self
            .blueprints
            .insert(BlueprintGuid::new(guid), blueprint)
            .is_some()
        {
            return Err(Error::Format(format!("the GUID {guid} twice")));
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.blueprints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blueprints.is_empty()
    }

    /// Every blueprint, in GUID order.
    pub fn iter(&self) -> impl Iterator<Item = (&BlueprintGuid, &Blueprint)> {
        self.blueprints.iter()
    }

    pub fn get(&self, guid: &BlueprintGuid) -> Option<&Blueprint> {
        self.blueprints.get(guid)
    }

    /// A blueprint's JSON, when the dump holds it and its kind is one the check reads.
    pub fn json(&self, guid: &BlueprintGuid) -> Option<&Value> {
        let blueprint = self.blueprints.get(guid)?;
        let raw = blueprint.raw.as_deref()?;
        blueprint
            .json
            .get_or_init(|| serde_json::from_str(raw).map_err(|error| error.to_string()))
            .as_ref()
            .ok()
    }

    /// The blueprints a comparison read whose JSON doesn't parse, each with the parser's message: the check fails
    /// on them rather than count them as missing.
    pub fn unparsed(&self) -> Vec<String> {
        self.blueprints
            .iter()
            .filter_map(|(guid, blueprint)| match blueprint.json.get() {
                Some(Err(error)) => Some(format!("{} {guid}: {error}", blueprint.name)),
                _ => None,
            })
            .collect()
    }

    /// A blueprint's asset name, or its GUID when the dump lacks it.
    pub fn name(&self, guid: &BlueprintGuid) -> String {
        self.get(guid).map_or_else(
            || format!("(not in the dump) {guid}"),
            |blueprint| blueprint.name.clone(),
        )
    }

    /// A blueprint's type name, if the dump holds it.
    pub fn kind(&self, guid: &BlueprintGuid) -> Option<&str> {
        self.get(guid).map(|blueprint| blueprint.kind.as_str())
    }
}

/// The kinds whose JSON a comparison reads: classes, archetypes, races, features of every sort, progressions, stat
/// tables, items and their enchantments. The rest (dialogue, cutscenes, abilities: most of the 542 MB) keep their name
/// and type only.
fn keeps_json(kind: &str) -> bool {
    is_feature_kind(kind)
        || kind.starts_with("BlueprintItem")
        || kind.ends_with("Enchantment")
        || matches!(
            kind,
            "BlueprintCharacterClass"
                | "BlueprintArchetype"
                | "BlueprintRace"
                | "BlueprintStatProgression"
        )
}

/// Whether a type is a feature of some sort: a feature, selection, parametrized feature or progression.
pub fn is_feature_kind(kind: &str) -> bool {
    kind.starts_with("BlueprintFeature")
        || matches!(
            kind,
            "BlueprintParametrizedFeature" | "BlueprintProgression"
        )
}

/// A blueprint reference in the JSON: `"!bp_<guid>"`; `None` for `null` or an empty one.
pub fn reference(value: &Value) -> Option<BlueprintGuid> {
    value
        .as_str()
        .and_then(|text| text.strip_prefix("!bp_"))
        .filter(|guid| !guid.is_empty())
        .map(BlueprintGuid::new)
}

/// The references of a JSON array (`m_AllFeatures`, `m_Features`), in order, empty ones left out.
pub fn references(value: Option<&Value>) -> Vec<BlueprintGuid> {
    value
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(reference).collect())
        .unwrap_or_default()
}

/// A component's type name, without the hash before it: `"<hash>, PrerequisiteFeature"` is `PrerequisiteFeature`.
pub fn component_kind(component: &Value) -> &str {
    let full = component["$type"].as_str().unwrap_or("");
    full.rsplit(", ").next().unwrap_or(full)
}

/// A blueprint's components.
pub fn components(json: &Value) -> &[Value] {
    json["Components"].as_array().map_or(&[], Vec::as_slice)
}

/// A JSON number as an integer, 0 when absent.
pub fn int(value: &Value) -> i64 {
    value.as_i64().unwrap_or(0)
}

/// A JSON boolean, false when absent.
pub fn flag(value: &Value) -> bool {
    value.as_bool().unwrap_or(false)
}

/// A progression's or archetype's level entries (`LevelEntries`, `AddFeatures`, `RemoveFeatures`): each level's
/// features, entries of one level merged in order, as `BlueprintProgression.GetLevelEntry` and
/// `BlueprintArchetype.GetAddEntry` merge them (WrathDecompiled 6ce4a6b).
pub fn level_entries(value: &Value) -> BTreeMap<i64, Vec<BlueprintGuid>> {
    let mut levels: BTreeMap<i64, Vec<BlueprintGuid>> = BTreeMap::new();
    for entry in value.as_array().map_or(&[][..], Vec::as_slice) {
        levels
            .entry(int(&entry["Level"]))
            .or_default()
            .extend(references(entry.get("m_Features")));
    }
    levels
}
