//! Which choices the sheet may remove or change now: those no later choice depends on, so a chain comes off from
//! the leaves inward. The pickers lock what would break a later choice.

use rules::{Ability, ArchetypeKey, Build, Choice, ClassChoice, Edit, Engine, Issue, SlotId};

use crate::support::*;

/// The picks among the choices that may be removed now.
fn removable_picks(engine: &Engine, build: &Build) -> Vec<SlotId> {
    engine
        .removable(build)
        .into_iter()
        .filter_map(|choice| match choice {
            Choice::Slot(slot) => Some(slot),
            _ => None,
        })
        .collect()
}

/// Elf fighter 5: Power Attack (F1 bonus), Cleave (L1), Great Cleave (F4 bonus), Cleaving Finish (L5).
fn cleave_chain() -> Build {
    let mut build = build("elf", &times("fighter", 5));
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    build.picks.insert(general_slot(1), pick_feat("cleave"));
    build
        .picks
        .insert(class_slot(4, 0), pick_feat("great-cleave"));
    build
        .picks
        .insert(general_slot(5), pick_feat("cleaving-finish"));
    assert!(engine().is_possible(&build));
    build
}

#[test]
fn a_chain_comes_off_from_the_leaves_inward() {
    let engine = engine();
    let mut build = cleave_chain();
    // Each step: only the last link may be removed; removing it frees the one before.
    for leaf in [
        general_slot(5),
        class_slot(4, 0),
        general_slot(1),
        class_slot(1, 0),
    ] {
        assert_eq!(
            removable_picks(&engine, &build),
            std::slice::from_ref(&leaf)
        );
        build.apply(&Edit::Clear(leaf));
        assert!(engine.is_possible(&build));
    }
    assert_eq!(removable_picks(&engine, &build), []);
}

#[test]
fn a_class_level_a_later_pick_depends_on_isnt_changeable() {
    let engine = engine();
    let build = cleave_chain();
    let removable = engine.removable(&build);
    // Fighter 1 and 4 hold Power Attack and Great Cleave in their bonus slots; changing fighter 2 or 3 would move
    // fighter 4's bonus slot to level 5.
    for level in 1..=4 {
        assert!(
            !removable.contains(&Choice::Class { level }),
            "level {level}"
        );
    }
    // Cleaving Finish sits in level 5's general slot, which any class has.
    assert!(removable.contains(&Choice::Class { level: 5 }));

    // With nothing in them, every level may change.
    let empty = build_with_no_picks(&build);
    let removable = engine.removable(&empty);
    assert!((1..=5).all(|level| removable.contains(&Choice::Class { level })));
}

fn build_with_no_picks(build: &Build) -> Build {
    Build {
        picks: Default::default(),
        ..build.clone()
    }
}

#[test]
fn an_archetype_a_later_pick_depends_on_isnt_changeable() {
    let engine = engine();
    let mut build = build("elf", &times("fighter", 2));
    build.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    let archetype = Choice::Archetype {
        class: class("fighter"),
    };
    assert!(engine.removable(&build).contains(&archetype));

    // The armiger's order fills a selection only its row has.
    build.picks.insert(
        selection_slot(1, 0),
        pick_option("fighter/armiger/order/cockatrice"),
    );
    assert!(engine.is_possible(&build));
    let removable = engine.removable(&build);
    assert!(!removable.contains(&archetype));
    assert!(removable.contains(&Choice::Slot(selection_slot(1, 0))));
}

#[test]
fn a_point_buy_score_a_later_pick_depends_on_isnt_changeable() {
    let engine = engine();
    let mut build = build("elf", &["fighter"]);
    let all_scores = |build: &Build| {
        let removable = engine.removable(build);
        rules::AbilityScores::ABILITIES
            .into_iter()
            .filter(|ability| removable.contains(&Choice::AbilityScore { ability: *ability }))
            .collect::<Vec<_>>()
    };
    assert_eq!(all_scores(&build).len(), 6);

    // Power Attack needs Str 13.
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    assert!(!all_scores(&build).contains(&Ability::Strength));
    assert_eq!(all_scores(&build).len(), 5);
}

#[test]
fn skill_ranks_a_prestige_class_needs_are_not_removable() {
    // Rogue 3 / Wizard 3, Mobility 4 by level 4, then Arcane Trickster.
    let mut build = build(
        "elf",
        &[
            "rogue",
            "rogue",
            "rogue",
            "wizard",
            "wizard",
            "wizard",
            "arcane-trickster",
        ],
    );
    for level in &mut build.levels[..4] {
        level.skill_ranks.insert(skill("mobility"), 1);
    }
    assert!(engine().is_possible(&build));
    let removable = engine().removable(&build);
    for level in 1..=4 {
        assert!(
            !removable.contains(&Choice::SkillRanks {
                level,
                skill: skill("mobility")
            }),
            "level {level}"
        );
    }
    // The prestige class is the last level: nothing depends on it.
    assert!(removable.contains(&Choice::Class { level: 7 }));
}

