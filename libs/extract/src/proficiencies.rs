//! Proficiency feats, from each class page's "… Proficiencies" section (prose the tables can't hold). A cell naming
//! a proficiency feature grants the Armor, Shield and Weapon Proficiency feats its text names, so feats such as
//! Shield Focus see them; a proficiency feature those feats cover in full is those feats alone, and is never defined.
//! Built with the classes: [`provide`] gives each feature's cell as a pick ([`Pick::Feats`] or
//! [`Pick::FeatureAndFeats`]) and the feats a class level grants that no feature carries as row feats; [`check`]
//! asserts the magus's armour features carry their feats on the finished rows.

use std::collections::BTreeSet;

use rules::{FeatKey, GameData};

use crate::picks::{Pick, Picks, Rows, Rule};

const SIMPLE: &str = "simple-weapon-proficiency";
const MARTIAL: &str = "martial-weapons-proficiency";
const LIGHT: &str = "armor-proficiency-light-armor";
const MEDIUM: &str = "armor-proficiency-medium-armor";
const HEAVY: &str = "armor-proficiency-heavy-armor";
const SHIELD: &str = "shield-proficiency";
const TOWER: &str = "tower-shield-proficiency";

const BARBARIAN: &[&str] = &[SIMPLE, MARTIAL, LIGHT, MEDIUM, SHIELD];
const CAVALIER: &[&str] = &[SIMPLE, MARTIAL, LIGHT, MEDIUM, HEAVY, SHIELD];

/// Each proficiency feature and the feats its text names. A feature not here grants none.
const FEATURES: &[(&str, &[&str])] = &[
    // "Proficient with simple weapons and bombs. … light armour (but not shields)."
    ("alchemist/alchemist-proficiencies", &[SIMPLE, LIGHT]),
    ("arcanist/arcanist-proficiencies", &[SIMPLE]),
    ("barbarian/barbarian-proficiencies", BARBARIAN),
    // "… all simple and martial weapons and light and heavy shields."
    (
        "barbarian/instinctual-warrior/instinctual-warrior-proficiencies",
        &[SIMPLE, MARTIAL, SHIELD],
    ),
    // "… simple weapons plus the longsword, rapier, short sword and shortbow … light armour and shields."
    ("bard/bard-proficiencies", &[SIMPLE, LIGHT, SHIELD]),
    // "Independent of bloodline, a Bloodrager has the same proficiencies as a Barbarian."
    ("bloodrager/bloodrager-proficiencies", BARBARIAN),
    ("bloodrager/bloodrider/bloodrager-proficiencies", BARBARIAN),
    ("bloodrager/greenrager/bloodrager-proficiencies", BARBARIAN),
    ("bloodrager/hag-riven/bloodrager-proficiencies", BARBARIAN),
    (
        "bloodrager/mixed-blood-rager/bloodrager-proficiencies",
        BARBARIAN,
    ),
    ("bloodrager/primalist/bloodrager-proficiencies", BARBARIAN),
    (
        "bloodrager/reformed-fiend/bloodrager-proficiencies",
        BARBARIAN,
    ),
    ("bloodrager/spelleater/bloodrager-proficiencies", BARBARIAN),
    ("bloodrager/steelblood/bloodrager-proficiencies", BARBARIAN),
    // "… all types of armour and shields (excluding tower shields)."
    ("cavalier/cavalier-proficiencies", CAVALIER),
    (
        "cavalier/beast-rider/beast-rider-proficiencies",
        &[SIMPLE, MARTIAL, LIGHT, MEDIUM, SHIELD],
    ),
    // The Ghost Rider's table names the cavalier's proficiencies.
    ("cavalier/ghost-rider/cavleier-proficiencies", CAVALIER),
    // "… simple weapons, light and medium armour and shields (but not tower shields) … their deity's weapon."
    (
        "cleric/cleric-proficiencies",
        &[SIMPLE, LIGHT, MEDIUM, SHIELD],
    ),
    (
        "cleric/angelfire-apostle/apostle-proficiencies",
        &[SIMPLE, LIGHT, SHIELD],
    ),
    (
        "cleric/demonbane-priest/demonbane-proficiencies",
        &[SIMPLE, LIGHT, MEDIUM, HEAVY, SHIELD],
    ),
    // "… the club, dagger, …, but not any armour."
    ("cleric/ecclesitheurge/ecclesitheurge-proficiencies", &[]),
    // "… the club, dagger, dart, … light and medium armour and shields" (no metal armour, wooden shields only).
    ("druid/druid-proficiencies", &[LIGHT, MEDIUM, SHIELD]),
    // "… all types of armour and shields (including tower shields)."
    (
        "fighter/fighter-proficiencies",
        &[SIMPLE, MARTIAL, LIGHT, MEDIUM, HEAVY, SHIELD, TOWER],
    ),
    ("hunter/hunter-proficiencies", BARBARIAN),
    // "… simple weapons, longbow, shortbow and the favoured weapon of his deity … light, medium and shields."
    (
        "inquisitor/inquisitor-proficiencies",
        &[SIMPLE, LIGHT, MEDIUM, SHIELD],
    ),
    // "… light and heavy crossbows, shortbows and longbows", on top of the kineticist's (the feats page doesn't
    // count it among those without light armour).
    (
        "kineticist/kinetic-sharpshooter/kinetic-sharpshooter-proficiencies",
        &[SIMPLE, LIGHT],
    ),
    // "At 1st level, the Kinetic Knight becomes proficient with medium and heavy armour and shields."
    (
        "kineticist/kinetic-knight/elemental-bastion",
        &[MEDIUM, HEAVY, SHIELD],
    ),
    // "… simple and martial weapons … light armour and may cast magus spells while wearing it."
    ("magus/magus-proficiencies", &[SIMPLE, MARTIAL, LIGHT]),
    (
        "magus/armored-battlemage/armored-battlemage-proficiencies",
        &[SIMPLE, MARTIAL, LIGHT, MEDIUM],
    ),
    // "… only with simple weapons and his chosen weapon. He is not proficient with armour."
    ("magus/sword-saint/sword-saint-proficiencies", &[SIMPLE]),
    ("monk/monk-proficiencies", &[]),
    ("monk/sohei/sohei-proficiencies", &[SIMPLE, MARTIAL, LIGHT]),
    ("monk/zen-archer/zen-archer-proficiencies", &[]),
    (
        "oracle/oracle-proficiencies",
        &[SIMPLE, LIGHT, MEDIUM, SHIELD],
    ),
    ("paladin/paladin-proficiencies", CAVALIER),
    (
        "paladin/divine-hunter-paladin/divine-hunter-proficiencies",
        BARBARIAN,
    ),
    ("ranger/ranger-proficiencies", BARBARIAN),
    // "… simple weapons, rapier, short sword and short bow … light armour but not with shields."
    ("rogue/rogue-proficiencies", &[SIMPLE, LIGHT]),
    ("shaman/shaman-proficiencies", &[SIMPLE, LIGHT, MEDIUM]),
    (
        "shifter/shifter-proficiencies",
        &[SIMPLE, LIGHT, MEDIUM, SHIELD],
    ),
    // "… can cast magic while wearing light or medium armour and using a shield".
    ("skald/skald-proficiencies", BARBARIAN),
    ("slayer/slayer-proficiencies", BARBARIAN),
    (
        "slayer/stygian-slayer/stygian-slayer-proficiencies",
        &[SIMPLE, MARTIAL, LIGHT],
    ),
    // "… all types of armour and shields (except tower shields)."
    ("warpriest/warpriest-proficiencies", CAVALIER),
    // "… simple weapons, light armour and light shields along with rapiers, shortswords and shortbows."
    (
        "warpriest/cult-leader/cult-leader-proficiencies",
        &[SIMPLE, LIGHT],
    ),
    // "… all simple and martial weapons, the Sawtooth Saber exotic weapon and light armour."
    (
        "warpriest/mantis-zealot/mantis-zealot-proficiencies",
        &[SIMPLE, MARTIAL, LIGHT],
    ),
    ("witch/witch-proficiencies", &[SIMPLE]),
    // "… the club, dagger, heavy crossbow, light crossbow and quarterstaff."
    ("wizard/wizard-proficiencies", &[]),
];

