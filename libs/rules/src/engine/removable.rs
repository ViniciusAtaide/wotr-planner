//! Which edits keep a possible Build possible ([`Engine::allows`]) and which choices may come off now
//! ([`Engine::removable`]), both by walking the edited Build again from the first level the edit changes. Those
//! walks keep no sheet and stop at the first choice that fails. Which choices a Build holds, each with its
//! [`Withdrawal`] ([`Engine::held_choices`]).

use std::cell::RefCell;
use std::collections::BTreeSet;

use super::walk::Walker;
use super::{
    Engine, FIRST_ASCENSION_RANK, PATH_CHOICE_RANK, PATH_SWITCH_RANK, SkillPoints, Withdrawal,
};
use crate::build::{AbilityScores, Build, BuildStart, ClassChoice, Edit};
use crate::choice::{Choice, Issue};
use crate::keys::ClassKey;

impl Engine {
    /// Whether an edit keeps a possible Build possible: it breaks no other choice (the choices it sets are the
    /// pickers' business). The edit is applied as [`apply`](Self::apply) does it. The sheet offers only edits this
    /// allows; every picker and stepper folds it into each option's status.
    ///
    /// Costs one walk of the Build up to the first level the edit changes, then one from there.
    pub fn allows(&self, build: &Build, edit: &Edit) -> bool {
        Recheck::new(self, build).keeps(edit, edit.choices())
    }

    /// The choices in a possible Build the sheet may remove or change now: each that no later choice depends on.
    /// A choice depends on another when taking the other back would break it, so a chain comes off from the leaves
    /// inward: of Power Attack, Cleave and Great Cleave, only Great Cleave, then Cleave, then Power Attack.
    ///
    /// Taking a choice back is its [`Withdrawal`]. A level's own skill points don't count against changing its
    /// class: the class picker locks a class with too few.
    ///
    /// Costs one walk of the Build, then for each choice a walk from its level (the Build start's: all of it). The
    /// sheet asks only for the class rows ([`removable_classes`](Self::removable_classes)) and for one choice at a
    /// time ([`withdrawal`](Self::withdrawal)).
    pub fn removable(&self, build: &Build) -> BTreeSet<Choice> {
        self.removable_among(build, |_| true)
    }

    /// Whether one choice may be removed or changed now: whether [`removable`](Self::removable) has it.
    ///
    /// Costs one walk of the Build up to the choice's level, then one from there.
    pub fn is_removable(&self, build: &Build, choice: &Choice) -> bool {
        !self.removable_among(build, |c| c == choice).is_empty()
    }

    /// What a picker's remove control asks of one choice: `None` when the Build doesn't hold it, otherwise its
    /// [`Withdrawal`]'s edit and whether it may be removed now ([`is_removable`](Self::is_removable)).
    ///
    /// Costs one walk of the Build up to the choice's level, then one from there.
    pub fn withdrawal(&self, build: &Build, choice: &Choice) -> Option<(Edit, bool)> {
        let held = self
            .held_choices(build)
            .into_iter()
            .find(|held| &held.choice == choice)?;
        let removable = Recheck::new(self, build).keeps(&held.edit, held.excused);
        Some((held.edit, removable))
    }

    /// The character levels whose class may be changed now: the [`Choice::Class`] levels
    /// [`removable`](Self::removable) has. What the levels grid locks its class buttons by.
    ///
    /// Costs one walk of the Build, then for each level one from there.
    pub fn removable_classes(&self, build: &Build) -> BTreeSet<u8> {
        self.removable_among(build, |choice| matches!(choice, Choice::Class { .. }))
            .into_iter()
            .filter_map(|choice| match choice {
                Choice::Class { level } => Some(level),
                _ => None,
            })
            .collect()
    }

    /// The choices [`removable`](Self::removable) has among those `wanted` picks out.
    fn removable_among(&self, build: &Build, wanted: impl Fn(&Choice) -> bool) -> BTreeSet<Choice> {
        let recheck = Recheck::new(self, build);
        self.held_choices(build)
            .into_iter()
            .filter(|held| wanted(&held.choice) && recheck.keeps(&held.edit, held.excused.clone()))
            .map(|held| held.choice)
            .collect()
    }

