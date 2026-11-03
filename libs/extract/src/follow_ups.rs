//! Ticket 32: the user's decisions on tickets 26–28's open questions (2026-11-01, "all recommended") and the skill
//! facts the user gave (`research/skills-from-user.md`), each built where its owner is ([`provide`]).
//!
//! - **Repeats** (27-1): every parameterised feat repeats once per value (in [`crate::parameters`]' feat shapes).
//! - **Weapons** (27-2): the Sawtooth Saber is exotic; natural attacks are one value per attack the guide names
//!   (bite, claw, gore, hoof, slam, spike, tail, talon, wing) instead of "natural weapons"; Kinetic Blast and Bomb
//!   are values ([`weapons`], where the weapons are built).
//! - **Armor Focus** (27-4) needs the armour proficiency of the armour type it's taken for (`FeatIfValue`), stated
//!   beside its lines.
//! - **Values on mythic feats** (27-5): Improved Critical, Spell Focus and Skill Focus (Mythic) need their base
//!   feat in the same value; Expanded Arsenal and School Mastery take a school; the Trickster's Improved Improved
//!   Critical chain takes "the selected weapon" (in [`crate::parameters`]' feat shapes, mythic feat shapes and
//!   prerequisite rules: the same-value rewrite is on the resolved prerequisites; the guide's sentences are asserted
//!   here, [`values`]).
//! - **Fixed values** (27-6): the Mantis Zealot's four Sawtooth Saber feats and its Exotic Weapon Proficiency, the
//!   Chirurgeon's and the Enlightened Philosopher's Skill Focus are feats granted for one value (`feats_for`): their
//!   cells are those grants.
//! - **Restricted choices** (27-6): the Zen Archer's Way of the Bow is a pick of a longbow or a shortbow (Weapon
//!   Focus at 2, Weapon Specialization in the same bow at 6), the shaman's Battle Master names a weapon for its
//!   feats at 8 and 16, the Sword Saint's Chosen Weapon and the half-elf's Adaptability are one-feat bonus lists.
//! - **The slayer's combat style feats** (27-8, with tickets 33 and 35's decisions): three slayer talents, one for
//!   each window (2 or 4, 6 or 8, 10+), each picking one of the ranger's styles and a feat from its list of the
//!   window's stage.
//! - **Perfect Mind** (26-3): BAB 15 and skill ranks at the Mythic rank, for prerequisites. **Destiny Beyond Birth**
//!   (26-4) removes the race's ability penalty, where the mythic feats are built.
//! - **Razmiri Infiltrator** (28-5): true neutral or any evil, the Assassin's reading: on the Magic Deceiver's ways.
//! - **Skill facts from the user**: the Tortured Crusader's 3 skill points and class skills (the guide's paladin page
//!   agrees), the Arcane Trickster without Use Magic Device, the Dragon Disciple with Knowledge (World).
//!
//! Each rule asserts the guide phrase it rests on (the user's skill facts excepted) when the providers are read;
//! [`check`] asserts what only the finished data shows.

use std::collections::BTreeSet;

use rules::{
    Alignment, ArmourType, Class, ClassKey, DataAt, FeatKey, FeatSlot, FeatValue, FeatureKey,
    FeatureSelectionKey, GameData, OptionLevelRow, Parameter, ParameterKind, Prerequisite,
    SkillKey, Weapon, WeaponKey, WeaponProficiency,
};

use crate::guide::Guide;
use crate::option_levels::{list, option, page, section, selection};
use crate::picks::{ArchetypeSkills, FeatureFact, Pick, Picks, Rule};
use crate::report::Settled;

/// The report item this module settles: the Armor Focus line the grammar can't resolve.
pub fn settles() -> Vec<Settled> {
    vec![Settled::is(
        "feats#armor-focus",
        "Armor Proficiency (Light. Medium or Heavy)",
    )]
}

