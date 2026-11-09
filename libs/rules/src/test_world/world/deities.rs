//! Deities, and the classes that serve one: the Priest (a deity-bound class with deity-limited domains and
//! blessings) and the Seeker (anyone, with a creed for the faithful and one for atheists).

use std::collections::BTreeSet;

use crate::*;

use crate::test_world::builds::{class, feature, option};
use crate::test_world::world::classes::{archetype, row};
use crate::test_world::world::selections::{levelled_selection, selection_option};
use crate::test_world::world::{guide, plain_feature};

/// Three invented deities over the test world's three domains (fire, war, water):
///
/// - the Lawgiver, lawful good, offers fire and war, favours the dueling sword (exotic);
/// - the Balance, true neutral, offers water, favours the longsword (martial);
/// - the Fist, lawful neutral, offers war and water, favours the unarmed strike.
pub(super) fn deities() -> Vec<Deity> {
    use Alignment::*;
    let deity = |key: &str, alignments: &[Alignment], weapons: &[&str], domains: &[&str]| Deity {
        key: DeityKey::new(key),
        name: capitalised(key),
        guide: guide("character-creation#deities"),
        alignments: alignments.iter().copied().collect(),
        favoured_weapons: weapons.iter().map(|key| WeaponKey::new(*key)).collect(),
        domains: domains
            .iter()
            .map(|key| option(&format!("cleric/domain/{key}")))
            .collect(),
    };
    vec![
        deity(
            "lawgiver",
            &[LawfulGood, NeutralGood, LawfulNeutral],
            &["dueling-sword"],
            &["fire", "war"],
        ),
        deity(
            "balance",
            &[
                NeutralGood,
                LawfulNeutral,
                TrueNeutral,
                ChaoticNeutral,
                NeutralEvil,
            ],
            &["longsword"],
            &["water"],
        ),
        deity(
            "fist",
            &[LawfulGood, LawfulNeutral, TrueNeutral, LawfulEvil],
            &["unarmed-strike"],
            &["war", "water"],
        ),
    ]
}

fn capitalised(key: &str) -> String {
    let mut chars = key.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// The domains a Priest's selections share: the cleric's.
const DOMAINS: [&str; 3] = [
    "cleric/domain/fire",
    "cleric/domain/war",
    "cleric/domain/water",
];

/// Priest: needs a deity. At 1, Priest Proficiencies (the deity's favoured weapons), a domain of the deity's and a
/// heresy (a domain the deity doesn't offer); at 2, a blessing, by its domain. The Zealot needs the Lawgiver, the
/// Apostle a deity offering the fire domain.
pub(in crate::test_world::world) fn priest() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 1 {
                r.features.push(feature("priest/proficiencies"));
                r.selections.push(FeatureSelectionKey::new("priest/domain"));
                r.selections.push(FeatureSelectionKey::new("priest/heresy"));
            }
            if l == 2 {
                r.selections
                    .push(FeatureSelectionKey::new("priest/blessing"));
            }
            r
        })
        .collect();
    let blessing = |key: &str, domain: &str| SelectionOption {
        domain: Some(option(&format!("cleric/domain/{domain}"))),
        ..selection_option(&format!("priest/blessing/{key}"), None, None)
    };
    Class {
        casting_ability: None,
        key: class("priest"),
        name: "Priest".into(),
        summary: String::new(),
        guide: guide("priest"),
        skill_points: 2,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::new(),
        prerequisite: Some(Prerequisite::Worships),
        features: vec![Feature {
            favoured_weapon_proficiency: true,
            ..plain_feature("priest/proficiencies", "Priest Proficiencies")
        }],
        selections: vec![
            FeatureSelection {
                deity_domains: Some(DeityDomains::Offered),
                ..levelled_selection("priest/domain", &["priest"], vec![], &DOMAINS)
            },
            FeatureSelection {
                deity_domains: Some(DeityDomains::NotOffered),
                ..levelled_selection("priest/heresy", &["priest"], vec![], &DOMAINS)
            },
            FeatureSelection {
                deity_domains: Some(DeityDomains::Offered),
                ..levelled_selection(
                    "priest/blessing",
                    &[],
                    vec![blessing("fire", "fire"), blessing("water", "water")],
                    &[],
                )
            },
        ],
        feat_lists: vec![],
        levels,
        archetypes: vec![
            Archetype {
                prerequisite: Some(Prerequisite::Deity(DeityKey::new("lawgiver"))),
                ..archetype("priest/zealot")
            },
            Archetype {
                prerequisite: Some(Prerequisite::DeityOffers(option("cleric/domain/fire"))),
                ..archetype("priest/apostle")
            },
        ],
    }
}

/// Seeker: anyone. At 1, a creed: Godless needs atheism, Devout a deity.
pub(in crate::test_world::world) fn seeker() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 1 {
                r.selections.push(FeatureSelectionKey::new("seeker/creed"));
            }
            r
        })
        .collect();
    Class {
        casting_ability: None,
        key: class("seeker"),
        name: "Seeker".into(),
        summary: String::new(),
        guide: guide("seeker"),
        skill_points: 4,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Good,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::new(),
        prerequisite: None,
        features: vec![],
        selections: vec![levelled_selection(
            "seeker/creed",
            &[],
            vec![
                selection_option("seeker/creed/godless", None, Some(Prerequisite::Atheist)),
                selection_option("seeker/creed/devout", None, Some(Prerequisite::Worships)),
            ],
            &[],
        )],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}
