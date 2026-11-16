//! Ticket 125: a mythic ability takes a value (Favorite Metamagic: a metamagic feat), and three kinds of value came
//! after links had shipped: metamagic, stat (Stat Focus) and element (Ascendant Element). A pick may leave one of
//! those open, so a link made before the item took its value still opens, with the value left to choose.

use std::collections::BTreeSet;

use rules::{
    Build, DataIssue, Edit, Element, Engine, Feat, FeatCategory, GameData, Issue, MythicAbility,
    MythicAbilityGroup, MythicAbilityKey, Parameter, ParameterKind, Pick, Prerequisite, SlotId,
    SlotOption, Stat, encode_build,
};

use crate::support::*;

const FAVORITE_METAMAGIC: &str = "favorite-metamagic";
const ASCENDANT_ELEMENT: &str = "ascendant-element";
const STAT_FOCUS: &str = "stat-focus";

/// The test world with two metamagic feats, Favorite Metamagic and Ascendant Element (mythic abilities taking a
/// metamagic and an element), and its Stat Focus taking a stat.
fn world_with_pick_values() -> GameData {
    let mut data = world();
    let metamagic = |key: &str, name: &str| Feat {
        components: Vec::new(),
        key: feat(key),
        blueprint: None,
        name: name.into(),
        category: FeatCategory::Magic,
        summary: String::new(),
        guide: Some(rules::GuideRef(format!("feats#{key}"))),
        parameter: None,
        ranks: 1,
        selection: None,
        aliases: BTreeSet::new(),
        requires_proficiency: false,
        prerequisite: None,
    };
    data.feats.push(metamagic(
        "metamagic-empower-spell",
        "Metamagic (Empower Spell)",
    ));
    data.feats.push(metamagic(
        "metamagic-persistent-spell",
        "Metamagic (Persistent Spell)",
    ));
    // The world's Stat Focus takes a stat, as the real one does; without its Trickster prerequisite here.
    let stat_focus = data
        .feats
        .iter_mut()
        .find(|f| f.key == feat(STAT_FOCUS))
        .unwrap();
    stat_focus.parameter = Some(ParameterKind::Stat);
    stat_focus.prerequisite = None;
    let ability = |key: &str, name: &str, parameter| MythicAbility {
        components: Vec::new(),
        key: MythicAbilityKey::new(key),
        blueprint: None,
        name: name.into(),
        group: MythicAbilityGroup::Magic,
        summary: String::new(),
        guide: Some(rules::GuideRef(format!("mythic-paths#{key}"))),
        prerequisite: None,
        features: vec![],
        selection: None,
        plain_feature: false,
        parameter: Some(parameter),
        ranks: 1,
    };
    data.mythic_abilities.push(ability(
        FAVORITE_METAMAGIC,
        "Favorite Metamagic",
        ParameterKind::Metamagic,
    ));
    data.mythic_abilities.push(ability(
        ASCENDANT_ELEMENT,
        "Ascendant Element",
        ParameterKind::Element,
    ));
    data
}

fn pick_engine() -> Engine {
    engine_on(world_with_pick_values())
}

fn metamagic(key: &str) -> Parameter {
    Parameter::Metamagic(feat(key))
}

fn favorite(value: Option<Parameter>) -> Pick {
    match value {
        Some(value) => pick_mythic_ability_for(FAVORITE_METAMAGIC, value),
        None => pick_mythic_ability(FAVORITE_METAMAGIC),
    }
}

/// A fighter 20 on the Trickster path: Mythic ranks 1 to 10 reached.
fn mythic_fighter() -> Build {
    let mut build = build("human", &times("fighter", 20));
    build.mythic_path = Some(path("trickster"));
    build
}

fn options_at(engine: &Engine, build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine
        .options(build, slot)
        .unwrap_or_else(|| panic!("the Build has no slot {slot:?}"))
}

const MR_1: SlotId = SlotId::Mythic { rank: 1 };
const MR_3: SlotId = SlotId::Mythic { rank: 3 };

// ---- Favorite Metamagic: a metamagic feat ---------------------------------------------------------------------

