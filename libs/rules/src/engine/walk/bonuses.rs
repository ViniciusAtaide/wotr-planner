//! The Item bonuses in force at the end of a level (ticket 151): which the character holds, through the Build start
//! (race, subrace, background) or the state (feats, mythic feats and abilities, features, options), whether each
//! meets its `requires`, and what each adds there. The walk records them on the sheet's states, so the caster line
//! and the totals read a level of the sheet alone. The bonuses of the Build's gear follow them at every level
//! (ticket 166; worked out in `gear`).
//!
//! The walk notes each bonus whose item it holds as it holds it ([`Walker::note_bonuses`]), so a level's end looks
//! only at those, not at every bonus of the data. A level's end then keeps the last level's list while what it finds
//! matches it entry by entry (most levels: it shares that list), and builds a new one only from the first entry
//! that differs, reusing the last level's entry (its name) for each bonus that only changed its amount or stayed.

use std::cell::OnceCell;
use std::collections::BTreeMap;

use super::{Held, Walker};
use crate::bonus::{
    BonusAmount, BonusInForce, BonusItem, BonusItemKind, BonusSource, BonusTarget, BonusType,
    InForceTarget, ItemBonus, SpellScope,
};
use crate::parameter::{Parameter, SpellSchool};

/// Where a bonus in force comes from: its position in the data's bonuses (a gear item's bonus: past the data's
/// last, in the order the Build holds them), and for Expanded Arsenal the school whose Spell Focus feats it
/// copies. With the entry's value, it says which bonus an entry is; the walk keeps one
/// beside each entry of the state's list (`Walker::bonus_keys`), so a level's end compares without the item's key.
pub(super) type BonusKey = (usize, Option<SpellSchool>);

