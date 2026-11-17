//! Reasons for negated prerequisites (ticket 65).

use std::collections::BTreeSet;

use rules::{Issue, Prerequisite, SlotId, Unmet};

use crate::support::*;

/// Ticket 59's decision 5: an infusion the Kinetic Sharpshooter "cannot use" reads "Not for the Kinetic Sharpshooter
/// archetype", alone or ahead of what else it needs; the base class's reason for one of the Sharpshooter's own is
/// unchanged.
#[test]
fn a_kinetic_sharpshooters_closed_infusion_reads_not_for_its_archetype() {
    let sharpshooter = with_option(
        archetype_build("kineticist", SHARPSHOOTER, 3),
        1,
        0,
        "kineticist/element/fire",
    );
    // Game: ExtendedRangeInfusion cb2d9e6355dd33940b2bef49e544b0bf and KineticBladeInfusion
    // 9ff81732daddb174aa8138ad1297c787 have PrerequisiteNoArchetype KineticSharpshooterArchetype
    // e0684d0a6ef44cae9d095f5318618a85 (the reason's wording is the planner's).
    for name in ["extended-range", "kinetic-blade"] {
        assert_eq!(
            infusion_reason(&sharpshooter, 3, name),
            "Not for the Kinetic Sharpshooter archetype",
            "{name}"
        );
    }
    // Game: EruptionInfusion 00f8e4b846c367141afcd133f4a1c816 needs kineticist 6 and Extended Range, which no
    // Sharpshooter holds (equivalent model: no archetype bar of its own).
    assert_eq!(
        infusion_reason(&sharpshooter, 3, "eruption"),
        "Not for the Kinetic Sharpshooter archetype; needs Kineticist level 6 (have 3), Extended Range"
    );
    // Game: ExplodingArrowsInfusion 7c8ded7015114e4086b3dbdd16388b29's PrerequisiteArchetypeLevel names
    // KineticSharpshooterArchetype e0684d0a6ef44cae9d095f5318618a85 at 3.
    assert_eq!(
        infusion_reason(&kineticist("fire", 3), 3, "exploding-arrows"),
        "Needs the Kinetic Sharpshooter archetype"
    );
}

