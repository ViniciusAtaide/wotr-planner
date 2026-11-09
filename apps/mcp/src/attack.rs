//! The attack line `sheet` and `compare` add for an optional `weapon` (ticket 114): the engine's
//! [`rules::Engine::attack_line`] at a character state, as JSON. The weapon is no part of the Build or its code.

use rules::{ATTACK_LINE_LEAVES_OUT, CharacterState, Engine, WeaponKey};
use schemars::JsonSchema;
use serde::Serialize;

use crate::error::ToolError;
use crate::shapes::Named;

/// One base weapon's full attack at a level, from the planner's engine: unconditional numbers only.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct AttackLineShape {
    pub weapon: Named,
    /// The attack bonus of each attack of a full attack, from the first at the full BAB.
    pub attacks: Vec<i16>,
    /// A hit's damage: the weapon's dice and the bonus ("1d8+4").
    pub damage: String,
    /// The threat range and multiplier ("19–20/×2"), Improved Critical counted.
    pub critical: String,
    /// Power Attack's trade, when the character has it and the weapon is a melee weapon: a penalty to every attack
    /// for a bonus to each hit's damage. The `attacks` and `damage` above don't take it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_attack: Option<PowerAttackShape>,
    /// What the numbers count besides the BAB: the ability modifiers, then each feat held for the weapon, with what
    /// each adds to attack and to damage.
    pub counts: Vec<AttackPartShape>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct PowerAttackShape {
    pub attack_penalty: i16,
    pub damage_bonus: i16,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct AttackPartShape {
    pub name: String,
    pub attack: i16,
    pub damage: i16,
}

/// The weapon a `weapon` argument names: `None` without one, refused when the data has no line for it.
pub(crate) fn weapon_argument(
    engine: &Engine,
    weapon: Option<&str>,
) -> Result<Option<WeaponKey>, ToolError> {
    let Some(weapon) = weapon else {
        return Ok(None);
    };
    let key = WeaponKey::new(weapon);
    if engine.attack_weapons().iter().any(|known| known.key == key) {
        Ok(Some(key))
    } else {
        Err(ToolError::invalid_arguments(format!(
            "`weapon`: {weapon:?} isn't a weapon with an attack line. Give a weapon's key (`search` finds it): \
             natural attacks, rays, touch attacks, Kinetic Blast and Bomb have no dice of their own."
        )))
    }
}

/// The attack line for a weapon at a character state.
pub(crate) fn attack_line(
    engine: &Engine,
    state: &CharacterState,
    weapon: &WeaponKey,
) -> Option<AttackLineShape> {
    let line = engine.attack_line(state, weapon)?;
    Some(AttackLineShape {
        weapon: Named {
            key: weapon.to_string(),
            name: engine.weapon_name(weapon).to_owned(),
        },
        attacks: line.attacks.clone(),
        damage: line.damage_text(),
        critical: line.critical_text(),
        power_attack: line.power_attack.map(|trade| PowerAttackShape {
            attack_penalty: trade.attack_penalty,
            damage_bonus: trade.damage_bonus,
        }),
        counts: line
            .parts
            .iter()
            .map(|part| AttackPartShape {
                name: part.name.clone(),
                attack: part.attack,
                damage: part.damage,
            })
            .collect(),
    })
}

/// What no attack line counts, for a result that has one.
pub(crate) fn leaves_out() -> Vec<String> {
    ATTACK_LINE_LEAVES_OUT
        .iter()
        .map(|item| (*item).to_owned())
        .collect()
}
