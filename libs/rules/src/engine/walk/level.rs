//! A Level-up's own steps: the class level and what its row grants, skill ranks, the level's slots, and the
//! tables (BAB, saves, spellcasting) after it.

use std::collections::{BTreeMap, BTreeSet};

use super::{GrantSource, Held, Pending, Walker};
use crate::build::{FeatSource, LevelUp, SlotId};
use crate::choice::Choice;
use crate::engine::catalog::Catalog;
use crate::engine::{
    SkillPoints, Slot, SlotKind, advanced_spellcasting, class_level_cap, effective_rows,
    most_advanced,
};
use crate::game_data::{Ability, Class, CountsToward, FeatSlot, Save};
use crate::keys::{ClassKey, FeatKey, FeatureKey, FeatureSelectionKey};
use crate::state::{CharacterState, ClassLevels};

impl<'a> Walker<'a> {
    /// On a class's first level: the class, archetype and advancement choices are checked against the character
    /// before the level. An archetype stating an alignment requirement replaces the class's
    /// ([`Catalog::entry_prerequisites`]).
    pub(super) fn check_class_entry(&mut self, level: u8, level_up: &LevelUp) {
        if self.state.class_level(&level_up.class) > 0 {
            return;
        }
        let catalog = &self.engine.catalog;
        let Some((class, _)) = catalog.class(&level_up.class) else {
            self.fail(Choice::Class { level });
            return;
        };
        let choice = self.build.class_choices.get(&level_up.class);
        let key = choice.and_then(|choice| choice.archetype.as_ref());
        let archetype = catalog.archetype_of(class, key);
        let [class_needs, archetype_needs] = catalog.entry_prerequisites(class, archetype);
        let fails = |needs: Option<&_>| needs.is_some_and(|needs| self.state.check(needs).is_err());
        let class_fails = fails(class_needs);
        // A key the data lacks, an archetype of another class, or one whose prerequisites aren't met.
        let archetype_fails = key.is_some() && (archetype.is_none() || fails(archetype_needs));
        if class_fails {
            self.fail(Choice::Class { level });
        }
        if archetype_fails {
            self.fail(Choice::Archetype {
                class: level_up.class.clone(),
            });
        }
        if let Some(choice) = choice
            && !choice.advances.is_empty()
        {
            self.check_advances(class, &choice.advances);
        }
    }

