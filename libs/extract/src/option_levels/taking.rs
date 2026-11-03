//! Taking a family where its class is built ([`Taking`]): planning a table's cells before its rows are read, and
//! taking the family once every table of the class is read.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArchetypeLevelRow, Class, FeatKey, FeatListKey, FeatSlot, FeatureKey, FeatureSelection,
    FeatureSelectionKey, SelectionOptionKey,
};

use super::class_takes::{Merged, SUMMARIES, Take, Takes};
use super::rows::row_names;
use super::{Family, bare};

/// A family and how its class's tables take it, given to the class stage through [`Picks`](crate::picks::Picks).
pub(crate) struct Taking {
    pub family: Family,
    pub takes: &'static Takes,
}

/// A cell of a family's class's table as the class stage reads it: the feature it names (key and name), or the feat
/// it grants.
pub(crate) struct Cell {
    pub level: u8,
    pub text: String,
    pub feature: Option<(FeatureKey, String)>,
    pub feat: Option<FeatKey>,
}

/// What a cell of a family's class's table is, in place of the feature or feat it names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Planned {
    /// A placeholder ("Bloodline Ability": the held option's row) or the choice ("Order": the selection). Nothing on
    /// the row. `kept`: its feature is still defined, off the rows, since a line elsewhere resolves to it.
    Marker { kept: bool },
    /// What a fixed option's row gives, listed in the archetype's own table: the option's row stands there.
    Covered,
}

/// How one table of a family's class takes the family, planned from its cells before its rows are built.
#[derive(Default)]
pub(crate) struct TablePlan {
    /// The archetype's way of taking it; `None` for the class's own table.
    take: Option<Take>,
    cells: Vec<(u8, String, Planned)>,
    /// The placeholder and choice cells' features by level: what the rows would hold if they were features, for
    /// telling which of an archetype's rows are its class's.
    markers: BTreeMap<u8, Vec<(FeatureKey, String)>>,
    /// The fixed option's levels the archetype's table doesn't give (`Take::Fixed`).
    fixed_skipped: BTreeSet<u8>,
}

impl TablePlan {
    /// What the cell `cell` at `level` is, if the plan says.
    pub(crate) fn planned(&self, level: u8, cell: &str) -> Option<Planned> {
        self.cells
            .iter()
            .find(|(l, c, _)| *l == level && c == cell)
            .map(|(_, _, planned)| *planned)
    }

    fn markers_at(&self, level: u8) -> &[(FeatureKey, String)] {
        self.markers.get(&level).map_or(&[], Vec::as_slice)
    }
}

impl Taking {
    fn is_placeholder(&self, name: &str) -> bool {
        let name = bare(name);
        self.takes.placeholders.iter().any(|p| bare(p) == name)
    }

    fn is_choice(&self, name: &str) -> bool {
        let name = bare(name);
        self.takes.choices.iter().any(|c| bare(c) == name)
    }

    fn option_levels(&self) -> BTreeSet<u8> {
        self.family
            .selection
            .options
            .iter()
            .flat_map(|o| o.levels.iter().map(|row| row.level))
            .collect()
    }

