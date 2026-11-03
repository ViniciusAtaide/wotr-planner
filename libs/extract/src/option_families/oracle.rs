//! Taking the mysteries: the oracle's and its archetypes' tables, the mystery and revelation selections each builds,
//! and their check.

use std::collections::BTreeSet;

use rules::{
    DataAt, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData, LevelFrom,
    MythicAbilityKey, Prerequisite, SelectionOptionKey,
};

use crate::guide::Guide;
use crate::option_levels::{page, section, selection};
use crate::picks::{Pick, Picks, Rule};

use super::mysteries::{
    DUAL_CURSED, DUAL_CURSED_REVELATION, LONE_STRIDER_MYSTERIES, MYSTERY, REVELATION,
    SECOND_MYSTERY, dual_cursed_revelations,
};
use super::{Family, contains, sharing};

/// The oracle's tables (the class's and each archetype's), each with the selection its mystery cell at 1 picks.
/// Archetypes that take the mystery their own way: "he does not gain additional class skills from his choice of
/// mystery" (Divine Herbalist: skips the class skills at 1), "Gains a unique Final Revelation" (Enlightened
/// Philosopher: skips 20), "Can choose from a restricted list of Mysteries" (Lone Strider), "Must select the Wind
/// Mystery" (Wind Whisperer: granted with no pick).
const ORACLE_TABLES: [(&str, &str); 9] = [
    ("oracle", MYSTERY),
    ("oracle/divine-herbalist", "oracle/divine-herbalist/mystery"),
    (
        "oracle/enlightened-philosopher",
        "oracle/enlightened-philosopher/mystery",
    ),
    ("oracle/lone-strider", "oracle/lone-strider/mystery"),
    ("oracle/wind-whisperer", "oracle/wind-whisperer/mystery"),
    ("oracle/dual-cursed-oracle", MYSTERY),
    ("oracle/possessed-oracle", MYSTERY),
    ("oracle/purifier", MYSTERY),
    ("oracle/seeker", MYSTERY),
];

/// The oracle's mysteries, built with the class: the selection, its options' rows, the revelation pools and the
/// archetypes' own selections. The "Mystery" cell at 1 (the Lone Strider's "Mystery (Lone Strider)", the Wind
/// Whisperer's "Wind") is the table's mystery pick, every "Revelation" cell a slot of `oracle/revelation`, and the
/// "Final Revelation" at 20 the held mystery's row (the Wind Whisperer's "Wind Final Revelation" too; the Enlightened
/// Philosopher's own stays).
pub(super) fn oracle_mysteries(guide: &Guide, mysteries: &Family, picks: &mut Picks) {
    for feature in &mysteries.features {
        picks.feature("oracle", feature.clone());
    }
    picks.selection("oracle", mysteries.selection.clone());
    for nested in &mysteries.nested {
        picks.selection("oracle", nested.clone());
    }
    let own =
        |archetype: &str, name: &str, skipped: &[u8], options: BTreeSet<SelectionOptionKey>| {
            FeatureSelection {
                skipped_levels: skipped.iter().copied().collect(),
                ..sharing(
                    &format!("{archetype}/mystery"),
                    &format!("Mystery ({name})"),
                    format!("oracle#{}", archetype.trim_start_matches("oracle/")),
                    &["oracle"],
                    options,
                )
            }
        };
    let class_skill_levels: BTreeSet<u8> = mysteries
        .selection
        .options
        .iter()
        .flat_map(|o| o.levels.iter())
        .filter(|row| !row.class_skills.is_empty())
        .map(|row| {
            assert!(
                row.features.is_empty(),
                "a mystery's class skills share their row"
            );
            row.level
        })
        .collect();
    assert_eq!(class_skill_levels, BTreeSet::from([1]));
    let specials = [
        (
            "oracle/divine-herbalist",
            "Divine Herbalist",
            vec![1u8],
            mysteries.all_keys(),
        ),
        (
            "oracle/enlightened-philosopher",
            "Enlightened Philosopher",
            vec![20],
            mysteries.all_keys(),
        ),
        (
            "oracle/lone-strider",
            "Lone Strider",
            vec![],
            mysteries.keys(&LONE_STRIDER_MYSTERIES),
        ),
        (
            "oracle/wind-whisperer",
            "Wind Whisperer",
            vec![],
            mysteries.keys(&["wind"]),
        ),
    ];
    for (archetype, name, skipped, options) in &specials {
        picks.selection(archetype, own(archetype, name, skipped, options.clone()));
    }
    // "he can choose two Revelations unique to the archetype" (Dual-Cursed Oracle): its revelations are the
    // mystery's pool plus Misfortune and Fortune, at every revelation level of its table (ticket 33).
    picks.selection(DUAL_CURSED, dual_cursed_revelations(guide));
    // Second Mystery (ticket 33): "You gain a second Oracle Mystery", levelled like the first; it needs a mystery
    // held.
    contains(
        &section(page(guide, "mythic-paths"), "second-mystery").text(),
        "You gain a second Oracle Mystery.",
        "mythic-paths#second-mystery",
    );
    picks.selection(
        "oracle",
        FeatureSelection {
            shared_options: mysteries.all_keys(),
            level_from: Some(LevelFrom::OtherOptions),
            ..selection(
                SECOND_MYSTERY,
                "Second Mystery",
                "mythic-paths#second-mystery".into(),
                &[],
            )
        },
    );
    picks.mythic_selection("second-mystery", FeatureSelectionKey::new(SECOND_MYSTERY));
    picks.prerequisite(
        DataAt::MythicAbility(MythicAbilityKey::new("second-mystery")),
        "Mystery",
        Prerequisite::Any(
            mysteries
                .all_keys()
                .into_iter()
                .map(Prerequisite::SelectionOption)
                .collect(),
        ),
    );

    let rule = |feature: &str, table: Option<&str>, levels: Option<Vec<u8>>, pick: Pick| Rule {
        feature: FeatureKey::new(feature),
        table: table.map(str::to_owned),
        cell: None,
        levels,
        pick,
    };
    // A table's own pick first: [`Picks::pick`] takes the first rule that matches.
    picks.rule(rule(
        REVELATION,
        Some(DUAL_CURSED),
        None,
        Pick::Selection(FeatureSelectionKey::new(DUAL_CURSED_REVELATION)),
    ));
    for (table, pick) in ORACLE_TABLES {
        picks.rule(rule(
            MYSTERY,
            Some(table),
            Some(vec![1]),
            Pick::Selection(FeatureSelectionKey::new(pick)),
        ));
    }
    picks.rule(rule(
        "oracle/lone-strider/mystery-lone-strider",
        Some("oracle/lone-strider"),
        Some(vec![1]),
        Pick::Selection(FeatureSelectionKey::new("oracle/lone-strider/mystery")),
    ));
    // The shaman's Air Barrier, Vortex Spells and Wind Ward resolve their "Wind" line to this feature, so it stays
    // defined (not on the row) until that's settled.
    picks.rule(rule(
        "oracle/wind-whisperer/wind",
        Some("oracle/wind-whisperer"),
        Some(vec![1]),
        Pick::SelectionKeepingFeature(FeatureSelectionKey::new("oracle/wind-whisperer/mystery")),
    ));
    picks.rule(rule(
        REVELATION,
        None,
        None,
        Pick::Selection(FeatureSelectionKey::new(REVELATION)),
    ));
    for final_revelation in [
        "oracle/final-revelation",
        "oracle/wind-whisperer/wind-final-revelation",
    ] {
        picks.rule(rule(
            final_revelation,
            None,
            Some(vec![20]),
            Pick::OptionRow,
        ));
    }
}

