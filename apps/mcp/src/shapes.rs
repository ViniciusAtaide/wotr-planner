//! The tools' JSON: their arguments and their results. `rules` types aren't serde-shaped for an agent, so every
//! shape a tool takes or returns is defined here, and the tools build them from the engine's answers.
//!
//! Conventions: field names are `snake_case`; a result is always an object (a list sits in a named field, as
//! `search`'s `matches`); an optional field that doesn't apply is left out, except `prerequisite`, which is
//! `null` when there is none; a reference to another item is a [`Named`] `{key, name}`. Every result about a Build
//! flattens the [`Envelope`] into it (`build`, `link`, `todo`); a choice is named by its address (`address.rs`) and
//! holds a [`ChoiceValue`].

use std::collections::BTreeMap;

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

// ---- Builds ------------------------------------------------------------------------------------------------------

/// What every tool that takes or makes a Build returns with its result: the Build's code, its link and its open
/// choices.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Envelope {
    /// The Build's code: what every Build-taking tool takes as `build`. A new code for every change; keep an old one
    /// to branch.
    pub build: String,
    /// The Build in the planner, for the user: `https://viniciusataide.github.io/wotr-planner/#b=<code>`.
    pub link: String,
    /// The choices still open, in Build order: unchosen Build start choices, then per level its ability increase,
    /// unspent skill points and empty slots, then the first ascension and path once their ranks are reached and the
    /// ranks' empty slots.
    pub todo: Vec<Todo>,
}

/// One open choice.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Todo {
    /// Its address, for `options`.
    pub choice: String,
    /// What it is, as the planner names it: "Level 1: Combat feat", "Level 3: 2 of 4 skill points unspent".
    pub label: String,
}

/// `new_build`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NewBuildArgs {
    /// A name for the Build, carried in its link.
    #[serde(default)]
    pub name: Option<String>,
    /// A planner link (`https://…/#b=<code>`) or a bare code, to start from that Build. Left out: an empty Build.
    #[serde(default)]
    pub link: Option<String>,
}

/// A result that is only the envelope.
#[derive(Debug, Serialize, JsonSchema)]
pub struct BuildResult {
    #[serde(flatten)]
    pub envelope: Envelope,
}

/// `sheet`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SheetArgs {
    /// The Build's code (a link works too).
    pub build: String,
    /// Also mark each held choice that can be withdrawn now (every choice that nothing later depends on). Costs a
    /// walk of the Build per choice.
    #[serde(default)]
    pub removable: bool,
}

/// A value for a choice, as `options` gives it and `choose` takes it:
/// - a key: a feat, option, mythic ability or mythic feat for a slot; a race or subrace, deity (or `atheism`),
///   alignment (`lawful-good`, …, `true-neutral`, …) or background key; an ability (`str`, `dex`, `con`, `int`,
///   `wis`, `cha`) for the racial ability or an increase; a path key; a skill key for a level's skill points
///   (`level.N.skill`: one rank more in it);
/// - `{key, parameter}`: a parameterised feat or option with its value (`{"key": "spell-focus", "parameter":
///   "conjuration"}`): a skill or weapon key, or a school, energy or armour type in lower case;
/// - `{class, archetype?, advances?}`: the class taken at a level, with the archetype when first taken, and for a
///   prestige class that asks, the classes it advances (`[]` for none);
/// - a number: a point-buy score (`start.score.int`), or the ranks a level adds to a skill (`level.N.skill.<skill>`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ChoiceValue {
    Number(u8),
    Key(String),
    Parameterised {
        key: String,
        parameter: String,
    },
    Class {
        class: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        archetype: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        advances: Option<Vec<String>>,
    },
}

impl ChoiceValue {
    /// Whether a value given for a choice names this offered one: keys and parameters ignoring case and spaces at the
    /// ends, numbers equal, and a class's archetype equal and its advanced classes the same set (left out matches
    /// only an offer that asks none). `choose` (ticket 109) finds the offer it applies with this.
    pub fn matches(&self, offered: &ChoiceValue) -> bool {
        let same = |a: &str, b: &str| a.trim().eq_ignore_ascii_case(b.trim());
        match (self, offered) {
            (ChoiceValue::Number(a), ChoiceValue::Number(b)) => a == b,
            (ChoiceValue::Key(a), ChoiceValue::Key(b)) => same(a, b),
            (
                ChoiceValue::Parameterised { key, parameter },
                ChoiceValue::Parameterised {
                    key: offered_key,
                    parameter: offered_parameter,
                },
            ) => same(key, offered_key) && same(parameter, offered_parameter),
            (
                ChoiceValue::Class {
                    class,
                    archetype,
                    advances,
                },
                ChoiceValue::Class {
                    class: offered_class,
                    archetype: offered_archetype,
                    advances: offered_advances,
                },
            ) => {
                let set = |list: &Option<Vec<String>>| {
                    list.as_ref().map(|list| {
                        let mut keys: Vec<String> =
                            list.iter().map(|key| key.trim().to_lowercase()).collect();
                        keys.sort();
                        keys.dedup();
                        keys
                    })
                };
                same(class, offered_class)
                    && match (archetype, offered_archetype) {
                        (Some(a), Some(b)) => same(a, b),
                        (None, None) => true,
                        _ => false,
                    }
                    && set(advances) == set(offered_advances)
            }
            _ => false,
        }
    }
}