/// Whether an extraction's report item is one this module settles ([`settles`]).
pub fn settled(at: &str, text: &str) -> bool {
    settles().iter().any(|entry| entry.matches(at, text))
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

/// Takes a feature off a row's grants, asserting it was there.
pub(crate) fn take_feature(features: &mut Vec<FeatureKey>, key: &str, at: &str) {
    let before = features.len();
    features.retain(|feature| feature.as_str() != key);
    assert_eq!(features.len(), before - 1, "{at}: no single {key}");
}

// ---- weapons --------------------------------------------------------------------------------------------------

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

/// The weapons as the user decided (27-2), where they're built: the Sawtooth Saber is exotic, and the natural
/// attacks the guide names, Kinetic Blast and Bomb are values in place of "natural weapons".
pub(crate) fn weapons(guide: &Guide, weapons: &mut Vec<Weapon>) {
    // "Exotic: Dueling Sword, Elven Curve Blade, Estoc, Kama, Sai, Swatooth Saber." (Weapon Finesse)
    says(guide, "feats", "weapon-finesse", "Swatooth Saber");
    let saber = weapons
        .iter_mut()
        .find(|weapon| weapon.key.as_str() == "sawtooth-saber")
        .expect("the Sawtooth Saber");
    assert_eq!(saber.proficiency, None);
    saber.proficiency = Some(WeaponProficiency::Exotic);

    let before = weapons.len();
    weapons.retain(|weapon| weapon.key.as_str() != "natural-weapons");
    assert_eq!(weapons.len(), before - 1, "no natural-weapons");
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
        add_weapon(weapons, name);
    }
    // "Weapon Focus (Heavy Crossbow) will add +1 to the attack roll but Weapon Focus (Kinetic Blast) will not."
    says(
        guide,
        "kineticist",
        "charged-ammunition",
        "Weapon Focus (Kinetic Blast)",
    );
    add_weapon(weapons, "Kinetic Blast");
    // "Bombs have a range of 30' and benefit from feats like Weapon Focus and Point-Blank Shot."
    says(
        guide,
        "alchemist",
        "bomb",
        "benefit from feats like Weapon Focus",
    );
    add_weapon(weapons, "Bomb");
}

fn add_weapon(weapons: &mut Vec<Weapon>, name: &str) {
    let key = WeaponKey::new(crate::names::slug(name));
    assert!(
        !weapons.iter().any(|weapon| weapon.key == key),
        "{key} exists"
    );
    weapons.push(Weapon {
        key,
        name: name.to_owned(),
        proficiency: None,
    });
}

// ---- prerequisites on the pick's value ------------------------------------------------------------------------

/// "Armor Proficiency (Light. Medium or Heavy)": the one of the armour type taken (27-4). The line names no feat the
/// grammar resolves, so an extraction leaves it out (and reports it: [`settled`]); the condition is stated beside
/// the feat's other line (its BAB).
fn armor_focus(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "feats",
        "armor-focus",
        "Armor Proficiency (Light. Medium or Heavy)",
    );
    let at = DataAt::Feat(feat("armor-focus"));
    for (armour, value) in [
        ("light", ArmourType::Light),
        ("medium", ArmourType::Medium),
        ("heavy", ArmourType::Heavy),
    ] {
        picks.stated(
            at.clone(),
            Prerequisite::FeatIfValue {
                value: Parameter::ArmourType(value),
                feat: feat(&format!("armor-proficiency-{armour}-armor")),
            },
        );
    }
}

/// The guide's sentences under the values [`crate::parameters`]' tables give the mythic feats and the Trickster's
/// critical chain (27-5): what each mythic feat is taken for, and the chain's "selected weapon".
fn values(guide: &Guide) {
    for (key, phrase) in [
        (
            "improved-critical-mythic",
            "The critical multiplier of your chosen weapon",
        ),
        (
            "spell-focus-mythic",
            "Select a school in which you have taken a Spell Focus",
        ),
        (
            "skill-focus-mythic",
            "Select a skill in which you have taken a Skill Focus",
        ),
        ("expanded-arsenal", "Select one school of magic"),
        ("school-mastery", "Select one school of magic"),
    ] {
        says(guide, "mythic-paths", key, phrase);
    }
    // "When using the selected weapon, its critical threat range is increased by +1."
    for key in [
        "improved-improved-critical",
        "improved-improved-improved-critical",
        "improved-improved-improved-critical-improved",
    ] {
        says(guide, "trickster", key, "When using the selected weapon");
    }
}

