//! Ticket 32's feat and rule follow-ups: the order picks are checked in within a level, mythic feats that take a
//! value, a prerequisite that depends on the pick's value (Armor Focus), feats granted for one value, options whose
//! pick takes a value of its own or from a short list, a race's own feat list, the slayer's combat style feats
//! (a prerequisite that must not hold), Perfect Mind's floors and Destiny Beyond Birth.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArmourType, Build, Choice, FeatSlot, Issue, MythicFeatKey, Parameter, Pick, Prerequisite,
    Problem, SkillKey, SlotId, SlotKind, SlotOption, SpellSchool, Unmet,
};

use crate::support::*;

fn problems(build: &Build) -> Vec<Problem> {
    engine().evaluate(build).problems
}

fn at(slot: SlotId, issue: Issue) -> Problem {
    Problem {
        at: Choice::Slot(slot),
        issue,
    }
}

fn options(build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("the Build has no slot {slot:?}"))
}

/// The values a feat is held for at the end of a level.
fn values_at(build: &Build, level: u8, key: &str) -> BTreeMap<Parameter, u8> {
    engine()
        .evaluate(build)
        .at(level)
        .unwrap()
        .feat_values
        .get(&feat(key))
        .cloned()
        .unwrap_or_default()
}

fn school(school: SpellSchool) -> Parameter {
    Parameter::SpellSchool(school)
}

fn armour(armour: ArmourType) -> Parameter {
    Parameter::ArmourType(armour)
}

fn pick_mythic_feat_for(key: &str, value: Parameter) -> Pick {
    Pick::mythic_feat_for(MythicFeatKey::new(key), value)
}

fn option_feat(level: u8) -> SlotId {
    SlotId::OptionFeat { level, index: 0 }
}

// ---- pick order within a level --------------------------------------------------------------------------------

#[test]
fn a_levels_slots_are_checked_class_selections_first_then_class_slots_race_slots_and_the_general_slot()
 {
    let mut druid = build("human", &["druid"]);
    druid.picks.insert(
        selection_slot(1, 0),
        pick_option("druid/nature-bond/animal-companion"),
    );
    let ids: Vec<SlotId> = engine().evaluate(&druid).levels[0]
        .slots
        .iter()
        .map(|slot| slot.id.clone())
        .collect();
    assert_eq!(ids, [selection_slot(1, 0), race_slot(0), general_slot(1)]);

    let warpriest = build("human", &["warpriest"]);
    let ids: Vec<SlotId> = engine().evaluate(&warpriest).levels[0]
        .slots
        .iter()
        .map(|slot| slot.id.clone())
        .collect();
    assert_eq!(ids, [class_slot(1, 0), race_slot(0), general_slot(1)]);
}

#[test]
fn a_human_warpriests_race_slot_takes_dazzling_display_on_the_class_slots_weapon_focus() {
    let mut build = build("human", &["warpriest"]);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build
        .picks
        .insert(race_slot(0), pick_feat("dazzling-display"));
    assert_eq!(problems(&build), []);
}

#[test]
fn a_human_druid_with_a_companion_takes_boon_companion_at_level_1() {
    let mut build = build("human", &["druid"]);
    build.picks.insert(
        selection_slot(1, 0),
        pick_option("druid/nature-bond/animal-companion"),
    );
    build
        .picks
        .insert(race_slot(0), pick_feat("boon-companion"));
    assert_eq!(problems(&build), []);
}

// ---- mythic feats with a value --------------------------------------------------------------------------------

/// A wizard of `levels` levels with Spell Focus (Evocation) at 1.
fn spell_focused(levels: usize) -> Build {
    let mut build = build("elf", &times("wizard", levels));
    build.picks.insert(
        general_slot(1),
        pick_feat_for("spell-focus", school(SpellSchool::Evocation)),
    );
    build
}

#[test]
fn spell_focus_mythic_needs_spell_focus_in_the_same_school() {
    let mut build = spell_focused(11);
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        pick_mythic_feat_for("spell-focus-mythic", school(SpellSchool::Evocation)),
    );
    build.picks.insert(
        SlotId::Mythic { rank: 4 },
        pick_mythic_feat_for("spell-focus-mythic", school(SpellSchool::Conjuration)),
    );
    assert_eq!(
        problems(&build),
        [at(
            SlotId::Mythic { rank: 4 },
            Issue::Unmet(Unmet::Condition {
                needs: Prerequisite::FeatForSameValue(feat("spell-focus")),
                has: None,
            })
        )]
    );
    let state = engine().evaluate(&build).at(11).unwrap().clone();
    assert_eq!(
        state.mythic_feat_values,
        BTreeMap::from([(
            MythicFeatKey::new("spell-focus-mythic"),
            BTreeSet::from([school(SpellSchool::Evocation)])
        )])
    );
}

