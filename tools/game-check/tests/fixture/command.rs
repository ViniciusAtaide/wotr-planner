//! The command line (ticket 293d, 293e): what `--write` refuses before it reads the dump or writes a file.

use std::process::Command;

fn game_check(args: &[&str]) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_game-check"))
        .args(args)
        // A dump that isn't there: a run that reads it first fails with its name, not the one under test.
        .args(["--binz", "/nonexistent/dump.binz"])
        .output()
        .unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn a_log_that_cannot_be_written_stops_the_run_before_the_dump_is_read() {
    let (code, stderr) = game_check(&["--write", "items", "--log", "/nonexistent/folder/log.tsv"]);
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("--log /nonexistent/folder/log.tsv"),
        "{stderr}"
    );
}

#[test]
fn write_with_a_stray_word_is_refused_and_runs_no_lookup() {
    let (code, stderr) = game_check(&["--write", "items", "show", "foo"]);
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("--write takes no lookup command"),
        "{stderr}"
    );
    let (code, stderr) = game_check(&["--write", "nothing"]);
    assert_eq!(code, Some(2));
    assert!(stderr.contains("not an area"), "{stderr}");
}

#[test]
fn only_without_the_option_lists_area_is_refused() {
    let (code, stderr) = game_check(&["--write", "items", "--only", "magic-feats"]);
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("--only limits the option-lists area"),
        "{stderr}"
    );
}

#[test]
fn option_lists_without_only_is_refused() {
    let (code, stderr) = game_check(&["--write", "option-lists"]);
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("option-lists writes only the selections --only names"),
        "{stderr}"
    );
}

#[test]
fn known_format_goes_with_the_carrier_audit() {
    let (code, stderr) = game_check(&["--known-format"]);
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("--known-format goes with --carrier-audit"),
        "{stderr}"
    );
}

#[test]
fn carriers_without_only_is_refused_until_the_registry_exists() {
    let (code, stderr) = game_check(&["--write", "carriers", "--dry-run"]);
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains(
            "carriers writes only the GUIDs --only names until 307c's carriers-left.txt registry"
        ),
        "{stderr}"
    );
}
