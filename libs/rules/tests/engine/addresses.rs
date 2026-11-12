//! Choice addresses (ticket 152, moved from the MCP server's tests): every kind written and read back, every choice
//! of the test world given an address and read back to the same choice, every value without a game-data key written
//! as a key and read back, and the labels each choice is named by.

use std::collections::BTreeSet;

use rules::{
    Ability, AbilityScores, Address, Alignment, Choice, ClassKey, Faith, FeatSource, GearPlace,
    GearSpot, Parameter, ParameterKind, Place, SkillKey, SlotId,
};

use crate::support::*;

/// One address of every kind, with its text: every Build start choice, every level choice, the class choices, the
/// Mythic track's choices and every `SlotId` variant (each feat source).
fn every_kind() -> Vec<(Address, &'static str)> {
    let feat = |source| SlotId::Feat {
        level: 3,
        source,
        index: 0,
    };
    vec![
        (Address::Race, "start.race"),
        (Address::Subrace, "start.subrace"),
        (Address::Faith, "start.faith"),
        (Address::Alignment, "start.alignment"),
        (Address::Background, "start.background"),
        (Address::RacialAbility, "start.racial-ability"),
        (Address::Score(Ability::Strength), "start.score.str"),
        (Address::Score(Ability::Dexterity), "start.score.dex"),
        (Address::Score(Ability::Constitution), "start.score.con"),
        (Address::Score(Ability::Intelligence), "start.score.int"),
        (Address::Score(Ability::Wisdom), "start.score.wis"),
        (Address::Score(Ability::Charisma), "start.score.cha"),
        (Address::Level(12), "level.12"),
        (Address::Class(4), "level.4.class"),
        (Address::Increase(4), "level.4.increase"),
        (Address::SkillPoints(4), "level.4.skill"),
        (
            Address::Skill(4, SkillKey::new("knowledge-arcana")),
            "level.4.skill.knowledge-arcana",
        ),
        (
            Address::Archetype(ClassKey::new("arcanist")),
            "class.arcanist.archetype",
        ),
        (
            Address::Advances(ClassKey::new("eldritch-knight")),
            "class.eldritch-knight.advances",
        ),
        (Address::FirstAscension, "mythic.first-ascension"),
        (Address::Path, "mythic.path"),
        (Address::Switch, "mythic.switch"),
        (Address::Slot(feat(FeatSource::Race)), "slot.feat.3.race.0"),
        (
            Address::Slot(feat(FeatSource::Class)),
            "slot.feat.3.class.0",
        ),
        (
            Address::Slot(feat(FeatSource::General)),
            "slot.feat.3.general.0",
        ),
        (
            Address::Slot(SlotId::Selection { level: 1, index: 2 }),
            "slot.selection.1.2",
        ),
        (Address::Slot(SlotId::Mythic { rank: 5 }), "slot.mythic.5"),
        (
            Address::Slot(SlotId::PathFeat { rank: 9, index: 1 }),
            "slot.path-feat.9.1",
        ),
        (
            Address::Slot(SlotId::PathSelection { rank: 3, index: 0 }),
            "slot.path-selection.3.0",
        ),
        (
            Address::Slot(SlotId::OptionSelection {
                level: 15,
                index: 0,
            }),
            "slot.option-selection.15.0",
        ),
        (
            Address::Slot(SlotId::MythicSelection { rank: 4, index: 0 }),
            "slot.mythic-selection.4.0",
        ),
        (
            Address::Slot(SlotId::OptionFeat { level: 8, index: 1 }),
            "slot.option-feat.8.1",
        ),
        (
            Address::Slot(SlotId::FeatSelection { level: 6, index: 0 }),
            "slot.feat-selection.6.0",
        ),
        (
            Address::Slot(SlotId::MythicExtra { rank: 6 }),
            "slot.mythic-extra.6",
        ),
        (Address::RankLevel(10), "mythic.10.level"),
        (Address::Gear(GearSpot::Worn(GearPlace::Belt)), "gear.belt"),
        (
            Address::Gear(GearSpot::Worn(GearPlace::FirstRing)),
            "gear.ring.0",
        ),
        (
            Address::Gear(GearSpot::Worn(GearPlace::SecondRing)),
            "gear.ring.1",
        ),
        (Address::Gear(GearSpot::Worn(GearPlace::Body)), "gear.body"),
        (
            Address::Gear(GearSpot::Used(gear_item("manual-of-dexterity-2"))),
            "gear.usable.manual-of-dexterity-2",
        ),
    ]
}

