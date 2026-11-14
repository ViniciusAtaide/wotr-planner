//! The tools' JSON: their arguments and their results. `rules` types aren't serde-shaped for an agent, so every
//! shape a tool takes or returns is defined here, and the tools build them from the engine's answers.
//!
//! Conventions: field names are `snake_case`; a result is always an object (a list sits in a named field, as
//! `search`'s `matches`); an optional field that doesn't apply is left out, except `prerequisite`, which is
//! `null` when there is none (`search`'s `compact` leaves it out); a reference to another item is a [`Named`] `{key, name}`. Every result about a Build
//! flattens the [`Envelope`] into it (`build`, `link`, `todo`); a choice is named by its address (`rules::Address`) and
//! holds a [`ChoiceValue`].

use std::collections::{BTreeMap, BTreeSet};

use rules::{Ability, ArchetypeKey, ClassKey, Pick, RaceKey, SubraceKey};
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
    /// A spell of the spell catalogue: reference data, which no Build choice names.
    Spell,
    /// A spell list: a class's, an archetype's or a Mythic path's spells by level.
    SpellList,
    /// A gear item: a belt, a ring, a robe, a manual or tome, …, which a Build holds only when the user states it
    /// (`gear.<place>`, `gear.usable.<item key>`).
    Gear,
}

impl Kind {
    /// The kind as the tools write it: `feat`, `mythic_feat`, `option`, …
    pub fn noun(self) -> &'static str {
        match self {
            Kind::Class => "class",
            Kind::Archetype => "archetype",
            Kind::Path => "path",
            Kind::Feat => "feat",
            Kind::MythicAbility => "mythic_ability",
            Kind::MythicFeat => "mythic_feat",
            Kind::SelectionOption => "option",
            Kind::Race => "race",
            Kind::Deity => "deity",
            Kind::Background => "background",
            Kind::Spell => "spell",
            Kind::SpellList => "spell_list",
            Kind::Gear => "gear",
        }
    }
}

/// `search`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// Words of a name, a key, or initials ("BFT" for Brown-Fur Transmuter), or words of what an item does ("DC"):
    /// a summary holding a query word as a whole word matches too, after the name matches. Case doesn't matter.
    /// May be empty or left out when `kinds` or `category` is given: then every item they allow matches.
    #[serde(default)]
    pub query: String,
    /// Only these kinds; every kind when left out.
    #[serde(default)]
    pub kinds: Option<Vec<Kind>>,
    /// Only items in this category, or in any of these (case doesn't matter): a feat's category ("Weapon", "Magic",
    /// …), a mythic ability's or feat's group ("Melee", "Ranged", …), an option's Feature selection by name ("Rage
    /// Power") or key, "Base class" or "Prestige class", a background's group, a spell's school ("Enchantment",
    /// "Universalist") or one of its descriptors, its subschool among them ("Compulsion", "Mind-Affecting"), a
    /// spell list's kind ("Class list", "Archetype list", "Mythic spellbook").
    #[serde(default)]
    pub category: Option<OneOrMany>,
    /// At most this many matches (default 10, or 50 for an empty query). Capped at 100: a larger `limit` returns
    /// 100, and `note` says so.
    #[serde(default)]
    pub limit: Option<u32>,
    /// Each match as its kind, key, name and category (and `matched`) only: no summary, parent, prerequisite or
    /// guide link (default false).
    #[serde(default)]
    pub compact: Option<bool>,
}

/// One text, or a list of them.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    pub fn into_vec(self) -> Vec<String> {
        match self {
            OneOrMany::One(one) => vec![one],
            OneOrMany::Many(many) => many,
        }
    }
}

