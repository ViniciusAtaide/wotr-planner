//! Options with rows by option level (ticket 30): a bloodline, school or combat style gives features, feats, feat
//! lists and selections as the levels of the classes its selection names reach each row. A selection can lag
//! behind its classes, take its option level from the options held elsewhere or from the option that granted its
//! slot, and draw on a pool beside options of its own.

use rules::{
    Build, CharacterState, ClassChoice, FeatSlot, FeatureKey, FeatureSelectionKey, Issue,
    Prerequisite, SlotId, SlotKind, SpellType, Unmet,
};

use crate::support::*;

/// `classes` in order, with `archetype` on `of` and `picks` at their slots.
fn with(classes: &[&str], archetype: Option<(&str, &str)>, picks: &[(SlotId, &str)]) -> Build {
    let mut build = build("elf", classes);
    if let Some((of, archetype)) = archetype {
        build.class_choices.insert(
            class(of),
            ClassChoice {
                archetype: Some(rules::ArchetypeKey::new(archetype)),
                ..ClassChoice::default()
            },
        );
    }
    for (slot, key) in picks {
        build.picks.insert(slot.clone(), pick_option(key));
    }
    build
}

fn sorcerer(bloodline: &str, levels: usize) -> Build {
    with(
        &times("sorcerer", levels),
        None,
        &[(selection_slot(1, 0), bloodline)],
    )
}

fn at(build: &Build, level: u8) -> CharacterState {
    engine().evaluate(build).at(level).cloned().unwrap()
}

fn holds(state: &CharacterState, key: &str) -> bool {
    state.feature_count(&feature(key)) > 0
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

/// The keys offered, each once (an option with a chosen feat is offered once per value).
fn offered(build: &Build, slot: &SlotId) -> Vec<String> {
    let mut keys: Vec<String> = engine()
        .options(build, slot)
        .unwrap()
        .into_iter()
        .map(|o| o.pick.key().to_owned())
        .collect();
    keys.dedup();
    keys
}

const ABYSSAL: &str = "sorcerer/bloodline/abyssal";
const ARCANE: &str = "sorcerer/bloodline/arcane";
const RED: &str = "sorcerer/bloodline/draconic-red";

// ---- rows by option level ---------------------------------------------------------------------------------

#[test]
fn a_sorcerer_holds_the_bloodlines_rows_as_sorcerer_levels_reach_them() {
    let build = sorcerer(ABYSSAL, 20);
    assert!(engine().is_possible(&build));
    assert!(holds(&at(&build, 1), "sorcerer/bloodline/abyssal/claws"));
    assert!(!holds(
        &at(&build, 2),
        "sorcerer/bloodline/abyssal/resistances"
    ));
    assert!(holds(
        &at(&build, 3),
        "sorcerer/bloodline/abyssal/resistances"
    ));
    let at_15 = at(&build, 15);
    assert!(holds(&at_15, "sorcerer/bloodline/abyssal/added-summonings"));
    assert!(at_15.feats.contains_key(&feat("toughness")));
    assert!(!holds(
        &at(&build, 19),
        "sorcerer/bloodline/abyssal/demonic-might"
    ));
    let at_20 = at(&build, 20);
    assert!(holds(&at_20, "sorcerer/bloodline/abyssal/demonic-might"));
    assert_eq!(at_20.option_level(&option(ABYSSAL)), 20);
}

#[test]
fn an_options_ability_bonus_counts_once_per_row() {
    // Strength of the Abyss: +2 Strength at 9, 13 and 17 (Str 16 at Build start).
    let build = sorcerer(ABYSSAL, 17);
    let strength = |level| at(&build, level).ability_scores.strength;
    assert_eq!(strength(8), 16);
    assert_eq!(strength(9), 18);
    assert_eq!(strength(13), 20);
    assert_eq!(strength(17), 22);
}

#[test]
fn the_bloodline_feat_slots_pick_from_the_held_bloodlines_list() {
    let build = sorcerer(ABYSSAL, 7);
    let slot = class_slot(7, 0);
    assert_eq!(offered(&build, &slot), ["cleave", "power-attack"]);
    // The bloodline's list checks prerequisites: Cleave needs Power Attack.
    assert!(matches!(
        status(&build, &slot, "cleave"),
        Err(Issue::Unmet(_))
    ));
    assert_eq!(status(&build, &slot, "power-attack"), Ok(()));

    let arcane = sorcerer(ARCANE, 7);
    assert_eq!(
        offered(&arcane, &slot),
        ["arcane-strike", "spell-penetration"]
    );
}

#[test]
fn a_seeker_skips_the_bloodline_rows_at_3_9_and_15() {
    let build = with(
        &times("sorcerer", 20),
        Some(("sorcerer", "sorcerer/seeker")),
        &[(selection_slot(1, 0), ABYSSAL)],
    );
    assert!(engine().is_possible(&build));
    assert!(holds(&at(&build, 1), "sorcerer/bloodline/abyssal/claws"));
    assert!(!holds(
        &at(&build, 3),
        "sorcerer/bloodline/abyssal/resistances"
    ));
    assert_eq!(at(&build, 9).ability_scores.strength, 16);
    assert_eq!(at(&build, 13).ability_scores.strength, 18);
    assert!(!holds(
        &at(&build, 15),
        "sorcerer/bloodline/abyssal/added-summonings"
    ));
    assert!(holds(
        &at(&build, 20),
        "sorcerer/bloodline/abyssal/demonic-might"
    ));
    // The feat list is the bloodline's still.
    assert_eq!(
        offered(&build, &class_slot(7, 0)),
        ["cleave", "power-attack"]
    );
}

#[test]
fn an_option_row_can_grant_a_selection_slot() {
    let mut build = sorcerer(ARCANE, 15);
    let slot = SlotId::OptionSelection {
        level: 15,
        index: 0,
    };
    let sheet = engine().evaluate(&build);
    let found = sheet.slot(&slot).expect("the school slot at 15");
    assert_eq!(
        found.kind,
        SlotKind::Selection(FeatureSelectionKey::new("sorcerer/school-power"))
    );
    assert!(
        sheet
            .slot(&SlotId::OptionSelection {
                level: 14,
                index: 0
            })
            .is_none()
    );

    build
        .picks
        .insert(slot.clone(), pick_option("sorcerer/school-power/evocation"));
    assert!(engine().is_possible(&build));
    assert_eq!(
        at(&build, 15).option_count(&option("sorcerer/school-power/evocation")),
        1
    );
}

#[test]
fn a_pick_for_an_option_rows_slot_the_build_lacks_makes_it_impossible() {
    let mut build = sorcerer(ARCANE, 14);
    let slot = SlotId::OptionSelection {
        level: 14,
        index: 0,
    };
    build
        .picks
        .insert(slot.clone(), pick_option("sorcerer/school-power/evocation"));
    assert_eq!(why_not(&build, &slot), None);
}

// ---- options taken late, and twice ------------------------------------------------------------------------

/// Sorcerer `levels` with `bloodline`, and Second Bloodline at MR 1 (character level 6) taking `second`.
fn second_bloodline(bloodline: &str, second: &str, levels: usize) -> Build {
    let mut build = sorcerer(bloodline, levels);
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("second-bloodline"),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        pick_option(second),
    );
    build
}

