//! Ticket 125, in the real data: Favorite Metamagic takes its metamagic, Stat Focus its stat and Ascendant Element
//! its element, and a link made before they took one still opens with the value left open. The audit of every item
//! whose summary says it chooses something is pinned: each such item has its pick, or is one the ticket checked.

use std::collections::BTreeSet;

use rules::{
    Element, MythicAbilityKey, Parameter, ParameterKind, Pick, SlotId, Stat, encode_build,
};

use crate::support::*;

const FAVORITE_METAMAGIC: &str = "favorite-metamagic";

fn mythic_ability(key: &str) -> &'static rules::MythicAbility {
    DATA.mythic_abilities
        .iter()
        .find(|ability| ability.key.as_str() == key)
        .unwrap_or_else(|| panic!("no mythic ability {key}"))
}

fn favorite(value: Option<&str>) -> Pick {
    let key = MythicAbilityKey::new(FAVORITE_METAMAGIC);
    match value {
        Some(feat) => {
            Pick::mythic_ability_for(key, Parameter::Metamagic(rules::FeatKey::new(feat)))
        }
        None => Pick::mythic_ability(key),
    }
}

const MR_1: SlotId = SlotId::Mythic { rank: 1 };

/// The guide's metamagic feats: every "Metamagic (…)" feat, the Trickster's Completely Normal Spell included.
const METAMAGIC_FEATS: [&str; 12] = [
    "metamagic-bolster-spell",
    "metamagic-completely-normal-spell",
    "metamagic-empower-spell",
    "metamagic-extend-spell",
    "metamagic-heighten-spell",
    "metamagic-intensified-spell",
    "metamagic-maximise-spell",
    "metamagic-persistent-spell",
    "metamagic-piercing-spell",
    "metamagic-quicken-spell",
    "metamagic-reach-spell",
    "metamagic-selective-spell",
];

#[test]
fn the_metamagic_feats_are_the_guides_metamagic_feats() {
    let metamagic: Vec<&str> = DATA
        .feats
        .iter()
        .filter(|feat| feat.is_metamagic())
        .map(|feat| feat.key.as_str())
        .collect();
    assert_eq!(metamagic, METAMAGIC_FEATS);
    // Each is named "Metamagic (…)", and no other feat is.
    for feat in &DATA.feats {
        assert_eq!(
            feat.name.starts_with("Metamagic ("),
            feat.is_metamagic(),
            "{}",
            feat.key.as_str()
        );
    }
}

#[test]
fn favorite_metamagic_offers_each_metamagic_at_a_mythic_rank() {
    assert_eq!(
        mythic_ability(FAVORITE_METAMAGIC).parameter,
        Some(ParameterKind::Metamagic)
    );
    let build = mythic("trickster", 20);
    let offered: Vec<(Pick, bool)> = engine()
        .options(&build, &MR_1)
        .unwrap()
        .into_iter()
        .filter(|option| option.pick.key() == FAVORITE_METAMAGIC)
        .map(|option| (option.pick, option.status.is_ok()))
        .collect();
    let expected: Vec<(Pick, bool)> = METAMAGIC_FEATS
        .iter()
        .map(|feat| (favorite(Some(feat)), true))
        .collect();
    assert_eq!(offered, expected);
}

#[test]
fn favorite_metamagic_for_persistent_spell_is_named_and_round_trips() {
    let engine = engine();
    let mut build = mythic("trickster", 20);
    let pick = favorite(Some("metamagic-persistent-spell"));
    build.picks.insert(MR_1, pick.clone());
    assert!(engine.is_possible(&build));
    assert_eq!(
        engine.pick_name(&pick),
        "Favorite Metamagic (Persistent Spell)"
    );
    assert_eq!(engine.open_link(&encode_build(&build)), Ok(build));
}

#[test]
fn an_older_link_with_favorite_metamagic_opens_with_its_metamagic_left_open() {
    let engine = engine();
    let mut build = mythic("trickster", 20);
    build.picks.insert(MR_1, favorite(None));
    assert_eq!(engine.open_link(&encode_build(&build)), Ok(build));
    assert_eq!(
        engine.value_left_open(&favorite(None)),
        Some(ParameterKind::Metamagic)
    );
    // Held once: a second one at MR 3 is refused, whatever its metamagic.
    let mut twice = mythic("trickster", 20);
    twice.picks.insert(MR_1, favorite(None));
    twice.picks.insert(
        SlotId::Mythic { rank: 3 },
        favorite(Some("metamagic-quicken-spell")),
    );
    assert_eq!(
        why_not(&twice, &SlotId::Mythic { rank: 3 }),
        Some(rules::Issue::AlreadyHeld)
    );
}

