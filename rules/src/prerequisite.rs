//! Prerequisites: nested AND / OR / at-least-n groups over single conditions.

use serde::{Deserialize, Serialize};

use crate::game_data::{Ability, Alignment, SpellType};
use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, FeatureKey, MythicAbilityKey, MythicFeatKey, RaceKey,
    SelectionOptionKey, SkillKey,
};

/// A condition that must hold before a feat, selection option, mythic pick, class or
/// archetype can be chosen.
///
/// The guide's lines are ANDed, so a typical entry is an `All` of leaf conditions. "One of" is `Any`
/// and "Two of" is `AtLeast(2, ..)`.
///
/// A group's children are a set: their order means nothing, and [`canonicalize`](Self::canonicalize) sorts them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Prerequisite {
    /// Every child must hold.
    All(Vec<Prerequisite>),
    /// At least one child must hold.
    Any(Vec<Prerequisite>),
    /// At least `n` children must hold.
    AtLeast(u8, Vec<Prerequisite>),

    /// Another feat.
    Feat(FeatKey),
    MythicFeat(MythicFeatKey),
    MythicAbility(MythicAbilityKey),
    /// A Class level in a base or prestige class ("Fighter level: 8").
    ClassLevel {
        class: ClassKey,
        level: u8,
    },
    Archetype(ArchetypeKey),
    /// A feature granted at least `count` times ("Sneak Attack: 1", "Sneak Attack (2d6)").
    Feature {
        feature: FeatureKey,
        count: u8,
    },
    /// An option already taken from a Feature selection ("Beast Totem, Lesser").
    SelectionOption(SelectionOptionKey),
    SkillRanks {
        skill: SkillKey,
        ranks: u8,
    },
    AbilityScore {
        ability: Ability,
        score: u8,
    },
    BaseAttackBonus(u8),
    /// Spell-level access: can cast spells of this level and type. Not the same as caster level.
    SpellLevel {
        spell_type: SpellType,
        level: u8,
    },
    /// Caster level, only where the guide states it.
    CasterLevel(u8),
    CharacterLevel(u8),
    /// The Build's alignment is exactly this one. "Any lawful" is an `Any` of three.
    Alignment(Alignment),
    Race(RaceKey),
}

impl Prerequisite {
    /// Sorts every group's children, recursively, so equal trees compare equal.
    pub fn canonicalize(&mut self) {
        if let Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) = self
        {
            for child in children.iter_mut() {
                child.canonicalize();
            }
            children.sort();
        }
    }
}
