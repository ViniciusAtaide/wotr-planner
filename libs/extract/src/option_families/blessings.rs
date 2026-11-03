//! The warpriest's blessings, the Feral Champion's and the Champion of the Faith's, and their check.

use std::collections::BTreeSet;

use rules::{FeatureKey, FeatureSelectionKey, GameData, OptionLevelRow, SelectionOptionKey};

use crate::guide::Guide;
use crate::option_levels::{empty_row, feature, page, section};
use crate::picks::{Pick, Picks, Rule};

use super::{contains, sharing};

const BLESSINGS: &str = "warpriest/blessings";
const FERAL_BLESSING: &str = "warpriest/feral-champion/blessing";
const CHOSEN_BLESSING: &str = "warpriest/champion-of-the-faith/chosen-alignment";

/// The warpriest's blessings, built with the class: each blessing's minor power is its row 1 and its major power its
/// row 10, so the table's level-10 "Blessings" picks are the held blessings' rows. The Feral Champion's "Feral
/// Blessing" cells are its fixed blessing's pick at 1 and that blessing's row at 10; the Champion of the Faith also
/// picks the blessing of his chosen alignment at 1.
pub(super) fn blessings(guide: &Guide, picks: &mut Picks) {
    let warpriest_page = page(guide, "warpriest");
    let within = section(warpriest_page, "blessings");
    contains(
        &within.text(),
        "Each Blessing grants a minor power at 1st level and a major power at 10th level",
        "warpriest#blessings",
    );
    picks.option_levels(BLESSINGS, &["warpriest"]);
    for heading in &within.children {
        let key = format!("{BLESSINGS}/{}", heading.anchor);
        let text = heading.text();
        contains(&text, "At 1st level", &heading.anchor);
        contains(&text, "At 10th level", &heading.anchor);
        let short = heading.name.trim_end_matches(" Domain").to_owned();
        let mut rows = Vec::new();
        for (level, power) in [(1u8, "Minor"), (10, "Major")] {
            let feature_key = FeatureKey::new(format!("{key}/{}-blessing", power.to_lowercase()));
            picks.feature(
                "warpriest",
                feature(
                    &feature_key,
                    &format!("{power} Blessing ({short})"),
                    warpriest_page.guide(&heading.anchor),
                ),
            );
            rows.push(OptionLevelRow {
                features: vec![feature_key],
                ..empty_row(level)
            });
        }
        picks.option_rows(&key, rows);
    }
    picks.rule(Rule {
        feature: FeatureKey::new(BLESSINGS),
        table: None,
        cell: None,
        levels: Some(vec![10]),
        pick: Pick::OptionRow,
    });

    // "A Feral Champion must take the Animal Domain blessing and does not gain a second." "The Warpriest
    // automatically gains the Blessing for his chosen alignment" (Champion of the Faith).
    picks.selection(
        "warpriest/feral-champion",
        sharing(
            FERAL_BLESSING,
            "Blessing (Feral Champion)",
            "warpriest#feral-blessing".to_owned(),
            &["warpriest"],
            BTreeSet::from([SelectionOptionKey::new(format!(
                "{BLESSINGS}/animal-domain"
            ))]),
        ),
    );
    for (level, pick) in [
        (1, Pick::Selection(FeatureSelectionKey::new(FERAL_BLESSING))),
        (10, Pick::OptionRow),
    ] {
        picks.rule(Rule {
            feature: FeatureKey::new("warpriest/feral-champion/feral-blessing"),
            table: Some("warpriest/feral-champion".to_owned()),
            cell: None,
            levels: Some(vec![level]),
            pick,
        });
    }
    picks.selection(
        "warpriest/champion-of-the-faith",
        sharing(
            CHOSEN_BLESSING,
            "Chosen Alignment Blessing",
            "warpriest#chosen-alignment".to_owned(),
            &["warpriest"],
            ["chaos-domain", "evil-domain", "good-domain", "law-domain"]
                .iter()
                .map(|b| SelectionOptionKey::new(format!("{BLESSINGS}/{b}")))
                .collect(),
        ),
    );
    picks.row_selection("warpriest/champion-of-the-faith", 1, CHOSEN_BLESSING);
}

/// Every blessing has its two rows, and the rows in force pick no blessing at 10: the base class's, and every
/// archetype's own.
pub(super) fn check_blessings(guide: &Guide, data: &GameData) {
    let within = section(page(guide, "warpriest"), "blessings");
    let warpriest = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "warpriest")
        .expect("the warpriest");
    let blessings = warpriest
        .selections
        .iter()
        .find(|s| s.key.as_str() == BLESSINGS)
        .expect("the blessings");
    assert_eq!(blessings.options.len(), within.children.len());
    for (option, heading) in blessings.options.iter().zip(&within.children) {
        assert_eq!(
            option.guide.0,
            page(guide, "warpriest").guide(&heading.anchor)
        );
        assert_eq!(
            option
                .levels
                .iter()
                .map(|row| row.level)
                .collect::<Vec<_>>(),
            [1, 10],
            "{}",
            option.key
        );
    }
    let rows = warpriest
        .levels
        .iter()
        .map(|row| (row.level, &row.selections))
        .chain(
            warpriest
                .archetypes
                .iter()
                .flat_map(|a| a.levels.iter().map(|row| (row.level, &row.selections))),
        );
    for (level, selections) in rows {
        assert!(
            level != 10 || !selections.iter().any(|s| s.as_str() == BLESSINGS),
            "a blessing pick at 10"
        );
    }
}
