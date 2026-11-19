//! A prerequisite tree's leaves (the single conditions under its groups and negations), and its normal form
//! (ticket 332).

use rules::{FeatKey, Issue, Leaf, Prerequisite, RaceKey, Reading, Unmet};

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
    // An empty group directly in a group of its kind gives no children: it goes, and what means the same is left.
    assert_eq!(
        canonical(all(vec![feat("dodge"), all(Vec::new())])),
        feat("dodge")
    );
    assert_eq!(
        canonical(any(vec![feat("toughness"), any(Vec::new()), feat("dodge")])),
        any(vec![feat("dodge"), feat("toughness")])
    );
}

/// An "and" or an "or" of one condition twice is of it once, those flattening brings together too (the game's
/// `PrerequisiteArchetypeLevel` beside a `PrerequisiteClassLevel` of the same class and level); in an "n of" each
/// counts, so they stay.
#[test]
fn equal_children_are_one_in_an_and_or_an_or_and_stay_in_an_at_least() {
    assert_eq!(
        canonical(all(vec![feat("dodge"), feat("dodge")])),
        feat("dodge")
    );
    assert_eq!(
        canonical(all(vec![
            level("kineticist", 16),
            all(vec![level("kineticist", 16), feat("dodge")]),
        ])),
        all(vec![feat("dodge"), level("kineticist", 16)])
    );
    assert_eq!(
        canonical(any(vec![
            feat("dodge"),
            any(vec![feat("dodge"), feat("toughness")])
        ])),
        any(vec![feat("dodge"), feat("toughness")])
    );
    assert_eq!(
        canonical(Prerequisite::AtLeast(
            2,
            vec![feat("toughness"), feat("dodge"), feat("dodge")]
        )),
        Prerequisite::AtLeast(2, vec![feat("dodge"), feat("dodge"), feat("toughness")])
    );
}

#[test]
fn at_least_none_keeps_its_count_and_normalises_its_children() {
    assert_eq!(
        canonical(Prerequisite::AtLeast(
            0,
            vec![
                feat("toughness"),
                not(not(feat("dodge"))),
                all(vec![feat("mobility")]),
            ]
        )),
        Prerequisite::AtLeast(0, vec![feat("dodge"), feat("mobility"), feat("toughness")])
    );
}

