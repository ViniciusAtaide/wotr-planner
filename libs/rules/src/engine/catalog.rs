//! The [`Catalog`]: the game data and its lookups by key. The engine finds a class, feat, option, path or feature
//! fact here, and nowhere else reads the indexes or the feature facts. Pure data access: nothing here walks a
//! Build.

use std::collections::{HashMap, HashSet};

use super::{EffectiveRow, effective_row, effective_rows};
use crate::build::BuildStart;
use crate::game_data::{
    AbilityBonus, AbilityModifier, Archetype, Background, Class, Feat, FeatList, FeatureSelection,
    GameData, MythicAbility, MythicFeat, MythicPath, OptionRow, Race, SelectionOption, Skill,
    Subrace, Weapon, WeaponProficiency,
};
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, FeatKey, FeatListKey, FeatureKey, FeatureSelectionKey,
    MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey,
    SubraceKey, WeaponKey,
};
use crate::parameter::{ArmourType, Energy, Parameter, ParameterKind, SpellSchool};
use crate::prerequisite::Prerequisite;

/// The game data in canonical order, with its positions by key and what the engine reads off its features. Built
/// once by [`Engine::new`](super::Engine::new); the engine reads the game data only through it.
#[derive(Clone, Debug)]
pub(super) struct Catalog {
    data: GameData,
    /// The game data's positions by key.
    indexes: Indexes,
    /// What the engine reads off the features the classes, archetypes and paths define.
    features: FeatureFacts,
    /// The prerequisite of each class with an alignment requirement and archetypes, without that requirement:
    /// what an archetype stating its own leaves of the class's (`None`: nothing).
    without_alignment: HashMap<ClassKey, Option<Prerequisite>>,
}

impl Catalog {
    /// Indexes the game data, after putting it in canonical order.
    pub(super) fn new(data: GameData) -> Self {
        let data = data.canonical();
        let without_alignment = data
            .classes
            .iter()
            .filter(|class| !class.archetypes.is_empty())
            .filter_map(|class| {
                let prerequisite = class.prerequisite.as_ref()?;
                prerequisite
                    .states_alignment()
                    .then(|| (class.key.clone(), prerequisite.without_alignment()))
            })
            .collect();
        Self {
            indexes: Indexes::new(&data),
            features: FeatureFacts::new(&data),
            without_alignment,
            data,
        }
    }

    /// The game data, in canonical order.
    pub(super) fn data(&self) -> &GameData {
        &self.data
    }

    pub(super) fn class(&self, key: &ClassKey) -> Option<(&Class, bool)> {
        Some(match *self.indexes.classes.get(key)? {
            ClassAt::Base(index) => (&self.data.classes[index], false),
            ClassAt::Prestige(index) => (&self.data.prestige_classes[index], true),
        })
    }

    pub(super) fn archetype(&self, key: &ArchetypeKey) -> Option<&Archetype> {
        let &(c, a) = self.indexes.archetypes.get(key)?;
        Some(&self.data.classes[c].archetypes[a])
    }

    pub(super) fn race(&self, key: &RaceKey) -> Option<&Race> {
        self.indexes
            .races
            .get(key)
            .map(|&index| &self.data.races[index])
    }

    /// A subrace with the race it belongs to.
    pub(super) fn subrace(&self, key: &SubraceKey) -> Option<(&Race, &Subrace)> {
        let &(r, s) = self.indexes.subraces.get(key)?;
        let race = &self.data.races[r];
        Some((race, &race.subraces[s]))
    }

    /// The ability modifiers a Build start's race gives: its subrace's in place of the race's, when the subrace is
    /// the race's. None without a race the data has.
    pub(super) fn racial_modifiers(&self, start: &BuildStart) -> &[AbilityModifier] {
        let Some(race) = start.race.as_ref().and_then(|key| self.race(key)) else {
            return &[];
        };
        match start.subrace.as_ref().and_then(|key| self.subrace(key)) {
            Some((owner, subrace)) if owner.key == race.key => &subrace.ability_modifiers,
            _ => &race.ability_modifiers,
        }
    }

