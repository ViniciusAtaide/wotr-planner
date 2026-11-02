//! Ticket 43: the extraction report item the saved pages settle.

use extract::report_gaps::settled;

#[test]
fn only_the_totems_prose_rule_is_settled() {
    assert!(settled("barbarian#rage-powers", ""));
    assert!(!settled("barbarian#rage-powers", "Beast Totem, Lesser"));
    for (at, text) in [
        ("kineticist#kineticist-bonus-feat", ""),
        ("slayer#advanced-talents", ""),
        (
            "feats#extra-slayer-talent",
            "Slayer Talents or Talented Slayer",
        ),
        (
            "cleric",
            "Alignment must be within 1 place of their chosen deity.",
        ),
        ("sorcerer#empyreal-sorcerer", "Bonus Feat"),
    ] {
        assert!(!settled(at, text), "{at}");
    }
}
