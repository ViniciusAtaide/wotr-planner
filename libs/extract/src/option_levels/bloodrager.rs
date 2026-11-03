//! The bloodrager's bloodlines, and the Primalist's choices at its "Bloodline Ability or Rage Powers" cells.

use std::collections::BTreeMap;

use rules::{
    FeatListKey, FeatureSelection, FeatureSelectionKey, OptionLevelRow, Repeat, SelectionOption,
};

use crate::guide::Guide;
use crate::names::slug;

use super::reading::{FeatNames, anchor_for, name_list, own_cells, page, section, table};
use super::rows::{RowBuilder, empty_row, list, option, selection, set_list};
use super::sorcerer::{section_text, subtypes};
use super::{Family, Owner};

pub(super) const BLOODRAGER_TABLES: [&str; 10] = [
    "abyssal",
    "arcane",
    "celestial",
    "draconic",
    "elemental",
    "fey",
    "hag",
    "infernal",
    "serpentine",
    "undead",
];

pub(super) fn bloodrager_bloodlines(guide: &Guide, feats: &FeatNames) -> Family {
    let bloodrager = page(guide, "bloodrager");
    let anchors: Vec<String> = BLOODRAGER_TABLES
        .iter()
        .map(|b| format!("{b}-bloodrager"))
        .collect();
    let tables: Vec<_> = anchors.iter().map(|a| table(bloodrager, a)).collect();
    let cells = own_cells(&tables);
    let mut family = Family {
        owner: Owner::Class("bloodrager"),
        selection: selection(
            "bloodrager/bloodline",
            "Bloodline",
            page(guide, "character-creation").guide("bloodlines"),
            &["bloodrager"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for ((bloodline, anchor), cells) in BLOODRAGER_TABLES.iter().zip(&anchors).zip(&cells) {
        let described = section(bloodrager, anchor);
        let name = described
            .name
            .strip_suffix(" Bloodrager")
            .unwrap_or(&described.name)
            .to_owned();
        let under = format!("bloodrager/bloodline/{bloodline}");
        let guide_for = |cell: &str| {
            bloodrager.guide(&anchor_for(described, cell).unwrap_or_else(|| anchor.clone()))
        };
        let mut builder = RowBuilder {
            feats,
            features: &mut family.features,
        };
        let rows = builder.rows(cells, &under, &guide_for);
        // "Bonus Feats (Abyssal)": "The Bloodrager may choose from the following feats: A, B and C."
        let bonus = described
            .children
            .iter()
            .find(|child| child.name.starts_with("Bonus Feats"))
            .unwrap_or_else(|| panic!("{anchor} has no bonus feats"));
        let text = bonus.text();
        let names = text
            .split("following feats:")
            .nth(1)
            .unwrap_or_else(|| panic!("{}: {text}", bonus.anchor));
        let list_key = format!("{under}/feats");
        family.lists.push(list(
            &list_key,
            &format!("{name} Bloodline Feats"),
            bloodrager.guide(&bonus.anchor),
            false,
            feats.all(&name_list(names), &list_key),
        ));
        let options: Vec<(String, String)> = match subtypes(guide, bloodline) {
            Some(kinds) => kinds
                .iter()
                .map(|kind| {
                    (
                        format!("{bloodline}-{}", slug(kind)),
                        format!("{name} ({kind})"),
                    )
                })
                .collect(),
            None => vec![(bloodline.to_string(), name.clone())],
        };
        for (key, option_name) in options {
            let mut option = option(
                &format!("bloodrager/bloodline/{key}"),
                &option_name,
                bloodrager.guide(anchor),
                rows.clone(),
            );
            set_list(&mut option, 1, &FeatListKey::new(&list_key));
            family.selection.options.push(option);
        }
    }
    family
}

/// Primalist: "At 4th level and every four levels thereafter, a Primalist can choose to take his bloodline ability
/// or two Rage Powers." A choice at each "Bloodline Ability or Rage Powers" cell (`Takes::row_choices`): the
/// bloodline's row, or two rage powers that skip it.
pub(super) fn primal_choices(guide: &Guide) -> FeatureSelection {
    let text = section_text(&page(guide, "bloodrager").root, "primal-choices");
    assert!(text.contains("choose to take his bloodline ability or two Rage Powers"));
    let rage_power = FeatureSelectionKey::new("barbarian/rage-power");
    FeatureSelection {
        options: vec![
            option(
                "bloodrager/primalist/primal-choices/bloodline-ability",
                "Bloodline Ability",
                "bloodrager#primal-choices".into(),
                Vec::new(),
            ),
            SelectionOption {
                repeat: Repeat::Unlimited,
                replaces_row_of: Some(FeatureSelectionKey::new("bloodrager/bloodline")),
                ..option(
                    "bloodrager/primalist/primal-choices/rage-powers",
                    "Two Rage Powers",
                    "bloodrager#primal-choices".into(),
                    vec![OptionLevelRow {
                        selections: vec![rage_power.clone(), rage_power],
                        ..empty_row(1)
                    }],
                )
            },
        ],
        ..selection(
            PRIMAL_CHOICES,
            "Primal Choices",
            "bloodrager#primal-choices".into(),
            &["bloodrager"],
        )
    }
}

pub(super) const PRIMAL_CHOICES: &str = "bloodrager/primalist/primal-choices";
