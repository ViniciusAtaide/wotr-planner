//! Where a choice sits in a Build, and why a picker's option can't be taken.

use std::collections::BTreeSet;

use crate::build::{GearSpot, Pick, SlotId};
use crate::engine::SlotKind;
use crate::game_data::{Ability, DeityDomains};
use crate::keys::{ClassKey, DeityKey, FeatureSelectionKey, MythicPathKey, SkillKey, WeaponKey};
use crate::prerequisite::Prerequisite;

/// Where a choice sits in a Build: what [`Engine::removable`](crate::Engine::removable) answers for, and where a
/// walk finds a choice an edit would break (which locks the edit).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Choice {
    Race,
    Subrace,
    Alignment,
    Background,
    /// The deity followed, or atheism.
    Deity,
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
    /// The first ascension's path (MR 1, [`Build::first_ascension`](crate::Build::first_ascension)).
    FirstAscension,
    MythicPath,
    PathSwitch,
    Slot(SlotId),
    /// An item the Build holds, by where it holds it (ticket 166). It belongs to no level; the walk checks it at the
    /// Build start.
    Gear(GearSpot),
    /// The character level a Mythic rank is taken at, when it is later than the rank table's
    /// ([`Build::rank_levels`](crate::Build::rank_levels), ticket 171).
    RankLevel {
        rank: u8,
    },
}

/// Why a picker's option can't be taken, or why a choice of a Build fails ([`Failure`]). A Build the sheet holds is
/// always possible, and a link to one that isn't opens nothing ([`Engine::open_link`](crate::Engine::open_link)), so
/// the pickers show these, the walk locks an edit on any of them, and [`Engine::failures`](crate::Engine::failures)
/// says which a whole Build has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Issue {
    /// The prerequisites aren't met at that point of the Build.
    Unmet(Unmet),
    /// Already held: taken earlier in the Build (or earlier at the same level). For a feat taken once per value,
    /// already held for this value. The walk gives it too for a gear item the Build holds twice (one ring in both
    /// ring places).
    AlreadyHeld,
    /// Already taken as many times as it can be (Extra Hex: 3).
    MaxTimes { times: u8 },
    /// A selection with nothing left to pick (ticket 303): every option of the selection it grants is held as often
    /// as that option can be, or fails its prerequisite, so the game refuses it
    /// (`NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable`, WrathDecompiled 6ce4a6b).
    NoOptionLeft(FeatureSelectionKey),
    /// Not taken here: a feat outside its slot's list, an option outside its selection, an item of another kind
    /// than the slot takes, or a value of the wrong kind (or none where one is needed). No picker shows it, since a
    /// picker offers only what its slot takes; it's how a walk finds a pick that an earlier change left in the wrong
    /// slot (another class at the level, another option held). The walk gives it too for a subrace of another race,
    /// an archetype of another class, an advancement the prestige class can't take, a path the first ascension or
    /// MR 3 doesn't take (or a switch that isn't one), skill points spent past the level's, and a gear item where
    /// its slot doesn't go (a belt on the hands, a manual worn, a belt used), and a Mythic rank's later level
    /// that is no later than the rank table's.
    NotAllowed,
    /// A key the game data lacks: a pick's item, a race, subrace, deity, background, class, archetype, skill, a
    /// class an advancement names, a path, or a gear item. No picker shows it, since a picker offers only what the data has; a
    /// walk finds it in a hand-made Build (a stale link, a declared Build).
    UnknownKey,
    /// A choice in a place the Build doesn't reach: a pick for a slot the walk never reached (one the level's class
    /// doesn't give, past the last level, at a Mythic rank not reached), a class choice for a class no level takes,
    /// the first ascension, path or switch while the Build doesn't reach its rank, a character level past the
    /// level cap, and a Mythic rank's later level past the last level (or of a rank the Build never gets to). No picker shows it: a picker only offers what's there.
    NotReached,
    /// The class level would go above the class's table.
    ClassLevelCap { cap: u8 },
    /// A Path switch open only from other paths than the Build's (Devil: from Aeon or Azata).
    NotFromPath { from: BTreeSet<MythicPathKey> },
    /// The Build start's deity doesn't allow its alignment: what the deity picker and the alignment wheel lock a
    /// misfit with ("Abadar allows Lawful Good, Lawful Neutral, True Neutral or Lawful Evil").
    DeityAlignment { deity: DeityKey },
    /// A Build start's alignment with no faith chosen yet: what the alignment wheel locks every alignment with
    /// until a deity or atheism is chosen ("Choose a deity or atheism first").
    NeedsFaith,
    /// A deity-limited selection's option whose domain the deity offers when the selection takes only the others,
    /// or doesn't when it takes only the deity's ("Not a domain of Erastil").
    DeityDomain {
        deity: DeityKey,
        limit: DeityDomains,
    },
    /// A feat taken for a weapon the character isn't proficient with, where the feat requires proficiency
    /// ([`Feat::requires_proficiency`](crate::Feat::requires_proficiency)), whatever its slot ignores ("Not
    /// proficient with Bastard Sword").
    NotProficient { weapon: WeaponKey },
    /// A picker's option that would break a later choice (a later pick of the same feat, or one that needs what
    /// the option replaces): the later choice comes off first, as choices come off from the leaves inward
    /// ([`Engine::removable`](crate::Engine::removable)).
    BreaksLater,
}

