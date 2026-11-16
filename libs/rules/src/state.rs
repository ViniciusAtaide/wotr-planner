//! What a character has at one point of its Build, and Prerequisite checks against it.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;
use std::sync::Arc;

use crate::bonus::BonusInForce;
use crate::build::{AbilityScores, Faith};
use crate::choice::Unmet;
use crate::component::{Fact, MechanicsFlag, Pool, StatRef, StatRule};
use crate::game_data::{Alignment, OptionGroup, SpellType};
use crate::keys::{
    ArchetypeKey, ClassKey, DeityKey, FeatKey, FeatureKey, FeatureSelectionKey, MythicAbilityKey,
    MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey, WeaponKey,
};
use crate::parameter::Parameter;
use crate::prerequisite::Prerequisite;

/// Everything derived from a Build up to some point: the end of a character level, or just before a slot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CharacterState {
    pub character_level: u8,
    pub race: Option<RaceKey>,
    pub alignment: Option<Alignment>,
    /// The deity followed, or atheism; `None` while not chosen.
    pub faith: Option<Faith>,
    /// The domains the deity followed offers; empty without one.
    pub deity_domains: BTreeSet<SelectionOptionKey>,
    /// One entry per class taken, in order of first entry.
    pub classes: Vec<ClassLevels>,
    pub bab: u8,
    pub fort: u8,
    pub reflex: u8,
    pub will: u8,
    /// The character's ability scores, as every derived number sees them: the base scores with every stat bonus held (an
    /// entry's, Legend's +4, and the always-on bonuses and penalties of the gear the Build holds: ticket 166), every
    /// source's stacked together by type, then the Gold Dragon's floor and highest-score bonus (ticket 297). A level's
    /// skill points read the scores without the gear, and ability prerequisites read
    /// [`permanent_ability_scores`](Self::permanent_ability_scores).
    pub ability_scores: AbilityScores,
    /// The scores ability prerequisites read (ticket 252): as [`ability_scores`](Self::ability_scores), but of the
    /// stat bonuses only the [permanent](crate::Descriptor::is_permanent) ones, a racial one or an item's enhancement:
    /// the game's permanent value.
    pub permanent_ability_scores: AbilityScores,
    /// The scores a full ability prerequisite reads (ticket 295): the [`permanent`](Self::permanent_ability_scores)
    /// ones plus the components' bonuses of descriptor `Feat` (`PrerequisiteFullStatValue.GetFullStatValue`). The
    /// permanent ones for data without components.
    pub full_ability_scores: AbilityScores,
    /// Point buy, racial modifiers and the level increases so far, before any feature's bonus.
    pub base_ability_scores: AbilityScores,
    /// Spell-level access: the highest arcane spell level castable; 0 when none.
    pub arcane_spell_level: u8,
    /// Spell-level access: the highest divine spell level castable; 0 when none.
    pub divine_spell_level: u8,
    /// The highest arcane spell level castable through a class that casts spontaneously; 0 when none.
    pub spontaneous_arcane_spell_level: u8,
    /// The highest divine spell level castable through a class that casts spontaneously; 0 when none.
    pub spontaneous_divine_spell_level: u8,
    /// The highest effective class level among classes that can cast (class levels plus prestige levels that
    /// advance them). Checked only where the guide states a caster level.
    pub caster_level: u8,
    pub skill_ranks: BTreeMap<SkillKey, u8>,
    /// The class skills: each class taken's (an archetype's list where it replaces its class's), the
    /// background's and those of the option rows held (an oracle's mystery, a kineticist's element).
    pub class_skills: BTreeSet<SkillKey>,
    /// The class levels a feat's prerequisites see, for each class another class's levels count as there
    /// (Diverse Training: fighter 1 + Eldritch Knight 3 is fighter 4). Classes left out are their class level.
    pub feat_class_levels: BTreeMap<ClassKey, u8>,
    /// The facts held, one list as the game keeps them (`UnitProgressionData.Features`, a `FeatureCollection`; ticket
    /// 305): each feat, feature, selection option, Feature selection, mythic ability and mythic feat, by its kind and
    /// key, with how many hold it and the values it's held for ([`HeldFact`]). An entry no pick or grant holds any
    /// more is removed. The kinds are read through views that iterate in key order ([`feats`](Self::feats),
    /// [`features`](Self::features), [`options`](Self::options), …) and counts by kind
    /// ([`feat_count`](Self::feat_count), …).
    pub held: BTreeMap<Fact, HeldFact>,
    /// The options held through each selection that gave them, with how many times (Expanded Element: the
    /// elements picked at 7 and 15). An option granted outside a selection's slot isn't here.
    pub options_through: BTreeMap<FeatureSelectionKey, BTreeMap<SelectionOptionKey, u8>>,
    /// The weapons a pick makes the character proficient with, with how many picks hold each: the weapon of a pick
    /// from a list that skips the proficiency check (the Sword Saint's chosen weapon,
    /// [`FeatureSelection::skips_proficiency`](crate::FeatureSelection::skips_proficiency); ticket 251).
    pub weapon_proficiencies: BTreeMap<WeaponKey, u8>,
    /// The group of each option held that belongs to one (a Draconic bloodline).
    pub option_groups: BTreeMap<SelectionOptionKey, OptionGroup>,
    /// The selection that defines each option held ([`FeatureSelection::options`](crate::FeatureSelection::options)),
    /// wherever it was taken.
    pub option_selections: BTreeMap<SelectionOptionKey, FeatureSelectionKey>,
    /// The option level of each option held through a selection with option levels (a sorcerer 5 / Dragon
    /// Disciple 5's bloodline: 10).
    pub option_levels: BTreeMap<SelectionOptionKey, u8>,
    /// While an entry held raises the BAB to at least this plus the Mythic rank
    /// ([`BaseAttackAtLeast`](crate::Component::BaseAttackAtLeast), Perfect Mind: 15), this; `None` otherwise.
    /// [`bab`](Self::bab) already counts it.
    pub bab_at_least_mythic_rank_plus: Option<u8>,
    /// Set while an entry makes every skill's total count at least the Mythic rank for its ranks, and every skill a
    /// class skill ([`GoldenDragonSkills`](crate::Component::GoldenDragonSkills), Perfect Mind). Prerequisites still read
    /// the ranks spent.
    pub skill_ranks_at_least_mythic_rank: bool,
    /// The highest spell level castable from each class taken that has a spellbook (with the prestige levels that
    /// advance it), 0 before its first: what a [`Prerequisite::ClassSpellLevel`] reads. Classes without one are left
    /// out.
    pub class_spell_levels: BTreeMap<ClassKey, u8>,
    /// The spell types of the classes taken that cast spells, whatever level they reach yet: what a
    /// [`Prerequisite::CasterType`] reads.
    pub caster_types: BTreeSet<SpellType>,
    /// The class levels the facts held add to the levels a `ClassLevel` prerequisite reads
    /// ([`Component::PrerequisiteClassLevels`](crate::Component::PrerequisiteClassLevels)), each with the picks it counts for; empty for data without components.
    pub prerequisite_class_levels: Vec<PrerequisiteClassLevels>,
    /// How the facts held change the stats a prerequisite reads ([`Component::PrerequisiteStat`](crate::Component::PrerequisiteStat)), in the order held.
    pub prerequisite_stats: Vec<(StatRef, StatRule)>,
    /// The mechanics flags the facts held set ([`Component::Flag`](crate::Component::Flag)), with how many hold each.
    pub flags: BTreeMap<MechanicsFlag, u8>,
    /// The Mythic rank reached: 3 after a Legend switch, whatever the rank before.
    pub mythic_rank: u8,
    pub mythic_path: Option<MythicPathKey>,
    /// The bonuses in force at the end of the level: the components' of the entries held, in the order
    /// [`BonusInForce`](crate::BonusInForce) gives, then those of the gear the Build holds. What the caster line and the totals add up. The walk records them on the sheet's states ([`Engine::evaluate`]), and levels with the same bonuses
    /// share one list; a walk that keeps no sheet (a picker's or a recheck's) leaves them empty, as does a state
    /// just before a slot. So the two lines take a level of the sheet ([`LevelSheet`]), never a bare state.
    ///
    /// [`Engine::evaluate`]: crate::Engine::evaluate
    /// [`LevelSheet`]: crate::LevelSheet
    pub bonuses_in_force: Arc<[BonusInForce]>,
}