/// "You no longer suffer your racial ability to an ability score." (26-4), where the mythic feats are built.
fn destiny_beyond_birth(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "mythic-paths",
        "destiny-beyond-birth",
        "You no longer suffer your racial",
    );
    picks.removes_racial_penalty("destiny-beyond-birth");
}

// ---- grants with values ---------------------------------------------------------------------------------------

fn granted(key: &str, value: Parameter) -> FeatValue {
    FeatValue {
        feat: feat(key),
        value,
    }
}

/// Grants of a parameterised feat with a fixed value (27-6): the cells that say them are feats granted for that
/// value.
fn fixed_values(guide: &Guide, picks: &mut Picks) {
    let cell = |picks: &mut Picks,
                table: &str,
                feature: &str,
                level: u8,
                feats: &[&str],
                feats_for: Vec<FeatValue>| {
        picks.rule(Rule {
            feature: FeatureKey::new(feature),
            table: Some(table.to_owned()),
            cell: None,
            levels: Some(vec![level]),
            pick: Pick::FeatsFor {
                feats: feats.iter().map(|key| feat(key)).collect(),
                feats_for: feats_for.into_iter().collect(),
            },
        });
    };
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
        cell(
            picks,
            zealot,
            "warpriest/mantis-zealot/saber-fighting",
            level,
            &[],
            vec![granted(key, weapon("sawtooth-saber"))],
        );
    }
    // "Zealots are proficient with all simple and martial weapons, the Sawtooth Saber exotic weapon and light
    // armour": the feats, and Exotic Weapon Proficiency for the saber (before the proficiencies' own rule, which
    // would keep the feature for the saber).
    says(
        guide,
        "warpriest",
        "mantis-zealot-proficiencies",
        "proficient with all simple and martial weapons, the Sawtooth Saber exotic weapon and light armour",
    );
    cell(
        picks,
        zealot,
        "warpriest/mantis-zealot/mantis-zealot-proficiencies",
        1,
        &[
            "armor-proficiency-light-armor",
            "martial-weapons-proficiency",
            "simple-weapon-proficiency",
        ],
        vec![granted(
            "exotic-weapon-proficiency",
            weapon("sawtooth-saber"),
        )],
    );

    // The Chirurgeon's table: "5 | Feat | - | Skill Focus - Lore (Religion) | - |"; the Enlightened Philosopher's:
    // "1 | Feat | Oracle's Curse | Revelation | Skill Focus - Knowledge World |".
    for (class, archetype, level, text, feature, skill) in [
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
        cell_at(guide, class, archetype, &level.to_string(), text);
        cell(
            picks,
            &format!("{class}/{archetype}"),
            feature,
            level,
            &[],
            vec![granted(
                "skill-focus",
                Parameter::Skill(SkillKey::new(skill)),
            )],
        );
    }
}

/// "At 2nd level, a Zen Archer gains Weapon Focus with Shortbows or Longbows as a bonus feat. At 6th level, she gains
/// Weapon Specialization with the same weapon type." A pick of its one option, limited to the two bows: its row-2
/// cell is the pick.
fn zen_archer(guide: &Guide, picks: &mut Picks) {
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
    picks.selection(archer, way_of_the_bow);
    picks.unnamed(&way);
    picks.rule(Rule {
        feature: FeatureKey::new(&way),
        table: Some(archer.into()),
        cell: None,
        levels: Some(vec![2]),
        pick: Pick::Selection(FeatureSelectionKey::new(&way)),
    });
}

/// Battle Master: "At 8th level, the Shaman gains Weapon Specialization in a weapon of choice. At 16th level, the
/// Shaman gains Greater Weapon Focus in the chosen weapon." The hex's pick names the weapon; its rows come by
/// shaman level, so the hex selection gets option levels.
fn battle_master(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "shaman",
        "battle-master",
        "At 8th level, the Shaman gains Weapon Specialization in a weapon of choice. At 16th level, the Shaman gains \
         Greater Weapon Focus in the chosen weapon.",
    );
    picks.option_levels("shaman/hex", &["shaman"]);
    let master = "shaman/hex/battle-master";
    picks.option_parameter(master, ParameterKind::Weapon);
    picks.option_rows(
        master,
        [(8, "weapon-specialization"), (16, "greater-weapon-focus")]
            .into_iter()
            .map(|(level, key)| OptionLevelRow {
                level,
                chosen_feats: BTreeSet::from([feat(key)]),
                ..OptionLevelRow::default()
            })
            .collect(),
    );
}

