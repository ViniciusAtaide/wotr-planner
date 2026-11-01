//! Shared selection options: a selection the guide says draws on another's list shares its options (all, or the
//! ones it names) instead of copying them; a gate naming the defining class's feature or level also accepts the
//! sharing class's; and a pick the guide describes over two lists becomes its archetype's own selection, taking
//! over the rogue-talent slots it describes.

use std::collections::BTreeSet;

use extract::shared_options::{Gate, NewSelection, Options, Share, SlotRule, apply_shares};
use rules::{
    Archetype, ArchetypeKey, ArchetypeLevelRow, BabProgression, Class, ClassKey, ClassLevelRow,
    Feat, FeatCategory, FeatKey, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData,
    GuideRef, Prerequisite, Progression, SaveProgression, SelectionOption, SelectionOptionKey,
    SpellLevels,
};

fn option(key: &str, anchor: &str, prerequisite: Option<Prerequisite>) -> SelectionOption {
    SelectionOption {
        key: SelectionOptionKey::new(key),
        name: key.rsplit('/').next().unwrap().into(),
        summary: String::new(),
        guide: GuideRef::new(anchor),
        feats: BTreeSet::new(),
        chosen_feat: None,
        repeat: rules::Repeat::Once,
        prerequisite,
        levels: vec![],
        pool: None,
        replaces_row_of: None,
        parameter: None,
        values: Default::default(),
    }
}

fn selection(key: &str, options: Vec<SelectionOption>) -> FeatureSelection {
    FeatureSelection {
        key: FeatureSelectionKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        options,
        shared_options: BTreeSet::new(),
        option_levels: BTreeSet::new(),
        skipped_levels: BTreeSet::new(),
        feat_lists_only: false,
        only_without_option: false,
        pool_of: None,
    }
}

fn has(feature: &str) -> Prerequisite {
    Prerequisite::Feature {
        feature: FeatureKey::new(feature),
        count: 1,
    }
}

fn taken(option: &str) -> Prerequisite {
    Prerequisite::SelectionOption(SelectionOptionKey::new(option))
}

fn row(level: u8, features: &[&str], selections: &[&str]) -> ClassLevelRow {
    ClassLevelRow {
        level,
        arcane_spell_level: 0,
        divine_spell_level: 0,
        advances_arcane: false,
        advances_divine: false,
        features: features.iter().map(|f| FeatureKey::new(*f)).collect(),
        feats: BTreeSet::new(),
        feat_slots: vec![],
        selections: selections
            .iter()
            .map(|s| FeatureSelectionKey::new(*s))
            .collect(),
        feats_for: Default::default(),
    }
}

/// An archetype's full row, as the extractor has it before the rows are made sparse.
fn full_row(level: u8, features: &[&str], selections: &[&str]) -> ArchetypeLevelRow {
    let row = row(level, features, selections);
    ArchetypeLevelRow {
        level,
        spell_levels: Some(SpellLevels::default()),
        features: row.features,
        feats: row.feats,
        feat_slots: vec![],
        selections: row.selections,
        feats_for: Default::default(),
    }
}

fn class(
    key: &str,
    selections: Vec<FeatureSelection>,
    levels: Vec<ClassLevelRow>,
    archetypes: Vec<Archetype>,
) -> Class {
    Class {
        key: ClassKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        skill_points: 2,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: rules::Advancement::Each,
        counts_as: std::collections::BTreeSet::new(),
        class_skills: Default::default(),
        prerequisite: None,
        features: vec![],
        selections,
        feat_lists: vec![],
        levels,
        archetypes,
        spontaneous: false,
    }
}

fn archetype(key: &str, levels: Vec<ArchetypeLevelRow>) -> Archetype {
    Archetype {
        key: ArchetypeKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        prerequisite: None,
        skill_points: None,
        class_skills: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        spontaneous: None,
    }
}