/// Every negated prerequisite in the data reads as what rules the option out, never "Needs not …": the five kinds
/// the data negates (an archetype, an option held, a class level, the slot picked through, a domain the deity
/// offers), each in a real reason.
#[test]
fn every_negated_prerequisite_in_the_data_reads_as_what_rules_the_option_out() {
    fn negations<'a>(prerequisite: &'a Prerequisite, out: &mut Vec<&'a Prerequisite>) {
        match prerequisite {
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                children.iter().for_each(|child| negations(child, out));
            }
            Prerequisite::Not(child) => out.push(child),
            _ => {}
        }
    }
    let engine = engine();
    let mut held = Vec::new();
    for prerequisite in data_prerequisites(&DATA) {
        negations(prerequisite, &mut held);
    }
    let mut kinds = BTreeSet::new();
    for child in held {
        let text = engine
            .issue_text(&Issue::Unmet(Unmet::Condition {
                needs: Prerequisite::Not(Box::new(child.clone())),
                has: None,
            }))
            .unwrap();
        let (kind, lead) = match child {
            Prerequisite::Archetype(_) => ("archetype", "Not for the "),
            Prerequisite::SelectionOption(_) => ("option", "Not with "),
            Prerequisite::ClassLevel { .. } => ("class level", "Only below "),
            Prerequisite::PickedThrough(_) => ("picked through", "Not as "),
            // Ticket 86: the Priest of Balance's deity offers neither Good nor Evil.
            Prerequisite::DeityOffers(_) => ("deity offers", "Not for a deity offering the "),
            // Ticket 303: a selection's own `PrerequisiteNoFeature` (Second Bloodline, Advanced Weapon Training).
            Prerequisite::Feat(_) => ("feat", "Not with "),
            // Ticket 306aa: Evasion's own `PrerequisiteNoFeature` (Evasion 576933720c440aa4d8d42b0c54b77e80), the class
            // feature `evasion` the Extra Rogue Talent's list would give a second time.
            Prerequisite::Feature { .. } => ("feature", "Not with "),
            Prerequisite::MythicAbility(_) => ("mythic ability", "Not with "),
            // Ticket 303: a selection's cap on its own rank (Extra Hex: `Not` of its rank 3).
            Prerequisite::Rank { .. } => ("rank", "Only below "),
            // Ticket 308d: the oracle's PrerequisiteNoFeature ApsuFeature 772e2673945e4583a804ae01f67efea0 (OracleClass
            // 20ce9bf8af32bee4c8557a045ab499b1).
            Prerequisite::Deity(_) => ("deity", "Not for a follower of "),
            // Ticket 308d: a selection's own PrerequisiteNoFeature of a blueprint it shares with another entry
            // (SecondMystery 277b0164740b97945a3f8022bd572f48, SecondBloodragerBloodline
            // b7f62628915bdb14d8888c25da3fac56), each held entry named once.
            Prerequisite::Any(_) => {
                assert!(!text.contains('('), "{child:?} reads {text:?}");
                ("any held", "Not with ")
            }
            other => panic!("a new negated kind: {other:?} reads {text:?}"),
        };
        assert!(text.starts_with(lead), "{child:?} reads {text:?}");
        kinds.insert(kind);
    }
    assert_eq!(
        kinds,
        BTreeSet::from([
            "any held",
            "archetype",
            "class level",
            "deity",
            "deity offers",
            "feat",
            "feature",
            "mythic ability",
            "option",
            "picked through",
            "rank"
        ])
    );

    // A slayer's 2nd-level combat style talent at slayer 6:
    // `a_slayer_6_who_skipped_it_takes_the_2nd_level_combat_style`.
    // A second totem line, ahead of what else it needs: Beast Totem for a barbarian of the Celestial line.
    let mut totems = barbarian(6);
    totems.picks.insert(
        selection_slot(2),
        pick("barbarian/rage-power/celestial-totem-lesser"),
    );
    totems.picks.insert(
        selection_slot(4),
        pick("barbarian/rage-power/celestial-totem"),
    );
    assert!(possible(&totems));
    // Game: BeastTotemFeature c085888db293f2741b881cc989a2ab14 needs BeastTotemLesserFeature
    // d99dfc9238a8a6646b32be09057c1729, which CelestialTotemLesserFeature aba61e0b0e66bf3439cc247ee89fddae shuts
    // out (equivalent model: the game's only reason is the missing Lesser).
    assert_eq!(
        lock_reason(&totems, 6, "barbarian/rage-power/beast-totem"),
        "Not with Celestial Totem or Celestial Totem, Lesser; needs Beast Totem, Lesser"
    );
    // Cinder Dance with the Lame curse.
    let lame = with_option(oracle("flame", 1), 1, 1, "oracle/oracles-curse/lame");
    let lame = with_option(lame, 1, 2, "oracle/revelations/flame/cinder-dance");
    // Game: OracleRevelationCinderDance 6e67eae3081853544b191943f5ed4534's PrerequisiteNoFeature
    // LameCurseProgression 3d319da9614c76a41a095a4467c3233f.
    assert_eq!(
        why_not(&lame, &SlotId::Selection { level: 1, index: 2 })
            .and_then(|issue| engine.issue_text(&issue))
            .as_deref(),
        Some("Not with Lame")
    );
    // A Lawful Good Champion of the Faith's evil blessing as his chosen alignment, and a Lawful Good Separatist's Chaos
    // Domain as his own: both refused. What each needs:
    // `a_champion_of_the_faiths_evil_alignment_needs_only_a_non_good_one` here, and
    // `a_true_neutral_separatist_takes_no_alignment_domain` (option_follow_ups.rs).
    let mut champion = archetype_build("warpriest", "warpriest/champion-of-the-faith", 1);
    align(&mut champion, rules::Alignment::LawfulGood);
    let chosen = slot_of(
        &champion,
        1,
        "warpriest/champion-of-the-faith/chosen-alignment",
    );
    champion
        .picks
        .insert(chosen.clone(), pick("warpriest/blessings/evil-domain"));
    // Game: ChampionOfTheFaithEvilFeature cf6e5f8927653434eab6179cbc76eb04's PrerequisiteAlignment admits no good
    // alignment (Blueprints/Classes/Prerequisites/PrerequisiteAlignment.CheckInternal, WrathDecompiled 6ce4a6b).
    assert!(why_not(&champion, &chosen).is_some());
    // Sarenrae's: Fire is hers, Luck isn't (ticket 86).
    let mut separatist = realigned(separatist(3), rules::Alignment::LawfulGood);
    let own = slot_of(&separatist, 1, "cleric/separatist/domain");
    separatist
        .picks
        .insert(own.clone(), pick("cleric/domain/chaos"));
    // Game: ChaosDomainProgressionSeparatist 714706ca6958440cbbb217c2d167e6ec needs a chaotic alignment.
    assert!(why_not(&separatist, &own).is_some());
}

/// The slayer's 2nd-level combat style talent needs only that it isn't held: a slayer who skipped it at 2 and 4
/// takes it at 6.
#[test]
#[ignore = "waits on stage S4 (the prerequisites writer): the 2nd-level combat style talent's option still carries the old upper slayer level (known.txt, 276)"]
fn a_slayer_6_who_skipped_it_takes_the_2nd_level_combat_style() {
    // Game: RangerStyleSelection2 c6d0da9124735a44f93ac31df803b9a9 needs only that it isn't held, and
    // SlayerTalentSelection6 43d1b15873e926848be2abf0ea3ad9a8 offers it.
    assert_eq!(
        offered(&build("slayer", 6), 6)["slayer/slayer-talents/combat-style-feat-2"],
        Ok(())
    );
}

/// A Champion of the Faith's Evil chosen alignment needs only an alignment that isn't good, and no deity: a lawful
/// neutral one takes it.
#[test]
#[ignore = "waits on ticket 267: the game gives the Evil chosen alignment to any non-good alignment, no deity"]
fn a_champion_of_the_faiths_evil_alignment_needs_only_a_non_good_one() {
    let mut champion = realigned(
        archetype_build("warpriest", "warpriest/champion-of-the-faith", 1),
        rules::Alignment::LawfulNeutral,
    );
    let chosen = slot_of(
        &champion,
        1,
        "warpriest/champion-of-the-faith/chosen-alignment",
    );
    champion
        .picks
        .insert(chosen, pick("warpriest/blessings/evil-domain"));
    // Game: ChampionOfTheFaithEvilFeature cf6e5f8927653434eab6179cbc76eb04's PrerequisiteAlignment admits LN, TN, CN
    // and the evil ones; it names no deity and grants EvilBlessingFeature f38f3abf6ca3a07499a61f96e342bb16.
    assert!(possible(&champion));
}
