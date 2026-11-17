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
//! - in its own rows or grant components ([`sites`]);
//! - or where the planner gives it: when no selection offers it as a pick and every row or grant component that gives
//!   it gives the fact too (the rogue's class row gives `rogue/rogue-proficiencies` and Armor Proficiency (Light Armor)
//!   at 1, where the game's `RogueProficiencies` gives the second);
//! - or, a deity, as a follower's favoured weapon, as the engine holds it (`libs/rules/src/engine/catalog.rs` `favoured_weapon_feats`: Irori's
//!   Improved Unarmed Strike);
//! - or the tree accepts that entry in the fact's place, in the same `Any`.
//!
//! A blueprint no entry maps ends no walk: what it grants is 307's carriers'. A class's own progression reaches no
//! class this way (a class names it, it doesn't grant it): its rows are the progression rows area's, as for the
//! carriers area.
//!
//! [`carriers::granters`]: super::carriers::granters

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    BlueprintGuid, Component, DataAt, EXOTIC_WEAPON_PROFICIENCY, Fact, FeatKey, FeatValue,
    FeatureKey, FeatureSelectionKey, GameData, IMPROVED_UNARMED_STRIKE, Prerequisite,
    UNARMED_STRIKE, WeaponProficiency,
};

use super::carriers::{fact_at, facts_of, granters, named};
use super::prerequisites::leaf_at;
use crate::areas::Ctx;
use crate::lookup::chains_up;

/// The grants the guard reads: the game's, upward (each blueprint's granters), and the planner's (each row or grant
/// component, as the facts it gives together, and the facts a selection offers as picks).
pub(super) struct Grants {
    granters: BTreeMap<BlueprintGuid, Vec<BlueprintGuid>>,
    sites: Vec<BTreeSet<Fact>>,
    picks: BTreeSet<Fact>,
}

impl Grants {
    pub(super) fn of(ctx: &Ctx) -> Grants {
        let data = ctx.planner.data;
        let sites = (ctx.planner.entries.iter())
            .flat_map(|(at, _)| sites(data, at))
            .collect();
        let mut picks = BTreeSet::new();
        for selection in &data.selections {
            picks.extend(
                (selection.options.iter())
                    .chain(&selection.shared_options)
                    .cloned()
                    .map(Fact::Option),
            );
            picks.extend(selection.feats.iter().cloned().map(Fact::Feat));
            picks.extend(
                (selection.mythic_abilities.iter())
                    .cloned()
                    .map(Fact::MythicAbility),
            );
        }
        Grants {
            granters: granters(ctx.dump),
            sites,
            picks,
        }
    }

    fn above(&self, guid: &BlueprintGuid) -> &[BlueprintGuid] {
        self.granters.get(guid).map_or(&[], Vec::as_slice)
    }

