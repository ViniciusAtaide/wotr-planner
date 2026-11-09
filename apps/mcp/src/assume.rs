//! What `sheet` and `compare` add for an optional `assume` (ticket 122): gear an agent assumes on top of a Build, and
//! the engine's attack line, caster line and totals with it ([`rules::Engine::assumed_attack_line`],
//! [`rules::Engine::assumed_caster_line`], [`rules::Engine::assumed_totals`]), at the Build's last level, beside the
//! unassisted numbers. Never part of the Build, its code or any prerequisite: the agent's assumption on top of the
//! planner's facts.

use std::collections::BTreeMap;

use rules::{
    AbilityScores, Assumed, BuildStart, CharacterState, Engine, GearKey, SpellTraits, WeaponKey,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::address::ability_token;
use crate::attack::{self, AttackLineShape};
use crate::caster::{self, CasterLineShape};
use crate::error::ToolError;
use crate::shapes::{Named, Scores};
use crate::totals::{self, TotalsShape};

/// The highest enhancement bonus to an ability score on the saved item pages: +8 (Belt of Physical Perfection +8 to
/// Strength, Dexterity and Constitution, `items-inevitable-excess`; Headband of Mental Perfection +8, Razmir's Mask
/// and Darkness Caress to Intelligence, Wisdom and Charisma, `items`).
pub(crate) const HIGHEST_ENHANCEMENT: u8 = 8;
/// The guide's inherent bonuses (`items`): +2 to one ability (a tome or manual), +2 to all (the Elixir of
/// Inconceivable Transmutations), +1 to all (Potion of Humanity), +4 to all (Tome of Ancient Perfection). The guide
/// doesn't say whether they stack: at most their sum.
pub(crate) const HIGHEST_INHERENT: u8 = 9;
/// The highest weapon enhancement on the saved item pages: +5.
pub(crate) const HIGHEST_WEAPON_ENHANCEMENT: u8 = 5;
/// The highest resistance bonus to saves on the saved item pages: +7 (Cloak of Resistance +7,
/// `items-inevitable-excess`; Cloak of Reflections and Grand Festival Cloak, `items`).
pub(crate) const HIGHEST_RESISTANCE: u8 = 7;

/// Gear assumed on top of a Build: generic bonuses, and items of the guide's caster level and spell DC tables.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssumeArgs {
    /// An enhancement bonus to each ability score named (`{"cha": 8, "dex": 6}`), 1 to 8.
    #[serde(default)]
    pub enhancement: BTreeMap<String, u8>,
    /// An inherent bonus to each ability score named (`{"cha": 4}`), 1 to 9.
    #[serde(default)]
    pub inherent: BTreeMap<String, u8>,
    /// The weapon's enhancement bonus to attack and damage, 0 to 5 (the attack line only).
    #[serde(default)]
    pub weapon_enhancement: u8,
    /// A resistance bonus to every save, 0 to 7 (the totals only).
    #[serde(default)]
    pub resistance: u8,
    /// Items of the guide's caster level and spell DC tables, by key (`rod-of-power-source`): the caster line adds
    /// each bonus whose circumstance matches its spell.
    #[serde(default)]
    pub items: Vec<String>,
}

/// The numbers at the last level with the assumed gear: the agent's assumption on top of the planner's facts, beside
/// the unassisted numbers. Every part an assumption adds is named "… (assumed)".
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct AssumedShape {
    /// What was assumed, as given.
    pub assumes: AssumesShape,
    /// The ability scores with the assumed bonuses. The Build's own scores, which every prerequisite checks, are
    /// unchanged.
    pub scores: Scores,
    /// With a `weapon`: the attack line with the assumed scores and weapon enhancement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<AttackLineShape>,
    /// With a `caster`: the caster line with the assumed scores and the items whose circumstance matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caster: Option<CasterLineShape>,
    /// `sheet` only: the totals with the assumed scores and resistance bonus.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totals: Option<TotalsShape>,
}

/// What was assumed.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct AssumesShape {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub enhancement: BTreeMap<String, u8>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub inherent: BTreeMap<String, u8>,
    #[serde(skip_serializing_if = "is_zero")]
    pub weapon_enhancement: u8,
    #[serde(skip_serializing_if = "is_zero")]
    pub resistance: u8,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Named>,
}

fn is_zero(value: &u8) -> bool {
    *value == 0
}

