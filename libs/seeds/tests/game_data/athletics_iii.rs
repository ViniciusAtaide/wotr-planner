//! Athletics III (ticket 212): the base attack bonus is at least the Athletics ranks spent, or those plus 5 while
//! the class tables' bonus equals the character level (`trickster#athletics-iii`). The sheet, the attack line and
//! every prerequisite read the raised bonus, from the level the Mythic rank is taken at.

use rules::{Prerequisite, SkillKey, WeaponKey};

use crate::support::*;

/// The guide-only Human Two-Handed Fighter 20 of the third martial run (the map's History, 2026-11-14): Athletics
/// III at MR 7 (level 16) with 16 Athletics ranks by then and 20 at level 20.
const GUIDE_ONLY_TRICKSTER: &str = "AeXQO0hCYRQH8HO-aw-upb1QehAkQUMNZpDLLcu2Lg3RfsHApcFeRJAFZVgIFyOKllsODRWCDUJIW1BIBUVECg0FQhZiQYM4SK_vFhmJOLm5_M_v_PmGjyPijHfAF5p3qC3mo62rt_hOe6xA7LcWjRFNWQev3HSfuXzGx86FmEHrWXoJW2jj6FMknBOBuuhgr0DXyOppremW4ubi-j5lpfA-TNLkLofiQY6i8l0475miWOeNBskvNblEt23jaWR6e7T7eJn2FSfNrKBPaV_vxFJOSvQ_6-26YHXreLTN1qVM8of7puEPc2PDXORTfeAMJWdbwvZA_ZpncW9lVwcIWIxYVa5iWbYGGET8C_y_5qejphtoEAAIyEkHAMrCXxHM1WWIpN9lUwl-k7BIkAE5SFo_oVAx8hnkvyACZjjX_AI";

fn athletics() -> SkillKey {
    SkillKey::new("athletics")
}

#[test]
fn the_guide_only_trickster_fighter_reads_bab_25_at_level_20_and_21_at_level_16() {
    let engine = engine();
    let build = engine
        .open_link(GUIDE_ONLY_TRICKSTER)
        .expect("the link opens");
    assert!(engine.is_possible(&build));
    let sheet = engine.evaluate(&build);
    assert_eq!(sheet.ranks[6].character_level, 16);

    let state = |level| sheet.at(level).unwrap();
    // Game: TricksterAthleticsTier3Feature e45bf795c4f84c3b8a83c011f8580491 (BAB at least the Athletics ranks, plus 5
    // while the character level equals the BAB's base value)
    assert_eq!(state(16).skill_ranks(&athletics()), 16);
    assert_eq!(state(20).skill_ranks(&athletics()), 20);
    assert_eq!(
        [state(15).bab, state(16).bab, state(17).bab, state(20).bab],
        [15, 21, 22, 25]
    );

    let falchion = WeaponKey::new("falchion");
    let line = |level| engine.attack_line(state(level), &falchion).unwrap();
    // The first attack is the BAB plus the Build's 9 (Strength, Weapon Focus and its Greater form).
    // Game: RuleCalculateAttacksCount.CalculatePenalizedAttacksCount (the cap of 3 is only for a unit without
    // UnitPartCompanion, which the main character has: Player.SetMainCharacter); UnitAttack (-5 a step), WrathDecompiled
    // 6ce4a6b
    assert_eq!(line(16).attacks, [30, 25, 20, 15, 10]);
    assert_eq!(line(20).attacks, [34, 29, 24, 19, 14]);
    // Game: PowerAttackWatcher (BAB / 4 + 1), WrathDecompiled 6ce4a6b
    // Power Attack: 1 + BAB / 4 steps, six steps at BAB 20 and a seventh at BAB 25.
    assert_eq!(line(16).power_attack.unwrap().attack_penalty, 6);
    assert_eq!(line(20).power_attack.unwrap().attack_penalty, 7);
    assert_eq!(line(15).attacks.len(), 3);
    // Without the trick the same level 20 is BAB 20: four attacks and six steps.
    let mut plain = build.clone();
    plain.picks.remove(&path_selection(7));
    let plain = engine.evaluate(&plain);
    let line = engine
        .attack_line(plain.at(20).unwrap(), &falchion)
        .unwrap();
    assert_eq!(line.attacks.len(), 4);
    assert_eq!(line.power_attack.unwrap().attack_penalty, 6);
}

#[test]
fn athletics_iii_counts_the_ranks_spent_not_the_skills_total() {
    let engine = engine();
    let mut build = engine.open_link(GUIDE_ONLY_TRICKSTER).unwrap();
    // Game: StatGetter.GetBaseValue and ModifiableValueSkill.CalculateBaseValue, WrathDecompiled 6ce4a6b (the base
    // value read is the ranks; the class-skill bonus and the ability are added after)
    // One rank less at level 20: the BAB follows the ranks (19 + 5), not the skill's total or the level.
    let level = &mut build.levels[19];
    assert_eq!(level.skill_ranks.get(&athletics()), Some(&1));
    level.skill_ranks.remove(&athletics());
    assert!(engine.is_possible(&build));
    let sheet = engine.evaluate(&build);
    assert_eq!(sheet.at(19).unwrap().skill_ranks(&athletics()), 19);
    assert_eq!(sheet.at(20).unwrap().skill_ranks(&athletics()), 19);
    assert_eq!(sheet.at(19).unwrap().bab, 24);
    assert_eq!(sheet.at(20).unwrap().bab, 24);
}

