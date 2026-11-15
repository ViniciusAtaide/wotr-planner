//! A known difference is grouped under its ticket and doesn't fail the check; a known line matching nothing does.

use crate::support::{report, report_with};

#[test]
fn a_known_group_names_its_ticket_and_a_stale_line_fails() {
    let report = report_with(
        "items / slot differs -> fixture-ticket\nitems / no such kind -> stale-ticket\n",
    );
    let text = report.text();
    assert!(
        text.contains("\nitems / slot differs [known: fixture-ticket]: 1\n"),
        "{text}"
    );
    assert!(text.contains(
        "Known lines that match no difference:\n  line 2: items / no such kind -> stale-ticket"
    ));
    assert!(report.failed());
}

#[test]
fn the_check_passes_when_every_difference_is_known() {
    assert!(report().failed());
    let every_area = "mapping -> a\nprerequisites -> a\nclass tables -> a\nprogression rows -> a\noption lists -> a\n\
                      mythic ranks -> a\nitems -> a\n";
    let report = report_with(every_area);
    assert!(!report.failed(), "{}", report.text());
}

#[test]
fn an_entry_line_wins_over_its_group() {
    let report = report_with(
        "items / slot differs -> group-ticket\nitems / slot differs / gear item headband-of-test -> entry-ticket\n",
    );
    assert!(
        report
            .text()
            .contains("items / slot differs [known: entry-ticket]: 1")
    );
    assert!(
        report
            .text()
            .contains("Known lines that match no difference:\n  line 1:")
    );
}
