//! Lists that name a selection's options (ticket 204): a feat's `Any` of the options of a selection must move with
//! the selection. Ticket 190 added two rogue talents and Extra Rogue Talent's `Any` fell behind until a code review
//! found it; these tests fail the same way for every list of the shape, with no code per feat.

use std::collections::{BTreeMap, BTreeSet};

use rules::{FeatureSelection, Prerequisite};

use crate::support::{DATA, feat, mythic_feat};

/// What an extra feat's prerequisite leaves out of what its extra pick may be of, each with its reason: the feat, an
/// option key prefix (a full key names one option; a prefix ending in `/` names every option under it), and why
/// that is no gap. An entry that covers nothing, or covers an option the prerequisite names, fails
/// [`every_exception_covers_an_option_the_prerequisite_leaves_out`].
const EXCEPTIONS: [(&str, &str, &str); 4] = [
    (
        "extra-rage-power",
        "skald/battle-scion/rage-power-or-teamwork-feat/teamwork-feat",
        "a teamwork feat is no rage power (ticket 189): the Battle Scion's slot offers it, the feat's prerequisite \
         names rage powers only",
    ),
    (
        "extra-arcana",
        "witch/hex/",
        "the witch hexes are Hex Arcana only for the Hexcrafter, whose branch of the prerequisite (Archetype, magus 6, \
         its first Hex Arcana slot) holds the feat (ticket 64)",
    ),
    (
        "extra-discovery",
        "rogue/rogue-talents/",
        "rogue talents are discoveries only for the Vivisectionist and the Underground Chemist, whose branches \
         (Archetype; the Feature) hold the feat (ticket 64)",
    ),
    (
        "extra-rogue-talent",
        "witch/hex/",
        "the witch hexes are Fey Tricks only for the Sylvan Trickster, whose Feature branch holds the feat",
    ),
];

/// The extra feats whose prerequisite names no option, so the rule doesn't apply: a class level is their guide's
/// "Slayer Talents or Talented Slayer" and "Wild Talents". A new extra feat of that shape needs a decision.
const NO_OPTIONS_NAMED: [&str; 2] = ["extra-slayer-talent", "extra-wild-talent"];

/// Every option condition anywhere in the prerequisite.
fn named_options(prerequisite: &Prerequisite) -> BTreeSet<String> {
    prerequisite
        .leaves()
        .filter_map(|leaf| match leaf {
            Prerequisite::SelectionOption(option) => Some(option.to_string()),
            _ => None,
        })
        .collect()
}

/// Every `Any` in the prerequisite, at any depth.
fn any_groups<'a>(prerequisite: &'a Prerequisite, found: &mut Vec<&'a [Prerequisite]>) {
    match prerequisite {
        Prerequisite::Any(children) => {
            found.push(children);
            children.iter().for_each(|child| any_groups(child, found));
        }
        Prerequisite::All(children) | Prerequisite::AtLeast(_, children) => {
            children.iter().for_each(|child| any_groups(child, found));
        }
        Prerequisite::Not(child) => any_groups(child, found),
        _ => {}
    }
}

/// The options an `Any` names as its children or through a nested `Any`: its "holds one of these".
fn options_of_any(children: &[Prerequisite]) -> BTreeSet<String> {
    children
        .iter()
        .flat_map(|child| match child {
            Prerequisite::SelectionOption(option) => BTreeSet::from([option.to_string()]),
            Prerequisite::Any(inner) => options_of_any(inner),
            _ => BTreeSet::new(),
        })
        .collect()
}

fn selection(key: &str) -> &'static FeatureSelection {
    DATA.selections()
        .find(|selection| selection.key.as_str() == key)
        .unwrap_or_else(|| panic!("no selection {key}"))
}

/// Every option the extra feat's pick may be of: the own and shared options of each selection it names.
fn offered(feat: &rules::Feat) -> BTreeSet<String> {
    feat.extra_pick_of
        .iter()
        .flat_map(|key| selection(key.as_str()).offered())
        .map(ToString::to_string)
        .collect()
}

fn excepted(feat: &str, option: &str) -> bool {
    EXCEPTIONS
        .iter()
        .any(|(f, prefix, _)| *f == feat && option.starts_with(prefix))
}

