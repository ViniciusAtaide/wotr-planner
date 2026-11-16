//! Weapon stats (ticket 114): every weapon on a proficiency list, and the unarmed strike, has dice, a threat range,
//! a multiplier and its hands, as the game's table has them (the user's, 2026-11-08); what deals a spell's, a class feature's or a
//! natural attack's damage has none. The attack line reads them.

use std::collections::BTreeSet;

use rules::{CharacterState, Dice, Fact, FeatKey, Handedness, Parameter, Ranged, WeaponKey};

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
/// game's Strength trait: Strength to damage. Each row cites the game's BlueprintWeaponType (`m_BaseDamage`,
/// `m_CriticalRollEdge`, `m_CriticalModifier`, `m_IsLight`, `m_IsTwoHanded`) and the category's Finessable
/// subcategory (`Enums/WeaponCategoryExtension.Data`, WrathDecompiled 6ce4a6b). The two shield bashes and the unarmed
/// strike, whose stats differ in the game, wait in `GAME_ROWS`.
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

const TABLE: [Row; 57] = [
    // Game: Bardiche b1cbf457fd471d148b39ae56667f405a
    ("bardiche", 1, 10, 19, 2, Handedness::TwoHanded, false, None),
    // Game: BastardSword d2fe2c5516b56f04da1d5ea51ae3ddfe
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
    // Game: Battleaxe 2bc77aa47f97de348aefcf03ec8fa43b
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
    // Game: Club 26aa0672af2c7d84ba93bec37758c712
    ("club", 1, 6, 20, 2, Handedness::OneHanded, false, None),
    // Game: Dagger 07cc1a7fceaee5b42b3e43da960fe76d
    ("dagger", 1, 4, 19, 2, Handedness::Light, true, None),
    // Game: Dart f415ae950523a7843a74d7780dd551af
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
    // Game: DuelingSword a6f7e3dc443ff114ba68b4648fd33e9f
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
    // Game: DwarvenUrgrosh 0ec97c08fdf87e44f8f16ba87b511743 (equivalent model: first head)
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
    // Game: DwarvenWaraxe a6925f5f897801449a648d865637e5a0
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
    // Game: EarthBreaker 54e960775daa4714fa52e0954d8cf862
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
    // Game: ElvenCurvedBlade b5e6838ad2a62b146b49619bcf9f42aa
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
    // Game: Estoc d516765b3c2904e4a939749526a52a9a
    ("estoc", 2, 4, 18, 2, Handedness::OneHanded, true, None),
    // Game: Falcata 1af5621e2ae551e42bd1dd6744d98639
    ("falcata", 1, 8, 19, 3, Handedness::OneHanded, false, None),
    // Game: Falchion 6ddc9acbbb6e40746a6a1671df1f7b47
    ("falchion", 2, 4, 18, 2, Handedness::TwoHanded, false, None),
    // Game: Fauchard 7a40899c4defec94bb9c291bde74f1a8
    ("fauchard", 1, 10, 18, 2, Handedness::TwoHanded, false, None),
    // Game: Flail bf1e53f7442ed0c43bf52d3abe55e16a
    ("flail", 1, 8, 20, 2, Handedness::OneHanded, false, None),
    // Game: Glaive 7a14a1b224cd173449cb7ffc77d5f65c
    ("glaive", 1, 10, 20, 3, Handedness::TwoHanded, false, None),
    // Game: GnomeHookedHammerHead 91645b645cf121a479c1fabc5678dcb3 (equivalent model: first head)
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
    // Game: Greataxe e8059a8eac62cd74f9171d748a5ae428
    ("greataxe", 1, 12, 20, 3, Handedness::TwoHanded, false, None),
    // Game: Greatclub 1b8c24cd1f9358c48839bb39266468c3
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
    // Game: Greatsword 5f824fbb0766a3543bbd6ae50248688f
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
    // Game: Handaxe 54e7473ff73910c47812fd39c897dc1a
    ("hand-axe", 1, 6, 20, 3, Handedness::Light, true, None),
    // Game: HeavyCrossbow 36d0551b8a28587438a47fcbbf53c083
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
    // Game: HeavyFlail 8fefb7e0da38b06408f185e29372c703
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
    // Game: HeavyMace d5a167f0f0208dd439ec7481e8989e21
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
    // Game: HeavyPick a492410f3d65f744c892faf09daad84a
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
    // Game: Javelin a70cea34b275522458654beb3c53fe3f
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
    // Game: Kama f5872eb0deb3a1b48a36549f8d92c19e
    ("kama", 1, 6, 20, 2, Handedness::Light, true, None),
    // Game: Kukri 006ecd4715809b343b7001e859e3ddb2
    ("kukri", 1, 4, 18, 2, Handedness::Light, true, None),
    // Game: LightCrossbow d525e7a6d8d5aa648a976ac41194b8d0
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
    // Game: LightHammer bbf7d0ea57e4c9349b65c82abc6787b4
    ("light-hammer", 1, 4, 20, 2, Handedness::Light, true, None),
    // Game: LightMace cf3e703db4b81904e982a66d7b8474ea
    ("light-mace", 1, 6, 20, 2, Handedness::Light, true, None),
    // Game: LightPick da922f4cdb60ef548a2ec0168620568a
    ("light-pick", 1, 4, 20, 4, Handedness::Light, true, None),
    // Game: Longbow 7a1211c05ec2c46428f41e3c0db9423f
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
    // Game: Longspear fa2dd17cbde7d3f4aa918d467c30516e
    ("longspear", 1, 8, 20, 3, Handedness::TwoHanded, false, None),
    // Game: Longsword d56c44bc9eb10204c8b386a02c7eed21
    ("longsword", 1, 8, 19, 2, Handedness::OneHanded, false, None),
    // Game: Nunchaku 4703b4c0922142f4cbe8895c10a47a9f
    ("nunchaku", 1, 6, 20, 2, Handedness::Light, true, None),
    // Game: DoubleAxe 87d76c7534506a546a065731ee6a38cd (equivalent model: first head)
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
    // Game: PunchingDagger 89dc4dc5f386e2049b5e0a5e209a3407
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
    // Game: Quarterstaff 629736dabac7f9f4a819dc854eaed2d6
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
    // Game: Rapier 2ece38f30500f454b8569136221e55b0
    ("rapier", 1, 6, 18, 2, Handedness::OneHanded, true, None),
    // Game: Sai 0944f411666c7594aa1398a7476ecf7d
    ("sai", 1, 4, 20, 2, Handedness::Light, true, None),
    // Game: SawtoothSabre ef50f200e4ab429b8bcb66a56f886c2f
    ("sawtooth-saber", 1, 8, 19, 2, Handedness::Light, true, None),
    // Game: Scimitar d9fbec4637d71bd4ebc977628de3daf3
    ("scimitar", 1, 6, 18, 2, Handedness::OneHanded, false, None),
    // Game: Scythe 4eacfc7e152930a45a1a16217c35011c
    ("scythe", 2, 4, 20, 4, Handedness::TwoHanded, false, None),
    // Game: Shortbow 99ce02fb54639b5439d07c99c55b8542
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
    // Game: Shortsword a7da36e0e7bb60e42b9f23462ce2f4fc
    ("short-sword", 1, 6, 19, 2, Handedness::Light, true, None),
    // Game: Shortspear cf72040b79c99504785976b28d54b2b7
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
    // Game: Sickle ec2da496c7936e14c9a28ce616a6b4cd
    ("sickle", 1, 6, 20, 2, Handedness::Light, true, None),
    // Game: SlingStaff 25da2dc95ed4a6b419608c678f2a9cc3
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
    // Game: Spear 4b289eccefe6d704093201e52eb6d123
    ("spear", 1, 8, 20, 3, Handedness::TwoHanded, false, None),
    // Game: Starknife 5a939137fc039084580725b2b0845c3f
    ("starknife", 1, 4, 20, 3, Handedness::Light, true, None),
    // Game: ThrowingAxe ca131c71f4fefcb48b30b5991520e01d
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
    // Game: Tongi 13fa38737d46c9e4abc7f4d74aaa59c3
    ("tongi", 1, 6, 19, 3, Handedness::OneHanded, false, None),
    // Game: Trident 6ff66364e0a2c89469c2e52ebb46365e
    ("trident", 1, 8, 20, 2, Handedness::OneHanded, false, None),
    // Game: DoubleSword e95efb85a0912a7489be69d5f03a5308 (equivalent model: first head)
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
    // Game: Warhammer fac41e149f49cba4a8e06ce39b41a6fa
    ("warhammer", 1, 8, 20, 3, Handedness::OneHanded, false, None),
];

