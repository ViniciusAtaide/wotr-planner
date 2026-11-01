//! Parameterised and repeatable feats and options: a pick names what a parameterised feat is taken for, a
//! prerequisite can need a feat for one value or for the pick's own value, and a feat or option repeats as far as
//! its data says.

use std::collections::BTreeMap;

use rules::{
    Build, Choice, Issue, Parameter, Pick, Prerequisite, SlotId, SlotOption, SpellSchool, Unmet,
};

use crate::support::*;

fn spell_school(school: SpellSchool) -> Parameter {
    Parameter::SpellSchool(school)
}

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

/// The values a feat is held for at the end of a level, with how many picks and grants hold each.
fn values_at(build: &Build, level: u8, key: &str) -> BTreeMap<Parameter, u8> {
    engine()
        .evaluate(build)
        .at(level)
        .unwrap()
        .feat_values
        .get(&feat(key))
        .cloned()
        .unwrap_or_default()
}

// ---- Weapon Focus: once per weapon ------------------------------------------------------------------------

#[test]
fn weapon_focus_can_be_taken_for_two_weapons() {
    let mut build = build("elf", &["fighter"]);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build.picks.insert(
        general_slot(1),
        pick_feat_for("weapon-focus", weapon("dagger")),
    );
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 1, "weapon-focus"),
        BTreeMap::from([(weapon("dagger"), 1), (weapon("longsword"), 1)])
    );
    // Held twice, so a prerequisite naming the feat sees it.
    let state = engine().evaluate(&build).at(1).unwrap().clone();
    assert_eq!(state.feat_count(&feat("weapon-focus")), 2);
}

#[test]
fn weapon_focus_cant_be_taken_twice_for_one_weapon() {
    let mut build = build("elf", &["fighter"]);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build.picks.insert(
        general_slot(1),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    assert_eq!(why_not(&build, &general_slot(1)), Some(Issue::AlreadyHeld));
    assert_eq!(
        values_at(&build, 1, "weapon-focus"),
        BTreeMap::from([(weapon("longsword"), 1)])
    );
}

#[test]
fn weapon_specialization_checks_the_matching_weapon() {
    // Fighter 4: Weapon Focus (Longsword) at 1, Weapon Specialization at 4.
    let mut build = build("elf", &times("fighter", 4));
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    let offered = options(&build, &class_slot(4, 0));
    assert_eq!(
        status_of(
            &offered,
            &pick_feat_for("weapon-specialization", weapon("longsword"))
        ),
        Ok(())
    );
    assert_eq!(
        status_of(
            &offered,
            &pick_feat_for("weapon-specialization", weapon("dagger"))
        ),
        Err(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::FeatForSameValue(feat("weapon-focus")),
            has: None,
        }))
    );

    build.picks.insert(
        class_slot(4, 0),
        pick_feat_for("weapon-specialization", weapon("dagger")),
    );
    assert_eq!(
        why_not(&build, &class_slot(4, 0)),
        Some(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::FeatForSameValue(feat("weapon-focus")),
            has: None,
        }))
    );
    build.picks.insert(
        class_slot(4, 0),
        pick_feat_for("weapon-specialization", weapon("longsword")),
    );
    assert!(engine().is_possible(&build));
}

#[test]
fn weapon_focus_stays_while_the_specialization_in_its_weapon_needs_it() {
    let mut build = build("elf", &times("fighter", 4));
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build.picks.insert(
        class_slot(4, 0),
        pick_feat_for("weapon-specialization", weapon("longsword")),
    );
    let removable = engine().removable(&build);
    assert!(!removable.contains(&Choice::Slot(class_slot(1, 0))));
    assert!(removable.contains(&Choice::Slot(class_slot(4, 0))));
    // Refocusing on another weapon is locked too; the same weapon stays open.
    let options = options(&build, &class_slot(1, 0));
    assert_eq!(
        status_of(&options, &pick_feat_for("weapon-focus", weapon("dagger"))),
        Err(Issue::BreaksLater)
    );
    assert_eq!(
        status_of(
            &options,
            &pick_feat_for("weapon-focus", weapon("longsword"))
        ),
        Ok(())
    );
}

