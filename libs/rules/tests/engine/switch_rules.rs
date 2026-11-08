//! The Mythic path's and the Path switch's rules on their own data, without a walk (ticket 99): a path's row at a
//! rank, which grants a switch takes away and whether it ends the ascensions.

use rules::{FeatureSelectionKey, MythicPath, PathSwitch};

use crate::support::*;

/// The world's path with this key.
fn mythic_path(key: &str) -> MythicPath {
    world()
        .mythic_paths
        .into_iter()
        .find(|candidate| candidate.key == path(key))
        .expect("the world has the path")
}

/// The switch rule of the world's path with this key.
fn switch(key: &str) -> PathSwitch {
    mythic_path(key).switch.expect("the path has a switch rule")
}

#[test]
fn a_path_gives_its_row_at_a_rank_and_none_at_a_rank_without_one() {
    let trickster = mythic_path("trickster");

    let mr_2 = trickster.row(2).expect("the Trickster has an MR 2 row");
    assert_eq!(mr_2.rank, 2);
    assert_eq!(mr_2.features, vec![feature("trickster/hard-to-kill")]);
    assert!(trickster.row(6).is_none());
    assert!(trickster.row(11).is_none());
}

#[test]
fn only_a_switch_without_further_ascensions_ends_them_at_mr_8() {
    assert_eq!(switch("legend").ends_ascensions_at(), Some(8));
    assert_eq!(switch("gold-dragon").ends_ascensions_at(), None);
    assert_eq!(switch("swarm-that-walks").ends_ascensions_at(), None);
    assert_eq!(switch("devil").ends_ascensions_at(), None);
}

/// The ranks of MR 1–7 whose general picks (`from_path: false`) or old-path grants (`true`) a switch keeps.
fn kept_ranks(switch: &PathSwitch, from_path: bool) -> Vec<u8> {
    (1..8)
        .filter(|&rank| switch.keeps(rank, from_path))
        .collect()
}

#[test]
fn a_switch_keeps_the_general_picks_and_the_old_paths_grants_each_up_to_its_own_rank() {
    let legend = switch("legend");
    assert_eq!(kept_ranks(&legend, false), vec![1, 2]);
    assert_eq!(kept_ranks(&legend, true), vec![1, 2]);

    let gold_dragon = switch("gold-dragon");
    assert_eq!(kept_ranks(&gold_dragon, false), vec![1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(kept_ranks(&gold_dragon, true), vec![1, 2]);

    let devil = switch("devil");
    assert_eq!(kept_ranks(&devil, false), vec![1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(kept_ranks(&devil, true), vec![1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn a_switch_drops_a_grant_past_its_cut_off_or_an_old_path_grant_it_names() {
    let gold_dragon = switch("gold-dragon");
    assert!(!gold_dragon.drops(7, false, None, None));
    assert!(gold_dragon.drops(3, true, Some(&feature("trickster/trickster-tricks")), None));
    assert!(!gold_dragon.drops(2, true, Some(&feature("trickster/hard-to-kill")), None));

    let devil = switch("devil");
    let gaze = FeatureSelectionKey::new("aeon/enforcing-gaze");
    let extra_gaze = feature("aeon/enforcing-gaze-extra-gaze");
    // The Aeon's extra gaze and the picks of its Enforcing Gaze go whatever their rank.
    assert!(devil.drops(1, true, Some(&extra_gaze), None));
    assert!(devil.drops(3, true, None, Some(&gaze)));
    assert!(!devil.drops(7, true, Some(&feature("aeon/other")), None));
    // A grant named only drops as the old path's: a general pick's grant goes by the general cut-off alone.
    assert!(!devil.drops(5, false, Some(&extra_gaze), Some(&gaze)));
}
