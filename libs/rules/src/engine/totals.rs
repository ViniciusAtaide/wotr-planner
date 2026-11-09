//! Initiative, the saves and each skill's total at one point of a Build (ticket 120).
//!
//! Unconditional numbers only, from what the Build fixes: the character's own scores, the saves' class progressions,
//! the skill ranks and class skills, and the bonuses of the items held ([`ItemBonus`](crate::ItemBonus),
//! `bonuses.ron`): Improved Initiative, Iron Will and its kin, Skill Focus, racial and background bonuses, Halfling
//! Luck, Mythic Resolve, Divine Grace. Everything else is [`TOTALS_LEAVE_OUT`].

use crate::bonus::BonusTarget;
use crate::build::{AbilityScores, BuildStart};
use crate::game_data::{Ability, Save};
use crate::gear::Assumed;
use crate::keys::SkillKey;
use crate::parameter::Parameter;
use crate::state::CharacterState;

use super::Engine;
use super::caster::{amount, modifier};

/// The class skill bonus: a trained class skill's checks get +3.
pub const CLASS_SKILL_BONUS: i16 = 3;

/// Initiative, the saves and the skills at one point of a Build ([`Engine::totals`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Totals {
    /// Dexterity's modifier plus each bonus.
    pub initiative: Total,
    pub fortitude: Total,
    pub reflex: Total,
    pub will: Total,
    /// Every skill of the data, in its (canonical: key) order.
    pub skills: Vec<SkillTotal>,
    /// What no total counts: [`TOTALS_LEAVE_OUT`].
    pub leaves_out: &'static [&'static str],
}

impl Totals {
    pub fn save(&self, save: Save) -> &Total {
        match save {
            Save::Fortitude => &self.fortitude,
            Save::Reflex => &self.reflex,
            Save::Will => &self.will,
        }
    }

    pub fn skill(&self, skill: &SkillKey) -> Option<&SkillTotal> {
        self.skills.iter().find(|total| &total.skill == skill)
    }
}

/// A number and what it counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Total {
    /// The sum of [`parts`](Self::parts).
    pub total: i16,
    pub parts: Vec<TotalPart>,
}

/// One skill's total.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillTotal {
    pub skill: SkillKey,
    /// The ranks the total counts: the ranks spent, or the Mythic rank under Perfect Mind when higher.
    pub ranks: u8,
    /// The ranks, the skill's ability modifier, the class skill bonus when trained, then each bonus.
    pub total: Total,
}

/// One thing a total counts, with what it adds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TotalPart {
    /// "Base save", "Ranks", "Dexterity", "Class skill", or the item's name ("Improved Initiative", "Skill Focus
    /// (Persuasion)", "Halfling").
    pub name: String,
    pub value: i16,
}

/// What the totals leave out, the same for every Build: they hold only the numbers a Build fixes whatever happens in
/// a fight.
pub const TOTALS_LEAVE_OUT: &[&str] = &[
    "gear: ability-score items, resistance bonuses, skill items",
    "buffs, spells, songs and auras cast on the character, an ally's included",
    "abilities switched on in a fight: rage, aspects, judgments, stances, a shifter's aspects",
    "bonuses against one kind of effect or foe (+2 against fear, poison, enchantments, demons, traps)",
    "bonuses to one use of a skill: Persuasion when used to intimidate, bluff or use diplomacy (an aasimar's, a \
     tiefling's, a dhampir's, the half-orc's, the Andoran Diplomat's), Knowledge to identify items",
    "bonuses in one place or with one weapon (underground, a favoured terrain, a dueling sword)",
    "bonuses that grow with class level (Danger Sense, Bravery, Forewarned, Resistant Touch, Purity, a familiar's)",
    "a background's other ability for a skill (Acolyte: Wisdom for Persuasion; Healer: Intelligence for the Lores): \
     each skill counts its own ability",
    "rerolls and taking 10 or 20 (the mythic Iron Will, Great Fortitude, Lightning Reflexes, Skill Focus (Mythic))",
    "armour check penalties and size",
];