/// One fact held ([`CharacterState::held`]): how many hold it and the values it's held for, the game's `Feature`
/// with its rank and its instance per `Param` (`FeatureCollection.PrepareFactForAttach`). Each kind keeps the
/// planner's own count:
/// - a feat, a feature, a selection option and a Feature selection: `holds` counts the picks and grants that hold
///   it (Extra Hex taken twice: 2; Sneak Attack: 3), and `values` how many hold each value (Weapon Focus: Longsword
///   1, Dagger 1; an option's own value, Finesse Training: Dagger 1, ticket 172);
/// - a mythic ability: `holds` counts the picks without a value (a link from before the ability took one), and
///   `values` the values it's held for, each once (Favorite Metamagic: Persistent Spell and Quicken Spell; ticket
///   249);
/// - a mythic feat: `holds` counts its picks (Extra Feat at two ranks: 2; ticket 303), and `values` the values it's
///   held for, each once (Improved Critical (Mythic): Longsword).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeldFact {
    pub holds: u8,
    pub values: BTreeMap<Parameter, u8>,
}

impl HeldFact {
    /// One more hold of `value`.
    pub(crate) fn count_value(&mut self, value: &Parameter) {
        let holds = self.values.entry(value.clone()).or_default();
        *holds = holds.saturating_add(1);
    }

