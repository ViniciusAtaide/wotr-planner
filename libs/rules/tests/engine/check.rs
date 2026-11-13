//! `GameData::check`: data mistakes that the types can't rule out, found before the data is seeded or used.

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, ArchetypeLevelRow, ClassKey, DataAt, DataIssue, DataProblem, FeatKey,
    FeatListKey, FeatSlot, FeatureKey, FeatureSelectionKey, GameData, MythicPathKey, OptionGroup,
    Prerequisite, RaceKey, Repeat, Save, SaveException, SaveProgression, SelectionOptionKey,
    SpellLevels,
};

use crate::support::*;

fn issues(data: &GameData) -> Vec<(DataAt, DataIssue)> {
    data.check()
        .into_iter()
        .map(|DataProblem { at, issue }| (at, issue))
        .collect()
}

fn class_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Class {
    data.all_classes_mut()
        .find(|c| c.key.as_str() == key)
        .unwrap()
}

/// [`issues`] but the unknown keys: for a test that renames a key, which leaves its old key named elsewhere.
fn issues_but_unknown_keys(data: &GameData) -> Vec<(DataAt, DataIssue)> {
    issues(data)
        .into_iter()
        .filter(|(_, issue)| !matches!(issue, DataIssue::UnknownKey(_)))
        .collect()
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
        feats_for: Default::default(),
        level: 3,
        spell_levels: None,
        features: base_3.features.iter().rev().cloned().collect(),
        feats: base_3.feats.clone(),
        feat_slots: base_3.feat_slots.clone(),
        selections: base_3.selections.clone(),
    });
    // Level 5: the fighter's grants and the fighter's spell levels set again.
    armiger.levels.push(ArchetypeLevelRow {
        feats_for: Default::default(),
        level: 5,
        spell_levels: Some(base_5.spell_levels()),
        features: base_5.features.clone(),
        feats: base_5.feats.clone(),
        feat_slots: base_5.feat_slots.clone(),
        selections: base_5.selections.clone(),
    });
    // Level 7: its own grants, but a spell override equal to the fighter's.
    armiger.levels.push(ArchetypeLevelRow {
        feats_for: Default::default(),
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
            feats_for: Default::default(),
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
fn an_archetype_save_progression_differs_from_its_classs() {
    let mut data = world();
    // Fighter: good Fortitude, poor Reflex and Will.
    let armiger = &mut class_mut(&mut data, "fighter").archetypes[0];
    armiger.fort = Some(SaveProgression::Good);
    armiger.reflex = Some(SaveProgression::Good);
    armiger.will = Some(SaveProgression::Poor);
    assert_eq!(
        issues(&data),
        [
            (
                at_armiger(),
                DataIssue::SaveProgressionSameAsClass(Save::Fortitude)
            ),
            (
                at_armiger(),
                DataIssue::SaveProgressionSameAsClass(Save::Will)
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
        issues_but_unknown_keys(&data),
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
        issues_but_unknown_keys(&data),
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
        issues_but_unknown_keys(&data),
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
    data.progression
        .mythic_ascensions
        .push(rules::MythicAscension {
            rank: 11,
            character_level: 20,
        });
    data.progression.mythic_ascensions[0].character_level = 41;
    let legend = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "legend")
        .unwrap();
    let switch = legend.switch.as_mut().unwrap();
    switch.level_cap = Some(41);
    switch.kept_through_rank = 8;
    switch.path_kept_through_rank = 0;
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
            (at_path("legend"), DataIssue::RankOutOfRange(0)),
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
fn only_class_levels_and_a_paths_own_ranks_grant_bonus_list_slots() {
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
    trickster
        .ranks
        .iter_mut()
        .find(|row| row.rank == 4)
        .unwrap()
        .feat_slots
        .push(bonus_list);
    let human = DataAt::Race(RaceKey::new("human"));
    assert_eq!(
        issues(&data),
        [
            (human.clone(), DataIssue::MisplacedBonusList),
            (at_path("trickster"), DataIssue::MisplacedBonusList),
        ]
    );
    assert_eq!(
        DataProblem {
            at: human,
            issue: DataIssue::MisplacedBonusList,
        }
        .to_string(),
        "race human: a feat slot that ignores prerequisites is granted outside a class's level rows from a list \
         that isn't the grantor's own"
    );
}

#[test]
fn a_path_rank_may_name_its_own_bonus_list_only() {
    // The world's Gold Dragon names its own Dragon Feats at MR 9; another path's rank may not.
    let mut data = world();
    assert_eq!(issues(&data), []);
    let trickster = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "trickster")
        .unwrap();
    trickster.ranks[2].feat_slots.push(rules::FeatSlot::OneOf {
        list: FeatListKey::new("gold-dragon/dragon-feats"),
    });
    assert_eq!(
        issues(&data),
        [(at_path("trickster"), DataIssue::MisplacedBonusList)]
    );
}

#[test]
fn a_list_of_every_feat_names_none_of_its_own() {
    let mut data = world();
    let dragon = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "gold-dragon")
        .unwrap();
    dragon.feat_lists[0]
        .feats
        .insert(rules::FeatKey::new("dodge"));
    let at = DataAt::FeatList(FeatListKey::new("gold-dragon/dragon-feats"));
    assert_eq!(
        issues(&data),
        [(at.clone(), DataIssue::EveryFeatListWithFeats)]
    );
    assert_eq!(
        DataProblem {
            at,
            issue: DataIssue::EveryFeatListWithFeats
        }
        .to_string(),
        "feat list gold-dragon/dragon-feats: a list of every feat that names feats of its own"
    );
}

#[test]
fn a_paths_mr_1_and_2_rows_grant_features_only() {
    let mut data = world();
    let trickster = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "trickster")
        .unwrap();
    trickster.ranks[0]
        .selections
        .push(rules::FeatureSelectionKey::new("trickster/tricks"));
    trickster.ranks[1].feat_slots.push(rules::FeatSlot::General);
    assert_eq!(
        issues(&data),
        [
            (at_path("trickster"), DataIssue::SlotBeforePathChoice(1)),
            (at_path("trickster"), DataIssue::SlotBeforePathChoice(2)),
        ]
    );
    assert_eq!(
        DataProblem {
            at: at_path("trickster"),
            issue: DataIssue::SlotBeforePathChoice(1)
        }
        .to_string(),
        "mythic path trickster: the MR 1 row grants a slot, but the rows before MR 3 are the first ascension's features"
    );
}

#[test]
fn a_switch_is_open_from_paths_chosen_at_mr_3_only() {
    let mut data = world();
    let devil = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == "devil")
        .unwrap();
    devil
        .switch
        .as_mut()
        .unwrap()
        .from
        .insert(rules::MythicPathKey::new("legend"));
    assert_eq!(
        issues(&data),
        [(
            at_path("devil"),
            DataIssue::SwitchFromASwitchPath(rules::MythicPathKey::new("legend"))
        )]
    );
}

fn feat_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Feat {
    data.feats
        .iter_mut()
        .find(|f| f.key.as_str() == key)
        .unwrap()
}

fn rogue_talent_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::SelectionOption {
    class_mut(data, "rogue").selections[0]
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == key)
        .unwrap()
}

