//! Opening a link (ticket 40): a link the current game data can't hold opens nothing, whatever the reason (a key
//! gone, a pick that no longer fits, a rule broken, a choice the Build doesn't reach, a link that doesn't decode),
//! and the sheet shows one generic error and a blank Build. A link that still fits loads unchanged, a pick a Path
//! switch takes away at an earlier rank included (ticket 42).

use rules::{
    Ability, Build, ClassChoice, Edit, Engine, LevelUp, Prerequisite, SlotId, StaleLink,
    encode_build,
};

use crate::support::*;

/// Elf fighter 3: Power Attack (F1 bonus), Cleave (L1), Toughness (L3), Mobility 1 at level 1.
fn fighter() -> Build {
    let mut build = build("elf", &times("fighter", 3));
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    build.picks.insert(general_slot(1), pick_feat("cleave"));
    build.picks.insert(general_slot(3), pick_feat("toughness"));
    build.levels[0].skill_ranks.insert(skill("mobility"), 1);
    build
}

fn open(engine: &Engine, build: &Build) -> Result<Build, StaleLink> {
    engine.open_link(&encode_build(build))
}

#[test]
fn a_link_that_still_fits_loads_unchanged() {
    let build = fighter();
    assert!(engine().is_possible(&build));
    assert_eq!(open(&engine(), &build), Ok(build));
    assert_eq!(open(&engine(), &Build::default()), Ok(Build::default()));
}

#[test]
fn a_link_with_a_key_the_game_data_lacks_is_stale() {
    let engine = engine();
    let gone: [fn(&mut Build); 8] = [
        |b| {
            b.picks
                .insert(general_slot(3), pick_feat("cut-in-a-later-patch"));
        },
        |b| b.levels[2].class = class("cut-in-a-later-patch"),
        |b| {
            b.levels[0]
                .skill_ranks
                .insert(skill("cut-in-a-later-patch"), 1);
        },
        |b| b.start.race = Some(rules::RaceKey::new("cut-in-a-later-patch")),
        |b| b.start.background = Some(rules::BackgroundKey::new("cut-in-a-later-patch")),
        |b| {
            b.class_choices.insert(
                class("fighter"),
                ClassChoice {
                    archetype: Some(rules::ArchetypeKey::new("fighter/cut-in-a-later-patch")),
                    ..ClassChoice::default()
                },
            );
        },
        // A path and a switch at ranks the Build reaches.
        |b| {
            b.levels.resize(17, LevelUp::new(class("fighter")));
            b.mythic_path = Some(path("cut-in-a-later-patch"));
        },
        |b| {
            b.levels.resize(17, LevelUp::new(class("fighter")));
            b.mythic_path = Some(path("trickster"));
            b.path_switch = Some(path("cut-in-a-later-patch"));
        },
    ];
    for (index, change) in gone.into_iter().enumerate() {
        let mut build = fighter();
        change(&mut build);
        assert!(!engine.is_possible(&build), "change {index}");
        assert_eq!(open(&engine, &build), Err(StaleLink), "change {index}");
    }
}

#[test]
fn a_link_whose_pick_no_longer_fits_is_stale() {
    let build = fighter();
    assert_eq!(open(&engine(), &build), Ok(build.clone()));

    // The game data changes: Power Attack now needs Str 18, which breaks it and Cleave with it.
    let mut data = world();
    let power_attack = data
        .feats
        .iter_mut()
        .find(|f| f.key == feat("power-attack"))
        .unwrap();
    power_attack.prerequisite = Some(Prerequisite::AbilityScore {
        ability: Ability::Strength,
        score: 18,
    });
    let stale = engine_on(data);
    assert!(!stale.is_possible(&build));
    assert_eq!(open(&stale, &build), Err(StaleLink));
}

