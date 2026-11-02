//! Ticket 49's extra picks: how the files write and read back a feat's extra pick.

fn fixture() -> rules::GameData {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    data::read_game_data_files(&fixture).unwrap()
}

#[test]
fn the_files_write_a_feats_extra_pick_on_its_line_and_read_it_back() {
    let mut data = fixture();
    let feat = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "extra-rage")
        .unwrap();
    feat.extra_pick_of = ["sorcerer/bloodline", "rogue/rogue-talents"]
        .into_iter()
        .map(rules::FeatureSelectionKey::new)
        .collect();
    let files = extract::ron_out::files(&data);
    let feats = &files
        .iter()
        .find(|(path, _)| path.ends_with("feats.ron"))
        .unwrap()
        .1;
    // After its repeat, in key order; a feat without one leaves it out.
    assert!(
        feats.contains(
            "repeat: Times(3), extra_pick_of: [\"rogue/rogue-talents\", \"sorcerer/bloodline\"])"
        ),
        "{feats}"
    );
    assert_eq!(feats.matches("extra_pick_of").count(), 1, "{feats}");

    let dir = tempfile::tempdir().unwrap();
    extract::output::write_files(dir.path(), &files, false).unwrap();
    assert_eq!(data::read_game_data_files(dir.path()).unwrap(), data);
}
