//! Assumed gear (ticket 122): bonuses an agent assumes on top of a Build, applied to the attack line, the caster line
//! and the totals beside the unassisted numbers, never to what the character holds or to a prerequisite.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    Ability, Assumed, BonusAmount, BonusItem, BonusTarget, BuildStart, CharacterState, ClassLevels,
    DataIssue, Engine, FeatKey, GameData, GearBonus, GearCircumstance, GearItem, GearKey, GearSlot,
    GearTarget, GuideRef, ItemBonus, Prerequisite, Save, SpellSchool, SpellScope, SpellTraits,
    Subschool, WeaponKey,
};

use crate::support::*;

fn item(key: &str, bonuses: Vec<GearBonus>) -> GearItem {
    GearItem {
        key: GearKey::new(key),
        name: key.to_owned(),
        slot: GearSlot::Robe,
        guide: GuideRef::new("items"),
        bonuses,
    }
}

fn gear(target: GearTarget, amount: i8, circumstance: GearCircumstance) -> GearBonus {
    GearBonus {
        target,
        amount,
        circumstance,
    }
}

/// The test world with a few items of each circumstance.
fn armoury() -> GameData {
    let mut data = world();
    data.gear = vec![
        item(
            "heartstone",
            vec![gear(
                GearTarget::SpellDc,
                3,
                GearCircumstance::Spells(SpellScope::All),
            )],
        ),
        item(
            "enchanting-glasses",
            vec![gear(
                GearTarget::SpellDc,
                2,
                GearCircumstance::Spells(SpellScope::School(SpellSchool::Enchantment)),
            )],
        ),
        item(
            "herald-cloak",
            vec![gear(
                GearTarget::SpellDc,
                2,
                GearCircumstance::Spells(SpellScope::Subschool(Subschool::Compulsion)),
            )],
        ),
        item(
            "sinful-robe",
            vec![
                gear(
                    GearTarget::CasterLevel,
                    3,
                    GearCircumstance::Spells(SpellScope::All),
                ),
                gear(
                    GearTarget::SpellPenetration,
                    5,
                    GearCircumstance::Spells(SpellScope::All),
                ),
            ],
        ),
        item(
            "fortitude-robe",
            vec![
                gear(
                    GearTarget::SpellDc,
                    2,
                    GearCircumstance::Save(Save::Fortitude),
                ),
                gear(
                    GearTarget::Concentration,
                    4,
                    GearCircumstance::Spells(SpellScope::All),
                ),
            ],
        ),
        item(
            "mind-goggles",
            vec![gear(
                GearTarget::SpellDc,
                2,
                GearCircumstance::MindAffecting,
            )],
        ),
        item(
            "witch-rags",
            vec![gear(
                GearTarget::SpellDc,
                1,
                GearCircumstance::Class(class("witch")),
            )],
        ),
        item(
            "sorcerer-rags",
            vec![gear(
                GearTarget::SpellDc,
                1,
                GearCircumstance::Class(class("sorcerer")),
            )],
        ),
    ];
    data
}

fn sorcerer(level: u8, charisma: u8) -> CharacterState {
    CharacterState {
        classes: vec![ClassLevels {
            class: class("sorcerer"),
            prestige: false,
            archetype: None,
            advances: BTreeSet::new(),
            level,
        }],
        ability_scores: scores([10, 10, 10, 10, 10, charisma]),
        ..CharacterState::default()
    }
}

fn spell(school: SpellSchool) -> SpellTraits {
    SpellTraits {
        school,
        subschool: None,
        energy: None,
        metamagic: false,
        save: None,
        mind_affecting: false,
    }
}

fn assuming(items: &[&str]) -> Assumed {
    Assumed {
        items: items.iter().map(|key| GearKey::new(*key)).collect(),
        ..Assumed::default()
    }
}

fn parts(parts: &[rules::CasterPart]) -> Vec<(&str, i16)> {
    parts
        .iter()
        .map(|part| (part.name.as_str(), part.value))
        .collect()
}

#[test]
fn assumed_ability_bonuses_raise_the_scores_they_name() {
    let assumed = Assumed {
        enhancement: BTreeMap::from([(Ability::Charisma, 8), (Ability::Strength, 6)]),
        inherent: BTreeMap::from([(Ability::Charisma, 4)]),
        ..Assumed::default()
    };
    let raised = assumed.scores(scores([16, 10, 10, 10, 10, 20]));

    assert_eq!(raised.get(Ability::Strength), 22);
    assert_eq!(raised.get(Ability::Charisma), 32);
    assert_eq!(raised.get(Ability::Dexterity), 10);
}

