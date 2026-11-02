//! Ticket 35's second follow-ups: how the files write and read back the new shapes (a subrace's feat slots, an
//! option list slot's fixed option level, a selection's option counted beside the others) and the bloodrager's
//! groups.

fn fixture() -> rules::GameData {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    seeds::read_game_data_files(&fixture).unwrap()
}

fn file<'f>(files: &'f [(std::path::PathBuf, String)], name: &str) -> &'f str {
    &files
        .iter()
        .find(|(path, _)| path.ends_with(name))
        .unwrap_or_else(|| panic!("no {name}"))
        .1
}

#[test]
fn the_files_write_the_new_shapes_and_read_them_back() {
    let mut data = fixture();
    // A bloodrager-like group on the fixture's Draconic option, for its name in the file.
    let red = data
        .classes
        .iter_mut()
        .flat_map(|class| &mut class.selections)
        .flat_map(|selection| &mut selection.options)
        .find(|option| option.key.as_str() == "sorcerer/bloodline/draconic-red")
        .unwrap();
    red.group = Some(rules::OptionGroup::BloodragerDraconic);
    red.prerequisite = Some(rules::Prerequisite::NoOtherOptionOfGroup(
        rules::OptionGroup::BloodragerDraconic,
    ));
    let files = extract::ron_out::files(&data);
    let races = file(&files, "races.ron");
    assert!(
        races.contains("ability_modifiers: [Chosen(amount: 2)], feat_slots: [OneOf(list: \"half-elf/adaptability\")])"),
        "{races}"
    );
    let sorcerer = file(&files, "classes/sorcerer.ron");
    for expected in [
        // A fixed option level is a plain number; an unfixed slot has none.
        "feat_slots: [OptionList(selection: \"sorcerer/bloodline\"), OptionList(selection: \"sorcerer/bloodline\", at_level: 1)]",
        "level_also_from: \"sorcerer/seeker/choice/power\",",
        "group: BloodragerDraconic,",
        "prerequisite: NoOtherOptionOfGroup(BloodragerDraconic),",
    ] {
        assert!(sorcerer.contains(expected), "{expected} in {sorcerer}");
    }
    // Unset, they're left out.
    let rogue = file(&files, "classes/rogue.ron");
    for absent in ["at_level", "level_also_from:", "None"] {
        assert!(!rogue.contains(absent), "{absent} in {rogue}");
    }
    let elf_line = races
        .lines()
        .find(|line| line.contains("elf/blightborn"))
        .unwrap();
    assert!(!elf_line.contains("feat_slots"), "{elf_line}");

    // And they read back as the same data.
    let dir = tempfile::tempdir().unwrap();
    for (path, text) in &files {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    assert_eq!(seeds::read_game_data_files(dir.path()).unwrap(), data);
}
