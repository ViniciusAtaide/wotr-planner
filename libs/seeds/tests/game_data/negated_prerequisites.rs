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
    for name in ["extended-range", "kinetic-blade"] {
        assert_eq!(
            infusion_reason(&sharpshooter, 3, name),
            "Not for the Kinetic Sharpshooter archetype",
            "{name}"
        );
    }
    assert_eq!(
        infusion_reason(&sharpshooter, 3, "eruption"),
        "Not for the Kinetic Sharpshooter archetype; needs Kineticist level 6 (have 3), Extended Range"
    );
    assert_eq!(
        infusion_reason(&kineticist("fire", 3), 3, "exploding-arrows"),
        "Needs the Kinetic Sharpshooter archetype"
    );
}

/// Every negated prerequisite in the data reads as what rules the option out, never "Needs not …": the four kinds
/// the data negates (an archetype, an option held, a class level, the slot picked through), each in a real reason.
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
            other => panic!("a new negated kind: {other:?} reads {text:?}"),
        };
        assert!(text.starts_with(lead), "{child:?} reads {text:?}");
        kinds.insert(kind);
    }
    assert_eq!(
        kinds,
        BTreeSet::from(["archetype", "class level", "option", "picked through"])
    );

    // A slayer's 2nd-level combat style talent at slayer 6.
    assert_eq!(
        lock_reason(
            &build("slayer", 6),
            6,
            "slayer/slayer-talents/combat-style-feat-2"
        ),
        "Only below Slayer level 6"
    );
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
    assert_eq!(
        lock_reason(&totems, 6, "barbarian/rage-power/beast-totem"),
        "Not with Celestial Totem or Celestial Totem, Lesser; needs Beast Totem, Lesser"
    );
    // Cinder Dance with the Lame curse.
    let lame = with_option(oracle("flame", 1), 1, 1, "oracle/oracles-curse/lame");
    let lame = with_option(lame, 1, 2, "oracle/revelations/flame/cinder-dance");
    assert_eq!(
        why_not(&lame, &SlotId::Selection { level: 1, index: 2 })
            .and_then(|issue| engine.issue_text(&issue))
            .as_deref(),
        Some("Not with Lame")
    );
    // A Lawful Good Champion of the Faith's evil blessing as his chosen alignment, and a Lawful Good Separatist's
    // Chaos Domain as his own: the alignments that case allows.
    let mut champion = archetype_build("warpriest", "warpriest/champion-of-the-faith", 1);
    champion.start.alignment = Some(rules::Alignment::LawfulGood);
    let chosen = slot_of(
        &champion,
        1,
        "warpriest/champion-of-the-faith/chosen-alignment",
    );
    champion
        .picks
        .insert(chosen.clone(), pick("warpriest/blessings/evil-domain"));
    assert_eq!(
        why_not(&champion, &chosen)
            .and_then(|issue| engine.issue_text(&issue))
            .as_deref(),
        Some(
            "As Chosen Alignment Blessing, needs Lawful Evil alignment or Neutral Evil alignment or Chaotic Evil \
             alignment"
        )
    );
    let mut separatist = separatist(3);
    separatist.start.alignment = Some(rules::Alignment::LawfulGood);
    let own = slot_of(&separatist, 1, "cleric/separatist/domain");
    separatist
        .picks
        .insert(own.clone(), pick("cleric/domain/chaos"));
    assert_eq!(
        why_not(&separatist, &own)
            .and_then(|issue| engine.issue_text(&issue))
            .as_deref(),
        Some(
            "As Domain (Separatist), needs Neutral Good alignment or Chaotic Good alignment or True Neutral \
             alignment or Chaotic Neutral alignment or Neutral Evil alignment or Chaotic Evil alignment"
        )
    );
}
