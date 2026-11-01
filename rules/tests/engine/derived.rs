//! Derived state at a character level: class levels, BAB, saves, ability scores, skills, spell-level access.

use rules::{Ability, ClassChoice, ClassKey, LevelUp, SkillPoints, SubraceKey};

use crate::support::*;

#[test]
fn multiclass_bab_and_saves_add_up_the_class_tables() {
    let build = build(
        "human",
        &["fighter", "fighter", "rogue", "rogue", "rogue", "wizard"],
    );
    let sheet = engine().evaluate(&build);

    // Fighter 2 (+2, F3 R0 W0) + Rogue 1 (+0, F0 R2 W0).
    let at_3 = sheet.at(3).expect("level 3");
    assert_eq!((at_3.bab, at_3.fort, at_3.reflex, at_3.will), (2, 3, 2, 0));

    // Fighter 2 + Rogue 3 (+2, F1 R3 W1) + Wizard 1 (+0, F0 R0 W2).
    let at_6 = sheet.at(6).expect("level 6");
    assert_eq!((at_6.bab, at_6.fort, at_6.reflex, at_6.will), (4, 4, 3, 3));
    assert_eq!(at_6.class_level(&class("fighter")), 2);
    assert_eq!(at_6.class_level(&class("rogue")), 3);
    assert_eq!(at_6.class_level(&class("wizard")), 1);
    assert_eq!(at_6.character_level, 6);
}

#[test]
fn ability_scores_add_racial_modifiers_and_the_increase_every_4_levels() {
    // Base Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8; elf: Dex +2, Int +2, Con -2.
    let mut build = build("elf", &times("fighter", 8));
    build.levels[3].ability_increase = Some(Ability::Strength);
    build.levels[7].ability_increase = Some(Ability::Strength);
    let sheet = engine().evaluate(&build);

    assert_eq!(
        sheet.at(1).unwrap().ability_scores,
        scores([16, 16, 10, 15, 10, 8])
    );
    assert_eq!(sheet.at(4).unwrap().ability_scores.strength, 17);
    assert_eq!(sheet.at(8).unwrap().ability_scores.strength, 18);
}

#[test]
fn a_human_puts_the_chosen_racial_bonus_where_the_build_says() {
    let mut build = build("human", &["fighter"]);
    build.start.racial_ability = Some(Ability::Strength);
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.at(1).unwrap().ability_scores.strength, 18);
}

#[test]
fn a_subrace_replaces_the_race_ability_modifiers() {
    // Blightborn: Dex +2, Wis +2, Con -2 instead of the elf's Int +2.
    let mut build = build("elf", &["fighter"]);
    build.start.subrace = Some(SubraceKey::new("elf/blightborn"));
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet.at(1).unwrap().ability_scores,
        scores([16, 16, 10, 13, 12, 8])
    );
}

#[test]
fn skill_ranks_accumulate_and_skill_points_come_from_class_and_intelligence() {
    // Int 13 (+1). Rogue: 8 + 1 = 9 points a level; fighter: 2 + 1 = 3.
    let mut build = build("dwarf", &["rogue", "fighter"]);
    build.levels[0].skill_ranks.insert(skill("mobility"), 1);
    build.levels[0].skill_ranks.insert(skill("athletics"), 1);
    build.levels[1].skill_ranks.insert(skill("mobility"), 1);
    let sheet = engine().evaluate(&build);

    let points = |available, spent| SkillPoints { available, spent };
    assert_eq!(sheet.levels[0].skill_points, points(9, 2));
    assert_eq!(sheet.levels[1].skill_points, points(3, 1));
    let at_2 = sheet.at(2).unwrap();
    assert_eq!(at_2.skill_ranks(&skill("mobility")), 2);
    assert_eq!(at_2.skill_ranks(&skill("athletics")), 1);
    assert_eq!(at_2.skill_ranks(&skill("persuasion")), 0);
}

#[test]
fn spell_level_access_comes_from_the_class_table() {
    let sheet = engine().evaluate(&build("elf", &times("wizard", 5)));
    assert_eq!(sheet.at(1).unwrap().arcane_spell_level, 1);
    assert_eq!(sheet.at(3).unwrap().arcane_spell_level, 2);
    assert_eq!(sheet.at(5).unwrap().arcane_spell_level, 3);
    assert_eq!(sheet.at(5).unwrap().divine_spell_level, 0);
    assert_eq!(sheet.at(5).unwrap().caster_level, 5);
}

/// Rogue 3 (Sneak Attack x2), Wizard 3 (2nd-level arcane), Mobility 4: ready for Arcane Trickster at level 7.
fn arcane_trickster_build(advances: Vec<ClassKey>) -> rules::Build {
    let mut build = build(
        "elf",
        &["rogue", "rogue", "rogue", "wizard", "wizard", "wizard"],
    );
    for level in &mut build.levels[..4] {
        level.skill_ranks.insert(skill("mobility"), 1);
    }
    for _ in 0..2 {
        build.levels.push(LevelUp::new(class("arcane-trickster")));
    }
    build.class_choices.insert(
        class("arcane-trickster"),
        ClassChoice {
            archetype: None,
            advances: advances.into_iter().collect(),
        },
    );
    build
}

#[test]
fn prestige_levels_advance_the_spell_level_access_of_the_class_chosen_on_entry() {
    let sheet = engine().evaluate(&arcane_trickster_build(vec![class("wizard")]));

    // Wizard 3 + Arcane Trickster 1 casts as a wizard 4, then as a wizard 5.
    assert_eq!(sheet.at(6).unwrap().arcane_spell_level, 2);
    assert_eq!(sheet.at(7).unwrap().arcane_spell_level, 2);
    assert_eq!(sheet.at(8).unwrap().arcane_spell_level, 3);
    assert_eq!(sheet.at(8).unwrap().caster_level, 5);
    assert_eq!(
        sheet
            .at(8)
            .unwrap()
            .feature_count(&feature("rogue/sneak-attack")),
        3
    );
}

#[test]
fn prestige_levels_advance_nothing_without_a_choice_on_entry() {
    let sheet = engine().evaluate(&arcane_trickster_build(vec![]));
    assert_eq!(sheet.at(8).unwrap().arcane_spell_level, 2);
}
