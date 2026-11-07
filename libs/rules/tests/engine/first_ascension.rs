//! The first ascension (ticket 91): at MR 1 the character takes one path's path-linked ability (its MR 1 row) and at
//! MR 2 that row's follow-ups (its MR 2 row), whatever path it chooses at MR 3, which no longer grants its own MR 1
//! and 2 rows. A Build without the choice (every link from before it) takes its MR 3 path's.

use rules::{Choice, Edit, Issue, SlotKind, decode_build, encode_build};

use crate::support::*;

/// A fighter of `levels` levels with the first ascension `first` and the path `path` (each if any), and nothing else
/// chosen.
fn ascended(first: Option<&str>, path_key: Option<&str>, levels: usize) -> rules::Build {
    let mut build = build("human", &times("fighter", levels));
    build.first_ascension = first.map(path);
    build.mythic_path = path_key.map(path);
    build
}

fn count(build: &rules::Build, level: u8, key: &str) -> u8 {
    engine()
        .evaluate(build)
        .at(level)
        .unwrap()
        .feature_count(&feature(key))
}

#[test]
fn the_first_ascension_gives_its_paths_mr_1_row_at_mr_1_and_its_mr_2_row_at_mr_2() {
    let build = ascended(Some("trickster"), Some("aeon"), 11);
    assert!(engine().is_possible(&build));

    // MR 1 comes at 6, MR 2 at 7.
    assert_eq!(count(&build, 5, "trickster/bit-of-fun"), 0);
    assert_eq!(count(&build, 6, "trickster/bit-of-fun"), 1);
    assert_eq!(
        count(&build, 6, "trickster/bypass-epic-damage-reduction"),
        1
    );
    assert_eq!(count(&build, 6, "trickster/hard-to-kill"), 0);
    assert_eq!(count(&build, 7, "trickster/hard-to-kill"), 1);
    // A pick at level 7 comes before MR 2's ascension; one at 8 sees it.
    let before_7 = engine().state_before(&build, &general_slot(7)).unwrap();
    assert_eq!(before_7.feature_count(&feature("trickster/bit-of-fun")), 1);
    assert_eq!(
        before_7.feature_count(&feature("trickster/hard-to-kill")),
        0
    );

    // The Aeon chosen at MR 3 gives its own rows from MR 3 only: none of its MR 1 and 2 rows.
    assert_eq!(count(&build, 11, "aeon/force-reality"), 0);
    assert_eq!(count(&build, 11, "aeon/hard-to-kill"), 0);
    assert_eq!(count(&build, 11, "aeon/aeon-bane"), 1);
    assert_eq!(count(&build, 11, "trickster/bit-of-fun"), 1);
    assert_eq!(count(&build, 11, "trickster/hard-to-kill"), 1);

    // MR 1 and 2 show the first ascension's path, MR 3 on the path's; MR 1 has only its general pick.
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.ranks[0].path, Some(path("trickster")));
    assert_eq!(sheet.ranks[1].path, Some(path("trickster")));
    assert_eq!(sheet.ranks[2].path, Some(path("aeon")));
    assert_eq!(sheet.ranks[0].slots.len(), 1);
    assert_eq!(sheet.ranks[0].slots[0].kind, SlotKind::MythicAbility);
}

#[test]
fn the_mythic_track_names_each_ranks_row_features() {
    let engine = engine();
    let sheet = engine.evaluate(&ascended(Some("trickster"), Some("aeon"), 11));
    let names = |index: usize| -> Vec<&str> {
        engine
            .rank_features(&sheet.ranks[index])
            .into_iter()
            .map(|key| engine.feature_name(key))
            .collect()
    };
    // In the row's (key) order.
    assert_eq!(names(0), ["Bit of Fun", "Bypass Epic Damage Reduction"]);
    assert_eq!(names(1), ["Hard to Kill"]);
    assert_eq!(names(3), ["Aeon Bane"]);
    // A rank not reached gives nothing.
    assert!(names(5).is_empty());
}

#[test]
fn without_a_first_ascension_the_build_takes_its_mr_3_paths_from_mr_1() {
    let build = ascended(None, Some("trickster"), 11);
    assert!(engine().is_possible(&build));
    assert_eq!(count(&build, 6, "trickster/bit-of-fun"), 1);
    assert_eq!(count(&build, 7, "trickster/hard-to-kill"), 1);
    assert_eq!(count(&build, 11, "trickster/bit-of-fun"), 1);
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.ranks[0].path, Some(path("trickster")));
    assert_eq!(sheet.ranks[1].path, Some(path("trickster")));

    // Taking the MR 3 path's own is the same Build in all but the choice.
    let chosen = ascended(Some("trickster"), Some("trickster"), 11);
    assert_eq!(engine().evaluate(&chosen), sheet);
}

#[test]
fn a_build_with_neither_holds_no_row_at_mr_1_and_2() {
    let build = ascended(None, None, 8);
    assert!(engine().is_possible(&build));
    assert_eq!(count(&build, 8, "trickster/bit-of-fun"), 0);
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.ranks[0].path, None);
    assert_eq!(sheet.ranks[1].path, None);
}

