//! The last pass: references to keys the data doesn't define are dropped and reported.

use extract::references::drop_dangling;
use extract::report::Report;
use rules::{
    Feat, FeatCategory, FeatKey, Feature, FeatureKey, GameData, GuideRef, MythicPath,
    MythicPathKey, Prerequisite, Progression,
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
            ranks: vec![],
            switch: None,
        }],
    };
    let mut report = Report::default();

    drop_dangling(&mut data, &mut report);

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
