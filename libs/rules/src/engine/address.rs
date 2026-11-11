//! One name per choice (ticket 152): a choice's **Address**, the text an agent copies between the MCP server's tools,
//! read and written here; its label, the words the sheet, the pickers and the MCP server name it by; and the key of
//! each value that has no game-data key (an ability, an alignment, a faith, a parameter such as a school), written
//! out rather than derived from the value's display name, so renaming a name never breaks a key.
//!
//! Each address names one [`Choice`] the engine knows, except the racial ability, which the engine folds into the
//! race's: it is an address beside the choices, so the engine's `Choice` stays as the walk needs it.
//!
//! The grammar (ticket 104's, refined in ticket 107):
//!
//! - The Build start: `start.race`, `start.subrace`, `start.faith`, `start.alignment`, `start.background`,
//!   `start.racial-ability`, `start.score.<ability>` (`str`, `dex`, `con`, `int`, `wis`, `cha`).
//! - A level: `level.N` (the level itself: withdrawing it removes the last level), `level.N.class` (one past the
//!   end adds a level), `level.N.increase`, `level.N.skill` (the level's skill points as a whole),
//!   `level.N.skill.<skill>`.
//! - A class's own choices, made when it's first taken: `class.<class>.archetype`, `class.<class>.advances`.
//! - The Mythic track: `mythic.first-ascension`, `mythic.path`, `mythic.switch`.
//! - A slot, one form per [`SlotId`] variant: `slot.feat.N.<race|class|general>.I`, `slot.selection.N.I`,
//!   `slot.option-feat.N.I`, `slot.option-selection.N.I`, `slot.feat-selection.N.I` (N a character level), and
//!   `slot.mythic.R`, `slot.mythic-extra.R`, `slot.path-feat.R.I`, `slot.path-selection.R.I`,
//!   `slot.mythic-selection.R.I` (R a Mythic rank). I counts from 0, levels and ranks from 1.

use std::fmt;
use std::str::FromStr;

use super::{Engine, FIRST_ASCENSION_RANK, PATH_CHOICE_RANK, PATH_SWITCH_RANK, Slot};
use crate::build::{AbilityScores, Faith, FeatSource, SlotId};
use crate::choice::Choice;
use crate::game_data::{Ability, Alignment};
use crate::keys::{ClassKey, DeityKey, SkillKey};
use crate::parameter::{ArmourType, Element, Energy, Parameter, ParameterKind, SpellSchool, Stat};

/// One choice of a Build, by where it sits.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Address {
    Race,
    /// The subrace alone (the race picker chooses race and subrace together; withdrawing it keeps the race).
    Subrace,
    /// The deity followed, or atheism.
    Faith,
    Alignment,
    Background,
    /// Where a "+2 to one ability score" racial bonus goes: the one address with no [`Choice`] of its own.
    RacialAbility,
    /// One point-buy score.
    Score(Ability),
    /// A character level itself.
    Level(u8),
    /// The class taken at a character level.
    Class(u8),
    /// A level's +1 ability increase (every 4th level).
    Increase(u8),
    /// A level's skill points as a whole: one rank more in any skill.
    SkillPoints(u8),
    /// The ranks a level adds to one skill.
    Skill(u8, SkillKey),
    /// A class's archetype.
    Archetype(ClassKey),
    /// The classes a prestige class's levels advance.
    Advances(ClassKey),
    FirstAscension,
    Path,
    Switch,
    Slot(SlotId),
}

/// Where a choice sits: the Build start, a character level or a Mythic rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Start,
    Level(u8),
    Rank(u8),
}

impl Place {
    /// The place as a picker's heading says it: "Build start", "Level 4", "Mythic rank 3".
    pub fn heading(self) -> String {
        match self {
            Place::Rank(rank) => format!("Mythic rank {rank}"),
            place => place.to_string(),
        }
    }
}

impl fmt::Display for Place {
    /// The place as a label begins: "Build start", "Level 4", "MR 3".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Place::Start => write!(f, "Build start"),
            Place::Level(level) => write!(f, "Level {level}"),
            Place::Rank(rank) => write!(f, "MR {rank}"),
        }
    }
}

