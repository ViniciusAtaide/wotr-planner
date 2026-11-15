//! The gear items (ticket 165): `gear.ron`'s first stage, every item on the three saved item pages with an always-on
//! ability score bonus, and the user's generic stat belts and headbands. Items with no such bonus wait for the second
//! stage (ticket 169), so these rules are the first stage's, not the data check's.

use std::collections::BTreeMap;

use rules::{Ability, BonusType, GearItem, GearSlot, GearTarget};

use crate::support::*;

fn item(key: &str) -> &'static GearItem {
    DATA.gear
        .iter()
        .find(|item| item.key.as_str() == key)
        .unwrap_or_else(|| panic!("no gear item {key}"))
}

fn summary(key: &str) -> String {
    item(key).summary(&DATA.skills)
}

#[test]
fn every_item_has_a_positive_ability_score_bonus() {
    for item in &DATA.gear {
        assert!(
            item.bonuses.iter().any(|bonus| {
                matches!(bonus.target, GearTarget::Ability(_)) && bonus.amount > 0
            }),
            "{} gives no ability score bonus",
            item.name
        );
    }
}

#[test]
fn every_bonus_type_is_one_of_the_first_stages_ten() {
    let allowed = [
        BonusType::Enhancement,
        BonusType::Inherent,
        BonusType::Luck,
        BonusType::Profane,
        BonusType::Circumstance,
        BonusType::Competence,
        BonusType::Resistance,
        BonusType::Armour,
        BonusType::Alchemical,
        BonusType::Untyped,
    ];
    for item in &DATA.gear {
        for bonus in &item.bonuses {
            assert!(
                allowed.contains(&bonus.kind),
                "{}: {:?}",
                item.name,
                bonus.kind
            );
        }
    }
}

#[test]
fn guide_references_are_one_of_the_three_pages_and_match_the_slot() {
    let sections: BTreeMap<GearSlot, &[&str]> = BTreeMap::from([
        (GearSlot::Belt, &["belts"][..]),
        (GearSlot::Boots, &["boots"]),
        (GearSlot::Bracers, &["bracers"]),
        (GearSlot::Cloak, &["cloaks"]),
        (GearSlot::Glasses, &["glasses"]),
        (GearSlot::Gloves, &["gloves"]),
        (GearSlot::Headwear, &["headwear"]),
        (GearSlot::Necklace, &["necklaces"]),
        (GearSlot::Ring, &["rings"]),
        (GearSlot::Robe, &["robes"]),
        (GearSlot::Usable, &["equipable-and-consumable"]),
        (GearSlot::Armour, &["unique-armour"]),
        (GearSlot::Weapon, &["weapon-properties", "weapons"]),
    ]);
    for item in &DATA.gear {
        let Some(guide) = &item.guide else { continue };
        let (page, section) = guide.0.split_once('#').expect("page#section");
        assert!(
            ["items", "items-inevitable-excess", "items-lord-of-nothing"].contains(&page),
            "{}: {guide:?}",
            item.name
        );
        // Trusted Friend sits in the guide's "Equipable and Consumable" table, but its Skill Boosting Items row calls
        // it an amulet.
        let exception = item.key.as_str() == "trusted-friend"
            && item.slot == GearSlot::Necklace
            && section == "equipable-and-consumable";
        assert!(
            exception || sections[&item.slot].contains(&section),
            "{}: {section} for {:?}",
            item.name,
            item.slot
        );
    }
}

