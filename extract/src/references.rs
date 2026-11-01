//! The last pass: every key the data refers to must exist, or the seed's foreign keys fail. Dangling references
//! are dropped and reported.
//!
//! Keys carry no type prefix, so a reference must exist as its own kind: a feature called `trick` doesn't satisfy
//! a reference to a feat called `trick`.

use std::collections::BTreeSet;

use rules::{DataAt, FeatSlot, Feature, FeatureSelection, GameData, Prerequisite};

use crate::report::{Kind, Report};

/// Every defined key, with its kind.
#[derive(Default)]
struct Keys {
    all: BTreeSet<DataAt>,
}

impl Keys {
    fn has(&self, at: DataAt) -> bool {
        self.all.contains(&at)
    }

    /// Whether a leaf condition's references exist.
    fn leaf_ok(&self, prerequisite: &Prerequisite) -> bool {
        match prerequisite {
            Prerequisite::Feat(key) => self.has(DataAt::Feat(key.clone())),
            Prerequisite::MythicFeat(key) => self.has(DataAt::MythicFeat(key.clone())),
            Prerequisite::MythicAbility(key) => self.has(DataAt::MythicAbility(key.clone())),
            Prerequisite::ClassLevel { class, .. } => self.has(DataAt::Class(class.clone())),
            Prerequisite::Archetype(key) => self.has(DataAt::Archetype(key.clone())),
            Prerequisite::Feature { feature, .. } => self.has(DataAt::Feature(feature.clone())),
            Prerequisite::SelectionOption(key) => self.has(DataAt::SelectionOption(key.clone())),
            Prerequisite::SkillRanks { skill, .. } => self.has(DataAt::Skill(skill.clone())),
            Prerequisite::Race(key) => self.has(DataAt::Race(key.clone())),
            _ => true,
        }
    }

    /// The prerequisite without dangling references: an `All` drops the child, an `Any` or `AtLeast` with one is
    /// dropped whole.
    fn prune(&self, prerequisite: Prerequisite, dropped: &mut Vec<String>) -> Option<Prerequisite> {
        match prerequisite {
            Prerequisite::All(children) => {
                let kept: Vec<_> = children
                    .into_iter()
                    .filter_map(|child| self.prune(child, dropped))
                    .collect();
                match kept.len() {
                    0 => None,
                    1 => kept.into_iter().next(),
                    _ => Some(Prerequisite::All(kept)),
                }
            }
            Prerequisite::Any(children) | Prerequisite::AtLeast(_, children)
                if children.iter().any(|child| !self.deep_ok(child)) =>
            {
                dropped.push(format!("{:?}", Prerequisite::Any(children)));
                None
            }
            leaf if !self.leaf_ok(&leaf) => {
                dropped.push(format!("{leaf:?}"));
                None
            }
            other => Some(other),
        }
    }

    fn deep_ok(&self, prerequisite: &Prerequisite) -> bool {
        match prerequisite {
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                children.iter().all(|child| self.deep_ok(child))
            }
            leaf => self.leaf_ok(leaf),
        }
    }
}

fn keys(data: &GameData) -> Keys {
    let mut all = BTreeSet::new();
    all.extend(data.skills.iter().map(|s| DataAt::Skill(s.key.clone())));
    all.extend(data.races.iter().map(|r| DataAt::Race(r.key.clone())));
    all.extend(data.feats.iter().map(|f| DataAt::Feat(f.key.clone())));
    let mut features_and_selections = |features: &[Feature], selections: &[FeatureSelection]| {
        all.extend(features.iter().map(|f| DataAt::Feature(f.key.clone())));
        for selection in selections {
            all.insert(DataAt::FeatureSelection(selection.key.clone()));
            all.extend(
                selection
                    .options
                    .iter()
                    .map(|o| DataAt::SelectionOption(o.key.clone())),
            );
        }
    };
    for class in data.classes.iter().chain(&data.prestige_classes) {
        features_and_selections(&class.features, &class.selections);
        for archetype in &class.archetypes {
            features_and_selections(&archetype.features, &archetype.selections);
        }
    }
    for path in &data.mythic_paths {
        features_and_selections(&path.features, &path.selections);
    }
    for class in data.classes.iter().chain(&data.prestige_classes) {
        all.insert(DataAt::Class(class.key.clone()));
        all.extend(
            class
                .archetypes
                .iter()
                .map(|a| DataAt::Archetype(a.key.clone())),
        );
    }
    all.extend(
        data.mythic_abilities
            .iter()
            .map(|a| DataAt::MythicAbility(a.key.clone())),
    );
    all.extend(
        data.mythic_feats
            .iter()
            .map(|f| DataAt::MythicFeat(f.key.clone())),
    );
    all.extend(
        data.mythic_paths
            .iter()
            .map(|p| DataAt::MythicPath(p.key.clone())),
    );
    Keys { all }
}