impl Address {
    /// The engine's choice at this address; `None` for the racial ability, which the engine keeps with the race.
    pub fn choice(&self) -> Option<Choice> {
        Some(match self {
            Address::Race => Choice::Race,
            Address::Subrace => Choice::Subrace,
            Address::Faith => Choice::Deity,
            Address::Alignment => Choice::Alignment,
            Address::Background => Choice::Background,
            Address::RacialAbility => return None,
            Address::Score(ability) => Choice::AbilityScore { ability: *ability },
            Address::Level(level) => Choice::CharacterLevel { level: *level },
            Address::Class(level) => Choice::Class { level: *level },
            Address::Increase(level) => Choice::AbilityIncrease { level: *level },
            Address::SkillPoints(level) => Choice::SkillPoints { level: *level },
            Address::Skill(level, skill) => Choice::SkillRanks {
                level: *level,
                skill: skill.clone(),
            },
            Address::Archetype(class) => Choice::Archetype {
                class: class.clone(),
            },
            Address::Advances(class) => Choice::Advances {
                class: class.clone(),
            },
            Address::FirstAscension => Choice::FirstAscension,
            Address::Path => Choice::MythicPath,
            Address::Switch => Choice::PathSwitch,
            Address::Slot(slot) => Choice::Slot(slot.clone()),
        })
    }

    /// Where the choice sits: the Build start for its choices, the level for a level's choices and slots, the rank
    /// for the Mythic track's; `None` for a class's own choices, which sit with the class wherever it's first taken.
    pub fn place(&self) -> Option<Place> {
        match self {
            Address::Race
            | Address::Subrace
            | Address::Faith
            | Address::Alignment
            | Address::Background
            | Address::RacialAbility
            | Address::Score(_) => Some(Place::Start),
            Address::Level(level)
            | Address::Class(level)
            | Address::Increase(level)
            | Address::SkillPoints(level)
            | Address::Skill(level, _) => Some(Place::Level(*level)),
            Address::FirstAscension => Some(Place::Rank(FIRST_ASCENSION_RANK)),
            Address::Path => Some(Place::Rank(PATH_CHOICE_RANK)),
            Address::Switch => Some(Place::Rank(PATH_SWITCH_RANK)),
            Address::Slot(slot) => match (slot.level(), slot.rank()) {
                (Some(level), _) => Some(Place::Level(level)),
                (_, Some(rank)) => Some(Place::Rank(rank)),
                _ => None,
            },
            Address::Archetype(_) | Address::Advances(_) => None,
        }
    }

    /// A text about the choice, after its level or rank when it has one: "Level 3: 2 of 4 skill points unspent".
    /// The Build start's choices need no place before them.
    pub fn placed(&self, what: &str) -> String {
        match self.place() {
            Some(place @ (Place::Level(_) | Place::Rank(_))) => format!("{place}: {what}"),
            _ => what.to_owned(),
        }
    }
}

impl From<&Choice> for Address {
    /// Every choice the engine names has an address.
    fn from(choice: &Choice) -> Self {
        match choice {
            Choice::Race => Address::Race,
            Choice::Subrace => Address::Subrace,
            Choice::Alignment => Address::Alignment,
            Choice::Background => Address::Background,
            Choice::Deity => Address::Faith,
            Choice::AbilityScore { ability } => Address::Score(*ability),
            Choice::Class { level } => Address::Class(*level),
            Choice::Archetype { class } => Address::Archetype(class.clone()),
            Choice::Advances { class } => Address::Advances(class.clone()),
            Choice::AbilityIncrease { level } => Address::Increase(*level),
            Choice::SkillRanks { level, skill } => Address::Skill(*level, skill.clone()),
            Choice::SkillPoints { level } => Address::SkillPoints(*level),
            Choice::CharacterLevel { level } => Address::Level(*level),
            Choice::FirstAscension => Address::FirstAscension,
            Choice::MythicPath => Address::Path,
            Choice::PathSwitch => Address::Switch,
            Choice::Slot(slot) => Address::Slot(slot.clone()),
        }
    }
}

// ---- labels ---------------------------------------------------------------------------------------------------

impl Engine {
    /// What the choice is, in sentence case, as the sheet's lines and the pickers' headings name it: "Race",
    /// "Deity", "Strength (point buy)", "Class", "Ability increase", "Skill ranks", "Athletics ranks", "Fighter:
    /// archetype", "Mythic path", and for a slot what it takes ([`slot_label`](Self::slot_label)), or "slot"
    /// without the sheet's slot. A level itself is "Level 4".
    pub fn choice_label(&self, address: &Address, slot: Option<&Slot>) -> String {
        match address {
            Address::Race => "Race".to_owned(),
            Address::Subrace => "Subrace".to_owned(),
            Address::Faith => "Deity".to_owned(),
            Address::Alignment => "Alignment".to_owned(),
            Address::Background => "Background".to_owned(),
            Address::RacialAbility => "Racial ability bonus".to_owned(),
            Address::Score(ability) => format!("{} (point buy)", ability.name()),
            Address::Level(level) => format!("Level {level}"),
            Address::Class(_) => "Class".to_owned(),
            Address::Increase(_) => "Ability increase".to_owned(),
            Address::SkillPoints(_) => "Skill ranks".to_owned(),
            Address::Skill(_, skill) => format!("{} ranks", self.skill_name(skill)),
            Address::Archetype(class) => format!("{}: archetype", self.class_name(class)),
            Address::Advances(class) => format!("{}: advanced classes", self.class_name(class)),
            Address::FirstAscension => "First ascension".to_owned(),
            Address::Path => "Mythic path".to_owned(),
            Address::Switch => "Path switch".to_owned(),
            Address::Slot(_) => {
                slot.map_or_else(|| "slot".to_owned(), |slot| self.slot_label(slot))
            }
        }
    }

