//! Archetype bonus-feat choices (ticket 61).

use std::collections::BTreeSet;

use rules::{Build, FeatSlot, Issue, WeaponKey, WeaponProficiency};

use crate::support::*;

const TITAN: &str = "fighter/titan-fighter";
const URBAN_HUNTER: &str = "hunter/urban-hunter";
const STORMWALKER: &str = "ranger/stormwalker";

/// The features that stood for the four bonus-feat choices: each is a slot of its list now, as the Divine
/// Guardian's bonus feats are (a list and its slots, no feature).
const BONUS_FEAT_FEATURES: [(&str, &str); 4] = [
    ("fighter", "fighter/titan-fighter/massive-attacks"),
    ("hunter", "hunter/precise-companion"),
    ("hunter", "hunter/urban-hunter/captor"),
    ("ranger", "ranger/stormwalker/archery"),
];

/// Each choice's lists: the slot at each level picks from the list of the last threshold reached.
const MASSIVE_ATTACKS: [&str; 3] = [
    "fighter/titan-fighter/bonus-feats-2",
    "fighter/titan-fighter/bonus-feats-6",
    "fighter/titan-fighter/bonus-feats-10",
];
const PRECISE_COMPANION: &str = "hunter/bonus-feats-2";
const CAPTOR: [&str; 2] = [
    "hunter/urban-hunter/bonus-feats-6",
    "hunter/urban-hunter/bonus-feats-12",
];
const STORMWALKER_ARCHERY: [&str; 3] = [
    "ranger/bonus-feats-2b",
    "ranger/bonus-feats-6b",
    "ranger/bonus-feats-10b",
];

/// Every feat list of the data, by key.
fn any_feat_list(key: &str) -> &'static rules::FeatList {
    DATA.classes
        .iter()
        .flat_map(|class| {
            class
                .feat_lists
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.feat_lists))
        })
        .find(|list| list.key.as_str() == key)
        .unwrap_or_else(|| panic!("no feat list {key}"))
}

fn list_feats(key: &str) -> BTreeSet<String> {
    any_feat_list(key)
        .feats
        .iter()
        .map(|f| f.to_string())
        .collect()
}

fn feat_set(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|k| k.to_string()).collect()
}