#[test]
fn every_kind_of_address_is_written_and_read_back() {
    for (address, text) in every_kind() {
        assert_eq!(address.to_string(), text);
        assert_eq!(text.parse::<Address>(), Ok(address.clone()), "{text}");
        // Read back from its own text, whatever that is.
        assert_eq!(address.to_string().parse::<Address>(), Ok(address));
    }
}

#[test]
fn every_engine_choice_has_an_address_and_back() {
    for (address, text) in every_kind() {
        match address.choice() {
            Some(choice) => assert_eq!(Address::from(&choice), address, "{text}"),
            // The engine keeps the racial ability with the race.
            None => assert_eq!(address, Address::RacialAbility),
        }
    }
}

/// Which variant of `Choice` a choice is: an exhaustive match, so a new variant fails to compile here until the
/// round trip below covers it.
fn variant(choice: &Choice) -> &'static str {
    match choice {
        Choice::Race => "Race",
        Choice::Subrace => "Subrace",
        Choice::Alignment => "Alignment",
        Choice::Background => "Background",
        Choice::Deity => "Deity",
        Choice::AbilityScore { .. } => "AbilityScore",
        Choice::Class { .. } => "Class",
        Choice::Archetype { .. } => "Archetype",
        Choice::Advances { .. } => "Advances",
        Choice::AbilityIncrease { .. } => "AbilityIncrease",
        Choice::SkillRanks { .. } => "SkillRanks",
        Choice::SkillPoints { .. } => "SkillPoints",
        Choice::CharacterLevel { .. } => "CharacterLevel",
        Choice::FirstAscension => "FirstAscension",
        Choice::MythicPath => "MythicPath",
        Choice::PathSwitch => "PathSwitch",
        Choice::Slot(_) => "Slot",
        Choice::Gear(_) => "Gear",
        Choice::RankLevel { .. } => "RankLevel",
    }
}

/// Every choice the test world can name: the Build start's, each ability's score, each level's (40 of them, Legend's
/// cap) with each skill's ranks, each class's archetype and advanced classes, the Mythic track's, and every slot of
/// every kind at each level and rank.
fn every_choice_of_the_test_world() -> Vec<Choice> {
    let world = world();
    let mut choices = vec![
        Choice::Race,
        Choice::Subrace,
        Choice::Alignment,
        Choice::Background,
        Choice::Deity,
        Choice::FirstAscension,
        Choice::MythicPath,
        Choice::PathSwitch,
    ];
    choices.extend(
        AbilityScores::ABILITIES
            .into_iter()
            .map(|ability| Choice::AbilityScore { ability }),
    );
    for class in world.classes.iter().chain(&world.prestige_classes) {
        choices.push(Choice::Archetype {
            class: class.key.clone(),
        });
        choices.push(Choice::Advances {
            class: class.key.clone(),
        });
    }
    for level in 1..=40 {
        choices.extend([
            Choice::CharacterLevel { level },
            Choice::Class { level },
            Choice::AbilityIncrease { level },
            Choice::SkillPoints { level },
        ]);
        choices.extend(world.skills.iter().map(|skill| Choice::SkillRanks {
            level,
            skill: skill.key.clone(),
        }));
        for index in 0..3 {
            choices.extend(
                [FeatSource::Race, FeatSource::Class, FeatSource::General].map(|source| {
                    Choice::Slot(SlotId::Feat {
                        level,
                        source,
                        index,
                    })
                }),
            );
            choices.extend(
                [
                    SlotId::Selection { level, index },
                    SlotId::OptionSelection { level, index },
                    SlotId::OptionFeat { level, index },
                    SlotId::FeatSelection { level, index },
                ]
                .map(Choice::Slot),
            );
        }
    }
    choices.extend(
        GearPlace::ALL
            .into_iter()
            .map(|place| Choice::Gear(GearSpot::Worn(place))),
    );
    choices.extend((world.gear.iter()).map(|item| Choice::Gear(GearSpot::Used(item.key.clone()))));
    for rank in 1..=10 {
        choices.push(Choice::RankLevel { rank });
        choices.push(Choice::Slot(SlotId::Mythic { rank }));
        choices.push(Choice::Slot(SlotId::MythicExtra { rank }));
        for index in 0..3 {
            choices.extend(
                [
                    SlotId::PathFeat { rank, index },
                    SlotId::PathSelection { rank, index },
                    SlotId::MythicSelection { rank, index },
                ]
                .map(Choice::Slot),
            );
        }
    }
    choices
}