#[test]
fn a_link_that_breaks_a_rule_is_stale() {
    let engine = engine();
    // A rule of the game data: a wizard's 3 skill points (2, Int 13) overspent.
    let mut overspent = build("dwarf", &["wizard"]);
    for key in ["athletics", "mobility", "persuasion", "knowledge-arcana"] {
        overspent.levels[0].skill_ranks.insert(skill(key), 1);
    }
    // Level 21 without the Legend switch.
    let past_the_cap = build("elf", &times("fighter", 21));
    // A switch-only path chosen at MR 3.
    let mut legend_at_3 = build("elf", &times("fighter", 9));
    legend_at_3.mythic_path = Some(path("legend"));
    for build in [overspent, past_the_cap, legend_at_3] {
        assert_eq!(open(&engine, &build), Err(StaleLink), "{build:?}");
    }

    // A fixed rule, which the link decoder keeps: two ranks at level 1, or an ability increase at level 3.
    let mut ranks = fighter();
    ranks.levels[0].skill_ranks.insert(skill("mobility"), 2);
    let mut increase = fighter();
    increase.levels[2].ability_increase = Some(Ability::Strength);
    for build in [ranks, increase] {
        assert_eq!(open(&engine, &build), Err(StaleLink), "{build:?}");
    }
}

#[test]
fn a_link_with_a_choice_the_build_does_not_reach_is_stale() {
    let engine = engine();
    // A pick for a slot level 2 doesn't have, and one for a level the Build doesn't have.
    let mut no_such_slot = fighter();
    no_such_slot
        .picks
        .insert(general_slot(2), pick_feat("dodge"));
    let mut past_the_end = fighter();
    past_the_end
        .picks
        .insert(general_slot(5), pick_feat("dodge"));
    // A mythic pick for a rank a 3-level Build doesn't reach.
    let mut unreached = fighter();
    unreached.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("mythic-charge"),
    );
    // A class choice for a class no level takes.
    let mut unused = fighter();
    unused
        .class_choices
        .insert(class("rogue"), ClassChoice::default());
    // A path before MR 3 (level 9), and a switch before MR 8 (level 17).
    let mut early_path = fighter();
    early_path.mythic_path = Some(path("trickster"));
    let mut early_switch = build("elf", &times("fighter", 16));
    early_switch.mythic_path = Some(path("trickster"));
    assert!(engine.is_possible(&early_switch));
    early_switch.path_switch = Some(path("legend"));
    for build in [
        no_such_slot,
        past_the_end,
        unreached,
        unused,
        early_path,
        early_switch,
    ] {
        assert!(!engine.is_possible(&build), "{build:?}");
        assert_eq!(open(&engine, &build), Err(StaleLink), "{build:?}");
    }
}

#[test]
fn a_link_holding_a_pick_a_switch_takes_away_loads_unchanged() {
    let engine = engine();
    let on = |path_key: &str| {
        let mut build = build("human", &times("fighter", 17));
        build.mythic_path = Some(path(path_key));
        build
    };
    // Legend takes MR 3's general pick away, Gold Dragon the Trickster's MR 3 trick, Devil the Aeon's gaze.
    let cases = [
        (
            on("trickster"),
            "legend",
            SlotId::Mythic { rank: 3 },
            pick_mythic_ability("mythic-charge"),
        ),
        (
            on("trickster"),
            "gold-dragon",
            SlotId::PathSelection { rank: 3, index: 0 },
            pick_option("trickster/tricks/perception-1"),
        ),
        (
            on("aeon"),
            "devil",
            SlotId::PathSelection { rank: 3, index: 0 },
            pick_option("aeon/enforcing-gaze/attack"),
        ),
    ];
    for (mut build, switch, slot, pick) in cases {
        build.picks.insert(slot.clone(), pick);
        build.path_switch = Some(path(switch));
        // The pick is held from its rank until the switch: the Build is possible, and the sheet's switch keeps it.
        assert!(engine.is_possible(&build), "{switch}");
        assert_eq!(open(&engine, &build), Ok(build.clone()), "{switch}");
        let mut applied = build.clone();
        applied.path_switch = None;
        engine.apply(&mut applied, &Edit::PathSwitch(Some(path(switch))));
        assert_eq!(applied, build, "{switch}");
    }
}

#[test]
fn a_link_that_does_not_decode_is_stale() {
    let engine = engine();
    let code = encode_build(&fighter());
    for bad in ["", "not base64!", "CQ", "Af____8", &code[..code.len() / 2]] {
        assert_eq!(engine.open_link(bad), Err(StaleLink), "{bad}");
    }
    assert_eq!(
        StaleLink.to_string(),
        "This link no longer works with the current game data"
    );
}