#[test]
fn a_parameterised_mythic_feat_names_its_value_and_repeats_once_per_value() {
    let mut build = build("elf", &times("wizard", 11));
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        pick_mythic_feat_for("school-mastery", school(SpellSchool::Evocation)),
    );
    // Another school at MR 4 is fine; the same school again is not, and no value at all isn't either.
    build.picks.insert(
        SlotId::Mythic { rank: 4 },
        pick_mythic_feat_for("school-mastery", school(SpellSchool::Illusion)),
    );
    assert_eq!(problems(&build), []);
    assert_eq!(
        engine().evaluate(&build).at(11).unwrap().mythic_feat_values,
        BTreeMap::from([(
            MythicFeatKey::new("school-mastery"),
            BTreeSet::from([
                school(SpellSchool::Evocation),
                school(SpellSchool::Illusion)
            ])
        )])
    );
    build.picks.insert(
        SlotId::Mythic { rank: 4 },
        pick_mythic_feat_for("school-mastery", school(SpellSchool::Evocation)),
    );
    assert_eq!(
        problems(&build),
        [at(SlotId::Mythic { rank: 4 }, Issue::AlreadyHeld)]
    );
    build.picks.insert(
        SlotId::Mythic { rank: 4 },
        pick_mythic_feat("school-mastery"),
    );
    assert_eq!(
        problems(&build),
        [at(SlotId::Mythic { rank: 4 }, Issue::ParameterMissing)]
    );
    // A mythic feat that takes no value takes none.
    build.picks.insert(
        SlotId::Mythic { rank: 4 },
        pick_mythic_feat_for("toughness-mythic", school(SpellSchool::Evocation)),
    );
    assert_eq!(
        problems(&build),
        [at(SlotId::Mythic { rank: 4 }, Issue::WrongParameter)]
    );
}

#[test]
fn the_mythic_feat_picker_offers_a_parameterised_one_once_per_value() {
    let build = spell_focused(7);
    let offered: Vec<Pick> = options(&build, &SlotId::Mythic { rank: 2 })
        .into_iter()
        .map(|option| option.pick)
        .filter(|pick| pick.key() == "spell-focus-mythic")
        .collect();
    let expected: Vec<Pick> = SpellSchool::ALL
        .into_iter()
        .map(|value| pick_mythic_feat_for("spell-focus-mythic", school(value)))
        .collect();
    assert_eq!(offered, expected);
}

// ---- Armor Focus: the proficiency of its armour type ----------------------------------------------------------

#[test]
fn armor_focus_needs_the_armour_proficiency_of_the_armour_type_it_is_taken_for() {
    let mut build = build("elf", &times("fighter", 3));
    build
        .picks
        .insert(general_slot(1), pick_feat("armor-proficiency-heavy"));
    build.picks.insert(
        class_slot(2, 0),
        pick_feat_for("armor-focus", armour(ArmourType::Heavy)),
    );
    build.picks.insert(
        general_slot(3),
        pick_feat_for("armor-focus", armour(ArmourType::Light)),
    );
    assert_eq!(
        problems(&build),
        [at(
            general_slot(3),
            Issue::Unmet(Unmet::Condition {
                needs: Prerequisite::FeatIfValue {
                    value: armour(ArmourType::Light),
                    feat: feat("armor-proficiency-light"),
                },
                has: None,
            })
        )]
    );
}

// ---- feats granted for one value ------------------------------------------------------------------------------

#[test]
fn a_mantis_zealot_holds_its_weapon_feats_in_its_one_weapon() {
    let mut build = build("elf", &times("warpriest", 4));
    build.class_choices.insert(
        class("warpriest"),
        rules::ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("warpriest/mantis-zealot")),
            ..Default::default()
        },
    );
    assert_eq!(problems(&build), []);
    assert_eq!(
        values_at(&build, 1, "weapon-focus"),
        BTreeMap::from([(weapon("dueling-sword"), 1)])
    );
    assert_eq!(
        values_at(&build, 3, "weapon-specialization"),
        BTreeMap::new()
    );
    assert_eq!(
        values_at(&build, 4, "weapon-specialization"),
        BTreeMap::from([(weapon("dueling-sword"), 1)])
    );
    // Its level-1 row replaces the warpriest's: no Weapon Focus slot.
    let slots = &engine().evaluate(&build).levels[0].slots;
    assert!(
        !slots
            .iter()
            .any(|slot| matches!(slot.kind, SlotKind::Feat(FeatSlot::OneOf { .. })))
    );
}

