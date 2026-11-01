//! Ticket 32: the user's decisions on tickets 26–28's open questions (2026-11-01, "all recommended") and the skill
//! facts the user gave (`research/skills-from-user.md`), applied to finished data.
//!
//! - **Repeats** (27-1): every parameterised feat repeats once per value.
//! - **Weapons** (27-2): the Sawtooth Saber is exotic; natural attacks are one value per attack the guide names
//!   (bite, claw, gore, hoof, slam, spike, tail, talon, wing) instead of "natural weapons"; Kinetic Blast and Bomb
//!   are values.
//! - **Armor Focus** (27-4) needs the armour proficiency of the armour type it's taken for (`FeatIfValue`).
//! - **Values on mythic feats** (27-5): Improved Critical, Spell Focus and Skill Focus (Mythic) need their base
//!   feat in the same value; Expanded Arsenal and School Mastery take a school; the Trickster's Improved Improved
//!   Critical chain takes "the selected weapon".
//! - **Fixed values** (27-6): the Mantis Zealot's four Sawtooth Saber feats and its Exotic Weapon Proficiency, the
//!   Chirurgeon's and the Enlightened Philosopher's Skill Focus are feats granted for one value (`feats_for`).
//! - **Restricted choices** (27-6): the Zen Archer's Way of the Bow is a pick of a longbow or a shortbow (Weapon
//!   Focus at 2, Weapon Specialization in the same bow at 6), the shaman's Battle Master names a weapon for its
//!   feats at 8 and 16, the Sword Saint's Chosen Weapon and the half-elf's Adaptability are one-feat bonus lists.
//! - **The slayer's combat style feats** (27-8): three slayer talents, one for each window (2 or 4, 6 or 8, 10+),
//!   each a bonus feat from the ranger styles' lists of its stage.
//! - **Perfect Mind** (26-3): BAB 15 and skill ranks at the Mythic rank, for prerequisites. **Destiny Beyond Birth**
//!   (26-4) removes the race's ability penalty.
//! - **Razmiri Infiltrator** (28-5): true neutral or any evil, the Assassin's reading.
//! - **Skill facts from the user**: the Tortured Crusader's 3 skill points and class skills (the guide's paladin page
//!   agrees), the Arcane Trickster without Use Magic Device, the Dragon Disciple with Knowledge (World).
//!
//! Each rule asserts the guide phrase it rests on (the user's skill facts excepted) and what it finds in the data,
//! so it fails loudly on data it doesn't expect. [`apply`] runs on the finished data of an extraction and, through
//! ticket 32's one-off rewrite, on `game-data/`.

use std::collections::BTreeSet;

use rules::{
    Alignment, ArmourType, Class, ClassKey, FeatKey, FeatList, FeatSlot, FeatValue, FeatureKey,
    FeatureSelectionKey, GameData, MythicFeatKey, OptionLevelRow, Parameter, ParameterKind,
    Prerequisite, Repeat, SkillKey, Weapon, WeaponKey, WeaponProficiency,
};

use crate::guide::Guide;
use crate::option_levels::{Log, list, option, page, section, selection};

/// Applies every rule, in the order above.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    repeats(data, &mut log);
    weapons(guide, data, &mut log);
    armor_focus(guide, data, &mut log);
    mythic_values(guide, data, &mut log);
    fixed_values(guide, data, &mut log);
    zen_archer(guide, data, &mut log);
    battle_master(guide, data, &mut log);
    sword_saint(guide, data, &mut log);
    half_elf(guide, data, &mut log);
    slayer_combat_styles(guide, data, &mut log);
    perfect_mind(guide, data, &mut log);
    razmiri_infiltrator(guide, data, &mut log);
    skills(guide, data, &mut log);
    log
}

/// Whether an extraction's report item is one [`apply`] settles: the Armor Focus line the grammar can't resolve.
pub fn settled(at: &str, text: &str) -> bool {
    at == "feats#armor-focus" && text == "Armor Proficiency (Light. Medium or Heavy)"
}

/// Asserts that a section of a page says `phrase`.
pub(crate) fn says(guide: &Guide, page_slug: &str, anchor: &str, phrase: &str) {
    let text = section(page(guide, page_slug), anchor).text();
    assert!(
        text.contains(phrase),
        "{page_slug}#{anchor}: no {phrase:?} in {text:?}"
    );
}

