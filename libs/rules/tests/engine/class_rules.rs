//! Class rules from the hand-fix follow-ups: a prestige class advancing one class, arcane or divine; levels that
//! count as another class's (for feat prerequisites, or toward a feature); story conditions that never block.

use std::collections::BTreeSet;

use rules::{Build, ClassChoice, Issue, Prerequisite, SlotId, Story, Unmet};

use crate::support::*;

/// `classes` in order, with `prestige` advancing `advances`.
fn with_advances(classes: &[&str], prestige: &str, advances: &[&str]) -> Build {
    let mut build = build("elf", classes);
    build.class_choices.insert(
        class(prestige),
        ClassChoice {
            archetype: None,
            advances: advances.iter().map(|key| class(key)).collect(),
        },
    );
    build
}

/// Whether the Build is possible with `prestige` advancing `advances` instead.
fn possible_advancing(build: &Build, prestige: &str, advances: &[&str]) -> bool {
    let mut build = build.clone();
    build
        .class_choices
        .get_mut(&class(prestige))
        .unwrap()
        .advances = advances.iter().map(|key| class(key)).collect();
    engine().is_possible(&build)
}

fn status(build: &Build, slot: &SlotId, key: &str) -> Result<(), Issue> {
    engine()
        .options(build, slot)
        .unwrap()
        .into_iter()
        .find(|o| o.pick.key() == key)
        .unwrap_or_else(|| panic!("no option {key}"))
        .status
}

// ---- advances one class, arcane or divine ---------------------------------------------------------------------

#[test]
fn a_winter_witch_advances_one_class_arcane_or_divine() {
    let arcane = with_advances(
        &["witch", "witch", "witch", "winter-witch", "winter-witch"],
        "winter-witch",
        &["witch"],
    );
    assert!(engine().is_possible(&arcane));
    // Witch 3 / Winter Witch 2 casts as a witch 5.
    assert_eq!(
        engine().evaluate(&arcane).at(5).unwrap().arcane_spell_level,
        3
    );

    let divine = with_advances(
        &["cleric", "cleric", "cleric", "winter-witch", "winter-witch"],
        "winter-witch",
        &["cleric"],
    );
    assert!(engine().is_possible(&divine));
    assert_eq!(
        engine().evaluate(&divine).at(5).unwrap().divine_spell_level,
        3
    );
}

#[test]
fn a_winter_witch_cannot_advance_an_arcane_and_a_divine_class() {
    let build = with_advances(
        &[
            "witch",
            "witch",
            "witch",
            "cleric",
            "cleric",
            "cleric",
            "winter-witch",
        ],
        "winter-witch",
        &["witch", "cleric"],
    );
    assert!(!engine().is_possible(&build));
    assert!(possible_advancing(&build, "winter-witch", &["witch"]));
}

// ---- levels counting as another class's -----------------------------------------------------------------------

