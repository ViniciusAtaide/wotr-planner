//! The rules engine: walks a Build level by level against the game data.
//!
//! One walk produces everything: the state at each level, the slots and their picks, the Mythic track, and every
//! problem. Picks are checked in Build order against the state just before their slot, and a pick that fails
//! doesn't count as held, so a broken pick also breaks whatever depended on it (removing Cleave breaks Great
//! Cleave, which breaks anything needing Great Cleave).

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::build::{AbilityScores, Build, Edit, FeatSource, LevelUp, Pick, SlotId};
use crate::game_data::{
    Ability, AbilityBonus, AbilityModifier, Archetype, Class, Feat, FeatList, FeatSlot,
    FeatureSelection, GameData, MythicAbility, MythicFeat, MythicPath, PathSwitch, Race, Save,
    SelectionOption, SpellLevels,
};
use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, FeatListKey, FeatureKey, FeatureSelectionKey,
    MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey,
    SubraceKey,
};
use crate::problem::{Choice, Issue, Problem, Unmet};
use crate::state::{CharacterState, ClassLevels};

/// The highest character level without a path that raises it.
pub const BASE_LEVEL_CAP: u8 = 20;
/// The Mythic rank at which the path is chosen.
pub const PATH_CHOICE_RANK: u8 = 3;
/// The Mythic rank at which a Path switch can happen.
pub const PATH_SWITCH_RANK: u8 = 8;
/// The point-buy budget at Build start.
pub const POINT_BUY_BUDGET: u8 = 25;

/// The game data plus lookups by key. Build it once when the game data arrives, then ask it about Builds.
#[derive(Clone, Debug)]
pub struct Engine {
    data: GameData,
    classes: HashMap<ClassKey, ClassAt>,
    archetypes: HashMap<ArchetypeKey, (usize, usize)>,
    races: HashMap<RaceKey, usize>,
    subraces: HashMap<SubraceKey, (usize, usize)>,
    skills: HashMap<SkillKey, usize>,
    feats: HashMap<FeatKey, usize>,
    feat_lists: HashMap<FeatListKey, FeatList>,
    selections: HashMap<FeatureSelectionKey, SelectionAt>,
    options: HashMap<SelectionOptionKey, (FeatureSelectionKey, usize)>,
    mythic_abilities: HashMap<MythicAbilityKey, usize>,
    mythic_feats: HashMap<MythicFeatKey, usize>,
    paths: HashMap<MythicPathKey, usize>,
    /// The ability bonuses of every feature that has some.
    feature_bonuses: HashMap<FeatureKey, Vec<AbilityBonus>>,
}

/// Where a class is: an index into the base or the prestige classes.
#[derive(Clone, Copy, Debug)]
enum ClassAt {
    Base(usize),
    Prestige(usize),
}

/// Where a Feature selection is defined.
#[derive(Clone, Copy, Debug)]
enum SelectionAt {
    Class {
        prestige: bool,
        class: usize,
        index: usize,
    },
    Archetype {
        class: usize,
        archetype: usize,
        index: usize,
    },
    Path {
        path: usize,
        index: usize,
    },
}

/// What a Mythic rank's general pick is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MythicPickKind {
    Ability,
    Feat,
}

/// Odd ranks give a mythic ability and even ranks a mythic feat (MR 1 an ability, MR 2 a feat, then alternating).
pub fn mythic_pick_kind(rank: u8) -> MythicPickKind {
    if rank % 2 == 1 {
        MythicPickKind::Ability
    } else {
        MythicPickKind::Feat
    }
}

/// What one Class level gives a character of a class, with or without an archetype: the base class's row, or
/// the archetype's row at that level when it has one. BAB and saves come from the class's progressions instead
/// ([`Class::base_attack_bonus`], [`Class::save_bonus`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveRow<'a> {
    pub level: u8,
    pub spell_levels: SpellLevels,
    /// Prestige classes only: this level advances the chosen arcane class's spellcasting.
    pub advances_arcane: bool,
    /// Prestige classes only: this level advances the chosen divine class's spellcasting.
    pub advances_divine: bool,
    pub features: &'a [FeatureKey],
    pub feats: &'a BTreeSet<FeatKey>,
    pub feat_slots: &'a [FeatSlot],
    pub selections: &'a [FeatureSelectionKey],
}

/// The row in force at one level of `class`, taken with `archetype` (which must be one of the class's): the
/// archetype's row at that level replaces the base row's grants in full, and its spell levels when it sets them;
/// a level the archetype leaves out is the base row. `None` past the class's table. The one place rows merge.
pub fn effective_row<'a>(
    class: &'a Class,
    archetype: Option<&'a Archetype>,
    level: u8,
) -> Option<EffectiveRow<'a>> {
    let base = class.levels.iter().find(|row| row.level == level)?;
    let base_row = EffectiveRow {
        level,
        spell_levels: base.spell_levels(),
        advances_arcane: base.advances_arcane,
        advances_divine: base.advances_divine,
        features: &base.features,
        feats: &base.feats,
        feat_slots: &base.feat_slots,
        selections: &base.selections,
    };
    let Some(own) =
        archetype.and_then(|archetype| archetype.levels.iter().find(|row| row.level == level))
    else {
        return Some(base_row);
    };
    Some(EffectiveRow {
        spell_levels: own.spell_levels.unwrap_or(base_row.spell_levels),
        features: &own.features,
        feats: &own.feats,
        feat_slots: &own.feat_slots,
        selections: &own.selections,
        ..base_row
    })
}

/// Every row of a class's table, in level order, as [`effective_row`] gives it.
fn effective_rows<'a>(
    class: &'a Class,
    archetype: Option<&'a Archetype>,
) -> impl Iterator<Item = EffectiveRow<'a>> {
    class
        .levels
        .iter()
        .filter_map(move |row| effective_row(class, archetype, row.level))
}

/// What a Build gives at each of its levels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sheet {
    /// `levels[0]` is character level 1.
    pub levels: Vec<LevelSheet>,
    /// MR 1–10 in rank order, reached or not.
    pub ranks: Vec<RankSheet>,
    /// The character level cap at the end of the Build: 20, or 40 after a Legend switch.
    pub level_cap: u8,
    /// Everything that doesn't hold up, in Build order. Within a level, picks for slots the level doesn't have
    /// come after its slots.
    pub problems: Vec<Problem>,
}

/// One row of the sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelSheet {
    pub character_level: u8,
    pub class: ClassKey,
    /// The class level this Level-up reaches in its class.
    pub class_level: u8,
    /// Whether this level has the +1 ability increase.
    pub ability_increase: bool,
    pub skill_points: SkillPoints,
    /// Feat and Feature selection slots, in the order picks are checked: race, class, general, selections.
    pub slots: Vec<Slot>,
    /// The Mythic ranks reached at this level (two at 17).
    pub mythic_ranks: Vec<u8>,
    /// The state at the end of this level, after its Level-up and any Mythic ascensions.
    pub state: CharacterState,
}

