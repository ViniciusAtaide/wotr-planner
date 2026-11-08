//! The Build start's pickers (ticket 96): race and subrace, faith, alignment, background and the racial ability.
//! Each option carries its status (a faith and alignment that don't fit, or `BreaksLater`) and the edit that takes
//! it, side effects included.

use rules::{
    Ability, Alignment, BackgroundKey, Build, BuildStart, ChoiceOption, DeityKey, Edit, Faith,
    Issue, PointBuy, RaceChoice, RaceKey, Step, Stepper, SubraceKey,
};

use crate::support::*;

fn deity(key: &str) -> Faith {
    Faith::Deity(DeityKey::new(key))
}

/// Each option's value and status, in order.
fn statuses<T: Clone>(options: &[ChoiceOption<T>]) -> Vec<(T, Result<(), Issue>)> {
    options
        .iter()
        .map(|option| (option.value.clone(), option.status.clone()))
        .collect()
}

/// A human with no levels, no faith and no alignment yet.
fn unfaithful() -> Build {
    let mut build = build("human", &[]);
    build.start.deity = None;
    build.start.alignment = None;
    build
}

/// The Build start edit that changes `build`'s start by `change`.
fn start_edit(build: &Build, change: impl FnOnce(&mut BuildStart)) -> Edit {
    let mut start = build.start.clone();
    change(&mut start);
    Edit::Start(start)
}

// ---- the faith and the alignment ------------------------------------------------------------------------------

#[test]
fn every_alignment_needs_a_faith_first() {
    let build = unfaithful();
    let options = engine().alignment_options(&build);
    assert_eq!(
        statuses(&options),
        Alignment::ALL.map(|alignment| (alignment, Err(Issue::NeedsFaith)))
    );
    for option in &options {
        assert_eq!(
            option.edit,
            start_edit(&build, |start| start.alignment = Some(option.value))
        );
    }
    assert_eq!(
        engine().issue_text(&Issue::NeedsFaith).unwrap(),
        "Choose a deity or atheism first"
    );
}

#[test]
fn atheism_opens_every_alignment_and_a_deity_its_own() {
    let mut build = unfaithful();
    build.start.deity = Some(Faith::Atheism);
    assert_eq!(
        statuses(&engine().alignment_options(&build)),
        Alignment::ALL.map(|alignment| (alignment, Ok(())))
    );
    build.start.deity = Some(deity("lawgiver"));
    let lawgiver = Issue::DeityAlignment {
        deity: DeityKey::new("lawgiver"),
    };
    let open = [
        Alignment::LawfulGood,
        Alignment::NeutralGood,
        Alignment::LawfulNeutral,
    ];
    assert_eq!(
        statuses(&engine().alignment_options(&build)),
        Alignment::ALL.map(|alignment| (
            alignment,
            if open.contains(&alignment) {
                Ok(())
            } else {
                Err(lawgiver.clone())
            }
        ))
    );
    assert_eq!(
        engine().issue_text(&lawgiver).unwrap(),
        "Lawgiver allows Lawful Good, Neutral Good or Lawful Neutral"
    );
}

#[test]
fn a_deity_that_does_not_allow_the_alignment_is_locked_with_its_alignments() {
    let mut build = unfaithful();
    build.start.deity = Some(Faith::Atheism);
    build.start.alignment = Some(Alignment::ChaoticNeutral);
    let options = engine().faith_options(&build);
    let misfit = |key: &str| {
        Err(Issue::DeityAlignment {
            deity: DeityKey::new(key),
        })
    };
    assert_eq!(
        statuses(&options),
        [
            (Faith::Atheism, Ok(())),
            (deity("balance"), Ok(())),
            (deity("fist"), misfit("fist")),
            (deity("lawgiver"), misfit("lawgiver")),
        ]
    );
    for option in &options {
        assert_eq!(
            option.edit,
            start_edit(&build, |start| start.deity = Some(option.value.clone()))
        );
    }
    // With no alignment yet, every faith fits.
    build.start.alignment = None;
    assert!(
        engine()
            .faith_options(&build)
            .iter()
            .all(|option| option.status.is_ok())
    );
}

