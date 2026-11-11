//! The caster line: a spell's save DC at each spell level, caster level against spell resistance and the
//! concentration bonus, at one point of a Build (ticket 119).
//!
//! Unconditional numbers only, from what the Build fixes: the casting class's caster level and casting ability
//! (the character's own score), and the Item bonuses in force ([`BonusInForce`], from `bonuses.ron`): Spell Focus
//! and its kin, racial DCs, bloodline arcana, School Power, Spell Penetration, Combat Casting. Everything else is
//! [`CASTER_LINE_LEAVES_OUT`]. The line is for a spell the caller describes ([`SpellTraits`]: a school, and
//! optionally a subschool, an energy type and metamagic), so no spell catalogue is needed.

use crate::bonus::{BonusInForce, BonusTarget, SpellTraits};
use crate::game_data::Ability;
use crate::keys::{ClassKey, FeatKey};
use crate::state::CharacterState;

use super::walk::advancing_levels;
use super::{Engine, class_level_cap};

/// A spell's numbers at one point of a Build ([`Engine::caster_line`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CasterLine {
    /// The casting class the line follows: the one with the highest caster level (the first entered on a tie).
    pub class: ClassKey,
    /// The ability its spells use.
    pub ability: Ability,
    /// The spell the line is for.
    pub spell: SpellTraits,
    /// The class's caster level: its class levels plus the prestige levels that advance it.
    pub caster_level: u8,
    /// The save DC of a spell of each level the class can cast, from the 1st to the highest: 10 + the spell level +
    /// what [`dc_parts`](Self::dc_parts) add.
    pub dcs: Vec<SpellDc>,
    /// With Heighten Spell held: the DC of any spell of the line raised to the highest level castable (the same
    /// number as that level's row, reached from a lower-level spell).
    pub heightened: Option<SpellDc>,
    /// What every DC counts besides 10 and the spell level: the casting ability's modifier, then each bonus.
    pub dc_parts: Vec<CasterPart>,
    /// Caster level checks against spell resistance: the caster level plus [`spell_penetration_parts`].
    ///
    /// [`spell_penetration_parts`]: Self::spell_penetration_parts
    pub spell_penetration: i16,
    pub spell_penetration_parts: Vec<CasterPart>,
    /// The concentration bonus: the caster level plus [`concentration_parts`](Self::concentration_parts) (the
    /// casting ability's modifier, then each bonus).
    pub concentration: i16,
    pub concentration_parts: Vec<CasterPart>,
    /// What no caster line counts: [`CASTER_LINE_LEAVES_OUT`].
    pub leaves_out: &'static [&'static str],
}

/// One spell level's save DC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellDc {
    pub spell_level: u8,
    pub dc: i16,
}

/// One thing a caster line's number counts, with what it adds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CasterPart {
    /// "Charisma", or the item's name with its value ("Spell Focus (Enchantment)", "School Power (Enchantment)").
    pub name: String,
    pub value: i16,
}

/// What a caster line leaves out, the same for every line: it holds only the numbers a Build fixes whatever happens
/// in a fight.
pub const CASTER_LINE_LEAVES_OUT: &[&str] = &[
    "gear: ability-score items, and items raising DCs, caster level or concentration",
    "buffs and spells cast on the character",
    "abilities switched on in a fight: the Aeon's gaze, Demonic Rage and aspects, the Devil's decrees, Studied Target",
    "bonuses that grow with class level (Combat Casting Adept, Horn Call, Cold Specialization, Overwhelming \
     Penetration, Haunting Refrain, Blessed Script)",
    "caster level raised for one school (School Mastery) and casting defensively (Arcane Focus)",
    "what changes the target's roll rather than the DC (Persistent Spell, Favorable Magic, Ambuscading Spells)",
    "spell-like abilities, hexes, exploits, bombs and other class abilities with DCs of their own",
    "a second casting class: the line follows the one with the highest caster level",
];

