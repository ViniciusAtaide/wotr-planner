//! Talents that repeat (ticket 47).

use std::collections::{BTreeSet, HashMap};

use rules::{
    Build, ClassKey, FeatKey, FeatSlot, Issue, LevelUp, Parameter, Pick, Repeat,
    SelectionOptionKey, SlotId, SlotKind, SlotOption,
};

use crate::support::*;

/// A pick of an option for a value (Skill Focus (Perception)).
fn pick_for(key: &str, value: Parameter) -> Pick {
    Pick::option_for(SelectionOptionKey::new(key), value)
}

/// The status of each pick a slot offers, by pick (an option once for each value it may name).
fn statuses(build: &Build, slot: &SlotId) -> HashMap<Pick, Result<(), Issue>> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .map(|SlotOption { pick, status, .. }| (pick, status))
        .collect()
}

/// A feat slot an option's row grants: the first one of its level.
fn option_feat_slot(level: u8) -> SlotId {
    SlotId::OptionFeat { level, index: 0 }
}

/// The Skill Focus wild talent and the rogue talent Skill Focus repeat once per skill (user, 2026-11-02): the option
/// takes the skill of its chosen feat, Skill Focus, once for each.
#[test]
fn the_skill_focus_talents_repeat_once_per_skill_and_weapon_focus_doesnt() {
    for key in [talent("skill-focus"), rogue_talent("skill-focus")] {
        let option = selection_option(&key);
        assert_eq!(
            option.chosen_feat,
            Some(FeatKey::new("skill-focus")),
            "{key}"
        );
        assert_eq!(option.repeat, Repeat::PerValue, "{key}");
    }
    let weapon_focus = selection_option(&rogue_talent("weapon-focus"));
    assert_eq!(weapon_focus.chosen_feat, Some(FeatKey::new("weapon-focus")));
    assert_eq!(weapon_focus.repeat, Repeat::Once);
}

/// A kineticist takes Skill Focus (Perception) at 2 and Skill Focus (Stealth) at 4; Skill Focus (Perception) again at
/// 6 is refused, another skill is open.
#[test]
fn a_kineticist_takes_the_skill_focus_talent_for_two_skills_but_not_one_twice() {
    let focus = talent("skill-focus");
    let build = with_picks(
        kineticist("earth", 6),
        &[
            (2, pick_for(&focus, skill("perception"))),
            (4, pick_for(&focus, skill("stealth"))),
        ],
    );
    assert!(possible(&build));
    let state = at(&build, 4);
    for value in ["perception", "stealth"] {
        assert!(state.has_feat_for(&FeatKey::new("skill-focus"), &skill(value)));
    }
    let at_6 = statuses(&build, &selection_slot(6));
    assert_eq!(
        at_6[&pick_for(&focus, skill("perception"))],
        Err(Issue::AlreadyHeld)
    );
    assert_eq!(
        at_6[&pick_for(&focus, skill("stealth"))],
        Err(Issue::AlreadyHeld)
    );
    assert_eq!(at_6[&pick_for(&focus, skill("athletics"))], Ok(()));
    let again = with_picks(build, &[(6, pick_for(&focus, skill("perception")))]);
    assert_eq!(
        why_not(&again, &selection_slot(6)),
        Some(Issue::AlreadyHeld)
    );
}

/// A rogue takes the Skill Focus talent for two different skills and is refused a second one for the same skill; a
/// slayer, who shares the rogue talents, too.
#[test]
fn a_rogue_and_a_slayer_take_the_skill_focus_talent_for_two_skills_but_not_one_twice() {
    let focus = rogue_talent("skill-focus");
    for class in ["rogue", "slayer"] {
        let build = with_picks(
            build(class, 6),
            &[
                (2, pick_for(&focus, skill("perception"))),
                (4, pick_for(&focus, skill("stealth"))),
            ],
        );
        assert!(possible(&build), "{class}");
        assert_eq!(
            statuses(&build, &selection_slot(6))[&pick_for(&focus, skill("trickery"))],
            Ok(()),
            "{class}"
        );
        let again = with_picks(build, &[(6, pick_for(&focus, skill("stealth")))]);
        assert_eq!(
            why_not(&again, &selection_slot(6)),
            Some(Issue::AlreadyHeld),
            "{class}"
        );
    }
}

/// The rogue talent Weapon Focus stays taken once (user, 2026-11-02), whatever the weapon.
#[test]
fn the_rogue_weapon_focus_talent_cant_be_taken_twice() {
    let focus = rogue_talent("weapon-focus");
    let build = with_picks(
        build("rogue", 4),
        &[
            (2, pick_for(&focus, weapon("longsword"))),
            (4, pick_for(&focus, weapon("dagger"))),
        ],
    );
    assert_eq!(
        why_not(&build, &selection_slot(4)),
        Some(Issue::AlreadyHeld)
    );
}