fn prune_slots(keys: &Keys, slots: &mut Vec<FeatSlot>, at: &str, report: &mut Report) {
    slots.retain_mut(|slot| {
        if let FeatSlot::OneOf { feats, .. } = slot {
            feats.retain(|feat| keys.has(DataAt::Feat(feat.clone())));
            if feats.is_empty() {
                report.add(
                    Kind::Cell,
                    at,
                    "",
                    "a feat list with no known feats was dropped",
                );
                return false;
            }
        }
        true
    });
}

fn prune_prerequisite(
    keys: &Keys,
    prerequisite: &mut Option<Prerequisite>,
    at: &str,
    report: &mut Report,
) {
    if let Some(taken) = prerequisite.take() {
        let mut dropped = Vec::new();
        *prerequisite = keys.prune(taken, &mut dropped);
        for what in dropped {
            report.add(
                Kind::Prerequisite,
                at,
                "",
                format!("refers to something missing: {what}"),
            );
        }
    }
}

/// Drops every reference to a key the data doesn't define, reporting each.
pub fn drop_dangling(data: &mut GameData, report: &mut Report) {
    let keys = keys(data);
    for feat in &mut data.feats {
        prune_prerequisite(&keys, &mut feat.prerequisite, &feat.guide.0, report);
    }
    for ability in &mut data.mythic_abilities {
        prune_prerequisite(&keys, &mut ability.prerequisite, &ability.guide.0, report);
    }
    for feat in &mut data.mythic_feats {
        prune_prerequisite(&keys, &mut feat.prerequisite, &feat.guide.0, report);
    }
    for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        prune_prerequisite(&keys, &mut class.prerequisite, &class.guide.0, report);
        for row in &mut class.levels {
            row.features
                .retain(|key| keys.has(DataAt::Feature(key.clone())));
            row.selections
                .retain(|key| keys.has(DataAt::FeatureSelection(key.clone())));
            prune_slots(&keys, &mut row.feat_slots, &class.guide.0, report);
        }
        for selection in &mut class.selections {
            for option in &mut selection.options {
                prune_prerequisite(&keys, &mut option.prerequisite, &option.guide.0, report);
                option.feat = option
                    .feat
                    .take()
                    .filter(|feat| keys.has(DataAt::Feat(feat.clone())));
            }
        }
        for archetype in &mut class.archetypes {
            prune_prerequisite(
                &keys,
                &mut archetype.prerequisite,
                &archetype.guide.0,
                report,
            );
            for row in &mut archetype.levels {
                row.features
                    .retain(|key| keys.has(DataAt::Feature(key.clone())));
                row.selections
                    .retain(|key| keys.has(DataAt::FeatureSelection(key.clone())));
                prune_slots(&keys, &mut row.feat_slots, &archetype.guide.0, report);
            }
            for selection in &mut archetype.selections {
                for option in &mut selection.options {
                    prune_prerequisite(&keys, &mut option.prerequisite, &option.guide.0, report);
                    option.feat = option
                        .feat
                        .take()
                        .filter(|feat| keys.has(DataAt::Feat(feat.clone())));
                }
            }
        }
    }
    for path in &mut data.mythic_paths {
        for row in &mut path.ranks {
            row.features
                .retain(|key| keys.has(DataAt::Feature(key.clone())));
            row.selections
                .retain(|key| keys.has(DataAt::FeatureSelection(key.clone())));
            prune_slots(&keys, &mut row.feat_slots, &path.guide.0, report);
        }
        for selection in &mut path.selections {
            for option in &mut selection.options {
                prune_prerequisite(&keys, &mut option.prerequisite, &option.guide.0, report);
            }
        }
    }
}