#[test]
fn exactly_the_users_twenty_two_generic_items_have_no_guide_reference() {
    // The user's items: each is the game's, with an Enhancement bonus of its size.
    let mut expected: Vec<String> = Vec::new();
    for name in [
        "Belt of Incredible Dexterity", // Game: BeltOfDexterity2 b6af4c1834999e74497b41588d1071cd, BeltOfDexterity4 20539ffabe4365e419ca7413129ab78a, BeltOfDexterity6 a214f971f937c2c46805211dcb9c1576
        "Belt of Giant Strength", // Game: BeltOfStrength2 c354de364a714ee4dbe2cfee628df267, BeltOfStrength4 ede812de6b31b194686595fab33fc3ed, BeltOfStrength6 d8062d43606e89e4bb3f0483ed3aba5d
        "Belt of the Mighty Constitution", // Game: BeltOfConstitution2 b9cc5d85d4a5032458ef4491f5fed251, BeltOfConstitution4 dbeb5ba83e9fd2046a31590636b8b36d, BeltOfConstitution6 f2c5555b4cce199488f9e6d3796a661f
        "Headband of Alluring Charisma", // Game: HeadbandOfCharisma2 a3e8e907908ca7a40ac6da78e70bf33d, HeadbandOfCharisma4 71b4c3f8976784d4abf46976b15eba4a, HeadbandOfCharisma6 66fa9c56ef410104a9a3d7fb3c78d140
        "Headband of Vast Intelligence", // Game: HeadbandOfIntelligence2 22184af355826444087506d7f2f8f0b1, HeadbandOfIntelligence4 ee15329357f8f974bacf02521d8d37fe, HeadbandOfIntelligence6 eee1c6ee3542ffd47a7804990da11c21
        "Headband of Inspired Wisdom", // Game: HeadbandOfWisdom2 3d33605336c62274483f382f58908f5b, HeadbandOfWisdom4 716bdfe5c57be9b43a958bb612c1ecd4, HeadbandOfWisdom6 bc931a618226e044caaebdf4ace1188a
    ] {
        expected.extend([2, 4, 6].map(|n| format!("{name} +{n}")));
    }
    for name in [
        "Belt of Physical Perfection", // Game: BeltOfPerfection2 7abdce2cab3e3dd43b9e6c41c1b44525, BeltOfPerfection4 7cda6252ed289b74fb667b4e9f69fcbd
        "Headband of Mental Perfection", // Game: HeadbandOfPerfection2 b9b49933be6ac7f4aa3c71c602227332, HeadbandOfPerfection4 df2aa195ecce07c4c8dcabb5ea8c696e
    ] {
        expected.extend([2, 4].map(|n| format!("{name} +{n}")));
    }
    expected.sort();
    let mut without: Vec<String> = DATA
        .gear
        .iter()
        .filter(|item| item.guide.is_none())
        .map(|item| item.name.clone())
        .collect();
    without.sort();
    assert_eq!(without, expected);
    for item in DATA.gear.iter().filter(|item| item.guide.is_none()) {
        assert!(item.other_effects.is_empty(), "{}", item.name);
        assert!(item.always_on.is_empty(), "{}", item.name);
        assert!(
            item.bonuses
                .iter()
                .all(|bonus| bonus.kind == BonusType::Enhancement && bonus.amount % 2 == 0),
            "{}",
            item.name
        );
    }
    // The guide lists the +6 three-score items itself.
    for key in [
        "belt-of-physical-perfection-6",
        "headband-of-mental-perfection-6",
    ] {
        assert!(item(key).guide.is_some(), "{key}");
    }
}

/// The game has a Headband of Mental Perfection +8 beside the belt +8 the data holds: +8 enhancement to Intelligence,
/// Wisdom and Charisma (the user approved the game's +8 items).
// Game: HeadbandOfPerfection8 f94d69b19fe07ca4a8ade7993a1a7510; BeltOfPerfection8 dc2f2613634c2f947b306fa3379232c2
#[test]
#[ignore = "waits on ticket 288: the game gives a Headband of Mental Perfection +8"]
fn the_game_has_a_headband_of_mental_perfection_8() {
    let headband = DATA
        .gear
        .iter()
        .find(|item| {
            item.blueprint
                .as_ref()
                .is_some_and(|guid| guid.as_str() == "f94d69b19fe07ca4a8ade7993a1a7510")
        })
        .expect("an item for HeadbandOfPerfection8");
    assert_eq!(headband.slot, GearSlot::Headwear);
    let mut abilities: Vec<Ability> = headband
        .bonuses
        .iter()
        .map(|bonus| {
            assert_eq!((bonus.amount, bonus.kind), (8, BonusType::Enhancement));
            let GearTarget::Ability(ability) = bonus.target else {
                panic!("{:?}", bonus.target)
            };
            ability
        })
        .collect();
    abilities.sort();
    assert_eq!(
        abilities,
        [Ability::Intelligence, Ability::Wisdom, Ability::Charisma]
    );
}

