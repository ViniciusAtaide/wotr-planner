//! The caster line: a spell's save DC at each spell level, caster level against spell resistance and concentration,
//! from the casting class, the character's own casting ability and the flat bonuses of the items held. Unconditional
//! numbers only.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    Ability, BonusAmount, BonusItem, BonusTarget, BuildStart, CharacterState, ClassLevels,
    DataIssue, Energy, Engine, FeatKey, FeatureKey, GameData, ItemBonus, MythicFeat, MythicFeatKey,
    Parameter, ParameterKind, Prerequisite, RaceKey, Repeat, SpellDc, SpellSchool, SpellScope,
    SpellTraits, SubraceKey, Subschool,
};

use crate::support::*;

fn levels(class_key: &str, level: u8) -> ClassLevels {
    ClassLevels {
        class: class(class_key),
        prestige: false,
        archetype: None,
        advances: BTreeSet::new(),
        level,
    }
}

/// A sorcerer of this level with this Charisma (the rest 10), holding nothing else.
fn sorcerer(level: u8, charisma: u8) -> CharacterState {
    CharacterState {
        classes: vec![levels("sorcerer", level)],
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
    }
}

fn holding_for(state: &mut CharacterState, feat: &str, school: SpellSchool) {
    state
        .feat_values
        .entry(FeatKey::new(feat))
        .or_default()
        .insert(Parameter::SpellSchool(school), 1);
    *state.feats.entry(FeatKey::new(feat)).or_default() += 1;
}

fn holding_mythic_for(state: &mut CharacterState, feat: &str, school: SpellSchool) {
    let key = MythicFeatKey::new(feat);
    state.mythic_feats.insert(key.clone());
    state
        .mythic_feat_values
        .entry(key)
        .or_default()
        .insert(Parameter::SpellSchool(school));
}

fn bonus(item: BonusItem, target: BonusTarget, amount: BonusAmount) -> ItemBonus {
    ItemBonus {
        item,
        target,
        amount,
        requires: None,
    }
}

fn with_bonuses(bonuses: impl IntoIterator<Item = ItemBonus>) -> GameData {
    let mut data = world();
    data.bonuses.extend(bonuses);
    data
}

fn dcs(line: &rules::CasterLine) -> Vec<(u8, i16)> {
    line.dcs.iter().map(|dc| (dc.spell_level, dc.dc)).collect()
}

fn parts(parts: &[rules::CasterPart]) -> Vec<(&str, i16)> {
    parts
        .iter()
        .map(|part| (part.name.as_str(), part.value))
        .collect()
}

#[test]
fn a_sorcerers_dcs_are_10_plus_the_spell_level_plus_charisma_at_each_level_castable() {
    // Sorcerer 20, Cha 24 (+7): spells of levels 1 to 9.
    let line = engine()
        .caster_line(
            &BuildStart::default(),
            &sorcerer(20, 24),
            spell(SpellSchool::Enchantment),
        )
        .expect("a sorcerer casts");

    assert_eq!(line.class, class("sorcerer"));
    assert_eq!(line.ability, Ability::Charisma);
    assert_eq!(line.caster_level, 20);
    assert_eq!(
        dcs(&line),
        (1..=9)
            .map(|level| (level, 17 + i16::from(level)))
            .collect::<Vec<_>>()
    );
    assert_eq!(parts(&line.dc_parts), [("Charisma", 7)]);
    assert_eq!(line.heightened, None);
    assert_eq!(line.spell_penetration, 20);
    assert!(line.spell_penetration_parts.is_empty());
    assert_eq!(line.concentration, 27);
    assert_eq!(parts(&line.concentration_parts), [("Charisma", 7)]);
    assert_eq!(line.leaves_out, rules::CASTER_LINE_LEAVES_OUT);
}

#[test]
fn the_spell_levels_follow_the_class_level() {
    // The test world's sorcerer casts up to ⌈level/2⌉: 3rd-level spells at 5.
    let line = engine()
        .caster_line(
            &BuildStart::default(),
            &sorcerer(5, 16),
            spell(SpellSchool::Evocation),
        )
        .unwrap();

    assert_eq!(dcs(&line), [(1, 14), (2, 15), (3, 16)]);
    assert_eq!(line.caster_level, 5);
}

