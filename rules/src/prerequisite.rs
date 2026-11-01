//! Prerequisites: nested AND / OR / at-least-n groups over single conditions.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::game_data::{Ability, Alignment, SpellType};
use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, FeatureKey, FeatureSelectionKey, MythicAbilityKey,
    MythicFeatKey, RaceKey, SelectionOptionKey, SkillKey,
};
use crate::parameter::Parameter;

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

    /// Another feat (a parameterised one for any value: "Spell Focus (any)").
    Feat(FeatKey),
    /// A parameterised feat held for this value ("Weapon Focus (Dueling Sword)").
    FeatFor {
        feat: FeatKey,
        value: Parameter,
    },
    /// A parameterised feat held for the value of the pick being checked ("Choose a single weapon for which you
    /// have already selected Weapon Focus"). Only in the prerequisite of a parameterised feat, or of an option
    /// with a chosen feat.
    FeatForSameValue(FeatKey),
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
    /// No option held that this selection offers ("no known bloodline": none of the sorcerer's bloodlines).
    NoOptionFrom(FeatureSelectionKey),
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
    /// Spell-level access through a class that casts spontaneously ("Able to cast 1st level arcane spells
    /// spontaneously").
    SpontaneousSpellLevel {
        spell_type: SpellType,
        level: u8,
    },
    /// Caster level, only where the guide states it.
    CasterLevel(u8),
    CharacterLevel(u8),
    /// The character's Mythic rank at least this ("Mythic Rank: 9"). After a Legend switch the rank is 3.
    MythicRank(u8),
    /// The Build's alignment is exactly this one. "Any lawful" is an `Any` of three.
    Alignment(Alignment),
    Race(RaceKey),
    /// A story condition (Vow of Dahak). Story is out of scope, so it always holds: the sheet shows it as a note
    /// ([`Prerequisite::stories`]) and never blocks on it.
    Story(Story),
}

/// A condition set by the game's story, not by a Build: the planner never checks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Story {
    /// Gold Dragon: sworn to Dahak.
    VowOfDahak,
    /// Gold Dragon: not sworn to Dahak.
    Uncorrupted,
}

impl Story {
    pub const ALL: [Story; 2] = [Story::VowOfDahak, Story::Uncorrupted];

    /// The condition as the guide prints it.
    pub fn name(self) -> &'static str {
        match self {
            Story::VowOfDahak => "Vow of Dahak",
            Story::Uncorrupted => "Uncorrupted",
        }
    }
}

impl Prerequisite {
    /// The story conditions anywhere in the tree, for the note beside the option: they never block.
    pub fn stories(&self) -> BTreeSet<Story> {
        let mut stories = BTreeSet::new();
        self.collect_stories(&mut stories);
        stories
    }

    fn collect_stories(&self, stories: &mut BTreeSet<Story>) {
        match self {
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                for child in children {
                    child.collect_stories(stories);
                }
            }
            Prerequisite::Story(story) => {
                stories.insert(*story);
            }
            _ => {}
        }
    }

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
