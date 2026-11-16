//! The components the walk evaluates while their entry is held (ticket 295): the grants hold their facts through the
//! walk's own holds, as a level row's do; the conditional grants are looked at again at every level; the conditional
//! stat bonuses on the events the game's component handles; and the numbers the state keeps (the ability scores, the
//! base attack bonus and its floor, the skill points and floor, the prerequisite modifiers) are worked out again when an
//! entry with one comes or goes, the level changes, or, for a value that reads the state, whenever anything is held
//! (ticket 297). The numbers only the lines and totals read (a save's, a skill's, a spell DC's) are worked out at a
//! level's end (`bonuses`). The catalog says which entries have any ([`Catalog::has_components`]): for data without,
//! none of this runs.
//!
//! [`Catalog::has_components`]: crate::engine::catalog::Catalog

use std::collections::BTreeMap;

use super::{Held, Walker};
use crate::build::AbilityScores;
use crate::component::{
    Component, Descriptor, Fact, Holder, IfHeld, MechanicsFlag, StatTarget, ValueContext,
    game_stacked, grant_level, grants_at, stat_bonus_amount,
};
use crate::engine::catalog::HolderKind;
use crate::gear::GearTarget;
use crate::parameter::Parameter;
use crate::state::PrerequisiteClassLevels;

/// The entries whose components apply, each with what they hold, by the component's position: what to take back when
/// the entry stops being held, and which conditional grants hold now. With how many of them change a number the state
/// keeps ([`Component::changes_state_numbers`]) and how many of those read the state ([`Component::reads_state`]), so
/// a hold works the numbers out again only when one may have moved.
#[derive(Clone, Debug, Default)]
pub(super) struct HeldComponents {
    held: BTreeMap<Holder, Vec<(usize, Applied)>>,
    numbers: usize,
    reading: usize,
}

/// What the stat bonuses held add to each ability score ([`Walker::ability_amounts`]).
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

/// One stat bonus in force to an ability score or the base attack bonus: the stat, its type and its amount.
pub(super) struct StatAmount {
    pub stat: StatTarget,
    pub descriptor: Descriptor,
    pub amount: i16,
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

impl<'a> Walker<'a> {
    /// The components of an entry, as a holder.
    fn components_of(&self, holder: &Holder) -> &'a [Component] {
        let (kind, key) = HolderKind::of(holder);
        self.engine.catalog.held_components(kind, key)
    }

    /// Whether a held item has components of its own the walk acts on ([`Component::walk_reads`]).
    fn has_own_components(&self, item: &Held) -> bool {
        held_kind(item).is_some_and(|(kind, key)| {
            (self.engine.catalog.held_components(kind, key).iter()).any(Component::walk_reads)
        })
    }

    /// After the walk holds an item: its components apply, the first time it's held; the conditional bonuses that
    /// look again when a checked fact is gained do; and the numbers are worked out again when they may have moved (a
    /// second rank doubles a [`PerRank`](crate::Value::PerRank) value as it comes, `AddStatBonus.OnTurnOn`'s
    /// `Value × GetRank()`).
    pub(super) fn hold_components(&mut self, item: &Held) {
        let Some(fact) = held_fact(item) else {
            return;
        };
        let own = self.has_own_components(item);
        if own {
            self.activate(Holder::Fact(fact.clone()));
        }
        let switched = fact.rank(&self.state) == 1 && self.recheck_if_held(&fact, true);
        self.refresh_if_moved(switched || (own && self.own_numbers(item)));
    }

    /// After the walk releases an item: once it's no longer held at all, what its components hold is released and the
    /// conditional bonuses that look again when a checked fact is lost do; the numbers are worked out again when they
    /// may have moved.
    pub(super) fn release_components(&mut self, item: &Held) {
        let Some(fact) = held_fact(item) else {
            return;
        };
        let mut moved = self.has_own_components(item) && self.own_numbers(item);
        if fact.rank(&self.state) == 0 {
            if self.has_own_components(item) {
                self.drop_components(&Holder::Fact(fact.clone()));
            }
            moved |= self.recheck_if_held(&fact, false);
        }
        self.refresh_if_moved(moved);
    }