    /// A prestige class advances at most one arcane and one divine class (one class in all when it advances
    /// `Either`: [`most_advanced`]), each already taken and casting the spells it advances.
    fn check_advances(&mut self, prestige: &Class, advances: &BTreeSet<ClassKey>) {
        let catalog = &self.engine.catalog;
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
            self.fail(Choice::Advances {
                class: prestige.key.clone(),
            });
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
                self.state.classes.len() - 1
            }
        };
        let class_level = self.state.classes[index].level;
        let Some((class, _)) = class else {
            return class_level;
        };
        let cap = class_level_cap(class);
        if class_level > cap {
            self.fail(Choice::Class { level });
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
                    let grants: usize = effective_rows(class, archetype)
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
    /// (`Build::keeps_fixed_rules`), so only unknown skills and overspent points fail.
    pub(super) fn skill_ranks(&mut self, level: u8, level_up: &LevelUp) -> SkillPoints {
        for (skill, ranks) in &level_up.skill_ranks {
            if self.engine.catalog.skill(skill).is_none() {
                self.fail(Choice::SkillRanks {
                    level,
                    skill: skill.clone(),
                });
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
        let intelligence = self.state.ability_scores.modifier(Ability::Intelligence);
        let available = (i16::from(class_points) + i16::from(intelligence)).max(1);
        let available = u8::try_from(available).unwrap_or(u8::MAX);
        let spent = level_up
            .skill_ranks
            .values()
            .fold(0u8, |sum, ranks| sum.saturating_add(*ranks));
        if spent > available {
            self.fail(Choice::SkillPoints { level });
        }
        SkillPoints { available, spent }
    }

    /// The slots of a Level-up, in checking order: the class row's Feature selections (pool selections last, since
    /// they draw on the others), its feat slots (option-list ones last, since they depend on the selections), the
    /// race's (level 1), then the general slot (odd levels).
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
        let mut option_lists = Vec::new();
        if let Some((class, _)) = catalog.class(&level_up.class)
            && let Some(entry) = self
                .state
                .classes
                .iter()
                .find(|c| c.class == level_up.class)
        {
            let row = catalog.row(class, entry.archetype.as_ref(), entry.level);
            for (index, slot) in row.iter().flat_map(|row| row.feat_slots).enumerate() {
                let kind = (SlotKind::Feat(slot.clone()), feat(FeatSource::Class, index));
                if matches!(slot, FeatSlot::OptionList { .. }) {
                    option_lists.push(kind);
                } else {
                    class_slots.push(kind);
                }
            }
            for (index, selection) in row.iter().flat_map(|row| row.selections).enumerate() {
                selections.push((
                    SlotKind::Selection(selection.clone()),
                    SlotId::Selection {
                        level,
                        index: u8::try_from(index).unwrap_or(u8::MAX),
                    },
                ));
            }
        }
        // A pool selection after the others, so the option whose pool it offers is held first (the Dual-Cursed
        // Oracle's revelation, keyed before the mystery, at level 1).
        let (pools, selections): (Vec<_>, Vec<_>) =
            selections.into_iter().partition(|(kind, _)| {
                matches!(kind, SlotKind::Selection(key)
                    if catalog.selection(key).is_some_and(|s| s.pool_of.is_some()))
            });
        slots.extend(selections);
        slots.extend(pools);
        slots.extend(class_slots);
        slots.extend(option_lists);
        if level == 1 {
            for (index, slot) in self.race_slots().iter().enumerate() {
                slots.push((SlotKind::Feat(slot.clone()), feat(FeatSource::Race, index)));
            }
        }
        if level % 2 == 1 {
            slots.push((
                SlotKind::Feat(FeatSlot::General),
                feat(FeatSource::General, 0),
            ));
        }
        slots
    }

    /// The race's feat slots at level 1, or its subrace's in their place (a Dual Heritage half-elf has none).
    fn race_slots(&self) -> &'a [FeatSlot] {
        let catalog = &self.engine.catalog;
        let Some(race) = self.state.race.as_ref().and_then(|key| catalog.race(key)) else {
            return &[];
        };
        let subrace = self
            .build
            .start
            .subrace
            .as_ref()
            .and_then(|key| race.subraces.iter().find(|subrace| &subrace.key == key));
        match subrace {
            Some(subrace) => &subrace.feat_slots,
            None => &race.feat_slots,
        }
    }

    /// Settles the Feature selection slots in `slots` before they're filled: a selection granted only to a
    /// character holding none of its options gets no slot when one is held, and a selection offering exactly one
    /// option is granted that option with no pick. The rest stay slots.
    pub(super) fn grant_selections(
        &mut self,
        slots: Vec<(SlotKind, SlotId)>,
    ) -> Vec<(SlotKind, SlotId)> {
        let mut kept = Vec::with_capacity(slots.len());
        for (kind, id) in slots {
            match &kind {
                SlotKind::Selection(selection) => {
                    if self.grant_without_slot(selection, None) {
                        continue;
                    }
                    kept.push((kind, id));
                }
                _ => kept.push((kind, id)),
            }
        }
        kept
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
                .offered()
                .iter()
                .any(|option| self.state.option_count(option) > 0)
        {
            return true;
        }
        let Some(option) = catalog.single_option(selection) else {
            return false;
        };
        if self.state.option_count(&option) == 0 {
            self.take_option(selection, &option, None, source);
        }
        true
    }

    /// Fills the selection and feat slots that options' rows and feats granted at this level, in order, until none
    /// are left (a pick can grant more).
    pub(super) fn fill_pending(&mut self, level: u8) -> Vec<Slot> {
        let mut slots = Vec::new();
        while !self.pending.is_empty() {
            let (kind, granting) = match self.pending.remove(0) {
                Pending::Row(kind, granting) => (kind, granting),
                Pending::Feat(feat) => {
                    if let Some(selection) = self.extra_pick_selection(&feat)
                        && !self.grant_without_slot(&selection, None)
                    {
                        let index = self.feat_selection_slots;
                        self.feat_selection_slots = self.feat_selection_slots.saturating_add(1);
                        let id = SlotId::FeatSelection { level, index };
                        slots.push(self.fill(id, SlotKind::Selection(selection), None));
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
            slots.push(self.fill(id, kind, None));
            self.granting = None;
        }
        slots
    }

    /// The selection a feat's extra pick is of: the one of its [`extra_pick_of`](crate::Feat::extra_pick_of) the
    /// walk reached a slot of first, if any.
    fn extra_pick_selection(&self, feat: &FeatKey) -> Option<FeatureSelectionKey> {
        let of = &self.engine.catalog.feat(feat)?.extra_pick_of;
        self.selections_had
            .iter()
            .find(|selection| of.contains(*selection))
            .cloned()
    }

    // ---- Tables -------------------------------------------------------------------------------------------

    /// BAB and saves from every class's progressions at its class level; spell-level access and caster level
    /// from its row at its class level plus the prestige levels that advance it.
    pub(super) fn recompute_tables(&mut self) {
        let catalog = &self.engine.catalog;
        let state = &self.state;
        let (mut bab, mut fort, mut reflex, mut will) = (0u8, 0u8, 0u8, 0u8);
        let (mut arcane, mut divine, mut caster_level) = (0u8, 0u8, 0u8);
        let (mut spontaneous_arcane, mut spontaneous_divine) = (0u8, 0u8);
        for entry in &state.classes {
            let Some((class, prestige)) = catalog.class(&entry.class) else {
                continue;
            };
            // A level past the class's table (which fails) adds none.
            if entry.level <= class_level_cap(class) {
                let level = entry.level;
                bab = bab.saturating_add(class.base_attack_bonus(level));
                let [f, r, w] = Save::ALL.map(|save| class.save_bonus(save, level, prestige));
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
            if let Some(row) = catalog.row(class, archetype, effective.min(class_level_cap(class)))
            {
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
        state.fort = fort;
        state.reflex = reflex;
        state.will = will;
        state.arcane_spell_level = arcane;
        state.divine_spell_level = divine;
        state.spontaneous_arcane_spell_level = spontaneous_arcane;
        state.spontaneous_divine_spell_level = spontaneous_divine;
        state.caster_level = caster_level;
    }
}

/// Prestige levels that advance `class`: rows marked `advances_arcane` count when `class` casts arcane spells,
/// `advances_divine` when it casts divine ones, in prestige classes that chose `class` on entry.
fn advancing_levels(
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
