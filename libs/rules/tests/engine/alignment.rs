//! Ticket 63: an archetype's alignment requirement replaces its class's (user, 2026-11-02: "check the alignment per
//! archetype and not on the base class"). The class's other conditions still hold, and an archetype without one
//! keeps the class's.

use rules::{
    Ability, Alignment, Archetype, ArchetypeKey, Build, ClassChoice, ClassOption, DataAt,
    DataIssue, DataProblem, Edit, Engine, GameData, GuideRef, Issue, Prerequisite, Unmet,
};

use crate::support::*;

use Alignment::*;

const LAWFUL: [Alignment; 3] = [LawfulGood, LawfulNeutral, LawfulEvil];
const NOT_LAWFUL: [Alignment; 6] = [
    NeutralGood,
    ChaoticGood,
    TrueNeutral,
    ChaoticNeutral,
    NeutralEvil,
    ChaoticEvil,
];

fn any_of(alignments: &[Alignment]) -> Prerequisite {
    Prerequisite::Any(
        alignments
            .iter()
            .copied()
            .map(Prerequisite::Alignment)
            .collect(),
    )
}

/// What an `Any` of alignments reads as when none holds.
fn none_of(alignments: &[Alignment]) -> Unmet {
    Unmet::Any(
        alignments
            .iter()
            .map(|alignment| Unmet::Condition {
                needs: Prerequisite::Alignment(*alignment),
                has: None,
            })
            .collect(),
    )
}

fn wisdom_12() -> Prerequisite {
    Prerequisite::AbilityScore {
        ability: Ability::Wisdom,
        score: 12,
    }
}

fn archetype(key: &str, prerequisite: Prerequisite) -> Archetype {
    Archetype {
        key: ArchetypeKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(format!("monk#{key}")),
        prerequisite: Some(prerequisite),
        skill_points: None,
        class_skills: None,
        spontaneous: None,
        fort: None,
        reflex: None,
        will: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels: vec![],
    }
}

/// The world with a monk who must be lawful and have Wisdom 12, as the guide's monk must be lawful: a Drunken
/// Master who "Cannot be Lawful", a Lawful Good one with Strength 13 (an archetype that narrows the class's
/// alignment beside another condition), and the Zen Archer, with no alignment of its own.
fn world_with_lawful_monk() -> GameData {
    let mut data = world();
    let monk = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == "monk")
        .unwrap();
    monk.prerequisite = Some(Prerequisite::All(vec![any_of(&LAWFUL), wisdom_12()]));
    monk.archetypes
        .push(archetype("monk/drunken-master", any_of(&NOT_LAWFUL)));
    monk.archetypes.push(archetype(
        "monk/lawful-good",
        Prerequisite::All(vec![
            Prerequisite::Alignment(LawfulGood),
            Prerequisite::AbilityScore {
                ability: Ability::Strength,
                score: 13,
            },
        ]),
    ));
    assert_eq!(data.check(), []);
    data
}

fn lawful_monk_engine() -> Engine {
    Engine::new(world_with_lawful_monk())
}

/// A monk 1 of `archetype` (or none) at `alignment`, with Wisdom 12.
fn monk(archetype: Option<&str>, alignment: Alignment) -> Build {
    let mut build = build("elf", &["monk"]);
    build.start.alignment = Some(alignment);
    build.start.ability_scores = point_buy([16, 14, 12, 13, 12, 8]);
    build.class_choices.insert(
        class("monk"),
        ClassChoice {
            archetype: archetype.map(ArchetypeKey::new),
            ..ClassChoice::default()
        },
    );
    build
}

/// The level-1 class picker's status for the monk with `archetype`, for a Build like `build` before its first level.
fn entry(build: &Build, archetype: Option<&str>) -> Result<(), Issue> {
    let mut before = build.clone();
    before.levels.clear();
    before.class_choices.clear();
    lawful_monk_engine()
        .class_options(&before, 1)
        .unwrap()
        .into_iter()
        .find(
            |ClassOption {
                 class: key,
                 archetype: a,
                 ..
             }| {
                *key == class("monk") && a.as_ref().map(ArchetypeKey::as_str) == archetype
            },
        )
        .unwrap_or_else(|| panic!("no monk option {archetype:?}"))
        .status
}

#[test]
fn a_drunken_master_needs_only_its_own_alignment_not_the_monks() {
    let engine = lawful_monk_engine();
    for alignment in NOT_LAWFUL {
        let build = monk(Some("monk/drunken-master"), alignment);
        assert!(engine.is_possible(&build), "{alignment:?}");
        assert_eq!(entry(&build, Some("monk/drunken-master")), Ok(()));
    }
}