#[test]
fn favorite_metamagic_is_offered_once_per_metamagic_feat() {
    let engine = pick_engine();
    let offered: Vec<Pick> = options_at(&engine, &mythic_fighter(), &MR_1)
        .into_iter()
        .filter(|option| option.pick.key() == FAVORITE_METAMAGIC)
        .inspect(|option| assert_eq!(option.status, Ok(()), "{:?}", option.pick))
        .map(|option| option.pick)
        .collect();
    // Only the feats whose key starts `metamagic-`, and no option without a value.
    assert_eq!(
        offered,
        [
            favorite(Some(metamagic("metamagic-empower-spell"))),
            favorite(Some(metamagic("metamagic-persistent-spell"))),
        ]
    );
}

#[test]
fn favorite_metamagic_taken_for_a_metamagic_names_it() {
    let engine = pick_engine();
    let mut build = mythic_fighter();
    let pick = favorite(Some(metamagic("metamagic-persistent-spell")));
    build.picks.insert(MR_1, pick.clone());
    assert!(engine.is_possible(&build));
    assert_eq!(
        engine.pick_name(&pick),
        "Favorite Metamagic (Persistent Spell)"
    );
    assert_eq!(engine.value_left_open(&pick), None);
    // The ability is held.
    let sheet = engine.evaluate(&build);
    assert!(
        sheet
            .at(20)
            .unwrap()
            .mythic_ability_count(&MythicAbilityKey::new(FAVORITE_METAMAGIC))
            > 0
    );
}

#[test]
fn favorite_metamagic_takes_no_feat_but_a_metamagic_and_no_other_kind() {
    let engine = pick_engine();
    for value in [
        metamagic("power-attack"),
        Parameter::Stat(Stat::ArmourClass),
        weapon("longsword"),
    ] {
        let mut build = mythic_fighter();
        build.picks.insert(MR_1, favorite(Some(value.clone())));
        // Not offered at all, and refused.
        assert_eq!(why_not_in(&engine, &build, &MR_1), None, "{value:?}");
    }
}

/// Ticket 249: a mythic ability that takes a value is taken again for another value, never twice for one; the game's
/// is a selection, which `BlueprintFeatureSelection.CanSelect` lets repeat, of one-rank features (WrathDecompiled
/// 6ce4a6b).
#[test]
fn favorite_metamagic_is_taken_again_for_another_metamagic_but_not_the_same() {
    let engine = pick_engine();
    let mut build = mythic_fighter();
    let persistent = favorite(Some(metamagic("metamagic-persistent-spell")));
    let empower = favorite(Some(metamagic("metamagic-empower-spell")));
    build.picks.insert(MR_1, persistent.clone());
    // The picker offers the other metamagic and refuses the held one.
    let options = options_at(&engine, &build, &MR_3);
    let status = |pick: &Pick| {
        options
            .iter()
            .find(|option| &option.pick == pick)
            .unwrap()
            .status
            .clone()
    };
    assert_eq!(status(&empower), Ok(()));
    assert_eq!(status(&persistent), Err(Issue::AlreadyHeld));

    let mut twice = build.clone();
    twice.picks.insert(MR_3, empower);
    assert!(engine.is_possible(&twice));
    let state = engine.evaluate(&twice).at(20).unwrap().clone();
    let held = |value: &str| {
        state.has_mythic_ability_for(
            &MythicAbilityKey::new(FAVORITE_METAMAGIC),
            &metamagic(value),
        )
    };
    assert!(held("metamagic-persistent-spell") && held("metamagic-empower-spell"));
    let agreement = rules::agreement::judge(&engine, &twice);
    assert!(
        agreement.disagreements.is_empty(),
        "{:?}",
        agreement.lines("twice")
    );

    let mut same = build;
    same.picks.insert(MR_3, persistent);
    assert_eq!(why_not_in(&engine, &same, &MR_3), Some(Issue::AlreadyHeld));
}

