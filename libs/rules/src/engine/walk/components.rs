//! The components the walk evaluates while their entry is held (ticket 295): the grants hold their facts through the
//! walk's own holds, as a level row's do; the conditional grants are looked at again at every level; the conditional
//! stat bonuses on the events the game's component handles; and the numbers (stat bonuses, skill points, the
//! prerequisite modifiers) are worked out from the state whenever anything is held or released, or the level changes,
//! since a rank or a fact a value reads may have changed. The catalog says which entries have any
//! ([`Catalog::has_components`]): for data without, none of this runs.
//!
//! [`Catalog::has_components`]: crate::engine::catalog::Catalog

use std::collections::BTreeMap;

use super::{Held, Walker};
use crate::bonus::{BonusInForce, BonusSource, BonusTarget, InForceTarget};
use crate::build::AbilityScores;
use crate::component::{
    Component, Descriptor, Fact, Holder, IfHeld, MechanicsFlag, StatTarget, ValueContext,
    game_stacked, grant_level, grants_at,
};
use crate::engine::catalog::HolderKind;
use crate::state::PrerequisiteClassLevels;

/// The entries whose components apply, each with what they hold, by the component's position: what to take back when
/// the entry stops being held, and which conditional grants hold now.
#[derive(Clone, Debug, Default)]
pub(super) struct HeldComponents(BTreeMap<Holder, Vec<(usize, Applied)>>);

/// What the stat bonuses held add to each ability score ([`Walker::component_ability_amounts`]).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct AbilityAmounts {
    /// All of them: what every number reads.
    pub every: [i16; 6],
    /// The permanent ones an ability prerequisite reads (`Modifier.IsPermanent`).
    pub permanent: [i16; 6],
    /// The `Feat` ones a full ability prerequisite adds to the permanent value.
    pub feat: [i16; 6],
}

/// One thing a component holds.
#[derive(Clone, Debug)]
enum Applied {
    Held(Held),
    Flag(MechanicsFlag),
    /// A conditional stat bonus that is on ([`IfHeld`](crate::IfHeld)).
    On,
}

/// The kind and key of an item the walk holds, as a holder of components: a fact. A class skill, a proficiency or an
/// option's value holds none.
fn held_kind(item: &Held) -> Option<(HolderKind, &str)> {
    Some(match item {
        Held::Feat(key, _) => (HolderKind::Feat, key.as_str()),
        Held::Feature(key) => (HolderKind::Feature, key.as_str()),
        Held::SelectionOption(key, _) => (HolderKind::Option, key.as_str()),
        Held::Selection(key) => (HolderKind::Selection, key.as_str()),
        Held::MythicAbility(key, _) => (HolderKind::MythicAbility, key.as_str()),
        Held::MythicFeat(key, _) => (HolderKind::MythicFeat, key.as_str()),
        Held::OptionValue(..) | Held::ClassSkill(_) | Held::Proficiency(_) => return None,
    })
}

/// The fact a held item is ([`held_kind`]'s: a copy of the key, taken only for an entry with components).
fn held_fact(item: &Held) -> Option<Fact> {
    Some(match item {
        Held::Feat(key, _) => Fact::Feat(key.clone()),
        Held::Feature(key) => Fact::Feature(key.clone()),
        Held::SelectionOption(key, _) => Fact::Option(key.clone()),
        Held::Selection(key) => Fact::Selection(key.clone()),
        Held::MythicAbility(key, _) => Fact::MythicAbility(key.clone()),
        Held::MythicFeat(key, _) => Fact::MythicFeat(key.clone()),
        Held::OptionValue(..) | Held::ClassSkill(_) | Held::Proficiency(_) => return None,
    })
}

/// The conditional stat bonuses of an entry's components, by position.
fn conditional_bonuses(components: &[Component]) -> impl Iterator<Item = (usize, &IfHeld)> {
    components
        .iter()
        .enumerate()
        .filter_map(|(at, component)| match component {
            Component::StatBonus {
                only_if: Some(only_if),
                ..
            } => Some((at, only_if)),
            _ => None,
        })
}