const HEIGHTEN_SPELL: &str = "metamagic-heighten-spell";

/// The casting class a line follows, with its ability, caster level and highest spell level.
struct Casting {
    class: ClassKey,
    ability: Ability,
    caster_level: u8,
    spell_level: u8,
}

impl Engine {
    /// The caster line for a spell at a character state of the sheet: `None` for a character with no class that casts
    /// spells (or none castable yet).
    pub fn caster_line(&self, state: &CharacterState, spell: SpellTraits) -> Option<CasterLine> {
        let casting = self.casting(state)?;
        let parts = |adds: &dyn Fn(&BonusInForce) -> bool| {
            state
                .bonuses_in_force
                .iter()
                .filter(|bonus| adds(bonus))
                .map(|bonus| CasterPart {
                    name: bonus.name.clone(),
                    value: bonus.amount,
                })
                .collect::<Vec<_>>()
        };
        let ability = CasterPart {
            name: casting.ability.name().to_owned(),
            value: i16::from(state.ability_scores.modifier(casting.ability)),
        };
        let mut dc_parts = vec![ability.clone()];
        dc_parts.extend(parts(&|bonus| match bonus.target {
            BonusTarget::SpellDc(scope) => scope.applies(&spell, bonus.value.as_ref()),
            _ => false,
        }));
        let spell_penetration_parts = parts(&|bonus| bonus.target == BonusTarget::SpellPenetration);
        let mut concentration_parts = vec![ability];
        concentration_parts.extend(parts(&|bonus| bonus.target == BonusTarget::Concentration));

        let sum = |parts: &[CasterPart]| parts.iter().map(|part| part.value).sum::<i16>();
        let dc_bonus = sum(&dc_parts);
        let dc = |spell_level: u8| SpellDc {
            spell_level,
            dc: 10 + i16::from(spell_level) + dc_bonus,
        };
        let caster_level = i16::from(casting.caster_level);
        Some(CasterLine {
            class: casting.class,
            ability: casting.ability,
            spell,
            caster_level: casting.caster_level,
            dcs: (1..=casting.spell_level).map(dc).collect(),
            heightened: (state.feat_count(&FeatKey::new(HEIGHTEN_SPELL)) > 0)
                .then(|| dc(casting.spell_level)),
            spell_penetration: caster_level + sum(&spell_penetration_parts),
            spell_penetration_parts,
            concentration: caster_level + sum(&concentration_parts),
            concentration_parts,
            dc_parts,
            leaves_out: CASTER_LINE_LEAVES_OUT,
        })
    }

    /// The casting class with the highest caster level (the first entered on a tie) that can cast a spell now and
    /// has a casting ability.
    fn casting(&self, state: &CharacterState) -> Option<Casting> {
        let catalog = &self.catalog;
        let mut best: Option<Casting> = None;
        for entry in &state.classes {
            let Some((class, _)) = catalog.class(&entry.class) else {
                continue;
            };
            let archetype = entry.archetype.as_ref();
            let Some(ability) = catalog
                .archetype_of(class, archetype)
                .and_then(|archetype| archetype.casting_ability)
                .or(class.casting_ability)
            else {
                continue;
            };
            let (arcane, divine) = catalog.casts(class, archetype);
            let effective = entry.level.saturating_add(advancing_levels(
                catalog,
                state,
                &entry.class,
                arcane,
                divine,
            ));
            let Some(row) = catalog.row(class, archetype, effective.min(class_level_cap(class)))
            else {
                continue;
            };
            let spell_level = row.spell_levels.arcane.max(row.spell_levels.divine);
            if spell_level == 0 || best.as_ref().is_some_and(|b| b.caster_level >= effective) {
                continue;
            }
            best = Some(Casting {
                class: entry.class.clone(),
                ability,
                caster_level: effective,
                spell_level,
            });
        }
        best
    }
}
