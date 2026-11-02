//! Ticket 47's talent follow-ups: how the files write and read back the new shapes (an option once per value of its
//! chosen feat, an option list slot's own label).

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
fn the_files_write_a_per_value_option_and_a_labelled_slot_and_read_them_back() {
    let mut data = fixture();
    let talent = data
        .classes
        .iter_mut()
        .flat_map(|class| &mut class.selections)
        .flat_map(|selection| &mut selection.options)
        .find(|option| option.key.as_str() == "rogue/rogue-talents/weapon-focus")
        .unwrap();
    talent.repeat = rules::Repeat::PerValue;
    let slot = data
        .classes
        .iter_mut()
        .filter(|class| class.key.as_str() == "sorcerer")
        .flat_map(|class| &mut class.levels)
        .flat_map(|row| &mut row.feat_slots)
        .find_map(|slot| match slot {
            rules::FeatSlot::OptionList {
                at_level: None,
                label,
                ..
            } => Some(label),
            _ => None,
        })
        .unwrap();
    *slot = Some("Wild talent feat".into());
    let files = extract::ron_out::files(&data);
    let rogue = file(&files, "classes/rogue.ron");
    assert!(
        rogue.contains("chosen_feat: \"weapon-focus\", repeat: PerValue"),
        "{rogue}"
    );
    // A label is a plain string on the slot's one line; unset, it's left out.
    let sorcerer = file(&files, "classes/sorcerer.ron");
    assert!(
        sorcerer.contains(
            "feat_slots: [OptionList(selection: \"sorcerer/bloodline\", label: \"Wild talent feat\"), \
             OptionList(selection: \"sorcerer/bloodline\", at_level: 1)]"
        ),
        "{sorcerer}"
    );
    assert_eq!(sorcerer.matches("label:").count(), 1, "{sorcerer}");

    // And they read back as the same data.
    let dir = tempfile::tempdir().unwrap();
    for (path, text) in &files {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    assert_eq!(seeds::read_game_data_files(dir.path()).unwrap(), data);
}
