//! Weapon stats (ticket 114): every weapon on a proficiency list, and the unarmed strike, has dice, a threat range,
//! a multiplier and its hands, as the game's table has them (the user's, 2026-11-08); what deals a spell's, a class feature's or a
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

/// The game's weapon table, as the user gave it (2026-11-08): dice, threat, multiplier, hands, finesse and ranged
/// kind for every weapon with stats. Where the game has a Kingmaker and a Wrath of the Righteous row, the Wrath row; a
/// double weapon (Urgrosh, Hooked Hammer, Orc Double Axe, Two-bladed Sword) is its first head, as one two-handed
/// weapon. The table wins over the guide's Weapon Finesse list (`feats#weapon-finesse`), which leaves out the
/// Nunchaku and the Light Shield's bash. The Sling Staff, the Dart, the Javelin and the Throwing Axe carry the
/// game's Strength trait: Strength to damage.
/// A weapon's key, dice count and sides, threat, multiplier, hands, finesse and ranged kind.
type Row = (
    &'static str,
    u8,
    u8,
    u8,
    u8,
    Handedness,
    bool,
    Option<Ranged>,
);

const TABLE: [Row; 60] = [
    ("bardiche", 1, 10, 19, 2, Handedness::TwoHanded, false, None),
    (
        "bastard-sword",
        1,
        10,
        19,
        2,
        Handedness::OneHanded,
        false,
        None,
    ),
    (
        "battle-axe",
        1,
        8,
        20,
        3,
        Handedness::OneHanded,
        false,
        None,
    ),
    ("club", 1, 6, 20, 2, Handedness::OneHanded, false, None),
    ("dagger", 1, 4, 19, 2, Handedness::Light, true, None),
    (
        "dart",
        1,
        4,
        20,
        2,
        Handedness::Light,
        false,
        Some(Ranged::Thrown),
    ),
    (
        "dueling-sword",
        1,
        8,
        19,
        2,
        Handedness::OneHanded,
        true,
        None,
    ),
    (
        "dwarven-urgrosh",
        1,
        8,
        20,
        3,
        Handedness::TwoHanded,
        false,
        None,
    ),
    (
        "dwarven-waraxe",
        1,
        10,
        20,
        3,
        Handedness::OneHanded,
        false,
        None,
    ),
    (
        "earth-breaker",
        2,
        6,
        20,
        3,
        Handedness::TwoHanded,
        false,
        None,
    ),
    (
        "elven-curve-blade",
        1,
        10,
        18,
        2,
        Handedness::TwoHanded,
        true,
        None,
    ),
    ("estoc", 2, 4, 18, 2, Handedness::OneHanded, true, None),
    ("falcata", 1, 8, 19, 3, Handedness::OneHanded, false, None),
    ("falchion", 2, 4, 18, 2, Handedness::TwoHanded, false, None),
    ("fauchard", 1, 10, 18, 2, Handedness::TwoHanded, false, None),
    ("flail", 1, 8, 20, 2, Handedness::OneHanded, false, None),
    ("glaive", 1, 10, 20, 3, Handedness::TwoHanded, false, None),
    (
        "gnome-hooked-hammer",
        1,
        8,
        20,
        3,
        Handedness::TwoHanded,
        false,
        None,
    ),
    ("greataxe", 1, 12, 20, 3, Handedness::TwoHanded, false, None),
    (
        "greatclub",
        1,
        10,
        20,
        2,
        Handedness::TwoHanded,
        false,
        None,
    ),
    (
        "greatsword",
        2,
        6,
        19,
        2,
        Handedness::TwoHanded,
        false,
        None,
    ),
    ("hand-axe", 1, 6, 20, 3, Handedness::Light, true, None),
    (
        "heavy-crossbow",
        1,
        10,
        19,
        2,
        Handedness::TwoHanded,
        false,
        Some(Ranged::Projectile),
    ),
    (
        "heavy-flail",
        1,
        10,
        19,
        2,
        Handedness::TwoHanded,
        false,
        None,
    ),
    (
        "heavy-mace",
        1,
        8,
        20,
        2,
        Handedness::OneHanded,
        false,
        None,
    ),
    (
        "heavy-pick",
        1,
        6,
        20,
        4,
        Handedness::OneHanded,
        false,
        None,
    ),
    (
        "heavy-shield-bash",
        1,
        4,
        20,
        2,
        Handedness::OneHanded,
        false,
        None,
    ),
    (
        "javelin",
        1,
        6,
        20,
        2,
        Handedness::OneHanded,
        false,
        Some(Ranged::Thrown),
    ),
    ("kama", 1, 6, 20, 2, Handedness::Light, true, None),
    ("kukri", 1, 4, 18, 2, Handedness::Light, true, None),
    (
        "light-crossbow",
        1,
        8,
        19,
        2,
        Handedness::TwoHanded,
        false,
        Some(Ranged::Projectile),
    ),
    ("light-hammer", 1, 4, 20, 2, Handedness::Light, true, None),
    ("light-mace", 1, 6, 20, 2, Handedness::Light, true, None),
    ("light-pick", 1, 4, 20, 4, Handedness::Light, true, None),
    (
        "light-shield-bash",
        1,
        3,
        20,
        2,
        Handedness::Light,
        true,
        None,
    ),
    (
        "long-bow",
        1,
        8,
        20,
        3,
        Handedness::TwoHanded,
        false,
        Some(Ranged::Projectile),
    ),
    ("longspear", 1, 8, 20, 3, Handedness::TwoHanded, false, None),
    ("longsword", 1, 8, 19, 2, Handedness::OneHanded, false, None),
    ("nunchaku", 1, 6, 20, 2, Handedness::Light, true, None),
    (
        "orc-double-axe",
        1,
        8,
        20,
        3,
        Handedness::TwoHanded,
        false,
        None,
    ),
    (
        "punching-dagger",
        1,
        4,
        20,
        3,
        Handedness::Light,
        true,
        None,
    ),
    (
        "quarterstaff",
        1,
        6,
        20,
        2,
        Handedness::TwoHanded,
        false,
        None,
    ),
    ("rapier", 1, 6, 18, 2, Handedness::OneHanded, true, None),
    ("sai", 1, 4, 20, 2, Handedness::Light, true, None),
    ("sawtooth-saber", 1, 8, 19, 2, Handedness::Light, true, None),
    ("scimitar", 1, 6, 18, 2, Handedness::OneHanded, false, None),
    ("scythe", 2, 4, 20, 4, Handedness::TwoHanded, false, None),
    (
        "short-bow",
        1,
        6,
        20,
        3,
        Handedness::TwoHanded,
        false,
        Some(Ranged::Projectile),
    ),
    ("short-sword", 1, 6, 19, 2, Handedness::Light, true, None),
    (
        "shortspear",
        1,
        6,
        20,
        2,
        Handedness::OneHanded,
        false,
        None,
    ),
    ("sickle", 1, 6, 20, 2, Handedness::Light, true, None),
    (
        "sling-staff",
        1,
        8,
        20,
        3,
        Handedness::TwoHanded,
        false,
        Some(Ranged::ProjectileWithStrength),
    ),
    ("spear", 1, 8, 20, 3, Handedness::TwoHanded, false, None),
    ("starknife", 1, 4, 20, 3, Handedness::Light, true, None),
    (
        "throwing-axe",
        1,
        6,
        20,
        2,
        Handedness::Light,
        false,
        Some(Ranged::Thrown),
    ),
    ("tongi", 1, 6, 19, 3, Handedness::OneHanded, false, None),
    ("trident", 1, 8, 20, 2, Handedness::OneHanded, false, None),
    (
        "two-bladed-sword",
        1,
        8,
        19,
        2,
        Handedness::TwoHanded,
        false,
        None,
    ),
    (
        "unarmed-strike",
        1,
        3,
        20,
        2,
        Handedness::Light,
        false,
        None,
    ),
    ("warhammer", 1, 8, 20, 3, Handedness::OneHanded, false, None),
];

