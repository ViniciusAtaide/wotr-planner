//! A Level-up's own steps: the class level and what its row grants, skill ranks, the level's slots, and the
//! tables (BAB, saves, spellcasting) after it.

use std::collections::{BTreeMap, BTreeSet};

use super::judge::{Entry, class_entry};
use super::{Deferred, GrantSource, Held, Pending, Walker};
use crate::build::{Build, FeatSource, LevelUp, SlotId};
use crate::choice::{Choice, Issue};
use crate::component::{Fact, Holder};
use crate::engine::catalog::Catalog;
use crate::engine::{
    SkillPoints, Slot, SlotKind, advanced_spellcasting, class_level_cap, most_advanced,
};
use crate::game_data::{
    Ability, BASIC_FEAT_SELECTION, Band, Class, CountsToward, FeatureSelection, Save, SpellType,
};
use crate::keys::{ClassKey, FeatureKey, FeatureSelectionKey};
use crate::state::{CharacterState, ClassLevels};

impl<'a> Walker<'a> {
    /// On a class's first level: the class, archetype and advancement choices are checked against the character
    /// before the level ([`class_entry`], the check a class option's verdict shares). A class or archetype whose
    /// prerequisites aren't met fails with [`Issue::Unmet`]; a key the data lacks (the class's, the archetype's, an
    /// advanced class's) with [`Issue::UnknownKey`]; an archetype of another class or an advancement the class
    /// can't take with [`Issue::NotAllowed`].
    pub(super) fn check_class_entry(&mut self, level: u8, level_up: &LevelUp) {
        if self.state.class_level(&level_up.class) > 0 {
            return;
        }
        let catalog = &self.engine.catalog;
        let Some((class, _)) = catalog.class(&level_up.class) else {
            self.fail(Choice::Class { level }, Issue::UnknownKey);
            return;
        };
        let choice = self.build.class_choices.get(&level_up.class);
        let key = choice.and_then(|choice| choice.archetype.as_ref());
        let Entry {
            class: class_unmet,
            archetype: archetype_issue,
        } = class_entry(catalog, &self.state, class, key);
        if let Some(unmet) = class_unmet {
            self.fail(Choice::Class { level }, Issue::Unmet(unmet));
        }
        if let Some(issue) = archetype_issue {
            self.fail(
                Choice::Archetype {
                    class: level_up.class.clone(),
                },
                issue,
            );
        }
        if let Some(choice) = choice
            && !choice.advances.is_empty()
        {
            self.check_advances(class, &choice.advances);
        }
    }

    /// A prestige class advances at most one arcane and one divine class (one class in all when it advances
    /// `Either`: [`most_advanced`]), each already taken and casting the spells it advances; one the data lacks is
    /// [`Issue::UnknownKey`].
    fn check_advances(&mut self, prestige: &Class, advances: &BTreeSet<ClassKey>) {
        let catalog = &self.engine.catalog;
        if advances.iter().any(|key| catalog.class(key).is_none()) {
            self.fail(
                Choice::Advances {
                    class: prestige.key.clone(),
                },
                Issue::UnknownKey,
            );
            return;
        }
        let (advances_arcane, advances_divine) = advanced_spellcasting(prestige);
        // For each named class: (can take the arcane advancement, can take the divine one).
        let mut serves = Vec::new();
        for key in advances {
            let Some((class, _)) = catalog.class(key) else {
                serves.push((false, false));
                continue;
            };
            let held = self.state.classes.iter().find(|c| &c.class == key);
            let (casts_arcane, casts_divine) =
                catalog.casts(class, held.and_then(|c| c.archetype.as_ref()));
            let arcane = advances_arcane && casts_arcane;
            let divine = advances_divine && casts_divine;
            serves.push((held.is_some() && arcane, held.is_some() && divine));
        }
        let fits = serves.len() <= most_advanced(prestige)
            && match serves.as_slice() {
                [] => true,
                [(arcane, divine)] => *arcane || *divine,
                [(a1, d1), (a2, d2)] => (*a1 && *d2) || (*d1 && *a2),
                _ => false,
            };
        if !fits {
            self.fail(
                Choice::Advances {
                    class: prestige.key.clone(),
                },
                Issue::NotAllowed,
            );
        }
    }