/// Asserts that a section's table has `cell` at `level` (the guide's level column).
pub(crate) fn cell_at(guide: &Guide, page_slug: &str, anchor: &str, level: &str, cell: &str) {
    let found = section(page(guide, page_slug), anchor)
        .tables()
        .flatten()
        .any(|row| row.first().map(String::as_str) == Some(level) && row.iter().any(|c| c == cell));
    assert!(found, "{page_slug}#{anchor}: no {cell:?} at {level}");
}

fn feat(key: &str) -> FeatKey {
    FeatKey::new(key)
}

fn weapon(key: &str) -> Parameter {
    Parameter::Weapon(WeaponKey::new(key))
}

pub(crate) fn class_mut<'d>(data: &'d mut GameData, key: &str) -> &'d mut Class {
    data.classes
        .iter_mut()
        .chain(&mut data.prestige_classes)
        .find(|class| class.key.as_str() == key)
        .unwrap_or_else(|| panic!("no class {key}"))
}

pub(crate) fn archetype_mut<'d>(
    data: &'d mut GameData,
    class: &str,
    key: &str,
) -> &'d mut rules::Archetype {
    class_mut(data, class)
        .archetypes
        .iter_mut()
        .find(|archetype| archetype.key.as_str() == key)
        .unwrap_or_else(|| panic!("no archetype {key}"))
}

pub(crate) fn archetype_row_mut<'d>(
    data: &'d mut GameData,
    class: &str,
    archetype: &str,
    level: u8,
) -> &'d mut rules::ArchetypeLevelRow {
    archetype_mut(data, class, archetype)
        .levels
        .iter_mut()
        .find(|row| row.level == level)
        .unwrap_or_else(|| panic!("{archetype} has no row {level}"))
}

/// Takes a feature off a row's grants, asserting it was there.
pub(crate) fn take_feature(features: &mut Vec<FeatureKey>, key: &str, at: &str) {
    let before = features.len();
    features.retain(|feature| feature.as_str() != key);
    assert_eq!(features.len(), before - 1, "{at}: no single {key}");
}

/// Removes a feature's definition from its owner once no row grants it.
pub(crate) fn drop_feature(
    data: &mut GameData,
    class: &str,
    archetype: Option<&str>,
    key: &str,
    log: &mut Log,
) {
    let class = class_mut(data, class);
    let granted = class
        .levels
        .iter()
        .any(|row| row.features.iter().any(|f| f.as_str() == key))
        || class
            .archetypes
            .iter()
            .flat_map(|a| &a.levels)
            .any(|row| row.features.iter().any(|f| f.as_str() == key));
    assert!(!granted, "{key} is still granted");
    let features = match archetype {
        Some(archetype) => {
            &mut class
                .archetypes
                .iter_mut()
                .find(|a| a.key.as_str() == archetype)
                .unwrap_or_else(|| panic!("no archetype {archetype}"))
                .features
        }
        None => &mut class.features,
    };
    let before = features.len();
    features.retain(|feature| feature.key.as_str() != key);
    assert_eq!(features.len(), before - 1, "no feature {key}");
    log.push(format!("{key}: feature removed"));
}

// ---- repeats and weapons --------------------------------------------------------------------------------------

/// Every parameterised feat repeats once per value (27-1, the user's decision).
fn repeats(data: &mut GameData, log: &mut Log) {
    for feat in &mut data.feats {
        if feat.parameter.is_some() && feat.repeat != Repeat::PerValue {
            assert_eq!(feat.repeat, Repeat::Once, "{}", feat.key);
            feat.repeat = Repeat::PerValue;
            log.push(format!("{}: repeats once per value", feat.key));
        }
    }
}

/// The natural attacks the guide names, with the phrase on the shifter page that names each (the shifter's forms
/// list them all).
const NATURAL_ATTACKS: [(&str, &str); 9] = [
    ("Bite", "a 1d8 bite attack"),
    ("Claw", "two 1d6 claw attacks"),
    ("Gore", "a 1d8 gore attack"),
    ("Hoof", "two 1d4 hoof attacks"),
    ("Slam", "two 1d8 slam attacks"),
    ("Spike", "Spike attacks become more powerful"),
    ("Tail", "a tail attack"),
    ("Talon", "two 1d8 talon attacks"),
    ("Wing", "two wing attacks"),
];

