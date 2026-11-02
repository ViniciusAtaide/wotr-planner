//! Proficiency feats, from each class page's "… Proficiencies" section (prose the tables can't hold). A row that
//! grants a proficiency feature grants the Armor, Shield and Weapon Proficiency feats its text names, so feats
//! such as Shield Focus see them; a proficiency feature those feats cover in full is dropped. Runs on full rows,
//! before the archetype rows are made sparse.

use std::collections::BTreeSet;

use rules::{Class, FeatKey, FeatureKey, GameData};

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

/// Who a row addition applies to.
enum Who {
    /// The base class's rows.
    Class,
    /// Every archetype's rows.
    EveryArchetype,
    Archetype(&'static str),
}

/// Feats a class level grants that no feature in its table carries: (class, whose rows, level, feats).
const ROWS: &[(&str, Who, u8, &[&str])] = &[
    // "Kineticists are proficient with all simple weapons and light armour."
    ("kineticist", Who::Class, 1, &[SIMPLE, LIGHT]),
    ("kineticist", Who::EveryArchetype, 1, &[SIMPLE, LIGHT]),
    // "At 11th level, he gains Armor Proficiency (Heavy) as a bonus feat."
    ("oracle", Who::Archetype("oracle/purifier"), 11, &[HEAVY]),
];

/// "At 7th level, a Magus gains proficiency with medium armour … At 13th level … heavy armour", on the magus's
/// own features and each archetype's copy of them.
const MAGUS_ARMOR: [(&str, &str); 2] = [
    ("arcane-medium-armor", MEDIUM),
    ("arcane-heavy-armor", HEAVY),
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

/// The feats a feature grants.
fn feature_feats(feature: &FeatureKey) -> &'static [&'static str] {
    if let Some((_, feats)) = FEATURES.iter().find(|(key, _)| *key == feature.as_str()) {
        return feats;
    }
    let key = feature.as_str();
    if key.starts_with("magus/") {
        let last = key.rsplit('/').next().unwrap_or(key);
        if let Some((_, feat)) = MAGUS_ARMOR.iter().find(|(name, _)| *name == last) {
            return match *feat {
                MEDIUM => &[MEDIUM],
                _ => &[HEAVY],
            };
        }
    }
    &[]
}

/// The proficiency feats one row grants, and its features without the covered ones.
fn rewrite(
    class: &str,
    archetype: Option<&str>,
    level: u8,
    features: &mut Vec<FeatureKey>,
    feats: &mut BTreeSet<FeatKey>,
) {
    for feature in features.iter() {
        let overridden = archetype.and_then(|archetype| {
            OVERRIDES
                .iter()
                .find(|(a, f, _)| *a == archetype && *f == feature.as_str())
                .map(|(_, _, feats)| *feats)
        });
        let granted = overridden.unwrap_or_else(|| feature_feats(feature));
        feats.extend(granted.iter().map(|key| FeatKey::new(*key)));
    }
    for (row_class, who, row_level, granted) in ROWS {
        let applies = *row_class == class
            && *row_level == level
            && match (who, archetype) {
                (Who::Class, None) | (Who::EveryArchetype, Some(_)) => true,
                (Who::Archetype(wanted), Some(archetype)) => *wanted == archetype,
                _ => false,
            };
        if applies {
            feats.extend(granted.iter().map(|key| FeatKey::new(*key)));
        }
    }
    features.retain(|feature| !COVERED.contains(&feature.as_str()));
}

/// Adds the proficiency feats to every class and archetype row and drops the covered features, rows and
/// definitions both.
pub fn apply(data: &mut GameData) {
    for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        apply_to_class(class);
    }
}

fn apply_to_class(class: &mut Class) {
    let key = class.key.to_string();
    for row in &mut class.levels {
        rewrite(&key, None, row.level, &mut row.features, &mut row.feats);
    }
    class
        .features
        .retain(|feature| !COVERED.contains(&feature.key.as_str()));
    for archetype in &mut class.archetypes {
        let archetype_key = archetype.key.to_string();
        for row in &mut archetype.levels {
            rewrite(
                &key,
                Some(&archetype_key),
                row.level,
                &mut row.features,
                &mut row.feats,
            );
        }
        archetype
            .features
            .retain(|feature| !COVERED.contains(&feature.key.as_str()));
    }
}