/// `search`'s result.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SearchResult {
    /// The best match first: exact names and keys, then initials, then names starting with the query, then
    /// names holding its words, then any name or key containing it, then summaries holding a query word as a whole
    /// word (`matched: "summary"`), those holding every query word first. For an empty query, by kind, then name.
    pub matches: Vec<Hit>,
    /// How many items match in all; more than `matches` holds when `limit` cut the list.
    pub total: u32,
    /// When nothing matches: says so, and names the near names (or categories), if any is close. When every match
    /// is by its summary: says no name, key or initials matched. When `limit` left out summary matches: how many.
    /// When `limit` is over its cap of 100: says so.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// One item, as `search` lists it and `describe` heads it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Hit {
    pub kind: Kind,
    /// The key the other tools take.
    pub key: String,
    /// The name the planner shows.
    pub name: String,
    /// A short summary in the planner's own words, when the data has one (a background's: its note). Left out when
    /// empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// What it belongs to: an archetype's class, an option's Feature selection, a spell list's owner (keys).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Its group: "Base class" or "Prestige class", a feat's category, a mythic ability's or feat's group, an
    /// option's Feature selection, a background's group, a spell's school, a spell list's kind ("Class list",
    /// "Archetype list", "Mythic spellbook").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// A spell's descriptors as the guide prints them, its subschool among them ("Mind-Affecting", "Compulsion");
    /// `category` filters by them too. Left out when it has none, by `search`'s `compact`, and by `describe`, whose
    /// `facts` give them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptors: Option<Vec<String>>,
    /// What taking it needs, in the planner's words ("Spell Focus (Conjuration)"); `null` when nothing. In
    /// `search`, a long one has each group of many alternatives shortened ("one of 49 hexes (Air Barrier, Battle
    /// Master, Battle Ward, …)"), and `describe` gives it whole; left out by `search`'s `compact`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite: Option<Option<String>>,
    /// Where the guide describes it, for the user to read. Never fetch it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guide_url: Option<String>,
    /// `search` only: `"summary"` when the item matched by its summary's words rather than its name, key or
    /// initials. Left out for those.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched: Option<Matched>,
    /// A gear item's other effects, one short line each in the planner's own words: what it does only in some
    /// circumstance or by use. What it always does is in its `summary`. Text only: no number counts them. Left out when it has none, and by `search`'s
    /// `compact`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_effects: Option<Vec<String>>,
    /// A gear item's always-on lines that no number counts ("DR 3/-"): the part of its `summary` after its
    /// bonuses, which the planner's numbers include and none of these. Left out when it has none, and by
    /// `search`'s `compact`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncounted: Option<Vec<String>>,
    /// A gear item's addresses, where `choose` takes its key: `gear.belt`; `gear.ring.0` and `gear.ring.1`;
    /// `gear.body` for a robe or an armour; `gear.usable.<its key>` for a manual, tome or other usable item. Left
    /// out for other kinds, and by `search`'s `compact`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub places: Option<Vec<String>>,
}

/// How a `search` hit matched, when not by its name, key or initials.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Matched {
    /// A query word is a whole word of its summary.
    Summary,
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
    pub features: Option<Vec<Summarised>>,
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
    /// The spell lists a class, an archetype or a Mythic path casts from: those it owns and those its page says it
    /// uses (the sorcerer: the wizard's). An archetype's are the ones naming it; its class's are on the class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spell_lists: Option<Vec<Named>>,
    /// A spell's facts, from its block in the guide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facts: Option<SpellFacts>,
    /// Every spell list a spell is on, each with the spell's level there (twice when a list has it at two levels).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<Vec<OnList>>,
    /// A spell list's owner: the class, archetype or Mythic path whose page it's on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Caster>,
    /// The other classes and archetypes that cast from a spell list, where their pages say so.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_by: Option<Vec<Caster>>,
    /// A spell list's spells by spell level, lowest first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spells: Option<Vec<SpellLevelRow>>,
    /// What the guide says about how the item combines with others, in the planner's own words, for the few items
    /// that have a note ("the guide doesn't say" where it doesn't).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactions: Option<String>,
}

/// A spell's facts as its block in the guide prints them.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SpellFacts {
    /// "Enchantment", …; "Universalist" for a spell of no school (Dismiss Spell).
    pub school: String,
    /// "Mind-Affecting", "Compulsion", "Fear", an energy, …; the enchantment subschools (Compulsion, Charmed) and
    /// Summoning are among them.
    pub descriptors: Vec<String>,
    /// Whom or what area it reaches, and its range.
    pub target: String,
    /// "1 round / level"; left out where the block has no duration (instantaneous spells).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    /// "Will negates", "Reflex half", "N/A", …
    pub saving_throw: String,
    /// Whether spell resistance applies.
    pub spell_resistance: bool,
    /// "Standard Action", "Full Round", "Move Action", "Swift Action", "Free Action".
    pub casting_time: String,
    /// The material component, when the block names one ("Diamond dust x10").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<String>,
}

/// A spell list a spell is on, at its level there.
#[derive(Debug, Serialize, JsonSchema)]
pub struct OnList {
    /// The list's key, for `describe {kind: spell_list}`.
    pub key: String,
    pub name: String,
    /// The spell's level on this list: 0 to 9 on a class's or an archetype's list, 1 to 10 in a Mythic spellbook.
    pub level: u8,
    pub owner: Caster,
    /// The other classes and archetypes that cast from the list.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub used_by: Vec<Caster>,
    /// The table's footnote when only some of the list's casters get the spell ("Flame Dancer only", "Uncorrupted
    /// only": a Gold Dragon story branch). Left out for all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<String>,
}

/// A class, an archetype or a Mythic path that owns or uses a spell list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Caster {
    /// `class`, `archetype` or `path`.
    pub kind: Kind,
    pub key: String,
    pub name: String,
}

/// One spell level of a spell list.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SpellLevelRow {
    pub level: u8,
    pub spells: Vec<ListedSpell>,
}

/// A spell as a list names it: by key and name, with its school and descriptors (`describe {kind: spell}` gives
/// the rest).
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListedSpell {
    pub key: String,
    pub name: String,
    pub school: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub descriptors: Vec<String>,
    /// The table's footnote when only some of the list's casters get it ("Flame Dancer only").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<String>,
}

/// Another item, by key and name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Named {
    pub key: String,
    pub name: String,
}