// ---- options whose pick takes a value -------------------------------------------------------------------------

fn zen_archer(levels: usize) -> Build {
    let mut build = build("elf", &times("monk", levels));
    build.class_choices.insert(
        class("monk"),
        rules::ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("monk/zen-archer")),
            ..Default::default()
        },
    );
    build
}

const WAY: &str = "monk/zen-archer/way-of-the-bow/weapon-focus";

#[test]
fn a_selection_whose_one_option_takes_a_value_is_still_a_pick() {
    let build = zen_archer(2);
    let level_2 = &engine().evaluate(&build).levels[1];
    assert!(level_2.slots.iter().any(|slot| slot.kind
        == SlotKind::Selection(rules::FeatureSelectionKey::new(
            "monk/zen-archer/way-of-the-bow"
        ))));
    assert_eq!(level_2.state.option_count(&option(WAY)), 0);
}

#[test]
fn the_way_of_the_bow_offers_only_its_listed_weapons() {
    let build = zen_archer(2);
    let offered: Vec<Pick> = options(&build, &selection_slot(2, 0))
        .into_iter()
        .map(|option| option.pick)
        .collect();
    assert_eq!(
        offered,
        [
            pick_option_for(WAY, weapon("dagger")),
            pick_option_for(WAY, weapon("longsword")),
        ]
    );
    let mut build = build;
    build.picks.insert(
        selection_slot(2, 0),
        pick_option_for(WAY, weapon("bastard-sword")),
    );
    assert_eq!(
        problems(&build),
        [at(selection_slot(2, 0), Issue::WrongParameter)]
    );
}

#[test]
fn the_way_of_the_bow_gives_weapon_focus_at_2_and_weapon_specialization_in_the_same_weapon_at_6() {
    let mut build = zen_archer(6);
    build.picks.insert(
        selection_slot(2, 0),
        pick_option_for(WAY, weapon("longsword")),
    );
    assert_eq!(problems(&build), []);
    assert_eq!(
        values_at(&build, 2, "weapon-focus"),
        BTreeMap::from([(weapon("longsword"), 1)])
    );
    assert_eq!(
        values_at(&build, 5, "weapon-specialization"),
        BTreeMap::new()
    );
    assert_eq!(
        values_at(&build, 6, "weapon-specialization"),
        BTreeMap::from([(weapon("longsword"), 1)])
    );
}

#[test]
fn an_option_can_take_a_value_for_its_rows_alone() {
    let mut build = build("elf", &times("slayer", 8));
    build.picks.insert(
        selection_slot(2, 0),
        pick_option_for("slayer/slayer-talents/battle-master", weapon("dagger")),
    );
    assert_eq!(problems(&build), []);
    assert_eq!(
        values_at(&build, 7, "weapon-specialization"),
        BTreeMap::new()
    );
    assert_eq!(
        values_at(&build, 8, "weapon-specialization"),
        BTreeMap::from([(weapon("dagger"), 1)])
    );
    build.picks.insert(
        selection_slot(2, 0),
        pick_option("slayer/slayer-talents/battle-master"),
    );
    assert_eq!(
        problems(&build),
        [at(selection_slot(2, 0), Issue::ParameterMissing)]
    );
}

// ---- a race's own feat list -----------------------------------------------------------------------------------

#[test]
fn a_half_elfs_race_slot_is_skill_focus_in_a_skill_of_choice() {
    let build = build("half-elf", &["fighter"]);
    let offered: Vec<Pick> = options(&build, &race_slot(0))
        .into_iter()
        .map(|option| option.pick)
        .collect();
    let skills = ["athletics", "knowledge-arcana", "mobility", "persuasion"];
    let expected: Vec<Pick> = skills
        .into_iter()
        .map(|key| pick_feat_for("skill-focus", skill_value(key)))
        .collect();
    assert_eq!(offered, expected);
}

// ---- the slayer's combat style feats --------------------------------------------------------------------------

fn slayer(levels: usize) -> Build {
    build("elf", &times("slayer", levels))
}

fn combat_style(stage: u8) -> Pick {
    pick_option(&format!("slayer/slayer-talents/combat-style-feat-{stage}"))
}

