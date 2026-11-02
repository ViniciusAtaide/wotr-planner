//! Ticket 55's extra picks: how the files write and read back a mythic feat's extra pick.

fn fixture() -> rules::GameData {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    seeds::read_game_data_files(&fixture).unwrap()
}

#[test]
fn the_files_write_a_mythic_feats_extra_pick_on_its_line_and_read_it_back() {
    let mut data = fixture();
    let feat = data
        .mythic_feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "power-attack-mythic")
        .unwrap();
    feat.extra_pick = Some(rules::MythicExtraPick::MythicAbility);
    let files = extract::ron_out::files(&data);
    let feats = &files
        .iter()
        .find(|(path, _)| path.ends_with("mythic/feats.ron"))
        .unwrap()
        .1;
    // Before its prerequisite; a mythic feat without one leaves it out.
    assert!(
        feats.contains("extra_pick: MythicAbility, prerequisite: Feat(\"power-attack\"))"),
        "{feats}"
    );
    assert_eq!(feats.matches("extra_pick").count(), 1, "{feats}");

    let dir = tempfile::tempdir().unwrap();
    extract::output::write_files(dir.path(), &files, false).unwrap();
    assert_eq!(seeds::read_game_data_files(dir.path()).unwrap(), data);
}
