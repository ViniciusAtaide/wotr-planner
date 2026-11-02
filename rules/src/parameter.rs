//! Parameterised and repeatable feats: what a feat is taken for, and how many times it can be taken.
//!
//! A parameterised feat names the kind of thing it's taken for (Weapon Focus: a weapon), and each pick of it
//! carries the value (Weapon Focus (Longsword)). Prerequisites can need a feat for a specific value or for the
//! value of the pick being checked (Weapon Specialization: Weapon Focus in the same weapon).

use serde::{Deserialize, Serialize};

use crate::keys::{SkillKey, WeaponKey};

/// The kind of value a parameterised feat is taken for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ParameterKind {
    /// A skill from `skills.ron` (Skill Focus).
    Skill,
    /// Any weapon in `weapons.ron` (Weapon Focus).
    Weapon,
    /// An exotic weapon: one the guide's Exotic Weapon Proficiency lists.
    ExoticWeapon,
    /// A school of magic (Spell Focus).
    SpellSchool,
    /// An energy type (Elemental Focus).
    Energy,
    /// Light, medium or heavy armour (Armor Focus).
    ArmourType,
}

/// What one pick of a parameterised feat is taken for (Weapon Focus (Longsword): `Weapon("longsword")`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Parameter {
    Skill(SkillKey),
    /// A weapon, for a `Weapon` or `ExoticWeapon` feat.
    Weapon(WeaponKey),
    SpellSchool(SpellSchool),
    Energy(Energy),
    ArmourType(ArmourType),
}

impl Parameter {
    /// The value as a key, for display and for "removed from game data" messages.
    pub fn key(&self) -> &str {
        match self {
            Parameter::Skill(key) => key.as_str(),
            Parameter::Weapon(key) => key.as_str(),
            Parameter::SpellSchool(school) => school.name(),
            Parameter::Energy(energy) => energy.name(),
            Parameter::ArmourType(armour) => armour.name(),
        }
    }
}

/// The eight schools of magic, as the guide's spell tables and the wizard's specialist schools name them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SpellSchool {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,
}

impl SpellSchool {
    pub const ALL: [Self; 8] = [
        Self::Abjuration,
        Self::Conjuration,
        Self::Divination,
        Self::Enchantment,
        Self::Evocation,
        Self::Illusion,
        Self::Necromancy,
        Self::Transmutation,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Abjuration => "Abjuration",
            Self::Conjuration => "Conjuration",
            Self::Divination => "Divination",
            Self::Enchantment => "Enchantment",
            Self::Evocation => "Evocation",
            Self::Illusion => "Illusion",
            Self::Necromancy => "Necromancy",
            Self::Transmutation => "Transmutation",
        }
    }
}

/// The energy types Elemental Focus lists: "acid, cold, electricity or fire".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Energy {
    Acid,
    Cold,
    Electricity,
    Fire,
}

impl Energy {
    pub const ALL: [Self; 4] = [Self::Acid, Self::Cold, Self::Electricity, Self::Fire];

    pub fn name(self) -> &'static str {
        match self {
            Self::Acid => "Acid",
            Self::Cold => "Cold",
            Self::Electricity => "Electricity",
            Self::Fire => "Fire",
        }
    }
}

/// The armour types Armor Focus lists: "light, medium, or heavy".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArmourType {
    Light,
    Medium,
    Heavy,
}

impl ArmourType {
    pub const ALL: [Self; 3] = [Self::Light, Self::Medium, Self::Heavy];

    pub fn name(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Medium => "Medium",
            Self::Heavy => "Heavy",
        }
    }
}

/// How many times a feat or a selection option can be held.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Repeat {
    /// Once: what the guide means when it doesn't say otherwise.
    #[default]
    Once,
    /// Up to this many times, 2 or more ("You can take this feat up to 3 times").
    Times(u8),
    /// Any number of times ("This feat may be taken multiple times").
    Unlimited,
    /// Once for each value of the feat's parameter ("it must be applied to a different weapon each time"). Only a
    /// parameterised feat, or an option with a chosen feat: once for each value of that feat (the Skill Focus
    /// talents, once per skill).
    PerValue,
}

impl Repeat {
    /// Whether this is the default, so the data files can leave it out.
    pub fn is_once(&self) -> bool {
        *self == Repeat::Once
    }
}