#[test]
fn a_slayer_talent_at_2_can_be_a_combat_style_feat_from_the_first_list() {
    let mut build = slayer(2);
    build.picks.insert(selection_slot(2, 0), combat_style(2));
    build
        .picks
        .insert(option_feat(2), pick_feat("power-attack"));
    assert_eq!(problems(&build), []);
    let state = engine().evaluate(&build).at(2).unwrap().clone();
    assert_eq!(state.feat_count(&feat("power-attack")), 1);
    // Cleave comes with the second list.
    build.picks.insert(option_feat(2), pick_feat("cleave"));
    assert_eq!(problems(&build), [at(option_feat(2), Issue::NotAllowed)]);
}

#[test]
fn each_combat_style_feat_comes_in_its_own_window_of_levels() {
    // The first at 2 or 4, not from 6.
    let mut build = slayer(6);
    build.picks.insert(selection_slot(6, 0), combat_style(2));
    assert_eq!(
        problems(&build),
        [at(
            selection_slot(6, 0),
            Issue::Unmet(Unmet::Condition {
                needs: Prerequisite::Not(Box::new(Prerequisite::ClassLevel {
                    class: class("slayer"),
                    level: 6,
                })),
                has: None,
            })
        )]
    );
    // The second at 6 or 8, not before.
    let mut build = slayer(4);
    build.picks.insert(selection_slot(4, 0), combat_style(6));
    assert_eq!(
        problems(&build),
        [at(
            selection_slot(4, 0),
            Issue::Unmet(Unmet::Condition {
                needs: Prerequisite::ClassLevel {
                    class: class("slayer"),
                    level: 6,
                },
                has: Some(4),
            })
        )]
    );
}

#[test]
fn a_slayer_takes_three_combat_style_feats_at_most() {
    let mut build = slayer(12);
    for (level, stage) in [(2, 2), (6, 6), (10, 10)] {
        build
            .picks
            .insert(selection_slot(level, 0), combat_style(stage));
        build.picks.insert(option_feat(level), pick_feat("dodge"));
    }
    // Dodge each time: the lists ignore prerequisites but a feat is held once.
    build
        .picks
        .insert(option_feat(6), pick_feat("power-attack"));
    build.picks.insert(option_feat(10), pick_feat("cleave"));
    assert_eq!(problems(&build), []);
    build.picks.insert(selection_slot(12, 0), combat_style(10));
    assert_eq!(
        problems(&build),
        [at(selection_slot(12, 0), Issue::AlreadyHeld)]
    );
}

// ---- Perfect Mind's floors ------------------------------------------------------------------------------------

#[test]
fn perfect_mind_raises_bab_and_every_skills_ranks_for_prerequisites() {
    let mut build = build("human", &times("wizard", 17));
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("gold-dragon"));
    let sheet = engine().evaluate(&build);
    let bab_15 = Prerequisite::BaseAttackBonus(15);
    let mobility_9 = Prerequisite::SkillRanks {
        skill: SkillKey::new("mobility"),
        ranks: 9,
    };
    let at_16 = sheet.at(16).unwrap();
    assert!(at_16.check(&bab_15).is_err());
    assert!(at_16.check(&mobility_9).is_err());
    // MR 9 at 17: Perfect Mind.
    let at_17 = sheet.at(17).unwrap();
    assert_eq!(at_17.bab, 8);
    assert_eq!(at_17.prerequisite_bab(), 15);
    assert_eq!(at_17.check(&bab_15), Ok(()));
    assert_eq!(at_17.check(&mobility_9), Ok(()));
    assert!(
        at_17
            .check(&Prerequisite::SkillRanks {
                skill: SkillKey::new("mobility"),
                ranks: 10,
            })
            .is_err()
    );
}

// ---- Destiny Beyond Birth -------------------------------------------------------------------------------------

#[test]
fn destiny_beyond_birth_removes_the_races_ability_penalty() {
    let mut build = build("elf", &times("fighter", 7));
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        pick_mythic_feat("destiny-beyond-birth"),
    );
    assert_eq!(problems(&build), []);
    let sheet = engine().evaluate(&build);
    // Elf: Con 12 - 2. The bonuses stay (Dex and Int +2).
    assert_eq!(sheet.at(6).unwrap().ability_scores.constitution, 10);
    let at_7 = sheet.at(7).unwrap();
    assert_eq!(at_7.ability_scores.constitution, 12);
    assert_eq!(at_7.ability_scores.dexterity, 16);
    assert_eq!(at_7.base_ability_scores.constitution, 10);
}
