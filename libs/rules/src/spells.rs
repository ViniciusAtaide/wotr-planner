//! The spell catalogue (ticket 121): each spell's facts from its block in the guide, and the spell lists that say
//! which spells a class, an archetype or a Mythic path casts, and at which spell level.
//!
//! Reference data only: no Build choice names a spell, and the engine never reads them. A spell has no level of its
//! own, since it differs by list (Hold Person: wizard 3, cleric 2, bard 2); a list's entries give it.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::game_data::GuideRef;
use crate::keys::{ArchetypeKey, ClassKey, MythicPathKey, SpellKey, SpellListKey};
use crate::parameter::SpellSchool;
use crate::prerequisite::Story;

/// The highest spell level on a class's or an archetype's list.
pub const MAX_CLASS_SPELL_LEVEL: u8 = 9;
/// The highest spell level in a Mythic spellbook (Angel, Lich, Gold Dragon: 10).
pub const MAX_MYTHIC_SPELL_LEVEL: u8 = 10;

/// A spell, from its block in the guide: a heading "Name (School)" and a table of facts. The block sits once, under
/// the first list that has the spell; other lists only name it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spell {
    pub key: SpellKey,
    pub name: String,
    /// `None`: Universalist, the guide's school for a spell of no school (Dismiss Spell). Spell Focus and the
    /// wizard's specialist schools have no Universalist, so it isn't a [`SpellSchool`].
    pub school: Option<SpellSchool>,
    pub guide: GuideRef,
    /// What the spell does, in one sentence in our own words.
    #[serde(default)]
    pub summary: String,
    /// The material component the block names, as printed ("Diamond dust x10").
    #[serde(default)]
    pub components: Option<String>,
    /// The block's Target: whom or what area the spell reaches, and its range ("One enemy creature within medium
    /// range").
    pub target: String,
    /// The block's Duration, as printed ("1 round / level"); `None` where the block has no Duration row
    /// (instantaneous spells).
    #[serde(default)]
    pub duration: Option<String>,
    /// The block's Saving Throw, as printed ("Will negates", "Reflex half", "N/A").
    pub saving_throw: String,
    /// Whether spell resistance applies (the block's ✓; N/A is `false`).
    pub spell_resistance: bool,
    /// The block's Spell Descriptors, the subschool among them (Hold Monster: Mind-Affecting, Compulsion). Empty
    /// where the block prints "-" or has no row.
    #[serde(default)]
    pub descriptors: BTreeSet<SpellDescriptor>,
    pub casting_time: CastingTime,
}

/// A spell descriptor as the guide's blocks name it. The guide puts the enchantment subschools (Compulsion,
/// Charmed) and Summoning among them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SpellDescriptor {
    Acid,
    /// Wracking Ray's only descriptor.
    ChannelNegativeEnergyHarm,
    Chaos,
    /// The guide's word on three enchantments (Haze of Dreams, the Litanies of Eloquence and Madness).
    Charmed,
    Cold,
    Compulsion,
    Cure,
    Curse,
    Death,
    Disease,
    Divine,
    Electricity,
    Emotion,
    Evil,
    Fear,
    Fire,
    Force,
    Good,
    Law,
    MindAffecting,
    MovementImpairing,
    NegativeEmotion,
    NegativeLevel,
    Poison,
    Polymorph,
    Sight,
    Sleep,
    Sonic,
    Stunned,
    Summoning,
}

impl SpellDescriptor {
    pub const ALL: [Self; 30] = [
        Self::Acid,
        Self::ChannelNegativeEnergyHarm,
        Self::Chaos,
        Self::Charmed,
        Self::Cold,
        Self::Compulsion,
        Self::Cure,
        Self::Curse,
        Self::Death,
        Self::Disease,
        Self::Divine,
        Self::Electricity,
        Self::Emotion,
        Self::Evil,
        Self::Fear,
        Self::Fire,
        Self::Force,
        Self::Good,
        Self::Law,
        Self::MindAffecting,
        Self::MovementImpairing,
        Self::NegativeEmotion,
        Self::NegativeLevel,
        Self::Poison,
        Self::Polymorph,
        Self::Sight,
        Self::Sleep,
        Self::Sonic,
        Self::Stunned,
        Self::Summoning,
    ];

