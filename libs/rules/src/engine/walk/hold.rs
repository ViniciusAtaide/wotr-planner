//! Filling a slot, and holding or releasing what a pick or grant gives, with the ability scores and floors that
//! follow from it.

use super::{GrantSource, Held, Pending, Probe, RankGrant, Walker};
use crate::build::{Pick, SlotId};
use crate::choice::{Choice, Issue};
use crate::engine::slots::check_pick;
use crate::engine::{Slot, SlotKind, with_bonuses};
use crate::keys::{FeatKey, FeatureKey, MythicFeatKey};
use crate::parameter::Parameter;
use crate::state::CharacterState;

impl<'a> Walker<'a> {
    /// Fills a Level-up's or Mythic ascension's slots in order.
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

    /// Checks the slot's pick against the state just before it and, when it holds, adds it to the state.
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
                Ok(()) => self.hold_pick(&kind, pick, source.as_ref()),
                Err(_) => self.fail(Choice::Slot(id.clone())),
            }
        }
        Slot { id, kind, pick }
    }

    fn check_pick(&self, kind: &SlotKind, pick: &Pick) -> Result<(), Issue> {
        check_pick(&self.engine.catalog, &self.state, kind, pick)
    }

    fn hold_pick(&mut self, kind: &SlotKind, pick: &Pick, source: Option<&GrantSource>) {
        match pick {
            Pick::Feat {
                feat: key,
                parameter,
            } => {
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
            Pick::MythicAbility(key) => {
                self.hold(Held::MythicAbility(key.clone()), source.cloned());
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
            }
            Held::MythicAbility(key) => {
                state.mythic_abilities.insert(key.clone());
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
        }
        match &item {
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
            Held::MythicAbility(key) => {
                state.mythic_abilities.remove(key);
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
        }
        match item {
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

    /// The floors prerequisites see, from the features held (Perfect Mind: BAB 15, skill ranks at the Mythic rank).
    fn refresh_floors(&mut self) {
        let catalog = &self.engine.catalog;
        let (mut bab, mut skills) = (0u8, false);
        for key in self.state.features.keys() {
            if let Some((floor, at_rank)) = catalog.floor(key) {
                bab = bab.max(floor.unwrap_or(0));
                skills |= at_rank;
            }
        }
        self.state.bab_floor = bab;
        self.state.skill_ranks_at_least_mythic_rank = skills;
    }

    /// The character's own scores: the base scores with the bonuses of every feature held, once per grant, and
    /// without the race's penalties while a mythic feat removes them (Destiny Beyond Birth).
    pub(super) fn refresh_ability_scores(&mut self) {
        let catalog = &self.engine.catalog;
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
        let bonuses = self
            .state
            .features
            .iter()
            .filter_map(|(key, count)| {
                let bonuses = catalog.ability_bonuses(key)?;
                Some(std::iter::repeat_n(bonuses, usize::from(*count)).flatten())
            })
            .flatten();
        self.state.ability_scores = with_bonuses(base, bonuses);
    }
}

fn add_feature(state: &mut CharacterState, feature: &FeatureKey) {
    let count = state.features.entry(feature.clone()).or_default();
    *count = count.saturating_add(1);
}
