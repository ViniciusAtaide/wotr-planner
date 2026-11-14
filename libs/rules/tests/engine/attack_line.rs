//! The attack line: one base weapon's attacks at a point of a Build, from BAB, the character's own ability scores and
//! the feats held for that weapon. Unconditional numbers only.

use rules::{
    CharacterState, Dice, Engine, FeatKey, Handedness, Ranged, Weapon, WeaponKey, WeaponStats,
};

use crate::support::{self, *};

/// The test world with a greatsword, a longbow, a javelin and rays (no stats) beside its own weapons.
fn armoury() -> Engine {
    let mut data = world();
    let weapon = |key: &str, stats| Weapon {
        key: WeaponKey::new(key),
        name: key.into(),
        proficiency: None,
        stats,
    };
    let stats = |count, sides, threat, multiplier, handedness, ranged| {
        Some(WeaponStats {
            damage: Dice { count, sides },
            threat,
            multiplier,
            handedness,
            ranged,
            finesse: false,
        })
    };
    data.weapons.extend([
        weapon(
            "greatsword",
            stats(2, 6, 19, 2, Handedness::TwoHanded, None),
        ),
        weapon(
            "longbow",
            stats(1, 8, 20, 3, Handedness::TwoHanded, Some(Ranged::Projectile)),
        ),
        weapon(
            "javelin",
            stats(1, 6, 20, 2, Handedness::OneHanded, Some(Ranged::Thrown)),
        ),
        weapon("rays", None),
    ]);
    engine_on(data)
}

/// A character with this BAB and Str and Dex (the rest 10), holding no feats.
fn character(bab: u8, strength: u8, dexterity: u8) -> CharacterState {
    CharacterState {
        bab,
        ability_scores: scores([strength, dexterity, 10, 10, 10, 10]),
        ..CharacterState::default()
    }
}

#[test]
fn a_one_handed_weapon_attacks_at_each_iterative_with_strength_to_attack_and_damage() {
    // BAB +11, Str 18 (+4), a longsword (1d8, 19–20/×2).
    let line = engine()
        .attack_line(&character(11, 18, 10), &WeaponKey::new("longsword"))
        .expect("the longsword has stats");

    assert_eq!(line.attacks, [15, 10, 5]);
    assert_eq!(line.damage, Dice { count: 1, sides: 8 });
    assert_eq!(line.damage_bonus, 4);
    assert_eq!((line.threat, line.multiplier), (19, 2));
    assert_eq!(line.power_attack, None);
}

#[test]
fn a_further_attack_comes_at_bab_6_11_and_16() {
    let engine = engine();
    let attacks = |bab| {
        engine
            .attack_line(&character(bab, 10, 10), &WeaponKey::new("longsword"))
            .unwrap()
            .attacks
    };

    assert_eq!(attacks(0), [0]);
    assert_eq!(attacks(5), [5]);
    assert_eq!(attacks(6), [6, 1]);
    assert_eq!(attacks(11), [11, 6, 1]);
    assert_eq!(attacks(16), [16, 11, 6, 1]);
    assert_eq!(attacks(20), [20, 15, 10, 5]);
}

#[test]
fn a_weapon_without_stats_or_unknown_has_no_line() {
    let engine = armoury();
    let state = character(6, 14, 10);

    assert_eq!(engine.attack_line(&state, &WeaponKey::new("rays")), None);
    assert_eq!(engine.attack_line(&state, &WeaponKey::new("no-such")), None);
}

#[test]
fn the_line_names_what_it_counts_and_what_it_leaves_out() {
    let state = holding(
        holding(character(6, 18, 10), "weapon-focus", Some("longsword")),
        "weapon-specialization",
        Some("longsword"),
    );
    let line = engine()
        .attack_line(&state, &WeaponKey::new("longsword"))
        .unwrap();

    let parts: Vec<(&str, i16, i16)> = line
        .parts
        .iter()
        .map(|part| (part.name.as_str(), part.attack, part.damage))
        .collect();
    assert_eq!(
        parts,
        [
            ("Strength", 4, 4),
            ("weapon-focus", 1, 0),
            ("weapon-specialization", 0, 2),
        ]
    );
    for left_out in [
        "rage",
        "stances",
        "mythic",
        "class features",
        "gear",
        "buffs",
    ] {
        // In short too: the short form is what an MCP response carries.
        assert!(
            line.leaves_out
                .iter()
                .any(|item| item.short.contains(left_out) && item.full.contains(left_out)),
            "leaves out {left_out}"
        );
    }
}

