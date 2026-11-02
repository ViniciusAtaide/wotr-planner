//! What the sheet calls things: names by key, slot labels, picker categories and readable reasons for an Issue.

use std::collections::BTreeSet;

use rules::{
    Ability, Alignment, ArchetypeKey, FeatCategory, FeatureSelectionKey, Issue, MythicPathKey,
    OptionGroup, Parameter, Prerequisite, RaceKey, SpellType, Story, Unmet,
};

use crate::support::*;

fn needs(prerequisite: Prerequisite, has: Option<u8>) -> Unmet {
    Unmet::Condition {
        needs: prerequisite,
        has,
    }
}

#[test]
fn names_come_from_the_game_data() {
    let engine = engine();
    assert_eq!(engine.class_name(&class("rogue")), "Rogue");
    assert_eq!(
        engine.archetype_name(&ArchetypeKey::new("rogue/eldritch-scoundrel")),
        "Eldritch Scoundrel"
    );
    assert_eq!(engine.race_name(&RaceKey::new("half-elf")), "Half-Elf");
    assert_eq!(engine.skill_name(&skill("mobility")), "Mobility");
    assert_eq!(
        engine.feature_name(&feature("rogue/sneak-attack")),
        "Sneak Attack"
    );
    assert_eq!(
        engine.selection_name(&FeatureSelectionKey::new("rogue/rogue-talents")),
        "Rogue Talents"
    );
    assert_eq!(engine.path_name(&path("legend")), "Legend");
}

#[test]
fn a_key_the_data_lacks_is_named_by_its_key() {
    let engine = engine();
    assert_eq!(engine.class_name(&class("gunslinger")), "gunslinger");
    assert_eq!(engine.feat_name(&feat("no-such-feat")), "no-such-feat");
    assert_eq!(
        engine.path_name(&MythicPathKey::new("no-such-path")),
        "no-such-path"
    );
}

#[test]
fn a_pick_is_named_with_its_value() {
    let engine = engine();
    assert_eq!(engine.pick_name(&pick_feat("power-attack")), "power-attack");
    assert_eq!(
        engine.pick_name(&pick_feat_for("weapon-focus", weapon("longsword"))),
        "weapon-focus (longsword)"
    );
    assert_eq!(
        engine.pick_name(&pick_feat_for("skill-focus", skill_value("mobility"))),
        "skill-focus (Mobility)"
    );
    assert_eq!(
        engine.pick_name(&pick_option("rogue/rogue-talents/fast-stealth")),
        "fast-stealth"
    );
    assert_eq!(
        engine.parameter_name(&Parameter::SpellSchool(rules::SpellSchool::Evocation)),
        "Evocation"
    );
}

#[test]
fn ability_alignment_and_category_names_are_the_games() {
    assert_eq!(Ability::Strength.name(), "Strength");
    assert_eq!(Ability::Charisma.short_name(), "Cha");
    assert_eq!(Alignment::TrueNeutral.name(), "True Neutral");
    assert_eq!(Alignment::LawfulGood.short_name(), "LG");
    assert_eq!(FeatCategory::ArmourAndShield.name(), "Armour and Shield");
    assert_eq!(FeatCategory::CombatManoeuvres.name(), "Combat Manoeuvres");
}

#[test]
fn numeric_conditions_say_what_the_character_has() {
    let engine = engine();
    let text = |prerequisite, has| engine.issue_text(&Issue::Unmet(needs(prerequisite, has)));
    assert_eq!(
        text(Prerequisite::BaseAttackBonus(4), Some(1)),
        "Needs BAB +4 (have +1)"
    );
    assert_eq!(
        text(
            Prerequisite::AbilityScore {
                ability: Ability::Strength,
                score: 13
            },
            Some(10)
        ),
        "Needs Str 13 (have 10)"
    );
    assert_eq!(
        text(
            Prerequisite::ClassLevel {
                class: class("fighter"),
                level: 4
            },
            Some(1)
        ),
        "Needs Fighter level 4 (have 1)"
    );
    assert_eq!(
        text(
            Prerequisite::SkillRanks {
                skill: skill("mobility"),
                ranks: 5
            },
            Some(2)
        ),
        "Needs 5 ranks in Mobility (have 2)"
    );
    assert_eq!(
        text(
            Prerequisite::SpellLevel {
                spell_type: SpellType::Arcane,
                level: 3
            },
            Some(0)
        ),
        "Needs to cast 3rd-level arcane spells (can't cast any)"
    );
    assert_eq!(
        text(
            Prerequisite::SpontaneousSpellLevel {
                spell_type: SpellType::Divine,
                level: 1
            },
            Some(0)
        ),
        "Needs to cast 1st-level divine spells spontaneously (can't cast any)"
    );
    assert_eq!(
        text(Prerequisite::MythicRank(9), Some(3)),
        "Needs Mythic rank 9 (have 3)"
    );
    assert_eq!(
        text(Prerequisite::CharacterLevel(11), Some(9)),
        "Needs character level 11 (have 9)"
    );
    assert_eq!(
        text(Prerequisite::CasterLevel(5), None),
        "Needs caster level 5"
    );
}