/// An archetype that grants its class's proficiency feature but has other proficiencies: (archetype, feature,
/// feats).
const OVERRIDES: &[(&str, &str, &[&str])] = &[
    // "Doesn't gain proficiency with medium armour or shields."
    (
        "cleric/herald-caller",
        "cleric/cleric-proficiencies",
        &[SIMPLE, LIGHT],
    ),
    // "The Eldritch Scoundrel is not proficient with light armour."
    (
        "rogue/eldritch-scoundrel",
        "rogue/rogue-proficiencies",
        &[SIMPLE],
    ),
];

/// Feats a class level grants that no feature in its table carries: (whose rows, level, feats).
fn rows() -> [(Rows, u8, &'static [&'static str]); 3] {
    [
        // "Kineticists are proficient with all simple weapons and light armour."
        (Rows::Of("kineticist".into()), 1, &[SIMPLE, LIGHT]),
        (
            Rows::EveryArchetypeOf("kineticist".into()),
            1,
            &[SIMPLE, LIGHT],
        ),
        // "At 11th level, he gains Armor Proficiency (Heavy) as a bonus feat."
        (Rows::Of("oracle/purifier".into()), 11, &[HEAVY]),
    ]
}

/// "At 7th level, a Magus gains proficiency with medium armour … At 13th level … heavy armour", on the magus's
/// own features and each archetype's copy of them ([`check`] asserts none is missed).
const MAGUS_ARMOR: [(&str, &str); 2] = [
    ("arcane-medium-armor", MEDIUM),
    ("arcane-heavy-armor", HEAVY),
];

/// The tables that name the magus's armour features: the magus's (whose features the archetypes that keep them
/// share) and the archetypes that have copies of their own.
const MAGUS_ARMOR_TABLES: [(&str, &[&str]); 6] = [
    ("magus", &[MEDIUM, HEAVY]),
    ("magus/armored-battlemage", &[HEAVY]),
    ("magus/bladebound", &[MEDIUM, HEAVY]),
    ("magus/eldritch-archer", &[MEDIUM, HEAVY]),
    ("magus/eldritch-scion", &[MEDIUM, HEAVY]),
    ("magus/hexcrafter", &[MEDIUM, HEAVY]),
];

