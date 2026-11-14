//! A failure before it's placed, what it would have given, and the tree: each failure under the earliest earlier
//! failure that one of its unmet prerequisites names (the feat, option or mythic pick it picks, an option's chosen and
//! granted feats and the features of its rows, the class levels and features of a refused class, a refused
//! archetype, a score the declaration couldn't set), or that it can't be reached without (a slot of a refused class's
//! level, of a refused path's rank, of a refused mythic pick; a level past a missing one). No prerequisite names a
//! path, so a refused path holds only its rank's unreached slots.

use std::fmt;

use rules::{
    Ability, Address, ArchetypeKey, ClassKey, Engine, FeatKey, GameData, Issue, MythicAbilityKey,
    MythicFeatKey, Parameter, Pick, Prerequisite, SelectionOptionKey, Unmet,
};

use super::Refused;
use super::wording::Wording;
use crate::options::{issue_code, issue_reason};
use crate::planner::Planner;
use crate::shapes::FailureLine;

/// Where a failure is: text given as an address that isn't one, or an address declared. They order in that order, an
/// address in Build order.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum At {
    Unparsed(String),
    Address(Address),
}

impl At {
    fn address(&self) -> Option<&Address> {
        match self {
            At::Address(address) => Some(address),
            At::Unparsed(_) => None,
        }
    }
}

impl fmt::Display for At {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            At::Address(address) => address.fmt(f),
            At::Unparsed(text) => f.write_str(text),
        }
    }
}

/// Why a choice fails: the walk's issue, or the declaration's refusal.
#[derive(Clone, Debug)]
pub(super) enum Why {
    Walk(Issue),
    Refused(Refused),
}

impl Why {
    fn code(&self) -> &'static str {
        match self {
            Why::Walk(issue) => issue_code(issue),
            Why::Refused(refused) => refused.code(),
        }
    }

    /// The unmet prerequisite, for a walk's `unmet`: what its `needs` say and what links it under a cause.
    fn unmet(&self) -> Option<&Unmet> {
        match self {
            Why::Walk(Issue::Unmet(unmet)) => Some(unmet),
            _ => None,
        }
    }
}

/// A failure: one the declaration refused, or one of the walk's.
pub(super) struct Node {
    pub(super) at: At,
    pub(super) label: String,
    pub(super) why: Why,
    pub(super) reason: String,
    /// What it would have given, for the failures that need it.
    pub(super) gives: Vec<Given>,
    /// The failure it can't be reached without, by address: the class of its slot's level, the path choice of its
    /// slot's rank, the mythic pick that gives its slot, the level missing below its own.
    pub(super) under: Option<Address>,
    /// For an unmet prerequisite, each part not met, in short ([`needs`]).
    pub(super) needs: Vec<String>,
}

impl Node {
    /// A failure the declaration finds at an address. It has no label until the Build's sheet is there to name its
    /// slot ([`labelled`](Self::labelled)), unless the declaration gives it one.
    pub(super) fn refused(address: Address, refused: Refused, reason: String) -> Self {
        Self {
            label: String::new(),
            at: At::Address(address),
            why: Why::Refused(refused),
            reason,
            gives: Vec::new(),
            under: None,
            needs: Vec::new(),
        }
    }

    /// Text given as an address that isn't one.
    pub(super) fn unparsed(text: String, reason: String) -> Self {
        Self {
            at: At::Unparsed(text),
            label: "Not a choice".to_owned(),
            why: Why::Refused(Refused::NoSuchChoice),
            reason,
            gives: Vec::new(),
            under: None,
            needs: Vec::new(),
        }
    }

    /// The failure labelled at its address as the walk's failures are ([`Wording::label`]), so a refused slot is
    /// "Level 3: Feat" like a failed one. A label the declaration gave (the point buy's, text that isn't an address)
    /// stays.
    pub(super) fn labelled(mut self, wording: &Wording) -> Self {
        if let (true, At::Address(address)) = (self.label.is_empty(), &self.at) {
            self.label = wording.label(address);
        }
        self
    }

    /// The failure with what it would have given, so the failures that need that sit under it.
    pub(super) fn giving(mut self, gives: Vec<Given>) -> Self {
        self.gives = gives;
        self
    }

    /// The failure with the one it can't be reached without, by address, so it sits under that one.
    pub(super) fn under(mut self, address: Address) -> Self {
        self.under = Some(address);
        self
    }

