//! The [`Catalog`]: the game data and its lookups by key. The engine finds a class, feat, option, path or feature
//! fact here, and nowhere else reads the indexes or the feature facts. Pure data access: nothing here walks a
//! Build.

use std::collections::{BTreeSet, HashMap, HashSet};

use super::{EffectiveRow, canonical_archetype_row, class_row, effective_rows, merged_row};
use crate::build::{BuildStart, Pick};
use crate::game_data::{
    AbilityBonus, AbilityModifier, Archetype, Background, Class, Deity, EXOTIC_WEAPON_PROFICIENCY,
    Feat, FeatCategory, FeatList, FeatureSelection, GameData, IMPROVED_UNARMED_STRIKE,
    MythicAbility, MythicFeat, MythicPath, OptionRow, Race, SelectionOption, Skill, Subrace,
    UNARMED_STRIKE, Weapon, WeaponProficiency,
};
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, DeityKey, FeatKey, FeatListKey, FeatureKey,
    FeatureSelectionKey, MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey,
    SelectionOptionKey, SkillKey, SubraceKey, WeaponKey,
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
    /// The one option each selection offering exactly one grants with no pick
    /// ([`find_single_option`](Self::find_single_option)).
    single_options: HashMap<FeatureSelectionKey, SelectionOptionKey>,
    /// The members of each option's pool (a mystery's revelations): the options its pool selection offers that the
    /// data has. Only options whose pool selection the data has.
    pools: HashMap<SelectionOptionKey, BTreeSet<SelectionOptionKey>>,
    /// Whether each class, and each class taken with each of its archetypes, casts spells of each type.
    casting: Casting,
    /// The prerequisite of each class with an alignment requirement and archetypes, without that requirement:
    /// what an archetype stating its own leaves of the class's (`None`: nothing).
    without_alignment: HashMap<ClassKey, Option<Prerequisite>>,
}

