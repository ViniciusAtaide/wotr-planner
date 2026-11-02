//! Ticket 33's option follow-ups: how the files write and read back the new shapes (an option's group and arcana,
//! a selection's lag, level source and once-per-option, a pool selection's own options, `NoOtherOptionOfGroup` and
//! `PickedThrough`).

fn fixture() -> rules::GameData {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    data::read_game_data_files(&fixture).unwrap()
}

#[test]
fn the_files_write_the_new_shapes_and_read_them_back() {
    let data = fixture();
    let files = extract::ron_out::files(&data);
    let sorcerer = &files
        .iter()
        .find(|(path, _)| path.ends_with("classes/sorcerer.ron"))
        .expect("the fixture's sorcerer")
        .1;
    for expected in [
        "prerequisite: All([NoOptionFrom(\"sorcerer/bloodline\"), NoOtherOptionOfGroup(Draconic)]),",
        "group: Draconic,",
        "arcana: \"sorcerer/bloodline/draconic/arcana\",",
        "prerequisite: Any([Not(PickedThrough(\"sorcerer/school\")), Alignment(NeutralEvil)])",
        "level_from: GrantingOption,",
        "level_from: OtherOptions,",
        "levels_behind: 2,",
        "once_per_option: true,",
        "pool_of: \"sorcerer/bloodline\",",
        "(key: \"sorcerer/power/surge\", name: \"Surge\", guide: \"sorcerer\")",
    ] {
        assert!(sorcerer.contains(expected), "{expected} in {sorcerer}");
    }
    // Unset, they're left out.
    let rogue = &files
        .iter()
        .find(|(path, _)| path.ends_with("classes/rogue.ron"))
        .unwrap()
        .1;
    for absent in [
        "group:",
        "arcana:",
        "levels_behind:",
        "level_from:",
        "once_per_option:",
    ] {
        assert!(!rogue.contains(absent), "{absent} in {rogue}");
    }

    // And they read back as the same data.
    let dir = tempfile::tempdir().unwrap();
    for (path, text) in &files {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    assert_eq!(data::read_game_data_files(dir.path()).unwrap(), data);
}
