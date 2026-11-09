//! Fighter, Monk (with Zen Archer), Warpriest and Ranger.

use std::collections::BTreeSet;

use crate::*;

use crate::test_world::builds::{class, feat, feat_list, feature, skill, weapon};
use crate::test_world::world::classes::{archetype, archetype_row, row};
use crate::test_world::world::selections::{
    levelled_option, levelled_selection, option_row, selection_option,
};
use crate::test_world::world::{guide, list, plain_feature};

/// Fighter: full BAB, good Fortitude, Shield and Tower Shield Proficiency granted at 1, a Combat feat bonus slot
/// at 1 and every even level.
pub(in crate::test_world::world) fn fighter() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 1 {
                r.feats =
                    BTreeSet::from([feat("shield-proficiency"), feat("tower-shield-proficiency")]);
            }
            if l == 1 || l % 2 == 0 {
                r.feat_slots.push(FeatSlot::CombatFeat);
            }
            if l % 4 == 2 {
                r.features.push(feature("fighter/bravery"));
            }
            r
        })
        .collect();
    // Armiger: no Combat feat slots, no Bravery and no granted proficiencies, so a row at 1 and every even level
    // (empty ones take the fighter's grants away); an order selection at 1 and a fixed-list bonus slot at 2. Odd
    // levels from 3 are the fighter's.
    let armiger_levels = (1..=20)
        .filter(|l| l % 2 == 0 || *l == 1)
        .map(|l| {
            let mut r = archetype_row(l);
            if l == 1 {
                r.selections
                    .push(FeatureSelectionKey::new("fighter/armiger/order"));
            }
            if l == 2 {
                r.feat_slots.push(FeatSlot::OneOf {
                    list: feat_list("fighter/armiger/bonus-feats"),
                });
            }
            r
        })
        .collect();
    Class {
        casting_ability: None,
        key: class("fighter"),
        name: "Fighter".into(),
        summary: String::new(),
        guide: guide("fighter"),
        skill_points: 2,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("athletics")]),
        prerequisite: None,
        features: vec![plain_feature("fighter/bravery", "Bravery")],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![Archetype {
            casting_ability: None,
            key: ArchetypeKey::new("fighter/armiger"),
            name: "Armiger".into(),
            summary: String::new(),
            guide: guide("fighter#armiger"),
            prerequisite: None,
            skill_points: Some(4),
            class_skills: None,
            spontaneous: None,
            fort: None,
            reflex: None,
            will: None,
            features: vec![],
            selections: vec![FeatureSelection {
                key: FeatureSelectionKey::new("fighter/armiger/order"),
                name: "Order".into(),
                summary: String::new(),
                guide: guide("fighter#order"),
                shared_options: BTreeSet::new(),
                option_levels: BTreeSet::new(),
                skipped_levels: BTreeSet::new(),
                feat_lists_only: false,
                only_without_option: false,
                pool_of: None,
                levels_behind: 0,
                level_from: None,
                once_per_option: false,
                level_also_from: None,
                deity_domains: None,
                options: vec![
                    selection_option("fighter/armiger/order/cockatrice", None, None),
                    selection_option("fighter/armiger/order/dragon", None, None),
                ],
            }],
            feat_lists: vec![list(
                "fighter/armiger/bonus-feats",
                false,
                &["combat-expertise", "power-attack"],
            )],
            levels: armiger_levels,
        }],
    }
}

/// Monk: 3/4 BAB, good saves, Improved Unarmed Strike granted at 1, a bonus-list slot (Crane Style or Dodge,
/// prerequisites ignored) at 1 and at 2, 6, 10, 14 and 18.
pub(in crate::test_world::world) fn monk() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 1 {
                r.feats.insert(feat("improved-unarmed-strike"));
            }
            if l == 1 || l % 4 == 2 {
                r.feat_slots.push(FeatSlot::OneOf {
                    list: feat_list("monk/bonus-feats"),
                });
            }
            r
        })
        .collect();
    Class {
        casting_ability: None,
        key: class("monk"),
        name: "Monk".into(),
        summary: String::new(),
        guide: guide("monk"),
        skill_points: 4,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Good,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("mobility")]),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![list("monk/bonus-feats", true, &["crane-style", "dodge"])],
        levels,
        archetypes: vec![zen_archer()],
    }
}

/// Zen Archer: Way of the Bow at 2, a pick of its one option, Weapon Focus in a dagger or a longsword (the world's
/// stand-ins for the guide's bows), and Weapon Specialization in the same weapon at 6.
fn zen_archer() -> Archetype {
    let way = "monk/zen-archer/way-of-the-bow";
    let mut row_2 = archetype_row(2);
    row_2.feat_slots.push(FeatSlot::OneOf {
        list: feat_list("monk/bonus-feats"),
    });
    row_2.selections.push(FeatureSelectionKey::new(way));
    Archetype {
        selections: vec![levelled_selection(
            way,
            &["monk"],
            vec![SelectionOption {
                chosen_feat: Some(feat("weapon-focus")),
                values: BTreeSet::from([weapon("dagger"), weapon("longsword")]),
                ..levelled_option(
                    "monk/zen-archer/way-of-the-bow/weapon-focus",
                    vec![OptionLevelRow {
                        chosen_feats: BTreeSet::from([feat("weapon-specialization")]),
                        ..option_row(6, &[])
                    }],
                )
            }],
            &[],
        )],
        levels: vec![row_2],
        ..archetype("monk/zen-archer")
    }
}

