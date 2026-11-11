//! The build sheet (ticket 87) of the six bench Builds (`tools/proof/bench-links.tsv`): their second line reopens
//! them, each level and each pick has its line, and the parts that matter are pinned.

use crate::support::*;

/// The planner's address before the fragment, as the page gives it.
const PAGE: &str = "https://viniciusataide.github.io/wotr-planner/";

fn sheet_of(name: &str) -> String {
    let (_, build) = bench()
        .into_iter()
        .find(|(bench_name, _)| bench_name == name)
        .unwrap_or_else(|| panic!("no bench Build {name}"));
    engine().build_sheet(&build, PAGE)
}

/// The lines of a sheet from the one that starts with `from` up to the next blank line.
fn block<'a>(sheet: &'a str, from: &str) -> Vec<&'a str> {
    sheet
        .lines()
        .skip_while(|line| !line.starts_with(from))
        .take_while(|line| !line.is_empty())
        .collect()
}

#[test]
fn each_bench_sheet_s_second_line_reopens_its_build() {
    let engine = engine();
    let benches = bench();
    assert_eq!(benches.len(), 6);
    for (name, build) in benches {
        let sheet = engine.build_sheet(&build, PAGE);
        let mut lines = sheet.lines();
        assert_eq!(lines.next(), Some("WotR build v1"), "{name}");
        let code = lines
            .next()
            .and_then(|line| line.strip_prefix(PAGE))
            .and_then(|fragment| fragment.strip_prefix("#b="))
            .unwrap_or_else(|| panic!("{name}: no link on the second line"));
        assert_eq!(engine.open_link(code).as_ref(), Ok(&build), "{name}");
        // The same Build always renders the same text.
        assert_eq!(engine.build_sheet(&build, PAGE), sheet, "{name}");
    }
}

#[test]
fn each_bench_sheet_has_a_line_per_level_in_order_and_per_pick() {
    for (name, build) in bench() {
        let sheet = engine().build_sheet(&build, PAGE);
        let levels: Vec<u8> = sheet
            .lines()
            .filter_map(|line| line.strip_prefix("Level "))
            .map(|rest| rest.split_once(':').unwrap().0.parse().unwrap())
            .collect();
        let expected: Vec<u8> = (1..=build.levels.len() as u8).collect();
        assert_eq!(levels, expected, "{name}");
        // Every indented line but a level's ability increase and skills is one pick.
        let picks = sheet
            .lines()
            .filter(|line| line.starts_with("  "))
            .filter(|line| !line.starts_with("  Ability: ") && !line.starts_with("  Skills: "))
            .count();
        assert_eq!(picks, build.picks.len(), "{name}");
    }
}

#[test]
fn the_blank_and_start_bench_sheets_are_pinned() {
    let link = |name: &str| {
        let (_, build) = bench().into_iter().find(|(n, _)| n == name).unwrap();
        format!("{PAGE}#b={}", rules::encode_build(&build))
    };
    assert_eq!(
        sheet_of("blank"),
        format!(
            "WotR build v1\n{}\nPoint buy: Str 10, Dex 10, Con 10, Int 10, Wis 10, Cha 10\n",
            link("blank")
        )
    );
    assert_eq!(
        sheet_of("start"),
        format!(
            "WotR build v1\n{}\n\
             Race: Human\n\
             Racial +2: Strength\n\
             Deity: Atheism\n\
             Alignment: True Neutral\n\
             Background: Acolyte\n\
             Point buy: Str 17, Dex 10, Con 16, Int 10, Wis 10, Cha 10\n",
            link("start")
        )
    );
}