#[test]
fn a_switch_the_levels_past_20_need_is_not_removable() {
    let engine = engine();
    let mut build = build("elf", &times("fighter", 20));
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    assert!(engine.is_possible(&build));
    assert!(engine.removable(&build).contains(&Choice::PathSwitch));

    build.apply(&Edit::Class {
        level: 21,
        class: class("rogue"),
        choice: None,
    });
    assert!(engine.is_possible(&build));
    assert!(!engine.removable(&build).contains(&Choice::PathSwitch));
}

#[test]
fn an_option_a_later_pick_conflicts_with_is_locked() {
    let engine = engine();
    let mut build = build("elf", &times("fighter", 3));
    build.picks.insert(general_slot(3), pick_feat("toughness"));
    let options = engine.options(&build, &general_slot(1)).unwrap();
    let status = |key: &str| {
        options
            .iter()
            .find(|option| option.pick.key() == key)
            .unwrap()
            .status
            .clone()
    };
    // Toughness at 1 would make level 3's Toughness held twice.
    assert_eq!(status("toughness"), Err(Issue::BreaksLater));
    assert_eq!(status("dodge"), Ok(()));
}

#[test]
fn replacing_a_pick_with_dependants_is_locked() {
    let engine = engine();
    let build = cleave_chain();
    let options = engine.options(&build, &general_slot(1)).unwrap();
    let status = |key: &str| {
        options
            .iter()
            .find(|option| option.pick.key() == key)
            .unwrap()
            .status
            .clone()
    };
    assert_eq!(status("cleave"), Ok(()));
    assert_eq!(status("toughness"), Err(Issue::BreaksLater));
}

#[test]
fn the_class_picker_locks_a_class_that_would_break_a_later_pick() {
    let engine = engine();
    let build = cleave_chain();
    let status = |level: u8, key: &str| {
        engine
            .class_options(&build, level)
            .unwrap()
            .into_iter()
            .find(|option| option.class == class(key) && option.archetype.is_none())
            .unwrap()
            .status
    };
    assert_eq!(status(4, "fighter"), Ok(()));
    assert_eq!(status(4, "wizard"), Err(Issue::BreaksLater));
    assert_eq!(status(5, "wizard"), Ok(()));
    // A level after the last breaks nothing.
    assert_eq!(status(6, "wizard"), Ok(()));
}

#[test]
fn the_switch_picker_locks_a_switch_the_levels_past_20_need() {
    let engine = engine();
    let mut build = build("elf", &[times("fighter", 20), vec!["rogue"]].concat());
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    assert!(engine.is_possible(&build));
    let status = |key: &str| {
        engine
            .switch_options(&build)
            .into_iter()
            .find(|option| option.path == path(key))
            .unwrap()
            .status
    };
    assert_eq!(status("legend"), Ok(()));
    // Gold Dragon keeps the cap at 20.
    assert_eq!(status("gold-dragon"), Err(Issue::BreaksLater));
}

#[test]
fn an_edit_is_allowed_when_every_other_choice_still_holds() {
    let engine = engine();
    // Wizard: 2 + Int 13 (+1) = 3 points, all spent.
    let mut build = build("dwarf", &["wizard"]);
    for key in ["athletics", "mobility", "persuasion"] {
        build.levels[0].skill_ranks.insert(skill(key), 1);
    }
    assert!(engine.is_possible(&build));
    let lower = |ability: Ability, score: u8| {
        let mut start = build.start.clone();
        start.ability_scores = start.ability_scores.with(ability, score).unwrap();
        Edit::Start(start)
    };
    // Int 11 leaves 2 points for the 3 ranks.
    assert!(!engine.allows(&build, &lower(Ability::Intelligence, 11)));
    assert!(engine.allows(&build, &lower(Ability::Charisma, 7)));
    assert!(engine.allows(&build, &Edit::Clear(general_slot(1))));
    let removable = engine.removable(&build);
    assert!(!removable.contains(&Choice::AbilityScore {
        ability: Ability::Intelligence
    }));
}

// ---- one choice, the class rows, many edits at once ------------------------------------------------------------

