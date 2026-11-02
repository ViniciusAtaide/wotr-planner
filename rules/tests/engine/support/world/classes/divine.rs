//! Druid, and Cleric with its domains and archetypes.

use std::collections::BTreeSet;

use rules::*;

use crate::support::builds::{class, feat, option, skill};
use crate::support::world::classes::{archetype, archetype_row, row};
use crate::support::world::selections::{
    levelled_option, levelled_selection, option_row, selection_option,
};
use crate::support::world::{guide, plain_feature};

/// Druid: Nature Bond at 1, an animal companion or a domain.
pub(in crate::support::world) fn druid() -> Class {
    let nature_bond = FeatureSelectionKey::new("druid/nature-bond");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 1 {
                r.selections.push(nature_bond.clone());
            }
            r
        })
        .collect();
    Class {
        key: class("druid"),
        name: "Druid".into(),
        summary: String::new(),
        guide: guide("druid"),
        skill_points: 4,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::new(),
        prerequisite: None,
        features: vec![],
        selections: vec![levelled_selection(
            "druid/nature-bond",
            &[],
            vec![
                selection_option("druid/nature-bond/animal-companion", None, None),
                selection_option("druid/nature-bond/domain", None, None),
            ],
            &[],
        )],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Cleric: 3/4 BAB, good Fortitude and Will, full divine spell-level access.
pub(in crate::support::world) fn cleric() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            r.divine_spell_level = l.div_ceil(2).min(9);
            if l == 1 {
                r.selections.push(FeatureSelectionKey::new("cleric/domain"));
            }
            if l == 3 {
                r.selections
                    .push(FeatureSelectionKey::new("cleric/revelation"));
            }
            r
        })
        .collect();
    Class {
        key: class("cleric"),
        name: "Cleric".into(),
        summary: String::new(),
        guide: guide("cleric"),
        skill_points: 2,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("persuasion")]),
        prerequisite: None,
        features: vec![
            plain_feature("cleric/domain/war/battle-rage", "Battle Rage"),
            plain_feature("cleric/domain/fire/fire-bolt", "Fire Bolt"),
        ],
        // An invented pool: a domain at 1, and at 3 a revelation from the held domain's own list. The War domain's
        // row 8 is a bonus Combat feat slot; the Weapon Mastery revelation's row 8 grants Weapon Specialization for
        // the weapon its pick named (its chosen feat, Weapon Focus). The Maneuver Mastery revelation picks a
        // maneuver, whose Trip feat comes at the revelation's level 7. Impossible Domain (a mythic ability) picks a
        // further domain, levelled like the domains held.
        selections: vec![
            levelled_selection(
                "cleric/domain",
                &["cleric"],
                vec![
                    SelectionOption {
                        pool: Some(FeatureSelectionKey::new("cleric/fire-revelations")),
                        ..levelled_option(
                            "cleric/domain/fire",
                            vec![option_row(1, &["cleric/domain/fire/fire-bolt"])],
                        )
                    },
                    SelectionOption {
                        levels: vec![
                            option_row(1, &["cleric/domain/war/battle-rage"]),
                            OptionLevelRow {
                                feat_slots: vec![FeatSlot::CombatFeat],
                                ..option_row(8, &[])
                            },
                        ],
                        ..selection_option("cleric/domain/war", None, None)
                    },
                    SelectionOption {
                        pool: Some(FeatureSelectionKey::new("cleric/water-revelations")),
                        ..selection_option("cleric/domain/water", None, None)
                    },
                ],
                &[],
            ),
            FeatureSelection {
                pool_of: Some(FeatureSelectionKey::new("cleric/domain")),
                ..levelled_selection("cleric/revelation", &["cleric"], vec![], &[])
            },
            FeatureSelection {
                options: vec![
                    selection_option("cleric/fire-revelations/burning-hands", None, None),
                    selection_option("cleric/fire-revelations/fire-breath", None, None),
                    levelled_option(
                        "cleric/fire-revelations/maneuver-mastery",
                        vec![OptionLevelRow {
                            selections: vec![FeatureSelectionKey::new("cleric/maneuver-mastery")],
                            ..option_row(1, &[])
                        }],
                    ),
                    SelectionOption {
                        chosen_feat: Some(feat("weapon-focus")),
                        levels: vec![OptionLevelRow {
                            chosen_feats: BTreeSet::from([feat("weapon-specialization")]),
                            ..option_row(8, &[])
                        }],
                        ..selection_option("cleric/fire-revelations/weapon-mastery", None, None)
                    },
                ],
                ..levelled_selection("cleric/fire-revelations", &[], vec![], &[])
            },
            FeatureSelection {
                options: vec![
                    selection_option("cleric/water-revelations/icy-skin", None, None),
                    selection_option("cleric/water-revelations/wave", None, None),
                ],
                ..levelled_selection("cleric/water-revelations", &[], vec![], &[])
            },
            FeatureSelection {
                level_from: Some(LevelFrom::GrantingOption),
                ..levelled_selection(
                    "cleric/maneuver-mastery",
                    &[],
                    vec![
                        levelled_option(
                            "cleric/maneuver-mastery/disarm",
                            vec![OptionLevelRow {
                                feats: BTreeSet::from([feat("dodge")]),
                                ..option_row(7, &[])
                            }],
                        ),
                        levelled_option(
                            "cleric/maneuver-mastery/trip",
                            vec![OptionLevelRow {
                                feats: BTreeSet::from([feat("combat-expertise")]),
                                ..option_row(7, &[])
                            }],
                        ),
                    ],
                    &[],
                )
            },
            // A false god's further domain runs on witch levels from the pick, before its own domain at 20.
            FeatureSelection {
                level_from: Some(LevelFrom::OtherOptions),
                level_also_from: Some(option("witch/false-god/way/deity")),
                ..levelled_selection("cleric/impossible-domain", &[], vec![], &CLERIC_DOMAINS)
            },
        ],
        feat_lists: vec![],
        levels,
        archetypes: vec![accursed(), separatist()],
    }
}

