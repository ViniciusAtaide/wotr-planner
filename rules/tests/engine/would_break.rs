//! The would-break check: which later choices a proposed edit would invalidate.

use rules::{ArchetypeKey, Build, Choice, ClassChoice, Edit, Issue, Problem, Unmet};

use crate::support::*;

fn breaks(build: &Build, edit: Edit) -> Vec<Choice> {
    engine()
        .would_break(build, &edit)
        .into_iter()
        .map(|problem| problem.at)
        .collect()
}

/// Elf fighter 5: Power Attack (F1 bonus), Cleave (L1), Great Cleave (F4 bonus), Cleaving Finish (L5).
fn cleave_chain() -> Build {
    let mut build = build("race:elf", &times("class:fighter", 5));
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("feat:power-attack"));
    build
        .picks
        .insert(general_slot(1), pick_feat("feat:cleave"));
    build
        .picks
        .insert(class_slot(4, 0), pick_feat("feat:great-cleave"));
    build
        .picks
        .insert(general_slot(5), pick_feat("feat:cleaving-finish"));
    assert!(engine().validate(&build).is_empty());
    build
}

#[test]
fn removing_cleave_breaks_great_cleave_and_what_depends_on_it() {
    let build = cleave_chain();
    assert_eq!(
        breaks(&build, Edit::Clear(general_slot(1))),
        [
            Choice::Slot(class_slot(4, 0)),
            Choice::Slot(general_slot(5))
        ]
    );
    // Replacing Cleave with something else breaks the same chain.
    assert_eq!(
        breaks(
            &build,
            Edit::Pick {
                slot: general_slot(1),
                pick: pick_feat("feat:toughness"),
            }
        ),
        [
            Choice::Slot(class_slot(4, 0)),
            Choice::Slot(general_slot(5))
        ]
    );
    // Removing Power Attack breaks Cleave too, at the same level.
    assert_eq!(
        breaks(&build, Edit::Clear(class_slot(1, 0))),
        [
            Choice::Slot(general_slot(1)),
            Choice::Slot(class_slot(4, 0)),
            Choice::Slot(general_slot(5))
        ]
    );
    // The last link breaks nothing.
    assert_eq!(breaks(&build, Edit::Clear(general_slot(5))), []);
}

#[test]
fn the_reason_is_the_one_at_the_broken_pick() {
    let build = cleave_chain();
    let problems = engine().would_break(&build, &Edit::Clear(general_slot(1)));
    assert_eq!(
        problems[0],
        Problem {
            at: Choice::Slot(class_slot(4, 0)),
            issue: Issue::Unmet(Unmet::Condition {
                needs: rules::Prerequisite::Feat(feat("feat:cleave")),
                has: None,
            }),
        }
    );
}

#[test]
fn a_class_change_breaks_picks_in_slots_it_takes_away() {
    let build = cleave_chain();
    // Fighter 4 -> Wizard 1: the F4 bonus slot is gone, and BAB drops to +3.
    assert_eq!(
        breaks(
            &build,
            Edit::Class {
                level: 4,
                class: class("class:wizard"),
                choice: None,
            }
        ),
        [
            Choice::Slot(class_slot(4, 0)),
            Choice::Slot(general_slot(5))
        ]
    );
}

#[test]
fn an_archetype_change_breaks_picks_in_replaced_slots() {
    let build = cleave_chain();
    // The armiger has no Combat feat slots at fighter levels 1 and 4. Within a level, picks for slots that are
    // gone are reported after the level's remaining slots.
    assert_eq!(
        breaks(
            &build,
            Edit::Class {
                level: 1,
                class: class("class:fighter"),
                choice: Some(ClassChoice {
                    archetype: Some(ArchetypeKey::new("archetype:fighter/armiger")),
                    ..ClassChoice::default()
                }),
            }
        ),
        [
            Choice::Slot(general_slot(1)),
            Choice::Slot(class_slot(1, 0)),
            Choice::Slot(class_slot(4, 0)),
            Choice::Slot(general_slot(5))
        ]
    );
}

#[test]
fn an_earlier_edit_can_break_a_prestige_class_entry() {
    // Rogue 3 / Wizard 3, Mobility 4 by level 4, then Arcane Trickster.
    let mut build = build(
        "race:elf",
        &[
            "class:rogue",
            "class:rogue",
            "class:rogue",
            "class:wizard",
            "class:wizard",
            "class:wizard",
            "class:arcane-trickster",
        ],
    );
    for level in &mut build.levels[..4] {
        level.skill_ranks.insert(skill("skill:mobility"), 1);
    }
    assert!(engine().validate(&build).is_empty());
    assert_eq!(
        breaks(
            &build,
            Edit::SkillRanks {
                level: 4,
                skill: skill("skill:mobility"),
                ranks: 0,
            }
        ),
        [Choice::Class { level: 7 }]
    );
}

#[test]
fn the_edited_choice_itself_is_not_reported() {
    let build = cleave_chain();
    // Great Cleave at level 1 doesn't qualify, but that's the picker's "locked" reason, not a would-break.
    assert_eq!(
        breaks(
            &build,
            Edit::Pick {
                slot: general_slot(3),
                pick: pick_feat("feat:outflank"),
            }
        ),
        []
    );
}

#[test]
fn the_picker_asks_what_the_chosen_option_would_break() {
    let build = cleave_chain();
    let options = engine().options(&build, &general_slot(1)).unwrap();
    let toughness = options
        .iter()
        .find(|o| o.pick.key() == "feat:toughness")
        .unwrap();
    assert_eq!(toughness.status, Ok(()));

    // Options carry no would-break; the picker asks for the one option the player taps.
    let would_break = |pick: &rules::Pick| {
        breaks(
            &build,
            Edit::Pick {
                slot: general_slot(1),
                pick: pick.clone(),
            },
        )
    };
    assert_eq!(would_break(&pick_feat("feat:cleave")), []);
    assert_eq!(
        would_break(&toughness.pick),
        [
            Choice::Slot(class_slot(4, 0)),
            Choice::Slot(general_slot(5))
        ]
    );
}