    /// The descriptor as the guide prints it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Acid => "Acid",
            Self::ChannelNegativeEnergyHarm => "Channel Negative Energy Harm",
            Self::Chaos => "Chaos",
            Self::Charmed => "Charmed",
            Self::Cold => "Cold",
            Self::Compulsion => "Compulsion",
            Self::Cure => "Cure",
            Self::Curse => "Curse",
            Self::Death => "Death",
            Self::Disease => "Disease",
            Self::Divine => "Divine",
            Self::Electricity => "Electricity",
            Self::Emotion => "Emotion",
            Self::Evil => "Evil",
            Self::Fear => "Fear",
            Self::Fire => "Fire",
            Self::Force => "Force",
            Self::Good => "Good",
            Self::Law => "Law",
            Self::MindAffecting => "Mind-Affecting",
            Self::MovementImpairing => "Movement Impairing",
            Self::NegativeEmotion => "Negative Emotion",
            Self::NegativeLevel => "Negative Level",
            Self::Poison => "Poison",
            Self::Polymorph => "Polymorph",
            Self::Sight => "Sight",
            Self::Sleep => "Sleep",
            Self::Sonic => "Sonic",
            Self::Stunned => "Stunned",
            Self::Summoning => "Summoning",
        }
    }
}

/// The action a spell takes to cast, from the block's Casting Time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CastingTime {
    Standard,
    FullRound,
    Move,
    Swift,
    Free,
}

impl CastingTime {
    pub const ALL: [Self; 5] = [
        Self::Standard,
        Self::FullRound,
        Self::Move,
        Self::Swift,
        Self::Free,
    ];

    /// As the guide prints it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Standard => "Standard Action",
            Self::FullRound => "Full Round",
            Self::Move => "Move Action",
            Self::Swift => "Swift Action",
            Self::Free => "Free Action",
        }
    }
}

/// A spell list: one of the guide's tables of spells by level (a class's, an archetype's or a Mythic path's
/// spellbook).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellList {
    /// The owner's key (`wizard`, `arcanist/magic-deceiver`, `azata`), or one under it for a second list of the
    /// same owner (`angel/merged`).
    pub key: SpellListKey,
    pub name: String,
    pub guide: GuideRef,
    /// The class, archetype or path whose list it is: the page it's on.
    pub owner: SpellCaster,
    /// Other classes and archetypes that cast from it, where the guide says so: the sorcerer and the arcanist from
    /// the wizard's ("Uses the Wizard spell list"), the oracle from the cleric's, the skald from the bard's.
    #[serde(default)]
    pub used_by: BTreeSet<SpellCaster>,
    /// The spells on it, each at its level on this list. A spell may be on it twice: at two levels (the Dirge
    /// Bard's Fear at 4, everyone's at 3).
    pub entries: BTreeSet<SpellListEntry>,
}

/// Whose spell list it is, or who also casts from it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SpellCaster {
    Class(ClassKey),
    Archetype(ArchetypeKey),
    Path(MythicPathKey),
}

impl SpellCaster {
    /// The owner's key, whatever its kind.
    pub fn key(&self) -> &str {
        match self {
            Self::Class(key) => key.as_str(),
            Self::Archetype(key) => key.as_str(),
            Self::Path(key) => key.as_str(),
        }
    }
}

/// A spell on a list, at a spell level.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellListEntry {
    /// 0 to 9 on a class's or an archetype's list, 1 to 10 in a Mythic spellbook.
    pub level: u8,
    pub spell: SpellKey,
    /// The table's footnote limiting the entry to some casters of the list ("Flame Dancer only"); `None` for all.
    #[serde(default)]
    pub only: Option<SpellListOnly>,
}

/// Who alone gets a list entry, from the table's footnote.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SpellListOnly {
    /// An archetype of the list's class ("Flame Dancer only"; the druid's "Only available through a Drovier's
    /// Communal Aspect ability").
    Archetype(ArchetypeKey),
    /// A story branch of the path (Gold Dragon: "Spells exclusive to the uncorrupted path", `Uncorrupted`, or to the
    /// corrupted one, `VowOfDahak`).
    Story(Story),
}
