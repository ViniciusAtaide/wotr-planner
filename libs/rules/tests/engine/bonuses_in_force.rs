//! The bonuses in force (ticket 151; from components since ticket 297): the walk records them on each state of the
//! sheet, with what each adds at that level, so the caster line and the totals read a level of the sheet alone.

use std::sync::Arc;

use rules::{
    BonusInForce, BonusSource, BonusTarget, Build, Component, Condition, Descriptor, Engine, Fact,
    Holder, InForceTarget, Parameter, RankBase, RankConfig, RankProgression, SpellSchool,
    SpellScope, Value,
};

use crate::support::*;

/// The test world's bonuses, and toughness giving half the Mythic rank to spell penetration once the character has 4
/// sorcerer levels.
fn engine_with_toughness() -> Engine {
    let mut data = world();
    let toughness = data.feats.iter_mut().find(|f| f.key == feat("toughness"));
    toughness
        .unwrap()
        .components
        .push(Component::SpellPenetration {
            value: Value::Rank(RankConfig {
                base: RankBase::MythicLevel,
                progression: RankProgression::Div2,
                min: None,
                max: None,
            }),
            only_while: Some(Condition::ClassLevel(class("sorcerer"), 4)),
        });
    engine_on(data)
}

/// A human sorcerer of 9 levels with Spell Focus (Evocation) and Toughness at 1 and Spell Penetration at 5.
fn sorcerer() -> Build {
    let mut build = build("human", &times("sorcerer", 9));
    build.picks.insert(
        general_slot(1),
        pick_feat_for(
            "spell-focus",
            Parameter::SpellSchool(SpellSchool::Evocation),
        ),
    );
    build.picks.insert(race_slot(0), pick_feat("toughness"));
    build
        .picks
        .insert(general_slot(5), pick_feat("spell-penetration"));
    build
}

/// The bonuses in force at a level: each one's name and amount, in the order recorded.
fn in_force(engine: &Engine, build: &Build, level: u8) -> Vec<(String, i16)> {
    walked(engine, build)
        .at(level)
        .expect("a level of the Build")
        .bonuses_in_force
        .iter()
        .map(|bonus| (bonus.name.clone(), bonus.amount))
        .collect()
}

fn named(names: &[(&str, i16)]) -> Vec<(String, i16)> {
    names
        .iter()
        .map(|(name, amount)| ((*name).to_owned(), *amount))
        .collect()
}

#[test]
fn each_state_of_the_sheet_records_the_bonuses_held_there_with_their_item_value_and_target() {
    let engine = engine_with_toughness();
    let sheet = walked(&engine, &sorcerer());

    assert_eq!(
        sheet.at(1).unwrap().bonuses_in_force[..],
        [BonusInForce {
            source: BonusSource::Component(Holder::Fact(Fact::Feat(feat("spell-focus")))),
            value: Some(Parameter::SpellSchool(SpellSchool::Evocation)),
            target: InForceTarget::Number(BonusTarget::SpellDc(SpellScope::School(
                SpellSchool::Evocation
            ))),
            amount: 1,
            descriptor: Descriptor::UntypedStackable,
            name: "spell-focus (Evocation)".to_owned(),
        }]
    );
    // Spell Penetration from the level it's picked at.
    assert_eq!(
        in_force(&engine, &sorcerer(), 4).len(),
        2,
        "Spell Focus, and Toughness once its requirement holds"
    );
    assert!(
        in_force(&engine, &sorcerer(), 5).contains(&("spell-penetration".to_owned(), 2)),
        "{:?}",
        in_force(&engine, &sorcerer(), 5)
    );
}

#[test]
fn a_bonus_is_in_force_once_its_requirement_holds_with_its_amount_at_each_level() {
    let engine = engine_with_toughness();
    let build = sorcerer();
    let toughness = |level| {
        in_force(&engine, &build, level)
            .into_iter()
            .find(|(name, _)| name == "toughness")
            .map(|(_, amount)| amount)
    };

    // Not before sorcerer 4; then half the Mythic rank: MR 1 at 6, 2 at 7, 3 at 9.
    assert_eq!(toughness(3), None);
    assert_eq!(
        [4, 6, 7, 9].map(toughness),
        [Some(0), Some(0), Some(1), Some(1)]
    );
    assert_eq!(
        in_force(&engine, &build, 9),
        named(&[
            ("spell-focus (Evocation)", 1),
            ("spell-penetration", 2),
            ("toughness", 1),
        ])
    );
}

#[test]
fn a_level_with_the_last_levels_bonuses_shares_its_list_and_one_that_changes_keeps_the_rest() {
    let engine = engine_with_toughness();
    let sheet = walked(&engine, &sorcerer());
    let list = |level: usize| &sheet.levels[level - 1].state.bonuses_in_force;

    // Toughness's half Mythic rank: 0 at 6 (MR 1), 1 at 7 (MR 2), still 1 at 8 and 9 (MR 3).
    assert!(!Arc::ptr_eq(list(6), list(7)));
    assert!(Arc::ptr_eq(list(7), list(8)) && Arc::ptr_eq(list(8), list(9)));
    // The new list keeps Spell Focus and Spell Penetration as they were.
    assert_eq!(list(6)[..2], list(7)[..2]);
    assert_eq!((list(6)[2].amount, list(7)[2].amount), (0, 1));
}
