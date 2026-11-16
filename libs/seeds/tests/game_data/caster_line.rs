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

/// The Max DC run's line at 20 for an Enchantment (Compulsion) spell, with and without metamagic.
fn max_dc_run_lines_at_20(engine: &Engine) -> (CasterLine, CasterLine) {
    let build = max_dc_run(engine);
    let line = |metamagic| {
        lines(engine, &build, compulsion(metamagic))
            .pop()
            .flatten()
            .expect("a sorcerer 20 casts")
    };
    (line(false), line(true))
}

#[test]
fn the_max_dc_runs_enchantment_compulsion_dc_at_20_with_its_breakdown() {
    let engine = engine();
    let (line, metamagic) = max_dc_run_lines_at_20(&engine);

    assert_eq!(line.class, ClassKey::new("sorcerer"));
    // Game: SorcererSpellbook b3db3766a4b605040b366265e2af0e50 (CastingAttribute Charisma)
    assert_eq!((line.ability, line.caster_level), (Ability::Charisma, 20));
    // The parts but the Spell Focus feats' (whose values wait on a ticket, see below), exactly.
    let dc_parts: Vec<(&str, i16)> = parts(&line.dc_parts)
        .into_iter()
        .filter(|(name, _)| !name.contains("Spell Focus"))
        .collect();
    assert_eq!(
        dc_parts,
        [
            ("Charisma", 7),
            ("Kitsune", 1), // Game: KitsuneMagic 0b71c1a7941bfa4429e281245371efc6 (+1 Enchantment)
            ("Fey Arcana", 2), // Game: BloodlineFeyArcana 6fe1c3f9437d88749a07e294c08799b9 (+2 Compulsion)
            ("School Power (Enchantment)", 2), // Game: BloodlineArcaneSchoolPowerFeature 8891303b67eb273428f1691864b08cf8
        ]
    );

    // With a metamagic feat, the Arcane bloodline's arcana adds 1.
    // Game: ArcaneBloodlineArcana.OnEventAboutToTrigger, WrathDecompiled 6ce4a6b; BloodlineArcaneArcana
    // e8e4f56618dd8b04490aa6a0b75ac24f
    assert!(
        parts(&metamagic.dc_parts).contains(&("Arcane Arcana (Arcane)", 1)),
        "{:?}",
        metamagic.dc_parts
    );
}

/// Spell Focus (Mythic) doubles each Spell Focus and Greater Spell Focus of its school: the Max DC run's focus feats
/// add 4 (the Mythic feat no bonus of its own), so its DCs are 26 + the spell level, a heightened 9th 35 and a 9th
/// with metamagic 36.
// Game: SpellFocusParametrized.GetBonusDC, WrathDecompiled 6ce4a6b (BonusDC × 2 when MythicFocus is held for the
// school); SpellFocus 16fa59cc9a72a6043b566b49184f53fe; SpellFocusGreater 5b04b45b228461c43bad768eb0f7c7bf;
// SpellFocusMythicFeat 41fa2470ab50ff441b4cfbb2fc725109
#[test]
#[ignore = "waits on ticket 258: the game gives the Max DC run's focus feats +4 and its DCs 26 + spell level, 35 heightened, 36 with metamagic"]
fn mythic_spell_focus_doubles_the_max_dc_runs_spell_focus_feats() {
    let engine = engine();
    let (line, metamagic) = max_dc_run_lines_at_20(&engine);
    let focus: i16 = line
        .dc_parts
        .iter()
        .filter(|part| part.name.contains("Spell Focus"))
        .map(|part| part.value)
        .sum();
    assert_eq!(focus, 4);
    assert_eq!(
        line.dcs,
        (1..=9)
            .map(|spell_level| SpellDc {
                spell_level,
                dc: 26 + i16::from(spell_level),
            })
            .collect::<Vec<_>>()
    );
    // The Build holds Heighten Spell.
    assert_eq!(
        line.heightened,
        Some(SpellDc {
            spell_level: 9,
            dc: 35
        })
    );
    assert_eq!(metamagic.dcs[8].dc, 36);
}

