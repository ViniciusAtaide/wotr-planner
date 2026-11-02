//! Tickets 43 and 44: the extraction report items the saved pages or the user's decisions settle.

use extract::report_gaps::settled;

const SORCERER_LEVEL_1_FEATS: [&str; 6] = [
    "sorcerer#crossblooded",
    "sorcerer#empyreal-sorcerer",
    "sorcerer#geomancer",
    "sorcerer#nine-tailed-heir",
    "sorcerer#sage-sorcerer",
    "sorcerer#sylvan-sorcerer",
];

#[test]
fn the_totems_the_sorcerers_level_1_feat_talented_slayer_and_the_deity_are_settled() {
    assert!(settled("barbarian#rage-powers", ""));
    assert!(!settled("barbarian#rage-powers", "Beast Totem, Lesser"));
    for at in SORCERER_LEVEL_1_FEATS {
        assert!(settled(at, "Bonus Feat"), "{at}");
        assert!(!settled(at, "Bloodline Feat"), "{at}");
    }
    assert!(settled(
        "feats#extra-slayer-talent",
        "Slayer Talents or Talented Slayer"
    ));
    assert!(settled(
        "cleric",
        "Alignment must be within 1 place of their chosen deity."
    ));
    for (at, text) in [
        ("kineticist#kineticist-bonus-feat", ""),
        ("slayer#advanced-talents", ""),
        (
            "feats#extra-channel",
            "Channel (Cleric) or Channel Positive Energy (Empyreal Sorcerer / Hospitaler)",
        ),
        ("sorcerer#overwhelming-mage", "Bonus Feat"),
        ("ranger#nomad", "Archery Feat"),
        ("cavalier#tactician-bonus-feat", "Tactician - Bonus Feat"),
    ] {
        assert!(!settled(at, text), "{at}");
    }
}