/// A feature a Build gains, by key and name, with how many times when that is more than once (the Trickster's third
/// Mythic rank gives Sneak Attack twice).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Gained {
    pub key: String,
    pub name: String,
    /// How many times it is gained at once; left out for one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<u8>,
}

/// A feature or selection option as `describe` lists it: by key and name, with its summary when the data has one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Summarised {
    pub key: String,
    pub name: String,
    /// A short summary in the planner's own words; left out when the data has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
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
    pub features: Vec<Summarised>,
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
    pub features: Vec<Summarised>,
    pub feat_slots: Vec<String>,
    pub selections: Vec<Named>,
}

/// A Feature selection and its options.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Selection {
    pub key: String,
    pub name: String,
    /// A short summary in the planner's own words; left out when the data has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub options: Vec<Summarised>,
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
    /// The base weapon of each level's attack line (a weapon key: `greatsword`), from the planner's engine. Left
    /// out: the weapon of the Build's earliest Weapon Focus (by level; within a level, the first weapon key), and no
    /// attack line without one. Not part of the Build.
    #[serde(default)]
    pub weapon: Option<String>,
    /// Also give each level the caster line for these spells (`{"school": "enchantment", "subschool":
    /// "compulsion", "metamagic": true}`), from the planner's engine. Not part of the Build.
    #[serde(default)]
    pub caster: Option<crate::caster::CasterArgs>,
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
/// - a gear item's key, for its place (`gear.belt`: `belt-of-incredible-dexterity-2`; `none` takes the place's item
///   off) or for its own `gear.usable.<item key>`;
/// - a number: a point-buy score (`start.score.int`), the ranks a level adds to a skill (`level.N.skill.<skill>`),
///   or the character level a Mythic rank is taken at (`mythic.R.level`);
/// - `{skill: ranks}`: for `level.N.skill`, the ranks the level adds to each skill (`{"athletics": 1, "perception":
///   1}`), each as `level.N.skill.<skill>` takes it, in one step.
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
    Ranks(BTreeMap<String, u8>),
}

impl ChoiceValue {
    /// A pick's value: its key, with its parameter's key ([`rules::Parameter::value_key`]) when it names one. The
    /// same for what the sheet shows a choice holds and for what `options` offers, so a value copied from either
    /// matches.
    pub(crate) fn pick(pick: &Pick) -> Self {
        match pick.parameter() {
            Some(parameter) => ChoiceValue::Parameterised {
                key: pick.key().to_owned(),
                parameter: parameter.value_key().to_owned(),
            },
            None => ChoiceValue::Key(pick.key().to_owned()),
        }
    }

    /// An ability's value: its key, `str`, `dex`, …
    pub(crate) fn ability(ability: Ability) -> Self {
        ChoiceValue::Key(ability.key().to_owned())
    }

    /// The race picker's value: the subrace's key when there is one (it starts with the race's), else the race's.
    pub(crate) fn race(race: &RaceKey, subrace: Option<&SubraceKey>) -> Self {
        ChoiceValue::Key(subrace.map_or(race.as_str(), SubraceKey::as_str).to_owned())
    }

    /// A class's value: the class, its archetype, and for a prestige class that asks, the classes it advances.
    pub(crate) fn class(
        class: &ClassKey,
        archetype: Option<&ArchetypeKey>,
        advances: Option<&BTreeSet<ClassKey>>,
    ) -> Self {
        ChoiceValue::Class {
            class: class.as_str().to_owned(),
            archetype: archetype.map(|key| key.as_str().to_owned()),
            advances: advances.map(|set| set.iter().map(|key| key.as_str().to_owned()).collect()),
        }
    }

    /// Whether a value given for a choice names this offered one: keys and parameters ignoring case and spaces at the
    /// ends, numbers equal, and a class's archetype equal and its advanced classes the same set (left out matches
    /// only an offer that asks none). `choose` finds the offer it applies with this.
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
    /// With `removable: true`: whether `withdraw` can take it back now (nothing later depends on it). Left out for an
    /// open choice, and for a level's class and the racial ability, which aren't withdrawn (`choose` another value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removable: Option<bool>,
    /// For a choice the Build can make only because of its gear: the items it needs ("With Belt of Incredible
    /// Dexterity +2"), as `gear_dependent` lists it. Left out for every other.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_gear: Option<String>,
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
    /// At the last level: initiative, the saves and each skill's total, each with what it counts. Unconditional
    /// numbers only (`totals_leave_out` lists what they leave out). Absent before the first level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totals: Option<crate::totals::TotalsShape>,
    /// What no total counts: gear, buffs, abilities switched on in a fight, bonuses against one kind of effect or for
    /// one use of a skill, bonuses that grow with class level.
    pub totals_leave_out: Vec<String>,
    /// What the planner notes beside the Build: the background's note, favoured weapons, the choices the game asks
    /// for again each day.
    pub notes: Vec<String>,
    /// With `removable: true`: every choice `withdraw` can take back now, by address (a class's archetype and
    /// advanced classes, and the last level, `level.N`, included; a level's class never: `choose` changes it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removable: Option<Vec<String>>,
    /// With an attack line: what no attack line counts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack_leaves_out: Option<Vec<String>>,
    /// With a `caster`: what no caster line counts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caster_leaves_out: Option<Vec<String>>,
    /// The gear the Build holds, as the user stated it: each item at its address, with its summary and other effects.
    /// Left out when it holds none (a Build has no gear unless it's chosen).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gear: Vec<GearLine>,
    /// The choices the Build can make only because of its gear, each with the items it needs: an item one of them
    /// needs can't be withdrawn, or replaced by an item that doesn't keep the choice, until that choice is withdrawn.
    /// Left out when there are none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gear_dependent: Vec<GearDependentLine>,
    /// The numbers without the gear, where the gear changes them: every other number of the sheet counts the Build's
    /// gear. They are the numbers of the same Build with no gear at all, so a choice only gear allows counts for
    /// nothing in them. Left out for a Build without gear, or whose gear changes none of them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub naked: Option<NakedNumbers>,
}

