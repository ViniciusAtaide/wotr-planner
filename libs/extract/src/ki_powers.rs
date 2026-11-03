//! Ticket 58: ticket 54's decision 2 (user, 2026-11-02). The monk's ki powers are a pick, as ticket 45 made the wild
//! talents; the class stage reads them so ([`provide`]).
//!
//! - **The pick** (`monk#ki-power`: "At 4th level and every 2 levels thereafter a Monk may select a ki power"). Each
//!   "Ki Power" cell is a slot of the monk's selection `monk/ki-power`.
//! - **The options** are the guide's list (`monk#ki-powers`: "The Monk must be a certain level to unlock them"): 14
//!   powers, each needing the monk level of its `Lvl` column (`ClassLevel`). The cost in ki points isn't a planning
//!   fact. An option's guide link is its own section where the guide has one, else the page (the data's rule for an
//!   option with no heading of its own; seven have none, so a link to the list would make them copies).
//! - **The archetypes** pick where their own tables have "Ki Power" (the Scaled Fist not at 12, where Dragon Breath
//!   stands; the Sohei from 6; the Student of Stone not at 6 or 12; the Zen Archer not at 10), which [`check`]
//!   asserts on the rows in force.
//! - **The Traditional Monk** "Gains fixed Ki Powers": its table names one at 6, 8, 10, 12 and 14 ("Ki Power:
//!   Wholeness of Body", …) and no "Ki Power" cell. Each named cell is a feature of its own
//!   (`monk/traditional-monk/ki-power-wholeness-of-body`, …), not a pick. "Ki Power: Perfect Self" at 20 is the base
//!   class's own feature already.
//!
//! No feat or mythic pick names ki powers: Extra Ki and Abundant Ki Pool grow the ki pool. Each rule asserts the guide
//! phrase it rests on where the pick is read, and [`check`] what it finds in the finished data, so it fails loudly on
//! data it doesn't expect.

use std::collections::BTreeMap;

use rules::{ClassKey, FeatureKey, FeatureSelectionKey, GameData, Prerequisite, effective_row};

use crate::follow_ups::says;
use crate::guide::Guide;
use crate::html::Page;
use crate::mercies_and_exploits::levels_in_tables;
use crate::names::slug;
use crate::option_levels::{feature, option, page, section, selection};
use crate::picks::{Pick, Picks, Rule};

const MONK: &str = "monk";
/// The feature the rows held, which the pick replaces.
const KI_POWER_FEATURE: &str = "monk/ki-power";
/// The selection, keyed as the feature was (as `paladin/mercy`).
const KI_POWER: &str = "monk/ki-power";
const TRADITIONAL: &str = "monk/traditional-monk";
/// The base class's own feature for the table's "Ki Power: Perfect Self", which isn't one of the list's powers.
const PERFECT_SELF: &str = "monk/ki-power-perfect-self";
/// 4th level and every 2 levels after.
const EVERY: [u8; 9] = [4, 6, 8, 10, 12, 14, 16, 18, 20];

/// The ki powers are a pick, and the Traditional Monk's fixed ones its features.
pub fn provide(guide: &Guide, picks: &mut Picks) {
    let monk_page = page(guide, MONK);
    says(
        guide,
        MONK,
        "ki-power",
        "At 4th level and every 2 levels thereafter a Monk may select a ki power",
    );
    says(
        guide,
        MONK,
        "ki-powers",
        "The Monk must be a certain level to unlock them",
    );
    assert_eq!(
        levels_in_tables(monk_page, "Ki Power"),
        expected_levels(),
        "monk: the tables' Ki Power levels"
    );
    no_feat_names_ki_powers(guide);

    let powers = read_powers(monk_page);
    assert_eq!(powers.len(), 14, "the guide's ki powers");
    let mut chosen = selection(KI_POWER, "Ki Power", monk_page.guide("ki-power"), &[]);
    for power in &powers {
        let key = power_key(&power.name);
        let mut one = option(&key, &power.name, power.guide(monk_page), Vec::new());
        one.prerequisite = Some(Prerequisite::ClassLevel {
            class: ClassKey::new(MONK),
            level: power.level,
        });
        chosen.options.push(one);
    }
    picks.selection(MONK, chosen);
    picks.rule(Rule::every(
        KI_POWER_FEATURE,
        Pick::Selection(FeatureSelectionKey::new(KI_POWER)),
    ));
    traditional(guide, monk_page, &powers, picks);
}

