//! The [`Catalog`]: the game data and its lookups by key. The engine finds a class, feat, option, path or feature
//! fact here, and nowhere else reads the indexes or the feature facts. Pure data access: nothing here walks a
//! Build.

use std::collections::BTreeSet;

use rustc_hash::{FxHashMap, FxHashSet};

mod components;

use components::Components;
pub(super) use components::HolderKind;

use super::{
    EffectiveRow, SlotKind, canonical_archetype_row, class_row, effective_rows, merged_row,
};
use crate::build::{BuildStart, Pick};
use crate::check::CheckedGameData;
use crate::game_data::{
    AbilityModifier, Archetype, BabFromRanks, Background, Band, Class, Deity,
    EXOTIC_WEAPON_PROFICIENCY, Feat, FeatCategory, FeatureSelection, GameData,
    IMPROVED_UNARMED_STRIKE, MARTIAL_WEAPONS_PROFICIENCY, MythicAbility, MythicFeat, MythicPath,
    OptionRow, ProgressionHeader, Race, SIMPLE_WEAPON_PROFICIENCY, SelectionOption, Skill, Subrace,
    UNARMED_STRIKE, Weapon, WeaponProficiency,
};
use crate::gear::GearItem;
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, DeityKey, FeatKey, FeatureKey, FeatureSelectionKey,
    GearKey, MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey,
    SubraceKey, WeaponKey,
};
use crate::parameter::{ArmourType, Element, Energy, Parameter, ParameterKind, SpellSchool, Stat};
use crate::prerequisite::Prerequisite;
use crate::state::CharacterState;

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
    single_options: FxHashMap<FeatureSelectionKey, SelectionOptionKey>,
    /// Whether each class, and each class taken with each of its archetypes, casts spells of each type.
    casting: Casting,
    /// The prerequisite of each class with an alignment requirement and archetypes, without that requirement:
    /// what an archetype stating its own leaves of the class's (`None`: nothing).
    without_alignment: FxHashMap<ClassKey, Option<Prerequisite>>,
    /// The entries' components the walk evaluates (ticket 295).
    components: Components,
    /// The selections that nest lists of feats ([`GameData::nests_feat_lists`]), whose slot is a Feat slot.
    nests: FxHashSet<FeatureSelectionKey>,
    /// The header each selection naming classes makes ([`ProgressionHeader::of_selection`]), the planner's form until
    /// 293 and 294.
    own_headers: FxHashMap<FeatureSelectionKey, ProgressionHeader>,
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
    pub(super) fn new(data: CheckedGameData) -> Self {
        let mut data = data.into_data();
        let components = Components::compile(&mut data);
        let data = data.canonical();
        let without_alignment = data
            .classes
            .iter()
            .filter(|class| data.archetypes_of(&class.key).next().is_some())
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
            single_options: FxHashMap::default(),
            casting: Casting::new(&data),
            without_alignment,
            components,
            nests: FxHashSet::default(),
            own_headers: data
                .selections
                .iter()
                .filter_map(|selection| {
                    Some((
                        selection.key.clone(),
                        ProgressionHeader::of_selection(selection)?,
                    ))
                })
                .collect(),
            data,
        };
        catalog.nests = (catalog.data.selections.iter())
            .filter(|selection| catalog.find_nest(selection))
            .map(|selection| selection.key.clone())
            .collect();
        catalog.single_options = catalog
            .indexes
            .selections
            .keys()
            .filter_map(|key| Some((key.clone(), catalog.find_single_option(key)?)))
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
        let &a = self.indexes.archetypes.get(key)?;
        Some(&self.data.archetypes[a])
    }

    /// A base class's archetypes, in key order.
    pub(super) fn archetypes_of(&self, class: &ClassKey) -> impl Iterator<Item = &Archetype> {
        (self
            .indexes
            .class_archetypes
            .get(class)
            .into_iter()
            .flatten())
        .map(|&a| &self.data.archetypes[a])
    }

    /// A race's subraces, in key order.
    pub(super) fn subraces_of(&self, race: &RaceKey) -> impl Iterator<Item = &Subrace> {
        (self.indexes.race_subraces.get(race).into_iter().flatten())
            .map(|&s| &self.data.subraces[s])
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
        Some((&self.data.races[r], &self.data.subraces[s]))
    }

    /// A Build start's subrace when it is one of its race's: what then stands in place of the race (its ability
    /// modifiers, weapons, bonuses and components).
    pub(super) fn own_subrace<'s>(&self, start: &'s BuildStart) -> Option<&'s SubraceKey> {
        start.subrace.as_ref().filter(|key| {
            self.subrace(key)
                .is_some_and(|(owner, _)| start.race.as_ref() == Some(&owner.key))
        })
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

    /// A Build start's weapon familiarity: the weapons it's proficient with and the exotic ones it treats as martial,
    /// its subrace's in place of the race's, when the subrace is the race's. None without a race the data has.
    pub(super) fn racial_weapons(
        &self,
        start: &BuildStart,
    ) -> Option<(&BTreeSet<WeaponKey>, &BTreeSet<WeaponKey>)> {
        let race = start.race.as_ref().and_then(|key| self.race(key))?;
        Some(
            match start.subrace.as_ref().and_then(|key| self.subrace(key)) {
                Some((owner, subrace)) if owner.key == race.key => {
                    (&subrace.weapons, &subrace.weapons_as_martial)
                }
                _ => (&race.weapons, &race.weapons_as_martial),
            },
        )
    }

    /// The skill points a Build start's race adds at every level: its subrace's in place of the race's, when the
    /// subrace is the race's. 0 without a race the data has.
    pub(super) fn racial_skill_points(&self, start: &BuildStart) -> u8 {
        let Some(race) = start.race.as_ref().and_then(|key| self.race(key)) else {
            return 0;
        };
        match start.subrace.as_ref().and_then(|key| self.subrace(key)) {
            Some((owner, subrace)) if owner.key == race.key => subrace.skill_points,
            _ => race.skill_points,
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

    pub(super) fn gear_item(&self, key: &GearKey) -> Option<&GearItem> {
        self.indexes
            .gear
            .get(key)
            .map(|&index| &self.data.gear[index])
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
            ParameterKind::Metamagic => self
                .data
                .feats
                .iter()
                .filter(|feat| feat.is_metamagic())
                .map(|feat| Parameter::Metamagic(feat.key.clone()))
                .collect(),
            ParameterKind::Stat => Stat::ALL.map(Parameter::Stat).to_vec(),
            ParameterKind::Element => Element::ALL.map(Parameter::Element).to_vec(),
        }
    }

    /// Whether a pick's value fits what it takes (`kind`: `None` when it takes none): a known value of that kind,
    /// or none for a pick that takes none or whose value may stay open ([`ParameterKind::may_stay_open`]).
    pub(super) fn value_fits(
        &self,
        kind: Option<ParameterKind>,
        value: Option<&Parameter>,
    ) -> bool {
        use ParameterKind as K;
        match (kind, value) {
            (None, None) => true,
            (Some(kind), None) => kind.may_stay_open(),
            (Some(K::Skill), Some(Parameter::Skill(key))) => self.indexes.skills.contains_key(key),
            (Some(K::Metamagic), Some(Parameter::Metamagic(key))) => {
                self.feat(key).is_some_and(Feat::is_metamagic)
            }
            (Some(kind @ (K::Weapon | K::ExoticWeapon)), Some(Parameter::Weapon(key))) => {
                self.weapon(key).is_some_and(|weapon| {
                    kind == K::Weapon || weapon.proficiency == Some(WeaponProficiency::Exotic)
                })
            }
            (Some(K::SpellSchool), Some(Parameter::SpellSchool(_)))
            | (Some(K::Energy), Some(Parameter::Energy(_)))
            | (Some(K::ArmourType), Some(Parameter::ArmourType(_)))
            | (Some(K::Stat), Some(Parameter::Stat(_)))
            | (Some(K::Element), Some(Parameter::Element(_))) => true,
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

    pub(super) fn selection(&self, key: &FeatureSelectionKey) -> Option<&FeatureSelection> {
        Some(&self.data.selections[*self.indexes.selections.get(key)?])
    }

    /// The kind of slot a selection makes: a Feat slot for a selection of feats ([`GameData::offers_feats`]),
    /// a mythic ability slot for one of mythic abilities, a Feature selection slot for any other (ticket 302).
    pub(super) fn slot_kind(&self, selection: &FeatureSelection) -> SlotKind {
        let key = selection.key.clone();
        if self.nests.contains(&key) || !selection.feats.is_empty() {
            SlotKind::Feat(key)
        } else if selection.offers_mythic_abilities() {
            SlotKind::MythicAbilities(key)
        } else {
            SlotKind::Selection(key)
        }
    }

    /// The band of the slot a selection makes, as [`band`](Self::band) gives it for its [`slot_kind`](Self::slot_kind)
    /// without building one.
    pub(super) fn slot_band(&self, selection: &FeatureSelection) -> Band {
        if self.offers_feats(&selection.key) {
            Band::Features
        } else {
            selection.band
        }
    }

    /// Whether a row's selection is one of feats, whose slot is a Feat slot ([`GameData::offers_feats`]);
    /// `false` for one the data lacks.
    pub(super) fn offers_feats(&self, key: &FeatureSelectionKey) -> bool {
        self.nests.contains(key)
            || self
                .selection(key)
                .is_some_and(|selection| !selection.feats.is_empty())
    }

    /// Whether a selection nests lists of feats ([`GameData::nests_feat_lists`]), found once by [`Catalog::new`].
    fn find_nest(&self, selection: &FeatureSelection) -> bool {
        selection.nests_feat_lists_in(|key| self.option(key), |key| self.selection(key))
    }

    /// Whether a selection nests lists of feats ([`GameData::nests_feat_lists`]): its slot offers the feats of the
    /// nested lists whose options' prerequisites hold.
    pub(super) fn nests(&self, key: &FeatureSelectionKey) -> bool {
        self.nests.contains(key)
    }

    /// The header that makes the level of `option` held through `selection` (ticket 304): the option's own, where the
    /// selection takes it ([`FeatureSelection::takes_header_of`]), else the one the selection's
    /// [`option_levels`](FeatureSelection::option_levels) make. `None`: neither, and class levels make no option level.
    pub(super) fn level_header<'s>(
        &'s self,
        selection: &FeatureSelection,
        option: &'s SelectionOption,
    ) -> Option<&'s ProgressionHeader> {
        if selection.takes_header_of(option) {
            option.header.as_ref()
        } else {
            self.own_headers.get(&selection.key)
        }
    }

    /// A row's selections split by the slot each makes, each part in row order: the selections of feats (Feat slots),
    /// then the others (Feature selection slots). A row numbers each part's slots apart (ticket 301), so a link's slot
    /// ids are the ones the feat slots had; every grantor's row splits here.
    pub(super) fn split_selections<'k>(
        &self,
        keys: impl IntoIterator<Item = &'k FeatureSelectionKey>,
    ) -> (Vec<&'k FeatureSelectionKey>, Vec<&'k FeatureSelectionKey>) {
        keys.into_iter().partition(|key| self.offers_feats(key))
    }

    /// The band a slot's pick comes in among its level-up's picks ([`Band`]): a selection's own; a mythic ability's
    /// or feat's, [`Band::MythicAbility`]; a feat slot's, [`Band::Features`].
    pub(super) fn band(&self, kind: &SlotKind) -> Band {
        match kind {
            SlotKind::Selection(key) | SlotKind::MythicAbilities(key) => self.selection_band(key),
            SlotKind::MythicAbility | SlotKind::MythicFeat => Band::MythicAbility,
            SlotKind::Feat(_) => Band::Features,
        }
    }

    /// A selection's band ([`FeatureSelection::band`]); [`Band::Features`] for one the data lacks.
    pub(super) fn selection_band(&self, key: &FeatureSelectionKey) -> Band {
        self.selection(key)
            .map_or(Band::Features, |selection| selection.band)
    }

    /// The options that set the base attack bonus by skill ranks (Athletics III), with their rule.
    pub(super) fn bab_options(&self) -> &[(SelectionOptionKey, BabFromRanks)] {
        &self.indexes.bab_options
    }

    pub(super) fn option(&self, key: &SelectionOptionKey) -> Option<&SelectionOption> {
        let &(index, _) = self.indexes.options.get(key)?;
        Some(&self.data.options[index])
    }

    /// Whether a selection's slot offers `option` by the game data alone: one of its own options (the one
    /// [`FeatureSelection::offers`] would find by a scan, the index saying which selection defines each option) or one
    /// it shares. A filtered pool's picker lists fewer, by what's held: see `offered_now`.
    pub(super) fn offers(&self, selection: &FeatureSelection, option: &SelectionOptionKey) -> bool {
        selection.shared_options.contains(option)
            || self
                .indexes
                .options
                .get(option)
                .is_some_and(|(_, defined_by)| defined_by.as_ref() == Some(&selection.key))
    }

    /// The one option a selection offers, when it offers exactly one: then it's granted with no pick.
    pub(super) fn single_option(
        &self,
        selection: &FeatureSelectionKey,
    ) -> Option<&SelectionOptionKey> {
        self.single_options.get(selection)
    }

    /// The Feature selection that defines an option.
    pub(super) fn selection_of(&self, option: &SelectionOptionKey) -> Option<&FeatureSelection> {
        let (_, selection) = self.indexes.options.get(option)?;
        self.selection(selection.as_ref()?)
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
            Pick::MythicAbility { ability, .. } => {
                let ability = self.mythic_ability(ability)?;
                PickItem {
                    name: &ability.name,
                    category: ability.group.name(),
                    summary: &ability.summary,
                    prerequisite: ability.prerequisite.as_ref(),
                    parameter: ability.parameter,
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
        let archetype = self.archetype(archetype?)?;
        (archetype.class == class.key).then_some(archetype)
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

    /// Whether a feature is a daily choice.
    pub(super) fn is_daily_choice(&self, feature: &FeatureKey) -> bool {
        self.features.daily_choices.contains(feature)
    }

    /// Whether a feature grants the deity's favoured weapons (Cleric Proficiencies).
    pub(super) fn grants_favoured_weapons(&self, feature: &FeatureKey) -> bool {
        self.features.favoured_weapons.contains(feature)
    }

    /// The weapons a feature makes the character proficient with while it's held; none for most.
    pub(super) fn feature_weapons(&self, feature: &FeatureKey) -> Option<&BTreeSet<WeaponKey>> {
        self.features.weapons.get(feature)
    }

    /// Whether a character is proficient with a weapon (ticket 198): one on no proficiency list (the unarmed strike,
    /// rays, natural attacks) always; a simple or martial one while holding that list's feat; an exotic one while
    /// holding Exotic Weapon Proficiency for it, or Martial Weapons Proficiency when the race treats it as martial;
    /// and any a held feature, the race's weapon familiarity (the subrace's in its place), the background or a pick
    /// ([`CharacterState::weapon_proficiencies`]: the Sword Saint's chosen weapon) names, with the deity's favoured
    /// weapons while a held feature grants them. `start` is the Build's and `state` a point
    /// of it.
    pub(super) fn proficient(
        &self,
        start: &BuildStart,
        state: &CharacterState,
        weapon: &Weapon,
    ) -> bool {
        let key = &weapon.key;
        let martial = || state.feat_count(&FeatKey::new(MARTIAL_WEAPONS_PROFICIENCY)) > 0;
        let listed = match weapon.proficiency {
            None => true,
            Some(WeaponProficiency::Simple) => {
                state.feat_count(&FeatKey::new(SIMPLE_WEAPON_PROFICIENCY)) > 0
            }
            Some(WeaponProficiency::Martial) => martial(),
            Some(WeaponProficiency::Exotic) => {
                state.has_feat_for(
                    &FeatKey::new(EXOTIC_WEAPON_PROFICIENCY),
                    &Parameter::Weapon(key.clone()),
                ) || (martial()
                    && self
                        .racial_weapons(start)
                        .is_some_and(|(_, as_martial)| as_martial.contains(key)))
            }
        };
        listed
            || state.weapon_proficiencies.contains_key(key)
            || (self.racial_weapons(start)).is_some_and(|(weapons, _)| weapons.contains(key))
            || (start.background.as_ref())
                .and_then(|background| self.background(background))
                .is_some_and(|background| background.weapons.contains(key))
            || (state.features())
                .filter_map(|(feature, _)| self.feature_weapons(feature))
                .any(|weapons| weapons.contains(key))
            || state
                .deity()
                .and_then(|deity| self.deity(deity))
                .is_some_and(|deity| {
                    deity.favoured_weapons.contains(key)
                        && (self.features.favoured_weapons.iter())
                            .any(|feature| state.feature_count(feature) > 0)
                })
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

    /// What grants a feature by its rows (ticket 244): the classes, then the archetypes, the paths and the options,
    /// each in key order. Empty for a feature only a mythic ability, a path switch or a selection gives.
    pub(super) fn feature_holders(&self, feature: &FeatureKey) -> &[GrantedBy] {
        self.features
            .holders
            .get(feature)
            .map_or(&[], Vec::as_slice)
    }

    /// What names a Feature selection in its rows, as [`feature_holders`](Self::feature_holders) for a feature.
    pub(super) fn selection_holders(&self, selection: &FeatureSelectionKey) -> &[GrantedBy] {
        self.features
            .selection_holders
            .get(selection)
            .map_or(&[], Vec::as_slice)
    }
}

/// The game data's positions by key, built once by [`Catalog::new`].
#[derive(Clone, Debug)]
struct Indexes {
    classes: FxHashMap<ClassKey, ClassAt>,
    archetypes: FxHashMap<ArchetypeKey, usize>,
    /// Each base class's archetypes' positions, in key order.
    class_archetypes: FxHashMap<ClassKey, Vec<usize>>,
    /// Each race's subraces' positions, in key order.
    race_subraces: FxHashMap<RaceKey, Vec<usize>>,
    races: FxHashMap<RaceKey, usize>,
    /// Each subrace's position, with its race's.
    subraces: FxHashMap<SubraceKey, (usize, usize)>,
    skills: FxHashMap<SkillKey, usize>,
    weapons: FxHashMap<WeaponKey, usize>,
    feats: FxHashMap<FeatKey, usize>,
    selections: FxHashMap<FeatureSelectionKey, usize>,
    /// Each option's position, with the selection that defines it ([`FeatureSelection::options`]).
    options: FxHashMap<SelectionOptionKey, (usize, Option<FeatureSelectionKey>)>,
    /// The options that set the base attack bonus by skill ranks (Athletics III), few and in key order.
    bab_options: Vec<(SelectionOptionKey, BabFromRanks)>,
    mythic_abilities: FxHashMap<MythicAbilityKey, usize>,
    mythic_feats: FxHashMap<MythicFeatKey, usize>,
    paths: FxHashMap<MythicPathKey, usize>,
    backgrounds: FxHashMap<BackgroundKey, usize>,
    deities: FxHashMap<DeityKey, usize>,
    gear: FxHashMap<GearKey, usize>,
}

impl Indexes {
    fn new(data: &GameData) -> Self {
        let races = index(&data.races, |r| &r.key);
        let subraces = data
            .subraces
            .iter()
            .enumerate()
            .filter_map(|(s, subrace)| Some((subrace.key.clone(), (*races.get(&subrace.race)?, s))))
            .collect();
        let mut bab_options: Vec<_> = data
            .options
            .iter()
            .filter_map(|option| Some((option.key.clone(), option.bab_from_ranks.clone()?)))
            .collect();
        bab_options.sort_by(|a, b| a.0.cmp(&b.0));
        let mut class_archetypes: FxHashMap<ClassKey, Vec<usize>> = FxHashMap::default();
        for (a, archetype) in data.archetypes.iter().enumerate() {
            class_archetypes
                .entry(archetype.class.clone())
                .or_default()
                .push(a);
        }
        let mut race_subraces: FxHashMap<RaceKey, Vec<usize>> = FxHashMap::default();
        for (s, subrace) in data.subraces.iter().enumerate() {
            race_subraces
                .entry(subrace.race.clone())
                .or_default()
                .push(s);
        }
        Self {
            class_archetypes,
            race_subraces,
            classes: index_classes(data),
            archetypes: index(&data.archetypes, |a| &a.key),
            selections: index(&data.selections, |s| &s.key),
            options: index_options(data),
            bab_options,
            skills: index(&data.skills, |s| &s.key),
            weapons: index(&data.weapons, |w| &w.key),
            feats: index(&data.feats, |f| &f.key),
            mythic_abilities: index(&data.mythic_abilities, |a| &a.key),
            mythic_feats: index(&data.mythic_feats, |f| &f.key),
            paths: index(&data.mythic_paths, |p| &p.key),
            backgrounds: index(&data.backgrounds, |b| &b.key),
            deities: index(&data.deities, |d| &d.key),
            gear: index(&data.gear, |item| &item.key),
            races,
            subraces,
        }
    }
}

/// Each item's position in a list, by its key.
fn index<K: std::hash::Hash + Eq + Clone, T>(
    items: &[T],
    key: impl Fn(&T) -> &K,
) -> FxHashMap<K, usize> {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| (key(item).clone(), i))
        .collect()
}

/// The base and prestige classes by key.
fn index_classes(data: &GameData) -> FxHashMap<ClassKey, ClassAt> {
    let base = data.classes.iter().enumerate();
    let prestige = data.prestige_classes.iter().enumerate();
    base.map(|(c, class)| (class.key.clone(), ClassAt::Base(c)))
        .chain(prestige.map(|(c, class)| (class.key.clone(), ClassAt::Prestige(c))))
        .collect()
}

/// Every option by key, with its position and the selection that defines it.
fn index_options(
    data: &GameData,
) -> FxHashMap<SelectionOptionKey, (usize, Option<FeatureSelectionKey>)> {
    let defined_by: FxHashMap<&SelectionOptionKey, &FeatureSelectionKey> = data
        .selections
        .iter()
        .flat_map(|selection| {
            selection
                .options
                .iter()
                .map(|option| (option, &selection.key))
        })
        .collect();
    data.options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let selection = defined_by.get(&option.key).map(|&key| key.clone());
            (option.key.clone(), (i, selection))
        })
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
    classes: FxHashMap<ClassKey, (bool, bool)>,
    /// Each base class taken with one of its archetypes, by the archetype's key.
    archetypes: FxHashMap<ArchetypeKey, (bool, bool)>,
}