/// What a pick takes, read once ([`Catalog::pick_item`]): its name, picker chip, summary, prerequisite, the kind
/// of value its pick names, and the feat holding it holds.
pub(super) struct PickItem<'a> {
    pub name: &'a str,
    /// A feat's category, the Feature selection that defines an option (a slayer's shared talents are "Rogue
    /// Talents"), a mythic pick's guide heading.
    pub category: &'a str,
    pub summary: &'a str,
    pub prerequisite: Option<&'a Prerequisite>,
    /// The kind of value its pick names: a parameterised feat's, an option's chosen feat's or its own.
    pub parameter: Option<ParameterKind>,
    /// The feat holding it holds: the feat itself, or an option's chosen feat.
    pub feat: Option<&'a FeatKey>,
    /// That feat's category, which picks its icon. `None` when it holds no feat, mythic picks included.
    pub feat_category: Option<FeatCategory>,
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
        let mut catalog = Self {
            indexes: Indexes::new(&data),
            features: FeatureFacts::new(&data),
            single_options: HashMap::new(),
            pools: HashMap::new(),
            casting: Casting::new(&data),
            without_alignment,
            data,
        };
        catalog.single_options = catalog
            .indexes
            .selections
            .keys()
            .filter_map(|key| Some((key.clone(), catalog.find_single_option(key)?)))
            .collect();
        catalog.pools = catalog
            .indexes
            .options
            .keys()
            .filter_map(|key| {
                let pool = catalog
                    .selection(catalog.option(key)?.pool.as_ref()?)
                    .expect("an option's pool is a selection of the data (GameData::check)");
                let members = pool.offered().into_iter().cloned().collect();
                Some((key.clone(), members))
            })
            .collect();
        catalog
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

    pub(super) fn deity(&self, key: &DeityKey) -> Option<&Deity> {
        self.indexes
            .deities
            .get(key)
            .map(|&index| &self.data.deities[index])
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

    /// Whether a selection's slot offers `option` by the game data alone: one of its own options (the one
    /// [`FeatureSelection::offers`] would find by a scan, the index saying which selection defines each option) or one
    /// it shares. A pool selection may offer more: see `offers_now`.
    pub(super) fn offers(&self, selection: &FeatureSelection, option: &SelectionOptionKey) -> bool {
        selection.shared_options.contains(option)
            || self
                .indexes
                .options
                .get(option)
                .is_some_and(|(defined_by, _)| defined_by == &selection.key)
    }

    /// The one option a selection offers, when it offers exactly one: then it's granted with no pick.
    pub(super) fn single_option(
        &self,
        selection: &FeatureSelectionKey,
    ) -> Option<&SelectionOptionKey> {
        self.single_options.get(selection)
    }

    /// The options of an option's pool, when it has one.
    pub(super) fn pool(
        &self,
        option: &SelectionOptionKey,
    ) -> Option<&BTreeSet<SelectionOptionKey>> {
        self.pools.get(option)
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

    /// The item a pick takes (its feat, selection option, mythic ability or mythic feat) as the labels and pickers
    /// read it. `None` for a key the data lacks.
    pub(super) fn pick_item(&self, pick: &Pick) -> Option<PickItem<'_>> {
        Some(match pick {
            Pick::Feat { feat, .. } => {
                let feat = self.feat(feat)?;
                PickItem {
                    name: &feat.name,
                    category: feat.category.name(),
                    summary: &feat.summary,
                    prerequisite: feat.prerequisite.as_ref(),
                    parameter: feat.parameter,
                    feat: Some(&feat.key),
                    feat_category: Some(feat.category),
                }
            }
            Pick::SelectionOption { option, .. } => {
                let category = self
                    .selection_of(option)
                    .map_or("", |selection| &selection.name);
                let option = self.option(option)?;
                PickItem {
                    name: &option.name,
                    category,
                    summary: &option.summary,
                    prerequisite: option.prerequisite.as_ref(),
                    parameter: self.option_parameter(option),
                    feat: option.chosen_feat.as_ref(),
                    feat_category: option
                        .chosen_feat
                        .as_ref()
                        .and_then(|feat| self.feat(feat))
                        .map(|feat| feat.category),
                }
            }
            Pick::MythicAbility(key) => {
                let ability = self.mythic_ability(key)?;
                PickItem {
                    name: &ability.name,
                    category: ability.group.name(),
                    summary: &ability.summary,
                    prerequisite: ability.prerequisite.as_ref(),
                    parameter: None,
                    feat: None,
                    feat_category: None,
                }
            }
            Pick::MythicFeat { feat, .. } => {
                let feat = self.mythic_feat(feat)?;
                PickItem {
                    name: &feat.name,
                    category: feat.group.name(),
                    summary: &feat.summary,
                    prerequisite: feat.prerequisite.as_ref(),
                    parameter: feat.parameter,
                    feat: None,
                    feat_category: None,
                }
            }
        })
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

    /// The row in force at one level of a class taken with an archetype key (ignored unless it's the class's), as
    /// [`effective_row`](super::effective_row) gives it: the base row read by its position, the archetype's found by
    /// halving.
    pub(super) fn row<'a>(
        &'a self,
        class: &'a Class,
        archetype: Option<&ArchetypeKey>,
        level: u8,
    ) -> Option<EffectiveRow<'a>> {
        self.row_with(class, self.archetype_of(class, archetype), level)
    }

    /// [`row`](Self::row) with the archetype found already (one of the class's, or none).
    fn row_with<'a>(
        &'a self,
        class: &'a Class,
        archetype: Option<&'a Archetype>,
        level: u8,
    ) -> Option<EffectiveRow<'a>> {
        let base = class_row(class, level)?;
        let own = archetype.and_then(|archetype| canonical_archetype_row(archetype, level));
        Some(merged_row(base, own))
    }

    /// Every row of a class's table taken with an archetype (one of the class's, or none), in level order, as
    /// [`row`](Self::row) gives it.
    pub(super) fn rows<'a>(
        &'a self,
        class: &'a Class,
        archetype: Option<&'a Archetype>,
    ) -> impl Iterator<Item = EffectiveRow<'a>> {
        class
            .levels
            .iter()
            .filter_map(move |row| self.row_with(class, archetype, row.level))
    }

    /// Whether a class taken with an archetype key casts spells of each type at any level: (arcane, divine).
    pub(super) fn casts(&self, class: &Class, archetype: Option<&ArchetypeKey>) -> (bool, bool) {
        let archetype = self.archetype_of(class, archetype);
        let known = match archetype {
            Some(archetype) => self.casting.archetypes.get(&archetype.key),
            None => self.casting.classes.get(&class.key),
        };
        known
            .copied()
            .unwrap_or_else(|| casts_with(class, archetype))
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

    /// Whether a feature grants the deity's favoured weapons (Cleric Proficiencies).
    pub(super) fn grants_favoured_weapons(&self, feature: &FeatureKey) -> bool {
        self.features.favoured_weapons.contains(feature)
    }

    /// The feats a follower of `deity` holds for its favoured weapons while holding a feature that grants them,
    /// each with the values it's held for: an exotic weapon's proficiency for that weapon, Improved Unarmed Strike for
    /// the unarmed strike, and nothing for a simple or martial weapon (no feat checks proficiency with one weapon).
    pub(super) fn favoured_weapon_feats(&self, deity: &DeityKey) -> Vec<(FeatKey, Vec<Parameter>)> {
        let Some(deity) = self.deity(deity) else {
            return Vec::new();
        };
        let mut feats = Vec::new();
        for key in &deity.favoured_weapons {
            if key.as_str() == UNARMED_STRIKE {
                feats.push((FeatKey::new(IMPROVED_UNARMED_STRIKE), Vec::new()));
            } else if self
                .weapon(key)
                .is_some_and(|weapon| weapon.proficiency == Some(WeaponProficiency::Exotic))
            {
                feats.push((
                    FeatKey::new(EXOTIC_WEAPON_PROFICIENCY),
                    vec![Parameter::Weapon(key.clone())],
                ));
            }
        }
        feats
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
    deities: HashMap<DeityKey, usize>,
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
            deities: index(&data.deities, |d| &d.key),
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

/// Whether a class taken with an archetype (one of the class's, or none) casts spells of each type at any level:
/// (arcane, divine).
fn casts_with(class: &Class, archetype: Option<&Archetype>) -> (bool, bool) {
    effective_rows(class, archetype).fold((false, false), |(arcane, divine), row| {
        (
            arcane || row.spell_levels.arcane > 0,
            divine || row.spell_levels.divine > 0,
        )
    })
}

/// Whether each class casts spells of each type, alone and with each of its archetypes, built once by
/// [`Catalog::new`]: a walk asks for each class held at every level.
#[derive(Clone, Debug)]
struct Casting {
    /// Each class (base or prestige) without an archetype.
    classes: HashMap<ClassKey, (bool, bool)>,
    /// Each base class taken with one of its archetypes, by the archetype's key.
    archetypes: HashMap<ArchetypeKey, (bool, bool)>,
}

impl Casting {
    fn new(data: &GameData) -> Self {
        let classes = data
            .all_classes()
            .map(|class| (class.key.clone(), casts_with(class, None)))
            .collect();
        let archetypes = data
            .classes
            .iter()
            .flat_map(|class| {
                class.archetypes.iter().map(move |archetype| {
                    (archetype.key.clone(), casts_with(class, Some(archetype)))
                })
            })
            .collect();
        Self {
            classes,
            archetypes,
        }
    }
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
    /// Every feature that grants the deity's favoured weapons (Cleric Proficiencies).
    favoured_weapons: HashSet<FeatureKey>,
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
            .class_features()
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
        let favoured_weapons = features
            .iter()
            .filter(|feature| feature.favoured_weapon_proficiency)
            .map(|feature| feature.key.clone())
            .collect();
        Self {
            bonuses,
            rows,
            daily_choices,
            favoured_weapons,
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