#[test]
fn the_assumed_attack_line_takes_the_raised_strength_and_the_weapons_enhancement() {
    let engine = engine();
    let state = CharacterState {
        bab: 11,
        ability_scores: scores([18, 10, 10, 10, 10, 10]),
        ..CharacterState::default()
    };
    let longsword = WeaponKey::new("longsword");
    let assumed = Assumed {
        enhancement: BTreeMap::from([(Ability::Strength, 6)]),
        weapon_enhancement: 5,
        ..Assumed::default()
    };
    let own = engine.attack_line(&state, &longsword).unwrap();
    let line = engine
        .assumed_attack_line(&state, &longsword, &assumed)
        .unwrap();

    // Str 18 (+4) raised to 24 (+7), and +5 to attack and damage.
    assert_eq!(own.attacks, [15, 10, 5]);
    assert_eq!(line.attacks, [23, 18, 13]);
    assert_eq!(own.damage_bonus, 4);
    assert_eq!(line.damage_bonus, 12);
    let names: Vec<(&str, i16, i16)> = line
        .parts
        .iter()
        .map(|part| (part.name.as_str(), part.attack, part.damage))
        .collect();
    assert_eq!(
        names,
        [
            ("Strength", 7, 7),
            ("Weapon enhancement +5 (assumed)", 5, 5)
        ]
    );
    assert_eq!(
        engine.assumed_attack_line(&state, &longsword, &Assumed::default()),
        Some(own)
    );
}

#[test]
fn the_assumed_caster_line_takes_the_raised_casting_ability_and_the_items_whose_circumstance_matches()
 {
    let engine = Engine::new(armoury());
    let state = sorcerer(20, 24);
    let start = BuildStart::default();
    let assumed = Assumed {
        enhancement: BTreeMap::from([(Ability::Charisma, 6)]),
        ..assuming(&[
            "heartstone",
            "enchanting-glasses",
            "herald-cloak",
            "sorcerer-rags",
            "witch-rags",
        ])
    };

    let evocation = engine
        .assumed_caster_line(&start, &state, spell(SpellSchool::Evocation), &assumed)
        .unwrap();
    // Cha 24 (+7) raised to 30 (+10); the heartstone and the sorcerer's rags count for any sorcerer spell.
    assert_eq!(
        parts(&evocation.dc_parts),
        [
            ("Charisma", 10),
            ("heartstone (assumed)", 3),
            ("sorcerer-rags (assumed)", 1)
        ]
    );
    assert_eq!(evocation.dcs[8].dc, 10 + 9 + 14);
    assert_eq!(evocation.concentration, 20 + 10);

    let compulsion = engine
        .assumed_caster_line(
            &start,
            &state,
            SpellTraits {
                subschool: Some(Subschool::Compulsion),
                ..spell(SpellSchool::Enchantment)
            },
            &assumed,
        )
        .unwrap();
    assert_eq!(
        parts(&compulsion.dc_parts),
        [
            ("Charisma", 10),
            ("heartstone (assumed)", 3),
            ("enchanting-glasses (assumed)", 2),
            ("herald-cloak (assumed)", 2),
            ("sorcerer-rags (assumed)", 1)
        ]
    );

    // The unassisted line is the same as before, and assuming nothing changes nothing.
    let own = engine
        .caster_line(&start, &state, spell(SpellSchool::Evocation))
        .unwrap();
    assert_eq!(parts(&own.dc_parts), [("Charisma", 7)]);
    assert_eq!(
        engine.assumed_caster_line(
            &start,
            &state,
            spell(SpellSchool::Evocation),
            &Assumed::default()
        ),
        Some(own)
    );
}

#[test]
fn an_assumed_caster_level_item_raises_the_caster_level_and_both_checks_with_it() {
    let engine = Engine::new(armoury());
    let line = engine
        .assumed_caster_line(
            &BuildStart::default(),
            &sorcerer(20, 24),
            spell(SpellSchool::Evocation),
            &assuming(&["sinful-robe", "fortitude-robe"]),
        )
        .unwrap();

    assert_eq!(line.caster_level, 23);
    assert_eq!(
        parts(&line.caster_level_parts),
        [("sinful-robe (assumed)", 3)]
    );
    // 23 + the robe's +5 against spell resistance.
    assert_eq!(line.spell_penetration, 28);
    // 23 + Charisma 7 + the other robe's +4 concentration.
    assert_eq!(line.concentration, 34);
    // The other robe's DC counts only for spells with a Fortitude save.
    assert_eq!(parts(&line.dc_parts), [("Charisma", 7)]);
}