/// Ticket 249 review: a Path switch that takes the later valued pick away (Legend keeps the general picks of MR 1
/// and 2) leaves the older valueless one holding the ability, as it did before the valued pick.
#[test]
fn releasing_a_valued_pick_keeps_an_older_valueless_one_held() {
    let engine = pick_engine();
    let mut build = mythic_fighter();
    build.picks.insert(MR_1, favorite(None));
    build
        .picks
        .insert(MR_3, favorite(Some(metamagic("metamagic-empower-spell"))));
    assert!(engine.is_possible(&build));
    let key = MythicAbilityKey::new(FAVORITE_METAMAGIC);
    let before = engine.evaluate(&build);
    let before = before.at(20).unwrap();
    assert!(before.mythic_abilities().any(|held| held == &key));
    assert!(before.has_mythic_ability_for(&key, &metamagic("metamagic-empower-spell")));

    engine.apply(&mut build, &Edit::PathSwitch(Some(path("legend"))));
    let sheet = engine.evaluate(&build);
    let after = sheet.at(20).unwrap();
    // The valued pick is lost to the switch; the valueless one at MR 1 still holds the ability.
    assert!(!after.has_mythic_ability_for(&key, &metamagic("metamagic-empower-spell")));
    assert!(
        after.mythic_abilities().any(|held| held == &key),
        "{after:?}"
    );

    // A second valueless pick is refused as held (the picker doesn't offer a pick without a value).
    let mut again = mythic_fighter();
    again.picks.insert(MR_1, favorite(None));
    assert!(engine.is_possible(&again));
    again.picks.insert(MR_3, favorite(None));
    assert!(!engine.is_possible(&again));
}

#[test]
fn a_mythic_ability_that_takes_no_value_names_none() {
    let engine = pick_engine();
    let mut build = mythic_fighter();
    build.picks.insert(
        MR_1,
        pick_mythic_ability_for("mythic-charge", metamagic("metamagic-empower-spell")),
    );
    assert_eq!(why_not_in(&engine, &build, &MR_1), None);
}

// ---- A value left open: links made before the item took one ---------------------------------------------------

#[test]
fn favorite_metamagic_without_its_metamagic_is_held_with_the_value_left_open() {
    let engine = pick_engine();
    let mut build = mythic_fighter();
    build.picks.insert(MR_1, favorite(None));
    assert!(engine.is_possible(&build));
    assert_eq!(
        engine.value_left_open(&favorite(None)),
        Some(ParameterKind::Metamagic)
    );
    // Its options still offer every metamagic, the open value included: picking one names it.
    let options = options_at(&engine, &build, &MR_1);
    let persistent = favorite(Some(metamagic("metamagic-persistent-spell")));
    let offered = options
        .iter()
        .find(|option| option.pick == persistent)
        .unwrap();
    assert_eq!(offered.status, Ok(()));
}

#[test]
fn a_link_with_favorite_metamagic_and_no_metamagic_still_opens() {
    let engine = pick_engine();
    let mut build = mythic_fighter();
    build.picks.insert(MR_1, favorite(None));
    // Written with tag 2, as every link before ticket 125.
    assert_eq!(engine.open_link(&encode_build(&build)), Ok(build));
}

#[test]
fn a_link_with_favorite_metamagic_for_a_metamagic_round_trips() {
    let engine = pick_engine();
    let mut build = mythic_fighter();
    build.picks.insert(
        MR_1,
        favorite(Some(metamagic("metamagic-persistent-spell"))),
    );
    build.picks.insert(
        MR_3,
        pick_mythic_ability_for(
            ASCENDANT_ELEMENT,
            Parameter::Element(Element::NegativeEnergy),
        ),
    );
    build.picks.insert(
        general_slot(1),
        pick_feat_for(STAT_FOCUS, Parameter::Stat(Stat::AttacksOfOpportunity)),
    );
    assert_eq!(engine.open_link(&encode_build(&build)), Ok(build));
}

#[test]
fn a_weapon_focus_without_its_weapon_is_still_refused() {
    // Only the kinds added after links shipped may stay open.
    let engine = pick_engine();
    let mut build = build("human", &["fighter"]);
    build
        .picks
        .insert(general_slot(1), pick_feat("weapon-focus"));
    // Not offered without a weapon, and refused.
    assert_eq!(why_not_in(&engine, &build, &general_slot(1)), None);
    assert!(!ParameterKind::Weapon.may_stay_open());
}