fn at_feat(key: &str) -> DataAt {
    DataAt::Feat(FeatKey::new(key))
}

fn at_option(key: &str) -> DataAt {
    DataAt::SelectionOption(SelectionOptionKey::new(key))
}

#[test]
fn repeats_and_chosen_feats_fit_their_feats() {
    let mut data = world();
    // Once per value, but the feat takes none.
    feat_mut(&mut data, "toughness").repeat = Repeat::PerValue;
    // "Up to once" is just once.
    feat_mut(&mut data, "dodge").repeat = Repeat::Times(1);
    // A chosen feat is one that takes a value.
    rogue_talent_mut(&mut data, "rogue/rogue-talents/weapon-focus").chosen_feat =
        Some(FeatKey::new("toughness"));
    assert_eq!(
        sorted(issues(&data)),
        sorted(vec![
            (at_feat("toughness"), DataIssue::PerValueWithoutParameter),
            (at_feat("dodge"), DataIssue::TimesOutOfRange(1)),
            (
                at_option("rogue/rogue-talents/weapon-focus"),
                DataIssue::ChosenFeatNotParameterised
            ),
        ])
    );
}

#[test]
fn an_option_repeats_once_per_value_only_when_it_takes_a_value() {
    let mut data = world();
    // The rogue talent Weapon Focus once per weapon, as its chosen feat: the Skill Focus talents' kind (ticket 47).
    rogue_talent_mut(&mut data, "rogue/rogue-talents/weapon-focus").repeat = Repeat::PerValue;
    // An option taking a value of its own (Battle Master's weapon), once per weapon: Finesse Training's kind (ticket
    // 172).
    slayer_talent_mut(&mut data, "slayer/slayer-talents/battle-master").repeat = Repeat::PerValue;
    assert_eq!(issues(&data), []);
    // An option that takes no value has none to count.
    rogue_talent_mut(&mut data, "rogue/rogue-talents/fast-stealth").repeat = Repeat::PerValue;
    assert_eq!(
        issues(&data),
        [(
            at_option("rogue/rogue-talents/fast-stealth"),
            DataIssue::PerValueOptionWithoutValue
        )]
    );
}

#[test]
fn a_value_in_a_prerequisite_fits_its_feat() {
    let weapon_focus = FeatKey::new("weapon-focus");
    let mut data = world();
    // A value for a feat that takes none, one of another kind, and a weapon that isn't exotic.
    feat_mut(&mut data, "dodge").prerequisite = Some(Prerequisite::FeatFor {
        feat: FeatKey::new("toughness"),
        value: weapon("longsword"),
    });
    feat_mut(&mut data, "toughness").prerequisite = Some(Prerequisite::FeatFor {
        feat: FeatKey::new("skill-focus"),
        value: weapon("longsword"),
    });
    feat_mut(&mut data, "power-attack").prerequisite = Some(Prerequisite::FeatFor {
        feat: FeatKey::new("exotic-weapon-proficiency"),
        value: weapon("longsword"),
    });
    // "The same value" where the pick takes none: an unparameterised feat, a class, an option without a chosen
    // feat.
    feat_mut(&mut data, "cleave").prerequisite =
        Some(Prerequisite::FeatForSameValue(weapon_focus.clone()));
    class_mut(&mut data, "fighter").prerequisite =
        Some(Prerequisite::FeatForSameValue(weapon_focus.clone()));
    rogue_talent_mut(&mut data, "rogue/rogue-talents/fast-stealth").prerequisite =
        Some(Prerequisite::FeatForSameValue(weapon_focus.clone()));
    // ... or one of another kind: a skill isn't a weapon.
    feat_mut(&mut data, "skill-focus").prerequisite =
        Some(Prerequisite::FeatForSameValue(weapon_focus.clone()));
    // Fine: the rogue talent Weapon Focus's pick names a weapon, and a weapon is a weapon whatever the feat.
    rogue_talent_mut(&mut data, "rogue/rogue-talents/weapon-focus").prerequisite = Some(
        Prerequisite::FeatForSameValue(FeatKey::new("exotic-weapon-proficiency")),
    );
    assert_eq!(
        sorted(issues(&data)),
        sorted(vec![
            (
                at_feat("dodge"),
                DataIssue::ValueOfWrongKind(FeatKey::new("toughness"))
            ),
            (
                at_feat("toughness"),
                DataIssue::ValueOfWrongKind(FeatKey::new("skill-focus"))
            ),
            (
                at_feat("power-attack"),
                DataIssue::ValueOfWrongKind(FeatKey::new("exotic-weapon-proficiency"))
            ),
            (at_feat("cleave"), DataIssue::SameValueWithoutValue),
            (at_class("fighter"), DataIssue::SameValueWithoutValue),
            (
                at_option("rogue/rogue-talents/fast-stealth"),
                DataIssue::SameValueWithoutValue
            ),
            (
                at_feat("skill-focus"),
                DataIssue::ValueOfWrongKind(weapon_focus.clone())
            ),
        ])
    );
    assert_eq!(
        DataProblem {
            at: at_feat("cleave"),
            issue: DataIssue::SameValueWithoutValue,
        }
        .to_string(),
        "feat cleave: \"the same value\" in a prerequisite whose pick takes no value"
    );
}