/// Whether a holder's applied entry at `of` is the conditional stat bonus at `at` turned on.
fn is_on(of: usize, applied: &Applied, at: usize) -> bool {
    of == at && matches!(applied, Applied::On)
}

/// Whether a component is a number the walk works out from what's held (rather than something it holds).
fn is_number(component: &Component) -> bool {
    matches!(
        component,
        Component::StatBonus { .. }
            | Component::SkillPoints(_)
            | Component::PrerequisiteClassLevels { .. }
            | Component::PrerequisiteStat { .. }
    )
}

impl<'a> Walker<'a> {
    /// The components of an entry, as a holder.
    fn components_of(&self, holder: &Holder) -> &'a [Component] {
        let (kind, key) = HolderKind::of(holder);
        self.engine.catalog.held_components(kind, key)
    }

    /// Whether a held item has components of its own.
    fn has_own_components(&self, item: &Held) -> bool {
        held_kind(item)
            .is_some_and(|(kind, key)| !self.engine.catalog.held_components(kind, key).is_empty())
    }

    /// After the walk holds an item: its components apply, the first time it's held; the conditional bonuses that
    /// look again when a checked fact is gained do; and the numbers are worked out again (a second rank doubles a
    /// [`PerRank`](crate::Value::PerRank) value as it comes, `AddStatBonus.OnTurnOn`'s `Value × GetRank()`).
    pub(super) fn hold_components(&mut self, item: &Held) {
        let Some(fact) = held_fact(item) else {
            return;
        };
        if self.has_own_components(item) {
            self.activate(Holder::Fact(fact.clone()));
        }
        if fact.rank(&self.state) == 1 {
            self.recheck_if_held(&fact, true);
        }
        self.refresh_if_numbers();
    }

    /// After the walk releases an item: once it's no longer held at all, what its components hold is released and the
    /// conditional bonuses that look again when a checked fact is lost do; the numbers are worked out again.
    pub(super) fn release_components(&mut self, item: &Held) {
        let Some(fact) = held_fact(item) else {
            return;
        };
        let mut dropped_numbers = false;
        if fact.rank(&self.state) == 0 {
            if self.has_own_components(item) {
                dropped_numbers = self.drop_components(&Holder::Fact(fact.clone()));
            }
            self.recheck_if_held(&fact, false);
        }
        if dropped_numbers {
            self.refresh_component_numbers();
        } else {
            self.refresh_if_numbers();
        }
    }

    /// An entry's components apply, and the numbers are worked out again: the race's, the background's, the deity's, a
    /// class's and its archetype's.
    pub(super) fn apply_components(&mut self, holder: impl FnOnce() -> Holder) {
        // Data without components builds no holder (a copy of a key).
        if self.engine.catalog.has_components() && self.activate(holder()) {
            self.refresh_if_numbers();
        }
    }

    /// An entry's components apply: its grants hold their facts, its conditional grants are looked at, and its
    /// conditional stat bonuses are decided (`OnTurnOn`). Nothing when they apply already (the entry held twice holds
    /// its grants once: `AddFacts.OnActivate` runs once per fact). Whether they applied now.
    fn activate(&mut self, holder: Holder) -> bool {
        let components = self.components_of(&holder);
        if components.is_empty() || self.components.0.contains_key(&holder) {
            return false;
        }
        self.components.0.insert(holder.clone(), Vec::new());
        let catalog = &self.engine.catalog;
        for (at, component) in components.iter().enumerate() {
            match component {
                Component::Grant(facts) => {
                    for fact in facts {
                        self.grant(&holder, at, fact);
                    }
                }
                Component::GrantWithValue { feat, value } => {
                    let item = Held::Feat(feat.clone(), vec![value.clone()]);
                    self.hold_for(&holder, at, item);
                }
                // `AddFeatureOnApply` has no `OnDeactivate`: what it adds stays.
                Component::GrantOnApply(fact) => {
                    if let Some(item) = self.held_of(fact) {
                        self.hold(item, None);
                    }
                }
                // `RemoveFeatureOnApply.OnActivate`: the fact goes, every rank of it, and nothing gives it back.
                Component::RemoveOnApply(fact) => {
                    if let Some(item) = self.held_of(fact) {
                        for _ in 0..u8::MAX {
                            if fact.rank(&self.state) == 0 {
                                break;
                            }
                            self.release(&item);
                        }
                    }
                }
                Component::ClassSkill(skill) => {
                    self.hold_for(&holder, at, Held::ClassSkill(skill.clone()));
                }
                Component::Proficiency {
                    weapons,
                    lists,
                    armour,
                } => {
                    for weapon in weapons {
                        self.hold_for(&holder, at, Held::Proficiency(weapon.clone()));
                    }
                    let feats = (lists.iter().map(|list| list.feat()))
                        .chain(armour.iter().map(|group| group.feat()));
                    for feat in feats {
                        let values = catalog.every_value(&feat);
                        self.hold_for(&holder, at, Held::Feat(feat, values));
                    }
                }
                Component::Flag(flag) => {
                    let holds = self.state.flags.entry(*flag).or_default();
                    *holds = holds.saturating_add(1);
                    self.record(&holder, at, Applied::Flag(*flag));
                }
                _ => {}
            }
        }
        self.update_conditionals(&holder);
        for (at, only_if) in conditional_bonuses(components) {
            let on = only_if.holds(&self.state);
            self.switch(&holder, at, on);
        }
        true
    }

    /// `AddStatBonusIfHasFact`'s two events: a checked fact gained (`HandleUnitGainFact`, looked at only while
    /// inverted) or lost (`HandleUnitLostFact`, only while not); each bonus listening for it is decided again
    /// (`Update`).
    fn recheck_if_held(&mut self, fact: &Fact, gained: bool) {
        let mut decided = Vec::new();
        for holder in self.components.0.keys() {
            for (at, only_if) in conditional_bonuses(self.components_of(holder)) {
                if only_if.not == gained && only_if.facts.contains(fact) {
                    decided.push((holder.clone(), at, only_if.holds(&self.state)));
                }
            }
        }
        for (holder, at, on) in decided {
            self.switch(&holder, at, on);
        }
    }

    /// Turns a holder's conditional stat bonus at `at` on or off.
    fn switch(&mut self, holder: &Holder, at: usize, on: bool) {
        if let Some(list) = self.components.0.get_mut(holder) {
            list.retain(|(of, applied)| !is_on(*of, applied, at));
            if on {
                list.push((at, Applied::On));
            }
        }
    }

    /// The numbers worked out again, when an entry held has any.
    fn refresh_if_numbers(&mut self) {
        if (self.components.0.keys()).any(|holder| self.components_of(holder).iter().any(is_number))
        {
            self.refresh_component_numbers();
        }
    }

    /// An entry's components stop applying: what they hold is released, in the reverse order (`OnDeactivate`,
    /// `OnTurnOff`). Whether it had numbers, which the caller then works out again.
    fn drop_components(&mut self, holder: &Holder) -> bool {
        let Some(applied) = self.components.0.remove(holder) else {
            return false;
        };
        for (_, applied) in applied.into_iter().rev() {
            self.take_back(applied);
        }
        self.components_of(holder).iter().any(is_number)
    }

    /// Releases one thing a component held.
    fn take_back(&mut self, applied: Applied) {
        match applied {
            Applied::Held(item) => self.release(&item),
            Applied::On => {}
            Applied::Flag(flag) => {
                if let Some(holds) = self.state.flags.get_mut(&flag) {
                    *holds = holds.saturating_sub(1);
                    if *holds == 0 {
                        self.state.flags.remove(&flag);
                    }
                }
            }
        }
    }

    /// Holds what a `Grant` names; a selection is held and offers no pick (`AddFacts`).
    fn grant(&mut self, holder: &Holder, at: usize, fact: &Fact) {
        if let Some(item) = self.held_of(fact) {
            self.hold_for(holder, at, item);
        }
    }

    /// Holds an item for a holder's component, noted so it's released with it.
    fn hold_for(&mut self, holder: &Holder, at: usize, item: Held) {
        self.hold(item.clone(), None);
        self.record(holder, at, Applied::Held(item));
    }

    fn record(&mut self, holder: &Holder, at: usize, applied: Applied) {
        if let Some(list) = self.components.0.get_mut(holder) {
            list.push((at, applied));
        }
    }

    /// The walk's item for a fact a grant holds; `None` for what is chosen, not granted (a class, an archetype, a
    /// race, a deity: the check refuses them) and an option no selection defines. An option is held through the
    /// selection that defines it, without its rows by option level.
    fn held_of(&self, fact: &Fact) -> Option<Held> {
        let catalog = &self.engine.catalog;
        Some(match fact {
            Fact::Feat(key) => Held::Feat(key.clone(), catalog.every_value(key)),
            Fact::Feature(key) => Held::Feature(key.clone()),
            Fact::Option(key) => {
                Held::SelectionOption(key.clone(), catalog.selection_of(key)?.key.clone())
            }
            Fact::Selection(key) => Held::Selection(key.clone()),
            Fact::MythicAbility(key) => Held::MythicAbility(key.clone(), None),
            Fact::MythicFeat(key) => Held::MythicFeat(key.clone(), None),
            Fact::Class(_) | Fact::Archetype(_) | Fact::Race(_) | Fact::Deity(_) => return None,
        })
    }

    /// At each level gained (`IOwnerGainLevelHandler.HandleUnitGainLevel`), every entry's conditional grants are looked
    /// at again, and the numbers worked out at the new level.
    pub(super) fn level_components(&mut self) {
        let holders: Vec<Holder> = self.components.0.keys().cloned().collect();
        for holder in &holders {
            self.update_conditionals(holder);
        }
        self.refresh_if_numbers();
    }

    /// A holder's conditional grants: a `GrantAtClassLevel` holds its fact while its level is reached (or, `before`,
    /// isn't) and releases it otherwise (`AddFeatureOnClassLevel.Apply`); a `GrantIf` holds its fact the first time its
    /// condition holds and keeps it (`AddFeatureIfHasFact.Apply`).
    fn update_conditionals(&mut self, holder: &Holder) {
        let engine = self.engine;
        let class_of = |archetype: &_| engine.catalog.archetype_class(archetype);
        for (at, component) in self.components_of(holder).iter().enumerate() {
            let applied = (self.components.0.get(holder))
                .is_some_and(|list| list.iter().any(|(of, _)| *of == at));
            match component {
                Component::GrantAtClassLevel {
                    fact,
                    classes,
                    archetypes,
                    level,
                    before,
                } => {
                    let reached = grant_level(&self.state, classes, archetypes, &class_of);
                    let holds = grants_at(reached, *level, *before);
                    if holds && !applied {
                        if let Some(item) = self.held_of(fact) {
                            self.hold_for(holder, at, item);
                        }
                    } else if !holds && applied {
                        let taken = (self.components.0.get_mut(holder))
                            .map(|list| {
                                let (taken, kept) = std::mem::take(list)
                                    .into_iter()
                                    .partition(|(of, _)| *of == at);
                                *list = kept;
                                taken
                            })
                            .unwrap_or_default();
                        for (_, applied) in taken.into_iter().rev() {
                            self.take_back(applied);
                        }
                    }
                }
                Component::GrantIf { has, not, fact } => {
                    if !applied
                        && (has.rank(&self.state) > 0) != *not
                        && let Some(item) = self.held_of(fact)
                    {
                        self.hold_for(holder, at, item);
                    }
                }
                _ => {}
            }
        }
    }

    /// What the entries held add to the prerequisites, the ability scores and the base attack bonus, worked out again.
    pub(super) fn refresh_component_numbers(&mut self) {
        let mut class_levels = Vec::new();
        let mut stats = Vec::new();
        for holder in self.components.0.keys() {
            for component in self.components_of(holder) {
                match component {
                    Component::PrerequisiteClassLevels {
                        class,
                        from,
                        share,
                        plus,
                        only_for,
                    } => class_levels.push(PrerequisiteClassLevels {
                        class: class.clone(),
                        levels: share.levels(self.state.class_level(from), *plus),
                        only_for: only_for.clone(),
                    }),
                    Component::PrerequisiteStat { stat, rule } => {
                        stats.push((stat.clone(), rule.clone()));
                    }
                    _ => {}
                }
            }
        }
        self.state.prerequisite_class_levels = class_levels;
        self.state.prerequisite_stats = stats;
        self.refresh_ability_scores();
        self.refresh_bab();
    }

    /// Every stat bonus of the entries held that is in force, with the stat, its type and its amount: the value worked
    /// out now, times its multiplier, at least its minimum when it has one, for a conditional one only while it's on.
    fn stat_bonuses(&self) -> Vec<(&'a StatTarget, Descriptor, i16, &Holder)> {
        let engine = self.engine;
        let class_of = |archetype: &_| engine.catalog.archetype_class(archetype);
        let base_scores = self.without_racial(|_| true);
        let mut bonuses = Vec::new();
        for (holder, applied) in &self.components.0 {
            let own_rank = match holder {
                Holder::Fact(fact) => fact.rank(&self.state),
                Holder::Subrace(_) | Holder::Background(_) => 1,
            };
            let context = ValueContext {
                state: &self.state,
                own_rank,
                base_scores,
                archetype_class: &class_of,
            };
            for (at, component) in self.components_of(holder).iter().enumerate() {
                let Component::StatBonus {
                    stat,
                    value,
                    descriptor,
                    multiplier,
                    at_least,
                    only_if,
                } = component
                else {
                    continue;
                };
                if only_if.is_some() && !applied.iter().any(|(of, applied)| is_on(*of, applied, at))
                {
                    continue;
                }
                let mut amount = value.at(&context) * i32::from(*multiplier);
                if let Some(minimum) = at_least {
                    amount = amount.max(i32::from(*minimum));
                }
                let amount =
                    i16::try_from(amount).unwrap_or(if amount < 0 { i16::MIN } else { i16::MAX });
                bonuses.push((stat, *descriptor, amount, holder));
            }
        }
        bonuses
    }

    /// What the stat bonuses held add to each ability score, stacked by type (`ApplyModifiersFiltered`): all of them;
    /// the permanent ones an ability prerequisite reads (`Modifier.IsPermanent`); and the `Feat` ones a full ability
    /// prerequisite adds to those (`PrerequisiteFullStatValue.GetFullStatValue`, every one).
    pub(super) fn component_ability_amounts(&self) -> AbilityAmounts {
        let mut amounts = AbilityAmounts::default();
        if !self.engine.catalog.has_components() {
            return amounts;
        }
        let bonuses = self.stat_bonuses();
        for (at, ability) in AbilityScores::ABILITIES.into_iter().enumerate() {
            let of_ability: Vec<(Descriptor, i16)> = (bonuses.iter())
                .filter(|(stat, ..)| **stat == StatTarget::Ability(ability))
                .map(|(_, descriptor, amount, _)| (*descriptor, *amount))
                .collect();
            amounts.every[at] = game_stacked(&of_ability);
            amounts.feat[at] = (of_ability.iter())
                .filter(|(descriptor, _)| descriptor.counts_in_full_value())
                .map(|(_, amount)| amount)
                .sum();
            let lasting: Vec<(Descriptor, i16)> = (of_ability.into_iter())
                .filter(|(descriptor, _)| descriptor.is_permanent())
                .collect();
            amounts.permanent[at] = game_stacked(&lasting);
        }
        amounts
    }

    /// What the stat bonuses held add to the base attack bonus, stacked by type.
    pub(super) fn component_bab(&self) -> i16 {
        if !self.engine.catalog.has_components() {
            return 0;
        }
        let amounts: Vec<(Descriptor, i16)> = (self.stat_bonuses().into_iter())
            .filter(|(stat, ..)| **stat == StatTarget::BaseAttack)
            .map(|(_, descriptor, amount, _)| (descriptor, amount))
            .collect();
        game_stacked(&amounts)
    }

    /// The skill points the entries held add at a level (`SkillPoints`): `AddSkillPointPerCharacterLevel` adds the
    /// character level to the points `ApplySkillPoints.Apply` counts over every level, less those spent, so an entry
    /// held since an earlier level gives every level's point at once, and one a level after. `ApplySkillPoints` comes
    /// before a level's picks (`LevelUpActionPriority`), so an entry picked at a level counts from the next.
    pub(super) fn component_skill_points(&mut self, level: u8) -> u8 {
        if !self.engine.catalog.has_components() {
            return 0;
        }
        let per_level = (self.components.0.keys())
            .flat_map(|holder| self.components_of(holder))
            .filter_map(|component| match component {
                Component::SkillPoints(points) => Some(u16::from(*points)),
                _ => None,
            })
            .sum::<u16>();
        let due = per_level * u16::from(level);
        let extra = due.saturating_sub(self.skill_points_given);
        self.skill_points_given = self.skill_points_given.max(due);
        u8::try_from(extra).unwrap_or(u8::MAX)
    }

    /// The stat bonuses held to a save, initiative or a skill, as bonuses in force: what the totals add up, stacked by
    /// their type ([`counted`](crate::counted)). A bonus to a stat with no line (armour class, combat manoeuvres,
    /// hit points, speed) counts in no total.
    pub(super) fn component_bonuses_in_force(&self) -> Vec<BonusInForce> {
        if !self.engine.catalog.has_components() {
            return Vec::new();
        }
        (self.stat_bonuses().into_iter())
            .filter_map(|(stat, descriptor, amount, holder)| {
                let target = match stat {
                    StatTarget::Save(save) => BonusTarget::Save(*save),
                    StatTarget::Initiative => BonusTarget::Initiative,
                    StatTarget::Skill(skill) => BonusTarget::Skill(skill.clone()),
                    _ => return None,
                };
                Some(BonusInForce {
                    source: BonusSource::Component {
                        holder: holder.clone(),
                        descriptor,
                    },
                    value: None,
                    target: InForceTarget::Number(target),
                    amount,
                    kind: descriptor.bonus_type(),
                    name: self.holder_name(holder),
                })
            })
            .collect()
    }

    /// A holder's name, as a bonus in force shows it: the engine's name for it (its key when the data lacks one).
    fn holder_name(&self, holder: &Holder) -> String {
        let engine = self.engine;
        match holder {
            Holder::Fact(fact) => match fact {
                Fact::Feat(key) => engine.feat_name(key),
                Fact::Feature(key) => engine.feature_name(key),
                Fact::Option(key) => engine.option_name(key),
                Fact::Selection(key) => engine.selection_name(key),
                Fact::MythicAbility(key) => engine.mythic_ability_name(key),
                Fact::MythicFeat(key) => engine.mythic_feat_name(key),
                Fact::Class(key) => engine.class_name(key),
                Fact::Archetype(key) => engine.archetype_name(key),
                Fact::Race(key) => engine.race_name(key),
                Fact::Deity(key) => engine.deity_name(key),
            },
            Holder::Subrace(key) => engine.subrace_name(key),
            Holder::Background(key) => engine.background_name(key),
        }
        .to_owned()
    }
}