fn data(classes: Vec<Class>, feats: Vec<Feat>) -> GameData {
    GameData {
        progression: Progression {
            mythic_ascensions: vec![],
        },
        races: vec![],
        skills: vec![],
        weapons: vec![],
        feats,
        classes,
        prestige_classes: vec![],
        mythic_abilities: vec![],
        mythic_feats: vec![],
        mythic_paths: vec![],
    }
}

fn keys(selection: &FeatureSelection) -> Vec<&str> {
    selection
        .shared_options
        .iter()
        .map(SelectionOptionKey::as_str)
        .collect()
}

/// Rogue talents (Fast Stealth; Opportunist, gated on the rogue's Advanced Talents) and slayer talents that copied
/// both and add Slayer Camouflage, which names the copied Fast Stealth.
fn rogue_and_slayer() -> GameData {
    let rogue = class(
        "rogue",
        vec![selection(
            "rogue/rogue-talents",
            vec![
                option(
                    "rogue/rogue-talents/fast-stealth",
                    "rogue#fast-stealth",
                    None,
                ),
                option(
                    "rogue/rogue-talents/opportunist",
                    "rogue#opportunist",
                    Some(has("rogue/advanced-talents")),
                ),
            ],
        )],
        vec![row(2, &[], &["rogue/rogue-talents"])],
        vec![],
    );
    let slayer = class(
        "slayer",
        vec![selection(
            "slayer/slayer-talents",
            vec![
                option(
                    "slayer/slayer-talents/fast-stealth",
                    "rogue#fast-stealth",
                    None,
                ),
                option(
                    "slayer/slayer-talents/opportunist",
                    "rogue#opportunist",
                    Some(has("slayer/advanced-talents")),
                ),
                option(
                    "slayer/slayer-talents/slayer-camouflage",
                    "slayer#slayer-camouflage",
                    Some(taken("slayer/slayer-talents/fast-stealth")),
                ),
            ],
        )],
        vec![row(2, &[], &["slayer/slayer-talents"])],
        vec![],
    );
    // A feat elsewhere that names a copy.
    let feat = Feat {
        key: FeatKey::new("extra-slayer-talent"),
        name: "Extra Slayer Talent".into(),
        category: FeatCategory::Class,
        summary: String::new(),
        guide: GuideRef::new("feats#extra-slayer-talent"),
        parameter: None,
        repeat: rules::Repeat::Once,
        prerequisite: Some(taken("slayer/slayer-talents/opportunist")),
    };
    data(vec![rogue, slayer], vec![feat])
}

const SLAYER: Share = Share {
    selection: "slayer/slayer-talents",
    new: None,
    sources: &[("rogue/rogue-talents", Options::All)],
    gates: &[(
        Gate::Feature("rogue/advanced-talents"),
        Gate::Feature("slayer/advanced-talents"),
    )],
    slots: None,
};

#[test]
fn a_selection_shares_the_source_options_and_its_copies_go() {
    let mut data = rogue_and_slayer();

    apply_shares(&mut data, &[SLAYER]);

    let slayer = &data.classes[1].selections[0];
    assert_eq!(
        keys(slayer),
        [
            "rogue/rogue-talents/fast-stealth",
            "rogue/rogue-talents/opportunist"
        ]
    );
    // Only its own option is left, and its reference to the copy points at the original.
    assert_eq!(slayer.options.len(), 1);
    assert_eq!(
        slayer.options[0].prerequisite,
        Some(taken("rogue/rogue-talents/fast-stealth"))
    );
    // So does every other reference to a copy.
    assert_eq!(
        data.feats[0].prerequisite,
        Some(taken("rogue/rogue-talents/opportunist"))
    );
}

#[test]
fn a_gate_on_the_defining_class_accepts_the_sharing_class_too() {
    let mut data = rogue_and_slayer();

    apply_shares(&mut data, &[SLAYER]);

    let opportunist = &data.classes[0].selections[0].options[1];
    assert_eq!(
        opportunist.prerequisite,
        Some(Prerequisite::Any(vec![
            has("rogue/advanced-talents"),
            has("slayer/advanced-talents")
        ]))
    );
    // An option without the gate is left alone.
    assert_eq!(data.classes[0].selections[0].options[0].prerequisite, None);
}