/// "Combat Trick: Any combat feat that the character qualifies for." Taken any number of times, each time a bonus
/// Combat feat slot whose feat's prerequisites are checked (user, 2026-11-02).
#[test]
fn combat_trick_repeats_and_grants_a_combat_feat_slot_each_time() {
    let trick = selection_option(&rogue_talent("combat-trick"));
    assert_eq!(trick.repeat, Repeat::Unlimited);
    assert_eq!(
        trick
            .levels
            .iter()
            .map(|row| (row.level, row.feat_slots.clone()))
            .collect::<Vec<_>>(),
        [(1, vec![FeatSlot::CombatFeat])]
    );
    let engine = engine();
    let build = with_picks(
        build("rogue", 4),
        &[
            (2, pick(&rogue_talent("combat-trick"))),
            (4, pick(&rogue_talent("combat-trick"))),
        ],
    );
    assert!(possible(&build));
    let sheet = engine.evaluate(&build);
    for level in [2, 4] {
        let slot = sheet
            .slot(&option_feat_slot(level))
            .expect("a Combat Trick slot");
        assert_eq!(slot.kind, SlotKind::Feat(FeatSlot::CombatFeat));
        assert_eq!(engine.slot_label(slot), "Combat feat");
    }
    // Its feat's prerequisites are checked; a feat that isn't a Combat feat isn't offered.
    let offered = feats_offered(&build, &option_feat_slot(2));
    assert!(offered.contains("weapon-focus"));
    assert!(!offered.contains("skill-focus"));
    let cleave = feat_status(&build, &option_feat_slot(2), "cleave");
    assert_eq!(
        engine.issue_text(&cleave.unwrap_err()).as_deref(),
        Some("Needs Power Attack")
    );
}

/// A rogue takes Combat Trick twice and Weapon Focus (Longsword) in one slot, Weapon Focus (Dagger) in the other;
/// Weapon Focus (Longsword) twice is refused (the feat's own once per weapon). The link holds it all.
#[test]
fn a_rogue_takes_weapon_focus_in_two_weapons_through_two_combat_tricks() {
    let engine = engine();
    let focus = |value: &str| Pick::feat_for(FeatKey::new("weapon-focus"), weapon(value));
    let mut build = with_picks(
        build("rogue", 4),
        &[
            (2, pick(&rogue_talent("combat-trick"))),
            (4, pick(&rogue_talent("combat-trick"))),
        ],
    );
    build.picks.insert(option_feat_slot(2), focus("longsword"));
    build.picks.insert(option_feat_slot(4), focus("dagger"));
    assert!(possible(&build));
    let state = at(&build, 4);
    for value in ["longsword", "dagger"] {
        assert!(state.has_feat_for(&FeatKey::new("weapon-focus"), &weapon(value)));
    }
    assert_eq!(
        engine.open_link(&rules::encode_build(&build)),
        Ok(build.clone())
    );
    assert_eq!(
        statuses(&build, &option_feat_slot(4))[&focus("longsword")],
        Err(Issue::AlreadyHeld)
    );
    let mut twice = build.clone();
    twice.picks.insert(option_feat_slot(4), focus("longsword"));
    assert_eq!(
        why_not(&twice, &option_feat_slot(4)),
        Some(Issue::AlreadyHeld)
    );
    // The rogue talent Weapon Focus beside them: another weapon is open, Longsword isn't.
    let mut rogue_6 = build.clone();
    rogue_6.levels.extend([
        LevelUp::new(ClassKey::new("rogue")),
        LevelUp::new(ClassKey::new("rogue")),
    ]);
    let at_6 = statuses(&rogue_6, &selection_slot(6));
    assert_eq!(
        at_6[&pick_for(&rogue_talent("weapon-focus"), weapon("longsword"))],
        Err(Issue::AlreadyHeld)
    );
    assert_eq!(
        at_6[&pick_for(&rogue_talent("weapon-focus"), weapon("rapier"))],
        Ok(())
    );
}

/// Combat Trick grants its slot through every selection that offers it: the slayer's talents, the Sylvan Trickster's
/// Fey Tricks and the Underground Chemist's Discovery (from rogue 10).
#[test]
fn combat_trick_grants_its_slot_through_every_selection_offering_it() {
    let offering: BTreeSet<String> = DATA
        .classes
        .iter()
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .filter(|selection| {
            selection.offers(&SelectionOptionKey::new(rogue_talent("combat-trick")))
        })
        .map(|selection| selection.key.to_string())
        .collect();
    assert_eq!(
        offering,
        BTreeSet::from(
            [
                "alchemist/vivisectionist/medical-discovery",
                "rogue/rogue-talents",
                "rogue/sylvan-trickster/fey-tricks",
                "rogue/underground-chemist/discovery",
                "shifter/feyform-shifter/rogue-talent",
                "slayer/slayer-talents",
            ]
            .map(String::from)
        )
    );
    let trick = pick(&rogue_talent("combat-trick"));
    for (build, level) in [
        (build("slayer", 2), 2),
        (archetype_build("rogue", "rogue/sylvan-trickster", 2), 2),
        (
            archetype_build("rogue", "rogue/underground-chemist", 10),
            10,
        ),
        // Ticket 60: through the rogue's and the slayer's selections on another class's row, and the archetypes' own.
        (build("bard", 2), 2),
        (build("skald", 2), 2),
        (
            chaotic(archetype_build("shifter", "shifter/feyform-shifter", 8)),
            8,
        ),
        (
            archetype_build("alchemist", "alchemist/vivisectionist", 2),
            2,
        ),
        (
            archetype_build("inquisitor", "inquisitor/sanctified-slayer", 8),
            8,
        ),
    ] {
        let build = with_picks(build, &[(level, trick.clone())]);
        assert!(possible(&build), "{:?}", build.class_choices);
        assert_eq!(
            engine()
                .evaluate(&build)
                .slot(&option_feat_slot(level))
                .map(|slot| slot.kind.clone()),
            Some(SlotKind::Feat(FeatSlot::CombatFeat)),
            "{:?}",
            build.class_choices
        );
    }
}
