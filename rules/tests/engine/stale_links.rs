//! Opening a link (ticket 40): a link the current game data can't hold opens nothing, whatever the reason (a key
//! gone, a pick that no longer fits, a rule broken, a pick the Build doesn't reach, a link that doesn't decode), and
//! the sheet shows one generic error and a blank Build. A link that still fits loads unchanged.

use rules::{Ability, Build, ClassChoice, Engine, Prerequisite, SlotId, StaleLink, encode_build};

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
        // The path and the switch are checked even while their ranks aren't reached.
        |b| b.mythic_path = Some(path("cut-in-a-later-patch")),
        |b| b.path_switch = Some(path("cut-in-a-later-patch")),
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
    let stale = Engine::new(data);
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
    for build in [no_such_slot, past_the_end, unreached, unused] {
        assert!(!engine.is_possible(&build), "{build:?}");
        assert_eq!(open(&engine, &build), Err(StaleLink), "{build:?}");
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