    /// Adds one level of the Level-up's class and the features and feats its row grants. Returns the class level
    /// reached.
    pub(super) fn take_class_level(&mut self, level: u8, level_up: &LevelUp) -> u8 {
        let catalog = &self.engine.catalog;
        let class = catalog.class(&level_up.class);
        let index = match self
            .state
            .classes
            .iter()
            .position(|c| c.class == level_up.class)
        {
            Some(index) => {
                self.state.classes[index].level += 1;
                index
            }
            None => {
                let choice = self.build.class_choices.get(&level_up.class);
                // Its class skills: an archetype's list where it replaces the class's.
                if let Some((class, _)) = class {
                    let archetype = choice.and_then(|choice| choice.archetype.as_ref());
                    let skills = catalog
                        .archetype_of(class, archetype)
                        .and_then(|archetype| archetype.class_skills.as_ref())
                        .unwrap_or(&class.class_skills);
                    for skill in skills {
                        self.hold(Held::ClassSkill(skill.clone()), None);
                    }
                }
                self.state.classes.push(ClassLevels {
                    class: level_up.class.clone(),
                    prestige: class.is_some_and(|(_, prestige)| prestige),
                    archetype: choice.and_then(|choice| choice.archetype.clone()),
                    advances: choice
                        .map(|choice| choice.advances.clone())
                        .unwrap_or_default(),
                    level: 1,
                });
                // The class's components and its archetype's, held from its first level on (ticket 295).
                self.apply_components(|| Holder::Fact(Fact::Class(level_up.class.clone())));
                if let Some(archetype) = choice.and_then(|choice| choice.archetype.as_ref()) {
                    self.apply_components(|| Holder::Fact(Fact::Archetype(archetype.clone())));
                }
                self.state.classes.len() - 1
            }
        };
        let class_level = self.state.classes[index].level;
        let Some((class, _)) = class else {
            return class_level;
        };
        let cap = class_level_cap(class);
        if class_level > cap {
            self.fail(Choice::Class { level }, Issue::ClassLevelCap { cap });
        }
        let archetype = self.state.classes[index].archetype.as_ref();
        if let Some(row) = catalog.row(class, archetype, class_level) {
            // Through `hold`, so a feature's ability bonus counts (Dragon Disciple's Strength +2).
            for feature in row.features {
                self.hold(Held::Feature(feature.clone()), None);
            }
            // Granted feats skip their prerequisites; one already held adds nothing. A parameterised one is held
            // for every value.
            for feat in row.feats {
                let values = catalog.every_value(feat);
                self.hold(Held::Feat(feat.clone(), values), None);
            }
            for granted in row.feats_for {
                self.hold(
                    Held::Feat(granted.feat.clone(), vec![granted.value.clone()]),
                    None,
                );
            }
            // Its selections are held from here, whatever their slots hold; its selections of feats aren't facts yet
            // (ticket 305).
            for selection in row.selections {
                if !catalog.offers_feats(selection) {
                    self.hold(Held::Selection(selection.clone()), None);
                }
            }
        }
        class_level
    }

