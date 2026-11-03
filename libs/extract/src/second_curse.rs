//! Ticket 67: ticket 62's decision (user, 2026-11-02, by recommendation). The Dual-Cursed Oracle's "Second Curse" is
//! a pick, not a feature: "The Oracle must choose a second curse" (`oracle#second-curse`), and its table has "Second
//! Curse" at 1 beside "Oracle's Curse". The class stage reads that cell as a second slot of the oracle's own curses
//! (`oracle/oracles-curse`) on the archetype's level-1 row ([`provide`]). Every curse is taken once, so the second
//! slot doesn't offer the curse taken in the first, and Beneficial Curse's slot (ticket 62) offers neither.
//!
//! "One of these curses doesn't change as the oracle gains levels" needs nothing: no curse has rows in the data, and
//! the archetype's "Curse 2", "Curse 3" and "Curse 4" features stay as they are.
//!
//! Each rule asserts the guide phrase it rests on where the pick is read, and [`check`] what it finds in the finished
//! data, so it fails loudly on data it doesn't expect.

use std::collections::BTreeMap;

use rules::{FeatureSelectionKey, GameData, Repeat, effective_row};

use crate::follow_ups::says;
use crate::guide::Guide;
use crate::mercies_and_exploits::levels_in_tables;
use crate::option_levels::page;
use crate::picks::{Pick, Picks, Rule};

const ORACLE: &str = "oracle";
const DUAL_CURSED: &str = "oracle/dual-cursed-oracle";
const ANCHOR: &str = "dual-cursed-oracle";
const SECOND_CURSE: &str = "oracle/dual-cursed-oracle/second-curse";
const CURSES: &str = "oracle/oracles-curse";
/// "Second Curse" in the archetype's table.
const LEVEL: u8 = 1;

/// The archetype's "Second Curse" cell is a slot of the oracle's curses.
pub fn provide(guide: &Guide, picks: &mut Picks) {
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
    picks.rule(Rule {
        table: Some(DUAL_CURSED.to_owned()),
        levels: Some(vec![LEVEL]),
        ..Rule::every(
            SECOND_CURSE,
            Pick::Selection(FeatureSelectionKey::new(CURSES)),
        )
    });
}

/// Every curse is taken once, and the archetype's rows in force hold two curse slots, at 1.
pub fn check(_guide: &Guide, data: &GameData) {
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
        [(LEVEL, 2)],
        "{DUAL_CURSED}'s curse slots"
    );
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
