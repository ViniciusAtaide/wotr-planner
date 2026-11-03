//! Ticket 63's archetype alignment rules: the alignments a prerequisite's alignment rule allows.

use std::collections::BTreeSet;

use extract::archetype_alignments::alignment_rule;
use rules::{Alignment, Prerequisite, RaceKey};

use Alignment::*;

fn any_of(alignments: &[Alignment]) -> Prerequisite {
    Prerequisite::Any(
        alignments
            .iter()
            .copied()
            .map(Prerequisite::Alignment)
            .collect(),
    )
}

#[test]
fn an_alignment_rule_is_the_alignments_its_own_conditions_allow() {
    let good = [LawfulGood, NeutralGood, ChaoticGood];
    // The whole prerequisite, or a top-level `All`'s alignment conditions beside others (the Reformed Fiend's race).
    assert_eq!(alignment_rule(&any_of(&good)), Some(BTreeSet::from(good)));
    let fiend = Prerequisite::All(vec![
        any_of(&good),
        Prerequisite::Race(RaceKey::new("tiefling")),
    ]);
    assert_eq!(alignment_rule(&fiend), Some(BTreeSet::from(good)));
    // "Cannot be Lawful" written as a `Not`.
    let not_lawful = Prerequisite::Not(Box::new(any_of(&[LawfulGood, LawfulNeutral, LawfulEvil])));
    assert_eq!(
        alignment_rule(&not_lawful),
        Some(BTreeSet::from([
            NeutralGood,
            ChaoticGood,
            TrueNeutral,
            ChaoticNeutral,
            NeutralEvil,
            ChaoticEvil
        ]))
    );
    // No alignment condition of its own: none.
    assert_eq!(alignment_rule(&Prerequisite::BaseAttackBonus(1)), None);
    let mixed = Prerequisite::Any(vec![
        Prerequisite::Alignment(LawfulGood),
        Prerequisite::BaseAttackBonus(1),
    ]);
    assert_eq!(alignment_rule(&mixed), None);
}