    /// Levels that count toward another class's feature (Frost Power: Winter Witch levels stack with witch levels
    /// for Major and Grand Hexes): the other class's rows that grant it are reached at its own level plus those
    /// levels, so the rows past its own level hold it now. When the class reaches such a row itself, its own grant
    /// takes over. Nothing while the other class isn't held.
    pub(super) fn count_toward_features(&mut self) {
        let catalog = &self.engine.catalog;
        let mut toward: BTreeMap<(ClassKey, FeatureKey), u8> = BTreeMap::new();
        for entry in &self.state.classes {
            let Some((class, _)) = catalog.class(&entry.class) else {
                continue;
            };
            for counts in &class.counts_as {
                if let CountsToward::Feature(feature) = &counts.toward {
                    let levels = toward
                        .entry((counts.class.clone(), feature.clone()))
                        .or_default();
                    *levels = levels.saturating_add(entry.level);
                }
            }
        }
        for (key, levels) in toward {
            let (other, feature) = &key;
            let reached = self
                .state
                .classes
                .iter()
                .find(|c| &c.class == other)
                .and_then(|held| {
                    let (class, _) = catalog.class(other)?;
                    let archetype = catalog.archetype_of(class, held.archetype.as_ref());
                    let reach = held
                        .level
                        .saturating_add(levels)
                        .min(class_level_cap(class));
                    let grants: usize = catalog
                        .rows(class, archetype)
                        .filter(|row| row.level > held.level && row.level <= reach)
                        .map(|row| row.features.iter().filter(|f| *f == feature).count())
                        .sum();
                    Some(u8::try_from(grants).unwrap_or(u8::MAX))
                })
                .unwrap_or(0);
            let held = self.counted_features.get(&key).copied().unwrap_or(0);
            for _ in held..reached {
                self.hold(Held::Feature(feature.clone()), None);
            }
            for _ in reached..held {
                self.release(&Held::Feature(feature.clone()));
            }
            self.counted_features.insert(key, reached);
        }
    }

    /// The Level-up's ranks and its skill points. Ranks never go above the character level
    /// (`Build::keeps_fixed_rules`), so only unknown skills ([`Issue::UnknownKey`]) and overspent points
    /// ([`Issue::NotAllowed`]) fail.
    pub(super) fn skill_ranks(&mut self, level: u8, level_up: &LevelUp) -> SkillPoints {
        for (skill, ranks) in &level_up.skill_ranks {
            if self.engine.catalog.skill(skill).is_none() {
                self.fail(
                    Choice::SkillRanks {
                        level,
                        skill: skill.clone(),
                    },
                    Issue::UnknownKey,
                );
            }
            let total = self.state.skill_ranks.entry(skill.clone()).or_default();
            *total = total.saturating_add(*ranks);
        }

        let catalog = &self.engine.catalog;
        let class_points = catalog.class(&level_up.class).map_or(0, |(class, _)| {
            let archetype = self
                .state
                .classes
                .iter()
                .find(|c| c.class == level_up.class)
                .and_then(|c| c.archetype.as_ref());
            catalog
                .archetype_of(class, archetype)
                .and_then(|archetype| archetype.skill_points)
                .unwrap_or(class.skill_points)
        });
        // The score without gear, so the ranks stay valid whatever is worn.
        let intelligence = self.skill_point_scores().modifier(Ability::Intelligence);
        let race_points = catalog
            .racial_skill_points(&self.build.start)
            .saturating_add(self.component_skill_points(level));
        let available =
            (i16::from(class_points) + i16::from(intelligence)).max(1) + i16::from(race_points);
        let available = u8::try_from(available).unwrap_or(u8::MAX);
        let spent = level_up
            .skill_ranks
            .values()
            .fold(0u8, |sum, ranks| sum.saturating_add(*ranks));
        if spent > available {
            self.fail(Choice::SkillPoints { level }, Issue::NotAllowed);
        }
        SkillPoints { available, spent }
    }