#[test]
fn a_one_feat_list_is_a_bonus_feat_of_the_players_choosing() {
    // The rogue's Skill Focus list holds one feat: fine, the pick names the skill. One unparameterised feat isn't.
    let mut data = world();
    assert_eq!(issues(&data), []);
    class_mut(&mut data, "rogue").feat_lists[0].feats = BTreeSet::from([FeatKey::new("dodge")]);
    assert_eq!(
        issues(&data),
        [(
            DataAt::FeatList(FeatListKey::new("rogue/skill-focus")),
            DataIssue::FeatListTooShort
        )]
    );
}

// ---- class rules ----------------------------------------------------------------------------------------------

#[test]
fn advancing_either_class_needs_rows_of_both_spell_types() {
    let mut data = world();
    // The Eldritch Knight advances arcane spellcasting only.
    class_mut(&mut data, "eldritch-knight").advances = rules::Advancement::Either;
    assert_eq!(
        issues(&data),
        [(
            at_class("eldritch-knight"),
            DataIssue::EitherWithOneSpellType
        )]
    );
}

#[test]
fn a_class_does_not_count_as_itself() {
    let mut data = world();
    class_mut(&mut data, "eldritch-knight")
        .counts_as
        .insert(rules::CountsAs {
            class: ClassKey::new("eldritch-knight"),
            toward: rules::CountsToward::FeatPrerequisites,
        });
    assert_eq!(
        issues(&data),
        [(at_class("eldritch-knight"), DataIssue::CountsAsItself)]
    );
}

#[test]
fn counting_toward_a_feature_needs_the_other_class_to_grant_it() {
    let mut data = world();
    // The fighter's rows grant Bravery, never Major Hexes.
    class_mut(&mut data, "winter-witch")
        .counts_as
        .insert(rules::CountsAs {
            class: ClassKey::new("fighter"),
            toward: rules::CountsToward::Feature(FeatureKey::new("witch/major-hexes")),
        });
    assert_eq!(
        issues(&data),
        [(
            at_class("winter-witch"),
            DataIssue::CountsTowardUngrantedFeature(FeatureKey::new("witch/major-hexes"))
        )]
    );
}

#[test]
fn a_story_condition_is_required_not_one_of_several() {
    let mut data = world();
    let story = Prerequisite::Story(rules::Story::Uncorrupted);
    data.feats[0].prerequisite = Some(Prerequisite::All(vec![
        Prerequisite::BaseAttackBonus(1),
        story.clone(),
    ]));
    data.feats[1].prerequisite = Some(Prerequisite::Any(vec![
        Prerequisite::BaseAttackBonus(1),
        story,
    ]));
    assert_eq!(
        issues(&data),
        [(
            DataAt::Feat(data.feats[1].key.clone()),
            DataIssue::StoryNotRequired
        )]
    );
}

// ---- options with rows by option level ------------------------------------------------------------------------

fn sorcerer_selection<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::FeatureSelection {
    let sorcerer = class_mut(data, "sorcerer");
    sorcerer
        .selections
        .iter_mut()
        .chain(
            sorcerer
                .archetypes
                .iter_mut()
                .flat_map(|archetype| &mut archetype.selections),
        )
        .find(|selection| selection.key.as_str() == key)
        .unwrap()
}

#[test]
fn an_option_has_one_row_per_option_level() {
    let mut data = world();
    let bloodlines = sorcerer_selection(&mut data, "sorcerer/bloodline");
    let abyssal = &mut bloodlines.options[0];
    let first = abyssal.levels[0].clone();
    abyssal.levels.push(first);
    assert_eq!(
        issues(&data),
        [(
            DataAt::SelectionOption(SelectionOptionKey::new("sorcerer/bloodline/abyssal")),
            DataIssue::DuplicateOptionRow(1)
        )]
    );
}

#[test]
fn a_selection_offering_options_with_rows_names_the_classes_whose_levels_count() {
    let mut data = world();
    sorcerer_selection(&mut data, "sorcerer/seeker/bloodline")
        .option_levels
        .clear();
    assert_eq!(
        issues(&data),
        [(
            DataAt::FeatureSelection(FeatureSelectionKey::new("sorcerer/seeker/bloodline")),
            DataIssue::OptionRowsWithoutLevels(SelectionOptionKey::new(
                "sorcerer/bloodline/abyssal"
            ))
        )]
    );
}

#[test]
fn a_pool_selection_may_offer_options_of_its_own() {
    // The Dual-Cursed Oracle's revelations: the mystery's pool and two of its own.
    let mut data = world();
    let cleric = class_mut(&mut data, "cleric");
    let revelation = cleric
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == "cleric/revelation")
        .unwrap();
    revelation.shared_options.insert(SelectionOptionKey::new(
        "cleric/fire-revelations/fire-breath",
    ));
    assert_eq!(issues(&data), []);
}

#[test]
fn an_options_arcana_is_one_of_its_rows_features() {
    let mut data = world();
    let bloodline = class_mut(&mut data, "sorcerer")
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == "sorcerer/bloodline")
        .unwrap();
    let arcane = bloodline
        .options
        .iter_mut()
        .find(|option| option.key.as_str() == "sorcerer/bloodline/arcane")
        .unwrap();
    arcane.arcana = Some(FeatureKey::new("sorcerer/bloodline/abyssal/arcana"));
    assert_eq!(
        issues(&data),
        [(
            DataAt::SelectionOption(SelectionOptionKey::new("sorcerer/bloodline/arcane")),
            DataIssue::ArcanaNotInRows(FeatureKey::new("sorcerer/bloodline/abyssal/arcana"))
        )]
    );
}