/// The gear an `assume` argument names: `None` without one, refused when an ability, a number or an item isn't one.
pub(crate) fn assume_argument(
    engine: &Engine,
    assume: Option<&AssumeArgs>,
) -> Result<Option<Assumed>, ToolError> {
    let Some(assume) = assume else {
        return Ok(None);
    };
    let abilities = |what: &str, given: &BTreeMap<String, u8>, highest: u8| {
        given
            .iter()
            .map(|(token, bonus)| {
                let ability = AbilityScores::ABILITIES
                    .into_iter()
                    .find(|ability| ability_token(*ability) == *token)
                    .ok_or_else(|| {
                        ToolError::invalid_arguments(format!(
                            "`assume.{what}`: {token:?} isn't an ability. Give `str`, `dex`, `con`, `int`, `wis` or \
                             `cha`."
                        ))
                    })?;
                if !(1..=highest).contains(bonus) {
                    return Err(ToolError::invalid_arguments(format!(
                        "`assume.{what}.{token}`: {bonus} is outside 1 to {highest}, the guide's range."
                    )));
                }
                Ok((ability, *bonus))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()
    };
    let at_most = |what: &str, value: u8, highest: u8| {
        if value > highest {
            Err(ToolError::invalid_arguments(format!(
                "`assume.{what}`: {value} is above {highest}, the guide's highest."
            )))
        } else {
            Ok(value)
        }
    };
    let items = assume
        .items
        .iter()
        .map(|key| {
            let key = GearKey::new(key.as_str());
            if engine.gear_item(&key).is_some() {
                Ok(key)
            } else {
                let known: Vec<&str> = engine
                    .gear()
                    .iter()
                    .map(|item| item.key.as_str())
                    .collect();
                Err(ToolError::invalid_arguments(format!(
                    "`assume.items`: {:?} isn't an item of the guide's caster level and spell DC tables. Give one \
                     of: {}.",
                    key.as_str(),
                    known.join(", ")
                )))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(Assumed {
        enhancement: abilities("enhancement", &assume.enhancement, HIGHEST_ENHANCEMENT)?,
        inherent: abilities("inherent", &assume.inherent, HIGHEST_INHERENT)?,
        weapon_enhancement: at_most(
            "weapon_enhancement",
            assume.weapon_enhancement,
            HIGHEST_WEAPON_ENHANCEMENT,
        )?,
        resistance: at_most("resistance", assume.resistance, HIGHEST_RESISTANCE)?,
        items,
    }))
}

/// The numbers at a character state with the assumed gear: the attack line for a `weapon`, the caster line for a
/// `spell`, and with `with_totals` the totals.
pub(crate) fn assumed(
    engine: &Engine,
    start: &BuildStart,
    state: &CharacterState,
    assumed: &Assumed,
    weapon: Option<&WeaponKey>,
    spell: Option<SpellTraits>,
    with_totals: bool,
) -> AssumedShape {
    let names = |map: &BTreeMap<rules::Ability, u8>| {
        map.iter()
            .map(|(ability, bonus)| (ability_token(*ability), *bonus))
            .collect()
    };
    AssumedShape {
        assumes: AssumesShape {
            enhancement: names(&assumed.enhancement),
            inherent: names(&assumed.inherent),
            weapon_enhancement: assumed.weapon_enhancement,
            resistance: assumed.resistance,
            items: assumed
                .items
                .iter()
                .filter_map(|key| engine.gear_item(key))
                .map(|item| Named {
                    key: item.key.to_string(),
                    name: item.name.clone(),
                })
                .collect(),
        },
        scores: crate::sheet::scores(assumed.scores(state.ability_scores)),
        attack: weapon.and_then(|weapon| {
            engine
                .assumed_attack_line(state, weapon, assumed)
                .map(|line| attack::shape(engine, weapon, &line))
        }),
        caster: spell.and_then(|spell| {
            engine
                .assumed_caster_line(start, state, spell, assumed)
                .map(|line| caster::shape(engine, &line))
        }),
        totals: with_totals
            .then(|| totals::shape_of(&engine.assumed_totals(start, state, assumed))),
    }
}

/// A fixed leaves-out list as the server gives it: its gear entry says gear is left out unless assumed.
pub(crate) fn unless_assumed(list: &[&str]) -> Vec<String> {
    list.iter()
        .map(|item| {
            if item.starts_with("gear") {
                format!("{item} (unless assumed: `assume`, shown apart in `assumed`)")
            } else {
                (*item).to_owned()
            }
        })
        .collect()
}
