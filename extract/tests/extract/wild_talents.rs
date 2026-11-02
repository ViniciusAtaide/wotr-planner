//! Ticket 45: the kineticist's wild talents as a selection settle the bonus-feat wild talent's report item.

use extract::wild_talents::settled;

#[test]
fn only_the_bonus_feat_wild_talents_item_is_settled() {
    assert!(settled("kineticist#kineticist-bonus-feat", ""));
    for (at, text) in [
        ("kineticist#wild-talents", ""),
        ("kineticist#kineticist-bonus-feat", "Bonus Feat"),
        ("slayer#advanced-talents", ""),
        ("sorcerer#crossblooded", "Bonus Feat"),
    ] {
        assert!(!settled(at, text), "{at}");
    }
}