#[test]
fn an_option_taken_late_gets_every_row_up_to_its_level_at_once() {
    let build = second_bloodline(ABYSSAL, ARCANE, 7);
    assert!(engine().is_possible(&build));
    let at_6 = at(&build, 6);
    assert!(holds(&at_6, "sorcerer/bloodline/arcane/arcane-bond"));
    assert!(holds(
        &at_6,
        "sorcerer/bloodline/arcane/combat-casting-adept"
    ));
    assert_eq!(at_6.option_level(&option(ARCANE)), 6);
    // Both bloodlines' feats at the next bloodline feat slot.
    assert_eq!(
        offered(&build, &class_slot(7, 0)),
        [
            "arcane-strike",
            "cleave",
            "power-attack",
            "spell-penetration"
        ]
    );
}

#[test]
fn a_selection_held_twice_takes_another_option() {
    let build = second_bloodline(ABYSSAL, ABYSSAL, 6);
    assert_eq!(
        status(
            &build,
            &SlotId::MythicSelection { rank: 1, index: 0 },
            ABYSSAL
        ),
        Err(Issue::AlreadyHeld)
    );
}

#[test]
fn crossblooded_second_bloodline_adds_only_its_feat_list() {
    let build = with(
        &times("sorcerer", 7),
        Some(("sorcerer", "sorcerer/crossblooded")),
        &[
            (selection_slot(1, 0), ABYSSAL),
            (selection_slot(1, 1), ARCANE),
        ],
    );
    assert!(engine().is_possible(&build));
    let at_3 = at(&build, 3);
    assert!(holds(&at_3, "sorcerer/bloodline/abyssal/resistances"));
    assert!(!holds(&at_3, "sorcerer/bloodline/arcane/arcane-bond"));
    assert!(!holds(
        &at_3,
        "sorcerer/bloodline/arcane/combat-casting-adept"
    ));
    assert_eq!(at_3.option_count(&option(ARCANE)), 1);
    assert_eq!(
        offered(&build, &class_slot(7, 0)),
        [
            "arcane-strike",
            "cleave",
            "power-attack",
            "spell-penetration"
        ]
    );
}

