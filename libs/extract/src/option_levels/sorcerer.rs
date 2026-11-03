//! The sorcerer's bloodlines, from the sorcerer page's tables and the character-creation bloodline sections, with
//! Strength of the Abyss's prose rows and the Arcane bloodline's school power.

use std::collections::BTreeMap;

use rules::{
    AbilityBonus, FeatListKey, Feature, FeatureKey, FeatureSelection, FeatureSelectionKey,
    OptionLevelRow,
};

use crate::guide::Guide;
use crate::html::{Page, Section};
use crate::names::slug;

use super::reading::{
    FeatNames, anchor_for, bloodline_feats, name_list, own_cells, page, pairs, section, table,
};
use super::rows::{RowBuilder, empty_row, list, option, selection, set_list};
use super::{Family, Owner};

/// The guide's bloodline tables on the sorcerer page, in its order.
pub(super) const SORCERER_TABLES: [&str; 9] = [
    "abyssal",
    "arcane",
    "celestial",
    "draconic",
    "elemental",
    "fey",
    "infernal",
    "serpentine",
    "undead",
];

/// "Every Draconic bloodline has certain characteristics in common" and "The element linked to each specific
/// bloodline is shown in the following table" (character-creation): one option per dragon and per element, rows
/// and list shared.
pub(super) fn subtypes(guide: &Guide, table: &str) -> Option<Vec<String>> {
    let creation = page(guide, "character-creation");
    match table {
        "draconic" => Some(
            pairs(section(creation, "draconic-bloodline"), "Dragon")
                .into_iter()
                .flat_map(|(dragons, _)| name_list(&dragons))
                .collect(),
        ),
        "elemental" => Some(
            pairs(section(creation, "elemental-bloodline"), "Elemental")
                .into_iter()
                .map(|(element, _)| element)
                .collect(),
        ),
        _ => None,
    }
}

pub(super) fn sorcerer_bloodlines(guide: &Guide, feats: &FeatNames) -> Family {
    let sorcerer = page(guide, "sorcerer");
    let creation = page(guide, "character-creation");
    let tables: Vec<_> = SORCERER_TABLES.iter().map(|b| table(sorcerer, b)).collect();
    let cells = own_cells(&tables);
    let mut family = Family {
        owner: Owner::Class("sorcerer"),
        selection: selection(
            "sorcerer/bloodline",
            "Bloodline",
            creation.guide("bloodlines"),
            &["sorcerer", "dragon-disciple"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for (bloodline, cells) in SORCERER_TABLES.iter().zip(&cells) {
        let described = section(creation, &format!("{bloodline}-bloodline"));
        let cells = &school_power(creation, bloodline, cells, &mut family.nested);
        let under = format!("sorcerer/bloodline/{bloodline}");
        let guide_for = |cell: &str| match anchor_for(described, cell) {
            Some(anchor) => creation.guide(&anchor),
            None => sorcerer.guide(bloodline),
        };
        let mut builder = RowBuilder {
            feats,
            features: &mut family.features,
        };
        let mut rows = builder.rows(cells, &under, &guide_for);
        if *bloodline == "arcane" {
            let school = FeatureSelectionKey::new("sorcerer/bloodline/arcane/school-power");
            match rows.iter_mut().find(|row| row.level == 15) {
                Some(row) => row.selections.push(school),
                None => rows.push(OptionLevelRow {
                    selections: vec![school],
                    ..empty_row(15)
                }),
            }
            rows.sort_by_key(|row| row.level);
        }
        if *bloodline == "abyssal" {
            strength_of_the_abyss(described, &mut rows, &mut family.features, &mut family.ties);
        }
        let list_key = format!("{under}/feats");
        let name = section(sorcerer, bloodline).name.clone();
        family.lists.push(list(
            &list_key,
            &format!("{name} Bloodline Feats"),
            creation.guide(&described.anchor),
            false,
            feats.all(&bloodline_feats(described), &list_key),
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
                &format!("sorcerer/bloodline/{key}"),
                &option_name,
                sorcerer.guide(bloodline),
                rows.clone(),
            );
            set_list(&mut option, 1, &FeatListKey::new(&list_key));
            family.selection.options.push(option);
        }
    }
    family
}

/// "At 9th level, you gain a +2 inherent bonus to Strength. This increases to +4 at level 13 and +6 at level 17."
/// One feature, +2 Strength, granted at 9, 13 and 17.
fn strength_of_the_abyss(
    described: &Section,
    rows: &mut Vec<OptionLevelRow>,
    features: &mut [Feature],
    ties: &mut BTreeMap<u8, u8>,
) {
    let text = section_text(described, "strength-of-the-abyss");
    assert!(
        text.contains("+2 inherent bonus to Strength")
            && text.contains("+4 at level 13 and +6 at level 17"),
        "Strength of the Abyss reads differently: {text}"
    );
    let key = FeatureKey::new("sorcerer/bloodline/abyssal/strength-of-the-abyss");
    assert!(
        rows.iter()
            .any(|row| row.level == 9 && row.features.contains(&key)),
        "no Strength of the Abyss at 9"
    );
    let feature = features
        .iter_mut()
        .find(|feature| feature.key == key)
        .expect("Strength of the Abyss");
    feature.ability_bonuses = vec![AbilityBonus::Fixed {
        ability: rules::Ability::Strength,
        amount: 2,
    }];
    for level in [13, 17] {
        match rows.iter_mut().find(|row| row.level == level) {
            Some(row) => row.features.push(key.clone()),
            None => rows.push(OptionLevelRow {
                features: vec![key.clone()],
                ..empty_row(level)
            }),
        }
        ties.insert(level, 9);
    }
    rows.sort_by_key(|row| row.level);
}

/// "At 15th level, pick one school of magic. The DC for any spells you cast from this school increases by +2"
/// (School Power, Arcane bloodline): a choice inside the option, so a selection its 15th-level row grants, one
/// option per school.
fn school_power(
    creation: &Page,
    bloodline: &str,
    cells: &BTreeMap<u8, Vec<String>>,
    nested: &mut Vec<FeatureSelection>,
) -> BTreeMap<u8, Vec<String>> {
    if bloodline != "arcane" {
        return cells.clone();
    }
    let text = section_text(&creation.root, "school-power");
    assert!(text.starts_with("At 15th level, pick one school of magic."));
    let key = "sorcerer/bloodline/arcane/school-power";
    let mut selection = selection(key, "School Power", creation.guide("school-power"), &[]);
    for school in rules::SpellSchool::ALL {
        let name = format!("{school:?}");
        selection.options.push(option(
            &format!("{key}/{}", slug(&name)),
            &name,
            creation.guide("school-power"),
            Vec::new(),
        ));
    }
    nested.push(selection);
    let mut cells = cells.clone();
    let at_15 = cells.get_mut(&15).expect("the Arcane bloodline at 15");
    let before = at_15.len();
    at_15.retain(|cell| cell != "School Power");
    assert_eq!(at_15.len() + 1, before, "School Power at 15");
    cells
}

pub(super) fn section_text(within: &Section, anchor: &str) -> String {
    within
        .find(anchor)
        .unwrap_or_else(|| panic!("no {anchor}"))
        .text()
}
