//! Available options for a slot: filtered by prerequisites at that point of the Build, with reasons.

use std::collections::BTreeSet;

use rules::{Ability, Build, Issue, Pick, Prerequisite, SlotId, SlotOption, Unmet};

use crate::support::*;

fn options(build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("the Build has no slot {slot:?}"))
}

fn status_of(options: &[SlotOption], pick: &Pick) -> Result<(), Issue> {
    options
        .iter()
        .find(|option| &option.pick == pick)
        .unwrap_or_else(|| panic!("{pick:?} isn't offered"))
        .status
        .clone()
}

fn needs(prerequisite: Prerequisite, has: Option<u8>) -> Unmet {
    Unmet::Condition {
        needs: prerequisite,
        has,
    }
}

fn needs_feat(key: &str) -> Unmet {
    needs(Prerequisite::Feat(feat(key)), None)
}

#[test]
fn fighter_bonus_slots_offer_only_combat_feats() {
    let build = build("elf", &["fighter"]);
    let offered: BTreeSet<_> = options(&build, &class_slot(1, 0))
        .into_iter()
        .map(|option| option.pick.key().to_owned())
        .collect();
    let expected: BTreeSet<_> = [
        "power-attack",
        "cleave",
        "great-cleave",
        "cleaving-finish",
        "combat-expertise",
        "crane-style",
        "crane-wing",
        "dodge",
        "toughness",
        "weapon-focus",
        "weapon-specialization",
        "aldori-dueling-mastery",
        "exotic-weapon-proficiency",
        "dirty-trick",
        "critical-mastery",
        "deft-maneuvers",
        "improved-unarmed-strike",
        "shield-focus",
        "shield-proficiency",
        "tower-shield-proficiency",
        "stat-focus",
        "dazzling-display",
        "armor-proficiency-light",
        "armor-proficiency-heavy",
        "armor-focus",
        "boon-companion",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(offered, expected);

    // The general slot at the same level offers everything, Magic and Skill feats included (a parameterised feat
    // once per value).
    let general: BTreeSet<_> = options(&build, &general_slot(1))
        .into_iter()
        .map(|option| option.pick.key().to_owned())
        .collect();
    assert_eq!(general.len(), engine().data().feats.len());

    // A non-combat feat in the bonus slot doesn't qualify: the picker doesn't offer it.
    let mut build = build;
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("skill-focus"));
    assert_eq!(why_not(&build, &class_slot(1, 0)), None);
}

#[test]
fn an_and_prerequisite_reports_each_failing_condition() {
    // Elf fighter 1: BAB +1, no Power Attack.
    let build = build("elf", &["fighter"]);
    let offered = options(&build, &general_slot(1));

    assert_eq!(status_of(&offered, &pick_feat("power-attack")), Ok(()));
    // Cleave: Power Attack and BAB +1; only Power Attack is missing, so it's reported on its own.
    assert_eq!(
        status_of(&offered, &pick_feat("cleave")),
        Err(Issue::Unmet(needs_feat("power-attack")))
    );
    // Great Cleave: Cleave and BAB +4 both fail.
    assert_eq!(
        status_of(&offered, &pick_feat("great-cleave")),
        Err(Issue::Unmet(Unmet::All(vec![
            needs_feat("cleave"),
            needs(Prerequisite::BaseAttackBonus(4), Some(1)),
        ])))
    );
}

#[test]
fn an_or_prerequisite_holds_when_any_branch_does() {
    // Dirty Trick: Combat Expertise, or Sneak Attack and Int 13. An elf has Int 15.
    let fighter = build("elf", &["fighter"]);
    assert_eq!(
        status_of(
            &options(&fighter, &general_slot(1)),
            &pick_feat("dirty-trick")
        ),
        Err(Issue::Unmet(Unmet::Any(vec![
            // The Sneak Attack branch fails only on Sneak Attack (Int is fine).
            needs(
                Prerequisite::Feature {
                    feature: feature("rogue/sneak-attack"),
                    count: 1,
                },
                Some(0),
            ),
            needs_feat("combat-expertise"),
        ])))
    );

    let rogue = build("elf", &["rogue"]);
    assert_eq!(
        status_of(
            &options(&rogue, &general_slot(1)),
            &pick_feat("dirty-trick")
        ),
        Ok(())
    );
}

#[test]
fn an_at_least_prerequisite_counts_the_branches_that_hold() {
    // Critical Mastery: two of (Power Attack, Dirty Trick, Cleave or Rogue 3), and Fighter 2.
    let mut build = build("elf", &times("fighter", 3));
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    let offered = options(&build, &general_slot(3));
    let Err(Issue::Unmet(Unmet::AtLeast {
        needed,
        met,
        failed,
    })) = status_of(&offered, &pick_feat("critical-mastery"))
    else {
        panic!("expected an at-least failure");
    };
    assert_eq!((needed, met, failed.len()), (2, 1, 2));

    // With Cleave as well, two branches hold.
    build.picks.insert(class_slot(2, 0), pick_feat("cleave"));
    assert_eq!(
        status_of(
            &options(&build, &general_slot(3)),
            &pick_feat("critical-mastery")
        ),
        Ok(())
    );
}

#[test]
fn options_see_earlier_picks_of_the_same_level_but_not_later_ones() {
    // Human fighter 1: the fighter slot, then the race slot, then the general slot.
    let mut build = build("human", &["fighter"]);
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    assert_eq!(
        status_of(&options(&build, &race_slot(0)), &pick_feat("cleave")),
        Ok(())
    );
    // Power Attack is already held from the fighter slot.
    assert_eq!(
        status_of(
            &options(&build, &general_slot(1)),
            &pick_feat("power-attack")
        ),
        Err(Issue::AlreadyHeld)
    );
    // The fighter slot comes first, so it doesn't see a later pick of Power Attack.
    let mut later = build.clone();
    later.picks.remove(&class_slot(1, 0));
    later.picks.insert(race_slot(0), pick_feat("power-attack"));
    assert_eq!(
        status_of(&options(&later, &class_slot(1, 0)), &pick_feat("cleave")),
        Err(Issue::Unmet(needs_feat("power-attack")))
    );
}