fn weapons(guide: &Guide, data: &mut GameData, log: &mut Log) {
    // "Exotic: Dueling Sword, Elven Curve Blade, Estoc, Kama, Sai, Swatooth Saber." (Weapon Finesse)
    says(guide, "feats", "weapon-finesse", "Swatooth Saber");
    let saber = data
        .weapons
        .iter_mut()
        .find(|weapon| weapon.key.as_str() == "sawtooth-saber")
        .expect("the Sawtooth Saber");
    assert_eq!(saber.proficiency, None);
    saber.proficiency = Some(WeaponProficiency::Exotic);
    log.push("sawtooth-saber: exotic".into());

    let before = data.weapons.len();
    data.weapons
        .retain(|weapon| weapon.key.as_str() != "natural-weapons");
    assert_eq!(data.weapons.len(), before - 1, "no natural-weapons");
    let shifter = page(guide, "shifter");
    let text: String = shifter
        .root
        .walk()
        .iter()
        .map(|section| section.text())
        .collect();
    let lower = text.to_lowercase();
    for (name, phrase) in NATURAL_ATTACKS {
        assert!(
            lower.contains(&phrase.to_lowercase()),
            "the shifter page doesn't name {name}: {phrase:?}"
        );
        add_weapon(data, name, log);
    }
    // "Weapon Focus (Heavy Crossbow) will add +1 to the attack roll but Weapon Focus (Kinetic Blast) will not."
    says(
        guide,
        "kineticist",
        "charged-ammunition",
        "Weapon Focus (Kinetic Blast)",
    );
    add_weapon(data, "Kinetic Blast", log);
    // "Bombs have a range of 30' and benefit from feats like Weapon Focus and Point-Blank Shot."
    says(
        guide,
        "alchemist",
        "bomb",
        "benefit from feats like Weapon Focus",
    );
    add_weapon(data, "Bomb", log);
}

fn add_weapon(data: &mut GameData, name: &str, log: &mut Log) {
    let key = WeaponKey::new(crate::names::slug(name));
    assert!(
        !data.weapons.iter().any(|weapon| weapon.key == key),
        "{key} exists"
    );
    log.push(format!("{key}: new weapon"));
    data.weapons.push(Weapon {
        key,
        name: name.to_owned(),
        proficiency: None,
    });
}

// ---- prerequisites on the pick's value ------------------------------------------------------------------------

/// "Armor Proficiency (Light. Medium or Heavy)": the one of the armour type taken (27-4).
fn armor_focus(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "feats",
        "armor-focus",
        "Armor Proficiency (Light. Medium or Heavy)",
    );
    let proficiency = |armour: &str| feat(&format!("armor-proficiency-{armour}-armor"));
    let any = Prerequisite::Any(
        ["heavy", "light", "medium"]
            .iter()
            .map(|armour| Prerequisite::Feat(proficiency(armour)))
            .collect(),
    );
    let focus = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "armor-focus")
        .expect("Armor Focus");
    // The hand-fixed data names the three proficiencies; an extraction leaves the line out (its grammar resolves no
    // feat by that name) and keeps the BAB alone.
    let mut children = match focus.prerequisite.take() {
        Some(Prerequisite::All(mut children)) => {
            let at = children
                .iter()
                .position(|child| child == &any)
                .unwrap_or_else(|| panic!("Armor Focus doesn't need {any:?}"));
            children.remove(at);
            children
        }
        Some(bab @ Prerequisite::BaseAttackBonus(1)) => vec![bab],
        other => panic!("Armor Focus's prerequisite: {other:?}"),
    };
    for (armour, value) in [
        ("light", ArmourType::Light),
        ("medium", ArmourType::Medium),
        ("heavy", ArmourType::Heavy),
    ] {
        children.push(Prerequisite::FeatIfValue {
            value: Parameter::ArmourType(value),
            feat: proficiency(armour),
        });
    }
    children.sort();
    focus.prerequisite = Some(Prerequisite::All(children));
    log.push("armor-focus: the proficiency of its armour type".into());
}

