//! Ticket 32's follow-ups: which report item they settle, and how the files write and read back the new shapes
//! (feats granted for a value, an option's own value and value list, a race's list, a mythic feat's value, repeat
//! and racial penalty, a feature's floors, `Not` and `FeatIfValue`).

use extract::follow_ups::settled;

fn fixture() -> rules::GameData {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    seeds::read_game_data_files(&fixture).unwrap()
}

#[test]
fn only_the_armor_focus_line_is_settled() {
    assert!(settled(
        "feats#armor-focus",
        "Armor Proficiency (Light. Medium or Heavy)"
    ));
    assert!(!settled("feats#armor-focus", "Base Attack Bonus: 1"));
    assert!(!settled(
        "feats#extra-slayer-talent",
        "Slayer Talents or Talented Slayer"
    ));
}

#[test]
fn the_files_write_the_new_shapes_and_read_them_back() {
    let data = fixture();
    let files = extract::ron_out::files(&data);
    let file = |name: &str| {
        files
            .iter()
            .find(|(path, _)| path.ends_with(name))
            .unwrap_or_else(|| panic!("no {name}"))
            .1
            .clone()
    };
    let rogue = file("classes/rogue.ron");
    for expected in [
        "feats_for: [(feat: \"weapon-focus\", value: Weapon(\"dagger\"))]",
        "feats_for: [(feat: \"skill-focus\", value: Skill(\"mobility\"))]",
        "parameter: Weapon, values: [Weapon(\"dagger\"), Weapon(\"longsword\")]",
    ] {
        assert!(rogue.contains(expected), "{expected} in {rogue}");
    }
    let races = file("races.ron");
    assert!(
        races.contains("feat_slots: [OneOf(list: \"half-elf/adaptability\")]")
            && races.contains("feat_lists: [\n"),
        "{races}"
    );
    let mythic = file("mythic/feats.ron");
    for expected in [
        "parameter: SpellSchool, repeat: PerValue, prerequisite: FeatForSameValue(\"spell-focus\")",
        "repeat: Times(2)",
        "removes_racial_penalty: true, prerequisite: Not(CharacterLevel(11))",
    ] {
        assert!(mythic.contains(expected), "{expected} in {mythic}");
    }
    let feats = file("feats.ron");
    assert!(
        feats.contains("FeatIfValue(value: ArmourType(Heavy), feat: \"toughness\")"),
        "{feats}"
    );
    let dragon = file("mythic/paths/gold-dragon.ron");
    assert!(
        dragon.contains("bab_floor: Some(15), skill_ranks_at_least_mythic_rank: true"),
        "{dragon}"
    );

    // And they read back as the same data.
    let dir = tempfile::tempdir().unwrap();
    for (path, text) in &files {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    assert_eq!(seeds::read_game_data_files(dir.path()).unwrap(), data);
}