#[test]
fn feature_selection_options_come_from_the_selection_list() {
    let build = build("elf", &times("rogue", 2));
    let offered = options(&build, &selection_slot(2, 0));
    let keys: Vec<_> = offered.iter().map(|o| o.pick.key()).collect();
    // An option with a chosen feat comes once per value (Weapon Focus: the five weapons).
    assert_eq!(
        keys,
        [
            "rogue/rogue-talents/combat-trick",
            "rogue/rogue-talents/crippling-strike",
            "rogue/rogue-talents/fast-stealth",
            "rogue/rogue-talents/resiliency",
            "rogue/rogue-talents/weapon-focus",
            "rogue/rogue-talents/weapon-focus",
            "rogue/rogue-talents/weapon-focus",
            "rogue/rogue-talents/weapon-focus",
            "rogue/rogue-talents/weapon-focus",
        ]
    );
    assert_eq!(
        status_of(
            &offered,
            &pick_option("rogue/rogue-talents/crippling-strike")
        ),
        Err(Issue::Unmet(Unmet::All(vec![
            needs(
                Prerequisite::ClassLevel {
                    class: class("rogue"),
                    level: 10
                },
                Some(2)
            ),
            needs(
                Prerequisite::SelectionOption(option("rogue/rogue-talents/combat-trick")),
                None
            ),
        ])))
    );
}

#[test]
fn an_option_that_grants_a_feat_makes_the_feat_held() {
    let mut build = build("elf", &times("rogue", 3));
    build.picks.insert(
        selection_slot(2, 0),
        pick_option("rogue/rogue-talents/combat-trick"),
    );
    let sheet = engine().evaluate(&build);
    assert!(
        sheet
            .at(2)
            .unwrap()
            .feats
            .contains_key(&feat("combat-expertise"))
    );
    assert_eq!(
        status_of(
            &options(&build, &general_slot(3)),
            &pick_feat("combat-expertise")
        ),
        Err(Issue::AlreadyHeld)
    );
}

#[test]
fn ability_scores_skill_ranks_spell_levels_and_caster_level_are_checked() {
    let mut build = build("dwarf", &times("wizard", 5));
    build.start.ability_scores = build
        .start
        .ability_scores
        .with(rules::Ability::Dexterity, 12)
        .unwrap();
    build.levels[0].skill_ranks.insert(skill("mobility"), 1);
    let offered = options(&build, &general_slot(1));
    assert_eq!(
        status_of(&offered, &pick_feat("dodge")),
        Err(Issue::Unmet(needs(
            Prerequisite::AbilityScore {
                ability: Ability::Dexterity,
                score: 13
            },
            Some(12)
        )))
    );
    assert_eq!(
        status_of(&offered, &pick_feat("deft-maneuvers")),
        Err(Issue::Unmet(needs(
            Prerequisite::SkillRanks {
                skill: skill("mobility"),
                ranks: 3
            },
            Some(1)
        )))
    );
    assert_eq!(status_of(&offered, &pick_feat("arcane-strike")), Ok(()));
    assert_eq!(
        status_of(&offered, &pick_feat("spell-penetration")),
        Err(Issue::Unmet(needs(Prerequisite::CasterLevel(5), Some(1))))
    );
    assert_eq!(
        status_of(
            &options(&build, &general_slot(5)),
            &pick_feat("spell-penetration")
        ),
        Ok(())
    );
}

#[test]
fn race_and_alignment_are_checked() {
    let human = build("human", &times("fighter", 4));
    let offered = options(&human, &general_slot(3));
    assert_eq!(
        status_of(&offered, &pick_feat("breadth-of-experience")),
        Err(Issue::Unmet(Unmet::Any(vec![
            needs(Prerequisite::Race(rules::RaceKey::new("dwarf")), None),
            needs(Prerequisite::Race(rules::RaceKey::new("elf")), None),
        ])))
    );
    // Outflank: BAB +4 and lawful good; this Build is true neutral with BAB +3.
    assert_eq!(
        status_of(&offered, &pick_feat("outflank")),
        Err(Issue::Unmet(Unmet::All(vec![
            needs(Prerequisite::BaseAttackBonus(4), Some(3)),
            needs(Prerequisite::Alignment(rules::Alignment::LawfulGood), None),
        ])))
    );
}

#[test]
fn mythic_slots_offer_mythic_abilities_or_feats_by_rank() {
    let build = build("elf", &times("wizard", 7));
    let abilities = options(&build, &SlotId::Mythic { rank: 1 });
    assert!(
        abilities
            .iter()
            .all(|o| matches!(o.pick, Pick::MythicAbility(_)))
    );
    assert_eq!(
        status_of(&abilities, &pick_mythic_ability("limitless-smite")),
        Err(Issue::Unmet(needs(
            Prerequisite::ClassLevel {
                class: class("fighter"),
                level: 1
            },
            Some(0)
        )))
    );
    let feats = options(&build, &SlotId::Mythic { rank: 2 });
    assert!(
        feats
            .iter()
            .all(|o| matches!(o.pick, Pick::MythicFeat { .. }))
    );
}

#[test]
fn a_slot_the_build_does_not_have_has_no_options() {
    let build = build("elf", &times("wizard", 2));
    assert_eq!(engine().options(&build, &general_slot(2)), None);
    assert_eq!(engine().options(&build, &general_slot(3)), None);
}
