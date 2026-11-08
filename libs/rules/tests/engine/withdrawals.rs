//! Withdrawals (ticket 95): the choices a Build holds, each with the edit that takes it back, as the engine lists
//! them for the proof and answers them one at a time for a picker's remove control.

use std::collections::BTreeSet;

use rules::{
    Ability, ArchetypeKey, BackgroundKey, Build, BuildStart, Choice, ClassChoice, Edit, Faith,
    SubraceKey,
};

use crate::removable::removable_builds;
use crate::support::*;

/// A variant's name, matched exhaustively so a new `Choice` fails to compile here until the tests say whether a
/// Build holds it.
fn kind(choice: &Choice) -> &'static str {
    match choice {
        Choice::Race => "Race",
        Choice::Subrace => "Subrace",
        Choice::Alignment => "Alignment",
        Choice::Background => "Background",
        Choice::Deity => "Deity",
        Choice::AbilityScore { .. } => "AbilityScore",
        Choice::Class { .. } => "Class",
        Choice::Archetype { .. } => "Archetype",
        Choice::Advances { .. } => "Advances",
        Choice::AbilityIncrease { .. } => "AbilityIncrease",
        Choice::SkillRanks { .. } => "SkillRanks",
        Choice::SkillPoints { .. } => "SkillPoints",
        Choice::CharacterLevel { .. } => "CharacterLevel",
        Choice::FirstAscension => "FirstAscension",
        Choice::MythicPath => "MythicPath",
        Choice::PathSwitch => "PathSwitch",
        Choice::Slot(_) => "Slot",
    }
}

/// A Build that makes every kind of choice: a half-elf of the Dual Heritage with its +2 on Strength, a background,
/// atheism, an armiger fighter, a wizard and an Arcane Trickster advancing it (21 levels), an ability increase, skill
/// ranks, a first ascension, a path, a Legend switch and a pick.
fn holding_everything() -> Build {
    let mut build = build(
        "half-elf",
        &[times("fighter", 19), vec!["wizard", "arcane-trickster"]].concat(),
    );
    build.start.subrace = Some(SubraceKey::new("half-elf/dual-heritage"));
    build.start.racial_ability = Some(Ability::Strength);
    build.start.background = Some(BackgroundKey::new("shackles-corsair"));
    build.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    build.class_choices.insert(
        class("arcane-trickster"),
        ClassChoice {
            archetype: None,
            advances: BTreeSet::from([class("wizard")]),
        },
    );
    build.levels[3].ability_increase = Some(Ability::Strength);
    build.levels[0].skill_ranks.insert(skill("mobility"), 1);
    build.first_ascension = Some(path("trickster"));
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    build
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    build
}

/// The held choices' withdrawals, by choice.
fn withdrawals(build: &Build) -> Vec<(Choice, Edit)> {
    engine()
        .held_choices(build)
        .into_iter()
        .map(|withdrawal| (withdrawal.choice, withdrawal.edit))
        .collect()
}

#[test]
fn the_held_choices_cover_every_kind_of_choice_a_build_makes() {
    let held: BTreeSet<&str> = withdrawals(&holding_everything())
        .iter()
        .map(|(choice, _)| kind(choice))
        .collect();
    // A level's skill points are what its ranks spend, not a choice of their own.
    let expected = BTreeSet::from([
        "Race",
        "Subrace",
        "Alignment",
        "Background",
        "Deity",
        "AbilityScore",
        "Class",
        "Archetype",
        "Advances",
        "AbilityIncrease",
        "SkillRanks",
        "CharacterLevel",
        "FirstAscension",
        "MythicPath",
        "PathSwitch",
        "Slot",
    ]);
    assert_eq!(held, expected);
}

#[test]
fn a_build_holds_only_the_choices_it_makes() {
    // An elf fighter 1 with no subrace, background, path or picks.
    let build = build("elf", &["fighter"]);
    let held: Vec<Choice> = withdrawals(&build)
        .into_iter()
        .map(|(choice, _)| choice)
        .collect();
    let mut expected = vec![Choice::Race, Choice::Alignment, Choice::Deity];
    expected
        .extend(rules::AbilityScores::ABILITIES.map(|ability| Choice::AbilityScore { ability }));
    expected.extend([
        Choice::Class { level: 1 },
        Choice::CharacterLevel { level: 1 },
    ]);
    assert_eq!(
        held.iter().collect::<BTreeSet<_>>(),
        expected.iter().collect::<BTreeSet<_>>()
    );
    assert_eq!(held.len(), expected.len(), "each choice once");
}

#[test]
fn the_race_withdrawal_clears_the_race_its_subrace_and_its_racial_ability() {
    let build = holding_everything();
    let (_, edit) = withdrawals(&build)
        .into_iter()
        .find(|(choice, _)| choice == &Choice::Race)
        .unwrap();
    assert_eq!(
        edit,
        Edit::Start(BuildStart {
            race: None,
            subrace: None,
            racial_ability: None,
            ..build.start.clone()
        })
    );
}

