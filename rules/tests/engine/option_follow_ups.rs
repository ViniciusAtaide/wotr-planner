//! Option follow-ups (ticket 33): option groups, Crossblooded's second arcana, Expanded Element, a selection
//! lagging behind its classes, option levels from the other options held or from the option that granted the
//! slot, a pool selection with options of its own, and a prerequisite on the selection picked through.

use rules::{
    Alignment, Build, CharacterState, ClassChoice, Issue, OptionGroup, Prerequisite, SlotId,
    SlotKind, Unmet,
};

use crate::support::*;

/// `classes` in order, with `archetype` on `of` and option `picks` at their slots.
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
const GOLD: &str = "sorcerer/bloodline/draconic-gold";
const RED: &str = "sorcerer/bloodline/draconic-red";

fn one_draconic() -> Issue {
    Issue::Unmet(Unmet::Condition {
        needs: Prerequisite::NoOtherOptionOfGroup(OptionGroup::Draconic),
        has: None,
    })
}

// ---- option groups ------------------------------------------------------------------------------------------

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
fn a_crossblooded_sorcerer_cant_take_a_second_draconic_bloodline() {
    let build = crossblooded(GOLD, RED, 1);
    assert_eq!(
        status(&build, &selection_slot(1, 1), RED),
        Err(one_draconic())
    );
    assert_eq!(status(&build, &selection_slot(1, 1), ABYSSAL), Ok(()));
    // The first pick saw no other dragon.
    assert_eq!(status(&build, &selection_slot(1, 0), GOLD), Ok(()));
    let problems = engine().validate(&build);
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].at, rules::Choice::Slot(selection_slot(1, 1)));
    assert_eq!(problems[0].issue, one_draconic());
}

#[test]
fn second_bloodline_cant_take_a_second_draconic_bloodline() {
    let mut build = with(&times("sorcerer", 6), None, &[(selection_slot(1, 0), RED)]);
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("second-bloodline"),
    );
    let slot = SlotId::MythicSelection { rank: 1, index: 0 };
    assert_eq!(status(&build, &slot, GOLD), Err(one_draconic()));
    assert_eq!(status(&build, &slot, ABYSSAL), Ok(()));
}

#[test]
fn an_option_of_a_group_is_held_in_the_states_groups() {
    let state = at(&crossblooded(RED, ABYSSAL, 1), 1);
    assert_eq!(
        state.option_groups,
        [(option(RED), OptionGroup::Draconic)].into()
    );
    assert!(state.holds_other_option_of(OptionGroup::Draconic, Some(&option(GOLD))));
    assert!(!state.holds_other_option_of(OptionGroup::Draconic, Some(&option(RED))));
}

// ---- Crossblooded's arcana ------------------------------------------------------------------------------------

