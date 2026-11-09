//! The caster line `sheet` and `compare` add for an optional `caster` (ticket 119): the engine's
//! [`rules::Engine::caster_line`] at a character state, as JSON. The spell it describes is no part of the Build or its
//! code.

use rules::{
    BuildStart, CASTER_LINE_LEAVES_OUT, CasterPart, CharacterState, Energy, Engine, SpellDc,
    SpellSchool, SpellTraits, Subschool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ToolError;
use crate::shapes::Named;

/// The spells a caster line is for: a school, and optionally a subschool, an energy type and metamagic.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct CasterArgs {
    /// The school, in lower case: `abjuration`, `conjuration`, `divination`, `enchantment`, `evocation`,
    /// `illusion`, `necromancy` or `transmutation`.
    pub school: String,
    /// The subschool, in lower case, one of the school's (`compulsion`, `charm`, `summoning`, `phantasm`, …): the
    /// Fey and Infernal arcana count only with it.
    #[serde(default)]
    pub subschool: Option<String>,
    /// The energy type the spell deals: `acid`, `cold`, `electricity` or `fire` (Elemental Focus, a gnome's or an
    /// oread's racial DC).
    #[serde(default)]
    pub energy: Option<String>,
    /// Whether a metamagic feat other than Heighten Spell is applied (the Arcane bloodline's arcana).
    #[serde(default)]
    pub metamagic: bool,
}

/// A spell's numbers at a level, from the planner's engine: unconditional numbers only.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct CasterLineShape {
    /// The casting class the line follows: the one with the highest caster level.
    pub class: Named,
    /// The ability its spells use ("Charisma").
    pub ability: String,
    /// Its caster level: class levels plus the prestige levels that advance it.
    pub caster_level: u8,
    /// The save DC of a spell of each level castable, from the 1st to the highest: 10 + the spell level + `dc_counts`.
    pub dcs: Vec<SpellDcShape>,
    /// With Heighten Spell held: the DC of any of these spells raised to the highest level castable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heightened: Option<SpellDcShape>,
    /// What every DC counts besides 10 and the spell level: the casting ability's modifier, then each item's bonus.
    pub dc_counts: Vec<CasterPartShape>,
    /// Caster level checks against spell resistance: `caster_level` plus `spell_penetration_counts`.
    pub spell_penetration: i16,
    pub spell_penetration_counts: Vec<CasterPartShape>,
    /// The concentration bonus: `caster_level` plus `concentration_counts` (the ability's modifier, then each bonus).
    pub concentration: i16,
    pub concentration_counts: Vec<CasterPartShape>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct SpellDcShape {
    pub spell_level: u8,
    pub dc: i16,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct CasterPartShape {
    pub name: String,
    pub value: i16,
}

/// The spell a `caster` argument describes: `None` without one, refused when a value isn't known or the subschool
/// isn't the school's.
pub(crate) fn caster_argument(
    caster: Option<&CasterArgs>,
) -> Result<Option<SpellTraits>, ToolError> {
    let Some(caster) = caster else {
        return Ok(None);
    };
    let named = |what: &str, given: &str, known: Vec<&'static str>| {
        ToolError::invalid_arguments(format!(
            "`caster.{what}`: {given:?} isn't one. Give one of: {}.",
            known.join(", ")
        ))
    };
    let lower = |name: &str| name.to_lowercase();
    let school = SpellSchool::ALL
        .into_iter()
        .find(|school| lower(school.name()) == caster.school)
        .ok_or_else(|| {
            named(
                "school",
                &caster.school,
                SpellSchool::ALL.map(SpellSchool::name).to_vec(),
            )
        })?;
    let subschool = match &caster.subschool {
        None => None,
        Some(given) => {
            let subschool = Subschool::ALL
                .into_iter()
                .find(|subschool| lower(subschool.name()) == *given)
                .ok_or_else(|| {
                    named(
                        "subschool",
                        given,
                        Subschool::ALL.map(Subschool::name).to_vec(),
                    )
                })?;
            if subschool.school() != school {
                return Err(ToolError::invalid_arguments(format!(
                    "`caster.subschool`: {given:?} is a subschool of {}, not of {}.",
                    subschool.school().name(),
                    school.name()
                )));
            }
            Some(subschool)
        }
    };
    let energy = match &caster.energy {
        None => None,
        Some(given) => Some(
            Energy::ALL
                .into_iter()
                .find(|energy| lower(energy.name()) == *given)
                .ok_or_else(|| named("energy", given, Energy::ALL.map(Energy::name).to_vec()))?,
        ),
    };
    Ok(Some(SpellTraits {
        school,
        subschool,
        energy,
        metamagic: caster.metamagic,
    }))
}

fn dc(dc: &SpellDc) -> SpellDcShape {
    SpellDcShape {
        spell_level: dc.spell_level,
        dc: dc.dc,
    }
}

fn counts(parts: &[CasterPart]) -> Vec<CasterPartShape> {
    parts
        .iter()
        .map(|part| CasterPartShape {
            name: part.name.clone(),
            value: part.value,
        })
        .collect()
}

/// The caster line for a spell at a character state; `None` for a character that casts no spells yet.
pub(crate) fn caster_line(
    engine: &Engine,
    start: &BuildStart,
    state: &CharacterState,
    spell: SpellTraits,
) -> Option<CasterLineShape> {
    let line = engine.caster_line(start, state, spell)?;
    Some(CasterLineShape {
        class: Named {
            key: line.class.to_string(),
            name: engine.class_name(&line.class).to_owned(),
        },
        ability: line.ability.name().to_owned(),
        caster_level: line.caster_level,
        dcs: line.dcs.iter().map(dc).collect(),
        heightened: line.heightened.as_ref().map(dc),
        dc_counts: counts(&line.dc_parts),
        spell_penetration: line.spell_penetration,
        spell_penetration_counts: counts(&line.spell_penetration_parts),
        concentration: line.concentration,
        concentration_counts: counts(&line.concentration_parts),
    })
}

/// What no caster line counts, for a result that has one.
pub(crate) fn leaves_out() -> Vec<String> {
    CASTER_LINE_LEAVES_OUT
        .iter()
        .map(|item| (*item).to_owned())
        .collect()
}