    /// Every choice the Build holds, removable or not, each once with its [`Withdrawal`]: the Build start's
    /// (every point-buy score always), each level's class, ability increase and skill ranks, the last level, each
    /// class's archetype and advanced classes, the first ascension, path and Path switch, and each pick. A level's
    /// skill points are what its ranks spend, not a choice of their own.
    ///
    /// Costs no walk.
    pub fn held_choices(&self, build: &Build) -> Vec<Withdrawal> {
        let mut withdrawals: Vec<Withdrawal> = Vec::new();
        let start = |change: &dyn Fn(&mut BuildStart)| {
            let mut start = build.start.clone();
            change(&mut start);
            Edit::Start(start)
        };
        let mut push = |choice: Choice, edit: Edit, own: Vec<Choice>| {
            // The faith can't come off while the alignment needs it: a Build start choice excuses only itself.
            let mut excused = match edit {
                Edit::Start(_) => vec![choice.clone()],
                _ => edit.choices(),
            };
            excused.extend(own);
            withdrawals.push(Withdrawal {
                choice,
                edit,
                excused,
            });
        };
        let begun = &build.start;
        if begun.race.is_some() {
            let edit = start(&|start| {
                start.race = None;
                start.subrace = None;
                start.racial_ability = None;
            });
            push(Choice::Race, edit, vec![]);
        }
        if begun.subrace.is_some() {
            let edit = start(&|start| start.subrace = None);
            push(Choice::Subrace, edit, vec![]);
        }
        if begun.alignment.is_some() {
            let edit = start(&|start| start.alignment = None);
            push(Choice::Alignment, edit, vec![]);
        }
        if begun.background.is_some() {
            let edit = start(&|start| start.background = None);
            push(Choice::Background, edit, vec![]);
        }
        if begun.deity.is_some() {
            let edit = start(&|start| start.deity = None);
            push(Choice::Deity, edit, vec![]);
        }
        for ability in AbilityScores::ABILITIES {
            let edit = start(&|start| {
                start.ability_scores = start
                    .ability_scores
                    .with(ability, LOWEST_SCORE)
                    .expect("a lower score always fits the budget");
            });
            push(Choice::AbilityScore { ability }, edit, vec![]);
        }
        for (index, level_up) in build.levels.iter().enumerate() {
            let level = u8::try_from(index + 1).unwrap_or(u8::MAX);
            push(
                Choice::Class { level },
                Edit::Class {
                    level,
                    class: no_class(),
                    choice: None,
                },
                vec![Choice::SkillPoints { level }],
            );
            if level_up.ability_increase.is_some() {
                push(
                    Choice::AbilityIncrease { level },
                    Edit::AbilityIncrease {
                        level,
                        ability: None,
                    },
                    vec![],
                );
            }
            for skill in level_up.skill_ranks.keys() {
                push(
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
                );
            }
        }
        if let Ok(level) = u8::try_from(build.levels.len())
            && level > 0
        {
            push(Choice::CharacterLevel { level }, Edit::RemoveLevel, vec![]);
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
                push(
                    Choice::Archetype {
                        class: class.clone(),
                    },
                    unchosen(ClassChoice {
                        archetype: None,
                        ..choice.clone()
                    }),
                    vec![],
                );
            }
            if !choice.advances.is_empty() {
                push(
                    Choice::Advances {
                        class: class.clone(),
                    },
                    unchosen(ClassChoice {
                        advances: BTreeSet::new(),
                        ..choice.clone()
                    }),
                    vec![],
                );
            }
        }
        if build.first_ascension.is_some() {
            push(Choice::FirstAscension, Edit::FirstAscension(None), vec![]);
        }
        if build.mythic_path.is_some() {
            push(Choice::MythicPath, Edit::MythicPath(None), vec![]);
        }
        if build.path_switch.is_some() {
            push(Choice::PathSwitch, Edit::PathSwitch(None), vec![]);
        }
        for slot in build.picks.keys() {
            push(
                Choice::Slot(slot.clone()),
                Edit::Clear(slot.clone()),
                vec![],
            );
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
            Edit::FirstAscension(_) => self.rank_level(FIRST_ASCENSION_RANK),
            // Without a first ascension of its own, the Build takes the MR 3 path's, from MR 1.
            Edit::MythicPath(_) if build.first_ascension.is_none() => {
                self.rank_level(FIRST_ASCENSION_RANK)
            }
            Edit::MythicPath(_) => self.rank_level(PATH_CHOICE_RANK),
            Edit::PathSwitch(_) => self.rank_level(PATH_SWITCH_RANK),
            Edit::RemoveLevel | Edit::SetName(_) => u8::MAX,
        }
    }
}