impl Casting {
    fn new(data: &GameData) -> Self {
        let classes = data
            .all_classes()
            .map(|class| (class.key.clone(), casts_with(class, None)))
            .collect();
        let archetypes = data
            .archetypes
            .iter()
            .filter_map(|archetype| {
                let class = data
                    .classes
                    .iter()
                    .find(|class| class.key == archetype.class)?;
                Some((archetype.key.clone(), casts_with(class, Some(archetype))))
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
    /// The option row of every feature that grants one (Bloodline Ascendance).
    rows: FxHashMap<FeatureKey, OptionRow>,
    /// Every feature that is a daily choice.
    daily_choices: FxHashSet<FeatureKey>,
    /// Every feature that grants the deity's favoured weapons (Cleric Proficiencies).
    favoured_weapons: FxHashSet<FeatureKey>,
    /// The weapons of every feature that makes the character proficient with some (Wizard Proficiencies).
    weapons: FxHashMap<FeatureKey, BTreeSet<WeaponKey>>,
    /// The name of every feature a class, archetype or path defines, for the sheet's labels.
    names: FxHashMap<FeatureKey, String>,
    /// What grants each feature by its rows ([`Catalog::feature_holders`]).
    holders: FxHashMap<FeatureKey, Vec<GrantedBy>>,
    /// What names each selection in its rows ([`Catalog::selection_holders`]).
    selection_holders: FxHashMap<FeatureSelectionKey, Vec<GrantedBy>>,
    /// The features whose name another feature shares (the Fighter's and the Sohei's Weapon Training), so a reason
    /// can tell them apart.
    shared_names: FxHashSet<FeatureKey>,
}

impl FeatureFacts {
    fn new(data: &GameData) -> Self {
        let features: Vec<&crate::game_data::Feature> = data.features.iter().collect();
        let (holders, selection_holders) = holders(data);
        let mut name_counts: FxHashMap<&str, usize> = FxHashMap::default();
        for feature in &features {
            *name_counts.entry(feature.name.as_str()).or_default() += 1;
        }
        let shared_names = features
            .iter()
            .filter(|feature| name_counts[feature.name.as_str()] > 1)
            .map(|feature| feature.key.clone())
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
        let names = features
            .iter()
            .map(|feature| (feature.key.clone(), feature.name.clone()))
            .collect();
        let favoured_weapons = features
            .iter()
            .filter(|feature| feature.favoured_weapon_proficiency)
            .map(|feature| feature.key.clone())
            .collect();
        let weapons = features
            .iter()
            .filter(|feature| !feature.weapons.is_empty())
            .map(|feature| (feature.key.clone(), feature.weapons.clone()))
            .collect();
        Self {
            rows,
            daily_choices,
            favoured_weapons,
            weapons,
            names,
            shared_names,
            holders,
            selection_holders,
        }
    }
}

/// What grants a feature by its rows: the sheet's holder of the feature (ticket 244, decision 7).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum GrantedBy {
    Class(ClassKey),
    Archetype(ArchetypeKey),
    Path(MythicPathKey),
    Option(SelectionOptionKey),
}

/// The holders of the features and of the selections rows name, each in [`GrantedBy`]'s order and once.
type Holders = (
    FxHashMap<FeatureKey, Vec<GrantedBy>>,
    FxHashMap<FeatureSelectionKey, Vec<GrantedBy>>,
);

/// What grants each feature and names each selection by its rows: classes', archetypes', paths' and options'.
fn holders(data: &GameData) -> Holders {
    let mut features: FxHashMap<FeatureKey, BTreeSet<GrantedBy>> = FxHashMap::default();
    let mut selections: FxHashMap<FeatureSelectionKey, BTreeSet<GrantedBy>> = FxHashMap::default();
    let mut add =
        |by: GrantedBy, row_features: &[FeatureKey], row_selections: &[FeatureSelectionKey]| {
            for feature in row_features {
                features
                    .entry(feature.clone())
                    .or_default()
                    .insert(by.clone());
            }
            for selection in row_selections {
                selections
                    .entry(selection.clone())
                    .or_default()
                    .insert(by.clone());
            }
        };
    for class in data.all_classes() {
        for row in &class.levels {
            add(
                GrantedBy::Class(class.key.clone()),
                &row.features,
                &row.selections,
            );
        }
    }
    for archetype in &data.archetypes {
        for row in &archetype.levels {
            add(
                GrantedBy::Archetype(archetype.key.clone()),
                &row.features,
                &row.selections,
            );
        }
    }
    for path in &data.mythic_paths {
        for row in &path.ranks {
            add(
                GrantedBy::Path(path.key.clone()),
                &row.features,
                &row.selections,
            );
        }
    }
    for option in &data.options {
        for row in &option.levels {
            add(
                GrantedBy::Option(option.key.clone()),
                &row.features,
                &row.selections,
            );
        }
    }
    let ordered = |by: BTreeSet<GrantedBy>| by.into_iter().collect();
    (
        features
            .into_iter()
            .map(|(k, by)| (k, ordered(by)))
            .collect(),
        selections
            .into_iter()
            .map(|(k, by)| (k, ordered(by)))
            .collect(),
    )
}

/// Where a class is: an index into the base or the prestige classes.
#[derive(Clone, Copy, Debug)]
enum ClassAt {
    Base(usize),
    Prestige(usize),
}