    pub(super) fn skill(&self, key: &SkillKey) -> Option<&Skill> {
        self.indexes
            .skills
            .get(key)
            .map(|&index| &self.data.skills[index])
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
    pub(super) fn parameter_values(&self, kind: ParameterKind) -> Vec<Parameter> {
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
    pub(super) fn feat_list(&self, key: &FeatListKey) -> Option<&FeatList> {
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

    /// Whether some selection defines this option (a shared key the data lacks offers nothing).
    pub(super) fn has_option(&self, key: &SelectionOptionKey) -> bool {
        self.indexes.options.contains_key(key)
    }

    /// The Feature selection that defines an option.
    pub(super) fn selection_of(&self, option: &SelectionOptionKey) -> Option<&FeatureSelection> {
        let (selection, _) = self.indexes.options.get(option)?;
        self.selection(selection)
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

    /// The character level of a Mythic rank's ascension, if the data has the rank.
    pub(super) fn ascension_level(&self, rank: u8) -> Option<u8> {
        self.data
            .progression
            .mythic_ascensions
            .iter()
            .find(|ascension| ascension.rank == rank)
            .map(|ascension| ascension.character_level)
    }

    /// The archetype when it exists and belongs to `class`.
    pub(super) fn archetype_of(
        &self,
        class: &Class,
        archetype: Option<&ArchetypeKey>,
    ) -> Option<&Archetype> {
        let &(c, a) = self.indexes.archetypes.get(archetype?)?;
        let owner = &self.data.classes[c];
        (owner.key == class.key).then(|| &owner.archetypes[a])
    }

    /// What taking a class with an archetype (one of the class's, or none) needs: the class's prerequisite and the
    /// archetype's. An archetype that states an alignment requirement replaces the class's (ticket 63: the Drunken
    /// Master "Cannot be Lawful", though a monk must be): the rest of the class's still holds. An archetype without
    /// one keeps the class's.
    pub(super) fn entry_prerequisites<'a>(
        &'a self,
        class: &'a Class,
        archetype: Option<&'a Archetype>,
    ) -> [Option<&'a Prerequisite>; 2] {
        let own = archetype.and_then(|archetype| archetype.prerequisite.as_ref());
        let class_needs = match self.without_alignment.get(&class.key) {
            Some(rest) if own.is_some_and(Prerequisite::states_alignment) => rest.as_ref(),
            _ => class.prerequisite.as_ref(),
        };
        [class_needs, own]
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

    /// A feature's ability bonuses, when it has some.
    pub(super) fn ability_bonuses(&self, feature: &FeatureKey) -> Option<&[AbilityBonus]> {
        self.features.bonuses.get(feature).map(Vec::as_slice)
    }

    /// The option row a feature grants (Bloodline Ascendance).
    pub(super) fn option_row(&self, feature: &FeatureKey) -> Option<&OptionRow> {
        self.features.rows.get(feature)
    }

    /// The features whose option row is this row of a selection's options.
    pub(super) fn features_granting_row(
        &self,
        selection: &FeatureSelectionKey,
        level: u8,
    ) -> impl Iterator<Item = &FeatureKey> {
        self.features
            .rows
            .iter()
            .filter(move |(_, row)| &row.selection == selection && row.level == level)
            .map(|(feature, _)| feature)
    }

    /// The floor a feature sets for prerequisites (Perfect Mind): a BAB floor, and whether every skill's ranks are
    /// at least the Mythic rank. `None` for a feature that sets none.
    pub(super) fn floor(&self, feature: &FeatureKey) -> Option<(Option<u8>, bool)> {
        self.features.floors.get(feature).copied()
    }

    /// Whether a feature is a daily choice.
    pub(super) fn is_daily_choice(&self, feature: &FeatureKey) -> bool {
        self.features.daily_choices.contains(feature)
    }

    /// The name of a feature a class, archetype or path defines.
    pub(super) fn feature_name(&self, feature: &FeatureKey) -> Option<&str> {
        self.features.names.get(feature).map(String::as_str)
    }

    /// Whether another feature shares this feature's name (the Fighter's and the Sohei's Weapon Training).
    pub(super) fn shares_name(&self, feature: &FeatureKey) -> bool {
        self.features.shared_names.contains(feature)
    }
}

/// The game data's positions by key, built once by [`Catalog::new`].
#[derive(Clone, Debug)]
struct Indexes {
    classes: HashMap<ClassKey, ClassAt>,
    archetypes: HashMap<ArchetypeKey, (usize, usize)>,
    races: HashMap<RaceKey, usize>,
    subraces: HashMap<SubraceKey, (usize, usize)>,
    skills: HashMap<SkillKey, usize>,
    weapons: HashMap<WeaponKey, usize>,
    feats: HashMap<FeatKey, usize>,
    feat_lists: HashMap<FeatListKey, FeatList>,
    selections: HashMap<FeatureSelectionKey, SelectionAt>,
    options: HashMap<SelectionOptionKey, (FeatureSelectionKey, usize)>,
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
struct FeatureFacts {
    /// The ability bonuses of every feature that has some.
    bonuses: HashMap<FeatureKey, Vec<AbilityBonus>>,
    /// The option row of every feature that grants one (Bloodline Ascendance).
    rows: HashMap<FeatureKey, OptionRow>,
    /// Every feature that is a daily choice.
    daily_choices: HashSet<FeatureKey>,
    /// The features that set a floor for prerequisites (Perfect Mind): a BAB floor, and whether every skill's ranks
    /// are at least the Mythic rank.
    floors: HashMap<FeatureKey, (Option<u8>, bool)>,
    /// The name of every feature a class, archetype or path defines, for the sheet's labels.
    names: HashMap<FeatureKey, String>,
    /// The features whose name another feature shares (the Fighter's and the Sohei's Weapon Training), so a reason
    /// can tell them apart.
    shared_names: HashSet<FeatureKey>,
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