#[test]
fn a_prerequisite_can_need_a_feat_for_one_value() {
    // Aldori Dueling Mastery: Weapon Focus (Dueling Sword).
    let needs_dueling_sword = Err(Issue::Unmet(Unmet::Condition {
        needs: Prerequisite::FeatFor {
            feat: feat("weapon-focus"),
            value: weapon("dueling-sword"),
        },
        has: None,
    }));
    let mut build = build("elf", &["fighter"]);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    assert_eq!(
        status_of(
            &options(&build, &general_slot(1)),
            &pick_feat("aldori-dueling-mastery")
        ),
        needs_dueling_sword
    );
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("dueling-sword")),
    );
    assert_eq!(
        status_of(
            &options(&build, &general_slot(1)),
            &pick_feat("aldori-dueling-mastery")
        ),
        Ok(())
    );
    // "Weapon Focus" on its own is any weapon.
    assert_eq!(
        rules::CharacterState::default().check(&Prerequisite::Feat(feat("weapon-focus"))),
        Err(Unmet::Condition {
            needs: Prerequisite::Feat(feat("weapon-focus")),
            has: None,
        })
    );
}

// ---- values ---------------------------------------------------------------------------------------------------

#[test]
fn a_parameterised_feat_needs_a_value_of_its_kind() {
    // Each of these alone makes the Build impossible, and no picker offers it.
    let bad = [
        // No value.
        (class_slot(1, 0), pick_feat("weapon-focus")),
        // A value of another kind.
        (
            general_slot(1),
            pick_feat_for("skill-focus", weapon("longsword")),
        ),
        // Exotic Weapon Proficiency takes only an exotic weapon.
        (
            class_slot(1, 0),
            pick_feat_for("exotic-weapon-proficiency", weapon("longsword")),
        ),
        // A feat that takes no value takes none.
        (
            general_slot(1),
            pick_feat_for("toughness", weapon("longsword")),
        ),
        // A weapon or skill the data lacks.
        (
            class_slot(1, 0),
            pick_feat_for("weapon-focus", weapon("spork")),
        ),
        (
            general_slot(1),
            pick_feat_for("skill-focus", skill_value("juggling")),
        ),
    ];
    for (slot, pick) in bad {
        let mut build = build("elf", &["fighter"]);
        build.picks.insert(slot.clone(), pick.clone());
        assert_eq!(why_not(&build, &slot), None, "{pick:?}");
    }
}

#[test]
fn a_parameterised_feat_is_offered_once_per_value() {
    let mut build = build("elf", &["fighter"]);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    let offered = options(&build, &general_slot(1));
    let values = |key: &str| -> Vec<Option<Parameter>> {
        offered
            .iter()
            .filter(|option| option.pick.key() == key)
            .map(|option| option.pick.parameter().cloned())
            .collect()
    };
    // Weapons in key order, exotic ones only for Exotic Weapon Proficiency.
    assert_eq!(
        values("weapon-focus"),
        [
            "bastard-sword",
            "dagger",
            "dueling-sword",
            "longsword",
            "unarmed-strike"
        ]
        .map(|key| Some(weapon(key)))
    );
    assert_eq!(
        values("exotic-weapon-proficiency"),
        ["bastard-sword", "dueling-sword"].map(|key| Some(weapon(key)))
    );
    assert_eq!(
        values("skill-focus"),
        ["athletics", "knowledge-arcana", "mobility", "persuasion"]
            .map(|key| Some(skill_value(key)))
    );
    assert_eq!(
        values("spell-focus"),
        SpellSchool::ALL.map(|school| Some(spell_school(school)))
    );
    assert_eq!(values("toughness"), [None]);

    // Each value has its own status.
    assert_eq!(
        status_of(
            &offered,
            &pick_feat_for("weapon-focus", weapon("longsword"))
        ),
        Err(Issue::AlreadyHeld)
    );
    assert_eq!(
        status_of(&offered, &pick_feat_for("weapon-focus", weapon("dagger"))),
        Ok(())
    );
}

#[test]
fn a_parameterised_feat_taken_once_is_held_for_one_value() {
    // Spell Focus: the guide doesn't say it repeats.
    let mut build = build("elf", &["wizard", "wizard", "wizard"]);
    build.picks.insert(
        general_slot(1),
        pick_feat_for("spell-focus", spell_school(SpellSchool::Evocation)),
    );
    build.picks.insert(
        general_slot(3),
        pick_feat_for("spell-focus", spell_school(SpellSchool::Conjuration)),
    );
    assert_eq!(why_not(&build, &general_slot(3)), Some(Issue::AlreadyHeld));

    // Augment Summoning needs Spell Focus (Conjuration), not any school.
    build
        .picks
        .insert(general_slot(3), pick_feat("augment-summoning"));
    assert_eq!(
        why_not(&build, &general_slot(3)),
        Some(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::FeatFor {
                feat: feat("spell-focus"),
                value: spell_school(SpellSchool::Conjuration),
            },
            has: None,
        }))
    );
    build.picks.insert(
        general_slot(1),
        pick_feat_for("spell-focus", spell_school(SpellSchool::Conjuration)),
    );
    assert!(engine().is_possible(&build));
}

