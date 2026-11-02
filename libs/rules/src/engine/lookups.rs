//! The game data's lookups: [`Engine::new`] indexes the game data once, and the engine finds a class, feat,
//! option, path or feature fact by its key here.

use std::collections::{HashMap, HashSet};

use super::{EffectiveRow, Engine, effective_row, effective_rows};
use crate::codec::LinkKeys;
use crate::game_data::{
    AbilityBonus, Background, Class, Feat, FeatList, FeatureSelection, GameData, MythicAbility,
    MythicFeat, MythicPath, OptionRow, Race, SelectionOption, Weapon, WeaponProficiency,
};
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, FeatKey, FeatListKey, FeatureKey, FeatureSelectionKey,
    MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey,
    SubraceKey, WeaponKey,
};
use crate::parameter::{ArmourType, Energy, Parameter, ParameterKind, SpellSchool};
use crate::state::CharacterState;

impl Engine {
    /// Indexes the game data, after putting it in canonical order.
    pub fn new(data: GameData) -> Self {
        let data = data.canonical();
        Self {
            indexes: Indexes::new(&data),
            features: FeatureFacts::new(&data),
            link_keys: LinkKeys::new(&data),
            data,
        }
    }

    pub fn data(&self) -> &GameData {
        &self.data
    }

    pub(super) fn class(&self, key: &ClassKey) -> Option<(&Class, bool)> {
        Some(match *self.indexes.classes.get(key)? {
            ClassAt::Base(index) => (&self.data.classes[index], false),
            ClassAt::Prestige(index) => (&self.data.prestige_classes[index], true),
        })
    }

    pub(super) fn race(&self, key: &RaceKey) -> Option<&Race> {
        self.indexes
            .races
            .get(key)
            .map(|&index| &self.data.races[index])
    }

    pub(super) fn background(&self, key: &BackgroundKey) -> Option<&Background> {
        self.indexes
            .backgrounds
            .get(key)
            .map(|&index| &self.data.backgrounds[index])
    }

    pub(super) fn feat(&self, key: &FeatKey) -> Option<&Feat> {
        self.indexes
            .feats
            .get(key)
            .map(|&index| &self.data.feats[index])
    }

    pub(super) fn weapon(&self, key: &WeaponKey) -> Option<&Weapon> {
        self.indexes
            .weapons
            .get(key)
            .map(|&index| &self.data.weapons[index])
    }

    /// Every value of a kind, in key (or declaration) order: the skills, weapons, exotic weapons, schools, energy
    /// types or armour types.
    pub fn parameter_values(&self, kind: ParameterKind) -> Vec<Parameter> {
        match kind {
            ParameterKind::Skill => self
                .data
                .skills
                .iter()
                .map(|skill| Parameter::Skill(skill.key.clone()))
                .collect(),
            ParameterKind::Weapon | ParameterKind::ExoticWeapon => self
                .data
                .weapons
                .iter()
                .filter(|weapon| {
                    kind == ParameterKind::Weapon
                        || weapon.proficiency == Some(WeaponProficiency::Exotic)
                })
                .map(|weapon| Parameter::Weapon(weapon.key.clone()))
                .collect(),
            ParameterKind::SpellSchool => SpellSchool::ALL.map(Parameter::SpellSchool).to_vec(),
            ParameterKind::Energy => Energy::ALL.map(Parameter::Energy).to_vec(),
            ParameterKind::ArmourType => ArmourType::ALL.map(Parameter::ArmourType).to_vec(),
        }
    }

    /// Whether a pick's value fits what it takes (`kind`: `None` when it takes none): a known value of that kind,
    /// or none for a pick that takes none.
    pub(super) fn value_fits(
        &self,
        kind: Option<ParameterKind>,
        value: Option<&Parameter>,
    ) -> bool {
        use ParameterKind as K;
        match (kind, value) {
            (None, None) => true,
            (Some(K::Skill), Some(Parameter::Skill(key))) => self.indexes.skills.contains_key(key),
            (Some(kind @ (K::Weapon | K::ExoticWeapon)), Some(Parameter::Weapon(key))) => {
                self.weapon(key).is_some_and(|weapon| {
                    kind == K::Weapon || weapon.proficiency == Some(WeaponProficiency::Exotic)
                })
            }
            (Some(K::SpellSchool), Some(Parameter::SpellSchool(_)))
            | (Some(K::Energy), Some(Parameter::Energy(_)))
            | (Some(K::ArmourType), Some(Parameter::ArmourType(_))) => true,
            _ => false,
        }
    }