#[test]
fn a_selection_has_one_source_of_option_levels() {
    // Impossible Domain's levels come from the other domains held, not from classes as well.
    let mut data = world();
    let selection = cleric_selection(&mut data, "cleric/impossible-domain");
    selection.option_levels.insert(ClassKey::new("cleric"));
    assert_eq!(
        issues(&data),
        [(
            DataAt::FeatureSelection(FeatureSelectionKey::new("cleric/impossible-domain")),
            DataIssue::LevelFromWithClasses
        )]
    );
}

#[test]
fn a_leaf_about_the_pick_is_only_in_an_options_prerequisite() {
    // A feat isn't picked through a selection and belongs to no group.
    let mut data = world();
    let toughness = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "toughness")
        .unwrap();
    toughness.prerequisite = Some(Prerequisite::Any(vec![
        Prerequisite::NoOtherOptionOfGroup(OptionGroup::Draconic),
        Prerequisite::PickedThrough(FeatureSelectionKey::new("sorcerer/bloodline")),
    ]));
    assert_eq!(
        issues(&data),
        [
            (
                DataAt::Feat(FeatKey::new("toughness")),
                DataIssue::PickLeafOutsideOption
            ),
            (
                DataAt::Feat(FeatKey::new("toughness")),
                DataIssue::PickLeafOutsideOption
            )
        ]
    );
}

#[test]
fn an_option_rows_feat_list_counts_as_named() {
    // The bloodline lists are named only by option rows, and no problem says they're unused.
    let data = world();
    assert!(
        data.feat_lists()
            .any(|list| list.key.as_str() == "sorcerer/bloodline/abyssal-feats")
    );
    assert_eq!(issues(&data), []);
}

fn cleric_selection<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::FeatureSelection {
    class_mut(data, "cleric")
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == key)
        .unwrap()
}

#[test]
fn an_option_rows_chosen_feat_takes_the_options_chosen_value() {
    let at = DataAt::SelectionOption(SelectionOptionKey::new(
        "cleric/fire-revelations/weapon-mastery",
    ));
    // Another kind of value than Weapon Focus's (a weapon): no value of the pick fits it.
    let mut data = world();
    let mastery = &mut cleric_selection(&mut data, "cleric/fire-revelations").options[3];
    mastery.levels[0]
        .chosen_feats
        .insert(FeatKey::new("skill-focus"));
    assert_eq!(
        issues(&data),
        [(
            at.clone(),
            DataIssue::RowChosenFeatOfOtherKind(FeatKey::new("skill-focus"))
        )]
    );
    // An option whose pick names no value.
    let mut data = world();
    let mastery = &mut cleric_selection(&mut data, "cleric/fire-revelations").options[3];
    mastery.chosen_feat = None;
    assert_eq!(
        issues(&data),
        [(
            at,
            DataIssue::RowChosenFeatOfOtherKind(FeatKey::new("weapon-specialization"))
        )]
    );
}

#[test]
fn an_option_rows_bonus_slot_names_its_list() {
    let mut data = world();
    // An unused list is a problem; naming it from an option row's slot uses it.
    class_mut(&mut data, "cleric")
        .feat_lists
        .push(rules::FeatList {
            key: FeatListKey::new("cleric/war-feats"),
            name: "War feats".into(),
            guide: rules::GuideRef::new("cleric"),
            ignores_prerequisites: false,
            every_feat: false,
            feats: BTreeSet::from([FeatKey::new("toughness"), FeatKey::new("dodge")]),
        });
    assert_eq!(
        issues(&data),
        [(
            DataAt::FeatList(FeatListKey::new("cleric/war-feats")),
            DataIssue::UnusedFeatList
        )]
    );
    let war = &mut cleric_selection(&mut data, "cleric/domain").options[1];
    war.levels[1].feat_slots = vec![FeatSlot::OneOf {
        list: FeatListKey::new("cleric/war-feats"),
    }];
    assert_eq!(issues(&data), []);
}

// ---- ticket 32: values asked about, granted and offered ---------------------------------------------------------

fn mythic_feat_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::MythicFeat {
    data.mythic_feats
        .iter_mut()
        .find(|f| f.key.as_str() == key)
        .unwrap()
}

fn slayer_talent_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::SelectionOption {
    class_mut(data, "slayer").selections[0]
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == key)
        .unwrap()
}

#[test]
fn a_prerequisite_asking_about_the_picks_value_is_on_a_pick_that_takes_one_of_its_kind() {
    let if_light = || Prerequisite::FeatIfValue {
        value: rules::Parameter::ArmourType(rules::ArmourType::Light),
        feat: FeatKey::new("armor-proficiency-light"),
    };
    let mut data = world();
    // A feat that takes no value, and one that takes a weapon.
    feat_mut(&mut data, "dodge").prerequisite = Some(if_light());
    feat_mut(&mut data, "weapon-focus").prerequisite = Some(if_light());
    // Fine: the parameterised mythic feat takes a school, and asks about one.
    mythic_feat_mut(&mut data, "spell-focus-mythic").prerequisite =
        Some(Prerequisite::FeatIfValue {
            value: rules::Parameter::SpellSchool(rules::SpellSchool::Evocation),
            feat: FeatKey::new("toughness"),
        });
    assert_eq!(
        sorted(issues(&data)),
        sorted(vec![
            (at_feat("dodge"), DataIssue::SameValueWithoutValue),
            (at_feat("weapon-focus"), DataIssue::IfValueOfWrongKind),
        ])
    );
    assert_eq!(
        DataProblem {
            at: at_feat("weapon-focus"),
            issue: DataIssue::IfValueOfWrongKind,
        }
        .to_string(),
        "feat weapon-focus: a prerequisite asks about a value of a kind the pick doesn't take"
    );
}