    /// One hold of `value` fewer; the value goes with its last.
    pub(crate) fn uncount_value(&mut self, value: &Parameter) {
        if let Some(holds) = self.values.get_mut(value) {
            *holds = holds.saturating_sub(1);
            if *holds == 0 {
                self.values.remove(value);
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.holds == 0 && self.values.is_empty()
    }
}

/// A [`Fact`] by reference, to find one in [`CharacterState::held`] without a copy of its key. Its variants are
/// `Fact`'s, in `Fact`'s order, so the two order alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FactRef<'a> {
    Feat(&'a FeatKey),
    Feature(&'a FeatureKey),
    Option(&'a SelectionOptionKey),
    Selection(&'a FeatureSelectionKey),
    MythicAbility(&'a MythicAbilityKey),
    MythicFeat(&'a MythicFeatKey),
    Class(&'a ClassKey),
    Archetype(&'a ArchetypeKey),
    Race(&'a RaceKey),
    Deity(&'a DeityKey),
}

/// What a key of [`CharacterState::held`] is looked up by: a [`Fact`], or a [`FactRef`] to one.
pub(crate) trait AsFactRef {
    fn fact_ref(&self) -> FactRef<'_>;
}

impl AsFactRef for Fact {
    fn fact_ref(&self) -> FactRef<'_> {
        match self {
            Self::Feat(key) => FactRef::Feat(key),
            Self::Feature(key) => FactRef::Feature(key),
            Self::Option(key) => FactRef::Option(key),
            Self::Selection(key) => FactRef::Selection(key),
            Self::MythicAbility(key) => FactRef::MythicAbility(key),
            Self::MythicFeat(key) => FactRef::MythicFeat(key),
            Self::Class(key) => FactRef::Class(key),
            Self::Archetype(key) => FactRef::Archetype(key),
            Self::Race(key) => FactRef::Race(key),
            Self::Deity(key) => FactRef::Deity(key),
        }
    }
}

impl AsFactRef for FactRef<'_> {
    fn fact_ref(&self) -> FactRef<'_> {
        *self
    }
}

impl<'a> Borrow<dyn AsFactRef + 'a> for Fact {
    fn borrow(&self) -> &(dyn AsFactRef + 'a) {
        self
    }
}

impl PartialEq for dyn AsFactRef + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.fact_ref() == other.fact_ref()
    }
}

impl Eq for dyn AsFactRef + '_ {}

impl PartialOrd for dyn AsFactRef + '_ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn AsFactRef + '_ {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fact_ref().cmp(&other.fact_ref())
    }
}

/// The key of a kind [`CharacterState::held`] holds, for the views of one kind.
trait HeldKey: Sized + 'static {
    /// The kind's first place in the map's order, where a view of it starts: the kind with the empty key.
    fn first() -> FactRef<'static>;

    /// The key of a fact of this kind; `None` for another kind.
    fn of(fact: &Fact) -> Option<&Self>;
}

macro_rules! held_key {
    ($($key:ident => $variant:ident),* $(,)?) => {$(
        impl HeldKey for $key {
            fn first() -> FactRef<'static> {
                static FIRST: $key = $key(String::new());
                FactRef::$variant(&FIRST)
            }

            fn of(fact: &Fact) -> Option<&Self> {
                match fact {
                    Fact::$variant(key) => Some(key),
                    _ => None,
                }
            }
        }
    )*};
}

held_key!(
    FeatKey => Feat,
    FeatureKey => Feature,
    SelectionOptionKey => Option,
    FeatureSelectionKey => Selection,
    MythicAbilityKey => MythicAbility,
    MythicFeatKey => MythicFeat,
);

/// Class levels a held fact adds to the levels a `ClassLevel` prerequisite reads of `class`
/// ([`Component::PrerequisiteClassLevels`](crate::Component::PrerequisiteClassLevels)).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrerequisiteClassLevels {
    pub class: ClassKey,
    pub levels: i16,
    /// The picks it counts for; `None`: every pick.
    pub only_for: Option<Pool>,
}

/// The levels taken in one class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassLevels {
    pub class: ClassKey,
    pub prestige: bool,
    /// From [`Build::class_choices`](crate::Build::class_choices).
    pub archetype: Option<ArchetypeKey>,
    /// Prestige classes: the classes whose spellcasting this one advances.
    pub advances: BTreeSet<ClassKey>,
    pub level: u8,
}

impl CharacterState {
    /// The deity followed, if any.
    pub fn deity(&self) -> Option<&DeityKey> {
        self.faith.as_ref().and_then(Faith::deity)
    }