    /// The values a grant with no pick holds a feat for: every value of a parameterised one, none otherwise.
    pub(super) fn every_value(&self, feat: &FeatKey) -> Vec<Parameter> {
        self.feat(feat)
            .and_then(|feat| feat.parameter)
            .map(|kind| self.parameter_values(kind))
            .unwrap_or_default()
    }

    /// What an option's pick takes: its chosen feat's kind, else its own, else nothing.
    pub(super) fn option_parameter(&self, option: &SelectionOption) -> Option<ParameterKind> {
        option
            .chosen_feat
            .as_ref()
            .and_then(|feat| self.feat(feat))
            .and_then(|feat| feat.parameter)
            .or(option.parameter)
    }

    /// The values an option's pick may name: its own list, else every value of its kind.
    pub(super) fn option_values(&self, option: &SelectionOption) -> Vec<Parameter> {
        if !option.values.is_empty() {
            return option.values.iter().cloned().collect();
        }
        self.option_parameter(option)
            .map(|kind| self.parameter_values(kind))
            .unwrap_or_default()
    }

    /// A feat list by key, wherever it's defined.
    pub fn feat_list(&self, key: &FeatListKey) -> Option<&FeatList> {
        self.indexes.feat_lists.get(key)
    }

    pub(super) fn selection(&self, key: &FeatureSelectionKey) -> Option<&FeatureSelection> {
        Some(self.indexes.selections.get(key)?.selection(&self.data))
    }

    pub(super) fn option(&self, key: &SelectionOptionKey) -> Option<&SelectionOption> {
        let (selection, index) = self.indexes.options.get(key)?;
        self.selection(selection)
            .and_then(|selection| selection.options.get(*index))
    }

    pub(super) fn mythic_ability(&self, key: &MythicAbilityKey) -> Option<&MythicAbility> {
        self.indexes
            .mythic_abilities
            .get(key)
            .map(|&index| &self.data.mythic_abilities[index])
    }

    pub(super) fn mythic_feat(&self, key: &MythicFeatKey) -> Option<&MythicFeat> {
        self.indexes
            .mythic_feats
            .get(key)
            .map(|&index| &self.data.mythic_feats[index])
    }

    pub(super) fn path(&self, key: &MythicPathKey) -> Option<&MythicPath> {
        self.indexes
            .paths
            .get(key)
            .map(|&index| &self.data.mythic_paths[index])
    }

    /// The archetype when it exists and belongs to `class`.
    pub(super) fn archetype_of(
        &self,
        class: &Class,
        archetype: Option<&ArchetypeKey>,
    ) -> Option<&crate::game_data::Archetype> {
        let &(c, a) = self.indexes.archetypes.get(archetype?)?;
        let owner = &self.data.classes[c];
        (owner.key == class.key).then(|| &owner.archetypes[a])
    }

    /// The row in force at one level of a class taken with an archetype key (ignored unless it's the class's).
    pub(super) fn row<'a>(
        &'a self,
        class: &'a Class,
        archetype: Option<&ArchetypeKey>,
        level: u8,
    ) -> Option<EffectiveRow<'a>> {
        effective_row(class, self.archetype_of(class, archetype), level)
    }

    /// Whether a class taken with an archetype key casts spells of each type at any level: (arcane, divine).
    pub(super) fn casts(&self, class: &Class, archetype: Option<&ArchetypeKey>) -> (bool, bool) {
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

    /// Whether a class taken with an archetype key casts spontaneously: the archetype's say, else the class's.
    pub(super) fn spontaneous(&self, class: &Class, archetype: Option<&ArchetypeKey>) -> bool {
        self.archetype_of(class, archetype)
            .and_then(|archetype| archetype.spontaneous)
            .unwrap_or(class.spontaneous)
    }

    /// The daily choices a character holds (Mixed Bloodline): the sheet shows each as a note, since the game asks
    /// for it again every day and a Build doesn't record it.
    pub fn daily_choices(&self, state: &CharacterState) -> Vec<FeatureKey> {
        state
            .features
            .keys()
            .filter(|feature| self.features.daily_choices.contains(*feature))
            .cloned()
            .collect()
    }
}