    /// Whether a held item's own components change a number the state keeps.
    fn own_numbers(&self, item: &Held) -> bool {
        held_kind(item).is_some_and(|(kind, key)| {
            (self.engine.catalog.held_components(kind, key).iter())
                .any(Component::changes_state_numbers)
        })
    }

    /// An entry's components apply, and the numbers are worked out again: the race's, the background's, the deity's, a
    /// class's and its archetype's.
    pub(super) fn apply_components(&mut self, holder: impl FnOnce() -> Holder) {
        // Data without components builds no holder (a copy of a key).
        if self.engine.catalog.has_components() {
            let holder = holder();
            let numbers = self
                .components_of(&holder)
                .iter()
                .any(Component::changes_state_numbers);
            if self.activate(holder) {
                self.refresh_if_moved(numbers);
            }
        }
    }

    /// An entry's components apply: its grants hold their facts, its conditional grants are looked at, and its
    /// conditional stat bonuses are decided (`OnTurnOn`). Nothing when they apply already (the entry held twice holds
    /// its grants once: `AddFacts.OnActivate` runs once per fact). Whether they applied now.
    fn activate(&mut self, holder: Holder) -> bool {
        let components = self.components_of(&holder);
        if !components.iter().any(Component::walk_reads)
            || self.components.held.contains_key(&holder)
        {
            return false;
        }
        self.components.held.insert(holder.clone(), Vec::new());
        if components.iter().any(Component::changes_state_numbers) {
            self.components.numbers += 1;
        }
        if components.iter().any(Component::reads_state) {
            self.components.reading += 1;
        }
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
    fn recheck_if_held(&mut self, fact: &Fact, gained: bool) -> bool {
        let mut decided = Vec::new();
        for holder in self.components.held.keys() {
            for (at, only_if) in conditional_bonuses(self.components_of(holder)) {
                if only_if.not == gained && only_if.facts.contains(fact) {
                    decided.push((holder.clone(), at, only_if.holds(&self.state)));
                }
            }
        }
        let mut moved = false;
        for (holder, at, on) in decided {
            moved |= self.components_of(&holder)[at].changes_state_numbers();
            self.switch(&holder, at, on);
        }
        moved
    }

    /// Turns a holder's conditional stat bonus at `at` on or off.
    fn switch(&mut self, holder: &Holder, at: usize, on: bool) {
        if let Some(list) = self.components.held.get_mut(holder) {
            list.retain(|(of, applied)| !is_on(*of, applied, at));
            if on {
                list.push((at, Applied::On));
            }
        }
    }

    /// The numbers worked out again, when `moved` (an entry with one came or went) or a held one reads the state.
    fn refresh_if_moved(&mut self, moved: bool) {
        if moved || self.components.reading > 0 {
            self.refresh_component_numbers();
        }
    }

    /// An entry's components stop applying: what they hold is released, in the reverse order (`OnDeactivate`,
    /// `OnTurnOff`).
    fn drop_components(&mut self, holder: &Holder) {
        let Some(applied) = self.components.held.remove(holder) else {
            return;
        };
        let components = self.components_of(holder);
        if components.iter().any(Component::changes_state_numbers) {
            self.components.numbers -= 1;
        }
        if components.iter().any(Component::reads_state) {
            self.components.reading -= 1;
        }
        for (_, applied) in applied.into_iter().rev() {
            self.take_back(applied);
        }
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
        if let Some(list) = self.components.held.get_mut(holder) {
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
        let holders: Vec<Holder> = self.components.held.keys().cloned().collect();
        for holder in &holders {
            self.update_conditionals(holder);
        }
        if self.components.numbers > 0 {
            self.refresh_component_numbers();
        }
    }

    /// A holder's conditional grants: a `GrantAtClassLevel` holds its fact while its level is reached (or, `before`,
    /// isn't) and releases it otherwise (`AddFeatureOnClassLevel.Apply`); a `GrantIf` holds its fact the first time its
    /// condition holds and keeps it (`AddFeatureIfHasFact.Apply`).
    fn update_conditionals(&mut self, holder: &Holder) {
        let engine = self.engine;
        let class_of = |archetype: &_| engine.catalog.archetype_class(archetype);
        for (at, component) in self.components_of(holder).iter().enumerate() {
            let applied = (self.components.held.get(holder))
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
                        let taken = (self.components.held.get_mut(holder))
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

    /// What the entries held add to the prerequisites, the ability scores, the base attack bonus and the skill floor,
    /// worked out again.
    pub(super) fn refresh_component_numbers(&mut self) {
        let mut class_levels = Vec::new();
        let mut stats = Vec::new();
        let (mut bab_floor, mut skill_floor) = (None, false);
        for holder in self.components.held.keys() {
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
                    Component::BaseAttackAtLeast(plus) => bab_floor = bab_floor.max(Some(*plus)),
                    Component::GoldenDragonSkills => skill_floor = true,
                    _ => {}
                }
            }
        }
        self.state.prerequisite_class_levels = class_levels;
        self.state.prerequisite_stats = stats;
        self.state.bab_at_least_mythic_rank_plus = bab_floor;
        self.state.skill_ranks_at_least_mythic_rank = skill_floor;
        self.refresh_ability_scores();
        self.refresh_bab();
    }

    /// Calls `each` for every component of the entries whose components apply, once for each value a parameterised
    /// entry is held for (with that value in the context its value reads), component by component; a conditional stat
    /// bonus only while it's on.
    pub(super) fn each_component<'s>(
        &'s self,
        mut each: impl FnMut(&'s Holder, &'a Component, Option<&'s Parameter>, &ValueContext<'_>),
    ) {
        let engine = self.engine;
        let class_of = |archetype: &_| engine.catalog.archetype_class(archetype);
        let base_scores = self.without_racial(|_| true);
        let state = &self.state;
        for (holder, applied) in &self.components.held {
            let own_rank = match holder {
                Holder::Fact(fact) => fact.rank(state),
                Holder::Subrace(_) | Holder::Background(_) => 1,
            };
            let values: Vec<Option<&'s Parameter>> = match holder {
                Holder::Fact(Fact::Feat(key)) => match state.feat_values.get(key) {
                    Some(values) => values.keys().map(Some).collect(),
                    None => vec![None],
                },
                Holder::Fact(Fact::MythicFeat(key)) => match state.mythic_feat_values.get(key) {
                    Some(values) => values.iter().map(Some).collect(),
                    None => vec![None],
                },
                _ => vec![None],
            };
            let contexts: Vec<(Option<&'s Parameter>, ValueContext<'_>)> = (values.into_iter())
                .map(|value| {
                    let context = ValueContext {
                        state,
                        own_rank,
                        value,
                        base_scores,
                        archetype_class: &class_of,
                    };
                    (value, context)
                })
                .collect();
            for (at, component) in self.components_of(holder).iter().enumerate() {
                if let Component::StatBonus {
                    only_if: Some(_), ..
                } = component
                    && !applied.iter().any(|(of, applied)| is_on(*of, applied, at))
                {
                    continue;
                }
                for (value, context) in &contexts {
                    each(holder, component, *value, context);
                }
            }
        }
    }

