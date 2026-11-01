//! The last pass: every key the data refers to must exist, or the seed's foreign keys fail. Dangling references
//! are dropped and reported.

use std::collections::BTreeSet;

use rules::{FeatSlot, GameData, Prerequisite};

use crate::report::{Kind, Report};

#[derive(Default)]
struct Keys {
    all: BTreeSet<String>,
}

impl Keys {
    fn has(&self, key: &str) -> bool {
        self.all.contains(key)
    }

    /// Whether a leaf condition's references exist.
    fn leaf_ok(&self, prerequisite: &Prerequisite) -> bool {
        match prerequisite {
            Prerequisite::Feat(key) => self.has(key.as_str()),
            Prerequisite::MythicFeat(key) => self.has(key.as_str()),
            Prerequisite::MythicAbility(key) => self.has(key.as_str()),
            Prerequisite::ClassLevel { class, .. } => self.has(class.as_str()),
            Prerequisite::Archetype(key) => self.has(key.as_str()),
            Prerequisite::Feature { feature, .. } => self.has(feature.as_str()),
            Prerequisite::SelectionOption(key) => self.has(key.as_str()),
            Prerequisite::SkillRanks { skill, .. } => self.has(skill.as_str()),
            Prerequisite::Race(key) => self.has(key.as_str()),
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
    let mut keys = Keys::default();
    let mut add = |key: &str| {
        keys.all.insert(key.to_owned());
    };
    for skill in &data.skills {
        add(skill.key.as_str());
    }
    for race in &data.races {
        add(race.key.as_str());
    }
    for feat in &data.feats {
        add(feat.key.as_str());
    }
    for class in data.classes.iter().chain(&data.prestige_classes) {
        add(class.key.as_str());
        for feature in &class.features {
            add(feature.key.as_str());
        }
        for selection in &class.selections {
            add(selection.key.as_str());
            for option in &selection.options {
                add(option.key.as_str());
            }
        }
        for archetype in &class.archetypes {
            add(archetype.key.as_str());
            for feature in &archetype.features {
                add(feature.key.as_str());
            }
            for selection in &archetype.selections {
                add(selection.key.as_str());
                for option in &selection.options {
                    add(option.key.as_str());
                }
            }
        }
    }
    for ability in &data.mythic_abilities {
        add(ability.key.as_str());
    }
    for feat in &data.mythic_feats {
        add(feat.key.as_str());
    }
    for path in &data.mythic_paths {
        add(path.key.as_str());
        for feature in &path.features {
            add(feature.key.as_str());
        }
        for selection in &path.selections {
            add(selection.key.as_str());
            for option in &selection.options {
                add(option.key.as_str());
            }
        }
    }
    keys
}

fn prune_slots(keys: &Keys, slots: &mut Vec<FeatSlot>, at: &str, report: &mut Report) {
    slots.retain_mut(|slot| {
        if let FeatSlot::OneOf { feats, .. } = slot {
            feats.retain(|feat| keys.has(feat.as_str()));
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
            row.features.retain(|key| keys.has(key.as_str()));
            row.selections.retain(|key| keys.has(key.as_str()));
            prune_slots(&keys, &mut row.feat_slots, &class.guide.0, report);
        }
        for selection in &mut class.selections {
            for option in &mut selection.options {
                prune_prerequisite(&keys, &mut option.prerequisite, &option.guide.0, report);
                option.feat = option.feat.take().filter(|feat| keys.has(feat.as_str()));
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
                row.features.retain(|key| keys.has(key.as_str()));
                row.selections.retain(|key| keys.has(key.as_str()));
                prune_slots(&keys, &mut row.feat_slots, &archetype.guide.0, report);
            }
            for selection in &mut archetype.selections {
                for option in &mut selection.options {
                    prune_prerequisite(&keys, &mut option.prerequisite, &option.guide.0, report);
                    option.feat = option.feat.take().filter(|feat| keys.has(feat.as_str()));
                }
            }
        }
    }
    for path in &mut data.mythic_paths {
        for row in &mut path.ranks {
            row.features.retain(|key| keys.has(key.as_str()));
            row.selections.retain(|key| keys.has(key.as_str()));
            prune_slots(&keys, &mut row.feat_slots, &path.guide.0, report);
        }
        for selection in &mut path.selections {
            for option in &mut selection.options {
                prune_prerequisite(&keys, &mut option.prerequisite, &option.guide.0, report);
            }
        }
    }
}