/// Wizard 3, Fighter 1, Eldritch Knight 3: Weapon Focus (Longsword) at the fighter's bonus slot.
fn eldritch_knight() -> Build {
    let mut build = build(
        "elf",
        &[
            "wizard",
            "wizard",
            "wizard",
            "fighter",
            "eldritch-knight",
            "eldritch-knight",
            "eldritch-knight",
        ],
    );
    build.picks.insert(
        class_slot(4, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build
}

#[test]
fn eldritch_knight_levels_count_as_fighter_levels_for_feat_prerequisites() {
    let mut build = eldritch_knight();
    // Weapon Specialization needs fighter level 4: fighter 1 + Eldritch Knight 3.
    build.picks.insert(
        general_slot(7),
        pick_feat_for("weapon-specialization", weapon("longsword")),
    );
    assert!(engine().is_possible(&build));

    let at_7 = engine().evaluate(&build).at(7).cloned().unwrap();
    assert_eq!(at_7.class_level(&class("fighter")), 1);
    assert_eq!(at_7.feat_class_level(&class("fighter")), 4);
}

#[test]
fn before_enough_eldritch_knight_levels_the_feat_stays_locked() {
    // At character level 5 the knight has 1 level: fighter level 2 for feats.
    let build = eldritch_knight();
    let specialization = pick_feat_for("weapon-specialization", weapon("longsword"));
    let Err(Issue::Unmet(unmet)) = engine()
        .options(&build, &general_slot(5))
        .unwrap()
        .into_iter()
        .find(|o| o.pick == specialization)
        .unwrap()
        .status
    else {
        panic!("Weapon Specialization is open at fighter 1 / Eldritch Knight 1");
    };
    assert_eq!(
        unmet,
        Unmet::Condition {
            needs: Prerequisite::ClassLevel {
                class: class("fighter"),
                level: 4,
            },
            has: Some(2),
        }
    );
}

#[test]
fn diverse_training_counts_for_feats_only() {
    let at_7 = engine()
        .evaluate(&eldritch_knight())
        .at(7)
        .cloned()
        .unwrap();
    let fighter_4 = Prerequisite::ClassLevel {
        class: class("fighter"),
        level: 4,
    };
    assert!(at_7.check(&fighter_4).is_err());
    assert_eq!(at_7.check_feat_for(&fighter_4, None), Ok(()));
}

fn hexes_held(build: &Build, level: u8) -> (u8, u8) {
    let state = engine().evaluate(build).at(level).cloned().unwrap();
    (
        state.feature_count(&feature("witch/major-hexes")),
        state.feature_count(&feature("witch/grand-hexes")),
    )
}

#[test]
fn winter_witch_levels_stack_with_witch_levels_for_major_hexes() {
    // Witch 9, Winter Witch 1, then Witch 10.
    let mut classes = times("witch", 9);
    classes.push("winter-witch");
    classes.push("witch");
    let build = with_advances(&classes, "winter-witch", &["witch"]);
    assert!(engine().is_possible(&build));

    assert_eq!(hexes_held(&build, 9), (0, 0));
    assert_eq!(hexes_held(&build, 10), (1, 0));
    // The witch's own 10th level doesn't grant it a second time.
    assert_eq!(hexes_held(&build, 11), (1, 0));
    // So the witch's 10th-level hex can be a major hex.
    assert_eq!(
        status(&build, &selection_slot(11, 0), "witch/hex/agony"),
        Ok(())
    );
}

#[test]
fn winter_witch_levels_stack_with_witch_levels_for_grand_hexes() {
    let mut classes = times("witch", 9);
    classes.extend(times("winter-witch", 9));
    let build = with_advances(&classes, "winter-witch", &["witch"]);
    assert_eq!(hexes_held(&build, 17), (1, 0));
    assert_eq!(hexes_held(&build, 18), (1, 1));
}

#[test]
fn without_witch_levels_winter_witch_levels_reach_no_witch_row() {
    let mut classes = times("cleric", 3);
    classes.extend(times("winter-witch", 7));
    let build = with_advances(&classes, "winter-witch", &["cleric"]);
    assert_eq!(hexes_held(&build, 10), (0, 0));
}

// ---- story conditions -----------------------------------------------------------------------------------------

#[test]
fn a_story_condition_always_holds() {
    let blank = engine()
        .evaluate(&build("elf", &["fighter"]))
        .at(1)
        .cloned()
        .unwrap();
    for story in Story::ALL {
        assert_eq!(blank.check(&Prerequisite::Story(story)), Ok(()));
    }
}

#[test]
fn a_story_condition_is_a_note_beside_the_option() {
    let engine = engine();
    assert_eq!(
        engine.story_conditions(&pick_option("gold-dragon/breaths/annihilating")),
        BTreeSet::from([Story::VowOfDahak])
    );
    assert_eq!(
        engine.story_conditions(&pick_option("gold-dragon/breaths/empowered")),
        BTreeSet::new()
    );
    assert_eq!(
        engine.story_conditions(&pick_feat("power-attack")),
        BTreeSet::new()
    );
}

#[test]
fn a_story_condition_is_never_a_reason_an_option_is_locked() {
    let mut build = build("human", &times("fighter", 17));
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("gold-dragon"));
    let slot = SlotId::PathSelection { rank: 8, index: 0 };
    let Err(Issue::Unmet(unmet)) = status(&build, &slot, "gold-dragon/breaths/annihilating") else {
        panic!("the breath is open at MR 8");
    };
    assert_eq!(
        conditions(&unmet),
        [(&Prerequisite::MythicRank(9), Some(8))]
    );
}
