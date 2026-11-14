//! The pickers: every option a slot, a level's class, the path or the Path switch can take, each with whether it
//! can be taken there and why not, and the edit that takes it; a slot's options also come grouped as its picker shows
//! them ([`Engine::slot_entries`]). The Build start's and a level's other controls are in `controls`.
//!
//! A picker lists its candidates and presents them; whether each can be taken is the walk's verdict, asked of the
//! walk paused at the picker's slot or level ([`Cursor`]), so a picker and the walk can't judge a choice two ways.

use std::cell::OnceCell;
use std::collections::BTreeSet;

use super::gear::GearLean;
use super::walk::{Cursor, first_taken_at};
use super::{
    AdvancesOption, ClassOption, Engine, EntryOption, PathOption, Sheet, Slot, SlotEntry, SlotKind,
    SlotOption, advanced_spellcasting,
};
use crate::build::{Build, ClassChoice, Edit, Pick, SlotId};
use crate::choice::Issue;
use crate::game_data::Archetype;
use crate::keys::{ClassKey, SelectionOptionKey};
use crate::parameter::{ParameterKind, Repeat};
use crate::prerequisite::Prerequisite;

impl Engine {
    /// Every option a slot accepts, in key order (a selection's own options and the ones it shares), each with
    /// whether it can be picked at that point of the Build and why not. An option that would break a later choice
    /// is [`Issue::BreaksLater`] (Toughness at level 1 with Toughness at level 3; anything but Cleave in Cleave's
    /// slot while Great Cleave needs it). `None` when the Build has no such slot.
    ///
    /// Each option's status is the walk's verdict at the slot ([`Cursor::until_slot`], `AtSlot::try_pick`); the
    /// picker lists the candidates and words the reasons.
    ///
    /// An option open only because of the Build's gear says which items it needs ([`SlotOption::with_gear`]).
    ///
    /// Costs one walk to the slot's level, then a walk from the slot's level for the slot left empty and for each
    /// option open there that a later check could see (the cursor's named feats); each stops at the first choice
    /// that fails. A Build with gear costs one more walk to the slot's level, without the gear, and when an option
    /// needs gear one more for each item held (two, when an option needs no single item).
    pub fn options(&self, build: &Build, slot: &SlotId) -> Option<Vec<SlotOption>> {
        let cursor = Cursor::new(self, build);
        let at = cursor.until_slot(slot)?;
        let (state, kind, lists) = (at.state(), at.kind(), at.lists());
        let candidates: Vec<Pick> = match kind {
            // A parameterised feat once per value.
            SlotKind::Feat(feat_slot) => self
                .catalog
                .data()
                .feats
                .iter()
                .filter(|feat| self.catalog.slot_accepts_with(feat_slot, feat, lists))
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
                    .expect("a slot names a selection of the data (CheckedGameData)");
                // Own options and shared ones (a pool selection's: the held options' pools), in key order.
                let pools = self.catalog.pools(selection, state);
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
            // A parameterised one once per value.
            SlotKind::MythicAbility => self
                .catalog
                .data()
                .mythic_abilities
                .iter()
                .flat_map(|ability| match ability.parameter {
                    None => vec![Pick::mythic_ability(ability.key.clone())],
                    Some(kind) => self
                        .catalog
                        .parameter_values(kind)
                        .into_iter()
                        .map(|value| Pick::mythic_ability_for(ability.key.clone(), value))
                        .collect(),
                })
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
        // With gear: the same slot without it, to mark what only the gear opens.
        let gear = self.gear_at_slot(build, slot);
        Some(
            candidates
                .into_iter()
                .map(|pick| {
                    let status = at
                        .try_pick(&pick)
                        .map_err(|issue| self.reason_for(issue, state));
                    let with_gear = match (&gear, &status) {
                        (Some(gear), Ok(())) => gear.need(&pick),
                        _ => None,
                    };
                    SlotOption {
                        status,
                        edit: Edit::Pick {
                            slot: slot.clone(),
                            pick: pick.clone(),
                        },
                        with_gear,
                        pick,
                    }
                })
                .collect(),
        )
    }

    /// A slot's options as its picker shows them: grouped by what the pick takes, in [`options`](Self::options)'
    /// order, each entry named, filed under its chip and with its summary, story note and the one reason it shows,
    /// and each option with its value's name and its own reason. `None` when the Build has no such slot.
    pub fn slot_entries(&self, build: &Build, slot: &SlotId) -> Option<Vec<SlotEntry>> {
        self.slot_entries_in(build, None, slot)
    }

    /// [`slot_entries`](Self::slot_entries) with the Build's sheet when the caller has it: an option held already
    /// says where it is held from the sheet, which is otherwise walked once, when the first such option asks.
    pub fn slot_entries_in(
        &self,
        build: &Build,
        sheet: Option<&Sheet>,
        slot: &SlotId,
    ) -> Option<Vec<SlotEntry>> {
        let mut entries: Vec<SlotEntry> = Vec::new();
        let walked: OnceCell<Sheet> = OnceCell::new();
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
                // Where a pick is held is its own to say: the last value's place is another's.
                (Err(Issue::AlreadyHeld), _) => {
                    let sheet =
                        sheet.unwrap_or_else(|| walked.get_or_init(|| self.evaluate(build)));
                    self.held_where(build, sheet, slot, &option.pick)
                        .or_else(|| self.issue_text(&Issue::AlreadyHeld))
                }
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
            let mut open = (entry.options.iter()).filter(|o| o.option.status.is_ok());
            entry.open = open.clone().next().is_some();
            if !entry.open {
                entry.reason = entry_reason(&entry.options);
            } else if open.clone().all(|o| o.option.with_gear.is_some()) {
                entry.gear_note = (open.next())
                    .and_then(|o| o.option.with_gear.as_ref())
                    .map(|needs| self.gear_note(needs));
            }
        }
        Some(entries)
    }

