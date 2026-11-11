//! Initiative, saves and skill totals on the real data (ticket 120): the bonuses of `bonuses.ron` on the Max DC
//! run's Build (Pickpocket, Improved Initiative and its mythic feat, Iron Will, a Kitsune with 20 ranks of
//! Persuasion), a halfling paladin's saves, and Skill Focus rising at 10 ranks. And the interactions notes of
//! `interactions.ron` (ticket 124).

use rules::{
    Alignment, BackgroundKey, BonusItem, Build, Engine, FeatKey, MythicAbilityKey, MythicFeatKey,
    Pick, RaceKey, SelectionOptionKey, SkillKey, Total, Totals,
};

use crate::support::*;

/// The Max DC run's Build, as its link encodes it (2026-11-09; the caster line's test has it too).
const MAX_DC_RUN: &str = "AdWOPUgDQRCFd3bPE2OyAQnYSNzGSi0sbKzU8xqDKIg_oEiWsKAGznCeKCkkWIigEBALU1gaTCdYqCjpVNAmYCFBEAwpIldYWCro7uFPKot0KebxzfDmzfCnyGpf4Gzwsxh96C98uNmel_qCEpue8AsHyqPtY_utHU3ddLtwNBfP5dxKkt4HZi-n7nqzW-mD2_UNQ0vtpheLummEu8pXbuI8_BaajCaO-Yzkw7bN5vj1c3DHzJ8M31RO85n3zCu-eJT50BlZcJZXLMFMKzbPLUfYrJTaYyN8jQ0ZbCDJHc7Gl-yYsIUNCBHAjZT6_C0hhAF-Sq_iWnsZDkhHVAFCWJ4i-Fel6XuK8J_8O1OJMtwHGAghnoJS5KHuoaZpylNVDRBUb3i78AU";

/// The totals at the end of each level of a Build.
fn totals(engine: &Engine, build: &Build) -> Vec<Totals> {
    engine
        .evaluate(build)
        .levels
        .iter()
        .map(|level| engine.totals(level))
        .collect()
}

fn parts(total: &Total) -> Vec<(&str, i16)> {
    total
        .parts
        .iter()
        .map(|part| (part.name.as_str(), part.value))
        .collect()
}

fn skill_total<'a>(totals: &'a Totals, key: &str) -> &'a Total {
    &totals
        .skill(&SkillKey::new(key))
        .unwrap_or_else(|| panic!("no skill {key}"))
        .total
}

#[test]
fn the_max_dc_runs_initiative_will_and_persuasion_at_20() {
    let engine = engine();
    let build = engine
        .open_link(MAX_DC_RUN)
        .expect("the Max DC run's link opens");
    let last = totals(&engine, &build).pop().expect("20 levels");

    // The run's question: Dex 16, Pickpocket, Improved Initiative and its mythic feat at MR 10.
    assert_eq!(
        parts(&last.initiative),
        [
            ("Dexterity", 3),
            ("Pickpocket", 2),
            ("Improved Initiative", 4),
            ("Improved Initiative (Mythic)", 10),
        ]
    );
    assert_eq!(last.initiative.total, 19);
    assert_eq!(
        parts(&last.will),
        [("Base save", 12), ("Wisdom", 1), ("Iron Will", 2)]
    );
    // Arcane Spirit's check: 20 ranks, Charisma 25, a class skill.
    assert_eq!(
        parts(skill_total(&last, "persuasion")),
        [("Ranks", 20), ("Charisma", 7), ("Class skill", 3)]
    );
    assert_eq!(skill_total(&last, "persuasion").total, 30);
    // The Kitsune's Agile and Keen Senses count untrained.
    assert_eq!(
        parts(skill_total(&last, "mobility")),
        [("Ranks", 0), ("Dexterity", 3), ("Kitsune", 2)]
    );
    assert_eq!(skill_total(&last, "perception").total, 3);
}