    /// Class level in one class; 0 when not taken.
    pub fn class_level(&self, class: &ClassKey) -> u8 {
        self.classes
            .iter()
            .find(|c| &c.class == class)
            .map_or(0, |c| c.level)
    }

    pub fn skill_ranks(&self, skill: &SkillKey) -> u8 {
        self.skill_ranks.get(skill).copied().unwrap_or(0)
    }

    /// The ranks a skill's total counts: the ranks, or the Mythic rank under Perfect Mind when higher. A skill-rank
    /// prerequisite reads [`skill_ranks`](Self::skill_ranks) instead.
    pub fn skill_ranks_in_total(&self, skill: &SkillKey) -> u8 {
        let floor = if self.skill_ranks_at_least_mythic_rank {
            self.mythic_rank
        } else {
            0
        };
        self.skill_ranks(skill).max(floor)
    }

    /// The entry held for a fact; `None` when nothing holds it.
    pub(crate) fn held_fact(&self, fact: FactRef<'_>) -> Option<&HeldFact> {
        self.held.get(&fact as &dyn AsFactRef)
    }

    /// Changes the entry held for a fact with `change`, if one is, and removes it once nothing holds it.
    pub(crate) fn release_fact(&mut self, fact: FactRef<'_>, change: impl FnOnce(&mut HeldFact)) {
        let key = &fact as &dyn AsFactRef;
        if let Some(held) = self.held.get_mut(key) {
            change(held);
            if held.is_empty() {
                self.held.remove(key);
            }
        }
    }

    /// The entries held of one kind, in key order.
    fn kind<K: HeldKey>(&self) -> impl Iterator<Item = (&K, &HeldFact)> {
        let first = K::first();
        self.held
            .range::<dyn AsFactRef, _>((
                Bound::Included(&first as &dyn AsFactRef),
                Bound::Unbounded,
            ))
            .map_while(|(fact, held)| K::of(fact).map(|key| (key, held)))
    }

    /// The entries of one kind that `holds` counts, with it, in key order.
    fn counted<K: HeldKey>(&self) -> impl Iterator<Item = (&K, u8)> {
        (self.kind())
            .filter(|(_, held)| held.holds > 0)
            .map(|(key, held)| (key, held.holds))
    }

    /// The entries of one kind held for values, with them, in key order.
    fn valued<K: HeldKey>(&self) -> impl Iterator<Item = (&K, &BTreeMap<Parameter, u8>)> {
        (self.kind())
            .filter(|(_, held)| !held.values.is_empty())
            .map(|(key, held)| (key, &held.values))
    }

    /// Each feat held, with how many picks and grants hold it (Extra Hex taken twice: 2), in key order. A class
    /// grant and a pick may both hold one, so losing one of them keeps the feat.
    pub fn feats(&self) -> impl Iterator<Item = (&FeatKey, u8)> {
        self.counted()
    }

