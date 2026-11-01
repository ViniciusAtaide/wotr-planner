//! The prerequisite-line grammar: one guide line in, an unresolved condition out.

use extract::grammar::{Cond, parse_line, parse_requirement, prerequisite_lines};
use rules::{Ability, Alignment, SpellType};

#[test]
fn an_ability_score_line() {
    assert_eq!(
        parse_line("Strength: 13"),
        Ok(Cond::AbilityScore(Ability::Strength, 13))
    );
}

fn lines(text: &[&str]) -> Vec<String> {
    text.iter().map(|line| (*line).to_owned()).collect()
}

#[test]
fn a_paragraph_without_the_label_has_no_prerequisites() {
    assert_eq!(
        prerequisite_lines(&lines(&["+4 to Perception checks."])),
        None
    );
}

#[test]
fn the_label_is_found_however_it_is_spelt() {
    for label in [
        "Prerequisites:",
        "Prerequsites:",
        "Prerequsities:",
        "Prerequisites",
    ] {
        assert_eq!(
            prerequisite_lines(&lines(&[label, "Strength: 13", "Power Attack"])),
            Some(lines(&["Strength: 13", "Power Attack"])),
            "{label}"
        );
    }
}

#[test]
fn the_first_condition_can_share_the_labels_line_or_follow_an_empty_one() {
    assert_eq!(
        prerequisite_lines(&lines(&["Prerequisites: Base Attack Bonus: 6"])),
        Some(lines(&["Base Attack Bonus: 6"]))
    );
    assert_eq!(
        prerequisite_lines(&lines(&["Prerequisites: ", "", "Ki Pool"])),
        Some(lines(&["Ki Pool"]))
    );
    // Other facts can come before the label.
    assert_eq!(
        prerequisite_lines(&lines(&["Cost: 2", "Prerequisites:", "Flame Arc"])),
        Some(lines(&["Flame Arc"]))
    );
}

#[test]
fn a_line_ending_in_or_continues_on_the_next() {
    assert_eq!(
        prerequisite_lines(&lines(&[
            "Prerequisites:",
            "Plague Mutation: Strength Damage or",
            "Plague Mutation: Wisdom Damage",
            "Level 6"
        ])),
        Some(lines(&[
            "Plague Mutation: Strength Damage or Plague Mutation: Wisdom Damage",
            "Level 6"
        ]))
    );
}

#[test]
fn prestige_requirements_list_conditions_that_all_hold() {
    assert_eq!(
        parse_requirement("Required feats: Dodge, Weapon Focus (Dueling Sword)"),
        Some(Ok(vec![
            named("Dodge"),
            named("Weapon Focus (Dueling Sword)")
        ]))
    );
    assert_eq!(
        parse_requirement("Required skills: Mobility (4), Knowledge: Arcana (4);"),
        Some(Ok(vec![
            counted("Mobility", 4),
            counted("Knowledge: Arcana", 4)
        ]))
    );
    assert_eq!(
        parse_requirement("Required special: Sneak Attack (2d6), 2nd level arcane spells"),
        Some(Ok(vec![
            counted("Sneak Attack", 2),
            Cond::SpellLevel(SpellType::Arcane, 2)
        ]))
    );
    assert_eq!(
        parse_requirement("Required BAB: 5"),
        Some(Ok(vec![Cond::Bab(5)]))
    );
    assert_eq!(
        parse_requirement("Minimum character level: 6"),
        Some(Ok(vec![Cond::CharacterLevel(6)]))
    );
}

#[test]
fn a_requirement_item_can_hold_alternatives() {
    assert_eq!(
        parse_requirement("Required feats: Arcane Armor Training or Warrior Priest"),
        Some(Ok(vec![Cond::Any(vec![
            named("Arcane Armor Training"),
            named("Warrior Priest")
        ])]))
    );
    assert_eq!(
        parse_requirement("Required special: 3rd level arcane spells or 3rd level divine spells"),
        Some(Ok(vec![Cond::Any(vec![
            Cond::SpellLevel(SpellType::Arcane, 3),
            Cond::SpellLevel(SpellType::Divine, 3)
        ])]))
    );
}