/// An item the Build holds.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GearLine {
    /// Its address: `gear.belt`, `gear.ring.0`, `gear.usable.<item key>`, …
    pub choice: String,
    /// Where it is: "Gear: Belt", "Gear: Usable: Manual of Gainful Exercise +2", …
    pub label: String,
    /// Its key, as `choose` takes it.
    pub value: String,
    pub name: String,
    /// Its always-on bonuses, which every number counts ("Dex +2 enhancement"), then its other always-on effects,
    /// which no number counts ("DR 3/-").
    pub summary: String,
    /// What it does only in some circumstance or by use, one short line each: text only, in no number.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub other_effects: Vec<String>,
    /// The always-on lines of `summary` that no number counts (the ones after its bonuses); the planner's numbers
    /// include the item's bonuses and none of these.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub uncounted: Vec<String>,
    /// With `removable: true`: whether `withdraw` can take it off now (no choice needs it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removable: Option<bool>,
}

/// A choice the Build can make only because of its gear.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct GearDependentLine {
    /// Its address.
    pub choice: String,
    /// What it is, with its place: "Level 19: Feat".
    pub label: String,
    /// What it holds: "Greater Two-Weapon Fighting".
    pub name: String,
    /// The items it needs, in the planner's words: "With Belt of Incredible Dexterity +2"; "With A or B" when either
    /// is enough.
    pub needs: String,
    /// The keys of the items `needs` names.
    pub items: Vec<String>,
}

/// The sheet's numbers without the Build's gear, only where they differ from those with it.
#[derive(Debug, Serialize, JsonSchema)]
pub struct NakedNumbers {
    /// By character level, the levels where any differs.
    pub levels: Vec<NakedLevel>,
    /// The last level's totals without gear, when any differs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totals: Option<crate::totals::TotalsShape>,
}

/// One level's numbers without gear, each left out where the gear doesn't change it.
#[derive(Debug, Serialize, JsonSchema)]
pub struct NakedLevel {
    pub level: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scores: Option<Scores>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<crate::attack::AttackLineShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caster: Option<crate::caster::CasterLineShape>,
}

/// Which numbers `compare` and `improve` count: the Builds' without gear (the default), or with the gear each holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Numbers {
    /// Without gear: the numbers of each Build with no gear at all.
    Naked,
    /// With the gear each Build holds (`with_gear: true`).
    WithGear,
}

impl Numbers {
    pub(crate) fn from_flag(with_gear: Option<bool>) -> Self {
        match with_gear {
            Some(true) => Self::WithGear,
            _ => Self::Naked,
        }
    }
}

/// A Build summary's other numbers: with gear beside naked ones, or naked beside ones with gear, each left out where
/// the gear doesn't change it.
#[derive(Debug, Serialize, JsonSchema)]
pub struct OtherNumbers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scores: Option<Scores>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<crate::attack::AttackLineShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caster: Option<crate::caster::CasterLineShape>,
    /// With `by`: the number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i16>,
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
    /// The scores at the end of the level: the point buy, racial and level increases, feature bonuses and the
    /// Build's gear (`naked` gives them without the gear where it changes them). A level's skill points never count
    /// gear.
    pub scores: Scores,
    /// Skill points: available at this level and spent by it.
    pub skill_points: SkillPointsLine,
    /// Total ranks by skill key at the end of the level (skills with none left out).
    pub skill_ranks: BTreeMap<String, u8>,
    /// The feats first held at this level, granted or picked (a parameterised one with its value).
    pub feats: Vec<Named>,
    /// The features gained at this level; one gained more than once at the level (the Trickster's Sneak Attack at
    /// MR 3) says `times`.
    pub features: Vec<Gained>,
    /// The Mythic ranks reached at this level.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mythic_ranks: Vec<u8>,
    /// The attack line at the end of the level, for the `weapon` given or the Build's Weapon Focus weapon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<crate::attack::AttackLineShape>,
    /// With a `caster`: the caster line at the end of the level, once a class casts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caster: Option<crate::caster::CasterLineShape>,
    /// The level's choices by address: class, ability increase (every 4th level), ranks added, then its slots in the
    /// order they're checked.
    pub picks: Vec<PickLine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
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
    /// The character level its ascension is taken at: the rank table's level, or a later one the Build places it at
    /// (`mythic.R.level`; a rank placed later takes the ranks above it along).
    pub character_level: u8,
    /// For a rank taken later than the rank table's level: that level, the earliest the rank ever comes at. A rank
    /// that is never reached has none, but for MR 8 under a Legend switch taken later (the switch's level).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_level: Option<u8>,
    pub status: RankState,
    /// The path whose row it gives (MR 1–2: the first ascension's).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Named>,
    /// Its slots by address: the general mythic pick, then the path's.
    pub picks: Vec<PickLine>,
    /// The features its path row gives, one gained more than once with `times`.
    pub features: Vec<Gained>,
    /// The slots whose picks a Path switch took away, by address.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dropped: Vec<String>,
}