/// Every choice the Build makes, as `Engine::removable` names them, plus a few it doesn't make.
fn every_choice(build: &Build) -> Vec<Choice> {
    let mut choices = vec![
        Choice::Race,
        Choice::Subrace,
        Choice::Alignment,
        Choice::Background,
        Choice::MythicPath,
        Choice::PathSwitch,
    ];
    choices.extend(rules::AbilityScores::ABILITIES.map(|ability| Choice::AbilityScore { ability }));
    for (index, level_up) in build.levels.iter().enumerate() {
        let level = u8::try_from(index + 1).unwrap();
        choices.push(Choice::Class { level });
        choices.push(Choice::AbilityIncrease { level });
        choices.push(Choice::SkillPoints { level });
        for skill in level_up.skill_ranks.keys() {
            choices.push(Choice::SkillRanks {
                level,
                skill: skill.clone(),
            });
        }
        choices.push(Choice::Slot(general_slot(level)));
        choices.push(Choice::Slot(class_slot(level, 0)));
        choices.push(Choice::Slot(selection_slot(level, 0)));
    }
    let past = u8::try_from(build.levels.len() + 1).unwrap();
    choices.push(Choice::Class { level: past });
    for class in build.levels.iter().map(|level| level.class.clone()) {
        choices.push(Choice::Archetype {
            class: class.clone(),
        });
        choices.push(Choice::Advances { class });
    }
    choices.extend(build.picks.keys().cloned().map(Choice::Slot));
    choices
}

/// Builds that exercise each kind of choice: a feat chain, an archetype's order, skill ranks a prestige class
/// needs, a Legend switch the levels past 20 need.
fn removable_builds() -> Vec<Build> {
    let mut armiger = build("elf", &times("fighter", 2));
    armiger.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    armiger.picks.insert(
        selection_slot(1, 0),
        pick_option("fighter/armiger/order/cockatrice"),
    );
    let mut trickster = build(
        "elf",
        &[
            "rogue",
            "rogue",
            "rogue",
            "wizard",
            "wizard",
            "wizard",
            "arcane-trickster",
        ],
    );
    for level in &mut trickster.levels[..4] {
        level.skill_ranks.insert(skill("mobility"), 1);
    }
    let mut legend = build("elf", &[times("fighter", 20), vec!["rogue"]].concat());
    legend.mythic_path = Some(path("trickster"));
    legend.path_switch = Some(path("legend"));
    let builds = vec![
        cleave_chain(),
        build_with_no_picks(&cleave_chain()),
        armiger,
        trickster,
        legend,
    ];
    for build in &builds {
        assert!(engine().is_possible(build));
    }
    builds
}

#[test]
fn one_choice_is_removable_when_removable_names_it() {
    let engine = engine();
    for build in removable_builds() {
        let removable = engine.removable(&build);
        for choice in every_choice(&build) {
            assert_eq!(
                engine.is_removable(&build, &choice),
                removable.contains(&choice),
                "{choice:?}"
            );
        }
    }
}

#[test]
fn the_removable_classes_are_the_levels_whose_class_removable_names() {
    let engine = engine();
    assert_eq!(
        engine.removable_classes(&cleave_chain()),
        std::collections::BTreeSet::from([5])
    );
    for build in removable_builds() {
        let levels: std::collections::BTreeSet<u8> = engine
            .removable(&build)
            .into_iter()
            .filter_map(|choice| match choice {
                Choice::Class { level } => Some(level),
                _ => None,
            })
            .collect();
        assert_eq!(engine.removable_classes(&build), levels);
    }
}

#[test]
fn many_edits_are_each_allowed_as_allows_says() {
    let engine = engine();
    let mut answers = std::collections::BTreeSet::new();
    for build in removable_builds() {
        let mut edits = vec![Edit::RemoveLevel, Edit::SetName(Some("x".into()))];
        for ability in rules::AbilityScores::ABILITIES {
            for score in [7, 10, 14, 17] {
                if let Some(scores) = build.start.ability_scores.with(ability, score) {
                    let mut start = build.start.clone();
                    start.ability_scores = scores;
                    edits.push(Edit::Start(start));
                }
            }
        }
        for level in 1..=u8::try_from(build.levels.len()).unwrap() {
            edits.push(Edit::Clear(general_slot(level)));
            edits.push(Edit::Pick {
                slot: general_slot(level),
                pick: pick_feat("toughness"),
            });
            edits.push(Edit::SkillRanks {
                level,
                skill: skill("mobility"),
                ranks: 0,
            });
            edits.push(Edit::Class {
                level,
                class: class("wizard"),
                choice: None,
            });
        }
        edits.extend(build.picks.keys().cloned().map(Edit::Clear));
        let one_by_one: Vec<bool> = edits
            .iter()
            .map(|edit| engine.allows(&build, edit))
            .collect();
        assert_eq!(engine.allows_many(&build, &edits), one_by_one);
        answers.extend(one_by_one);
    }
    assert_eq!(answers.len(), 2, "some edits allowed, some not");
}

