//! The blueprint lookups (ticket 299) on three invented blueprints: a feature that refers to another and carries
//! localized text, a progression that grants it, and a bystander.

use std::path::Path;
use std::process::Command;

use game_check::{Library, default_dump};

use crate::support::{Record, binz};

const FEATURE: &str = "000000000000000000000000000000f1";
const PREREQUISITE: &str = "000000000000000000000000000000f2";
const GRANTER: &str = "000000000000000000000000000000f3";

fn record(guid: &str, name: &str, type_name: &str, raw: &str) -> Record {
    Record {
        guid: guid.to_owned(),
        name: name.to_owned(),
        type_name: type_name.to_owned(),
        raw: raw.to_owned(),
    }
}

fn library() -> Library {
    let records = [
        record(
            FEATURE,
            "LookupFeature",
            "Kingmaker.Blueprints.Classes.BlueprintFeature",
            &format!(
                r#"{{"$type": "0, BlueprintFeature", "Components": [
                    {{"$type": "0, PrerequisiteLookupMarker", "m_Feature": "!bp_{PREREQUISITE}", "Other": "!bp_00000000000000000000000000000fff"}}],
                  "m_DisplayName": {{"Shared": null, "m_Key": "invented-key-one", "m_JsonPath": "Strings/invented"}},
                  "m_Description": "invented plain words",
                  "Description": {{"Shared": {{"assetguid": "x", "stringkey": "invented-key-two"}}}},
                  "Ranks": 1}}"#
            ),
        ),
        record(
            PREREQUISITE,
            "LookupPrerequisite",
            "Kingmaker.Blueprints.Classes.BlueprintFeature",
            r#"{"$type": "0, BlueprintFeature", "Ranks": 1}"#,
        ),
        record(
            GRANTER,
            "LookupGranter",
            "Kingmaker.Blueprints.Classes.BlueprintProgression",
            &format!(
                r#"{{"$type": "0, BlueprintProgression", "LevelEntries": [{{"Level": 1, "m_Features": ["!bp_{FEATURE}"]}}]}}"#
            ),
        ),
    ];
    Library::parse(&binz(&records, 2)).unwrap()
}

#[test]
fn show_resolves_a_reference_and_prints_no_localized_text() {
    let text = library().show(&["LookupFeature".to_owned()]).unwrap();
    assert!(
        text.starts_with(&format!("LookupFeature {FEATURE} BlueprintFeature\n")),
        "{text}"
    );
    assert!(
        text.contains(&format!("\"LookupPrerequisite {PREREQUISITE}\"")),
        "{text}"
    );
    assert!(
        text.contains("\"(not in the dump) 00000000000000000000000000000fff\""),
        "{text}"
    );
    for hidden in ["invented-key", "invented plain words", "Strings/invented"] {
        assert!(!text.contains(hidden), "{hidden} in:\n{text}");
    }
    assert!(
        text.contains("\"m_DisplayName\": \"<localized text>\""),
        "{text}"
    );
    assert!(
        text.contains("\"Description\": \"<localized text>\""),
        "{text}"
    );
    // The GUID names it as well as the asset name does.
    assert_eq!(library().show(&[FEATURE.to_owned()]).unwrap(), text);
}

#[test]
fn show_of_an_unknown_name_says_so() {
    let error = library().show(&["Nope".to_owned()]).unwrap_err();
    assert!(error.contains("Nope"), "{error}");
}

#[test]
fn grep_finds_by_name_and_by_a_component_type() {
    let library = library();
    let by_name = library.grep("^LookupGran").unwrap();
    assert_eq!(
        by_name,
        format!("{GRANTER} LookupGranter BlueprintProgression\n")
    );
    let by_component = library.grep("PrerequisiteLookupMarker").unwrap();
    assert_eq!(
        by_component,
        format!("{FEATURE} LookupFeature BlueprintFeature\n")
    );
}

#[test]
fn grep_does_not_match_the_text() {
    let library = library();
    for hidden in ["invented-key", "invented plain words", "Strings/invented"] {
        assert_eq!(library.grep(hidden).unwrap(), "", "{hidden}");
    }
    assert!(library.grep("(").is_err());
}

#[test]
fn refs_finds_the_blueprints_that_grant_or_refer_to_one() {
    let library = library();
    assert_eq!(
        library.refs("LookupFeature").unwrap(),
        format!("{GRANTER} LookupGranter BlueprintProgression\n")
    );
    assert_eq!(
        library.refs(PREREQUISITE).unwrap(),
        format!("{FEATURE} LookupFeature BlueprintFeature\n")
    );
    assert_eq!(library.refs(GRANTER).unwrap(), "");
}

/// Runs git in `dir`, with an identity so the commit works anywhere.
fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t", "-C"])
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

#[test]
fn the_default_dump_is_the_main_checkouts_from_a_worktree() {
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("main").canonicalize_or_create();
    git(&main, &["init", "-q"]);
    git(&main, &["commit", "-q", "--allow-empty", "-m", "start"]);
    let worktree = dir.path().join("lane");
    git(
        &main,
        &[
            "worktree",
            "add",
            "-q",
            worktree.to_str().unwrap(),
            "-b",
            "lane",
        ],
    );
    let expected = main.join("blueprints/dump.binz");

    // The main checkout's own folder, and a worktree with none: both give the main checkout's path.
    assert_eq!(default_dump(&main, "dump.binz"), expected);
    let found = default_dump(&worktree, "dump.binz");
    assert_eq!(found.canonicalize().ok(), expected.canonicalize().ok());
    assert_eq!(
        found
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap(),
        main
    );

    // A checkout with a dump of its own keeps it.
    std::fs::create_dir(worktree.join("blueprints")).unwrap();
    std::fs::write(worktree.join("blueprints/dump.binz"), b"").unwrap();
    assert_eq!(
        default_dump(&worktree, "dump.binz"),
        worktree.join("blueprints/dump.binz")
    );
}

trait CanonicalizeOrCreate {
    fn canonicalize_or_create(self) -> std::path::PathBuf;
}

impl CanonicalizeOrCreate for std::path::PathBuf {
    /// Creates the folder, then names it as git does (macOS temp folders sit behind a symlink).
    fn canonicalize_or_create(self) -> std::path::PathBuf {
        std::fs::create_dir_all(&self).unwrap();
        self.canonicalize().unwrap()
    }
}
