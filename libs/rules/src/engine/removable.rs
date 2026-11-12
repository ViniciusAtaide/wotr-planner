//! Which edits keep a possible Build possible ([`Engine::allows`]) and which choices may come off now
//! ([`Engine::removable`]), both by walking the edited Build again from the first level the edit changes (the
//! cursor's walks, [`Cursor::keeps`]). Which choices a Build holds, each with its [`Withdrawal`]
//! ([`Engine::held_choices`]).

use std::collections::BTreeSet;

use super::walk::Cursor;
use super::{Engine, Withdrawal};
use crate::build::{AbilityScores, Build, BuildStart, ClassChoice, Edit};
use crate::choice::Choice;
use crate::keys::ClassKey;

impl Engine {
    /// Whether an edit keeps a possible Build possible: it breaks no other choice (the choices it sets are the
    /// pickers' business). The edit is applied as [`apply`](Self::apply) does it. The sheet offers only edits this
    /// allows; every picker and stepper folds it into each option's status.
    ///
    /// Costs one walk of the Build up to the first level the edit changes, then one from there.
    pub fn allows(&self, build: &Build, edit: &Edit) -> bool {
        Cursor::new(self, build).keeps_once(edit, edit.choices())
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
        // The Build holds each choice once (`held_choices`).
        self.held_choices(build)
            .into_iter()
            .find(|held| &held.choice == choice)
            .is_some_and(|held| Cursor::new(self, build).keeps_once(&held.edit, held.excused))
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
        let removable = Cursor::new(self, build).keeps_once(&held.edit, held.excused);
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
        let cursor = Cursor::new(self, build);
        self.held_choices(build)
            .into_iter()
            .filter(|held| wanted(&held.choice) && cursor.keeps(&held.edit, held.excused.clone()))
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
}

/// The lowest point-buy score, which taking a score back lowers it to.
const LOWEST_SCORE: u8 = 7;

/// The class of a level taken back: a key no game data has (keys are never empty), so the level gives nothing.
fn no_class() -> ClassKey {
    ClassKey::new("")
}
