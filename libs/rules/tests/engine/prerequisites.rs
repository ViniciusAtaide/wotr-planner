//! A prerequisite tree's leaves (the single conditions under its groups and negations), and its normal form
//! (ticket 332).

use rules::{FeatKey, Issue, Prerequisite, RaceKey, Unmet};

use crate::support::*;

fn feat(key: &str) -> Prerequisite {
    Prerequisite::Feat(FeatKey::new(key))
}

#[test]
fn a_single_condition_is_its_own_only_leaf() {
    let condition = Prerequisite::BaseAttackBonus(4);
    assert_eq!(
        condition.leaves().collect::<Vec<_>>(),
        [&Prerequisite::BaseAttackBonus(4)]
    );
}

#[test]
fn the_leaves_of_nested_groups_come_left_to_right() {
    // An `Any` holding a feat, an `All`, an `AtLeast` and another feat.
    let tree = Prerequisite::Any(vec![
        feat("combat-expertise"),
        Prerequisite::All(vec![feat("sneak-attack"), Prerequisite::CharacterLevel(3)]),
        Prerequisite::AtLeast(
            2,
            vec![feat("dodge"), feat("mobility"), feat("spring-attack")],
        ),
        feat("improved-trip"),
    ]);
    assert_eq!(
        tree.leaves().collect::<Vec<_>>(),
        [
            &feat("combat-expertise"),
            &feat("sneak-attack"),
            &Prerequisite::CharacterLevel(3),
            &feat("dodge"),
            &feat("mobility"),
            &feat("spring-attack"),
            &feat("improved-trip"),
        ]
    );
}

#[test]
fn the_leaves_under_a_negation_are_the_trees_leaves_too() {
    let not = |child| Prerequisite::Not(Box::new(child));
    let tree = Prerequisite::All(vec![
        not(feat("dodge")),
        not(Prerequisite::Any(vec![
            feat("mobility"),
            not(not(feat("spring-attack"))),
        ])),
    ]);
    assert_eq!(
        tree.leaves().collect::<Vec<_>>(),
        [&feat("dodge"), &feat("mobility"), &feat("spring-attack")]
    );
}

#[test]
fn an_empty_group_has_no_leaves() {
    let tree = Prerequisite::All(vec![
        Prerequisite::Any(Vec::new()),
        Prerequisite::Not(Box::new(Prerequisite::AtLeast(1, Vec::new()))),
    ]);
    assert_eq!(tree.leaves().count(), 0);
}

// ---- the normal form (ticket 332) -----------------------------------------------------------------------------

fn not(child: Prerequisite) -> Prerequisite {
    Prerequisite::Not(Box::new(child))
}

fn all(children: Vec<Prerequisite>) -> Prerequisite {
    Prerequisite::All(children)
}

fn any(children: Vec<Prerequisite>) -> Prerequisite {
    Prerequisite::Any(children)
}

fn level(class: &str, level: u8) -> Prerequisite {
    Prerequisite::ClassLevel {
        class: rules::ClassKey::new(class),
        level,
    }
}

fn race(key: &str) -> Prerequisite {
    Prerequisite::Race(RaceKey::new(key))
}

/// The tree in its normal form.
fn canonical(mut tree: Prerequisite) -> Prerequisite {
    tree.canonicalize();
    tree
}

#[test]
fn an_and_inside_an_and_is_one_and() {
    let tree = all(vec![
        feat("toughness"),
        all(vec![
            Prerequisite::BaseAttackBonus(4),
            all(vec![feat("mobility"), feat("dodge")]),
        ]),
    ]);
    assert_eq!(
        canonical(tree),
        all(vec![
            feat("dodge"),
            feat("mobility"),
            feat("toughness"),
            Prerequisite::BaseAttackBonus(4),
        ])
    );
    // An "or" inside an "and" stays a group of its own.
    let mixed = all(vec![
        feat("dodge"),
        any(vec![feat("toughness"), feat("mobility")]),
    ]);
    assert_eq!(
        canonical(mixed),
        all(vec![
            any(vec![feat("mobility"), feat("toughness")]),
            feat("dodge"),
        ])
    );
}

#[test]
fn an_or_inside_an_or_is_one_or() {
    let tree = any(vec![
        feat("toughness"),
        any(vec![
            feat("dodge"),
            any(vec![feat("mobility"), Prerequisite::BaseAttackBonus(4)]),
        ]),
        all(vec![feat("power-attack"), Prerequisite::BaseAttackBonus(1)]),
    ]);
    assert_eq!(
        canonical(tree),
        any(vec![
            all(vec![feat("power-attack"), Prerequisite::BaseAttackBonus(1)]),
            feat("dodge"),
            feat("mobility"),
            feat("toughness"),
            Prerequisite::BaseAttackBonus(4),
        ])
    );
}