const CLERIC_DOMAINS: [&str; 3] = [
    "cleric/domain/fire",
    "cleric/domain/war",
    "cleric/domain/water",
];

/// Separatist: a second domain at 1, two levels behind.
fn separatist() -> Archetype {
    Archetype {
        selections: vec![FeatureSelection {
            levels_behind: 2,
            ..levelled_selection(
                "cleric/separatist/domain",
                &["cleric"],
                vec![],
                &CLERIC_DOMAINS,
            )
        }],
        levels: vec![ArchetypeLevelRow {
            selections: vec![
                FeatureSelectionKey::new("cleric/domain"),
                FeatureSelectionKey::new("cleric/separatist/domain"),
            ],
            ..archetype_row(1)
        }],
        ..archetype("cleric/separatist")
    }
}

/// Accursed (the Dual-Cursed Oracle, on the cleric): its revelations at 1 and 3 offer the domain's pool and a
/// revelation of its own. Its revelation's key sorts before `cleric/domain`, which it draws on at level 1.
fn accursed() -> Archetype {
    let revelation = FeatureSelectionKey::new("cleric/accursed/revelation");
    Archetype {
        selections: vec![FeatureSelection {
            pool_of: Some(FeatureSelectionKey::new("cleric/domain")),
            ..levelled_selection(
                revelation.as_str(),
                &["cleric"],
                vec![selection_option(
                    "cleric/accursed/revelation/misfortune",
                    None,
                    None,
                )],
                &[],
            )
        }],
        levels: vec![
            ArchetypeLevelRow {
                selections: vec![
                    revelation.clone(),
                    FeatureSelectionKey::new("cleric/domain"),
                ],
                ..archetype_row(1)
            },
            ArchetypeLevelRow {
                selections: vec![revelation],
                ..archetype_row(3)
            },
        ],
        ..archetype("cleric/accursed")
    }
}
