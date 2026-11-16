//! Filling a slot, and holding or releasing what a pick or grant gives, with the ability scores and floors that
//! follow from it.

use super::gear::WornGear;
use super::judge::{check_pick, deferrable, skips_proficiency};
use super::{Checkpoint, Deferred, GrantSource, Held, Pending, Probe, RankGrant, Walker};
use crate::build::{AbilityScores, Pick, SlotId};
use crate::choice::{Choice, Issue};
use crate::component::Fact;
use crate::engine::{Slot, SlotKind, with_floors};
use crate::keys::{FeatKey, FeatureKey, MythicFeatKey, SelectionOptionKey};
use crate::parameter::Parameter;
use crate::state::{CharacterState, FactRef};

impl<'a> Walker<'a> {
    /// Fills a Mythic ascension's slots in order.
    pub(super) fn fill_all(
        &mut self,
        slots: Vec<(SlotKind, SlotId)>,
        rank: Option<u8>,
    ) -> Vec<Slot> {
        slots
            .into_iter()
            .map(|(kind, id)| self.fill(id, kind, rank))
            .collect()
    }

    /// Checks the slot's pick against the state just before it and, when it holds, adds it to the state. While a
    /// level's band is filled ([`fill_bands`](Self::fill_bands)), a pick refused for what the band's other picks may
    /// give ([`deferrable`]) is deferred, to be tried again after them ([`retry`](Self::retry)), rather than failing
    /// now.
    pub(super) fn fill(&mut self, id: SlotId, kind: SlotKind, rank: Option<u8>) -> Slot {
        let source = rank.map(|rank| GrantSource {
            rank,
            from_path: !matches!(
                id,
                SlotId::Mythic { .. } | SlotId::MythicSelection { .. } | SlotId::MythicExtra { .. }
            ),
            selection: match &kind {
                SlotKind::Selection(selection) => Some(selection.clone()),
                _ => None,
            },
            slot: Some(id.clone()),
        });
        if let Some(Probe::Slot(probed)) = self.probe
            && probed == &id
        {
            self.probed.state = Some(self.state.clone());
            self.probed.kind = Some(kind.clone());
        }
        self.reached.insert(id.clone());
        if let SlotKind::Selection(selection) = &kind
            && !self.selections_had.contains(selection)
        {
            self.selections_had.push(selection.clone());
        }
        let pick = self.build.picks.get(&id).cloned();
        if let Some(pick) = &pick {
            match self.check_pick(&kind, pick) {
                Ok(()) => self.hold_noted(&id, &kind, pick, source.as_ref()),
                Err(issue) => match self.deferred.as_mut() {
                    Some(deferred) if deferrable(&issue) => deferred.push(Deferred {
                        id: id.clone(),
                        kind: kind.clone(),
                        pick: pick.clone(),
                        issue,
                    }),
                    _ => self.fail(Choice::Slot(id.clone()), issue),
                },
            }
        }
        if let Some(Probe::Later(probed)) = self.probe
            && probed == &id
        {
            self.probed.later = Some(Vec::new());
        }
        Slot { id, kind, pick }
    }

    /// Tries each deferred pick again, in order, against the state now, holding those that pass: whether any did.
    /// Those still refused stay, with why.
    pub(super) fn retry(&mut self, deferred: &mut Vec<Deferred>) -> bool {
        let before = deferred.len();
        deferred.retain_mut(|entry| match self.check_pick(&entry.kind, &entry.pick) {
            Ok(()) => {
                self.hold_noted(&entry.id, &entry.kind, &entry.pick, None);
                false
            }
            Err(issue) => {
                entry.issue = issue;
                true
            }
        });
        deferred.len() < before
    }

    /// Holds a pick that passed its check, and keeps it with the state either side of it when a later probe has
    /// passed its slot ([`Probed::later`](super::Probed::later)).
    fn hold_noted(
        &mut self,
        id: &SlotId,
        kind: &SlotKind,
        pick: &Pick,
        source: Option<&GrantSource>,
    ) {
        let before = match (self.probe, &self.probed.later) {
            (Some(Probe::Later(probed)), Some(_)) if probed != id => Some(self.state.clone()),
            _ => None,
        };
        self.hold_pick(kind, pick, source);
        if let (Some(before), Some(later)) = (before, self.probed.later.as_mut()) {
            later.push(Checkpoint {
                slot: id.clone(),
                kind: kind.clone(),
                pick: pick.clone(),
                before,
                after: self.state.clone(),
            });
        }
    }

