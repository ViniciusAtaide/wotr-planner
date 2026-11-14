//! The weapons a character is proficient with (ticket 198): the proficiency feats, the class features' weapon lists,
//! the race's weapon familiarity, the background's weapons and the deity's favoured weapons. The web app's weapon
//! picker lists only these.

use std::collections::BTreeSet;

use rules::{BackgroundKey, Build, RaceKey, SubraceKey};

use crate::support::*;

/// The weapons with stats (those the picker lists) a Build is proficient with at level 1, by key.
fn wieldable(build: &Build) -> BTreeSet<String> {
    let engine = engine();
    let proficient = engine.proficient_weapons(&build.start, &at(build, 1));
    engine
        .attack_weapons()
        .into_iter()
        .filter(|weapon| proficient.contains(&weapon.key))
        .map(|weapon| weapon.key.as_str().to_owned())
        .collect()
}

fn keys(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|key| (*key).to_owned()).collect()
}

/// The simple weapons with stats, by key, and the unarmed strike, which is on no list.
fn simple() -> BTreeSet<String> {
    let mut weapons = of_list(rules::WeaponProficiency::Simple);
    weapons.insert("unarmed-strike".to_owned());
    weapons
}

fn of_list(list: rules::WeaponProficiency) -> BTreeSet<String> {
    DATA.weapons
        .iter()
        .filter(|weapon| weapon.proficiency == Some(list) && weapon.stats.is_some())
        .map(|weapon| weapon.key.as_str().to_owned())
        .collect()
}

fn with_race(mut build: Build, race: &str, subrace: Option<&str>) -> Build {
    build.start.race = Some(RaceKey::new(race));
    build.start.subrace = subrace.map(SubraceKey::new);
    build
}

#[test]
fn a_wizard_has_the_wizard_proficiencies_and_the_unarmed_strike() {
    assert_eq!(
        wieldable(&build("wizard", 1)),
        keys(&[
            "club",
            "dagger",
            "heavy-crossbow",
            "light-crossbow",
            "quarterstaff",
            "unarmed-strike",
        ])
    );
}

#[test]
fn a_rogue_has_the_simple_weapons_rapier_short_sword_and_short_bow() {
    let mut expected = simple();
    expected.extend(keys(&["rapier", "short-bow", "short-sword"]));
    assert_eq!(wieldable(&build("rogue", 1)), expected);
}

#[test]
fn a_hunter_has_the_simple_and_martial_weapons_and_no_exotic_one() {
    let mut expected = simple();
    expected.extend(of_list(rules::WeaponProficiency::Martial));
    assert_eq!(wieldable(&build("hunter", 1)), expected);
}

#[test]
fn a_sorcerer_has_the_simple_weapons() {
    assert_eq!(wieldable(&build("sorcerer", 1)), simple());
}

#[test]
fn an_elf_wizard_adds_the_elven_weapons_but_not_the_curve_blade() {
    let elf = wieldable(&with_race(build("wizard", 1), "elf", None));
    for weapon in ["long-bow", "longsword", "rapier", "short-bow", "club"] {
        assert!(elf.contains(weapon), "{weapon}");
    }
    // Treated as martial, and a wizard isn't proficient with martial weapons.
    assert!(!elf.contains("elven-curve-blade"));
}

#[test]
fn a_dwarf_fighter_wields_the_dwarven_waraxe_as_a_martial_weapon() {
    let dwarf = wieldable(&with_race(build("fighter", 1), "dwarf", None));
    assert!(dwarf.contains("dwarven-waraxe"));
    assert!(dwarf.contains("dwarven-urgrosh"));
    assert!(!dwarf.contains("bastard-sword"));
    assert!(!wieldable(&build("fighter", 1)).contains("dwarven-waraxe"));
}

#[test]
fn a_subrace_s_weapon_familiarity_replaces_the_race_s() {
    let human_raised = wieldable(&with_race(
        build("wizard", 1),
        "half-orc",
        Some("half-orc/human-raised"),
    ));
    assert!(!human_raised.contains("falchion"));
    let tribal = wieldable(&with_race(
        build("wizard", 1),
        "half-orc",
        Some("half-orc/tribal"),
    ));
    assert!(tribal.contains("falchion"));
    assert!(tribal.contains("greataxe"));
}

#[test]
fn a_background_adds_its_weapons() {
    let mut farmhand = build("wizard", 1);
    farmhand.start.background = Some(BackgroundKey::new("farmhand"));
    let farmhand = wieldable(&farmhand);
    assert!(farmhand.contains("scythe"));
    assert!(!wieldable(&build("wizard", 1)).contains("scythe"));
}

#[test]
fn a_cleric_wields_the_deity_s_martial_favoured_weapon() {
    // Erastil favours the longbow, a martial weapon no cleric feat grants.
    let mut cleric = build("cleric", 1);
    cleric.start.deity = Some(rules::Faith::Deity(rules::DeityKey::new("erastil")));
    cleric.start.alignment = Some(rules::Alignment::LawfulGood);
    let cleric = faithful(cleric);
    assert!(engine().is_possible(&cleric));
    assert!(wieldable(&cleric).contains("long-bow"));
    assert!(!wieldable(&build("cleric", 1)).contains("long-bow"));
}

/// Every class and archetype is proficient at level 1 with some weapon besides the unarmed strike: a class whose
/// weapons are only text would have none, as the wizard and the sorcerer had.
#[test]
fn every_class_and_archetype_wields_some_weapon_at_level_1() {
    let mut empty = Vec::new();
    for class in &DATA.classes {
        let mut builds = vec![(class.key.as_str().to_owned(), build(class.key.as_str(), 1))];
        for archetype in &class.archetypes {
            builds.push((
                archetype.key.as_str().to_owned(),
                archetype_build(class.key.as_str(), archetype.key.as_str(), 1),
            ));
        }
        for (name, build) in builds {
            let weapons = wieldable(&build);
            if weapons.iter().all(|weapon| weapon == "unarmed-strike") {
                empty.push(name);
            }
        }
    }
    assert!(empty.is_empty(), "no weapon at level 1: {empty:?}");
}
