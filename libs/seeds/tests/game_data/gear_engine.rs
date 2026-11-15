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
    // Game: TwoWeaponFightingGreater c126adbdf6ddd8245bda33694cd774e8 (Dex 19, BAB 11); TwoWeaponFightingImproved
    // 9af88f3ed8a017b45a6837eab7437629 (Dex 17, BAB 6); TwoWeaponFighting ac8aaf29054f5b74eb18f2af950e752d (Dex 15)
    let failures = engine.failures(&naked);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].choice, slot);
    assert_eq!(
        engine.issue_text(&failures[0].issue).as_deref(),
        Some("Needs Dex 19 (have 17)")
    );

    // With it the Build is possible, and the feat is reported as needing the belt.
    // Game: BeltOfDexterity2 b6af4c1834999e74497b41588d1071cd (Dex +2 Enhancement);
    // Kingmaker/Blueprints/Classes/Prerequisites/PrerequisiteStatValue.cs GetStatValue (PermanentValue) and
    // Kingmaker/EntitySystem/Stats/ModifiableValueAttributeStat.cs CalculatePermanentValue (an item's Enhancement counts),
    // WrathDecompiled 6ce4a6b
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
    // Game: BeltOfDexterity4 20539ffabe4365e419ca7413129ab78a (Dex +4); BeltOfStrength2 c354de364a714ee4dbe2cfee628df267
    // (Str +2 only)
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

/// A manual's Dexterity is an inherent bonus: it counts in every number but not toward an ability prerequisite, so the
/// manual doesn't do what the belt does.
// Game: ManualOfQuicknessOfActionPlus2 b8ea6d2f787c7004c9cab9f519f687f8 (uses ManualOfQuicknessOfActionPlus2_Feature
// 741d033afc9eebf4ca4a5ba014100ee6: AddStatBonus, Inherent, Dexterity +2); Kingmaker/UnitLogic/FactLogic/AddStatBonus.cs
// OnTurnOn (a modifier, not the base value), Kingmaker/EntitySystem/Stats/ModifiableValueAttributeStat.cs
// CalculatePermanentValue and Kingmaker/EntitySystem/Stats/ModifiableValue.cs Modifier.IsPermanent (only a racial
// modifier or an item's Enhancement), Kingmaker/Blueprints/Classes/Prerequisites/PrerequisiteStatValue.cs
// GetStatValue (PermanentValue), WrathDecompiled 6ce4a6b
#[test]
fn a_manuals_inherent_dexterity_doesnt_meet_greater_two_weapon_fightings_dexterity() {
    let engine = engine();
    let mut read = two_weapon_fighter();
    read.gear
        .used
        .insert(GearKey::new("manual-of-quickness-of-action-2"));
    assert_eq!(at(&read, 13).ability_scores.dexterity, 19);
    let failures = engine.failures(&read);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].choice, Choice::Slot(general_slot(13)));
    assert_eq!(
        engine.issue_text(&failures[0].issue).as_deref(),
        Some("Needs Dex 19 (have 17)")
    );
}

/// A fighter 1 with Intelligence 11 taking Combat Expertise (Intelligence 13) at level 1.
fn expert_at_int_11() -> Build {
    let mut build = build("fighter", 1);
    let mut scores = AbilityScores::default();
    *scores.get_mut(Ability::Intelligence) = 11;
    build.start.ability_scores = PointBuy::try_from(scores).expect("in budget");
    build
        .picks
        .insert(general_slot(1), feat_pick_plain("combat-expertise"));
    build
}

/// Draven's Hat's profane Intelligence +2 counts in the scores but not toward Combat Expertise's Intelligence 13; a
/// Headband of Vast Intelligence +2's enhancement does.
// Game: CombatExpertiseFeature 4c44724ffa8844f4d9bedb5bb27d144a (PrerequisiteStatValue Intelligence 13); DravensHatItem
// 0160c8eabe704db41b6309af0d26a8e0 (DravensHatEnchantment b3894ce33c4c0ce4abf0dbcff0d6729f: AddStatBonusEquipment,
// Profane, Intelligence +2); HeadbandOfIntelligence2 22184af355826444087506d7f2f8f0b1 (Intelligence2
// 0f157adfc02708441b02725a24e8eb71: AddStatBonusEquipment, Enhancement, Intelligence +2);
// Kingmaker/Designers/Mechanics/EquipmentEnchants/AddStatBonusEquipment.cs OnTurnOn (AddItemModifierUnique: the item is
// the modifier's source), Kingmaker/EntitySystem/Stats/ModifiableValue.cs Modifier.IsPermanent,
// Kingmaker/EntitySystem/Stats/ModifiableValueAttributeStat.cs CalculatePermanentValue and
// Kingmaker/Blueprints/Classes/Prerequisites/PrerequisiteStatValue.cs GetStatValue, WrathDecompiled 6ce4a6b
#[test]
fn dravens_hats_profane_intelligence_doesnt_meet_combat_expertise_and_an_enhancement_headband_does()
{
    let engine = engine();
    let wearing = |item: &str| {
        let mut build = expert_at_int_11();
        build
            .gear
            .worn
            .insert(GearPlace::Headwear, GearKey::new(item));
        build
    };

    let hat = wearing("dravens-hat");
    assert_eq!(at(&hat, 1).ability_scores.intelligence, 13);
    let failures = engine.failures(&hat);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].choice, Choice::Slot(general_slot(1)));
    assert_eq!(
        engine.issue_text(&failures[0].issue).as_deref(),
        Some("Needs Int 13 (have 11)")
    );

    assert!(engine.is_possible(&wearing("headband-of-vast-intelligence-2")));
}