#[test]
fn every_weapons_stats_are_the_games_table() {
    let data: Vec<Row> = DATA
        .weapons
        .iter()
        .filter_map(|weapon| {
            let stats = weapon.stats?;
            Some((
                weapon.key.as_str(),
                stats.damage.count,
                stats.damage.sides,
                stats.threat,
                stats.multiplier,
                stats.handedness,
                stats.finesse,
                stats.ranged,
            ))
        })
        .collect();
    assert_eq!(data, TABLE);
}

#[test]
fn a_sling_staff_adds_strength_to_damage() {
    // BAB +1, Str 14 (+2), Dex 16 (+3): Dexterity to attack, Strength to damage (the game's Strength trait).
    let mut state = CharacterState {
        bab: 1,
        ..CharacterState::default()
    };
    state.ability_scores.strength = 14;
    state.ability_scores.dexterity = 16;
    let engine = engine();
    let sling_staff = engine
        .attack_line(&state, &WeaponKey::new("sling-staff"))
        .unwrap();
    assert_eq!(
        (sling_staff.attacks, sling_staff.damage_bonus),
        (vec![4], 2)
    );
    let long_bow = engine
        .attack_line(&state, &WeaponKey::new("long-bow"))
        .unwrap();
    assert_eq!((long_bow.attacks, long_bow.damage_bonus), (vec![4], 0));
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