/// A Mythic rank's status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RankState {
    Reached,
    /// Reached, then partly lost to a Path switch.
    PartlyDropped,
    /// Reached, then lost to a Path switch.
    Dropped,
    NotReached,
}

// ---- options -----------------------------------------------------------------------------------------------------

/// `options`' arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OptionsArgs {
    /// The Build's code (a link works too).
    pub build: String,
    /// The choice's address, as `sheet` and `todo` print it: `start.race`, `level.4.class`,
    /// `slot.feat.3.general.0`, …; a gear place (`gear.belt`, `gear.ring.0`, `gear.body`, …) or a usable item
    /// (`gear.usable.<item key>`).
    pub choice: String,
    /// Only options whose name or value holds every word of this (case doesn't matter).
    #[serde(default)]
    pub query: Option<String>,
    /// Only the options that can be taken now (default true); the others are counted in `closed`. False lists them
    /// too, each with its issue and reason.
    #[serde(default)]
    pub only_open: Option<bool>,
    /// For a slot: list a parameterised feat or option once per value (`{key, parameter}`), each with its own
    /// status (default false: once, with `takes` saying what a value is).
    #[serde(default)]
    pub values: Option<bool>,
    /// Each option without its summary and a gear item's other effects (default false).
    #[serde(default)]
    pub compact: Option<bool>,
    /// Also return the envelope, `build`, `link` and `todo` (default false: `options` changes nothing, so the
    /// Build's code and its open choices are those given).
    #[serde(default)]
    pub envelope: Option<bool>,
}

/// `options`' result.
#[derive(Debug, Serialize, JsonSchema)]
pub struct OptionsResult {
    /// Only with `envelope: true`.
    #[serde(flatten)]
    pub envelope: Option<Envelope>,
    /// The address the options are for, as the server writes it.
    pub choice: String,
    /// In the planner's order.
    pub options: Vec<OptionLine>,
    /// How many options `only_open` hid: those that can't be taken now.
    pub closed: u32,
}

/// One option for a choice.
#[derive(Debug, Serialize, JsonSchema)]
pub struct OptionLine {
    /// What `choose` takes for it.
    pub value: ChoiceValue,
    /// As the planner names it.
    pub name: String,
    /// A short summary in the planner's own words (a background's: its note; a gear item's: its bonuses, then its
    /// other always-on effects); left out when the data has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// A gear item's other effects, one short line each: what it does only in some circumstance or by use; text
    /// only, in no number. Left out when it has none, and by `compact`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_effects: Option<Vec<String>>,
    /// A gear item's always-on lines that no number counts: the part of its `summary` after its bonuses. The
    /// planner's numbers include the item's bonuses and none of these. Left out when it has none, and by `compact`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncounted: Option<Vec<String>>,
    pub status: OptionStatus,
    /// The filter group it's under: a feat's category, an option's Feature selection, "Base class", …
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// For an entry listed without its values (`values: false`): what a value is ("a weapon"). Ask again with
    /// `values: true` for the values and their statuses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takes: Option<String>,
    /// The Build holds it now.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub current: bool,
}

/// `"ok"` when it can be taken now; `{"with_gear": …}` when it can be taken now only because of the Build's gear;
/// `{"leans_on_gear": …}` when taking it would leave other choices allowed only because of the gear; else why not.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum OptionStatus {
    Open(Open),
    /// Open, only because of the Build's gear: the items it needs ("With Belt of Incredible Dexterity +2"). Once
    /// taken, those items can't come off (or give way to items that don't keep it) until it is withdrawn.
    WithGear {
        with_gear: String,
    },
    /// Open, but taking it leaves these choices allowed only because of the gear ("Greater Two-Weapon Fighting
    /// would then be allowed only because of the gear"): legal, but not safe if the gear changes.
    LeansOnGear {
        leans_on_gear: String,
    },
    Closed {
        /// The engine's issue: `unmet`, `already_held`, `max_times`, `not_allowed`, `class_level_cap`,
        /// `not_from_path`, `deity_alignment`, `needs_faith`, `deity_domain`, `breaks_later`.
        issue: String,
        /// The planner's reason ("Needs Power Attack, BAB +4 (have +1)").
        reason: String,
    },
}

