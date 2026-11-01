//! Ticket 34: the backgrounds, from the user's list (2026-11-01, `research/backgrounds-from-user.md`). The saved
//! guide pages don't list backgrounds, so this table is the source and the extractor carries it as it stands.
//!
//! The user's decisions (2026-11-01, "all recommended") say what is modelled:
//!
//! 1. **Weapon proficiencies:** the Aldori Swordsman's dueling sword is Exotic Weapon Proficiency (Dueling Sword),
//!    granted for that value. Every other weapon (no feat checks a single simple or martial weapon) and the
//!    buckler are in the note.
//! 2. **Light armour:** Armor Proficiency (Light Armor), granted (Bounty Hunter, Gladiator, Guard, Mercenary).
//! 3. **Feats:** granted (Martial Disciple: Improved Unarmed Strike; Arcane: Scribe Scrolls; Divine: Brew Potions).
//! 4. **Everything else** is the note.
//!
//! Names, skills, proficiencies and bonuses are spelt as the user gave them; keys are their [`slug`]s. Skills, feats
//! and the weapon are found in the data by name, and [`backgrounds`] panics on one it doesn't find.

use std::collections::BTreeSet;

use rules::{
    Background, BackgroundGroup, BackgroundKey, FeatValue, GameData, Parameter, ParameterKind,
    WeaponProficiency,
};

use crate::names::slug;

/// One row of the user's table.
struct Row {
    group: Option<BackgroundGroup>,
    name: &'static str,
    class_skills: &'static [&'static str],
    proficiencies: &'static [&'static str],
    feat: Option<&'static str>,
    other: Option<&'static str>,
}

const fn row(
    group: BackgroundGroup,
    name: &'static str,
    class_skills: &'static [&'static str],
    proficiencies: &'static [&'static str],
    feat: Option<&'static str>,
    other: Option<&'static str>,
) -> Row {
    Row {
        group: Some(group),
        name,
        class_skills,
        proficiencies,
        feat,
        other,
    }
}