    /// Each parameterised feat held, with the values it's held for and how many picks and grants hold each (Weapon
    /// Focus: Longsword 1, Dagger 1), in key order.
    pub fn feat_values(&self) -> impl Iterator<Item = (&FeatKey, &BTreeMap<Parameter, u8>)> {
        self.valued()
    }

    /// Each feature held, with the number of times it has been granted ("Sneak Attack: 3"), in key order.
    pub fn features(&self) -> impl Iterator<Item = (&FeatureKey, u8)> {
        self.counted()
    }

    /// Each selection option held, with how many times it was taken (Increased Damage Reduction: 2), in key order.
    pub fn options(&self) -> impl Iterator<Item = (&SelectionOptionKey, u8)> {
        self.counted()
    }

    /// Each option held whose pick names a value of the option's own (not a chosen feat's, which
    /// [`feat_values`](Self::feat_values) gives), with the values and how many picks hold each (Finesse Training:
    /// Dagger 1, Light Shield Bash 1; ticket 172), in key order.
    pub fn option_values(
        &self,
    ) -> impl Iterator<Item = (&SelectionOptionKey, &BTreeMap<Parameter, u8>)> {
        self.valued()
    }

    /// Each Feature selection held, with how many grants hold it, whatever their slots hold (ticket 247), in key
    /// order: what [`Prerequisite::Selection`] reads.
    pub fn selections(&self) -> impl Iterator<Item = (&FeatureSelectionKey, u8)> {
        self.counted()
    }

    /// Each mythic ability held, in key order.
    pub fn mythic_abilities(&self) -> impl Iterator<Item = &MythicAbilityKey> {
        self.kind().map(|(key, _)| key)
    }

    /// Each mythic feat held, in key order.
    pub fn mythic_feats(&self) -> impl Iterator<Item = &MythicFeatKey> {
        self.counted().map(|(key, _)| key)
    }

    /// Whether a mythic ability that takes a value is held for this one.
    pub fn has_mythic_ability_for(&self, ability: &MythicAbilityKey, value: &Parameter) -> bool {
        self.held_fact(FactRef::MythicAbility(ability))
            .is_some_and(|held| held.values.contains_key(value))
    }

    /// How many picks hold a mythic ability: one for each value it's held for, and each pick without one; 0 when it
    /// isn't held.
    pub fn mythic_ability_count(&self, ability: &MythicAbilityKey) -> u8 {
        self.held_fact(FactRef::MythicAbility(ability))
            .map_or(0, |held| {
                u8::try_from(held.values.len())
                    .unwrap_or(u8::MAX)
                    .saturating_add(held.holds)
            })
    }

    /// How many picks hold a mythic feat; 0 when none.
    pub fn mythic_feat_count(&self, feat: &MythicFeatKey) -> u8 {
        self.holds(FactRef::MythicFeat(feat))
    }

    /// The values a parameterised mythic feat is held for, each once; `None` when it's held for none.
    pub fn mythic_feat_values_of(&self, feat: &MythicFeatKey) -> Option<&BTreeMap<Parameter, u8>> {
        self.held_fact(FactRef::MythicFeat(feat))
            .map(|held| &held.values)
            .filter(|values| !values.is_empty())
    }

    /// Whether a parameterised mythic feat is held for a value.
    pub fn has_mythic_feat_for(&self, feat: &MythicFeatKey, value: &Parameter) -> bool {
        self.held_fact(FactRef::MythicFeat(feat))
            .is_some_and(|held| held.values.contains_key(value))
    }

    /// How many times a feature has been granted; 0 when never.
    pub fn feature_count(&self, feature: &FeatureKey) -> u8 {
        self.holds(FactRef::Feature(feature))
    }

    /// How many grants hold a Feature selection; 0 when none.
    pub fn selection_count(&self, selection: &FeatureSelectionKey) -> u8 {
        self.holds(FactRef::Selection(selection))
    }

    /// How many picks and grants hold a fact (its entry's `holds`); 0 when none.
    fn holds(&self, fact: FactRef<'_>) -> u8 {
        self.held_fact(fact).map_or(0, |held| held.holds)
    }

    pub fn spell_level(&self, spell_type: SpellType) -> u8 {
        match spell_type {
            SpellType::Arcane => self.arcane_spell_level,
            SpellType::Divine => self.divine_spell_level,
        }
    }

