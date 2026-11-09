//! The guide's caster level and spell DC items on the real data (ticket 122, `gear.ron`), from the two "Caster Level
//! and Spell DC Boosting Items" tables of the saved `items` page, and the Max DC run's Build with gear assumed on top.

use std::collections::BTreeMap;

use rules::{
    Ability, Assumed, Build, ClassKey, Engine, GearBonus, GearCircumstance, GearKey, GearSlot,
    GearTarget, Save, SpellSchool, SpellScope, SpellTraits, Subschool,
};

use crate::support::*;

/// The Max DC run's Build, as its link encodes it (2026-11-09; the same as `caster_line.rs`'s).
const MAX_DC_RUN: &str = "AdWOPUgDQRCFd3bPE2OyAQnYSNzGSi0sbKzU8xqDKIg_oEiWsKAGznCeKCkkWIigEBALU1gaTCdYqCjpVNAmYCFBEAwpIldYWCro7uFPKot0KebxzfDmzfCnyGpf4Gzwsxh96C98uNmel_qCEpue8AsHyqPtY_utHU3ddLtwNBfP5dxKkt4HZi-n7nqzW-mD2_UNQ0vtpheLummEu8pXbuI8_BaajCaO-Yzkw7bN5vj1c3DHzJ8M31RO85n3zCu-eJT50BlZcJZXLMFMKzbPLUfYrJTaYyN8jQ0ZbCDJHc7Gl-yYsIUNCBHAjZT6_C0hhAF-Sq_iWnsZDkhHVAFCWJ4i-Fel6XuK8J_8O1OJMtwHGAghnoJS5KHuoaZpylNVDRBUb3i78AU";

fn bonuses(key: &str) -> Vec<GearBonus> {
    DATA.gear
        .iter()
        .find(|item| item.key == GearKey::new(key))
        .unwrap_or_else(|| panic!("{key} is in gear.ron"))
        .bonuses
        .clone()
}

fn bonus(target: GearTarget, amount: i8, circumstance: GearCircumstance) -> GearBonus {
    GearBonus {
        target,
        amount,
        circumstance,
    }
}

const ALL: GearCircumstance = GearCircumstance::Spells(SpellScope::All);

#[test]
fn the_items_are_the_tables_rows_each_citing_the_items_page() {
    // 10 caster level rows and 31 DC rows, three items in both, six left out (their circumstance isn't one the
    // caster line can tell).
    assert_eq!(DATA.gear.len(), 32);
    assert!(DATA.gear.iter().all(|item| item.guide.0 == "items"));
    for left_out in [
        "delameres-armor",
        "natures-agony",
        "voice-of-the-fearless",
        "amulet-of-blackened-mirror",
        "legacy-of-the-last-azlanti",
        "cloak-of-the-ancient-herald",
    ] {
        assert!(
            DATA.gear.iter().all(|item| item.key.as_str() != left_out),
            "{left_out}"
        );
    }

    assert_eq!(
        bonuses("heartstone-of-the-night-hag-queen"),
        [bonus(GearTarget::SpellDc, 3, ALL)]
    );
    assert_eq!(
        bonuses("rod-of-power-source"),
        [bonus(
            GearTarget::SpellDc,
            4,
            GearCircumstance::Spells(SpellScope::School(SpellSchool::Conjuration))
        )]
    );
    assert_eq!(
        bonuses("sin-mages-staff"),
        [
            bonus(GearTarget::SpellDc, 1, ALL),
            bonus(GearTarget::CasterLevel, 3, ALL)
        ]
    );
    assert_eq!(
        bonuses("robe-of-determination"),
        [
            bonus(
                GearTarget::SpellDc,
                2,
                GearCircumstance::Save(Save::Fortitude)
            ),
            bonus(GearTarget::SpellPenetration, 2, ALL),
            bonus(GearTarget::Concentration, 4, ALL),
        ]
    );
    assert_eq!(
        bonuses("goggles-of-mind-control"),
        [
            bonus(GearTarget::SpellDc, 2, GearCircumstance::MindAffecting),
            bonus(GearTarget::CasterLevel, 2, GearCircumstance::MindAffecting),
        ]
    );
    assert_eq!(
        bonuses("web-strider"),
        [bonus(
            GearTarget::CasterLevel,
            1,
            GearCircumstance::Class(ClassKey::new("druid"))
        )]
    );
    assert_eq!(
        bonuses("robe-of-water"),
        [bonus(
            GearTarget::SpellDc,
            2,
            GearCircumstance::Spells(SpellScope::Energy(rules::Energy::Cold))
        )]
    );
    let slot = |key: &str| {
        DATA.gear
            .iter()
            .find(|item| item.key == GearKey::new(key))
            .map(|item| item.slot)
    };
    assert_eq!(slot("mindmasters-eyes"), Some(GearSlot::Glasses));
    assert_eq!(slot("twisted-temptation"), Some(GearSlot::Gloves));
}

