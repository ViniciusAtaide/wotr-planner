//! The pickers: every option a slot, a level's class, the path or the Path switch can take, each with whether it
//! can be taken there and why not.

use std::cell::OnceCell;
use std::collections::{BTreeSet, HashSet};

use super::removable::Recheck;
use super::slots::check_pick;
use super::walk::Probe;
use super::{ClassOption, Engine, PathOption, SlotKind, SlotOption, class_level_cap};
use crate::build::{Build, ClassChoice, Edit, Pick, SlotId};
use crate::choice::{Issue, Unmet};
use crate::game_data::{Archetype, Class};
use crate::keys::{ArchetypeKey, FeatKey, SelectionOptionKey};
use crate::prerequisite::{Prerequisite, Story};
use crate::state::{CharacterState, ClassLevels};

impl Engine {
    /// Every option a slot accepts, in key order (a selection's own options and the ones it shares), each with
    /// whether it can be picked at that point of the Build and why not. An option that would break a later choice
    /// is [`Issue::BreaksLater`] (Toughness at level 1 with Toughness at level 3; anything but Cleave in Cleave's
    /// slot while Great Cleave needs it). `None` when the Build has no such slot.
    ///
    /// Costs one walk to the slot's level, then a walk from the slot's level for the slot left empty and for each
    /// option open there that a later check could see ([`named_feats`](Self::named_feats)); each stops at the
    /// first choice that fails.
    pub fn options(&self, build: &Build, slot: &SlotId) -> Option<Vec<SlotOption>> {
        let probed = self.probe(build, Probe::Slot(slot));
        let (state, kind) = (probed.state?, probed.kind?);
        let candidates: Vec<Pick> = match &kind {
            // A parameterised feat once per value.
            SlotKind::Feat(feat_slot) => self
                .data
                .feats
                .iter()
                .filter(|feat| self.slot_accepts_in(feat_slot, feat, &state))
                .flat_map(|feat| match feat.parameter {
                    None => vec![Pick::feat(feat.key.clone())],
                    Some(kind) => self
                        .parameter_values(kind)
                        .into_iter()
                        .map(|value| Pick::feat_for(feat.key.clone(), value))
                        .collect(),
                })
                .collect(),
            SlotKind::Selection(key) => self
                .selection(key)
                .map(|selection| {
                    // Own options and shared ones (a pool selection's: the held options' pools), in key order. A
                    // shared key the data lacks offers nothing.
                    let pools = self.pools(selection, &state);
                    let mut keys: Vec<&SelectionOptionKey> = selection
                        .options
                        .iter()
                        .map(|option| &option.key)
                        .chain(
                            selection
                                .shared_options
                                .iter()
                                .filter(|key| self.indexes.options.contains_key(*key)),
                        )
                        .chain(&pools)
                        .collect();
                    keys.sort();
                    // An option with a chosen feat once per value.
                    keys.into_iter()
                        .flat_map(|key| match self.option(key) {
                            Some(option) if self.option_parameter(option).is_some() => self
                                .option_values(option)
                                .into_iter()
                                .map(|value| Pick::option_for(key.clone(), value))
                                .collect(),
                            _ => vec![Pick::option(key.clone())],
                        })
                        .collect()
                })
                .unwrap_or_default(),
            SlotKind::MythicAbility => self
                .data
                .mythic_abilities
                .iter()
                .map(|ability| Pick::MythicAbility(ability.key.clone()))
                .collect(),
            SlotKind::MythicFeat => self
                .data
                .mythic_feats
                .iter()
                .flat_map(|feat| match feat.parameter {
                    None => vec![Pick::mythic_feat(feat.key.clone())],
                    Some(kind) => self
                        .parameter_values(kind)
                        .into_iter()
                        .map(|value| Pick::mythic_feat_for(feat.key.clone(), value))
                        .collect(),
                })
                .collect(),
        };
        let recheck = Recheck::new(self, build);
        let named = self.named_feats(build);
        let left_empty = OnceCell::new();
        let later = |pick: &Pick| match pick {
            // A feat no check names breaks later just what the slot left empty would.
            Pick::Feat { feat, .. } if !named.contains(feat) => left_empty
                .get_or_init(|| recheck.later_walked(&Edit::Clear(slot.clone())))
                .clone(),
            _ => recheck.later(&Edit::Pick {
                slot: slot.clone(),
                pick: pick.clone(),
            }),
        };
        Some(
            candidates
                .into_iter()
                .map(|pick| SlotOption {
                    status: check_pick(self, &state, &kind, &pick)
                        .map_err(|issue| self.reason_for(issue, &state))
                        .and_then(|()| later(&pick)),
                    pick,
                })
                .collect(),
        )
    }