#[test]
fn every_choice_of_the_test_world_reads_back_from_its_address() {
    let choices = every_choice_of_the_test_world();
    for choice in &choices {
        let text = Address::from(choice).to_string();
        let address: Address = text.parse().unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(address.choice().as_ref(), Some(choice), "{text}");
    }
    // Every variant of `Choice` is among them.
    let covered: BTreeSet<&str> = choices.iter().map(variant).collect();
    assert_eq!(covered.len(), 19, "{covered:?}");
}

/// Every parameter kind. The match is exhaustive and each arm names a kind's place in the array, so a new kind fails
/// to compile here until the array (and with it the round trip below) holds it.
fn every_parameter_kind() -> [ParameterKind; 9] {
    use ParameterKind::*;
    let kinds = [
        Skill,
        Weapon,
        ExoticWeapon,
        SpellSchool,
        Energy,
        ArmourType,
        Metamagic,
        Stat,
        Element,
    ];
    let [
        skill,
        weapon,
        exotic,
        school,
        energy,
        armour,
        metamagic,
        stat,
        element,
    ] = kinds;
    for kind in kinds {
        let placed = match kind {
            Skill => skill,
            Weapon => weapon,
            ExoticWeapon => exotic,
            SpellSchool => school,
            Energy => energy,
            ArmourType => armour,
            Metamagic => metamagic,
            Stat => stat,
            Element => element,
        };
        assert_eq!(placed, kind);
    }
    kinds
}