#[test]
fn the_line_reads_as_the_game_writes_attacks_damage_and_criticals() {
    let engine = armoury();
    let line = |bab, strength, weapon: &str| {
        engine
            .attack_line(&character(bab, strength, 10), &WeaponKey::new(weapon))
            .unwrap()
    };

    let longsword = line(11, 18, "longsword");
    assert_eq!(longsword.attacks_text(), "+15/+10/+5");
    assert_eq!(longsword.damage_text(), "1d8+4");
    assert_eq!(longsword.critical_text(), "19–20/×2");

    let weak = line(0, 6, "longbow");
    assert_eq!(weak.attacks_text(), "+0");
    assert_eq!(weak.damage_text(), "1d8");
    assert_eq!(weak.critical_text(), "20/×3");

    let feeble = line(1, 6, "longsword");
    assert_eq!(feeble.attacks_text(), "-1");
    assert_eq!(feeble.damage_text(), "1d8-2");
}

#[test]
fn the_weapons_with_a_line_are_those_with_stats_by_name() {
    let engine = armoury();
    let keys: Vec<&str> = engine
        .attack_weapons()
        .iter()
        .map(|weapon| weapon.key.as_str())
        .collect();

    assert_eq!(
        keys,
        [
            "bastard-sword",
            "dagger",
            "dueling-sword",
            "greatsword",
            "javelin",
            "longbow",
            "longsword",
            "unarmed-strike",
        ],
        "not the rays"
    );
}

#[test]
fn the_default_weapon_is_the_first_weapon_focus_taken_with_stats() {
    let engine = armoury();
    let mut build = build("human", &times("fighter", 3));
    // Rays at level 1 (no stats), then the longsword at level 1's general slot, the dagger at level 3.
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("rays")),
    );
    build.picks.insert(
        general_slot(1),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build.picks.insert(
        general_slot(3),
        pick_feat_for("weapon-focus", weapon("dagger")),
    );
    assert!(engine.is_possible(&build));

    assert_eq!(
        engine.default_attack_weapon(&engine.evaluate(&build)),
        Some(WeaponKey::new("longsword"))
    );

    let without = support::build("human", &times("fighter", 3));
    assert_eq!(
        engine.default_attack_weapon(&engine.evaluate(&without)),
        None
    );
}

#[test]
fn a_two_handed_weapon_adds_one_and_a_half_times_the_strength_bonus_but_only_the_penalty() {
    let greatsword = WeaponKey::new("greatsword");
    let damage = |strength| {
        armoury()
            .attack_line(&character(1, strength, 10), &greatsword)
            .unwrap()
            .damage_bonus
    };

    // +4 → +6; +3 → +4 (rounded down); −1 stays −1.
    assert_eq!((damage(18), damage(16), damage(8)), (6, 4, -1));
}

/// The character holding a feat, for nothing or (with `value`) for one weapon.
fn holding(mut state: CharacterState, key: &str, value: Option<&str>) -> CharacterState {
    let feat = FeatKey::new(key);
    state.feats.insert(feat.clone(), 1);
    if let Some(value) = value {
        state
            .feat_values
            .entry(feat)
            .or_default()
            .insert(weapon(value), 1);
    }
    state
}

#[test]
fn weapon_focus_and_specialization_count_for_their_weapon_only() {
    let mut state = character(6, 10, 10);
    for feat in [
        "weapon-focus",
        "greater-weapon-focus",
        "weapon-specialization",
        "greater-weapon-specialization",
    ] {
        state = holding(state, feat, Some("longsword"));
    }
    state = holding(state, "weapon-focus", Some("dagger"));
    let engine = engine();

    let longsword = engine
        .attack_line(&state, &WeaponKey::new("longsword"))
        .unwrap();
    assert_eq!(longsword.attacks, [8, 3], "+1 and +1 to attack");
    assert_eq!(longsword.damage_bonus, 4, "+2 and +2 to damage");

    let dagger = engine
        .attack_line(&state, &WeaponKey::new("dagger"))
        .unwrap();
    assert_eq!(dagger.attacks, [7, 2]);
    assert_eq!(dagger.damage_bonus, 0);
}

#[test]
fn power_attack_trades_one_attack_for_two_damage_per_four_bab_and_three_two_handed() {
    let engine = armoury();
    let trade = |bab, weapon: &str| {
        engine
            .attack_line(
                &holding(character(bab, 14, 10), "power-attack", None),
                &WeaponKey::new(weapon),
            )
            .unwrap()
            .power_attack
            .map(|trade| (trade.attack_penalty, trade.damage_bonus))
    };

    // The guide: −1/+2, one step more at BAB 4 and every four after (BAB 8: −3/+6).
    assert_eq!(trade(1, "longsword"), Some((1, 2)));
    assert_eq!(trade(3, "longsword"), Some((1, 2)));
    assert_eq!(trade(4, "longsword"), Some((2, 4)));
    assert_eq!(trade(8, "longsword"), Some((3, 6)));
    assert_eq!(trade(8, "greatsword"), Some((3, 9)));
    assert_eq!(trade(8, "longbow"), None, "melee attacks only");
    assert_eq!(trade(8, "javelin"), None, "melee attacks only");

    let without = engine
        .attack_line(&character(8, 14, 10), &WeaponKey::new("longsword"))
        .unwrap();
    assert_eq!(without.power_attack, None);
}