    /// The slots of a Level-up, in checking order: the class row's Feature selections (filtered pools last, since
    /// the others admit their options), its selections of feats (those that nest lists last, since their lists depend
    /// on the selections), the race's (level 1), then the general slot (odd levels). A row numbers its selections of
    /// feats and its other selections apart, each in row order (ticket 301), so a link's slot ids are the ones the
    /// feat slots had.
    /// So a race or general pick can rely on what the class grants at the same level (a human warpriest's Dazzling
    /// Display on the class's Weapon Focus).
    pub(super) fn level_slots(&self, level: u8, level_up: &LevelUp) -> Vec<(SlotKind, SlotId)> {
        let catalog = &self.engine.catalog;
        let mut slots = Vec::new();
        let feat = |source, index: usize| SlotId::Feat {
            level,
            source,
            index: u8::try_from(index).unwrap_or(u8::MAX),
        };
        let mut selections = Vec::new();
        let mut class_slots = Vec::new();
        let mut nests = Vec::new();
        if let Some((class, _)) = catalog.class(&level_up.class)
            && let Some(entry) = self
                .state
                .classes
                .iter()
                .find(|c| c.class == level_up.class)
        {
            let row = catalog.row(class, entry.archetype.as_ref(), entry.level);
            let (feat_selections, row_selections) =
                catalog.split_selections(row.iter().flat_map(|row| row.selections));
            for (index, selection) in feat_selections.into_iter().enumerate() {
                let kind = (
                    SlotKind::Feat(selection.clone()),
                    feat(FeatSource::Class, index),
                );
                if catalog.nests(selection) {
                    nests.push(kind);
                } else {
                    class_slots.push(kind);
                }
            }
            for (index, selection) in row_selections.into_iter().enumerate() {
                selections.push((
                    SlotKind::Selection(selection.clone()),
                    SlotId::Selection {
                        level,
                        index: u8::try_from(index).unwrap_or(u8::MAX),
                    },
                ));
            }
        }
        // A filtered pool after the others, so the option that admits its options is held first (the Dual-Cursed
        // Oracle's revelation, keyed before the mystery, at level 1).
        let (pools, selections): (Vec<_>, Vec<_>) =
            selections.into_iter().partition(|(kind, _)| {
                matches!(kind, SlotKind::Selection(key)
                    if catalog.selection(key).is_some_and(|s| s.filtered_by.is_some()))
            });
        slots.extend(selections);
        slots.extend(pools);
        slots.extend(class_slots);
        slots.extend(nests);
        if level == 1 {
            for (index, selection) in self.race_slots().iter().enumerate() {
                slots.push((
                    SlotKind::Feat(selection.clone()),
                    feat(FeatSource::Race, index),
                ));
            }
        }
        if level % 2 == 1 {
            slots.push((
                SlotKind::Feat(FeatureSelectionKey::new(BASIC_FEAT_SELECTION)),
                feat(FeatSource::General, 0),
            ));
        }
        slots
    }