#[test]
fn only_the_named_options_are_shared() {
    let mut data = rogue_and_slayer();
    let share = Share {
        sources: &[(
            "rogue/rogue-talents",
            Options::Anchors(&["rogue#fast-stealth"]),
        )],
        gates: &[],
        ..SLAYER
    };

    apply_shares(&mut data, &[share]);

    let slayer = &data.classes[1].selections[0];
    assert_eq!(keys(slayer), ["rogue/rogue-talents/fast-stealth"]);
    // The opportunist copy isn't a copy of anything shared, so it stays.
    assert!(
        slayer
            .options
            .iter()
            .any(|o| o.key.as_str() == "slayer/slayer-talents/opportunist")
    );
}

#[test]
fn a_new_selection_takes_over_the_rows_its_rule_picks() {
    // A Sylvan Trickster-like archetype: "Fey Tricks" at 2 (beside a rogue talent slot) and 4 (on its own), and a
    // plain rogue talent at 6.
    let rogue = class(
        "rogue",
        vec![selection(
            "rogue/rogue-talents",
            vec![option(
                "rogue/rogue-talents/fast-stealth",
                "rogue#fast-stealth",
                None,
            )],
        )],
        vec![
            row(2, &[], &["rogue/rogue-talents"]),
            row(4, &[], &["rogue/rogue-talents"]),
            row(6, &[], &["rogue/rogue-talents"]),
        ],
        vec![archetype(
            "rogue/trickster",
            vec![
                full_row(2, &["rogue/trickster/fey-tricks"], &["rogue/rogue-talents"]),
                full_row(4, &["rogue/trickster/fey-tricks"], &[]),
                full_row(6, &[], &["rogue/rogue-talents"]),
            ],
        )],
    );
    let witch = class(
        "witch",
        vec![selection(
            "witch/hex",
            vec![option(
                "witch/hex/agony",
                "witch#agony",
                Some(has("witch/major-hexes")),
            )],
        )],
        vec![row(1, &[], &["witch/hex"])],
        vec![],
    );
    let mut data = data(vec![rogue, witch], vec![]);
    let share = Share {
        selection: "rogue/trickster/fey-tricks",
        new: Some(NewSelection {
            archetype: "rogue/trickster",
            name: "Fey Tricks",
            guide: "rogue#fey-tricks",
        }),
        sources: &[
            ("rogue/rogue-talents", Options::All),
            ("witch/hex", Options::All),
        ],
        gates: &[(
            Gate::Feature("witch/major-hexes"),
            Gate::Feature("rogue/trickster/major-hexes"),
        )],
        slots: Some(SlotRule::WithFeature("rogue/trickster/fey-tricks")),
    };

    apply_shares(&mut data, &[share]);

    let trickster = &data.classes[0].archetypes[0];
    let fey_tricks = &trickster.selections[0];
    assert_eq!(fey_tricks.key.as_str(), "rogue/trickster/fey-tricks");
    assert_eq!(fey_tricks.name, "Fey Tricks");
    assert_eq!(fey_tricks.guide, GuideRef::new("rogue#fey-tricks"));
    assert!(fey_tricks.options.is_empty());
    assert_eq!(
        keys(fey_tricks),
        ["rogue/rogue-talents/fast-stealth", "witch/hex/agony"]
    );
    let slots: Vec<Vec<&str>> = trickster
        .levels
        .iter()
        .map(|row| row.selections.iter().map(|s| s.as_str()).collect())
        .collect();
    assert_eq!(
        slots,
        [
            vec!["rogue/trickster/fey-tricks"],
            vec!["rogue/trickster/fey-tricks"],
            vec!["rogue/rogue-talents"],
        ]
    );
    // The class's own rows are untouched.
    assert_eq!(
        data.classes[0].levels[0].selections,
        [FeatureSelectionKey::new("rogue/rogue-talents")]
    );
    assert_eq!(
        data.classes[1].selections[0].options[0].prerequisite,
        Some(Prerequisite::Any(vec![
            has("rogue/trickster/major-hexes"),
            has("witch/major-hexes")
        ]))
    );
}