/// The status of an option that can be taken now.
#[derive(Debug, Serialize, JsonSchema)]
pub enum Open {
    #[serde(rename = "ok")]
    Ok,
}

// ---- choose and withdraw -----------------------------------------------------------------------------------------

/// `choose`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChooseArgs {
    /// The Build's code (a link works too).
    pub build: String,
    /// The choices to make, in order. Each is checked against the Build as the steps before it left it; the first
    /// one refused stops the rest.
    pub steps: Vec<ChooseStep>,
}

/// One choice to make.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChooseStep {
    /// The choice's address, as `sheet`, `todo` and `options` print it: `slot.feat.3.general.0`, `level.4.class`
    /// (one past the last level adds a level), `start.score.int`, `level.2.skill.athletics`, `mythic.path`, …
    pub choice: String,
    /// The value, as `options` lists it (with `values: true` for a parameterised feat's `{key, parameter}`). For a
    /// point-buy score or a skill's ranks at a level, the number to reach: it's reached one step at a time, each
    /// step checked. For `level.N.skill`, a map `{skill: ranks}` sets several skills' ranks at that level in one
    /// step, each rank checked.
    pub value: ChoiceValue,
}

/// `choose`'s result.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ChooseResult {
    /// The Build after the steps applied: the input's code when none was.
    #[serde(flatten)]
    pub envelope: Envelope,
    /// How many steps were applied, from the first (a step that changes nothing counts).
    pub applied: u32,
    /// The step that was refused, which stopped the rest. Left out when every step was applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected: Option<Rejected>,
    /// The items a step took from another gear place: a ring chosen at `gear.ring.1` while it was worn at
    /// `gear.ring.0` moves, and `gear.ring.0` is then empty. Left out when none moved.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub moved: Vec<MovedItem>,
    /// The applied steps that left choices allowed only because of the Build's gear (a Dexterity lowered under a
    /// belt, with a feat that needs it): applied all the same, each with those choices and the items they now
    /// need. Left out when none did.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub leans_on_gear: Vec<LeaningStep>,
}

/// An item a `choose` step moved from another gear place.
#[derive(Debug, Serialize, JsonSchema)]
pub struct MovedItem {
    /// The step's index in `steps`, from 0.
    pub step: u32,
    /// The step's address: where the item is now.
    pub choice: String,
    /// The item's key and name.
    pub item: String,
    pub name: String,
    /// Where it was, now empty.
    pub from: String,
}

/// A `choose` step that left choices allowed only because of the gear.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LeaningStep {
    /// The step's index in `steps`, from 0.
    pub step: u32,
    /// The step's address, as given.
    pub choice: String,
    /// The choices it left allowed only because of the gear, as `sheet`'s `gear_dependent` lists them.
    pub leaning: Vec<GearDependentLine>,
}

/// `withdraw`'s result: the envelope, and what the withdrawal left leaning on the gear.
#[derive(Debug, Serialize, JsonSchema)]
pub struct WithdrawResult {
    #[serde(flatten)]
    pub envelope: Envelope,
    /// The choices the withdrawal left allowed only because of the Build's gear (an ability increase in Dexterity
    /// withdrawn under a belt), with the items they now need, as `sheet`'s `gear_dependent` lists them. The
    /// withdrawal is made all the same. Left out when there are none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub leans_on_gear: Vec<GearDependentLine>,
}

/// A refused step of `choose`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Rejected {
    /// Its index in `steps`, from 0 (always equal to `applied`).
    pub step: u32,
    /// Its address, as given.
    pub choice: String,
    /// Why: the engine's issue (`unmet`, `already_held`, `max_times`, `not_allowed`, `class_level_cap`,
    /// `not_from_path`, `deity_alignment`, `needs_faith`, `deity_domain`, `breaks_later`), or the server's
    /// (`not_an_option`, `needs_parameter`, `no_such_choice`, `internal`).
    pub issue: String,
    /// The planner's reason; for `not_an_option`, the closest values offered; for `needs_parameter`, what the value
    /// is and the open ones.
    pub reason: String,
}

/// `validate`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateArgs {
    /// The whole Build, declared: each choice's address and its value, as `choose` takes them (`{"start.race":
    /// "human", "start.faith": "atheism", "start.alignment": "true-neutral", "start.score.str": 16,
    /// "level.1.class": {"class": "fighter"}, "level.1.skill": {"athletics": 1}, "slot.feat.1.general.0":
    /// "power-attack", "mythic.path": "aeon", …}`). The order doesn't matter. A level is declared by its class
    /// (`level.N.class`, every level from 1 with no gap); a class's archetype and advanced classes come with it,
    /// at any of its levels (or the archetype at `class.<class>.archetype`), the same at each. A skill's ranks at a
    /// level go in `level.N.skill`'s map or at `level.N.skill.<skill>`. Choices left out are open, not mistakes.
    pub choices: BTreeMap<String, ChoiceValue>,
}

