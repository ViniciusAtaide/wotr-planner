//! The caster line on the real data (ticket 119): the casting abilities, the bonuses of `bonuses.ron`, and the Max
//! DC run's Build (a Kitsune Sorcerer 20, Arcane bloodline, Second Bloodline Fey, Spell Focus, Greater and Mythic in
//! Enchantment, School Power Enchantment, Expanded Arsenal in Conjuration and Necromancy, the three Spell
//! Penetrations, Combat Casting), from its planner link.

use rules::{
    Ability, BonusAmount, BonusItem, BonusTarget, Build, CasterLine, ClassKey, Engine, SpellDc,
    SpellSchool, SpellScope, SpellTraits, Subschool,
};

use crate::support::*;

/// The Max DC run's Build, as its link encodes it (2026-11-09).
const MAX_DC_RUN: &str = "AdWOPUgDQRCFd3bPE2OyAQnYSNzGSi0sbKzU8xqDKIg_oEiWsKAGznCeKCkkWIigEBALU1gaTCdYqCjpVNAmYCFBEAwpIldYWCro7uFPKot0KebxzfDmzfCnyGpf4Gzwsxh96C98uNmel_qCEpue8AsHyqPtY_utHU3ddLtwNBfP5dxKkt4HZi-n7nqzW-mD2_UNQ0vtpheLummEu8pXbuI8_BaajCaO-Yzkw7bN5vj1c3DHzJ8M31RO85n3zCu-eJT50BlZcJZXLMFMKzbPLUfYrJTaYyN8jQ0ZbCDJHc7Gl-yYsIUNCBHAjZT6_C0hhAF-Sq_iWnsZDkhHVAFCWJ4i-Fel6XuK8J_8O1OJMtwHGAghnoJS5KHuoaZpylNVDRBUb3i78AU";

fn max_dc_run(engine: &Engine) -> Build {
    engine
        .open_link(MAX_DC_RUN)
        .expect("the Max DC run's link opens")
}

fn compulsion(metamagic: bool) -> SpellTraits {
    SpellTraits {
        school: SpellSchool::Enchantment,
        subschool: Some(Subschool::Compulsion),
        energy: None,
        metamagic,
    }
}

/// The line at each character level of a Build.
fn lines(engine: &Engine, build: &Build, spell: SpellTraits) -> Vec<Option<CasterLine>> {
    engine
        .evaluate(build)
        .levels
        .iter()
        .map(|level| engine.caster_line(level, spell))
        .collect()
}

fn parts(parts: &[rules::CasterPart]) -> Vec<(&str, i16)> {
    parts
        .iter()
        .map(|part| (part.name.as_str(), part.value))
        .collect()
}

#[test]
fn the_max_dc_runs_enchantment_compulsion_dc_at_20_with_its_breakdown() {
    let engine = engine();
    let build = max_dc_run(&engine);
    let line = lines(&engine, &build, compulsion(false))
        .pop()
        .flatten()
        .expect("a sorcerer 20 casts");

    assert_eq!(line.class, ClassKey::new("sorcerer"));
    assert_eq!((line.ability, line.caster_level), (Ability::Charisma, 20));
    assert_eq!(
        parts(&line.dc_parts),
        [
            ("Charisma", 7),
            ("Kitsune", 1),
            ("Greater Spell Focus (Enchantment)", 1),
            ("Spell Focus (Enchantment)", 1),
            ("Spell Focus (Mythic) (Enchantment)", 1),
            ("Fey Arcana", 2),
            ("School Power (Enchantment)", 2),
        ]
    );
    // 10 + spell level + 15.
    assert_eq!(
        line.dcs,
        (1..=9)
            .map(|spell_level| SpellDc {
                spell_level,
                dc: 25 + i16::from(spell_level),
            })
            .collect::<Vec<_>>()
    );
    // The Build holds Heighten Spell.
    assert_eq!(
        line.heightened,
        Some(SpellDc {
            spell_level: 9,
            dc: 34
        })
    );

    // With a metamagic feat, the Arcane bloodline's arcana adds 1: the run's "about DC 35".
    let metamagic = lines(&engine, &build, compulsion(true))
        .pop()
        .flatten()
        .unwrap();
    assert_eq!(metamagic.dcs[8].dc, 35);
    assert!(
        parts(&metamagic.dc_parts).contains(&("Arcane Arcana (Arcane)", 1)),
        "{:?}",
        metamagic.dc_parts
    );
}