/// The reason names the archetype's alignment rule, not the class's.
#[test]
fn a_lawful_drunken_master_is_locked_by_the_archetypes_alignment() {
    let engine = lawful_monk_engine();
    for alignment in LAWFUL {
        let build = monk(Some("monk/drunken-master"), alignment);
        assert!(!engine.is_possible(&build), "{alignment:?}");
        assert_eq!(
            entry(&build, Some("monk/drunken-master")),
            Err(Issue::Unmet(none_of(&NOT_LAWFUL)))
        );
    }
}

#[test]
fn the_class_keeps_its_other_conditions() {
    let engine = lawful_monk_engine();
    let mut build = monk(Some("monk/drunken-master"), ChaoticNeutral);
    build.start.ability_scores = point_buy([16, 14, 12, 13, 10, 8]);
    assert!(!engine.is_possible(&build));
    assert_eq!(
        entry(&build, Some("monk/drunken-master")),
        Err(Issue::Unmet(Unmet::Condition {
            needs: wisdom_12(),
            has: Some(10),
        }))
    );
}

#[test]
fn a_monk_and_an_archetype_without_an_alignment_keep_the_monks() {
    let engine = lawful_monk_engine();
    for archetype in [None, Some("monk/zen-archer")] {
        for alignment in LAWFUL {
            let build = monk(archetype, alignment);
            assert!(engine.is_possible(&build), "{archetype:?} {alignment:?}");
            assert_eq!(entry(&build, archetype), Ok(()));
        }
        for alignment in NOT_LAWFUL {
            let build = monk(archetype, alignment);
            assert!(!engine.is_possible(&build), "{archetype:?} {alignment:?}");
            assert_eq!(
                entry(&build, archetype),
                Err(Issue::Unmet(none_of(&LAWFUL)))
            );
        }
    }
}

/// An archetype that narrows the class's alignment works the same way: its rule is the one checked.
#[test]
fn an_archetype_that_narrows_the_alignment_is_checked_alone() {
    let engine = lawful_monk_engine();
    let archetype = Some("monk/lawful-good");
    assert!(engine.is_possible(&monk(archetype, LawfulGood)));
    for alignment in [LawfulNeutral, ChaoticGood] {
        let build = monk(archetype, alignment);
        assert!(!engine.is_possible(&build), "{alignment:?}");
        assert_eq!(
            entry(&build, archetype),
            Err(Issue::Unmet(Unmet::Condition {
                needs: Prerequisite::Alignment(LawfulGood),
                has: None,
            }))
        );
    }
}

/// The Build start's alignment control asks the walk, which checks the archetype's rule.
#[test]
fn a_drunken_masters_alignment_can_change_only_within_its_rule() {
    let engine = lawful_monk_engine();
    let build = monk(Some("monk/drunken-master"), ChaoticNeutral);
    let set = |alignment| {
        let mut start = build.start.clone();
        start.alignment = Some(alignment);
        engine.allows(&build, &Edit::Start(start))
    };
    assert!(set(NeutralEvil));
    assert!(!set(LawfulNeutral));
    let plain = monk(None, LawfulNeutral);
    let mut start = plain.start.clone();
    start.alignment = Some(ChaoticNeutral);
    assert!(!engine.allows(&plain, &Edit::Start(start)));
}

/// "The class's alignment requirement" is a condition of its own: an alignment mixed into a group with other
/// conditions, in a class's or an archetype's prerequisite, is refused.
#[test]
fn an_alignment_inside_another_condition_of_a_class_or_archetype_is_a_data_problem() {
    let mut data = world_with_lawful_monk();
    let mixed = Prerequisite::Any(vec![
        Prerequisite::Alignment(LawfulGood),
        Prerequisite::BaseAttackBonus(1),
    ]);
    let monk = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == "monk")
        .unwrap();
    monk.prerequisite = Some(Prerequisite::All(vec![mixed.clone(), wisdom_12()]));
    monk.archetypes[0].prerequisite = Some(Prerequisite::Not(Box::new(mixed.clone())));
    let zen_archer = monk.archetypes[0].key.clone();
    // A feat or an option may mix them (the Champion's blessings).
    data.feats[0].prerequisite = Some(mixed);
    let mut found: Vec<(DataAt, DataIssue)> = data
        .check()
        .into_iter()
        .map(|DataProblem { at, issue }| (at, issue))
        .collect();
    found.sort();
    assert_eq!(
        found,
        [
            (
                DataAt::Class(class("monk")),
                DataIssue::AlignmentNotOnItsOwn
            ),
            (
                DataAt::Archetype(zen_archer),
                DataIssue::AlignmentNotOnItsOwn
            ),
        ]
    );
    assert_eq!(
        DataProblem {
            at: DataAt::Class(class("monk")),
            issue: DataIssue::AlignmentNotOnItsOwn,
        }
        .to_string(),
        "class monk: an alignment sits in a group with other conditions; the alignment requirement must be a \
         condition of its own"
    );
}
