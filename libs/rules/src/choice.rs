//! Where a choice sits in a Build, and why a picker's option can't be taken.

use std::collections::BTreeSet;

use crate::build::SlotId;
use crate::game_data::Ability;
use crate::keys::{ClassKey, MythicPathKey, SkillKey};
use crate::prerequisite::Prerequisite;

/// Where a choice sits in a Build: what [`Engine::removable`](crate::Engine::removable) answers for, and where a
/// walk finds a choice an edit would break (which locks the edit).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Choice {
    Race,
    Subrace,
    Alignment,
    Background,
    /// One point-buy score.
    AbilityScore {
        ability: Ability,
    },
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

/// Why a picker's option can't be taken. A Build the sheet holds is always possible, and a link to one that isn't
/// opens nothing ([`Engine::open_link`](crate::Engine::open_link)), so these are only the reasons a picker shows,
/// and [`NotAllowed`](Self::NotAllowed), which the walk needs to lock an edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Issue {
    /// The prerequisites aren't met at that point of the Build.
    Unmet(Unmet),
    /// Already held: taken earlier in the Build (or earlier at the same level). For a feat taken once per value,
    /// already held for this value.
    AlreadyHeld,
    /// Already taken as many times as it can be (Extra Hex: 3).
    MaxTimes { times: u8 },
    /// Not a pick the slot takes: a feat outside its list, an option outside its selection, a key the game data
    /// lacks, or a value of the wrong kind (or none where one is needed). No picker shows it, since a picker offers
    /// only what its slot takes; it's how a walk finds a pick that an earlier change left in the wrong slot (another
    /// class at the level, another option held).
    NotAllowed,
    /// The class level would go above the class's table.
    ClassLevelCap { cap: u8 },
    /// A Path switch open only from other paths than the Build's (Devil: from Aeon or Azata).
    NotFromPath { from: BTreeSet<MythicPathKey> },
    /// A picker's option that would break a later choice (a later pick of the same feat, or one that needs what
    /// the option replaces): the later choice comes off first, as choices come off from the leaves inward
    /// ([`Engine::removable`](crate::Engine::removable)).
    BreaksLater,
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
    /// Every child of an `Any`, all failing. A picker's reason keeps only those the character's classes give, when
    /// any does ([`Engine::options`](crate::Engine::options)).
    Any(Vec<Unmet>),
    /// An `AtLeast(needed, ..)` with only `met` children holding; `failed` are the others.
    AtLeast {
        needed: u8,
        met: u8,
        failed: Vec<Unmet>,
    },
}