#[test]
fn a_selection_offering_one_option_is_granted_with_no_pick() {
    let build = with(
        &times("sorcerer", 7),
        Some(("sorcerer", "sorcerer/sage")),
        &[],
    );
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet.levels[0]
            .slots
            .iter()
            .map(|slot| slot.id.clone())
            .collect::<Vec<_>>(),
        [general_slot(1)]
    );
    let at_1 = at(&build, 1);
    assert_eq!(at_1.option_count(&option(ARCANE)), 1);
    // Its own row replaces the bloodline's first.
    assert!(holds(&at_1, "sorcerer/sage/arcane-bolt"));
    assert!(!holds(&at_1, "sorcerer/bloodline/arcane/arcane-bond"));
    assert!(holds(
        &at(&build, 3),
        "sorcerer/bloodline/arcane/combat-casting-adept"
    ));
    assert_eq!(
        offered(&build, &class_slot(7, 0)),
        ["arcane-strike", "spell-penetration"]
    );
}

// ---- other classes' levels, and entry ---------------------------------------------------------------------

fn sorcerer_then_dragon_disciple(bloodline: &str, sorcerer: usize, disciple: usize) -> Build {
    let mut classes = times("sorcerer", sorcerer);
    classes.extend(times("dragon-disciple", disciple));
    with(&classes, None, &[(selection_slot(1, 0), bloodline)])
}

#[test]
fn dragon_disciple_levels_continue_a_sorcerers_bloodline() {
    let build = sorcerer_then_dragon_disciple(RED, 5, 5);
    assert!(engine().is_possible(&build));
    assert_eq!(at(&build, 10).option_level(&option(RED)), 10);
    // A sorcerer with a Draconic bloodline picks none on entry.
    let sheet = engine().evaluate(&build);
    assert!(sheet.slot(&selection_slot(6, 0)).is_none());
}

#[test]
fn a_bard_picks_a_draconic_bloodline_on_entering_dragon_disciple() {
    let mut classes = times("bard", 5);
    classes.extend(times("dragon-disciple", 3));
    let build = with(
        &classes,
        None,
        &[(selection_slot(6, 0), "sorcerer/bloodline/draconic-gold")],
    );
    assert!(engine().is_possible(&build));
    let gold = option("sorcerer/bloodline/draconic-gold");
    assert_eq!(at(&build, 7).option_level(&gold), 2);
    assert!(!holds(
        &at(&build, 7),
        "sorcerer/bloodline/draconic/resistances"
    ));
    assert!(holds(
        &at(&build, 8),
        "sorcerer/bloodline/draconic/resistances"
    ));
    assert_eq!(
        offered(&build, &selection_slot(6, 0)),
        [
            "sorcerer/bloodline/draconic-gold",
            "sorcerer/bloodline/draconic-red"
        ]
    );
}

fn disciple_status(build: &Build) -> Result<(), Issue> {
    let level = u8::try_from(build.levels.len() + 1).unwrap();
    engine()
        .class_options(build, level)
        .unwrap()
        .into_iter()
        .find(|o| o.class == class("dragon-disciple"))
        .unwrap()
        .status
}

#[test]
fn dragon_disciple_entry_needs_spontaneous_arcane_spells_and_a_draconic_bloodline_or_none() {
    assert_eq!(disciple_status(&sorcerer(RED, 5)), Ok(()));
    assert_eq!(disciple_status(&build("elf", &times("bard", 5))), Ok(()));

    let Err(Issue::Unmet(unmet)) = disciple_status(&sorcerer(ABYSSAL, 5)) else {
        panic!("an Abyssal sorcerer may not enter");
    };
    assert!(
        conditions(&unmet)
            .iter()
            .any(|(needs, _)| matches!(needs, Prerequisite::NoOptionFrom(_)))
    );

    let Err(Issue::Unmet(unmet)) = disciple_status(&build("elf", &times("wizard", 5))) else {
        panic!("a wizard casts prepared spells");
    };
    assert_eq!(
        unmet,
        Unmet::Condition {
            needs: Prerequisite::SpontaneousSpellLevel {
                spell_type: SpellType::Arcane,
                level: 1,
            },
            has: Some(0),
        }
    );
}

#[test]
fn no_option_from_and_spontaneous_spell_level_check_the_state() {
    let no_bloodline = Prerequisite::NoOptionFrom(FeatureSelectionKey::new("sorcerer/bloodline"));
    let spontaneous = Prerequisite::SpontaneousSpellLevel {
        spell_type: SpellType::Arcane,
        level: 1,
    };
    let sorcerer = at(&sorcerer(ABYSSAL, 1), 1);
    assert!(sorcerer.check(&no_bloodline).is_err());
    assert_eq!(sorcerer.check(&spontaneous), Ok(()));
    let bard = at(&build("elf", &["bard"]), 1);
    assert_eq!(bard.check(&no_bloodline), Ok(()));
    let wizard = at(&build("elf", &["wizard"]), 1);
    assert_eq!(wizard.arcane_spell_level, 1);
    assert!(wizard.check(&spontaneous).is_err());
}

// ---- a feature granting a row; a choice replacing one -----------------------------------------------------

