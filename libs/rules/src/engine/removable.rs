//! Which edits keep a possible Build possible ([`Engine::allows`]) and which choices may come off now
//! ([`Engine::removable`]), both by walking the edited Build again from the first level the edit changes.

use std::collections::BTreeSet;

use super::walk::Walker;
use super::{Engine, PATH_CHOICE_RANK, PATH_SWITCH_RANK};
use crate::build::{AbilityScores, Build, BuildStart, ClassChoice, Edit};
use crate::choice::{Choice, Issue};
use crate::keys::ClassKey;

impl Engine {
    /// Whether an edit keeps a possible Build possible: it breaks no other choice (the choices it sets are the
    /// pickers' business). The edit is applied as [`apply`](Self::apply) does it. The sheet offers only edits this
    /// allows; [`options`](Self::options) and the class and path pickers fold it into each option's status.
    ///
    /// Costs one walk of the Build, then one from the first level the edit changes.
    pub fn allows(&self, build: &Build, edit: &Edit) -> bool {
        Recheck::new(self, build).keeps(edit, &[])
    }

    /// The choices in a possible Build the sheet may remove or change now: each that no later choice depends on.
    /// A choice depends on another when taking the other back would break it, so a chain comes off from the leaves
    /// inward: of Power Attack, Cleave and Great Cleave, only Great Cleave, then Cleave, then Power Attack.
    ///
    /// Taking a choice back means: a pick cleared; a level taken by no class (what its class row gives goes, and
    /// the levels of its class after it each move down one); a class's archetype or advanced classes unchosen; a
    /// point-buy score at 7; a level's ability increase or a skill's ranks at a level gone; the race (with its
    /// subrace and racial ability), subrace, alignment, background, path or Path switch unchosen. A level's own
    /// skill points don't count against changing its class: the class picker locks a class with too few.
    ///
    /// Costs one walk of the Build, then for each choice a walk from its level (the Build start's: all of it).
    pub fn removable(&self, build: &Build) -> BTreeSet<Choice> {
        let recheck = Recheck::new(self, build);
        self.withdrawals(build)
            .into_iter()
            .filter(|(_, edit, own)| recheck.keeps(edit, own.as_slice()))
            .map(|(choice, ..)| choice)
            .collect()
    }

    /// Each choice in the Build, with the edit that takes it back and any other choice whose failing that edit
    /// doesn't count (see [`removable`](Self::removable)).
    fn withdrawals(&self, build: &Build) -> Vec<(Choice, Edit, Vec<Choice>)> {
        let mut withdrawals = Vec::new();
        let start = |change: &dyn Fn(&mut BuildStart)| {
            let mut start = build.start.clone();
            change(&mut start);
            Edit::Start(start)
        };
        let begun = &build.start;
        if begun.race.is_some() {
            let edit = start(&|start| {
                start.race = None;
                start.subrace = None;
                start.racial_ability = None;
            });
            withdrawals.push((Choice::Race, edit, vec![]));
        }
        if begun.subrace.is_some() {
            let edit = start(&|start| start.subrace = None);
            withdrawals.push((Choice::Subrace, edit, vec![]));
        }
        if begun.alignment.is_some() {
            let edit = start(&|start| start.alignment = None);
            withdrawals.push((Choice::Alignment, edit, vec![]));
        }
        if begun.background.is_some() {
            let edit = start(&|start| start.background = None);
            withdrawals.push((Choice::Background, edit, vec![]));
        }
        for ability in AbilityScores::ABILITIES {
            let edit = start(&|start| {
                start.ability_scores = start
                    .ability_scores
                    .with(ability, LOWEST_SCORE)
                    .expect("a lower score always fits the budget");
            });
            withdrawals.push((Choice::AbilityScore { ability }, edit, vec![]));
        }
        for (index, level_up) in build.levels.iter().enumerate() {
            let level = u8::try_from(index + 1).unwrap_or(u8::MAX);
            withdrawals.push((
                Choice::Class { level },
                Edit::Class {
                    level,
                    class: no_class(),
                    choice: None,
                },
                vec![Choice::SkillPoints { level }],
            ));
            if level_up.ability_increase.is_some() {
                withdrawals.push((
                    Choice::AbilityIncrease { level },
                    Edit::AbilityIncrease {
                        level,
                        ability: None,
                    },
                    vec![],
                ));
            }
            for skill in level_up.skill_ranks.keys() {
                withdrawals.push((
                    Choice::SkillRanks {
                        level,
                        skill: skill.clone(),
                    },
                    Edit::SkillRanks {
                        level,
                        skill: skill.clone(),
                        ranks: 0,
                    },
                    vec![],
                ));
            }
        }
        for (class, choice) in &build.class_choices {
            let Some(first) = build.levels.iter().position(|level| &level.class == class) else {
                continue;
            };
            let level = u8::try_from(first + 1).unwrap_or(u8::MAX);
            let unchosen = |choice: ClassChoice| Edit::Class {
                level,
                class: class.clone(),
                choice: Some(choice),
            };
            if choice.archetype.is_some() {
                withdrawals.push((
                    Choice::Archetype {
                        class: class.clone(),
                    },
                    unchosen(ClassChoice {
                        archetype: None,
                        ..choice.clone()
                    }),
                    vec![],
                ));
            }
            if !choice.advances.is_empty() {
                withdrawals.push((
                    Choice::Advances {
                        class: class.clone(),
                    },
                    unchosen(ClassChoice {
                        advances: BTreeSet::new(),
                        ..choice.clone()
                    }),
                    vec![],
                ));
            }
        }
        if build.mythic_path.is_some() {
            withdrawals.push((Choice::MythicPath, Edit::MythicPath(None), vec![]));
        }
        if build.path_switch.is_some() {
            withdrawals.push((Choice::PathSwitch, Edit::PathSwitch(None), vec![]));
        }
        for slot in build.picks.keys() {
            withdrawals.push((
                Choice::Slot(slot.clone()),
                Edit::Clear(slot.clone()),
                vec![],
            ));
        }
        withdrawals
    }