/// `validate`'s result: the Build, when every choice holds; otherwise every mistake, as a tree of causes.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ValidateResult {
    /// Every declared choice holds: the Build's envelope (`build`, `link`, `todo`), as `choose` would have made it.
    Valid {
        #[serde(flatten)]
        envelope: Envelope,
    },
    /// Some choices fail: no Build code. `failures` are the root causes, each with the failures it causes as its
    /// `children`.
    Invalid {
        /// Every failure, children included.
        count: u32,
        failures: Vec<FailureLine>,
    },
}

/// One declared choice that fails, and the later failures it causes.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FailureLine {
    /// Its address, as declared.
    pub choice: String,
    /// What it is: "Level 1: Combat feat", "Race", "Level 4: Class".
    pub label: String,
    /// Why, as a code: the engine's issue (`unmet`, `already_held`, `max_times`, `not_allowed`, `unknown_key`,
    /// `not_reached`, `class_level_cap`, `not_from_path`, `deity_alignment`, `needs_faith`, `deity_domain`), or the
    /// declaration's (`no_such_choice`, `not_an_option`, `needs_parameter`, `declared_twice`, `fixed_rule`,
    /// `missing_level`, `not_reached`).
    pub issue: String,
    /// The planner's reason, in full.
    pub reason: String,
    /// For `unmet`: each prerequisite not met, in short ("Strength 13 (have 12)", "Power Attack"), as `reason` words
    /// it: of an "or", only the alternatives the character's classes give.
    pub needs: Vec<String>,
    /// Other failed choices its prerequisites name, besides the one it sits under, by address: fix them too.
    pub also_needs: Vec<String>,
    /// The failures this one causes: they often go once it's fixed.
    pub children: Vec<FailureLine>,
}

/// `withdraw`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WithdrawArgs {
    /// The Build's code (a link works too).
    pub build: String,
    /// The address of the choice to take back: a slot, `level.N` (the last level), `level.N.increase`,
    /// `level.N.skill.<skill>`, `class.<class>.archetype|advances`, `start.race|subrace|faith|alignment|background`,
    /// `start.score.<ability>` (back to 7), `mythic.first-ascension|path|switch`, `mythic.R.level` (a Mythic rank
    /// taken later: back to the earliest level it comes at), a gear place (`gear.belt`, …) or a usable item
    /// (`gear.usable.<item key>`).
    pub choice: String,
}

// ---- compare -----------------------------------------------------------------------------------------------------

/// `compare`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompareArgs {
    /// 1 to 4 Build codes (links work too), for example two branches of one Build; one gets its summary alone. Each
    /// is opened checked, and every per-Build list in the result follows this order.
    pub builds: Vec<String>,
    /// The base weapon of each Build's summary's attack line (a weapon key: `greatsword`). Left out: each Build's
    /// own Weapon Focus weapon (its earliest Weapon Focus by level; within a level, the first weapon key), and no
    /// attack line for a Build without one.
    #[serde(default)]
    pub weapon: Option<String>,
    /// Also give each Build's summary the caster line for these spells (`{"school": "enchantment", "subschool":
    /// "compulsion"}`).
    #[serde(default)]
    pub caster: Option<crate::caster::CasterArgs>,
    /// Order the summaries by this number at each Build's last level, highest first: `{"caster_line": {"school":
    /// "enchantment", "subschool": "compulsion"}}` (`number`: `dc`, the default, `spell_penetration` or
    /// `concentration`) or `{"attack_line": {"weapon": "greatsword"}}` (`weapon` left out: each Build's own Weapon
    /// Focus weapon; `number`: `attack`, the default, or `damage`). Every per-Build list then follows that order.
    #[serde(default)]
    pub by: Option<crate::rank::NumberArgs>,
    /// Count each Build's gear in the summaries' numbers, the differences and `by` (default false: they are naked,
    /// the numbers of each Build with no gear; each summary's `with_gear` then gives the numbers its gear changes).
    /// `compare` never adds gear: each Build counts only the gear it holds.
    #[serde(default)]
    pub with_gear: Option<bool>,
}

/// `compare`'s result: each Build's summary, then only what differs between them, all read from the planner's
/// sheets. Facts only: nothing here says which Build is better.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CompareResult {
    /// One per Build, in the order given.
    pub builds: Vec<BuildSummary>,
    /// The Build start's choices that differ (race, faith, alignment, background, point-buy scores, …).
    pub start: Vec<PickDiff>,
    /// The character levels where anything differs, in level order. A level that only some Builds reach differs.
    pub levels: Vec<LevelDiff>,
    /// The Mythic ranks whose status, path or choices differ, in rank order.
    pub ranks: Vec<RankDiff>,
    /// By Mythic rank: the path features each Build's sheet lists at that rank and some other Build's doesn't.
    pub path_features: Vec<FeaturesDiff>,
    /// With an attack line: what no attack line counts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack_leaves_out: Option<Vec<String>>,
    /// With a `caster`: what no caster line counts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caster_leaves_out: Option<Vec<String>>,
    /// With `by`: the number the Builds are ordered by, the order, and what the number leaves out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<crate::rank::RankedBy>,
    /// Which numbers the summaries, the differences and `by` count: `"naked"` (the default: each Build with no gear)
    /// or `"with_gear"` (each with the gear it holds).
    pub numbers: Numbers,
    /// The gear places and usable items whose item differs, by address.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gear: Vec<PickDiff>,
}

