//! The build sheet (ticket 87): a Build as text a player reads like a build sheet and follows in the game. The
//! header and the Build's link, the Build start, the path choices, each level's class and picks by slot label, then
//! the Mythic track; derived values only as `#` comments.

use rules::{
    Ability, Alignment, Build, BuildStart, ClassChoice, DeityKey, Edit, Faith, SlotId, encode_build,
};

use crate::support::*;

/// The planner's address before the fragment, as the page gives it.
const PAGE: &str = "https://example.test/planner/";

fn sheet(build: &Build) -> String {
    let engine = engine();
    assert!(engine.is_possible(build), "the Build is possible");
    engine.build_sheet(build, PAGE)
}

fn link(build: &Build) -> String {
    format!("{PAGE}#b={}", encode_build(build))
}

/// The lines of the sheet from the first that starts with `from` up to the next blank line.
fn block(sheet: &str, from: &str) -> Vec<String> {
    sheet
        .lines()
        .skip_while(|line| !line.starts_with(from))
        .take_while(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_blank_build_gives_the_header_its_link_and_the_point_buy() {
    let blank = Build::default();
    assert_eq!(
        sheet(&blank),
        format!(
            "WotR build v1\n{}\nPoint buy: Str 10, Dex 10, Con 10, Int 10, Wis 10, Cha 10\n",
            link(&blank)
        )
    );
}

#[test]
fn the_build_start_comes_in_the_order_the_game_asks_for_it() {
    let build = Build {
        name: Some("Spear of the Balance".into()),
        start: BuildStart {
            race: Some(rules::RaceKey::new("half-elf")),
            subrace: Some(rules::SubraceKey::new("half-elf/dual-heritage")),
            racial_ability: Some(Ability::Strength),
            alignment: Some(Alignment::TrueNeutral),
            background: Some(rules::BackgroundKey::new("guard")),
            deity: Some(Faith::Deity(DeityKey::new("balance"))),
            ability_scores: point_buy([16, 14, 12, 13, 10, 8]),
        },
        ..Build::default()
    };
    assert_eq!(
        sheet(&build),
        format!(
            "WotR build v1\n{}\n\
             Name: Spear of the Balance\n\
             Race: Half-Elf\n\
             Subrace: Dual Heritage\n\
             Racial +2: Strength\n\
             Deity: Balance\n\
             Alignment: True Neutral\n\
             Background: guard\n\
             Point buy: Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8\n",
            link(&build)
        )
    );
}

#[test]
fn atheism_is_the_faith_s_name_on_the_sheet() {
    let atheist = build("dwarf", &[]);
    assert!(sheet(&atheist).contains("\nDeity: Atheism\nAlignment: True Neutral\n"));
}

/// A human fighter of four levels: a Combat feat (for a weapon), the race's and the general feat, two skills at
/// level 1 and the ability increase at 4.
fn fighter() -> Build {
    let mut build = build("human", &times("fighter", 4));
    build.start.racial_ability = Some(Ability::Strength);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build.picks.insert(race_slot(0), pick_feat("toughness"));
    build
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    build.levels[0].skill_ranks.insert(skill("mobility"), 1);
    build.levels[0].skill_ranks.insert(skill("athletics"), 1);
    build.levels[3].ability_increase = Some(Ability::Strength);
    build
}

#[test]
fn each_level_gives_its_class_its_picks_by_slot_label_in_slot_order_and_its_skills_on_one_line() {
    let engine = engine();
    let build = fighter();
    let text = sheet(&build);
    let label = |id: &SlotId| engine.slot_label(engine.evaluate(&build).slot(id).unwrap());
    let name = |id: &SlotId| engine.pick_name(&build.picks[id]);
    // The class's feat slots, the race's, then the general slot: the order the sheet checks picks in.
    let ids = [class_slot(1, 0), race_slot(0), general_slot(1)];
    let mut expected = vec!["Level 1: Fighter".to_owned()];
    expected.extend(
        ids.iter()
            .map(|id| format!("  {}: {}", label(id), name(id))),
    );
    expected.push(format!(
        "  Skills: {} 1, {} 1",
        engine.skill_name(&skill("athletics")),
        engine.skill_name(&skill("mobility"))
    ));
    assert_eq!(block(&text, "Level 1:"), expected);
    assert_eq!(
        expected[1], "  Combat feat: weapon-focus (longsword)",
        "the test world names things by key"
    );
    // A level with nothing chosen still names its class; the ability increase comes first in its level.
    assert_eq!(block(&text, "Level 2:"), ["Level 2: Fighter"]);
    assert_eq!(
        block(&text, "Level 4:"),
        ["Level 4: Fighter", "  Ability: +1 Strength"]
    );
    // Levels in order, each after a blank line, and the sheet ends with a newline.
    let heads: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("Level "))
        .collect();
    assert_eq!(heads.len(), 4);
    assert!(text.contains("\n\nLevel 1: Fighter\n"));
    assert!(text.ends_with('\n') && !text.ends_with("\n\n"));
}

#[test]
fn a_class_s_archetype_shows_where_the_class_is_first_taken() {
    let mut build = build("human", &times("fighter", 2));
    build.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    let text = sheet(&build);
    assert!(text.contains("\nLevel 1: Fighter (Armiger)\n"), "{text}");
    assert!(text.contains("\nLevel 2: Fighter\n"), "{text}");
}

#[test]
fn a_prestige_class_names_the_classes_it_advances_where_it_is_first_taken() {
    let mut build = build(
        "elf",
        &[
            "wizard",
            "wizard",
            "wizard",
            "cleric",
            "cleric",
            "cleric",
            "mystic-theurge",
            "mystic-theurge",
        ],
    );
    for level in build.levels.iter_mut().take(4) {
        level.skill_ranks.insert(skill("mobility"), 1);
    }
    build.class_choices.insert(
        class("mystic-theurge"),
        ClassChoice {
            archetype: None,
            advances: [class("wizard"), class("cleric")].into(),
        },
    );
    let text = sheet(&build);
    assert!(
        text.contains("\nLevel 7: Mystic Theurge, advancing Cleric and Wizard\n"),
        "{text}"
    );
    assert!(text.contains("\nLevel 8: Mystic Theurge\n"), "{text}");
}

/// A fighter of 20 levels on the Trickster path with a general mythic pick at every rank, two tricks and a bonus
/// feat from the path, and the feats the mythic feats need.
fn trickster() -> Build {
    let mut build = build("human", &times("fighter", 20));
    build.mythic_path = Some(path("trickster"));
    let abilities = [
        "mythic-charge",
        "limitless-smite",
        "unstoppable",
        "inspired-strike",
        "dimensional-retribution",
    ];
    let feats = [
        "extra-mythic-ability",
        "improved-critical-mythic",
        "dodge-mythic",
        "power-attack-mythic",
        "toughness-mythic",
    ];
    for (i, key) in abilities.iter().enumerate() {
        let rank = 2 * i as u8 + 1;
        build
            .picks
            .insert(SlotId::Mythic { rank }, pick_mythic_ability(key));
    }
    for (i, key) in feats.iter().enumerate() {
        let rank = 2 * i as u8 + 2;
        build
            .picks
            .insert(SlotId::Mythic { rank }, pick_mythic_feat(key));
    }
    build
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    build.picks.insert(race_slot(0), pick_feat("toughness"));
    build.picks.insert(
        SlotId::PathSelection { rank: 3, index: 0 },
        pick_option("trickster/tricks/perception-1"),
    );
    build
        .picks
        .insert(SlotId::PathFeat { rank: 4, index: 0 }, pick_feat("dodge"));
    build
}

#[test]
fn the_path_choices_close_the_header_and_the_mythic_track_lists_each_rank_s_picks() {
    let engine = engine();
    let mut build = trickster();
    build.first_ascension = Some(path("trickster"));
    let text = sheet(&build);
    let evaluated = engine.evaluate(&build);
    assert!(
        text.contains(&format!(
            "\nPoint buy: Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8\nFirst ascension: {}\nMythic path: \
             Trickster\n\nLevel 1:",
            engine.first_ascension_name(&path("trickster"))
        )),
        "{text}"
    );
    // The Mythic track follows the levels: each rank with a pick, its character level, then its picks in slot order.
    let track: Vec<&str> = text
        .split("\n\nMythic track\n")
        .nth(1)
        .expect("a Mythic track")
        .lines()
        .collect();
    let mut expected = Vec::new();
    for rank in evaluated
        .ranks
        .iter()
        .filter(|rank| rank.slots.iter().any(|slot| slot.pick.is_some()))
    {
        expected.push(format!("MR {} (level {})", rank.rank, rank.character_level));
        for slot in rank.slots.iter().filter(|slot| slot.pick.is_some()) {
            expected.push(format!(
                "  {}: {}",
                engine.slot_label(slot),
                engine.pick_name(slot.pick.as_ref().unwrap())
            ));
        }
    }
    assert_eq!(track, expected);
    assert_eq!(track[0], "MR 1 (level 6)");
    assert_eq!(track.len(), 10 + 10 + 2, "{track:?}");
}

#[test]
fn a_pick_a_path_switch_takes_away_is_marked_by_a_comment() {
    let engine = engine();
    let mut build = trickster();
    engine.apply(&mut build, &Edit::PathSwitch(Some(path("gold-dragon"))));
    let text = sheet(&build);
    assert!(
        text.contains("\nMythic path: Trickster\nPath switch: Gold Dragon\n"),
        "{text}"
    );
    let lost: Vec<&str> = text
        .lines()
        .filter(|line| line.ends_with(" # lost to the Path switch"))
        .collect();
    let evaluated = engine.evaluate(&build);
    let dropped: Vec<String> = evaluated
        .ranks
        .iter()
        .flat_map(|rank| rank.dropped_slots.iter())
        .map(|id| {
            let slot = evaluated.slot(id).unwrap();
            format!(
                "  {}: {} # lost to the Path switch",
                engine.slot_label(slot),
                engine.pick_name(slot.pick.as_ref().unwrap())
            )
        })
        .collect();
    assert_eq!(lost, dropped);
    assert_eq!(lost.len(), 2, "{text}");
}

#[test]
fn the_second_line_reopens_the_same_build() {
    let engine = engine();
    let mut switched = trickster();
    engine.apply(&mut switched, &Edit::PathSwitch(Some(path("gold-dragon"))));
    for build in [Build::default(), fighter(), trickster(), switched] {
        let text = sheet(&build);
        let line = text.lines().nth(1).unwrap();
        let code = line.strip_prefix(&format!("{PAGE}#b=")).expect("the link");
        assert_eq!(engine.open_link(code).as_ref(), Ok(&build));
        // The same Build always gives the same text.
        assert_eq!(engine.build_sheet(&build, PAGE), text);
    }
}
