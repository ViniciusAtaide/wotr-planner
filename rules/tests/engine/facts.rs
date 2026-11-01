//! Rule facts that live in code rather than in the game data: which feats are Combat feats, when the path is
//! chosen and switched, where a switch leaves the Mythic rank, and the class level caps.

use std::collections::BTreeSet;

use rules::{Choice, FeatCategory, Issue, PATH_CHOICE_RANK, PATH_SWITCH_RANK, PathSwitch};

use crate::support::*;

#[test]
fn combat_feats_are_every_category_but_class_racial_skill_teamwork_and_magic() {
    use FeatCategory::*;
    let all = [
        Weapon,
        Ranged,
        UnarmedCombat,
        ArmourAndShield,
        Magic,
        Miscellaneous,
        MountedCombat,
        CombatManoeuvres,
        Teamwork,
        Skill,
        Class,
        Racial,
    ];
    let not_combat: BTreeSet<_> = all.into_iter().filter(|c| !c.is_combat()).collect();
    assert_eq!(
        not_combat,
        BTreeSet::from([Class, Racial, Skill, Teamwork, Magic])
    );
}

#[test]
fn the_path_is_chosen_at_mr_3_and_switched_at_mr_8() {
    assert_eq!((PATH_CHOICE_RANK, PATH_SWITCH_RANK), (3, 8));
}

#[test]
fn a_switch_leaves_the_rank_one_above_the_general_picks_it_keeps() {
    let switch = |kept: u8| PathSwitch {
        from: BTreeSet::new(),
        kept_through_rank: kept,
        path_kept_through_rank: 2,
        dropped_features: BTreeSet::new(),
        dropped_selections: BTreeSet::new(),
        features: BTreeSet::new(),
        further_ascensions: false,
        level_cap: None,
    };
    // Legend keeps MR 1-2 and stays at MR 3.
    assert_eq!(switch(2).rank_after_switch(), 3);
    assert_eq!(switch(7).rank_after_switch(), 8);
}

#[test]
fn class_levels_are_capped_at_the_length_of_the_class_table() {
    // Fighter's table has 20 rows and Mystic Theurge's 10, Legend or not.
    let mut build = build("elf", &times("fighter", 21));
    build.path_switch = Some(path("legend"));
    let problems: Vec<_> = engine()
        .validate(&build)
        .into_iter()
        .map(|p| (p.at, p.issue))
        .collect();
    assert_eq!(
        problems,
        [(
            Choice::Class { level: 21 },
            Issue::ClassLevelCap { cap: 20 }
        )]
    );
}

#[test]
fn the_legend_switch_raises_the_character_level_cap_to_40() {
    let mut build = build("elf", &times("fighter", 17));
    build.path_switch = Some(path("legend"));
    assert_eq!(engine().evaluate(&build).level_cap, 40);
    build.path_switch = None;
    assert_eq!(engine().evaluate(&build).level_cap, 20);
}