/// A parameterised feat or mythic feat needing its base feat for the pick's value instead of any.
fn same_value(prerequisite: &mut Option<Prerequisite>, base: &str, at: &str) {
    let leaf = |p: &Prerequisite| matches!(p, Prerequisite::Feat(key) if key.as_str() == base);
    let replaced = match prerequisite {
        Some(p) if leaf(p) => {
            *p = Prerequisite::FeatForSameValue(feat(base));
            true
        }
        Some(Prerequisite::All(children)) => children
            .iter_mut()
            .find(|child| leaf(child))
            .map(|child| *child = Prerequisite::FeatForSameValue(feat(base)))
            .is_some(),
        _ => false,
    };
    assert!(replaced, "{at} doesn't need {base}: {prerequisite:?}");
}

/// The mythic feats that pick a value (27-5) and the Trickster's critical chain ("the selected weapon").
fn mythic_values(guide: &Guide, data: &mut GameData, log: &mut Log) {
    use ParameterKind as K;
    let mythic: [(&str, K, Option<&str>, &str); 5] = [
        (
            "improved-critical-mythic",
            K::Weapon,
            Some("improved-critical"),
            "The critical multiplier of your chosen weapon",
        ),
        (
            "spell-focus-mythic",
            K::SpellSchool,
            Some("spell-focus"),
            "Select a school in which you have taken a Spell Focus",
        ),
        (
            "skill-focus-mythic",
            K::Skill,
            Some("skill-focus"),
            "Select a skill in which you have taken a Skill Focus",
        ),
        (
            "expanded-arsenal",
            K::SpellSchool,
            None,
            "Select one school of magic",
        ),
        (
            "school-mastery",
            K::SpellSchool,
            None,
            "Select one school of magic",
        ),
    ];
    for (key, kind, base, phrase) in mythic {
        says(guide, "mythic-paths", key, phrase);
        let feat = data
            .mythic_feats
            .iter_mut()
            .find(|feat| feat.key.as_str() == key)
            .unwrap_or_else(|| panic!("no mythic feat {key}"));
        assert_eq!((feat.parameter, feat.repeat), (None, Repeat::Once), "{key}");
        feat.parameter = Some(kind);
        feat.repeat = Repeat::PerValue;
        if let Some(base) = base {
            same_value(&mut feat.prerequisite, base, key);
        } else {
            assert_eq!(feat.prerequisite, None, "{key}");
        }
        log.push(format!("{key}: takes a {kind:?}, once per value"));
    }
    // "You no longer suffer your racial ability to an ability score."
    says(
        guide,
        "mythic-paths",
        "destiny-beyond-birth",
        "You no longer suffer your racial",
    );
    let destiny = data
        .mythic_feats
        .iter_mut()
        .find(|feat| feat.key == MythicFeatKey::new("destiny-beyond-birth"))
        .expect("Destiny Beyond Birth");
    destiny.removes_racial_penalty = true;
    log.push("destiny-beyond-birth: removes the racial penalty".into());

    // "When using the selected weapon, its critical threat range is increased by +1." Each needs the one before in
    // the same weapon.
    for (key, base) in [
        ("improved-improved-critical", "improved-critical"),
        (
            "improved-improved-improved-critical",
            "improved-improved-critical",
        ),
        (
            "improved-improved-improved-critical-improved",
            "improved-improved-improved-critical",
        ),
    ] {
        says(guide, "trickster", key, "When using the selected weapon");
        let feat = data
            .feats
            .iter_mut()
            .find(|feat| feat.key.as_str() == key)
            .unwrap_or_else(|| panic!("no feat {key}"));
        assert_eq!(feat.parameter, None, "{key}");
        feat.parameter = Some(K::Weapon);
        feat.repeat = Repeat::PerValue;
        same_value(&mut feat.prerequisite, base, key);
        log.push(format!("{key}: takes a weapon, once per value"));
    }
}

// ---- grants with values ---------------------------------------------------------------------------------------

fn granted(key: &str, value: Parameter) -> FeatValue {
    FeatValue {
        feat: feat(key),
        value,
    }
}