#[test]
fn stat_focus_takes_a_stat_the_guide_lists() {
    assert_eq!(feat("stat-focus").parameter, Some(ParameterKind::Stat));
    let build = mythic("trickster", 20);
    let stats: Vec<Parameter> = engine()
        .options(&build, &general_slot(1))
        .unwrap()
        .into_iter()
        .filter(|option| option.pick.key() == "stat-focus")
        .map(|option| option.pick.parameter().cloned().unwrap())
        .collect();
    assert_eq!(stats, Stat::ALL.map(Parameter::Stat));
    // An older link's Stat Focus, with no stat, has the stat left open.
    assert_eq!(
        engine().value_left_open(&Pick::feat(rules::FeatKey::new("stat-focus"))),
        Some(ParameterKind::Stat)
    );
}

#[test]
fn ascendant_element_takes_an_element_the_guide_lists() {
    assert_eq!(
        mythic_ability("ascendant-element").parameter,
        Some(ParameterKind::Element)
    );
    let build = mythic("trickster", 20);
    let elements: Vec<Parameter> = engine()
        .options(&build, &MR_1)
        .unwrap()
        .into_iter()
        .filter(|option| option.pick.key() == "ascendant-element")
        .map(|option| option.pick.parameter().cloned().unwrap())
        .collect();
    assert_eq!(elements, Element::ALL.map(Parameter::Element));
}

// ---- The audit -------------------------------------------------------------------------------------------------

/// The items whose summary says they choose something but that the ticket's audit left without a pick, each for a
/// reason the ticket gives: the choice is made in play (Selective Spell's creatures), a spell (no spell values
/// yet, ticket 121), the guide names no list of values (Energized Wild Shape's energy), a repeat rather than a
/// choice (Increased Damage Reduction), or a pick the model can't hold yet (Advanced Weapon Training, Lore
/// (Religion) II and III, Rogue Secret: ticket 125's follow-ups).
const CHECKED_WITHOUT_A_PICK: [&str; 8] = [
    "feat advanced-weapon-training",
    "feat energized-wild-shape",
    "feat metamagic-selective-spell",
    "feat spell-specialization",
    "option loremaster/secret/rogue-secret",
    "option stalwart-defender/defensive-powers/increased-damage-reduction",
    "option trickster/greater-mythic-trick/lore-religion-iii",
    "option trickster/improved-mythic-trick/lore-religion-ii",
];

fn chooses(summary: &str) -> bool {
    let summary = summary.to_lowercase();
    ["chosen", "choose", "choice", "pick"]
        .iter()
        .any(|word| summary.contains(word))
}

#[test]
fn every_item_whose_summary_chooses_has_its_pick_or_was_checked() {
    let mut without_pick = BTreeSet::new();
    for feat in DATA.feats.iter().filter(|feat| chooses(&feat.summary)) {
        if feat.parameter.is_none() && feat.extra_pick_of.is_empty() {
            without_pick.insert(format!("feat {}", feat.key.as_str()));
        }
    }
    for ability in DATA.mythic_abilities.iter().filter(|a| chooses(&a.summary)) {
        if ability.parameter.is_none() && ability.selections.is_empty() {
            without_pick.insert(format!("mythic ability {}", ability.key.as_str()));
        }
    }
    for feat in DATA.mythic_feats.iter().filter(|f| chooses(&f.summary)) {
        if feat.parameter.is_none() && feat.extra_pick.is_none() {
            without_pick.insert(format!("mythic feat {}", feat.key.as_str()));
        }
    }
    for option in DATA
        .selections()
        .flat_map(|selection| &selection.options)
        .filter(|option| chooses(&option.summary))
    {
        let picks = option.parameter.is_some()
            || option.chosen_feat.is_some()
            || option.pool.is_some()
            || option
                .levels
                .iter()
                .any(|row| !row.selections.is_empty() || !row.feat_slots.is_empty());
        if !picks {
            without_pick.insert(format!("option {}", option.key.as_str()));
        }
    }
    assert_eq!(
        without_pick,
        CHECKED_WITHOUT_A_PICK
            .iter()
            .map(|item| (*item).to_owned())
            .collect::<BTreeSet<_>>()
    );
}