    fn check_pick(&self, kind: &SlotKind, pick: &Pick) -> Result<(), Issue> {
        check_pick(
            &self.engine.catalog,
            &self.build.start,
            &self.state,
            kind,
            pick,
        )
    }

    fn hold_pick(&mut self, kind: &SlotKind, pick: &Pick, source: Option<&GrantSource>) {
        match pick {
            Pick::Feat {
                feat: key,
                parameter,
            } => {
                // A list whose pick skips the proficiency check makes the character proficient with its weapon
                // (the Sword Saint's chosen weapon). Only a feat that requires proficiency can skip it.
                let catalog = &self.engine.catalog;
                if let (SlotKind::Feat(selection), Some(Parameter::Weapon(weapon))) =
                    (kind, parameter)
                    && catalog
                        .feat(key)
                        .is_some_and(|feat| feat.requires_proficiency)
                    && skips_proficiency(&catalog.slot_lists(selection, &self.state), key)
                {
                    self.hold(Held::Proficiency(weapon.clone()), source.cloned());
                }
                self.hold(
                    Held::Feat(key.clone(), parameter.iter().cloned().collect()),
                    source.cloned(),
                );
                // Its selection's slot comes once the level's own slots are filled (ticket 302).
                if self
                    .engine
                    .catalog
                    .feat(key)
                    .is_some_and(|feat| feat.selection.is_some())
                {
                    self.pending.push(Pending::Feat(key.clone()));
                }
            }
            Pick::SelectionOption {
                option: key,
                parameter,
            } => {
                // Its feats for every value; its chosen feat for the pick's; its rows.
                if let SlotKind::Selection(selection) = kind {
                    self.take_option(selection, key, parameter.as_ref(), source);
                }
            }
            Pick::MythicAbility {
                ability: key,
                parameter,
            } => {
                self.hold(
                    Held::MythicAbility(key.clone(), parameter.clone()),
                    source.cloned(),
                );
                if let Some(ability) = self.engine.catalog.mythic_ability(key) {
                    for feature in &ability.features {
                        self.hold(Held::Feature(feature.clone()), source.cloned());
                    }
                }
            }
            Pick::MythicFeat {
                feat: key,
                parameter,
            } => self.hold(
                Held::MythicFeat(key.clone(), parameter.clone()),
                source.cloned(),
            ),
        }
    }

    /// Holds an item; `source` says which Mythic ascension granted it, so a Path switch can take it back.
    pub(super) fn hold(&mut self, item: Held, source: Option<GrantSource>) {
        let state = &mut self.state;
        match &item {
            Held::Feat(key, values) => {
                let held = state.held.entry(Fact::Feat(key.clone())).or_default();
                held.holds = held.holds.saturating_add(1);
                for value in values {
                    held.count_value(value);
                }
            }
            Held::Feature(key) => {
                add_feature(state, key);
                for (feat, values) in self.favoured_weapon_feats(key) {
                    self.hold(Held::Feat(feat, values), None);
                }
            }
            Held::SelectionOption(key, through) => {
                let held = state.held.entry(Fact::Option(key.clone())).or_default();
                held.holds = held.holds.saturating_add(1);
                let times = state
                    .options_through
                    .entry(through.clone())
                    .or_default()
                    .entry(key.clone())
                    .or_default();
                *times = times.saturating_add(1);
                if let Some(group) = self
                    .engine
                    .catalog
                    .option(key)
                    .and_then(|option| option.group)
                {
                    state.option_groups.insert(key.clone(), group);
                }
                if let Some(selection) = self.engine.catalog.selection_of(key) {
                    state
                        .option_selections
                        .insert(key.clone(), selection.key.clone());
                }
            }
            Held::OptionValue(key, value) => {
                (state.held.entry(Fact::Option(key.clone())).or_default()).count_value(value);
            }
            // Its values are held once each, however many picks name one.
            Held::MythicAbility(key, value) => {
                let held = (state.held)
                    .entry(Fact::MythicAbility(key.clone()))
                    .or_default();
                match value {
                    Some(value) => {
                        held.values.insert(value.clone(), 1);
                    }
                    None => held.holds = held.holds.saturating_add(1),
                }
            }
            Held::MythicFeat(key, value) => {
                let held = state.held.entry(Fact::MythicFeat(key.clone())).or_default();
                held.holds = held.holds.saturating_add(1);
                if let Some(value) = value {
                    held.values.insert(value.clone(), 1);
                }
            }
            Held::ClassSkill(key) => {
                let holds = self.class_skill_holds.entry(key.clone()).or_default();
                *holds = holds.saturating_add(1);
                state.class_skills.insert(key.clone());
            }
            Held::Selection(key) => {
                let held = state.held.entry(Fact::Selection(key.clone())).or_default();
                held.holds = held.holds.saturating_add(1);
            }
            Held::Proficiency(key) => {
                let holds = state.weapon_proficiencies.entry(key.clone()).or_default();
                *holds = holds.saturating_add(1);
            }
        }
        match &item {
            Held::SelectionOption(key, _) if self.raises_bab(key) => self.refresh_bab(),
            Held::MythicFeat(key, _) if self.removes_penalty(key) => {
                self.refresh_ability_scores();
            }
            _ => {}
        }
        if self.engine.catalog.has_components() {
            self.hold_components(&item);
        }
        let option_row = match &item {
            Held::Feature(key) => self.engine.catalog.option_row(key),
            _ => None,
        };
        if let Some(source) = &source {
            self.rank_grants.push(RankGrant {
                source: source.clone(),
                item,
            });
        }
        if let Some(row) = option_row {
            self.grant_option_row(row, source.as_ref());
        }
    }

