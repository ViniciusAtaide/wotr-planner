//! `GameData::check`: data mistakes that the types can't rule out, found before the data is seeded or used.

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, ClassKey, DataAt, DataIssue, DataProblem, FeatKey, FeatureKey,
    FeatureSelectionKey, GameData, MythicPathKey, Prerequisite, RaceKey, SelectionOptionKey,
};

use crate::support::*;

fn issues(data: &GameData) -> Vec<(DataAt, DataIssue)> {
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

fn at_class(key: &str) -> DataAt {
    DataAt::Class(ClassKey::new(key))
}

fn at_path(key: &str) -> DataAt {
    DataAt::MythicPath(MythicPathKey::new(key))
}

#[test]
fn the_test_world_has_no_problems() {
    assert_eq!(issues(&world()), []);
}

#[test]
fn level_rows_run_from_1_with_none_missing() {
    let mut data = world();
    class_mut(&mut data, "wizard").levels.remove(4);
    let fighter = class_mut(&mut data, "fighter");
    fighter.archetypes[0].levels[0].level = 0;
    assert_eq!(
        issues(&data),
        [
            (at_class("wizard"), DataIssue::LevelRowsNotContiguous),
            (
                DataAt::Archetype(ArchetypeKey::new("fighter/armiger")),
                DataIssue::LevelRowsNotContiguous
            ),
        ]
    );
}

#[test]
fn only_base_classes_have_archetypes() {
    let mut data = world();
    let armiger = data.classes[1].archetypes[0].clone();
    class_mut(&mut data, "arcane-trickster")
        .archetypes
        .push(rules::Archetype {
            key: ArchetypeKey::new("arcane-trickster/x"),
            ..armiger
        });
    assert_eq!(
        issues(&data),
        [(
            at_class("arcane-trickster"),
            DataIssue::ArchetypeOnPrestigeClass
        )]
    );
}

#[test]
fn only_prestige_classes_advance_spellcasting() {
    let mut data = world();
    class_mut(&mut data, "wizard").levels[2].advances_divine = true;
    assert_eq!(
        issues(&data),
        [(
            at_class("wizard"),
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
    let first = DataAt::Feat(data.feats[0].key.clone());
    let second = DataAt::Feat(data.feats[1].key.clone());
    let mut found = issues(&data);
    found.sort();
    let mut expected = vec![
        (first, DataIssue::EmptyPrerequisiteGroup),
        (second, DataIssue::AtLeastOutOfRange { n: 3, children: 2 }),
    ];
    expected.sort();
    assert_eq!(found, expected);
}

#[test]
fn keys_are_unique_within_each_kind() {
    let mut data = world();
    let copy = data.feats[0].clone();
    data.feats.push(copy.clone());
    assert_eq!(
        issues(&data),
        [(DataAt::Feat(copy.key), DataIssue::DuplicateKey)]
    );
}

/// Keys carry no type prefix, so a feat, a feature and a selection may share one: each kind has its own table.
#[test]
fn different_kinds_may_share_a_key() {
    let mut data = world();
    let rogue = class_mut(&mut data, "rogue");
    let mut talents_feature = rogue.features[0].clone();
    talents_feature.key = FeatureKey::new("rogue/rogue-talents");
    let mut power_attack_feature = rogue.features[0].clone();
    power_attack_feature.key = FeatureKey::new("power-attack");
    rogue
        .features
        .extend([talents_feature, power_attack_feature]);
    assert_eq!(
        rogue.selections[0].key,
        FeatureSelectionKey::new("rogue/rogue-talents")
    );
    assert!(
        data.feats
            .iter()
            .any(|f| f.key == FeatKey::new("power-attack"))
    );

    assert_eq!(issues(&data), []);
}

/// Base and prestige classes are one kind (one `class` table), so they can't share a key.
#[test]
fn a_prestige_class_cant_reuse_a_base_class_key() {
    let mut data = world();
    class_mut(&mut data, "mystic-theurge").key = ClassKey::new("wizard");
    assert_eq!(
        issues(&data),
        [(at_class("wizard"), DataIssue::DuplicateKey)]
    );
}

#[test]
fn a_selection_options_key_starts_with_its_selection_key() {
    let mut data = world();
    let talents = &mut class_mut(&mut data, "rogue").selections[0];
    talents.options[0].key = SelectionOptionKey::new("fighter/stray");
    // Sharing only the text before the `/` isn't enough.
    talents.options[1].key = SelectionOptionKey::new("rogue/rogue-talents-x");
    assert_eq!(
        issues(&data),
        [
            (
                DataAt::SelectionOption(SelectionOptionKey::new("fighter/stray")),
                DataIssue::OptionOutsideItsSelection
            ),
            (
                DataAt::SelectionOption(SelectionOptionKey::new("rogue/rogue-talents-x")),
                DataIssue::OptionOutsideItsSelection
            ),
        ]
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
        .find(|p| p.key.as_str() == "legend")
        .unwrap();
    let switch = legend.switch.as_mut().unwrap();
    switch.level_cap = Some(41);
    switch.kept_ranks = BTreeSet::from([1, 2, 8]);
    let trickster = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "trickster")
        .unwrap();
    trickster.ranks[0].rank = 0;
    class_mut(&mut data, "wizard").levels[19].arcane_spell_level = 10;

    assert_eq!(
        issues(&data),
        [
            (DataAt::Progression, DataIssue::LevelOutOfRange(41)),
            (DataAt::Progression, DataIssue::RankOutOfRange(11)),
            (at_class("wizard"), DataIssue::SpellLevelOutOfRange(10)),
            (at_path("legend"), DataIssue::LevelOutOfRange(41)),
            (at_path("legend"), DataIssue::RankOutOfRange(8)),
            (at_path("trickster"), DataIssue::RankOutOfRange(0)),
        ]
    );
}

#[test]
fn problems_read_as_sentences_that_name_the_kind() {
    let problem = |at| DataProblem {
        at,
        issue: DataIssue::DuplicateKey,
    };
    assert_eq!(
        problem(DataAt::Feat(FeatKey::new("cleave"))).to_string(),
        "feat cleave: the key is used twice"
    );
    assert_eq!(
        problem(DataAt::Feature(FeatureKey::new("cleave"))).to_string(),
        "feature cleave: the key is used twice"
    );
    assert_eq!(
        DataProblem {
            at: DataAt::Progression,
            issue: DataIssue::RankOutOfRange(11),
        }
        .to_string(),
        "progression: Mythic rank 11 is out of range"
    );
}

#[test]
fn only_class_levels_grant_bonus_list_slots() {
    let bonus_list = rules::FeatSlot::OneOf {
        feats: BTreeSet::from([FeatKey::new("dodge")]),
        ignores_prerequisites: true,
    };
    let mut data = world();
    // A class's and an archetype's level rows may grant one.
    class_mut(&mut data, "fighter").archetypes[0].levels[4]
        .feat_slots
        .push(bonus_list.clone());
    assert_eq!(issues(&data), []);

    // A race and a mythic rank may not: the list is a class's own.
    let human = data
        .races
        .iter_mut()
        .find(|r| r.key.as_str() == "human")
        .unwrap();
    human.feat_slots.push(bonus_list.clone());
    let trickster = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "trickster")
        .unwrap();
    trickster.ranks[1].feat_slots.push(bonus_list);
    let human = DataAt::Race(RaceKey::new("human"));
    assert_eq!(
        issues(&data),
        [
            (human.clone(), DataIssue::BonusListOutsideAClass),
            (at_path("trickster"), DataIssue::BonusListOutsideAClass),
        ]
    );
    assert_eq!(
        DataProblem {
            at: human,
            issue: DataIssue::BonusListOutsideAClass,
        }
        .to_string(),
        "race human: a feat slot that ignores prerequisites is granted outside a class's level rows"
    );
}
