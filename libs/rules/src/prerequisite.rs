//! Prerequisites: nested AND / OR / at-least-n groups over single conditions.

use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

use crate::game_data::{Ability, Alignment, OptionGroup, SpellType};
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
    /// The child must not hold (a slayer's first combat style feat: not yet slayer level 6).
    Not(Box<Prerequisite>),

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
    /// When the pick being checked is taken for `value`, `feat` is needed too; for any other value it holds
    /// ("Armor Focus (Heavy)" needs heavy armour proficiency). Only in the prerequisite of a pick that takes a
    /// value.
    FeatIfValue {
        value: Parameter,
        feat: FeatKey,
    },
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
    /// No option of this group held but the one being picked ("You can only take a single draconic bloodline").
    /// Only in a selection option's prerequisite.
    NoOtherOptionOfGroup(OptionGroup),
    /// The pick being checked fills a slot of this selection (the Champion of the Faith's alignment blessing,
    /// which must match his alignment there and only there). Only in a selection option's prerequisite.
    PickedThrough(FeatureSelectionKey),
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
    /// The single conditions in the tree, left to right: everything under its groups and `Not`s, and never a group
    /// or a `Not` itself.
    pub fn leaves(&self) -> impl Iterator<Item = &Prerequisite> {
        // A stack of the nodes still to visit, the next one on top.
        let mut pending = vec![self];
        std::iter::from_fn(move || {
            while let Some(node) = pending.pop() {
                match node {
                    Prerequisite::All(children)
                    | Prerequisite::Any(children)
                    | Prerequisite::AtLeast(_, children) => pending.extend(children.iter().rev()),
                    Prerequisite::Not(child) => pending.push(child),
                    leaf => return Some(leaf),
                }
            }
            None
        })
    }

    /// The story conditions anywhere in the tree, for the note beside the option: they never block.
    pub fn stories(&self) -> BTreeSet<Story> {
        self.leaves()
            .filter_map(|leaf| match leaf {
                Prerequisite::Story(story) => Some(*story),
                _ => None,
            })
            .collect()
    }

    /// Adds the feats the tree names anywhere, in a `Not` too: every feat whose holding it can look at.
    pub(crate) fn collect_feats<'p>(&'p self, feats: &mut HashSet<&'p FeatKey>) {
        feats.extend(self.leaves().filter_map(|leaf| match leaf {
            Prerequisite::Feat(feat)
            | Prerequisite::FeatFor { feat, .. }
            | Prerequisite::FeatForSameValue(feat)
            | Prerequisite::FeatIfValue { feat, .. } => Some(feat),
            _ => None,
        }));
    }

    /// Whether this is an alignment requirement: an `Alignment`, or a group (or `Not`) of nothing but alignment
    /// requirements ("Cannot be Lawful": an `Any` of the six others).
    pub(crate) fn is_alignment(&self) -> bool {
        // Its own walk, not `leaves`: an empty group anywhere makes it false, and the leaves can't show one.
        match self {
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                !children.is_empty() && children.iter().all(Prerequisite::is_alignment)
            }
            Prerequisite::Not(child) => child.is_alignment(),
            Prerequisite::Alignment(_) => true,
            _ => false,
        }
    }

    /// Whether the tree states an alignment requirement as a condition of its own: it is one, or it's an `All` with
    /// one among its conditions. A class's or an archetype's alignment requirement is always so placed
    /// ([`DataIssue::AlignmentNotOnItsOwn`](crate::DataIssue::AlignmentNotOnItsOwn)).
    pub(crate) fn states_alignment(&self) -> bool {
        match self {
            Prerequisite::All(children) if !self.is_alignment() => {
                children.iter().any(Prerequisite::is_alignment)
            }
            _ => self.is_alignment(),
        }
    }

    /// The tree without its alignment requirement: an `All`'s other conditions (the lone one by itself), or the
    /// tree as it is when it states none. `None` when nothing is left.
    pub(crate) fn without_alignment(&self) -> Option<Prerequisite> {
        if self.is_alignment() {
            return None;
        }
        let Prerequisite::All(children) = self else {
            return Some(self.clone());
        };
        let mut rest: Vec<Prerequisite> = children
            .iter()
            .filter(|child| !child.is_alignment())
            .cloned()
            .collect();
        Some(if rest.len() == 1 {
            rest.remove(0)
        } else {
            Prerequisite::All(rest)
        })
    }

    /// Whether an `Alignment` appears anywhere in the tree, in a `Not` too.
    pub(crate) fn names_alignment(&self) -> bool {
        self.leaves()
            .any(|leaf| matches!(leaf, Prerequisite::Alignment(_)))
    }

    /// Sorts every group's children, recursively, so equal trees compare equal.
    pub fn canonicalize(&mut self) {
        match self {
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                for child in children.iter_mut() {
                    child.canonicalize();
                }
                children.sort();
            }
            Prerequisite::Not(child) => child.canonicalize(),
            _ => {}
        }
    }
}
