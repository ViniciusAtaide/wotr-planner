//! The last pass: references to keys the data doesn't define are dropped and reported.

use extract::feat_lists::PendingLists;
use extract::references::{drop_dangling, drop_redundant_alternatives};
use extract::report::Report;
use rules::{
    BabProgression, Class, ClassKey, Feat, FeatCategory, FeatKey, Feature, FeatureKey,
    FeatureSelection, FeatureSelectionKey, GameData, GuideRef, MythicPath, MythicPathKey,
    Prerequisite, Progression, SaveProgression, SelectionOption, SelectionOptionKey,
};

fn feat(key: &str, prerequisite: Option<Prerequisite>) -> Feat {
    Feat {
        key: FeatKey::new(key),
        name: key.into(),
        category: FeatCategory::Miscellaneous,
        summary: String::new(),
        guide: GuideRef::new(format!("feats#{key}")),
        prerequisite,
    }
}

/// Keys carry no type prefix, so a reference must exist as its own kind: a feature with the right key doesn't
/// satisfy a feat reference.
#[test]
fn a_reference_must_exist_as_its_own_kind() {
    let mut data = GameData {
        progression: Progression {
            mythic_ascensions: vec![],
        },
        races: vec![],
        skills: vec![],
        feats: vec![
            feat("dodge", None),
            feat(
                "mobility",
                Some(Prerequisite::All(vec![
                    Prerequisite::Feat(FeatKey::new("dodge")),
                    // Only a feature is called `trick`.
                    Prerequisite::Feat(FeatKey::new("trick")),
                    Prerequisite::Feature {
                        feature: FeatureKey::new("trick"),
                        count: 1,
                    },
                ])),
            ),
        ],
        classes: vec![],
        prestige_classes: vec![],
        mythic_abilities: vec![],
        mythic_feats: vec![],
        mythic_paths: vec![MythicPath {
            key: MythicPathKey::new("trickster"),
            name: "Trickster".into(),
            summary: String::new(),
            guide: GuideRef::new("trickster"),
            features: vec![Feature {
                key: FeatureKey::new("trick"),
                name: "Trick".into(),
                summary: String::new(),
                guide: GuideRef::new("trickster#trick"),
            }],
            selections: vec![],
            feat_lists: vec![],
            ranks: vec![],
            switch: None,
        }],
    };
    let mut report = Report::default();

    drop_dangling(&mut data, &mut PendingLists::default(), &mut report);

    assert_eq!(
        data.feats[1].prerequisite,
        Some(Prerequisite::All(vec![
            Prerequisite::Feat(FeatKey::new("dodge")),
            Prerequisite::Feature {
                feature: FeatureKey::new("trick"),
                count: 1,
            },
        ]))
    );
    assert_eq!(report.len(), 1);
    assert!(report.markdown("2026-10-31").contains("feats#mobility"));
}

/// A rogue talent that grants Weapon Focus satisfies "Weapon Focus" through the feat it grants, so an `Any` that
/// names both keeps only the feat.
#[test]
fn an_option_beside_the_feat_it_grants_is_dropped() {
    let weapon_focus = || Prerequisite::Feat(FeatKey::new("weapon-focus"));
    let talent = |key: &str| Prerequisite::SelectionOption(SelectionOptionKey::new(key));
    let option = |key: &str, feat: Option<&str>| SelectionOption {
        key: SelectionOptionKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        feat: feat.map(FeatKey::new),
        prerequisite: None,
    };
    let rogue = Class {
        key: ClassKey::new("rogue"),
        name: "Rogue".into(),
        summary: String::new(),
        guide: GuideRef::new("rogue"),
        skill_points: 8,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        class_skills: Default::default(),
        prerequisite: None,
        features: vec![],
        selections: vec![FeatureSelection {
            key: FeatureSelectionKey::new("rogue/rogue-talents"),
            name: "Rogue Talents".into(),
            summary: String::new(),
            guide: GuideRef::new("rogue#rogue-talents"),
            options: vec![
                option("rogue/rogue-talents/weapon-focus", Some("weapon-focus")),
                option("rogue/rogue-talents/fast-stealth", None),
            ],
            shared_options: Default::default(),
        }],
        feat_lists: vec![],
        levels: vec![],
        archetypes: vec![],
    };
    let mut data = GameData {
        progression: Progression {
            mythic_ascensions: vec![],
        },
        races: vec![],
        skills: vec![],
        feats: vec![
            feat(
                "dazzling-display",
                Some(Prerequisite::Any(vec![
                    weapon_focus(),
                    talent("rogue/rogue-talents/weapon-focus"),
                ])),
            ),
            // "Any rogue talent": the talent is wanted as a talent here, not for its feat.
            feat(
                "extra-rogue-talent",
                Some(Prerequisite::Any(vec![
                    talent("rogue/rogue-talents/weapon-focus"),
                    talent("rogue/rogue-talents/fast-stealth"),
                ])),
            ),
        ],
        classes: vec![rogue],
        prestige_classes: vec![],
        mythic_abilities: vec![],
        mythic_feats: vec![],
        mythic_paths: vec![],
    };

    drop_redundant_alternatives(&mut data);

    assert_eq!(data.feats[0].prerequisite, Some(weapon_focus()));
    assert_eq!(
        data.feats[1].prerequisite,
        Some(Prerequisite::Any(vec![
            talent("rogue/rogue-talents/weapon-focus"),
            talent("rogue/rogue-talents/fast-stealth"),
        ]))
    );
}
