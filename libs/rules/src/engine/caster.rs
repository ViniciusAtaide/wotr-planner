//! The caster line: a spell's save DC at each spell level, caster level against spell resistance and the
//! concentration bonus, at one point of a Build (ticket 119).
//!
//! Unconditional numbers only, from what the Build fixes: the casting class's caster level and casting ability
//! (the character's own score), and the flat bonuses of the items held ([`ItemBonus`], `bonuses.ron`): Spell Focus
//! and its kin, racial DCs, bloodline arcana, School Power, Spell Penetration, Combat Casting. Everything else is
//! [`CASTER_LINE_LEAVES_OUT`]. The line is for a spell the caller describes ([`SpellTraits`]: a school, and
//! optionally a subschool, an energy type and metamagic), so no spell catalogue is needed.

use std::collections::BTreeMap;

use crate::bonus::{BonusAmount, BonusItem, BonusTarget, ItemBonus, SpellScope, SpellTraits};
use crate::build::BuildStart;
use crate::game_data::Ability;
use crate::keys::{ClassKey, FeatKey, SkillKey};
use crate::parameter::{Parameter, SpellSchool};
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
    /// The caster line for a spell at a character state, the Build start giving the race and subrace: `None` for a
    /// character with no class that casts spells (or none castable yet).
    pub fn caster_line(
        &self,
        start: &BuildStart,
        state: &CharacterState,
        spell: SpellTraits,
    ) -> Option<CasterLine> {
        let casting = self.casting(state)?;
        let ability = modifier(state.ability_scores.get(casting.ability));
        let held = self.held_bonuses(start, state);

        let mut dc_parts = vec![CasterPart {
            name: casting.ability.name().to_owned(),
            value: ability,
        }];
        let focus = spell_focus_totals(&held);
        for (bonus, value, name) in &held {
            let BonusTarget::SpellDc(scope) = bonus.target else {
                continue;
            };
            if bonus.amount == BonusAmount::SpellFocusOfAnotherSchool {
                if let Some(part) = expanded_arsenal(&focus, spell, value.as_ref(), name) {
                    dc_parts.push(part);
                }
            } else if scope.applies(&spell, value.as_ref()) {
                dc_parts.push(CasterPart {
                    name: name.clone(),
                    value: amount(bonus.amount, state, None),
                });
            }
        }
        let flat = |target: BonusTarget| -> Vec<CasterPart> {
            held.iter()
                .filter(|(bonus, _, _)| bonus.target == target)
                .map(|(bonus, _, name)| CasterPart {
                    name: name.clone(),
                    value: amount(bonus.amount, state, None),
                })
                .collect()
        };
        let spell_penetration_parts = flat(BonusTarget::SpellPenetration);
        let mut concentration_parts = vec![dc_parts[0].clone()];
        concentration_parts.extend(flat(BonusTarget::Concentration));

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

    /// Every bonus the character holds, once for each value its item is held for, with the part's name; those whose
    /// `requires` the character doesn't meet are left out. In the data's order. Shared with [`Engine::totals`].
    pub(super) fn held_bonuses<'a>(
        &'a self,
        start: &BuildStart,
        state: &CharacterState,
    ) -> Vec<(&'a ItemBonus, Option<Parameter>, String)> {
        let mut held = Vec::new();
        for bonus in &self.catalog.data().bonuses {
            if bonus
                .requires
                .as_ref()
                .is_some_and(|requires| state.check(requires).is_err())
            {
                continue;
            }
            for (value, name) in self.holdings(&bonus.item, start, state) {
                held.push((bonus, value, name));
            }
        }
        held
    }

    /// How a character holds an item: once per value for a parameterised feat or mythic feat (with the value), once
    /// for anything else held, never for what it doesn't hold. Each with the part's name.
    fn holdings(
        &self,
        item: &BonusItem,
        start: &BuildStart,
        state: &CharacterState,
    ) -> Vec<(Option<Parameter>, String)> {
        let with_value = |name: &str, value: &Parameter| {
            (
                Some(value.clone()),
                format!("{name} ({})", self.parameter_name(value)),
            )
        };
        match item {
            BonusItem::Race(race) => {
                let own_subrace = start
                    .subrace
                    .as_ref()
                    .and_then(|key| self.catalog.subrace(key))
                    .is_some_and(|(owner, _)| &owner.key == race);
                if start.race.as_ref() == Some(race) && !own_subrace {
                    vec![(None, self.race_name(race).to_owned())]
                } else {
                    vec![]
                }
            }
            BonusItem::Subrace(subrace) => {
                let holds = start.subrace.as_ref() == Some(subrace)
                    && self
                        .catalog
                        .subrace(subrace)
                        .is_some_and(|(owner, _)| start.race.as_ref() == Some(&owner.key));
                if holds {
                    vec![(None, self.subrace_name(subrace).to_owned())]
                } else {
                    vec![]
                }
            }
            BonusItem::Background(background) => {
                if start.background.as_ref() == Some(background) {
                    vec![(None, self.background_name(background).to_owned())]
                } else {
                    vec![]
                }
            }
            BonusItem::MythicAbility(ability) => {
                if state.mythic_abilities.contains(ability) {
                    vec![(None, self.mythic_ability_name(ability).to_owned())]
                } else {
                    vec![]
                }
            }
            BonusItem::Feat(feat) => {
                let name = self.feat_name(feat);
                match state.feat_values.get(feat) {
                    Some(values) => values.keys().map(|value| with_value(name, value)).collect(),
                    None if state.feat_count(feat) > 0 => vec![(None, name.to_owned())],
                    None => vec![],
                }
            }
            BonusItem::MythicFeat(feat) => {
                let name = self.mythic_feat_name(feat);
                match state.mythic_feat_values.get(feat) {
                    Some(values) => values.iter().map(|value| with_value(name, value)).collect(),
                    None if state.mythic_feats.contains(feat) => vec![(None, name.to_owned())],
                    None => vec![],
                }
            }
            BonusItem::Feature(feature) => {
                if state.feature_count(feature) > 0 {
                    vec![(None, self.feature_name_apart(feature))]
                } else {
                    vec![]
                }
            }
            BonusItem::SelectionOption(option) => {
                if state.option_count(option) == 0 {
                    return vec![];
                }
                let name = match self.catalog.selection_of(option) {
                    Some(selection) => {
                        format!("{} ({})", selection.name, self.option_name(option))
                    }
                    None => self.option_name(option).to_owned(),
                };
                vec![(None, name)]
            }
        }
    }
}