/// Every oracle and archetype holds one mystery from level 1, of its table's selection.
pub(super) fn check_mysteries(data: &GameData) {
    let oracle = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "oracle")
        .expect("the oracle");
    let archetypes: BTreeSet<&str> = oracle.archetypes.iter().map(|a| a.key.as_str()).collect();
    assert_eq!(
        archetypes,
        ORACLE_TABLES[1..].iter().map(|(table, _)| *table).collect(),
        "the oracle's archetypes"
    );
    let base = oracle
        .levels
        .iter()
        .find(|row| row.level == 1)
        .expect("row 1");
    for (table, pick) in ORACLE_TABLES {
        let own = oracle
            .archetypes
            .iter()
            .find(|a| a.key.as_str() == table)
            .and_then(|a| a.levels.iter().find(|row| row.level == 1));
        let selections = own.map_or(&base.selections, |row| &row.selections);
        let picks: Vec<&str> = selections
            .iter()
            .map(FeatureSelectionKey::as_str)
            .filter(|s| s.ends_with("/mystery"))
            .collect();
        assert_eq!(picks, [pick], "{table}: mystery picks at 1");
    }
    let revelation_levels = |rows: Vec<(u8, &Vec<FeatureSelectionKey>)>, key: &str| -> Vec<u8> {
        rows.into_iter()
            .filter(|(_, selections)| selections.iter().any(|s| s.as_str() == key))
            .map(|(level, _)| level)
            .collect()
    };
    let base_rows: Vec<(u8, &Vec<FeatureSelectionKey>)> = oracle
        .levels
        .iter()
        .map(|row| (row.level, &row.selections))
        .collect();
    assert_eq!(
        revelation_levels(base_rows, REVELATION),
        [1, 3, 7, 11, 15, 19],
        "the oracle's revelations"
    );
    let dual = oracle
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == DUAL_CURSED)
        .expect("the Dual-Cursed Oracle");
    let dual_rows: Vec<(u8, &Vec<FeatureSelectionKey>)> = dual
        .levels
        .iter()
        .map(|row| (row.level, &row.selections))
        .collect();
    assert_eq!(
        revelation_levels(dual_rows, DUAL_CURSED_REVELATION),
        [1, 3, 5, 7, 11, 13, 15, 19],
        "the Dual-Cursed's revelations"
    );
}