    /// Whether every holder of `by` holds one of `facts` in the planner: `by` gives one, or wherever the planner gives
    /// `by` it gives one too and no selection offers `by` as a pick.
    fn carry(&self, data: &GameData, by: &DataAt, facts: &[Fact]) -> bool {
        let gives = |site: &BTreeSet<Fact>| facts.iter().any(|fact| site.contains(fact));
        if sites(data, by).iter().any(gives) {
            return true;
        }
        let Some(held) = fact_at(by).filter(|held| !self.picks.contains(held)) else {
            return false;
        };
        let mut giving = self.sites.iter().filter(|site| site.contains(&held));
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
                    || grants.carry(ctx.planner.data, by, &held)
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

/// What a planner entry gives, as the facts each of its sites gives together: each of its rows (a class's or
/// archetype's level rows, an option's own feats and its rows, a mythic ability's features and selection, a path's
/// ranks, a background's feats, a deity's favoured weapons' feats) and each grant component.
fn sites(data: &GameData, at: &DataAt) -> Vec<BTreeSet<Fact>> {
    fn row<'a>(
        features: &[FeatureKey],
        feats: impl Iterator<Item = &'a FeatKey>,
        selections: &[FeatureSelectionKey],
    ) -> BTreeSet<Fact> {
        (features.iter().cloned().map(Fact::Feature))
            .chain(feats.cloned().map(Fact::Feat))
            .chain(selections.iter().cloned().map(Fact::Selection))
            .collect()
    }
    fn valued(values: &BTreeSet<FeatValue>) -> impl Iterator<Item = &FeatKey> {
        values.iter().map(|value| &value.feat)
    }
    let mut out: Vec<BTreeSet<Fact>> = Vec::new();
    let components: &[Component] = match at {
        DataAt::Race(key) => data
            .races
            .iter()
            .find(|e| e.key == *key)
            .map_or(&[], |e| &e.components),
        DataAt::Subrace(key) => data
            .subraces
            .iter()
            .find(|e| e.key == *key)
            .map_or(&[], |e| &e.components),
        DataAt::Feat(key) => data
            .feats
            .iter()
            .find(|e| e.key == *key)
            .map_or(&[], |e| &e.components),
        DataAt::Class(key) => match data.class(key) {
            Some(class) => {
                out.extend(class.levels.iter().map(|level| {
                    row(
                        &level.features,
                        level.feats.iter().chain(valued(&level.feats_for)),
                        &level.selections,
                    )
                }));
                &class.components
            }
            None => &[],
        },
        DataAt::Archetype(key) => match data.archetypes.iter().find(|e| e.key == *key) {
            Some(archetype) => {
                out.extend(archetype.levels.iter().map(|level| {
                    row(
                        &level.features,
                        level.feats.iter().chain(valued(&level.feats_for)),
                        &level.selections,
                    )
                }));
                &archetype.components
            }
            None => &[],
        },
        DataAt::Feature(key) => data.feature(key).map_or(&[], |e| &e.components),
        DataAt::FeatureSelection(key) => data.selection(key).map_or(&[], |e| &e.components),
        DataAt::SelectionOption(key) => match data.options.iter().find(|e| e.key == *key) {
            Some(option) => {
                out.push(row(
                    &[],
                    option.feats.iter().chain(&option.chosen_feat),
                    &[],
                ));
                out.extend(option.levels.iter().map(|level| {
                    row(
                        &level.features,
                        level.feats.iter().chain(&level.chosen_feats),
                        &level.selections,
                    )
                }));
                &option.components
            }
            None => &[],
        },
        DataAt::MythicAbility(key) => match data.mythic_abilities.iter().find(|e| e.key == *key) {
            Some(ability) => {
                out.push(row(
                    &ability.features,
                    std::iter::empty(),
                    ability.selection.as_slice(),
                ));
                &ability.components
            }
            None => &[],
        },
        DataAt::MythicFeat(key) => data
            .mythic_feats
            .iter()
            .find(|e| e.key == *key)
            .map_or(&[], |e| &e.components),
        DataAt::MythicPath(key) => {
            if let Some(path) = data.mythic_paths.iter().find(|e| e.key == *key) {
                out.extend(
                    (path.ranks.iter())
                        .map(|rank| row(&rank.features, std::iter::empty(), &rank.selections)),
                );
            }
            &[]
        }
        DataAt::Background(key) => match data.backgrounds.iter().find(|e| e.key == *key) {
            Some(background) => {
                out.push(row(
                    &[],
                    background.feats.iter().chain(valued(&background.feats_for)),
                    &[],
                ));
                &background.components
            }
            None => &[],
        },
        DataAt::Deity(key) => match data.deities.iter().find(|e| e.key == *key) {
            Some(deity) => {
                out.push(favoured_weapon_feats(data, &deity.favoured_weapons));
                &deity.components
            }
            None => &[],
        },
        DataAt::Gear(key) => data
            .gear
            .iter()
            .find(|e| e.key == *key)
            .map_or(&[], |e| &e.components),
        DataAt::Progression
        | DataAt::Skill(_)
        | DataAt::Weapon(_)
        | DataAt::Spell(_)
        | DataAt::SpellList(_) => &[],
    };
    for part in components {
        let mut site: BTreeSet<Fact> = named(part).into_iter().cloned().collect();
        if let Component::GrantWithValue { feat, .. } = part {
            site.insert(Fact::Feat(feat.clone()));
        }
        out.push(site);
    }
    out.retain(|site| !site.is_empty());
    out
}

/// The feats a follower holds for a deity's favoured weapons, as the engine's `favoured_weapon_feats` holds them:
/// Improved Unarmed Strike for the unarmed strike, Exotic Weapon Proficiency for an exotic weapon.
fn favoured_weapon_feats(data: &GameData, weapons: &BTreeSet<rules::WeaponKey>) -> BTreeSet<Fact> {
    let mut feats = BTreeSet::new();
    for key in weapons {
        if key.as_str() == UNARMED_STRIKE {
            feats.insert(Fact::Feat(FeatKey::new(IMPROVED_UNARMED_STRIKE)));
        } else if (data.weapons.iter()).any(|weapon| {
            weapon.key == *key && weapon.proficiency == Some(WeaponProficiency::Exotic)
        }) {
            feats.insert(Fact::Feat(FeatKey::new(EXOTIC_WEAPON_PROFICIENCY)));
        }
    }
    feats
}
