//! Assumed gear (ticket 122): bonuses an agent assumes on top of a Build, applied to the numbers the engine computes
//! (the attack line, the caster line, the totals) and shown beside them. Never part of a Build, its link or any
//! prerequisite: the map keeps gear out of a character's choices, and a prerequisite checks the character's own
//! scores.
//!
//! Two kinds of assumption:
//! - generic bonuses ([`Assumed`]): an enhancement and an inherent bonus to each ability score, a weapon's
//!   enhancement bonus and a resistance bonus to saves, each a number the agent gives;
//! - named items ([`GearItem`], `gear.ron`): the guide's two "Caster Level and Spell DC Boosting Items" tables on its
//!   `items` page, each bonus with the circumstance the table gives it ([`GearCircumstance`]), which the caster line
//!   applies only where it matches.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::bonus::{SpellScope, SpellTraits};
use crate::build::AbilityScores;
use crate::game_data::{Ability, GuideRef, Save};
use crate::keys::{ClassKey, GearKey};

/// An item of the guide's "Caster Level and Spell DC Boosting Items" tables, with the bonuses to the caster line's
/// numbers its row on the same page lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearItem {
    pub key: GearKey,
    pub name: String,
    /// The table's group heading: the slot it's worn in, or the weapon it is.
    pub slot: GearSlot,
    /// The guide page the tables are on: `items`.
    pub guide: GuideRef,
    pub bonuses: Vec<GearBonus>,
}

/// One bonus a [`GearItem`] gives the caster line: to what, how much, and for which spells.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearBonus {
    pub target: GearTarget,
    pub amount: i8,
    /// The spells it counts for: the table's "Circumstances" column. Every spell when left out (the table's "-").
    #[serde(default)]
    pub circumstance: GearCircumstance,
}

/// What a [`GearBonus`] adds to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GearTarget {
    /// The save DC of the spells.
    SpellDc,
    /// The caster level, and with it caster level checks against spell resistance and concentration.
    CasterLevel,
    /// Caster level checks to overcome spell resistance only.
    SpellPenetration,
    /// Concentration checks only.
    Concentration,
}

/// The spells a [`GearBonus`] counts for.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GearCircumstance {
    /// A scope of ticket 119's: every spell ([`SpellScope::All`], the table's "-"), a school, a subschool, an energy
    /// type. Never [`SpellScope::Value`]: an item isn't taken for a value.
    Spells(SpellScope),
    /// Spells that allow this saving throw (the table's "vs. Will").
    Save(Save),
    /// Mind-affecting spells.
    MindAffecting,
    /// The spells of this class, when the caster line follows it (the table's "Druid", "Witch spells").
    Class(ClassKey),
}

impl Default for GearCircumstance {
    /// Every spell.
    fn default() -> Self {
        Self::Spells(SpellScope::All)
    }
}

impl GearCircumstance {
    /// Whether a bonus with this circumstance counts for a spell cast by `class`.
    pub fn applies(&self, spell: &SpellTraits, class: &ClassKey) -> bool {
        match self {
            Self::Spells(scope) => scope.applies(spell, None),
            Self::Save(save) => spell.save == Some(*save),
            Self::MindAffecting => spell.mind_affecting,
            Self::Class(own) => own == class,
        }
    }
}

/// The guide's group headings in the two tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GearSlot {
    Dagger,
    Quarterstaff,
    Scimitar,
    Estoc,
    Bracers,
    Cloak,
    Glasses,
    Gloves,
    Headwear,
    Necklace,
    Ring,
    Robe,
    Rod,
}

impl GearSlot {
    pub const ALL: [Self; 13] = [
        Self::Dagger,
        Self::Quarterstaff,
        Self::Scimitar,
        Self::Estoc,
        Self::Bracers,
        Self::Cloak,
        Self::Glasses,
        Self::Gloves,
        Self::Headwear,
        Self::Necklace,
        Self::Ring,
        Self::Robe,
        Self::Rod,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Dagger => "Dagger",
            Self::Quarterstaff => "Quarterstaff",
            Self::Scimitar => "Scimitar",
            Self::Estoc => "Estoc",
            Self::Bracers => "Bracers",
            Self::Cloak => "Cloak",
            Self::Glasses => "Glasses",
            Self::Gloves => "Gloves",
            Self::Headwear => "Headwear",
            Self::Necklace => "Necklace",
            Self::Ring => "Ring",
            Self::Robe => "Robe",
            Self::Rod => "Rod",
        }
    }
}

/// What an agent assumes on top of a Build ([`Engine::assumed_attack_line`], [`Engine::assumed_caster_line`],
/// [`Engine::assumed_totals`]). The numbers are the agent's; the planner adds them as given, and how the game's
/// bonuses stack is the agent's to judge.
///
/// [`Engine::assumed_attack_line`]: crate::Engine::assumed_attack_line
/// [`Engine::assumed_caster_line`]: crate::Engine::assumed_caster_line
/// [`Engine::assumed_totals`]: crate::Engine::assumed_totals
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Assumed {
    /// An enhancement bonus to each ability score named (a belt, a headband).
    pub enhancement: BTreeMap<Ability, u8>,
    /// An inherent bonus to each ability score named (tomes and manuals).
    pub inherent: BTreeMap<Ability, u8>,
    /// The weapon's enhancement bonus, to attack and damage.
    pub weapon_enhancement: u8,
    /// A resistance bonus to every save (a cloak of resistance).
    pub resistance: u8,
    /// Items of `gear.ron`, for the caster line.
    pub items: Vec<GearKey>,
}

impl Assumed {
    /// The character's own scores raised by the assumed enhancement and inherent bonuses.
    pub fn scores(&self, own: AbilityScores) -> AbilityScores {
        let mut scores = own;
        for ability in AbilityScores::ABILITIES {
            let raise = self.enhancement.get(&ability).copied().unwrap_or(0)
                + self.inherent.get(&ability).copied().unwrap_or(0);
            let score = scores.get_mut(ability);
            *score = score.saturating_add(raise);
        }
        scores
    }

    /// The name of an assumed part: "Weapon enhancement +5 (assumed)".
    pub fn label(name: &str) -> String {
        format!("{name} (assumed)")
    }
}
