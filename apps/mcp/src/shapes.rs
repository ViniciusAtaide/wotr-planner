//! The tools' JSON: their arguments and their results. `rules` types aren't serde-shaped for an agent, so every
//! shape a tool takes or returns is defined here, and the tools build them from the engine's answers.
//!
//! Conventions: field names are `snake_case`; a result is always an object (a list sits in a named field, as
//! `search`'s `matches`); an optional field that doesn't apply is left out, except `prerequisite`, which is
//! `null` when there is none; a reference to another item is a [`Named`] `{key, name}`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// What an item is. A search can be limited to some kinds, and `describe` takes the kind with the key, since
/// keys are unique only within a kind.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A base or prestige class.
    Class,
    /// A class's archetype; its key starts with its class's (`arcanist/brown-fur-transmuter`).
    Archetype,
    /// A Mythic path (Lich, Aeon, …).
    Path,
    /// A feat, taken in a feat slot.
    Feat,
    /// A mythic ability, a Mythic rank's pick at odd ranks.
    MythicAbility,
    /// A mythic feat, a Mythic rank's pick at even ranks.
    MythicFeat,
    /// An option of a Feature selection: an arcanist exploit, a rogue talent, a bloodline, a Lich power, …
    #[serde(rename = "option")]
    SelectionOption,
    /// A race, chosen at Build start.
    Race,
    /// A deity, chosen at Build start.
    Deity,
    /// A background, chosen at Build start.
    Background,
}

/// `search`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// Words of a name, a key, or initials ("BFT" for Brown-Fur Transmuter). Case doesn't matter.
    pub query: String,
    /// Only these kinds; every kind when left out.
    #[serde(default)]
    pub kinds: Option<Vec<Kind>>,
    /// At most this many matches (default 10, at most 50).
    #[serde(default)]
    pub limit: Option<u32>,
}

/// `search`'s result.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SearchResult {
    /// The best match first: exact names and keys, then initials, then names starting with the query, then
    /// names holding its words, then any name or key containing it.
    pub matches: Vec<Hit>,
}

/// One item, as `search` lists it and `describe` heads it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Hit {
    pub kind: Kind,
    /// The key the other tools take.
    pub key: String,
    /// The name the planner shows.
    pub name: String,
    /// What it belongs to: an archetype's class, an option's Feature selection (keys).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Its group: "Base class" or "Prestige class", a feat's category, a mythic ability's or feat's group, an
    /// option's Feature selection, a background's group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// What taking it needs, in the planner's words ("Spell Focus (Conjuration)"); `null` when nothing.
    pub prerequisite: Option<String>,
    /// Where the guide describes it, for the user to read. Never fetch it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guide_url: Option<String>,
}

/// `describe`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DescribeArgs {
    /// The item's key, as `search` gives it.
    pub key: String,
    pub kind: Kind,
}

/// `describe`'s result: the item's [`Hit`] fields, plus what its kind has.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Item {
    #[serde(flatten)]
    pub hit: Hit,
    /// A short summary in the planner's own words, when the data has one (it rarely does).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// A parameterised feat's or option's value: "weapon", "skill", "school", …
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taken_for: Option<String>,
    /// A class's or archetype's rows by class level (an archetype's merged over its class's), or an option's
    /// rows by the level it counts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<Vec<LevelRow>>,
    /// A Mythic path's rows by Mythic rank, beyond each rank's general mythic pick.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranks: Option<Vec<RankRow>>,
    /// A class's archetypes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archetypes: Option<Vec<Named>>,
    /// The Feature selections a class, archetype, path or mythic ability defines or grants, with their options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selections: Option<Vec<Selection>>,
    /// The features a mythic ability grants while held.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<Named>>,
    /// The feats a background grants.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feats: Option<Vec<Named>>,
    /// A race's subraces.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subraces: Option<Vec<Named>>,
    /// The alignments a deity's followers may have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignments: Option<Vec<String>>,
    /// The domains a deity offers (options).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<Named>>,
}

/// Another item, by key and name.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Named {
    pub key: String,
    pub name: String,
}

/// What one class level (or an option's level) gives.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LevelRow {
    pub level: u8,
    /// Base attack bonus at this class level (classes and archetypes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bab: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fort: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reflex: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will: Option<u8>,
    /// The highest spell level castable at this class level, when it casts any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spell_levels: Option<SpellLevels>,
    /// A prestige class level that advances another class's spellcasting: "arcane", "divine".
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub advances: Vec<String>,
    pub features: Vec<Named>,
    /// Feats granted outright (a parameterised one named with its value).
    pub feats: Vec<Named>,
    /// Feat slots the level opens, by what they take ("Bonus feat", "Combat feat", a list's name).
    pub feat_slots: Vec<String>,
    /// Feature selections the level gives a pick of.
    pub selections: Vec<Named>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SpellLevels {
    pub arcane: u8,
    pub divine: u8,
}

/// What one Mythic rank of a path gives beyond the general mythic pick.
#[derive(Debug, Serialize, JsonSchema)]
pub struct RankRow {
    pub rank: u8,
    pub features: Vec<Named>,
    pub feat_slots: Vec<String>,
    pub selections: Vec<Named>,
}

/// A Feature selection and its options.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Selection {
    pub key: String,
    pub name: String,
    pub options: Vec<Named>,
}
