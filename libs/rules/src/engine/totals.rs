//! Initiative, the saves and each skill's total at one point of a Build (ticket 120).
//!
//! Unconditional numbers only, from what the Build fixes: the character's scores (its gear's bonuses included), the
//! saves' class progressions, the skill ranks and class skills, and the bonuses in force ([`BonusInForce`]): the Item
//! bonuses of `bonuses.ron` (Improved Initiative, Iron Will and its kin, Skill Focus, racial and background bonuses,
//! Halfling Luck, Mythic Resolve, Divine Grace) and the always-on bonuses of the Build's gear, stacked by type
//! ([`counted`]). Everything else is [`TOTALS_LEAVE_OUT`].

use crate::bonus::{BonusInForce, BonusTarget, counted};
use crate::game_data::{Ability, Save};
use crate::keys::SkillKey;

use super::{Engine, LevelSheet};

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
    /// (Persuasion)", "Halfling", "Grand Festival Cloak").
    pub name: String,
    pub value: i16,
}

/// What the totals leave out, the same for every Build: they hold only the numbers a Build fixes whatever happens in
/// a fight.
pub const TOTALS_LEAVE_OUT: &[&str] = &[
    "gear the Build doesn't hold, and what its items do besides their always-on bonuses: bonuses in some \
     circumstance or against one thing, activated abilities, set bonuses",
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
    /// Initiative, the saves and each skill's total at a level of the sheet ([`Engine::evaluate`]), at the state
    /// that ends it.
    ///
    /// It takes the sheet's level rather than a bare state because only the sheet's states record the bonuses in
    /// force ([`bonuses_in_force`](crate::CharacterState::bonuses_in_force)): a state from anywhere else
    /// ([`Engine::state_before`], one built by hand) has none, and its totals would leave out every item bonus
    /// without an error.
    pub fn totals(&self, level: &LevelSheet) -> Totals {
        let state = &level.state;
        let ability = |ability: Ability| TotalPart {
            name: ability.name().to_owned(),
            value: i16::from(state.ability_scores.modifier(ability)),
        };
        // The bonuses in force that add to one number and count there: gear's stack by type.
        let bonuses = |adds: &dyn Fn(&BonusTarget, &BonusInForce) -> bool| {
            let adding = (state.bonuses_in_force.iter())
                .filter(|bonus| bonus.target.number().is_some_and(|to| adds(to, bonus)));
            counted(adding)
                .into_iter()
                .map(|bonus| TotalPart {
                    name: bonus.name.clone(),
                    value: bonus.amount,
                })
                .collect::<Vec<_>>()
        };

        let mut initiative = vec![ability(Ability::Dexterity)];
        initiative.extend(bonuses(&|target, bonus| {
            target.adds_to_initiative(bonus.value.as_ref())
        }));

        let save = |save: Save, base: u8, own: Ability| {
            let mut parts = vec![
                TotalPart {
                    name: "Base save".to_owned(),
                    value: i16::from(base),
                },
                ability(own),
            ];
            parts.extend(bonuses(&|target, bonus| {
                target.adds_to_save(save, bonus.value.as_ref())
            }));
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
                parts.extend(bonuses(&|target, bonus| {
                    target.adds_to_skill(key, bonus.value.as_ref())
                }));
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