#[test]
fn improved_critical_in_the_weapon_doubles_its_threat_range() {
    let engine = armoury();
    let state = holding(
        holding(character(8, 10, 10), "improved-critical", Some("longsword")),
        "improved-critical",
        Some("longbow"),
    );
    let critical = |weapon: &str| {
        let line = engine.attack_line(&state, &WeaponKey::new(weapon)).unwrap();
        (line.threat, line.multiplier)
    };

    assert_eq!(critical("longsword"), (17, 2), "19–20 → 17–20");
    assert_eq!(critical("longbow"), (19, 3), "20 → 19–20");
    assert_eq!(critical("greatsword"), (19, 2), "not held for it");
}

#[test]
fn a_ranged_weapon_attacks_with_dexterity_and_only_a_thrown_one_adds_strength_to_damage() {
    let engine = armoury();
    // BAB +6, Str 14 (+2), Dex 18 (+4).
    let state = character(6, 14, 18);

    let longbow = engine
        .attack_line(&state, &WeaponKey::new("longbow"))
        .unwrap();
    assert_eq!(longbow.attacks, [10, 5]);
    assert_eq!(longbow.damage_bonus, 0);

    let javelin = engine
        .attack_line(&state, &WeaponKey::new("javelin"))
        .unwrap();
    assert_eq!(javelin.attacks, [10, 5]);
    assert_eq!(javelin.damage_bonus, 2);
}

#[test]
fn weapon_finesse_attacks_with_dexterity_when_higher_with_a_finessable_weapon() {
    let engine = engine();
    let attack = |strength, dexterity, weapon: &str| {
        let state = holding(character(1, strength, dexterity), "weapon-finesse", None);
        let line = engine.attack_line(&state, &WeaponKey::new(weapon)).unwrap();
        (line.attacks[0], line.damage_bonus)
    };

    // Str 12 (+1), Dex 18 (+4): the dagger and dueling sword are on the guide's list, the longsword isn't.
    assert_eq!(
        attack(12, 18, "dagger"),
        (5, 1),
        "Dex to attack, Str to damage"
    );
    assert_eq!(attack(12, 18, "dueling-sword"), (5, 1));
    assert_eq!(attack(12, 18, "longsword"), (2, 1));
    assert_eq!(attack(18, 12, "dagger"), (5, 4), "Str is higher");

    let without = engine
        .attack_line(&character(1, 12, 18), &WeaponKey::new("dagger"))
        .unwrap();
    assert_eq!(without.attacks[0], 2);
}

#[test]
fn the_trickster_critical_feats_add_to_the_threat_range_and_the_multiplier_after_improved_critical()
{
    let engine = armoury();
    let critical = |feats: &[&str]| {
        let state = feats.iter().fold(character(8, 10, 10), |state, feat| {
            holding(state, feat, Some("greatsword"))
        });
        let line = engine
            .attack_line(&state, &WeaponKey::new("greatsword"))
            .unwrap();
        line.critical_text()
    };

    // The guide's trickster page: +1 to the threat range, +1 more, then +1 to the multiplier. The +1s come after
    // Improved Critical doubles the range: 19–20 → 17–20 → 16–20 → 15–20.
    assert_eq!(critical(&["improved-critical"]), "17–20/×2");
    assert_eq!(
        critical(&["improved-critical", "improved-improved-critical"]),
        "16–20/×2"
    );
    assert_eq!(
        critical(&[
            "improved-critical",
            "improved-improved-critical",
            "improved-improved-improved-critical",
        ]),
        "15–20/×2"
    );
    assert_eq!(
        critical(&[
            "improved-critical",
            "improved-improved-critical",
            "improved-improved-improved-critical",
            "improved-improved-improved-critical-improved",
        ]),
        "15–20/×3"
    );

    // Held for another weapon, they don't count.
    let state = [
        "improved-improved-critical",
        "improved-improved-improved-critical",
        "improved-improved-improved-critical-improved",
    ]
    .iter()
    .fold(character(8, 10, 10), |state, feat| {
        holding(state, feat, Some("longsword"))
    });
    let greatsword = engine
        .attack_line(&state, &WeaponKey::new("greatsword"))
        .unwrap();
    assert_eq!(greatsword.critical_text(), "19–20/×2");
}
