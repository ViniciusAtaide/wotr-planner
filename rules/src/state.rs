//! What a character has at one point of its Build, and Prerequisite checks against it.

use std::collections::{BTreeMap, BTreeSet};

use crate::build::AbilityScores;
use crate::game_data::{Alignment, OptionGroup, SpellType};
use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, FeatListKey, FeatureKey, FeatureSelectionKey,
    MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey,
};
use crate::parameter::Parameter;
use crate::prerequisite::Prerequisite;
use crate::problem::Unmet;

/// Everything derived from a Build up to some point: the end of a character level, or just before a slot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CharacterState {
    pub character_level: u8,
    pub race: Option<RaceKey>,
    pub alignment: Option<Alignment>,
    /// One entry per class taken, in order of first entry.
    pub classes: Vec<ClassLevels>,
    pub bab: u8,
    pub fort: u8,
    pub reflex: u8,
    pub will: u8,
    /// The character's own ability scores, as prerequisites and skill points see them: the base scores plus the
    /// ability bonuses of the features held (Legend's +4). Gear never counts.
    pub ability_scores: AbilityScores,
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
    /// The class skills: each class taken's (an archetype's list where it replaces its class's) and the
    /// background's.
    pub class_skills: BTreeSet<SkillKey>,
    /// The class levels a feat's prerequisites see, for each class another class's levels count as there
    /// (Diverse Training: fighter 1 + Eldritch Knight 3 is fighter 4). Classes left out are their class level.
    pub feat_class_levels: BTreeMap<ClassKey, u8>,
    /// Each feat held, with how many picks and grants hold it (Extra Hex taken twice: 2). A class grant and a pick
    /// may both hold one, so losing one of them keeps the feat.
    pub feats: BTreeMap<FeatKey, u8>,
    /// Each parameterised feat held, with the values it's held for and how many picks and grants hold each
    /// (Weapon Focus: Longsword 1, Dagger 1).
    pub feat_values: BTreeMap<FeatKey, BTreeMap<Parameter, u8>>,
    /// Each feature with the number of times it has been granted ("Sneak Attack: 3").
    pub features: BTreeMap<FeatureKey, u8>,
    /// Each selection option held, with how many times it was taken (Increased Damage Reduction: 2).
    pub selection_options: BTreeMap<SelectionOptionKey, u8>,
    /// The options held through each selection that gave them, with how many times (Expanded Element: the
    /// elements picked at 7 and 15). An option granted outside a selection's slot isn't here.
    pub options_through: BTreeMap<FeatureSelectionKey, BTreeMap<SelectionOptionKey, u8>>,
    /// The group of each option held that belongs to one (a Draconic bloodline).
    pub option_groups: BTreeMap<SelectionOptionKey, OptionGroup>,
    /// The option level of each option held through a selection with option levels (a sorcerer 5 / Dragon
    /// Disciple 5's bloodline: 10).
    pub option_levels: BTreeMap<SelectionOptionKey, u8>,
    /// The feat list in force of each option held that has one: what its [`FeatSlot::OptionList`] slots offer.
    ///
    /// [`FeatSlot::OptionList`]: crate::FeatSlot::OptionList
    pub option_feat_lists: BTreeMap<SelectionOptionKey, FeatListKey>,
    pub mythic_abilities: BTreeSet<MythicAbilityKey>,
    pub mythic_feats: BTreeSet<MythicFeatKey>,
    /// Each parameterised mythic feat held, with the values it's held for (Improved Critical (Mythic): Longsword).
    pub mythic_feat_values: BTreeMap<MythicFeatKey, BTreeSet<Parameter>>,
    /// The base attack bonus prerequisites see at least (Perfect Mind: 15); 0 when no feature sets one.
    pub bab_floor: u8,
    /// Set while a feature makes every skill's ranks at least the Mythic rank for prerequisites (Perfect Mind).
    pub skill_ranks_at_least_mythic_rank: bool,
    /// The Mythic rank reached: 3 after a Legend switch, whatever the rank before.
    pub mythic_rank: u8,
    pub mythic_path: Option<MythicPathKey>,
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

    /// The ranks in a skill that prerequisites see: the ranks, or the Mythic rank under Perfect Mind when higher.
    pub fn prerequisite_skill_ranks(&self, skill: &SkillKey) -> u8 {
        let floor = if self.skill_ranks_at_least_mythic_rank {
            self.mythic_rank
        } else {
            0
        };
        self.skill_ranks(skill).max(floor)
    }

    /// The base attack bonus that prerequisites see: the BAB, or a feature's floor when higher (Perfect Mind).
    pub fn prerequisite_bab(&self) -> u8 {
        self.bab.max(self.bab_floor)
    }

    /// Whether a parameterised mythic feat is held for a value.
    pub fn has_mythic_feat_for(&self, feat: &MythicFeatKey, value: &Parameter) -> bool {
        self.mythic_feat_values
            .get(feat)
            .is_some_and(|values| values.contains(value))
    }

    /// How many times a feature has been granted; 0 when never.
    pub fn feature_count(&self, feature: &FeatureKey) -> u8 {
        self.features.get(feature).copied().unwrap_or(0)
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

    /// Whether any option of this selection is held (its own options, keyed under it, wherever they were taken).
    pub fn holds_option_of(&self, selection: &FeatureSelectionKey) -> bool {
        self.selection_options.keys().any(|option| {
            option
                .as_str()
                .strip_prefix(selection.as_str())
                .is_some_and(|rest| rest.starts_with('/'))
        })
    }

    /// How many picks and grants hold a feat; 0 when none.
    pub fn feat_count(&self, feat: &FeatKey) -> u8 {
        self.feats.get(feat).copied().unwrap_or(0)
    }

    /// Whether a parameterised feat is held for a value.
    pub fn has_feat_for(&self, feat: &FeatKey, value: &Parameter) -> bool {
        self.feat_values
            .get(feat)
            .is_some_and(|values| values.contains_key(value))
    }

    /// How many times a selection option was taken; 0 when never.
    pub fn option_count(&self, option: &SelectionOptionKey) -> u8 {
        self.selection_options.get(option).copied().unwrap_or(0)
    }

    /// The class level a feat's prerequisites see: the class level plus the levels of classes that count as it
    /// for feats.
    pub fn feat_class_level(&self, class: &ClassKey) -> u8 {
        self.feat_class_levels
            .get(class)
            .copied()
            .unwrap_or_else(|| self.class_level(class))
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
            P::Feat(feat) => flag(self.feats.contains_key(feat)),
            P::FeatFor { feat, value } => flag(self.has_feat_for(feat, value)),
            P::FeatForSameValue(feat) => {
                flag(value.is_some_and(|value| self.has_feat_for(feat, value)))
            }
            P::FeatIfValue { value: when, feat } => {
                flag(value != Some(when) || self.feats.contains_key(feat))
            }
            P::Not(child) => flag(self.check_in(child, picking).is_err()),
            P::MythicFeat(feat) => flag(self.mythic_feats.contains(feat)),
            P::MythicAbility(ability) => flag(self.mythic_abilities.contains(ability)),
            P::ClassLevel { class, level } => {
                let has = if for_feat {
                    self.feat_class_level(class)
                } else {
                    self.class_level(class)
                };
                numeric(has, *level)
            }
            P::Archetype(archetype) => flag(
                self.classes
                    .iter()
                    .any(|c| c.archetype.as_ref() == Some(archetype)),
            ),
            P::Feature { feature, count } => numeric(self.feature_count(feature), *count),
            P::SelectionOption(option) => flag(self.selection_options.contains_key(option)),
            P::NoOptionFrom(selection) => flag(!self.holds_option_of(selection)),
            P::NoOtherOptionOfGroup(group) => {
                flag(!self.holds_other_option_of(*group, picking.option))
            }
            P::PickedThrough(selection) => flag(picking.selection == Some(selection)),
            P::SkillRanks { skill, ranks } => numeric(self.prerequisite_skill_ranks(skill), *ranks),
            P::AbilityScore { ability, score } => {
                numeric(self.ability_scores.get(*ability), *score)
            }
            P::BaseAttackBonus(bab) => numeric(self.prerequisite_bab(), *bab),
            P::SpellLevel { spell_type, level } => numeric(self.spell_level(*spell_type), *level),
            P::SpontaneousSpellLevel { spell_type, level } => {
                numeric(self.spontaneous_spell_level(*spell_type), *level)
            }
            P::CasterLevel(level) => numeric(self.caster_level, *level),
            P::CharacterLevel(level) => numeric(self.character_level, *level),
            P::MythicRank(rank) => numeric(self.mythic_rank, *rank),
            P::Alignment(alignment) => flag(self.alignment == Some(*alignment)),
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