#[test]
fn every_value_without_a_game_data_key_reads_back_from_its_key() {
    let engine = engine();
    for ability in AbilityScores::ABILITIES {
        assert_eq!(Ability::from_key(ability.key()), Some(ability));
    }
    for alignment in Alignment::ALL {
        assert_eq!(Alignment::from_key(alignment.key()), Some(alignment));
    }
    let faiths = std::iter::once(Faith::Atheism).chain(
        engine
            .deities()
            .iter()
            .map(|deity| Faith::Deity(deity.key.clone())),
    );
    for faith in faiths {
        assert_eq!(Faith::from_key(faith.key()), Some(faith));
    }
    for kind in every_parameter_kind() {
        let values = engine.parameter_values(kind);
        // Each kind for real; the test world has no metamagic feat, so the game data's round trip covers Metamagic
        // (`seeds`' `game_data::value_keys`).
        assert!(
            !values.is_empty() || kind == ParameterKind::Metamagic,
            "no {kind:?} in the test world"
        );
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
fn value_keys_are_the_ones_agents_already_use() {
    let keys: Vec<&str> = AbilityScores::ABILITIES
        .into_iter()
        .map(Ability::key)
        .collect();
    assert_eq!(keys, ["str", "dex", "con", "int", "wis", "cha"]);
    let keys: Vec<&str> = Alignment::ALL.into_iter().map(Alignment::key).collect();
    assert_eq!(
        keys,
        [
            "lawful-good",
            "neutral-good",
            "chaotic-good",
            "lawful-neutral",
            "true-neutral",
            "chaotic-neutral",
            "lawful-evil",
            "neutral-evil",
            "chaotic-evil",
        ]
    );
    assert_eq!(Faith::Atheism.key(), "atheism");
    // A parameter without a game-data key: its name in lower case with dashes, written out.
    assert_eq!(
        Parameter::Stat(rules::Stat::AttacksOfOpportunity).value_key(),
        "attacks-of-opportunity"
    );
    assert_eq!(
        Parameter::Element(rules::Element::NegativeEnergy).value_key(),
        "negative-energy"
    );
    assert_eq!(Parameter::Stat(rules::Stat::ArmourClass).value_key(), "ac");
    assert_eq!(weapon("longsword").value_key(), "longsword");
}

#[test]
fn a_key_is_read_in_any_case_and_an_ability_by_its_full_name() {
    assert_eq!(
        Ability::from_key(" Intelligence "),
        Some(Ability::Intelligence)
    );
    assert_eq!(Ability::from_key("CHA"), Some(Ability::Charisma));
    assert_eq!(Ability::from_key("luck"), None);
    assert_eq!(
        Alignment::from_key("Lawful-Good"),
        Some(Alignment::LawfulGood)
    );
    assert_eq!(Alignment::from_key("lawful good"), None);
    assert_eq!(Faith::from_key("Atheism"), Some(Faith::Atheism));
    assert_eq!(
        Faith::from_key(" Lawgiver "),
        Some(Faith::Deity(rules::DeityKey::new("lawgiver")))
    );
    assert_eq!(
        "start.score.Intelligence".parse::<Address>(),
        Ok(Address::Score(Ability::Intelligence))
    );
}

#[test]
fn an_empty_or_blank_key_names_no_value() {
    for text in ["", " ", "\t"] {
        assert_eq!(Faith::from_key(text), None, "{text:?}");
        assert_eq!(Ability::from_key(text), None, "{text:?}");
        assert_eq!(Alignment::from_key(text), None, "{text:?}");
    }
}

#[test]
fn text_outside_the_grammar_is_no_address() {
    for text in [
        "",
        "start",
        "start.deity",
        "start.score.luck",
        "level.0.class",
        "level.x.class",
        "level.4.skill.",
        "level.4.feat",
        "class..archetype",
        "class.fighter",
        "mythic.rank",
        "slot.feat.3.mythic.0",
        "slot.feat.3.general",
        "slot.mythic.0",
        "slot.selection.1.300",
        "slot.unknown.1.0",
    ] {
        assert!(text.parse::<Address>().is_err(), "{text:?} parsed");
    }
}

#[test]
fn each_choice_is_labelled_in_sentence_case_after_its_place() {
    let engine = engine();
    let label = |address: Address| engine.placed_label(&address, None);
    assert_eq!(label(Address::Race), "Race");
    // The glossary's Faith: a deity, or atheism.
    assert_eq!(label(Address::Faith), "Faith");
    assert_eq!(label(Address::RacialAbility), "Racial ability bonus");
    assert_eq!(
        label(Address::Score(Ability::Strength)),
        "Strength (point buy)"
    );
    // A level is its own place: named once.
    assert_eq!(label(Address::Level(4)), "Level 4");
    assert_eq!(
        Address::Level(4).placed(&engine.choice_label(&Address::Level(4), None)),
        "Level 4"
    );
    assert_eq!(label(Address::Class(4)), "Level 4: Class");
    assert_eq!(label(Address::Increase(4)), "Level 4: Ability increase");
    assert_eq!(label(Address::SkillPoints(4)), "Level 4: Skill ranks");
    assert_eq!(
        label(Address::Skill(4, skill("athletics"))),
        "Level 4: Athletics ranks"
    );
    assert_eq!(
        label(Address::Archetype(class("fighter"))),
        "Fighter: archetype"
    );
    assert_eq!(
        label(Address::Advances(class("fighter"))),
        "Fighter: advanced classes"
    );
    assert_eq!(label(Address::FirstAscension), "MR 1: First ascension");
    assert_eq!(label(Address::Path), "MR 3: Mythic path");
    assert_eq!(label(Address::Switch), "MR 8: Path switch");
    assert_eq!(
        label(Address::Slot(SlotId::Mythic { rank: 2 })),
        "MR 2: slot"
    );
    // Unplaced, as the sheet's lines and the pickers' headings name them.
    assert_eq!(
        engine.choice_label(&Address::Increase(4), None),
        "Ability increase"
    );
    assert_eq!(engine.choice_label(&Address::Class(4), None), "Class");
}

#[test]
fn a_place_is_short_in_a_label_and_long_in_a_heading() {
    assert_eq!(Address::Race.place(), Some(Place::Start));
    assert_eq!(Address::Increase(4).place(), Some(Place::Level(4)));
    assert_eq!(Address::Path.place(), Some(Place::Rank(3)));
    assert_eq!(Address::Archetype(class("fighter")).place(), None);
    assert_eq!(Place::Rank(3).to_string(), "MR 3");
    assert_eq!(Place::Rank(3).heading(), "Mythic rank 3");
    assert_eq!(Place::Level(4).heading(), "Level 4");
    assert_eq!(Place::Start.heading(), "Build start");
    assert_eq!(
        Address::Increase(4).placed("2 of 4 skill points unspent"),
        "Level 4: 2 of 4 skill points unspent"
    );
    assert_eq!(Address::Race.placed("Race"), "Race");
}

#[test]
fn an_ability_increase_is_named_by_its_ability() {
    assert_eq!(Ability::Strength.increase_name(), "+1 Strength");
    assert_eq!(Ability::Strength.increase_short_name(), "+1 Str");
}