    pub(super) fn release(&mut self, item: &Held) {
        let state = &mut self.state;
        match item {
            Held::Feat(key, values) => state.release_fact(FactRef::Feat(key), |held| {
                held.holds = held.holds.saturating_sub(1);
                for value in values {
                    held.uncount_value(value);
                }
            }),
            Held::Feature(key) => {
                state.release_fact(FactRef::Feature(key), |held| {
                    held.holds = held.holds.saturating_sub(1);
                });
                for (feat, values) in self.favoured_weapon_feats(key) {
                    self.release(&Held::Feat(feat, values));
                }
            }
            Held::SelectionOption(key, through) => {
                let mut emptied = false;
                state.release_fact(FactRef::Option(key), |held| {
                    emptied = held.holds == 1;
                    held.holds = held.holds.saturating_sub(1);
                });
                if emptied {
                    state.option_groups.remove(key);
                    state.option_selections.remove(key);
                }
                if let Some(options) = state.options_through.get_mut(through) {
                    if let Some(times) = options.get_mut(key) {
                        *times = times.saturating_sub(1);
                        if *times == 0 {
                            options.remove(key);
                        }
                    }
                    if options.is_empty() {
                        state.options_through.remove(through);
                    }
                }
            }
            Held::OptionValue(key, value) => {
                state.release_fact(FactRef::Option(key), |held| held.uncount_value(value));
            }
            // Held while any pick holds it: for another value, or without one.
            Held::MythicAbility(key, value) => {
                state.release_fact(FactRef::MythicAbility(key), |held| match value {
                    Some(value) => {
                        held.values.remove(value);
                    }
                    None => held.holds = held.holds.saturating_sub(1),
                });
            }
            // Held while another pick holds it: for another value, or again (Extra Feat); its values go with its
            // last pick.
            Held::MythicFeat(key, value) => {
                state.release_fact(FactRef::MythicFeat(key), |held| {
                    if let Some(value) = value {
                        held.values.remove(value);
                    }
                    held.holds = held.holds.saturating_sub(1);
                    if held.holds == 0 {
                        held.values.clear();
                    }
                });
            }
            Held::ClassSkill(key) => {
                if let Some(holds) = self.class_skill_holds.get_mut(key) {
                    *holds = holds.saturating_sub(1);
                    if *holds == 0 {
                        self.class_skill_holds.remove(key);
                        state.class_skills.remove(key);
                    }
                }
            }
            Held::Selection(key) => state.release_fact(FactRef::Selection(key), |held| {
                held.holds = held.holds.saturating_sub(1);
            }),
            Held::Proficiency(key) => {
                if let Some(holds) = state.weapon_proficiencies.get_mut(key) {
                    *holds = holds.saturating_sub(1);
                    if *holds == 0 {
                        state.weapon_proficiencies.remove(key);
                    }
                }
            }
        }
        match item {
            Held::SelectionOption(key, _) if self.raises_bab(key) => self.refresh_bab(),
            Held::MythicFeat(key, _) if self.removes_penalty(key) => {
                self.refresh_ability_scores();
            }
            _ => {}
        }
        if self.engine.catalog.has_components() {
            self.release_components(item);
        }
    }

    /// The feats a feature holds for the deity's favoured weapons while it's held (Cleric Proficiencies), as granted
    /// feats with no slot: none for another feature, or without a deity.
    fn favoured_weapon_feats(&self, feature: &FeatureKey) -> Vec<(FeatKey, Vec<Parameter>)> {
        let catalog = &self.engine.catalog;
        match self.state.deity() {
            Some(deity) if catalog.grants_favoured_weapons(feature) => {
                catalog.favoured_weapon_feats(deity)
            }
            _ => Vec::new(),
        }
    }