#[test]
fn bloodline_ascendance_grants_the_bloodlines_20th_level_row() {
    let mut build = sorcerer(ABYSSAL, 7);
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("bloodline-ascendance"),
    );
    assert!(engine().is_possible(&build));
    assert!(!holds(
        &at(&build, 5),
        "sorcerer/bloodline/abyssal/demonic-might"
    ));
    assert!(holds(
        &at(&build, 6),
        "sorcerer/bloodline/abyssal/demonic-might"
    ));
    assert!(holds(
        &at(&build, 7),
        "sorcerer/bloodline/abyssal/demonic-might"
    ));
}

/// A bloodline held through a selection that skips its 20th row gets nothing from Bloodline Ascendance (ticket 180);
/// one whose selection skips other rows (the Seeker's 3, 9 and 15) still gets it.
#[test]
fn bloodline_ascendance_passes_over_a_row_the_holdings_selection_skips() {
    let mut build = with(
        &times("sorcerer", 7),
        Some(("sorcerer", "sorcerer/seeker")),
        &[(selection_slot(1, 0), ABYSSAL)],
    );
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("bloodline-ascendance"),
    );
    let might = FeatureKey::new("sorcerer/bloodline/abyssal/demonic-might");
    let held_at_7 = |engine: &rules::Engine| {
        assert!(engine.is_possible(&build));
        engine.evaluate(&build).at(7).unwrap().feature_count(&might)
    };
    assert_eq!(held_at_7(&engine()), 1);
    // The Seeker invented to skip the 20th row too.
    let mut data = world();
    let seeker = data
        .all_classes_mut()
        .find(|class| class.key.as_str() == "sorcerer")
        .unwrap()
        .archetypes
        .iter_mut()
        .flat_map(|archetype| &mut archetype.selections)
        .find(|selection| selection.key.as_str() == "sorcerer/seeker/bloodline")
        .unwrap();
    seeker.skipped_levels.insert(20);
    assert_eq!(held_at_7(&engine_on(data)), 0);
}

/// A selection's `replaced_by` is what a holding gets at a skipped level in place of the row (ticket 181): early
/// through Bloodline Ascendance, else at the option level, and once at every level either way.
#[test]
fn a_replacement_for_a_skipped_row_is_held_once_early_or_at_its_level() {
    let mut build = with(
        &times("sorcerer", 20),
        Some(("sorcerer", "sorcerer/seeker")),
        &[(selection_slot(1, 0), ABYSSAL)],
    );
    let plain = build.clone();
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("bloodline-ascendance"),
    );
    let mut data = world();
    let seeker = data
        .all_classes_mut()
        .find(|class| class.key.as_str() == "sorcerer")
        .unwrap()
        .archetypes
        .iter_mut()
        .flat_map(|archetype| &mut archetype.selections)
        .find(|selection| selection.key.as_str() == "sorcerer/seeker/bloodline")
        .unwrap();
    seeker.skipped_levels.insert(20);
    let replacement = FeatureKey::new("sorcerer/bloodline/arcane/apotheosis");
    seeker.replaced_by.insert(20, replacement.clone());
    let engine = engine_on(data);
    let might = FeatureKey::new("sorcerer/bloodline/abyssal/demonic-might");
    assert!(engine.is_possible(&build) && engine.is_possible(&plain));
    let rank = engine.rank_level(&build, 1).unwrap();
    for level in 1..=20 {
        let state = engine.evaluate(&build).at(level).cloned().unwrap();
        assert_eq!(
            state.feature_count(&replacement),
            u8::from(level >= rank),
            "with the ability, level {level}"
        );
        assert_eq!(state.feature_count(&might), 0, "level {level}");
        let state = engine.evaluate(&plain).at(level).cloned().unwrap();
        assert_eq!(
            state.feature_count(&replacement),
            u8::from(level == 20),
            "without it, level {level}"
        );
    }
}

const FIRE: &str = "kineticist/element/fire";
const AIR: &str = "kineticist/element/air";
const WATER: &str = "kineticist/element/water";
const EXPANDED_DEFENSE: &str = "kineticist/wild-talents/expanded-defense";

/// A fire kineticist with `picks` at the selection slots of their levels (Expanded Element at 7 and 15, a wild
/// talent at even levels).
fn kineticist(levels: usize, picks: &[(u8, &str)]) -> Build {
    let mut all = vec![(selection_slot(1, 0), FIRE)];
    all.extend(
        picks
            .iter()
            .map(|&(level, key)| (selection_slot(level, 0), key)),
    );
    with(&times("kineticist", levels), None, &all)
}

#[test]
fn an_expanded_element_gives_its_feat_list_not_its_defensive_talent() {
    let build = kineticist(8, &[(7, AIR)]);
    assert!(engine().is_possible(&build));
    let at_8 = at(&build, 8);
    assert!(holds(&at_8, "kineticist/element/fire/defense"));
    assert!(!holds(&at_8, "kineticist/element/air/defense"));
}