impl Engine {
    /// Initiative, the saves and each skill's total at a character state, the Build start giving the race, subrace
    /// and background.
    pub fn totals(&self, start: &BuildStart, state: &CharacterState) -> Totals {
        self.totals_with(start, state, &state.ability_scores, 0)
    }

    /// [`totals`](Self::totals) with what an agent assumes on top of the Build (ticket 122): the scores raised by the
    /// assumed ability bonuses, and the assumed resistance bonus as a part of its own on each save.
    pub fn assumed_totals(
        &self,
        start: &BuildStart,
        state: &CharacterState,
        assumed: &Assumed,
    ) -> Totals {
        self.totals_with(
            start,
            state,
            &assumed.scores(state.ability_scores),
            assumed.resistance,
        )
    }

    /// The totals from these ability scores (the character's own, or with assumed bonuses) and a resistance bonus
    /// to saves (only assumed). What the character holds, and so every `requires`, is read from `state` alone.
    fn totals_with(
        &self,
        start: &BuildStart,
        state: &CharacterState,
        scores: &AbilityScores,
        resistance: u8,
    ) -> Totals {
        let held = self.held_bonuses(start, state);
        let ability = |ability: Ability| TotalPart {
            name: ability.name().to_owned(),
            value: modifier(scores.get(ability)),
        };
        let bonuses = |adds: &dyn Fn(&BonusTarget, Option<&Parameter>) -> bool,
                       skill: Option<&SkillKey>| {
            held.iter()
                .filter(|(bonus, value, _)| adds(&bonus.target, value.as_ref()))
                .map(|(bonus, _, name)| TotalPart {
                    name: name.clone(),
                    value: amount(bonus.amount, state, scores, skill),
                })
                .collect::<Vec<_>>()
        };

        let mut initiative = vec![ability(Ability::Dexterity)];
        initiative.extend(bonuses(
            &|target, value| target.adds_to_initiative(value),
            None,
        ));

        let save = |save: Save, base: u8, own: Ability| {
            let mut parts = vec![
                TotalPart {
                    name: "Base save".to_owned(),
                    value: i16::from(base),
                },
                ability(own),
            ];
            parts.extend(bonuses(
                &|target, value| target.adds_to_save(save, value),
                None,
            ));
            if resistance > 0 {
                parts.push(TotalPart {
                    name: Assumed::label(&format!("Resistance +{resistance}")),
                    value: i16::from(resistance),
                });
            }
            total(parts)
        };

        // Perfect Mind: every skill is a class skill, with ranks at least the Mythic rank.
        let every_skill_a_class_skill = state.skill_ranks_at_least_mythic_rank;
        let skills = self
            .catalog
            .data()
            .skills
            .iter()
            .map(|skill| {
                let key = &skill.key;
                let ranks = state.prerequisite_skill_ranks(key);
                let mut parts = vec![
                    TotalPart {
                        name: "Ranks".to_owned(),
                        value: i16::from(ranks),
                    },
                    ability(skill.ability),
                ];
                if ranks > 0 && (every_skill_a_class_skill || state.class_skills.contains(key)) {
                    parts.push(TotalPart {
                        name: "Class skill".to_owned(),
                        value: CLASS_SKILL_BONUS,
                    });
                }
                parts.extend(bonuses(
                    &|target, value| target.adds_to_skill(key, value),
                    Some(key),
                ));
                SkillTotal {
                    skill: key.clone(),
                    ranks,
                    total: total(parts),
                }
            })
            .collect();

        Totals {
            initiative: total(initiative),
            fortitude: save(Save::Fortitude, state.fort, Ability::Constitution),
            reflex: save(Save::Reflex, state.reflex, Ability::Dexterity),
            will: save(Save::Will, state.will, Ability::Wisdom),
            skills,
            leaves_out: TOTALS_LEAVE_OUT,
        }
    }
}

fn total(parts: Vec<TotalPart>) -> Total {
    Total {
        total: parts.iter().map(|part| part.value).sum(),
        parts,
    }
}