/// Feature-encoded grants of a parameterised feat with a fixed value become feats granted for that value (27-6).
fn fixed_values(guide: &Guide, data: &mut GameData, log: &mut Log) {
    // Mantis Zealot: "At 1st level, a Mantis Zealot gains Weapon Focus (Sawtooth Saber) as a bonus feat. At 4th
    // level, he gains Weapon Specialization. At 8th level, he gains Greater Weapon Focus. Finally, at 12th level, he
    // gains Greater Weapon Specialization."
    says(
        guide,
        "warpriest",
        "saber-fighting",
        "At 1st level, a Mantis Zealot gains Weapon Focus (Sawtooth Saber) as a bonus feat. At 4th level, he \
         gains Weapon Specialization. At 8th level, he gains Greater Weapon Focus. Finally, at 12th level, he gains \
         Greater Weapon Specialization.",
    );
    let zealot = "warpriest/mantis-zealot";
    for (level, key) in [
        (1, "weapon-focus"),
        (4, "weapon-specialization"),
        (8, "greater-weapon-focus"),
        (12, "greater-weapon-specialization"),
    ] {
        let row = archetype_row_mut(data, "warpriest", zealot, level);
        take_feature(
            &mut row.features,
            "warpriest/mantis-zealot/saber-fighting",
            zealot,
        );
        row.feats_for.insert(granted(key, weapon("sawtooth-saber")));
        log.push(format!("{zealot} {level}: {key} (sawtooth-saber)"));
    }
    drop_feature(
        data,
        "warpriest",
        Some(zealot),
        "warpriest/mantis-zealot/saber-fighting",
        log,
    );
    // "Zealots are proficient with all simple and martial weapons, the Sawtooth Saber exotic weapon and light
    // armour." The row grants the rest already.
    says(
        guide,
        "warpriest",
        "mantis-zealot-proficiencies",
        "proficient with all simple and martial weapons, the Sawtooth Saber exotic weapon and light armour",
    );
    let row = archetype_row_mut(data, "warpriest", zealot, 1);
    for key in [
        "armor-proficiency-light-armor",
        "martial-weapons-proficiency",
        "simple-weapon-proficiency",
    ] {
        assert!(row.feats.contains(&feat(key)), "{zealot} 1 lacks {key}");
    }
    take_feature(
        &mut row.features,
        "warpriest/mantis-zealot/mantis-zealot-proficiencies",
        zealot,
    );
    row.feats_for.insert(granted(
        "exotic-weapon-proficiency",
        weapon("sawtooth-saber"),
    ));
    log.push(format!(
        "{zealot} 1: exotic-weapon-proficiency (sawtooth-saber)"
    ));
    drop_feature(
        data,
        "warpriest",
        Some(zealot),
        "warpriest/mantis-zealot/mantis-zealot-proficiencies",
        log,
    );

    // The Chirurgeon's table: "5 | Feat | - | Skill Focus - Lore (Religion) | - |"; the Enlightened Philosopher's:
    // "1 | Feat | Oracle's Curse | Revelation | Skill Focus - Knowledge World |".
    for (class, archetype, level, cell, feature, skill) in [
        (
            "alchemist",
            "chirurgeon",
            5,
            "Skill Focus - Lore (Religion)",
            "alchemist/chirurgeon/skill-focus-lore-religion",
            "lore-religion",
        ),
        (
            "oracle",
            "enlightened-philosopher",
            1,
            "Skill Focus - Knowledge World",
            "oracle/enlightened-philosopher/skill-focus-knowledge-world",
            "knowledge-world",
        ),
    ] {
        cell_at(guide, class, archetype, &level.to_string(), cell);
        let key = format!("{class}/{archetype}");
        let row = archetype_row_mut(data, class, &key, level);
        take_feature(&mut row.features, feature, &key);
        row.feats_for.insert(granted(
            "skill-focus",
            Parameter::Skill(SkillKey::new(skill)),
        ));
        log.push(format!("{key} {level}: skill-focus ({skill})"));
        drop_feature(data, class, Some(&key), feature, log);
    }
}