/// A feature's row of a selection whose options give only their feat lists (Expanded Defense: "You unlock the
/// Defensive Wild Talent for any element you selected with Expanded Element") gives that row in full to each option
/// held through it, whether taken before the feature or after.
#[test]
fn a_features_option_row_unlocks_the_row_of_a_lists_only_selections_options() {
    let build = kineticist(16, &[(7, AIR), (8, EXPANDED_DEFENSE), (15, WATER)]);
    assert!(engine().is_possible(&build));
    assert!(!holds(&at(&build, 7), "kineticist/element/air/defense"));
    let at_8 = at(&build, 8);
    assert!(holds(&at_8, "kineticist/element/air/defense"));
    assert_eq!(
        at_8.feature_count(&feature("kineticist/element/fire/defense")),
        1
    );
    assert!(!holds(&at(&build, 14), "kineticist/element/water/defense"));
    let at_15 = at(&build, 15);
    assert!(holds(&at_15, "kineticist/element/water/defense"));
    assert_eq!(
        at_15.feature_count(&feature("kineticist/element/air/defense")),
        1
    );
    assert_eq!(
        at(&build, 16).feature_count(&feature("kineticist/element/water/defense")),
        1
    );
    // Another talent unlocks nothing.
    let plain = kineticist(
        16,
        &[
            (7, AIR),
            (8, "kineticist/wild-talents/skilled-kineticist"),
            (15, WATER),
        ],
    );
    assert!(!holds(&at(&plain, 16), "kineticist/element/air/defense"));
    assert!(!holds(&at(&plain, 16), "kineticist/element/water/defense"));
}

/// Bloodline Ascendance still passes over a lists-only holding: only a row of the selection it's held through
/// unlocks one.
#[test]
fn bloodline_ascendance_gives_a_crossblooded_second_bloodline_nothing() {
    let mut build = with(
        &times("sorcerer", 7),
        Some(("sorcerer", "sorcerer/crossblooded")),
        &[
            (selection_slot(1, 0), ARCANE),
            (selection_slot(1, 1), ABYSSAL),
        ],
    );
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("bloodline-ascendance"),
    );
    assert!(engine().is_possible(&build));
    assert!(!holds(
        &at(&build, 7),
        "sorcerer/bloodline/abyssal/demonic-might"
    ));
}

fn primalist(choice: &str) -> Build {
    with(
        &times("sorcerer", 4),
        Some(("sorcerer", "sorcerer/primalist")),
        &[
            (selection_slot(1, 0), ABYSSAL),
            (
                selection_slot(3, 0),
                &format!("sorcerer/primalist/primal-choice/{choice}"),
            ),
        ],
    )
}

