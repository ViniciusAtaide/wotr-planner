//! Effect summaries (ticket 112): every pickable item's short summary in our own words, written from the saved guide
//! pages and reviewed against them in full. The empty ones are items the guide says nothing usable about (cantrips,
//! orisons, features named only in a level table, guide links without an anchor); their count only goes down.

use std::collections::BTreeMap;

use crate::support::*;

/// Each kind's (summaries, empty), in the order the counts are kept below.
fn counts() -> BTreeMap<&'static str, (usize, usize)> {
    let mut counts = BTreeMap::new();
    let mut add = |kind: &'static str, summaries: Vec<&str>| {
        let empty = summaries
            .iter()
            .filter(|summary| summary.is_empty())
            .count();
        counts.insert(kind, (summaries.len(), empty));
    };
    add(
        "race",
        DATA.races
            .iter()
            .map(|race| race.summary.as_str())
            .collect(),
    );
    add(
        "subrace",
        DATA.races
            .iter()
            .flat_map(|race| &race.subraces)
            .map(|subrace| subrace.summary.as_str())
            .collect(),
    );
    add(
        "feat",
        DATA.feats
            .iter()
            .map(|feat| feat.summary.as_str())
            .collect(),
    );
    add(
        "class",
        DATA.all_classes()
            .map(|class| class.summary.as_str())
            .collect(),
    );
    add(
        "archetype",
        DATA.classes
            .iter()
            .flat_map(|class| &class.archetypes)
            .map(|archetype| archetype.summary.as_str())
            .collect(),
    );
    add(
        "class feature",
        DATA.class_features()
            .map(|feature| feature.summary.as_str())
            .collect(),
    );
    add(
        "class selection",
        DATA.class_selections()
            .map(|selection| selection.summary.as_str())
            .collect(),
    );
    add(
        "class selection option",
        DATA.class_selections()
            .flat_map(|selection| &selection.options)
            .map(|option| option.summary.as_str())
            .collect(),
    );
    add(
        "mythic ability",
        DATA.mythic_abilities
            .iter()
            .map(|ability| ability.summary.as_str())
            .collect(),
    );
    add(
        "mythic feat",
        DATA.mythic_feats
            .iter()
            .map(|feat| feat.summary.as_str())
            .collect(),
    );
    add(
        "mythic path",
        DATA.mythic_paths
            .iter()
            .map(|path| path.summary.as_str())
            .collect(),
    );
    add(
        "path feature",
        DATA.mythic_paths
            .iter()
            .flat_map(|path| &path.features)
            .map(|feature| feature.summary.as_str())
            .collect(),
    );
    add(
        "path selection",
        DATA.mythic_paths
            .iter()
            .flat_map(|path| &path.selections)
            .map(|selection| selection.summary.as_str())
            .collect(),
    );
    add(
        "path selection option",
        DATA.mythic_paths
            .iter()
            .flat_map(|path| &path.selections)
            .flat_map(|selection| &selection.options)
            .map(|option| option.summary.as_str())
            .collect(),
    );
    counts
}

/// Each kind's count of items without a summary, as of ticket 112. Filling one in lowers its count here; a data
/// change that adds an item without a summary raises it, and must write one or say here why the guide has none.
const EMPTY: [(&str, usize); 14] = [
    ("archetype", 0),
    ("class", 0),
    ("class feature", 104),
    ("class selection", 0),
    ("class selection option", 6),
    ("feat", 0),
    ("mythic ability", 0),
    ("mythic feat", 0),
    ("mythic path", 7),
    ("path feature", 7),
    ("path selection", 0),
    ("path selection option", 0),
    ("race", 0),
    ("subrace", 0),
];

#[test]
fn items_without_a_summary_never_grow_in_number() {
    let counts = counts();
    let empty: Vec<(&str, usize)> = counts
        .iter()
        .map(|(&kind, &(_, empty))| (kind, empty))
        .collect();
    assert_eq!(
        empty, EMPTY,
        "the empty summaries per kind (all, empty): {counts:?}"
    );
}

#[test]
fn every_summary_is_one_short_line() {
    let long: Vec<&str> = [
        DATA.feats
            .iter()
            .map(|feat| feat.summary.as_str())
            .collect::<Vec<_>>(),
        DATA.class_features()
            .map(|feature| feature.summary.as_str())
            .collect(),
        DATA.selections()
            .flat_map(|selection| &selection.options)
            .map(|option| option.summary.as_str())
            .collect(),
        DATA.mythic_abilities
            .iter()
            .map(|ability| ability.summary.as_str())
            .collect(),
        DATA.mythic_feats
            .iter()
            .map(|feat| feat.summary.as_str())
            .collect(),
    ]
    .concat()
    .into_iter()
    .filter(|summary| summary.chars().count() > 160 || summary.contains('\n'))
    .collect();
    assert_eq!(long, Vec::<&str>::new());
}
