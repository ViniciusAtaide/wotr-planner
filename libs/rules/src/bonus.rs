//! Flat bonuses that items give while a character holds them (ticket 119): the save DC of spells, caster level
//! against spell resistance and concentration. `bonuses.ron` lists them, one [`ItemBonus`] per bonus, each naming
//! the item that gives it; the engine sums the ones a character holds ([`Engine::caster_line`]).
//!
//! The shape is meant to grow: a new kind of number is a new [`BonusTarget`] (ticket 120: initiative, a skill, a
//! save), a new kind of giver a new [`BonusItem`] (a background, a mythic ability), and the amount, the condition
//! and the item stay as they are.
//!
//! [`Engine::caster_line`]: crate::Engine::caster_line

use serde::{Deserialize, Serialize};

use crate::keys::{FeatKey, FeatureKey, MythicFeatKey, RaceKey, SelectionOptionKey, SubraceKey};
use crate::parameter::{Energy, Parameter, SpellSchool};
use crate::prerequisite::Prerequisite;

/// One flat bonus an item gives while it's held: to what, how much, and on what else it depends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemBonus {
    /// The item that gives it.
    pub item: BonusItem,
    /// What it adds to.
    pub target: BonusTarget,
    pub amount: BonusAmount,
    /// What the character must hold besides the item for it to count, checked like a prerequisite (Spell
    /// Penetration (Mythic)'s full Mythic rank: Greater Spell Penetration). `None`: the item alone.
    #[serde(default)]
    pub requires: Option<Prerequisite>,
}

/// The item that gives a bonus. A race's bonus counts for a character of that race without a subrace, a subrace's
/// for one of that subrace: the guide lists a subrace's racial traits in full, so they replace the race's, as its
/// ability modifiers do.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusItem {
    Race(RaceKey),
    Subrace(SubraceKey),
    /// A feat, held however (picked or granted). A parameterised one's bonus counts for each value it's held for.
    Feat(FeatKey),
    /// A mythic feat; a parameterised one's counts for each value, as a feat's.
    MythicFeat(MythicFeatKey),
    /// A class, archetype or path feature, held however it's granted (a bloodline's arcana through Second
    /// Bloodline or Crossblooded too).
    Feature(FeatureKey),
    /// A selection option held (School Power's Enchantment).
    SelectionOption(SelectionOptionKey),
}

/// What a bonus adds to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusTarget {
    /// The save DC of the character's spells of a scope.
    SpellDc(SpellScope),
    /// Caster level checks to overcome spell resistance.
    SpellPenetration,
    /// Concentration checks.
    Concentration,
}

/// Which spells a spell DC bonus applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SpellScope {
    /// Every spell.
    All,
    School(SpellSchool),
    Subschool(Subschool),
    /// Spells of an energy type.
    Energy(Energy),
    /// Spells cast with a metamagic feat other than Heighten Spell (the Arcane bloodline's arcana).
    Metamagic,
    /// The school or energy type the item is held for: Spell Focus (Enchantment), Elemental Focus (Fire). Only on a
    /// feat or mythic feat taken for a school or an energy type.
    Value,
}

impl SpellScope {
    /// Whether a bonus of this scope from an item held for `value` applies to a spell. A scope of `Value` needs the
    /// value; any other ignores it.
    pub fn applies(self, spell: &SpellTraits, value: Option<&Parameter>) -> bool {
        match self {
            Self::All => true,
            Self::School(school) => spell.school == school,
            Self::Subschool(subschool) => spell.subschool == Some(subschool),
            Self::Energy(energy) => spell.energy == Some(energy),
            Self::Metamagic => spell.metamagic,
            Self::Value => match value {
                Some(Parameter::SpellSchool(school)) => spell.school == *school,
                Some(Parameter::Energy(energy)) => spell.energy == Some(*energy),
                _ => false,
            },
        }
    }
}

/// How much a bonus adds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusAmount {
    /// A fixed number (+1, +2, +4).
    Flat(i8),
    /// Half the Mythic rank, rounded down.
    HalfMythicRank,
    /// The Mythic rank.
    MythicRank,
    /// Expanded Arsenal: the school it's held for gets the bonuses the Spell Focus feats (the feats and mythic feats
    /// whose spell DC bonus is for the school they're held for) give one other school, the one where they add up to
    /// the most; nothing when it has Spell Focus feats of its own (the guide: "this has no effect"). Only with
    /// [`SpellScope::Value`].
    SpellFocusOfAnotherSchool,
}

/// What a spell is, as far as the DC bonuses look: a school, and optionally a subschool, an energy type and whether
/// a metamagic feat (other than Heighten Spell) is applied. No spell catalogue is needed: the caster line takes
/// these as given.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpellTraits {
    pub school: SpellSchool,
    pub subschool: Option<Subschool>,
    pub energy: Option<Energy>,
    pub metamagic: bool,
}

/// The subschools of magic, as the core rules name them. Only charm and compulsion change a DC in the data today
/// (the Infernal and Fey arcana); the rest are here so a caster line can name any spell's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Subschool {
    Calling,
    Creation,
    Healing,
    Summoning,
    Teleportation,
    Scrying,
    Charm,
    Compulsion,
    Figment,
    Glamer,
    Pattern,
    Phantasm,
    Shadow,
    Polymorph,
}

impl Subschool {
    pub const ALL: [Self; 14] = [
        Self::Calling,
        Self::Creation,
        Self::Healing,
        Self::Summoning,
        Self::Teleportation,
        Self::Scrying,
        Self::Charm,
        Self::Compulsion,
        Self::Figment,
        Self::Glamer,
        Self::Pattern,
        Self::Phantasm,
        Self::Shadow,
        Self::Polymorph,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Calling => "Calling",
            Self::Creation => "Creation",
            Self::Healing => "Healing",
            Self::Summoning => "Summoning",
            Self::Teleportation => "Teleportation",
            Self::Scrying => "Scrying",
            Self::Charm => "Charm",
            Self::Compulsion => "Compulsion",
            Self::Figment => "Figment",
            Self::Glamer => "Glamer",
            Self::Pattern => "Pattern",
            Self::Phantasm => "Phantasm",
            Self::Shadow => "Shadow",
            Self::Polymorph => "Polymorph",
        }
    }

    /// The school it belongs to.
    pub fn school(self) -> SpellSchool {
        match self {
            Self::Calling
            | Self::Creation
            | Self::Healing
            | Self::Summoning
            | Self::Teleportation => SpellSchool::Conjuration,
            Self::Scrying => SpellSchool::Divination,
            Self::Charm | Self::Compulsion => SpellSchool::Enchantment,
            Self::Figment | Self::Glamer | Self::Pattern | Self::Phantasm | Self::Shadow => {
                SpellSchool::Illusion
            }
            Self::Polymorph => SpellSchool::Transmutation,
        }
    }
}
