//! Filling a slot, and holding or releasing what a pick or grant gives, with the ability scores and floors that
//! follow from it.

use super::gear::WornGear;
use super::judge::{check_pick, deferrable, skips_proficiency};
use super::{Checkpoint, Deferred, GrantSource, Held, Pending, Probe, RankGrant, Walker};
use crate::build::{AbilityScores, Pick, SlotId};
use crate::choice::{Choice, Issue};
use crate::engine::{Slot, SlotKind, with_bonuses};
use crate::keys::{FeatKey, FeatureKey, MythicFeatKey, SelectionOptionKey};
use crate::parameter::Parameter;
use crate::state::CharacterState;

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
                if let (SlotKind::Feat(slot), Some(Parameter::Weapon(weapon))) = (kind, parameter)
                    && catalog
                        .feat(key)
                        .is_some_and(|feat| feat.requires_proficiency)
                    && skips_proficiency(&catalog.slot_lists(slot, &self.state), key)
                {
                    self.hold(Held::Proficiency(weapon.clone()), source.cloned());
                }
                self.hold(
                    Held::Feat(key.clone(), parameter.iter().cloned().collect()),
                    source.cloned(),
                );
                // Its extra pick's slot comes once the level's own slots are filled.
                if self
                    .engine
                    .catalog
                    .feat(key)
                    .is_some_and(|feat| !feat.extra_pick_of.is_empty())
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
        if self.sheet {
            self.note_bonuses(&item);
        }
        let state = &mut self.state;
        match &item {
            Held::Feat(key, values) => {
                let holds = state.feats.entry(key.clone()).or_default();
                *holds = holds.saturating_add(1);
                for value in values {
                    let holds = state
                        .feat_values
                        .entry(key.clone())
                        .or_default()
                        .entry(value.clone())
                        .or_default();
                    *holds = holds.saturating_add(1);
                }
            }
            Held::Feature(key) => {
                add_feature(state, key);
                if self.engine.catalog.ability_bonuses(key).is_some() {
                    self.refresh_ability_scores();
                }
                for (feat, values) in self.favoured_weapon_feats(key) {
                    self.hold(Held::Feat(feat, values), None);
                }
            }
            Held::SelectionOption(key, through) => {
                let times = state.selection_options.entry(key.clone()).or_default();
                *times = times.saturating_add(1);
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
                let holds = state
                    .option_values
                    .entry(key.clone())
                    .or_default()
                    .entry(value.clone())
                    .or_default();
                *holds = holds.saturating_add(1);
            }
            Held::MythicAbility(key, value) => {
                state.mythic_abilities.insert(key.clone());
                if let Some(value) = value {
                    state
                        .mythic_ability_values
                        .entry(key.clone())
                        .or_default()
                        .insert(value.clone());
                } else {
                    let holds = state
                        .mythic_ability_open_holds
                        .entry(key.clone())
                        .or_default();
                    *holds = holds.saturating_add(1);
                }
            }
            Held::MythicFeat(key, value) => {
                state.mythic_feats.insert(key.clone());
                if let Some(value) = value {
                    state
                        .mythic_feat_values
                        .entry(key.clone())
                        .or_default()
                        .insert(value.clone());
                }
            }
            Held::ClassSkill(key) => {
                let holds = self.class_skill_holds.entry(key.clone()).or_default();
                *holds = holds.saturating_add(1);
                state.class_skills.insert(key.clone());
            }
            Held::Selection(key) => {
                let holds = state.selections.entry(key.clone()).or_default();
                *holds = holds.saturating_add(1);
            }
            Held::Proficiency(key) => {
                let holds = state.weapon_proficiencies.entry(key.clone()).or_default();
                *holds = holds.saturating_add(1);
            }
        }
        match &item {
            Held::SelectionOption(key, _) if self.raises_bab(key) => self.refresh_bab(),
            Held::Feature(key) if self.engine.catalog.floor(key).is_some() => {
                self.refresh_floors();
            }
            Held::MythicFeat(key, _) if self.removes_penalty(key) => {
                self.refresh_ability_scores();
            }
            _ => {}
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
            Held::Feat(key, values) => {
                if let Some(holds) = state.feats.get_mut(key) {
                    *holds = holds.saturating_sub(1);
                    if *holds == 0 {
                        state.feats.remove(key);
                    }
                }
                if let Some(held) = state.feat_values.get_mut(key) {
                    for value in values {
                        if let Some(holds) = held.get_mut(value) {
                            *holds = holds.saturating_sub(1);
                            if *holds == 0 {
                                held.remove(value);
                            }
                        }
                    }
                    if held.is_empty() {
                        state.feat_values.remove(key);
                    }
                }
            }
            Held::Feature(key) => {
                if let Some(count) = state.features.get_mut(key) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        state.features.remove(key);
                    }
                }
                if self.engine.catalog.ability_bonuses(key).is_some() {
                    self.refresh_ability_scores();
                }
                for (feat, values) in self.favoured_weapon_feats(key) {
                    self.release(&Held::Feat(feat, values));
                }
            }
            Held::SelectionOption(key, through) => {
                if let Some(times) = state.selection_options.get_mut(key) {
                    *times = times.saturating_sub(1);
                    if *times == 0 {
                        state.selection_options.remove(key);
                        state.option_groups.remove(key);
                        state.option_selections.remove(key);
                    }
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
                if let Some(held) = state.option_values.get_mut(key) {
                    if let Some(holds) = held.get_mut(value) {
                        *holds = holds.saturating_sub(1);
                        if *holds == 0 {
                            held.remove(value);
                        }
                    }
                    if held.is_empty() {
                        state.option_values.remove(key);
                    }
                }
            }
            Held::MythicAbility(key, value) => {
                // Held while any pick holds it: for another value, or without one.
                match value {
                    Some(value) => {
                        if let Some(values) = state.mythic_ability_values.get_mut(key) {
                            values.remove(value);
                            if values.is_empty() {
                                state.mythic_ability_values.remove(key);
                            }
                        }
                    }
                    None => {
                        if let Some(holds) = state.mythic_ability_open_holds.get_mut(key) {
                            *holds = holds.saturating_sub(1);
                            if *holds == 0 {
                                state.mythic_ability_open_holds.remove(key);
                            }
                        }
                    }
                }
                if !state.mythic_ability_values.contains_key(key)
                    && !state.mythic_ability_open_holds.contains_key(key)
                {
                    state.mythic_abilities.remove(key);
                }
            }
            Held::MythicFeat(key, value) => {
                // A parameterised one stays held while it's held for another value.
                let values = state.mythic_feat_values.get_mut(key);
                let left = match (values, value) {
                    (Some(values), Some(value)) => {
                        values.remove(value);
                        !values.is_empty()
                    }
                    _ => false,
                };
                if !left {
                    state.mythic_feat_values.remove(key);
                    state.mythic_feats.remove(key);
                }
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
            Held::Selection(key) => {
                if let Some(holds) = state.selections.get_mut(key) {
                    *holds = holds.saturating_sub(1);
                    if *holds == 0 {
                        state.selections.remove(key);
                    }
                }
            }
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
            Held::Feature(key) if self.engine.catalog.floor(key).is_some() => {
                self.refresh_floors();
            }
            Held::MythicFeat(key, _) if self.removes_penalty(key) => {
                self.refresh_ability_scores();
            }
            _ => {}
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

    /// The base attack bonus: the class tables', or the most the options held set by ranks spent (Athletics III: the
    /// ranks, and 5 more while the class tables' bonus equals the character level), or the most a feature held
    /// raises it to (Perfect Mind: 15 + the Mythic rank). The sheet, the attack line and every prerequisite read it.
    pub(super) fn refresh_bab(&mut self) {
        let state = &self.state;
        let full = self.class_bab == state.character_level;
        let raised = self
            .engine
            .catalog
            .bab_options()
            .iter()
            .filter(|(key, _)| state.selection_options.contains_key(key))
            .map(|(_, rule)| {
                let extra = if full { rule.extra_at_full_bab } else { 0 };
                state.skill_ranks(&rule.skill).saturating_add(extra)
            })
            .max()
            .unwrap_or(0);
        let by_rank = state
            .bab_at_least_mythic_rank_plus
            .map_or(0, |plus| plus.saturating_add(state.mythic_rank));
        self.state.bab = self.class_bab.max(raised).max(by_rank);
    }

    /// The floors of the features held (Perfect Mind: BAB 15 + MR, skill totals at the Mythic rank), and the BAB they
    /// raise.
    fn refresh_floors(&mut self) {
        let catalog = &self.engine.catalog;
        let (mut bab, mut skills) = (None, false);
        for key in self.state.features.keys() {
            if let Some((plus, at_rank)) = catalog.floor(key) {
                bab = bab.max(plus);
                skills |= at_rank;
            }
        }
        self.state.bab_at_least_mythic_rank_plus = bab;
        self.state.skill_ranks_at_least_mythic_rank = skills;
        self.refresh_bab();
    }

    /// The character's scores: as every number reads them (with all the gear's bonuses), and as ability prerequisites
    /// do (with the gear's bonuses that count there). Both from one base ([`own_base`](Self::own_base)).
    pub(super) fn refresh_ability_scores(&mut self) {
        let (base, gear) = (self.own_base(), &self.gear);
        self.state.ability_scores = self.with_feature_bonuses(WornGear::onto(gear.scores, base));
        self.state.permanent_ability_scores = if gear.permanent == gear.scores {
            self.state.ability_scores
        } else {
            self.with_feature_bonuses(WornGear::onto(gear.permanent, base))
        };
    }

    /// The character's scores without gear: the base scores, without the race's penalties while a mythic feat removes
    /// them (Destiny Beyond Birth), then the bonuses of every feature held ([`with_feature_bonuses`]).
    ///
    /// [`with_feature_bonuses`]: Self::with_feature_bonuses
    pub(super) fn own_scores(&self) -> AbilityScores {
        self.with_feature_bonuses(self.own_base())
    }

    /// The base scores, without the race's penalties while a mythic feat removes them (Destiny Beyond Birth).
    fn own_base(&self) -> AbilityScores {
        let mut base = self.state.base_ability_scores;
        if self
            .state
            .mythic_feats
            .iter()
            .any(|key| self.removes_penalty(key))
        {
            for (ability, penalty) in &self.racial_penalties {
                let score = base.get_mut(*ability);
                *score = score.saturating_add(*penalty);
            }
        }
        base
    }

    /// `base` (with or without the gear's amounts, [`WornGear`]) with the bonuses of every feature held, once per grant.
    /// Gear counts before the features (the user's choice, ticket 163), so a belt that makes another score the highest
    /// moves a "highest score" bonus (Perfect Body) to it.
    fn with_feature_bonuses(&self, base: AbilityScores) -> AbilityScores {
        let catalog = &self.engine.catalog;
        let bonuses = self
            .state
            .features
            .iter()
            .filter_map(|(key, count)| {
                let bonuses = catalog.ability_bonuses(key)?;
                Some(std::iter::repeat_n(bonuses, usize::from(*count)).flatten())
            })
            .flatten();
        with_bonuses(base, bonuses)
    }
}

fn add_feature(state: &mut CharacterState, feature: &FeatureKey) {
    let count = state.features.entry(feature.clone()).or_default();
    *count = count.saturating_add(1);
}