#[test]
fn a_character_with_no_casting_class_has_no_line() {
    let fighter = CharacterState {
        classes: vec![levels("fighter", 10)],
        ..CharacterState::default()
    };

    assert_eq!(
        engine().caster_line(
            &BuildStart::default(),
            &fighter,
            spell(SpellSchool::Evocation)
        ),
        None
    );
}

#[test]
fn spell_focus_its_mythic_feat_and_school_power_count_for_their_school_only() {
    let mut state = sorcerer(20, 24);
    holding_for(&mut state, "spell-focus", SpellSchool::Evocation);
    holding_mythic_for(&mut state, "spell-focus-mythic", SpellSchool::Evocation);
    *state
        .selection_options
        .entry(rules::SelectionOptionKey::new(
            "sorcerer/school-power/evocation",
        ))
        .or_default() += 1;
    let engine = engine();
    let line = |school| {
        engine
            .caster_line(&BuildStart::default(), &state, spell(school))
            .unwrap()
    };

    let evocation = line(SpellSchool::Evocation);
    assert_eq!(
        parts(&evocation.dc_parts),
        [
            ("Charisma", 7),
            ("spell-focus (Evocation)", 1),
            ("spell-focus-mythic (Evocation)", 1),
            ("sorcerer/school-power (evocation)", 2),
        ]
    );
    assert_eq!(
        evocation.dcs[8],
        SpellDc {
            spell_level: 9,
            dc: 30
        }
    );
    assert_eq!(
        parts(&line(SpellSchool::Necromancy).dc_parts),
        [("Charisma", 7)]
    );
}

#[test]
fn the_line_follows_the_casting_class_with_the_highest_caster_level() {
    // Wizard 3 / sorcerer 5: the sorcerer's caster level, spells and Charisma.
    let state = CharacterState {
        classes: vec![levels("wizard", 3), levels("sorcerer", 5)],
        ability_scores: scores([10, 10, 10, 18, 10, 14]),
        ..CharacterState::default()
    };
    let line = engine()
        .caster_line(
            &BuildStart::default(),
            &state,
            spell(SpellSchool::Evocation),
        )
        .unwrap();

    assert_eq!(line.class, class("sorcerer"));
    assert_eq!((line.ability, line.caster_level), (Ability::Charisma, 5));
    assert_eq!(parts(&line.dc_parts), [("Charisma", 2)]);
}

#[test]
fn prestige_levels_that_advance_the_class_raise_its_caster_level() {
    // Sorcerer 6 / Arcane Trickster 4 advancing the sorcerer: caster level 10, 5th-level spells.
    let state = CharacterState {
        classes: vec![
            levels("sorcerer", 6),
            ClassLevels {
                prestige: true,
                advances: BTreeSet::from([class("sorcerer")]),
                ..levels("arcane-trickster", 4)
            },
        ],
        ability_scores: scores([10, 10, 10, 10, 10, 10]),
        ..CharacterState::default()
    };
    let line = engine()
        .caster_line(
            &BuildStart::default(),
            &state,
            spell(SpellSchool::Evocation),
        )
        .unwrap();

    assert_eq!(line.caster_level, 10);
    assert_eq!(line.dcs.len(), 5);
    assert_eq!(line.spell_penetration, 10);
}

#[test]
fn an_archetype_may_cast_with_another_ability() {
    // The Sage casts with Intelligence.
    let mut state = sorcerer(10, 20);
    state.classes[0].archetype = Some(rules::ArchetypeKey::new("sorcerer/sage"));
    state.ability_scores.intelligence = 16;
    let line = engine()
        .caster_line(
            &BuildStart::default(),
            &state,
            spell(SpellSchool::Evocation),
        )
        .unwrap();

    assert_eq!(line.ability, Ability::Intelligence);
    assert_eq!(parts(&line.dc_parts), [("Intelligence", 3)]);
    assert_eq!(line.concentration, 13);
}

