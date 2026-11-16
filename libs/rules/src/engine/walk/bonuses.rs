//! The bonuses in force at the end of a level (ticket 151): what the components of the entries the character holds
//! (ticket 297) and the Build's gear (ticket 166) add to the numbers the caster line and the totals read, each worked
//! out there. The walk records them on the sheet's states, so the lines and the totals read a level of the sheet
//! alone. A level whose list is the last one's shares it.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::Walker;
use crate::bonus::{BonusInForce, BonusSource, BonusTarget, InForceTarget, SpellScope};
use crate::component::{Component, Descriptor, Fact, Holder, StatTarget, stat_bonus_amount};
use crate::engine::catalog::HolderKind;
use crate::parameter::{Parameter, SpellSchool};

/// A Spell Focus held for a school, before the Expanded Arsenals held decide what it adds.
struct Focus<'s> {
    holder: &'s Holder,
    value: &'s Parameter,
    school: SpellSchool,
    bonus: i8,
    mythic_focus: &'s Fact,
}

/// An Expanded Arsenal held for a school, with the Spell Focus feats it adds up.
struct Arsenal<'s> {
    holder: &'s Holder,
    value: &'s Parameter,
    focuses: &'s BTreeSet<Fact>,
}

impl Walker<'_> {
    /// Records the bonuses in force on the state ([`CharacterState::bonuses_in_force`]): the components' ([`in_force`]),
    /// then the gear's, in the order the Build holds its items. The last level's list stands when the new one is the
    /// same.
    ///
    /// [`CharacterState::bonuses_in_force`]: crate::CharacterState::bonuses_in_force
    /// [`in_force`]: Self::in_force
    pub(super) fn record_bonuses(&mut self) {
        let mut list = if self.engine.catalog.has_components() {
            self.in_force()
        } else {
            Vec::new()
        };
        list.extend_from_slice(&self.gear.in_force);
        if *self.state.bonuses_in_force != *list {
            self.state.bonuses_in_force = Arc::from(list);
        }
    }

    /// The bonuses in force the components of the entries held give, once for each value a parameterised entry is held
    /// for: a stat bonus to a save, initiative or a skill; a spell DC, spell penetration and concentration bonus; and the
    /// spell DCs of Spell Focus and Expanded Arsenal ([`spell_focus`](Self::spell_focus)). In a fixed order: the race (or
    /// subrace), the background, the feats, mythic feats, mythic abilities, features and options, then the other
    /// entries, each kind by key ([`HolderKind::bonus_rank`]); an entry's in the order of its components.
    fn in_force(&self) -> Vec<BonusInForce> {
        let state = &self.state;
        let mut list: Vec<(&Holder, BonusInForce)> = Vec::new();
        let (mut focus, mut arsenal) = (Vec::new(), Vec::new());
        let entry =
            |holder: &Holder, value: Option<&Parameter>, target, amount, descriptor| BonusInForce {
                source: BonusSource::Component(holder.clone()),
                value: value.cloned(),
                target: InForceTarget::Number(target),
                amount,
                descriptor,
                name: self.part_name(holder, value),
            };
        self.each_component(|holder, component, value, context| {
            let school = match value {
                Some(Parameter::SpellSchool(school)) => Some(*school),
                _ => None,
            };
            let (target, amount, descriptor) = match component {
                Component::StatBonus {
                    stat,
                    value: amount,
                    descriptor,
                    multiplier,
                    at_least,
                    ..
                } => {
                    let target = match stat.resolve(value) {
                        Some(StatTarget::Save(save)) => BonusTarget::Save(save),
                        Some(StatTarget::Initiative) => BonusTarget::Initiative,
                        Some(StatTarget::Skill(skill)) => BonusTarget::Skill(skill),
                        // A stat with no line, or one the state keeps (a score, the base attack bonus).
                        _ => return,
                    };
                    let amount = stat_bonus_amount(amount, *multiplier, *at_least, context);
                    (target, amount, *descriptor)
                }
                Component::SpellDc {
                    scope,
                    value: amount,
                } => {
                    let scope = match (scope, value) {
                        (SpellScope::Value, Some(Parameter::SpellSchool(school))) => {
                            SpellScope::School(*school)
                        }
                        (SpellScope::Value, Some(Parameter::Energy(energy))) => {
                            SpellScope::Energy(*energy)
                        }
                        (SpellScope::Value, _) => return,
                        (scope, _) => *scope,
                    };
                    let amount = stat_bonus_amount(amount, 1, None, context);
                    (
                        BonusTarget::SpellDc(scope),
                        amount,
                        Descriptor::UntypedStackable,
                    )
                }
                Component::SpellPenetration {
                    value: amount,
                    only_while,
                } => {
                    let holds = only_while.as_ref().is_none_or(|condition| {
                        let catalog = &self.engine.catalog;
                        let compiled =
                            condition.compile(&|archetype| catalog.archetype_class(archetype));
                        state.check(&compiled).is_ok()
                    });
                    if !holds {
                        return;
                    }
                    let amount = stat_bonus_amount(amount, 1, None, context);
                    (
                        BonusTarget::SpellPenetration,
                        amount,
                        Descriptor::UntypedStackable,
                    )
                }
                Component::Concentration(amount) => (
                    BonusTarget::Concentration,
                    stat_bonus_amount(amount, 1, None, context),
                    Descriptor::Untyped,
                ),
                Component::SpellFocus {
                    bonus,
                    mythic_focus,
                } => {
                    if let (Some(school), Some(value)) = (school, value) {
                        focus.push(Focus {
                            holder,
                            value,
                            school,
                            bonus: *bonus,
                            mythic_focus,
                        });
                    }
                    return;
                }
                Component::ExpandedArsenal { focuses } => {
                    if let (Some(_), Some(value)) = (school, value) {
                        arsenal.push(Arsenal {
                            holder,
                            value,
                            focuses,
                        });
                    }
                    return;
                }
                _ => return,
            };
            list.push((holder, entry(holder, value, target, amount, descriptor)));
        });
        list.extend(self.spell_focus(&focus, &arsenal));
        // A stable sort: an entry's keep their order.
        list.sort_by(|(a, _), (b, _)| holder_order(a).cmp(&holder_order(b)));
        list.into_iter().map(|(_, bonus)| bonus).collect()
    }

    /// The spell DC bonuses of the Spell Focus feats and the Expanded Arsenals held, each for the school it adds to
    /// (`SpellFocusParametrized.OnEventAboutToTrigger`, `GetBonusDC`; `ExpandedArsenalMagicSchools.OnTurnOn`,
    /// `OnEventAboutToTrigger`, WrathDecompiled 6ce4a6b):
    /// - a Spell Focus gives its bonus, doubled while its mythic focus is held for its school (or for any school while an
    ///   Expanded Arsenal names its school), to its school, unless an Expanded Arsenal names that school;
    /// - each Expanded Arsenal gives each school an Expanded Arsenal names the most its focuses held for one other school
    ///   add up to (the first such school in school order names the part), when there are any: each fact's handler
    ///   checks the shared list of schools, so two add it twice.
    fn spell_focus<'s>(
        &self,
        focus: &[Focus<'s>],
        arsenal: &[Arsenal<'s>],
    ) -> Vec<(&'s Holder, BonusInForce)> {
        let state = &self.state;
        let schools: BTreeSet<SpellSchool> = (arsenal.iter())
            .filter_map(|held| match held.value {
                Parameter::SpellSchool(school) => Some(*school),
                _ => None,
            })
            .collect();
        let amount = |held: &Focus<'_>| {
            let for_school = Parameter::SpellSchool(held.school);
            let mythic_held_for = |value: &Parameter| match held.mythic_focus {
                Fact::Feat(key) => state.has_feat_for(key, value),
                Fact::MythicFeat(key) => state.has_mythic_feat_for(key, value),
                _ => false,
            };
            let doubled = mythic_held_for(&for_school)
                || (schools.contains(&held.school) && held.mythic_focus.rank(state) > 0);
            i16::from(held.bonus) * if doubled { 2 } else { 1 }
        };
        let bonus = |holder: &'s Holder, value, school, amount, name| {
            let bonus = BonusInForce {
                source: BonusSource::Component(holder.clone()),
                value: Some(Parameter::clone(value)),
                target: InForceTarget::Number(BonusTarget::SpellDc(SpellScope::School(school))),
                amount,
                descriptor: Descriptor::UntypedStackable,
                name,
            };
            (holder, bonus)
        };
        let mut out = Vec::new();
        for held in focus.iter().filter(|held| !schools.contains(&held.school)) {
            let name = self.part_name(held.holder, Some(held.value));
            out.push(bonus(
                held.holder,
                held.value,
                held.school,
                amount(held),
                name,
            ));
        }
        for held in arsenal {
            for &school in &schools {
                let mut by_school: BTreeMap<SpellSchool, i16> = BTreeMap::new();
                for other in focus.iter().filter(|other| other.school != school) {
                    if let Holder::Fact(fact) = other.holder
                        && held.focuses.contains(fact)
                    {
                        *by_school.entry(other.school).or_default() += amount(other);
                    }
                }
                let most = (by_school.into_iter()).fold(
                    None::<(SpellSchool, i16)>,
                    |best, next| match best {
                        Some((_, top)) if top >= next.1 => best,
                        _ => Some(next),
                    },
                );
                if let Some((from, total)) = most {
                    let name = format!(
                        "{}: {}'s Spell Focus feats",
                        self.part_name(held.holder, Some(held.value)),
                        from.name()
                    );
                    out.push(bonus(held.holder, held.value, school, total, name));
                }
            }
        }
        out
    }

    /// A part's name: its entry's, a selection option's with its selection's ("School Power (Evocation)"), with the
    /// value it's held for ("Spell Focus (Evocation)").
    fn part_name(&self, holder: &Holder, value: Option<&Parameter>) -> String {
        let engine = self.engine;
        let name = match holder {
            Holder::Fact(fact) => match fact {
                Fact::Feature(key) => engine.feature_name_for(key, &self.state),
                Fact::Option(key) => match engine.catalog.selection_of(key) {
                    Some(selection) => format!("{} ({})", selection.name, engine.option_name(key)),
                    None => engine.option_name(key).to_owned(),
                },
                other => engine.fact_name(other).to_owned(),
            },
            Holder::Subrace(key) => engine.subrace_name(key).to_owned(),
            Holder::Background(key) => engine.background_name(key).to_owned(),
        };
        match value {
            Some(value) => format!("{name} ({})", engine.parameter_name(value)),
            None => name,
        }
    }
}

/// Where an entry's bonuses go in the bonuses in force ([`HolderKind::bonus_rank`]), then by key.
fn holder_order(holder: &Holder) -> (u8, &str) {
    let (kind, key) = HolderKind::of(holder);
    (kind.bonus_rank(), key)
}
