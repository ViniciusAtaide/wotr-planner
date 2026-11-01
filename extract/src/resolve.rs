//! Resolving the names in parsed prerequisite lines to the keys of the extracted data.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArchetypeKey, ClassKey, FeatKey, FeatureKey, MythicAbilityKey, MythicFeatKey, Prerequisite,
    RaceKey, SelectionOptionKey, SkillKey,
};

use crate::grammar::Cond;
use crate::names::norm;

/// Every name a prerequisite can refer to, by its [`norm`]al form.
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    pub skills: BTreeMap<String, SkillKey>,
    pub races: BTreeMap<String, RaceKey>,
    pub classes: BTreeMap<String, ClassKey>,
    pub archetypes: BTreeMap<String, BTreeSet<ArchetypeKey>>,
    pub feats: BTreeMap<String, FeatKey>,
    pub features: BTreeMap<String, BTreeSet<FeatureKey>>,
    pub options: BTreeMap<String, BTreeSet<SelectionOptionKey>>,
    /// A Feature selection's name to its options: naming the selection means having one of its options.
    pub selections: BTreeMap<String, BTreeSet<SelectionOptionKey>>,
    pub mythic_feats: BTreeMap<String, MythicFeatKey>,
    pub mythic_abilities: BTreeMap<String, MythicAbilityKey>,
    /// The Character level at which each Mythic rank is reached.
    pub rank_levels: BTreeMap<u8, u8>,
}

/// Where a line sits, for the conditions that depend on it.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// The class whose page the line is on: "Level 10" is a Class level in it.
    pub own_class: Option<ClassKey>,
}

/// Misspellings in the guide, as [`norm`]al forms, and what they mean.
const TYPOS: [(&str, &str); 5] = [
    ("dicovery", "discovery"),
    ("knowlege", "knowledge"),
    ("spell-reistance", "spell-resistance"),
    ("fearl-mutagen", "feral-mutagen"),
    ("judgement", "judgment"),
];

fn fix_typos(name: &str) -> String {
    let mut name = norm(name);
    for (typo, fixed) in TYPOS {
        if name.contains(typo) {
            name = name.replace(typo, fixed);
        }
    }
    name
}

/// Names written as `Knowledge: Arcana` in some places and `Knowledge (Arcana)` in others share one form.
fn skill_form(name: &str) -> String {
    fix_typos(&name.replace(':', " "))
}

impl Catalog {
    /// Resolves a parsed condition, or says why it can't be.
    pub fn resolve(&self, cond: &Cond, context: &Context) -> Result<Prerequisite, String> {
        Ok(match cond {
            Cond::AbilityScore(ability, score) => Prerequisite::AbilityScore {
                ability: *ability,
                score: *score,
            },
            Cond::Bab(bab) => Prerequisite::BaseAttackBonus(*bab),
            Cond::CharacterLevel(level) => Prerequisite::CharacterLevel(*level),
            Cond::SpellLevel(spell_type, level) => Prerequisite::SpellLevel {
                spell_type: *spell_type,
                level: *level,
            },
            Cond::Alignment(alignments) => any(alignments
                .iter()
                .map(|alignment| Prerequisite::Alignment(*alignment))
                .collect()),
            Cond::OwnClassLevel(level) => Prerequisite::ClassLevel {
                class: context
                    .own_class
                    .clone()
                    .ok_or("a bare level outside a class page")?,
                level: *level,
            },
            Cond::MythicRank(rank) => Prerequisite::CharacterLevel(
                *self
                    .rank_levels
                    .get(rank)
                    .ok_or_else(|| format!("no Mythic rank {rank}"))?,
            ),
            Cond::ClassLevel(name, level) => Prerequisite::ClassLevel {
                class: self
                    .classes
                    .get(&fix_typos(name))
                    .cloned()
                    .ok_or_else(|| format!("no class named {name:?}"))?,
                level: *level,
            },
            Cond::Named { name, count } => self.named(name, *count)?,
            Cond::All(children) => Prerequisite::All(
                children
                    .iter()
                    .map(|child| self.resolve(child, context))
                    .collect::<Result<_, _>>()?,
            ),
            Cond::Any(children) => any(children
                .iter()
                .map(|child| self.resolve(child, context))
                .collect::<Result<_, _>>()?),
            Cond::AtLeast(n, children) => Prerequisite::AtLeast(
                *n,
                children
                    .iter()
                    .map(|child| self.resolve(child, context))
                    .collect::<Result<_, _>>()?,
            ),
        })
    }