#[test]
fn crossblooded_second_bloodline_gives_its_arcana_too() {
    let build = crossblooded(ARCANE, ABYSSAL, 3);
    assert_eq!(engine().validate(&build), []);
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

// ---- Expanded Element -----------------------------------------------------------------------------------------

const AIR: &str = "kineticist/element/air";
const FIRE: &str = "kineticist/element/fire";
const WATER: &str = "kineticist/element/water";

fn kineticist(first: &str, seventh: &str) -> Build {
    with(
        &times("kineticist", 15),
        None,
        &[
            (selection_slot(1, 0), first),
            (selection_slot(7, 0), seventh),
        ],
    )
}

#[test]
fn expanded_element_at_15_cant_take_the_7th_level_element_again() {
    let build = kineticist(AIR, FIRE);
    assert_eq!(engine().validate(&build), []);
    let fifteenth = selection_slot(15, 0);
    assert_eq!(status(&build, &fifteenth, FIRE), Err(Issue::AlreadyHeld));
    // The 1st-level element (a second time) or a third.
    assert_eq!(status(&build, &fifteenth, AIR), Ok(()));
    assert_eq!(status(&build, &fifteenth, WATER), Ok(()));
    assert!(at(&build, 7).holds_through(
        &rules::FeatureSelectionKey::new("kineticist/expanded-element"),
        &option(FIRE)
    ));
}

#[test]
fn an_element_doubled_at_7_isnt_taken_a_third_time() {
    let build = kineticist(AIR, AIR);
    assert_eq!(engine().validate(&build), []);
    let fifteenth = selection_slot(15, 0);
    assert_eq!(
        status(&build, &fifteenth, AIR),
        Err(Issue::MaxTimes { times: 2 })
    );
    assert_eq!(status(&build, &fifteenth, FIRE), Ok(()));
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
    assert_eq!(engine().validate(&build), []);
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
    assert_eq!(engine().validate(&build), []);
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
    assert_eq!(engine().validate(&build), []);
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
    assert_eq!(engine().validate(&build), []);
    let sheet = engine().evaluate(&build);
    let order: Vec<&SlotId> = sheet.levels[0].slots.iter().map(|slot| &slot.id).collect();
    assert_eq!(order[..2], [&selection_slot(1, 1), &selection_slot(1, 0)]);
    build.picks.remove(&selection_slot(1, 1));
    assert_eq!(engine().validate(&build).len(), 1, "no domain, no pool");
}

// ---- the selection picked through -----------------------------------------------------------------------------

const GOOD: &str = "warpriest/blessings/good";

fn champion(alignment: Alignment) -> Build {
    let mut build = with(
        &["warpriest"],
        Some(("warpriest", "warpriest/champion")),
        &[(selection_slot(1, 0), GOOD)],
    );
    build.start.alignment = Some(alignment);
    build
}

#[test]
fn a_champions_alignment_blessing_matches_his_alignment() {
    let neutral = champion(Alignment::TrueNeutral);
    assert!(matches!(
        status(&neutral, &selection_slot(1, 0), GOOD),
        Err(Issue::Unmet(_))
    ));
    assert_eq!(
        status(&neutral, &selection_slot(1, 0), "warpriest/blessings/war"),
        Ok(())
    );
    assert_eq!(engine().validate(&champion(Alignment::NeutralGood)), []);
}

#[test]
fn a_warpriests_blessing_doesnt_check_his_alignment() {
    let build = with(
        &times("warpriest", 2),
        None,
        &[(selection_slot(2, 0), GOOD)],
    );
    assert_eq!(build.start.alignment, Some(Alignment::TrueNeutral));
    assert_eq!(engine().validate(&build), []);
}

// ---- the slayer's combat style (ticket 32's decision 2) ------------------------------------------------------

fn slayer_talent(stage: u8) -> rules::Pick {
    pick_option(&format!("slayer/slayer-talents/combat-style-feat-{stage}"))
}

fn option_feat(level: u8) -> SlotId {
    SlotId::OptionFeat { level, index: 0 }
}

fn style_slot(level: u8) -> SlotId {
    SlotId::OptionSelection { level, index: 0 }
}

#[test]
fn a_slayer_picks_one_ranger_style_with_his_first_combat_style_feat() {
    let mut build = build("elf", &times("slayer", 6));
    build.picks.insert(selection_slot(2, 0), slayer_talent(2));
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet.slot(&style_slot(2)).unwrap().kind,
        SlotKind::Selection(rules::FeatureSelectionKey::new("slayer/combat-style"))
    );
    assert_eq!(
        offered(&build, &style_slot(2)),
        ["ranger/combat-style/archery", "ranger/combat-style/crane"]
    );
    build
        .picks
        .insert(style_slot(2), pick_option("ranger/combat-style/archery"));
    // The feat comes from the style's list of the stage.
    assert_eq!(offered(&build, &option_feat(2)), ["dodge", "toughness"]);
    build.picks.insert(option_feat(2), pick_feat("crane-wing"));
    assert_eq!(
        engine().validate(&build),
        [rules::Problem {
            at: rules::Choice::Slot(option_feat(2)),
            issue: Issue::NotAllowed
        }]
    );
    build.picks.insert(option_feat(2), pick_feat("toughness"));
    assert_eq!(engine().validate(&build), []);
    // The second talent picks no style: the feat comes from the same style's next list.
    build.picks.insert(selection_slot(6, 0), slayer_talent(6));
    let sheet = engine().evaluate(&build);
    assert!(sheet.slot(&style_slot(6)).is_none());
    assert_eq!(
        offered(&build, &option_feat(6)),
        ["dodge", "great-cleave", "toughness"]
    );
}
