//! The Item bonuses in force at the end of a level (ticket 151): which the character holds, through the Build start
//! (race, subrace, background) or the state (feats, mythic feats and abilities, features, options), whether each
//! meets its `requires`, and what each adds there. The walk records them on the sheet's states, so the caster line
//! and the totals read the state alone.
//!
//! The walk notes each bonus whose item it holds as it holds it ([`Walker::note_bonuses`]), so a level's end looks
//! only at those, not at every bonus of the data. A level whose bonuses are the last level's (most are) shares the
//! last level's list rather than building it again.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::{Held, Walker};
use crate::bonus::{BonusAmount, BonusInForce, BonusItem, BonusTarget, ItemBonus, SpellScope};
use crate::engine::catalog::BonusGiver;
use crate::parameter::{Parameter, SpellSchool};

/// A bonus in force as a level's end finds it, before it's named: its position in the data's bonuses, the value its
/// item is held for, what it adds, and for Expanded Arsenal the school whose Spell Focus feats it copies.
#[derive(Clone)]
pub(super) struct Found<V> {
    row: usize,
    value: Option<V>,
    amount: i16,
    copies: Option<SpellSchool>,
}

impl Walker<'_> {
    /// Notes the bonuses an item gives as the walk holds it, so the level's end looks at them.
    pub(super) fn note_bonuses(&mut self, item: &Held) {
        let giver = match item {
            Held::Feat(key, _) => BonusGiver::Feat(key),
            Held::Feature(key) => BonusGiver::Feature(key),
            Held::SelectionOption(key, _) => BonusGiver::SelectionOption(key),
            Held::MythicAbility(key) => BonusGiver::MythicAbility(key),
            Held::MythicFeat(key, _) => BonusGiver::MythicFeat(key),
            Held::ClassSkill(_) => return,
        };
        for row in self.engine.catalog.bonus_rows(giver) {
            self.bonuses_noted[*row] = true;
        }
    }

    /// Notes the bonuses of the race, subrace and background the Build start names.
    pub(super) fn note_start_bonuses(&mut self) {
        let catalog = &self.engine.catalog;
        let start = &self.build.start;
        let givers = [
            start.race.as_ref().map(BonusGiver::Race),
            start.subrace.as_ref().map(BonusGiver::Subrace),
            start.background.as_ref().map(BonusGiver::Background),
        ];
        self.bonuses_noted = vec![false; catalog.data().bonuses.len()];
        for giver in givers.into_iter().flatten() {
            for row in catalog.bonus_rows(giver) {
                self.bonuses_noted[*row] = true;
            }
        }
    }

    /// Records the Item bonuses in force on the state ([`CharacterState::bonuses_in_force`]): the last level's list
    /// when they're the same, a new one otherwise.
    ///
    /// [`CharacterState::bonuses_in_force`]: crate::CharacterState::bonuses_in_force
    pub(super) fn record_bonuses(&mut self) {
        let found = self.find_bonuses();
        let same = found.len() == self.bonuses_found.len()
            && found.iter().zip(&self.bonuses_found).all(|(new, old)| {
                (new.row, new.value, new.amount, new.copies)
                    == (old.row, old.value.as_ref(), old.amount, old.copies)
            });
        if same {
            return;
        }
        let bonuses = &self.engine.catalog.data().bonuses;
        let recorded: Arc<[BonusInForce]> = found
            .iter()
            .map(|found| {
                let bonus = &bonuses[found.row];
                BonusInForce {
                    item: bonus.item.clone(),
                    value: found.value.cloned(),
                    target: bonus.target.clone(),
                    amount: found.amount,
                    name: self.bonus_name(&bonus.item, found.value, found.copies),
                }
            })
            .collect();
        let owned = found
            .into_iter()
            .map(|found| Found {
                value: found.value.cloned(),
                row: found.row,
                amount: found.amount,
                copies: found.copies,
            })
            .collect();
        self.bonuses_found = owned;
        self.state.bonuses_in_force = recorded;
    }

    /// The bonuses in force now, in the data's order: those noted ([`note_bonuses`](Self::note_bonuses)) whose item
    /// is held and whose `requires` holds, once per value the item is held for, with what each adds.
    fn find_bonuses(&self) -> Vec<Found<&Parameter>> {
        let bonuses = &self.engine.catalog.data().bonuses;
        let mut found = Vec::new();
        let noted =
            (self.bonuses_noted.iter().enumerate()).filter_map(|(row, noted)| noted.then_some(row));
        for row in noted {
            let bonus = &bonuses[row];
            if bonus
                .requires
                .as_ref()
                .is_some_and(|requires| self.state.check(requires).is_err())
            {
                continue;
            }
            self.for_each_holding(&bonus.item, |value| {
                found.push(Found {
                    row,
                    value,
                    amount: self.amount(bonus, value),
                    copies: None,
                });
            });
        }
        // Expanded Arsenal's copies, once the Spell Focus feats held are known; one that copies nothing isn't in
        // force.
        if found
            .iter()
            .any(|found| bonuses[found.row].amount == BonusAmount::SpellFocusOfAnotherSchool)
        {
            let focus = spell_focus_totals(bonuses, &found);
            found.retain_mut(|found| {
                if bonuses[found.row].amount != BonusAmount::SpellFocusOfAnotherSchool {
                    return true;
                }
                let Some((amount, from)) = expanded_arsenal(&focus, found.value) else {
                    return false;
                };
                (found.amount, found.copies) = (amount, Some(from));
                true
            });
        }
        found
    }

    /// What a bonus held for `value` adds for the character now, from its own scores, its Mythic rank and the ranks of
    /// the one skill a ten-ranks bonus is for. Not for Expanded Arsenal's copy ([`expanded_arsenal`]).
    fn amount(&self, bonus: &ItemBonus, value: Option<&Parameter>) -> i16 {
        let state = &self.state;
        match bonus.amount {
            BonusAmount::Flat(amount) => i16::from(amount),
            BonusAmount::HalfMythicRank => i16::from(state.mythic_rank / 2),
            BonusAmount::MythicRank => i16::from(state.mythic_rank),
            // Worked out apart: `expanded_arsenal`.
            BonusAmount::SpellFocusOfAnotherSchool => 0,
            BonusAmount::TenRanks {
                amount,
                with_ten_ranks,
            } => {
                let skill = match (&bonus.target, value) {
                    (BonusTarget::Skill(skill), _)
                    | (BonusTarget::Held, Some(Parameter::Skill(skill))) => Some(skill),
                    _ => None,
                };
                let ranks = skill.map_or(0, |skill| state.prerequisite_skill_ranks(skill));
                i16::from(if ranks >= 10 { with_ten_ranks } else { amount })
            }
            BonusAmount::AbilityModifier(ability) => {
                i16::from(state.ability_scores.modifier(ability))
            }
        }
    }

    /// Calls `held` for each way the character holds an item: once per value for a parameterised feat or mythic feat
    /// (with the value), once for anything else held, never for what it doesn't hold. A race's item counts without
    /// a subrace of it, a subrace's with its own race.
    fn for_each_holding<'s>(
        &'s self,
        item: &BonusItem,
        mut held: impl FnMut(Option<&'s Parameter>),
    ) {
        let catalog = &self.engine.catalog;
        let (start, state) = (&self.build.start, &self.state);
        let holds = match item {
            BonusItem::Race(race) => {
                let own_subrace = start
                    .subrace
                    .as_ref()
                    .and_then(|key| catalog.subrace(key))
                    .is_some_and(|(owner, _)| &owner.key == race);
                start.race.as_ref() == Some(race) && !own_subrace
            }
            BonusItem::Subrace(subrace) => {
                start.subrace.as_ref() == Some(subrace)
                    && catalog
                        .subrace(subrace)
                        .is_some_and(|(owner, _)| start.race.as_ref() == Some(&owner.key))
            }
            BonusItem::Background(background) => start.background.as_ref() == Some(background),
            BonusItem::MythicAbility(ability) => state.mythic_abilities.contains(ability),
            BonusItem::Feat(feat) => match state.feat_values.get(feat) {
                Some(values) => {
                    values.keys().for_each(|value| held(Some(value)));
                    false
                }
                None => state.feat_count(feat) > 0,
            },
            BonusItem::MythicFeat(feat) => match state.mythic_feat_values.get(feat) {
                Some(values) => {
                    values.iter().for_each(|value| held(Some(value)));
                    false
                }
                None => state.mythic_feats.contains(feat),
            },
            BonusItem::Feature(feature) => state.feature_count(feature) > 0,
            BonusItem::SelectionOption(option) => state.option_count(option) > 0,
        };
        if holds {
            held(None);
        }
    }

    /// A bonus's part name: its item's, with the value it's held for ("Spell Focus (Evocation)"), a selection
    /// option's with its selection's ("School Power (Evocation)"); for Expanded Arsenal, the school it copies.
    fn bonus_name(
        &self,
        item: &BonusItem,
        value: Option<&Parameter>,
        copies: Option<SpellSchool>,
    ) -> String {
        let engine = self.engine;
        let name = match item {
            BonusItem::Race(race) => engine.race_name(race).to_owned(),
            BonusItem::Subrace(subrace) => engine.subrace_name(subrace).to_owned(),
            BonusItem::Background(background) => engine.background_name(background).to_owned(),
            BonusItem::MythicAbility(ability) => engine.mythic_ability_name(ability).to_owned(),
            BonusItem::Feat(feat) => engine.feat_name(feat).to_owned(),
            BonusItem::MythicFeat(feat) => engine.mythic_feat_name(feat).to_owned(),
            BonusItem::Feature(feature) => engine.feature_name_apart(feature),
            BonusItem::SelectionOption(option) => match engine.catalog.selection_of(option) {
                Some(selection) => format!("{} ({})", selection.name, engine.option_name(option)),
                None => engine.option_name(option).to_owned(),
            },
        };
        let name = match value {
            Some(value) => format!("{name} ({})", engine.parameter_name(value)),
            None => name,
        };
        match copies {
            Some(from) => format!("{name}: {}'s Spell Focus feats", from.name()),
            None => name,
        }
    }
}