#[test]
fn the_items_count_per_slot_matches_the_extraction_report() {
    let mut counts: BTreeMap<GearSlot, usize> = BTreeMap::new();
    for item in DATA.gear.iter().filter(|item| item.guide.is_some()) {
        *counts.entry(item.slot).or_default() += 1;
    }
    assert_eq!(
        counts,
        BTreeMap::from([
            (GearSlot::Belt, 17),
            (GearSlot::Boots, 2),
            (GearSlot::Bracers, 1),
            (GearSlot::Cloak, 1),
            (GearSlot::Glasses, 1),
            (GearSlot::Gloves, 3),
            (GearSlot::Headwear, 21),
            (GearSlot::Necklace, 3),
            (GearSlot::Ring, 3),
            (GearSlot::Robe, 3),
            (GearSlot::Armour, 4),
            (GearSlot::Weapon, 2),
            (GearSlot::Usable, 9),
        ])
    );
    assert_eq!(DATA.gear.len(), 92);
}

#[test]
fn the_manuals_tomes_and_all_score_consumables_are_usable_inherent_bonuses() {
    let usable: Vec<&GearItem> = DATA
        .gear
        .iter()
        .filter(|item| item.slot == GearSlot::Usable)
        .collect();
    assert_eq!(usable.len(), 9);
    let mut scores = Vec::new();
    let mut all_scores = Vec::new();
    for item in usable {
        assert!(item.other_effects.is_empty(), "{}", item.name);
        assert!(item.always_on.is_empty(), "{}", item.name);
        // Game: ManualOfGainfulExercisePlus2_Feature e5ed74f06e40f994da1727662f9c9169 (as the other +2 manuals and
        // tomes), PotionOfHumanityFeature 35ba5e8c91c34becbdcdbe0bf529562e, TomeOfPerfection_Feature
        // 968e61c643344565b67d2a7f5e101132: Inherent. The Elixir's are Alchemical:
        // `the_elixir_of_inconceivable_transmutations_gives_alchemical_bonuses`.
        if item.key.as_str() != ELIXIR {
            assert!(
                item.bonuses
                    .iter()
                    .all(|bonus| bonus.kind == BonusType::Inherent),
                "{}",
                item.name
            );
        }
        match item.bonuses.as_slice() {
            [bonus] => {
                assert_eq!(bonus.amount, 2, "{}", item.name);
                let GearTarget::Ability(ability) = bonus.target else {
                    panic!("{}", item.name)
                };
                scores.push(ability);
            }
            bonuses => {
                assert_eq!(bonuses.len(), 6, "{}", item.name);
                assert!(bonuses.iter().all(|b| b.amount == bonuses[0].amount));
                all_scores.push((item.name.as_str(), bonuses[0].amount));
            }
        }
    }
    scores.sort();
    assert_eq!(
        scores,
        [
            Ability::Strength,
            Ability::Dexterity,
            Ability::Constitution,
            Ability::Intelligence,
            Ability::Wisdom,
            Ability::Charisma,
        ]
    );
    all_scores.sort();
    // Game: ElixirMasterpieceFeature e52dae8a5a68b5040a0c3c8761a412d7 (+2), PotionOfHumanityFeature
    // 35ba5e8c91c34becbdcdbe0bf529562e (+1), TomeOfPerfection_Feature 968e61c643344565b67d2a7f5e101132 (+4)
    assert_eq!(
        all_scores,
        [
            (
                "Elixir of Inconceivable Transmutations of Body, and Soul, and Also Mind",
                2
            ),
            ("Potion of Humanity", 1),
            ("Tome of Ancient Perfection", 4),
        ]
    );
}

/// The Elixir of Inconceivable Transmutations' +2 to every score is Alchemical, not Inherent.
// Game: ElixirMasterpieceFeature e52dae8a5a68b5040a0c3c8761a412d7 (Alchemical)
#[test]
#[ignore = "waits on ticket 288: the game gives the Elixir of Inconceivable Transmutations Alchemical bonuses"]
fn the_elixir_of_inconceivable_transmutations_gives_alchemical_bonuses() {
    assert!(
        item(ELIXIR)
            .bonuses
            .iter()
            .all(|bonus| bonus.kind == BonusType::Alchemical)
    );
}

/// The usable item whose bonuses the game makes Alchemical.
const ELIXIR: &str = "elixir-of-inconceivable-transmutations-of-body-and-soul-and-also-mind";

#[test]
fn trusted_friend_is_a_necklace() {
    // The slot is the game's otherwise: `trusted_friend_is_a_usable_toggle`.
    // Game: TokenOfHonorBuff ab092ad237384b46b2fb192e6305f673 (Cha +2 and Persuasion +5 circumstance)
    assert_eq!(
        summary("trusted-friend"),
        "Cha +2 circumstance; Persuasion +5 circumstance"
    );
}