#[test]
fn at_least_one_is_an_or_and_a_lone_alternative_is_itself() {
    assert_eq!(
        canonical(Prerequisite::AtLeast(
            1,
            vec![feat("toughness"), feat("dodge")]
        )),
        any(vec![feat("dodge"), feat("toughness")])
    );
    // Kinetic Healer alone (the game's PrerequisiteFeaturesFromList of one, Amount 1, ticket 308f).
    assert_eq!(
        canonical(Prerequisite::AtLeast(1, vec![feat("dodge")])),
        feat("dodge")
    );
    // As an "or", it joins an "or" around it, and alone, an "and".
    assert_eq!(
        canonical(any(vec![
            feat("mobility"),
            Prerequisite::AtLeast(1, vec![feat("toughness"), feat("dodge")]),
        ])),
        any(vec![feat("dodge"), feat("mobility"), feat("toughness")])
    );
    assert_eq!(
        canonical(all(vec![
            level("kineticist", 8),
            Prerequisite::AtLeast(1, vec![feat("dodge")]),
        ])),
        all(vec![feat("dodge"), level("kineticist", 8)])
    );
}

#[test]
fn a_double_negation_is_its_condition() {
    assert_eq!(canonical(not(not(feat("dodge")))), feat("dodge"));
    assert_eq!(canonical(not(not(not(feat("dodge"))))), not(feat("dodge")));
    // What it holds then joins the group around it.
    assert_eq!(
        canonical(all(vec![
            feat("toughness"),
            not(not(all(vec![feat("mobility"), feat("dodge")]))),
        ])),
        all(vec![feat("dodge"), feat("mobility"), feat("toughness")])
    );
    // One negation inside a negated group is no double negation: the tree stays, sorted.
    assert_eq!(
        canonical(not(any(vec![feat("toughness"), not(feat("dodge"))]))),
        not(any(vec![not(feat("dodge")), feat("toughness")]))
    );
}

#[test]
fn a_group_of_one_is_its_child() {
    assert_eq!(canonical(all(vec![feat("dodge")])), feat("dodge"));
    assert_eq!(canonical(any(vec![feat("dodge")])), feat("dodge"));
    assert_eq!(
        canonical(any(vec![all(vec![any(vec![feat("dodge")])])])),
        feat("dodge")
    );
    assert_eq!(canonical(not(all(vec![feat("dodge")]))), not(feat("dodge")));
    // The child left alone joins the group around it.
    assert_eq!(
        canonical(any(vec![
            feat("toughness"),
            all(vec![any(vec![feat("mobility"), feat("dodge")])]),
        ])),
        any(vec![feat("dodge"), feat("mobility"), feat("toughness")])
    );
}

#[test]
fn at_least_two_keeps_its_count() {
    // Not an "and" when it counts every child, and not merged into another "n of".
    let every = Prerequisite::AtLeast(2, vec![feat("toughness"), feat("dodge")]);
    assert_eq!(
        canonical(every),
        Prerequisite::AtLeast(2, vec![feat("dodge"), feat("toughness")])
    );
    let nested = Prerequisite::AtLeast(
        2,
        vec![
            feat("toughness"),
            Prerequisite::AtLeast(
                2,
                vec![feat("mobility"), feat("dodge"), feat("power-attack")],
            ),
            any(vec![
                feat("dodge"),
                any(vec![feat("mobility"), feat("toughness")]),
            ]),
        ],
    );
    assert_eq!(
        canonical(nested),
        Prerequisite::AtLeast(
            2,
            vec![
                any(vec![feat("dodge"), feat("mobility"), feat("toughness")]),
                Prerequisite::AtLeast(
                    2,
                    vec![feat("dodge"), feat("mobility"), feat("power-attack")]
                ),
                feat("toughness"),
            ]
        )
    );
    // A lone child keeps the count it can't meet.
    assert_eq!(
        canonical(Prerequisite::AtLeast(2, vec![feat("dodge")])),
        Prerequisite::AtLeast(2, vec![feat("dodge")])
    );
}

#[test]
fn empty_groups_and_never_stay_as_they_are() {
    for tree in [
        all(Vec::new()),
        any(Vec::new()),
        not(all(Vec::new())),
        Prerequisite::AtLeast(2, Vec::new()),
    ] {
        assert_eq!(canonical(tree.clone()), tree);
        assert!(tree.is_canonical(), "{tree:?}");
    }
    assert_eq!(
        canonical(Prerequisite::AtLeast(1, Vec::new())),
        any(Vec::new())
    );
    // An empty "or" inside an "and" is a group of another kind: it stays.
    assert_eq!(
        canonical(all(vec![feat("dodge"), any(Vec::new())])),
        all(vec![any(Vec::new()), feat("dodge")])
    );
    // Duplicate children stay too.
    assert_eq!(
        canonical(all(vec![feat("dodge"), feat("dodge")])),
        all(vec![feat("dodge"), feat("dodge")])
    );
}

