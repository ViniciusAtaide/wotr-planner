//! The families whose options each have a table: the wizard's schools, the cavalier's orders and the Magic
//! Deceiver's ways.

use std::collections::BTreeMap;

use rules::FeatureSelection;

use crate::guide::Guide;

use super::reading::{FeatNames, anchor_for, own_cells, page, section, table};
use super::rows::{RowBuilder, option, selection};
use super::sorcerer::section_text;
use super::{Family, Owner};

/// A family whose every option has a table and its features under headings below it.
fn tabled(
    guide: &Guide,
    feats: &FeatNames,
    page_slug: &str,
    owner: Owner,
    selection: FeatureSelection,
    anchors: &[&str],
) -> Family {
    let page = page(guide, page_slug);
    let tables: Vec<_> = anchors.iter().map(|a| table(page, a)).collect();
    let cells = own_cells(&tables);
    let mut family = Family {
        owner,
        selection,
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for (anchor, cells) in anchors.iter().zip(&cells) {
        let described = section(page, anchor);
        let key = format!("{}/{anchor}", family.selection.key);
        let guide_for = |cell: &str| {
            page.guide(&anchor_for(described, cell).unwrap_or_else(|| anchor.to_string()))
        };
        let mut builder = RowBuilder {
            feats,
            features: &mut family.features,
        };
        let rows = builder.rows(cells, &key, &guide_for);
        family
            .selection
            .options
            .push(option(&key, &described.name, page.guide(anchor), rows));
    }
    family
}

pub(super) const SCHOOLS: [&str; 9] = [
    "universalist",
    "abjurer",
    "conjurer",
    "diviner",
    "enchanter",
    "illusionist",
    "invoker",
    "necromancer",
    "transmuter",
];

pub(super) fn wizard_schools(guide: &Guide, feats: &FeatNames) -> Family {
    let wizard = page(guide, "wizard");
    let mut family = tabled(
        guide,
        feats,
        "wizard",
        Owner::Class("wizard"),
        selection(
            "wizard/school",
            "School",
            wizard.guide("wizard"),
            &["wizard"],
        ),
        &SCHOOLS,
    );
    // "When you prepare spells, choose an energy type and gain resistance 5": a daily choice.
    let resistance = section_text(&wizard.root, "resistance");
    assert!(resistance.starts_with("When you prepare spells, choose an energy type"));
    for feature in &mut family.features {
        if feature.key.as_str() == "wizard/school/abjurer/resistance" {
            feature.daily_choice = true;
        }
    }
    family
}

pub(super) const ORDERS: [&str; 5] = [
    "order-of-the-cockatrice",
    "order-of-the-lion",
    "order-of-the-shroud",
    "order-of-the-star",
    "order-of-the-sword",
];

pub(super) fn cavalier_orders(guide: &Guide, feats: &FeatNames) -> Family {
    let cavalier = page(guide, "cavalier");
    let mut family = tabled(
        guide,
        feats,
        "cavalier",
        Owner::Class("cavalier"),
        selection(
            "cavalier/order",
            "Order",
            cavalier.guide("order"),
            &["cavalier"],
        ),
        &ORDERS,
    );
    // Braggart: "At 2nd level, the Cavalier receives Dazzling Display as a bonus feat."
    let braggart = section_text(&cavalier.root, "braggart");
    assert!(braggart.contains("receives Dazzling Display as a bonus feat"));
    let feat = feats.get("Dazzling Display").expect("Dazzling Display");
    let cockatrice = family
        .selection
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == "cavalier/order/order-of-the-cockatrice")
        .expect("Cockatrice");
    let row = cockatrice
        .levels
        .iter_mut()
        .find(|row| {
            row.features
                .iter()
                .any(|f| f.as_str().ends_with("/braggart"))
        })
        .expect("Braggart's row");
    row.feats.insert(feat);
    family
}

pub(super) const WAYS: [&str; 6] = [
    "fake-celebrity",
    "living-deity",
    "magic-inventor",
    "razmiri-infiltrator",
    "time-manipulator",
    "way-of-the-charlatan",
];

pub(super) fn magic_deceiver_ways(guide: &Guide, feats: &FeatNames) -> Family {
    let arcanist = page(guide, "arcanist");
    tabled(
        guide,
        feats,
        "arcanist",
        Owner::Archetype("arcanist", "arcanist/magic-deceiver"),
        selection(
            "arcanist/magic-deceiver/ways-of-the-deceiver",
            "Ways of the Deceiver",
            arcanist.guide("ways-of-the-deceiver"),
            &["arcanist"],
        ),
        &WAYS,
    )
}
