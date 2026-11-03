//! Archetype bonus-feat choices (ticket 61).

use std::collections::BTreeSet;

use rules::{Build, FeatSlot, Issue};

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

/// The lists are the guide's, each ignoring prerequisites. Massive Attacks: "a feat from the Ranger's Two-Weapon
/// Combat Style feats" (Double Slice, Shield Bash and Two-Weapon Fighting; Improved Two-Weapon Fighting from 6;
/// Greater Two-Weapon Fighting from 10), the style feats without the extras every ranger style offers, as the
/// Stormwalker's archery lists are the archery style's. Precise Companion: Precise Shot or Outflank. Captor: four
/// feats, six more from 12.
#[test]
fn the_bonus_feat_lists_are_the_guides() {
    let extras = feat_set(&[
        "dodge",
        "great-fortitude",
        "iron-will",
        "lightning-reflexes",
        "skill-focus",
        "toughness",
    ]);
    let two_weapon = [
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
    ];
    let names = [
        "Massive Attacks",
        "Massive Attacks (from level 6)",
        "Massive Attacks (from level 10)",
    ];
    for (i, level) in [2, 6, 10].into_iter().enumerate() {
        let list = any_feat_list(MASSIVE_ATTACKS[i]);
        assert_eq!(list_feats(MASSIVE_ATTACKS[i]), two_weapon[i], "{level}");
        assert_eq!(list.name, names[i]);
        assert_eq!(list.guide.0, "fighter#massive-attacks");
        assert!(list.ignores_prerequisites && !list.every_feat);
        let ranger = format!("ranger/combat-style-feat/two-weapon-combat-feats-{level}");
        assert_eq!(&list_feats(&ranger) - &extras, two_weapon[i], "{ranger}");
        let archery = format!("ranger/combat-style-feat/archery-feats-{level}");
        assert_eq!(
            &list_feats(&archery) - &extras,
            list_feats(STORMWALKER_ARCHERY[i]),
            "{archery}"
        );
        assert!(any_feat_list(STORMWALKER_ARCHERY[i]).ignores_prerequisites);
    }
    assert_eq!(
        archetype_feat_lists("fighter", TITAN),
        feat_set(&MASSIVE_ATTACKS)
    );

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
/// Precise Shot taken through Precise Companion is in a hunter's teamwork slot at 3. Improved Critical repeats once
/// per weapon, so a Stormwalker takes it at 10 and again for another weapon at 14.
#[test]
fn a_bonus_feat_choice_offers_a_held_feat_again_only_when_it_repeats() {
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
        .insert(class_slot(2), feat_pick_plain("precise-shot"));
    assert!(possible(&hunter));
    assert_eq!(
        feat_status(&hunter, &class_slot(3), "precise-shot"),
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

/// The hunter's, the Urban Hunter's and the Stormwalker's slots offer exactly their lists, prerequisites ignored as
/// their sections say ("She does not need to meet the prerequisites", "Neither she nor her companion need to meet the
/// prerequisites", the ranger's style feats), and the Forester has no Precise Companion.
#[test]
fn the_hunters_and_the_stormwalkers_choices_offer_their_lists_ignoring_prerequisites() {
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
            for option in engine().options(&build, &class_slot(*level)).unwrap() {
                assert_eq!(
                    option.status,
                    Ok(()),
                    "{list} at {level}: {:?}",
                    option.pick
                );
            }
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