#[test]
fn a_faith_a_later_choice_needs_is_kept() {
    // A Priest serves a deity: atheism would break its level.
    let mut build = build("human", &["priest"]);
    build.start.deity = Some(deity("lawgiver"));
    build.start.alignment = Some(Alignment::LawfulNeutral);
    assert!(engine().is_possible(&build));
    let options = engine().faith_options(&build);
    assert_eq!(options[0].value, Faith::Atheism);
    assert_eq!(options[0].status, Err(Issue::BreaksLater));
    let lawgiver = options
        .iter()
        .find(|option| option.value == deity("lawgiver"))
        .unwrap();
    assert_eq!(lawgiver.status, Ok(()), "what the Build holds");
}

// ---- the race and subrace -------------------------------------------------------------------------------------

fn race(race: &str, subrace: Option<&str>) -> RaceChoice {
    RaceChoice {
        race: RaceKey::new(race),
        subrace: subrace.map(SubraceKey::new),
    }
}

#[test]
fn the_race_picker_lists_each_race_then_its_subraces_and_resets_the_racial_ability() {
    let mut build = build("human", &[]);
    build.start.racial_ability = Some(Ability::Strength);
    let options = engine().race_options(&build);
    // Each race and subrace, and whether its start keeps the racial +2: only those that choose one do.
    let expected = [
        (race("dwarf", None), None),
        (race("elf", None), None),
        (race("elf", Some("elf/blightborn")), None),
        (race("half-elf", None), Some(Ability::Strength)),
        (
            race("half-elf", Some("half-elf/adaptable")),
            Some(Ability::Strength),
        ),
        (
            race("half-elf", Some("half-elf/dual-heritage")),
            Some(Ability::Strength),
        ),
        (race("human", None), Some(Ability::Strength)),
    ];
    assert_eq!(options.len(), expected.len());
    for (option, (value, racial_ability)) in options.iter().zip(expected) {
        assert_eq!(option.value, value);
        assert_eq!(option.status, Ok(()), "{value:?}");
        assert_eq!(
            option.edit,
            start_edit(&build, |start| {
                start.race = Some(value.race.clone());
                start.subrace = value.subrace.clone();
                start.racial_ability = racial_ability;
            }),
            "{value:?}"
        );
    }
}

#[test]
fn a_race_without_the_slot_a_pick_fills_breaks_it() {
    // A human's bonus feat at level 1: no other race has a slot that takes Toughness.
    let mut build = build("human", &["fighter"]);
    build.picks.insert(race_slot(0), pick_feat("toughness"));
    assert!(engine().is_possible(&build));
    let options = engine().race_options(&build);
    assert_eq!(options.len(), 7);
    for option in options {
        let expected = if option.value == race("human", None) {
            Ok(())
        } else {
            Err(Issue::BreaksLater)
        };
        assert_eq!(option.status, expected, "{:?}", option.value);
    }
}

// ---- the background -------------------------------------------------------------------------------------------

#[test]
fn the_background_picker_lists_every_background_and_keeps_the_one_a_pick_needs() {
    let mut build = build("dwarf", &["wizard"]);
    build.start.background = Some(BackgroundKey::new("guard"));
    let keys = [
        "aldori-swordsman",
        "guard",
        "martial-disciple",
        "none",
        "shackles-corsair",
    ];
    let options = engine().background_options(&build);
    assert_eq!(
        statuses(&options),
        keys.map(|key| (BackgroundKey::new(key), Ok(())))
    );
    for option in &options {
        assert_eq!(
            option.edit,
            start_edit(&build, |start| start.background =
                Some(option.value.clone()))
        );
    }
    // Armor Focus (Light) at level 1 needs the Guard's light armour.
    build.picks.insert(
        general_slot(1),
        pick_feat_for(
            "armor-focus",
            rules::Parameter::ArmourType(rules::ArmourType::Light),
        ),
    );
    assert!(engine().is_possible(&build));
    assert_eq!(
        statuses(&engine().background_options(&build)),
        keys.map(|key| (
            BackgroundKey::new(key),
            if key == "guard" {
                Ok(())
            } else {
                Err(Issue::BreaksLater)
            }
        ))
    );
}

// ---- the racial ability ---------------------------------------------------------------------------------------

#[test]
fn the_racial_ability_is_offered_only_by_a_race_that_chooses_one() {
    let human = build("human", &[]);
    let options = engine().racial_ability_options(&human);
    assert_eq!(
        statuses(&options),
        rules::AbilityScores::ABILITIES.map(|ability| (ability, Ok(())))
    );
    for option in &options {
        assert_eq!(
            option.edit,
            start_edit(&human, |start| start.racial_ability = Some(option.value))
        );
    }
    assert_eq!(engine().racial_ability_options(&build("dwarf", &[])), []);
}