#[test]
fn save_mind_affecting_and_class_circumstances_need_the_spell_or_the_class_to_match() {
    let engine = Engine::new(armoury());
    let start = BuildStart::default();
    let state = sorcerer(20, 10);
    let assumed = assuming(&["fortitude-robe", "mind-goggles", "witch-rags"]);
    let dc_bonus = |spell: SpellTraits| {
        let line = engine
            .assumed_caster_line(&start, &state, spell, &assumed)
            .unwrap();
        line.dc_parts.iter().map(|part| part.value).sum::<i16>()
    };
    let plain = spell(SpellSchool::Necromancy);

    assert_eq!(dc_bonus(plain), 0);
    assert_eq!(
        dc_bonus(SpellTraits {
            save: Some(Save::Fortitude),
            ..plain
        }),
        2
    );
    assert_eq!(
        dc_bonus(SpellTraits {
            save: Some(Save::Will),
            mind_affecting: true,
            ..plain
        }),
        2
    );
}

#[test]
fn the_assumed_totals_take_the_raised_scores_and_a_resistance_bonus_on_each_save() {
    let engine = engine();
    let state = CharacterState {
        fort: 12,
        reflex: 6,
        will: 6,
        ability_scores: scores([10, 14, 10, 10, 10, 10]),
        ..CharacterState::default()
    };
    let assumed = Assumed {
        enhancement: BTreeMap::from([(Ability::Dexterity, 6)]),
        resistance: 7,
        ..Assumed::default()
    };
    let start = BuildStart::default();
    let own = engine.totals(&start, &state);
    let totals = engine.assumed_totals(&start, &state, &assumed);

    // Dex 14 (+2) raised to 20 (+5).
    assert_eq!(own.initiative.total, 2);
    assert_eq!(totals.initiative.total, 5);
    assert_eq!(own.fortitude.total, 12);
    assert_eq!(totals.fortitude.total, 19);
    assert_eq!(totals.reflex.total, 6 + 5 + 7);
    assert_eq!(
        totals.will.parts.last().map(|part| part.name.as_str()),
        Some("Resistance +7 (assumed)")
    );
    assert_eq!(
        engine.assumed_totals(&start, &state, &Assumed::default()),
        own
    );
}

#[test]
fn a_prerequisite_never_sees_an_assumed_bonus() {
    // A bonus that needs Charisma 20 besides its feat: the character has 14, and assumes +8.
    let mut data = armoury();
    let requires = Prerequisite::AbilityScore {
        ability: Ability::Charisma,
        score: 20,
    };
    data.bonuses.push(ItemBonus {
        item: BonusItem::Feat(FeatKey::new("spell-penetration")),
        target: BonusTarget::Concentration,
        amount: BonusAmount::Flat(4),
        requires: Some(requires.clone()),
    });
    let engine = Engine::new(data);
    let mut state = sorcerer(20, 14);
    *state
        .feats
        .entry(FeatKey::new("spell-penetration"))
        .or_default() += 1;
    let assumed = Assumed {
        enhancement: BTreeMap::from([(Ability::Charisma, 8)]),
        ..Assumed::default()
    };
    let line = engine
        .assumed_caster_line(
            &BuildStart::default(),
            &state,
            spell(SpellSchool::Evocation),
            &assumed,
        )
        .unwrap();

    // Charisma counts at 22 (+6), but the bonus that needs 20 doesn't: its `requires` reads the character's own 14.
    assert_eq!(parts(&line.concentration_parts), [("Charisma", 6)]);
    assert!(state.check(&requires).is_err());
    assert_eq!(state.ability_scores.get(Ability::Charisma), 14);
}

#[test]
fn the_data_check_refuses_a_gear_bonus_for_a_value_an_empty_item_and_an_unknown_class() {
    let mut data = armoury();
    data.gear.push(item(
        "valued",
        vec![gear(
            GearTarget::SpellDc,
            1,
            GearCircumstance::Spells(SpellScope::Value),
        )],
    ));
    data.gear.push(item("empty", vec![]));
    data.gear.push(item(
        "unknown-class",
        vec![gear(
            GearTarget::SpellDc,
            1,
            GearCircumstance::Class(class("no-such-class")),
        )],
    ));
    data.gear.push(item(
        "heartstone",
        vec![gear(GearTarget::SpellDc, 1, GearCircumstance::default())],
    ));
    let issues: Vec<(String, DataIssue)> = data
        .check()
        .into_iter()
        .map(|problem| (problem.at.to_string(), problem.issue))
        .collect();

    assert!(issues.contains(&("gear item valued".into(), DataIssue::GearBonusForValue)));
    assert!(issues.contains(&("gear item empty".into(), DataIssue::GearWithoutBonus)));
    assert!(issues.contains(&(
        "gear item unknown-class".into(),
        DataIssue::UnknownKey(rules::DataAt::Class(class("no-such-class")))
    )));
    assert!(issues.contains(&("gear item heartstone".into(), DataIssue::DuplicateKey)));
    assert!(armoury().check().is_empty());
}