// ---- what a picker's option can break later -----------------------------------------------------------------

/// The world changed by `change`, which must keep it passing the data check.
fn engine_with(change: impl FnOnce(&mut rules::GameData)) -> Engine {
    let mut data = world();
    change(&mut data);
    assert_eq!(data.check(), []);
    Engine::new(data)
}

/// An option's status in a slot's picker.
fn status_in(
    engine: &Engine,
    build: &Build,
    slot: &SlotId,
    pick: &rules::Pick,
) -> Result<(), Issue> {
    engine
        .options(build, slot)
        .unwrap()
        .into_iter()
        .find(|option| &option.pick == pick)
        .unwrap()
        .status
}

#[test]
fn a_feat_a_later_feat_rules_out_breaks_later() {
    // An invented feat for those without Dodge.
    let engine = engine_with(|data| {
        let mut lone = data
            .feats
            .iter()
            .find(|f| f.key.as_str() == "toughness")
            .unwrap()
            .clone();
        lone.key = feat("steady-stance");
        lone.prerequisite = Some(rules::Prerequisite::Not(Box::new(
            rules::Prerequisite::Feat(feat("dodge")),
        )));
        data.feats.push(lone);
        data.canonicalize();
    });
    let mut build = build("elf", &times("fighter", 3));
    build
        .picks
        .insert(general_slot(3), pick_feat("steady-stance"));
    assert!(engine.is_possible(&build));
    let slot = general_slot(1);
    assert_eq!(
        status_in(&engine, &build, &slot, &pick_feat("dodge")),
        Err(Issue::BreaksLater)
    );
    assert_eq!(
        status_in(&engine, &build, &slot, &pick_feat("toughness")),
        Ok(())
    );
}

#[test]
fn a_feat_slot_holding_what_a_later_class_needs_keeps_only_it() {
    // The wizard needs Dodge to enter.
    let engine = engine_with(|data| {
        let wizard = data
            .classes
            .iter_mut()
            .find(|c| c.key.as_str() == "wizard")
            .unwrap();
        wizard.prerequisite = Some(rules::Prerequisite::Feat(feat("dodge")));
    });
    let mut build = build("elf", &["fighter", "wizard"]);
    build.picks.insert(general_slot(1), pick_feat("dodge"));
    assert!(engine.is_possible(&build));
    let slot = general_slot(1);
    assert_eq!(
        status_in(&engine, &build, &slot, &pick_feat("dodge")),
        Ok(())
    );
    assert_eq!(
        status_in(&engine, &build, &slot, &pick_feat("toughness")),
        Err(Issue::BreaksLater)
    );
    // Taking it in the class slot instead leaves the general slot free.
    build.picks.remove(&general_slot(1));
    build.picks.insert(class_slot(1, 0), pick_feat("dodge"));
    assert!(engine.is_possible(&build));
    assert_eq!(
        status_in(&engine, &build, &slot, &pick_feat("toughness")),
        Ok(())
    );
    assert_eq!(
        status_in(&engine, &build, &slot, &pick_feat("dodge")),
        Err(Issue::AlreadyHeld)
    );
}

#[test]
fn a_feat_a_later_options_chosen_feat_holds_breaks_later() {
    // The rogue talent Weapon Focus (Longsword) at 4: Weapon Focus (Longsword) at 3 would hold it twice.
    let mut build = build("elf", &times("rogue", 4));
    build.picks.insert(
        selection_slot(4, 0),
        pick_option_for("rogue/rogue-talents/weapon-focus", weapon("longsword")),
    );
    let engine = engine();
    assert!(engine.is_possible(&build));
    let slot = general_slot(3);
    assert_eq!(
        status_in(
            &engine,
            &build,
            &slot,
            &pick_feat_for("weapon-focus", weapon("longsword"))
        ),
        Err(Issue::BreaksLater)
    );
    assert_eq!(
        status_in(
            &engine,
            &build,
            &slot,
            &pick_feat_for("weapon-focus", weapon("dagger"))
        ),
        Ok(())
    );
}

#[test]
fn in_a_build_that_isnt_possible_only_an_option_that_mends_it_is_open() {
    // Cleave at 3 without Power Attack: no Build the sheet makes, but the pickers still answer for it.
    let engine = engine();
    let mut build = build("elf", &times("fighter", 3));
    build.picks.insert(general_slot(3), pick_feat("cleave"));
    assert!(!engine.is_possible(&build));
    let slot = general_slot(1);
    assert_eq!(
        status_in(&engine, &build, &slot, &pick_feat("toughness")),
        Err(Issue::BreaksLater)
    );
    assert_eq!(
        status_in(&engine, &build, &slot, &pick_feat("power-attack")),
        Ok(())
    );
}