/// "At 2nd level, a Zen Archer gains Weapon Focus with Shortbows or Longbows as a bonus feat. At 6th level, she gains
/// Weapon Specialization with the same weapon type." A pick of its one option, limited to the two bows.
fn zen_archer(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "monk",
        "way-of-the-bow",
        "At 2nd level, a Zen Archer gains Weapon Focus with Shortbows or Longbows as a bonus feat. At 6th level, she \
         gains Weapon Specialization with the same weapon type.",
    );
    let monk = page(guide, "monk");
    let archer = "monk/zen-archer";
    let way = format!("{archer}/way-of-the-bow");
    let mut bow = option(
        &format!("{way}/bow"),
        "Bow",
        monk.guide("way-of-the-bow"),
        vec![OptionLevelRow {
            level: 6,
            chosen_feats: BTreeSet::from([feat("weapon-specialization")]),
            ..OptionLevelRow::default()
        }],
    );
    bow.chosen_feat = Some(feat("weapon-focus"));
    bow.values = BTreeSet::from([weapon("long-bow"), weapon("short-bow")]);
    let mut way_of_the_bow = selection(
        &way,
        "Way of the Bow",
        monk.guide("way-of-the-bow"),
        &["monk"],
    );
    way_of_the_bow.options.push(bow);
    archetype_mut(data, "monk", archer)
        .selections
        .push(way_of_the_bow);
    let row = archetype_row_mut(data, "monk", archer, 2);
    take_feature(&mut row.features, &way, archer);
    row.selections.push(FeatureSelectionKey::new(&way));
    log.push(format!("{archer} 2: {way} (a longbow or a shortbow)"));
    drop_feature(data, "monk", Some(archer), &way, log);
}

/// Battle Master: "At 8th level, the Shaman gains Weapon Specialization in a weapon of choice. At 16th level, the
/// Shaman gains Greater Weapon Focus in the chosen weapon." The hex's pick names the weapon; its rows come by
/// shaman level, so the hex selection gets option levels.
fn battle_master(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "shaman",
        "battle-master",
        "At 8th level, the Shaman gains Weapon Specialization in a weapon of choice. At 16th level, the Shaman gains \
         Greater Weapon Focus in the chosen weapon.",
    );
    let shaman = class_mut(data, "shaman");
    let hex = shaman
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == "shaman/hex")
        .expect("the shaman's hex");
    assert!(hex.option_levels.is_empty());
    hex.option_levels.insert(ClassKey::new("shaman"));
    let master = hex
        .options
        .iter_mut()
        .find(|option| option.key.as_str() == "shaman/hex/battle-master")
        .expect("Battle Master");
    assert!(master.levels.is_empty() && master.chosen_feat.is_none());
    master.parameter = Some(ParameterKind::Weapon);
    for (level, key) in [(8, "weapon-specialization"), (16, "greater-weapon-focus")] {
        master.levels.push(OptionLevelRow {
            level,
            chosen_feats: BTreeSet::from([feat(key)]),
            ..OptionLevelRow::default()
        });
    }
    log.push("shaman/hex: levelled by shaman".into());
    log.push("shaman/hex/battle-master: a weapon; weapon-specialization at 8, greater-weapon-focus at 16".into());
}

/// "At 1st level, a sword saint gains Weapon Focus with his chosen weapon as a bonus feat": a one-feat bonus list,
/// like the warpriest's.
fn sword_saint(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "magus",
        "chosen-weapon",
        "At 1st level, a sword saint gains Weapon Focus with his chosen weapon as a bonus feat.",
    );
    let saint = "magus/sword-saint";
    let chosen = format!("{saint}/chosen-weapon");
    let list_key = rules::FeatListKey::new(&chosen);
    archetype_mut(data, "magus", saint).feat_lists.push(list(
        &chosen,
        "Chosen Weapon",
        page(guide, "magus").guide("chosen-weapon"),
        true,
        BTreeSet::from([feat("weapon-focus")]),
    ));
    let row = archetype_row_mut(data, "magus", saint, 1);
    take_feature(&mut row.features, &chosen, saint);
    row.feat_slots.push(FeatSlot::OneOf { list: list_key });
    log.push(format!(
        "{saint} 1: {chosen} (weapon-focus of the player's choosing)"
    ));
    drop_feature(data, "magus", Some(saint), &chosen, log);
}