    /// Whether `cause`, an earlier failure, is one this failure needs: the one it can't be reached without, or one
    /// that would have given what an unmet prerequisite of it names.
    fn needs(&self, data: &GameData, cause: &Node) -> bool {
        if self.under.is_some() && self.under.as_ref() == cause.at.address() {
            return true;
        }
        let Some(unmet) = self.why.unmet() else {
            return false;
        };
        leaves(unmet).iter().any(|leaf| {
            cause
                .gives
                .iter()
                .any(|given| given.is_named_by(data, leaf))
        })
    }
}

/// What a failed choice would have given, which a later prerequisite may name.
#[derive(Clone, Debug)]
pub(super) enum Given {
    /// A feat, for a value or for none.
    Feat(FeatKey, Option<Parameter>),
    SelectionOption(SelectionOptionKey),
    MythicAbility(MythicAbilityKey),
    MythicFeat(MythicFeatKey),
    /// A class's levels (and the features it defines).
    Class(ClassKey),
    Archetype(ArchetypeKey),
    /// An ability score the declaration couldn't set (a point-buy score, an ability increase).
    Score(Ability),
}

impl Given {
    /// Whether a prerequisite's single condition names this.
    fn is_named_by(&self, data: &GameData, leaf: &Prerequisite) -> bool {
        match (self, leaf) {
            (
                Given::Feat(feat, _),
                Prerequisite::Feat(needed)
                | Prerequisite::FeatForSameValue(needed)
                | Prerequisite::FeatIfValue { feat: needed, .. },
            ) => feat == needed,
            (
                Given::Feat(feat, value),
                Prerequisite::FeatFor {
                    feat: needed,
                    value: needs,
                },
            ) => feat == needed && value.as_ref().is_none_or(|value| value == needs),
            (Given::SelectionOption(option), Prerequisite::SelectionOption(needed)) => {
                option == needed
            }
            (Given::SelectionOption(option), Prerequisite::Feature { feature, .. }) => {
                data.selection_option(option).is_some_and(|option| {
                    option
                        .levels
                        .iter()
                        .any(|row| row.features.contains(feature))
                })
            }
            (Given::MythicAbility(ability), Prerequisite::MythicAbility(needed)) => {
                ability == needed
            }
            (Given::MythicFeat(feat), Prerequisite::MythicFeat(needed)) => feat == needed,
            (Given::Class(class), Prerequisite::ClassLevel { class: needed, .. }) => {
                class == needed
            }
            (Given::Class(class), Prerequisite::Feature { feature, .. }) => {
                data.class(class).is_some_and(|class| {
                    class.features.iter().any(|f| &f.key == feature)
                        || class
                            .archetypes
                            .iter()
                            .any(|archetype| archetype.features.iter().any(|f| &f.key == feature))
                })
            }
            (Given::Archetype(archetype), Prerequisite::Archetype(needed)) => archetype == needed,
            (
                Given::Score(ability),
                Prerequisite::AbilityScore {
                    ability: needed, ..
                },
            ) => ability == needed,
            _ => false,
        }
    }
}

/// What a pick would have given: its item, and an option's chosen and granted feats.
pub(super) fn pick_gives(data: &GameData, pick: &Pick) -> Vec<Given> {
    match pick {
        Pick::Feat { feat, parameter } => vec![Given::Feat(feat.clone(), parameter.clone())],
        Pick::SelectionOption { option, parameter } => {
            let mut gives = vec![Given::SelectionOption(option.clone())];
            if let Some(option) = data.selection_option(option) {
                gives.extend(
                    option
                        .chosen_feat
                        .iter()
                        .map(|feat| Given::Feat(feat.clone(), parameter.clone())),
                );
                gives.extend(
                    option
                        .feats
                        .iter()
                        .chain(option.levels.iter().flat_map(|row| &row.feats))
                        .map(|feat| Given::Feat(feat.clone(), None)),
                );
            }
            gives
        }
        Pick::MythicAbility { ability, .. } => vec![Given::MythicAbility(ability.clone())],
        Pick::MythicFeat { feat, .. } => vec![Given::MythicFeat(feat.clone())],
    }
}

