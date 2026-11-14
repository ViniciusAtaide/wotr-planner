//! Ticket 203: `explain` groups two dumps' differences by wildcarded path and change, largest group first, with a
//! slot or a Build only on one side as its own group.

use proof::explain::{explain, wildcard};

/// Dump lines from `build path value` rows, tab-separated as a dump is.
fn dump(rows: &[&str]) -> String {
    rows.iter()
        .map(|row| format!("{}\n", row.replace(" | ", "\t")))
        .collect()
}

fn run(a: &[&str], b: &[&str]) -> String {
    explain(dump(a).as_bytes(), dump(b).as_bytes(), 0).expect("two sorted dumps")
}

#[test]
fn wildcards_levels_and_indexes_but_not_an_options_key() {
    assert_eq!(
        wildcard("slot.Feat.5.General.0.option/rogue-1"),
        "slot.Feat.*.General.*.option/rogue-1"
    );
    assert_eq!(
        wildcard("sheet.levels.3.state.bab"),
        "sheet.levels.*.state.bab"
    );
    assert_eq!(wildcard("totals.12.ac"), "totals.*.ac");
}

#[test]
fn differences_are_grouped_by_wildcarded_path_and_change_largest_first() {
    let a = [
        "b1 | slot.Feat.1.General.0.kind | Feat(General)",
        "b1 | slot.Feat.1.General.0.option/x | locked: Needs a",
        "b1 | slot.Feat.3.General.0.option/x | locked: Needs a",
        "b1 | slot.Feat.9.Race.0.kind | Feat(Race)",
        "b1 | slot.Feat.9.Race.0.option/x | open",
        "b1 | slot.Feat.9.Race.0.total | 4",
        "b1 | totals.1.ac | 10",
        "b2 | slot.Feat.1.General.0.kind | Feat(General)",
        "b2 | slot.Feat.1.General.0.option/x | locked: Needs a",
        "b2 | totals.1.ac | 10",
        "b3 | totals.1.ac | 10",
        "b4 | totals.1.ac | 10",
    ];
    let b = [
        "b1 | slot.Feat.1.General.0.kind | Feat(General)",
        "b1 | slot.Feat.1.General.0.option/x | locked: Needs a or b",
        "b1 | slot.Feat.3.General.0.option/x | locked: Needs a or b",
        "b1 | totals.1.ac | 11",
        "b1 | totals.1.new | 1",
        "b2 | slot.Feat.1.General.0.kind | Feat(General)",
        "b2 | slot.Feat.1.General.0.option/x | locked: Needs a or b",
        "b2 | totals.1.ac | 10",
        "b4 | totals.1.ac | 10",
        "b5 | totals.1.ac | 10",
    ];
    let text = run(&a, &b);
    let groups: Vec<&str> = text.lines().filter(|line| line.starts_with("  ")).collect();
    let expected = [
        // Two Builds, three lines: the largest group.
        "  slot.Feat.*.General.*.option/x: locked: Needs a → locked: Needs a or b  [2 Builds, 3 lines; b1, b2]",
        // The same size in Builds: more lines first, then by the keys.
        "  slot.Feat.*.Race.*: slot only in A (Feat(Race))  [1 Build, 3 lines; b1]",
        "  Build: only in A  [1 Build, 1 line; b3]",
        "  Build: only in B  [1 Build, 1 line; b5]",
        "  totals.*.ac: 10 → 11  [1 Build, 1 line; b1]",
        "  totals.*.new: + 1  [1 Build, 1 line; b1]",
    ];
    // The slot only in A is one group of its own: its option and total lines aren't groups of lines only in A.
    assert_eq!(groups, expected, "{text}");
    assert!(
        text.starts_with("4 of 5 Builds differ (2 in both dumps, 1 only in A, 1 only in B)"),
        "{text}"
    );
}

#[test]
fn agreeing_dumps_have_no_differences() {
    let rows = ["b1 | build.levels | 3", "b1 | build.picks | 4"];
    assert!(run(&rows, &rows).contains("No differences."));
}

#[test]
fn a_long_value_is_cut_where_the_two_differ_and_given_in_full_once() {
    let long = |extra: &str| {
        format!(
            "locked: Needs {}Skill Focus or {}",
            "Alpha or ".repeat(12),
            extra.to_owned() + &"Omega or ".repeat(12)
        )
    };
    let (old, new) = (long(""), long("Improved Uncanny Dodge or "));
    let a = format!("b1 | slot.Feat.1.General.0.option/x | {old}");
    let b = format!("b1 | slot.Feat.1.General.0.option/x | {new}");
    let text = run(&[a.as_str()], &[b.as_str()]);
    assert!(text.contains("Improved Uncanny Dodge or"), "{text}");
    assert!(text.contains("[1]") && text.contains("[2]"), "{text}");
    assert!(
        text.contains("(the alternatives: + Improved Uncanny Dodge)"),
        "{text}"
    );
    assert!(
        text.contains(&format!("  [1] {old}\n")) && text.contains(&format!("  [2] {new}\n")),
        "{text}"
    );
    let group = text
        .lines()
        .find(|line| line.contains("→"))
        .expect("a group");
    assert!(
        group.chars().count() < 2 * (proof::explain::WIDTH + 12) + 120,
        "{group}"
    );
}

#[test]
fn a_dump_out_of_order_is_refused() {
    let unsorted = dump(&["b1 | z | 1", "b1 | a | 1"]);
    let error = explain(unsorted.as_bytes(), unsorted.as_bytes(), 0).unwrap_err();
    assert!(error.contains("out of order"), "{error}");
}

#[test]
fn a_long_value_that_is_no_list_is_shown_around_where_the_two_differ() {
    let long = |middle: &str| format!("{}{middle}{}", "a".repeat(150), "z".repeat(150));
    let a = format!("b1 | x | {}", long("OLD"));
    let b = format!("b1 | x | {}", long("NEW"));
    let text = run(&[a.as_str()], &[b.as_str()]);
    let group = text
        .lines()
        .find(|line| line.contains("→"))
        .expect("a group");
    assert!(group.contains("OLD") && group.contains("NEW"), "{group}");
    assert!(group.contains("…"), "{group}");
}