// ---- repeats --------------------------------------------------------------------------------------------------

#[test]
fn a_feat_repeats_up_to_its_number_of_times() {
    // Extra Rogue Talent: up to three times. General slots at 1, 3, 5 and 7.
    let mut build = build("elf", &times("wizard", 7));
    for level in [1, 3, 5, 7] {
        build
            .picks
            .insert(general_slot(level), pick_feat("extra-rogue-talent"));
    }
    assert_eq!(
        why_not(&build, &general_slot(7)),
        Some(Issue::MaxTimes { times: 3 })
    );
    let state = engine().evaluate(&build).at(7).unwrap().clone();
    assert_eq!(state.feat_count(&feat("extra-rogue-talent")), 3);

    // Extra Ki: any number of times.
    for level in [1, 3, 5, 7] {
        build
            .picks
            .insert(general_slot(level), pick_feat("extra-ki"));
    }
    assert!(engine().is_possible(&build));
    let state = engine().evaluate(&build).at(7).unwrap().clone();
    assert_eq!(state.feat_count(&feat("extra-ki")), 4);

    // A feat that doesn't repeat still can't be taken twice.
    build.picks.insert(general_slot(1), pick_feat("toughness"));
    build.picks.insert(general_slot(3), pick_feat("toughness"));
    assert_eq!(why_not(&build, &general_slot(3)), Some(Issue::AlreadyHeld));
}

#[test]
fn an_option_repeats_up_to_its_number_of_times() {
    // Resiliency: up to twice. Rogue talents at 2, 4 and 6.
    let mut build = build("elf", &times("rogue", 6));
    for level in [2, 4, 6] {
        build.picks.insert(
            selection_slot(level, 0),
            pick_option("rogue/rogue-talents/resiliency"),
        );
    }
    assert_eq!(
        why_not(&build, &selection_slot(6, 0)),
        Some(Issue::MaxTimes { times: 2 })
    );
    let state = engine().evaluate(&build).at(6).unwrap().clone();
    assert_eq!(
        state.option_count(&option("rogue/rogue-talents/resiliency")),
        2
    );
    assert_eq!(
        status_of(
            &options(&build, &selection_slot(6, 0)),
            &pick_option("rogue/rogue-talents/resiliency")
        ),
        Err(Issue::MaxTimes { times: 2 })
    );
}

// ---- grants ---------------------------------------------------------------------------------------------------

#[test]
fn a_one_feat_bonus_list_picks_the_value() {
    // The rogue's Skill Focus at 3, like the Master of All's: a bonus slot of one parameterised feat.
    let mut build = build("elf", &times("rogue", 3));
    build.picks.insert(
        class_slot(3, 0),
        pick_feat_for("skill-focus", skill_value("mobility")),
    );
    build
        .picks
        .insert(general_slot(3), pick_feat("acrobatic-spellcaster"));
    assert!(engine().is_possible(&build));

    // A second Skill Focus in another skill from a general slot.
    build.picks.insert(
        general_slot(1),
        pick_feat_for("skill-focus", skill_value("persuasion")),
    );
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 3, "skill-focus"),
        BTreeMap::from([(skill_value("mobility"), 1), (skill_value("persuasion"), 1)])
    );

    // Its list holds only Skill Focus, and it takes a value.
    let offered = options(&build, &class_slot(3, 0));
    assert!(
        offered
            .iter()
            .all(|option| option.pick.key() == "skill-focus")
    );
    assert_eq!(offered.len(), 4);
    assert!(
        offered
            .iter()
            .all(|option| option.pick.parameter().is_some())
    );
    build
        .picks
        .insert(class_slot(3, 0), pick_feat("skill-focus"));
    assert!(!engine().is_possible(&build));
}

