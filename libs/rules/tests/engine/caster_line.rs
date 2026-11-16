//! The caster line: a spell's save DC at each spell level, caster level against spell resistance and concentration,
//! from the casting class, the character's own casting ability and the components of the entries held (ticket 297).
//! Unconditional numbers only.
//!
//! Each line is read off a walked Build's state (ticket 151).

use rules::{
    Ability, BonusItem, Build, ClassChoice, Component, Condition, DataIssue, Energy, Engine, Fact,
    Feat, FeatCategory, FeatKey, FeatureKey, GameData, MetamagicKind, MythicFeat, MythicFeatKey,
    Parameter, ParameterKind, Pick, RaceKey, RankBase, RankProgression, Repeat, SlotId, SpellDc,
    SpellSchool, SpellScope, SpellTraits, SubraceKey, Subschool, Value,
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

/// The test world with these components on an entry.
fn with_components(item: BonusItem, components: impl IntoIterator<Item = Component>) -> GameData {
    let mut data = world();
    components_mut(&mut data, &item).extend(components);
    data
}

fn dc(scope: SpellScope, amount: i16) -> Component {
    Component::SpellDc {
        scope,
        value: Value::Flat(amount),
    }
}

/// The caster line for a spell at the last level of a Build that holds up.
fn line(engine: &Engine, build: &Build, spell: SpellTraits) -> Option<rules::CasterLine> {
    engine.caster_line(&last_level(engine, build), spell)
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

/// Spell Focus's +1 is doubled while Spell Focus (Mythic) is held for its school, which adds nothing of its own; School
/// Power's +2 is its school's.
// Game: SpellFocusParametrized.GetBonusDC, WrathDecompiled 6ce4a6b (BonusDC × 2 while m_MythicFocus is held for the
// school)
#[test]
fn spell_focus_doubled_by_its_mythic_feat_and_school_power_count_for_their_school_only() {
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
    let level = last_level(&engine, &build);
    let line = |school| engine.caster_line(&level, spell(school)).unwrap();

    let evocation = line(SpellSchool::Evocation);
    assert_eq!(
        parts(&evocation.dc_parts),
        [
            ("Charisma", 7),
            ("spell-focus (Evocation)", 2),
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
    let level = last_level(&engine, &build);
    assert_eq!(level.state.ability_scores.charisma, 20);
    let line = engine
        .caster_line(&level, spell(SpellSchool::Evocation))
        .unwrap();

    assert_eq!(line.ability, Ability::Intelligence);
    assert_eq!(parts(&line.dc_parts), [("Intelligence", 3)]);
    assert_eq!(line.concentration, 13);
}

#[test]
fn a_subraces_bonuses_replace_its_races() {
    // An invented +1 for the half-elf race; Dual Heritage's +2 replaces it.
    let engine = engine_on(with_components(
        BonusItem::Race(RaceKey::new("half-elf")),
        [Component::SpellPenetration {
            value: Value::Flat(1),
            only_while: None,
        }],
    ));
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
    let greater = || Fact::Feat(FeatKey::new("spell-penetration"));
    let engine = engine_on(with_components(
        BonusItem::MythicFeat(MythicFeatKey::new("dodge-mythic")),
        [
            Component::SpellPenetration {
                value: rank(RankBase::MythicLevel, RankProgression::Div2),
                only_while: Some(Condition::HasNot(greater())),
            },
            Component::SpellPenetration {
                value: rank(RankBase::MythicLevel, RankProgression::AsIs),
                only_while: Some(Condition::Has(greater())),
            },
        ],
    ));
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
    let engine = engine_on(with_components(
        BonusItem::Feature(FeatureKey::new("sorcerer/bloodline/arcane/arcane-bond")),
        [
            dc(SpellScope::Subschool(Subschool::Compulsion), 2),
            dc(SpellScope::Energy(Energy::Fire), 1),
            dc(SpellScope::Metamagic, 1),
        ],
    ));
    // The Arcane bloodline gives Arcane Bond at 1.
    let mut build = sorcerer(20);
    build.picks.insert(
        selection_slot(1, 0),
        pick_option("sorcerer/bloodline/arcane"),
    );
    let level = last_level(&engine, &build);
    let bonus_for = |spell: SpellTraits| {
        let line = engine.caster_line(&level, spell).unwrap();
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

/// The test world with Expanded Arsenal (a mythic feat taken for a school) adding up Spell Focus feats, and Spell
/// Focus taken once per school.
fn with_expanded_arsenal() -> Engine {
    let mut data = world();
    let template = data.mythic_feats[0].clone();
    data.mythic_feats.push(MythicFeat {
        components: vec![Component::ExpandedArsenal {
            focuses: [Fact::Feat(feat("spell-focus"))].into(),
        }],
        key: MythicFeatKey::new("expanded-arsenal"),
        name: "Expanded Arsenal".into(),
        parameter: Some(ParameterKind::SpellSchool),
        repeat: Repeat::PerValue,
        prerequisite: None,
        extra_pick: None,
        requires_proficiency: false,
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
    let level = last_level(&engine, &build);
    let line = |school| engine.caster_line(&level, spell(school)).unwrap();

    // Necromancy, an arsenal school, gets from each Expanded Arsenal held (two) the school whose Spell Focus feats add
    // up to the most: Evocation's, doubled by its mythic feat (2), level with Illusion's, doubled because an Expanded
    // Arsenal names Illusion, the first in school order.
    // Game: ExpandedArsenalMagicSchools.OnEventAboutToTrigger and SpellFocusParametrized.GetBonusDC, WrathDecompiled
    // 6ce4a6b
    let arsenal = [
        ("Charisma", 0),
        (
            "Expanded Arsenal (Illusion): Evocation's Spell Focus feats",
            2,
        ),
        (
            "Expanded Arsenal (Necromancy): Evocation's Spell Focus feats",
            2,
        ),
    ];
    assert_eq!(parts(&line(SpellSchool::Necromancy).dc_parts), arsenal);
    // Illusion is an arsenal school too: its own Spell Focus counts nothing for it (`SpellFocusParametrized` skips a
    // school with an arsenal entry), and the arsenal gives it Evocation's.
    assert_eq!(parts(&line(SpellSchool::Illusion).dc_parts), arsenal);
    // Evocation isn't: its doubled Spell Focus.
    assert_eq!(
        parts(&line(SpellSchool::Evocation).dc_parts),
        [("Charisma", 0), ("spell-focus (Evocation)", 2)]
    );
}

#[test]
fn heighten_spell_gives_the_highest_levels_dc_to_any_spell() {
    let mut data = world();
    let template = data.feats[0].clone();
    data.feats.push(Feat {
        components: vec![Component::Metamagic(MetamagicKind::Heighten)],
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
    components_mut(
        &mut data,
        &BonusItem::Feat(FeatKey::new("spell-penetration")),
    )
    .extend([
        dc(SpellScope::Value, 1),
        Component::SpellPenetration {
            value: Value::Flat(2),
            only_while: Some(Condition::Has(Fact::Feat(FeatKey::new("no-such-feat")))),
        },
    ]);
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
        "feat spell-penetration".into(),
        DataIssue::UnknownKey(rules::DataAt::Feat(FeatKey::new("no-such-feat")))
    )));
}

fn class_key(key: &str) -> rules::ClassKey {
    class(key)
}
