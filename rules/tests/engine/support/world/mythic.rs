//! The test world's mythic abilities and mythic feats.

use rules::*;

use crate::support::builds::{class, feat, feature, option};
use crate::support::world::feats::needs_feat;
use crate::support::world::guide;

pub(super) fn mythic_abilities() -> Vec<MythicAbility> {
    let ability = |key: &str, prerequisite| MythicAbility {
        key: MythicAbilityKey::new(key),
        name: key.into(),
        group: MythicAbilityGroup::Melee,
        summary: String::new(),
        guide: guide(key),
        prerequisite,
        features: vec![],
        selections: vec![],
    };
    vec![
        ability("mythic-charge", None),
        ability(
            "limitless-smite",
            Some(Prerequisite::ClassLevel {
                class: class("fighter"),
                level: 1,
            }),
        ),
        ability("unstoppable", None),
        ability("inspired-strike", None),
        ability("dimensional-retribution", None),
        ability("abundant-casting", None),
        // The sorcerer's 20th-level bloodline row, whatever the level.
        MythicAbility {
            features: vec![feature("sorcerer/bloodline-ascendance")],
            ..ability(
                "bloodline-ascendance",
                Some(Prerequisite::ClassLevel {
                    class: class("sorcerer"),
                    level: 1,
                }),
            )
        },
        // A further domain, levelled like the domains held (or a false god's way).
        MythicAbility {
            selections: vec![FeatureSelectionKey::new("cleric/impossible-domain")],
            ..ability(
                "impossible-domain",
                Some(Prerequisite::Any(vec![
                    Prerequisite::ClassLevel {
                        class: class("cleric"),
                        level: 1,
                    },
                    Prerequisite::SelectionOption(option("witch/false-god/way/deity")),
                ])),
            )
        },
        // A second bloodline, a full pick on the same class levels.
        MythicAbility {
            selections: vec![FeatureSelectionKey::new("sorcerer/bloodline")],
            ..ability(
                "second-bloodline",
                Some(Prerequisite::ClassLevel {
                    class: class("sorcerer"),
                    level: 1,
                }),
            )
        },
    ]
}

pub(super) fn mythic_feats() -> Vec<MythicFeat> {
    let mythic_feat = |key: &str, prerequisite| MythicFeat {
        key: MythicFeatKey::new(key),
        name: key.into(),
        group: MythicFeatGroup::Melee,
        summary: String::new(),
        guide: guide(key),
        parameter: None,
        repeat: Repeat::Once,
        removes_racial_penalty: false,
        prerequisite,
    };
    vec![
        mythic_feat("power-attack-mythic", Some(needs_feat("power-attack"))),
        mythic_feat("toughness-mythic", Some(needs_feat("toughness"))),
        mythic_feat("extra-mythic-ability", None),
        mythic_feat("improved-critical-mythic", None),
        mythic_feat("dodge-mythic", None),
        mythic_feat("cleave-mythic", Some(needs_feat("cleave"))),
        // "Select a school in which you have taken a Spell Focus."
        MythicFeat {
            parameter: Some(ParameterKind::SpellSchool),
            repeat: Repeat::PerValue,
            ..mythic_feat(
                "spell-focus-mythic",
                Some(Prerequisite::FeatForSameValue(feat("spell-focus"))),
            )
        },
        // "Select one school of magic": no prerequisite.
        MythicFeat {
            parameter: Some(ParameterKind::SpellSchool),
            repeat: Repeat::PerValue,
            ..mythic_feat("school-mastery", None)
        },
        MythicFeat {
            removes_racial_penalty: true,
            ..mythic_feat("destiny-beyond-birth", None)
        },
    ]
}