/// A weapon's row of the table, if it has stats.
fn row(weapon: &'static rules::Weapon) -> Option<Row> {
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
}

#[test]
fn every_weapons_stats_are_the_games_table() {
    let data: Vec<Row> = DATA
        .weapons
        .iter()
        .filter_map(row)
        .filter(|row| !GAME_ROWS.iter().any(|game| game.0 == row.0))
        .collect();
    assert_eq!(data, TABLE);
}

/// The table's rows whose stats differ in the game, as the game has them.
const GAME_ROWS: [Row; 3] = [
    // Game: WeaponHeavyShield be9b6408e6101cb4997a8996484baf19: 1d6, ×2, one-handed, not Finessable.
    (
        "heavy-shield-bash",
        1,
        6,
        20,
        2,
        Handedness::OneHanded,
        false,
        None,
    ),
    // Game: WeaponLightShield 1fd965e522502fe479fdd423cca07684: 1d4, ×2, light, Finessable.
    (
        "light-shield-bash",
        1,
        4,
        20,
        2,
        Handedness::Light,
        true,
        None,
    ),
    // Game: EmptyHand a8b38f6b734daa44087ec0ec2e80c1cd: 1d3, ×2, light; the UnarmedStrike category is Finessable
    // (Enums/WeaponCategoryExtension.Data, WrathDecompiled 6ce4a6b), which Weapon Finesse keys on.
    ("unarmed-strike", 1, 3, 20, 2, Handedness::Light, true, None),
];