#[test]
fn other_characteristics_are_not_requirements() {
    assert_eq!(parse_requirement("6 HP per level"), None);
    assert_eq!(parse_requirement("Full caster level progression"), None);
}

#[test]
fn a_missing_colon_reads_the_same() {
    assert_eq!(
        parse_line("Dexterity 13"),
        Ok(Cond::AbilityScore(Ability::Dexterity, 13))
    );
}

#[test]
fn base_attack_bonus_in_full_or_abbreviated() {
    assert_eq!(parse_line("Base Attack Bonus: 6"), Ok(Cond::Bab(6)));
    assert_eq!(parse_line("BAB: 8"), Ok(Cond::Bab(8)));
}

#[test]
fn class_levels_with_any_spelling() {
    let fighter = |level| Ok(Cond::ClassLevel("Fighter".into(), level));
    assert_eq!(parse_line("Fighter level: 8"), fighter(8));
    assert_eq!(parse_line("Fighter level 5"), fighter(5));
    assert_eq!(parse_line("Fighter Level 6"), fighter(6));
}

#[test]
fn character_level_own_class_level_and_mythic_rank() {
    assert_eq!(
        parse_line("Character level: 10"),
        Ok(Cond::CharacterLevel(10))
    );
    assert_eq!(parse_line("Character level 9"), Ok(Cond::CharacterLevel(9)));
    // On a class page, a bare level is that class's level.
    assert_eq!(parse_line("Level 10"), Ok(Cond::OwnClassLevel(10)));
    assert_eq!(parse_line("Rank 5"), Ok(Cond::MythicRank(5)));
    assert_eq!(parse_line("Mythic Rank: 9"), Ok(Cond::MythicRank(9)));
}

fn named(name: &str) -> Cond {
    Cond::Named {
        name: name.into(),
        count: None,
    }
}

fn counted(name: &str, count: u8) -> Cond {
    Cond::Named {
        name: name.into(),
        count: Some(count),
    }
}

#[test]
fn a_bare_name_is_resolved_later() {
    assert_eq!(parse_line("Power Attack"), Ok(named("Power Attack")));
    assert_eq!(
        parse_line("Weapon Focus (Dueling Sword)"),
        Ok(named("Weapon Focus (Dueling Sword)"))
    );
    // A colon followed by words is part of the name.
    assert_eq!(
        parse_line("Plague Mutation: Strength Damage"),
        Ok(named("Plague Mutation: Strength Damage"))
    );
}

#[test]
fn a_name_with_a_number_is_skill_ranks_or_a_feature_count() {
    assert_eq!(parse_line("Persuasion: 6"), Ok(counted("Persuasion", 6)));
    assert_eq!(parse_line("Mobility 1"), Ok(counted("Mobility", 1)));
    assert_eq!(
        parse_line("Sneak Attack: 1"),
        Ok(counted("Sneak Attack", 1))
    );
    // Dice count as the number of times the feature was granted.
    assert_eq!(
        parse_line("Sneak Attack (2d6)"),
        Ok(counted("Sneak Attack", 2))
    );
}

#[test]
fn or_joins_alternatives_of_any_kind() {
    assert_eq!(
        parse_line("Monk level: 1 or Base Attack Bonus: 2"),
        Ok(Cond::Any(vec![
            Cond::ClassLevel("Monk".into(), 1),
            Cond::Bab(2)
        ]))
    );
    assert_eq!(
        parse_line("Smite Evil / Smite Chaos"),
        Ok(Cond::Any(vec![named("Smite Evil"), named("Smite Chaos")]))
    );
}

#[test]
fn plus_binds_tighter_than_or() {
    assert_eq!(
        parse_line("Combat Expertise or Kinetic Warrior + Intelligence 13"),
        Ok(Cond::Any(vec![
            named("Combat Expertise"),
            Cond::All(vec![
                named("Kinetic Warrior"),
                Cond::AbilityScore(Ability::Intelligence, 13)
            ]),
        ]))
    );
}

#[test]
fn a_semicolon_separates_conditions_that_all_hold() {
    assert_eq!(
        parse_line("Beast Totem, Lesser; Barbarian level: 6"),
        Ok(Cond::All(vec![
            named("Beast Totem, Lesser"),
            Cond::ClassLevel("Barbarian".into(), 6)
        ]))
    );
}