    /// Every stat bonus of the entries held that is in force to an ability score or the base attack bonus, with its
    /// stat, type and amount ([`stat_bonus_amount`]), a conditional one only while it's on. The others (a save's, a
    /// skill's) the totals read at a level's end.
    pub(super) fn stat_bonuses(&self) -> Vec<StatAmount> {
        let mut bonuses = Vec::new();
        self.each_component(|_, component, _, context| {
            if let Component::StatBonus {
                stat: stat @ (StatTarget::Ability(_) | StatTarget::BaseAttack),
                value,
                descriptor,
                multiplier,
                at_least,
                ..
            } = component
            {
                bonuses.push(StatAmount {
                    stat: stat.clone(),
                    descriptor: *descriptor,
                    amount: stat_bonus_amount(value, *multiplier, *at_least, context),
                });
            }
        });
        bonuses
    }

    /// What the stat bonuses held and, `with_gear`, the gear's add to each ability score, every source's stacked
    /// together by type (`ApplyModifiersFiltered`): all of them; the permanent ones an ability prerequisite reads
    /// (`Modifier.IsPermanent`: a racial one, or an item's enhancement); and the `Feat` ones a full ability prerequisite
    /// adds to those (`PrerequisiteFullStatValue.GetFullStatValue`, every one).
    pub(super) fn ability_amounts(&self, with_gear: bool) -> AbilityAmounts {
        let mut amounts = AbilityAmounts::default();
        let bonuses = if self.engine.catalog.has_components() {
            self.stat_bonuses()
        } else {
            Vec::new()
        };
        let gear: &[super::gear::WornStat] = if with_gear { &self.gear.stats } else { &[] };
        for (at, ability) in AbilityScores::ABILITIES.into_iter().enumerate() {
            // Each with whether it comes from an item.
            let of_ability: Vec<(Descriptor, i16, bool)> = (bonuses.iter())
                .filter(|bonus| bonus.stat == StatTarget::Ability(ability))
                .map(|bonus| (bonus.descriptor, bonus.amount, false))
                .chain(
                    (gear.iter())
                        .filter(|stat| stat.target == GearTarget::Ability(ability))
                        .map(|stat| (stat.descriptor, stat.amount, true)),
                )
                .collect();
            if of_ability.is_empty() {
                continue;
            }
            let every: Vec<(Descriptor, i16)> =
                of_ability.iter().map(|(d, a, _)| (*d, *a)).collect();
            amounts.every[at] = game_stacked(&every);
            amounts.feat[at] = (of_ability.iter())
                .filter(|(descriptor, ..)| descriptor.counts_in_full_value())
                .map(|(_, amount, _)| amount)
                .sum();
            let lasting: Vec<(Descriptor, i16)> = (of_ability.iter())
                .filter(|(descriptor, _, item)| descriptor.is_permanent(*item))
                .map(|(d, a, _)| (*d, *a))
                .collect();
            amounts.permanent[at] = game_stacked(&lasting);
        }
        amounts
    }

