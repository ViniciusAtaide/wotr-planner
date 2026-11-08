//! Sorcerer, its bloodlines and archetypes.

use std::collections::BTreeSet;

use crate::*;

use crate::test_world::builds::{class, feat, feat_list, feature, skill};
use crate::test_world::world::classes::{archetype, archetype_row, row};
use crate::test_world::world::selections::{
    levelled_option, levelled_selection, option_row, selection_option, with_class_skills,
};
use crate::test_world::world::{bonus_feature, guide, list, plain_feature};

const BLOODLINES: [&str; 4] = [
    "sorcerer/bloodline/abyssal",
    "sorcerer/bloodline/arcane",
    "sorcerer/bloodline/draconic-gold",
    "sorcerer/bloodline/draconic-red",
];

/// Sorcerer: 1/2 BAB, good Will, spontaneous arcane spell-level access like the wizard's, a bloodline at 1
/// (levelled by sorcerer and Dragon Disciple levels) whose feat list the 7, 13 and 19 bonus slots pick from.
/// Four bloodlines: Abyssal (powers and the class skill Athletics at 1, powers at 3, 15 and 20, +2 Strength at 9,
/// 13 and 17, Toughness at 15), Arcane (a school at 15), and two Draconic ones (the class skill Persuasion at 1,
/// invented: the real one is Perception). Archetypes: Seeker (skips 3, 9 and 15), Crossblooded (a second
/// bloodline's feat list), Sage (Arcane only, its own 1st-level row) and an invented Primalist (at 3 and 9, the
/// bloodline row or two rogue talents).
pub(in crate::test_world::world) fn sorcerer() -> Class {
    let bloodline = FeatureSelectionKey::new("sorcerer/bloodline");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            r.arcane_spell_level = l.div_ceil(2).min(9);
            if l == 1 {
                r.selections.push(bloodline.clone());
            }
            if [7, 13, 19].contains(&l) {
                r.feat_slots.push(FeatSlot::OptionList {
                    selection: bloodline.clone(),
                    at_level: None,
                    label: None,
                });
            }
            r
        })
        .collect();
    let with_list = |mut row: OptionLevelRow, list: &str| {
        row.feat_list = Some(feat_list(list));
        row
    };
    // One Draconic bloodline at most, each with the Draconic arcana.
    let draconic = |key: &str| SelectionOption {
        group: Some(OptionGroup::Draconic),
        arcana: Some(feature("sorcerer/bloodline/draconic/arcana")),
        prerequisite: Some(Prerequisite::NoOtherOptionOfGroup(OptionGroup::Draconic)),
        ..levelled_option(
            key,
            vec![
                with_class_skills(
                    with_list(
                        option_row(
                            1,
                            &[
                                "sorcerer/bloodline/draconic/arcana",
                                "sorcerer/bloodline/draconic/claws",
                            ],
                        ),
                        "sorcerer/bloodline/draconic-feats",
                    ),
                    &["persuasion"],
                ),
                option_row(3, &["sorcerer/bloodline/draconic/resistances"]),
                option_row(20, &["sorcerer/bloodline/draconic/power-of-wyrms"]),
            ],
        )
    };
    let strength = "sorcerer/bloodline/abyssal/strength-of-the-abyss";
    let bloodlines = levelled_selection(
        "sorcerer/bloodline",
        &["sorcerer", "dragon-disciple"],
        vec![
            SelectionOption {
                arcana: Some(feature("sorcerer/bloodline/abyssal/arcana")),
                ..levelled_option(
                    "sorcerer/bloodline/abyssal",
                    vec![
                        with_class_skills(
                            with_list(
                                option_row(
                                    1,
                                    &[
                                        "sorcerer/bloodline/abyssal/arcana",
                                        "sorcerer/bloodline/abyssal/claws",
                                    ],
                                ),
                                "sorcerer/bloodline/abyssal-feats",
                            ),
                            &["athletics"],
                        ),
                        option_row(3, &["sorcerer/bloodline/abyssal/resistances"]),
                        option_row(9, &[strength]),
                        option_row(13, &[strength]),
                        OptionLevelRow {
                            feats: BTreeSet::from([feat("toughness")]),
                            ..option_row(15, &["sorcerer/bloodline/abyssal/added-summonings"])
                        },
                        option_row(17, &[strength]),
                        option_row(20, &["sorcerer/bloodline/abyssal/demonic-might"]),
                    ],
                )
            },
            levelled_option(
                "sorcerer/bloodline/arcane",
                vec![
                    with_list(
                        option_row(1, &["sorcerer/bloodline/arcane/arcane-bond"]),
                        "sorcerer/bloodline/arcane-feats",
                    ),
                    option_row(3, &["sorcerer/bloodline/arcane/combat-casting-adept"]),
                    OptionLevelRow {
                        selections: vec![FeatureSelectionKey::new("sorcerer/school-power")],
                        ..option_row(15, &[])
                    },
                    option_row(20, &["sorcerer/bloodline/arcane/apotheosis"]),
                ],
            ),
            draconic("sorcerer/bloodline/draconic-gold"),
            draconic("sorcerer/bloodline/draconic-red"),
        ],
        &[],
    );
    let school_power = FeatureSelection {
        options: vec![
            selection_option("sorcerer/school-power/evocation", None, None),
            selection_option("sorcerer/school-power/necromancy", None, None),
        ],
        ..levelled_selection("sorcerer/school-power", &[], vec![], &[])
    };
    let features = [
        "sorcerer/bloodline/abyssal/arcana",
        "sorcerer/bloodline/abyssal/claws",
        "sorcerer/bloodline/abyssal/resistances",
        "sorcerer/bloodline/abyssal/added-summonings",
        "sorcerer/bloodline/abyssal/demonic-might",
        "sorcerer/bloodline/arcane/arcane-bond",
        "sorcerer/bloodline/arcane/combat-casting-adept",
        "sorcerer/bloodline/arcane/apotheosis",
        "sorcerer/bloodline/draconic/arcana",
        "sorcerer/bloodline/draconic/claws",
        "sorcerer/bloodline/draconic/resistances",
        "sorcerer/bloodline/draconic/power-of-wyrms",
    ]
    .into_iter()
    .map(|key| plain_feature(key, key))
    .chain([
        bonus_feature(
            strength,
            vec![AbilityBonus::Fixed {
                ability: Ability::Strength,
                amount: 2,
            }],
        ),
        Feature {
            option_row: Some(OptionRow {
                selection: bloodline.clone(),
                level: 20,
            }),
            ..plain_feature("sorcerer/bloodline-ascendance", "Bloodline Ascendance")
        },
    ])
    .collect();

    // Seeker: the bloodline without its 3, 9 and 15 rows.
    let seeker = Archetype {
        selections: vec![FeatureSelection {
            skipped_levels: BTreeSet::from([3, 9, 15]),
            ..levelled_selection(
                "sorcerer/seeker/bloodline",
                &["sorcerer", "dragon-disciple"],
                vec![],
                &BLOODLINES,
            )
        }],
        levels: vec![ArchetypeLevelRow {
            selections: vec![FeatureSelectionKey::new("sorcerer/seeker/bloodline")],
            ..archetype_row(1)
        }],
        ..archetype("sorcerer/seeker")
    };
    // Crossblooded: a second bloodline that gives only its feat list and its arcana.
    let crossblooded = Archetype {
        selections: vec![FeatureSelection {
            feat_lists_only: true,
            ..levelled_selection(
                "sorcerer/crossblooded/second-bloodline",
                &["sorcerer", "dragon-disciple"],
                vec![],
                &BLOODLINES,
            )
        }],
        levels: vec![ArchetypeLevelRow {
            selections: vec![
                bloodline.clone(),
                FeatureSelectionKey::new("sorcerer/crossblooded/second-bloodline"),
            ],
            ..archetype_row(1)
        }],
        ..archetype("sorcerer/crossblooded")
    };
    // Sage: the Arcane bloodline, granted with no pick, with Arcane Bolt instead of its 1st-level row.
    let sage = Archetype {
        features: vec![plain_feature("sorcerer/sage/arcane-bolt", "Arcane Bolt")],
        selections: vec![FeatureSelection {
            skipped_levels: BTreeSet::from([1]),
            ..levelled_selection(
                "sorcerer/sage/bloodline",
                &["sorcerer", "dragon-disciple"],
                vec![],
                &["sorcerer/bloodline/arcane"],
            )
        }],
        levels: vec![ArchetypeLevelRow {
            features: vec![feature("sorcerer/sage/arcane-bolt")],
            selections: vec![FeatureSelectionKey::new("sorcerer/sage/bloodline")],
            ..archetype_row(1)
        }],
        ..archetype("sorcerer/sage")
    };
    // Primalist (invented on a sorcerer): at 3 and 9, the bloodline row or two rogue talents instead of it.
    let choice = FeatureSelectionKey::new("sorcerer/primalist/primal-choice");
    let primalist = Archetype {
        selections: vec![levelled_selection(
            "sorcerer/primalist/primal-choice",
            &["sorcerer"],
            vec![
                selection_option(
                    "sorcerer/primalist/primal-choice/bloodline-power",
                    None,
                    None,
                ),
                SelectionOption {
                    repeat: Repeat::Unlimited,
                    replaces_row_of: Some(bloodline.clone()),
                    levels: vec![OptionLevelRow {
                        selections: vec![
                            FeatureSelectionKey::new("rogue/rogue-talents"),
                            FeatureSelectionKey::new("rogue/rogue-talents"),
                        ],
                        ..option_row(1, &[])
                    }],
                    ..selection_option("sorcerer/primalist/primal-choice/talents", None, None)
                },
            ],
            &[],
        )],
        levels: [3, 9]
            .into_iter()
            .map(|level| ArchetypeLevelRow {
                selections: vec![choice.clone()],
                ..archetype_row(level)
            })
            .collect(),
        ..archetype("sorcerer/primalist")
    };
    Class {
        key: class("sorcerer"),
        name: "Sorcerer".into(),
        summary: String::new(),
        guide: guide("sorcerer"),
        skill_points: 2,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: true,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("knowledge-arcana")]),
        prerequisite: None,
        features,
        selections: vec![bloodlines, school_power],
        feat_lists: vec![
            list(
                "sorcerer/bloodline/abyssal-feats",
                false,
                &["cleave", "power-attack"],
            ),
            list(
                "sorcerer/bloodline/arcane-feats",
                false,
                &["arcane-strike", "spell-penetration"],
            ),
            list(
                "sorcerer/bloodline/draconic-feats",
                false,
                &["power-attack", "toughness"],
            ),
        ],
        levels,
        archetypes: vec![crossblooded, primalist, sage, seeker],
    }
}
