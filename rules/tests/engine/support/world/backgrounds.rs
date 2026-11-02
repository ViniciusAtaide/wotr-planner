//! The test world's backgrounds.

use std::collections::BTreeSet;

use rules::*;

use crate::support::builds::{feat, skill, weapon};

/// Five backgrounds after the user's list: skills only (Shackles Corsair), a feat (Martial Disciple's Improved
/// Unarmed Strike, whose invented Dex 13 it skips), a feat that a feat's prerequisite checks (Guard's light armour,
/// for Armor Focus (Light)), a feat for one value (Aldori Swordsman's dueling sword), and None.
pub(super) fn backgrounds() -> Vec<Background> {
    let background = |key: &str, group, skills: &[&str], feats: &[&str], note: &str| Background {
        key: BackgroundKey::new(key),
        name: key.into(),
        group,
        note: note.into(),
        class_skills: skills.iter().map(|s| skill(s)).collect(),
        feats: feats.iter().map(|f| feat(f)).collect(),
        feats_for: BTreeSet::new(),
    };
    vec![
        background(
            "shackles-corsair",
            Some(BackgroundGroup::Regional),
            &["mobility", "athletics"],
            &[],
            "Scimitar proficiency; +2 CMD against trip.",
        ),
        background(
            "martial-disciple",
            Some(BackgroundGroup::Oblate),
            &["mobility"],
            &["improved-unarmed-strike"],
            "",
        ),
        background(
            "guard",
            Some(BackgroundGroup::Warrior),
            &["persuasion"],
            &["armor-proficiency-light"],
            "Shortsword and light crossbow proficiency.",
        ),
        Background {
            feats_for: BTreeSet::from([FeatValue {
                feat: feat("exotic-weapon-proficiency"),
                value: weapon("dueling-sword"),
            }]),
            ..background(
                "aldori-swordsman",
                Some(BackgroundGroup::Regional),
                &["persuasion"],
                &[],
                "Buckler proficiency.",
            )
        },
        background("none", None, &[], &[], ""),
    ]
}
