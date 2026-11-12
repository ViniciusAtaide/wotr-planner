//! Gear in the engine on the real data (ticket 166): the belt the Enforcer's write-up needs. Greater Two-Weapon
//! Fighting asks for Dexterity 19; with 17 of its own a character takes it under the Belt of Incredible Dexterity +2,
//! the feat is reported as needing that belt, and the belt can't come off while the feat is held.

use rules::{
    Ability, AbilityScores, Build, Choice, GearDependent, GearKey, GearNeed, GearPlace, GearSpot,
    Issue, Pick, PointBuy,
};

use crate::support::*;

const BELT: &str = "belt-of-incredible-dexterity-2";
const GTWF: &str = "greater-two-weapon-fighting";

/// A fighter 13 with Dexterity 17 and the three two-weapon feats in the general slots of levels 1, 7 and 13.
fn two_weapon_fighter() -> Build {
    let mut build = build("fighter", 13);
    let mut scores = AbilityScores::default();
    *scores.get_mut(Ability::Dexterity) = 17;
    build.start.ability_scores = PointBuy::try_from(scores).expect("13 points");
    for (level, feat) in [
        (1, "two-weapon-fighting"),
        (7, "improved-two-weapon-fighting"),
        (13, GTWF),
    ] {
        build
            .picks
            .insert(general_slot(level), Pick::feat(rules::FeatKey::new(feat)));
    }
    build
}

#[test]
fn greater_two_weapon_fighting_at_dex_17_needs_the_belt_of_incredible_dexterity_2() {
    let engine = engine();
    let naked = two_weapon_fighter();
    let slot = Choice::Slot(general_slot(13));

    // Without the belt only the last feat fails, on its Dexterity.
    let failures = engine.failures(&naked);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].choice, slot);
    assert_eq!(
        engine.issue_text(&failures[0].issue).as_deref(),
        Some("Needs Dex 19 (have 17)")
    );

    // With it the Build is possible, and the feat is reported as needing the belt.
    let mut geared = naked.clone();
    geared.gear.worn.insert(GearPlace::Belt, GearKey::new(BELT));
    assert!(engine.is_possible(&geared));
    let needs = GearNeed {
        all_of: vec![GearKey::new(BELT)],
        ..GearNeed::default()
    };
    assert_eq!(
        engine.gear_dependents(&geared),
        [GearDependent {
            choice: slot.clone(),
            needs: needs.clone(),
        }]
    );
    assert_eq!(
        engine.gear_note(&needs),
        "With Belt of Incredible Dexterity +2"
    );
    let link = rules::encode_build(&geared);
    assert_eq!(engine.open_link(&link), Ok(geared.clone()));

    // The belt can't be removed: its withdrawal isn't removable, and the belt picker locks "None", naming the feat.
    let the_belt = Choice::Gear(GearSpot::Worn(GearPlace::Belt));
    let (edit, removable) = engine.withdrawal(&geared, &the_belt).expect("the belt");
    assert!(!removable);
    assert!(!engine.allows(&geared, &edit));
    assert!(!engine.removable(&geared).contains(&the_belt));
    let options = engine.gear_options(&geared, GearPlace::Belt);
    let none = &options[0];
    assert_eq!((none.item.as_ref(), none.edit.clone()), (None, edit));
    assert_eq!(none.status, Err(Issue::BreaksLater));
    assert_eq!(none.needed_by, [slot]);
    assert_eq!(
        none.reason.as_deref(),
        Some("Greater Two-Weapon Fighting needs Belt of Incredible Dexterity +2")
    );
    // A belt that gives the Dexterity too is open; one that doesn't is locked.
    let status = |key: &str| {
        (options.iter())
            .find(|option| {
                option
                    .item
                    .as_ref()
                    .is_some_and(|item| item.as_str() == key)
            })
            .unwrap_or_else(|| panic!("no belt {key}"))
            .status
            .clone()
    };
    assert_eq!(status("belt-of-incredible-dexterity-4"), Ok(()));
    assert_eq!(status("belt-of-giant-strength-2"), Err(Issue::BreaksLater));
    // The feat cleared first, the belt comes off.
    assert!(engine.is_removable(&without(&geared, &general_slot(13)), &the_belt));
}