    /// Spell-level access through spontaneous casting.
    pub fn spontaneous_spell_level(&self, spell_type: SpellType) -> u8 {
        match spell_type {
            SpellType::Arcane => self.spontaneous_arcane_spell_level,
            SpellType::Divine => self.spontaneous_divine_spell_level,
        }
    }

    /// The option level of a held option; 0 when it isn't held through a selection with option levels.
    pub fn option_level(&self, option: &SelectionOptionKey) -> u8 {
        self.option_levels.get(option).copied().unwrap_or(0)
    }

    /// Whether any option of this selection is held (its own options, wherever they were taken).
    pub fn holds_option_of(&self, selection: &FeatureSelectionKey) -> bool {
        self.option_selections
            .values()
            .any(|defined| defined == selection)
    }

    /// How many picks and grants hold a feat; 0 when none.
    pub fn feat_count(&self, feat: &FeatKey) -> u8 {
        self.holds(FactRef::Feat(feat))
    }

    /// Whether a pick or a grant holds a feat.
    pub fn has_feat(&self, feat: &FeatKey) -> bool {
        self.feat_count(feat) > 0
    }

    /// Whether a parameterised feat is held for a value.
    pub fn has_feat_for(&self, feat: &FeatKey, value: &Parameter) -> bool {
        self.held_fact(FactRef::Feat(feat))
            .is_some_and(|held| held.values.contains_key(value))
    }

    /// The values a parameterised feat is held for, with how many picks and grants hold each; `None` when it's held
    /// for none.
    pub fn feat_values_of(&self, feat: &FeatKey) -> Option<&BTreeMap<Parameter, u8>> {
        self.held_fact(FactRef::Feat(feat))
            .map(|held| &held.values)
            .filter(|values| !values.is_empty())
    }

    /// How many times a selection option was taken; 0 when never.
    pub fn option_count(&self, option: &SelectionOptionKey) -> u8 {
        self.holds(FactRef::Option(option))
    }

    /// Whether an option that takes a value of its own is held for a value (Finesse Training for the dagger).
    pub fn has_option_for(&self, option: &SelectionOptionKey, value: &Parameter) -> bool {
        self.held_fact(FactRef::Option(option))
            .is_some_and(|held| held.values.contains_key(value))
    }

    /// The class level a feat's prerequisites see: the class level plus the levels of classes that count as it
    /// for feats.
    pub fn feat_class_level(&self, class: &ClassKey) -> u8 {
        self.feat_class_levels
            .get(class)
            .copied()
            .unwrap_or_else(|| self.class_level(class))
    }

    /// The class level a `ClassLevel` prerequisite reads for the pick being checked: `has` plus the levels the facts
    /// held add to it for that pick (`PrerequisiteClassLevel.GetClassLevel`: a `ClassLevelsForPrerequisites` counts
    /// for the picks of its `ForSelection`, or for every pick without one).
    fn prerequisite_class_level(&self, class: &ClassKey, has: u8, picking: &Picking<'_>) -> u8 {
        let added: i16 = self
            .prerequisite_class_levels
            .iter()
            .filter(|added| &added.class == class)
            .filter(|added| match &added.only_for {
                None => true,
                Some(Pool::Feats) => picking.for_feat,
                Some(Pool::Options(selection)) => picking.selection == Some(selection),
            })
            .map(|added| added.levels)
            .sum();
        u8::try_from((i16::from(has) + added).max(0)).unwrap_or(u8::MAX)
    }

    /// The value a prerequisite reads of a stat whose own value is `has`: changed by each fact held that replaces it
    /// for prerequisites, in the order held, a replacing ability read in `scores` (`PrerequisiteStatValue.GetStatValue`,
    /// `PrerequisiteFullStatValue.GetFullStatValue`, `ReplaceStatForPrerequisites.ResultStat`).
    fn prerequisite_stat(
        &self,
        is_stat: impl Fn(&StatRef) -> bool,
        has: u8,
        scores: &AbilityScores,
    ) -> u8 {
        if self.prerequisite_stats.is_empty() {
            return has;
        }
        let value = self
            .prerequisite_stats
            .iter()
            .filter(|(of, _)| is_stat(of))
            .fold(i16::from(has), |value, (_, rule)| {
                rule.apply(value, self, scores)
            });
        u8::try_from(value.max(0)).unwrap_or(u8::MAX)
    }