#[test]
fn the_fighter_s_sheet_gives_the_path_and_each_level_s_picks_by_slot_label() {
    let sheet = sheet_of("fighter20");
    assert!(sheet.contains(
        "\nPoint buy: Str 17, Dex 10, Con 16, Int 10, Wis 10, Cha 10\nMythic path: Aeon\n\nLevel 1: Fighter\n"
    ));
    assert_eq!(
        block(&sheet, "Level 1:"),
        [
            "Level 1: Fighter",
            "  Combat feat: Armor Focus (Light)",
            "  Bonus feat: Alertness",
            "  Feat: Allied Spellcaster",
            "  Skills: Athletics 1, Knowledge (Arcana) 1",
        ]
    );
    assert_eq!(
        block(&sheet, "Level 4:"),
        [
            "Level 4: Fighter",
            "  Ability: +1 Strength",
            "  Combat feat: Armor Focus (Heavy)",
            "  Skills: Athletics 1, Knowledge (Arcana) 1",
        ]
    );
    // The track's ranks follow one another without blank lines.
    assert_eq!(
        block(&sheet, "MR 3 ")[..6],
        [
            "MR 3 (level 9)",
            "  Mythic ability: Always a Chance",
            "  Enforcing Gaze: Enforcing Gaze - Attack",
            "MR 4 (level 11)",
            "  Mythic feat: Emergency Potion",
            "  Enforcing Gaze: Enforcing Gaze - Attacks of Opportunity",
        ]
    );
    assert!(!sheet.contains(" # "), "nothing is lost without a switch");
}

#[test]
fn the_multiclass_sheet_names_each_level_s_class() {
    let sheet = sheet_of("multi_prestige20");
    let classes: Vec<&str> = sheet
        .lines()
        .filter(|line| line.starts_with("Level "))
        .map(|line| line.split_once(": ").unwrap().1)
        .collect();
    let mut expected = vec!["Rogue", "Fighter", "Rogue", "Fighter", "Rogue", "Rogue"];
    expected.extend(["Assassin"; 10]);
    expected.extend(["Rogue"; 4]);
    assert_eq!(classes, expected);
    assert_eq!(
        block(&sheet, "Level 1:"),
        [
            "Level 1: Rogue",
            "  Bonus feat: Accomplished Sneak Attacker",
            "  Feat: Alertness",
            "  Skills: Athletics 1, Knowledge (Arcana) 1, Knowledge (World) 1, Lore (Nature) 1, Lore (Religion) 1, \
             Mobility 1, Perception 1, Stealth 1",
        ]
    );
}

#[test]
fn the_legend_sheet_runs_to_level_40_and_marks_the_picks_the_switch_takes_away() {
    let sheet = sheet_of("legend40");
    assert!(sheet.contains("\nMythic path: Aeon\nPath switch: Legend\n\nLevel 1: Fighter\n"));
    assert!(sheet.contains("\nLevel 21: Alchemist\n"));
    assert_eq!(
        block(&sheet, "Level 40:"),
        [
            "Level 40: Alchemist",
            "  Ability: +1 Strength",
            "  Grand Discovery: Awakened Intellect",
            "  Skills: Athletics 1, Knowledge (Arcana) 1, Knowledge (World) 1, Lore (Nature) 1, Lore (Religion) 1",
        ]
    );
    // Legend ends the ascensions at MR 8 and takes away every pick of MR 3 to 7, which stay on the track, marked.
    let track = sheet.split("\n\nMythic track\n").nth(1).unwrap();
    let ranks: Vec<&str> = track
        .lines()
        .filter(|line| line.starts_with("MR "))
        .collect();
    assert_eq!(
        ranks,
        [
            "MR 1 (level 6)",
            "MR 2 (level 7)",
            "MR 3 (level 9)",
            "MR 4 (level 11)",
            "MR 5 (level 14)",
            "MR 6 (level 15)",
            "MR 7 (level 16)",
        ]
    );
    let lost: Vec<&str> = track
        .lines()
        .filter_map(|line| line.strip_suffix(" # lost to the Path switch"))
        .collect();
    assert_eq!(lost.len(), 10, "{lost:?}");
    assert_eq!(lost[0], "  Mythic ability: Always a Chance");
    // MR 1 and 2 stay.
    assert!(track.lines().take(4).all(|line| !line.contains(" # ")));
}