/// Trusted Friend is a usable item (a toggle), not a necklace.
// Game: TokenOfHonor 905ec00744cd4627b0e00f187e215d4b (a usable item whose toggle gives TokenOfHonorBuff
// ab092ad237384b46b2fb192e6305f673)
#[test]
#[ignore = "waits on ticket 288: the game gives Trusted Friend the usable slot"]
fn trusted_friend_is_a_usable_toggle() {
    assert_eq!(item("trusted-friend").slot, GearSlot::Usable);
}

#[test]
fn an_armour_and_a_weapon_name_their_own_enhancement_and_base_type() {
    // The Half-Plate of Vigor's is the game's otherwise: `the_half_plate_of_vigor_is_a_2_half_plate_with_con_2_enhancement`.
    for (key, line) in [
        ("snakeskin", "A +4 leather armour"), // Game: DLC3_DemonhideLeatherArmorItem f125e2f29ffd44e7acfb7619586a7e24
        ("life-infuser", "A +5 breastplate"), // Game: LifeInfuser e5d3c2148c854b5ba5d56390f8fcc72c
        ("ustlavic-pride", "A +3 full plate"), // Game: FullplateOfUstalavianOfficerArmorItem 59a33c8783ad4de8b547022ec274c930
        ("sin-mages-staff", "A +5 quarterstaff"), // Game: SinmageStaff 19f9506b61ed4190bffc66eb0cad39de
        ("dirge-for-sarkoris", "A +2 keen club"), // Game: SarkorisDirgeItem cffc4037c224475ca2a0c0b6bfda8688
    ] {
        // Always on, so in the summary and not behind the detail (ticket 176).
        assert!(item(key).always_on.iter().any(|l| l == line), "{key}");
        assert!(summary(key).contains(line), "{key}");
    }
}

/// The Half-Plate of Vigor is a +2 half-plate, and its Constitution bonus is an enhancement bonus.
// Game: HalfplateOfVigorItem 053873b9eca18854696dfc0e49d260fe (+2; Con +2 Enhancement)
#[test]
#[ignore = "waits on ticket 288: the game gives the Half-Plate of Vigor +2 and Con +2 enhancement"]
fn the_half_plate_of_vigor_is_a_2_half_plate_with_con_2_enhancement() {
    assert_eq!(
        summary("half-plate-of-vigor"),
        "Con +2 enhancement; A +2 half-plate"
    );
}

#[test]
fn the_venerated_champion_gives_con_6_enhancement() {
    // Its Fortitude is the game's otherwise: `the_venerated_champions_fortitude_bonus_is_always_on`.
    // Game: BeltOfVeneratedChampionItem 87c5f3e1b5a01d0448d550b727f45cb4 (Con +6 Enhancement)
    assert!(summary("belt-of-the-venerated-champion").starts_with("Con +6 enhancement"));
}

/// The Venerated Champion's belt gives an always-on +2 untyped bonus to Fortitude.
// Game: BeltOfVeneratedChampionFeature 49fb12e9d6cac0041b6de3507049e7d0 (Fortitude +2, untyped, always on)
#[test]
#[ignore = "waits on ticket 288: the game gives the Venerated Champion's belt an always-on Fortitude +2"]
fn the_venerated_champions_fortitude_bonus_is_always_on() {
    assert!(
        item("belt-of-the-venerated-champion")
            .always_on
            .iter()
            .any(|line| line.contains("Fortitude +2")),
        "{:?}",
        item("belt-of-the-venerated-champion").always_on
    );
}

#[test]
fn a_single_score_belt_matches_its_page_line() {
    // items#belts: "+6 enhancement bonus to Dexterity / On crit: the wearer gains a +2 circumstance bonus to attack
    // and damage rolls for 3 rounds".
    // Game: AssailantsBeltItem 8d48d164b7a729b459db96ab7d5cf55a (Dex +6 Enhancement; AssailantsBeltFeature)
    let belt = item("assailants-belt");
    assert_eq!(belt.slot, GearSlot::Belt);
    assert_eq!(belt.guide.as_ref().unwrap().0, "items#belts");
    assert_eq!(summary("assailants-belt"), "Dex +6 enhancement");
    assert_eq!(belt.other_effects.len(), 1);
}