/// The game data's positions by key, built once by [`Engine::new`].
#[derive(Clone, Debug)]
pub(super) struct Indexes {
    classes: HashMap<ClassKey, ClassAt>,
    pub(super) archetypes: HashMap<ArchetypeKey, (usize, usize)>,
    races: HashMap<RaceKey, usize>,
    pub(super) subraces: HashMap<SubraceKey, (usize, usize)>,
    pub(super) skills: HashMap<SkillKey, usize>,
    weapons: HashMap<WeaponKey, usize>,
    feats: HashMap<FeatKey, usize>,
    feat_lists: HashMap<FeatListKey, FeatList>,
    selections: HashMap<FeatureSelectionKey, SelectionAt>,
    pub(super) options: HashMap<SelectionOptionKey, (FeatureSelectionKey, usize)>,
    mythic_abilities: HashMap<MythicAbilityKey, usize>,
    mythic_feats: HashMap<MythicFeatKey, usize>,
    paths: HashMap<MythicPathKey, usize>,
    backgrounds: HashMap<BackgroundKey, usize>,
}

impl Indexes {
    fn new(data: &GameData) -> Self {
        let (classes, archetypes) = index_classes(data);
        let (races, subraces) = index_races(data);
        let selections = index_selections(data);
        Self {
            options: index_options(data, &selections),
            feat_lists: index_feat_lists(data),
            skills: index(&data.skills, |s| &s.key),
            weapons: index(&data.weapons, |w| &w.key),
            feats: index(&data.feats, |f| &f.key),
            mythic_abilities: index(&data.mythic_abilities, |a| &a.key),
            mythic_feats: index(&data.mythic_feats, |f| &f.key),
            paths: index(&data.mythic_paths, |p| &p.key),
            backgrounds: index(&data.backgrounds, |b| &b.key),
            classes,
            archetypes,
            races,
            subraces,
            selections,
        }
    }
}

/// Each item's position in a list, by its key.
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

/// The base and prestige classes by key, and each archetype by key with its class's position.
fn index_classes(
    data: &GameData,
) -> (
    HashMap<ClassKey, ClassAt>,
    HashMap<ArchetypeKey, (usize, usize)>,
) {
    let mut classes = HashMap::new();
    let mut archetypes = HashMap::new();
    for (c, class) in data.classes.iter().enumerate() {
        classes.insert(class.key.clone(), ClassAt::Base(c));
        for (a, archetype) in class.archetypes.iter().enumerate() {
            archetypes.insert(archetype.key.clone(), (c, a));
        }
    }
    for (c, class) in data.prestige_classes.iter().enumerate() {
        classes.insert(class.key.clone(), ClassAt::Prestige(c));
    }
    (classes, archetypes)
}

