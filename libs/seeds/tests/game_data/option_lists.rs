//! Lists that name a selection's options (ticket 204): a feat's `Any` of the options of a selection must move with
//! the selection. Ticket 190 added two rogue talents and Extra Rogue Talent's `Any` fell behind until a code review
//! found it; these tests fail the same way for every list of the shape, with no code per feat.

use std::collections::{BTreeMap, BTreeSet};

use rules::{FeatureSelection, Prerequisite};

use crate::support::{DATA, feat, mythic_feat, picks_of};

/// What an extra feat's prerequisite leaves out of what its extra pick may be of, each with its reason: the feat, an
/// option key prefix (a full key names one option; a prefix ending in `/` names every option under it), and why
/// that is no gap. An entry that covers nothing, or covers an option the prerequisite names, fails
/// [`every_exception_covers_an_option_the_prerequisite_leaves_out`].
const EXCEPTIONS: [(&str, &str, &str); 2] = [
    (
        "extra-arcanist-exploit",
        "arcanist/greater-exploits/",
        "the six greater exploits are in the game's list (306i), and the feat's prerequisite still names the sixteen \
         exploits only: the importer's prerequisite stage (293) writes the game's selections",
    ),
    (
        "extra-rogue-talent",
        "rogue/rogue-talents/evasion",
        "Evasion is a rogue talent only in the game's Extra Rogue Talent list (306aa), where it sits beside the \
         rogue's 38; the feat's prerequisite names the talents",
    ),
];

/// The extra feats whose prerequisite names no option, so the rule doesn't apply: a class level is their guide's
/// "Slayer Talents or Talented Slayer" and "Wild Talents". A new extra feat of that shape needs a decision.
const NO_OPTIONS_NAMED: [&str; 3] = [
    "extra-discovery-vivisectionist",
    "extra-slayer-talent",
    "extra-wild-talent",
];

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
    DATA.selections
        .iter()
        .find(|selection| selection.key.as_str() == key)
        .unwrap_or_else(|| panic!("no selection {key}"))
}

/// Every option the extra feat's pick may be of: the own and shared options of each selection it names.
fn offered(feat: &rules::Feat) -> BTreeSet<String> {
    picks_of(feat)
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
    for feat in DATA.feats.iter().filter(|f| f.selection.is_some()) {
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
    for selection in &DATA.selections {
        for option in &selection.options {
            owner.insert(option.to_string(), selection);
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
                    .map(|o| o.to_string())
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

/// Accomplished Sneak Attacker and Mythic Sneak Attack ("Prerequisites: Sneak Attack") name the one Sneak Attack,
/// which every class, archetype and path that has a sneak attack grants (one entry since ticket 244; the Knife
/// Master's Sneak Stab and the like are other features).
#[test]
fn the_sneak_attack_feats_name_every_sneak_attack_feature() {
    let features: BTreeSet<String> = DATA
        .features
        .iter()
        .filter(|feature| {
            feature.blueprint.as_ref().map(|guid| guid.as_str())
                == Some("9b9eac6709e1c084cb18c3a366e0ec87")
        })
        .map(|feature| feature.key.to_string())
        .collect();
    // Game: SneakAttack 9b9eac6709e1c084cb18c3a366e0ec87, one feature.
    assert_eq!(features, BTreeSet::from(["sneak-attack".to_owned()]));
    // Game: AccomplishedSneakAttacker 9f0187869dc23744292c0e5bb364464e and SneakAttackerMythicFeat
    // d0a53bf03b978634890e5ebab4a90ecb need the SneakAttack stat at 1 or more (equivalent model: any of the
    // features that raise it).
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
