//! The Hellknight's order (the Armiger's seven) and Pentamic Faith's domain, the Signifer's order, and their check.

use std::collections::BTreeSet;

use rules::{
    FeatureKey, FeatureSelection, FeatureSelectionKey, GameData, OptionLevelRow, SelectionOptionKey,
};

use crate::guide::Guide;
use crate::option_levels::{empty_row, page, section};
use crate::picks::{Pick, Picks, Rule};

use super::domain_selections::{PENTAMIC_DISCIPLINE, PENTAMIC_DOMAINS};
use super::{Family, contains, sharing};

/// "If the character was not previously an Armiger, they must choose one of the seven Hellknight orders. Otherwise,
/// they retain their current order." (the Armiger's seven, for a character holding none), and Pentamic Faith: "The
/// Hellknight gains access to one of the following domains … She uses her class level to determine which domain
/// abilities she can use." Built with the class: the "Order" cell at 1 is the pick, and the disciplines are levelled
/// by Hellknight levels, Pentamic Faith's row 1 granting its domain.
pub(super) fn hellknight(guide: &Guide, domains: &Family, picks: &mut Picks) {
    let armiger_orders = section(page(guide, "fighter"), "order");
    let orders: BTreeSet<SelectionOptionKey> = armiger_orders
        .children
        .iter()
        .map(|order| {
            assert!(
                order.anchor.starts_with("order-of-the-"),
                "{}",
                order.anchor
            );
            SelectionOptionKey::new(format!("{ARMIGER_ORDER}/{}", order.anchor))
        })
        .collect();
    assert_eq!(orders.len(), 7);
    picks.selection(
        "hellknight",
        FeatureSelection {
            only_without_option: true,
            ..sharing(
                HELLKNIGHT_ORDER,
                "Order",
                "prestige-classes#order".to_owned(),
                &[],
                orders.clone(),
            )
        },
    );
    // The Signifer's "Order" cell at 1: "The Signifer must choose one of the seven Hellknight orders" (ticket 33),
    // offered, as the Hellknight's, only to a character holding none ("Otherwise, they retain their current order",
    // ticket 35).
    let prestige = page(guide, "prestige-classes");
    contains(
        &section(prestige, "order-hellknight-signifer").text(),
        "The Signifer must choose one of the seven Hellknight orders.",
        "prestige-classes#order-hellknight-signifer",
    );
    contains(
        &section(prestige, "order").text(),
        "Otherwise, they retain their current order.",
        "prestige-classes#order",
    );
    picks.selection(
        "hellknight-signifer",
        FeatureSelection {
            only_without_option: true,
            ..sharing(
                SIGNIFER_ORDER,
                "Order",
                "prestige-classes#order-hellknight-signifer".to_owned(),
                &[],
                orders,
            )
        },
    );
    picks.rule(Rule {
        feature: FeatureKey::new(SIGNIFER_ORDER),
        table: Some("hellknight-signifer".to_owned()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::Selection(FeatureSelectionKey::new(SIGNIFER_ORDER)),
    });
    let faith = "hellknight/pentamic-faith";
    picks.selection(
        "hellknight",
        sharing(
            faith,
            "Pentamic Faith",
            "prestige-classes#pentamic-faith".to_owned(),
            &["hellknight"],
            domains.keys(&PENTAMIC_DOMAINS),
        ),
    );
    picks.option_levels("hellknight/disciplines", &["hellknight"]);
    picks.option_rows(
        PENTAMIC_DISCIPLINE,
        vec![OptionLevelRow {
            selections: vec![FeatureSelectionKey::new(faith)],
            ..empty_row(1)
        }],
    );
    picks.rule(Rule {
        feature: FeatureKey::new(HELLKNIGHT_ORDER),
        table: Some("hellknight".to_owned()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::Selection(FeatureSelectionKey::new(HELLKNIGHT_ORDER)),
    });
}

const ARMIGER_ORDER: &str = "fighter/armiger/order";
const HELLKNIGHT_ORDER: &str = "hellknight/order";
const SIGNIFER_ORDER: &str = "hellknight-signifer/order";

/// The Hellknight's order shares exactly the Armiger's orders and is its row 1's pick, as the Signifer's is its own;
/// Pentamic Faith's option has its row.
pub(super) fn check_hellknight(data: &GameData) {
    let armiger: BTreeSet<&SelectionOptionKey> = data
        .classes
        .iter()
        .flat_map(|c| &c.archetypes)
        .flat_map(|a| &a.selections)
        .find(|s| s.key.as_str() == ARMIGER_ORDER)
        .expect("the Armiger's orders")
        .options
        .iter()
        .map(|o| &o.key)
        .collect();
    let hellknight = data
        .prestige_classes
        .iter()
        .find(|c| c.key.as_str() == "hellknight")
        .expect("the Hellknight");
    let order = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == HELLKNIGHT_ORDER)
        .expect("the Hellknight's order");
    assert_eq!(
        order.shared_options.iter().collect::<BTreeSet<_>>(),
        armiger
    );
    let signifer = data
        .prestige_classes
        .iter()
        .find(|c| c.key.as_str() == "hellknight-signifer")
        .expect("the Signifer");
    assert!(
        signifer.levels.iter().any(
            |row| row.level == 1 && row.selections.iter().any(|s| s.as_str() == SIGNIFER_ORDER)
        ),
        "the Signifer's Order cell"
    );
    let first = hellknight
        .levels
        .iter()
        .find(|row| row.level == 1)
        .expect("row 1");
    assert_eq!(
        first
            .selections
            .iter()
            .filter(|s| s.as_str() == HELLKNIGHT_ORDER)
            .count(),
        1,
        "the Hellknight's Order cell"
    );
    let disciplines = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight/disciplines")
        .expect("the disciplines");
    assert!(
        disciplines.options.iter().any(|o| o.key.as_str()
            == "hellknight/disciplines/pentamic-faith"
            && !o.levels.is_empty()),
        "Pentamic Faith's row"
    );
}