/// A wizard 20 on the Trickster path, a rank in Athletics at every level, Athletics I at MR 3, II at MR 4 and III
/// at MR 7 when `athletics_iii`.
fn trickster_wizard(athletics_iii: bool) -> rules::Build {
    let mut wizard = build("wizard", 20);
    wizard.mythic_path = Some(rules::MythicPathKey::new("trickster"));
    let mut build = wizard;
    for level in &mut build.levels {
        level.skill_ranks.insert(athletics(), 1);
    }
    build.picks.extend(build_picks(athletics_iii));
    faithful(build)
}

fn build_picks(athletics_iii: bool) -> Vec<(rules::SlotId, rules::Pick)> {
    let mut picks = vec![
        (
            path_selection(3),
            pick("trickster/mythic-trick/athletics-i"),
        ),
        (
            path_selection(4),
            pick("trickster/improved-mythic-trick/athletics-ii"),
        ),
    ];
    if athletics_iii {
        picks.push((
            path_selection(7),
            pick("trickster/greater-mythic-trick/athletics-iii"),
        ));
    }
    picks
}

#[test]
fn a_wizard_with_athletics_iii_opens_critical_focus_from_the_next_level_it_couldnt_before() {
    let engine = engine();
    let with = trickster_wizard(true);
    let without = trickster_wizard(false);
    assert!(engine.is_possible(&with));
    assert!(engine.is_possible(&without));
    let sheet = engine.evaluate(&with);
    let rank_7 = sheet.ranks[6].character_level;
    // The wizard's table is half the level; the ranks spent are the level.
    // Game: WizardClass ba34257984f4c41408ce1dc2004e342e (BABLow)
    assert_eq!(at(&with, rank_7).bab, rank_7);
    assert_eq!(at(&without, rank_7).bab, rank_7 / 2);

    let status = |build: &rules::Build, level| {
        engine
            .options(build, &general_slot(level))
            .unwrap()
            .into_iter()
            .find(|option| option.pick.key() == "critical-focus")
            .map(|option| option.status)
    };
    // Needs BAB 9: the next odd level's slot sees the raise.
    // Game: CriticalFocus 8ac59959b1b23c347a0361dc97cc786d (PrerequisiteStatValue BaseAttackBonus 9)
    let next = (rank_7 + 1..=20).find(|level| level % 2 == 1).unwrap();
    assert_eq!(status(&with, next), Some(Ok(())));
    assert!(status(&without, next).unwrap().is_err());
    assert!(
        at(&without, next)
            .check(&Prerequisite::BaseAttackBonus(9))
            .is_err()
    );
}

#[test]
fn the_level_athletics_iii_is_taken_at_doesnt_see_it_in_its_own_feat_slot() {
    let engine = engine();
    // MR 7 at level 17, which has a general slot: the slot is filled before the rank's picks.
    let mut build = trickster_wizard(true);
    build.rank_levels.insert(7, 17);
    assert!(engine.is_possible(&build));
    assert_eq!(engine.evaluate(&build).ranks[6].character_level, 17);
    let status = |level| {
        engine
            .options(&build, &general_slot(level))
            .unwrap()
            .into_iter()
            .find(|option| option.pick.key() == "critical-focus")
            .unwrap()
            .status
    };
    // The wizard's table gives 8 at 17 and 9 at 19; the ranks spent are the level. The slot's order before the
    // rank's picks is the planner's model, not a game fact.
    // Game: WizardClass ba34257984f4c41408ce1dc2004e342e (BABLow); CriticalFocus 8ac59959b1b23c347a0361dc97cc786d
    assert!(status(17).is_err());
    assert_eq!(engine.evaluate(&build).at(17).unwrap().bab, 17);
    assert_eq!(status(19), Ok(()));
}

#[test]
fn athletics_iii_is_the_only_option_that_sets_the_bab_by_ranks() {
    let raising: Vec<_> = DATA
        .mythic_paths
        .iter()
        .flat_map(|path| &path.selections)
        .flat_map(|selection| &selection.options)
        .filter_map(|option| Some((option.key.as_str(), option.bab_from_ranks.as_ref()?)))
        .collect();
    // Game: TricksterAthleticsTier3Feature e45bf795c4f84c3b8a83c011f8580491, the only blueprint with a BAB bonus read
    // from a skill
    let [(key, rule)] = raising[..] else {
        panic!("{raising:?}");
    };
    assert_eq!(key, "trickster/greater-mythic-trick/athletics-iii");
    assert_eq!(
        (rule.skill.as_str(), rule.extra_at_full_bab),
        ("athletics", 5)
    );
    // No class, archetype or prestige class sets one.
    let selections = DATA
        .classes
        .iter()
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .chain(
            DATA.prestige_classes
                .iter()
                .flat_map(|class| &class.selections),
        );
    assert!(
        selections
            .flat_map(|selection| &selection.options)
            .all(|option| option.bab_from_ranks.is_none())
    );
}