    /// The feats a walk of the Build can look at: each feat picked (whether it's held already), each option's
    /// chosen feat, and the feats named by the prerequisites of the picks and of the classes and archetypes the
    /// Build takes. Holding a feat touches nothing but that feat's entries in the state, so a feat pick outside
    /// these changes no later check: the walk with it fails exactly the choices the walk without it fails.
    fn named_feats<'b>(&'b self, build: &'b Build) -> HashSet<&'b FeatKey> {
        let mut named = HashSet::new();
        for pick in build.picks.values() {
            let prerequisite = match pick {
                Pick::Feat { feat, .. } => {
                    named.insert(feat);
                    self.feat(feat).and_then(|f| f.prerequisite.as_ref())
                }
                Pick::SelectionOption { option, .. } => self.option(option).and_then(|o| {
                    named.extend(&o.chosen_feat);
                    o.prerequisite.as_ref()
                }),
                Pick::MythicAbility(key) => self
                    .mythic_ability(key)
                    .and_then(|a| a.prerequisite.as_ref()),
                Pick::MythicFeat { feat, .. } => {
                    self.mythic_feat(feat).and_then(|f| f.prerequisite.as_ref())
                }
            };
            if let Some(prerequisite) = prerequisite {
                prerequisite.collect_feats(&mut named);
            }
        }
        for level in &build.levels {
            if let Some((class, _)) = self.class(&level.class) {
                let archetypes = class.archetypes.iter().map(|a| a.prerequisite.as_ref());
                for prerequisite in std::iter::once(class.prerequisite.as_ref())
                    .chain(archetypes)
                    .flatten()
                {
                    prerequisite.collect_feats(&mut named);
                }
            }
        }
        named
    }

    /// The story conditions in a pick's prerequisite (Vow of Dahak), for the note the sheet shows beside it. They
    /// never block the pick. Empty for a key the data lacks.
    pub fn story_conditions(&self, pick: &Pick) -> BTreeSet<Story> {
        let prerequisite = match pick {
            Pick::Feat { feat, .. } => self.feat(feat).and_then(|f| f.prerequisite.as_ref()),
            Pick::SelectionOption { option, .. } => {
                self.option(option).and_then(|o| o.prerequisite.as_ref())
            }
            Pick::MythicAbility(key) => self
                .mythic_ability(key)
                .and_then(|a| a.prerequisite.as_ref()),
            Pick::MythicFeat { feat: key, .. } => {
                self.mythic_feat(key).and_then(|f| f.prerequisite.as_ref())
            }
        };
        prerequisite.map(Prerequisite::stories).unwrap_or_default()
    }

