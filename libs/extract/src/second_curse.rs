//! Ticket 67: ticket 62's decision (user, 2026-11-02, by recommendation), applied to finished data. The Dual-Cursed
//! Oracle's "Second Curse" is a pick, not a feature: "The Oracle must choose a second curse" (`oracle#second-curse`),
//! and its table has "Second Curse" at 1 beside "Oracle's Curse". Its level-1 row holds a second slot of the oracle's
//! own curses (`oracle/oracles-curse`) in place of the feature, which goes. Every curse is taken once, so the second
//! slot doesn't offer the curse taken in the first, and Beneficial Curse's slot (ticket 62) offers neither.
//!
//! "One of these curses doesn't change as the oracle gains levels" needs nothing: no curse has rows in the data, and
//! the archetype's "Curse 2", "Curse 3" and "Curse 4" features stay as they are.
//!
//! Nothing else names the feature that goes. Each rule asserts the guide phrase it rests on and what it finds in the
//! data, so it fails loudly on data it doesn't expect. [`apply`] runs on the finished data of an extraction (after
//! ticket 66's `instinctual_rage_powers`) and, through ticket 67's one-off rewrite, on `libs/game-data/`.

use std::collections::BTreeMap;

use rules::{FeatureKey, FeatureSelectionKey, GameData, Repeat, effective_row};

use crate::follow_ups::{archetype_mut, says};
use crate::guide::Guide;
use crate::mercies_and_exploits::levels_in_tables;
use crate::option_levels::{Log, page};
use crate::wild_talents::{needing, take};

const ORACLE: &str = "oracle";
const DUAL_CURSED: &str = "oracle/dual-cursed-oracle";
const ANCHOR: &str = "dual-cursed-oracle";
const SECOND_CURSE: &str = "oracle/dual-cursed-oracle/second-curse";
const CURSES: &str = "oracle/oracles-curse";
/// "Second Curse" in the archetype's table.
const LEVEL: u8 = 1;

/// Makes the Dual-Cursed Oracle's second curse a slot of the oracle's curses.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    says(guide, ORACLE, ANCHOR, "Must accept two curses");
    says(
        guide,
        ORACLE,
        "second-curse",
        "The Oracle must choose a second curse.",
    );
    let oracle_page = page(guide, ORACLE);
    assert_eq!(
        levels_in_tables(oracle_page, "Second Curse"),
        BTreeMap::from([(ANCHOR.to_owned(), vec![LEVEL])]),
        "oracle: the tables' Second Curse levels"
    );
    let first = levels_in_tables(oracle_page, "Oracle's Curse");
    assert_eq!(first.get(""), Some(&vec![LEVEL]), "the oracle's curse");
    assert_eq!(
        first.get(ANCHOR),
        Some(&vec![LEVEL]),
        "{DUAL_CURSED}'s first curse"
    );

    let curses = FeatureSelectionKey::new(CURSES);
    let oracle = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == ORACLE)
        .expect("the oracle");
    let selection = oracle
        .selections
        .iter()
        .find(|s| s.key == curses)
        .unwrap_or_else(|| panic!("no {CURSES}"));
    assert!(
        selection
            .options
            .iter()
            .all(|option| option.repeat == Repeat::Once),
        "a curse taken more than once"
    );
    assert_eq!(
        slots_in_force(data, &curses),
        [(LEVEL, 1)],
        "{DUAL_CURSED}'s curse slots"
    );

    let feature = FeatureKey::new(SECOND_CURSE);
    assert_eq!(
        needing(data, &feature),
        Vec::<String>::new(),
        "what needs {feature}"
    );
    assert!(
        data.classes
            .iter()
            .find(|c| c.key.as_str() == ORACLE)
            .is_some_and(|c| c.levels.iter().all(|row| !row.features.contains(&feature))),
        "the base oracle holds {feature}"
    );
    let archetype = archetype_mut(data, ORACLE, DUAL_CURSED);
    let before = archetype.features.len();
    archetype.features.retain(|f| f.key != feature);
    assert_eq!(archetype.features.len(), before - 1, "{feature}");
    let mut levels = Vec::new();
    for row in &mut archetype.levels {
        if take(&mut row.features, &feature) {
            assert_eq!(
                row.selections.iter().filter(|s| **s == curses).count(),
                1,
                "{DUAL_CURSED} {}: its curse slots",
                row.level
            );
            row.selections.push(curses.clone());
            row.selections.sort();
            levels.push(row.level);
        }
    }
    assert_eq!(levels, [LEVEL], "{DUAL_CURSED}'s Second Curse rows");
    assert_eq!(
        slots_in_force(data, &curses),
        [(LEVEL, 2)],
        "{DUAL_CURSED}'s curse slots"
    );
    vec![format!("{DUAL_CURSED} {levels:?}: a second curse")]
}

/// The Dual-Cursed Oracle's levels whose row in force holds a slot of `selection`, with how many.
fn slots_in_force(data: &GameData, selection: &FeatureSelectionKey) -> Vec<(u8, usize)> {
    let oracle = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == ORACLE)
        .expect("the oracle");
    let archetype = oracle
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == DUAL_CURSED)
        .unwrap_or_else(|| panic!("no {DUAL_CURSED}"));
    (1..=20)
        .filter_map(|level| {
            let row = effective_row(oracle, Some(archetype), level)?;
            let count = row.selections.iter().filter(|s| *s == selection).count();
            (count > 0).then_some((level, count))
        })
        .collect()
}
