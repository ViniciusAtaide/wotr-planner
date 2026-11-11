//! The totals `sheet` gives at the Build's last level (ticket 120): the engine's [`rules::Engine::totals`] as JSON,
//! initiative, the three saves and each skill's total, each with what it counts.

use std::collections::BTreeMap;

use rules::{CharacterState, Engine, TOTALS_LEAVE_OUT, Total, Totals};
use schemars::JsonSchema;
use serde::Serialize;

use crate::caster::CasterPartShape;

/// Initiative, the saves and the skills at the last level, from the planner's engine: unconditional numbers only.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct TotalsShape {
    /// Dexterity's modifier plus each bonus held (Improved Initiative, its mythic feat, the Pickpocket background).
    pub initiative: TotalShape,
    /// Each save: the class progressions' base save, the ability's modifier, then each bonus held.
    pub fortitude: TotalShape,
    pub reflex: TotalShape,
    pub will: TotalShape,
    /// Every skill by key: its ranks, the skill's ability modifier, +3 for a trained class skill, then each bonus
    /// held (racial, background, Skill Focus).
    pub skills: BTreeMap<String, SkillTotalShape>,
}

/// A number and what it counts, which add up to it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct TotalShape {
    pub total: i16,
    pub counts: Vec<CasterPartShape>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct SkillTotalShape {
    pub ranks: u8,
    pub total: i16,
    pub counts: Vec<CasterPartShape>,
}

fn counts(total: &Total) -> Vec<CasterPartShape> {
    total
        .parts
        .iter()
        .map(|part| CasterPartShape {
            name: part.name.clone(),
            value: part.value,
        })
        .collect()
}

fn shape(total: &Total) -> TotalShape {
    TotalShape {
        total: total.total,
        counts: counts(total),
    }
}

/// The totals at a character state.
pub(crate) fn totals(engine: &Engine, state: &CharacterState) -> TotalsShape {
    shape_of(&engine.totals(state))
}

/// Totals as JSON.
fn shape_of(totals: &Totals) -> TotalsShape {
    TotalsShape {
        initiative: shape(&totals.initiative),
        fortitude: shape(&totals.fortitude),
        reflex: shape(&totals.reflex),
        will: shape(&totals.will),
        skills: totals
            .skills
            .iter()
            .map(|skill| {
                (
                    skill.skill.to_string(),
                    SkillTotalShape {
                        ranks: skill.ranks,
                        total: skill.total.total,
                        counts: counts(&skill.total),
                    },
                )
            })
            .collect(),
    }
}

/// What no total counts.
pub(crate) fn leaves_out() -> Vec<String> {
    TOTALS_LEAVE_OUT.iter().map(ToString::to_string).collect()
}