    /// Whether an option of `group` other than `option` is held.
    pub fn holds_other_option_of(
        &self,
        group: OptionGroup,
        option: Option<&SelectionOptionKey>,
    ) -> bool {
        self.option_groups
            .iter()
            .any(|(held, held_group)| *held_group == group && Some(held) != option)
    }

    /// Whether `option` is held through `selection`'s slots.
    pub fn holds_through(
        &self,
        selection: &FeatureSelectionKey,
        option: &SelectionOptionKey,
    ) -> bool {
        self.options_through
            .get(selection)
            .is_some_and(|options| options.contains_key(option))
    }

    /// Evaluates a Prerequisite against this state, for a pick with no value: a `FeatForSameValue` fails.
    pub fn check(&self, prerequisite: &Prerequisite) -> Result<(), Unmet> {
        self.check_for(prerequisite, None)
    }

    /// Evaluates a Prerequisite against this state for a pick taken for `value` (`FeatForSameValue` compares with
    /// it).
    pub fn check_for(
        &self,
        prerequisite: &Prerequisite,
        value: Option<&Parameter>,
    ) -> Result<(), Unmet> {
        self.check_in(
            prerequisite,
            &Picking {
                value,
                ..Picking::default()
            },
        )
    }

    /// [`check_for`](Self::check_for) for a feat's prerequisites: a class level is the one feats see
    /// ([`feat_class_level`](Self::feat_class_level)).
    pub fn check_feat_for(
        &self,
        prerequisite: &Prerequisite,
        value: Option<&Parameter>,
    ) -> Result<(), Unmet> {
        self.check_in(
            prerequisite,
            &Picking {
                value,
                for_feat: true,
                ..Picking::default()
            },
        )
    }

    /// Evaluates a selection option's Prerequisite for a pick of `option` through `selection`'s slot, taken for
    /// `value` (`NoOtherOptionOfGroup` and `PickedThrough` look at the pick).
    pub fn check_option(
        &self,
        prerequisite: &Prerequisite,
        option: &SelectionOptionKey,
        selection: &FeatureSelectionKey,
        value: Option<&Parameter>,
    ) -> Result<(), Unmet> {
        self.check_in(
            prerequisite,
            &Picking {
                value,
                option: Some(option),
                selection: Some(selection),
                for_feat: false,
            },
        )
    }