    /// The choice's label after its level or rank, as the MCP server's `todo` and `validate` name it: "Race",
    /// "Level 4: Class", "MR 3: Mythic path", "Level 1: Combat feat"; a level itself, "Level 4".
    pub fn placed_label(&self, address: &Address, slot: Option<&Slot>) -> String {
        let label = self.choice_label(address, slot);
        match address {
            Address::Level(_) => label,
            address => address.placed(&label),
        }
    }

    /// A parameter of a kind from its key ([`Parameter::value_key`]), in any case: one of the kind's values.
    pub fn parameter_from_key(&self, kind: ParameterKind, key: &str) -> Option<Parameter> {
        self.parameter_values(kind)
            .into_iter()
            .find(|value| value.value_key().eq_ignore_ascii_case(key))
    }
}

impl Ability {
    /// An ability increase's value as the sheet names it: "+1 Strength".
    pub fn increase_name(self) -> String {
        format!("+1 {}", self.name())
    }

    /// An ability increase's value as a level's chip names it: "+1 Str".
    pub fn increase_short_name(self) -> String {
        format!("+1 {}", self.short_name())
    }
}

// ---- value keys -----------------------------------------------------------------------------------------------

impl Ability {
    /// The ability as an address and a value name it: `str`, `dex`, `con`, `int`, `wis`, `cha`.
    pub fn key(self) -> &'static str {
        match self {
            Ability::Strength => "str",
            Ability::Dexterity => "dex",
            Ability::Constitution => "con",
            Ability::Intelligence => "int",
            Ability::Wisdom => "wis",
            Ability::Charisma => "cha",
        }
    }

    /// An ability from its key, or its full name ("Intelligence"), in any case.
    pub fn from_key(text: &str) -> Option<Ability> {
        let text = text.trim();
        AbilityScores::ABILITIES.into_iter().find(|ability| {
            ability.key().eq_ignore_ascii_case(text) || ability.name().eq_ignore_ascii_case(text)
        })
    }
}

impl Alignment {
    /// The alignment as a value names it: `lawful-good`, …, `true-neutral`, …, `chaotic-evil`.
    pub fn key(self) -> &'static str {
        match self {
            Alignment::LawfulGood => "lawful-good",
            Alignment::NeutralGood => "neutral-good",
            Alignment::ChaoticGood => "chaotic-good",
            Alignment::LawfulNeutral => "lawful-neutral",
            Alignment::TrueNeutral => "true-neutral",
            Alignment::ChaoticNeutral => "chaotic-neutral",
            Alignment::LawfulEvil => "lawful-evil",
            Alignment::NeutralEvil => "neutral-evil",
            Alignment::ChaoticEvil => "chaotic-evil",
        }
    }

    /// An alignment from its key, in any case.
    pub fn from_key(text: &str) -> Option<Alignment> {
        let text = text.trim();
        Alignment::ALL
            .into_iter()
            .find(|alignment| alignment.key().eq_ignore_ascii_case(text))
    }
}

/// Atheism's key: no deity has it.
const ATHEISM: &str = "atheism";

impl Faith {
    /// The faith as a value names it: the deity's key, or `atheism`.
    pub fn key(&self) -> &str {
        match self {
            Faith::Deity(key) => key.as_str(),
            Faith::Atheism => ATHEISM,
        }
    }

    /// A faith from its key, in any case: atheism, or the deity the key names (which the walk checks the game data
    /// has).
    pub fn from_key(text: &str) -> Faith {
        let text = text.trim().to_lowercase();
        if text == ATHEISM {
            Faith::Atheism
        } else {
            Faith::Deity(DeityKey::new(text))
        }
    }
}

