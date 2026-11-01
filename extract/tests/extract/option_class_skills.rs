//! Ticket 37: an option row's class skills, as the files write them.

fn sorcerer_file() -> String {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/tests/fixtures/game-data");
    let data = data::read_game_data_files(&fixture).unwrap();
    extract::ron_out::files(&data)
        .into_iter()
        .find(|(path, _)| path.ends_with("classes/sorcerer.ron"))
        .unwrap()
        .1
}

#[test]
fn an_option_rows_class_skills_come_last_and_a_row_of_them_alone_is_one_field() {
    let sorcerer = sorcerer_file();
    for expected in [
        "feat_list: \"sorcerer/bloodline/abyssal-feats\", class_skills: [\"athletics\"]),\n",
        "(level: 9, class_skills: [\"mobility\", \"persuasion\"]),\n",
    ] {
        assert!(sorcerer.contains(expected), "{expected} in {sorcerer}");
    }
}

#[test]
fn a_row_without_class_skills_leaves_the_field_out() {
    let sorcerer = sorcerer_file();
    let rows: Vec<&str> = sorcerer
        .lines()
        .filter(|line| line.trim_start().starts_with("(level: 3,"))
        .collect();
    assert!(!rows.is_empty());
    assert!(
        rows.iter().all(|row| !row.contains("class_skills")),
        "{rows:?}"
    );
}