impl Walker<'_> {
    /// Notes the bonuses an item gives as the walk holds it, so the level's end looks at them.
    pub(super) fn note_bonuses(&mut self, item: &Held) {
        let (kind, key) = match item {
            Held::Feat(key, _) => (BonusItemKind::Feat, key.as_str()),
            Held::Feature(key) => (BonusItemKind::Feature, key.as_str()),
            Held::SelectionOption(key, _) => (BonusItemKind::SelectionOption, key.as_str()),
            Held::MythicAbility(key) => (BonusItemKind::MythicAbility, key.as_str()),
            Held::MythicFeat(key, _) => (BonusItemKind::MythicFeat, key.as_str()),
            Held::ClassSkill(_) => return,
        };
        self.note(kind, key);
    }

    /// Notes the bonuses of the Build start's race or subrace and background: those of its subrace when the subrace
    /// is the race's (they replace the race's, as its ability modifiers do: `Catalog::racial_modifiers`), the race's
    /// otherwise. The start is never released, so these stay held.
    pub(super) fn note_start_bonuses(&mut self) {
        let (engine, start) = (self.engine, &self.build.start);
        self.bonuses_noted.clear();
        let own_subrace = start.subrace.as_ref().filter(|key| {
            (engine.catalog.subrace(key))
                .is_some_and(|(owner, _)| start.race.as_ref() == Some(&owner.key))
        });
        let racial = match own_subrace {
            Some(subrace) => Some((BonusItemKind::Subrace, subrace.as_str())),
            None => (start.race.as_ref()).map(|race| (BonusItemKind::Race, race.as_str())),
        };
        let background =
            (start.background.as_ref()).map(|key| (BonusItemKind::Background, key.as_str()));
        for (kind, key) in [racial, background].into_iter().flatten() {
            self.note(kind, key);
        }
    }

    /// Adds an item's bonuses to those noted, in the data's order.
    fn note(&mut self, kind: BonusItemKind, key: &str) {
        let engine = self.engine;
        for &row in engine.catalog.bonus_rows(kind, key) {
            if let Err(at) = self.bonuses_noted.binary_search(&row) {
                self.bonuses_noted.insert(at, row);
            }
        }
    }

    /// Records the bonuses in force on the state ([`CharacterState::bonuses_in_force`]): the Item bonuses noted
    /// ([`note_bonuses`](Self::note_bonuses)) whose item is held and whose `requires` holds, in the data's order,
    /// once per value the item is held for, with what each adds; then the gear's (ticket 166). The last level's list
    /// stands while each entry found matches its own; from the first that doesn't, a new list.
    ///
    /// [`CharacterState::bonuses_in_force`]: crate::CharacterState::bonuses_in_force
    pub(super) fn record_bonuses(&mut self) {
        let bonuses = &self.engine.catalog.data().bonuses;
        let (last, last_keys) = (&self.state.bonuses_in_force, &self.bonus_keys);
        let mut keys = std::mem::take(&mut self.bonus_keys_next);
        keys.clear();
        // The new list, once an entry differs from the last level's.
        let mut new: Option<Vec<BonusInForce>> = None;
        // How far the search for a last level's entry to reuse has gone.
        let mut reuse_from = 0;
        // Adds the bonus `key` held for `value` with its amount: nothing while the last level's list still matches,
        // else onto the new list, as the last level's entry when it had one and as `fresh` otherwise.
        let mut add = |key: BonusKey,
                       value: Option<&Parameter>,
                       amount: i16,
                       fresh: &dyn Fn() -> BonusInForce| {
            let at = keys.len();
            keys.push(key);
            let list = match &mut new {
                Some(list) => list,
                None if last.get(at).is_some_and(|entry| {
                    last_keys[at] == key && entry.value.as_ref() == value && entry.amount == amount
                }) =>
                {
                    return;
                }
                None => new.insert(last[..at].to_vec()),
            };
            let entry = match reusable(last, last_keys, &mut reuse_from, key, value) {
                Some(entry) => BonusInForce {
                    amount,
                    ..entry.clone()
                },
                None => fresh(),
            };
            list.push(entry);
        };
        let focus = OnceCell::new();
        for &row in &self.bonuses_noted {
            let bonus = &bonuses[row];
            if !self.requirement_holds(bonus) {
                continue;
            }
            self.for_each_holding(&bonus.item, |value| {
                let (amount, copies) = if bonus.amount == BonusAmount::SpellFocusOfAnotherSchool {
                    // Expanded Arsenal's copy; one that copies nothing isn't in force.
                    let focus = focus.get_or_init(|| self.spell_focus_totals());
                    let Some((amount, from)) = expanded_arsenal(focus, value) else {
                        return;
                    };
                    (amount, Some(from))
                } else {
                    (self.amount(bonus, value), None)
                };
                add((row, copies), value, amount, &|| BonusInForce {
                    source: BonusSource::Item(bonus.item.clone()),
                    value: value.cloned(),
                    target: InForceTarget::Number(bonus.target.clone()),
                    amount,
                    kind: BonusType::Untyped,
                    name: self.bonus_name(&bonus.item, value, copies),
                });
            });
        }
        // The gear's bonuses, the same at every level, after the data's: each keyed past the data's last row.
        for (at, bonus) in self.gear.bonuses.iter().enumerate() {
            add((bonuses.len() + at, None), None, bonus.amount, &|| {
                bonus.clone()
            });
        }
        // The last level's list stands, or its first entries do (its last ones are no longer in force).
        let new = new.or_else(|| (keys.len() < last.len()).then(|| last[..keys.len()].to_vec()));
        if let Some(new) = new {
            self.state.bonuses_in_force = new.into();
        }
        self.bonus_keys_next = std::mem::replace(&mut self.bonus_keys, keys);
    }

    /// Whether a bonus's `requires` holds now (one without holds).
    fn requirement_holds(&self, bonus: &ItemBonus) -> bool {
        (bonus.requires.as_ref()).is_none_or(|requires| self.state.check(requires).is_ok())
    }

    /// What the Spell Focus feats held add up to for each school: the flat spell DC bonuses in force of the feats and
    /// mythic feats held for a school whose scope is the school they're held for. Expanded Arsenal copies them.
    fn spell_focus_totals(&self) -> BTreeMap<SpellSchool, i16> {
        let bonuses = &self.engine.catalog.data().bonuses;
        let mut totals = BTreeMap::new();
        for &row in &self.bonuses_noted {
            let bonus = &bonuses[row];
            let (BonusTarget::SpellDc(SpellScope::Value), BonusAmount::Flat(amount)) =
                (&bonus.target, bonus.amount)
            else {
                continue;
            };
            if !matches!(bonus.item, BonusItem::Feat(_) | BonusItem::MythicFeat(_))
                || !self.requirement_holds(bonus)
            {
                continue;
            }
            self.for_each_holding(&bonus.item, |value| {
                if let Some(Parameter::SpellSchool(school)) = value {
                    *totals.entry(*school).or_insert(0) += i16::from(amount);
                }
            });
        }
        totals
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
    /// (with the value), once for anything else held, never for what it doesn't hold. Only for an item noted: a Build
    /// start's item is held once noted ([`note_start_bonuses`](Self::note_start_bonuses)).
    fn for_each_holding<'s>(
        &'s self,
        item: &BonusItem,
        mut held: impl FnMut(Option<&'s Parameter>),
    ) {
        let state = &self.state;
        let holds = match item {
            BonusItem::Race(_) | BonusItem::Subrace(_) | BonusItem::Background(_) => true,
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

/// The last level's entry for the bonus `key` held for `value`, if it had one: the search goes on from `from`, as
/// the entries of both levels are in the data's order.
fn reusable<'l>(
    last: &'l [BonusInForce],
    last_keys: &[BonusKey],
    from: &mut usize,
    key: BonusKey,
    value: Option<&Parameter>,
) -> Option<&'l BonusInForce> {
    while last_keys.get(*from).is_some_and(|(row, _)| *row < key.0) {
        *from += 1;
    }
    (*from..last_keys.len())
        .take_while(|&at| last_keys[at].0 == key.0)
        .find(|&at| last_keys[at] == key && last[at].value.as_ref() == value)
        .map(|at| &last[at])
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