/// "At 1st level, a sword saint gains Weapon Focus with his chosen weapon as a bonus feat": a one-feat bonus list,
/// like the warpriest's, whose slot its row-1 cell is.
fn sword_saint(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "magus",
        "chosen-weapon",
        "At 1st level, a sword saint gains Weapon Focus with his chosen weapon as a bonus feat.",
    );
    let saint = "magus/sword-saint";
    let chosen = format!("{saint}/chosen-weapon");
    picks.feat_list(
        saint,
        list(
            &chosen,
            "Chosen Weapon",
            page(guide, "magus").guide("chosen-weapon"),
            true,
            BTreeSet::from([feat("weapon-focus")]),
        ),
    );
    picks.rule(Rule {
        feature: FeatureKey::new(&chosen),
        table: Some(saint.into()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::Slots(vec![FeatSlot::OneOf {
            list: rules::FeatListKey::new(&chosen),
        }]),
    });
}

/// "Adaptability: Half-elves receive Skill Focus as a bonus feat at 1st level": the race's own one-feat list, in
/// place of the general slot its table's "bonus feat at 1st level" would give.
fn half_elf(guide: &Guide, picks: &mut Picks) {
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
    picks.race_list(
        "half-elf",
        list(
            "half-elf/adaptability",
            "Adaptability",
            creation.guide("half-elf"),
            true,
            BTreeSet::from([feat("skill-focus")]),
        ),
    );
}

// ---- the slayer's combat style feats --------------------------------------------------------------------------

/// "Although the slayer can choose from Combat Style Feats, he can only do so three times in total and at certain
/// levels: 2 or 4, 6 or 8 and 10 or greater" and "Alternatively, he may choose from a Ranger's Combat Style Feats"
/// (27-8, 32-2 and 33-7, the user's decisions): three slayer talents, one per window, each picking one of the ranger's
/// combat styles (only for a character holding none: the first one picks it, as a ranger does) and a bonus feat from
/// that style's list of its window's stage (2, 6, 10), whoever levels the style. The slayer talents and the style
/// are levelled by slayer levels.
fn slayer_combat_styles(guide: &Guide, picks: &mut Picks) {
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
        "Alternatively, he may choose from a Ranger's Combat Style Feats.",
    );
    let styles: BTreeSet<rules::SelectionOptionKey> =
        section(page(guide, "ranger"), "combat-style-feat")
            .children
            .iter()
            .map(|style| {
                rules::SelectionOptionKey::new(format!("ranger/combat-style-feat/{}", style.anchor))
            })
            .collect();
    assert_eq!(styles.len(), 5, "the ranger's five styles");
    let style = FeatureSelectionKey::new("slayer/combat-style");
    picks.selection(
        "slayer",
        rules::FeatureSelection {
            shared_options: styles,
            only_without_option: true,
            ..selection(
                style.as_str(),
                "Combat Style (Slayer)",
                "slayer#slayer-talents".into(),
                &["slayer"],
            )
        },
    );
    picks.unnamed(style.as_str());
    let talents = "slayer/slayer-talents";
    picks.option_levels(talents, &["slayer"]);
    let slayer_page = page(guide, "slayer");
    let level_at_least = |level: u8| Prerequisite::ClassLevel {
        class: ClassKey::new("slayer"),
        level,
    };
    let below = |level: u8| Prerequisite::Not(Box::new(level_at_least(level)));
    for stage in [2, 6, 10] {
        let (name, prerequisite) = match stage {
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
            &format!("{talents}/combat-style-feat-{stage}"),
            name,
            slayer_page.guide("slayer-talents"),
            vec![OptionLevelRow {
                level: 1,
                feat_slots: vec![FeatSlot::OptionList {
                    selection: style.clone(),
                    at_level: Some(stage),
                    label: None,
                }],
                selections: vec![style.clone()],
                ..OptionLevelRow::default()
            }],
        );
        talent.prerequisite = Some(prerequisite);
        picks.option(talents, talent);
    }
}