    /// For a pick a slot refuses as already held: where the Build holds it, by the pick it holds, in the place
    /// phrase a pick order's reasons use: "Finesse Training (Rapier) is taken in level 3's Finesse Training slot",
    /// or in a Mythic rank's slot "…is taken in Mythic rank 4's Improved Mythic Trick slot, at level 14". The pick
    /// named is the one held, which can differ from the one refused (a chosen-feat option refused for the feat in a
    /// feat slot; a mythic ability held with another value). `None` when no slot of the sheet holds it: a grant has
    /// no slot, and the reason stays "Already taken". `sheet` is the Build's.
    ///
    /// Costs no walk.
    pub fn held_where(
        &self,
        build: &Build,
        sheet: &Sheet,
        slot: &SlotId,
        pick: &Pick,
    ) -> Option<String> {
        let held: &Slot = (sheet.levels.iter())
            .flat_map(|level| &level.slots)
            .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
            .filter(|other| &other.id != slot)
            .find(|other| {
                other
                    .pick
                    .as_ref()
                    .is_some_and(|held| self.holds_the_same(held, pick))
            })?;
        self.taken_in(build, held)
    }

    /// Where the Build holds a pick, in the phrase [`held_where`](Self::held_where) uses: "Toughness is taken in
    /// level 3's Feat slot". `None` for a slot with no pick, or none of a level or a rank.
    pub fn taken_in(&self, build: &Build, held: &Slot) -> Option<String> {
        let name = self.pick_name(held.pick.as_ref()?);
        let label = self.slot_label(held);
        Some(match (held.id.level(), held.id.rank()) {
            (Some(level), _) => format!("{name} is taken in level {level}'s {label} slot"),
            (None, Some(rank)) => {
                let at = self.rank_level(build, rank)?;
                format!("{name} is taken in Mythic rank {rank}'s {label} slot, at level {at}")
            }
            (None, None) => return None,
        })
    }

    /// Whether a held pick is what keeps a pick from being taken again: the same pick; the same feat, option or
    /// mythic ability that counts once whatever its value; an option's chosen feat and the feat itself.
    fn holds_the_same(&self, held: &Pick, wanted: &Pick) -> bool {
        if held == wanted {
            return true;
        }
        let catalog = &self.catalog;
        match (held, wanted) {
            (Pick::Feat { feat: a, .. }, Pick::Feat { feat: b, .. }) => {
                a == b
                    && catalog
                        .feat(a)
                        .is_some_and(|feat| feat.repeat == Repeat::Once)
            }
            (Pick::SelectionOption { option: a, .. }, Pick::SelectionOption { option: b, .. }) => {
                a == b
                    && catalog
                        .option(a)
                        .is_some_and(|option| option.repeat == Repeat::Once)
            }
            (Pick::MythicAbility { ability: a, .. }, Pick::MythicAbility { ability: b, .. }) => {
                a == b
            }
            (Pick::MythicFeat { feat: a, .. }, Pick::MythicFeat { feat: b, .. }) => {
                a == b
                    && catalog
                        .mythic_feat(a)
                        .is_some_and(|feat| feat.repeat != Repeat::PerValue)
            }
            (
                Pick::SelectionOption {
                    option,
                    parameter: value,
                },
                Pick::Feat {
                    feat,
                    parameter: wanted,
                },
            )
            | (
                Pick::Feat {
                    feat,
                    parameter: wanted,
                },
                Pick::SelectionOption {
                    option,
                    parameter: value,
                },
            ) => {
                value == wanted
                    && catalog
                        .option(option)
                        .and_then(|option| option.chosen_feat.as_ref())
                        == Some(feat)
            }
            _ => false,
        }
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
            feat_category: item.as_ref().and_then(|item| item.feat_category),
            summary: item.as_ref().map_or("", |item| item.summary).to_owned(),
            story_note: item
                .as_ref()
                .and_then(|item| item.prerequisite)
                .and_then(story_note),
            gear_note: None,
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
    /// a level after the last breaks nothing. A prestige class that asks which classes it advances is open when some
    /// advancement keeps the Build ([`advances_options`](Self::advances_options) lists those). Each status is the
    /// walk's verdict at the level ([`Cursor::until_level`], `AtLevel::try_class`). `None` for a level that isn't in
    /// the Build or next. An open class that only the Build's gear lets in, or that would leave another choice
    /// leaning on the gear, says so ([`ClassOption::leans_on_gear`]).
    pub fn class_options(&self, build: &Build, level: u8) -> Option<Vec<ClassOption>> {
        if level == 0 || usize::from(level) > build.levels.len() + 1 {
            return None;
        }
        let cursor = Cursor::new(self, build);
        let at = cursor.until_level(level)?;
        let naked = self.naked_build(build);
        let lean = naked
            .as_ref()
            .map(|naked| GearLean::new(self, build, naked));
        let mut options = Vec::new();
        let data = self.catalog.data();
        let base = data.classes.iter().map(|class| (class, false));
        for (class, prestige) in base.chain(data.prestige_classes.iter().map(|class| (class, true)))
        {
            let taken = at.state().classes.iter().find(|c| c.class == class.key);
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
                let status = at.try_class(class, prestige, choice.as_ref());
                let edit = Edit::Class {
                    level,
                    class: class.key.clone(),
                    choice,
                };
                options.push(ClassOption {
                    class: class.key.clone(),
                    archetype,
                    leans_on_gear: GearLean::lean(lean.as_ref(), &status, &edit),
                    status,
                    edit,
                });
            }
        }
        Some(options)
    }