/// Each table's levels are the rows in force: the archetype's where it differs, the base's elsewhere. The Traditional
/// Monk's row 20 holds the base class's Perfect Self.
pub fn check(_guide: &Guide, data: &GameData) {
    let expected = expected_levels();
    let selection_key = FeatureSelectionKey::new(KI_POWER);
    let monk = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == MONK)
        .expect("the monk");
    let held = |archetype| -> Vec<u8> {
        (1..=20)
            .filter(|level| {
                effective_row(monk, archetype, *level).is_some_and(|row| {
                    row.selections
                        .iter()
                        .filter(|s| **s == selection_key)
                        .count()
                        == 1
                })
            })
            .collect()
    };
    assert_eq!(held(None), EVERY, "the monk's ki powers");
    for archetype in &monk.archetypes {
        let anchor = archetype.guide.0.strip_prefix("monk#").unwrap_or_default();
        assert_eq!(
            held(Some(archetype)),
            expected.get(anchor).cloned().unwrap_or_default(),
            "{}'s ki powers",
            archetype.key
        );
    }
    assert_eq!(monk.archetypes.len(), expected.len(), "the archetypes");
    let perfect = monk
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == TRADITIONAL)
        .and_then(|a| a.levels.iter().find(|row| row.level == 20))
        .unwrap_or_else(|| panic!("{TRADITIONAL} has no row 20"));
    assert_eq!(
        perfect.features,
        [FeatureKey::new(PERFECT_SELF)],
        "{TRADITIONAL} 20"
    );
}

/// The levels of each table's "Ki Power" cell, by the anchor of the section holding it (the base class's: `""`).
/// Every archetype but the Traditional Monk has some.
fn expected_levels() -> BTreeMap<String, Vec<u8>> {
    let every = EVERY.to_vec();
    let without = |level: u8| every.iter().copied().filter(|l| *l != level).collect();
    [
        ("", every.clone()),
        ("drunken-master", every.clone()),
        ("quarterstaff-master", every.clone()),
        ("scaled-fist", without(12)),
        ("sensei", every.clone()),
        ("sohei", without(4)),
        ("student-of-stone", vec![4, 8, 10, 14, 16, 18, 20]),
        ("zen-archer", without(10)),
    ]
    .into_iter()
    .map(|(anchor, levels)| (anchor.to_owned(), levels))
    .collect()
}

/// Neither the feats nor the mythic picks name a ki power: their Extra feats (Extra Ki, Abundant Ki Pool) grow the
/// pool, so no feat gives a ki power pick (ticket 49's rule).
fn no_feat_names_ki_powers(guide: &Guide) {
    says(
        guide,
        "feats",
        "extra-ki",
        "Your ki pool increases by 2. This feat may be taken multiple times.",
    );
    says(
        guide,
        "mythic-paths",
        "abundant-ki-pool",
        "Your Ki Pool increases by a number of points equal to your mythic rank.",
    );
    for slug in ["feats", "mythic-paths"] {
        for section in page(guide, slug).root.walk() {
            assert!(
                !section.text().to_lowercase().contains("ki power"),
                "{slug}#{} names ki powers",
                section.anchor
            );
        }
    }
}

