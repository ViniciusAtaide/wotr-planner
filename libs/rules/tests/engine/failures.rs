//! `Engine::failures`: every choice of a Build that fails, with why, in walk order (ticket 133). One test per kind of
//! place the walk fails a choice; whether a Build is possible is `possible.rs`'s.

use std::collections::BTreeSet;

use rules::{
    Alignment, Build, Choice, ClassChoice, Failure, Faith, Issue, Prerequisite, SlotId, Unmet,
};

use crate::support::*;

/// The Build's failures, checked against `is_possible` on the way: none exactly when it's possible.
fn failures(build: &Build) -> Vec<Failure> {
    let failures = engine().failures(build);
    assert_eq!(
        failures.is_empty(),
        engine().is_possible(build),
        "{failures:?}"
    );
    failures
}

/// The failures as (choice, issue) pairs.
fn pairs(build: &Build) -> Vec<(Choice, Issue)> {
    failures(build)
        .into_iter()
        .map(|failure| (failure.choice, failure.issue))
        .collect()
}

fn slot(id: SlotId) -> Choice {
    Choice::Slot(id)
}

#[test]
fn a_possible_build_has_none() {
    let mut fighter = build("human", &["fighter", "fighter", "fighter"]);
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    fighter.picks.insert(general_slot(1), pick_feat("cleave"));
    assert!(failures(&fighter).is_empty());
    assert!(failures(&Build::default()).is_empty());
}

#[test]
fn a_refused_pick_gives_its_issue_and_what_needs_it_fails_after_it() {
    // Strength 12: Power Attack (Str 13) is refused at level 1, so Cleave, which needs it, fails too.
    let mut fighter = build("human", &["fighter", "fighter", "fighter"]);
    fighter.start.ability_scores = point_buy([12, 14, 14, 13, 10, 8]);
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    fighter.picks.insert(general_slot(3), pick_feat("cleave"));
    let failures = failures(&fighter);
    assert_eq!(failures.len(), 2, "{failures:?}");
    assert_eq!(failures[0].choice, slot(class_slot(1, 0)));
    let Issue::Unmet(Unmet::Condition { needs, has }) = &failures[0].issue else {
        panic!("Power Attack's Strength: {:?}", failures[0].issue);
    };
    assert_eq!(
        (needs, *has),
        (
            &Prerequisite::AbilityScore {
                ability: rules::Ability::Strength,
                score: 13
            },
            Some(12)
        )
    );
    assert_eq!(failures[1].choice, slot(general_slot(3)));
    assert_eq!(
        failures[1].issue,
        Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::Feat(feat("power-attack")),
            has: None,
        })
    );
}

#[test]
fn a_pick_held_twice_or_outside_its_slot_says_so() {
    let mut fighter = build("human", &["fighter", "fighter", "fighter"]);
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    fighter
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    fighter
        .picks
        .insert(general_slot(3), pick_feat("no-such-feat"));
    assert_eq!(
        pairs(&fighter),
        [
            (slot(general_slot(1)), Issue::AlreadyHeld),
            (slot(general_slot(3)), Issue::UnknownKey),
        ]
    );
}

#[test]
fn a_key_the_data_lacks_is_unknown_and_one_out_of_place_is_not_allowed() {
    // In a slot: a feat the data lacks, and one the data has outside the slot's list (the fighter's level-1 class
    // slot takes combat feats, not Scribe Scrolls).
    let mut fighter = build("human", &["fighter"]);
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("scribe-scrolls"));
    fighter
        .picks
        .insert(general_slot(1), pick_feat("no-such-feat"));
    assert_eq!(
        pairs(&fighter),
        [
            (slot(class_slot(1, 0)), Issue::NotAllowed),
            (slot(general_slot(1)), Issue::UnknownKey),
        ]
    );

    // An archetype the data lacks, and a subrace of another race.
    let mut elf = build("human", &["fighter"]);
    elf.start.subrace = Some(rules::SubraceKey::new("elf/blightborn"));
    elf.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("fighter/no-such-archetype")),
            advances: BTreeSet::new(),
        },
    );
    assert_eq!(
        pairs(&elf),
        [
            (Choice::Subrace, Issue::NotAllowed),
            (
                Choice::Archetype {
                    class: class("fighter")
                },
                Issue::UnknownKey
            ),
        ]
    );

    // An advancement naming a class the data lacks.
    let mut theurge = build(
        "elf",
        &[
            "wizard",
            "wizard",
            "wizard",
            "cleric",
            "cleric",
            "cleric",
            "mystic-theurge",
        ],
    );
    theurge.class_choices.insert(
        class("mystic-theurge"),
        ClassChoice {
            archetype: None,
            advances: BTreeSet::from([class("wizard"), class("no-such-class")]),
        },
    );
    assert_eq!(
        pairs(&theurge),
        [(
            Choice::Advances {
                class: class("mystic-theurge")
            },
            Issue::UnknownKey
        )]
    );
}