/// What the Spell Focus feats held add up to for each school: the flat spell DC bonuses of the feats and mythic
/// feats held for a school whose scope is the school they're held for.
fn spell_focus_totals(
    bonuses: &[ItemBonus],
    found: &[Found<&Parameter>],
) -> BTreeMap<SpellSchool, i16> {
    let mut totals = BTreeMap::new();
    for found in found {
        let bonus = &bonuses[found.row];
        let (BonusTarget::SpellDc(SpellScope::Value), BonusAmount::Flat(amount)) =
            (&bonus.target, bonus.amount)
        else {
            continue;
        };
        if !matches!(bonus.item, BonusItem::Feat(_) | BonusItem::MythicFeat(_)) {
            continue;
        }
        if let Some(Parameter::SpellSchool(school)) = found.value {
            *totals.entry(*school).or_insert(0) += i16::from(amount);
        }
    }
    totals
}

/// Expanded Arsenal held for `value`: what the Spell Focus feats of the school where they add up to the most (the
/// first such school in school order) add, and that school, when the school it's held for has none of its own;
/// `None` otherwise, when it adds nothing.
fn expanded_arsenal(
    focus: &BTreeMap<SpellSchool, i16>,
    value: Option<&Parameter>,
) -> Option<(i16, SpellSchool)> {
    let Some(Parameter::SpellSchool(school)) = value else {
        return None;
    };
    if focus.get(school).is_some_and(|total| *total > 0) {
        return None;
    }
    let (from, total) = focus.iter().filter(|(other, _)| *other != school).fold(
        None::<(SpellSchool, i16)>,
        |best, (other, total)| match best {
            Some((_, most)) if most >= *total => best,
            _ => Some((*other, *total)),
        },
    )?;
    (total > 0).then_some((total, from))
}
