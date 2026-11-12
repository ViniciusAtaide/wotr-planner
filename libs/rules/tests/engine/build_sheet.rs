//! The build sheet (ticket 87): a Build as text a player reads like a build sheet and follows in the game. The
//! header and the Build's link, the Build start, the path choices, each level's class and picks by slot label, then
//! the Mythic track; derived values only as `#` comments.

use rules::{
    Ability, Alignment, Build, BuildStart, ClassChoice, DeityKey, Edit, Faith, GearPlace, SlotId,
    encode_build,
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

/// A half-elf of the Balance with every Build start choice made and no level.
fn spear_of_the_balance() -> Build {
    Build {
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
    }
}

#[test]
fn the_build_start_comes_in_the_order_the_game_asks_for_it() {
    let build = spear_of_the_balance();
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

/// An elf wizard 3 / cleric 3 / Mystic Theurge 2 advancing both, with a rank in Mobility at each of the first four
/// levels.
fn mystic_theurge() -> Build {
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
    build
}

#[test]
fn a_prestige_class_names_the_classes_it_advances_where_it_is_first_taken() {
    let text = sheet(&mystic_theurge());
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

/// What the build sheet of `8d8be9b` wrote for [`spear_of_the_balance`], after the header and the link.
const SPEAR_OF_THE_BALANCE_V1: &str = r#"Name: Spear of the Balance
Race: Half-Elf
Subrace: Dual Heritage
Racial +2: Strength
Deity: Balance
Alignment: True Neutral
Background: guard
Point buy: Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8
"#;

/// What the build sheet of `8d8be9b` wrote for [`fighter`], after the header and the link.
const FIGHTER_V1: &str = r#"Race: Human
Racial +2: Strength
Deity: Atheism
Alignment: True Neutral
Point buy: Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8

Level 1: Fighter
  Combat feat: weapon-focus (longsword)
  Bonus feat: toughness
  Feat: power-attack
  Skills: Athletics 1, Mobility 1

Level 2: Fighter

Level 3: Fighter

Level 4: Fighter
  Ability: +1 Strength
"#;

/// What the build sheet of `8d8be9b` wrote for an Armiger of two levels, after the header and the link.
const ARMIGER_V1: &str = r#"Race: Human
Deity: Atheism
Alignment: True Neutral
Point buy: Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8

Level 1: Fighter (Armiger)

Level 2: Fighter
"#;

/// What the build sheet of `8d8be9b` wrote for [`mystic_theurge`], after the header and the link.
const MYSTIC_THEURGE_V1: &str = r#"Race: Elf
Deity: Atheism
Alignment: True Neutral
Point buy: Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8

Level 1: Wizard
  Skills: Mobility 1

Level 2: Wizard
  Skills: Mobility 1

Level 3: Wizard
  Skills: Mobility 1

Level 4: Cleric
  Skills: Mobility 1

Level 5: Cleric

Level 6: Cleric

Level 7: Mystic Theurge, advancing Cleric and Wizard

Level 8: Mystic Theurge
"#;

/// What the build sheet of `8d8be9b` wrote for [`trickster`] with its first ascension, switched to Gold Dragon, after the header and the link.
const SWITCHED_TRICKSTER_V1: &str = r#"Race: Human
Deity: Atheism
Alignment: True Neutral
Point buy: Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8
First ascension: Bit of Fun
Mythic path: Trickster
Path switch: Gold Dragon

Level 1: Fighter
  Bonus feat: toughness
  Feat: power-attack

Level 2: Fighter

Level 3: Fighter

Level 4: Fighter

Level 5: Fighter

Level 6: Fighter

Level 7: Fighter

Level 8: Fighter

Level 9: Fighter

Level 10: Fighter

Level 11: Fighter

Level 12: Fighter

Level 13: Fighter

Level 14: Fighter

Level 15: Fighter

Level 16: Fighter

Level 17: Fighter

Level 18: Fighter

Level 19: Fighter

Level 20: Fighter

Mythic track
MR 1 (level 6)
  Mythic ability: mythic-charge
MR 2 (level 7)
  Mythic feat: extra-mythic-ability
MR 3 (level 9)
  Mythic ability: limitless-smite
  Trickster Tricks: perception-1 # lost to the Path switch
MR 4 (level 11)
  Mythic feat: improved-critical-mythic
  trickster/bonus-feats: dodge # lost to the Path switch
MR 5 (level 14)
  Mythic ability: unstoppable
MR 6 (level 15)
  Mythic feat: dodge-mythic
MR 7 (level 16)
  Mythic ability: inspired-strike
MR 8 (level 17)
  Mythic feat: power-attack-mythic
MR 9 (level 17)
  Mythic ability: dimensional-retribution
MR 10 (level 19)
  Mythic feat: toughness-mythic
"#;

/// The Build sheet's format is "WotR build v1": its line keys are its own, not the labels the sheet and the pickers
/// show, so rewording a label never changes an export. Each text is the one `8d8be9b` (before ticket 152) wrote for
/// the same Build, byte for byte.
#[test]
fn the_v1_text_is_byte_for_byte_the_one_ticket_87_wrote() {
    let engine = engine();
    let mut armiger = build("human", &times("fighter", 2));
    armiger.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    let mut switched = trickster();
    switched.first_ascension = Some(path("trickster"));
    engine.apply(&mut switched, &Edit::PathSwitch(Some(path("gold-dragon"))));
    for (build, v1) in [
        (spear_of_the_balance(), SPEAR_OF_THE_BALANCE_V1),
        (fighter(), FIGHTER_V1),
        (armiger, ARMIGER_V1),
        (mystic_theurge(), MYSTIC_THEURGE_V1),
        (switched, SWITCHED_TRICKSTER_V1),
    ] {
        assert_eq!(
            sheet(&build),
            format!("WotR build v1\n{}\n{v1}", link(&build))
        );
    }
}

/// [`fighter`] with gear in three places (the second ring's among them) and two usable items.
fn geared_fighter() -> Build {
    let build = wearing(fighter(), GearPlace::Belt, "belt-of-dexterity-2");
    let build = wearing(build, GearPlace::SecondRing, "ring-of-luck");
    let build = wearing(build, GearPlace::Body, "robe-of-the-mage");
    let build = using(build, "tome-of-perfection-4");
    using(build, "manual-of-dexterity-2")
}

/// The build sheet of [`geared_fighter`] as ticket 167 first wrote it, after the header and the link: [`FIGHTER_V1`]
/// with the gear's block between the start and the levels.
const GEARED_FIGHTER_V1: &str = r#"Race: Human
Racial +2: Strength
Deity: Atheism
Alignment: True Neutral
Point buy: Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8

Gear
  Belt: Belt of Dexterity +2
  Ring 2: Ring of Luck
  Body: Robe of the Mage
  Used: Manual of Dexterity +2
  Used: Tome of Perfection +4

Level 1: Fighter
  Combat feat: weapon-focus (longsword)
  Bonus feat: toughness
  Feat: power-attack
  Skills: Athletics 1, Mobility 1

Level 2: Fighter

Level 3: Fighter

Level 4: Fighter
  Ability: +1 Strength
"#;

/// A Build with gear gets a block of its own after the start: one line per item worn, in the places' order, then
/// one per usable item used, by key, each with its own key. The point buy line is still the point buy.
#[test]
fn a_build_with_gear_lists_it_between_the_start_and_the_levels() {
    let build = geared_fighter();
    assert_eq!(
        sheet(&build),
        format!("WotR build v1\n{}\n{GEARED_FIGHTER_V1}", link(&build))
    );
    // The gear is all the two sheets differ by, but the link.
    let without: String = GEARED_FIGHTER_V1
        .split("\n\n")
        .filter(|block| !block.starts_with("Gear\n"))
        .collect::<Vec<_>>()
        .join("\n\n");
    assert_eq!(without, FIGHTER_V1);
    // The second line still reopens the exact Build, gear and all.
    let text = sheet(&build);
    let code = text
        .lines()
        .nth(1)
        .unwrap()
        .strip_prefix(&format!("{PAGE}#b="))
        .unwrap();
    assert_eq!(engine().open_link(code), Ok(build));
}

/// Ticket 171: a rank taken later than the rank table's level says so on its line, with or without a pick; a rank
/// it takes along does too; and a Build that moves no rank gives the text it always did (the v1 test above).
#[test]
fn a_rank_taken_later_says_so_on_its_line_and_no_other_line_changes() {
    let engine = engine();
    let mut build = build("human", &times("fighter", 13));
    build.mythic_path = Some(path("trickster"));
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("mythic-charge"),
    );
    let before = sheet(&build);
    assert!(
        before.ends_with("\n\nMythic track\nMR 1 (level 6)\n  Mythic ability: mythic-charge\n")
    );

    let mut later = build.clone();
    engine.apply(
        &mut later,
        &Edit::RankLevel {
            rank: 3,
            level: Some(12),
        },
    );
    let text = sheet(&later);
    let track = text.split("\n\nMythic track\n").nth(1).unwrap();
    assert_eq!(
        track,
        "MR 1 (level 6)\n  Mythic ability: mythic-charge\nMR 3 (level 12, taken later than level 9)\nMR 4 (level \
         12, taken later than level 11)\n"
    );
    // The link aside, nothing before the track differs.
    let head = |text: &str| -> Vec<String> {
        let head = text.split("\n\nMythic track\n").next().unwrap();
        head.lines()
            .filter(|line| !line.contains("#b="))
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(head(&text), head(&before));
}
