//! Bonus-list slots: a class's own fixed bonus-feat list (the monk's), whose picks ignore the feat's prerequisites
//! but still count as held. Every other feat slot checks them.

use rules::{
    Ability, ArchetypeKey, Build, Choice, ClassChoice, Edit, Issue, Pick, Prerequisite, Problem,
    SlotId, SlotOption, Unmet,
};

use crate::support::*;

fn options(build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("the Build has no slot {slot:?}"))
}

fn status_of(options: &[SlotOption], pick: &Pick) -> Result<(), Issue> {
    options
        .iter()
        .find(|option| &option.pick == pick)
        .unwrap_or_else(|| panic!("{pick:?} isn't offered"))
        .status
        .clone()
}

fn needs_feat(key: &str) -> Issue {
    Issue::Unmet(Unmet::Condition {
        needs: Prerequisite::Feat(feat(key)),
        has: None,
    })
}

/// A dwarf monk with Dexterity 12, so Dodge (Dex 13) and Crane Style (Dodge) are out of reach in a general slot.
fn clumsy_monk(levels: usize) -> Build {
    let mut build = build("race:dwarf", &times("class:monk", levels));
    build.start.ability_scores.dexterity = 12;
    build
}

#[test]
fn a_bonus_list_pick_ignores_the_feats_prerequisites() {
    let build = clumsy_monk(1);
    let crane_style = pick_feat("feat:crane-style");

    // A general slot checks that Crane Style needs Dodge; the monk's bonus slot doesn't.
    assert_eq!(
        status_of(&options(&build, &general_slot(1)), &crane_style),
        Err(needs_feat("feat:dodge"))
    );
    let bonus = options(&build, &class_slot(1, 0));
    assert_eq!(status_of(&bonus, &crane_style), Ok(()));
    assert_eq!(status_of(&bonus, &pick_feat("feat:dodge")), Ok(()));

    let mut picked = build;
    picked.picks.insert(class_slot(1, 0), crane_style);
    assert_eq!(engine().validate(&picked), []);
}

#[test]
fn a_bonus_list_slot_still_takes_only_its_list_and_no_feat_already_held() {
    let mut build = clumsy_monk(2);
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("feat:crane-style"));
    let offered = options(&build, &class_slot(2, 0));
    let keys: Vec<_> = offered.iter().map(|o| o.pick.key()).collect();
    assert_eq!(keys, ["feat:crane-style", "feat:dodge"]);
    assert_eq!(
        status_of(&offered, &pick_feat("feat:crane-style")),
        Err(Issue::AlreadyHeld)
    );

    build
        .picks
        .insert(class_slot(2, 0), pick_feat("feat:toughness"));
    assert_eq!(
        engine().validate(&build),
        [Problem {
            at: Choice::Slot(class_slot(2, 0)),
            issue: Issue::NotAllowed,
        }]
    );
}

#[test]
fn a_follow_on_feat_checks_its_prerequisites_and_sees_the_bonus_pick_as_held() {
    // Crane Wing needs Crane Style and Dodge. Crane Style came from the bonus slot, so only Dodge is missing.
    let mut build = clumsy_monk(3);
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("feat:crane-style"));
    let crane_wing = pick_feat("feat:crane-wing");
    assert_eq!(
        status_of(&options(&build, &general_slot(3)), &crane_wing),
        Err(needs_feat("feat:dodge"))
    );

    // Dodge from the bonus slot at 2, without Dex 13, completes it.
    build
        .picks
        .insert(class_slot(2, 0), pick_feat("feat:dodge"));
    assert_eq!(
        status_of(&options(&build, &general_slot(3)), &crane_wing),
        Ok(())
    );
}

#[test]
fn a_fighter_combat_feat_slot_still_checks_prerequisites() {
    let mut build = build("race:dwarf", &["class:fighter"]);
    build.start.ability_scores.dexterity = 12;
    assert_eq!(
        status_of(
            &options(&build, &class_slot(1, 0)),
            &pick_feat("feat:crane-style")
        ),
        Err(needs_feat("feat:dodge"))
    );

    build
        .picks
        .insert(class_slot(1, 0), pick_feat("feat:crane-style"));
    assert_eq!(
        engine().validate(&build),
        [Problem {
            at: Choice::Slot(class_slot(1, 0)),
            issue: needs_feat("feat:dodge"),
        }]
    );
}

#[test]
fn a_fixed_list_that_is_not_a_bonus_list_still_checks_prerequisites() {
    // The Armiger's level-2 slot is a fixed list (Power Attack or Combat Expertise) that checks them.
    let mut build = build("race:dwarf", &times("class:fighter", 2));
    build.start.ability_scores.intelligence = 12;
    build.class_choices.insert(
        class("class:fighter"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("archetype:fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    assert_eq!(
        status_of(
            &options(&build, &class_slot(2, 0)),
            &pick_feat("feat:combat-expertise")
        ),
        Err(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::AbilityScore {
                ability: Ability::Intelligence,
                score: 13
            },
            has: Some(12),
        }))
    );
}

#[test]
fn would_break_never_reports_a_bonus_list_pick() {
    // Dwarf monk 3 with Dex 14: Dodge (L1), Crane Style in the bonus slot at 2, Crane Wing (L3).
    let mut build = build("race:dwarf", &times("class:monk", 3));
    build.picks.insert(general_slot(1), pick_feat("feat:dodge"));
    build
        .picks
        .insert(class_slot(2, 0), pick_feat("feat:crane-style"));
    build
        .picks
        .insert(general_slot(3), pick_feat("feat:crane-wing"));
    assert_eq!(engine().validate(&build), []);

    // Removing Dodge breaks Crane Wing, which needs it, but not Crane Style in the bonus slot.
    let broken: Vec<Choice> = engine()
        .would_break(&build, &Edit::Clear(general_slot(1)))
        .into_iter()
        .map(|problem| problem.at)
        .collect();
    assert_eq!(broken, [Choice::Slot(general_slot(3))]);
}