impl Parameter {
    /// The value as a choice's value names it: a skill's, weapon's or metamagic feat's key, or a school, energy
    /// type, armour type, stat or element in lower case with dashes (`attacks-of-opportunity`, `negative-energy`).
    /// [`Engine::parameter_from_key`] reads it back.
    pub fn value_key(&self) -> &str {
        match self {
            Parameter::Skill(key) => key.as_str(),
            Parameter::Weapon(key) => key.as_str(),
            Parameter::Metamagic(key) => key.as_str(),
            Parameter::SpellSchool(school) => school.key(),
            Parameter::Energy(energy) => energy.key(),
            Parameter::ArmourType(armour) => armour.key(),
            Parameter::Stat(stat) => stat.key(),
            Parameter::Element(element) => element.key(),
        }
    }
}

impl SpellSchool {
    fn key(self) -> &'static str {
        match self {
            Self::Abjuration => "abjuration",
            Self::Conjuration => "conjuration",
            Self::Divination => "divination",
            Self::Enchantment => "enchantment",
            Self::Evocation => "evocation",
            Self::Illusion => "illusion",
            Self::Necromancy => "necromancy",
            Self::Transmutation => "transmutation",
        }
    }
}

impl Energy {
    fn key(self) -> &'static str {
        match self {
            Self::Acid => "acid",
            Self::Cold => "cold",
            Self::Electricity => "electricity",
            Self::Fire => "fire",
        }
    }
}

impl ArmourType {
    fn key(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Medium => "medium",
            Self::Heavy => "heavy",
        }
    }
}

impl Stat {
    fn key(self) -> &'static str {
        match self {
            Self::ArmourClass => "ac",
            Self::Attack => "attack",
            Self::AttacksOfOpportunity => "attacks-of-opportunity",
            Self::CombatManeuverBonus => "cmb",
            Self::CombatManeuverDefense => "cmd",
            Self::Damage => "damage",
            Self::HitPoints => "hp",
            Self::Initiative => "initiative",
            Self::Fortitude => "fortitude",
            Self::Reflex => "reflex",
            Self::Will => "will",
            Self::Speed => "speed",
        }
    }
}

impl Element {
    fn key(self) -> &'static str {
        match self {
            Self::Acid => "acid",
            Self::Cold => "cold",
            Self::Electricity => "electricity",
            Self::Fire => "fire",
            Self::Sonic => "sonic",
            Self::NegativeEnergy => "negative-energy",
        }
    }
}

// ---- the grammar ----------------------------------------------------------------------------------------------

fn source_token(source: FeatSource) -> &'static str {
    match source {
        FeatSource::Race => "race",
        FeatSource::Class => "class",
        FeatSource::General => "general",
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Race => write!(f, "start.race"),
            Address::Subrace => write!(f, "start.subrace"),
            Address::Faith => write!(f, "start.faith"),
            Address::Alignment => write!(f, "start.alignment"),
            Address::Background => write!(f, "start.background"),
            Address::RacialAbility => write!(f, "start.racial-ability"),
            Address::Score(ability) => write!(f, "start.score.{}", ability.key()),
            Address::Level(level) => write!(f, "level.{level}"),
            Address::Class(level) => write!(f, "level.{level}.class"),
            Address::Increase(level) => write!(f, "level.{level}.increase"),
            Address::SkillPoints(level) => write!(f, "level.{level}.skill"),
            Address::Skill(level, skill) => write!(f, "level.{level}.skill.{}", skill.as_str()),
            Address::Archetype(class) => write!(f, "class.{}.archetype", class.as_str()),
            Address::Advances(class) => write!(f, "class.{}.advances", class.as_str()),
            Address::FirstAscension => write!(f, "mythic.first-ascension"),
            Address::Path => write!(f, "mythic.path"),
            Address::Switch => write!(f, "mythic.switch"),
            Address::Slot(slot) => match *slot {
                SlotId::Feat {
                    level,
                    source,
                    index,
                } => write!(f, "slot.feat.{level}.{}.{index}", source_token(source)),
                SlotId::Selection { level, index } => write!(f, "slot.selection.{level}.{index}"),
                SlotId::Mythic { rank } => write!(f, "slot.mythic.{rank}"),
                SlotId::PathFeat { rank, index } => write!(f, "slot.path-feat.{rank}.{index}"),
                SlotId::PathSelection { rank, index } => {
                    write!(f, "slot.path-selection.{rank}.{index}")
                }
                SlotId::OptionSelection { level, index } => {
                    write!(f, "slot.option-selection.{level}.{index}")
                }
                SlotId::MythicSelection { rank, index } => {
                    write!(f, "slot.mythic-selection.{rank}.{index}")
                }
                SlotId::OptionFeat { level, index } => {
                    write!(f, "slot.option-feat.{level}.{index}")
                }
                SlotId::FeatSelection { level, index } => {
                    write!(f, "slot.feat-selection.{level}.{index}")
                }
                SlotId::MythicExtra { rank } => write!(f, "slot.mythic-extra.{rank}"),
            },
        }
    }
}

