//! Ticket 152, in the real data: every value without a game-data key of its own, and every metamagic feat, reads back
//! from the key a choice's value names it by (the test world has no metamagic feat; `rules`' `addresses` test makes
//! the kinds exhaustive).

use rules::{Faith, ParameterKind};

use crate::support::*;

#[test]
fn every_parameter_value_reads_back_from_its_key() {
    let engine = engine();
    for kind in [
        ParameterKind::Skill,
        ParameterKind::Weapon,
        ParameterKind::ExoticWeapon,
        ParameterKind::SpellSchool,
        ParameterKind::Energy,
        ParameterKind::ArmourType,
        ParameterKind::Metamagic,
        ParameterKind::Stat,
        ParameterKind::Element,
    ] {
        let values = engine.parameter_values(kind);
        assert!(!values.is_empty(), "no {kind:?} in the game data");
        for value in values {
            assert_eq!(
                engine.parameter_from_key(kind, value.value_key()),
                Some(value.clone()),
                "{kind:?}"
            );
        }
    }
}

#[test]
fn every_metamagic_feat_is_a_metamagic_value_by_its_own_key() {
    let engine = engine();
    let metamagic: Vec<&str> = DATA
        .feats
        .iter()
        .filter(|feat| feat.is_metamagic())
        .map(|feat| feat.key.as_str())
        .collect();
    assert!(metamagic.len() > 5, "{metamagic:?}");
    for key in metamagic {
        let value = engine
            .parameter_from_key(ParameterKind::Metamagic, key)
            .unwrap_or_else(|| panic!("{key} isn't a metamagic value"));
        assert_eq!(value.value_key(), key);
    }
}

#[test]
fn every_faith_reads_back_from_its_key() {
    let faiths = std::iter::once(Faith::Atheism).chain(
        DATA.deities
            .iter()
            .map(|deity| Faith::Deity(deity.key.clone())),
    );
    for faith in faiths {
        assert_eq!(Faith::from_key(faith.key()), Some(faith));
    }
}