/// Warpriest: a Weapon Focus of the player's choosing at 1 (a one-feat bonus list). Mantis Zealot: Weapon Focus
/// (Dueling Sword) at 1 and Weapon Specialization (Dueling Sword) at 4 instead.
pub(in crate::test_world::world) fn warpriest() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 1 {
                r.feat_slots.push(FeatSlot::OneOf {
                    list: feat_list("warpriest/weapon-focus"),
                });
            }
            if l == 2 {
                r.selections
                    .push(FeatureSelectionKey::new("warpriest/blessings"));
            }
            r
        })
        .collect();
    // The Good blessing needs a good alignment when the Champion of the Faith picks it as his chosen alignment.
    let champion = FeatureSelectionKey::new("warpriest/champion/chosen-alignment");
    let good = Prerequisite::Any(vec![
        Prerequisite::Not(Box::new(Prerequisite::PickedThrough(champion.clone()))),
        Prerequisite::Alignment(Alignment::LawfulGood),
        Prerequisite::Alignment(Alignment::NeutralGood),
        Prerequisite::Alignment(Alignment::ChaoticGood),
    ]);
    let mut champion_1 = archetype_row(1);
    champion_1.feat_slots.push(FeatSlot::OneOf {
        list: feat_list("warpriest/weapon-focus"),
    });
    champion_1.selections.push(champion.clone());
    let dueling_sword = |key: &str| FeatValue {
        feat: feat(key),
        value: weapon("dueling-sword"),
    };
    let mut zealot_1 = archetype_row(1);
    zealot_1.feats_for.insert(dueling_sword("weapon-focus"));
    let mut zealot_4 = archetype_row(4);
    zealot_4
        .feats_for
        .insert(dueling_sword("weapon-specialization"));
    Class {
        casting_ability: None,
        key: class("warpriest"),
        name: "Warpriest".into(),
        summary: String::new(),
        guide: guide("warpriest"),
        skill_points: 2,
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
            "warpriest/blessings",
            &["warpriest"],
            vec![
                selection_option("warpriest/blessings/good", None, Some(good)),
                selection_option("warpriest/blessings/war", None, None),
            ],
            &[],
        )],
        feat_lists: vec![list("warpriest/weapon-focus", true, &["weapon-focus"])],
        levels,
        archetypes: vec![
            Archetype {
                selections: vec![levelled_selection(
                    champion.as_str(),
                    &["warpriest"],
                    vec![],
                    &["warpriest/blessings/good", "warpriest/blessings/war"],
                )],
                levels: vec![champion_1],
                ..archetype("warpriest/champion")
            },
            Archetype {
                levels: vec![zealot_1, zealot_4],
                ..archetype("warpriest/mantis-zealot")
            },
        ],
    }
}

/// Ranger: full BAB, a combat style at 2 whose list grows at 6 (option levels are ranger levels), and a style
/// bonus slot at 2, 6 and 10 that picks from it, ignoring prerequisites.
pub(in crate::test_world::world) fn ranger() -> Class {
    let style = FeatureSelectionKey::new("ranger/combat-style");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 2 {
                r.selections.push(style.clone());
            }
            if [2, 6, 10].contains(&l) {
                r.feat_slots.push(FeatSlot::OptionList {
                    selection: style.clone(),
                    at_level: None,
                    label: None,
                });
            }
            r
        })
        .collect();
    let stage = |level, list: &str| OptionLevelRow {
        feat_list: Some(feat_list(list)),
        ..option_row(level, &[])
    };
    Class {
        casting_ability: None,
        key: class("ranger"),
        name: "Ranger".into(),
        summary: String::new(),
        guide: guide("ranger"),
        skill_points: 6,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("athletics")]),
        prerequisite: None,
        features: vec![],
        selections: vec![levelled_selection(
            "ranger/combat-style",
            &["ranger"],
            vec![
                levelled_option(
                    "ranger/combat-style/archery",
                    vec![
                        stage(2, "ranger/combat-style/archery-2"),
                        stage(6, "ranger/combat-style/archery-6"),
                    ],
                ),
                levelled_option(
                    "ranger/combat-style/crane",
                    vec![stage(2, "ranger/combat-style/crane-2")],
                ),
            ],
            &[],
        )],
        feat_lists: vec![
            list(
                "ranger/combat-style/archery-2",
                true,
                &["dodge", "toughness"],
            ),
            list(
                "ranger/combat-style/archery-6",
                true,
                &["dodge", "great-cleave", "toughness"],
            ),
            list(
                "ranger/combat-style/crane-2",
                true,
                &["crane-style", "crane-wing"],
            ),
        ],
        levels,
        archetypes: vec![],
    }
}