#[test]
fn a_rule_from_a_level_picks_the_rogue_talent_slots_from_there_and_the_feature_row() {
    let rogue = class(
        "rogue",
        vec![selection("rogue/rogue-talents", vec![])],
        vec![],
        vec![archetype(
            "rogue/chemist",
            vec![
                full_row(8, &[], &["rogue/rogue-talents"]),
                full_row(10, &["rogue/chemist/discovery"], &[]),
                full_row(11, &[], &[]),
                full_row(12, &[], &["rogue/rogue-talents"]),
            ],
        )],
    );
    let alchemist = class(
        "alchemist",
        vec![selection(
            "alchemist/discovery",
            vec![option(
                "alchemist/discovery/mummification",
                "alchemist#mummification",
                Some(Prerequisite::ClassLevel {
                    class: ClassKey::new("alchemist"),
                    level: 10,
                }),
            )],
        )],
        vec![],
        vec![],
    );
    let mut data = data(vec![rogue, alchemist], vec![]);
    let share = Share {
        selection: "rogue/chemist/discovery",
        new: Some(NewSelection {
            archetype: "rogue/chemist",
            name: "Discovery",
            guide: "rogue#discovery",
        }),
        sources: &[("alchemist/discovery", Options::All)],
        gates: &[(
            Gate::ClassLevel("alchemist", 10),
            Gate::Feature("rogue/chemist/discovery"),
        )],
        slots: Some(SlotRule::FromLevel {
            level: 10,
            feature: "rogue/chemist/discovery",
        }),
    };

    apply_shares(&mut data, &[share]);

    let slots: Vec<Vec<&str>> = data.classes[0].archetypes[0]
        .levels
        .iter()
        .map(|row| row.selections.iter().map(|s| s.as_str()).collect())
        .collect();
    assert_eq!(
        slots,
        [
            vec!["rogue/rogue-talents"],
            vec!["rogue/chemist/discovery"],
            vec![],
            vec!["rogue/chemist/discovery"],
        ]
    );
    // A `ClassLevel` gate gets an `Any` in canonical order (a class level sorts before a feature).
    assert_eq!(
        data.classes[1].selections[0].options[0].prerequisite,
        Some(Prerequisite::Any(vec![
            Prerequisite::ClassLevel {
                class: ClassKey::new("alchemist"),
                level: 10
            },
            has("rogue/chemist/discovery"),
        ]))
    );
}

#[test]
fn a_share_whose_selection_or_archetype_is_missing_changes_nothing() {
    let mut data = rogue_and_slayer();
    let before = data.clone();
    let share = Share {
        selection: "witch/elsewhere/tricks",
        new: Some(NewSelection {
            archetype: "witch/elsewhere",
            name: "Tricks",
            guide: "witch#tricks",
        }),
        ..SLAYER
    };

    apply_shares(&mut data, &[share]);

    assert_eq!(data, before);
}

#[test]
fn a_selections_shared_options_are_written_one_per_line_and_read_back() {
    let mut data = rogue_and_slayer();
    apply_shares(&mut data, &[SLAYER]);

    let files = extract::ron_out::files(&data);
    let (_, slayer) = files
        .iter()
        .find(|(path, _)| path.ends_with("classes/slayer.ron"))
        .unwrap();
    assert!(
        slayer.contains(concat!(
            "            shared_options: [\n",
            "                \"rogue/rogue-talents/fast-stealth\",\n",
            "                \"rogue/rogue-talents/opportunist\",\n",
            "            ],\n",
        )),
        "{slayer}"
    );
    // As `data` reads the files.
    let read: Class = ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(slayer)
        .unwrap();
    assert_eq!(read, data.classes[1]);
    // A selection that shares nothing writes no `shared_options`.
    let (_, rogue) = files
        .iter()
        .find(|(path, _)| path.ends_with("classes/rogue.ron"))
        .unwrap();
    assert!(!rogue.contains("shared_options"), "{rogue}");
}