    /// How a table of the family's class takes the family (the class's own when `archetype` is `None`), from its
    /// cells: its placeholder and choice cells are markers, and a fixed option's features its own table lists where
    /// the option's row stands are that row. An archetype the guide doesn't list takes the family when its table
    /// names a placeholder or the choice; one that takes none keeps its cells as they are.
    pub(crate) fn plan(&self, archetype: Option<&str>, cells: &[Cell]) -> TablePlan {
        let takes = self.takes;
        let marker = |name: &str| self.is_placeholder(name) || self.is_choice(name);
        let mut plan = TablePlan::default();
        let take = match archetype {
            None if !takes.base_takes => return plan,
            None => None,
            Some(archetype) => {
                let listed = takes
                    .archetypes
                    .iter()
                    .find(|(key, _)| *key == archetype)
                    .map(|(_, take)| *take);
                let names_any = cells
                    .iter()
                    .filter(|cell| cell.level <= 20)
                    .any(|cell| cell.feature.as_ref().is_some_and(|(_, name)| marker(name)));
                let take = match listed {
                    Some(take) => take,
                    None if names_any && takes.base_takes => Take::Yes,
                    None => Take::No,
                };
                plan.take = Some(take);
                if let Take::No = take {
                    return plan;
                }
                Some(take)
            }
        };
        let second = match take {
            Some(Take::SecondListOnly(choice)) => Some(choice),
            _ => None,
        };
        // A fixed option's row stands where the archetype's table lists every feature it gives.
        let mut fixed_names: BTreeMap<u8, Vec<String>> = BTreeMap::new();
        let mut fixed_feats: BTreeMap<u8, BTreeSet<FeatKey>> = BTreeMap::new();
        if let Some(Take::Fixed(option)) = take {
            let option = self.family.option(option);
            for level in self.option_levels() {
                let row_level = self.family.ties.get(&level).copied().unwrap_or(level);
                let names: Vec<String> = cells
                    .iter()
                    .filter(|cell| cell.level == row_level)
                    .filter_map(|cell| cell.feature.as_ref())
                    .map(|(_, name)| bare(name))
                    .collect();
                let covered = option
                    .levels
                    .iter()
                    .find(|row| row.level == row_level)
                    .is_none_or(|row| {
                        row_names(&self.family, row)
                            .iter()
                            .all(|name| names.contains(name))
                    });
                if !covered {
                    plan.fixed_skipped.insert(level);
                }
            }
            for row in &option.levels {
                if !plan.fixed_skipped.contains(&row.level) {
                    fixed_names.insert(row.level, row_names(&self.family, row));
                    fixed_feats.insert(row.level, row.feats.clone());
                }
            }
        }
        for cell in cells {
            let planned = match &cell.feature {
                Some((key, name))
                    if marker(name) || second.is_some_and(|choice| name == choice) =>
                {
                    plan.markers
                        .entry(cell.level)
                        .or_default()
                        .push((key.clone(), name.clone()));
                    Some(Planned::Marker {
                        kept: takes.kept_choices && self.is_choice(name),
                    })
                }
                Some((_, name))
                    if fixed_names
                        .get(&cell.level)
                        .is_some_and(|names| names.contains(&bare(name))) =>
                {
                    Some(Planned::Covered)
                }
                Some(_) => None,
                None => cell
                    .feat
                    .as_ref()
                    .filter(|feat| {
                        fixed_feats
                            .get(&cell.level)
                            .is_some_and(|feats| feats.contains(feat))
                    })
                    .map(|_| Planned::Covered),
            };
            if let Some(planned) = planned {
                plan.cells.push((cell.level, cell.text.clone(), planned));
            }
        }
        plan
    }

