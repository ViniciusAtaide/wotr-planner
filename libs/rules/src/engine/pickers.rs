//! The pickers: every option a slot, a level's class, the path or the Path switch can take, each with whether it
//! can be taken there and why not, and the edit that takes it; a slot's options also come grouped as its picker shows
//! them ([`Engine::slot_entries`]). The Build start's and a level's other controls are in `controls`.

use std::cell::OnceCell;
use std::collections::{BTreeSet, HashSet};

use super::removable::Recheck;
use super::slots::check_pick_with;
use super::walk::Probe;
use super::{
    AdvancesOption, ClassOption, Engine, EntryOption, PathOption, SlotEntry, SlotKind, SlotOption,
    advanced_spellcasting, class_level_cap, most_advanced,
};
use crate::build::{Build, ClassChoice, Edit, Pick, SlotId};
use crate::choice::{Issue, Unmet};
use crate::game_data::{Archetype, Class};
use crate::keys::{ArchetypeKey, ClassKey, FeatKey, SelectionOptionKey};
use crate::parameter::ParameterKind;
use crate::prerequisite::Prerequisite;
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
        // A feat slot's lists, worked out once for every feat.
        let lists = match &kind {
            SlotKind::Feat(feat_slot) => self.catalog.slot_lists(feat_slot, &state),
            _ => Vec::new(),
        };
        let candidates: Vec<Pick> = match &kind {
            // A parameterised feat once per value.
            SlotKind::Feat(feat_slot) => self
                .catalog
                .data()
                .feats
                .iter()
                .filter(|feat| self.catalog.slot_accepts_with(feat_slot, feat, &lists))
                .flat_map(|feat| match feat.parameter {
                    None => vec![Pick::feat(feat.key.clone())],
                    Some(kind) => self
                        .catalog
                        .parameter_values(kind)
                        .into_iter()
                        .map(|value| Pick::feat_for(feat.key.clone(), value))
                        .collect(),
                })
                .collect(),
            SlotKind::Selection(key) => {
                let selection = self
                    .catalog
                    .selection(key)
                    .expect("a slot names a selection of the data (GameData::check)");
                // Own options and shared ones (a pool selection's: the held options' pools), in key order.
                let pools = self.catalog.pools(selection, &state);
                let mut keys: Vec<&SelectionOptionKey> = selection
                    .options
                    .iter()
                    .map(|option| &option.key)
                    .chain(&selection.shared_options)
                    .chain(&pools)
                    .collect();
                keys.sort();
                // An option with a chosen feat once per value.
                keys.into_iter()
                    .flat_map(|key| match self.catalog.option(key) {
                        Some(option) if self.catalog.option_parameter(option).is_some() => self
                            .catalog
                            .option_values(option)
                            .into_iter()
                            .map(|value| Pick::option_for(key.clone(), value))
                            .collect(),
                        _ => vec![Pick::option(key.clone())],
                    })
                    .collect()
            }
            SlotKind::MythicAbility => self
                .catalog
                .data()
                .mythic_abilities
                .iter()
                .map(|ability| Pick::MythicAbility(ability.key.clone()))
                .collect(),
            SlotKind::MythicFeat => self
                .catalog
                .data()
                .mythic_feats
                .iter()
                .flat_map(|feat| match feat.parameter {
                    None => vec![Pick::mythic_feat(feat.key.clone())],
                    Some(kind) => self
                        .catalog
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
        let later = |pick: &Pick, edit: &Edit| match pick {
            // A feat no check names breaks later just what the slot left empty would.
            Pick::Feat { feat, .. } if !named.contains(feat) => left_empty
                .get_or_init(|| recheck.later_walked(&Edit::Clear(slot.clone())))
                .clone(),
            _ => recheck.later(edit),
        };
        Some(
            candidates
                .into_iter()
                .map(|pick| {
                    let edit = Edit::Pick {
                        slot: slot.clone(),
                        pick: pick.clone(),
                    };
                    SlotOption {
                        status: check_pick_with(&self.catalog, &state, &kind, &lists, &pick)
                            .map_err(|issue| self.reason_for(issue, &state))
                            .and_then(|()| later(&pick, &edit)),
                        pick,
                        edit,
                    }
                })
                .collect(),
        )
    }

    /// A slot's options as its picker shows them: grouped by what the pick takes, in [`options`](Self::options)'
    /// order, each entry named, filed under its chip and with its summary, story note and the one reason it shows,
    /// and each option with its value's name and its own reason. `None` when the Build has no such slot.
    pub fn slot_entries(&self, build: &Build, slot: &SlotId) -> Option<Vec<SlotEntry>> {
        let mut entries: Vec<SlotEntry> = Vec::new();
        for option in self.options(build, slot)? {
            let same_item = entries
                .last()
                .is_some_and(|entry| entry.options[0].option.pick.key() == option.pick.key());
            if !same_item {
                entries.push(self.slot_entry(&option));
            }
            let entry = entries.last_mut().expect("an entry");
            // Values often fail alike (BAB +1 for every weapon): the last one's reason, when it's the same issue.
            let reason = match (&option.status, entry.options.last()) {
                (Ok(()), _) => None,
                (Err(issue), Some(last)) if last.option.status.as_ref().err() == Some(issue) => {
                    last.reason.clone()
                }
                (Err(issue), _) => self.issue_text(issue),
            };
            entry.options.push(EntryOption {
                value: option
                    .pick
                    .parameter()
                    .map_or(String::new(), |value| self.parameter_name(value).to_owned()),
                reason,
                option,
            });
        }
        for entry in &mut entries {
            entry.open = entry.options.iter().any(|o| o.option.status.is_ok());
            if !entry.open {
                entry.reason = entry_reason(&entry.options);
            }
        }
        Some(entries)
    }

    /// The feats a walk of the Build can look at: each feat picked (whether it's held already), each option's
    /// chosen feat, the feats named by the prerequisites of the picks and of the classes and archetypes the Build
    /// takes, and every feat with an extra pick. Holding any other feat touches nothing but that feat's entries in
    /// the state, so a feat pick outside these changes no later check: the walk with it fails exactly the choices
    /// the walk without it fails. A feat with an extra pick also gives a slot, which takes its level's next
    /// [`SlotId::FeatSelection`] index from a later feat's, so it's always walked.
    fn named_feats<'b>(&'b self, build: &'b Build) -> HashSet<&'b FeatKey> {
        let mut named: HashSet<&FeatKey> = self
            .catalog
            .data()
            .feats
            .iter()
            .filter(|feat| !feat.extra_pick_of.is_empty())
            .map(|feat| &feat.key)
            .collect();
        for item in build
            .picks
            .values()
            .filter_map(|pick| self.catalog.pick_item(pick))
        {
            named.extend(item.feat);
            if let Some(prerequisite) = item.prerequisite {
                prerequisite.collect_feats(&mut named);
            }
        }
        for level in &build.levels {
            if let Some((class, _)) = self.catalog.class(&level.class) {
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

    /// The entry that starts with `option`: what its pick takes, as the picker shows it.
    fn slot_entry(&self, option: &SlotOption) -> SlotEntry {
        let item = self.catalog.pick_item(&option.pick);
        SlotEntry {
            name: item
                .as_ref()
                .map_or(option.pick.key(), |item| item.name)
                .to_owned(),
            category: item.as_ref().map_or("", |item| item.category).to_owned(),
            feat_category: self.feat_category(&option.pick),
            summary: item.as_ref().map_or("", |item| item.summary).to_owned(),
            story_note: item
                .as_ref()
                .and_then(|item| item.prerequisite)
                .and_then(story_note),
            // The kind its picks name ("a value" for a key the data lacks, which no option has).
            noun: option.pick.parameter().map(|_| {
                item.as_ref()
                    .and_then(|item| item.parameter)
                    .map_or("a value", ParameterKind::with_article)
            }),
            open: false,
            reason: None,
            options: Vec::new(),
        }
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
        for class in self.catalog.data().all_classes() {
            let taken = state.classes.iter().find(|c| c.class == class.key);
            let archetypes: Vec<Option<&Archetype>> = match taken {
                Some(taken) => vec![self.catalog.archetype_of(class, taken.archetype.as_ref())],
                None => std::iter::once(None)
                    .chain(class.archetypes.iter().map(Some))
                    .collect(),
            };
            for archetype in archetypes {
                let archetype = archetype.map(|a| a.key.clone());
                // A class first taken here gets this archetype and keeps any advancement it has.
                let choice = taken.is_none().then(|| ClassChoice {
                    archetype: archetype.clone(),
                    advances: held_advances(build, &class.key),
                });
                let edit = Edit::Class {
                    level,
                    class: class.key.clone(),
                    choice,
                };
                let mut status = self.class_status(&state, class, archetype.as_ref(), taken);
                if status.is_ok()
                    && let Some(recheck) = &recheck
                {
                    status = recheck.later(&edit);
                }
                options.push(ClassOption {
                    class: class.key.clone(),
                    archetype,
                    status,
                    edit,
                });
            }
        }
        Some(options)
    }

    /// Whether a class (with an archetype) can be taken next by a character in `state`. An archetype stating an
    /// alignment requirement replaces the class's, so the reason names the archetype's.
    fn class_status(
        &self,
        state: &CharacterState,
        class: &Class,
        archetype: Option<&ArchetypeKey>,
        taken: Option<&ClassLevels>,
    ) -> Result<(), Issue> {
        let archetype = self.catalog.archetype_of(class, archetype);
        let cap = class_level_cap(class);
        if state.class_level(&class.key) >= cap {
            return Err(Issue::ClassLevelCap { cap });
        }
        if taken.is_some() {
            return Ok(());
        }
        let unmet: Vec<Unmet> = self
            .catalog
            .entry_prerequisites(class, archetype)
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

    /// The advancement choices for a prestige class option taken at a character level: which held classes its
    /// levels advance. Every set of the classes the Build takes before that level (in the order it first takes
    /// them), from one class up to as many as the prestige class may advance, then none; only those the Build
    /// stays possible with, each with the edit that takes the class with it.
    pub fn advances_options(
        &self,
        build: &Build,
        level: u8,
        option: &ClassOption,
    ) -> Vec<AdvancesOption> {
        let most = self
            .catalog
            .class(&option.class)
            .map_or(0, |(class, _)| most_advanced(class));
        let mut held: Vec<&ClassKey> = Vec::new();
        for level_up in build
            .levels
            .iter()
            .take(usize::from(level).saturating_sub(1))
        {
            if !held.contains(&&level_up.class) {
                held.push(&level_up.class);
            }
        }
        let mut candidates: Vec<BTreeSet<ClassKey>> =
            (1..=most).flat_map(|size| subsets(&held, size)).collect();
        candidates.push(BTreeSet::new());
        candidates
            .into_iter()
            .filter(|advances| {
                let mut after = build.clone();
                self.apply(
                    &mut after,
                    &self.class_edit(build, level, option, advances.clone()),
                );
                self.is_possible(&after)
            })
            .map(|advances| AdvancesOption {
                edit: self.class_edit(build, level, option, advances.clone()),
                advances,
            })
            .collect()
    }

    /// Whether taking a class at a character level asks which classes it advances: a prestige class whose rows
    /// advance spellcasting, first taken at that level ([`advances_options`](Self::advances_options) lists the
    /// answers).
    pub fn asks_advances(&self, build: &Build, level: u8, class: &ClassKey) -> bool {
        self.catalog.class(class).is_some_and(|(class, prestige)| {
            let (arcane, divine) = advanced_spellcasting(class);
            prestige && (arcane || divine)
        }) && first_taken_at(build, class, level)
    }

    /// The edit for taking a class option at a level with an advancement: a class first taken here gets the
    /// option's archetype and the advancement; one taken before keeps its choice.
    fn class_edit(
        &self,
        build: &Build,
        level: u8,
        option: &ClassOption,
        advances: BTreeSet<ClassKey>,
    ) -> Edit {
        let choice = first_taken_at(build, &option.class, level).then(|| ClassChoice {
            archetype: option.archetype.clone(),
            advances,
        });
        Edit::Class {
            level,
            class: option.class.clone(),
            choice,
        }
    }

    /// The path picker at MR 3: every path without a switch rule, in key order; one that would break a later
    /// choice (a pick in the Build's path's slots) is [`Issue::BreaksLater`].
    pub fn path_options(&self, build: &Build) -> Vec<PathOption> {
        let recheck = Recheck::new(self, build);
        self.catalog
            .data()
            .mythic_paths
            .iter()
            .filter(|path| path.switch.is_none())
            .map(|path| {
                let edit = Edit::MythicPath(Some(path.key.clone()));
                PathOption {
                    path: path.key.clone(),
                    status: recheck.later(&edit),
                    edit,
                }
            })
            .collect()
    }

    /// The first ascension's picker at MR 1: every path with a path-linked ability, in key order; one that would
    /// break a later choice is [`Issue::BreaksLater`].
    pub fn first_ascension_options(&self, build: &Build) -> Vec<PathOption> {
        let recheck = Recheck::new(self, build);
        self.catalog
            .data()
            .mythic_paths
            .iter()
            .filter(|path| self.first_ascension_option(&path.key).is_some())
            .map(|path| {
                let edit = Edit::FirstAscension(Some(path.key.clone()));
                PathOption {
                    path: path.key.clone(),
                    status: recheck.later(&edit),
                    edit,
                }
            })
            .collect()
    }

    /// The Path switch picker at MR 8: every path with a switch rule, in key order, each open or not from the
    /// path the Build chose at MR 3; one that would break a later choice (Gold Dragon for a Build past level 20)
    /// is [`Issue::BreaksLater`].
    pub fn switch_options(&self, build: &Build) -> Vec<PathOption> {
        let recheck = Recheck::new(self, build);
        self.catalog
            .data()
            .mythic_paths
            .iter()
            .filter_map(|path| Some((path, path.switch.as_ref()?)))
            .map(|(path, switch)| {
                let edit = Edit::PathSwitch(Some(path.key.clone()));
                PathOption {
                    path: path.key.clone(),
                    status: if switch.open_from(build.mythic_path.as_ref()) {
                        recheck.later(&edit)
                    } else {
                        Err(Issue::NotFromPath {
                            from: switch.from.clone(),
                        })
                    },
                    edit,
                }
            })
            .collect()
    }
}

/// The note beside an option with story conditions in its prerequisite, which never block it: "(Story: Vow of
/// Dahak)". `None` without any.
fn story_note(prerequisite: &Prerequisite) -> Option<String> {
    let stories = prerequisite.stories();
    if stories.is_empty() {
        return None;
    }
    let names: Vec<&str> = stories.iter().map(|story| story.name()).collect();
    Some(format!("(Story: {})", names.join(", ")))
}

/// The one reason a locked entry shows: the first option's whose issue isn't already holding it (Weapon Focus held
/// for a longsword but needing BAB +1 for the rest says the BAB), else the first option's. `None` when that one shows
/// none (it would break a later choice).
fn entry_reason(options: &[EntryOption]) -> Option<String> {
    let held = |option: &&EntryOption| {
        matches!(
            option.option.status,
            Err(Issue::AlreadyHeld | Issue::MaxTimes { .. })
        )
    };
    let failing = || options.iter().filter(|o| o.option.status.is_err());
    failing()
        .find(|option| !held(option))
        .or_else(|| failing().next())
        .and_then(|option| option.reason.clone())
}

/// Whether a class is first taken at this level: no level before it takes it.
fn first_taken_at(build: &Build, class: &ClassKey, level: u8) -> bool {
    !build
        .levels
        .iter()
        .take(usize::from(level).saturating_sub(1))
        .any(|level_up| &level_up.class == class)
}

/// The advancement the Build has for a class: none when it has no choice for it.
fn held_advances(build: &Build, class: &ClassKey) -> BTreeSet<ClassKey> {
    build
        .class_choices
        .get(class)
        .map(|choice| choice.advances.clone())
        .unwrap_or_default()
}

/// Every set of `size` of the items, in order: by the first item's place, then the next's.
fn subsets(items: &[&ClassKey], size: usize) -> Vec<BTreeSet<ClassKey>> {
    if size == 0 {
        return vec![BTreeSet::new()];
    }
    let mut sets = Vec::new();
    for (index, first) in items.iter().enumerate() {
        for mut rest in subsets(&items[index + 1..], size - 1) {
            rest.insert((*first).clone());
            sets.push(rest);
        }
    }
    sets
}