    /// The advancement choices for a prestige class option taken at a character level: which held classes its
    /// levels advance. Every set of the classes the Build takes before that level (in the order it first takes
    /// them), from one class up to as many as the prestige class may advance, then none; only those the Build
    /// stays possible with, each with the edit that takes the class with it. The class picker's status for the
    /// option reads the same list ([`class_options`](Self::class_options): open when it isn't empty).
    pub fn advances_options(
        &self,
        build: &Build,
        level: u8,
        option: &ClassOption,
    ) -> Vec<AdvancesOption> {
        let Some((class, _)) = self.catalog.class(&option.class) else {
            return Vec::new();
        };
        Cursor::new(self, build)
            .kept_advancements(level, class, option.archetype.as_ref(), None)
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

    /// The path picker at MR 3: every path without a switch rule, in key order; one that would break a later
    /// choice (a pick in the Build's path's slots) is [`Issue::BreaksLater`].
    pub fn path_options(&self, build: &Build) -> Vec<PathOption> {
        let cursor = Cursor::new(self, build);
        self.catalog
            .data()
            .mythic_paths
            .iter()
            .filter(|path| path.switch.is_none())
            .map(|path| {
                let edit = Edit::MythicPath(Some(path.key.clone()));
                PathOption {
                    path: path.key.clone(),
                    status: cursor.try_edit(&edit),
                    edit,
                }
            })
            .collect()
    }

    /// The first ascension's picker at MR 1: every path with a path-linked ability, in key order; one that would
    /// break a later choice is [`Issue::BreaksLater`].
    pub fn first_ascension_options(&self, build: &Build) -> Vec<PathOption> {
        let cursor = Cursor::new(self, build);
        self.catalog
            .data()
            .mythic_paths
            .iter()
            .filter(|path| self.first_ascension_option(&path.key).is_some())
            .map(|path| {
                let edit = Edit::FirstAscension(Some(path.key.clone()));
                PathOption {
                    path: path.key.clone(),
                    status: cursor.try_edit(&edit),
                    edit,
                }
            })
            .collect()
    }

    /// The Path switch picker at MR 8: every path with a switch rule, in key order, each open or not from the
    /// path the Build chose at MR 3; one that would break a later choice (Gold Dragon for a Build past level 20)
    /// is [`Issue::BreaksLater`].
    pub fn switch_options(&self, build: &Build) -> Vec<PathOption> {
        let cursor = Cursor::new(self, build);
        self.catalog
            .data()
            .mythic_paths
            .iter()
            .filter(|path| path.switch.is_some())
            .map(|path| {
                let edit = Edit::PathSwitch(Some(path.key.clone()));
                PathOption {
                    path: path.key.clone(),
                    status: cursor.try_edit(&edit),
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

/// The one reason a locked entry shows: that of the option [`shown_option`] names. `None` when it shows none (it
/// would break a later choice).
fn entry_reason(options: &[EntryOption]) -> Option<String> {
    shown_option(options).and_then(|option| option.reason.clone())
}

/// The option whose refusal a locked entry shows: the first whose issue isn't already holding it (Weapon Focus held
/// for a longsword but needing BAB +1 for the rest says the BAB), else the first locked one. `None` when none is
/// locked.
pub(super) fn shown_option(options: &[EntryOption]) -> Option<&EntryOption> {
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
}

/// The advancement the Build has for a class: none when it has no choice for it.
fn held_advances(build: &Build, class: &ClassKey) -> BTreeSet<ClassKey> {
    build
        .class_choices
        .get(class)
        .map(|choice| choice.advances.clone())
        .unwrap_or_default()
}
