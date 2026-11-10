//! Choice addresses: the strings the sheet prints for each choice of a Build and the agent copies back to `options`
//! (and to `choose` and `withdraw`). Each one names one [`Choice`] the engine knows, except the
//! racial ability, which the engine folds into the race's.
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

use rules::{
    Ability, AbilityScores, Choice, ClassKey, Engine, FIRST_ASCENSION_RANK, FeatSource,
    PATH_CHOICE_RANK, PATH_SWITCH_RANK, SkillKey, Slot, SlotId,
};

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
    /// Where a "+2 to one ability score" racial bonus goes.
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

    /// Where the choice sits, as its label begins: "Level 4" for a level's choices and slots, "MR 3" for a Mythic
    /// rank's; `None` for the Build start's and a class's own.
    pub(crate) fn place(&self) -> Option<String> {
        let rank = |rank: u8| Some(format!("MR {rank}"));
        match self {
            Address::Level(level)
            | Address::Class(level)
            | Address::Increase(level)
            | Address::SkillPoints(level)
            | Address::Skill(level, _) => Some(format!("Level {level}")),
            Address::FirstAscension => rank(FIRST_ASCENSION_RANK),
            Address::Path => rank(PATH_CHOICE_RANK),
            Address::Switch => rank(PATH_SWITCH_RANK),
            Address::Slot(slot) => match (slot.level(), slot.rank()) {
                (Some(level), _) => Some(format!("Level {level}")),
                (_, Some(r)) => rank(r),
                _ => None,
            },
            _ => None,
        }
    }

    /// What the choice is, as `todo` and `validate` name it: "Race", "Strength (point buy)", "Level 4: class",
    /// "MR 3: Mythic path", and for a slot what it takes, from the sheet's `slot` ("Level 1: Combat feat"), or "slot"
    /// without one.
    pub(crate) fn label(&self, engine: &Engine, slot: Option<&Slot>) -> String {
        let what = match self {
            Address::Race => return "Race".to_owned(),
            Address::Subrace => return "Subrace".to_owned(),
            Address::Faith => return "Faith".to_owned(),
            Address::Alignment => return "Alignment".to_owned(),
            Address::Background => return "Background".to_owned(),
            Address::RacialAbility => return "Racial ability bonus".to_owned(),
            Address::Score(ability) => return format!("{} (point buy)", ability.name()),
            Address::Archetype(class) => return format!("{}: archetype", engine.class_name(class)),
            Address::Advances(class) => {
                return format!("{}: advanced classes", engine.class_name(class));
            }
            Address::Level(level) => return format!("Level {level}"),
            Address::Class(_) => "class".to_owned(),
            Address::Increase(_) => "ability increase".to_owned(),
            Address::SkillPoints(_) => "skill points".to_owned(),
            Address::Skill(_, skill) => format!("{} ranks", engine.skill_name(skill)),
            Address::FirstAscension => "first ascension".to_owned(),
            Address::Path => "Mythic path".to_owned(),
            Address::Switch => "Path switch".to_owned(),
            Address::Slot(_) => {
                slot.map_or_else(|| "slot".to_owned(), |slot| engine.slot_label(slot))
            }
        };
        self.placed(&what)
    }

    /// A text about the choice, after its place when it has one: "Level 3: 2 of 4 skill points unspent".
    pub(crate) fn placed(&self, what: &str) -> String {
        match self.place() {
            Some(place) => format!("{place}: {what}"),
            None => what.to_owned(),
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

/// An ability as an address names it: `str`, `dex`, `con`, `int`, `wis`, `cha`.
pub fn ability_token(ability: Ability) -> String {
    ability.short_name().to_lowercase()
}

/// An ability from its token, or its full name in any case ("Intelligence").
pub fn parse_ability(text: &str) -> Option<Ability> {
    let text = text.trim().to_lowercase();
    AbilityScores::ABILITIES
        .into_iter()
        .find(|ability| ability_token(*ability) == text || ability.name().to_lowercase() == text)
}

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
            Address::Score(ability) => write!(f, "start.score.{}", ability_token(*ability)),
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
        ["start", "score", ability] => Address::Score(parse_ability(ability)?),
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