/// A choice of a Build that fails, and why: what [`Engine::failures`](crate::Engine::failures) lists, in walk order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub choice: Choice,
    pub issue: Issue,
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
    /// The failing children of an `All` that has more than one failure (a single failure is reported on its own),
    /// none of them an `All` (`Unmet::all`).
    All(Vec<Unmet>),
    /// Every child of an `Any`, all failing, none of them an `Any` (`Unmet::any`): an "and" among the alternatives
    /// whose one failure is an "or" gives that "or"'s alternatives, flattened in. A picker's reason then keeps only
    /// those the character's classes give, when any does ([`Engine::options`](crate::Engine::options)).
    Any(Vec<Unmet>),
    /// An `AtLeast(needed, ..)` with only `met` children holding; `failed` are the others.
    AtLeast {
        needed: u8,
        met: u8,
        failed: Vec<Unmet>,
    },
    /// `unmet` where the pick would go, though a pick in a later slot of the same character level meets it: a level
    /// fills its slots in a fixed order (class selections, class feat slots, race slots, the general slot, the slots
    /// options' rows and feats give, then each Mythic rank the level reaches), so that pick comes too late (ticket
    /// 126). Only a picker's reason has it, always outermost, around what the slot's check gave.
    MetLater {
        unmet: Box<Unmet>,
        later: Box<LaterPick>,
    },
}

impl Unmet {
    /// An "and" of failures, in order: an "and" among them gives its parts, and one failure alone is itself. With
    /// [`any`](Self::any), what keeps an `Unmet` of a canonical tree ([`Prerequisite::canonicalize`]) free of an "and"
    /// in an "and" and an "or" in an "or" (an "and" in an "or" whose one failure is an "or" gives its alternatives).
    pub(crate) fn all(failed: Vec<Unmet>) -> Unmet {
        debug_assert!(!failed.is_empty(), "an \"and\" of no failure");
        Self::joined(failed, Unmet::All, |unmet| match unmet {
            Unmet::All(parts) => Ok(parts),
            unmet => Err(unmet),
        })
    }

    /// An "or" of failures, in order: an "or" among them gives its alternatives, and one alone is itself. An empty
    /// "or" fails with none.
    pub(crate) fn any(failed: Vec<Unmet>) -> Unmet {
        Self::joined(failed, Unmet::Any, |unmet| match unmet {
            Unmet::Any(alternatives) => Ok(alternatives),
            unmet => Err(unmet),
        })
    }

    /// `group` of `failed`, each of its kind (`Ok` from `parts`) giving its children in its place; one alone is
    /// itself.
    fn joined(
        failed: Vec<Unmet>,
        group: fn(Vec<Unmet>) -> Unmet,
        parts: fn(Unmet) -> Result<Vec<Unmet>, Unmet>,
    ) -> Unmet {
        let mut joined = Vec::with_capacity(failed.len());
        for unmet in failed {
            match parts(unmet) {
                Ok(children) => joined.extend(children),
                Err(unmet) => joined.push(unmet),
            }
        }
        if joined.len() == 1 {
            return joined.remove(0);
        }
        group(joined)
    }

    /// The parts of an "and": its children, or itself alone. An "and" holds no "and" (`Unmet::all`), so the
    /// game's `PrerequisiteArchetypeLevel` beside a `PrerequisiteClassLevel` of the same class reads as their parts
    /// (ticket 329).
    pub fn and_parts(&self) -> Vec<&Unmet> {
        match self {
            Unmet::All(children) => children.iter().collect(),
            unmet => vec![unmet],
        }
    }
}

/// The pick in a later slot of the same character level that would meet an unmet prerequisite
/// ([`Unmet::MetLater`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaterPick {
    pub slot: SlotId,
    pub kind: SlotKind,
    pub pick: Pick,
    /// The character level both slots are at (a Mythic rank's slots are at the level its ascension comes at).
    pub level: u8,
    /// Whether the two picks could trade slots: each slot takes the other's pick (prerequisites included), so the
    /// reason says to swap them.
    pub swap: bool,
}
