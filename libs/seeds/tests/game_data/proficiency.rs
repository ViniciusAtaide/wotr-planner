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
    // Game: WizardProficiencies a98d7cc4e30fe6c4bb3a2c2f69acc3fe: club, dagger, heavy and light crossbow,
    // quarterstaff.
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
    // Game: RogueProficiencies 33e2a7e4ad9daa54eaf808e1483bb43c: simple, rapier, short bow, short sword (and the hand
    // crossbow, a category Enums/WeaponCategoryExtension.Data marks Disabled, WrathDecompiled 6ce4a6b).
    let mut expected = simple();
    expected.extend(keys(&["rapier", "short-bow", "short-sword"]));
    assert_eq!(wieldable(&build("rogue", 1)), expected);
}

#[test]
fn a_hunter_has_the_simple_and_martial_weapons_and_no_exotic_one() {
    // Game: HunterProficiencies 873f1f12bb43cda40bcc7d2878d16bf1: simple and martial.
    let mut expected = simple();
    expected.extend(of_list(rules::WeaponProficiency::Martial));
    assert_eq!(wieldable(&build("hunter", 1)), expected);
}

#[test]
fn a_sorcerer_has_the_simple_weapons() {
    // Game: SorcererProficiencies 25c97697236ccf2479d0c6a4185eae7f: simple.
    assert_eq!(wieldable(&build("sorcerer", 1)), simple());
}

#[test]
fn an_elf_wizard_adds_the_elven_weapons_but_not_the_curve_blade() {
    // Game: ElvenWeaponFamiliarity 03fd1e043fc678a4baf73fe67c3780ce: longbow, longsword, rapier, short bow; the curved
    // blade comes only through MartialWeaponProficiency 203992ef5b35c864390b4e4a1e200629's ElfRace entry.
    let elf = wieldable(&with_race(build("wizard", 1), "elf", None));
    for weapon in ["long-bow", "longsword", "rapier", "short-bow", "club"] {
        assert!(elf.contains(weapon), "{weapon}");
    }
    // Treated as martial, and a wizard isn't proficient with martial weapons.
    assert!(!elf.contains("elven-curve-blade"));
}

#[test]
fn a_dwarf_fighter_wields_the_dwarven_waraxe_as_a_martial_weapon() {
    // Game: MartialWeaponProficiency 203992ef5b35c864390b4e4a1e200629 adds the dwarven waraxe and urgrosh for DwarfRace
    // (DwarvenWeaponFamiliarity a1619e8d27fe97c40ba443f6f8ab1763).
    let dwarf = wieldable(&with_race(build("fighter", 1), "dwarf", None));
    assert!(dwarf.contains("dwarven-waraxe"));
    assert!(dwarf.contains("dwarven-urgrosh"));
    assert!(!dwarf.contains("bastard-sword"));
    assert!(!wieldable(&build("fighter", 1)).contains("dwarven-waraxe"));
}

#[test]
fn a_subrace_s_weapon_familiarity_replaces_the_race_s() {
    // Game: HumanRaisedHalfOrc 289d95c2c26f4e638c1f26d5ea7c594b removes OrcWeaponFamiliarity
    // 6ab6c271d1558344cbc746350243d17d (greataxe, falchion); TribalHalfOrc eb20cb0326444e348fc18f9b4d1644da keeps it.
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
    // Game: BackgroundFarmhand 25b35e09665310d4faac3020f8198cfb: ScytheProficiency.
    farmhand.start.background = Some(BackgroundKey::new("farmhand"));
    let farmhand = wieldable(&farmhand);
    assert!(farmhand.contains("scythe"));
    assert!(!wieldable(&build("wizard", 1)).contains("scythe"));
}

#[test]
fn a_cleric_wields_the_deity_s_martial_favoured_weapon() {
    // Erastil favours the longbow, a martial weapon no cleric feat grants.
    // Game: ErastilFeature afc775188deb7a44aa4cbde03512c671: LongbowProficiency at cleric (or warpriest) level 1
    // (AddFeatureOnClassLevel).
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
        for archetype in DATA.archetypes_of(&class.key) {
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