#[test]
fn other_conditions_name_what_is_needed() {
    let engine = engine();
    let text = |prerequisite| engine.issue_text(&Issue::Unmet(needs(prerequisite, None)));
    assert_eq!(
        text(Prerequisite::Feat(feat("power-attack"))),
        "Needs power-attack"
    );
    assert_eq!(
        text(Prerequisite::FeatFor {
            feat: feat("weapon-focus"),
            value: weapon("dueling-sword")
        }),
        "Needs weapon-focus (dueling-sword)"
    );
    assert_eq!(
        text(Prerequisite::FeatForSameValue(feat("weapon-focus"))),
        "Needs weapon-focus in the same weapon"
    );
    assert_eq!(
        text(Prerequisite::Feature {
            feature: feature("rogue/sneak-attack"),
            count: 1
        }),
        "Needs Sneak Attack"
    );
    assert_eq!(
        text(Prerequisite::Alignment(Alignment::LawfulGood)),
        "Needs Lawful Good alignment"
    );
    assert_eq!(text(Prerequisite::Race(RaceKey::new("elf"))), "Needs Elf");
    assert_eq!(
        text(Prerequisite::NoOtherOptionOfGroup(OptionGroup::Draconic)),
        "Needs no other Draconic option"
    );
    assert_eq!(
        text(Prerequisite::NoOptionFrom(FeatureSelectionKey::new(
            "rogue/rogue-talents"
        ))),
        "Needs no Rogue Talents yet"
    );
    assert_eq!(
        text(Prerequisite::Not(Box::new(Prerequisite::ClassLevel {
            class: class("slayer"),
            level: 6
        }))),
        "Needs not Slayer level 6"
    );
    assert_eq!(
        text(Prerequisite::Story(Story::VowOfDahak)),
        "Needs Vow of Dahak"
    );
}

#[test]
fn groups_of_unmet_conditions_read_as_a_list() {
    let engine = engine();
    let all = Unmet::All(vec![
        needs(Prerequisite::Feat(feat("power-attack")), None),
        needs(Prerequisite::BaseAttackBonus(1), Some(0)),
    ]);
    assert_eq!(
        engine.issue_text(&Issue::Unmet(all)),
        "Needs power-attack, BAB +1 (have +0)"
    );
    let any = Unmet::Any(vec![
        needs(Prerequisite::Race(RaceKey::new("human")), None),
        needs(Prerequisite::Race(RaceKey::new("elf")), None),
    ]);
    assert_eq!(
        engine.issue_text(&Issue::Unmet(any.clone())),
        "Needs Human or Elf"
    );
    // An `Any` inside a longer list is bracketed so it reads unambiguously.
    let nested = Unmet::All(vec![needs(Prerequisite::BaseAttackBonus(1), Some(0)), any]);
    assert_eq!(
        engine.issue_text(&Issue::Unmet(nested)),
        "Needs BAB +1 (have +0), (Human or Elf)"
    );
    let at_least = Unmet::AtLeast {
        needed: 2,
        met: 1,
        failed: vec![
            needs(Prerequisite::Feat(feat("dodge")), None),
            needs(Prerequisite::Feat(feat("toughness")), None),
        ],
    };
    assert_eq!(
        engine.issue_text(&Issue::Unmet(at_least)),
        "Needs 1 more of: dodge, toughness"
    );
}

#[test]
fn the_other_issues_read_plainly() {
    let engine = engine();
    assert_eq!(engine.issue_text(&Issue::AlreadyHeld), "Already taken");
    assert_eq!(
        engine.issue_text(&Issue::MaxTimes { times: 3 }),
        "Already taken 3 times"
    );
    assert_eq!(
        engine.issue_text(&Issue::ClassLevelCap { cap: 10 }),
        "All 10 levels taken"
    );
    assert_eq!(
        engine.issue_text(&Issue::NotFromPath {
            from: BTreeSet::from([path("aeon"), path("trickster")])
        }),
        "Only from Aeon or Trickster"
    );
    assert_eq!(
        engine.issue_text(&Issue::BreaksLater),
        "Would break a later choice"
    );
    assert_eq!(engine.issue_text(&Issue::NotAllowed), "Not allowed here");
}

#[test]
fn a_picker_reason_comes_from_the_engine() {
    // Cleave at level 1 without Power Attack: the picker's reason reads as the game would say it.
    let engine = engine();
    let build = build("elf", &["fighter"]);
    let cleave = engine
        .options(&build, &general_slot(1))
        .unwrap()
        .into_iter()
        .find(|option| option.pick == pick_feat("cleave"))
        .unwrap();
    assert_eq!(
        engine.issue_text(&cleave.status.unwrap_err()),
        "Needs power-attack"
    );
}

#[test]
fn slots_are_labelled_by_what_they_take() {
    let engine = engine();
    let sheet = engine.evaluate(&build("human", &["fighter", "rogue", "rogue"]));
    let labels: Vec<String> = sheet.levels[0]
        .slots
        .iter()
        .map(|slot| engine.slot_label(slot))
        .collect();
    // Class slots, then the race's, then the general slot.
    assert_eq!(labels, ["Combat feat", "Bonus feat", "Feat"]);
    // A rogue's talent at its 2nd class level, before the general slot.
    let labels: Vec<String> = sheet.levels[2]
        .slots
        .iter()
        .map(|slot| engine.slot_label(slot))
        .collect();
    assert_eq!(labels, ["Rogue Talents", "Feat"]);
}

#[test]
fn picker_categories_follow_the_guides_groups() {
    let engine = engine();
    assert_eq!(engine.pick_category(&pick_feat("power-attack")), "Weapon");
    assert_eq!(engine.pick_category(&pick_feat("dodge")), "Miscellaneous");
    // A shared option keeps the category of the selection that defines it.
    assert_eq!(
        engine.pick_category(&pick_option("rogue/rogue-talents/fast-stealth")),
        "Rogue Talents"
    );
}