#[test]
fn a_three_score_belt_matches_its_page_line() {
    // items#belts: "+6 enhancement bonus to Strength, Dexterity and Constitution".
    // Game: BeltOfPerfection6 2b400628f960a8a43bc2f085d84788cf (Str, Dex, Con +6 Enhancement)
    let belt = item("belt-of-physical-perfection-6");
    assert_eq!(belt.guide.as_ref().unwrap().0, "items#belts");
    assert_eq!(
        summary("belt-of-physical-perfection-6"),
        "Str, Dex, Con +6 enhancement"
    );
    assert!(belt.other_effects.is_empty());
}

#[test]
fn an_item_with_other_effects_matches_its_page_line() {
    // items#cloaks: "+7 resistance bonus to saves / +5 circumstance bonus to Persuasion (Bluff) / +2 circumstance
    // bonus to Charisma": the use of Persuasion for bluffing is an always-on line (ticket 176), not a bonus.
    // Game: GrandFestivalCloak 12836afa5ce54260895c4f54d8a3bb4d (CheckBluff for the Persuasion bonus)
    assert_eq!(
        summary("grand-festival-cloak"),
        "Cha +2 circumstance; Saves +7 resistance; Persuasion (Bluff) +5 circumstance"
    );
    assert_eq!(
        item("grand-festival-cloak").always_on,
        ["Persuasion (Bluff) +5 circumstance"]
    );
    assert!(item("grand-festival-cloak").other_effects.is_empty());
    // Odd wordings and penalties.
    assert_eq!(summary("timely-assistance"), "Con, Wis +4 enhancement"); // Game: TimelyAssistanceItem b76383fb806d61b41bfe93f0651e3964
    // Game: LifeInfuser e5d3c2148c854b5ba5d56390f8fcc72c (Str, Con +8, descriptor Armor; 50% fortification)
    assert_eq!(
        summary("life-infuser"),
        "Str, Con +8 armour; A +5 breastplate; Moderate fortification"
    );
    // The Half-Plate of Vigor's: `the_half_plate_of_vigor_is_a_2_half_plate_with_con_2_enhancement`.
    // Game: SinmageStaffFeature a37d1e737e21433392b8292e2810906c (all three untyped, UntypedStackable)
    assert_eq!(
        summary("sin-mages-staff"),
        "Int +6; Spell DC +1; Caster level +3; A +5 quarterstaff"
    );
    // Game: BookwormsHeadbandItem 866f59db467f8414cbd909df9b10361f; BookwormsHeadbandFeature e536dc0b5dd89e14db611b0f03272a67
    assert_eq!(
        summary("bookworms-headband"),
        "Int +2 enhancement; Cha -2; Persuasion -3; Immune to compulsion effects"
    );
    // Game: GlovesOfTheAmbassadorItem beb4001a6c9da474da29a8c5ece30572
    assert_eq!(
        summary("gloves-of-the-ambassador"),
        "Cha +2 enhancement; Knowledge (World), Persuasion +5 competence"
    );
    // Game: GogglesOfFeriociousPactItem 76d219c4b02f2d64bbf15363565647c3
    assert_eq!(
        summary("goggles-of-ferocious-pact"),
        "Wis +2 enhancement; Fortitude -2; Will +3; Persuasion -5; AC -2; Damage +4"
    );
}

#[test]
fn mangling_frenzy_gives_str_dex_con_4_enhancement() {
    // items#belts: the three scores, a damage reduction of 3 with no condition, and +4d6 slashing damage on a
    // critical hit while raging. Ticket 176: what is always on is in the summary, what needs a trigger is in the detail.
    // Where the damage reduction applies is the game's otherwise: `mangling_frenzys_damage_reduction_is_while_raging`.
    // Game: ManglingFrenzyItem 5d855652aad04a9cad2fb1e70735ac7e (Str, Dex, Con +4 Enhancement)
    assert!(summary("mangling-frenzy").starts_with("Str, Dex, Con +4 enhancement"));
}

/// Mangling Frenzy's DR 3/- comes with its critical-hit dice in the buff it gives while raging, not always on.
// Game: ManglingFrenzyFeature 29e2f51e6dd7427099b015de88718990; ManglingFrenzyBuff 1581c5ceea24418cadc9f26ce4d391a9 (DR 3/-
// and the critical-hit damage, while raging)
#[test]
#[ignore = "waits on ticket 288: the game gives Mangling Frenzy its DR 3/- only while raging"]
fn mangling_frenzys_damage_reduction_is_while_raging() {
    assert_eq!(summary("mangling-frenzy"), "Str, Dex, Con +4 enhancement");
    assert!(item("mangling-frenzy").always_on.is_empty());
}