    fn named(&self, name: &str, count: Option<u8>) -> Result<Prerequisite, String> {
        if let Some(count) = count {
            if let Some(skill) = self.skills.get(&skill_form(name)) {
                return Ok(Prerequisite::SkillRanks {
                    skill: skill.clone(),
                    ranks: count,
                });
            }
            let features = self.features_named(name);
            if features.is_empty() {
                return Err(format!("no skill or class feature named {name:?}"));
            }
            return Ok(any(features
                .into_iter()
                .map(|feature| Prerequisite::Feature { feature, count })
                .collect()));
        }
        let mut found = self.everything_named(&fix_typos(name));
        if found.is_empty() {
            // "Weapon Focus (Dueling Sword)", "Skill Focus - Mobility": the feat without its parameter.
            let base = name
                .split_once(" (")
                .map(|(base, _)| base)
                .or_else(|| name.split_once(" - ").map(|(base, _)| base));
            if let Some(base) = base {
                found = self.everything_named(&fix_typos(base));
                found.retain(|found| matches!(found, Prerequisite::Feat(_)));
            }
        }
        if found.is_empty() {
            return Err(format!("nothing named {name:?}"));
        }
        Ok(any(found))
    }

    fn features_named(&self, name: &str) -> Vec<FeatureKey> {
        self.features
            .get(&fix_typos(name))
            .map(|keys| keys.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Everything a bare name can mean: a feat, a class feature, a selection option or a whole selection, a race,
    /// a class, an archetype, a mythic feat or ability.
    fn everything_named(&self, name: &str) -> Vec<Prerequisite> {
        let mut found = Vec::new();
        if let Some(feat) = self.feats.get(name) {
            found.push(Prerequisite::Feat(feat.clone()));
        }
        for feature in self.features.get(name).into_iter().flatten() {
            found.push(Prerequisite::Feature {
                feature: feature.clone(),
                count: 1,
            });
        }
        for option in self.options.get(name).into_iter().flatten() {
            found.push(Prerequisite::SelectionOption(option.clone()));
        }
        for option in self.selections.get(name).into_iter().flatten() {
            found.push(Prerequisite::SelectionOption(option.clone()));
        }
        if let Some(race) = self.races.get(name) {
            found.push(Prerequisite::Race(race.clone()));
        }
        if let Some(class) = self.classes.get(name) {
            found.push(Prerequisite::ClassLevel {
                class: class.clone(),
                level: 1,
            });
        }
        for archetype in self.archetypes.get(name).into_iter().flatten() {
            found.push(Prerequisite::Archetype(archetype.clone()));
        }
        if let Some(feat) = self.mythic_feats.get(name) {
            found.push(Prerequisite::MythicFeat(feat.clone()));
        }
        if let Some(ability) = self.mythic_abilities.get(name) {
            found.push(Prerequisite::MythicAbility(ability.clone()));
        }
        if found.is_empty() {
            // "Sacred Incense" for the option "Incense Fog - Sacred Incense".
            let suffix = format!("-{name}");
            for (option_name, options) in &self.options {
                if option_name.ends_with(&suffix) {
                    found.extend(options.iter().cloned().map(Prerequisite::SelectionOption));
                }
            }
        }
        found.sort();
        found.dedup();
        found
    }
}

/// One condition stays itself; several are an `Any`.
pub fn any(mut conditions: Vec<Prerequisite>) -> Prerequisite {
    conditions.sort();
    conditions.dedup();
    if conditions.len() == 1 {
        conditions.pop().expect("one")
    } else {
        Prerequisite::Any(conditions)
    }
}

/// Several ANDed conditions as one prerequisite: none is `None`, one stays itself.
pub fn all(mut conditions: Vec<Prerequisite>) -> Option<Prerequisite> {
    match conditions.len() {
        0 => None,
        1 => conditions.pop(),
        _ => Some(Prerequisite::All(conditions)),
    }
}