    /// The first character level whose walk an edit can change; 0 for the Build start, and past the end for an
    /// edit that changes no level.
    fn first_level(&self, build: &Build, edit: &Edit) -> u8 {
        match edit {
            Edit::Pick { slot, .. } | Edit::Clear(slot) => match (slot.level(), slot.rank()) {
                (Some(level), _) => level,
                (None, Some(rank)) => self.rank_level(rank),
                (None, None) => 0,
            },
            // A new archetype or advancement holds from the class's first level.
            Edit::Class {
                level,
                class,
                choice,
            } => {
                let first = choice
                    .as_ref()
                    .and_then(|_| build.levels.iter().position(|l| &l.class == class))
                    .map(|index| u8::try_from(index + 1).unwrap_or(u8::MAX));
                first.map_or(*level, |first| first.min(*level))
            }
            Edit::AbilityIncrease { level, .. } | Edit::SkillRanks { level, .. } => *level,
            Edit::Start(_) => 0,
            Edit::MythicPath(_) => self.rank_level(PATH_CHOICE_RANK),
            Edit::PathSwitch(_) => self.rank_level(PATH_SWITCH_RANK),
            Edit::RemoveLevel | Edit::SetName(_) => u8::MAX,
        }
    }
}

/// A possible Build walked once and paused before each level, so an edit is walked only from the first level it
/// changes.
pub(super) struct Recheck<'a> {
    engine: &'a Engine,
    build: &'a Build,
    /// `paused[i]` is the walk just before character level `i + 1` (after the Build start); the last one is after
    /// the last level.
    paused: Vec<Walker<'a>>,
}

impl<'a> Recheck<'a> {
    pub(super) fn new(engine: &'a Engine, build: &'a Build) -> Self {
        let mut walker = Walker::new(engine, build, None);
        walker.start();
        let mut paused = Vec::with_capacity(build.levels.len() + 1);
        for (index, level_up) in build.levels.iter().enumerate() {
            paused.push(walker.clone());
            walker.level(u8::try_from(index + 1).unwrap_or(u8::MAX), level_up);
        }
        paused.push(walker);
        Self {
            engine,
            build,
            paused,
        }
    }

    /// Whether `edit` breaks no choice, other than the ones it sets and `own`.
    fn keeps(&self, edit: &Edit, own: &[Choice]) -> bool {
        let mut edited = self.build.clone();
        self.engine.apply(&mut edited, edit);
        if &edited == self.build {
            return true;
        }
        let first = self.engine.first_level(self.build, edit);
        let walk = match first.checked_sub(1) {
            None => Walker::new(self.engine, &edited, None).run(),
            Some(index) => {
                let Some(paused) = self.paused.get(usize::from(index)) else {
                    // Nothing after the last level depends on an edit past it (Remove level, a name).
                    return true;
                };
                let mut walker = paused.clone();
                walker.build = &edited;
                walker.finish(first)
            }
        };
        let set = edit.choices();
        walk.failed
            .iter()
            .all(|choice| set.contains(choice) || own.contains(choice))
    }

    /// A picker's status for an option open at its point of the Build: `Ok`, or [`Issue::BreaksLater`].
    pub(super) fn later(&self, edit: &Edit) -> Result<(), Issue> {
        if self.keeps(edit, &[]) {
            Ok(())
        } else {
            Err(Issue::BreaksLater)
        }
    }
}

/// The lowest point-buy score, which taking a score back lowers it to.
const LOWEST_SCORE: u8 = 7;

/// The class of a level taken back: a key no game data has (keys are never empty), so the level gives nothing.
fn no_class() -> ClassKey {
    ClassKey::new("")
}