/// Trees of every shape the normal form rewrites, nested.
fn raw_trees() -> Vec<Prerequisite> {
    vec![
        all(vec![
            level("fighter", 2),
            all(vec![race("human"), not(not(level("fighter", 1)))]),
        ]),
        any(vec![
            race("elf"),
            any(vec![race("dwarf"), all(vec![level("rogue", 1)])]),
        ]),
        Prerequisite::AtLeast(1, vec![level("rogue", 2)]),
        Prerequisite::AtLeast(
            1,
            vec![
                race("elf"),
                Prerequisite::AtLeast(1, vec![level("fighter", 3)]),
            ],
        ),
        not(not(not(any(vec![race("human"), any(vec![race("elf")])])))),
        all(vec![
            Prerequisite::AtLeast(
                2,
                vec![
                    level("fighter", 1),
                    any(vec![race("elf"), any(vec![race("human")])]),
                    not(not(Prerequisite::CharacterLevel(3))),
                ],
            ),
            all(vec![not(all(vec![race("dwarf")]))]),
        ]),
        any(vec![
            all(vec![
                level("fighter", 1),
                any(vec![race("elf"), race("dwarf")]),
            ]),
            level("rogue", 1),
        ]),
    ]
}

#[test]
fn normalising_twice_changes_nothing() {
    for raw in raw_trees() {
        let once = canonical(raw.clone());
        assert!(once.is_canonical(), "{once:?}");
        assert_eq!(canonical(once.clone()), once);
        if raw != once {
            assert!(!raw.is_canonical(), "{raw:?}");
        }
    }
    assert!(!all(vec![feat("toughness"), feat("dodge")]).is_canonical());
}

/// The normal form means what the raw tree means: each tree is met, or not, alike in each state.
#[test]
fn a_normal_tree_is_met_as_its_raw_tree_is() {
    let engine = engine();
    let fighter = walked(&engine, &build("human", &times("fighter", 3)));
    let rogue = walked(&engine, &build("elf", &times("rogue", 2)));
    let states: Vec<&rules::CharacterState> = (1..=3)
        .filter_map(|level| fighter.at(level))
        .chain(rogue.at(1))
        .chain(rogue.at(2))
        .collect();
    let (mut met, mut unmet) = (0, 0);
    for raw in raw_trees() {
        let normal = canonical(raw.clone());
        for state in &states {
            let holds = state.check(&raw).is_ok();
            assert_eq!(state.check(&normal).is_ok(), holds, "{raw:?} as {normal:?}");
            if holds {
                met += 1;
            } else {
                unmet += 1;
            }
        }
    }
    assert!(met > 0 && unmet > 0, "met {met}, unmet {unmet}");
}

/// An "and" inside an "or" whose one failure is an "or": its alternatives are the outer "or"'s.
#[test]
fn a_failing_or_inside_an_or_reads_as_one_or() {
    let engine = engine();
    let fighter = walked(&engine, &build("human", &["fighter"]));
    let tree = canonical(any(vec![
        all(vec![
            level("fighter", 1),
            any(vec![race("elf"), race("dwarf")]),
        ]),
        level("rogue", 1),
    ]));
    let condition = |needs, has| Unmet::Condition { needs, has };
    assert_eq!(
        fighter.at(1).expect("level 1").check(&tree),
        Err(Unmet::Any(vec![
            condition(race("dwarf"), None),
            condition(race("elf"), None),
            condition(level("rogue", 1), Some(0)),
        ]))
    );
}

/// A class option whose class and archetype both fail reads as one "and" of every failure, the class's first.
#[test]
fn a_class_and_its_archetype_failing_read_as_one_and() {
    let engine = engine_with(|data| {
        class_mut(data, "sorcerer").prerequisite =
            Some(all(vec![feat("toughness"), feat("dodge")]));
        archetype_mut(data, "sorcerer/seeker").prerequisite = Some(all(vec![
            Prerequisite::BaseAttackBonus(5),
            feat("power-attack"),
        ]));
    });
    let options = engine
        .class_options(&build("human", &["fighter"]), 2)
        .expect("the class picker");
    let seeker = options
        .iter()
        .find(|option| {
            option.archetype.as_ref().map(rules::ArchetypeKey::as_str) == Some("sorcerer/seeker")
        })
        .expect("the Seeker");
    let condition = |needs, has| Unmet::Condition { needs, has };
    assert_eq!(
        seeker.status,
        Err(Issue::Unmet(Unmet::All(vec![
            condition(feat("dodge"), None),
            condition(feat("toughness"), None),
            condition(feat("power-attack"), None),
            condition(Prerequisite::BaseAttackBonus(5), Some(1)),
        ])))
    );
}
