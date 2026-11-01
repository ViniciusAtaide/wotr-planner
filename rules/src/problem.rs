//! What can be wrong with a choice in a Build, and where.

use std::collections::BTreeSet;

use crate::build::SlotId;
use crate::game_data::Ability;
use crate::keys::{ClassKey, MythicPathKey, SkillKey};
use crate::prerequisite::Prerequisite;

/// One choice in a Build that doesn't hold up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub at: Choice,
    pub issue: Issue,
}

impl Problem {
    /// A key the current game data doesn't have ("removed from game data" on the stale-link banner).
    pub fn is_unknown_key(&self) -> bool {
        matches!(self.issue, Issue::UnknownKey(_))
    }
}

/// Where a choice sits in a Build.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Choice {
    Race,
    Subrace,
    Alignment,
    AbilityScores,
    /// The class taken at a character level (its entry prerequisites, on first entry).
    Class {
        level: u8,
    },
    /// A class's archetype ([`Build::class_choices`](crate::Build::class_choices)), checked on its first level.
    Archetype {
        class: ClassKey,
    },
    /// The classes a prestige class advances ([`Build::class_choices`](crate::Build::class_choices)), checked on
    /// its first level.
    Advances {
        class: ClassKey,
    },
    AbilityIncrease {
        level: u8,
    },
    SkillRanks {
        level: u8,
        skill: SkillKey,
    },
    /// The Level-up's skill points as a whole.
    SkillPoints {
        level: u8,
    },
    /// The Character level itself (beyond the cap).
    CharacterLevel {
        level: u8,
    },
    MythicPath,
    PathSwitch,
    Slot(SlotId),
}

/// Why a choice doesn't hold up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Issue {
    /// The key isn't in the current game data.
    UnknownKey(String),
    /// The prerequisites aren't met at that point of the Build.
    Unmet(Unmet),
    /// Already held: taken earlier in the Build (or earlier at the same level). For a feat taken once per value,
    /// already held for this value.
    AlreadyHeld,
    /// Already taken as many times as it can be (Extra Hex: 3).
    MaxTimes {
        times: u8,
    },
    /// A pick of a parameterised feat, or of an option with a chosen feat, that names no value.
    ParameterMissing,
    /// A value the pick can't be taken for: one of another kind (a skill for Weapon Focus, a simple weapon for
    /// Exotic Weapon Proficiency), or any value for a pick that takes none.
    WrongParameter,
    /// Not one of the things this choice accepts: a feat outside the slot's list, an archetype of another class, a
    /// subrace of another race.
    NotAllowed,
    /// A pick for a slot the Build doesn't have (any more).
    NoSuchSlot,
    /// The character level is above the cap (20, or the switched-to path's).
    LevelCap {
        cap: u8,
    },
    /// The class level is above the class's table or the path's cap.
    ClassLevelCap {
        cap: u8,
    },
    /// Total ranks in a skill above the cap (the character level).
    SkillRankCap {
        cap: u8,
        ranks: u8,
    },
    SkillPointsOverspent {
        available: u8,
        spent: u8,
    },
    /// The point buy costs more than the budget.
    PointBuy {
        cost: u8,
        budget: u8,
    },
    /// A point-buy score outside 7..=18.
    ScoreOutOfRange {
        ability: Ability,
        score: u8,
    },
    /// A path that can't be taken this way: a switch-only path at MR 3, or a path without a switch rule at MR 8.
    NotChoosable,
    /// A Path switch open only from other paths than the Build's (Devil: from Aeon or Azata).
    NotFromPath {
        from: BTreeSet<MythicPathKey>,
    },
    /// An ability increase at a level that has none.
    NoAbilityIncrease,
}

/// Why a Prerequisite fails, shaped like the part of the tree that fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unmet {
    /// A single condition that doesn't hold. `has` is the character's current value for numeric conditions (BAB,
    /// ability score, ranks, class level, feature count, spell level, caster level, character level).
    Condition {
        needs: Prerequisite,
        has: Option<u8>,
    },
    /// The failing children of an `All` that has more than one failure (a single failure is reported on its own).
    All(Vec<Unmet>),
    /// Every child of an `Any`, all failing.
    Any(Vec<Unmet>),
    /// An `AtLeast(needed, ..)` with only `met` children holding; `failed` are the others.
    AtLeast {
        needed: u8,
        met: u8,
        failed: Vec<Unmet>,
    },
}

impl Unmet {
    /// Every failing single condition, for a flat "Needs: …" line.
    pub fn conditions(&self) -> Vec<(&Prerequisite, Option<u8>)> {
        match self {
            Unmet::Condition { needs, has } => vec![(needs, *has)],
            Unmet::All(children) | Unmet::Any(children) => {
                children.iter().flat_map(Unmet::conditions).collect()
            }
            Unmet::AtLeast { failed, .. } => failed.iter().flat_map(Unmet::conditions).collect(),
        }
    }
}