#[test]
fn an_archetype_that_fails_its_own_prerequisites_is_unmet() {
    // The Eldritch Scoundrel is for humans and elves: a dwarf rogue taking it fails the archetype, not the class.
    let mut dwarf = build("dwarf", &["rogue"]);
    dwarf.class_choices.insert(
        class("rogue"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("rogue/eldritch-scoundrel")),
            advances: BTreeSet::new(),
        },
    );
    let failures = failures(&dwarf);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(
        failures[0].choice,
        Choice::Archetype {
            class: class("rogue")
        }
    );
    let Issue::Unmet(unmet) = &failures[0].issue else {
        panic!("the archetype's prerequisites: {:?}", failures[0].issue);
    };
    assert!(
        conditions(unmet)
            .iter()
            .any(|(needs, _)| matches!(needs, Prerequisite::Race(_))),
        "{unmet:?}"
    );
}

#[test]
fn the_build_start_fails_with_the_pickers_issues() {
    // An unknown race, subrace and background.
    let mut start = build("no-such-race", &["fighter"]);
    start.start.subrace = Some(rules::SubraceKey::new("elf/no-such-subrace"));
    start.start.background = Some(rules::BackgroundKey::new("no-such-background"));
    assert_eq!(
        pairs(&start),
        [
            (Choice::Race, Issue::UnknownKey),
            (Choice::Subrace, Issue::UnknownKey),
            (Choice::Background, Issue::UnknownKey),
        ]
    );

    // A deity that doesn't allow the alignment; an alignment with no faith; a deity the data lacks.
    let mut misfit = build("human", &["fighter"]);
    misfit.start.deity = Some(Faith::Deity(rules::DeityKey::new("lawgiver")));
    misfit.start.alignment = Some(Alignment::ChaoticEvil);
    assert_eq!(
        pairs(&misfit),
        [(
            Choice::Deity,
            Issue::DeityAlignment {
                deity: rules::DeityKey::new("lawgiver")
            }
        )]
    );
    misfit.start.deity = None;
    assert_eq!(pairs(&misfit), [(Choice::Alignment, Issue::NeedsFaith)]);
    misfit.start.deity = Some(Faith::Deity(rules::DeityKey::new("no-such-deity")));
    assert_eq!(pairs(&misfit), [(Choice::Deity, Issue::UnknownKey)]);
}

#[test]
fn a_class_fails_its_entry_with_what_it_needs_and_an_unknown_one_is_unknown() {
    // Arcane Trickster straight away: its entry prerequisites, unmet.
    let trickster = build("elf", &["arcane-trickster"]);
    let failures = failures(&trickster);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].choice, Choice::Class { level: 1 });
    assert!(
        matches!(failures[0].issue, Issue::Unmet(_)),
        "{:?}",
        failures[0].issue
    );

    // A class the data lacks gives no slots: the pick in its class slot isn't reached.
    let mut unknown = build("human", &["no-such-class"]);
    unknown
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    assert_eq!(
        pairs(&unknown),
        [
            (Choice::Class { level: 1 }, Issue::UnknownKey),
            (slot(class_slot(1, 0)), Issue::NotReached),
        ]
    );
}