    /// The family taken once every table of its class is read, on the full rows: the class's row grants the
    /// selection; each archetype that takes it grants the class's selection or, when it skips option levels or limits
    /// the options, its own sharing them (an option level whose row its table has no placeholder for is skipped, and
    /// its own row stays); the bonus slots that took the merged lists pick from the held option's list. An archetype
    /// that takes none keeps its rows, but those that are its class's follow the class's.
    pub(crate) fn take(
        &self,
        class: &mut Class,
        plans: &BTreeMap<String, TablePlan>,
        lists: &mut crate::feat_lists::PendingLists,
    ) {
        let family = &self.family;
        let takes = self.takes;
        let selection_key = family.selection.key.clone();
        let none = TablePlan::default();
        let plan_of = |table: &str| plans.get(table).unwrap_or(&none);
        let base_plan = plan_of(class.key.as_str());
        let option_levels = self.option_levels();

        // The merged lists: the pending lists the merged tables' bonus slots name.
        let merged_in = |table: &str| match takes.merged {
            Merged::None => false,
            Merged::Every => true,
            Merged::Tables(tables) => tables.contains(&table),
        };
        let one_of = |slots: &[FeatSlot]| -> Vec<FeatListKey> {
            slots
                .iter()
                .filter_map(|slot| match slot {
                    FeatSlot::OneOf { list } => Some(list.clone()),
                    _ => None,
                })
                .collect()
        };
        let mut merged: BTreeSet<FeatListKey> = BTreeSet::new();
        if merged_in(class.key.as_str()) {
            for row in &class.levels {
                merged.extend(one_of(&row.feat_slots));
            }
        }
        for archetype in &class.archetypes {
            if merged_in(archetype.key.as_str()) {
                for row in &archetype.levels {
                    merged.extend(one_of(&row.feat_slots));
                }
            }
        }
        let option_list = || FeatSlot::OptionList {
            selection: selection_key.clone(),
            at_level: None,
            label: None,
        };

        // An archetype row is its class's when it holds the same, its markers counted as features, as making the
        // rows sparse tells.
        let base_before = class.levels.clone();
        let same_as_base = |row: &ArchetypeLevelRow, plan: &TablePlan| -> bool {
            let Some(base) = base_before.iter().find(|base| base.level == row.level) else {
                return false;
            };
            let keys = |features: &[FeatureKey], markers: &[(FeatureKey, String)]| {
                let mut keys: Vec<FeatureKey> = features.to_vec();
                keys.extend(markers.iter().map(|(key, _)| key.clone()));
                keys.sort();
                keys
            };
            fn sorted<T: Ord + Clone>(items: &[T]) -> Vec<T> {
                let mut items = items.to_vec();
                items.sort();
                items
            }
            row.spell_levels
                .is_none_or(|spells| spells == base.spell_levels())
                && keys(&row.features, plan.markers_at(row.level))
                    == keys(&base.features, base_plan.markers_at(row.level))
                && row.feats == base.feats
                && sorted(&row.feat_slots) == sorted(&base.feat_slots)
                && sorted(&row.selections) == sorted(&base.selections)
        };
        let base_placeholder = takes.placeholders.iter().any(|p| {
            base_plan
                .markers
                .values()
                .flatten()
                .any(|(_, name)| name == p)
        });

        // The base class: its row grants the selection, its bonus slots take the option's list.
        if takes.base_takes {
            for row in &mut class.levels {
                if row.level == takes.level && !row.selections.contains(&selection_key) {
                    row.selections.push(selection_key.clone());
                }
                for slot in &mut row.feat_slots {
                    if let FeatSlot::OneOf { list } = slot
                        && merged.contains(list)
                    {
                        lists.stand_in(&class.key, row.level, list);
                        *slot = option_list();
                    }
                }
            }
        }
        let base_after = class.levels.clone();

        for archetype in &mut class.archetypes {
            let plan = plan_of(archetype.key.as_str());
            let take = plan.take.unwrap_or(Take::No);
            if let Take::No = take {
                // Its rows that are the class's stay the class's: without the grant at the grant level.
                if takes.base_takes {
                    for row in &mut archetype.levels {
                        if !same_as_base(row, plan) {
                            continue;
                        }
                        let base = base_after
                            .iter()
                            .find(|base| base.level == row.level)
                            .expect("the class's row");
                        row.features = base.features.clone();
                        row.feats = base.feats.clone();
                        row.feat_slots = base.feat_slots.clone();
                        row.selections = base.selections.clone();
                        if row.level == takes.level {
                            row.selections.retain(|s| *s != selection_key);
                        }
                    }
                }
                continue;
            }
            let skipped: BTreeSet<u8> = match take {
                Take::Fixed(_) => plan.fixed_skipped.clone(),
                _ => option_levels
                    .iter()
                    .copied()
                    .filter(|level| {
                        let row_level = family.ties.get(level).copied().unwrap_or(*level);
                        let has_own = archetype
                            .levels
                            .iter()
                            .find(|row| row.level == row_level)
                            .is_some_and(|row| !same_as_base(row, plan));
                        let covered = !takes.skips_without_placeholder
                            // The base sorcerer has no table: its rows cover every level.
                            || (!has_own && !base_placeholder)
                            // The choice's own level gets the option's row too (an order's level-1 skills).
                            || plan
                                .markers_at(row_level)
                                .iter()
                                .any(|(_, name)| self.is_placeholder(name) || self.is_choice(name));
                        !covered
                    })
                    .collect(),
            };
            // Restrict to options or a second pick: an own selection sharing the base's options.
            let own_selection =
                |suffix: &str| FeatureSelectionKey::new(format!("{}/{suffix}", archetype.key));
            let shared: Option<BTreeSet<SelectionOptionKey>> = match take {
                Take::Only(names) => Some(
                    family
                        .option_keys()
                        .into_iter()
                        .filter(|key| {
                            let last = key.as_str().rsplit('/').next().unwrap_or_default();
                            names
                                .iter()
                                .any(|name| last == *name || last.starts_with(&format!("{name}-")))
                        })
                        .collect(),
                ),
                Take::Fixed(option) => Some(BTreeSet::from([SelectionOptionKey::new(option)])),
                _ => None,
            };
            let grant = if shared.is_some() || !skipped.is_empty() {
                let key = own_selection(
                    selection_key
                        .as_str()
                        .rsplit('/')
                        .next()
                        .unwrap_or_default(),
                );
                archetype.selections.push(FeatureSelection {
                    key: key.clone(),
                    name: format!("{} ({})", family.selection.name, archetype.name),
                    summary: SUMMARIES
                        .iter()
                        .find(|(of, _)| *of == archetype.key.as_str())
                        .map(|(_, summary)| (*summary).to_owned())
                        .unwrap_or_default(),
                    guide: archetype.guide.clone(),
                    options: Vec::new(),
                    shared_options: shared
                        .unwrap_or_else(|| family.option_keys().into_iter().collect()),
                    skipped_levels: skipped,
                    ..family.selection.clone()
                });
                key
            } else {
                selection_key.clone()
            };
            // The second pick's list-only selection.
            let second = match take {
                Take::SecondListOnly(_) => {
                    let key = own_selection("second-bloodline");
                    archetype.selections.push(FeatureSelection {
                        key: key.clone(),
                        name: format!("Second {} ({})", family.selection.name, archetype.name),
                        guide: archetype.guide.clone(),
                        options: Vec::new(),
                        shared_options: family.option_keys().into_iter().collect(),
                        feat_lists_only: true,
                        ..family.selection.clone()
                    });
                    Some(key)
                }
                _ => None,
            };
            for row in &mut archetype.levels {
                if row.level == takes.level {
                    row.selections.retain(|s| *s != selection_key);
                    row.selections.push(grant.clone());
                    row.selections.extend(second.clone());
                }
                for slot in &mut row.feat_slots {
                    if let FeatSlot::OneOf { list } = slot
                        && merged.contains(list)
                    {
                        *slot = option_list();
                    }
                }
            }
        }
        for (archetype, cell, choice, expected) in takes.row_choices {
            let plan = plan_of(archetype);
            let levels: Vec<u8> = plan
                .markers
                .iter()
                .filter(|(_, markers)| markers.iter().any(|(_, name)| name == cell))
                .map(|(level, _)| *level)
                .collect();
            assert_eq!(levels, *expected, "{archetype}: {cell}");
            let archetype = class
                .archetypes
                .iter_mut()
                .find(|a| a.key.as_str() == *archetype)
                .unwrap_or_else(|| panic!("no archetype {archetype}"));
            for row in &mut archetype.levels {
                if levels.contains(&row.level) {
                    row.selections.push(FeatureSelectionKey::new(*choice));
                }
            }
        }
    }
}