    /// The Gold Dragon's floor and its highest-score bonus of the entries held
    /// ([`AbilityAtLeast`](Component::AbilityAtLeast), [`HighestAbilityBonus`](Component::HighestAbilityBonus): one per
    /// rank of its entry, as each grant adds it).
    pub(super) fn ability_floors(&self) -> (u8, i16) {
        let (mut floor, mut highest) = (0u8, 0i16);
        if !self.engine.catalog.has_components() {
            return (floor, highest);
        }
        for holder in self.components.held.keys() {
            let rank = match holder {
                Holder::Fact(fact) => fact.rank(&self.state),
                _ => 1,
            };
            for component in self.components_of(holder) {
                match component {
                    Component::AbilityAtLeast(score) => floor = floor.max(*score),
                    Component::HighestAbilityBonus(amount) => {
                        highest = highest.saturating_add(i16::from(*amount) * i16::from(rank));
                    }
                    _ => {}
                }
            }
        }
        (floor, highest)
    }

    /// What the stat bonuses held add to the base attack bonus, stacked by type.
    pub(super) fn component_bab(&self) -> i16 {
        if !self.engine.catalog.has_components() {
            return 0;
        }
        let amounts: Vec<(Descriptor, i16)> = (self.stat_bonuses().into_iter())
            .filter(|bonus| bonus.stat == StatTarget::BaseAttack)
            .map(|bonus| (bonus.descriptor, bonus.amount))
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
        let per_level = (self.components.held.keys())
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
}