#[test]
fn a_primalist_takes_the_bloodline_row_or_two_talents_instead() {
    let power = primalist("bloodline-power");
    assert!(engine().is_possible(&power));
    assert!(holds(
        &at(&power, 3),
        "sorcerer/bloodline/abyssal/resistances"
    ));
    assert!(
        engine()
            .evaluate(&power)
            .slot(&SlotId::OptionSelection { level: 3, index: 0 })
            .is_none()
    );

    let mut talents = primalist("talents");
    talents.picks.insert(
        SlotId::OptionSelection { level: 3, index: 1 },
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    assert!(engine().is_possible(&talents));
    let at_3 = at(&talents, 3);
    assert!(!holds(&at_3, "sorcerer/bloodline/abyssal/resistances"));
    assert_eq!(
        at_3.option_count(&option("rogue/rogue-talents/fast-stealth")),
        1
    );
    let sheet = engine().evaluate(&talents);
    for index in 0..2 {
        assert_eq!(
            sheet
                .slot(&SlotId::OptionSelection { level: 3, index })
                .unwrap()
                .kind,
            SlotKind::Selection(FeatureSelectionKey::new("rogue/rogue-talents"))
        );
    }
    // The skipped row stays skipped as the bloodline goes on.
    assert!(!holds(
        &at(&talents, 4),
        "sorcerer/bloodline/abyssal/resistances"
    ));
}

// ---- lists that grow; pools; daily choices ----------------------------------------------------------------

#[test]
fn a_style_slot_follows_the_style_picked_at_the_same_level_and_grows_with_it() {
    let mut build = with(
        &times("ranger", 6),
        None,
        &[(selection_slot(2, 0), "ranger/combat-style/archery")],
    );
    build.picks.insert(class_slot(2, 0), pick_feat("dodge"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let ids: Vec<SlotId> = sheet.levels[1]
        .slots
        .iter()
        .map(|slot| slot.id.clone())
        .collect();
    assert_eq!(ids, [selection_slot(2, 0), class_slot(2, 0)]);
    assert_eq!(
        sheet.slot(&class_slot(2, 0)).unwrap().kind,
        SlotKind::Feat(FeatSlot::OptionList {
            selection: FeatureSelectionKey::new("ranger/combat-style"),
            at_level: None,
            label: None,
        })
    );
    assert_eq!(offered(&build, &class_slot(2, 0)), ["dodge", "toughness"]);
    // At 6 the style's next list, ignoring prerequisites (Great Cleave without Cleave).
    assert_eq!(
        offered(&build, &class_slot(6, 0)),
        ["dodge", "great-cleave", "toughness"]
    );
    assert_eq!(status(&build, &class_slot(6, 0), "great-cleave"), Ok(()));
}

#[test]
fn a_pool_selection_offers_the_held_options_pool() {
    let fire = with(
        &times("cleric", 3),
        None,
        &[(selection_slot(1, 0), "cleric/domain/fire")],
    );
    let slot = selection_slot(3, 0);
    assert_eq!(
        offered(&fire, &slot),
        [
            "cleric/fire-revelations/burning-hands",
            "cleric/fire-revelations/fire-breath",
            "cleric/fire-revelations/maneuver-mastery",
            "cleric/fire-revelations/weapon-mastery"
        ]
    );
    let mut water = with(
        &times("cleric", 3),
        None,
        &[(selection_slot(1, 0), "cleric/domain/water")],
    );
    assert_eq!(
        offered(&water, &slot),
        [
            "cleric/water-revelations/icy-skin",
            "cleric/water-revelations/wave"
        ]
    );
    water.picks.insert(
        slot.clone(),
        pick_option("cleric/fire-revelations/fire-breath"),
    );
    assert_eq!(
        why_not(&water, &slot),
        None,
        "a fire revelation without the fire domain"
    );
}

#[test]
fn a_wizard_holds_the_schools_first_power_at_1() {
    let build = with(
        &["wizard"],
        None,
        &[(selection_slot(1, 0), "wizard/school/universalist")],
    );
    assert!(engine().is_possible(&build));
    assert!(holds(&at(&build, 1), "wizard/hand-of-the-apprentice"));
}

#[test]
fn a_daily_choice_is_a_note_not_a_pick() {
    let abjurer = with(
        &["wizard"],
        None,
        &[(selection_slot(1, 0), "wizard/school/abjurer")],
    );
    let engine = engine();
    assert_eq!(
        engine.daily_choices(&at(&abjurer, 1)),
        [FeatureKey::new("wizard/resistance")]
    );
    let universalist = with(
        &["wizard"],
        None,
        &[(selection_slot(1, 0), "wizard/school/universalist")],
    );
    assert_eq!(engine.daily_choices(&at(&universalist, 1)), []);
}

// ---- path switches --------------------------------------------------------------------------------------------

#[test]
fn a_path_switch_drops_an_option_a_dropped_pick_took() {
    // Second Bloodline at MR 3 (character level 9); Legend keeps only MR 1-2's general picks.
    let mut build = sorcerer(ABYSSAL, 17);
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    build.picks.insert(
        SlotId::Mythic { rank: 3 },
        pick_mythic_ability("second-bloodline"),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 3, index: 0 },
        pick_option(ARCANE),
    );
    assert!(engine().is_possible(&build));
    let at_16 = at(&build, 16);
    assert!(holds(
        &at_16,
        "sorcerer/bloodline/arcane/combat-casting-adept"
    ));
    assert_eq!(at_16.option_level(&option(ARCANE)), 16);
    let at_17 = at(&build, 17);
    assert_eq!(at_17.option_count(&option(ARCANE)), 0);
    assert!(!holds(
        &at_17,
        "sorcerer/bloodline/arcane/combat-casting-adept"
    ));
    assert_eq!(at_17.option_level(&option(ARCANE)), 0);
    // The first bloodline stays.
    assert!(holds(&at_17, "sorcerer/bloodline/abyssal/resistances"));
}

#[test]
fn a_class_rows_ability_bonus_counts_once_per_grant() {
    // Dragon Disciple: +2 Strength at its 2nd and 4th levels (Str 16 at Build start).
    let mut classes = times("bard", 5);
    classes.extend(times("dragon-disciple", 4));
    let build = with(
        &classes,
        None,
        &[(selection_slot(6, 0), "sorcerer/bloodline/draconic-gold")],
    );
    let strength = |level| at(&build, level).ability_scores.strength;
    assert_eq!(strength(6), 16);
    assert_eq!(strength(7), 18);
    assert_eq!(strength(9), 20);
}

// ---- option rows with feat slots and chosen feats (ticket 31) ---------------------------------------------

fn cleric_with(domain: &str, levels: usize) -> Build {
    with(
        &times("cleric", levels),
        None,
        &[(selection_slot(1, 0), domain)],
    )
}

#[test]
fn an_option_rows_feat_slot_is_a_slot_at_the_level_that_reaches_it() {
    let slot = SlotId::OptionFeat { level: 8, index: 0 };
    let sheet = engine().evaluate(&cleric_with("cleric/domain/war", 8));
    let found = sheet
        .slot(&slot)
        .expect("the War domain's bonus feat slot at 8");
    assert_eq!(found.kind, SlotKind::Feat(FeatSlot::CombatFeat));
    let seven = engine().evaluate(&cleric_with("cleric/domain/war", 7));
    assert!(
        seven
            .slot(&SlotId::OptionFeat { level: 7, index: 0 })
            .is_none()
    );

    let mut build = cleric_with("cleric/domain/war", 8);
    build.picks.insert(slot.clone(), pick_feat("toughness"));
    assert!(engine().is_possible(&build));
    assert_eq!(
        at(&build, 8).feat_count(&rules::FeatKey::new("toughness")),
        1
    );
    assert_eq!(
        at(&build, 7).feat_count(&rules::FeatKey::new("toughness")),
        0
    );

    let mut fire = cleric_with("cleric/domain/fire", 8);
    fire.picks.insert(slot.clone(), pick_feat("toughness"));
    assert_eq!(why_not(&fire, &slot), None);
}

#[test]
fn an_option_rows_chosen_feat_is_held_for_the_value_its_pick_named() {
    let mut build = cleric_with("cleric/domain/fire", 8);
    build.picks.insert(
        selection_slot(3, 0),
        pick_option_for(
            "cleric/fire-revelations/weapon-mastery",
            weapon("longsword"),
        ),
    );
    assert!(engine().is_possible(&build));
    let specialization = rules::FeatKey::new("weapon-specialization");
    let seven = at(&build, 7);
    assert!(seven.has_feat_for(&rules::FeatKey::new("weapon-focus"), &weapon("longsword")));
    assert!(!seven.has_feat_for(&specialization, &weapon("longsword")));
    let eight = at(&build, 8);
    assert!(eight.has_feat_for(&specialization, &weapon("longsword")));
    assert!(!eight.has_feat_for(&specialization, &weapon("dagger")));
}

// ---- Crossblooded's arcana ------------------------------------------------------------------------------------

fn crossblooded(first: &str, second: &str, levels: usize) -> Build {
    with(
        &times("sorcerer", levels),
        Some(("sorcerer", "sorcerer/crossblooded")),
        &[
            (selection_slot(1, 0), first),
            (selection_slot(1, 1), second),
        ],
    )
}

#[test]
fn crossblooded_second_bloodline_gives_its_arcana_too() {
    let build = crossblooded(ARCANE, ABYSSAL, 3);
    assert!(engine().is_possible(&build));
    let at_1 = at(&build, 1);
    assert!(holds(&at_1, "sorcerer/bloodline/abyssal/arcana"));
    assert!(!holds(&at_1, "sorcerer/bloodline/abyssal/claws"));
    assert!(!holds(
        &at(&build, 3),
        "sorcerer/bloodline/abyssal/resistances"
    ));
    // The main bloodline gives everything (the Arcane one names no arcana here).
    assert!(holds(&at_1, "sorcerer/bloodline/arcane/arcane-bond"));
}

// ---- the Separatist's second domain ---------------------------------------------------------------------------

const FIRE_DOMAIN: &str = "cleric/domain/fire";
const WAR_DOMAIN: &str = "cleric/domain/war";

#[test]
fn a_separatists_second_domain_lags_two_levels_behind() {
    let build = with(
        &times("cleric", 10),
        Some(("cleric", "cleric/separatist")),
        &[
            (selection_slot(1, 0), FIRE_DOMAIN),
            (selection_slot(1, 1), WAR_DOMAIN),
        ],
    );
    assert!(engine().is_possible(&build));
    let at_1 = at(&build, 1);
    assert!(holds(&at_1, "cleric/domain/fire/fire-bolt"));
    assert!(!holds(&at_1, "cleric/domain/war/battle-rage"));
    assert_eq!(at_1.option_level(&option(WAR_DOMAIN)), 0);
    assert!(!holds(&at(&build, 2), "cleric/domain/war/battle-rage"));
    let at_3 = at(&build, 3);
    assert!(holds(&at_3, "cleric/domain/war/battle-rage"));
    assert_eq!(at_3.option_level(&option(WAR_DOMAIN)), 1);
    assert_eq!(at_3.option_level(&option(FIRE_DOMAIN)), 3);
    // War's bonus Combat feat (its row 8) at cleric 10.
    let sheet = engine().evaluate(&build);
    let war_slot = |level| SlotId::OptionFeat { level, index: 0 };
    assert!(sheet.slot(&war_slot(8)).is_none());
    assert!(sheet.slot(&war_slot(10)).is_some());
}

// ---- option levels from elsewhere -----------------------------------------------------------------------------

#[test]
fn impossible_domain_levels_its_domain_like_the_domains_held() {
    let mut build = with(
        &times("cleric", 8),
        None,
        &[(selection_slot(1, 0), FIRE_DOMAIN)],
    );
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("impossible-domain"),
    );
    let slot = SlotId::MythicSelection { rank: 1, index: 0 };
    build.picks.insert(slot.clone(), pick_option(WAR_DOMAIN));
    assert!(engine().is_possible(&build));
    let at_6 = at(&build, 6);
    assert!(holds(&at_6, "cleric/domain/war/battle-rage"));
    assert_eq!(at_6.option_level(&option(WAR_DOMAIN)), 6);
    assert_eq!(at(&build, 8).option_level(&option(WAR_DOMAIN)), 8);
    let sheet = engine().evaluate(&build);
    assert!(
        sheet
            .slot(&SlotId::OptionFeat { level: 8, index: 0 })
            .is_some()
    );
    // It offers every domain but the one held.
    assert_eq!(status(&build, &slot, FIRE_DOMAIN), Err(Issue::AlreadyHeld));
}

#[test]
fn a_maneuver_comes_at_its_revelations_option_level() {
    let mut build = with(
        &times("cleric", 7),
        None,
        &[
            (selection_slot(1, 0), FIRE_DOMAIN),
            (
                selection_slot(3, 0),
                "cleric/fire-revelations/maneuver-mastery",
            ),
        ],
    );
    let maneuver = SlotId::OptionSelection { level: 3, index: 0 };
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet.slot(&maneuver).unwrap().kind,
        SlotKind::Selection(rules::FeatureSelectionKey::new("cleric/maneuver-mastery"))
    );
    build
        .picks
        .insert(maneuver, pick_option("cleric/maneuver-mastery/trip"));
    assert!(engine().is_possible(&build));
    let combat_expertise = feat("combat-expertise");
    assert_eq!(at(&build, 6).feat_count(&combat_expertise), 0);
    let at_7 = at(&build, 7);
    assert_eq!(at_7.feat_count(&combat_expertise), 1);
    assert_eq!(
        at_7.option_level(&option("cleric/maneuver-mastery/trip")),
        at_7.option_level(&option("cleric/fire-revelations/maneuver-mastery"))
    );
}