#[test]
fn a_subraces_bonuses_replace_its_races() {
    // An invented +1 for the half-elf race; Dual Heritage's +2 replaces it.
    let engine = engine_on(with_bonuses([bonus(
        BonusItem::Race(RaceKey::new("half-elf")),
        BonusTarget::SpellPenetration,
        BonusAmount::Flat(1),
    )]));
    let start = |subrace: Option<&str>| BuildStart {
        race: Some(RaceKey::new("half-elf")),
        subrace: subrace.map(SubraceKey::new),
        ..BuildStart::default()
    };
    let penetration = |start: &BuildStart| {
        let line = engine
            .caster_line(start, &sorcerer(10, 10), spell(SpellSchool::Evocation))
            .unwrap();
        (line.spell_penetration, line.spell_penetration_parts)
    };

    let (race, race_parts) = penetration(&start(None));
    assert_eq!((race, parts(&race_parts)), (11, vec![("Half-Elf", 1)]));
    let (subrace, subrace_parts) = penetration(&start(Some("half-elf/dual-heritage")));
    assert_eq!(
        (subrace, parts(&subrace_parts)),
        (12, vec![("Dual Heritage", 2)])
    );
}

#[test]
fn a_bonus_counts_only_when_its_requirement_holds() {
    // As Spell Penetration (Mythic): half the Mythic rank, or the full rank with Greater Spell Penetration.
    let mythic = BonusItem::MythicFeat(MythicFeatKey::new("dodge-mythic"));
    let greater = Prerequisite::Feat(FeatKey::new("spell-penetration"));
    let engine = engine_on(with_bonuses([
        ItemBonus {
            requires: Some(Prerequisite::Not(Box::new(greater.clone()))),
            ..bonus(
                mythic.clone(),
                BonusTarget::SpellPenetration,
                BonusAmount::HalfMythicRank,
            )
        },
        ItemBonus {
            requires: Some(greater),
            ..bonus(
                mythic,
                BonusTarget::SpellPenetration,
                BonusAmount::MythicRank,
            )
        },
    ]));
    let mut state = sorcerer(20, 10);
    state.mythic_rank = 7;
    state
        .mythic_feats
        .insert(MythicFeatKey::new("dodge-mythic"));
    let penetration = |state: &CharacterState| {
        engine
            .caster_line(&BuildStart::default(), state, spell(SpellSchool::Evocation))
            .unwrap()
            .spell_penetration
    };

    assert_eq!(penetration(&state), 23);
    *state
        .feats
        .entry(FeatKey::new("spell-penetration"))
        .or_default() += 1;
    // Spell Penetration's own +2, and the full rank.
    assert_eq!(penetration(&state), 29);
}

#[test]
fn subschool_energy_and_metamagic_bonuses_count_when_the_spell_has_them() {
    let arcana = BonusItem::Feature(FeatureKey::new("sorcerer/bloodline/arcane/arcane-bond"));
    let dc = BonusTarget::SpellDc;
    let engine = engine_on(with_bonuses([
        bonus(
            arcana.clone(),
            dc(SpellScope::Subschool(Subschool::Compulsion)),
            BonusAmount::Flat(2),
        ),
        bonus(
            arcana.clone(),
            dc(SpellScope::Energy(Energy::Fire)),
            BonusAmount::Flat(1),
        ),
        bonus(arcana, dc(SpellScope::Metamagic), BonusAmount::Flat(1)),
    ]));
    let mut state = sorcerer(20, 10);
    state.features =
        BTreeMap::from([(FeatureKey::new("sorcerer/bloodline/arcane/arcane-bond"), 1)]);
    let bonus_for = |spell: SpellTraits| {
        let line = engine
            .caster_line(&BuildStart::default(), &state, spell)
            .unwrap();
        line.dc_parts.iter().map(|part| part.value).sum::<i16>()
    };
    let plain = spell(SpellSchool::Enchantment);

    assert_eq!(bonus_for(plain), 0);
    assert_eq!(
        bonus_for(SpellTraits {
            subschool: Some(Subschool::Compulsion),
            ..plain
        }),
        2
    );
    assert_eq!(
        bonus_for(SpellTraits {
            subschool: Some(Subschool::Charm),
            ..plain
        }),
        0
    );
    assert_eq!(
        bonus_for(SpellTraits {
            school: SpellSchool::Evocation,
            energy: Some(Energy::Fire),
            metamagic: true,
            subschool: None,
        }),
        2
    );
}

