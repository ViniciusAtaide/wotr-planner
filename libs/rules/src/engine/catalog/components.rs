//! The entries' components (ticket 295), as the catalog keeps them: `Requires` compiled into the entries'
//! prerequisites and `NoSelectionIf`'s own-options rule into its selection at load, and the rest indexed by the entry
//! that holds them, for the walk to evaluate while it holds that entry.

use std::collections::BTreeSet;

use rustc_hash::FxHashMap;

use super::Catalog;
use crate::component::{Component, Fact, Holder, compile_requires};
use crate::game_data::GameData;
use crate::keys::{ArchetypeKey, ClassKey, FeatureSelectionKey};
use crate::prerequisite::Prerequisite;

/// The kinds of entry that hold components, as the index keeps them apart: one table each, by the key's text, so the
/// walk finds an entry's components as it holds it without building a [`Holder`] (a copy of the key).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::engine) enum HolderKind {
    Feat,
    Feature,
    Option,
    Selection,
    MythicAbility,
    MythicFeat,
    Class,
    Archetype,
    Race,
    Deity,
    Subrace,
    Background,
}

const HOLDER_KINDS: usize = 12;

impl HolderKind {
    /// The kind of a holder, and its key's text.
    pub(in crate::engine) fn of(holder: &Holder) -> (Self, &str) {
        match holder {
            Holder::Fact(fact) => match fact {
                Fact::Feat(key) => (Self::Feat, key.as_str()),
                Fact::Feature(key) => (Self::Feature, key.as_str()),
                Fact::Option(key) => (Self::Option, key.as_str()),
                Fact::Selection(key) => (Self::Selection, key.as_str()),
                Fact::MythicAbility(key) => (Self::MythicAbility, key.as_str()),
                Fact::MythicFeat(key) => (Self::MythicFeat, key.as_str()),
                Fact::Class(key) => (Self::Class, key.as_str()),
                Fact::Archetype(key) => (Self::Archetype, key.as_str()),
                Fact::Race(key) => (Self::Race, key.as_str()),
                Fact::Deity(key) => (Self::Deity, key.as_str()),
            },
            Holder::Subrace(key) => (Self::Subrace, key.as_str()),
            Holder::Background(key) => (Self::Background, key.as_str()),
        }
    }
}

/// The components the walk evaluates while their entry is held, by the entry's kind and key, and what `NoSelectionIf`
/// names of each selection.
#[derive(Clone, Debug, Default)]
pub(in crate::engine) struct Components {
    held: Vec<FxHashMap<String, Vec<Component>>>,
    /// The facts whose holding takes a selection's pick away (`NoSelectionIf`'s `facts`).
    no_selection_if: FxHashMap<FeatureSelectionKey, BTreeSet<Fact>>,
    /// Whether any entry has a component the walk evaluates, so a walk of data without one skips them all.
    any: bool,
    /// Whether any prerequisite reads a class's own spellbook (`ClassSpellLevel`, `CasterType`), so the walk works
    /// those out only for data that reads them.
    reads_spellbooks: bool,
}

