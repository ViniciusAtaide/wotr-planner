//! The caster line: a spell's save DC at each spell level, caster level against spell resistance and concentration,
//! from the casting class, the character's own casting ability and the flat bonuses of the items held. Unconditional
//! numbers only.
//!
//! Each line is read off a walked Build's state (ticket 151).

use rules::{
    Ability, BonusAmount, BonusItem, BonusTarget, Build, ClassChoice, DataIssue, Energy, Engine,
    Feat, FeatCategory, FeatKey, FeatureKey, GameData, ItemBonus, MythicFeat, MythicFeatKey,
    Parameter, ParameterKind, Pick, Prerequisite, RaceKey, Repeat, SlotId, SpellDc, SpellSchool,
    SpellScope, SpellTraits, SubraceKey, Subschool,
};

use crate::support::*;

/// A human sorcerer of this many levels with these point-buy scores (Str, Dex, Con, Int, Wis, Cha), the race's +2
/// to Strength, holding nothing else.
fn sorcerer_with(levels: usize, scores: [u8; 6]) -> Build {
    let mut build = build("human", &times("sorcerer", levels));
    build.start.ability_scores = point_buy(scores);
    build.start.racial_ability = Some(Ability::Strength);
    build
}

/// A human sorcerer of this many levels with Charisma 10, holding nothing else.
fn sorcerer(levels: usize) -> Build {
    sorcerer_with(levels, [10, 10, 10, 10, 10, 10])
}

/// A sorcerer 20 with Charisma 24: 18 bought, the race's +2 and four of the five increases.
fn sorcerer_20_charisma_24() -> Build {
    let mut build = sorcerer_with(20, [10, 10, 10, 10, 10, 18]);
    build.start.racial_ability = Some(Ability::Charisma);
    for level in [4, 8, 12, 16] {
        build.levels[level - 1].ability_increase = Some(Ability::Charisma);
    }
    build
}

fn spell(school: SpellSchool) -> SpellTraits {
    SpellTraits {
        school,
        subschool: None,
        energy: None,
        metamagic: false,
    }
}

