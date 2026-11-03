//! Ticket 58: ticket 54's decision 2 (user, 2026-11-02), applied to finished data. The monk's ki powers, a feature in
//! the data, become a pick, as ticket 45 made the wild talents.
//!
//! - **The pick** (`monk#ki-power`: "At 4th level and every 2 levels thereafter a Monk may select a ki power"). The
//!   "Ki Power" feature on those rows becomes a slot of the selection `monk/ki-power`, and the feature goes.
//! - **The options** are the guide's list (`monk#ki-powers`: "The Monk must be a certain level to unlock them"): 14
//!   powers, each needing the monk level of its `Lvl` column (`ClassLevel`). The cost in ki points isn't a planning
//!   fact. An option's guide link is its own section where the guide has one, else the page (the data's rule for an
//!   option with no heading of its own; seven have none, so a link to the list would make them copies).
//! - **The archetypes** pick where their own tables have "Ki Power" (the Scaled Fist not at 12, where Dragon Breath
//!   stands; the Sohei from 6; the Student of Stone not at 6 or 12; the Zen Archer not at 10); the rows already say
//!   so, and this asserts it on the rows in force.
//! - **The Traditional Monk** "Gains fixed Ki Powers": its table names one at 6, 8, 10, 12 and 14 ("Ki Power:
//!   Wholeness of Body", …) and no "Ki Power" cell. Its rows at those levels were the base class's (a raw extraction
//!   reads the named cells as the plain feature), so it would pick a ki power there. Each now holds its named power as
//!   a feature of its own (`monk/traditional-monk/ki-power-wholeness-of-body`, …) in place of the pick, and the rest
//!   of the base row as it is. "Ki Power: Perfect Self" at 20 is the base class's own feature already.
//!
//! No feat or mythic pick names ki powers: Extra Ki and Abundant Ki Pool grow the ki pool, and nothing needs the
//! feature that goes. Each rule asserts the guide phrase it rests on and what it finds in the data, so it fails loudly
//! on data it doesn't expect. [`apply`] runs on the finished data of an extraction (after ticket 57's `exploits`)
//! and, through ticket 58's one-off rewrite, on `libs/game-data/`.

use std::collections::BTreeMap;

use rules::{
    ArchetypeLevelRow, ClassKey, FeatureKey, FeatureSelectionKey, GameData, Prerequisite,
    effective_row,
};

use crate::follow_ups::{class_mut, says};
use crate::guide::Guide;
use crate::html::Page;
use crate::mercies_and_exploits::levels_in_tables;
use crate::names::slug;
use crate::option_levels::{Log, feature, option, page, section, selection};
use crate::wild_talents::{needing, take};

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

/// Makes the ki powers a pick, and the Traditional Monk's fixed ones its features.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
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
    let expected = expected_levels();
    assert_eq!(
        levels_in_tables(monk_page, "Ki Power"),
        expected,
        "monk: the tables' Ki Power levels"
    );
    no_feat_names_ki_powers(guide);

    let feature_key = FeatureKey::new(KI_POWER_FEATURE);
    assert_eq!(
        needing(data, &feature_key),
        Vec::<String>::new(),
        "what needs {feature_key}"
    );
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
        log.push(format!("{key}: monk level {}", power.level));
        chosen.options.push(one);
    }

    let selection_key = FeatureSelectionKey::new(KI_POWER);
    let monk = class_mut(data, MONK);
    assert!(
        monk.selections.iter().all(|s| s.key != selection_key),
        "{KI_POWER}: a selection already"
    );
    monk.selections.push(chosen);
    let before = monk.features.len();
    monk.features.retain(|f| f.key != feature_key);
    assert_eq!(monk.features.len(), before - 1, "{feature_key}");
    let mut class_levels = Vec::new();
    for row in &mut monk.levels {
        if take(&mut row.features, &feature_key) {
            row.selections.push(selection_key.clone());
            class_levels.push(row.level);
        }
    }
    assert_eq!(class_levels, EVERY, "the base class's rows");
    log.push(format!("{MONK} {class_levels:?}: a ki power"));
    for archetype in &mut monk.archetypes {
        for row in &mut archetype.levels {
            if take(&mut row.features, &feature_key) {
                assert!(
                    !row.selections.contains(&selection_key),
                    "{} {}: a ki power slot already",
                    archetype.key,
                    row.level
                );
                row.selections.push(selection_key.clone());
                log.push(format!("{} {}: a ki power", archetype.key, row.level));
            }
        }
    }

    traditional(guide, monk_page, &powers, data, &mut log);

    // Each table's levels are the rows in force: the archetype's where it differs, the base's elsewhere.
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
    log
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

/// The Traditional Monk's fixed ki powers, each a feature of its own on its row in place of the pick.
fn traditional(
    guide: &Guide,
    monk_page: &Page,
    powers: &[Power],
    data: &mut GameData,
    log: &mut Log,
) {
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

    let selection_key = FeatureSelectionKey::new(KI_POWER);
    let monk = class_mut(data, MONK);
    let base: BTreeMap<u8, rules::ClassLevelRow> = monk
        .levels
        .iter()
        .map(|row| (row.level, row.clone()))
        .collect();
    let archetype = monk
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == TRADITIONAL)
        .unwrap_or_else(|| panic!("no {TRADITIONAL}"));
    let perfect = archetype
        .levels
        .iter()
        .find(|row| row.level == perfect_self.0)
        .expect("the Traditional Monk's row 20");
    assert_eq!(
        perfect.features,
        [FeatureKey::new(PERFECT_SELF)],
        "{TRADITIONAL} 20"
    );
    for (level, name) in named.iter().filter(|fixed| **fixed != perfect_self) {
        let power = powers
            .iter()
            .find(|power| &power.name == name)
            .unwrap_or_else(|| panic!("{name}: not one of the guide's ki powers"));
        assert!(
            archetype.levels.iter().all(|row| row.level != *level),
            "{TRADITIONAL} {level}: a row of its own already"
        );
        let full = format!("Ki Power: {name}");
        let key = FeatureKey::new(format!("{TRADITIONAL}/{}", slug(&full)));
        assert!(power.anchor.is_some(), "{name}: no section of its own");
        archetype
            .features
            .push(feature(&key, &full, power.guide(monk_page)));
        let row = &base[level];
        assert_eq!(
            row.selections,
            std::slice::from_ref(&selection_key),
            "the base monk's row {level}"
        );
        let mut features = row.features.clone();
        features.push(key.clone());
        features.sort();
        archetype.levels.push(ArchetypeLevelRow {
            level: *level,
            spell_levels: None,
            features,
            feats: row.feats.clone(),
            feats_for: row.feats_for.clone(),
            feat_slots: row.feat_slots.clone(),
            selections: Vec::new(),
        });
        log.push(format!("{TRADITIONAL} {level}: {key}, no ki power"));
    }
    archetype.features.sort_by(|a, b| a.key.cmp(&b.key));
    archetype.levels.sort_by_key(|row| row.level);
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