#[test]
fn a_racial_ability_a_pick_needs_stays_where_it_is() {
    // Str 12 + 2 meets Power Attack's Str 13; the +2 anywhere else doesn't.
    let mut build = build("human", &["fighter"]);
    build.start.ability_scores = point_buy([12, 14, 12, 13, 10, 8]);
    build.start.racial_ability = Some(Ability::Strength);
    build
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    assert!(engine().is_possible(&build));
    let options = engine().racial_ability_options(&build);
    assert_eq!(options.len(), 6);
    for option in options {
        let expected = if option.value == Ability::Strength {
            Ok(())
        } else {
            Err(Issue::BreaksLater)
        };
        assert_eq!(option.status, expected, "{:?}", option.value);
    }
}

// ---- the point buy --------------------------------------------------------------------------------------------

/// The Build start edit that sets the point buy.
fn scores_edit(build: &Build, scores: PointBuy) -> Edit {
    start_edit(build, |start| start.ability_scores = scores)
}

/// A step's status, or `None` past the end of the range.
type StepStatus = Option<Result<(), Issue>>;

/// Each score's steps: whether it has a step down and up, and their statuses.
fn step_statuses<T: Clone>(steppers: &[Stepper<T>]) -> Vec<(T, StepStatus, StepStatus)> {
    let status = |step: &Option<Step>| step.as_ref().map(|step| step.status.clone());
    steppers
        .iter()
        .map(|stepper| {
            (
                stepper.value.clone(),
                status(&stepper.down),
                status(&stepper.up),
            )
        })
        .collect()
}

#[test]
fn each_score_steps_down_and_up_by_one() {
    let build = build("human", &[]);
    let steppers = engine().point_buy_steps(&build);
    assert_eq!(
        step_statuses(&steppers),
        rules::AbilityScores::ABILITIES.map(|ability| (ability, Some(Ok(())), Some(Ok(()))))
    );
    // Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8.
    let strength = &steppers[0];
    assert_eq!(
        strength.down.as_ref().unwrap().edit,
        scores_edit(&build, point_buy([15, 14, 12, 13, 10, 8]))
    );
    assert_eq!(
        strength.up.as_ref().unwrap().edit,
        scores_edit(&build, point_buy([17, 14, 12, 13, 10, 8]))
    );
    let charisma = &steppers[5];
    assert_eq!(
        charisma.down.as_ref().unwrap().edit,
        scores_edit(&build, point_buy([16, 14, 12, 13, 10, 7]))
    );
}

#[test]
fn a_score_steps_no_lower_than_7_nor_higher_than_18() {
    let mut build = build("human", &[]);
    build.start.ability_scores = point_buy([18, 7, 7, 7, 7, 7]);
    let ok = Some(Ok(()));
    assert_eq!(
        step_statuses(&engine().point_buy_steps(&build)),
        [
            (Ability::Strength, ok.clone(), None),
            (Ability::Dexterity, None, ok.clone()),
            (Ability::Constitution, None, ok.clone()),
            (Ability::Intelligence, None, ok.clone()),
            (Ability::Wisdom, None, ok.clone()),
            (Ability::Charisma, None, ok.clone()),
        ]
    );
}

#[test]
fn no_score_steps_up_once_the_budget_is_spent() {
    let mut build = build("human", &[]);
    // 17 + 5 + 2 + 0 + 0 + 1: all 25 points.
    build.start.ability_scores = point_buy([18, 14, 12, 10, 10, 11]);
    assert_eq!(build.start.ability_scores.cost(), 25);
    assert_eq!(
        step_statuses(&engine().point_buy_steps(&build)),
        rules::AbilityScores::ABILITIES.map(|ability| (ability, Some(Ok(())), None))
    );
}

#[test]
fn a_score_a_pick_needs_does_not_step_below_it() {
    // Str 13 for Power Attack at level 1, with the human's +2 on nothing yet.
    let mut build = build("human", &["fighter"]);
    build.start.ability_scores = point_buy([13, 14, 12, 13, 10, 8]);
    build
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    assert!(engine().is_possible(&build));
    let steppers = engine().point_buy_steps(&build);
    assert_eq!(
        step_statuses(&steppers[..1]),
        [(
            Ability::Strength,
            Some(Err(Issue::BreaksLater)),
            Some(Ok(()))
        )]
    );
}
