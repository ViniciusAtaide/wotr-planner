//! A prerequisite tree's leaves: the single conditions under its groups and negations.

use rules::{FeatKey, Prerequisite};

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