    /// The class picker at a character level (one past the end adds a level): every base class, each followed by
    /// its archetypes, then every prestige class, each in key order. A class the Build already has before that
    /// level is offered only with its archetype, which was fixed when it was first taken. At a level the Build
    /// has, a class that would break a later choice (or the level's own skill ranks) is [`Issue::BreaksLater`];
    /// a level after the last breaks nothing. `None` for a level that isn't in the Build or next.
    pub fn class_options(&self, build: &Build, level: u8) -> Option<Vec<ClassOption>> {
        if level == 0 || usize::from(level) > build.levels.len() + 1 {
            return None;
        }
        let state = self.probe(build, Probe::Level(level)).state?;
        let recheck = (usize::from(level) <= build.levels.len()).then(|| Recheck::new(self, build));
        let mut options = Vec::new();
        for class in self.data.classes.iter().chain(&self.data.prestige_classes) {
            let taken = state.classes.iter().find(|c| c.class == class.key);
            let archetypes: Vec<Option<&Archetype>> = match taken {
                Some(taken) => vec![self.archetype_of(class, taken.archetype.as_ref())],
                None => std::iter::once(None)
                    .chain(class.archetypes.iter().map(Some))
                    .collect(),
            };
            for archetype in archetypes {
                let archetype = archetype.map(|a| a.key.clone());
                let mut status = self.class_status(&state, class, archetype.as_ref(), taken);
                if status.is_ok()
                    && let Some(recheck) = &recheck
                {
                    // A class first taken here gets this archetype and keeps any advancement it has.
                    let choice = taken.is_none().then(|| ClassChoice {
                        archetype: archetype.clone(),
                        advances: build
                            .class_choices
                            .get(&class.key)
                            .map(|choice| choice.advances.clone())
                            .unwrap_or_default(),
                    });
                    status = recheck.later(&Edit::Class {
                        level,
                        class: class.key.clone(),
                        choice,
                    });
                }
                options.push(ClassOption {
                    class: class.key.clone(),
                    archetype,
                    status,
                });
            }
        }
        Some(options)
    }

    /// Whether a class (with an archetype) can be taken next by a character in `state`.
    fn class_status(
        &self,
        state: &CharacterState,
        class: &Class,
        archetype: Option<&ArchetypeKey>,
        taken: Option<&ClassLevels>,
    ) -> Result<(), Issue> {
        let archetype = self.archetype_of(class, archetype);
        let cap = class_level_cap(class);
        if state.class_level(&class.key) >= cap {
            return Err(Issue::ClassLevelCap { cap });
        }
        if taken.is_some() {
            return Ok(());
        }
        let unmet: Vec<Unmet> = [
            class.prerequisite.as_ref(),
            archetype.and_then(|a| a.prerequisite.as_ref()),
        ]
        .into_iter()
        .flatten()
        .filter_map(|prerequisite| state.check(prerequisite).err())
        .collect();
        match <[Unmet; 1]>::try_from(unmet) {
            Ok([unmet]) => Err(Issue::Unmet(unmet)),
            Err(unmet) if unmet.is_empty() => Ok(()),
            Err(unmet) => Err(Issue::Unmet(Unmet::All(unmet))),
        }
    }

    /// The path picker at MR 3: every path without a switch rule, in key order; one that would break a later
    /// choice (a pick in the Build's path's slots) is [`Issue::BreaksLater`].
    pub fn path_options(&self, build: &Build) -> Vec<PathOption> {
        let recheck = Recheck::new(self, build);
        self.data
            .mythic_paths
            .iter()
            .filter(|path| path.switch.is_none())
            .map(|path| PathOption {
                path: path.key.clone(),
                status: recheck.later(&Edit::MythicPath(Some(path.key.clone()))),
            })
            .collect()
    }

    /// The Path switch picker at MR 8: every path with a switch rule, in key order, each open or not from the
    /// path the Build chose at MR 3; one that would break a later choice (Gold Dragon for a Build past level 20)
    /// is [`Issue::BreaksLater`].
    pub fn switch_options(&self, build: &Build) -> Vec<PathOption> {
        let recheck = Recheck::new(self, build);
        self.data
            .mythic_paths
            .iter()
            .filter_map(|path| Some((path, path.switch.as_ref()?)))
            .map(|(path, switch)| PathOption {
                path: path.key.clone(),
                status: if switch.open_from(build.mythic_path.as_ref()) {
                    recheck.later(&Edit::PathSwitch(Some(path.key.clone())))
                } else {
                    Err(Issue::NotFromPath {
                        from: switch.from.clone(),
                    })
                },
            })
            .collect()
    }
}