/// The single conditions an unmet prerequisite fails on, left to right.
fn leaves(unmet: &Unmet) -> Vec<&Prerequisite> {
    match unmet {
        Unmet::Condition { needs, .. } => vec![needs],
        Unmet::All(children) | Unmet::Any(children) => children.iter().flat_map(leaves).collect(),
        Unmet::AtLeast { failed, .. } => failed.iter().flat_map(leaves).collect(),
        Unmet::MetLater { unmet, .. } => leaves(unmet),
    }
}

/// Each part of an unmet "and" on its own, in the engine's words without "Needs ": "Str 13 (have 12)",
/// "power-attack". An "or" stays one part.
pub(super) fn needs(engine: &Engine, unmet: &Unmet) -> Vec<String> {
    let parts: Vec<&Unmet> = match unmet {
        Unmet::All(children) => children.iter().collect(),
        Unmet::MetLater { unmet, .. } => return needs(engine, unmet),
        unmet => vec![unmet],
    };
    parts
        .into_iter()
        .map(|part| {
            let text = issue_reason(engine, &Issue::Unmet(part.clone()));
            text.strip_prefix("Needs ")
                .map(str::to_owned)
                .unwrap_or(text)
        })
        .collect()
}

/// The failures placed in a tree, in order: each under the earliest earlier failure it needs, with the others it
/// needs as `also_needs`; the rest are roots.
pub(super) fn tree(planner: &Planner, nodes: Vec<Node>) -> Vec<FailureLine> {
    let data = &planner.data;
    let mut parents: Vec<Option<usize>> = Vec::with_capacity(nodes.len());
    let mut also: Vec<Vec<String>> = Vec::with_capacity(nodes.len());
    for (index, node) in nodes.iter().enumerate() {
        let causes: Vec<usize> = (0..index)
            .filter(|&earlier| node.needs(data, &nodes[earlier]))
            .collect();
        parents.push(causes.first().copied());
        let mut others: Vec<String> = Vec::new();
        for cause in causes.iter().skip(1) {
            let choice = nodes[*cause].at.to_string();
            if !others.contains(&choice) {
                others.push(choice);
            }
        }
        also.push(others);
    }
    let mut lines: Vec<Option<FailureLine>> = nodes
        .into_iter()
        .zip(also)
        .map(|(node, also_needs)| {
            let reason = Some(node.reason).filter(|reason| !echoes(reason, &node.needs));
            Some(FailureLine {
                choice: node.at.to_string(),
                label: node.label,
                issue: node.why.code().to_owned(),
                needs: node.needs,
                reason,
                also_needs,
                children: Vec::new(),
            })
        })
        .collect();
    // Children come after their parent, so placing from the last one up fills each before it's placed.
    for index in (0..lines.len()).rev() {
        if let Some(parent) = parents[index] {
            let line = lines[index].take().expect("each failure is placed once");
            lines[parent]
                .as_mut()
                .expect("a parent is placed after its children")
                .children
                .insert(0, line);
        }
    }
    lines.into_iter().flatten().collect()
}

/// Whether a reason says only "Needs " and the `needs`, joined by ", ", so a failure line can leave it out. A reason
/// that says more (a pick held that refuses it, a later slot to swap with) is kept.
fn echoes(reason: &str, needs: &[String]) -> bool {
    !needs.is_empty() && reason.strip_prefix("Needs ") == Some(needs.join(", ").as_str())
}

#[cfg(test)]
mod tests {
    use super::echoes;

    fn needs(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    #[test]
    fn a_reason_that_only_repeats_its_needs_is_left_out() {
        assert!(echoes("Needs Power Attack", &needs(&["Power Attack"])));
        assert!(echoes(
            "Needs Weapon Focus in the same weapon, Fighter level 4 (have 0)",
            &needs(&[
                "Weapon Focus in the same weapon",
                "Fighter level 4 (have 0)"
            ]),
        ));
    }

    #[test]
    fn a_reason_that_says_more_than_its_needs_is_kept() {
        // A pick held that refuses it.
        assert!(!echoes(
            "Has Two-Handed Fighter; needs Fighter level 4 (have 2)",
            &needs(&["Fighter level 4 (have 2)"]),
        ));
        // A later slot holding what it needs.
        assert!(!echoes(
            "Needs Power Attack: Power Attack is in level 1's Feat slot, which comes after this slot in the level's \
             pick order; swap them",
            &needs(&["Power Attack"]),
        ));
        // No `needs` at all: the reason is all there is.
        assert!(!echoes("Needs ", &[]));
    }
}