/// Trees of every shape the normal form rewrites, nested.
fn raw_trees() -> Vec<Prerequisite> {
    vec![
        all(vec![level("rogue", 1), all(Vec::new())]),
        Prerequisite::AtLeast(1, Vec::new()),
        any(vec![race("human"), any(Vec::new())]),
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

// ---- the polarity-aware view (ticket 333) -----------------------------------------------------------------------

fn leaf(condition: &Prerequisite, refused: bool) -> Leaf<'_> {
    Leaf { condition, refused }
}

#[test]
fn a_leaf_under_one_not_is_refused_and_under_two_is_needed() {
    let tree = all(vec![
        feat("a"),
        not(feat("b")),
        not(not(feat("c"))),
        not(any(vec![feat("d"), not(feat("e"))])),
    ]);
    let seen: Vec<(&Prerequisite, bool)> = tree
        .polar_leaves()
        .map(|l| (l.condition, l.refused))
        .collect();
    let (a, b, c, d, e) = (feat("a"), feat("b"), feat("c"), feat("d"), feat("e"));
    assert_eq!(
        seen,
        [
            (&a, false),
            (&b, true),
            (&c, false),
            (&d, true),
            (&e, false)
        ]
    );
}

#[test]
fn the_reading_pushes_a_not_onto_its_leaves() {
    let (a, b, c) = (feat("a"), feat("b"), feat("c"));
    let refused = |p| Reading::Leaf(leaf(p, true));
    // `Not(Any)` is an `All` of refused leaves.
    let tree = not(any(vec![a.clone(), b.clone()]));
    assert_eq!(tree.reading(), Reading::All(vec![refused(&a), refused(&b)]));
    // `Not(All)` is an `Any` of them.
    let tree = not(all(vec![a.clone(), b.clone()]));
    assert_eq!(tree.reading(), Reading::Any(vec![refused(&a), refused(&b)]));
    // `Not(AtLeast(2, 3 children))` needs 2 refused of the 3.
    let tree = not(Prerequisite::AtLeast(
        2,
        vec![a.clone(), b.clone(), c.clone()],
    ));
    assert_eq!(
        tree.reading(),
        Reading::AtLeast(2, vec![refused(&a), refused(&b), refused(&c)])
    );
    // `Not(Not(x))` is `x`.
    assert_eq!(
        not(not(a.clone())).reading(),
        Reading::Leaf(leaf(&a, false))
    );
}

#[test]
fn a_reading_of_a_not_inside_an_all_merges_into_it() {
    let (a, b, c) = (feat("a"), feat("b"), feat("c"));
    let tree = all(vec![a.clone(), not(any(vec![b.clone(), c.clone()]))]);
    let reading = tree.reading();
    assert_eq!(
        reading,
        Reading::All(vec![
            Reading::Leaf(leaf(&a, false)),
            Reading::Leaf(leaf(&b, true)),
            Reading::Leaf(leaf(&c, true)),
        ])
    );
    assert_eq!(reading.parts().len(), 3);
    // Not an `All`: itself alone.
    let lone = a.reading();
    assert_eq!(lone.parts(), [Reading::Leaf(leaf(&a, false))]);
    // The reading's leaves run left to right.
    assert_eq!(reading.leaves().count(), 3);
}

#[test]
fn each_fact_leaf_names_its_fact() {
    use rules::{
        ArchetypeKey, ClassKey, DeityKey, Fact, FeatKey, FeatureKey, FeatureSelectionKey,
        MythicAbilityKey, MythicFeatKey, RaceKey, SelectionOptionKey,
    };
    let feat_key = FeatKey::new("f");
    let cases: Vec<(Prerequisite, Option<Fact>)> = vec![
        (
            Prerequisite::Feat(feat_key.clone()),
            Some(Fact::Feat(feat_key.clone())),
        ),
        (
            Prerequisite::FeatForSameValue(feat_key.clone()),
            Some(Fact::Feat(feat_key.clone())),
        ),
        (
            Prerequisite::MythicFeat(MythicFeatKey::new("m")),
            Some(Fact::MythicFeat(MythicFeatKey::new("m"))),
        ),
        (
            Prerequisite::MythicAbility(MythicAbilityKey::new("m")),
            Some(Fact::MythicAbility(MythicAbilityKey::new("m"))),
        ),
        (
            Prerequisite::ClassLevel {
                class: ClassKey::new("c"),
                level: 3,
            },
            Some(Fact::Class(ClassKey::new("c"))),
        ),
        (
            Prerequisite::Archetype(ArchetypeKey::new("a")),
            Some(Fact::Archetype(ArchetypeKey::new("a"))),
        ),
        (
            Prerequisite::Feature {
                feature: FeatureKey::new("x"),
                count: 2,
            },
            Some(Fact::Feature(FeatureKey::new("x"))),
        ),
        (
            Prerequisite::Rank {
                fact: Fact::Feat(feat_key.clone()),
                rank: 3,
            },
            Some(Fact::Feat(feat_key.clone())),
        ),
        (
            Prerequisite::SelectionOption(SelectionOptionKey::new("o")),
            Some(Fact::Option(SelectionOptionKey::new("o"))),
        ),
        (
            Prerequisite::Selection(FeatureSelectionKey::new("s")),
            Some(Fact::Selection(FeatureSelectionKey::new("s"))),
        ),
        (
            Prerequisite::Race(RaceKey::new("r")),
            Some(Fact::Race(RaceKey::new("r"))),
        ),
        (
            Prerequisite::Deity(DeityKey::new("d")),
            Some(Fact::Deity(DeityKey::new("d"))),
        ),
        // No fact: a group, a `Not`, a score, a selection read as none held.
        (all(vec![feat("a")]), None),
        (not(feat("a")), None),
        (Prerequisite::BaseAttackBonus(4), None),
        (
            Prerequisite::NoOptionFrom(FeatureSelectionKey::new("s")),
            None,
        ),
    ];
    for (condition, fact) in cases {
        assert_eq!(condition.fact(), fact, "{condition:?}");
    }
}

#[test]
fn a_leaf_says_its_threshold_and_value() {
    use rules::{ClassKey, Fact, Parameter, SkillKey};
    let value = Parameter::Skill(SkillKey::new("stealth"));
    let with_value = Prerequisite::FeatFor {
        feat: FeatKey::new("f"),
        value: value.clone(),
    };
    let if_value = Prerequisite::FeatIfValue {
        value: value.clone(),
        feat: FeatKey::new("f"),
    };
    assert_eq!(leaf(&with_value, false).value(), Some(&value));
    assert_eq!(leaf(&if_value, false).value(), Some(&value));
    assert_eq!(leaf(&feat("f"), false).value(), None);
    let thresholds = [
        (
            Prerequisite::ClassLevel {
                class: ClassKey::new("c"),
                level: 5,
            },
            5,
        ),
        (
            Prerequisite::Rank {
                fact: Fact::Feat(FeatKey::new("f")),
                rank: 3,
            },
            3,
        ),
        (
            Prerequisite::SkillRanks {
                skill: SkillKey::new("s"),
                ranks: 7,
            },
            7,
        ),
        (Prerequisite::BaseAttackBonus(6), 6),
        (Prerequisite::CasterLevel(9), 9),
        (Prerequisite::CharacterLevel(11), 11),
        (Prerequisite::MythicRank(4), 4),
    ];
    for (condition, number) in thresholds {
        assert_eq!(
            leaf(&condition, true).threshold(),
            Some(number),
            "{condition:?}"
        );
    }
    assert_eq!(leaf(&feat("f"), false).threshold(), None);
}

#[test]
fn the_unmet_leaves_carry_polarity() {
    let needs = all(vec![feat("a"), not(feat("b"))]);
    let single = not(feat("c"));
    let unmet = Unmet::All(vec![
        Unmet::Condition {
            needs: needs.clone(),
            has: None,
        },
        Unmet::Any(vec![Unmet::Condition {
            needs: single.clone(),
            has: None,
        }]),
    ]);
    let seen: Vec<bool> = unmet.leaves().map(|l| l.refused).collect();
    assert_eq!(seen, [false, true, true]);
    let named: Vec<&Prerequisite> = unmet.leaves().map(|l| l.condition).collect();
    assert_eq!(named, [&feat("a"), &feat("b"), &feat("c")]);
}

#[test]
fn fact_and_data_at_convert_both_ways() {
    use rules::{DataAt, Fact, WeaponKey};
    let facts = [
        Fact::Feat(FeatKey::new("a")),
        Fact::Feature(rules::FeatureKey::new("a")),
        Fact::Option(rules::SelectionOptionKey::new("a")),
        Fact::Selection(rules::FeatureSelectionKey::new("a")),
        Fact::MythicAbility(rules::MythicAbilityKey::new("a")),
        Fact::MythicFeat(rules::MythicFeatKey::new("a")),
        Fact::Class(rules::ClassKey::new("a")),
        Fact::Archetype(rules::ArchetypeKey::new("a")),
        Fact::Race(rules::RaceKey::new("a")),
        Fact::Deity(rules::DeityKey::new("a")),
    ];
    for fact in facts {
        let at = DataAt::from(&fact);
        assert_eq!(Fact::try_from(&at), Ok(fact));
    }
    // An entry no character holds is no fact.
    assert!(Fact::try_from(&DataAt::Weapon(WeaponKey::new("w"))).is_err());
    assert!(Fact::try_from(&DataAt::Progression).is_err());
}