#[test]
fn each_withdrawal_takes_back_only_its_own_choice() {
    let build = holding_everything();
    let start = |change: fn(&mut BuildStart)| {
        let mut start = build.start.clone();
        change(&mut start);
        Edit::Start(start)
    };
    let held = withdrawals(&build);
    let edit_of = |choice: Choice| {
        held.iter()
            .find(|(held, _)| held == &choice)
            .map(|(_, edit)| edit.clone())
            .unwrap_or_else(|| panic!("{choice:?} isn't held"))
    };
    assert_eq!(
        edit_of(Choice::Subrace),
        start(|start| start.subrace = None)
    );
    assert_eq!(edit_of(Choice::Deity), start(|start| start.deity = None));
    assert_eq!(
        edit_of(Choice::Alignment),
        start(|start| start.alignment = None)
    );
    assert_eq!(
        edit_of(Choice::Background),
        start(|start| start.background = None)
    );
    assert_eq!(
        edit_of(Choice::AbilityScore {
            ability: Ability::Strength
        }),
        start(|start| {
            start.ability_scores = start.ability_scores.with(Ability::Strength, 7).unwrap();
        })
    );
    assert_eq!(edit_of(Choice::FirstAscension), Edit::FirstAscension(None));
    assert_eq!(edit_of(Choice::MythicPath), Edit::MythicPath(None));
    assert_eq!(edit_of(Choice::PathSwitch), Edit::PathSwitch(None));
    assert_eq!(
        edit_of(Choice::Slot(general_slot(1))),
        Edit::Clear(general_slot(1))
    );
    assert_eq!(
        edit_of(Choice::AbilityIncrease { level: 4 }),
        Edit::AbilityIncrease {
            level: 4,
            ability: None
        }
    );
    assert_eq!(
        edit_of(Choice::SkillRanks {
            level: 1,
            skill: skill("mobility")
        }),
        Edit::SkillRanks {
            level: 1,
            skill: skill("mobility"),
            ranks: 0
        }
    );
    // Only the last level comes off as a whole, by Remove level.
    assert_eq!(
        edit_of(Choice::CharacterLevel { level: 21 }),
        Edit::RemoveLevel
    );
    assert!(
        !held
            .iter()
            .any(|(choice, _)| choice == &Choice::CharacterLevel { level: 20 })
    );
}

#[test]
fn each_withdrawal_sets_its_choice() {
    // `Edit::choices` names the choice each withdrawal takes back, but Remove level, which knows no level.
    for (choice, edit) in withdrawals(&holding_everything()) {
        if edit == Edit::RemoveLevel {
            continue;
        }
        assert!(edit.choices().contains(&choice), "{choice:?}: {edit:?}");
    }
}

// ---- one choice, for a remove control --------------------------------------------------------------------------

#[test]
fn a_remove_control_gets_nothing_for_a_choice_the_build_doesnt_hold() {
    let engine = engine();
    let build = build("elf", &times("fighter", 2));
    for choice in [
        Choice::Subrace,
        Choice::Background,
        Choice::FirstAscension,
        Choice::MythicPath,
        Choice::PathSwitch,
        Choice::Slot(general_slot(1)),
        Choice::AbilityIncrease { level: 2 },
        Choice::Class { level: 3 },
        Choice::CharacterLevel { level: 1 },
        Choice::SkillPoints { level: 1 },
        Choice::Archetype {
            class: class("fighter"),
        },
    ] {
        assert_eq!(engine.withdrawal(&build, &choice), None, "{choice:?}");
    }
}

#[test]
fn a_remove_control_gets_the_withdrawal_and_whether_it_is_removable_now() {
    let engine = engine();
    let mut builds = removable_builds();
    builds.push(holding_everything());
    for build in builds {
        for held in engine.held_choices(&build) {
            assert_eq!(
                engine.withdrawal(&build, &held.choice),
                Some((held.edit.clone(), engine.is_removable(&build, &held.choice))),
                "{:?}",
                held.choice
            );
        }
    }
}

#[test]
fn the_faith_withdrawal_is_locked_while_the_alignment_needs_it() {
    let engine = engine();
    let mut build = build("elf", &["fighter"]);
    let unfaithful = Edit::Start(BuildStart {
        deity: None,
        ..build.start.clone()
    });
    assert_eq!(
        engine.withdrawal(&build, &Choice::Deity),
        Some((unfaithful, false))
    );

    build.start.alignment = None;
    let unfaithful = Edit::Start(BuildStart {
        deity: None,
        ..build.start.clone()
    });
    assert_eq!(
        engine.withdrawal(&build, &Choice::Deity),
        Some((unfaithful, true))
    );
    assert_eq!(build.start.deity, Some(Faith::Atheism));
}

#[test]
fn the_last_level_is_always_removable() {
    let engine = engine();
    let build = holding_everything();
    assert_eq!(
        engine.withdrawal(&build, &Choice::CharacterLevel { level: 21 }),
        Some((Edit::RemoveLevel, true))
    );
}
