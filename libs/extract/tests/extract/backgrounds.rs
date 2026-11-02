//! Ticket 34's backgrounds: the extractor carries the user's list, which is what `libs/game-data/` holds, and the files
//! write and read it back.

fn file<'f>(files: &'f [(std::path::PathBuf, String)], name: &str) -> &'f str {
    &files
        .iter()
        .find(|(path, _)| path.ends_with(name))
        .unwrap_or_else(|| panic!("no {name}"))
        .1
}

#[test]
fn the_users_backgrounds_are_what_game_data_holds() {
    let data = seeds::read_game_data_files(&seeds::game_data_dir()).unwrap();
    let mut carried = extract::backgrounds::backgrounds(&data);
    carried.sort_by(|a, b| a.key.cmp(&b.key));
    assert_eq!(carried.len(), 38);
    assert_eq!(carried, data.backgrounds);
}

#[test]
fn the_files_write_backgrounds_and_read_them_back() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    let data = seeds::read_game_data_files(&fixture).unwrap();
    let files = extract::ron_out::files(&data);
    let backgrounds = file(&files, "backgrounds.ron");
    for expected in [
        // None: no group, note or grants, so only its key and name.
        "(key: \"none\", name: \"None\"),",
        "(key: \"aldori-swordsman\", name: \"Aldori Swordsman\", group: Regional, note: \"buckler\", class_skills: [\"persuasion\"], feats_for: [(feat: \"exotic-weapon-proficiency\", value: Weapon(\"dueling-sword\"))]),",
        "(key: \"martial-disciple\", name: \"Martial Disciple\", group: Oblate, class_skills: [\"mobility\"], feats: [\"improved-unarmed-strike\"]),",
    ] {
        assert!(
            backgrounds.contains(expected),
            "{expected} in {backgrounds}"
        );
    }

    let dir = tempfile::tempdir().unwrap();
    for (path, text) in &files {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    assert_eq!(seeds::read_game_data_files(dir.path()).unwrap(), data);
}