/// The `OneOf` lists of the rows in force for `class` (with `archetype`) at levels 1 to 20, by level.
fn one_of_lists(class_key: &str, archetype: Option<&str>) -> Vec<(u8, String)> {
    let class = class(class_key);
    let archetype = archetype.map(|key| {
        class
            .archetypes
            .iter()
            .find(|a| a.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"))
    });
    (1..=20)
        .filter_map(|level| rules::effective_row(class, archetype, level))
        .flat_map(|row| {
            row.feat_slots
                .iter()
                .filter_map(move |slot| match slot {
                    FeatSlot::OneOf { list } => Some((row.level, list.to_string())),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The rows in force hold `lists` (level, list) among their `OneOf` slots, and no other slot of those lists.
fn holds_lists(class_key: &str, archetype: Option<&str>, lists: &[(u8, &str)]) {
    let names: BTreeSet<&str> = lists.iter().map(|(_, list)| *list).collect();
    let found: Vec<(u8, String)> = one_of_lists(class_key, archetype)
        .into_iter()
        .filter(|(_, list)| names.contains(list.as_str()))
        .collect();
    let expected: Vec<(u8, String)> = lists
        .iter()
        .map(|(level, list)| (*level, list.to_string()))
        .collect();
    assert_eq!(found, expected, "{class_key} {archetype:?}");
}

/// The guide's sentences, as slots: the bonus-feat choice is a pick, and the feature that stood for it is gone from
/// its class's (or archetype's) features and from every row. Nothing needed one.
#[test]
fn the_bonus_feat_choices_replace_their_features() {
    // Game: each choice is a BlueprintFeatureSelection: TitanFighterStyleTwoWeaponSelection2
    // 9c3e4b49b9f243f080c0432d2a1e5cbf, HunterPreciseCompanion 75e9a5179aeff4140a8235ebdeaf8df7,
    // UrbanHunterCaptorSelection6 a27990877466ec04e80d7cfa967ed69b, RangerStyleArcherySelection2
    // 3cf94df3f1cf9a443a880bc1ae9be3c6 (their keys and the absence of a feature are the data's shape)
    for (class_key, feature) in BONUS_FEAT_FEATURES {
        let class = class(class_key);
        let defined = class
            .features
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.features))
            .any(|f| f.key.as_str() == feature);
        assert!(!defined, "{feature} is still a feature");
        let on_a_row = class
            .levels
            .iter()
            .map(|row| &row.features)
            .chain(
                class
                    .archetypes
                    .iter()
                    .flat_map(|a| a.levels.iter().map(|row| &row.features)),
            )
            .flatten()
            .any(|f| f.as_str() == feature);
        assert!(!on_a_row, "{feature} is still on a row");
    }
}

/// Each choice is a slot where its table has the cell: Massive Attacks at 2 and every four levels (in place of the
/// fighter's Bonus Combat Feat there), Precise Companion at 2 for the hunter and every archetype but the Forester,
/// Captor at 6 and every three levels, the Stormwalker's Archery at 2, 6, 10, 14 and 18.
#[test]
fn each_bonus_feat_choice_is_a_slot_at_its_tables_levels() {
    // Game: TitanFighterArchetype 88a9dbb92a674ed995bae800896795f5 (2, 6, 10, 14, 18, in place of the fighter's feat);
    // HunterProgression f4943530be50c3c4da841a7b96cfae98 (Precise Companion at 2), ForesterArchetype
    // 74ee8172df6782f4792ea0f8647dea34 (removes it, adds a combat feat); UrbanHunterArchetype
    // f1341c61c77092643ae63d7cb52f83f8 (Captor 6, 9, 12, 15, 18); StormwalkerArchetype a330c05e484d98748a462acd43654a3a
    // (2, 6, 10, 14, 18)
    holds_lists(
        "fighter",
        Some(TITAN),
        &[
            (2, MASSIVE_ATTACKS[0]),
            (6, MASSIVE_ATTACKS[1]),
            (10, MASSIVE_ATTACKS[2]),
            (14, MASSIVE_ATTACKS[2]),
            (18, MASSIVE_ATTACKS[2]),
        ],
    );
    let titan_combat: Vec<u8> = {
        let class = class("fighter");
        let titan = class.archetypes.iter().find(|a| a.key.as_str() == TITAN);
        (1..=20)
            .filter(|level| {
                rules::effective_row(class, titan, *level)
                    .is_some_and(|row| row.feat_slots.contains(&FeatSlot::CombatFeat))
            })
            .collect()
    };
    assert_eq!(titan_combat, [1, 4, 8, 12, 16, 20]);

    holds_lists("hunter", None, &[(2, PRECISE_COMPANION)]);
    for archetype in &class("hunter").archetypes {
        let expected: &[(u8, &str)] = if archetype.key.as_str() == "hunter/forester" {
            &[]
        } else {
            &[(2, PRECISE_COMPANION)]
        };
        holds_lists("hunter", Some(archetype.key.as_str()), expected);
    }
    holds_lists(
        "hunter",
        Some(URBAN_HUNTER),
        &[
            (6, CAPTOR[0]),
            (9, CAPTOR[0]),
            (12, CAPTOR[1]),
            (15, CAPTOR[1]),
            (18, CAPTOR[1]),
        ],
    );
    holds_lists(
        "ranger",
        Some(STORMWALKER),
        &[
            (2, STORMWALKER_ARCHERY[0]),
            (6, STORMWALKER_ARCHERY[1]),
            (10, STORMWALKER_ARCHERY[2]),
            (14, STORMWALKER_ARCHERY[2]),
            (18, STORMWALKER_ARCHERY[2]),
        ],
    );
    // Massive Attacks is the Titan Fighter's alone.
    for (class_key, archetype) in std::iter::once(("fighter", None)).chain(
        class("fighter")
            .archetypes
            .iter()
            .filter(|a| a.key.as_str() != TITAN)
            .map(|a| ("fighter", Some(a.key.as_str()))),
    ) {
        holds_lists(class_key, archetype, &[]);
        assert!(
            one_of_lists(class_key, archetype)
                .iter()
                .all(|(_, list)| !MASSIVE_ATTACKS.contains(&list.as_str())),
            "{archetype:?}"
        );
    }
}

/// The feats every ranger style list offers besides its own, as the data's style lists hold them.
fn style_extras() -> BTreeSet<String> {
    feat_set(&[
        "dodge",
        "great-fortitude",
        "iron-will",
        "lightning-reflexes",
        "skill-focus",
        "toughness",
    ])
}

/// Massive Attacks' core feats at 2, 6 and 10: Double Slice, Shield Bash and Two-Weapon Fighting; Improved Two-Weapon
/// Fighting from 6; Greater Two-Weapon Fighting from 10.
fn two_weapon_core() -> [BTreeSet<String>; 3] {
    [
        feat_set(&["double-slice", "shield-bash", "two-weapon-fighting"]),
        feat_set(&[
            "double-slice",
            "improved-two-weapon-fighting",
            "shield-bash",
            "two-weapon-fighting",
        ]),
        feat_set(&[
            "double-slice",
            "greater-two-weapon-fighting",
            "improved-two-weapon-fighting",
            "shield-bash",
            "two-weapon-fighting",
        ]),
    ]
}

/// The lists, each ignoring prerequisites. Massive Attacks' core: Double Slice, Shield Bash and Two-Weapon Fighting;
/// Improved Two-Weapon Fighting from 6; Greater Two-Weapon Fighting from 10. Precise Companion: Precise Shot or
/// Outflank. Captor: four feats, six more from 12. Which extras the Massive Attacks and Stormwalker lists hold is the
/// game's (`the_titans_massive_attacks_lists_hold_the_style_extras` and `the_stormwalkers_lists_are_the_rangers_archery_lists`, waiting on tickets).
#[test]
fn the_bonus_feat_lists_are_the_guides() {
    let extras = style_extras();
    let two_weapon = two_weapon_core();
    let names = [
        "Massive Attacks",
        "Massive Attacks (from level 6)",
        "Massive Attacks (from level 10)",
    ];
    for (i, level) in [2, 6, 10].into_iter().enumerate() {
        let list = any_feat_list(MASSIVE_ATTACKS[i]);
        // Game: TitanFighterStyleTwoWeaponSelection2 9c3e4b49b9f243f080c0432d2a1e5cbf, …6
        // 855b2b531df740cfa4536d0ba0729fd8, …10 0e41c3c5e88c42c3af31162743982c22 (IgnorePrerequisites)
        assert!(list.ignores_prerequisites && !list.every_feat);
        assert_eq!(list.name, names[i]);
        assert_eq!(list.guide.as_ref().unwrap().0, "fighter#massive-attacks");
        // Game: RangerStyleTwoWeaponSelection2 019e68517c95c5447bff125b8a91c73f, …6 59d5445392ac62245a5ece9b01c05ee8,
        // …10 ee907845112b8aa4e907cf326e6142a6 (the core feats above and the five extras)
        let ranger = format!("ranger/combat-style-feat/two-weapon-combat-feats-{level}");
        assert_eq!(&list_feats(&ranger) - &extras, two_weapon[i], "{ranger}");
        // Game: RangerStyleArcherySelection2 3cf94df3f1cf9a443a880bc1ae9be3c6, …6 6c799d09d5b93f344b9ade0e0c765c2d,
        // …10 7ef950ca681955d47bc4efbe77073e2c (IgnorePrerequisites)
        assert!(any_feat_list(STORMWALKER_ARCHERY[i]).ignores_prerequisites);
    }
    assert_eq!(
        archetype_feat_lists("fighter", TITAN),
        feat_set(&MASSIVE_ATTACKS)
    );

    // Game: HunterPreciseCompanion 75e9a5179aeff4140a8235ebdeaf8df7 (Outflank, Precise Shot; IgnorePrerequisites)
    let precise = any_feat_list(PRECISE_COMPANION);
    assert_eq!(
        list_feats(PRECISE_COMPANION),
        feat_set(&["outflank", "precise-shot"])
    );
    assert_eq!(precise.name, "Precise Companion");
    assert!(precise.ignores_prerequisites);

    let captor_6 = feat_set(&["bull-rush", "dirty-trick", "disarm", "trip"]);
    let captor_12 = &captor_6
        | &feat_set(&[
            "combat-reflexes",
            "greater-bull-rush",
            "greater-dirty-trick",
            "greater-disarm",
            "greater-trip",
            "lunge",
        ]);
    // Game: UrbanHunterCaptorSelection6 a27990877466ec04e80d7cfa967ed69b; UrbanHunterCaptorSelection12
    // 9839caff86642584bbf07699e8238c1b (IgnorePrerequisites; the Improved maneuver feats, as the data's keys)
    assert_eq!(list_feats(CAPTOR[0]), captor_6);
    assert_eq!(list_feats(CAPTOR[1]), captor_12);
    assert_eq!(any_feat_list(CAPTOR[0]).name, "Captor");
    assert_eq!(any_feat_list(CAPTOR[1]).name, "Captor (from level 12)");
    assert!(
        CAPTOR
            .iter()
            .all(|key| any_feat_list(key).ignores_prerequisites)
    );
}

/// The game's Massive Attacks lists are the ranger's two-weapon style selections, the five extras (Dodge, Great
/// Fortitude, Iron Will, Lightning Reflexes, Toughness) included.
// Game: TitanFighterStyleTwoWeaponSelection2 9c3e4b49b9f243f080c0432d2a1e5cbf, …6 855b2b531df740cfa4536d0ba0729fd8,
// …10 0e41c3c5e88c42c3af31162743982c22 (the core feats and the five extras)
#[test]
#[ignore = "waits on ticket 285: the game gives Massive Attacks the five style extras"]
fn the_titans_massive_attacks_lists_hold_the_style_extras() {
    let extras = &style_extras() - &feat_set(&["skill-focus"]);
    let two_weapon = two_weapon_core();
    for (i, level) in [2, 6, 10].into_iter().enumerate() {
        assert_eq!(
            list_feats(MASSIVE_ATTACKS[i]),
            &two_weapon[i] | &extras,
            "{level}"
        );
    }
}

/// The game's Stormwalker slots are the ranger's archery style selections themselves, extras included.
// Game: StormwalkerArchetype a330c05e484d98748a462acd43654a3a adds RangerStyleArcherySelection2
// 3cf94df3f1cf9a443a880bc1ae9be3c6, …6 6c799d09d5b93f344b9ade0e0c765c2d, …10 7ef950ca681955d47bc4efbe77073e2c
#[test]
#[ignore = "waits on ticket 275: the game gives the Stormwalker the ranger's archery lists, extras included"]
fn the_stormwalkers_lists_are_the_rangers_archery_lists() {
    for (i, level) in [2, 6, 10].into_iter().enumerate() {
        let archery = format!("ranger/combat-style-feat/archery-feats-{level}");
        assert_eq!(
            list_feats(STORMWALKER_ARCHERY[i]),
            list_feats(&archery),
            "{level}"
        );
    }
}

/// The keys of the feat lists `archetype` defines.
fn archetype_feat_lists(class_key: &str, archetype: &str) -> BTreeSet<String> {
    class(class_key)
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == archetype)
        .unwrap_or_else(|| panic!("no {archetype}"))
        .feat_lists
        .iter()
        .map(|l| l.key.to_string())
        .collect()
}

/// A Titan Fighter's Massive Attacks slots offer exactly their list, open whatever the feats' prerequisites (Two-Weapon
/// Fighting needs Dexterity 15, Greater Two-Weapon Fighting Improved Two-Weapon Fighting), and a filled Titan 18 is
/// possible.
#[test]
fn a_titan_fighter_picks_massive_attacks_from_the_two_weapon_style_feats_ignoring_prerequisites() {
    // Game: TitanFighterStyleTwoWeaponSelection2 9c3e4b49b9f243f080c0432d2a1e5cbf (IgnorePrerequisites; Greater
    // Two-Weapon Fighting only in …10 0e41c3c5e88c42c3af31162743982c22); BlueprintFeatureSelection.CanSelect,
    // WrathDecompiled 6ce4a6b. The offered set is the data's list (its extras wait on a ticket, see above).
    let titan = archetype_build("fighter", TITAN, 18);
    for (level, list) in [
        (2, MASSIVE_ATTACKS[0]),
        (6, MASSIVE_ATTACKS[1]),
        (10, MASSIVE_ATTACKS[2]),
        (14, MASSIVE_ATTACKS[2]),
        (18, MASSIVE_ATTACKS[2]),
    ] {
        assert_eq!(
            feats_offered(&titan, &class_slot(level)),
            list_feats(list),
            "{level}"
        );
        for feat in list_feats(list) {
            assert_eq!(
                feat_status(&titan, &class_slot(level), &feat),
                Ok(()),
                "{feat} at {level}"
            );
        }
    }
    let mut filled = titan.clone();
    for (level, feat) in [
        (2, "two-weapon-fighting"),
        (6, "double-slice"),
        (10, "greater-two-weapon-fighting"),
        (14, "improved-two-weapon-fighting"),
        (18, "shield-bash"),
    ] {
        filled
            .picks
            .insert(class_slot(level), feat_pick_plain(feat));
    }
    assert!(possible(&filled));
    // Greater Two-Weapon Fighting isn't on the list before 10.
    let mut early = titan.clone();
    early.picks.insert(
        class_slot(6),
        feat_pick_plain("greater-two-weapon-fighting"),
    );
    assert!(!possible(&early));
}

/// A feat held is not offered again unless it repeats: Two-Weapon Fighting taken at 2 is "Already taken" at 6, and
/// Outflank taken through Precise Companion is in a hunter's teamwork slot at 3. Improved Critical repeats once per
/// weapon, so a Stormwalker takes it at 10 and again for another weapon at 14.
#[test]
fn a_bonus_feat_choice_offers_a_held_feat_again_only_when_it_repeats() {
    // Game: BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b (a feature held at its Ranks isn't selected
    // again, prerequisites ignored or not); ImprovedCritical f4201c85a991369408740c6888362e20 (one per weapon);
    // HunterTeamworkFeatSelection 01046afc774beee48abde8e35da0f4ba (Outflank among its 13)
    let mut titan = archetype_build("fighter", TITAN, 6);
    titan
        .picks
        .insert(class_slot(2), feat_pick_plain("two-weapon-fighting"));
    assert!(possible(&titan));
    assert_eq!(
        feat_status(&titan, &class_slot(6), "two-weapon-fighting"),
        Err(Issue::AlreadyHeld)
    );
    titan
        .picks
        .insert(class_slot(6), feat_pick_plain("two-weapon-fighting"));
    assert_eq!(why_not(&titan, &class_slot(6)), Some(Issue::AlreadyHeld));

    let mut hunter = build("hunter", 3);
    hunter
        .picks
        .insert(class_slot(2), feat_pick_plain("outflank"));
    assert!(possible(&hunter));
    assert_eq!(
        feat_status(&hunter, &class_slot(3), "outflank"),
        Err(Issue::AlreadyHeld)
    );

    let mut stormwalker = archetype_build("ranger", STORMWALKER, 14);
    stormwalker.picks.insert(
        class_slot(10),
        feat_pick("improved-critical", weapon("long-bow")),
    );
    stormwalker.picks.insert(
        class_slot(14),
        feat_pick("improved-critical", weapon("longsword")),
    );
    assert!(possible(&stormwalker));
    stormwalker.picks.insert(
        class_slot(14),
        feat_pick("improved-critical", weapon("long-bow")),
    );
    assert_eq!(
        why_not(&stormwalker, &class_slot(14)),
        Some(Issue::AlreadyHeld)
    );
}

/// The game's hunter teamwork list at 3 has no Precise Shot.
// Game: HunterTeamworkFeatSelection 01046afc774beee48abde8e35da0f4ba (13 teamwork feats, no PreciseShot)
#[test]
#[ignore = "waits on ticket 275: the game gives the hunter's level-3 teamwork list 13 feats and no Precise Shot"]
fn a_hunters_teamwork_slot_at_3_offers_no_precise_shot() {
    let hunter = build("hunter", 3);
    assert!(!feats_offered(&hunter, &class_slot(3)).contains("precise-shot"));
}

/// The hunter's, the Urban Hunter's and the Stormwalker's slots offer exactly their lists, prerequisites ignored as
/// their sections say ("She does not need to meet the prerequisites", "Neither she nor her companion need to meet the
/// prerequisites", the ranger's style feats), and the Forester has no Precise Companion.
#[test]
fn the_hunters_and_the_stormwalkers_choices_offer_their_lists_ignoring_prerequisites() {
    // Game: HunterPreciseCompanion 75e9a5179aeff4140a8235ebdeaf8df7, UrbanHunterCaptorSelection6
    // a27990877466ec04e80d7cfa967ed69b, UrbanHunterCaptorSelection12 9839caff86642584bbf07699e8238c1b,
    // RangerStyleArcherySelection2 3cf94df3f1cf9a443a880bc1ae9be3c6 (IgnorePrerequisites); ForesterArchetype
    // 74ee8172df6782f4792ea0f8647dea34 (a combat feat at 2). The offered sets are the data's lists.
    let cases: [(Build, &[(u8, &str)]); 3] = [
        (build("hunter", 2), &[(2, PRECISE_COMPANION)]),
        (
            archetype_build("hunter", URBAN_HUNTER, 18),
            &[
                (6, CAPTOR[0]),
                (9, CAPTOR[0]),
                (12, CAPTOR[1]),
                (15, CAPTOR[1]),
                (18, CAPTOR[1]),
            ],
        ),
        (
            archetype_build("ranger", STORMWALKER, 18),
            &[
                (2, STORMWALKER_ARCHERY[0]),
                (6, STORMWALKER_ARCHERY[1]),
                (10, STORMWALKER_ARCHERY[2]),
                (14, STORMWALKER_ARCHERY[2]),
                (18, STORMWALKER_ARCHERY[2]),
            ],
        ),
    ];
    for (build, slots) in cases {
        for (level, list) in slots {
            assert_eq!(
                feats_offered(&build, &class_slot(*level)),
                list_feats(list),
                "{list} at {level}"
            );
            // Every option is open but Improved Critical in an exotic weapon in the Stormwalker's list from 10: a
            // feat that requires proficiency is taken only for a weapon the character is proficient with, whatever
            // the list ignores (ticket 251). Game: ImprovedCritical f4201c85a991369408740c6888362e20 sets
            // RequireProficiency, which BlueprintParametrizedFeature.CanSelect checks whatever the selection ignores
            // (WrathDecompiled 6ce4a6b); RangerStyleArcherySelection10 7ef950ca681955d47bc4efbe77073e2c offers it;
            // RangerProficiencies c5e479367d07d62428f2fe92f39c0341 gives simple and martial weapons, and
            // StormwalkerArchetype a330c05e484d98748a462acd43654a3a keeps it. The Hunter's and the Urban Hunter's
            // lists hold no feat that requires proficiency.
            let refused: BTreeSet<(String, WeaponKey)> = engine()
                .options(&build, &class_slot(*level))
                .unwrap()
                .into_iter()
                .filter_map(|option| match option.status {
                    Ok(()) => None,
                    Err(Issue::NotProficient { weapon }) => {
                        Some((option.pick.key().to_string(), weapon))
                    }
                    Err(issue) => panic!("{list} at {level}: {:?} {issue:?}", option.pick),
                })
                .collect();
            let expected = if *list == STORMWALKER_ARCHERY[2] {
                (DATA.weapons.iter())
                    .filter(|weapon| weapon.proficiency == Some(WeaponProficiency::Exotic))
                    .map(|weapon| ("improved-critical".to_string(), weapon.key.clone()))
                    .collect()
            } else {
                BTreeSet::new()
            };
            assert_eq!(refused, expected, "{list} at {level}");
        }
    }
    let forester = archetype_build("hunter", "hunter/forester", 2);
    assert!(
        !feats_offered(&forester, &class_slot(2)).contains("outflank"),
        "the Forester's level-2 slot is a combat feat"
    );
}

/// The sheet labels each new slot by its list's name.
#[test]
fn the_bonus_feat_slots_read_their_lists_names() {
    let engine = engine();
    for (build, labels) in [
        (
            archetype_build("fighter", TITAN, 18),
            vec![
                (2, "Massive Attacks"),
                (6, "Massive Attacks (from level 6)"),
                (10, "Massive Attacks (from level 10)"),
                (18, "Massive Attacks (from level 10)"),
                (4, "Combat feat"),
            ],
        ),
        (build("hunter", 2), vec![(2, "Precise Companion")]),
        (
            archetype_build("hunter", URBAN_HUNTER, 12),
            vec![(6, "Captor"), (12, "Captor (from level 12)")],
        ),
        (
            archetype_build("ranger", STORMWALKER, 10),
            vec![
                (2, "Archery Feats"),
                (6, "Archery Feats (from level 6)"),
                (10, "Archery Feats (from level 10)"),
            ],
        ),
    ] {
        let sheet = engine.evaluate(&build);
        for (level, label) in labels {
            assert_eq!(
                engine.slot_label(sheet.slot(&class_slot(level)).expect("a class slot")),
                label,
                "{level}"
            );
        }
    }
}