#[test]
fn the_max_dc_runs_dcs_by_level_follow_its_picks() {
    let engine = engine();
    let build = max_dc_run(&engine);
    let ninth_or_highest: Vec<(u8, i16)> = lines(&engine, &build, compulsion(false))
        .iter()
        .map(|line| {
            let line = line.as_ref().expect("a sorcerer casts from level 1");
            let top = line.dcs.last().unwrap();
            (top.spell_level, top.dc)
        })
        .collect();

    // Level 1: Cha 20 (+5), Kitsune and Spell Focus: 10 + 1 + 7. Level 3: Greater Spell Focus. Level 6: 3rd-level
    // spells and Second Bloodline Fey's arcana (+2). Level 7: Spell Focus (Mythic) at MR 2. Level 8: Cha 22 (+6).
    // Level 15: School Power (+2). Level 16: Cha 24 (+7).
    assert_eq!(ninth_or_highest[0], (1, 18));
    assert_eq!(ninth_or_highest[2], (1, 19));
    assert_eq!(ninth_or_highest[5], (3, 23));
    assert_eq!(ninth_or_highest[6], (3, 24));
    assert_eq!(ninth_or_highest[7], (4, 26));
    assert_eq!(ninth_or_highest[14], (7, 31));
    assert_eq!(ninth_or_highest[15], (8, 33));
    assert_eq!(ninth_or_highest[19], (9, 34));
}

#[test]
fn the_max_dc_runs_expanded_arsenal_gives_conjuration_enchantments_spell_focus_feats() {
    let engine = engine();
    let build = max_dc_run(&engine);
    let conjuration = SpellTraits {
        school: SpellSchool::Conjuration,
        subschool: None,
        energy: None,
        metamagic: false,
    };
    let line = lines(&engine, &build, conjuration).pop().flatten().unwrap();

    assert_eq!(
        parts(&line.dc_parts),
        [
            ("Charisma", 7),
            (
                "Expanded Arsenal (Conjuration): Enchantment's Spell Focus feats",
                3
            ),
        ]
    );
}

#[test]
fn the_max_dc_runs_caster_level_against_spell_resistance_and_concentration() {
    let engine = engine();
    let build = max_dc_run(&engine);
    let line = lines(&engine, &build, compulsion(false))
        .pop()
        .flatten()
        .unwrap();

    // MR 10 with Greater Spell Penetration: Spell Penetration (Mythic) adds the full rank. Fey Magic at 15.
    assert_eq!(
        parts(&line.spell_penetration_parts),
        [
            ("Greater Spell Penetration", 2),
            ("Spell Penetration", 2),
            ("Spell Penetration (Mythic)", 10),
            ("Fey Magic", 2),
        ]
    );
    assert_eq!(line.spell_penetration, 36);
    assert_eq!(
        parts(&line.concentration_parts),
        [("Charisma", 7), ("Combat Casting", 4)]
    );
    assert_eq!(line.concentration, 31);
}

#[test]
fn every_class_that_casts_has_its_casting_ability() {
    let ability = |key: &str| {
        DATA.class(&ClassKey::new(key))
            .unwrap_or_else(|| panic!("no class {key}"))
            .casting_ability
    };

    assert_eq!(ability("sorcerer"), Some(Ability::Charisma));
    assert_eq!(ability("wizard"), Some(Ability::Intelligence));
    assert_eq!(ability("cleric"), Some(Ability::Wisdom));
    assert_eq!(ability("fighter"), None);
    assert_eq!(ability("dragon-disciple"), None);
    let archetype = |key: &str| {
        DATA.archetypes()
            .find(|archetype| archetype.key.as_str() == key)
            .unwrap_or_else(|| panic!("no archetype {key}"))
            .casting_ability
    };
    assert_eq!(
        archetype("sorcerer/sage-sorcerer"),
        Some(Ability::Intelligence)
    );
    assert_eq!(
        archetype("sorcerer/empyreal-sorcerer"),
        Some(Ability::Wisdom)
    );
}

#[test]
fn the_bonuses_cover_the_dc_runs_sources() {
    let has = |item: BonusItem, target: BonusTarget, amount: BonusAmount| {
        DATA.bonuses
            .iter()
            .any(|bonus| bonus.item == item && bonus.target == target && bonus.amount == amount)
    };
    let dc = BonusTarget::SpellDc;
    use BonusAmount::Flat;

    assert!(has(
        BonusItem::Feat(rules::FeatKey::new("spell-focus")),
        dc(SpellScope::Value),
        Flat(1)
    ));
    assert!(has(
        BonusItem::MythicFeat(rules::MythicFeatKey::new("expanded-arsenal")),
        dc(SpellScope::Value),
        BonusAmount::SpellFocusOfAnotherSchool
    ));
    assert!(has(
        BonusItem::Feature(rules::FeatureKey::new(
            "sorcerer/bloodline/infernal/infernal-arcana"
        )),
        dc(SpellScope::Subschool(Subschool::Charm)),
        Flat(2)
    ));
    assert!(has(
        BonusItem::Subrace(rules::SubraceKey::new("gnome/pyromaniac")),
        dc(SpellScope::Energy(rules::Energy::Fire)),
        Flat(1)
    ));
    assert!(has(
        BonusItem::Race(rules::RaceKey::new("oread")),
        dc(SpellScope::Energy(rules::Energy::Acid)),
        Flat(1)
    ));
    assert_eq!(
        DATA.bonuses
            .iter()
            .filter(
                |bonus| matches!(&bonus.item, BonusItem::SelectionOption(option)
                if option.as_str().starts_with("sorcerer/bloodline/arcane/school-power/"))
            )
            .count(),
        8
    );
}