/// "Adaptability: Half-elves receive Skill Focus as a bonus feat at 1st level": the race's own one-feat list, in
/// place of the general slot the extraction gave it.
fn half_elf(guide: &Guide, data: &mut GameData, log: &mut Log) {
    let creation = page(guide, "character-creation");
    let text: String = section(creation, "half-elf")
        .tables()
        .flatten()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        text.contains("Adaptability: Half-elves receive Skill Focus as a bonus feat at 1st level"),
        "half-elf: {text:?}"
    );
    let race = data
        .races
        .iter_mut()
        .find(|race| race.key.as_str() == "half-elf")
        .expect("the half-elf");
    assert_eq!(race.feat_slots, [FeatSlot::General]);
    let key = "half-elf/adaptability";
    race.feat_lists.push(list(
        key,
        "Adaptability",
        creation.guide("half-elf"),
        true,
        BTreeSet::from([feat("skill-focus")]),
    ));
    race.feat_slots = vec![FeatSlot::OneOf {
        list: rules::FeatListKey::new(key),
    }];
    log.push(format!(
        "half-elf: {key} (skill-focus of the player's choosing) for the general slot"
    ));
}

// ---- the slayer's combat style feats --------------------------------------------------------------------------

/// "Although the slayer can choose from Combat Style Feats, he can only do so three times in total and at certain
/// levels: 2 or 4, 6 or 8 and 10 or greater." Three slayer talents, one per window, each a bonus feat at once from
/// the ranger styles' lists of its stage put together (the slayer picks no style).
fn slayer_combat_styles(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "slayer",
        "slayer-camouflage",
        "he can only do so three times in total and at certain levels: 2 or 4, 6 or 8 and 10 or greater",
    );
    says(
        guide,
        "slayer",
        "slayer-talents",
        "Alternatively, he may choose from a Ranger's Combat Style Feats",
    );
    let ranger = class_mut(data, "ranger");
    let styles = ranger
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == "ranger/combat-style-feat")
        .expect("the ranger's combat styles");
    let lists: Vec<FeatList> = ranger
        .feat_lists
        .iter()
        .chain(ranger.archetypes.iter().flat_map(|a| &a.feat_lists))
        .cloned()
        .collect();
    let stage = |level: u8| -> BTreeSet<FeatKey> {
        let mut feats = BTreeSet::new();
        for style in &styles.options {
            let list = style
                .levels
                .iter()
                .find(|row| row.level == level)
                .and_then(|row| row.feat_list.as_ref())
                .unwrap_or_else(|| panic!("{} has no list at {level}", style.key));
            let list = lists
                .iter()
                .find(|l| &l.key == list)
                .unwrap_or_else(|| panic!("no list {list}"));
            assert!(list.ignores_prerequisites, "{}", list.key);
            feats.extend(list.feats.iter().cloned());
        }
        feats
    };
    assert_eq!(styles.options.len(), 5);
    let stages = [(2, stage(2)), (6, stage(6)), (10, stage(10))];
    let slayer_page = page(guide, "slayer");
    let slayer = class_mut(data, "slayer");
    let level_at_least = |level: u8| Prerequisite::ClassLevel {
        class: ClassKey::new("slayer"),
        level,
    };
    let below = |level: u8| Prerequisite::Not(Box::new(level_at_least(level)));
    for (level, feats) in stages {
        let key = format!("slayer/combat-style-feats-{level}");
        slayer.feat_lists.push(list(
            &key,
            &format!("Combat Style Feats (from level {level})"),
            slayer_page.guide("slayer-talents"),
            true,
            feats,
        ));
        let (name, prerequisite) = match level {
            2 => ("Combat Style Feat (2nd or 4th level)", below(6)),
            6 => (
                "Combat Style Feat (6th or 8th level)",
                Prerequisite::All(vec![level_at_least(6), below(10)]),
            ),
            _ => (
                "Combat Style Feat (10th level or later)",
                level_at_least(10),
            ),
        };
        let mut talent = option(
            &format!("slayer/slayer-talents/combat-style-feat-{level}"),
            name,
            slayer_page.guide("slayer-talents"),
            vec![OptionLevelRow {
                level: 1,
                feat_slots: vec![FeatSlot::OneOf {
                    list: rules::FeatListKey::new(&key),
                }],
                ..OptionLevelRow::default()
            }],
        );
        talent.prerequisite = Some(prerequisite);
        let talents = slayer
            .selections
            .iter_mut()
            .find(|selection| selection.key.as_str() == "slayer/slayer-talents")
            .expect("the slayer's talents");
        talents.options.push(talent);
        log.push(format!(
            "slayer/slayer-talents: combat-style-feat-{level} ({key})"
        ));
    }
    let talents = slayer
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == "slayer/slayer-talents")
        .expect("the slayer's talents");
    assert!(talents.option_levels.is_empty());
    talents.option_levels.insert(ClassKey::new("slayer"));
    log.push("slayer/slayer-talents: levelled by slayer".into());
}