#[test]
fn a_mythic_feat_repeats_per_value_only_with_a_value() {
    let mut data = world();
    mythic_feat_mut(&mut data, "toughness-mythic").repeat = Repeat::PerValue;
    mythic_feat_mut(&mut data, "dodge-mythic").prerequisite =
        Some(Prerequisite::FeatForSameValue(FeatKey::new("weapon-focus")));
    let at = |key: &str| DataAt::MythicFeat(rules::MythicFeatKey::new(key));
    assert_eq!(
        sorted(issues(&data)),
        sorted(vec![
            (at("toughness-mythic"), DataIssue::PerValueWithoutParameter),
            (at("dodge-mythic"), DataIssue::SameValueWithoutValue),
        ])
    );
}

#[test]
fn a_feat_granted_for_a_value_takes_a_value_of_that_kind() {
    let mut data = world();
    let mut row = fighter_level(&data, 3);
    row.feats_for.insert(rules::FeatValue {
        feat: FeatKey::new("skill-focus"),
        value: weapon("longsword"),
    });
    row.feats_for.insert(rules::FeatValue {
        feat: FeatKey::new("toughness"),
        value: weapon("longsword"),
    });
    class_mut(&mut data, "fighter").levels[2] = row;
    assert_eq!(
        sorted(issues(&data)),
        sorted(vec![
            (
                at_class("fighter"),
                DataIssue::ValueOfWrongKind(FeatKey::new("skill-focus"))
            ),
            (
                at_class("fighter"),
                DataIssue::ValueOfWrongKind(FeatKey::new("toughness"))
            ),
        ])
    );
}

#[test]
fn an_option_takes_one_kind_of_value_and_lists_only_values_of_it() {
    let mut data = world();
    // A value of its own beside a chosen feat's.
    rogue_talent_mut(&mut data, "rogue/rogue-talents/weapon-focus").parameter =
        Some(rules::ParameterKind::Weapon);
    // Values for a pick that takes none, and values of another kind.
    rogue_talent_mut(&mut data, "rogue/rogue-talents/fast-stealth").values =
        BTreeSet::from([weapon("dagger")]);
    slayer_talent_mut(&mut data, "slayer/slayer-talents/battle-master").values =
        BTreeSet::from([skill_value("mobility")]);
    assert_eq!(
        sorted(issues(&data)),
        sorted(vec![
            (
                at_option("rogue/rogue-talents/weapon-focus"),
                DataIssue::ParameterBesideChosenFeat
            ),
            (
                at_option("rogue/rogue-talents/fast-stealth"),
                DataIssue::OptionValueOfWrongKind
            ),
            (
                at_option("slayer/slayer-talents/battle-master"),
                DataIssue::OptionValueOfWrongKind
            ),
        ])
    );
}

#[test]
fn a_race_may_name_its_own_bonus_list_only() {
    // The world's half-elf names its own Adaptability list; another race may not.
    let mut data = world();
    assert_eq!(issues(&data), []);
    let human = data
        .races
        .iter_mut()
        .find(|r| r.key.as_str() == "human")
        .unwrap();
    human.feat_slots.push(FeatSlot::OneOf {
        list: FeatListKey::new("half-elf/adaptability"),
    });
    assert_eq!(
        issues(&data),
        [(
            DataAt::Race(RaceKey::new("human")),
            DataIssue::MisplacedBonusList
        )]
    );
}

#[test]
fn a_subrace_may_name_its_races_bonus_list_only() {
    // The world's Adaptable half-elf names its race's Adaptability list; the elf's Blightborn may not.
    let mut data = world();
    let elf = data
        .races
        .iter_mut()
        .find(|r| r.key.as_str() == "elf")
        .unwrap();
    elf.subraces[0].feat_slots.push(FeatSlot::OneOf {
        list: FeatListKey::new("half-elf/adaptability"),
    });
    assert_eq!(
        issues(&data),
        [(
            DataAt::Subrace(rules::SubraceKey::new("elf/blightborn")),
            DataIssue::MisplacedBonusList
        )]
    );
}

#[test]
fn an_option_levelled_alongside_the_others_needs_other_options() {
    // Impossible Domain's False God way counts beside the other domains held; a selection levelled any other way
    // has no other options for it to count beside.
    let mut data = world();
    let selection = cleric_selection(&mut data, "cleric/impossible-domain");
    selection.level_from = None;
    selection.option_levels.insert(ClassKey::new("cleric"));
    assert_eq!(
        issues(&data),
        [(
            DataAt::FeatureSelection(FeatureSelectionKey::new("cleric/impossible-domain")),
            DataIssue::LevelAlsoFromWithoutOtherOptions
        )]
    );
}

#[test]
fn a_replacement_belongs_to_a_skipped_level() {
    let mut data = world();
    let selection = cleric_selection(&mut data, "cleric/impossible-domain");
    selection.replaced_by.insert(
        7,
        rules::FeatureKey::new("sorcerer/bloodline/arcane/apotheosis"),
    );
    assert_eq!(
        issues(&data),
        [(
            DataAt::FeatureSelection(FeatureSelectionKey::new("cleric/impossible-domain")),
            DataIssue::ReplacementAtUnskippedLevel(7)
        )]
    );
}

#[test]
fn a_replacement_is_on_no_row_of_its_class() {
    let mut data = world();
    let class = data
        .all_classes_mut()
        .find(|class| {
            !class.selections.is_empty() && class.levels.iter().any(|row| !row.features.is_empty())
        })
        .unwrap();
    let feature = class
        .levels
        .iter()
        .find_map(|row| row.features.first().cloned())
        .unwrap();
    let selection = class.selections.first_mut().unwrap();
    let key = selection.key.clone();
    selection.skipped_levels.insert(7);
    selection.replaced_by.insert(7, feature.clone());
    assert_eq!(
        issues(&data),
        [(
            DataAt::FeatureSelection(key),
            DataIssue::ReplacementOnClassRow(feature)
        )]
    );
}

fn at_background(key: &str) -> DataAt {
    DataAt::Background(rules::BackgroundKey::new(key))
}

#[test]
fn a_background_key_is_used_once() {
    let mut data = world();
    let copy = data.backgrounds[0].clone();
    let key = copy.key.to_string();
    data.backgrounds.push(copy);
    assert_eq!(
        issues(&data),
        [(at_background(&key), DataIssue::DuplicateKey)]
    );
}