// ---- a pool selection with options of its own -----------------------------------------------------------------

#[test]
fn a_pool_selection_also_offers_its_own_options() {
    // The Accursed's level-1 slots in key order: its revelation, then the domain.
    let build = with(
        &times("cleric", 3),
        Some(("cleric", "cleric/accursed")),
        &[(selection_slot(1, 1), FIRE_DOMAIN)],
    );
    let slot = selection_slot(3, 0);
    assert_eq!(
        offered(&build, &slot),
        [
            "cleric/accursed/revelation/misfortune",
            "cleric/fire-revelations/burning-hands",
            "cleric/fire-revelations/fire-breath",
            "cleric/fire-revelations/maneuver-mastery",
            "cleric/fire-revelations/weapon-mastery"
        ]
    );
    assert_eq!(
        status(&build, &slot, "cleric/accursed/revelation/misfortune"),
        Ok(())
    );
}

#[test]
fn a_pool_selection_is_checked_after_the_selection_it_draws_on() {
    // At level 1 the revelation's slot comes first by key, but it's filled after the domain's.
    let mut build = with(
        &["cleric"],
        Some(("cleric", "cleric/accursed")),
        &[
            (selection_slot(1, 1), FIRE_DOMAIN),
            (selection_slot(1, 0), "cleric/fire-revelations/fire-breath"),
        ],
    );
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let order: Vec<&SlotId> = sheet.levels[0].slots.iter().map(|slot| &slot.id).collect();
    assert_eq!(order[..2], [&selection_slot(1, 1), &selection_slot(1, 0)]);
    build.picks.remove(&selection_slot(1, 1));
    assert_eq!(
        why_not(&build, &selection_slot(1, 0)),
        None,
        "no domain, no pool"
    );
}

