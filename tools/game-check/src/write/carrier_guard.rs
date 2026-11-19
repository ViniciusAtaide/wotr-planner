//! The prerequisites writer's carrier guard (ticket 308h): a tree that requires a fact some game blueprint grants
//! through a component, where the planner's entry of that blueprint doesn't grant the fact, refuses a character the
//! game allows (308d's Defensive Study and Stalwart Defender). The game's `PrerequisiteFeature` passes on `HasFact`
//! (`Blueprints/Classes/Prerequisites/PrerequisiteFeature.cs` `CheckInternal`, WrathDecompiled 6ce4a6b), however the
//! fact came.
//!
//! For each fact a tree requires (not under a `Not`), the guard climbs the game's grants ([`carriers::granters`]: the
//! vocabulary of 295, `AddFacts`, `AddFeatureOnClassLevel`, `AddFeatureIfHasFact`, `AddFeatureOnApply`, a progression's
//! level entries and an archetype's `AddFeatures`) up to each blueprint an entry maps, past those none does, with
//! `granted-by`'s walk ([`chains_up`]). Each entry of such a grantor that a character holds must give the fact (an
//! entry of its blueprint), or the entry is a gap. It gives it:
//! - in one of its own sites in the planner's grant index ([`GrantIndex`], ticket 334): its rows, its own fields (a
//!   deity's favoured weapons as the engine holds them: Irori's Improved Unarmed Strike; the features a switch to a
//!   path grants) and its grant components;
//! - or where the planner gives it: when no selection offers it as a pick and every site that gives it gives the fact
//!   too (the rogue's class row gives `rogue/rogue-proficiencies` and Armor Proficiency (Light Armor) at 1, where the
//!   game's `RogueProficiencies` gives the second);
//! - or the tree accepts that entry in the fact's place, in the same `Any`.
//!
//! A blueprint no entry maps ends no walk: what it grants is 307's carriers'. A class's own progression reaches no
//! class this way (a class names it, it doesn't grant it): its rows are the progression rows area's, as for the
//! carriers area.
//!
//! [`carriers::granters`]: super::carriers::granters

use std::collections::{BTreeMap, BTreeSet};

use rules::{BlueprintGuid, DataAt, Fact, GrantIndex, Prerequisite, Site};

use super::carriers::{fact_at, facts_of, granters};
use super::prerequisites::leaf_at;
use crate::areas::Ctx;
use crate::lookup::chains_up;

/// The grants the guard reads: the game's, upward (each blueprint's granters), and the planner's (its grant index:
/// each site, as the facts it gives together, and the selections that offer a fact as a pick).
pub(super) struct Grants {
    granters: BTreeMap<BlueprintGuid, Vec<BlueprintGuid>>,
    index: GrantIndex,
}

impl Grants {
    pub(super) fn of(ctx: &Ctx) -> Grants {
        Grants {
            granters: granters(ctx.dump),
            index: GrantIndex::new(ctx.planner.data),
        }
    }

    fn above(&self, guid: &BlueprintGuid) -> &[BlueprintGuid] {
        self.granters.get(guid).map_or(&[], Vec::as_slice)
    }

    /// Whether every holder of `by` holds one of `facts` in the planner: `by` gives one, or wherever the planner gives
    /// `by` it gives one too and no selection offers `by` as a pick.
    fn carry(&self, by: &DataAt, facts: &[Fact]) -> bool {
        let gives = |site: &Site| (site.grants.iter()).any(|grant| facts.contains(&grant.fact));
        if self.index.gives(by).iter().any(gives) {
            return true;
        }
        let Some(held) = fact_at(by).filter(|held| self.index.offered_in(held).is_empty()) else {
            return false;
        };
        let mut giving = self.index.given_by(&held).map(|(site, _)| site);
        giving.next().is_some_and(gives) && giving.all(gives)
    }
}

/// Each carrier gap of a tree `entry` holds or would hold, once each, in the tree's order:
/// `<fact> granted by <grantor> <GUID>, which <planner entry> doesn't grant -> 307`.
pub(super) fn gaps(
    ctx: &Ctx,
    grants: &Grants,
    entry: &DataAt,
    tree: Option<&Prerequisite>,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let Some(tree) = tree else {
        return out;
    };
    let mut required = Vec::new();
    walk(tree, &BTreeSet::new(), &mut required);
    for (fact, alternatives) in required {
        let Some(guid) = ctx.planner.guid(fact.clone()) else {
            continue;
        };
        let held = facts_of(ctx, guid);
        let mapped = |guid: &BlueprintGuid| ctx.planner.maps(guid);
        for chain in chains_up(guid, |at| grants.above(at), mapped) {
            let Some(grantor) = chain.last() else {
                continue;
            };
            for by in ctx.planner.entries_of(grantor) {
                if by == entry
                    || alternatives.contains(by)
                    || !held_by_a_character(by)
                    || grants.carry(by, &held)
                {
                    continue;
                }
                let gap = format!(
                    "{fact} granted by {} {grantor}, which {by} doesn't grant -> 307",
                    ctx.name(grantor)
                );
                if !out.contains(&gap) {
                    out.push(gap);
                }
            }
        }
    }
    out
}

/// The facts a tree requires, each with the facts its `Any`s accept in its place (an `AtLeast` of one is an `Any`).
/// A fact under a `Not` is refused, not required: no grant of it can refuse anyone.
fn walk(
    node: &Prerequisite,
    alternatives: &BTreeSet<DataAt>,
    out: &mut Vec<(DataAt, BTreeSet<DataAt>)>,
) {
    match node {
        Prerequisite::Any(children) | Prerequisite::AtLeast(0 | 1, children) => {
            let mut wider = alternatives.clone();
            wider.extend(children.iter().filter_map(leaf_at));
            for child in children {
                walk(child, &wider, out);
            }
        }
        Prerequisite::All(children) | Prerequisite::AtLeast(_, children) => {
            for child in children {
                walk(child, alternatives, out);
            }
        }
        Prerequisite::Not(_) => {}
        leaf => {
            if let Some(at) = leaf_at(leaf) {
                out.push((at, alternatives.clone()));
            }
        }
    }
}

/// Whether an entry is something a character holds, and so can carry a grant: not a weapon, spell, spell list or skill.
fn held_by_a_character(at: &DataAt) -> bool {
    !matches!(
        at,
        DataAt::Progression
            | DataAt::Skill(_)
            | DataAt::Weapon(_)
            | DataAt::Spell(_)
            | DataAt::SpellList(_)
    )
}