#[test]
fn a_backgrounds_feat_for_a_value_takes_a_value_of_that_kind() {
    let mut data = world();
    let guard = data
        .backgrounds
        .iter_mut()
        .find(|b| b.key.as_str() == "guard")
        .unwrap();
    // A martial weapon: Exotic Weapon Proficiency takes only exotic ones.
    guard.feats_for.insert(rules::FeatValue {
        feat: FeatKey::new("exotic-weapon-proficiency"),
        value: weapon("longsword"),
    });
    assert_eq!(
        issues(&data),
        [(
            at_background("guard"),
            DataIssue::ValueOfWrongKind(FeatKey::new("exotic-weapon-proficiency"))
        )]
    );
}

// ---- deities (ticket 86) ---------------------------------------------------------------------------------------

fn at_deity(key: &str) -> DataAt {
    DataAt::Deity(rules::DeityKey::new(key))
}

fn deity_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Deity {
    data.deities
        .iter_mut()
        .find(|d| d.key.as_str() == key)
        .unwrap()
}

fn priest_selection<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::FeatureSelection {
    class_mut(data, "priest")
        .selections
        .iter_mut()
        .find(|s| s.key.as_str() == key)
        .unwrap()
}

#[test]
fn a_deity_key_is_used_once() {
    let mut data = world();
    let copy = data.deities[0].clone();
    let key = copy.key.to_string();
    data.deities.push(copy);
    assert_eq!(issues(&data), [(at_deity(&key), DataIssue::DuplicateKey)]);
}

#[test]
fn a_deity_has_an_alignment_and_a_favoured_weapon() {
    let mut data = world();
    deity_mut(&mut data, "fist").alignments.clear();
    deity_mut(&mut data, "balance").favoured_weapons.clear();
    assert_eq!(
        sorted(issues(&data)),
        [
            (at_deity("balance"), DataIssue::DeityWithoutFavouredWeapon),
            (at_deity("fist"), DataIssue::DeityWithoutAlignment),
        ]
    );
}

#[test]
fn a_deitys_domains_and_a_blessings_domain_are_domains() {
    let mut data = world();
    // A revelation is no domain.
    let revelation = SelectionOptionKey::new("cleric/fire-revelations/fire-breath");
    deity_mut(&mut data, "fist")
        .domains
        .insert(revelation.clone());
    priest_selection(&mut data, "priest/blessing").options[0].domain = Some(revelation.clone());
    assert_eq!(
        sorted(issues(&data)),
        [
            (
                DataAt::SelectionOption(SelectionOptionKey::new("priest/blessing/fire")),
                DataIssue::NotADomain(revelation.clone()),
            ),
            (at_deity("fist"), DataIssue::NotADomain(revelation)),
        ]
    );
}

#[test]
fn a_deity_limited_selection_offers_only_options_with_a_domain() {
    let mut data = world();
    // The Seeker's creeds are neither domains nor stand for one.
    class_mut(&mut data, "seeker").selections[0].deity_domains = Some(rules::DeityDomains::Offered);
    assert_eq!(
        issues(&data),
        [(
            DataAt::FeatureSelection(FeatureSelectionKey::new("seeker/creed")),
            DataIssue::DeityLimitWithoutDomain(SelectionOptionKey::new("seeker/creed/devout")),
        )]
    );
}

#[test]
fn a_deity_offering_names_a_domain() {
    let mut data = world();
    let revelation = SelectionOptionKey::new("cleric/fire-revelations/fire-breath");
    class_mut(&mut data, "priest")
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == "priest/apostle")
        .unwrap()
        .prerequisite = Some(Prerequisite::DeityOffers(revelation.clone()));
    assert_eq!(
        issues(&data),
        [(
            DataAt::Archetype(ArchetypeKey::new("priest/apostle")),
            DataIssue::NotADomain(revelation),
        )]
    );
}

#[test]
fn a_favoured_weapon_feature_is_one_a_class_row_grants() {
    let mut data = world();
    let priest = class_mut(&mut data, "priest");
    priest.levels[0]
        .features
        .retain(|feature| feature.as_str() != "priest/proficiencies");
    assert_eq!(
        issues(&data),
        [(
            DataAt::Feature(FeatureKey::new("priest/proficiencies")),
            DataIssue::FavouredWeaponsNotGranted,
        )]
    );
}

fn unknown(at: DataAt, missing: DataAt) -> (DataAt, DataIssue) {
    (at, DataIssue::UnknownKey(missing))
}

#[test]
fn every_feat_named_is_one_the_data_has() {
    let mut data = world();
    let typo = || FeatKey::new("dodgee");
    class_mut(&mut data, "monk").feat_lists[0]
        .feats
        .insert(typo());
    class_mut(&mut data, "fighter").levels[0]
        .feats
        .insert(typo());
    data.backgrounds[0].feats.insert(typo());
    rogue_talent_mut(&mut data, "rogue/rogue-talents/fast-stealth").chosen_feat = Some(typo());
    feat_mut(&mut data, "toughness").prerequisite = Some(Prerequisite::Feat(typo()));
    assert_eq!(
        issues(&data),
        sorted(vec![
            unknown(
                DataAt::Background(rules::BackgroundKey::new("shackles-corsair")),
                at_feat("dodgee")
            ),
            unknown(at_class("fighter"), at_feat("dodgee")),
            unknown(at_feat("toughness"), at_feat("dodgee")),
            unknown(
                DataAt::FeatList(FeatListKey::new("monk/bonus-feats")),
                at_feat("dodgee")
            ),
            unknown(
                at_option("rogue/rogue-talents/fast-stealth"),
                at_feat("dodgee")
            ),
        ])
    );
}

fn path_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::MythicPath {
    data.mythic_paths
        .iter_mut()
        .find(|p| p.key.as_str() == key)
        .unwrap()
}