// ---- Impossible Domain for a Living Deity ---------------------------------------------------------------------

/// A False God witch (the world's Living Deity) of `levels` levels, granted the way at 1 (the selection's one
/// option), who picked Impossible Domain's War domain at MR 1.
fn false_god(levels: usize) -> Build {
    let mut build = build("elf", &times("witch", levels));
    build.class_choices.insert(
        class("witch"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("witch/false-god")),
            ..ClassChoice::default()
        },
    );
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("impossible-domain"),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        pick_option(WAR_DOMAIN),
    );
    build
}

#[test]
fn a_living_deitys_impossible_domain_runs_on_the_ways_levels_from_the_pick() {
    let build = false_god(9);
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let at_6 = sheet.at(6).unwrap();
    assert_eq!(at_6.option_level(&option(WAR_DOMAIN)), 6);
    assert_eq!(
        at_6.feature_count(&feature("cleric/domain/war/battle-rage")),
        1
    );
    assert_eq!(sheet.at(8).unwrap().option_level(&option(WAR_DOMAIN)), 8);
    // War's bonus Combat feat (its row 8) at witch 8, long before the way's own domain at 20.
    assert!(
        sheet
            .slot(&SlotId::OptionFeat { level: 8, index: 0 })
            .is_some()
    );
}