/// "You gain one additional hex for which you must meet the prerequisites", "Prerequisites: Hex": the prerequisite
/// of an extra feat that names options names every option its extra pick may be of (own and shared, of each
/// selection it names), less the table's exceptions, and names none it can't be of.
#[test]
fn extra_feats_name_every_option_of_their_selections() {
    let mut problems = Vec::new();
    let mut unnamed = BTreeSet::new();
    for feat in DATA.feats.iter().filter(|f| !f.extra_pick_of.is_empty()) {
        let key = feat.key.as_str();
        let named = feat
            .prerequisite
            .as_ref()
            .map(named_options)
            .unwrap_or_default();
        if named.is_empty() {
            unnamed.insert(key);
            continue;
        }
        let offered = offered(feat);
        let missing: Vec<&String> = offered
            .difference(&named)
            .filter(|option| !excepted(key, option))
            .collect();
        if !missing.is_empty() {
            problems.push(format!(
                "{key}: its pick may be of {} option(s) its prerequisite doesn't name: {missing:?}",
                missing.len()
            ));
        }
        let stray: Vec<&String> = named.difference(&offered).collect();
        if !stray.is_empty() {
            problems.push(format!(
                "{key}: its prerequisite names options its pick can't be of: {stray:?}"
            ));
        }
    }
    assert_eq!(
        unnamed,
        BTreeSet::from(NO_OPTIONS_NAMED),
        "the extra feats whose prerequisite names no option"
    );
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// An exception is a decision about options the prerequisite leaves out: one that covers none, or an option it now
/// names, is stale.
#[test]
fn every_exception_covers_an_option_the_prerequisite_leaves_out() {
    for (key, prefix, reason) in EXCEPTIONS {
        assert!(!reason.is_empty(), "{key} {prefix}: no reason");
        let feat = feat(key);
        let named = feat
            .prerequisite
            .as_ref()
            .map(named_options)
            .unwrap_or_default();
        let covered: Vec<String> = offered(feat)
            .into_iter()
            .filter(|option| option.starts_with(prefix))
            .collect();
        assert!(
            !covered.is_empty(),
            "{key}: nothing it offers starts with {prefix}"
        );
        let named_anyway: Vec<&String> = covered.iter().filter(|o| named.contains(*o)).collect();
        assert!(
            named_anyway.is_empty(),
            "{key}: the prerequisite names {named_anyway:?}, so the exception for {prefix} is stale"
        );
    }
}

/// An `Any` that names two or more of one selection's own options is that selection's "holds one of these", so it
/// names them all: the prerequisites of feats, mythic feats and mythic abilities (Extended Aspects, Second Mystery,
/// Witch Wandering Hex, the extra feats). Options', classes' and archetypes' prerequisites are left out: their
/// `Any`s of some elements or bloodlines are deliberate subsets.
#[test]
fn an_any_of_a_selections_options_names_all_of_them() {
    let mut owner: BTreeMap<String, &FeatureSelection> = BTreeMap::new();
    for selection in DATA.selections() {
        for option in &selection.options {
            owner.insert(option.key.to_string(), selection);
        }
    }
    let prerequisites = DATA
        .feats
        .iter()
        .map(|f| (f.key.to_string(), &f.prerequisite))
        .chain(
            DATA.mythic_feats
                .iter()
                .map(|f| (f.key.to_string(), &f.prerequisite)),
        )
        .chain(
            DATA.mythic_abilities
                .iter()
                .map(|a| (a.key.to_string(), &a.prerequisite)),
        );
    // A set: an `Any` nested in another names the same options twice.
    let mut problems = BTreeSet::new();
    for (key, prerequisite) in prerequisites {
        let mut groups = Vec::new();
        if let Some(prerequisite) = prerequisite {
            any_groups(prerequisite, &mut groups);
        }
        for children in groups {
            let named = options_of_any(children);
            let mut by_selection: BTreeMap<&str, BTreeSet<&String>> = BTreeMap::new();
            for option in &named {
                if let Some(selection) = owner.get(option) {
                    by_selection
                        .entry(selection.key.as_str())
                        .or_default()
                        .insert(option);
                }
            }
            for (selection_key, held) in by_selection.iter().filter(|(_, held)| held.len() >= 2) {
                let missing: Vec<String> = selection(selection_key)
                    .options
                    .iter()
                    .map(|o| o.key.to_string())
                    .filter(|o| !named.contains(o))
                    .collect();
                if !missing.is_empty() {
                    problems.insert(format!(
                        "{key}: names {} options of {selection_key} but not {missing:?}",
                        held.len()
                    ));
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{}",
        problems.into_iter().collect::<Vec<_>>().join("\n")
    );
}

/// Accomplished Sneak Attacker and Mythic Sneak Attack ("Prerequisites: Sneak Attack") name every class, archetype and
/// path feature that is a sneak attack: its key ends in `/sneak-attack` or `/sneak-attack-<archetype>` (the Knife
/// Master's Sneak Stab and the like are other features).
#[test]
fn the_sneak_attack_feats_name_every_sneak_attack_feature() {
    let is_sneak_attack = |key: &str| {
        let last = key.rsplit('/').next().unwrap_or(key);
        last == "sneak-attack" || last.starts_with("sneak-attack-")
    };
    let features: BTreeSet<String> = DATA
        .class_features()
        .chain(DATA.mythic_paths.iter().flat_map(|path| &path.features))
        .map(|feature| feature.key.to_string())
        .filter(|key| is_sneak_attack(key))
        .collect();
    assert_eq!(features.len(), 14, "{features:?}");
    for (name, prerequisite) in [
        (
            "accomplished-sneak-attacker",
            feat("accomplished-sneak-attacker").prerequisite.as_ref(),
        ),
        (
            "mythic-sneak-attack",
            mythic_feat("mythic-sneak-attack").prerequisite.as_ref(),
        ),
    ] {
        let named: BTreeSet<String> = prerequisite
            .unwrap_or_else(|| panic!("{name}: no prerequisite"))
            .leaves()
            .filter_map(|leaf| match leaf {
                Prerequisite::Feature { feature, .. } => Some(feature.to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(named, features, "{name}");
    }
}
