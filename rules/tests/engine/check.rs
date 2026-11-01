//! `GameData::check`: data mistakes that the types can't rule out, found before the data is seeded or used.

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, ArchetypeLevelRow, ClassKey, DataAt, DataIssue, DataProblem, FeatKey,
    FeatListKey, FeatSlot, FeatureKey, FeatureSelectionKey, GameData, MythicPathKey, Prerequisite,
    RaceKey, Save, SaveException, SelectionOptionKey, SpellLevels,
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

fn sorted(mut issues: Vec<(DataAt, DataIssue)>) -> Vec<(DataAt, DataIssue)> {
    issues.sort();
    issues
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

fn at_armiger() -> DataAt {
    DataAt::Archetype(ArchetypeKey::new("fighter/armiger"))
}

fn fighter_level(data: &GameData, level: u8) -> rules::ClassLevelRow {
    data.classes
        .iter()
        .find(|c| c.key.as_str() == "fighter")
        .unwrap()
        .levels
        .iter()
        .find(|row| row.level == level)
        .unwrap()
        .clone()
}

#[test]
fn level_rows_run_from_1_with_none_missing() {
    let mut data = world();
    class_mut(&mut data, "wizard").levels.remove(4);
    assert_eq!(
        issues(&data),
        [(at_class("wizard"), DataIssue::LevelRowsNotContiguous)]
    );
}

#[test]
fn an_archetype_row_is_at_one_of_its_class_levels_once() {
    let mut data = world();
    let armiger = &mut class_mut(&mut data, "fighter").archetypes[0];
    // Rows are kept only where the archetype differs, so gaps are fine; outside 1..=20 or twice isn't.
    armiger.levels[0].level = 0;
    let mut row_21 = armiger.levels[1].clone();
    row_21.level = 21;
    let twice = armiger.levels[1].clone();
    armiger.levels.extend([row_21, twice]);
    assert_eq!(
        issues(&data),
        sorted(vec![
            (at_armiger(), DataIssue::ArchetypeRowOutsideClass(0)),
            (at_armiger(), DataIssue::ArchetypeRowOutsideClass(21)),
            (at_armiger(), DataIssue::DuplicateArchetypeRow(2)),
        ])
    );
}

#[test]
fn an_archetype_row_differs_from_the_base_row() {
    let mut data = world();
    let fighter = class_mut(&mut data, "fighter");
    let base_3 = fighter.levels[2].clone();
    let base_5 = fighter.levels[4].clone();
    let armiger = &mut fighter.archetypes[0];
    // Level 3: the fighter's grants, written in another order, and no spell levels of its own.
    armiger.levels.push(ArchetypeLevelRow {
        level: 3,
        spell_levels: None,
        features: base_3.features.iter().rev().cloned().collect(),
        feats: base_3.feats.clone(),
        feat_slots: base_3.feat_slots.clone(),
        selections: base_3.selections.clone(),
    });
    // Level 5: the fighter's grants and the fighter's spell levels set again.
    armiger.levels.push(ArchetypeLevelRow {
        level: 5,
        spell_levels: Some(base_5.spell_levels()),
        features: base_5.features.clone(),
        feats: base_5.feats.clone(),
        feat_slots: base_5.feat_slots.clone(),
        selections: base_5.selections.clone(),
    });
    // Level 7: its own grants, but a spell override equal to the fighter's.
    armiger.levels.push(ArchetypeLevelRow {
        level: 7,
        spell_levels: Some(SpellLevels::default()),
        features: vec![],
        feats: BTreeSet::new(),
        feat_slots: vec![FeatSlot::CombatFeat],
        selections: vec![],
    });
    // Level 9: the fighter's grants plus a granted feat, which is a difference.
    let base_9 = fighter_level(&data, 9);
    class_mut(&mut data, "fighter").archetypes[0]
        .levels
        .push(ArchetypeLevelRow {
            level: 9,
            spell_levels: None,
            features: base_9.features.clone(),
            feats: BTreeSet::from([FeatKey::new("toughness")]),
            feat_slots: base_9.feat_slots.clone(),
            selections: base_9.selections.clone(),
        });
    assert_eq!(
        issues(&data),
        [
            (at_armiger(), DataIssue::ArchetypeRowSameAsBase(3)),
            (at_armiger(), DataIssue::ArchetypeRowSameAsBase(5)),
            (at_armiger(), DataIssue::SpellOverrideSameAsBase(7)),
        ]
    );
}

#[test]
fn a_spell_override_stays_in_range() {
    let mut data = world();
    let rogue = class_mut(&mut data, "rogue");
    rogue.archetypes[0].levels[19].spell_levels = Some(SpellLevels {
        arcane: 10,
        divine: 0,
    });
    assert_eq!(
        issues(&data),
        [(
            DataAt::Archetype(ArchetypeKey::new("rogue/eldritch-scoundrel")),
            DataIssue::SpellLevelOutOfRange(10)
        )]
    );
}

#[test]
fn a_save_exception_differs_from_the_formula_at_one_of_the_class_levels() {
    let mut data = world();
    let exception = |level, save, value| SaveException { level, save, value };
    // Wizard: poor Fortitude, good Will. The base formula gives Will +3 at 2 and Fortitude +1 at 3.
    class_mut(&mut data, "wizard").save_exceptions = vec![
        exception(2, Save::Will, 4),
        exception(2, Save::Will, 5),
        exception(3, Save::Fortitude, 1),
        exception(21, Save::Reflex, 9),
    ];
    // Arcane Trickster (prestige): good Reflex, ⌊(1+1)/2⌋ = +1 at 1.
    class_mut(&mut data, "arcane-trickster").save_exceptions = vec![exception(1, Save::Reflex, 1)];
    assert_eq!(
        issues(&data),
        sorted(vec![
            (
                at_class("arcane-trickster"),
                DataIssue::SaveExceptionSameAsFormula {
                    level: 1,
                    save: Save::Reflex
                }
            ),
            (
                at_class("wizard"),
                DataIssue::DuplicateSaveException {
                    level: 2,
                    save: Save::Will
                }
            ),
            (
                at_class("wizard"),
                DataIssue::SaveExceptionSameAsFormula {
                    level: 3,
                    save: Save::Fortitude
                }
            ),
            (
                at_class("wizard"),
                DataIssue::SaveExceptionOutsideClass {
                    level: 21,
                    save: Save::Reflex
                }
            ),
        ])
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
            levels: vec![],
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

/// Keys carry no type prefix, so a feature and a selection may share one (the real data's `shaman/spirit`): each
/// kind has its own table.
#[test]
fn different_kinds_may_share_a_key() {
    let mut data = world();
    let rogue = class_mut(&mut data, "rogue");
    let mut talents_feature = rogue.features[0].clone();
    talents_feature.key = FeatureKey::new("rogue/rogue-talents");
    rogue.features.push(talents_feature);
    assert_eq!(
        rogue.selections[0].key,
        FeatureSelectionKey::new("rogue/rogue-talents")
    );

    assert_eq!(issues(&data), []);
}

/// What a class, archetype or path owns has a key that starts with the owner's key and a `/`.
#[test]
fn owned_keys_start_with_their_owners_key() {
    let mut data = world();
    let fighter = class_mut(&mut data, "fighter");
    fighter.features[0].key = FeatureKey::new("power-attack");
    // An archetype's own: under the archetype's key, not just the class's.
    let mut stray = fighter.features[0].clone();
    stray.key = FeatureKey::new("fighter/stray");
    fighter.archetypes[0].features.push(stray);
    fighter.archetypes[0].selections[0].key = FeatureSelectionKey::new("fighter/order");
    for option in &mut fighter.archetypes[0].selections[0].options {
        option.key = SelectionOptionKey::new(option.key.as_str().replace("armiger/", ""));
    }
    let trickster = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "trickster")
        .unwrap();
    trickster.features[0].key = FeatureKey::new("tricksters/tricks");
    let monk = class_mut(&mut data, "monk");
    monk.feat_lists[0].key = FeatListKey::new("monk-bonus-feats");
    for row in &mut monk.levels {
        for slot in &mut row.feat_slots {
            *slot = FeatSlot::OneOf {
                list: FeatListKey::new("monk-bonus-feats"),
            };
        }
    }
    assert_eq!(
        issues(&data),
        [
            (
                DataAt::Feature(FeatureKey::new("fighter/stray")),
                DataIssue::KeyOutsideItsOwner
            ),
            (
                DataAt::Feature(FeatureKey::new("power-attack")),
                DataIssue::KeyOutsideItsOwner
            ),
            (
                DataAt::Feature(FeatureKey::new("tricksters/tricks")),
                DataIssue::KeyOutsideItsOwner
            ),
            (
                DataAt::FeatureSelection(FeatureSelectionKey::new("fighter/order")),
                DataIssue::KeyOutsideItsOwner
            ),
            (
                DataAt::FeatList(FeatListKey::new("monk-bonus-feats")),
                DataIssue::KeyOutsideItsOwner
            ),
        ]
    );
    assert_eq!(
        DataProblem {
            at: DataAt::Feature(FeatureKey::new("power-attack")),
            issue: DataIssue::KeyOutsideItsOwner,
        }
        .to_string(),
        "feature power-attack: the key doesn't start with its owner's key"
    );
}

#[test]
fn a_feat_list_holds_two_feats_or_more_and_some_slot_names_it() {
    let mut data = world();
    let monk = class_mut(&mut data, "monk");
    monk.feat_lists[0].feats = BTreeSet::from([FeatKey::new("dodge")]);
    let mut unused = monk.feat_lists[0].clone();
    unused.key = FeatListKey::new("monk/unused");
    unused.feats = BTreeSet::from([FeatKey::new("dodge"), FeatKey::new("toughness")]);
    monk.feat_lists.push(unused);
    let at = |key: &str| DataAt::FeatList(FeatListKey::new(key));
    assert_eq!(
        issues(&data),
        [
            (at("monk/bonus-feats"), DataIssue::FeatListTooShort),
            (at("monk/unused"), DataIssue::UnusedFeatList),
        ]
    );
    assert_eq!(
        DataProblem {
            at: at("monk/bonus-feats"),
            issue: DataIssue::FeatListTooShort,
        }
        .to_string(),
        "feat list monk/bonus-feats: a list of fewer than two feats; grant a lone feat instead"
    );
}

#[test]
fn feat_list_keys_are_unique_across_owners() {
    let mut data = world();
    let copy = class_mut(&mut data, "monk").feat_lists[0].clone();
    class_mut(&mut data, "wizard").feat_lists.push(copy.clone());
    assert_eq!(
        issues(&data),
        [
            (DataAt::FeatList(copy.key.clone()), DataIssue::DuplicateKey),
            (DataAt::FeatList(copy.key), DataIssue::KeyOutsideItsOwner),
        ]
    );
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

/// A selection shares another selection's options, never its own (the schema's CHECK on
/// `selection_shared_option`): an own option is offered already.
#[test]
fn a_selection_shares_only_other_selections_options() {
    let mut data = world();
    let talents = &mut class_mut(&mut data, "slayer").selections[0];
    talents.shared_options.insert(SelectionOptionKey::new(
        "slayer/slayer-talents/slayer-camouflage",
    ));
    // Any key under the selection's own counts, even one that isn't an option.
    talents
        .shared_options
        .insert(SelectionOptionKey::new("slayer/slayer-talents/ghost"));
    // Sharing only the text before the `/` is fine.
    talents
        .shared_options
        .insert(SelectionOptionKey::new("slayer/slayer-talents-x/other"));
    let at = DataAt::FeatureSelection(FeatureSelectionKey::new("slayer/slayer-talents"));
    assert_eq!(
        issues(&data),
        [
            (
                at.clone(),
                DataIssue::SharesOwnOption(SelectionOptionKey::new("slayer/slayer-talents/ghost"))
            ),
            (
                at,
                DataIssue::SharesOwnOption(SelectionOptionKey::new(
                    "slayer/slayer-talents/slayer-camouflage"
                ))
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
    // The monk's list ignores prerequisites.
    let bonus_list = rules::FeatSlot::OneOf {
        list: FeatListKey::new("monk/bonus-feats"),
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