    /// The race's selections of feats at level 1, or its subrace's in their place (a Dual Heritage half-elf has
    /// none).
    fn race_slots(&self) -> &'a [FeatureSelectionKey] {
        let catalog = &self.engine.catalog;
        let Some(race) = self.state.race.as_ref().and_then(|key| catalog.race(key)) else {
            return &[];
        };
        let subrace = self
            .build
            .start
            .subrace
            .as_ref()
            .and_then(|key| self.engine.catalog.subrace(key))
            .filter(|(owner, _)| owner.key == race.key)
            .map(|(_, subrace)| subrace);
        match subrace {
            Some(subrace) => &subrace.selections,
            None => &race.selections,
        }
    }

    /// Settles the Feature selection slots in `slots` before they're filled: a selection granted only to a
    /// character holding none of its options gets no slot when one is held, and a selection offering exactly one
    /// option is granted that option with no pick. The rest stay slots.
    pub(super) fn grant_selections(
        &mut self,
        slots: Vec<(SlotKind, SlotId)>,
    ) -> Vec<(SlotKind, SlotId)> {
        slots
            .into_iter()
            .filter(|(kind, _)| self.opens_slot(kind, None))
            .collect()
    }

    /// Whether a granted selection needs no slot: it's for characters without its options and one is held, or it
    /// offers one option, which is then held (through it, from `source`).
    pub(super) fn grant_without_slot(
        &mut self,
        selection: &FeatureSelectionKey,
        source: Option<&GrantSource>,
    ) -> bool {
        let catalog = &self.engine.catalog;
        let Some(defined) = catalog.selection(selection) else {
            return false;
        };
        if defined.only_without_option
            && defined
                .options
                .iter()
                .chain(&defined.shared_options)
                .any(|option| self.state.option_count(option) > 0)
        {
            return true;
        }
        // `NoSelectionIf`'s facts (`BlueprintFeatureSelection.IsSelectionProhibited`): one held, no pick.
        if catalog
            .no_selection_if(selection)
            .is_some_and(|facts| facts.iter().any(|fact| fact.rank(&self.state) > 0))
        {
            return true;
        }
        let Some(option) = catalog.single_option(selection) else {
            return false;
        };
        if self.state.option_count(option) == 0 {
            self.take_option(selection, option, None, source);
        }
        true
    }

    /// Fills a level's own slots and the slots its picks grant, band by band in the game's order ([`Band`]). Within a
    /// band, a pick refused at its turn for an unmet prerequisite or a weapon not yet proficient
    /// ([`deferrable`](super::judge::deferrable)) is tried again after the band's other picks and the slots they
    /// grant, until no more holds: so it holds when some order of the band's picks holds it, each checked with the ones
    /// before it, as the game applies the picks of one band in the order the player makes them
    /// (`LevelUpController.AddAction`, WrathDecompiled 6ce4a6b). Never with itself, nor with a later band's pick, a
    /// Mythic rank's or a later level's. Any other refusal at its turn is final, though the game's player could take
    /// the band's picks in another order: trying every order is left out. So a pick is still refused when another
    /// pick of its band whose slot comes first holds what it refuses (a `Not`, a `NoOtherOptionOfGroup`, an
    /// `AlreadyHeld` or `MaxTimes`), or when its slot's list depends on the state (a selection of feats that nests
    /// lists, [`Catalog::slot_lists`]) and the option that gates its list is a later pick of the band
    /// (`NotAllowed`). The slots come back in the level's slot order, then the granted ones in the order granted.
    pub(super) fn fill_bands(
        &mut self,
        level: u8,
        level_slots: Vec<(SlotKind, SlotId)>,
    ) -> Vec<Slot> {
        let engine = self.engine;
        let mut queued: Vec<Option<(Band, SlotKind, SlotId)>> = (level_slots.into_iter())
            .map(|(kind, id)| Some((engine.catalog.band(&kind), kind, id)))
            .collect();
        let mut own: Vec<(usize, Slot)> = Vec::with_capacity(queued.len());
        let mut granted = Vec::new();
        for band in Band::ALL {
            self.deferred = Some(Vec::new());
            for (index, queued) in queued.iter_mut().enumerate() {
                if queued.as_ref().is_some_and(|(of, ..)| *of == band)
                    && let Some((_, kind, id)) = queued.take()
                {
                    own.push((index, self.fill(id, kind, None)));
                }
            }
            let deferred = loop {
                granted.extend(self.fill_pending_up_to(level, band));
                let mut deferred = self.deferred.take().unwrap_or_default();
                let held = self.retry(&mut deferred);
                if !held {
                    break deferred;
                }
                self.deferred = Some(deferred);
            };
            if let Some(later) = &self.probed.later
                && self.probed.closure.is_none()
            {
                self.probed.closure = Some((later.len(), self.state.clone()));
            }
            for Deferred { id, issue, .. } in deferred {
                self.fail(Choice::Slot(id), issue);
            }
        }
        own.sort_by_key(|(index, _)| *index);
        own.into_iter()
            .map(|(_, slot)| slot)
            .chain(granted)
            .collect()
    }

    /// Fills the selection and feat slots that options' rows and feats granted at this level, in order, until none
    /// are left (a pick can grant more).
    pub(super) fn fill_pending(&mut self, level: u8) -> Vec<Slot> {
        self.fill_pending_up_to(level, Band::Features)
    }

    /// [`fill_pending`](Self::fill_pending) for the granted slots of `band` or an earlier one, the others staying
    /// pending.
    fn fill_pending_up_to(&mut self, level: u8, band: Band) -> Vec<Slot> {
        let mut slots = Vec::new();
        while let Some(at) =
            (self.pending.iter()).position(|pending| self.pending_band(pending) <= band)
        {
            let (kind, granting) = match self.pending.remove(at) {
                Pending::Row(kind, granting) => (kind, granting),
                Pending::Feat(feat) => {
                    let selection =
                        (self.engine.catalog.feat(&feat)).and_then(|feat| feat.selection.as_ref());
                    if let Some(kind) =
                        selection.and_then(|selection| self.granted_slot(selection, None))
                    {
                        let index = self.feat_selection_slots;
                        self.feat_selection_slots = self.feat_selection_slots.saturating_add(1);
                        let id = SlotId::FeatSelection { level, index };
                        self.note_rank_slot(&id);
                        slots.push(self.fill(id, kind, None));
                    }
                    continue;
                }
            };
            self.granting = Some(granting);
            let id = match &kind {
                SlotKind::Selection(selection) => {
                    if self.grant_without_slot(selection, None) {
                        self.granting = None;
                        continue;
                    }
                    let index = self.option_slots;
                    self.option_slots = self.option_slots.saturating_add(1);
                    SlotId::OptionSelection { level, index }
                }
                _ => {
                    let index = self.option_feat_slots;
                    self.option_feat_slots = self.option_feat_slots.saturating_add(1);
                    SlotId::OptionFeat { level, index }
                }
            };
            self.note_rank_slot(&id);
            slots.push(self.fill(id, kind, None));
            self.granting = None;
        }
        slots
    }

    /// The band a pending slot's pick comes in ([`Catalog::band`]).
    fn pending_band(&self, pending: &Pending) -> Band {
        let catalog = &self.engine.catalog;
        match pending {
            Pending::Row(kind, _) => catalog.band(kind),
            Pending::Feat(feat) => (catalog.feat(feat))
                .and_then(|feat| self.nested_selection(feat.selection.as_ref()?))
                .map_or(Band::Features, |selection| catalog.slot_band(selection)),
        }
    }

    /// Notes a slot keyed by its place at the level as a Mythic rank's, while one is being taken.
    fn note_rank_slot(&mut self, id: &SlotId) {
        if let Some(rank) = self.ascending {
            self.rank_slots.push((id.clone(), rank));
        }
    }

    /// The slot a picked entry's selection opens (ticket 302), granted from `source`: a slot of its
    /// [`nested_selection`](Self::nested_selection), unless that one is granted with no pick
    /// ([`opens_slot`](Self::opens_slot)). `None`: no slot.
    pub(super) fn granted_slot(
        &mut self,
        key: &FeatureSelectionKey,
        source: Option<&GrantSource>,
    ) -> Option<SlotKind> {
        let kind = self.engine.catalog.slot_kind(self.nested_selection(key)?);
        self.opens_slot(&kind, source).then_some(kind)
    }

    /// The selection whose slot a picked entry's selection opens: the selection itself, as the game adds a picked
    /// selection to the level-up's (`SelectFeature.Apply`, WrathDecompiled 6ce4a6b). The planner's condition until
    /// 293 ([`needs_slot_of`](crate::FeatureSelection::needs_slot_of)): only after a slot of one of the selections it
    /// names, and the first of them the walk reached a slot of when it lists nothing of its own. `None`: no slot.
    fn nested_selection(&self, key: &FeatureSelectionKey) -> Option<&'a FeatureSelection> {
        let catalog = &self.engine.catalog;
        let selection = catalog.selection(key)?;
        if selection.needs_slot_of.is_empty() {
            return Some(selection);
        }
        let had =
            (self.selections_had.iter()).find(|had| selection.needs_slot_of.contains(*had))?;
        if selection.has_list() {
            Some(selection)
        } else {
            catalog.selection(had)
        }
    }

    /// Whether a granted selection's slot of `kind` takes a pick: a Feature selection granted with no pick
    /// ([`grant_without_slot`](Self::grant_without_slot)) has none.
    fn opens_slot(&mut self, kind: &SlotKind, source: Option<&GrantSource>) -> bool {
        !matches!(kind, SlotKind::Selection(selection) if self.grant_without_slot(selection, source))
    }

    // ---- Tables -------------------------------------------------------------------------------------------

    /// BAB and saves from every class's progressions (its archetype's, for a save it replaces) at its class
    /// level; spell-level access and caster level
    /// from its row at its class level plus the prestige levels that advance it.
    pub(super) fn recompute_tables(&mut self) {
        let catalog = &self.engine.catalog;
        let state = &self.state;
        let (mut bab, mut fort, mut reflex, mut will) = (0u8, 0u8, 0u8, 0u8);
        let (mut arcane, mut divine, mut caster_level) = (0u8, 0u8, 0u8);
        let (mut spontaneous_arcane, mut spontaneous_divine) = (0u8, 0u8);
        // A class's own spellbook, worked out only for data whose prerequisites read it (ticket 295).
        let spellbooks = catalog.reads_spellbooks();
        let (mut class_spell_levels, mut caster_types) = (BTreeMap::new(), BTreeSet::new());
        for entry in &state.classes {
            let Some((class, prestige)) = catalog.class(&entry.class) else {
                continue;
            };
            // A level past the class's table (which fails) adds none.
            if entry.level <= class_level_cap(class) {
                let level = entry.level;
                bab = bab.saturating_add(class.base_attack_bonus(level));
                let archetype = catalog.archetype_of(class, entry.archetype.as_ref());
                let [f, r, w] =
                    Save::ALL.map(|save| class.save_bonus(save, level, prestige, archetype));
                fort = fort.saturating_add(f);
                reflex = reflex.saturating_add(r);
                will = will.saturating_add(w);
            }

            // Spellcasting: this class's own level plus the prestige levels that advance it.
            let archetype = entry.archetype.as_ref();
            let (casts_arcane, casts_divine) = catalog.casts(class, archetype);
            let advanced =
                advancing_levels(catalog, state, &entry.class, casts_arcane, casts_divine);
            let effective = entry.level.saturating_add(advanced);
            if spellbooks {
                if casts_arcane {
                    caster_types.insert(SpellType::Arcane);
                }
                if casts_divine {
                    caster_types.insert(SpellType::Divine);
                }
            }
            if let Some(row) = catalog.row(class, archetype, effective.min(class_level_cap(class)))
            {
                // A class with a spellbook (it casts at some level) is there from its first level, at 0 before its
                // first spell level: `Spellbook.GetMaxSpellLevel` of a row listing level 0 (ticket 295).
                if spellbooks && (casts_arcane || casts_divine) {
                    let own = row.spell_levels.arcane.max(row.spell_levels.divine);
                    class_spell_levels.insert(entry.class.clone(), own);
                }
                arcane = arcane.max(row.spell_levels.arcane);
                divine = divine.max(row.spell_levels.divine);
                if catalog.spontaneous(class, archetype) {
                    spontaneous_arcane = spontaneous_arcane.max(row.spell_levels.arcane);
                    spontaneous_divine = spontaneous_divine.max(row.spell_levels.divine);
                }
                if row.spell_levels.any() {
                    caster_level = caster_level.max(effective);
                }
            }
        }
        // Feats see the levels of classes that count as another (Diverse Training).
        let mut feat_class_levels: BTreeMap<ClassKey, u8> = BTreeMap::new();
        for entry in &state.classes {
            let Some((class, _)) = catalog.class(&entry.class) else {
                continue;
            };
            for counts in &class.counts_as {
                if counts.toward == CountsToward::FeatPrerequisites {
                    let levels = feat_class_levels
                        .entry(counts.class.clone())
                        .or_insert_with(|| state.class_level(&counts.class));
                    *levels = levels.saturating_add(entry.level);
                }
            }
        }
        let state = &mut self.state;
        state.feat_class_levels = feat_class_levels;
        state.bab = bab;
        self.class_bab = bab;
        state.fort = fort;
        state.reflex = reflex;
        state.will = will;
        state.arcane_spell_level = arcane;
        state.divine_spell_level = divine;
        state.spontaneous_arcane_spell_level = spontaneous_arcane;
        state.spontaneous_divine_spell_level = spontaneous_divine;
        state.caster_level = caster_level;
        state.class_spell_levels = class_spell_levels;
        state.caster_types = caster_types;
    }
}

