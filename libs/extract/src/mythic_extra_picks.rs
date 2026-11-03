//! Ticket 55: ticket 49's decision 4 (user, 2026-11-02). The mythic feats Extra Feat ("You can select any normal
//! feat") and Extra Mythic Ability ("You can select a mythic ability") give one more pick at the Mythic rank they're
//! taken (`MythicFeat::extra_pick`), set where the mythic feats are built ([`provide`]): a feat slot that offers what
//! the general slot does, and a mythic ability slot that offers what a rank's general pick does.
//!
//! - **Which mythic feats:** the mythic page's entries saying "You can select": only these two.
//! - **Once each:** neither entry says it can be taken again, so each stays `Repeat::Once` (what the guide means when it
//!   doesn't say otherwise).
//! - **The mythic abilities that give a pick** already do, through `MythicAbility::selections` (Impossible Domain,
//!   Second Bloodline, Second Bloodrager Bloodline, Second Mystery, Second Spirit), but two whose entry reads like one
//!   give none: Witch Wandering Hex ("You can select any hex") and Beneficial Curse ("You gain an additional Curse").
//!   Ticket 55 left them as they were and asked the user ([`LEFT`]); ticket 62's
//!   [`mythic_ability_picks`](crate::mythic_ability_picks) gives them their pick.
//!
//! Each rule asserts the guide phrase it rests on where it's read, and [`check`] what it finds in the finished data, so
//! it fails loudly on data it doesn't expect.

use std::collections::BTreeSet;

use rules::{GameData, MythicExtraPick, Repeat};

use crate::follow_ups::says;
use crate::option_levels::{page, section};
use crate::picks::Picks;

/// The mythic page's phrase for a mythic feat that gives a pick.
const GIVES_A_PICK: &str = "You can select";

/// Each mythic feat that gives a pick: its anchor, the sentence giving it, and the slot it gives.
const EXTRA_PICKS: [(&str, &str, MythicExtraPick); 2] = [
    (
        "extra-feat",
        "You can select any normal feat.",
        MythicExtraPick::Feat,
    ),
    (
        "extra-mythic-ability",
        "You can select a mythic ability.",
        MythicExtraPick::MythicAbility,
    ),
];

/// The phrases of a mythic ability entry that reads like one more pick.
const ABILITY_PICK_PHRASES: [&str; 4] = [
    "You can select",
    "You may select",
    "You gain a second",
    "You gain an additional",
];

/// The mythic abilities whose entry reads like a pick and that give one through their selections.
const GIVING: [&str; 5] = [
    "impossible-domain",
    "second-bloodline",
    "second-bloodrager-bloodline",
    "second-mystery",
    "second-spirit",
];

/// The mythic abilities whose entry reads like a pick but that give none here: ticket 55's questions, built by ticket
/// 62's [`mythic_ability_picks`](crate::mythic_ability_picks).
pub const LEFT: [(&str, &str); 2] = [
    (
        "beneficial-curse",
        "You gain an additional Curse with no penalties.",
    ),
    (
        "witch-wandering-hex",
        "You can select any hex for which you meet the prerequisites.",
    ),
];

/// The anchor of an entry on the mythic page, from its guide link.
fn mythic_anchor(guide: &rules::GuideRef) -> &str {
    guide.0.strip_prefix("mythic-paths#").unwrap_or_default()
}

/// Whether the mythic page's entry at `anchor` says any of `phrases`.
fn entry_says(guide: &crate::guide::Guide, anchor: &str, phrases: &[&str]) -> bool {
    !anchor.is_empty() && {
        let text = section(page(guide, "mythic-paths"), anchor).text();
        phrases.iter().any(|phrase| text.contains(phrase))
    }
}

/// Extra Feat and Extra Mythic Ability give their pick, from the start.
pub fn provide(guide: &crate::guide::Guide, picks: &mut Picks) {
    for (key, sentence, extra_pick) in EXTRA_PICKS {
        says(guide, "mythic-paths", key, sentence);
        let text = section(page(guide, "mythic-paths"), key).text();
        assert!(
            !text.contains("multiple times") && !text.contains("more than once"),
            "{key}: the guide says it repeats"
        );
        picks.mythic_extra_pick(key, extra_pick);
    }
    for (key, sentence) in LEFT {
        says(guide, "mythic-paths", key, sentence);
    }
}

/// Only Extra Feat and Extra Mythic Ability give a pick, each taken once, for nothing, with no prerequisite; the
/// mythic abilities whose entry reads like a pick give one through their selections.
pub fn check(guide: &crate::guide::Guide, data: &GameData) {
    // The mythic feats: only Extra Feat and Extra Mythic Ability give a pick.
    let giving: BTreeSet<&str> = data
        .mythic_feats
        .iter()
        .filter(|feat| entry_says(guide, mythic_anchor(&feat.guide), &[GIVES_A_PICK]))
        .map(|feat| feat.key.as_str())
        .collect();
    let expected: BTreeSet<&str> = EXTRA_PICKS.iter().map(|(key, ..)| *key).collect();
    assert_eq!(
        giving, expected,
        "the mythic feats whose entry gives a pick"
    );
    for feat in &data.mythic_feats {
        let key = feat.key.as_str();
        match EXTRA_PICKS.iter().find(|(k, ..)| *k == key) {
            Some((_, _, extra_pick)) => assert!(
                feat.extra_pick == Some(*extra_pick)
                    && feat.repeat == Repeat::Once
                    && feat.parameter.is_none()
                    && feat.prerequisite.is_none(),
                "{key}: its extra pick, taken once, for nothing, with no prerequisite"
            ),
            None => assert!(feat.extra_pick.is_none(), "{key}: an extra pick"),
        }
    }

    // The mythic abilities: the ones that read like a pick give one through their selections (the two left, ticket
    // 62's).
    let reading: BTreeSet<&str> = data
        .mythic_abilities
        .iter()
        .filter(|ability| entry_says(guide, mythic_anchor(&ability.guide), &ABILITY_PICK_PHRASES))
        .map(|ability| ability.key.as_str())
        .collect();
    let expected: BTreeSet<&str> = GIVING
        .iter()
        .chain(LEFT.iter().map(|(key, _)| key))
        .copied()
        .collect();
    assert_eq!(
        reading, expected,
        "the mythic abilities whose entry reads like a pick"
    );
    for ability in &data.mythic_abilities {
        let key = ability.key.as_str();
        if expected.contains(&key) {
            assert!(!ability.selections.is_empty(), "{key}: no selection");
        }
    }
}