/// One Mythic rank on the Mythic track.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankSheet {
    pub rank: u8,
    /// The fixed character level of this rank's Mythic ascension.
    pub character_level: u8,
    pub status: RankStatus,
    /// The path whose row this rank gives: the path in force from MR 3. MR 1 and 2 show the path chosen at MR 3
    /// once it's chosen, since choosing it grants their rows (held from MR 3 on); before that, none.
    pub path: Option<MythicPathKey>,
    /// The general mythic pick, then the path's picks. Empty when not reached.
    pub slots: Vec<Slot>,
    /// The slots whose picks a Path switch took away (all of a dropped rank's, the old path's on a partly dropped
    /// one), in slot order.
    pub dropped_slots: Vec<SlotId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RankStatus {
    Reached,
    /// Reached, then partly lost to a Path switch: its general pick or the old path's grants at it, or a grant the
    /// switch drops by name (Devil from Aeon: the gaze picks).
    PartlyDropped,
    /// Reached, then lost to a Path switch that keeps neither its general pick nor the old path's grants.
    Dropped,
    /// Not reached: the Build is too short, or a Legend switch ended the ascensions.
    NotReached,
}

/// One slot and what fills it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: SlotId,
    pub kind: SlotKind,
    pub pick: Option<Pick>,
}

/// What a slot accepts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotKind {
    Feat(FeatSlot),
    Selection(FeatureSelectionKey),
    MythicAbility,
    MythicFeat,
}

/// Skill points at one level: class skill points plus the Intelligence modifier (at least 1), and how many the
/// Level-up spends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkillPoints {
    pub available: u8,
    pub spent: u8,
}

/// One option for a slot. What picking it would break is asked separately, for the option the player taps, with
/// [`Engine::would_break`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotOption {
    pub pick: Pick,
    /// `Ok` when it can be picked here; otherwise why not (prerequisites with reasons, or already held).
    pub status: Result<(), Issue>,
}

/// One option of the class picker at a level: a class, or a base class with one of its archetypes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassOption {
    pub class: ClassKey,
    pub archetype: Option<ArchetypeKey>,
    /// `Ok` when it can be taken at that level; otherwise why not: the class's (and archetype's) entry
    /// prerequisites against the character before the level, or the class table's end.
    pub status: Result<(), Issue>,
}

/// One option of the path picker (MR 3) or the Path switch picker (MR 8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathOption {
    pub path: MythicPathKey,
    /// `Ok` when it can be taken; a switch open only from other paths than the Build's is
    /// [`Issue::NotFromPath`].
    pub status: Result<(), Issue>,
}

impl Sheet {
    /// The state at the end of a character level.
    pub fn at(&self, character_level: u8) -> Option<&CharacterState> {
        let index = usize::from(character_level).checked_sub(1)?;
        self.levels.get(index).map(|level| &level.state)
    }

    /// The slot with this id, wherever it is.
    pub fn slot(&self, id: &SlotId) -> Option<&Slot> {
        self.levels
            .iter()
            .flat_map(|level| &level.slots)
            .chain(self.ranks.iter().flat_map(|rank| &rank.slots))
            .find(|slot| &slot.id == id)
    }
}

impl Engine {
    /// Indexes the game data, after putting it in canonical order.
    pub fn new(data: GameData) -> Self {
        let data = data.canonical();
        fn index<K: std::hash::Hash + Eq + Clone, T>(
            items: &[T],
            key: impl Fn(&T) -> &K,
        ) -> HashMap<K, usize> {
            items
                .iter()
                .enumerate()
                .map(|(i, item)| (key(item).clone(), i))
                .collect()
        }

        let mut classes = HashMap::new();
        let mut archetypes = HashMap::new();
        let mut selections = HashMap::new();
        let mut add_selections = |list: &[FeatureSelection], at: &dyn Fn(usize) -> SelectionAt| {
            for (index, selection) in list.iter().enumerate() {
                selections.insert(selection.key.clone(), at(index));
            }
        };
        for (c, class) in data.classes.iter().enumerate() {
            classes.insert(class.key.clone(), ClassAt::Base(c));
            add_selections(&class.selections, &|index| SelectionAt::Class {
                prestige: false,
                class: c,
                index,
            });
            for (a, archetype) in class.archetypes.iter().enumerate() {
                archetypes.insert(archetype.key.clone(), (c, a));
                add_selections(&archetype.selections, &|index| SelectionAt::Archetype {
                    class: c,
                    archetype: a,
                    index,
                });
            }
        }
        for (c, class) in data.prestige_classes.iter().enumerate() {
            classes.insert(class.key.clone(), ClassAt::Prestige(c));
            add_selections(&class.selections, &|index| SelectionAt::Class {
                prestige: true,
                class: c,
                index,
            });
        }
        for (p, path) in data.mythic_paths.iter().enumerate() {
            add_selections(&path.selections, &|index| SelectionAt::Path {
                path: p,
                index,
            });
        }
        let mut races = HashMap::new();
        let mut subraces = HashMap::new();
        for (r, race) in data.races.iter().enumerate() {
            races.insert(race.key.clone(), r);
            for (s, subrace) in race.subraces.iter().enumerate() {
                subraces.insert(subrace.key.clone(), (r, s));
            }
        }
        let feat_lists = data
            .feat_lists()
            .map(|list| (list.key.clone(), list.clone()))
            .collect();
        let feature_bonuses = data
            .classes
            .iter()
            .chain(&data.prestige_classes)
            .flat_map(|class| {
                class.features.iter().chain(
                    class
                        .archetypes
                        .iter()
                        .flat_map(|archetype| &archetype.features),
                )
            })
            .chain(data.mythic_paths.iter().flat_map(|path| &path.features))
            .filter(|feature| !feature.ability_bonuses.is_empty())
            .map(|feature| (feature.key.clone(), feature.ability_bonuses.clone()))
            .collect();
        let mut engine = Self {
            skills: index(&data.skills, |s| &s.key),
            feats: index(&data.feats, |f| &f.key),
            feat_lists,
            mythic_abilities: index(&data.mythic_abilities, |a| &a.key),
            mythic_feats: index(&data.mythic_feats, |f| &f.key),
            paths: index(&data.mythic_paths, |p| &p.key),
            feature_bonuses,
            options: HashMap::new(),
            data,
            classes,
            archetypes,
            races,
            subraces,
            selections,
        };
        let options: HashMap<_, _> = engine
            .selections
            .keys()
            .filter_map(|key| engine.selection(key))
            .flat_map(|selection| {
                selection
                    .options
                    .iter()
                    .enumerate()
                    .map(|(i, option)| (option.key.clone(), (selection.key.clone(), i)))
            })
            .collect();
        engine.options = options;
        engine
    }

    pub fn data(&self) -> &GameData {
        &self.data
    }

    fn class(&self, key: &ClassKey) -> Option<(&Class, bool)> {
        Some(match *self.classes.get(key)? {
            ClassAt::Base(index) => (&self.data.classes[index], false),
            ClassAt::Prestige(index) => (&self.data.prestige_classes[index], true),
        })
    }

    fn race(&self, key: &RaceKey) -> Option<&Race> {
        self.races.get(key).map(|&index| &self.data.races[index])
    }

    fn feat(&self, key: &FeatKey) -> Option<&Feat> {
        self.feats.get(key).map(|&index| &self.data.feats[index])
    }

    /// A feat list by key, wherever it's defined.
    pub fn feat_list(&self, key: &FeatListKey) -> Option<&FeatList> {
        self.feat_lists.get(key)
    }