#[test]
fn one_of_and_two_of_lists() {
    assert_eq!(
        parse_line("One of: Bull Rush, Dirty Trick, Trip"),
        Ok(Cond::Any(vec![
            named("Bull Rush"),
            named("Dirty Trick"),
            named("Trip")
        ]))
    );
    assert_eq!(
        parse_line("Two of: Blinding Critical, Sickening Critical, Tiring Critical"),
        Ok(Cond::AtLeast(
            2,
            vec![
                named("Blinding Critical"),
                named("Sickening Critical"),
                named("Tiring Critical")
            ]
        ))
    );
}

#[test]
fn a_comma_list_is_alternatives() {
    assert_eq!(
        parse_line("Elf, Dwarf or Gnome"),
        Ok(Cond::Any(vec![
            named("Elf"),
            named("Dwarf"),
            named("Gnome")
        ]))
    );
    assert_eq!(
        parse_line("Elf, Half Elf"),
        Ok(Cond::Any(vec![named("Elf"), named("Half Elf")]))
    );
}

#[test]
fn commas_and_or_inside_a_name_do_not_split_it() {
    assert_eq!(
        parse_line("Beast Totem, Lesser"),
        Ok(named("Beast Totem, Lesser"))
    );
    assert_eq!(
        parse_line("Shield Proficiency (Buckler, Light or Heavy)"),
        Ok(named("Shield Proficiency (Buckler, Light or Heavy)"))
    );
}

#[test]
fn the_ability_to_cast_spells_is_spell_level_access() {
    assert_eq!(
        parse_line("Ability to cast arcane spells"),
        Ok(Cond::SpellLevel(SpellType::Arcane, 1))
    );
    assert_eq!(
        parse_line("Ability to cast 1st level divine spells"),
        Ok(Cond::SpellLevel(SpellType::Divine, 1))
    );
    assert_eq!(
        parse_line("Ability to cast 3rd-level arcane spells"),
        Ok(Cond::SpellLevel(SpellType::Arcane, 3))
    );
    assert_eq!(
        parse_line("2nd level arcane spells"),
        Ok(Cond::SpellLevel(SpellType::Arcane, 2))
    );
}

#[test]
fn alignment_exclusions_and_neighbours() {
    use Alignment::*;
    let not_lawful = Ok(Cond::Alignment(vec![
        NeutralGood,
        ChaoticGood,
        TrueNeutral,
        ChaoticNeutral,
        NeutralEvil,
        ChaoticEvil,
    ]));
    assert_eq!(parse_line("Alignment must be non-Lawful."), not_lawful);
    assert_eq!(parse_line("Cannot be Lawful."), not_lawful);
    assert_eq!(
        parse_line("Alignment must be within one place of Neutral."),
        Ok(Cond::Alignment(vec![
            NeutralGood,
            LawfulNeutral,
            TrueNeutral,
            ChaoticNeutral,
            NeutralEvil
        ]))
    );
    // Next to full alignments, a lone "Neutral" is true neutral.
    assert_eq!(
        parse_line("Alignment must be Neutral, Neutral Evil or Chaotic Evil."),
        Ok(Cond::Alignment(vec![TrueNeutral, NeutralEvil, ChaoticEvil]))
    );
}

#[test]
fn alignments_name_one_or_every_alignment_with_that_part() {
    use Alignment::*;
    assert_eq!(
        parse_line("Good alignment"),
        Ok(Cond::Alignment(vec![LawfulGood, NeutralGood, ChaoticGood]))
    );
    assert_eq!(
        parse_line("Alignment must be Lawful Good"),
        Ok(Cond::Alignment(vec![LawfulGood]))
    );
    assert_eq!(
        parse_line("Alignment must be Lawful"),
        Ok(Cond::Alignment(vec![LawfulGood, LawfulNeutral, LawfulEvil]))
    );
    // "Neutral" on its own is neutral on either axis, as in "any neutral".
    assert_eq!(
        parse_line("Alignment must be neutral or evil"),
        Ok(Cond::Alignment(vec![
            NeutralGood,
            LawfulNeutral,
            TrueNeutral,
            ChaoticNeutral,
            LawfulEvil,
            NeutralEvil,
            ChaoticEvil
        ]))
    );
}
