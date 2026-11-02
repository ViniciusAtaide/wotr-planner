//! Kineticist.

use std::collections::BTreeSet;

use rules::*;

use crate::support::builds::{class, feature};
use crate::support::world::classes::row;
use crate::support::world::selections::{levelled_selection, selection_option};
use crate::support::world::{guide, plain_feature};

/// Kineticist: an element at 1 (each up to twice, its defensive talent at element level 2), Expanded Element at 7
/// and 15, which can't give the same element twice and gives only its feat lists, and a wild talent at every even
/// level: Expanded Defense (its feature unlocks the expanded elements' defensive talents) or Skilled Kineticist.
pub(in crate::support::world) fn kineticist() -> Class {
    let element = FeatureSelectionKey::new("kineticist/element");
    let expanded = FeatureSelectionKey::new("kineticist/expanded-element");
    let talents = FeatureSelectionKey::new("kineticist/wild-talents");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 1 {
                r.selections.push(element.clone());
            }
            if l == 7 || l == 15 {
                r.selections.push(expanded.clone());
            }
            if l % 2 == 0 {
                r.selections.push(talents.clone());
            }
            r
        })
        .collect();
    let element_option = |key: &str| SelectionOption {
        repeat: Repeat::Times(2),
        levels: vec![OptionLevelRow {
            level: 2,
            features: vec![feature(&format!("{key}/defense"))],
            ..OptionLevelRow::default()
        }],
        ..selection_option(key, None, None)
    };
    let defense = "kineticist/wild-talents/expanded-defense/defensive-talents";
    let features = ["air", "fire", "water"]
        .iter()
        .map(|element| {
            let key = format!("kineticist/element/{element}/defense");
            plain_feature(&key, &key)
        })
        .chain([Feature {
            option_row: Some(OptionRow {
                selection: expanded.clone(),
                level: 2,
            }),
            ..plain_feature(defense, "Defensive Wild Talents")
        }])
        .collect();
    Class {
        key: class("kineticist"),
        name: "Kineticist".into(),
        summary: String::new(),
        guide: guide("kineticist"),
        skill_points: 2,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::new(),
        prerequisite: None,
        features,
        selections: vec![
            levelled_selection(
                element.as_str(),
                &["kineticist"],
                vec![
                    element_option("kineticist/element/air"),
                    element_option("kineticist/element/fire"),
                    element_option("kineticist/element/water"),
                ],
                &[],
            ),
            FeatureSelection {
                once_per_option: true,
                feat_lists_only: true,
                level_also_from: None,
                ..levelled_selection(
                    expanded.as_str(),
                    &["kineticist"],
                    vec![],
                    &[
                        "kineticist/element/air",
                        "kineticist/element/fire",
                        "kineticist/element/water",
                    ],
                )
            },
            levelled_selection(
                talents.as_str(),
                &["kineticist"],
                vec![
                    SelectionOption {
                        levels: vec![OptionLevelRow {
                            level: 1,
                            features: vec![feature(defense)],
                            ..OptionLevelRow::default()
                        }],
                        ..selection_option("kineticist/wild-talents/expanded-defense", None, None)
                    },
                    selection_option("kineticist/wild-talents/skilled-kineticist", None, None),
                ],
                &[],
            ),
        ],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}