#[test]
fn a_halfling_paladins_saves_count_halfling_luck_the_background_great_fortitude_and_divine_grace() {
    let engine = engine();
    let mut build = build("paladin", 2);
    build.start.race = Some(RaceKey::new("halfling"));
    build.start.background = Some(BackgroundKey::new("farmhand"));
    build
        .picks
        .insert(general_slot(1), Pick::feat(FeatKey::new("great-fortitude")));
    let build = realigned(build, Alignment::LawfulGood);
    assert!(engine.is_possible(&build), "the Build is possible");
    let levels = totals(&engine, &build);
    let state = at(&build, 2);
    let charisma = (i16::from(state.ability_scores.charisma) - 10).div_euclid(2);
    let constitution = (i16::from(state.ability_scores.constitution) - 10).div_euclid(2);

    // Divine Grace comes at paladin 2.
    assert!(
        !parts(&levels[0].fortitude)
            .iter()
            .any(|(name, _)| *name == "Divine Grace")
    );
    assert_eq!(
        parts(&levels[1].fortitude),
        [
            ("Base save", i16::from(state.fort)),
            ("Constitution", constitution),
            ("Halfling", 1),
            ("Farmhand", 1),
            ("Great Fortitude", 2),
            ("Divine Grace", charisma),
        ]
    );
    assert!(parts(&levels[1].will).contains(&("Halfling", 1)));
    assert!(parts(&levels[1].will).contains(&("Divine Grace", charisma)));
}

#[test]
fn skill_focus_gives_3_and_6_from_10_ranks() {
    let engine = engine();
    let mut build = build("rogue", 10);
    build.picks.insert(
        general_slot(1),
        Pick::feat_for(FeatKey::new("skill-focus"), skill("persuasion")),
    );
    for level in &mut build.levels {
        level.skill_ranks.insert(SkillKey::new("persuasion"), 1);
    }
    assert!(engine.is_possible(&build), "the Build is possible");
    let levels = totals(&engine, &build);
    let focus = |totals: &Totals| {
        parts(skill_total(totals, "persuasion"))
            .into_iter()
            .find(|(name, _)| *name == "Skill Focus (Persuasion)")
            .map(|(_, value)| value)
    };

    assert_eq!(focus(&levels[8]), Some(3));
    assert_eq!(focus(&levels[9]), Some(6));
}

#[test]
fn the_dc_runs_five_items_have_their_interactions_notes() {
    let engine = engine();
    let note = |item: BonusItem| {
        engine
            .interactions(&item)
            .unwrap_or_else(|| panic!("no note on {item:?}"))
            .to_owned()
    };

    let favorable = note(BonusItem::SelectionOption(SelectionOptionKey::new(
        "azata/superpowers/favorable-magic",
    )));
    let persistent = note(BonusItem::Feat(FeatKey::new("metamagic-persistent-spell")));
    let arsenal = note(BonusItem::MythicFeat(MythicFeatKey::new(
        "expanded-arsenal",
    )));
    let second = note(BonusItem::MythicAbility(MythicAbilityKey::new(
        "second-bloodline",
    )));
    let ascendance = note(BonusItem::MythicAbility(MythicAbilityKey::new(
        "bloodline-ascendance",
    )));
    let fey = note(BonusItem::SelectionOption(SelectionOptionKey::new(
        "sorcerer/bloodline/fey",
    )));

    // Rows 1 and 4: the guide doesn't say.
    assert!(favorable.contains("the guide doesn't say"), "{favorable}");
    assert!(persistent.contains("the guide doesn't say"), "{persistent}");
    assert!(ascendance.contains("doesn't say"), "{ascendance}");
    // Rows 2, 3 and 5: as the caster line counts them (ticket 119).
    assert!(arsenal.contains("Greater Spell Focus"), "{arsenal}");
    assert!(second.contains("arcana"), "{second}");
    assert!(fey.contains("doesn't say"), "{fey}");
}
