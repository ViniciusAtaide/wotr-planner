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

/// The metamagic feats: every "Metamagic (…)" feat, the Trickster's Completely Normal Spell included.
/// Game: the player feats with AddMetamagicFeat are these twelve, the rest being monsters'.
const METAMAGIC_FEATS: [&str; 12] = [
    "metamagic-bolster-spell", // Game: BolsteredSpellFeat fbf5d9ce931f47f3a0c818b3f8ef8414
    "metamagic-completely-normal-spell", // Game: CompletelyNormalSpellFeat 094b6278f7b570f42aeaa98379f07cf2
    "metamagic-empower-spell",           // Game: EmpowerSpellFeat a1de1e4f92195b442adb946f0e2b9d4e
    "metamagic-extend-spell",            // Game: ExtendSpellFeat f180e72e4a9cbaa4da8be9bc958132ef
    "metamagic-heighten-spell",          // Game: HeightenSpellFeat 2f5d1e705c7967546b72ad8218ccf99c
    "metamagic-intensified-spell",       // Game: IntensifiedSpell 8ad7fd39abea4722b39eb5a67d606a41
    "metamagic-maximise-spell",          // Game: MaximizeSpellFeat 7f2b282626862e345935bbea5e66424b
    "metamagic-persistent-spell", // Game: PersistentSpellFeat cd26b9fa3f734461a0fcedc81cafaaac
    "metamagic-piercing-spell",   // Game: PiercingSpell c101ad6879a94204a780506f7a554865
    "metamagic-quicken-spell",    // Game: QuickenSpellFeat ef7ece7bb5bb66a41b256976b27f424e
    "metamagic-reach-spell",      // Game: ReachSpellFeat 46fad72f54a33dc4692d3b62eca7bb78
    "metamagic-selective-spell",  // Game: SelectiveSpellFeat 85f3340093d144dd944fff9a9adfd2f2
];