/// The user's list, in its order.
const ROWS: [Row; 39] = {
    use BackgroundGroup::*;
    const FORT: Option<&str> = Some("+1 Fortitude");
    [
        row(
            Craftsman,
            "Farmhand",
            &["Athletics", "Lore (Nature)"],
            &["scythe"],
            None,
            FORT,
        ),
        row(
            Craftsman,
            "Lumberjack",
            &["Athletics"],
            &["handaxe", "battleaxe", "greataxe"],
            None,
            FORT,
        ),
        row(
            Craftsman,
            "Miner",
            &["Athletics"],
            &["light pick", "heavy pick"],
            None,
            FORT,
        ),
        row(
            Craftsman,
            "Smith",
            &["Athletics"],
            &["light hammer", "warhammer"],
            None,
            FORT,
        ),
        row(
            Noble,
            "Courtier",
            &["Persuasion", "Trickery"],
            &["rapier", "buckler"],
            None,
            None,
        ),
        row(
            Noble,
            "Emissary",
            &["Persuasion", "Knowledge (World)"],
            &["rapier", "buckler"],
            None,
            None,
        ),
        row(
            Noble,
            "Leader",
            &["Persuasion"],
            &["rapier", "longsword", "greatsword", "buckler"],
            None,
            None,
        ),
        row(
            Oblate,
            "Acolyte",
            &["Persuasion", "Lore (Religion)"],
            &[],
            None,
            Some("Wisdom instead of Charisma for Persuasion"),
        ),
        row(
            Oblate,
            "Healer",
            &["Lore (Nature)", "Lore (Religion)"],
            &[],
            None,
            Some("Intelligence instead of Wisdom for both Lores"),
        ),
        row(
            Oblate,
            "Martial Disciple",
            &["Mobility", "Lore (Religion)"],
            &[],
            Some("Improved Unarmed Strike"),
            None,
        ),
        row(
            Regional,
            "Aldori Swordsman",
            &["Persuasion"],
            &["dueling sword", "buckler"],
            None,
            None,
        ),
        row(
            Regional,
            "Alkenstar Alchemist",
            &["Knowledge (Arcana)"],
            &[],
            None,
            Some("+1 damage with bombs and alchemist items"),
        ),
        row(
            Regional,
            "Andoran Diplomat",
            &["Persuasion"],
            &[],
            None,
            Some("+2 Diplomacy"),
        ),
        row(
            Regional,
            "Cheliaxian Diabolist",
            &["Knowledge (Arcana)"],
            &[],
            None,
            Some("+4 dodge AC against devils"),
        ),
        row(
            Regional,
            "Gebbite Necromancer",
            &["Lore (Religion)"],
            &[],
            None,
            Some("summoned undead and undead companions +3 max HP"),
        ),
        row(
            Regional,
            "Mendevian Orphan",
            &["Lore (Religion)"],
            &[],
            None,
            Some("+1 saves against demons"),
        ),
        row(
            Regional,
            "Mwangi Hunter",
            &["Lore (Nature)", "Stealth"],
            &["dart", "throwing axe", "javelin"],
            None,
            Some("+1 damage with throwing weapons"),
        ),
        row(
            Regional,
            "Nexian Scholar",
            &["Knowledge (Arcana)"],
            &[],
            None,
            Some("+2 Knowledge (Arcana)"),
        ),
        row(
            Regional,
            "Osirian Historian",
            &["Knowledge (World)"],
            &[],
            None,
            Some("+2 Knowledge (World)"),
        ),
        row(
            Regional,
            "Qadiran Wanderer",
            &["Perception", "Lore (Nature)"],
            &[],
            None,
            Some("+2 saves against fatigue and exhaustion"),
        ),
        row(
            Regional,
            "River Kingdoms Daredevil",
            &["Mobility"],
            &["shortsword", "dagger"],
            None,
            Some("+2 AC against attacks of opportunity"),
        ),
        row(
            Regional,
            "Shackles Corsair",
            &["Mobility", "Athletics"],
            &["scimitar"],
            None,
            Some("+2 CMD against trip"),
        ),
        row(
            Regional,
            "Ustalavic Peasant",
            &["Athletics"],
            &["scythe"],
            None,
            Some("+1 AC and damage under fear effects"),
        ),
        row(
            Regional,
            "Varisian Explorer",
            &["Trickery", "Knowledge (World)"],
            &[],
            None,
            Some("+1 Reflex"),
        ),
        row(
            Regional,
            "Warrior of the Linnorm Kings",
            &["Mobility"],
            &["handaxe", "battleaxe", "greataxe"],
            None,
            Some("cold resistance 5"),
        ),
        row(
            Scholar,
            "Arcane",
            &["Use Magic Device", "Knowledge (Arcana)"],
            &[],
            Some("Scribe Scrolls"),
            None,
        ),
        row(
            Scholar,
            "Divine",
            &["Use Magic Device", "Lore (Religion)"],
            &[],
            Some("Brew Potions"),
            None,
        ),
        row(
            Scholar,
            "Nature",
            &["Use Magic Device", "Lore (Nature)"],
            &[],
            None,
            Some("one druid orison at will: Flare, Guidance, Light, Resistance or Virtue"),
        ),
        row(
            StreetUrchin,
            "Acrobat",
            &["Trickery", "Mobility"],
            &["throwing axe"],
            None,
            None,
        ),
        row(
            StreetUrchin,
            "Mugger",
            &["Trickery", "Persuasion"],
            &["dagger", "punching dagger", "dart"],
            None,
            None,
        ),
        row(
            StreetUrchin,
            "Pickpocket",
            &["Trickery", "Stealth"],
            &[],
            None,
            Some("+2 initiative"),
        ),
        row(
            Wanderer,
            "Hermit",
            &["Lore (Nature)"],
            &["club", "greatclub", "quarterstaff", "dart"],
            None,
            None,
        ),
        row(
            Wanderer,
            "Hunter",
            &["Lore (Nature)", "Perception"],
            &["shortbow", "longbow"],
            None,
            None,
        ),
        row(
            Wanderer,
            "Nomad",
            &["Lore (Nature)"],
            &["scimitar"],
            None,
            Some("animal companion +3 max HP"),
        ),
        row(
            Warrior,
            "Bounty Hunter",
            &["Stealth"],
            &["light armour", "club", "light mace", "heavy mace"],
            None,
            None,
        ),
        row(
            Warrior,
            "Gladiator",
            &["Persuasion"],
            &["light armour", "trident", "shortsword", "buckler"],
            None,
            None,
        ),
        row(
            Warrior,
            "Guard",
            &["Perception"],
            &["light armour", "shortsword", "light crossbow"],
            None,
            None,
        ),
        row(
            Warrior,
            "Mercenary",
            &["Knowledge (World)"],
            &["light armour", "spear", "longspear", "javelin"],
            None,
            None,
        ),
        Row {
            group: None,
            name: "None",
            class_skills: &[],
            proficiencies: &[],
            feat: None,
            other: None,
        },
    ]
};