#[test]
fn an_archetype_of_another_class_and_a_misfit_advancement_are_not_allowed() {
    let mut fighter = build("human", &["fighter"]);
    fighter.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("rogue/eldritch-scoundrel")),
            advances: BTreeSet::new(),
        },
    );
    assert_eq!(
        pairs(&fighter),
        [(
            Choice::Archetype {
                class: class("fighter")
            },
            Issue::NotAllowed
        )]
    );

    // A Mystic Theurge advancing a class the Build doesn't take.
    let mut theurge = build(
        "elf",
        &[
            "wizard",
            "wizard",
            "wizard",
            "cleric",
            "cleric",
            "cleric",
            "mystic-theurge",
        ],
    );
    theurge.class_choices.insert(
        class("mystic-theurge"),
        ClassChoice {
            archetype: None,
            advances: BTreeSet::from([class("rogue")]),
        },
    );
    assert_eq!(
        pairs(&theurge),
        [(
            Choice::Advances {
                class: class("mystic-theurge")
            },
            Issue::NotAllowed
        )]
    );
}

#[test]
fn a_level_past_the_cap_is_not_reached_and_its_class_level_past_the_table() {
    let fighter = build("human", &times("fighter", 21));
    assert_eq!(
        pairs(&fighter),
        [
            (Choice::CharacterLevel { level: 21 }, Issue::NotReached),
            (
                Choice::Class { level: 21 },
                Issue::ClassLevelCap { cap: 20 }
            ),
        ]
    );
}

#[test]
fn unknown_skills_are_unknown_and_overspent_points_not_allowed() {
    // Wizard: 2 + Int 13 (+1) = 3 points; four spent, one in a skill the data lacks.
    let mut wizard = build("dwarf", &["wizard"]);
    for key in ["athletics", "mobility", "persuasion", "no-such-skill"] {
        wizard.levels[0].skill_ranks.insert(skill(key), 1);
    }
    assert_eq!(
        pairs(&wizard),
        [
            (
                Choice::SkillRanks {
                    level: 1,
                    skill: skill("no-such-skill")
                },
                Issue::UnknownKey
            ),
            (Choice::SkillPoints { level: 1 }, Issue::NotAllowed),
        ]
    );
}

#[test]
fn what_the_build_never_reaches_is_not_reached() {
    // A pick past the last level, a class choice for a class no level takes, and a path before MR 3.
    let mut fighter = build("human", &["fighter"]);
    fighter.picks.insert(general_slot(3), pick_feat("cleave"));
    fighter
        .class_choices
        .insert(class("rogue"), ClassChoice::default());
    fighter.first_ascension = Some(path("trickster"));
    fighter.mythic_path = Some(path("trickster"));
    assert_eq!(
        pairs(&fighter),
        [
            (slot(general_slot(3)), Issue::NotReached),
            (
                Choice::Archetype {
                    class: class("rogue")
                },
                Issue::NotReached
            ),
            (Choice::FirstAscension, Issue::NotReached),
            (Choice::MythicPath, Issue::NotReached),
        ]
    );
}

#[test]
fn the_path_choices_split_unknown_not_from_the_path_and_not_reached() {
    let mut mythic = build("human", &times("fighter", 9));
    // An unknown path, a path only a switch takes, and a switch before its rank.
    mythic.mythic_path = Some(path("no-such-path"));
    mythic.first_ascension = Some(path("legend"));
    mythic.path_switch = Some(path("legend"));
    assert_eq!(
        pairs(&mythic),
        [
            (Choice::FirstAscension, Issue::NotAllowed),
            (Choice::MythicPath, Issue::UnknownKey),
            (Choice::PathSwitch, Issue::NotReached),
        ]
    );

    // The Devil switch is only from Aeon: from the Trickster it isn't, whatever the rank.
    mythic.mythic_path = Some(path("trickster"));
    mythic.first_ascension = None;
    mythic.path_switch = Some(path("devil"));
    assert_eq!(
        pairs(&mythic),
        [(
            Choice::PathSwitch,
            Issue::NotFromPath {
                from: BTreeSet::from([path("aeon")])
            }
        )]
    );

    // A switch that isn't one.
    mythic.path_switch = Some(path("aeon"));
    assert_eq!(pairs(&mythic), [(Choice::PathSwitch, Issue::NotAllowed)]);
}