// ---- mythic rules ---------------------------------------------------------------------------------------------

/// "Your base attack bonus is at least 15. All skills are class skills and your rank in any skill is at least your
/// mythic rank." (26-3: for prerequisites.)
fn perfect_mind(guide: &Guide, data: &mut GameData, log: &mut Log) {
    for phrase in [
        "Your base attack bonus is at least 15.",
        "your rank in any skill is at least your mythic rank",
    ] {
        says(guide, "gold-dragon", "perfect-mind", phrase);
    }
    let mind = data
        .mythic_paths
        .iter_mut()
        .flat_map(|path| &mut path.features)
        .find(|feature| feature.key.as_str() == "gold-dragon/perfect-mind")
        .expect("Perfect Mind");
    mind.bab_floor = Some(15);
    mind.skill_ranks_at_least_mythic_rank = true;
    log.push("gold-dragon/perfect-mind: BAB 15, skill ranks at the Mythic rank".into());
}

// ---- class rules ----------------------------------------------------------------------------------------------

/// "Her alignment must be Neutral or Evil": true neutral or any evil, as the Assassin's line reads (28-5).
fn razmiri_infiltrator(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "arcanist",
        "razmiri-infiltrator",
        "Her alignment must be Neutral or Evil",
    );
    let way = class_mut(data, "arcanist")
        .archetypes
        .iter_mut()
        .flat_map(|archetype| &mut archetype.selections)
        .flat_map(|selection| &mut selection.options)
        .find(|option| {
            option.key.as_str()
                == "arcanist/magic-deceiver/ways-of-the-deceiver/razmiri-infiltrator"
        })
        .expect("the Razmiri Infiltrator");
    assert_eq!(way.prerequisite, None);
    way.prerequisite = Some(Prerequisite::Any(
        [
            Alignment::TrueNeutral,
            Alignment::LawfulEvil,
            Alignment::NeutralEvil,
            Alignment::ChaoticEvil,
        ]
        .into_iter()
        .map(Prerequisite::Alignment)
        .collect(),
    ));
    log.push("arcanist/magic-deceiver/ways-of-the-deceiver/razmiri-infiltrator: true neutral or any evil".into());
}

/// The user's skill facts (2026-11-01) where they differ from the extraction: the Tortured Crusader's own skill
/// points and class skills (its section on the paladin page says the same), the Arcane Trickster without Use Magic
/// Device, the Dragon Disciple with Knowledge (World).
fn skills(guide: &Guide, data: &mut GameData, log: &mut Log) {
    const CRUSADER_SKILLS: [&str; 7] = [
        "athletics",
        "knowledge-arcana",
        "knowledge-world",
        "lore-nature",
        "lore-religion",
        "perception",
        "persuasion",
    ];
    for phrase in [
        "Class skills: Athletics, Knowledge (Arcana), Knowledge (World), Lore (Nature), Lore (Religion), \
         Perception, Persuasion",
        "3 skill points per level.",
    ] {
        says(guide, "paladin", "tortured-crusader", phrase);
    }
    let crusader = archetype_mut(data, "paladin", "paladin/tortured-crusader");
    assert_eq!(
        (crusader.skill_points, &crusader.class_skills),
        (None, &None)
    );
    crusader.skill_points = Some(3);
    crusader.class_skills = Some(CRUSADER_SKILLS.into_iter().map(SkillKey::new).collect());
    log.push("paladin/tortured-crusader: 3 skill points, its own class skills".into());

    let trickster = class_mut(data, "arcane-trickster");
    assert!(
        trickster
            .class_skills
            .remove(&SkillKey::new("use-magic-device"))
    );
    log.push("arcane-trickster: no use-magic-device".into());
    let disciple = class_mut(data, "dragon-disciple");
    assert!(
        disciple
            .class_skills
            .insert(SkillKey::new("knowledge-world"))
    );
    log.push("dragon-disciple: knowledge-world".into());
}