    fn selection(&self, key: &FeatureSelectionKey) -> Option<&FeatureSelection> {
        Some(match *self.selections.get(key)? {
            SelectionAt::Class {
                prestige,
                class,
                index,
            } => {
                let list = if prestige {
                    &self.data.prestige_classes
                } else {
                    &self.data.classes
                };
                &list[class].selections[index]
            }
            SelectionAt::Archetype {
                class,
                archetype,
                index,
            } => &self.data.classes[class].archetypes[archetype].selections[index],
            SelectionAt::Path { path, index } => &self.data.mythic_paths[path].selections[index],
        })
    }

    fn option(&self, key: &SelectionOptionKey) -> Option<&SelectionOption> {
        let (selection, index) = self.options.get(key)?;
        self.selection(selection)
            .and_then(|selection| selection.options.get(*index))
    }

    fn mythic_ability(&self, key: &MythicAbilityKey) -> Option<&MythicAbility> {
        self.mythic_abilities
            .get(key)
            .map(|&index| &self.data.mythic_abilities[index])
    }

    fn mythic_feat(&self, key: &MythicFeatKey) -> Option<&MythicFeat> {
        self.mythic_feats
            .get(key)
            .map(|&index| &self.data.mythic_feats[index])
    }

    fn path(&self, key: &MythicPathKey) -> Option<&MythicPath> {
        self.paths
            .get(key)
            .map(|&index| &self.data.mythic_paths[index])
    }

    /// The archetype when it exists and belongs to `class`.
    fn archetype_of(
        &self,
        class: &Class,
        archetype: Option<&ArchetypeKey>,
    ) -> Option<&crate::game_data::Archetype> {
        let &(c, a) = self.archetypes.get(archetype?)?;
        let owner = &self.data.classes[c];
        (owner.key == class.key).then(|| &owner.archetypes[a])
    }