// ---- mythic rules ---------------------------------------------------------------------------------------------

/// "Your base attack bonus is at least 15. All skills are class skills and your rank in any skill is at least your
/// mythic rank." (26-3: for prerequisites.)
fn perfect_mind(guide: &Guide, picks: &mut Picks) {
    for phrase in [
        "Your base attack bonus is at least 15.",
        "your rank in any skill is at least your mythic rank",
    ] {
        says(guide, "gold-dragon", "perfect-mind", phrase);
    }
    let mind = "gold-dragon/perfect-mind";
    picks.feature_fact(mind, FeatureFact::BabFloor(15));
    picks.feature_fact(mind, FeatureFact::SkillRanksAtLeastMythicRank);
}

// ---- class rules ----------------------------------------------------------------------------------------------

/// "Her alignment must be Neutral or Evil": true neutral or any evil, as the Assassin's line reads (28-5). On the
/// Magic Deceiver's ways before they're given to the class stage ([`crate::option_levels`]).
pub(crate) fn razmiri_infiltrator(guide: &Guide, ways: &mut crate::option_levels::Family) {
    says(
        guide,
        "arcanist",
        "razmiri-infiltrator",
        "Her alignment must be Neutral or Evil",
    );
    let way = ways
        .selection
        .options
        .iter_mut()
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
}

/// The user's skill facts (2026-11-01) where they differ from the extraction: the Tortured Crusader's own skill
/// points and class skills (its section on the paladin page says the same), the Arcane Trickster without Use Magic
/// Device, the Dragon Disciple with Knowledge (World).
fn skills(guide: &Guide, picks: &mut Picks) {
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
    picks.archetype_skills(
        "paladin/tortured-crusader",
        ArchetypeSkills::Own {
            points: 3,
            skills: CRUSADER_SKILLS.into_iter().map(SkillKey::new).collect(),
        },
    );
    picks.class_skills("arcane-trickster", &["use-magic-device"], &[]);
    picks.class_skills("dragon-disciple", &[], &["knowledge-world"]);
}

/// The owner facts of tickets 26–28's decisions, built where their owners are: the skill facts, Perfect Mind, the
/// Zen Archer's bow, the shaman's Battle Master, the Sword Saint's and the half-elf's one-feat lists, Destiny Beyond
/// Birth; and the guide's sentences under the values `parameters`' tables give.
pub(crate) fn provide(guide: &Guide, picks: &mut Picks) {
    armor_focus(guide, picks);
    fixed_values(guide, picks);
    zen_archer(guide, picks);
    battle_master(guide, picks);
    sword_saint(guide, picks);
    half_elf(guide, picks);
    perfect_mind(guide, picks);
    destiny_beyond_birth(guide, picks);
    values(guide);
    skills(guide, picks);
    slayer_combat_styles(guide, picks);
}

/// What [`provide`]'s owners hold once built: the Zen Archer's and the Sword Saint's cells are picks, and Perfect
/// Mind is the gold dragon's feature, and Destiny Beyond Birth a mythic feat. Changes nothing.
pub(crate) fn check(data: &GameData) {
    let defined = |key: &str| {
        data.classes
            .iter()
            .flat_map(|class| class.archetypes.iter().flat_map(|a| &a.features))
            .chain(data.mythic_paths.iter().flat_map(|path| &path.features))
            .find(|feature| feature.key.as_str() == key)
    };
    for key in [
        "monk/zen-archer/way-of-the-bow",
        "magus/sword-saint/chosen-weapon",
    ] {
        assert!(defined(key).is_none(), "{key} is a feature");
    }
    let mind = defined("gold-dragon/perfect-mind").expect("Perfect Mind");
    assert_eq!(
        (mind.bab_floor, mind.skill_ranks_at_least_mythic_rank),
        (Some(15), true)
    );
    assert!(
        data.mythic_feats
            .iter()
            .any(|feat| feat.key.as_str() == "destiny-beyond-birth" && feat.removes_racial_penalty),
        "Destiny Beyond Birth removes no racial penalty"
    );
}