/// The Traditional Monk's fixed ki powers: each "Ki Power: …" cell (but Perfect Self's, the base class's own
/// feature) is a feature of its own, in place of the pick.
fn traditional(guide: &Guide, monk_page: &Page, powers: &[Power], picks: &mut Picks) {
    says(guide, MONK, "traditional-monk", "Gains fixed Ki Powers.");
    let table = section(monk_page, "traditional-monk")
        .tables()
        .next()
        .expect("monk#traditional-monk has no table");
    let named: Vec<(u8, String)> = table[1..]
        .iter()
        .flat_map(|row| {
            let level: u8 = row[0]
                .parse()
                .unwrap_or_else(|_| panic!("monk#traditional-monk: level {:?}", row[0]));
            row.iter()
                .filter_map(|cell| cell.strip_prefix("Ki Power: "))
                .map(move |name| (level, name.to_owned()))
        })
        .collect();
    let perfect_self = (20, "Perfect Self".to_owned());
    assert_eq!(
        named,
        [
            (6, "Wholeness of Body".to_owned()),
            (8, "Abundant Step".to_owned()),
            (10, "Diamond Body".to_owned()),
            (12, "Diamond Soul".to_owned()),
            (14, "Quivering Palm".to_owned()),
            perfect_self.clone(),
        ],
        "monk#traditional-monk: its fixed ki powers"
    );
    for (level, name) in named.iter().filter(|fixed| **fixed != perfect_self) {
        let power = powers
            .iter()
            .find(|power| &power.name == name)
            .unwrap_or_else(|| panic!("{name}: not one of the guide's ki powers"));
        assert!(power.anchor.is_some(), "{name}: no section of its own");
        let full = format!("Ki Power: {name}");
        let key = FeatureKey::new(format!("{TRADITIONAL}/{}", slug(&full)));
        picks.rule(Rule {
            table: Some(TRADITIONAL.to_owned()),
            cell: Some(full.clone()),
            levels: Some(vec![*level]),
            ..Rule::every(
                KI_POWER_FEATURE,
                Pick::Feature(feature(&key, &full, power.guide(monk_page))),
            )
        });
    }
}

/// One ki power as the guide's list gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Power {
    pub name: String,
    /// The monk level it unlocks at.
    pub level: u8,
    /// Its own section under the list where the guide has one, else none (the page).
    pub anchor: Option<String>,
}

/// The list's powers (`monk#ki-powers`: "Lvl | Ki Power | Cost"), each with its section under the list if it has one.
/// Every section under the list must be one of them.
pub fn read_powers(page: &Page) -> Vec<Power> {
    let list = section(page, "ki-powers");
    let table = list.tables().next().expect("monk#ki-powers has no table");
    assert_eq!(
        table[0],
        ["Lvl", "Ki Power", "Cost"],
        "monk#ki-powers's header"
    );
    let powers: Vec<Power> = table[1..]
        .iter()
        .map(|row| {
            let [level, name, _cost] = row.as_slice() else {
                panic!("monk#ki-powers: row {row:?}")
            };
            let own = slug(name);
            Power {
                name: name.clone(),
                level: level
                    .parse()
                    .unwrap_or_else(|_| panic!("monk#ki-powers: level {level:?}")),
                anchor: list.find(&own).map(|_| own),
            }
        })
        .collect();
    // Every section under the list is one of its powers.
    for child in list.walk().into_iter().skip(1) {
        assert!(
            powers
                .iter()
                .any(|power| power.anchor.as_ref() == Some(&child.anchor)),
            "monk#{}: not a ki power",
            child.anchor
        );
    }
    powers
}

impl Power {
    /// Its guide link: its own section, or the page.
    fn guide(&self, page: &Page) -> String {
        match &self.anchor {
            Some(anchor) => page.guide(anchor),
            None => page.slug.clone(),
        }
    }
}

/// `monk/ki-power/cold-ice-strike`.
fn power_key(name: &str) -> String {
    format!("{KI_POWER}/{}", slug(name))
}