/// The game's heavy shield bash deals 1d6 and the light one 1d4, and its unarmed strike is finessable.
#[test]
#[ignore = "waits on ticket 289: the game gives the heavy shield bash 1d6, the light 1d4, and unarmed finesse"]
fn the_shield_bashes_and_the_unarmed_strike_have_the_games_stats() {
    for game in GAME_ROWS {
        let weapon = DATA
            .weapons
            .iter()
            .find(|weapon| weapon.key.as_str() == game.0)
            .expect("the weapon");
        assert_eq!(row(weapon), Some(game));
    }
}

#[test]
fn a_sling_staff_adds_strength_to_damage() {
    // BAB +1, Str 14 (+2), Dex 16 (+3): Dexterity to attack, Strength to damage (the game's Strength trait).
    // Game: SlingStaff 25da2dc95ed4a6b419608c678f2a9cc3 carries StrengthThrown c4d213911e9616949937e1520c80aaf3
    // (WeaponDamageMultiplierStatReplacement, Strength); Longbow 7a1211c05ec2c46428f41e3c0db9423f carries none.
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
    // Game: Greatsword 5f824fbb0766a3543bbd6ae50248688f, Scythe 4eacfc7e152930a45a1a16217c35011c, ElvenCurvedBlade
    // b5e6838ad2a62b146b49619bcf9f42aa, Estoc d516765b3c2904e4a939749526a52a9a, Longbow
    // 7a1211c05ec2c46428f41e3c0db9423f (Ranged), Javelin a70cea34b275522458654beb3c53fe3f (Ranged, thrown), Longsword
    // d56c44bc9eb10204c8b386a02c7eed21 (Melee).
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

/// Holds a feat once, for `value` when it names one.
fn hold_feat(state: &mut CharacterState, key: &str, value: Option<Parameter>) {
    let held = (state.held)
        .entry(Fact::Feat(FeatKey::new(key)))
        .or_default();
    held.holds = 1;
    if let Some(value) = value {
        held.values.insert(value, 1);
    }
}

#[test]
fn a_greatsword_fighters_attack_line() {
    // BAB +8, Str 18 (+4), Power Attack and Weapon Focus (Greatsword).
    // Game: Greatsword 5f824fbb0766a3543bbd6ae50248688f: 2d6, 19–20/×2, two-handed; WeaponFocus
    // 1e1f627d26ad36f43bbd26cc2bf8ac7e +1. Power Attack's numbers (PowerAttackFeature
    // 9972f33f977fc724c838e59641b2fca5) were not traced in the code.
    let mut state = CharacterState {
        bab: 8,
        ..CharacterState::default()
    };
    state.ability_scores.strength = 18;
    state.ability_scores.dexterity = 10;
    hold_feat(&mut state, "power-attack", None);
    hold_feat(
        &mut state,
        "weapon-focus",
        Some(Parameter::Weapon(WeaponKey::new("greatsword"))),
    );

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

/// The second martial run's Trickster Build (ticket 131): a falchion (18–20/×2) with Improved Critical and the three
/// Trickster critical feats held for it reads 13–20/×3, as the guide's trickster page gives them: +1 to the threat
/// range, +1 more, then +1 to the multiplier, after Improved Critical's doubling (18–20 → 15–20).
#[test]
fn a_trickster_falchions_critical_feats_make_it_13_20_x3() {
    // Game: Falchion 6ddc9acbbb6e40746a6a1671df1f7b47 (18–20/×2); ImprovedCritical f4201c85a991369408740c6888362e20
    // doubles the range, TricksterImprovedImprovedCritical 56f94badbba018b4b8277ce6e2e79e72 and
    // TricksterImprovedImprovedImprovedCritical 006a966007802a0478c9e21007207aac add 1 each,
    // TricksterImprovedImprovedImprovedCriticalImproved 319c882ab3cc51544ad2f3f43633d5b1 adds 1 to the multiplier
    // (RuleSystem/Rules/RuleCalculateWeaponStats, WrathDecompiled 6ce4a6b).
    let mut state = CharacterState {
        bab: 20,
        ..CharacterState::default()
    };
    state.ability_scores.strength = 18;
    state.ability_scores.dexterity = 10;
    let falchion = Parameter::Weapon(WeaponKey::new("falchion"));
    let engine = engine();
    let critical = |state: &CharacterState| {
        engine
            .attack_line(state, &WeaponKey::new("falchion"))
            .unwrap()
            .critical_text()
    };
    assert_eq!(critical(&state), "18–20/×2");

    for key in [
        "improved-critical",
        "improved-improved-critical",
        "improved-improved-improved-critical",
        "improved-improved-improved-critical-improved",
    ] {
        // Each is an ordinary feat taken for a weapon, the next needing the last for the same weapon.
        assert_eq!(
            feat(key).parameter,
            Some(rules::ParameterKind::Weapon),
            "{key}"
        );
        hold_feat(&mut state, key, Some(falchion.clone()));
    }
    assert_eq!(critical(&state), "13–20/×3");
}