/// Every Feature selection by key, wherever it's defined: a base class's, an archetype's, a prestige class's or a
/// path's.
fn index_selections(data: &GameData) -> HashMap<FeatureSelectionKey, SelectionAt> {
    let mut selections = HashMap::new();
    let mut add_selections = |list: &[FeatureSelection], at: &dyn Fn(usize) -> SelectionAt| {
        for (index, selection) in list.iter().enumerate() {
            selections.insert(selection.key.clone(), at(index));
        }
    };
    for (c, class) in data.classes.iter().enumerate() {
        add_selections(&class.selections, &|index| SelectionAt::Class {
            prestige: false,
            class: c,
            index,
        });
        for (a, archetype) in class.archetypes.iter().enumerate() {
            add_selections(&archetype.selections, &|index| SelectionAt::Archetype {
                class: c,
                archetype: a,
                index,
            });
        }
    }
    for (c, class) in data.prestige_classes.iter().enumerate() {
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
    selections
}

/// Every option by key, with the indexed selection that defines it and its position there.
fn index_options(
    data: &GameData,
    selections: &HashMap<FeatureSelectionKey, SelectionAt>,
) -> HashMap<SelectionOptionKey, (FeatureSelectionKey, usize)> {
    selections
        .values()
        .map(|at| at.selection(data))
        .flat_map(|selection| {
            selection
                .options
                .iter()
                .enumerate()
                .map(|(i, option)| (option.key.clone(), (selection.key.clone(), i)))
        })
        .collect()
}

/// The races by key, and each subrace by key with its race's position.
fn index_races(data: &GameData) -> (HashMap<RaceKey, usize>, HashMap<SubraceKey, (usize, usize)>) {
    let mut races = HashMap::new();
    let mut subraces = HashMap::new();
    for (r, race) in data.races.iter().enumerate() {
        races.insert(race.key.clone(), r);
        for (s, subrace) in race.subraces.iter().enumerate() {
            subraces.insert(subrace.key.clone(), (r, s));
        }
    }
    (races, subraces)
}

/// Every feat list by key, wherever it's defined.
fn index_feat_lists(data: &GameData) -> HashMap<FeatListKey, FeatList> {
    data.feat_lists()
        .map(|list| (list.key.clone(), list.clone()))
        .collect()
}

/// What the engine reads off the features every class, archetype and path defines, by key.
#[derive(Clone, Debug)]
pub(super) struct FeatureFacts {
    /// The ability bonuses of every feature that has some.
    pub(super) bonuses: HashMap<FeatureKey, Vec<AbilityBonus>>,
    /// The option row of every feature that grants one (Bloodline Ascendance).
    pub(super) rows: HashMap<FeatureKey, OptionRow>,
    /// Every feature that is a daily choice.
    daily_choices: HashSet<FeatureKey>,
    /// The features that set a floor for prerequisites (Perfect Mind): a BAB floor, and whether every skill's ranks
    /// are at least the Mythic rank.
    pub(super) floors: HashMap<FeatureKey, (Option<u8>, bool)>,
    /// The name of every feature a class, archetype or path defines, for the sheet's labels.
    pub(super) names: HashMap<FeatureKey, String>,
    /// The features whose name another feature shares (the Fighter's and the Sohei's Weapon Training), so a reason
    /// can tell them apart.
    pub(super) shared_names: HashSet<FeatureKey>,
}

impl FeatureFacts {
    fn new(data: &GameData) -> Self {
        let features: Vec<&crate::game_data::Feature> = data
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
            .collect();
        let mut name_counts: HashMap<&str, usize> = HashMap::new();
        for feature in &features {
            *name_counts.entry(feature.name.as_str()).or_default() += 1;
        }
        let shared_names = features
            .iter()
            .filter(|feature| name_counts[feature.name.as_str()] > 1)
            .map(|feature| feature.key.clone())
            .collect();
        let bonuses = features
            .iter()
            .filter(|feature| !feature.ability_bonuses.is_empty())
            .map(|feature| (feature.key.clone(), feature.ability_bonuses.clone()))
            .collect();
        let rows = features
            .iter()
            .filter_map(|feature| Some((feature.key.clone(), feature.option_row.clone()?)))
            .collect();
        let daily_choices = features
            .iter()
            .filter(|feature| feature.daily_choice)
            .map(|feature| feature.key.clone())
            .collect();
        let floors = features
            .iter()
            .filter(|feature| {
                feature.bab_floor.is_some() || feature.skill_ranks_at_least_mythic_rank
            })
            .map(|feature| {
                (
                    feature.key.clone(),
                    (feature.bab_floor, feature.skill_ranks_at_least_mythic_rank),
                )
            })
            .collect();
        let names = features
            .iter()
            .map(|feature| (feature.key.clone(), feature.name.clone()))
            .collect();
        Self {
            bonuses,
            rows,
            daily_choices,
            floors,
            names,
            shared_names,
        }
    }
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

impl SelectionAt {
    /// The selection it points to in `data`, the game data it was indexed from.
    fn selection(self, data: &GameData) -> &FeatureSelection {
        match self {
            SelectionAt::Class {
                prestige,
                class,
                index,
            } => {
                let list = if prestige {
                    &data.prestige_classes
                } else {
                    &data.classes
                };
                &list[class].selections[index]
            }
            SelectionAt::Archetype {
                class,
                archetype,
                index,
            } => &data.classes[class].archetypes[archetype].selections[index],
            SelectionAt::Path { path, index } => &data.mythic_paths[path].selections[index],
        }
    }
}