/// Prestige levels that advance `class`: rows marked `advances_arcane` count when `class` casts arcane spells,
/// `advances_divine` when it casts divine ones, in prestige classes that chose `class` on entry.
pub(in crate::engine) fn advancing_levels(
    catalog: &Catalog,
    state: &CharacterState,
    class: &ClassKey,
    casts_arcane: bool,
    casts_divine: bool,
) -> u8 {
    let mut levels = 0u8;
    for prestige in state.classes.iter().filter(|c| c.advances.contains(class)) {
        let Some((prestige_class, _)) = catalog.class(&prestige.class) else {
            continue;
        };
        let count = prestige_class
            .levels
            .iter()
            .filter(|row| row.level <= prestige.level)
            .filter(|row| {
                (row.advances_arcane && casts_arcane) || (row.advances_divine && casts_divine)
            })
            .count();
        levels = levels.saturating_add(u8::try_from(count).unwrap_or(u8::MAX));
    }
    levels
}

/// The advancements a prestige class first taken at a character level can name: every set of the classes the Build
/// takes before that level (in the order it first takes them), from one class up to as many as the prestige class
/// may advance ([`most_advanced`]), then none. The walk keeps those that fit ([`Walker::check_advances`]); the
/// cursor judges a class option by them, and the advancement picker lists the ones the Build stays possible with.
pub(in crate::engine) fn advancements(
    build: &Build,
    level: u8,
    prestige: &Class,
) -> Vec<BTreeSet<ClassKey>> {
    let mut held: Vec<&ClassKey> = Vec::new();
    for level_up in build
        .levels
        .iter()
        .take(usize::from(level).saturating_sub(1))
    {
        if !held.contains(&&level_up.class) {
            held.push(&level_up.class);
        }
    }
    let mut sets: Vec<BTreeSet<ClassKey>> = (1..=most_advanced(prestige))
        .flat_map(|size| subsets(&held, size))
        .collect();
    sets.push(BTreeSet::new());
    sets
}

/// Whether a class is first taken at this character level: no level before it takes it.
pub(in crate::engine) fn first_taken_at(build: &Build, class: &ClassKey, level: u8) -> bool {
    !build
        .levels
        .iter()
        .take(usize::from(level).saturating_sub(1))
        .any(|level_up| &level_up.class == class)
}

/// Every set of `size` of the items, in order: by the first item's place, then the next's.
fn subsets(items: &[&ClassKey], size: usize) -> Vec<BTreeSet<ClassKey>> {
    if size == 0 {
        return vec![BTreeSet::new()];
    }
    let mut sets = Vec::new();
    for (index, first) in items.iter().enumerate() {
        for mut rest in subsets(&items[index + 1..], size - 1) {
            rest.insert((*first).clone());
            sets.push(rest);
        }
    }
    sets
}