fn school(school: SpellSchool) -> Parameter {
    Parameter::SpellSchool(school)
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

/// The caster line for a spell at the last level of a Build that holds up.
fn line(engine: &Engine, build: &Build, spell: SpellTraits) -> Option<rules::CasterLine> {
    engine.caster_line(&last_state(engine, build), spell)
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
    let engine = engine();
    let line = line(
        &engine,
        &sorcerer_20_charisma_24(),
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
    let engine = engine();
    let line = line(
        &engine,
        &sorcerer_with(5, [10, 10, 10, 10, 10, 16]),
        spell(SpellSchool::Evocation),
    )
    .unwrap();

    assert_eq!(dcs(&line), [(1, 14), (2, 15), (3, 16)]);
    assert_eq!(line.caster_level, 5);
}

#[test]
fn a_character_with_no_casting_class_has_no_line() {
    let engine = engine();
    let fighter = build("human", &times("fighter", 10));

    assert_eq!(line(&engine, &fighter, spell(SpellSchool::Evocation)), None);
}

#[test]
fn spell_focus_its_mythic_feat_and_school_power_count_for_their_school_only() {
    // The Arcane bloodline, and its School Power at 15.
    let mut build = sorcerer_20_charisma_24();
    build.picks.extend([
        (
            selection_slot(1, 0),
            pick_option("sorcerer/bloodline/arcane"),
        ),
        (
            general_slot(1),
            pick_feat_for("spell-focus", school(SpellSchool::Evocation)),
        ),
        (
            SlotId::Mythic { rank: 2 },
            Pick::mythic_feat_for(
                MythicFeatKey::new("spell-focus-mythic"),
                school(SpellSchool::Evocation),
            ),
        ),
        (
            SlotId::OptionSelection {
                level: 15,
                index: 0,
            },
            pick_option("sorcerer/school-power/evocation"),
        ),
    ]);
    let engine = engine();
    let state = last_state(&engine, &build);
    let line = |school| engine.caster_line(&state, spell(school)).unwrap();

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
    let mut build = build(
        "human",
        &[
            "wizard", "wizard", "wizard", "sorcerer", "sorcerer", "sorcerer", "sorcerer",
            "sorcerer",
        ],
    );
    build.start.ability_scores = point_buy([10, 10, 10, 18, 10, 14]);
    build.start.racial_ability = Some(Ability::Strength);
    let engine = engine();
    let line = line(&engine, &build, spell(SpellSchool::Evocation)).unwrap();

    assert_eq!(line.class, class("sorcerer"));
    assert_eq!((line.ability, line.caster_level), (Ability::Charisma, 5));
    assert_eq!(parts(&line.dc_parts), [("Charisma", 2)]);
}

#[test]
fn prestige_levels_that_advance_the_class_raise_its_caster_level() {
    // Rogue 3 / sorcerer 6 / Arcane Trickster 4 advancing the sorcerer: caster level 10, 5th-level spells.
    let mut classes = times("rogue", 3);
    classes.extend(times("sorcerer", 6));
    classes.extend(times("arcane-trickster", 4));
    let mut build = build("elf", &classes);
    for level in build.levels.iter_mut().take(4) {
        level.skill_ranks.insert(skill("mobility"), 1);
    }
    build.class_choices.insert(
        class("arcane-trickster"),
        ClassChoice {
            archetype: None,
            advances: [class("sorcerer")].into(),
        },
    );
    let engine = engine();
    let line = line(&engine, &build, spell(SpellSchool::Evocation)).unwrap();

    assert_eq!(line.caster_level, 10);
    assert_eq!(line.dcs.len(), 5);
    assert_eq!(line.spell_penetration, 10);
}

#[test]
fn an_archetype_may_cast_with_another_ability() {
    // The Sage casts with Intelligence: a sorcerer 10 with Intelligence 16 and Charisma 20 (16 bought, the race's
    // +2 and two increases).
    let mut build = sorcerer_with(10, [10, 10, 10, 16, 10, 16]);
    build.start.racial_ability = Some(Ability::Charisma);
    for level in [4, 8] {
        build.levels[level - 1].ability_increase = Some(Ability::Charisma);
    }
    build.class_choices.insert(
        class("sorcerer"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("sorcerer/sage")),
            ..ClassChoice::default()
        },
    );
    let engine = engine();
    let state = last_state(&engine, &build);
    assert_eq!(state.ability_scores.charisma, 20);
    let line = engine
        .caster_line(&state, spell(SpellSchool::Evocation))
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
    let penetration = |subrace: Option<&str>| {
        let mut build = build("half-elf", &times("sorcerer", 10));
        build.start.subrace = subrace.map(SubraceKey::new);
        build.start.racial_ability = Some(Ability::Strength);
        let line = line(&engine, &build, spell(SpellSchool::Evocation)).unwrap();
        (line.spell_penetration, line.spell_penetration_parts)
    };

    let (race, race_parts) = penetration(None);
    assert_eq!((race, parts(&race_parts)), (11, vec![("Half-Elf", 1)]));
    let (subrace, subrace_parts) = penetration(Some("half-elf/dual-heritage"));
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
    // A sorcerer 16: Mythic rank 7.
    let mut build = sorcerer(16);
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(MythicFeatKey::new("dodge-mythic")),
    );
    let penetration = |build: &Build| {
        line(&engine, build, spell(SpellSchool::Evocation))
            .unwrap()
            .spell_penetration
    };

    assert_eq!(penetration(&build), 19);
    build
        .picks
        .insert(general_slot(5), pick_feat("spell-penetration"));
    // Spell Penetration's own +2, and the full rank.
    assert_eq!(penetration(&build), 25);
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
    // The Arcane bloodline gives Arcane Bond at 1.
    let mut build = sorcerer(20);
    build.picks.insert(
        selection_slot(1, 0),
        pick_option("sorcerer/bloodline/arcane"),
    );
    let state = last_state(&engine, &build);
    let bonus_for = |spell: SpellTraits| {
        let line = engine.caster_line(&state, spell).unwrap();
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

/// The test world with Expanded Arsenal (a mythic feat taken for a school) copying Spell Focus feats, and Spell
/// Focus taken once per school.
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
    let spell_focus = data.feats.iter_mut().find(|f| f.key == feat("spell-focus"));
    spell_focus.unwrap().repeat = Repeat::PerValue;
    engine_on(data)
}

#[test]
fn expanded_arsenal_gives_its_school_the_spell_focus_feats_of_another() {
    // A sorcerer 15: mythic feats at MR 2, 4 and 6.
    let mut build = sorcerer(15);
    build.picks.extend([
        (
            general_slot(1),
            pick_feat_for("spell-focus", school(SpellSchool::Evocation)),
        ),
        (
            race_slot(0),
            pick_feat_for("spell-focus", school(SpellSchool::Illusion)),
        ),
        (
            SlotId::Mythic { rank: 2 },
            Pick::mythic_feat_for(
                MythicFeatKey::new("spell-focus-mythic"),
                school(SpellSchool::Evocation),
            ),
        ),
        (
            SlotId::Mythic { rank: 4 },
            Pick::mythic_feat_for(
                MythicFeatKey::new("expanded-arsenal"),
                school(SpellSchool::Necromancy),
            ),
        ),
        (
            SlotId::Mythic { rank: 6 },
            Pick::mythic_feat_for(
                MythicFeatKey::new("expanded-arsenal"),
                school(SpellSchool::Illusion),
            ),
        ),
    ]);
    let engine = with_expanded_arsenal();
    let state = last_state(&engine, &build);
    let line = |school| engine.caster_line(&state, spell(school)).unwrap();

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
    let mut data = world();
    let template = data.feats[0].clone();
    data.feats.push(Feat {
        key: feat("metamagic-heighten-spell"),
        name: "Heighten Spell".into(),
        category: FeatCategory::Magic,
        prerequisite: None,
        ..template
    });
    let engine = engine_on(data);
    let mut build = sorcerer_20_charisma_24();
    build
        .picks
        .insert(general_slot(1), pick_feat("metamagic-heighten-spell"));
    let line = line(&engine, &build, spell(SpellSchool::Enchantment)).unwrap();

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