/// One Build's sheet in short, at its last level.
#[derive(Debug, Serialize, JsonSchema)]
pub struct BuildSummary {
    /// Its code.
    pub build: String,
    /// Its planner link, for the user.
    pub link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// How many choices are still open: the length of its `todo`.
    pub open_choices: u32,
    pub character_level: u8,
    /// The classes taken, in order of first entry, with their levels.
    pub classes: Vec<ClassSummary>,
    pub bab: u8,
    pub fort: u8,
    pub reflex: u8,
    pub will: u8,
    /// As `numbers` counts them.
    pub scores: Scores,
    /// Class levels only (no mythic, gear or buff bonus), as `sheet` gives it.
    pub caster_level: u8,
    pub spell_levels: SpellLevels,
    /// The Mythic rank reached (3 after a Legend switch).
    pub mythic_rank: u8,
    /// The first ascension's path (MR 1–2), when chosen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_ascension: Option<Named>,
    /// The Mythic path chosen at MR 3.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Named>,
    /// The Path switch at MR 8.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch: Option<Named>,
    /// The character level cap: 20, or 40 after a Legend switch.
    pub level_cap: u8,
    /// The attack line at the last level, for the `weapon` given or this Build's Weapon Focus weapon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<crate::attack::AttackLineShape>,
    /// With a `caster`: the caster line at the last level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caster: Option<crate::caster::CasterLineShape>,
    /// With `by`: the number at the last level; absent when the Build has no such line (no casting class, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i16>,
    /// With `by`: this Build's place in the `builds` given, from 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<u32>,
    /// The gear it holds, by key and name. Left out when none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gear: Vec<Named>,
    /// The choices it can make only because of its gear. Left out when none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gear_dependent: Vec<GearDependentLine>,
    /// With `numbers: "naked"`: the numbers with its gear, where the gear changes them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_gear: Option<OtherNumbers>,
    /// With `numbers: "with_gear"`: the numbers without its gear, where the gear changes them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub naked: Option<OtherNumbers>,
}

/// One class of a Build.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ClassSummary {
    pub class: Named,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archetype: Option<Named>,
    pub levels: u8,
    /// A prestige class's advanced spellcasting classes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub advances: Vec<Named>,
}

/// A choice that holds different things in different Builds.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PickDiff {
    /// Its address.
    pub choice: String,
    /// What it is: "Class", "Mythic ability", "Lich Powers", …; each Build's label once, joined by " / ", where they
    /// differ ("Lich Powers / Enforcing Gaze").
    pub label: String,
    /// One per Build: what it holds, as the planner names it; `null` where it's open or the Build lacks it.
    pub names: Vec<Option<String>>,
}

/// A number that differs between the Builds.
#[derive(Debug, Serialize, JsonSchema)]
pub struct NumberDiff {
    /// `class_level`, `bab`, `fort`, `reflex`, `will`, `str`, `dex`, `con`, `int`, `wis` or `cha`.
    pub what: String,
    /// One per Build; `null` where the Build lacks the level.
    pub values: Vec<Option<u8>>,
}

/// What differs at one character level.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LevelDiff {
    pub level: u8,
    /// Numbers at the end of the level.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub numbers: Vec<NumberDiff>,
    /// The level's choices: class, ability increase, ranks added, slots.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub picks: Vec<PickDiff>,
    /// One list per Build: the feats it first holds at this level that some other Build doesn't.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub feats: Vec<Vec<Named>>,
    /// One list per Build: the features it gains at this level that some other Build doesn't, apart from its path
    /// rows' features, which are in `path_features`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<Vec<Gained>>,
}

/// What differs at one Mythic rank, apart from its path features (those are in `path_features`).
#[derive(Debug, Serialize, JsonSchema)]
pub struct RankDiff {
    pub rank: u8,
    /// One per Build, when they differ: the character level the rank is taken at (`mythic.R.level`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_level: Option<Vec<u8>>,
    /// One per Build, when they differ.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<RankState>>,
    /// One per Build, when they differ: the path whose row the rank gives (MR 1–2: the first ascension's).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<Option<String>>>,
    /// The rank's choices: the general mythic pick and the path's.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub picks: Vec<PickDiff>,
}

/// One Mythic rank's path features that not every Build has.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FeaturesDiff {
    pub rank: u8,
    /// One list per Build: the features its path row gives at this rank that some other Build's doesn't.
    pub only: Vec<Vec<Gained>>,
}
