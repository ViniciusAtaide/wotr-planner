//! Backgrounds (ticket 34).

use std::collections::BTreeSet;

use rules::{
    Build, FeatKey, FeatSource, Issue, Parameter, Pick, Prerequisite, SkillKey, SlotId, WeaponKey,
};

use crate::support::*;

/// A wizard 1 (no class skill or proficiency a background gives) with a background.
fn wizard_with(background: Option<&str>) -> Build {
    let mut build = build("wizard", 1);
    build.start.background = background.map(rules::BackgroundKey::new);
    build
}

fn general_slot_1() -> SlotId {
    SlotId::Feat {
        level: 1,
        source: FeatSource::General,
        index: 0,
    }
}

/// The status of a feat in level 1's general slot.
fn general_feat_status(build: &Build, feat: &str) -> Result<(), Issue> {
    engine()
        .options(build, &general_slot_1())
        .unwrap()
        .into_iter()
        .find(|option| option.pick.key() == feat)
        .unwrap_or_else(|| panic!("{feat} isn't offered"))
        .status
}

#[test]
fn the_users_38_backgrounds_are_there_in_their_groups() {
    use rules::BackgroundGroup::*;
    assert_eq!(DATA.backgrounds.len(), 38);
    let in_group = |group| {
        DATA.backgrounds
            .iter()
            .filter(|b| b.group == Some(group))
            .count()
    };
    assert_eq!(
        [
            Craftsman,
            Noble,
            Oblate,
            Regional,
            Scholar,
            StreetUrchin,
            Wanderer,
            Warrior
        ]
        .map(in_group),
        [4, 3, 3, 15, 3, 3, 3, 4]
    );
    // No background stands for "none": a Build without one is the Build start's own state (user, 2026-11-02).
    // So every background has a group and makes at least one skill a class skill.
    assert!(
        DATA.backgrounds
            .iter()
            .all(|b| b.group.is_some() && !b.class_skills.is_empty())
    );
}

#[test]
fn only_the_decided_backgrounds_grant_feats() {
    let granted: Vec<(String, Vec<String>)> = DATA
        .backgrounds
        .iter()
        .filter(|b| !b.feats.is_empty() || !b.feats_for.is_empty())
        .map(|b| {
            let feats = b.feats.iter().map(|f| f.to_string()).chain(
                b.feats_for
                    .iter()
                    .map(|g| format!("{} ({})", g.feat, g.value.key())),
            );
            (b.key.to_string(), feats.collect())
        })
        .collect();
    let light = || vec!["armor-proficiency-light-armor".to_owned()];
    assert_eq!(
        granted,
        [
            (
                "aldori-swordsman".to_owned(),
                vec!["exotic-weapon-proficiency (dueling-sword)".to_owned()]
            ),
            ("arcane".to_owned(), vec!["scribe-scrolls".to_owned()]),
            ("bounty-hunter".to_owned(), light()),
            ("divine".to_owned(), vec!["brew-potions".to_owned()]),
            ("gladiator".to_owned(), light()),
            ("guard".to_owned(), light()),
            (
                "martial-disciple".to_owned(),
                vec!["improved-unarmed-strike".to_owned()]
            ),
            ("mercenary".to_owned(), light()),
        ]
    );
}

#[test]
fn a_shackles_corsair_holds_athletics_and_mobility_as_class_skills() {
    let skills = |build: &Build| -> BTreeSet<SkillKey> {
        engine().evaluate(build).at(1).unwrap().class_skills.clone()
    };
    let athletics = SkillKey::new("athletics");
    let mobility = SkillKey::new("mobility");
    let plain = skills(&wizard_with(None));
    assert!(!plain.contains(&athletics) && !plain.contains(&mobility));
    let corsair = skills(&wizard_with(Some("shackles-corsair")));
    assert!(corsair.contains(&athletics) && corsair.contains(&mobility));
    assert_eq!(
        corsair,
        plain.iter().cloned().chain([athletics, mobility]).collect()
    );
}

#[test]
fn an_aldori_swordsman_holds_exotic_weapon_proficiency_in_the_dueling_sword() {
    let sheet = engine().evaluate(&wizard_with(Some("aldori-swordsman")));
    let state = sheet.at(1).unwrap();
    let proficiency = FeatKey::new("exotic-weapon-proficiency");
    let weapon = |key: &str| Parameter::Weapon(WeaponKey::new(key));
    assert!(state.has_feat_for(&proficiency, &weapon("dueling-sword")));
    assert!(!state.has_feat_for(&proficiency, &weapon("bastard-sword")));
    assert!(engine().is_possible(&wizard_with(Some("aldori-swordsman"))));
}

#[test]
fn a_guard_can_take_armor_proficiency_medium_armor_at_level_1() {
    let medium = "armor-proficiency-medium-armor";
    assert!(general_feat_status(&wizard_with(None), medium).is_err());
    let mut guard = wizard_with(Some("guard"));
    assert_eq!(general_feat_status(&guard, medium), Ok(()));
    guard
        .picks
        .insert(general_slot_1(), Pick::feat(FeatKey::new(medium)));
    assert!(engine().is_possible(&guard));
}

#[test]
fn a_martial_disciples_improved_unarmed_strike_meets_stunning_fists_prerequisite() {
    let iua = Prerequisite::Feat(FeatKey::new("improved-unarmed-strike"));
    let unmet = general_feat_status(&wizard_with(None), "stunning-fist").unwrap_err();
    let Issue::Unmet(unmet) = unmet else {
        panic!("{unmet:?}")
    };
    assert!(conditions(&unmet).iter().any(|(needs, _)| **needs == iua));
    assert_eq!(
        general_feat_status(&wizard_with(Some("martial-disciple")), "stunning-fist"),
        Ok(())
    );
}