/// Decision 2: light armour is the feat.
const LIGHT_ARMOUR: (&str, &str) = ("light armour", "Armor Proficiency (Light Armor)");
/// Decision 1: the one weapon granted as a feat, for that weapon.
const DUELING_SWORD: (&str, &str, &str) = (
    "dueling sword",
    "Exotic Weapon Proficiency",
    "Dueling Sword",
);

/// The user's backgrounds, their skills, feats and weapon found in `data` by name.
pub fn backgrounds(data: &GameData) -> Vec<Background> {
    let skill = |name: &str| {
        data.skills
            .iter()
            .find(|skill| skill.name == name)
            .unwrap_or_else(|| panic!("no skill {name}"))
            .key
            .clone()
    };
    let feat = |name: &str| {
        data.feats
            .iter()
            .find(|feat| feat.name == name)
            .unwrap_or_else(|| panic!("no feat {name}"))
    };
    ROWS.iter()
        .map(|row| {
            let mut feats = BTreeSet::new();
            let mut feats_for = BTreeSet::new();
            if let Some(name) = row.feat {
                feats.insert(feat(name).key.clone());
            }
            let mut noted = Vec::new();
            for &proficiency in row.proficiencies {
                if proficiency == LIGHT_ARMOUR.0 {
                    feats.insert(feat(LIGHT_ARMOUR.1).key.clone());
                } else if proficiency == DUELING_SWORD.0 {
                    let (_, feat_name, weapon_name) = DUELING_SWORD;
                    let proficiency = feat(feat_name);
                    assert_eq!(proficiency.parameter, Some(ParameterKind::ExoticWeapon));
                    let weapon = data
                        .weapons
                        .iter()
                        .find(|weapon| weapon.name == weapon_name)
                        .unwrap_or_else(|| panic!("no weapon {weapon_name}"));
                    assert_eq!(weapon.proficiency, Some(WeaponProficiency::Exotic));
                    feats_for.insert(FeatValue {
                        feat: proficiency.key.clone(),
                        value: Parameter::Weapon(weapon.key.clone()),
                    });
                } else {
                    noted.push(proficiency);
                }
            }
            Background {
                key: BackgroundKey::new(slug(row.name)),
                name: row.name.to_owned(),
                group: row.group,
                note: note(&noted, row.other),
                class_skills: row.class_skills.iter().map(|name| skill(name)).collect(),
                feats,
                feats_for,
            }
        })
        .collect()
}

/// The proficiencies left to the note, then the other bonus, as sentences: "Proficient with scythe. +1
/// Fortitude."
fn note(proficiencies: &[&str], other: Option<&str>) -> String {
    let mut sentences = Vec::new();
    if let Some((last, rest)) = proficiencies.split_last() {
        let list = if rest.is_empty() {
            (*last).to_owned()
        } else {
            format!("{} and {last}", rest.join(", "))
        };
        sentences.push(format!("Proficient with {list}."));
    }
    if let Some(other) = other {
        let mut chars = other.chars();
        let first = chars.next().expect("a bonus has words");
        sentences.push(format!("{}{}.", first.to_uppercase(), chars.as_str()));
    }
    sentences.join(" ")
}