#[test]
fn a_first_ascension_before_the_mr_3_path_holds_from_mr_1() {
    let build = ascended(Some("aeon"), None, 8);
    assert!(engine().is_possible(&build));
    assert_eq!(count(&build, 6, "aeon/force-reality"), 1);
    assert_eq!(count(&build, 7, "aeon/hard-to-kill"), 1);
    assert_eq!(engine().evaluate(&build).ranks[0].path, Some(path("aeon")));
}

#[test]
fn the_picker_offers_the_paths_with_a_path_linked_ability() {
    let engine = engine();
    let build = ascended(None, None, 6);
    let options = engine.first_ascension_options(&build);
    // The Trickster and the Aeon have MR 1 rows; the switches (Legend, Gold Dragon, Swarm, Devil) don't.
    let paths: Vec<_> = options.iter().map(|option| option.path.clone()).collect();
    assert_eq!(paths, [path("aeon"), path("trickster")]);
    assert!(options.iter().all(|option| option.status.is_ok()));
    // Each is named by its path-linked ability, not the fixed Bypass Epic Damage Reduction every row has.
    assert_eq!(
        engine.first_ascension_name(&path("trickster")),
        "Bit of Fun"
    );
    assert_eq!(engine.first_ascension_name(&path("aeon")), "Force Reality");
}

#[test]
fn a_first_ascension_without_an_mr_1_row_or_before_mr_1_is_not_possible() {
    let engine = engine();
    assert!(!engine.is_possible(&ascended(Some("legend"), None, 8)));
    assert!(!engine.is_possible(&ascended(Some("no-such-path"), None, 8)));
    assert!(!engine.is_possible(&ascended(Some("trickster"), None, 5)));
}

#[test]
fn the_first_ascension_is_set_from_mr_1_and_goes_with_its_level() {
    let engine = engine();
    // Before MR 1 it isn't kept.
    let mut five = ascended(None, None, 5);
    engine.apply(&mut five, &Edit::FirstAscension(Some(path("trickster"))));
    assert_eq!(five.first_ascension, None);

    let mut six = ascended(None, None, 6);
    engine.apply(&mut six, &Edit::FirstAscension(Some(path("trickster"))));
    assert_eq!(six.first_ascension, Some(path("trickster")));
    assert!(engine.removable(&six).contains(&Choice::FirstAscension));
    assert_eq!(count(&six, 6, "trickster/bit-of-fun"), 1);

    // Removing level 6 drops it.
    engine.apply(&mut six, &Edit::RemoveLevel);
    assert_eq!(six.first_ascension, None);
    assert!(engine.is_possible(&six));
}

#[test]
fn changing_the_mr_3_path_moves_the_first_ascension_only_when_the_build_has_none_of_its_own() {
    let engine = engine();
    let mut default = ascended(None, Some("trickster"), 9);
    engine.apply(&mut default, &Edit::MythicPath(Some(path("aeon"))));
    assert_eq!(count(&default, 6, "aeon/force-reality"), 1);
    assert_eq!(count(&default, 6, "trickster/bit-of-fun"), 0);

    let mut chosen = ascended(Some("trickster"), Some("trickster"), 9);
    engine.apply(&mut chosen, &Edit::MythicPath(Some(path("aeon"))));
    assert_eq!(count(&chosen, 6, "trickster/bit-of-fun"), 1);
    assert_eq!(count(&chosen, 9, "aeon/force-reality"), 0);
}

#[test]
fn a_path_switch_keeps_the_first_ascensions_rows() {
    let engine = engine();
    // Legend keeps MR 1-2; the Devil, open from the Aeon chosen at MR 3, keeps MR 1-7.
    for switch in ["legend", "devil"] {
        let mut build = ascended(Some("trickster"), Some("aeon"), 17);
        engine.apply(&mut build, &Edit::PathSwitch(Some(path(switch))));
        assert!(engine.is_possible(&build), "{switch}");
        assert_eq!(count(&build, 17, "trickster/bit-of-fun"), 1, "{switch}");
        assert_eq!(count(&build, 17, "trickster/hard-to-kill"), 1, "{switch}");
    }
    // The Devil is open from the MR 3 path, not the first ascension's.
    let mut from_trickster = ascended(Some("aeon"), Some("trickster"), 17);
    from_trickster.path_switch = Some(path("devil"));
    assert!(!engine.is_possible(&from_trickster));
    assert!(matches!(
        engine
            .switch_options(&ascended(Some("aeon"), Some("trickster"), 17))
            .into_iter()
            .find(|option| option.path == path("devil"))
            .unwrap()
            .status,
        Err(Issue::NotFromPath { .. })
    ));
}

#[test]
fn a_first_ascension_round_trips_through_a_link_and_a_link_without_one_reads_as_none() {
    let engine = engine();
    let keys = rules::LinkKeys::new(&world().canonical());
    let with = ascended(Some("trickster"), Some("aeon"), 9);
    assert_eq!(decode_build(&encode_build(&with), &keys), Ok(with.clone()));
    assert_eq!(engine.open_link(&encode_build(&with)), Ok(with));

    // A Build without one writes the link it always did (the frozen v1 vector in `codec` holds the layout).
    let without = ascended(None, Some("aeon"), 9);
    let decoded = decode_build(&encode_build(&without), &keys).unwrap();
    assert_eq!(decoded.first_ascension, None);
    assert_eq!(decoded, without);
}
