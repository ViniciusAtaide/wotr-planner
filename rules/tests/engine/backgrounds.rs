//! Backgrounds (ticket 34): chosen at Build start, they make their skills class skills and grant their feats, which
//! count for prerequisites like any other granted feat.

use std::collections::BTreeSet;

use rules::{
    ArmourType, BackgroundKey, Build, Choice, ClassChoice, Edit, Issue, Parameter, Problem,
};

use crate::support::*;

fn with_background(mut build: Build, key: &str) -> Build {
    build.start.background = Some(BackgroundKey::new(key));
    build
}

fn wizard(background: Option<&str>) -> Build {
    let build = build("dwarf", &["wizard"]);
    match background {
        Some(key) => with_background(build, key),
        None => build,
    }
}

fn class_skills(build: &Build, level: u8) -> BTreeSet<String> {
    engine()
        .evaluate(build)
        .at(level)
        .unwrap()
        .class_skills
        .iter()
        .map(|skill| skill.to_string())
        .collect()
}

fn strings(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|key| (*key).to_owned()).collect()
}

// ---- class skills ---------------------------------------------------------------------------------------------

#[test]
fn a_backgrounds_skills_are_class_skills() {
    assert_eq!(
        class_skills(&wizard(None), 1),
        strings(&["knowledge-arcana"])
    );
    assert_eq!(
        class_skills(&wizard(Some("shackles-corsair")), 1),
        strings(&["athletics", "knowledge-arcana", "mobility"])
    );
}

#[test]
fn class_skills_are_every_class_takens() {
    let build = build("dwarf", &["fighter", "wizard"]);
    assert_eq!(class_skills(&build, 1), strings(&["athletics"]));
    assert_eq!(
        class_skills(&build, 2),
        strings(&["athletics", "knowledge-arcana"])
    );
}

#[test]
fn an_archetype_that_replaces_its_classs_skills_gives_its_own() {
    let mut data = world();
    let rogue = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == "rogue")
        .unwrap();
    let archetype = rogue
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == "rogue/eldritch-scoundrel")
        .unwrap();
    archetype.class_skills = Some(BTreeSet::from([skill("knowledge-arcana")]));
    let engine = rules::Engine::new(data);
    let mut build = build("dwarf", &["rogue"]);
    let plain: BTreeSet<String> = engine
        .evaluate(&build)
        .at(1)
        .unwrap()
        .class_skills
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(plain, strings(&["athletics", "mobility", "persuasion"]));
    build.class_choices.insert(
        class("rogue"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("rogue/eldritch-scoundrel")),
            ..ClassChoice::default()
        },
    );
    let scoundrel: BTreeSet<String> = engine
        .evaluate(&build)
        .at(1)
        .unwrap()
        .class_skills
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(scoundrel, strings(&["knowledge-arcana"]));
}

// ---- granted feats --------------------------------------------------------------------------------------------

#[test]
fn a_backgrounds_feat_is_held_from_the_start_whatever_its_prerequisites() {
    // Dex 10: the world's Improved Unarmed Strike needs 13, but a granted feat skips its prerequisites.
    let mut build = wizard(Some("martial-disciple"));
    build.start.ability_scores = scores([16, 10, 12, 14, 10, 10]);
    assert_eq!(engine().validate(&build), []);
    let state = engine()
        .state_before(&build, &general_slot(1))
        .expect("level 1's general slot");
    assert_eq!(state.feat_count(&feat("improved-unarmed-strike")), 1);
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet
            .at(1)
            .unwrap()
            .feat_count(&feat("improved-unarmed-strike")),
        1
    );
}

fn armor_focus_light() -> rules::Pick {
    pick_feat_for("armor-focus", Parameter::ArmourType(ArmourType::Light))
}

#[test]
fn a_backgrounds_feat_counts_for_prerequisites() {
    let mut guard = wizard(Some("guard"));
    guard.picks.insert(general_slot(1), armor_focus_light());
    assert_eq!(engine().validate(&guard), []);

    let mut plain = wizard(None);
    plain.picks.insert(general_slot(1), armor_focus_light());
    let problems = engine().validate(&plain);
    assert!(
        matches!(
            problems.as_slice(),
            [Problem {
                at: Choice::Slot(slot),
                issue: Issue::Unmet(_)
            }] if *slot == general_slot(1)
        ),
        "{problems:?}"
    );
}

#[test]
fn a_backgrounds_feat_for_one_value_is_held_for_that_value_only() {
    let sheet = engine().evaluate(&wizard(Some("aldori-swordsman")));
    let state = sheet.at(1).unwrap();
    let proficiency = feat("exotic-weapon-proficiency");
    assert!(state.has_feat_for(&proficiency, &weapon("dueling-sword")));
    assert!(!state.has_feat_for(&proficiency, &weapon("bastard-sword")));
}

#[test]
fn the_background_none_grants_nothing() {
    let none = engine().evaluate(&wizard(Some("none")));
    let plain = engine().evaluate(&wizard(None));
    assert_eq!(none.at(1), plain.at(1));
    assert_eq!(none.problems, []);
}

// ---- the choice -----------------------------------------------------------------------------------------------

#[test]
fn an_unknown_background_is_reported() {
    let build = wizard(Some("cut-in-a-later-patch"));
    assert_eq!(
        engine().validate(&build),
        [Problem {
            at: Choice::Background,
            issue: Issue::UnknownKey("cut-in-a-later-patch".into())
        }]
    );
}

#[test]
fn changing_the_background_would_break_what_its_feat_allowed() {
    let mut build = wizard(Some("guard"));
    build.picks.insert(general_slot(1), armor_focus_light());
    let mut start = build.start.clone();
    start.background = Some(BackgroundKey::new("shackles-corsair"));
    let broken: Vec<Choice> = engine()
        .would_break(&build, &Edit::Start(start))
        .into_iter()
        .map(|problem| problem.at)
        .collect();
    assert_eq!(broken, [Choice::Slot(general_slot(1))]);
}