/// What the Spell Focus feats held add up to for each school: the flat spell DC bonuses of the feats and mythic
/// feats held for a school whose scope is the school they're held for.
fn spell_focus_totals(
    held: &[(&ItemBonus, Option<Parameter>, String)],
) -> BTreeMap<SpellSchool, i16> {
    let mut totals = BTreeMap::new();
    for (bonus, value, _) in held {
        let (BonusTarget::SpellDc(SpellScope::Value), BonusAmount::Flat(amount)) =
            (&bonus.target, bonus.amount)
        else {
            continue;
        };
        if !matches!(bonus.item, BonusItem::Feat(_) | BonusItem::MythicFeat(_)) {
            continue;
        }
        if let Some(Parameter::SpellSchool(school)) = value {
            *totals.entry(*school).or_insert(0) += i16::from(amount);
        }
    }
    totals
}

/// Expanded Arsenal held for `value`, for a spell: the Spell Focus feats of the school where they add up to the
/// most (the first such school in school order), when the spell is of the school it's held for and that school has
/// none of its own; `None` otherwise.
fn expanded_arsenal(
    focus: &BTreeMap<SpellSchool, i16>,
    spell: SpellTraits,
    value: Option<&Parameter>,
    name: &str,
) -> Option<CasterPart> {
    let Some(Parameter::SpellSchool(school)) = value else {
        return None;
    };
    if *school != spell.school || focus.get(school).is_some_and(|total| *total > 0) {
        return None;
    }
    let (from, total) = focus.iter().filter(|(other, _)| *other != school).fold(
        None::<(SpellSchool, i16)>,
        |best, (other, total)| match best {
            Some((_, most)) if most >= *total => best,
            _ => Some((*other, *total)),
        },
    )?;
    (total > 0).then(|| CasterPart {
        name: format!("{name}: {}'s Spell Focus feats", from.name()),
        value: total,
    })
}

/// What a bonus adds for a character, from its own scores; for a skill bonus, `skill` is the skill whose ranks it may
/// rise with.
pub(super) fn amount(amount: BonusAmount, state: &CharacterState, skill: Option<&SkillKey>) -> i16 {
    match amount {
        BonusAmount::Flat(value) => i16::from(value),
        BonusAmount::HalfMythicRank => i16::from(state.mythic_rank / 2),
        BonusAmount::MythicRank => i16::from(state.mythic_rank),
        // Only Expanded Arsenal, which `caster_line` works out apart.
        BonusAmount::SpellFocusOfAnotherSchool => 0,
        BonusAmount::TenRanks {
            amount,
            with_ten_ranks,
        } => {
            let ranks = skill.map_or(0, |skill| state.prerequisite_skill_ranks(skill));
            i16::from(if ranks >= 10 { with_ten_ranks } else { amount })
        }
        BonusAmount::AbilityModifier(ability) => modifier(state.ability_scores.get(ability)),
    }
}

/// An ability score's modifier: −1 for 8 or 9, +7 for 24 or 25.
pub(super) fn modifier(score: u8) -> i16 {
    (i16::from(score) - 10).div_euclid(2)
}