/// Proficiency features whose feats cover all they say. The others name weapons no feat covers, bombs, casting in
/// armour, or are named by a prerequisite (Slayer's Feint names Slayer Proficiencies), or the guide gives them no
/// text (the sorcerer's).
const COVERED: &[&str] = &[
    "arcanist/arcanist-proficiencies",
    "barbarian/barbarian-proficiencies",
    "barbarian/instinctual-warrior/instinctual-warrior-proficiencies",
    "bloodrager/bloodrager-proficiencies",
    "bloodrager/bloodrider/bloodrager-proficiencies",
    "bloodrager/greenrager/bloodrager-proficiencies",
    "bloodrager/hag-riven/bloodrager-proficiencies",
    "bloodrager/mixed-blood-rager/bloodrager-proficiencies",
    "bloodrager/primalist/bloodrager-proficiencies",
    "bloodrager/reformed-fiend/bloodrager-proficiencies",
    "bloodrager/spelleater/bloodrager-proficiencies",
    "bloodrager/steelblood/bloodrager-proficiencies",
    "cavalier/cavalier-proficiencies",
    "cavalier/beast-rider/beast-rider-proficiencies",
    "cavalier/ghost-rider/cavleier-proficiencies",
    "cleric/angelfire-apostle/apostle-proficiencies",
    "cleric/demonbane-priest/demonbane-proficiencies",
    "fighter/fighter-proficiencies",
    "hunter/hunter-proficiencies",
    "oracle/oracle-proficiencies",
    "paladin/paladin-proficiencies",
    "paladin/divine-hunter-paladin/divine-hunter-proficiencies",
    "ranger/ranger-proficiencies",
    "shaman/shaman-proficiencies",
    "shifter/shifter-proficiencies",
    "slayer/stygian-slayer/stygian-slayer-proficiencies",
    "warpriest/warpriest-proficiencies",
    "witch/witch-proficiencies",
];

fn feats(keys: &[&str]) -> BTreeSet<FeatKey> {
    keys.iter().map(|key| FeatKey::new(*key)).collect()
}

/// A cell naming `feature` grants `granted`, in place of the feature when they cover it.
fn pick(feature: &str, granted: &[&str]) -> Pick {
    if COVERED.contains(&feature) {
        Pick::Feats(feats(granted))
    } else {
        Pick::FeatureAndFeats(feats(granted))
    }
}

/// Each proficiency feature's cell, an archetype's override first (a rule naming its table comes before the one for
/// every table), the magus's armour features, and the feats a level grants beside its cells.
pub fn provide(picks: &mut Picks) {
    for (archetype, feature, granted) in OVERRIDES {
        picks.rule(Rule {
            table: Some((*archetype).to_owned()),
            ..Rule::every(feature, pick(feature, granted))
        });
    }
    for (feature, granted) in FEATURES {
        picks.rule(Rule::every(feature, pick(feature, granted)));
    }
    for (table, armour) in MAGUS_ARMOR_TABLES {
        for (name, feat) in MAGUS_ARMOR {
            if armour.contains(&feat) {
                let feature = format!("{table}/{name}");
                picks.rule(Rule::every(&feature, pick(&feature, &[feat])));
            }
        }
    }
    for (rows, level, granted) in rows() {
        picks.row_feats(rows, level, feats(granted));
    }
}

/// Every magus feature for medium or heavy armour is one [`provide`] names, and each row holding it holds its feat.
/// Changes nothing.
pub fn check(data: &GameData) {
    let magus = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "magus")
        .expect("the magus");
    let named: BTreeSet<String> = MAGUS_ARMOR_TABLES
        .iter()
        .flat_map(|(table, armour)| {
            MAGUS_ARMOR
                .iter()
                .filter(|(_, feat)| armour.contains(feat))
                .map(move |(name, _)| format!("{table}/{name}"))
        })
        .collect();
    let defined = magus
        .features
        .iter()
        .chain(magus.archetypes.iter().flat_map(|a| &a.features));
    for feature in defined {
        if MAGUS_ARMOR
            .iter()
            .any(|(name, _)| feature.key.as_str().ends_with(&format!("/{name}")))
        {
            assert!(named.contains(feature.key.as_str()), "{}", feature.key);
        }
    }
    let rows = magus
        .levels
        .iter()
        .map(|row| (&row.features, &row.feats))
        .chain(
            magus
                .archetypes
                .iter()
                .flat_map(|a| a.levels.iter().map(|row| (&row.features, &row.feats))),
        );
    for (features, granted) in rows {
        for feature in features {
            for (name, feat) in MAGUS_ARMOR {
                if feature.as_str().ends_with(&format!("/{name}")) {
                    assert!(granted.contains(&FeatKey::new(feat)), "{feature}: {feat}");
                }
            }
        }
    }
}