impl Components {
    /// Compiles the components that become the entries' own fields, and indexes the others.
    pub(super) fn compile(data: &mut GameData) -> Self {
        let archetype_classes: FxHashMap<ArchetypeKey, ClassKey> = data
            .archetypes
            .iter()
            .map(|archetype| (archetype.key.clone(), archetype.class.clone()))
            .collect();
        let class_of = |archetype: &ArchetypeKey| archetype_classes.get(archetype);
        let compile = |prerequisite: &mut Option<Prerequisite>, components: &[Component]| {
            if let Some(compiled) = compile_requires(components, class_of) {
                *prerequisite = Some(compiled);
            }
        };
        for feat in &mut data.feats {
            compile(&mut feat.prerequisite, &feat.components);
        }
        for option in &mut data.options {
            compile(&mut option.prerequisite, &option.components);
        }
        for ability in &mut data.mythic_abilities {
            compile(&mut ability.prerequisite, &ability.components);
        }
        for feat in &mut data.mythic_feats {
            compile(&mut feat.prerequisite, &feat.components);
        }
        for class in data.all_classes_mut() {
            compile(&mut class.prerequisite, &class.components);
        }
        for archetype in &mut data.archetypes {
            compile(&mut archetype.prerequisite, &archetype.components);
        }

        let mut index = Self {
            held: vec![FxHashMap::default(); HOLDER_KINDS],
            ..Self::default()
        };
        for selection in &mut data.selections {
            for component in &selection.components {
                if let Component::NoSelectionIf { facts, any_option } = component {
                    // `IsSelectionProhibited`'s `AnyFeatureFromSelection`: the planner's rule for a selection granted
                    // only to a character holding none of its options.
                    selection.only_without_option |= *any_option;
                    if !facts.is_empty() {
                        index
                            .no_selection_if
                            .entry(selection.key.clone())
                            .or_default()
                            .extend(facts.iter().cloned());
                    }
                }
            }
        }
        let mut add = |kind: HolderKind, key: &str, components: &[Component]| {
            let held: Vec<Component> = components
                .iter()
                .filter(|component| component.applies_while_held())
                .cloned()
                .collect();
            if !held.is_empty() {
                index.held[kind as usize].insert(key.to_owned(), held);
            }
        };
        for feat in &data.feats {
            add(HolderKind::Feat, feat.key.as_str(), &feat.components);
        }
        for feature in &data.features {
            add(
                HolderKind::Feature,
                feature.key.as_str(),
                &feature.components,
            );
        }
        for option in &data.options {
            add(HolderKind::Option, option.key.as_str(), &option.components);
        }
        for selection in &data.selections {
            add(
                HolderKind::Selection,
                selection.key.as_str(),
                &selection.components,
            );
        }
        for ability in &data.mythic_abilities {
            add(
                HolderKind::MythicAbility,
                ability.key.as_str(),
                &ability.components,
            );
        }
        for feat in &data.mythic_feats {
            add(HolderKind::MythicFeat, feat.key.as_str(), &feat.components);
        }
        for class in data.all_classes() {
            add(HolderKind::Class, class.key.as_str(), &class.components);
        }
        for archetype in &data.archetypes {
            add(
                HolderKind::Archetype,
                archetype.key.as_str(),
                &archetype.components,
            );
        }
        for race in &data.races {
            add(HolderKind::Race, race.key.as_str(), &race.components);
        }
        for deity in &data.deities {
            add(HolderKind::Deity, deity.key.as_str(), &deity.components);
        }
        for subrace in &data.subraces {
            add(
                HolderKind::Subrace,
                subrace.key.as_str(),
                &subrace.components,
            );
        }
        for background in &data.backgrounds {
            add(
                HolderKind::Background,
                background.key.as_str(),
                &background.components,
            );
        }
        index.any =
            index.held.iter().any(|table| !table.is_empty()) || !index.no_selection_if.is_empty();
        let prerequisites = (data.feats.iter().map(|feat| &feat.prerequisite))
            .chain(data.options.iter().map(|option| &option.prerequisite))
            .chain(
                data.mythic_abilities
                    .iter()
                    .map(|ability| &ability.prerequisite),
            )
            .chain(data.mythic_feats.iter().map(|feat| &feat.prerequisite))
            .chain(data.all_classes().map(|class| &class.prerequisite))
            .chain(
                data.archetypes
                    .iter()
                    .map(|archetype| &archetype.prerequisite),
            )
            .flatten();
        index.reads_spellbooks = prerequisites.into_iter().any(|prerequisite| {
            prerequisite.leaves().any(|leaf| {
                matches!(
                    leaf,
                    Prerequisite::ClassSpellLevel { .. } | Prerequisite::CasterType(_)
                )
            })
        });
        index
    }
}

/// The components (ticket 295).
impl Catalog {
    /// Whether any entry has components the walk evaluates: without, the walk skips them.
    pub(in crate::engine) fn has_components(&self) -> bool {
        self.components.any
    }

    /// Whether any prerequisite reads a class's own spellbook.
    pub(in crate::engine) fn reads_spellbooks(&self) -> bool {
        self.components.reads_spellbooks
    }

    /// The components an entry holds that the walk evaluates while it's held, by its kind and its key's text; empty
    /// for an entry with none.
    pub(in crate::engine) fn held_components(&self, kind: HolderKind, key: &str) -> &[Component] {
        (self.components.held.get(kind as usize))
            .and_then(|table| table.get(key))
            .map_or(&[], Vec::as_slice)
    }

    /// The facts whose holding takes a selection's pick away (`NoSelectionIf`); none for most.
    pub(in crate::engine) fn no_selection_if(
        &self,
        selection: &FeatureSelectionKey,
    ) -> Option<&BTreeSet<Fact>> {
        self.components.no_selection_if.get(selection)
    }

    /// The class an archetype belongs to.
    pub(in crate::engine) fn archetype_class(&self, archetype: &ArchetypeKey) -> Option<&ClassKey> {
        self.archetype(archetype).map(|archetype| &archetype.class)
    }
}