/// The metamagics Favorite Metamagic offers: every metamagic feat but Completely Normal Spell and Heighten Spell.
/// Game: FavoriteMetamagicSelection 503fb196aa222b24cb6cfdc9a284e838's ten options.
const FAVORITE_METAMAGICS: [&str; 10] = [
    "metamagic-bolster-spell", // Game: FavoriteMetamagicBolstered 8d44a7d4950f4d6cbfda8530d108e63a
    "metamagic-empower-spell", // Game: FavoriteMetamagicEmpower abebb469211025b48ac7a32da41f06e8
    "metamagic-extend-spell",  // Game: FavoriteMetamagicExtend 37118163dcb572047b995158346f0058
    "metamagic-intensified-spell", // Game: FavoriteMetamagicIntensified 8713d3ce0275459b8ff67ef799d31d4b
    "metamagic-maximise-spell", // Game: FavoriteMetamagicMaximize 9430a7211ba62cb4091d90abfdde58d0
    "metamagic-persistent-spell", // Game: FavoriteMetamagicPersistent eb6483b366a14a83afda1963799c3387
    "metamagic-piercing-spell", // Game: FavoriteMetamagicPiercing bd38adc649b9431d89269f00b55014bc
    "metamagic-quicken-spell",  // Game: FavoriteMetamagicQuicken 4ecf1a86ddc45d04c8b2df3a1c318e78
    "metamagic-reach-spell",    // Game: FavoriteMetamagicReach ac946df6c29143544aeefbde2b360ff1
    "metamagic-selective-spell", // Game: FavoriteMetamagicSelective 3cdc012185334b7da4ed42c6fcc948cf
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
fn favorite_metamagic_takes_a_metamagic() {
    // Game: MythicAbilitySelection ba0e5a900b775be4a99702f1ed08914d offers FavoriteMetamagicSelection
    // 503fb196aa222b24cb6cfdc9a284e838, one feature per metamagic (equivalent model: an ability with a value).
    assert_eq!(
        mythic_ability(FAVORITE_METAMAGIC).parameter,
        Some(ParameterKind::Metamagic)
    );
}

/// The game's Favorite Metamagic offers ten metamagics, each needing its metamagic feat: none is open to a trickster
/// with no metamagic feat.
#[test]
#[ignore = "waits on ticket 265: the game gives ten metamagics, each needing its feat"]
fn favorite_metamagic_offers_ten_metamagics_each_needing_its_feat() {
    let build = mythic("trickster", 20);
    let offered: Vec<(Pick, bool)> = engine()
        .options(&build, &MR_1)
        .unwrap()
        .into_iter()
        .filter(|option| option.pick.key() == FAVORITE_METAMAGIC)
        .map(|option| (option.pick, option.status.is_ok()))
        .collect();
    // Game: each of FavoriteMetamagicSelection 503fb196aa222b24cb6cfdc9a284e838's options has a PrerequisiteFeature on
    // its metamagic feat (FavoriteMetamagicPersistent eb6483b366a14a83afda1963799c3387 on PersistentSpellFeat
    // cd26b9fa3f734461a0fcedc81cafaaac, and so on).
    let expected: Vec<(Pick, bool)> = FAVORITE_METAMAGICS
        .iter()
        .map(|feat| (favorite(Some(feat)), false))
        .collect();
    assert_eq!(offered, expected);
}

#[test]
fn favorite_metamagic_for_persistent_spell_is_named_and_round_trips() {
    let engine = engine();
    let mut build = mythic("trickster", 20);
    // Game: FavoriteMetamagicPersistent eb6483b366a14a83afda1963799c3387 needs PersistentSpellFeat
    // cd26b9fa3f734461a0fcedc81cafaaac, taken first.
    build.picks.insert(
        general_slot(1),
        feat_pick_plain("metamagic-persistent-spell"),
    );
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
}

/// The game takes Favorite Metamagic again for another metamagic: the mythic ability is a selection, which skips the
/// rank check, and only each metamagic's feature is held once.
#[test]
fn favorite_metamagic_is_taken_again_for_another_metamagic() {
    // Game: MythicAbilitySelection ba0e5a900b775be4a99702f1ed08914d offers FavoriteMetamagicSelection
    // 503fb196aa222b24cb6cfdc9a284e838 (no prerequisite of its own); Blueprints/Classes/Selection/
    // BlueprintFeatureSelection.CanSelect (WrathDecompiled 6ce4a6b) skips the rank check for a selection, and
    // NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable refuses it only when no option is left; each option
    // (FavoriteMetamagicQuicken 4ecf1a86ddc45d04c8b2df3a1c318e78) has Ranks 1.
    let mut twice = mythic("trickster", 20);
    twice.picks.insert(
        general_slot(1),
        feat_pick_plain("metamagic-persistent-spell"),
    );
    twice
        .picks
        .insert(general_slot(3), feat_pick_plain("metamagic-quicken-spell"));
    twice
        .picks
        .insert(MR_1, favorite(Some("metamagic-persistent-spell")));
    twice.picks.insert(
        SlotId::Mythic { rank: 3 },
        favorite(Some("metamagic-quicken-spell")),
    );
    assert!(possible(&twice));
}

/// The game takes Ascendant Element again for another element, never twice for one (ticket 249).
#[test]
fn ascendant_element_is_taken_again_for_another_element_but_not_the_same() {
    // Game: MythicAbilitySelection ba0e5a900b775be4a99702f1ed08914d offers AscendantElementSelection
    // 535d4e5d8a5664f4797560c280941782, whose six features (AscendantElementAcid d808baf783a063e4d84a1aaadbdd1cb7, …)
    // have Ranks 1; Blueprints/Classes/Selection/BlueprintFeatureSelection.CanSelect (WrathDecompiled 6ce4a6b) skips
    // the rank check for a selection and refuses a held one-rank feature.
    let element = |value: Element| {
        Pick::mythic_ability_for(
            MythicAbilityKey::new("ascendant-element"),
            Parameter::Element(value),
        )
    };
    for key in ["ascendant-element", FAVORITE_METAMAGIC] {
        assert_eq!(mythic_ability(key).repeat, rules::Repeat::PerValue, "{key}");
    }
    let mr_3 = SlotId::Mythic { rank: 3 };
    let mut build = mythic("trickster", 20);
    build.picks.insert(MR_1, element(Element::Fire));
    let mut two = build.clone();
    two.picks.insert(mr_3.clone(), element(Element::Cold));
    assert!(possible(&two));
    let mut same = build;
    same.picks.insert(mr_3.clone(), element(Element::Fire));
    assert_eq!(why_not(&same, &mr_3), Some(rules::Issue::AlreadyHeld));
}

#[test]
fn stat_focus_takes_a_stat_the_guide_lists() {
    // Game: TricksterStatFocusFeatSelection 0d1d80bd3820a78488412581da3ad9c7: the same twelve stats, one feature each
    // (equivalent model: a feat with a value).
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
    // Game: AscendantElementSelection 535d4e5d8a5664f4797560c280941782: the same six elements.
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
        .options
        .iter()
        .filter(|option| chooses(&option.summary))
    {
        let picks = option.parameter.is_some()
            || option.chosen_feat.is_some()
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
