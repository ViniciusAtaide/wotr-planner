//! Writing the data files: never over existing data.

use std::fs;
use std::path::PathBuf;

use extract::output::write_files;

fn files() -> Vec<(PathBuf, String)> {
    vec![
        ("skills.ron".into(), "[]".into()),
        ("classes/rogue.ron".into(), "Class()".into()),
    ]
}

#[test]
fn writes_every_file_into_an_empty_folder() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("game-data");

    write_files(&out, &files(), false).unwrap();

    assert_eq!(fs::read_to_string(out.join("skills.ron")).unwrap(), "[]");
    assert_eq!(
        fs::read_to_string(out.join("classes/rogue.ron")).unwrap(),
        "Class()"
    );
}

#[test]
fn refuses_a_folder_that_holds_data_even_when_forced() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("feats.ron"), "[]").unwrap();

    for force in [false, true] {
        let error = write_files(dir.path(), &files(), force).unwrap_err();
        assert!(error.contains("feats.ron"), "{error}");
    }
    assert!(!dir.path().join("skills.ron").exists());
}

#[test]
fn writes_next_to_other_files_only_when_forced() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "about the data").unwrap();

    let error = write_files(dir.path(), &files(), false).unwrap_err();
    assert!(error.contains("--force"), "{error}");
    assert!(!dir.path().join("skills.ron").exists());

    write_files(dir.path(), &files(), true).unwrap();
    assert!(dir.path().join("skills.ron").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("README.md")).unwrap(),
        "about the data"
    );
}
