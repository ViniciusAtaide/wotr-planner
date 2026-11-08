//! Choice addresses: every kind written and read back, and every engine choice given an address.

use mcp::Address;
use rules::{Ability, Choice, ClassKey, FeatSource, SkillKey, SlotId};

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
    // And the other way: each choice the engine names comes back from its address.
    let choices = [
        Choice::Race,
        Choice::Subrace,
        Choice::Alignment,
        Choice::Background,
        Choice::Deity,
        Choice::AbilityScore {
            ability: Ability::Wisdom,
        },
        Choice::Class { level: 2 },
        Choice::Archetype {
            class: ClassKey::new("fighter"),
        },
        Choice::Advances {
            class: ClassKey::new("mystic-theurge"),
        },
        Choice::AbilityIncrease { level: 8 },
        Choice::SkillRanks {
            level: 2,
            skill: SkillKey::new("athletics"),
        },
        Choice::SkillPoints { level: 2 },
        Choice::CharacterLevel { level: 20 },
        Choice::FirstAscension,
        Choice::MythicPath,
        Choice::PathSwitch,
        Choice::Slot(SlotId::Mythic { rank: 1 }),
    ];
    for choice in choices {
        let address = Address::from(&choice);
        assert_eq!(address.choice(), Some(choice.clone()), "{address}");
        assert_eq!(address.to_string().parse::<Address>(), Ok(address));
    }
}

#[test]
fn an_ability_may_be_named_in_full() {
    assert_eq!(
        "start.score.Intelligence".parse::<Address>(),
        Ok(Address::Score(Ability::Intelligence))
    );
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