/// A possible Build walked once and paused before each level, so an edit is walked only from the first level it
/// changes. The walk goes only as far as the edits asked about need.
pub(super) struct Recheck<'a> {
    engine: &'a Engine,
    build: &'a Build,
    /// `paused[i]` is the walk just before character level `i + 1` (after the Build start); the one after the last
    /// level is the last there can be. Filled as far as an edit has needed.
    paused: RefCell<Vec<Walker<'a>>>,
}

impl<'a> Recheck<'a> {
    pub(super) fn new(engine: &'a Engine, build: &'a Build) -> Self {
        Self {
            engine,
            build,
            paused: RefCell::new(Vec::new()),
        }
    }

    /// The walk just before character level `index + 1`; `None` past the one after the last level.
    fn paused(&self, index: usize) -> Option<Walker<'a>> {
        self.walk_to(index)
            .then(|| self.paused.borrow().get(index).cloned())
            .flatten()
    }

    /// The skill points of a character level the Build has, off the walk paused after it.
    pub(super) fn skill_points(&self, level: u8) -> Option<SkillPoints> {
        let index = usize::from(level);
        (level > 0 && self.walk_to(index)).then(|| self.paused.borrow()[index].skill_points)
    }

    /// Walks as far as just before character level `index + 1`; `false` past the one after the last level.
    fn walk_to(&self, index: usize) -> bool {
        let build = self.build;
        if index > build.levels.len() {
            return false;
        }
        let mut paused = self.paused.borrow_mut();
        if paused.is_empty() {
            let mut walker = Walker::new(self.engine, build, None).without_sheet();
            walker.start();
            paused.push(walker);
        }
        while paused.len() <= index {
            let walked = paused.len() - 1;
            let mut walker = paused[walked].clone();
            walker.level(
                u8::try_from(walked + 1).unwrap_or(u8::MAX),
                &build.levels[walked],
            );
            paused.push(walker);
        }
        true
    }

    /// Whether `edit` breaks no choice other than the `allowed` ones (an edit's own: [`Edit::choices`]). An edit
    /// that changes nothing breaks nothing.
    pub(super) fn keeps(&self, edit: &Edit, allowed: Vec<Choice>) -> bool {
        let mut edited = self.build.clone();
        self.engine.apply(&mut edited, edit);
        &edited == self.build || self.walks(edit, &edited, allowed)
    }

    /// Whether the walk of `edited` (the Build with `edit` applied) fails no choice but the `allowed` ones, even
    /// when `edit` changes nothing: then it's the Build's own walk.
    fn walks(&self, edit: &Edit, edited: &Build, allowed: Vec<Choice>) -> bool {
        let first = self.engine.first_level(self.build, edit);
        match first.checked_sub(1) {
            None => Walker::new(self.engine, edited, None)
                .without_sheet()
                .holds(0, allowed),
            Some(index) => {
                let Some(mut walker) = self.paused(usize::from(index)) else {
                    // Nothing after the last level depends on an edit past it (Remove level, a name).
                    return true;
                };
                walker.build = edited;
                walker.holds(first, allowed)
            }
        }
    }

    /// A picker's status for an option open at its point of the Build: `Ok`, or [`Issue::BreaksLater`].
    pub(super) fn later(&self, edit: &Edit) -> Result<(), Issue> {
        status(self.keeps(edit, edit.choices()))
    }

    /// [`later`](Self::later) for an edit's walk even when the edit changes nothing (the walk of the Build itself,
    /// which fails a choice only when the Build isn't possible): what a picker's option gets when its walk is the
    /// same as that one.
    pub(super) fn later_walked(&self, edit: &Edit) -> Result<(), Issue> {
        let mut edited = self.build.clone();
        self.engine.apply(&mut edited, edit);
        status(self.walks(edit, &edited, edit.choices()))
    }
}

fn status(keeps: bool) -> Result<(), Issue> {
    if keeps {
        Ok(())
    } else {
        Err(Issue::BreaksLater)
    }
}

/// The lowest point-buy score, which taking a score back lowers it to.
const LOWEST_SCORE: u8 = 7;

/// The class of a level taken back: a key no game data has (keys are never empty), so the level gives nothing.
fn no_class() -> ClassKey {
    ClassKey::new("")
}