    fn check_in(&self, prerequisite: &Prerequisite, picking: &Picking<'_>) -> Result<(), Unmet> {
        use Prerequisite as P;
        let (value, for_feat) = (picking.value, picking.for_feat);
        let numeric = |has: u8, needs: u8| (has >= needs, Some(has));
        let flag = |holds: bool| (holds, None);
        let (holds, has) = match prerequisite {
            P::All(children) => {
                let mut failed: Vec<Unmet> = children
                    .iter()
                    .filter_map(|c| self.check_in(c, picking).err())
                    .collect();
                return match failed.len() {
                    0 => Ok(()),
                    1 => Err(failed.remove(0)),
                    _ => Err(Unmet::All(failed)),
                };
            }
            P::Any(children) => {
                let mut failed = Vec::with_capacity(children.len());
                for child in children {
                    match self.check_in(child, picking) {
                        Ok(()) => return Ok(()),
                        Err(unmet) => failed.push(unmet),
                    }
                }
                return Err(Unmet::Any(failed));
            }
            P::AtLeast(needed, children) => {
                let failed: Vec<Unmet> = children
                    .iter()
                    .filter_map(|c| self.check_in(c, picking).err())
                    .collect();
                let met = u8::try_from(children.len() - failed.len()).unwrap_or(u8::MAX);
                return if met >= *needed {
                    Ok(())
                } else {
                    Err(Unmet::AtLeast {
                        needed: *needed,
                        met,
                        failed,
                    })
                };
            }
            P::Feat(feat) => flag(self.has_feat(feat)),
            P::FeatFor { feat, value } => flag(self.has_feat_for(feat, value)),
            P::FeatForSameValue(feat) => {
                flag(value.is_some_and(|value| self.has_feat_for(feat, value)))
            }
            P::FeatIfValue { value: when, feat } => {
                flag(value != Some(when) || self.has_feat(feat))
            }
            P::Not(child) => flag(self.check_in(child, picking).is_err()),
            P::MythicFeat(feat) => flag(self.mythic_feat_count(feat) > 0),
            P::MythicAbility(ability) => flag(self.mythic_ability_count(ability) > 0),
            P::ClassLevel { class, level } => {
                let has = if for_feat {
                    self.feat_class_level(class)
                } else {
                    self.class_level(class)
                };
                numeric(self.prerequisite_class_level(class, has, picking), *level)
            }
            P::Archetype(archetype) => flag(
                self.classes
                    .iter()
                    .any(|c| c.archetype.as_ref() == Some(archetype)),
            ),
            P::Feature { feature, count } => numeric(self.feature_count(feature), *count),
            P::Rank { fact, rank } => numeric(fact.rank(self), *rank),
            P::SelectionOption(option) => flag(self.option_count(option) > 0),
            P::Selection(selection) => flag(self.selection_count(selection) > 0),
            P::NoOptionFrom(selection) => flag(!self.holds_option_of(selection)),
            P::NoOtherOptionOfGroup(group) => {
                flag(!self.holds_other_option_of(*group, picking.option))
            }
            P::PickedThrough(selection) => flag(picking.selection == Some(selection)),
            P::SkillRanks { skill, ranks } => numeric(
                self.prerequisite_stat(
                    |stat| matches!(stat, StatRef::Skill(of) if of == skill),
                    self.skill_ranks(skill),
                    &self.ability_scores,
                ),
                *ranks,
            ),
            P::AbilityScore { ability, score } => numeric(
                self.prerequisite_stat(
                    |stat| *stat == StatRef::Ability(*ability),
                    self.permanent_ability_scores.get(*ability),
                    &self.ability_scores,
                ),
                *score,
            ),
            P::FullAbilityScore { ability, score } => numeric(
                self.prerequisite_stat(
                    |stat| *stat == StatRef::Ability(*ability),
                    self.full_ability_scores.get(*ability),
                    &self.permanent_ability_scores,
                ),
                *score,
            ),
            P::BaseAttackBonus(bab) => numeric(
                self.prerequisite_stat(
                    |stat| *stat == StatRef::BaseAttack,
                    self.bab,
                    &self.ability_scores,
                ),
                *bab,
            ),
            // `GetClassSpellLevel` is `null` for a class not taken or without a spellbook, which meets no level.
            P::ClassSpellLevel { class, level } => match self.class_spell_levels.get(class) {
                Some(has) => numeric(*has, *level),
                None => (false, None),
            },
            P::CasterType(spell_type) => flag(self.caster_types.contains(spell_type)),
            P::SpellLevel { spell_type, level } => numeric(self.spell_level(*spell_type), *level),
            P::SpontaneousSpellLevel { spell_type, level } => {
                numeric(self.spontaneous_spell_level(*spell_type), *level)
            }
            P::CasterLevel(level) => numeric(self.caster_level, *level),
            P::CharacterLevel(level) => numeric(self.character_level, *level),
            P::MythicRank(rank) => numeric(self.mythic_rank, *rank),
            P::Alignment(alignment) => flag(self.alignment == Some(*alignment)),
            P::Deity(deity) => flag(self.deity() == Some(deity)),
            P::Atheist => flag(self.faith == Some(Faith::Atheism)),
            P::Worships => flag(self.deity().is_some()),
            P::DeityOffers(domain) => flag(self.deity_domains.contains(domain)),
            P::Race(race) => flag(self.race.as_ref() == Some(race)),
            // Out of scope: never checked, shown as a note.
            P::Story(_) => flag(true),
        };
        if holds {
            Ok(())
        } else {
            Err(Unmet::Condition {
                needs: prerequisite.clone(),
                has,
            })
        }
    }
}

/// What the pick being checked is, for the prerequisites that look at it.
#[derive(Clone, Copy, Debug, Default)]
struct Picking<'a> {
    /// The value it's taken for (`FeatForSameValue`, `FeatIfValue`).
    value: Option<&'a Parameter>,
    /// The option picked (`NoOtherOptionOfGroup`).
    option: Option<&'a SelectionOptionKey>,
    /// The selection whose slot it fills (`PickedThrough`).
    selection: Option<&'a FeatureSelectionKey>,
    /// A feat's prerequisites: class levels are the ones feats see.
    for_feat: bool,
}