#[test]
fn an_option_with_a_chosen_feat_holds_it_for_the_picks_value() {
    // The rogue talent Weapon Focus, at 2.
    let mut build = build("elf", &times("rogue", 2));
    build.picks.insert(
        selection_slot(2, 0),
        pick_option_for("rogue/rogue-talents/weapon-focus", weapon("dagger")),
    );
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 2, "weapon-focus"),
        BTreeMap::from([(weapon("dagger"), 1)])
    );

    // It takes a value of its feat's kind.
    build.picks.insert(
        selection_slot(2, 0),
        pick_option("rogue/rogue-talents/weapon-focus"),
    );
    assert_eq!(why_not(&build, &selection_slot(2, 0)), None);
    build.picks.insert(
        selection_slot(2, 0),
        pick_option_for("rogue/rogue-talents/weapon-focus", skill_value("mobility")),
    );
    assert_eq!(why_not(&build, &selection_slot(2, 0)), None);
    // An option without a chosen feat takes none.
    build.picks.insert(
        selection_slot(2, 0),
        pick_option_for("rogue/rogue-talents/fast-stealth", weapon("dagger")),
    );
    assert_eq!(why_not(&build, &selection_slot(2, 0)), None);

    // Weapon Focus (Dagger) taken at 1 already (from a fighter level: a rogue 1 has BAB 0), so the talent can't
    // give it again.
    let mut build = crate::support::build("elf", &["fighter", "rogue", "rogue"]);
    build.picks.insert(
        general_slot(1),
        pick_feat_for("weapon-focus", weapon("dagger")),
    );
    build.picks.insert(
        selection_slot(3, 0),
        pick_option_for("rogue/rogue-talents/weapon-focus", weapon("dagger")),
    );
    assert_eq!(
        why_not(&build, &selection_slot(3, 0)),
        Some(Issue::AlreadyHeld)
    );
    // The talent's picker offers it once per weapon.
    let offered = options(&build, &selection_slot(3, 0));
    assert_eq!(
        status_of(
            &offered,
            &pick_option_for("rogue/rogue-talents/weapon-focus", weapon("longsword"))
        ),
        Ok(())
    );
    assert_eq!(
        offered
            .iter()
            .filter(|option| option.pick.key() == "rogue/rogue-talents/weapon-focus")
            .count(),
        5
    );
}

/// Fighter 17 on the Trickster path; the Trickster's tricks at MR 3 (level 9).
fn trickster_fighter() -> Build {
    let mut build = build("elf", &times("fighter", 17));
    build.mythic_path = Some(path("trickster"));
    build
}

#[test]
fn a_grant_with_no_pick_holds_a_parameterised_feat_for_every_value() {
    // Like All-skilled: Exotic Weapon Proficiency in every exotic weapon.
    let mut build = trickster_fighter();
    build.picks.insert(
        SlotId::PathSelection { rank: 3, index: 0 },
        pick_option("trickster/tricks/all-weapons"),
    );
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 9, "exotic-weapon-proficiency"),
        BTreeMap::from([(weapon("bastard-sword"), 1), (weapon("dueling-sword"), 1)])
    );
    // So a pick of it for an exotic weapon is already held.
    build.picks.insert(
        general_slot(11),
        pick_feat_for("exotic-weapon-proficiency", weapon("dueling-sword")),
    );
    assert_eq!(why_not(&build, &general_slot(11)), Some(Issue::AlreadyHeld));
}

#[test]
fn a_switch_that_drops_an_every_value_grant_keeps_a_value_a_pick_holds() {
    // Exotic Weapon Proficiency (Bastard Sword) picked at 1, then every exotic weapon from the MR 3 trick; the Gold
    // Dragon keeps only MR 1-2 of the old path's grants.
    let mut build = trickster_fighter();
    build.picks.insert(
        general_slot(1),
        pick_feat_for("exotic-weapon-proficiency", weapon("bastard-sword")),
    );
    build.picks.insert(
        SlotId::PathSelection { rank: 3, index: 0 },
        pick_option("trickster/tricks/all-weapons"),
    );
    build.path_switch = Some(path("gold-dragon"));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 16, "exotic-weapon-proficiency"),
        BTreeMap::from([(weapon("bastard-sword"), 2), (weapon("dueling-sword"), 1)])
    );
    assert_eq!(
        values_at(&build, 17, "exotic-weapon-proficiency"),
        BTreeMap::from([(weapon("bastard-sword"), 1)])
    );
}