/// The Max DC run's highest spell level and its Enchantment (Compulsion) DC, at each character level.
fn ninth_or_highest_by_level() -> Vec<(u8, i16)> {
    let engine = engine();
    let build = max_dc_run(&engine);
    lines(&engine, &build, compulsion(false))
        .iter()
        .map(|line| {
            let line = line.as_ref().expect("a sorcerer casts from level 1");
            let top = line.dcs.last().unwrap();
            (top.spell_level, top.dc)
        })
        .collect()
}

#[test]
fn the_max_dc_runs_dcs_by_level_follow_its_picks() {
    let ninth_or_highest = ninth_or_highest_by_level();

    // Level 1: Cha 20 (+5), Kitsune and Spell Focus: 10 + 1 + 7. Level 3: Greater Spell Focus. Level 6: 3rd-level
    // spells and Second Bloodline Fey's arcana (+2). From level 7 (Spell Focus (Mythic) at MR 2) the game's values
    // wait on a ticket (`the_max_dc_runs_dcs_from_level_7_count_mythic_spell_focus_doubled`).
    // Game: SpellFocus 16fa59cc9a72a6043b566b49184f53fe; SpellFocusGreater 5b04b45b228461c43bad768eb0f7c7bf;
    // KitsuneMagic 0b71c1a7941bfa4429e281245371efc6; BloodlineFeyArcana 6fe1c3f9437d88749a07e294c08799b9
    assert_eq!(ninth_or_highest[0], (1, 18));
    assert_eq!(ninth_or_highest[2], (1, 19));
    assert_eq!(ninth_or_highest[5], (3, 23));
}

/// From level 7, when Spell Focus (Mythic) doubles the two focus feats, each DC is one more than the data's: 25 at
/// 7, 27 at 8 (Cha 22), 32 at 15 (School Power), 34 at 16 (Cha 24) and 35 at 20.
// Game: SpellFocusParametrized.GetBonusDC, WrathDecompiled 6ce4a6b; SpellFocusMythicFeat
// 41fa2470ab50ff441b4cfbb2fc725109
#[test]
#[ignore = "waits on ticket 258: the game gives the Max DC run 25, 27, 32, 34 and 35 at levels 7, 8, 15, 16 and 20"]
fn the_max_dc_runs_dcs_from_level_7_count_mythic_spell_focus_doubled() {
    let ninth_or_highest = ninth_or_highest_by_level();
    assert_eq!(ninth_or_highest[6], (3, 25));
    assert_eq!(ninth_or_highest[7], (4, 27));
    assert_eq!(ninth_or_highest[14], (7, 32));
    assert_eq!(ninth_or_highest[15], (8, 34));
    assert_eq!(ninth_or_highest[19], (9, 35));
}

