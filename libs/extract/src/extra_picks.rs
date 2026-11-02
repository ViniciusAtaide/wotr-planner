//! Ticket 49: ticket 45's decision 2 (user, 2026-11-02), applied to finished data. The "Extra …" feats whose guide
//! entry gives one more pick of a class selection ("You gain one additional hex for which you must meet the
//! prerequisites") give one more slot of it each time they're taken (`Feat::extra_pick_of`); the resource ones
//! (Extra Ki, Extra Rage, Extra Channel, …: "Your ki pool increases by 2") give none.
//!
//! - **Which feats:** the feats page's entries saying "gain one additional": Extra Arcana, Extra Arcanist Exploit,
//!   Extra Discovery, Extra Hex, Extra Mercy, Extra Rage Power, Extra Rogue Talent, Extra Slayer Talent and Extra
//!   Wild Talent.
//! - **Which selections:** the ones its "Prerequisites" names, matched by name ("Rogue Talents or Fey Tricks": the
//!   rogue's talents and the Sylvan Trickster's Fey Tricks; "Hex": the witch's and the shaman's; the guide's
//!   "Dicovery" is the alchemist's and the Underground Chemist's Discovery). A name no selection has gives nothing:
//!   "Talented Slayer" is on no saved page, and the Arcane Enforcer's "Arcanist Exploits" is a feature, not picks.
//! - **Extra Mercy gives no pick:** "Mercy" is the paladin's feature; the guide's mercies aren't a selection in the
//!   data (ticket 49's question 1). Ticket 54's [`mercies_and_exploits`](crate::mercies_and_exploits), which runs
//!   after this, makes them one and gives Extra Mercy its pick; the Arcane Enforcer's exploits likewise.
//!
//! Each rule asserts the guide phrase it rests on and what it finds in the data, so it fails loudly on data it
//! doesn't expect. [`apply`] runs on the finished data of an extraction and, through ticket 49's one-off rewrite,
//! on `libs/game-data/`.

use std::collections::BTreeSet;

use rules::{FeatureSelection, FeatureSelectionKey, GameData, Repeat};

use crate::follow_ups::says;
use crate::option_levels::{Log, page, section};

/// The feats page's phrase for a feat that gives a pick.
const GIVES_A_PICK: &str = "gain one additional";

/// Each Extra feat that gives a pick: its anchor on the feats page, what its "Prerequisites" names, the phrase giving
/// the pick, and the selections those names are in the data.
const EXTRA_PICKS: [(&str, &str, &str, &[&str]); 8] = [
    (
        "extra-arcana",
        "Magus Arcana",
        "You gain one additional magus arcana for which you must meet the prerequisites.",
        &["magus/magus-arcana"],
    ),
    (
        "extra-arcanist-exploit",
        "Arcanist Exploits",
        "You gain one additional exploit for which you must meet the prerequisites.",
        &["arcanist/arcanist-exploits"],
    ),
    (
        "extra-discovery",
        "Dicovery",
        "You gain one additional discovery for which you must meet the prerequisites.",
        &["alchemist/discovery", "rogue/underground-chemist/discovery"],
    ),
    (
        "extra-hex",
        "Hex",
        "You gain one additional hex for which you must meet the prerequisites.",
        &["shaman/hex", "witch/hex"],
    ),
    (
        "extra-rage-power",
        "Rage Power",
        "gain one additional rage power.",
        &["barbarian/rage-power"],
    ),
    (
        "extra-rogue-talent",
        "Rogue Talents or Fey Tricks",
        "You gain one additional rogue talent for which you must meet the prerequisites.",
        &["rogue/rogue-talents", "rogue/sylvan-trickster/fey-tricks"],
    ),
    (
        "extra-slayer-talent",
        "Slayer Talents or Talented Slayer",
        "You gain one additional slayer talent for which you must meet the prerequisites.",
        &["slayer/slayer-talents"],
    ),
    (
        "extra-wild-talent",
        "Wild Talents",
        "You gain one additional wild talent for which you must meet the prerequisites.",
        &["kineticist/wild-talents"],
    ),
];

/// The Extra feat that gives a pick of something that isn't a selection.
const EXTRA_MERCY: &str = "extra-mercy";

/// Applies ticket 49's rule.
pub fn apply(guide: &crate::guide::Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    let giving: BTreeSet<String> = data
        .feats
        .iter()
        .filter(|feat| {
            let anchor = feat.guide.0.strip_prefix("feats#").unwrap_or_default();
            !anchor.is_empty()
                && section(page(guide, "feats"), anchor)
                    .text()
                    .contains(GIVES_A_PICK)
        })
        .map(|feat| feat.key.to_string())
        .collect();
    let expected: BTreeSet<String> = EXTRA_PICKS
        .iter()
        .map(|(feat, ..)| feat.to_string())
        .chain([EXTRA_MERCY.to_owned()])
        .collect();
    assert_eq!(giving, expected, "the feats whose entry gives a pick");

    for (feat, prerequisite, phrase, selections) in EXTRA_PICKS {
        says(
            guide,
            "feats",
            feat,
            &format!("Prerequisites: {prerequisite}"),
        );
        says(guide, "feats", feat, phrase);
        let named: BTreeSet<FeatureSelectionKey> = prerequisite
            .split(" or ")
            .flat_map(|name| named_selections(data, &name.replace("Dicovery", "Discovery")))
            .collect();
        let expected: BTreeSet<FeatureSelectionKey> = selections
            .iter()
            .map(|key| FeatureSelectionKey::new(*key))
            .collect();
        assert_eq!(
            named, expected,
            "{feat}: the selections {prerequisite:?} names"
        );
        let feat_data = data
            .feats
            .iter_mut()
            .find(|f| f.key.as_str() == feat)
            .unwrap_or_else(|| panic!("no feat {feat}"));
        assert!(
            feat_data.extra_pick_of.is_empty()
                && matches!(feat_data.repeat, Repeat::Times(3) | Repeat::Unlimited),
            "{feat}: no extra pick yet, taken more than once"
        );
        feat_data.extra_pick_of = named;
        log.push(format!(
            "{feat}: one more pick of {}",
            selections.join(" or ")
        ));
    }

    says(guide, "feats", EXTRA_MERCY, "Prerequisites: Mercy");
    says(
        guide,
        "feats",
        EXTRA_MERCY,
        "You gain one additional mercy for which you must meet the prerequisites.",
    );
    assert!(
        named_selections(data, "Mercy").is_empty(),
        "{EXTRA_MERCY}: a selection named Mercy"
    );
    log
}

/// The selections of every class, archetype and path named `name`.
fn named_selections(data: &GameData, name: &str) -> Vec<FeatureSelectionKey> {
    selections(data)
        .filter(|selection| selection.name == name)
        .map(|selection| selection.key.clone())
        .collect()
}

/// Every selection of every class, archetype and path.
fn selections(data: &GameData) -> impl Iterator<Item = &FeatureSelection> {
    data.classes
        .iter()
        .chain(&data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .chain(data.mythic_paths.iter().flat_map(|path| &path.selections))
}