fn max_dc_run(engine: &Engine) -> Build {
    engine
        .open_link(MAX_DC_RUN)
        .expect("the Max DC run's link opens")
}

fn parts(parts: &[rules::CasterPart]) -> Vec<(&str, i16)> {
    parts
        .iter()
        .map(|part| (part.name.as_str(), part.value))
        .collect()
}

#[test]
fn the_max_dc_runs_compulsion_dc_with_a_headband_tomes_and_enchantment_items_assumed() {
    let engine = engine();
    let build = max_dc_run(&engine);
    let last = &engine.evaluate(&build).levels[19].state;
    let compulsion = SpellTraits {
        school: SpellSchool::Enchantment,
        subschool: Some(Subschool::Compulsion),
        energy: None,
        metamagic: false,
        save: Some(Save::Will),
        mind_affecting: true,
    };
    let assumed = Assumed {
        enhancement: BTreeMap::from([(Ability::Charisma, 8)]),
        inherent: BTreeMap::from([(Ability::Charisma, 4)]),
        items: [
            "heartstone-of-the-night-hag-queen",
            "mindmasters-eyes",
            "twisted-temptation",
            "robe-of-the-seven-sins",
            "rod-of-power-source",
        ]
        .map(GearKey::new)
        .to_vec(),
        ..Assumed::default()
    };
    let own = engine
        .caster_line(&build.start, last, compulsion)
        .expect("a sorcerer 20 casts");
    let line = engine
        .assumed_caster_line(&build.start, last, compulsion, &assumed)
        .expect("a sorcerer 20 casts");

    // Charisma 25 (+7) raised to 37 (+13); the rod's DC is for conjuration only.
    assert_eq!(own.dcs[8].dc, 34);
    assert_eq!(
        parts(&line.dc_parts),
        [
            ("Charisma", 13),
            ("Kitsune", 1),
            ("Greater Spell Focus (Enchantment)", 1),
            ("Spell Focus (Enchantment)", 1),
            ("Spell Focus (Mythic) (Enchantment)", 1),
            ("Fey Arcana", 2),
            ("School Power (Enchantment)", 2),
            ("Heartstone of the Night Hag Queen (assumed)", 3),
            ("Mindmaster's Eyes (assumed)", 2),
            ("Twisted Temptation (assumed)", 2),
            ("Robe of the Seven Sins (assumed)", 1),
        ]
    );
    assert_eq!(line.dcs[8].dc, 10 + 9 + 29);
    // Caster level 20 + the robe's 3; against spell resistance 36 + 3 + the robe's 5; concentration 31 + 3 + 6.
    assert_eq!(line.caster_level, 23);
    assert_eq!(line.spell_penetration, 44);
    assert_eq!(line.concentration, 40);
    // The Build itself is unchanged: its scores, and so its prerequisites, are its own.
    assert_eq!(last.ability_scores.get(Ability::Charisma), 25);
}