/// Expanded Arsenal gives a spell of its school the largest sum of another school's Spell Focus feats (Enchantment's,
/// doubled by Spell Focus (Mythic): 4), and each of the Build's two Expanded Arsenal facts (Conjuration and
/// Necromancy) adds it, since each checks the shared list of arsenal schools, not its own school: 8.
// Game: ExpandedArsenalMagicSchools.OnEventAboutToTrigger and SpellFocusParametrized.GetBonusDC, WrathDecompiled
// 6ce4a6b; ExpandedArsenalSchool f137089c48364014aa3ec3b92ccaf2e2. That the two choices are two facts, each with its
// handler, is read from the code's per-fact component.
#[test]
#[ignore = "waits on ticket 258: the game gives the Max DC run's Conjuration spells +8 from Expanded Arsenal"]
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
    // Game: SorcererSpellbook b3db3766a4b605040b366265e2af0e50 (Charisma)
    assert_eq!(parts(&line.dc_parts)[0], ("Charisma", 7));
    let arsenal: i16 = line.dc_parts[1..].iter().map(|part| part.value).sum();
    assert_eq!(arsenal, 8);
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
    // Game: SpellPenetration ee7dc126939e4d9438357fbd5980d459; GreaterSpellPenetration 1978c3f91cfbbc24b9c9b0d017f4beec;
    // SpellPenetrationMythicFeat 51b6b22ff184eef46a675449e837365d and SpellPenetrationMythicFeatUpgrade
    // 544056cc5aca4d03823e13c5f8873c88 (the mythic rank with Greater Spell Penetration); BloodlineFeyFeyMagicFeature
    // a9ffe3e08a080c8478bae321e70c7de6; CombatCasting 06964d468fde1dc4aa71a92ea04d930d (+4 concentration)
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

    // Game: SorcererSpellbook b3db3766a4b605040b366265e2af0e50 (Charisma); WizardSpellbook
    // 5a38c9ac8607890409fcb8f6342da6f4 (Intelligence); ClericSpellbook 4673d19a0cf2fab4f885cc4d1353da33 (Wisdom); the
    // fighter and the Dragon Disciple have no spellbook of their own
    assert_eq!(ability("sorcerer"), Some(Ability::Charisma));
    assert_eq!(ability("wizard"), Some(Ability::Intelligence));
    assert_eq!(ability("cleric"), Some(Ability::Wisdom));
    assert_eq!(ability("fighter"), None);
    assert_eq!(ability("dragon-disciple"), None);
    let archetype = |key: &str| {
        DATA.archetypes
            .iter()
            .find(|archetype| archetype.key.as_str() == key)
            .unwrap_or_else(|| panic!("no archetype {key}"))
            .casting_ability
    };
    // Game: SageSpellbook cc2052732997b654e93eac268a39a0a9 (Intelligence); EmpyrealSpellbook
    // cb9d75e5cb08572439a6b5cd562297a5 (Wisdom)
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

    // Game: SpellFocus 16fa59cc9a72a6043b566b49184f53fe (BonusDC 1, doubled with the mythic feat: see above)
    assert!(has(
        BonusItem::Feat(rules::FeatKey::new("spell-focus")),
        dc(SpellScope::Value),
        Flat(1)
    ));
    // Game: ExpandedArsenalSchool f137089c48364014aa3ec3b92ccaf2e2 (another school's Spell Focus feats)
    assert!(has(
        BonusItem::MythicFeat(rules::MythicFeatKey::new("expanded-arsenal")),
        dc(SpellScope::Value),
        BonusAmount::SpellFocusOfAnotherSchool
    ));
    // Game: BloodlineInfernalArcana 67f151cdb9f0c9d4092ccfd5a9ccc6c9 (+2 Charm)
    assert!(has(
        BonusItem::Feature(rules::FeatureKey::new(
            "sorcerer/bloodline/infernal/infernal-arcana"
        )),
        dc(SpellScope::Subschool(Subschool::Charm)),
        Flat(2)
    ));
    // Game: PyromaniacGnome fc74a68e18a8479a9e8af34e761a70b3 (+1 Fire)
    assert!(has(
        BonusItem::Subrace(rules::SubraceKey::new("gnome/pyromaniac")),
        dc(SpellScope::Energy(rules::Energy::Fire)),
        Flat(1)
    ));
    // Game: OreadRace grants AcidAffinityOread 763a97c2216fd124da93b164994fd82b (+1 Acid)
    assert!(has(
        BonusItem::Race(rules::RaceKey::new("oread")),
        dc(SpellScope::Energy(rules::Energy::Acid)),
        Flat(1)
    ));
    // Game: BloodlineArcaneSchoolPowerFeature 8891303b67eb273428f1691864b08cf8 (one feature with a spell-school
    // parameter, +2; the eight schools are an inference from the parameter type)
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
