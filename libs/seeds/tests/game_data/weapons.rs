//! Weapon stats (ticket 114): every weapon on a proficiency list, and the unarmed strike, has dice, a threat range,
//! a multiplier and its hands; Weapon Finesse's list is the guide's; what deals a spell's, a class feature's or a
//! natural attack's damage has none. The attack line reads them.

use std::collections::BTreeSet;

use rules::{CharacterState, Dice, FeatKey, Handedness, Parameter, Ranged, WeaponKey};

use crate::support::*;

fn stats(key: &str) -> Option<rules::WeaponStats> {
    DATA.weapons
        .iter()
        .find(|weapon| weapon.key.as_str() == key)
        .unwrap_or_else(|| panic!("no weapon {key}"))
        .stats
}

#[test]
fn every_weapon_of_a_proficiency_list_and_the_unarmed_strike_has_stats() {
    let without: BTreeSet<&str> = DATA
        .weapons
        .iter()
        .filter(|weapon| weapon.stats.is_none())
        .map(|weapon| weapon.key.as_str())
        .collect();

    assert_eq!(
        without,
        BTreeSet::from([
            "bite",
            "bomb",
            "claw",
            "gore",
            "hoof",
            "kinetic-blast",
            "rays",
            "slam",
            "spike",
            "tail",
            "talon",
            "touch-attacks",
            "wing",
        ])
    );
    assert!(stats("unarmed-strike").is_some());
}

#[test]
fn the_finessable_weapons_are_the_guides_weapon_finesse_list() {
    // feats#weapon-finesse: "Simple: Dagger, Light Mace, Punching Dagger, Sickle; Martial: Handaxe, Kukri, Light
    // Hammer, Light Pick, Rapier, Short Sword, Starknife; Exotic: Dueling Sword, Elven Curve Blade, Estoc, Kama, Sai,
    // Sawtooth Saber".
    let finesse: BTreeSet<&str> = DATA
        .weapons
        .iter()
        .filter(|weapon| weapon.stats.is_some_and(|stats| stats.finesse))
        .map(|weapon| weapon.key.as_str())
        .collect();

    assert_eq!(
        finesse,
        BTreeSet::from([
            "dagger",
            "light-mace",
            "punching-dagger",
            "sickle",
            "hand-axe",
            "kukri",
            "light-hammer",
            "light-pick",
            "rapier",
            "short-sword",
            "starknife",
            "dueling-sword",
            "elven-curve-blade",
            "estoc",
            "kama",
            "sai",
            "sawtooth-saber",
        ])
    );
}

#[test]
fn a_few_weapons_stats() {
    let greatsword = stats("greatsword").unwrap();
    assert_eq!(greatsword.damage, Dice { count: 2, sides: 6 });
    assert_eq!((greatsword.threat, greatsword.multiplier), (19, 2));
    assert_eq!(greatsword.handedness, Handedness::TwoHanded);

    let scythe = stats("scythe").unwrap();
    assert_eq!(
        (scythe.damage, scythe.multiplier),
        (Dice { count: 2, sides: 4 }, 4)
    );

    // The guide: the Elven Curve Blade is two-handed, the Estoc a one-handed piercing weapon (Fencing Grace).
    assert_eq!(
        stats("elven-curve-blade").unwrap().handedness,
        Handedness::TwoHanded
    );
    assert_eq!(stats("estoc").unwrap().handedness, Handedness::OneHanded);

    assert_eq!(stats("long-bow").unwrap().ranged, Some(Ranged::Projectile));
    assert_eq!(stats("javelin").unwrap().ranged, Some(Ranged::Thrown));
    assert_eq!(stats("longsword").unwrap().ranged, None);
}

#[test]
fn a_greatsword_fighters_attack_line() {
    // BAB +8, Str 18 (+4), Power Attack and Weapon Focus (Greatsword).
    let mut state = CharacterState {
        bab: 8,
        ..CharacterState::default()
    };
    state.ability_scores.strength = 18;
    state.ability_scores.dexterity = 10;
    state.feats.insert(FeatKey::new("power-attack"), 1);
    state.feats.insert(FeatKey::new("weapon-focus"), 1);
    state
        .feat_values
        .entry(FeatKey::new("weapon-focus"))
        .or_default()
        .insert(Parameter::Weapon(WeaponKey::new("greatsword")), 1);

    let line = engine()
        .attack_line(&state, &WeaponKey::new("greatsword"))
        .unwrap();

    assert_eq!(line.attacks, [13, 8]);
    assert_eq!(
        (line.damage, line.damage_bonus),
        (Dice { count: 2, sides: 6 }, 6)
    );
    assert_eq!((line.threat, line.multiplier), (19, 2));
    let trade = line.power_attack.unwrap();
    assert_eq!((trade.attack_penalty, trade.damage_bonus), (3, 9));
    assert_eq!(
        line.parts
            .iter()
            .map(|part| part.name.as_str())
            .collect::<Vec<_>>(),
        ["Strength", "Weapon Focus"]
    );
}