    /// The row in force at one level of a class taken with an archetype key (ignored unless it's the class's).
    fn row<'a>(
        &'a self,
        class: &'a Class,
        archetype: Option<&ArchetypeKey>,
        level: u8,
    ) -> Option<EffectiveRow<'a>> {
        effective_row(class, self.archetype_of(class, archetype), level)
    }

    /// Whether a class taken with an archetype key casts spells of each type at any level: (arcane, divine).
    fn casts(&self, class: &Class, archetype: Option<&ArchetypeKey>) -> (bool, bool) {
        effective_rows(class, self.archetype_of(class, archetype)).fold(
            (false, false),
            |(arcane, divine), row| {
                (
                    arcane || row.spell_levels.arcane > 0,
                    divine || row.spell_levels.divine > 0,
                )
            },
        )
    }

    /// Walks the whole Build.
    pub fn evaluate(&self, build: &Build) -> Sheet {
        Walker::new(self, build, None).run().sheet
    }

    /// Walks the Build up to a probe, stopping after the level it's in.
    fn probe(&self, build: &Build, probe: Probe<'_>) -> Probed {
        Walker::new(self, build, Some(probe)).run().probed
    }

    /// Every problem in the Build: picks that no longer qualify, and keys missing from the game data. Used when a
    /// link is opened, for the stale-link banner.
    pub fn validate(&self, build: &Build) -> Vec<Problem> {
        self.evaluate(build).problems
    }

    /// The state just before a slot is filled, or `None` when the Build has no such slot.
    pub fn state_before(&self, build: &Build, slot: &SlotId) -> Option<CharacterState> {
        self.probe(build, Probe::Slot(slot)).state
    }

    /// Every option a slot accepts, in key order (a selection's own options and the ones it shares), each with
    /// whether it can be picked at that point of the Build and why not. `None` when the Build has no such slot.
    ///
    /// One walk, up to the slot's level. Would-break isn't computed here: that costs a walk of the whole Build
    /// per option, so the picker asks [`would_break`](Self::would_break) only for the option the player taps.
    pub fn options(&self, build: &Build, slot: &SlotId) -> Option<Vec<SlotOption>> {
        let probed = self.probe(build, Probe::Slot(slot));
        let (state, kind) = (probed.state?, probed.kind?);
        let candidates: Vec<Pick> = match &kind {
            SlotKind::Feat(feat_slot) => self
                .data
                .feats
                .iter()
                .filter(|feat| self.slot_accepts(feat_slot, feat))
                .map(|feat| Pick::Feat(feat.key.clone()))
                .collect(),
            SlotKind::Selection(key) => self
                .selection(key)
                .map(|selection| {
                    // Own options and shared ones, in key order. A shared key the data lacks offers nothing.
                    let mut keys: Vec<&SelectionOptionKey> = selection
                        .options
                        .iter()
                        .map(|option| &option.key)
                        .chain(
                            selection
                                .shared_options
                                .iter()
                                .filter(|key| self.options.contains_key(*key)),
                        )
                        .collect();
                    keys.sort();
                    keys.into_iter()
                        .map(|key| Pick::SelectionOption(key.clone()))
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
                .map(|feat| Pick::MythicFeat(feat.key.clone()))
                .collect(),
        };
        Some(
            candidates
                .into_iter()
                .map(|pick| SlotOption {
                    status: check_pick(self, &state, &kind, &pick),
                    pick,
                })
                .collect(),
        )
    }

    /// The class picker at a character level (one past the end adds a level): every base class, each followed by
    /// its archetypes, then every prestige class, each in key order. A class the Build already has before that
    /// level is offered only with its archetype, which was fixed when it was first taken. `None` for a level
    /// that isn't in the Build or next.
    pub fn class_options(&self, build: &Build, level: u8) -> Option<Vec<ClassOption>> {
        if level == 0 || usize::from(level) > build.levels.len() + 1 {
            return None;
        }
        let state = self.probe(build, Probe::Level(level)).state?;
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
                options.push(ClassOption {
                    class: class.key.clone(),
                    archetype: archetype.map(|a| a.key.clone()),
                    status: self.class_status(&state, class, archetype, taken.is_some()),
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
        archetype: Option<&Archetype>,
        taken: bool,
    ) -> Result<(), Issue> {
        let cap = class_level_cap(class);
        if state.class_level(&class.key) >= cap {
            return Err(Issue::ClassLevelCap { cap });
        }
        if taken {
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

    /// The path picker at MR 3: every path without a switch rule, in key order.
    pub fn path_options(&self, _build: &Build) -> Vec<PathOption> {
        self.data
            .mythic_paths
            .iter()
            .filter(|path| path.switch.is_none())
            .map(|path| PathOption {
                path: path.key.clone(),
                status: Ok(()),
            })
            .collect()
    }

    /// The Path switch picker at MR 8: every path with a switch rule, in key order, each open or not from the
    /// path the Build chose at MR 3.
    pub fn switch_options(&self, build: &Build) -> Vec<PathOption> {
        self.data
            .mythic_paths
            .iter()
            .filter_map(|path| Some((path, path.switch.as_ref()?)))
            .map(|(path, switch)| PathOption {
                path: path.key.clone(),
                status: if switch.open_from(build.mythic_path.as_ref()) {
                    Ok(())
                } else {
                    Err(Issue::NotFromPath {
                        from: switch.from.clone(),
                    })
                },
            })
            .collect()
    }

    /// The later choices an edit would break: problems the edited Build has that the Build doesn't, other than
    /// at the edited choices themselves. Chains are included, because a broken pick doesn't count as held.
    pub fn would_break(&self, build: &Build, edit: &Edit) -> Vec<Problem> {
        let before = problem_sites(&self.evaluate(build).problems);
        let mut edited = build.clone();
        edited.apply(edit);
        let own = edit.choices();
        self.evaluate(&edited)
            .problems
            .into_iter()
            .filter(|problem| !before.contains(&problem.at) && !own.contains(&problem.at))
            .collect()
    }

    /// Whether a feat slot accepts a feat, ignoring prerequisites. A list the data lacks accepts none.
    pub fn slot_accepts(&self, slot: &FeatSlot, feat: &Feat) -> bool {
        match slot {
            FeatSlot::General => true,
            FeatSlot::CombatFeat => feat.category.is_combat(),
            FeatSlot::Category(category) => feat.category == *category,
            FeatSlot::OneOf { list } => self
                .feat_list(list)
                .is_some_and(|list| list.contains(&feat.key)),
        }
    }

    /// Whether a pick in this slot ignores the feat's prerequisites: only a bonus-list slot's does, one whose list
    /// ignores them.
    pub fn slot_ignores_prerequisites(&self, slot: &FeatSlot) -> bool {
        match slot {
            FeatSlot::OneOf { list } => self
                .feat_list(list)
                .is_some_and(|list| list.ignores_prerequisites),
            _ => false,
        }
    }
}

/// Something held that a Path switch can take away again.
#[derive(Clone, Debug)]
enum Held {
    Feat(FeatKey),
    Feature(FeatureKey),
    SelectionOption(SelectionOptionKey),
    MythicAbility(MythicAbilityKey),
    MythicFeat(MythicFeatKey),
}

/// Where a Mythic ascension's grant came from, for what a Path switch keeps.
#[derive(Clone, Debug)]
struct GrantSource {
    /// The rank whose row or pick granted it (MR 1 and 2 for the rows granted on choosing the path).
    rank: u8,
    /// The path's own grant (its row's features, the picks in its slots) rather than the general pick.
    from_path: bool,
    /// The Feature selection whose slot was picked.
    selection: Option<FeatureSelectionKey>,
    /// The slot picked, if any.
    slot: Option<SlotId>,
}

impl GrantSource {
    /// A path rank row's own grant (its features).
    fn row(rank: u8) -> Self {
        Self {
            rank,
            from_path: true,
            selection: None,
            slot: None,
        }
    }
}

/// Whether a Path switch takes a grant away: a general pick past the general cut-off, the old path's grant past
/// the path cut-off, or one the switch drops by name whatever its rank.
fn switch_drops(switch: &PathSwitch, grant: &RankGrant) -> bool {
    let source = &grant.source;
    if !source.from_path {
        return source.rank > switch.kept_through_rank;
    }
    source.rank > switch.path_kept_through_rank
        || matches!(&grant.item, Held::Feature(key) if switch.dropped_features.contains(key))
        || source
            .selection
            .as_ref()
            .is_some_and(|selection| switch.dropped_selections.contains(selection))
}

/// One thing a Mythic ascension granted.
#[derive(Clone, Debug)]
struct RankGrant {
    source: GrantSource,
    item: Held,
}

/// Where a walk stops to report the state: just before a slot is filled, or just before a level's Level-up.
#[derive(Clone, Copy)]
enum Probe<'a> {
    Slot(&'a SlotId),
    Level(u8),
}

/// What a probe found: nothing when the Build has no such slot or level.
#[derive(Default)]
struct Probed {
    state: Option<CharacterState>,
    /// For a slot probe, what the slot accepts.
    kind: Option<SlotKind>,
}

struct Walker<'a> {
    engine: &'a Engine,
    build: &'a Build,
    state: CharacterState,
    problems: Vec<Problem>,
    /// What Mythic ascensions granted, by rank and source, so a Path switch can drop it.
    rank_grants: Vec<RankGrant>,
    /// How many sources hold each feat (a class grant and a path pick may both give Shield Proficiency), so
    /// dropping one source keeps a feat another still gives.
    feat_holds: HashMap<FeatKey, u8>,
    ranks: Vec<RankSheet>,
    level_cap: u8,
    /// Set by a switch to a path without further ascensions.
    ascensions_over: bool,
    probe: Option<Probe<'a>>,
    probed: Probed,
}

struct Walk {
    sheet: Sheet,
    probed: Probed,
}

impl<'a> Walker<'a> {
    fn new(engine: &'a Engine, build: &'a Build, probe: Option<Probe<'a>>) -> Self {
        let mut ascensions = engine.data.progression.mythic_ascensions.clone();
        ascensions.sort_by_key(|a| a.rank);
        let ranks = ascensions
            .iter()
            .map(|a| RankSheet {
                rank: a.rank,
                character_level: a.character_level,
                status: RankStatus::NotReached,
                path: None,
                slots: Vec::new(),
                dropped_slots: Vec::new(),
            })
            .collect();
        Self {
            engine,
            build,
            state: CharacterState::default(),
            problems: Vec::new(),
            rank_grants: Vec::new(),
            feat_holds: HashMap::new(),
            ranks,
            level_cap: BASE_LEVEL_CAP,
            ascensions_over: false,
            probe,
            probed: Probed::default(),
        }
    }

    fn problem(&mut self, at: Choice, issue: Issue) {
        self.problems.push(Problem { at, issue });
    }

    fn run(mut self) -> Walk {
        self.start();
        let mut levels = Vec::with_capacity(self.build.levels.len());
        for (index, level_up) in self.build.levels.iter().enumerate() {
            let level = u8::try_from(index + 1).unwrap_or(u8::MAX);
            levels.push(self.level(level, level_up));
            if self.probed.state.is_some() {
                break;
            }
        }
        if let Some(Probe::Level(level)) = self.probe
            && usize::from(level) == self.build.levels.len() + 1
        {
            self.probed.state = Some(self.state.clone());
        }
        Walk {
            sheet: Sheet {
                levels,
                ranks: self.ranks,
                level_cap: self.level_cap,
                problems: self.problems,
            },
            probed: self.probed,
        }
    }

    // ---- Build start --------------------------------------------------------------------------------------

    fn start(&mut self) {
        let engine = self.engine;
        let start = &self.build.start;
        self.state.race = start.race.clone();
        self.state.alignment = start.alignment;

        let race = match &start.race {
            Some(key) => {
                let race = engine.race(key);
                if race.is_none() {
                    self.problem(Choice::Race, Issue::UnknownKey(key.to_string()));
                }
                race
            }
            None => None,
        };
        let subrace = match &start.subrace {
            Some(key) => match engine.subraces.get(key) {
                None => {
                    self.problem(Choice::Subrace, Issue::UnknownKey(key.to_string()));
                    None
                }
                Some(&(r, s)) if race.is_some_and(|race| race.key == engine.data.races[r].key) => {
                    Some(&engine.data.races[r].subraces[s])
                }
                Some(_) => {
                    self.problem(Choice::Subrace, Issue::NotAllowed);
                    None
                }
            },
            None => None,
        };

        let mut cost = 0i16;
        for ability in AbilityScores::ABILITIES {
            let score = start.ability_scores.get(ability);
            match point_buy_cost(score) {
                Some(points) => cost += i16::from(points),
                None => self.problem(
                    Choice::AbilityScores,
                    Issue::ScoreOutOfRange { ability, score },
                ),
            }
        }
        if cost > i16::from(POINT_BUY_BUDGET) {
            self.problem(
                Choice::AbilityScores,
                Issue::PointBuy {
                    cost: u8::try_from(cost).unwrap_or(u8::MAX),
                    budget: POINT_BUY_BUDGET,
                },
            );
        }

        let mut scores = start.ability_scores;
        let modifiers = match (subrace, race) {
            (Some(subrace), _) => subrace.ability_modifiers.as_slice(),
            (None, Some(race)) => race.ability_modifiers.as_slice(),
            (None, None) => &[],
        };
        for modifier in modifiers {
            let (ability, amount) = match *modifier {
                AbilityModifier::Fixed { ability, amount } => (Some(ability), amount),
                AbilityModifier::Chosen { amount } => (start.racial_ability, amount),
            };
            if let Some(ability) = ability {
                let score = scores.get_mut(ability);
                *score = score.saturating_add_signed(amount);
            }
        }
        self.state.base_ability_scores = scores;
        self.refresh_ability_scores();
    }

    // ---- Level-up -----------------------------------------------------------------------------------------

    fn level(&mut self, level: u8, level_up: &'a LevelUp) -> LevelSheet {
        if let Some(Probe::Level(probed)) = self.probe
            && probed == level
        {
            self.probed.state = Some(self.state.clone());
        }
        if level > self.level_cap {
            self.problem(
                Choice::CharacterLevel { level },
                Issue::LevelCap {
                    cap: self.level_cap,
                },
            );
        }
        // Entry prerequisites are checked against the character before this level.
        self.check_class_entry(level, level_up);
        self.state.character_level = level;
        let class_level = self.take_class_level(level, level_up);

        let ability_increase = level % 4 == 0;
        match (ability_increase, level_up.ability_increase) {
            (true, Some(ability)) => {
                let score = self.state.base_ability_scores.get_mut(ability);
                *score = score.saturating_add(1);
                self.refresh_ability_scores();
            }
            (false, Some(_)) => {
                self.problem(Choice::AbilityIncrease { level }, Issue::NoAbilityIncrease)
            }
            (_, None) => {}
        }
        self.recompute_tables();
        let skill_points = self.skill_ranks(level, level_up);

        let slots = self.fill_all(self.level_slots(level, level_up), None, |id| {
            id.level() == Some(level)
        });

        let mut mythic_ranks = Vec::new();
        let due: Vec<u8> = self
            .ranks
            .iter()
            .filter(|rank| rank.character_level == level)
            .map(|rank| rank.rank)
            .collect();
        for rank in due {
            if self.ascend(rank) {
                mythic_ranks.push(rank);
            }
        }

        LevelSheet {
            character_level: level,
            class: level_up.class.clone(),
            class_level,
            ability_increase,
            skill_points,
            slots,
            mythic_ranks,
            state: self.state.clone(),
        }
    }

    /// On a class's first level: the class, archetype and advancement choices are checked against the character
    /// before the level.
    fn check_class_entry(&mut self, level: u8, level_up: &LevelUp) {
        if self.state.class_level(&level_up.class) > 0 {
            return;
        }
        let engine = self.engine;
        let Some((class, _)) = engine.class(&level_up.class) else {
            self.problem(
                Choice::Class { level },
                Issue::UnknownKey(level_up.class.to_string()),
            );
            return;
        };
        if let Some(prerequisite) = &class.prerequisite
            && let Err(unmet) = self.state.check(prerequisite)
        {
            self.problem(Choice::Class { level }, Issue::Unmet(unmet));
        }
        let choice = self.build.class_choices.get(&level_up.class);
        let at_archetype = || Choice::Archetype {
            class: level_up.class.clone(),
        };
        if let Some(key) = choice.and_then(|choice| choice.archetype.as_ref()) {
            match engine.archetypes.get(key) {
                None => self.problem(at_archetype(), Issue::UnknownKey(key.to_string())),
                Some(_) => match engine.archetype_of(class, Some(key)) {
                    None => self.problem(at_archetype(), Issue::NotAllowed),
                    Some(archetype) => {
                        if let Some(prerequisite) = &archetype.prerequisite
                            && let Err(unmet) = self.state.check(prerequisite)
                        {
                            self.problem(at_archetype(), Issue::Unmet(unmet));
                        }
                    }
                },
            }
        }
        if let Some(choice) = choice
            && !choice.advances.is_empty()
        {
            self.check_advances(class, &choice.advances);
        }
    }

    /// A prestige class advances at most one arcane and one divine class, each already taken and casting the
    /// spells it advances. Unknown keys are reported one by one; anything else wrong is one `NotAllowed`.
    fn check_advances(&mut self, prestige: &Class, advances: &BTreeSet<ClassKey>) {
        let engine = self.engine;
        let at = || Choice::Advances {
            class: prestige.key.clone(),
        };
        let advances_arcane = prestige.levels.iter().any(|row| row.advances_arcane);
        let advances_divine = prestige.levels.iter().any(|row| row.advances_divine);
        // For each named class: (can take the arcane advancement, can take the divine one).
        let mut serves = Vec::new();
        for key in advances {
            let Some((class, _)) = engine.class(key) else {
                self.problem(at(), Issue::UnknownKey(key.to_string()));
                continue;
            };
            let held = self.state.classes.iter().find(|c| &c.class == key);
            let (casts_arcane, casts_divine) =
                engine.casts(class, held.and_then(|c| c.archetype.as_ref()));
            let arcane = advances_arcane && casts_arcane;
            let divine = advances_divine && casts_divine;
            serves.push((held.is_some() && arcane, held.is_some() && divine));
        }
        let fits = match serves.as_slice() {
            [] => true,
            [(arcane, divine)] => *arcane || *divine,
            [(a1, d1), (a2, d2)] => (*a1 && *d2) || (*d1 && *a2),
            _ => false,
        };
        if !fits {
            self.problem(at(), Issue::NotAllowed);
        }
    }

    /// Adds one level of the Level-up's class and the features and feats its row grants. Returns the class level
    /// reached.
    fn take_class_level(&mut self, level: u8, level_up: &LevelUp) -> u8 {
        let engine = self.engine;
        let class = engine.class(&level_up.class);
        let index = match self
            .state
            .classes
            .iter()
            .position(|c| c.class == level_up.class)
        {
            Some(index) => {
                self.state.classes[index].level += 1;
                index
            }
            None => {
                let choice = self.build.class_choices.get(&level_up.class);
                self.state.classes.push(ClassLevels {
                    class: level_up.class.clone(),
                    prestige: class.is_some_and(|(_, prestige)| prestige),
                    archetype: choice.and_then(|choice| choice.archetype.clone()),
                    advances: choice
                        .map(|choice| choice.advances.clone())
                        .unwrap_or_default(),
                    level: 1,
                });
                self.state.classes.len() - 1
            }
        };
        let class_level = self.state.classes[index].level;
        let Some((class, _)) = class else {
            return class_level;
        };
        let cap = class_level_cap(class);
        if class_level > cap {
            self.problem(Choice::Class { level }, Issue::ClassLevelCap { cap });
        }
        let archetype = self.state.classes[index].archetype.as_ref();
        if let Some(row) = engine.row(class, archetype, class_level) {
            for feature in row.features {
                add_feature(&mut self.state, feature);
            }
            // Granted feats skip their prerequisites; one already held adds nothing.
            for feat in row.feats {
                self.hold(Held::Feat(feat.clone()), None);
            }
        }
        class_level
    }

    fn skill_ranks(&mut self, level: u8, level_up: &LevelUp) -> SkillPoints {
        // Ranks in a skill can't exceed the Character level.
        let cap = level;
        for (skill, ranks) in &level_up.skill_ranks {
            if !self.engine.skills.contains_key(skill) {
                self.problem(
                    Choice::SkillRanks {
                        level,
                        skill: skill.clone(),
                    },
                    Issue::UnknownKey(skill.to_string()),
                );
            }
            let total = self.state.skill_ranks.entry(skill.clone()).or_default();
            *total = total.saturating_add(*ranks);
            let total = *total;
            if total > cap {
                self.problem(
                    Choice::SkillRanks {
                        level,
                        skill: skill.clone(),
                    },
                    Issue::SkillRankCap { cap, ranks: total },
                );
            }
        }

        let engine = self.engine;
        let class_points = engine.class(&level_up.class).map_or(0, |(class, _)| {
            let archetype = self
                .state
                .classes
                .iter()
                .find(|c| c.class == level_up.class)
                .and_then(|c| c.archetype.as_ref());
            engine
                .archetype_of(class, archetype)
                .and_then(|archetype| archetype.skill_points)
                .unwrap_or(class.skill_points)
        });
        let intelligence = self.state.ability_scores.modifier(Ability::Intelligence);
        let available = (i16::from(class_points) + i16::from(intelligence)).max(1);
        let available = u8::try_from(available).unwrap_or(u8::MAX);
        let spent = level_up
            .skill_ranks
            .values()
            .fold(0u8, |sum, ranks| sum.saturating_add(*ranks));
        if spent > available {
            self.problem(
                Choice::SkillPoints { level },
                Issue::SkillPointsOverspent { available, spent },
            );
        }
        SkillPoints { available, spent }
    }

    /// The slots of a Level-up, in checking order: race (level 1), class rows, the general slot (odd levels), then
    /// Feature selections.
    fn level_slots(&self, level: u8, level_up: &LevelUp) -> Vec<(SlotKind, SlotId)> {
        let engine = self.engine;
        let mut slots = Vec::new();
        let feat = |source, index: usize| SlotId::Feat {
            level,
            source,
            index: u8::try_from(index).unwrap_or(u8::MAX),
        };
        if level == 1
            && let Some(race) = self.state.race.as_ref().and_then(|key| engine.race(key))
        {
            for (index, slot) in race.feat_slots.iter().enumerate() {
                slots.push((SlotKind::Feat(slot.clone()), feat(FeatSource::Race, index)));
            }
        }
        let mut selections = Vec::new();
        if let Some((class, _)) = engine.class(&level_up.class)
            && let Some(entry) = self
                .state
                .classes
                .iter()
                .find(|c| c.class == level_up.class)
        {
            let row = engine.row(class, entry.archetype.as_ref(), entry.level);
            for (index, slot) in row.iter().flat_map(|row| row.feat_slots).enumerate() {
                slots.push((SlotKind::Feat(slot.clone()), feat(FeatSource::Class, index)));
            }
            for (index, selection) in row.iter().flat_map(|row| row.selections).enumerate() {
                selections.push((
                    SlotKind::Selection(selection.clone()),
                    SlotId::Selection {
                        level,
                        index: u8::try_from(index).unwrap_or(u8::MAX),
                    },
                ));
            }
        }
        if level % 2 == 1 {
            slots.push((
                SlotKind::Feat(FeatSlot::General),
                feat(FeatSource::General, 0),
            ));
        }
        slots.extend(selections);
        slots
    }

    /// Fills a Level-up's or Mythic ascension's slots in order, then reports the picks that `belongs` says are
    /// its own but that fill none of its slots.
    fn fill_all(
        &mut self,
        slots: Vec<(SlotKind, SlotId)>,
        rank: Option<u8>,
        belongs: impl Fn(&SlotId) -> bool,
    ) -> Vec<Slot> {
        let slots: Vec<Slot> = slots
            .into_iter()
            .map(|(kind, id)| self.fill(id, kind, rank))
            .collect();
        let stray: Vec<SlotId> = self
            .build
            .picks
            .keys()
            .filter(|id| belongs(id) && !slots.iter().any(|slot| &slot.id == *id))
            .cloned()
            .collect();
        for id in stray {
            self.problem(Choice::Slot(id), Issue::NoSuchSlot);
        }
        slots
    }

    /// Checks the slot's pick against the state just before it and, when it holds, adds it to the state.
    fn fill(&mut self, id: SlotId, kind: SlotKind, rank: Option<u8>) -> Slot {
        let source = rank.map(|rank| GrantSource {
            rank,
            from_path: !matches!(id, SlotId::Mythic { .. }),
            selection: match &kind {
                SlotKind::Selection(selection) => Some(selection.clone()),
                _ => None,
            },
            slot: Some(id.clone()),
        });
        if let Some(Probe::Slot(probed)) = self.probe
            && probed == &id
        {
            self.probed.state = Some(self.state.clone());
            self.probed.kind = Some(kind.clone());
        }
        let pick = self.build.picks.get(&id).cloned();
        if let Some(pick) = &pick {
            match self.check_pick(&kind, pick) {
                Ok(()) => self.hold_pick(pick, source.as_ref()),
                Err(issue) => self.problem(Choice::Slot(id.clone()), issue),
            }
        }
        Slot { id, kind, pick }
    }

    fn check_pick(&self, kind: &SlotKind, pick: &Pick) -> Result<(), Issue> {
        check_pick(self.engine, &self.state, kind, pick)
    }

    fn hold_pick(&mut self, pick: &Pick, source: Option<&GrantSource>) {
        let mut held = vec![];
        match pick {
            Pick::Feat(key) => held.push(Held::Feat(key.clone())),
            Pick::SelectionOption(key) => {
                held.push(Held::SelectionOption(key.clone()));
                if let Some(option) = self.engine.option(key) {
                    held.extend(option.feats.iter().cloned().map(Held::Feat));
                }
            }
            Pick::MythicAbility(key) => held.push(Held::MythicAbility(key.clone())),
            Pick::MythicFeat(key) => held.push(Held::MythicFeat(key.clone())),
        }
        for item in held {
            self.hold(item, source.cloned());
        }
    }

    /// Holds an item; `source` says which Mythic ascension granted it, so a Path switch can take it back.
    fn hold(&mut self, item: Held, source: Option<GrantSource>) {
        let state = &mut self.state;
        match &item {
            Held::Feat(key) => {
                let holds = self.feat_holds.entry(key.clone()).or_default();
                *holds = holds.saturating_add(1);
                state.feats.insert(key.clone());
            }
            Held::Feature(key) => {
                add_feature(state, key);
                if self.engine.feature_bonuses.contains_key(key) {
                    self.refresh_ability_scores();
                }
            }
            Held::SelectionOption(key) => {
                state.selection_options.insert(key.clone());
            }
            Held::MythicAbility(key) => {
                state.mythic_abilities.insert(key.clone());
            }
            Held::MythicFeat(key) => {
                state.mythic_feats.insert(key.clone());
            }
        }
        if let Some(source) = source {
            self.rank_grants.push(RankGrant { source, item });
        }
    }

    fn release(&mut self, item: &Held) {
        let state = &mut self.state;
        match item {
            Held::Feat(key) => {
                let holds = self.feat_holds.entry(key.clone()).or_default();
                *holds = holds.saturating_sub(1);
                if *holds == 0 {
                    state.feats.remove(key);
                }
            }
            Held::Feature(key) => {
                if let Some(count) = state.features.get_mut(key) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        state.features.remove(key);
                    }
                }
                if self.engine.feature_bonuses.contains_key(key) {
                    self.refresh_ability_scores();
                }
            }
            Held::SelectionOption(key) => {
                state.selection_options.remove(key);
            }
            Held::MythicAbility(key) => {
                state.mythic_abilities.remove(key);
            }
            Held::MythicFeat(key) => {
                state.mythic_feats.remove(key);
            }
        }
    }

    // ---- Mythic ascension ---------------------------------------------------------------------------------

    /// Applies one Mythic rank. Returns whether the rank was reached.
    fn ascend(&mut self, rank: u8) -> bool {
        if self.ascensions_over {
            return false;
        }
        let engine = self.engine;
        if rank == PATH_SWITCH_RANK
            && let Some(key) = &self.build.path_switch
            && self.switch_path(rank, key)
        {
            return false;
        }
        if rank == PATH_CHOICE_RANK
            && let Some(key) = &self.build.mythic_path
        {
            match engine.path(key) {
                None => self.problem(Choice::MythicPath, Issue::UnknownKey(key.to_string())),
                Some(path) if path.switch.is_some() => {
                    self.problem(Choice::MythicPath, Issue::NotChoosable);
                }
                Some(path) => {
                    self.state.mythic_path = Some(path.key.clone());
                    self.grant_rows_before_the_choice(path);
                }
            }
        }
        self.state.mythic_rank = rank;

        let mut slot_kinds = vec![(
            match mythic_pick_kind(rank) {
                MythicPickKind::Ability => SlotKind::MythicAbility,
                MythicPickKind::Feat => SlotKind::MythicFeat,
            },
            SlotId::Mythic { rank },
        )];
        let path = self
            .state
            .mythic_path
            .as_ref()
            .and_then(|key| engine.path(key));
        if let Some(row) = path.and_then(|path| path.ranks.iter().find(|row| row.rank == rank)) {
            for feature in &row.features {
                self.hold(Held::Feature(feature.clone()), Some(GrantSource::row(rank)));
            }
            for (index, slot) in row.feat_slots.iter().enumerate() {
                slot_kinds.push((
                    SlotKind::Feat(slot.clone()),
                    SlotId::PathFeat {
                        rank,
                        index: u8::try_from(index).unwrap_or(u8::MAX),
                    },
                ));
            }
            for (index, selection) in row.selections.iter().enumerate() {
                slot_kinds.push((
                    SlotKind::Selection(selection.clone()),
                    SlotId::PathSelection {
                        rank,
                        index: u8::try_from(index).unwrap_or(u8::MAX),
                    },
                ));
            }
        }
        let slots = self.fill_all(slot_kinds, Some(rank), |id| id.rank() == Some(rank));

        let path = self.state.mythic_path.clone();
        if let Some(sheet) = self.ranks.iter_mut().find(|sheet| sheet.rank == rank) {
            sheet.status = RankStatus::Reached;
            sheet.path = path;
            sheet.slots = slots;
        }
        true
    }

    /// Choosing the path at MR 3 grants its MR 1 and 2 rows: held from now on, as those ranks' own grants (so a
    /// switch that keeps MR 1-2 keeps them), and shown at those ranks. Only their features: `GameData::check`
    /// gives those rows no slots.
    fn grant_rows_before_the_choice(&mut self, path: &'a MythicPath) {
        for row in path.ranks.iter().filter(|row| row.rank < PATH_CHOICE_RANK) {
            for feature in &row.features {
                self.hold(
                    Held::Feature(feature.clone()),
                    Some(GrantSource::row(row.rank)),
                );
            }
        }
        for sheet in &mut self.ranks {
            if sheet.rank < PATH_CHOICE_RANK && sheet.status == RankStatus::Reached {
                sheet.path = Some(path.key.clone());
            }
        }
    }

    /// Switches path at the switch rank. Returns whether the ascensions end here (a switch without further
    /// ascensions, such as Legend). A switch the Build's path can't take is a problem and doesn't happen.
    fn switch_path(&mut self, rank: u8, key: &MythicPathKey) -> bool {
        let Some(path) = self.engine.path(key) else {
            self.problem(Choice::PathSwitch, Issue::UnknownKey(key.to_string()));
            return false;
        };
        let Some(switch) = &path.switch else {
            self.problem(Choice::PathSwitch, Issue::NotChoosable);
            return false;
        };
        if !switch.open_from(self.state.mythic_path.as_ref()) {
            self.problem(
                Choice::PathSwitch,
                Issue::NotFromPath {
                    from: switch.from.clone(),
                },
            );
            return false;
        }

        let (dropped, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut self.rank_grants)
            .into_iter()
            .partition(|grant| switch_drops(switch, grant));
        self.rank_grants = kept;
        for grant in &dropped {
            self.release(&grant.item);
        }
        for sheet in &mut self.ranks {
            if sheet.rank >= rank || sheet.status != RankStatus::Reached {
                continue;
            }
            let general_lost = sheet.rank > switch.kept_through_rank;
            let path_lost = sheet.rank > switch.path_kept_through_rank;
            let named_lost = dropped
                .iter()
                .any(|grant| grant.source.rank == sheet.rank && grant.source.from_path);
            sheet.status = if general_lost && path_lost {
                RankStatus::Dropped
            } else if general_lost || path_lost || named_lost {
                RankStatus::PartlyDropped
            } else {
                RankStatus::Reached
            };
            let lost: BTreeSet<&SlotId> = dropped
                .iter()
                .filter(|grant| grant.source.rank == sheet.rank)
                .filter_map(|grant| grant.source.slot.as_ref())
                .collect();
            sheet.dropped_slots = sheet
                .slots
                .iter()
                .map(|slot| &slot.id)
                .filter(|id| lost.contains(id))
                .cloned()
                .collect();
        }

        self.state.mythic_path = Some(path.key.clone());
        for feature in &switch.features {
            self.hold(Held::Feature(feature.clone()), None);
        }
        if let Some(cap) = switch.level_cap {
            self.level_cap = cap;
        }
        if switch.further_ascensions {
            false
        } else {
            self.state.mythic_rank = switch.rank_after_switch();
            self.ascensions_over = true;
            true
        }
    }

    /// The character's own scores: the base scores with the bonuses of every feature held.
    fn refresh_ability_scores(&mut self) {
        let engine = self.engine;
        let bonuses = self
            .state
            .features
            .keys()
            .filter_map(|key| engine.feature_bonuses.get(key))
            .flatten();
        self.state.ability_scores = with_bonuses(self.state.base_ability_scores, bonuses);
    }

    // ---- Tables -------------------------------------------------------------------------------------------

    /// BAB and saves from every class's progressions at its class level; spell-level access and caster level
    /// from its row at its class level plus the prestige levels that advance it.
    fn recompute_tables(&mut self) {
        let engine = self.engine;
        let state = &self.state;
        let (mut bab, mut fort, mut reflex, mut will) = (0u8, 0u8, 0u8, 0u8);
        let (mut arcane, mut divine, mut caster_level) = (0u8, 0u8, 0u8);
        for entry in &state.classes {
            let Some((class, prestige)) = engine.class(&entry.class) else {
                continue;
            };
            // A level past the class's table (already a problem) adds none.
            if entry.level <= class_level_cap(class) {
                let level = entry.level;
                bab = bab.saturating_add(class.base_attack_bonus(level));
                let [f, r, w] = Save::ALL.map(|save| class.save_bonus(save, level, prestige));
                fort = fort.saturating_add(f);
                reflex = reflex.saturating_add(r);
                will = will.saturating_add(w);
            }

            // Spellcasting: this class's own level plus the prestige levels that advance it.
            let archetype = entry.archetype.as_ref();
            let (casts_arcane, casts_divine) = engine.casts(class, archetype);
            let advanced =
                advancing_levels(engine, state, &entry.class, casts_arcane, casts_divine);
            let effective = entry.level.saturating_add(advanced);
            if let Some(row) = engine.row(class, archetype, effective.min(class_level_cap(class))) {
                arcane = arcane.max(row.spell_levels.arcane);
                divine = divine.max(row.spell_levels.divine);
                if row.spell_levels.any() {
                    caster_level = caster_level.max(effective);
                }
            }
        }
        let state = &mut self.state;
        state.bab = bab;
        state.fort = fort;
        state.reflex = reflex;
        state.will = will;
        state.arcane_spell_level = arcane;
        state.divine_spell_level = divine;
        state.caster_level = caster_level;
    }
}

/// Whether `pick` may fill a slot of `kind` for a character in `state`: a known key, accepted by the slot, not
/// already held, prerequisites met (unless the slot is a bonus-list slot). Checked in that order.
pub(crate) fn check_pick(
    engine: &Engine,
    state: &CharacterState,
    kind: &SlotKind,
    pick: &Pick,
) -> Result<(), Issue> {
    let unknown = || Issue::UnknownKey(pick.key().to_owned());
    let prerequisite = match (kind, pick) {
        (_, Pick::Feat(key)) => {
            let feat = engine.feat(key).ok_or_else(unknown)?;
            let SlotKind::Feat(slot) = kind else {
                return Err(Issue::NotAllowed);
            };
            if !engine.slot_accepts(slot, feat) {
                return Err(Issue::NotAllowed);
            }
            if state.feats.contains(key) {
                return Err(Issue::AlreadyHeld);
            }
            if engine.slot_ignores_prerequisites(slot) {
                return Ok(());
            }
            feat.prerequisite.as_ref()
        }
        (_, Pick::SelectionOption(key)) => {
            let option = engine.option(key).ok_or_else(unknown)?;
            let SlotKind::Selection(selection) = kind else {
                return Err(Issue::NotAllowed);
            };
            // Its own option or one it shares; a shared option keeps its one key, so it's held once.
            if !engine
                .selection(selection)
                .is_some_and(|selection| selection.offers(key))
            {
                return Err(Issue::NotAllowed);
            }
            if state.selection_options.contains(key) {
                return Err(Issue::AlreadyHeld);
            }
            option.prerequisite.as_ref()
        }
        (_, Pick::MythicAbility(key)) => {
            let ability = engine.mythic_ability(key).ok_or_else(unknown)?;
            if kind != &SlotKind::MythicAbility {
                return Err(Issue::NotAllowed);
            }
            if state.mythic_abilities.contains(key) {
                return Err(Issue::AlreadyHeld);
            }
            ability.prerequisite.as_ref()
        }
        (_, Pick::MythicFeat(key)) => {
            let feat = engine.mythic_feat(key).ok_or_else(unknown)?;
            if kind != &SlotKind::MythicFeat {
                return Err(Issue::NotAllowed);
            }
            if state.mythic_feats.contains(key) {
                return Err(Issue::AlreadyHeld);
            }
            feat.prerequisite.as_ref()
        }
    };
    match prerequisite {
        Some(prerequisite) => state.check(prerequisite).map_err(Issue::Unmet),
        None => Ok(()),
    }
}

fn problem_sites(problems: &[Problem]) -> HashSet<Choice> {
    problems.iter().map(|problem| problem.at.clone()).collect()
}

fn add_feature(state: &mut CharacterState, feature: &FeatureKey) {
    let count = state.features.entry(feature.clone()).or_default();
    *count = count.saturating_add(1);
}

/// Prestige levels that advance `class`: rows marked `advances_arcane` count when `class` casts arcane spells,
/// `advances_divine` when it casts divine ones, in prestige classes that chose `class` on entry.
fn advancing_levels(
    engine: &Engine,
    state: &CharacterState,
    class: &ClassKey,
    casts_arcane: bool,
    casts_divine: bool,
) -> u8 {
    let mut levels = 0u8;
    for prestige in state.classes.iter().filter(|c| c.advances.contains(class)) {
        let Some((prestige_class, _)) = engine.class(&prestige.class) else {
            continue;
        };
        let count = prestige_class
            .levels
            .iter()
            .filter(|row| row.level <= prestige.level)
            .filter(|row| {
                (row.advances_arcane && casts_arcane) || (row.advances_divine && casts_divine)
            })
            .count();
        levels = levels.saturating_add(u8::try_from(count).unwrap_or(u8::MAX));
    }
    levels
}

/// The scores with features' ability bonuses applied, in the guide's terms: fixed and every-score bonuses add
/// first; the highest score is the highest of those (a tie goes to the first in Str, Dex, Con, Int, Wis, Cha
/// order, which the guide doesn't settle); then every score is raised to the highest floor ("at least 14"), and
/// the highest score gets the sum of the highest-score bonuses (Perfect Body, Mind and Soul: +8 in all).
pub fn with_bonuses<'b>(
    base: AbilityScores,
    bonuses: impl IntoIterator<Item = &'b AbilityBonus>,
) -> AbilityScores {
    let mut scores = base;
    let (mut floor, mut highest_bonus) = (0u8, 0u8);
    for bonus in bonuses {
        match *bonus {
            AbilityBonus::Fixed { ability, amount } => {
                let score = scores.get_mut(ability);
                *score = score.saturating_add(amount);
            }
            AbilityBonus::Every { amount } => {
                for ability in AbilityScores::ABILITIES {
                    let score = scores.get_mut(ability);
                    *score = score.saturating_add(amount);
                }
            }
            AbilityBonus::Highest { amount } => {
                highest_bonus = highest_bonus.saturating_add(amount)
            }
            AbilityBonus::AtLeast { score } => floor = floor.max(score),
        }
    }
    // `max_by_key` keeps the last of equal maxima, so walk the abilities backwards to keep the first.
    let highest = AbilityScores::ABILITIES
        .into_iter()
        .rev()
        .max_by_key(|ability| scores.get(*ability));
    for ability in AbilityScores::ABILITIES {
        let score = scores.get_mut(ability);
        *score = (*score).max(floor);
    }
    if let Some(highest) = highest
        && highest_bonus > 0
    {
        let score = scores.get_mut(highest);
        *score = score.saturating_add(highest_bonus);
    }
    scores
}

/// The cost of one point-buy score (Pathfinder's table; 7 to 9 refund points), or `None` outside 7..=18.
fn point_buy_cost(score: u8) -> Option<i8> {
    const COSTS: [i8; 12] = [-4, -2, -1, 0, 1, 2, 3, 5, 7, 10, 13, 17];
    COSTS.get(usize::from(score.checked_sub(7)?)).copied()
}

/// A class's level cap: the length of its table (20 for base classes, 10 for prestige classes). An archetype
/// has its base class's table.
fn class_level_cap(class: &Class) -> u8 {
    u8::try_from(class.levels.len()).unwrap_or(u8::MAX)
}
