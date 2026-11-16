//! The blueprint lookups (ticket 299) on three invented blueprints: a feature that refers to another and carries
//! localized text, a progression that grants it, and a bystander.

use std::path::Path;
use std::process::Command;

use game_check::{Library, default_dump};

use crate::support::{Record, binz};

const FEATURE: &str = "000000000000000000000000000000f1";
const PREREQUISITE: &str = "000000000000000000000000000000f2";
const GRANTER: &str = "000000000000000000000000000000f3";
/// Two blueprints that share a name: the first only a unit's feature list holds, the second a progression's chain.
const UNIT_SELECTION: &str = "000000000000000000000000000000e1";
const CLASS_SELECTION: &str = "000000000000000000000000000000e2";
const SELECTION_GROUP: &str = "000000000000000000000000000000e3";
const CLASS_PROGRESSION: &str = "000000000000000000000000000000e4";
const UNIT: &str = "000000000000000000000000000000e5";

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
        record(
            UNIT_SELECTION,
            "SharedSelection",
            "Kingmaker.Blueprints.Classes.Selection.BlueprintFeatureSelection",
            r#"{"$type": "0, BlueprintFeatureSelection"}"#,
        ),
        record(
            CLASS_SELECTION,
            "SharedSelection",
            "Kingmaker.Blueprints.Classes.Selection.BlueprintFeatureSelection",
            r#"{"$type": "0, BlueprintFeatureSelection"}"#,
        ),
        record(
            SELECTION_GROUP,
            "SharedGroup",
            "Kingmaker.Blueprints.Classes.BlueprintFeature",
            &format!(
                r#"{{"$type": "0, BlueprintFeature", "m_Features": ["!bp_{CLASS_SELECTION}"]}}"#
            ),
        ),
        record(
            CLASS_PROGRESSION,
            "SharedProgression",
            "Kingmaker.Blueprints.Classes.BlueprintProgression",
            &format!(
                r#"{{"$type": "0, BlueprintProgression", "m_Features": ["!bp_{SELECTION_GROUP}"]}}"#
            ),
        ),
        record(
            UNIT,
            "SharedUnit",
            "Kingmaker.Blueprints.BlueprintUnit",
            &format!(r#"{{"$type": "0, BlueprintUnit", "m_Features": ["!bp_{UNIT_SELECTION}"]}}"#),
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

#[test]
fn a_shared_name_gets_a_banner_naming_each_guid_and_what_grants_it() {
    let library = library();
    for text in [
        library.show(&["SharedSelection".to_owned()]).unwrap(),
        library.refs("SharedSelection").unwrap(),
    ] {
        let banner = text.split("\n\n").next().unwrap();
        assert!(
            banner.starts_with("shared name: SharedSelection is 2 blueprints"),
            "{text}"
        );
        assert!(
            banner.contains(&format!(
                "{UNIT_SELECTION} BlueprintFeatureSelection: granted by SharedUnit (BlueprintUnit)"
            )),
            "{banner}"
        );
        assert!(
            banner.contains(&format!(
                "{CLASS_SELECTION} BlueprintFeatureSelection: granted by SharedProgression (BlueprintProgression)"
            )),
            "{banner}"
        );
    }
}

#[test]
fn a_name_one_blueprint_holds_gets_no_banner() {
    let library = library();
    assert!(
        !library
            .show(&["LookupFeature".to_owned()])
            .unwrap()
            .contains("shared name")
    );
    assert!(
        !library
            .refs("LookupFeature")
            .unwrap()
            .contains("shared name")
    );
    // Nor does a GUID, even of a shared name: it names one blueprint.
    assert!(
        !library
            .show(&[UNIT_SELECTION.to_owned()])
            .unwrap()
            .contains("shared name")
    );
}

#[test]
fn granted_by_follows_referrers_up_to_the_first_granting_blueprint() {
    let text = library().granted_by(CLASS_SELECTION).unwrap();
    assert_eq!(
        text,
        format!(
            "SharedSelection {CLASS_SELECTION} BlueprintFeatureSelection\n  \
             SharedSelection {CLASS_SELECTION} <- SharedGroup {SELECTION_GROUP} <- SharedProgression {CLASS_PROGRESSION} [BlueprintProgression]\n"
        )
    );
    let unit = library().granted_by(UNIT_SELECTION).unwrap();
    assert!(
        unit.ends_with(&format!(
            "  SharedSelection {UNIT_SELECTION} <- SharedUnit {UNIT} [BlueprintUnit]\n"
        )),
        "{unit}"
    );
}

#[test]
fn granted_by_of_a_name_prints_a_block_per_guid_and_says_when_nothing_grants() {
    let library = library();
    let text = library.granted_by("SharedSelection").unwrap();
    assert!(text.contains(&format!(
        "SharedSelection {UNIT_SELECTION} BlueprintFeatureSelection\n  SharedSelection"
    )));
    assert!(text.contains(&format!(
        "SharedSelection {CLASS_SELECTION} BlueprintFeatureSelection\n  SharedSelection"
    )));
    let none = library.granted_by(PREREQUISITE).unwrap();
    // Reached through LookupFeature, then the progression.
    assert!(none.contains("[BlueprintProgression]"), "{none}");
    let lone = library.granted_by("SharedUnit").unwrap();
    assert!(
        lone.ends_with(&format!("  SharedUnit {UNIT} [BlueprintUnit]\n")),
        "{lone}"
    );
    assert!(library.granted_by("Nope").is_err());
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
