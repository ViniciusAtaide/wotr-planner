//! `GameData::check`: data mistakes that the types can't rule out, found before the data is seeded or used.

use std::collections::BTreeSet;

use rules::{DataIssue, DataProblem, FeatKey, GameData, Prerequisite, SelectionOptionKey};

use crate::support::*;

fn issues(data: &GameData) -> Vec<(String, DataIssue)> {
    data.check()
        .into_iter()
        .map(|DataProblem { at, issue }| (at, issue))
        .collect()
}

fn class_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Class {
    data.classes
        .iter_mut()
        .chain(&mut data.prestige_classes)
        .find(|c| c.key.as_str() == key)
        .unwrap()
}

#[test]
fn the_test_world_has_no_problems() {
    assert_eq!(issues(&world()), []);
}

#[test]
fn level_rows_run_from_1_with_none_missing() {
    let mut data = world();
    class_mut(&mut data, "class:wizard").levels.remove(4);
    let fighter = class_mut(&mut data, "class:fighter");
    fighter.archetypes[0].levels[0].level = 0;
    assert_eq!(
        issues(&data),
        [
            (
                "archetype:fighter/armiger".into(),
                DataIssue::LevelRowsNotContiguous
            ),
            ("class:wizard".into(), DataIssue::LevelRowsNotContiguous),
        ]
    );
}

#[test]
fn only_base_classes_have_archetypes() {
    let mut data = world();
    let armiger = data.classes[1].archetypes[0].clone();
    class_mut(&mut data, "class:arcane-trickster")
        .archetypes
        .push(rules::Archetype {
            key: rules::ArchetypeKey::new("archetype:arcane-trickster/x"),
            ..armiger
        });
    assert_eq!(
        issues(&data),
        [(
            "class:arcane-trickster".into(),
            DataIssue::ArchetypeOnPrestigeClass
        )]
    );
}

#[test]
fn only_prestige_classes_advance_spellcasting() {
    let mut data = world();
    class_mut(&mut data, "class:wizard").levels[2].advances_divine = true;
    assert_eq!(
        issues(&data),
        [(
            "class:wizard".into(),
            DataIssue::AdvancesOnBaseClass { level: 3 }
        )]
    );
}

#[test]
fn a_prerequisite_group_has_children_and_at_least_fits_them() {
    let mut data = world();
    data.feats[0].prerequisite = Some(Prerequisite::Any(vec![]));
    data.feats[1].prerequisite = Some(Prerequisite::AtLeast(
        3,
        vec![
            Prerequisite::BaseAttackBonus(1),
            Prerequisite::CasterLevel(1),
        ],
    ));
    let first = data.feats[0].key.to_string();
    let second = data.feats[1].key.to_string();
    let mut found = issues(&data);
    found.sort_by(|a, b| a.0.cmp(&b.0));
    let mut expected = vec![
        (first, DataIssue::EmptyPrerequisiteGroup),
        (second, DataIssue::AtLeastOutOfRange { n: 3, children: 2 }),
    ];
    expected.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(found, expected);
}

#[test]
fn keys_are_unique() {
    let mut data = world();
    let copy = data.feats[0].clone();
    data.feats.push(copy.clone());
    assert_eq!(
        issues(&data),
        [(copy.key.to_string(), DataIssue::DuplicateKey)]
    );
}

#[test]
fn a_selection_options_key_starts_with_its_selection_key() {
    let mut data = world();
    let talents = &mut class_mut(&mut data, "class:rogue").selections[0];
    talents.options[0].key = SelectionOptionKey::new("selection:fighter/stray");
    assert_eq!(
        issues(&data),
        [(
            "selection:fighter/stray".into(),
            DataIssue::OptionOutsideItsSelection
        )]
    );
}

#[test]
fn ranks_and_levels_stay_in_range() {
    let mut data = world();
    data.progression.mythic_ascensions[9].rank = 11;
    data.progression.mythic_ascensions[0].character_level = 41;
    let legend = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "path:legend")
        .unwrap();
    let switch = legend.switch.as_mut().unwrap();
    switch.level_cap = Some(41);
    switch.kept_ranks = BTreeSet::from([1, 2, 8]);
    let trickster = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "path:trickster")
        .unwrap();
    trickster.ranks[0].rank = 0;
    class_mut(&mut data, "class:wizard").levels[19].arcane_spell_level = 10;

    let mut found = issues(&data);
    found.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        found,
        [
            ("class:wizard".into(), DataIssue::SpellLevelOutOfRange(10)),
            ("path:legend".into(), DataIssue::LevelOutOfRange(41)),
            ("path:legend".into(), DataIssue::RankOutOfRange(8)),
            ("path:trickster".into(), DataIssue::RankOutOfRange(0)),
            ("progression".into(), DataIssue::LevelOutOfRange(41)),
            ("progression".into(), DataIssue::RankOutOfRange(11)),
        ]
    );
}

#[test]
fn problems_read_as_sentences() {
    let problem = DataProblem {
        at: FeatKey::new("feat:cleave").to_string(),
        issue: DataIssue::DuplicateKey,
    };
    assert_eq!(problem.to_string(), "feat:cleave: the key is used twice");
}