#[test]
fn an_item_keeps_its_conditional_lines_and_shows_its_always_on_ones() {
    // Snakeskin: all three lines apply while it is worn; nothing is left for a detail.
    // Game: DLC3_DemonhideLeatherArmorItem f125e2f29ffd44e7acfb7619586a7e24; DLC3_DemonhideLeatherArmorEnchantment
    // f8494f446b324cfcb5d8a4be3b859f2d
    assert_eq!(
        summary("snakeskin"),
        "Dex +4 profane; A +4 leather armour; +20 feet base speed; Immune to acid"
    );
    assert!(item("snakeskin").other_effects.is_empty());
    // Ustlavic Pride: the armour and the CMD bonus are always on, the damage reduction is against undead only.
    // Game: FullplateOfUstalavianOfficerArmorItem 59a33c8783ad4de8b547022ec274c930; FullplateOfUstalavianOfficerFeature
    // e80f8d366436483c9499ac84d313f98a
    assert_eq!(
        summary("ustlavic-pride"),
        "Cha +3 enhancement; A +3 full plate; CMD +4 enhancement"
    );
    assert_eq!(
        item("ustlavic-pride").other_effects,
        ["DR 5/- against attacks from undead."]
    );
    // Draven's Hat: the page's two lines, always-quickened Slow and Haste and one free quicken a day, were one line.
    // Game: DravensHatFeature 21007755b76402743bec22faacfb5d02 (Slow and Haste quickened); DravensHatItem
    // 0160c8eabe704db41b6309af0d26a8e0 (EquipmentRestrictionAlignment: not good). The once-a-day line is the game's
    // otherwise: `dravens_hat_has_no_once_a_day_quicken`.
    assert_eq!(
        item("dravens-hat").always_on,
        ["Slow and Haste spells are quickened"]
    );
    assert!(
        item("dravens-hat")
            .other_effects
            .iter()
            .any(|line| line == "Good characters cannot wear it.")
    );
    // Offroad Boots: ignoring difficult terrain is always on, the speed it grants is a one-round trigger.
    // Game: DLC3_BootsOfUndersumpItem 77e8e9ceb50447bfbae19f6c9a850d76; DLC3_BootsOfUndersumpBuff
    // eb4a1b7d127d44d98df6cc2120868479 (immunity to difficult terrain and +10 speed for a round on entering it; that
    // the immunity is always on in effect is an inference)
    assert!(summary("offroad-boots").ends_with("; Ignores difficult terrain"));
    assert_eq!(item("offroad-boots").other_effects.len(), 1);
}

/// Draven's Hat has no once-a-day quicken: its other effect is that a good character can't wear it.
// Game: DravensHatItem 0160c8eabe704db41b6309af0d26a8e0; DravensHatFeature 21007755b76402743bec22faacfb5d02 (no
// once-a-day component)
#[test]
#[ignore = "waits on ticket 288: the game gives Draven's Hat no once-a-day quicken"]
fn dravens_hat_has_no_once_a_day_quicken() {
    assert_eq!(
        item("dravens-hat").other_effects,
        ["Good characters cannot wear it."]
    );
}

#[test]
fn no_line_is_in_both_lists_and_every_line_is_short() {
    for item in &DATA.gear {
        for line in &item.always_on {
            assert!(!line.trim().is_empty(), "{}", item.name);
            assert!(line.chars().count() <= 60, "{}: {line}", item.name);
            assert!(!item.other_effects.contains(line), "{}: {line}", item.name);
            assert_eq!(
                item.always_on.iter().filter(|l| *l == line).count(),
                1,
                "{}: {line}",
                item.name
            );
        }
    }
}

#[test]
fn every_summary_is_non_empty_and_every_other_effect_is_short() {
    for item in &DATA.gear {
        assert!(!item.summary(&DATA.skills).is_empty(), "{}", item.name);
        for line in &item.other_effects {
            assert!(line.chars().count() <= 140, "{}: {line}", item.name);
        }
    }
}

#[test]
fn a_gear_key_names_one_link_hash() {
    // The link's key kind for gear (ticket 165): every item is in `LinkKeys` without a collision.
    assert_eq!(rules::LinkKeys::new(&DATA).collisions(), []);
}
