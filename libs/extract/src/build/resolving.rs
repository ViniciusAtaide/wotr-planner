//! The postlude's work on the builder: placing each feature with its owner, queuing extra conditions, and resolving
//! the queued prerequisite lines once every name is known.

use std::collections::BTreeMap;

use rules::{Class, DataAt, GameData, MythicPath, Prerequisite};

use crate::grammar::{Cond, Unparsed};
use crate::report::Kind;
use crate::resolve::{Context, all};

use super::{Builder, FeatureOwner, Pending};

impl<'g> Builder<'g> {
    pub(super) fn place_features(
        &mut self,
        classes: &mut [Class],
        prestige: &mut [Class],
        paths: &mut [MythicPath],
    ) {
        for (owner, feature) in std::mem::take(&mut self.features).into_values() {
            let feature = feature.clone();
            match owner {
                FeatureOwner::Class(class) => {
                    if let Some(class) = classes
                        .iter_mut()
                        .chain(prestige.iter_mut())
                        .find(|c| c.key == class)
                    {
                        class.features.push(feature);
                    }
                }
                FeatureOwner::Archetype(class, archetype) => {
                    if let Some(archetype) = classes
                        .iter_mut()
                        .filter(|c| c.key == class)
                        .flat_map(|c| c.archetypes.iter_mut())
                        .find(|a| a.key == archetype)
                    {
                        archetype.features.push(feature);
                    }
                }
                FeatureOwner::Path(path) => {
                    if let Some(path) = paths.iter_mut().find(|p| p.key == path) {
                        path.features.push(feature);
                    }
                }
            }
        }
    }

    /// Queues extra conditions for an owner (prose rules).
    pub fn add_prerequisite(&mut self, owner: &DataAt, at: String, extra: Vec<Prerequisite>) {
        self.pending.push(Pending {
            owner: owner.clone(),
            at,
            conds: Vec::new(),
            context: Context::default(),
            kind: Kind::Prerequisite,
            extra,
        });
    }

    pub(super) fn resolve_prerequisites(&mut self, data: &mut GameData) {
        let mut resolved: BTreeMap<DataAt, Vec<Prerequisite>> = BTreeMap::new();
        let mut alternatives: BTreeMap<DataAt, Vec<Prerequisite>> = BTreeMap::new();
        for mut pending in std::mem::take(&mut self.pending) {
            let conditions = resolved.entry(pending.owner.clone()).or_default();
            conditions.extend(pending.extra);
            if !pending.conds.is_empty() {
                let lines: Vec<String> =
                    pending.conds.iter().map(|(line, _)| line.clone()).collect();
                let also = self.picks.alternatives_of(&pending.owner, &lines);
                if !also.is_empty() {
                    alternatives.insert(pending.owner.clone(), also);
                }
                if let Some(stated) = self.picks.prerequisite_of(&pending.owner, &lines) {
                    conditions.push(stated);
                    pending.conds.clear();
                }
            }
            for (line, cond) in pending.conds {
                match cond {
                    Err(Unparsed(_)) => {
                        self.report
                            .add(pending.kind, pending.at.clone(), line, "doesn't parse");
                    }
                    Ok(cond) => match self.catalog.resolve(&cond, &pending.context) {
                        Ok(prerequisite) => conditions.push(prerequisite),
                        Err(why) => {
                            // An ANDed group keeps the children that resolve.
                            if let Cond::All(children) = &cond {
                                for child in children {
                                    match self.catalog.resolve(child, &pending.context) {
                                        Ok(prerequisite) => conditions.push(prerequisite),
                                        Err(why) => self.report.add(
                                            pending.kind,
                                            pending.at.clone(),
                                            line.clone(),
                                            why,
                                        ),
                                    }
                                }
                            } else {
                                self.report.add(pending.kind, pending.at.clone(), line, why);
                            }
                        }
                    },
                }
            }
        }
        let picks = &self.picks;
        let mut take = |owner: DataAt| -> Option<Prerequisite> {
            let mut own = resolved.remove(&owner).and_then(|mut conditions| {
                conditions.sort();
                conditions.dedup();
                all(conditions)
            });
            // A line's alternatives join the thing or things it names.
            if let Some(also) = alternatives.remove(&owner) {
                own = Some(match own {
                    Some(Prerequisite::Any(mut children)) => {
                        children.extend(also);
                        Prerequisite::Any(children)
                    }
                    Some(one) => Prerequisite::Any(std::iter::once(one).chain(also).collect()),
                    None => panic!("{owner}: its line names nothing"),
                });
            }
            // An option's stated conditions (the totem rage powers' exclusions) join what its own lines name.
            let stated = match owner {
                DataAt::SelectionOption(_) => picks.stated_for(&owner),
                _ => Vec::new(),
            };
            if stated.is_empty() {
                return own;
            }
            Some(match own {
                None => Prerequisite::All(stated),
                Some(Prerequisite::All(mut children)) => {
                    children.extend(stated);
                    Prerequisite::All(children)
                }
                Some(other) => Prerequisite::All(std::iter::once(other).chain(stated).collect()),
            })
        };
        for feat in &mut data.feats {
            feat.prerequisite = take(DataAt::Feat(feat.key.clone()));
        }
        for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
            class.prerequisite = take(DataAt::Class(class.key.clone()));
            for selection in &mut class.selections {
                for option in &mut selection.options {
                    option.prerequisite = take(DataAt::SelectionOption(option.key.clone()));
                }
            }
            for archetype in &mut class.archetypes {
                archetype.prerequisite = take(DataAt::Archetype(archetype.key.clone()));
                for selection in &mut archetype.selections {
                    for option in &mut selection.options {
                        option.prerequisite = take(DataAt::SelectionOption(option.key.clone()));
                    }
                }
            }
        }
        for ability in &mut data.mythic_abilities {
            ability.prerequisite = take(DataAt::MythicAbility(ability.key.clone()));
        }
        for feat in &mut data.mythic_feats {
            feat.prerequisite = take(DataAt::MythicFeat(feat.key.clone()));
        }
        for path in &mut data.mythic_paths {
            for selection in &mut path.selections {
                for option in &mut selection.options {
                    option.prerequisite = take(DataAt::SelectionOption(option.key.clone()));
                }
            }
        }
        for owner in resolved.keys() {
            self.report.add_about(
                Kind::Prerequisite,
                owner,
                "",
                "prerequisites for something the data doesn't have",
            );
        }
    }
}
