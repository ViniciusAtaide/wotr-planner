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
    let mut expected: Vec<String> = Vec::new();
    for name in [
        "Belt of Incredible Dexterity",
        "Belt of Giant Strength",
        "Belt of the Mighty Constitution",
        "Headband of Alluring Charisma",
        "Headband of Vast Intelligence",
        "Headband of Inspired Wisdom",
    ] {
        expected.extend([2, 4, 6].map(|n| format!("{name} +{n}")));
    }
    for name in [
        "Belt of Physical Perfection",
        "Headband of Mental Perfection",
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
        assert!(
            item.bonuses
                .iter()
                .all(|bonus| bonus.kind == BonusType::Inherent),
            "{}",
            item.name
        );
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

#[test]
fn trusted_friend_is_a_necklace() {
    assert_eq!(item("trusted-friend").slot, GearSlot::Necklace);
    assert_eq!(
        summary("trusted-friend"),
        "Cha +2 circumstance; Persuasion +5 circumstance"
    );
}

#[test]
fn an_armour_and_a_weapon_name_their_own_enhancement_and_base_type() {
    for (key, line) in [
        ("snakeskin", "A +4 leather armour"),
        ("life-infuser", "A +5 breastplate"),
        ("half-plate-of-vigor", "A +1 half-plate"),
        ("ustlavic-pride", "A +3 full plate"),
        ("sin-mages-staff", "A +5 quarterstaff"),
        ("dirge-for-sarkoris", "A +2 keen club"),
    ] {
        // Always on, so in the summary and not behind the detail (ticket 176).
        assert!(item(key).always_on.iter().any(|l| l == line), "{key}");
        assert!(summary(key).contains(line), "{key}");
    }
}

#[test]
fn the_venerated_champions_fortitude_is_an_effect_not_a_bonus() {
    assert_eq!(
        summary("belt-of-the-venerated-champion"),
        "Con +6 enhancement"
    );
}

#[test]
fn a_single_score_belt_matches_its_page_line() {
    // items#belts: "+6 enhancement bonus to Dexterity / On crit: the wearer gains a +2 circumstance bonus to attack
    // and damage rolls for 3 rounds".
    let belt = item("assailants-belt");
    assert_eq!(belt.slot, GearSlot::Belt);
    assert_eq!(belt.guide.as_ref().unwrap().0, "items#belts");
    assert_eq!(summary("assailants-belt"), "Dex +6 enhancement");
    assert_eq!(belt.other_effects.len(), 1);
}

#[test]
fn a_three_score_belt_matches_its_page_line() {
    // items#belts: "+6 enhancement bonus to Strength, Dexterity and Constitution".
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
    assert_eq!(summary("timely-assistance"), "Con, Wis +4 enhancement");
    assert_eq!(
        summary("life-infuser"),
        "Str, Con +8 armour; A +5 breastplate; Moderate fortification"
    );
    assert_eq!(summary("half-plate-of-vigor"), "Con +2; A +1 half-plate");
    assert_eq!(
        summary("sin-mages-staff"),
        "Int +6; Spell DC +1; Caster level +3; A +5 quarterstaff"
    );
    assert_eq!(
        summary("bookworms-headband"),
        "Int +2 enhancement; Cha -2; Persuasion -3; Immune to compulsion effects"
    );
    assert_eq!(
        summary("gloves-of-the-ambassador"),
        "Cha +2 enhancement; Knowledge (World), Persuasion +5 competence"
    );
    assert_eq!(
        summary("goggles-of-ferocious-pact"),
        "Wis +2 enhancement; Fortitude -2; Will +3; Persuasion -5; AC -2; Damage +4"
    );
}

#[test]
fn mangling_frenzys_damage_reduction_is_in_its_summary_and_its_crit_line_stays_in_the_detail() {
    // items#belts: the three scores, a damage reduction of 3 with no condition, and +4d6 slashing damage on a
    // critical hit while raging. Ticket 176: what is always on is in the summary, what needs a trigger is in the detail.
    assert_eq!(
        summary("mangling-frenzy"),
        "Str, Dex, Con +4 enhancement; DR 3/-"
    );
    assert_eq!(item("mangling-frenzy").always_on, ["DR 3/-"]);
    assert_eq!(
        item("mangling-frenzy").other_effects,
        ["Critical hit while raging: +4d6 slashing damage."]
    );
}

#[test]
fn an_item_keeps_its_conditional_lines_and_shows_its_always_on_ones() {
    // Snakeskin: all three lines apply while it is worn; nothing is left for a detail.
    assert_eq!(
        summary("snakeskin"),
        "Dex +4 profane; A +4 leather armour; +20 feet base speed; Immune to acid"
    );
    assert!(item("snakeskin").other_effects.is_empty());
    // Ustlavic Pride: the armour and the CMD bonus are always on, the damage reduction is against undead only.
    assert_eq!(
        summary("ustlavic-pride"),
        "Cha +3 enhancement; A +3 full plate; CMD +4 enhancement"
    );
    assert_eq!(
        item("ustlavic-pride").other_effects,
        ["DR 5/- against attacks from undead."]
    );
    // Draven's Hat: the page's two lines, always-quickened Slow and Haste and one free quicken a day, were one line.
    assert_eq!(
        item("dravens-hat").always_on,
        ["Slow and Haste spells are quickened"]
    );
    assert_eq!(
        item("dravens-hat").other_effects,
        [
            "Good characters cannot wear it.",
            "Once a day: any spell can be quickened."
        ]
    );
    // Offroad Boots: ignoring difficult terrain is always on, the speed it grants is a one-round trigger.
    assert!(summary("offroad-boots").ends_with("; Ignores difficult terrain"));
    assert_eq!(item("offroad-boots").other_effects.len(), 1);
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
