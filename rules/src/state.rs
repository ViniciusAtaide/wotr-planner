//! What a character has at one point of its Build, and Prerequisite checks against it.

use std::collections::{BTreeMap, BTreeSet};

use crate::build::AbilityScores;
use crate::game_data::{Alignment, SpellType};
use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, FeatureKey, MythicAbilityKey, MythicFeatKey, MythicPathKey,
    RaceKey, SelectionOptionKey, SkillKey,
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
    /// The highest effective class level among classes that can cast (class levels plus prestige levels that
    /// advance them). Checked only where the guide states a caster level.
    pub caster_level: u8,
    pub skill_ranks: BTreeMap<SkillKey, u8>,
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
    pub mythic_abilities: BTreeSet<MythicAbilityKey>,
    pub mythic_feats: BTreeSet<MythicFeatKey>,
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
        use Prerequisite as P;
        let numeric = |has: u8, needs: u8| (has >= needs, Some(has));
        let flag = |holds: bool| (holds, None);
        let (holds, has) = match prerequisite {
            P::All(children) => {
                let mut failed: Vec<Unmet> = children
                    .iter()
                    .filter_map(|c| self.check_for(c, value).err())
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
                    match self.check_for(child, value) {
                        Ok(()) => return Ok(()),
                        Err(unmet) => failed.push(unmet),
                    }
                }
                return Err(Unmet::Any(failed));
            }
            P::AtLeast(needed, children) => {
                let failed: Vec<Unmet> = children
                    .iter()
                    .filter_map(|c| self.check_for(c, value).err())
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
            P::MythicFeat(feat) => flag(self.mythic_feats.contains(feat)),
            P::MythicAbility(ability) => flag(self.mythic_abilities.contains(ability)),
            P::ClassLevel { class, level } => numeric(self.class_level(class), *level),
            P::Archetype(archetype) => flag(
                self.classes
                    .iter()
                    .any(|c| c.archetype.as_ref() == Some(archetype)),
            ),
            P::Feature { feature, count } => numeric(self.feature_count(feature), *count),
            P::SelectionOption(option) => flag(self.selection_options.contains_key(option)),
            P::SkillRanks { skill, ranks } => numeric(self.skill_ranks(skill), *ranks),
            P::AbilityScore { ability, score } => {
                numeric(self.ability_scores.get(*ability), *score)
            }
            P::BaseAttackBonus(bab) => numeric(self.bab, *bab),
            P::SpellLevel { spell_type, level } => numeric(self.spell_level(*spell_type), *level),
            P::CasterLevel(level) => numeric(self.caster_level, *level),
            P::CharacterLevel(level) => numeric(self.character_level, *level),
            P::MythicRank(rank) => numeric(self.mythic_rank, *rank),
            P::Alignment(alignment) => flag(self.alignment == Some(*alignment)),
            P::Race(race) => flag(self.race.as_ref() == Some(race)),
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