// ---- Stat Focus and Ascendant Element --------------------------------------------------------------------------

#[test]
fn stat_focus_is_offered_once_per_stat_the_guide_lists() {
    let engine = pick_engine();
    let build = build("human", &["fighter"]);
    let stats: Vec<Parameter> = options_at(&engine, &build, &general_slot(1))
        .into_iter()
        .filter(|option| option.pick.key() == STAT_FOCUS)
        .map(|option| option.pick.parameter().cloned().unwrap())
        .collect();
    assert_eq!(stats, Stat::ALL.map(Parameter::Stat));
    assert_eq!(
        Stat::ALL.map(Stat::name),
        [
            "AC",
            "Attack",
            "Attacks of Opportunity",
            "CMB",
            "CMD",
            "Damage",
            "HP",
            "Initiative",
            "Fortitude",
            "Reflex",
            "Will",
            "Speed",
        ]
    );
}

#[test]
fn stat_focus_without_its_stat_opens_with_the_stat_left_open() {
    let engine = pick_engine();
    let mut build = build("human", &["fighter"]);
    build.picks.insert(general_slot(1), pick_feat(STAT_FOCUS));
    assert_eq!(engine.open_link(&encode_build(&build)), Ok(build));
    assert_eq!(
        engine.value_left_open(&pick_feat(STAT_FOCUS)),
        Some(ParameterKind::Stat)
    );
}

#[test]
fn ascendant_element_is_offered_once_per_element_the_guide_lists() {
    let engine = pick_engine();
    let elements: Vec<Parameter> = options_at(&engine, &mythic_fighter(), &MR_1)
        .into_iter()
        .filter(|option| option.pick.key() == ASCENDANT_ELEMENT)
        .map(|option| option.pick.parameter().cloned().unwrap())
        .collect();
    assert_eq!(elements, Element::ALL.map(Parameter::Element));
    assert_eq!(
        engine.pick_name(&pick_mythic_ability_for(
            ASCENDANT_ELEMENT,
            Parameter::Element(Element::NegativeEnergy)
        )),
        "Ascendant Element (Negative energy)"
    );
}

// ---- The data check ---------------------------------------------------------------------------------------------

#[test]
fn only_a_pick_names_a_metamagic_stat_or_element() {
    let mut data = world_with_pick_values();
    let dodge = data
        .feats
        .iter_mut()
        .find(|f| f.key == feat("dodge"))
        .unwrap();
    dodge.prerequisite = Some(Prerequisite::FeatFor {
        feat: feat(STAT_FOCUS),
        value: Parameter::Stat(Stat::ArmourClass),
    });
    let issues: Vec<DataIssue> = data.check().into_iter().map(|p| p.issue).collect();
    assert!(issues.contains(&DataIssue::PickOnlyValue), "{issues:?}");
    assert_eq!(world_with_pick_values().check(), []);
}

#[test]
fn a_metamagic_kind_lists_only_metamagic_feats() {
    let engine = pick_engine();
    assert_eq!(
        engine.parameter_values(ParameterKind::Metamagic),
        [
            metamagic("metamagic-empower-spell"),
            metamagic("metamagic-persistent-spell"),
        ]
    );
}

/// [`why_not`] in `engine`: why the pick in `slot` can't be taken there, for a Build that only that pick makes
/// impossible, as the slot's picker says, or `None` when the picker doesn't offer it at all.
fn why_not_in(engine: &Engine, build: &Build, slot: &SlotId) -> Option<Issue> {
    assert!(!engine.is_possible(build), "the Build is possible");
    let rest = without(build, slot);
    assert!(engine.is_possible(&rest), "impossible without {slot:?} too");
    let pick = &build.picks[slot];
    let status = engine
        .options(&rest, slot)?
        .into_iter()
        .find(|option| &option.pick == pick)?
        .status;
    Some(status.expect_err("the picker offers it"))
}