/// Text that isn't a choice address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotAnAddress(pub String);

impl fmt::Display for NotAnAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is not a choice address. Addresses look like `start.race`, `start.score.int`, `level.4.class`, \
             `level.4.increase`, `level.4.skill.athletics`, `mythic.path` or `slot.feat.3.general.0`; `sheet` and \
             `todo` print every address a Build has.",
            self.0
        )
    }
}

impl std::error::Error for NotAnAddress {}

impl FromStr for Address {
    type Err = NotAnAddress;

    fn from_str(text: &str) -> Result<Self, NotAnAddress> {
        parse(text.trim()).ok_or_else(|| NotAnAddress(text.to_owned()))
    }
}

/// A level or rank: 1 or more.
fn ordinal(text: &str) -> Option<u8> {
    text.parse().ok().filter(|n| *n > 0)
}

/// A slot index: 0 or more.
fn index(text: &str) -> Option<u8> {
    text.parse().ok()
}

/// A key segment: not empty, and no spaces.
fn key(text: &str) -> Option<&str> {
    (!text.is_empty() && !text.contains(char::is_whitespace)).then_some(text)
}

fn parse(text: &str) -> Option<Address> {
    if let Some(class) = text.strip_prefix("class.") {
        if let Some(class) = class.strip_suffix(".archetype") {
            return Some(Address::Archetype(ClassKey::new(key(class)?)));
        }
        let class = class.strip_suffix(".advances")?;
        return Some(Address::Advances(ClassKey::new(key(class)?)));
    }
    let parts: Vec<&str> = text.split('.').collect();
    Some(match parts.as_slice() {
        ["start", "race"] => Address::Race,
        ["start", "subrace"] => Address::Subrace,
        ["start", "faith"] => Address::Faith,
        ["start", "alignment"] => Address::Alignment,
        ["start", "background"] => Address::Background,
        ["start", "racial-ability"] => Address::RacialAbility,
        ["start", "score", ability] => Address::Score(Ability::from_key(ability)?),
        ["level", n] => Address::Level(ordinal(n)?),
        ["level", n, "class"] => Address::Class(ordinal(n)?),
        ["level", n, "increase"] => Address::Increase(ordinal(n)?),
        ["level", n, "skill"] => Address::SkillPoints(ordinal(n)?),
        ["level", n, "skill", skill @ ..] if !skill.is_empty() => {
            Address::Skill(ordinal(n)?, SkillKey::new(key(&skill.join("."))?))
        }
        ["mythic", "first-ascension"] => Address::FirstAscension,
        ["mythic", "path"] => Address::Path,
        ["mythic", "switch"] => Address::Switch,
        ["slot", rest @ ..] => Address::Slot(parse_slot(rest)?),
        _ => return None,
    })
}

fn parse_slot(parts: &[&str]) -> Option<SlotId> {
    Some(match *parts {
        ["feat", level, source, i] => SlotId::Feat {
            level: ordinal(level)?,
            source: match source {
                "race" => FeatSource::Race,
                "class" => FeatSource::Class,
                "general" => FeatSource::General,
                _ => return None,
            },
            index: index(i)?,
        },
        ["selection", level, i] => SlotId::Selection {
            level: ordinal(level)?,
            index: index(i)?,
        },
        ["mythic", rank] => SlotId::Mythic {
            rank: ordinal(rank)?,
        },
        ["path-feat", rank, i] => SlotId::PathFeat {
            rank: ordinal(rank)?,
            index: index(i)?,
        },
        ["path-selection", rank, i] => SlotId::PathSelection {
            rank: ordinal(rank)?,
            index: index(i)?,
        },
        ["option-selection", level, i] => SlotId::OptionSelection {
            level: ordinal(level)?,
            index: index(i)?,
        },
        ["mythic-selection", rank, i] => SlotId::MythicSelection {
            rank: ordinal(rank)?,
            index: index(i)?,
        },
        ["option-feat", level, i] => SlotId::OptionFeat {
            level: ordinal(level)?,
            index: index(i)?,
        },
        ["feat-selection", level, i] => SlotId::FeatSelection {
            level: ordinal(level)?,
            index: index(i)?,
        },
        ["mythic-extra", rank] => SlotId::MythicExtra {
            rank: ordinal(rank)?,
        },
        _ => return None,
    })
}