#[test]
fn every_feature_selection_and_option_named_is_one_the_data_has() {
    let mut data = world();
    let feature = || FeatureKey::new("fighter/bravry");
    let selection = || FeatureSelectionKey::new("rogue/rogue-talent");
    let option = || SelectionOptionKey::new("rogue/rogue-talents/fast-stelth");
    class_mut(&mut data, "fighter").levels[0]
        .features
        .push(feature());
    class_mut(&mut data, "fighter").levels[1]
        .selections
        .push(selection());
    class_mut(&mut data, "slayer").selections[0]
        .shared_options
        .insert(option());
    rogue_talent_mut(&mut data, "rogue/rogue-talents/resiliency").pool = Some(selection());
    feat_mut(&mut data, "toughness")
        .extra_pick_of
        .insert(selection());
    data.mythic_abilities[0].features.push(feature());
    feat_mut(&mut data, "dodge").prerequisite = Some(Prerequisite::SelectionOption(option()));
    path_mut(&mut data, "legend")
        .switch
        .as_mut()
        .unwrap()
        .dropped_features
        .insert(feature());
    let feature = || DataAt::Feature(feature());
    let selection = || DataAt::FeatureSelection(selection());
    assert_eq!(
        issues(&data),
        sorted(vec![
            unknown(at_class("fighter"), feature()),
            unknown(at_class("fighter"), selection()),
            unknown(
                DataAt::FeatureSelection(FeatureSelectionKey::new("slayer/slayer-talents")),
                DataAt::SelectionOption(option())
            ),
            unknown(at_option("rogue/rogue-talents/resiliency"), selection()),
            unknown(at_feat("toughness"), selection()),
            unknown(
                DataAt::MythicAbility(rules::MythicAbilityKey::new("mythic-charge")),
                feature()
            ),
            unknown(at_feat("dodge"), DataAt::SelectionOption(option())),
            unknown(at_path("legend"), feature()),
        ])
    );
}

#[test]
fn every_class_race_deity_skill_weapon_and_mythic_pick_named_is_one_the_data_has() {
    let mut data = world();
    let skill = || rules::SkillKey::new("lore-nature");
    let weapon = || rules::WeaponKey::new("longsord");
    let fighter = class_mut(&mut data, "fighter");
    fighter.counts_as.insert(rules::CountsAs {
        class: ClassKey::new("fightr"),
        toward: rules::CountsToward::FeatPrerequisites,
    });
    fighter.levels[0].feats_for.insert(rules::FeatValue {
        feat: FeatKey::new("weapon-focus"),
        value: rules::Parameter::Weapon(weapon()),
    });
    class_mut(&mut data, "wizard").class_skills.insert(skill());
    class_mut(&mut data, "rogue").selections[0]
        .option_levels
        .insert(ClassKey::new("sorcerr"));
    data.backgrounds[0].class_skills.insert(skill());
    data.deities[0].favoured_weapons.insert(weapon());
    feat_mut(&mut data, "toughness").prerequisite = Some(Prerequisite::All(vec![
        Prerequisite::Archetype(ArchetypeKey::new("fighter/armigr")),
        Prerequisite::Race(RaceKey::new("elff")),
        Prerequisite::Deity(rules::DeityKey::new("erastill")),
        Prerequisite::MythicFeat(rules::MythicFeatKey::new("extra-feet")),
        Prerequisite::MythicAbility(rules::MythicAbilityKey::new("mythic-charg")),
        Prerequisite::SkillRanks {
            skill: skill(),
            ranks: 1,
        },
    ]));
    path_mut(&mut data, "legend")
        .switch
        .as_mut()
        .unwrap()
        .from
        .insert(MythicPathKey::new("aeonn"));
    let deity = data.deities[0].key.clone();
    let toughness = || at_feat("toughness");
    assert_eq!(
        issues(&data),
        sorted(vec![
            unknown(
                DataAt::Background(rules::BackgroundKey::new("shackles-corsair")),
                DataAt::Skill(skill())
            ),
            unknown(at_class("fighter"), at_class("fightr")),
            unknown(at_class("fighter"), DataAt::Weapon(weapon())),
            unknown(at_class("wizard"), DataAt::Skill(skill())),
            unknown(DataAt::Deity(deity), DataAt::Weapon(weapon())),
            unknown(
                DataAt::FeatureSelection(FeatureSelectionKey::new("rogue/rogue-talents")),
                at_class("sorcerr")
            ),
            unknown(
                toughness(),
                DataAt::Archetype(ArchetypeKey::new("fighter/armigr"))
            ),
            unknown(toughness(), DataAt::Race(RaceKey::new("elff"))),
            unknown(toughness(), DataAt::Deity(rules::DeityKey::new("erastill"))),
            unknown(
                toughness(),
                DataAt::MythicFeat(rules::MythicFeatKey::new("extra-feet"))
            ),
            unknown(
                toughness(),
                DataAt::MythicAbility(rules::MythicAbilityKey::new("mythic-charg"))
            ),
            unknown(toughness(), DataAt::Skill(skill())),
            unknown(at_path("legend"), at_path("aeonn")),
        ])
    );
}

#[test]
fn every_feat_list_and_slot_selection_named_is_one_the_data_has() {
    let mut data = world();
    let list = || FeatListKey::new("sorcerer/bloodline-feets");
    let fighter = class_mut(&mut data, "fighter");
    fighter.levels[0]
        .feat_slots
        .push(FeatSlot::OneOf { list: list() });
    fighter.levels[0].feat_slots.push(FeatSlot::OptionList {
        selection: FeatureSelectionKey::new("sorcerer/bloodlin"),
        at_level: None,
        label: None,
    });
    let bloodline = class_mut(&mut data, "sorcerer")
        .selections
        .iter_mut()
        .flat_map(|selection| &mut selection.options)
        .find(|option| option.levels.iter().any(|row| row.feat_list.is_some()))
        .unwrap();
    let row = bloodline
        .levels
        .iter_mut()
        .find(|row| row.feat_list.is_some())
        .unwrap();
    row.feat_list = Some(list());
    let bloodline = DataAt::SelectionOption(bloodline.key.clone());
    let issues = issues(&data);
    assert_eq!(
        issues
            .into_iter()
            .filter(|(_, issue)| !matches!(issue, DataIssue::UnusedFeatList))
            .collect::<Vec<_>>(),
        sorted(vec![
            unknown(at_class("fighter"), DataAt::FeatList(list())),
            unknown(
                at_class("fighter"),
                DataAt::FeatureSelection(FeatureSelectionKey::new("sorcerer/bloodlin"))
            ),
            unknown(bloodline, DataAt::FeatList(list())),
        ])
    );
}