    /// Whether a mythic feat removes the race's ability penalty (Destiny Beyond Birth).
    fn removes_penalty(&self, key: &MythicFeatKey) -> bool {
        self.engine
            .catalog
            .mythic_feat(key)
            .is_some_and(|feat| feat.removes_racial_penalty)
    }

    /// Whether an option sets the base attack bonus by ranks (Athletics III).
    fn raises_bab(&self, option: &SelectionOptionKey) -> bool {
        self.engine
            .catalog
            .bab_options()
            .iter()
            .any(|(key, _)| key == option)
    }

    /// The base attack bonus: the class tables' with the stat bonuses held, or the most the options held set by ranks
    /// spent (Athletics III: the ranks, and 5 more while the class tables' bonus equals the character level), or the
    /// most a feature held raises it to ([`BaseAttackAtLeast`](crate::Component::BaseAttackAtLeast), Perfect Mind: 15 +
    /// the Mythic rank). The sheet, the attack line and every prerequisite read it.
    pub(super) fn refresh_bab(&mut self) {
        let state = &self.state;
        let full = self.class_bab == state.character_level;
        let raised = self
            .engine
            .catalog
            .bab_options()
            .iter()
            .filter(|(key, _)| state.option_count(key) > 0)
            .map(|(_, rule)| {
                let extra = if full { rule.extra_at_full_bab } else { 0 };
                state.skill_ranks(&rule.skill).saturating_add(extra)
            })
            .max()
            .unwrap_or(0);
        let by_rank = state
            .bab_at_least_mythic_rank_plus
            .map_or(0, |plus| plus.saturating_add(state.mythic_rank));
        // The stat bonuses held add to the class tables' (`AddStatBonus` on `BaseAttackBonus`).
        let class_bab = u8::try_from((i16::from(self.class_bab) + self.component_bab()).max(0))
            .unwrap_or(u8::MAX);
        self.state.bab = class_bab.max(raised).max(by_rank);
    }

    /// The character's scores: as every number reads them (with every stat bonus, the gear's included), as ability
    /// prerequisites do (with the permanent ones), and as a full ability prerequisite does (those and the `Feat` ones),
    /// each stacked by type over every source ([`ability_amounts`](Self::ability_amounts)), then the Gold Dragon's
    /// floor and highest-score bonus ([`with_floors`]). All from one base ([`own_base`](Self::own_base)).
    pub(super) fn refresh_ability_scores(&mut self) {
        let base = self.own_base();
        let amounts = self.ability_amounts(true);
        let (floor, highest) = self.ability_floors();
        let scores = with_floors(WornGear::onto(amounts.every, base), floor, highest);
        let permanent = with_floors(WornGear::onto(amounts.permanent, base), floor, highest);
        self.state.ability_scores = scores;
        self.state.permanent_ability_scores = permanent;
        self.state.full_ability_scores = WornGear::onto(amounts.feat, permanent);
    }

    /// The character's scores without gear: the base scores, without the race's penalties while a mythic feat removes
    /// them (Destiny Beyond Birth), with every other stat bonus and the Gold Dragon's floor and highest-score bonus.
    pub(super) fn own_scores(&self) -> AbilityScores {
        let (floor, highest) = self.ability_floors();
        with_floors(
            WornGear::onto(self.ability_amounts(false).every, self.own_base()),
            floor,
            highest,
        )
    }

    /// The base scores, without the race's penalties while a mythic feat removes them (Destiny Beyond Birth).
    fn own_base(&self) -> AbilityScores {
        if self
            .state
            .mythic_feats()
            .any(|key| self.removes_penalty(key))
        {
            self.without_racial(|amount| amount < 0)
        } else {
            self.state.base_ability_scores
        }
    }

    /// The base scores without the race's modifiers that `undo` picks by amount: its penalties (Destiny Beyond Birth),
    /// or all of them (a component's base value of a score, the game's `BaseValue`).
    pub(super) fn without_racial(&self, undo: impl Fn(i8) -> bool) -> AbilityScores {
        let mut base = self.state.base_ability_scores;
        for &(ability, amount) in &self.racial_modifiers {
            if undo(amount) {
                let score = base.get_mut(ability);
                *score = score.saturating_add_signed(amount.saturating_neg());
            }
        }
        base
    }
}

fn add_feature(state: &mut CharacterState, feature: &FeatureKey) {
    let held = state
        .held
        .entry(Fact::Feature(feature.clone()))
        .or_default();
    held.holds = held.holds.saturating_add(1);
}