/// The test world with Expanded Arsenal (a mythic feat taken for a school) copying Spell Focus feats.
fn with_expanded_arsenal() -> Engine {
    let mut data = with_bonuses([bonus(
        BonusItem::MythicFeat(MythicFeatKey::new("expanded-arsenal")),
        BonusTarget::SpellDc(SpellScope::Value),
        BonusAmount::SpellFocusOfAnotherSchool,
    )]);
    let template = data.mythic_feats[0].clone();
    data.mythic_feats.push(MythicFeat {
        key: MythicFeatKey::new("expanded-arsenal"),
        name: "Expanded Arsenal".into(),
        parameter: Some(ParameterKind::SpellSchool),
        repeat: Repeat::PerValue,
        prerequisite: None,
        extra_pick: None,
        removes_racial_penalty: false,
        ..template
    });
    engine_on(data)
}

#[test]
fn expanded_arsenal_gives_its_school_the_spell_focus_feats_of_another() {
    let mut state = sorcerer(20, 10);
    holding_for(&mut state, "spell-focus", SpellSchool::Evocation);
    holding_mythic_for(&mut state, "spell-focus-mythic", SpellSchool::Evocation);
    holding_for(&mut state, "spell-focus", SpellSchool::Illusion);
    holding_mythic_for(&mut state, "expanded-arsenal", SpellSchool::Necromancy);
    holding_mythic_for(&mut state, "expanded-arsenal", SpellSchool::Illusion);
    let engine = with_expanded_arsenal();
    let line = |school| {
        engine
            .caster_line(&BuildStart::default(), &state, spell(school))
            .unwrap()
    };

    // Necromancy gets the school with the most: Evocation's two.
    assert_eq!(
        parts(&line(SpellSchool::Necromancy).dc_parts),
        [
            ("Charisma", 0),
            (
                "Expanded Arsenal (Necromancy): Evocation's Spell Focus feats",
                2
            ),
        ]
    );
    // Illusion has a Spell Focus of its own: Expanded Arsenal does nothing there.
    assert_eq!(
        parts(&line(SpellSchool::Illusion).dc_parts),
        [("Charisma", 0), ("spell-focus (Illusion)", 1)]
    );
}

#[test]
fn heighten_spell_gives_the_highest_levels_dc_to_any_spell() {
    let mut state = sorcerer(20, 24);
    *state
        .feats
        .entry(FeatKey::new("metamagic-heighten-spell"))
        .or_default() += 1;
    let line = engine()
        .caster_line(
            &BuildStart::default(),
            &state,
            spell(SpellSchool::Enchantment),
        )
        .unwrap();

    assert_eq!(
        line.heightened,
        Some(SpellDc {
            spell_level: 9,
            dc: 26
        })
    );
}

#[test]
fn the_data_check_refuses_a_caster_without_an_ability_and_a_value_bonus_on_an_item_without_one() {
    let mut data = world();
    for class in data.classes.iter_mut() {
        if class.key == class_key("sorcerer") {
            class.casting_ability = None;
        }
        if class.key == class_key("fighter") {
            class.casting_ability = Some(Ability::Wisdom);
        }
    }
    data.bonuses.push(bonus(
        BonusItem::Feat(FeatKey::new("spell-penetration")),
        BonusTarget::SpellDc(SpellScope::Value),
        BonusAmount::Flat(1),
    ));
    data.bonuses.push(bonus(
        BonusItem::Feat(FeatKey::new("no-such-feat")),
        BonusTarget::Concentration,
        BonusAmount::Flat(4),
    ));
    let issues: Vec<(String, DataIssue)> = data
        .check()
        .into_iter()
        .map(|problem| (problem.at.to_string(), problem.issue))
        .collect();

    assert!(issues.contains(&("class sorcerer".into(), DataIssue::NoCastingAbility)));
    assert!(issues.contains(&(
        "class fighter".into(),
        DataIssue::CastingAbilityWithoutSpells
    )));
    assert!(issues.contains(&(
        "feat spell-penetration".into(),
        DataIssue::BonusValueWithoutParameter
    )));
    assert!(issues.contains(&(
        "feat no-such-feat".into(),
        DataIssue::UnknownKey(rules::DataAt::Feat(FeatKey::new("no-such-feat")))
    )));
}

fn class_key(key: &str) -> rules::ClassKey {
    class(key)
}