/// A path's rank rows and its switch's kept ranks are ranks of the progression.
#[test]
fn every_mythic_rank_named_is_one_the_progression_has() {
    let mut data = world();
    data.progression
        .mythic_ascensions
        .retain(|ascension| ascension.rank != 7);
    let named = issues(&data)
        .into_iter()
        .map(|(at, issue)| (at.to_string(), issue))
        .collect::<Vec<_>>();
    // The Trickster and the Aeon have an MR 7 row; Gold Dragon, Swarm and Devil keep picks through MR 7.
    assert_eq!(
        named,
        [
            ("mythic path aeon".into(), DataIssue::UnknownRank(7)),
            ("mythic path devil".into(), DataIssue::UnknownRank(7)),
            ("mythic path gold-dragon".into(), DataIssue::UnknownRank(7)),
            (
                "mythic path swarm-that-walks".into(),
                DataIssue::UnknownRank(7)
            ),
            ("mythic path trickster".into(), DataIssue::UnknownRank(7)),
        ]
    );
}

#[test]
fn an_unknown_key_reads_as_a_sentence_naming_both_kinds() {
    let problem = |issue| DataProblem {
        at: at_class("fighter"),
        issue,
    };
    assert_eq!(
        problem(DataIssue::UnknownKey(at_feat("dodgee"))).to_string(),
        "class fighter: names feat dodgee, which the data doesn't have"
    );
    assert_eq!(
        problem(DataIssue::UnknownRank(7)).to_string(),
        "class fighter: names MR 7, which the progression doesn't have"
    );
}

#[test]
fn a_weapons_stats_are_dice_a_threat_range_and_multiplier_a_weapon_can_have() {
    fn stats<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::WeaponStats {
        data.weapons
            .iter_mut()
            .find(|weapon| weapon.key.as_str() == key)
            .unwrap()
            .stats
            .as_mut()
            .unwrap()
    }
    let mut data = world();
    stats(&mut data, "dagger").damage.count = 0;
    stats(&mut data, "longsword").threat = 21;
    stats(&mut data, "bastard-sword").multiplier = 1;
    // A thrown weapon on the Weapon Finesse list.
    stats(&mut data, "dueling-sword").ranged = Some(rules::Ranged::Thrown);
    let at = |key: &str| DataAt::Weapon(rules::WeaponKey::new(key));

    assert_eq!(
        sorted(issues(&data)),
        [
            (at("bastard-sword"), DataIssue::WeaponStatsOutOfRange),
            (at("dagger"), DataIssue::WeaponStatsOutOfRange),
            (at("dueling-sword"), DataIssue::FinesseOnRangedWeapon),
            (at("longsword"), DataIssue::WeaponStatsOutOfRange),
        ]
    );
}

fn gear_item(key: &str, name: &str, bonuses: Vec<rules::GearBonus>) -> rules::GearItem {
    rules::GearItem {
        key: rules::GearKey::new(key),
        name: name.to_owned(),
        slot: rules::GearSlot::Belt,
        guide: None,
        bonuses,
        always_on: Vec::new(),
        other_effects: Vec::new(),
    }
}

fn gear_bonus(target: rules::GearTarget, amount: i8, kind: rules::BonusType) -> rules::GearBonus {
    rules::GearBonus {
        target,
        amount,
        kind,
    }
}

#[test]
fn gear_mistakes_are_found() {
    use rules::{Ability, BonusType, GearTarget};
    let dex = || GearTarget::Ability(Ability::Dexterity);
    let mut data = world();
    let mut bad = gear_item(
        "bad",
        "Same",
        vec![
            gear_bonus(dex(), 2, BonusType::Enhancement),
            gear_bonus(dex(), 4, BonusType::Enhancement),
            gear_bonus(GearTarget::Initiative, -1, BonusType::Luck),
            gear_bonus(GearTarget::Concentration, 0, BonusType::Untyped),
            gear_bonus(
                GearTarget::Skill(rules::SkillKey::new("cooking")),
                1,
                BonusType::Untyped,
            ),
            gear_bonus(
                GearTarget::SpellDc(rules::SpellScope::Value),
                1,
                BonusType::Untyped,
            ),
        ],
    );
    bad.other_effects = vec![String::new(), "Twice.".to_owned(), "Twice.".to_owned()];
    let mut both = gear_item(
        "both",
        "Both",
        vec![gear_bonus(dex(), 2, BonusType::Enhancement)],
    );
    both.always_on = vec!["DR 3/-".to_owned()];
    both.other_effects = vec!["DR 3/-".to_owned()];
    data.gear = vec![
        bad,
        both,
        gear_item("empty", "Same", Vec::new()),
        gear_item("empty", "Other", Vec::new()),
    ];
    let at = |key: &str| DataAt::Gear(rules::GearKey::new(key));
    let found = issues(&data);
    for (key, issue) in [
        ("bad", DataIssue::GearBonusTwice),
        ("bad", DataIssue::GearAmountInvalid),
        ("bad", DataIssue::GearSpellDcOfValue),
        ("bad", DataIssue::GearEffectInvalid),
        ("both", DataIssue::GearEffectInvalid),
        ("empty", DataIssue::GearWithoutBonus),
        ("empty", DataIssue::DuplicateKey),
        ("empty", DataIssue::GearNameTwice),
    ] {
        assert!(
            found.contains(&(at(key), issue.clone())),
            "{key}: {issue:?}"
        );
    }
    assert!(found.contains(&(
        at("bad"),
        DataIssue::UnknownKey(DataAt::Skill(rules::SkillKey::new("cooking")))
    )));
}