/// One choice of a Build and what it holds.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PickLine {
    /// Its address.
    pub choice: String,
    /// What it is: "Race", "Class", "Combat feat", "Mythic ability", "Knowledge (Arcana) ranks".
    pub label: String,
    /// What it holds, as `options` gives values; `null` while open.
    pub value: Option<ChoiceValue>,
    /// What it holds, as the planner names it; `null` while open.
    pub name: Option<String>,
    /// With `removable: true`: whether it can be withdrawn now (nothing later depends on it). Left out for an open
    /// choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removable: Option<bool>,
}

/// `sheet`'s result.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SheetResult {
    #[serde(flatten)]
    pub envelope: Envelope,
    /// The Build's name, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The Build start's choices: race (and subrace), racial ability, faith, alignment, background, point buy.
    pub start: Vec<PickLine>,
    /// Point-buy points spent, of `point_buy_budget`.
    pub point_buy_spent: i8,
    pub point_buy_budget: u8,
    /// One per character level.
    pub levels: Vec<LevelLine>,
    /// MR 1–10, reached or not.
    pub ranks: Vec<RankLine>,
    /// The character level cap: 20, or 40 after a Legend switch.
    pub level_cap: u8,
    /// At the last level: the highest effective class level among casting classes (class levels plus prestige levels
    /// advancing them). Class levels only: no mythic, gear or buff bonus.
    pub caster_level: u8,
    /// At the last level: the highest spell level castable, by spell type.
    pub spell_levels: SpellLevels,
    /// At the last level: the class skills.
    pub class_skills: Vec<Named>,
    /// What the planner notes beside the Build: the background's note, favoured weapons, the choices the game asks
    /// for again each day.
    pub notes: Vec<String>,
    /// With `removable: true`: every choice that can be withdrawn now, by address (a class's archetype and advanced
    /// classes, and the last level, `level.N`, included).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removable: Option<Vec<String>>,
}

/// One character level of the sheet.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LevelLine {
    pub level: u8,
    pub class: Named,
    /// The class's archetype, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archetype: Option<Named>,
    /// The level this Level-up reaches in its class.
    pub class_level: u8,
    /// Base attack bonus at the end of the level.
    pub bab: u8,
    pub fort: u8,
    pub reflex: u8,
    pub will: u8,
    /// The character's own scores at the end of the level, feature bonuses included, gear never.
    pub scores: Scores,
    /// Skill points: available at this level and spent by it.
    pub skill_points: SkillPointsLine,
    /// Total ranks by skill key at the end of the level (skills with none left out).
    pub skill_ranks: BTreeMap<String, u8>,
    /// The feats first held at this level, granted or picked (a parameterised one with its value).
    pub feats: Vec<Named>,
    /// The features gained at this level (one granted again, as Sneak Attack, is listed again).
    pub features: Vec<Named>,
    /// The Mythic ranks reached at this level.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mythic_ranks: Vec<u8>,
    /// The level's choices by address: class, ability increase (every 4th level), ranks added, then its slots in the
    /// order they're checked.
    pub picks: Vec<PickLine>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Scores {
    pub str: u8,
    pub dex: u8,
    pub con: u8,
    pub int: u8,
    pub wis: u8,
    pub cha: u8,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SkillPointsLine {
    pub available: u8,
    pub spent: u8,
}

/// One Mythic rank of the sheet.
#[derive(Debug, Serialize, JsonSchema)]
pub struct RankLine {
    pub rank: u8,
    /// The character level its ascension comes at.
    pub character_level: u8,
    pub status: RankState,
    /// The path whose row it gives (MR 1–2: the first ascension's).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Named>,
    /// Its slots by address: the general mythic pick, then the path's.
    pub picks: Vec<PickLine>,
    /// The features its path row gives.
    pub features: Vec<Named>,
    /// The slots whose picks a Path switch took away, by address.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dropped: Vec<String>,
}

/// A Mythic rank's status.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RankState {
    Reached,
    /// Reached, then partly lost to a Path switch.
    PartlyDropped,
    /// Reached, then lost to a Path switch.
    Dropped,
    NotReached,
}
