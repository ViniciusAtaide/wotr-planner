//! The gold standard (ticket 160): two real build write-ups, Jack-in-the-Green and the Enforcer, which the user knows
//! work in the game. `gold/source.txt` is their text as written; `gold/<build>.ron` the same, converted by hand into
//! the write-ups' own names; this file builds each character slot by slot and asserts that everything the write-up
//! lists at a level or a Mythic rank is gained there on the engine's sheet (the engine may show more).
//!
//! The write-ups are right (user, 2026-11-11): a disagreement is the engine's or the data's. Those known today are
//! [`KNOWN_GAPS`], each naming its ticket or the map decision behind it. The test fails on a disagreement that isn't
//! listed, and on a listed one that now agrees, so a fix is proved by deleting its entry.
//!
//! What "gained there" means is what the snapshot check (`snapshots.rs`) projects as a level's or rank's gains: a
//! level's slot picks, ability increase and the feats, features and options its state holds more of than the level
//! before's; a rank's slot picks and the features its path row gives ([`Engine::rank_features`]).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use rules::{
    Ability, AbilityScores, Alignment, ArchetypeKey, BackgroundKey, Build, CharacterState,
    ClassChoice, ClassKey, Element, Energy, Engine, Faith, FeatKey, FeatSource, GearKey, GearPlace,
    LevelSheet, LevelUp, MythicAbilityKey, MythicFeatKey, MythicPathKey, POINT_BUY_BUDGET,
    Parameter, Pick, PointBuy, RaceKey, SelectionOptionKey, Sheet, SkillKey, Slot, SlotId,
    SpellSchool, SubraceKey,
};
use serde::Deserialize;

use crate::support::{self, DATA};

/// The write-ups as written.
const SOURCE: &str = include_str!("gold/source.txt");

/// Each gold file, with the short name its disagreements and known gaps go by.
const GOLD: [(&str, &str); 2] = [
    ("jack", include_str!("gold/jack-in-the-green.ron")),
    ("enforcer", include_str!("gold/enforcer.ron")),
];

// ---- the gold files -------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Gold {
    /// The write-up's heading.
    name: String,
    start: GoldStart,
    levels: Vec<GoldLevel>,
    ranks: Vec<GoldRank>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldStart {
    race: String,
    background: String,
    /// Each class with its level count, in the write-up's order.
    classes: Vec<(String, u8)>,
    /// The race's +2, for a race that chooses it.
    ability_bonus: Option<String>,
    /// The scores at level 1, by the write-up's label: `("STR", 10)`.
    attributes: Vec<(String, u8)>,
    skills: Vec<String>,
    /// The alignments the write-up allows, as it abbreviates them: `"CN"`.
    alignments: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldLevel {
    level: u8,
    class: String,
    gains: Vec<String>,
    /// "Bloodline Spell - …" items, apart from the gains: spells known aren't Build choices.
    #[serde(default)]
    bloodline_spells: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldRank {
    rank: u8,
    /// "Mythic Hero" before the path, then the path's name.
    path: String,
    gains: Vec<String>,
}

fn gold(text: &str) -> Gold {
    ron::from_str(text).unwrap_or_else(|error| panic!("a gold file doesn't parse: {error}"))
}

// ---- the conversion's rules -----------------------------------------------------------------------------------

/// The label prefixes the conversion drops: each names the slot a pick goes in, not something gained. A prefix that
/// names an item ("Second Bloodline - ", "Extra Mythic Ability - ", "Menacing Style - ") stays in the name.
const LABELS: [&str; 7] = [
    "Bonus Feat - ",
    "Bloodline Feat - ",
    "Exploit - ",
    "Slayer Talents - ",
    "Advanced Talents - ",
    "Fey Tricks - ",
    "Superpower - ",
];

/// The prefix of a level's bloodline spell, which goes in `bloodline_spells`.
const BLOODLINE_SPELL: &str = "Bloodline Spell - ";

/// A build's part of `source.txt`, as the conversion reads it.
#[derive(Debug, Default, PartialEq)]
struct Source {
    race: String,
    background: String,
    classes: Vec<(String, u8)>,
    ability_bonus: Option<String>,
    attributes: Vec<(String, u8)>,
    skills: Vec<String>,
    alignments: Vec<String>,
    /// `(level, class, gains, bloodline spells)`.
    levels: Vec<(u8, String, Vec<String>, Vec<String>)>,
    /// `(rank, path, gains)`.
    ranks: Vec<(u8, String, Vec<String>)>,
}

/// A cell's items: split at the semicolons outside brackets, trimmed, without empty ones and "-".
fn items(cell: &str) -> Vec<String> {
    let mut items = Vec::new();
    let (mut depth, mut item) = (0usize, String::new());
    for c in cell.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ';' if depth == 0 => {
                items.push(std::mem::take(&mut item));
                continue;
            }
            _ => {}
        }
        item.push(c);
    }
    items.push(item);
    items
        .into_iter()
        .map(|item| item.trim().to_owned())
        .filter(|item| !item.is_empty() && item != "-")
        .collect()
}

/// A table's entries, `(number, column, cell)`: a line `N<TAB>column<TAB>cell` starts one, and the lines after it up
/// to the next entry continue its cell (a cell wraps). The table's last entry, `last`, takes no continuation: the
/// prose after the table starts there, and both write-ups' last cells are one line.
fn table<'a>(lines: &mut impl Iterator<Item = &'a str>, last: u8) -> Vec<(u8, String, String)> {
    let mut entries: Vec<(u8, String, String)> = Vec::new();
    for line in lines.by_ref() {
        let mut fields = line.splitn(3, '\t');
        let first = fields.next().unwrap_or_default();
        if let Ok(number) = first.parse::<u8>() {
            let column = fields.next().unwrap_or_default().trim().to_owned();
            let cell = fields.next().unwrap_or_default().to_owned();
            entries.push((number, column, cell));
            if number == last {
                break;
            }
        } else if !line.trim().is_empty() {
            let (_, _, cell) = entries.last_mut().expect("a continuation after an entry");
            cell.push(' ');
            cell.push_str(line);
        }
    }
    entries
}

/// The write-up headed `name` in `source.txt`, read as the conversion reads it.
fn source(name: &str) -> Source {
    let mut lines = SOURCE.lines().skip_while(|line| *line != name);
    assert_eq!(lines.next(), Some(name), "no write-up headed {name}");
    let mut out = Source::default();
    for line in lines.by_ref() {
        let (key, value) = line.split_once('\t').unwrap_or((line, ""));
        let list = |value: &str| -> Vec<String> { value.split(", ").map(str::to_owned).collect() };
        match key {
            "Race" => out.race = value.to_owned(),
            "Background" => out.background = value.to_owned(),
            "Classes" => {
                out.classes = value
                    .split(" / ")
                    .map(|class| {
                        let (name, count) = class.rsplit_once(" (").expect("Name (N)");
                        (
                            name.to_owned(),
                            count.trim_end_matches(')').parse().unwrap(),
                        )
                    })
                    .collect()
            }
            "Ability Bonus" => out.ability_bonus = Some(value.to_owned()),
            "Skills" => out.skills = list(value),
            "Alignment" => out.alignments = list(value),
            "Lvl" => break,
            _ if key.starts_with("STR: ") => {
                out.attributes = list(key)
                    .iter()
                    .map(|pair| {
                        let (label, score) = pair.split_once(": ").expect("LABEL: N");
                        (label.to_owned(), score.parse().unwrap())
                    })
                    .collect()
            }
            _ => {}
        }
    }
    for (level, class, cell) in table(&mut lines, 20) {
        let (mut gains, mut spells) = (Vec::new(), Vec::new());
        for item in items(&cell) {
            match item.strip_prefix(BLOODLINE_SPELL) {
                Some(spell) => spells.push(spell.to_owned()),
                None => gains.push(unlabelled(&item)),
            }
        }
        out.levels.push((level, class, gains, spells));
    }
    assert_eq!(lines.next(), Some("Mythic Path"), "{name}'s Mythic table");
    for (rank, path, cell) in table(&mut lines, 10) {
        let gains = items(&cell).iter().map(|item| unlabelled(item)).collect();
        out.ranks.push((rank, path, gains));
    }
    out
}

/// An item without its label prefix.
fn unlabelled(item: &str) -> String {
    LABELS
        .iter()
        .find_map(|label| item.strip_prefix(label))
        .unwrap_or(item)
        .to_owned()
}

/// The conversion, checked: each gold file says what its write-up says, item by item, in order.
#[test]
fn gold_files_match_the_write_ups() {
    for (_, text) in GOLD {
        let gold = gold(text);
        let start = &gold.start;
        let converted = Source {
            race: start.race.clone(),
            background: start.background.clone(),
            classes: start.classes.clone(),
            ability_bonus: start.ability_bonus.clone(),
            attributes: start.attributes.clone(),
            skills: start.skills.clone(),
            alignments: start.alignments.clone(),
            levels: gold
                .levels
                .iter()
                .map(|level| {
                    let (gains, spells) = (level.gains.clone(), level.bloodline_spells.clone());
                    (level.level, level.class.clone(), gains, spells)
                })
                .collect(),
            ranks: gold
                .ranks
                .iter()
                .map(|rank| (rank.rank, rank.path.clone(), rank.gains.clone()))
                .collect(),
        };
        assert_eq!(converted, source(&gold.name), "{}", gold.name);
        let levels: Vec<u8> = gold.levels.iter().map(|level| level.level).collect();
        assert_eq!(levels, (1..=20).collect::<Vec<u8>>(), "{}", gold.name);
        let ranks: Vec<u8> = gold.ranks.iter().map(|rank| rank.rank).collect();
        assert_eq!(ranks, (1..=10).collect::<Vec<u8>>(), "{}", gold.name);
    }
}

// ---- the two characters ---------------------------------------------------------------------------------------

fn feat_slot(level: u8, source: FeatSource) -> SlotId {
    SlotId::Feat {
        level,
        source,
        index: 0,
    }
}

fn general(level: u8) -> SlotId {
    feat_slot(level, FeatSource::General)
}

fn selection(level: u8, index: u8) -> SlotId {
    SlotId::Selection { level, index }
}

fn feat(key: &str) -> Pick {
    Pick::feat(FeatKey::new(key))
}

fn feat_for(key: &str, value: Parameter) -> Pick {
    Pick::feat_for(FeatKey::new(key), value)
}

fn option(key: &str) -> Pick {
    Pick::option(SelectionOptionKey::new(key))
}

fn option_for(key: &str, value: Parameter) -> Pick {
    Pick::option_for(SelectionOptionKey::new(key), value)
}

fn ability(key: &str) -> Pick {
    Pick::mythic_ability(MythicAbilityKey::new(key))
}

fn mythic_feat(key: &str) -> Pick {
    Pick::mythic_feat(MythicFeatKey::new(key))
}

fn mythic_feat_for(key: &str, value: Parameter) -> Pick {
    Pick::mythic_feat_for(MythicFeatKey::new(key), value)
}

/// A Build start's point buy.
fn point_buy(scores: [u8; 6]) -> PointBuy {
    let mut out = AbilityScores::default();
    for (ability, score) in AbilityScores::ABILITIES.into_iter().zip(scores) {
        *out.get_mut(ability) = score;
    }
    PointBuy::try_from(out).expect("a point buy")
}

/// A Level-up of `class` with one rank in each of `skills`.
fn level_up(class: &str, skills: &[&str], increase: Option<Ability>) -> LevelUp {
    let mut level = LevelUp::new(ClassKey::new(class));
    level.ability_increase = increase;
    for skill in skills {
        level.skill_ranks.insert(SkillKey::new(*skill), 1);
    }
    level
}

/// The ability increase at levels 4, 8, 12, 16 and 20, from the write-up's "+1 Strength" lines.
fn increase(level: u8, abilities: [Ability; 5]) -> Option<Ability> {
    (level % 4 == 0).then(|| abilities[usize::from(level / 4 - 1)])
}

/// Jack-in-the-Green: a Halfling (Hasty) Sylvan Sorcerer 20, Chaotic Good and atheist (ticket 88's settled choices),
/// a rank each in Mobility, Persuasion and Use Magic Device a level, the Azata's first ascension and path. The level-1
/// "Bonus Feat" is the Magic feat slot (ticket 92); the "Bloodline Feat"s are the class slots at 7, 13 and 19.
fn jack() -> Build {
    let mut build = Build::default();
    let start = &mut build.start;
    start.race = Some(RaceKey::new("halfling"));
    start.subrace = Some(SubraceKey::new("halfling/hasty"));
    start.background = Some(BackgroundKey::new("warrior-of-the-linnorm-kings"));
    start.deity = Some(Faith::Atheism);
    start.alignment = Some(Alignment::ChaoticGood);
    start.ability_scores = point_buy([12, 14, 14, 12, 8, 17]);
    build.class_choices.insert(
        ClassKey::new("sorcerer"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("sorcerer/sylvan-sorcerer")),
            ..ClassChoice::default()
        },
    );
    let skills = ["mobility", "persuasion", "use-magic-device"];
    for level in 1..=20 {
        let increase = increase(level, [Ability::Charisma; 5]);
        build.levels.push(level_up("sorcerer", &skills, increase));
    }
    build.first_ascension = Some(MythicPathKey::new("azata"));
    build.mythic_path = Some(MythicPathKey::new("azata"));
    let evocation = || Parameter::SpellSchool(SpellSchool::Evocation);
    let picks = [
        (feat_slot(1, FeatSource::Class), feat("spell-penetration")),
        (general(1), feat("mounted-combat")),
        (general(3), feat("metamagic-bolster-spell")),
        (general(5), feat_for("spell-focus", evocation())),
        (
            feat_slot(7, FeatSource::Class),
            feat_for("skill-focus", skill("mobility")),
        ),
        (general(7), feat("metamagic-heighten-spell")),
        (general(9), feat("outflank")),
        (general(11), feat("spell-specialization")),
        (
            feat_slot(13, FeatSource::Class),
            feat("improved-initiative"),
        ),
        (general(13), feat("greater-spell-penetration")),
        (general(15), feat_for("greater-spell-focus", evocation())),
        (general(17), feat("indomitable-mount")),
        (feat_slot(19, FeatSource::Class), feat("great-fortitude")),
        (
            general(19),
            feat_for("elemental-focus", Parameter::Energy(Energy::Electricity)),
        ),
        (SlotId::Mythic { rank: 1 }, ability("second-bloodline")),
        (
            SlotId::MythicSelection { rank: 1, index: 0 },
            option("sorcerer/bloodline/draconic-bronze"),
        ),
        (
            SlotId::Mythic { rank: 2 },
            mythic_feat("extra-mythic-ability"),
        ),
        (SlotId::MythicExtra { rank: 2 }, ability("abundant-casting")),
        (
            SlotId::Mythic { rank: 3 },
            Pick::mythic_ability_for(
                MythicAbilityKey::new("ascendant-element"),
                Parameter::Element(Element::Electricity),
            ),
        ),
        (
            SlotId::Mythic { rank: 4 },
            mythic_feat("spell-penetration-mythic"),
        ),
        (path_selection(4), option("azata/superpowers/zippy-magic")),
        (
            SlotId::Mythic { rank: 5 },
            ability("improved-abundant-casting"),
        ),
        (
            SlotId::Mythic { rank: 6 },
            mythic_feat_for(
                "expanded-arsenal",
                Parameter::SpellSchool(SpellSchool::Enchantment),
            ),
        ),
        (path_selection(6), option(LIFE_BONDING_FRIENDSHIP)),
        (path_feat(6), feat("shake-it-off")),
        (SlotId::Mythic { rank: 7 }, ability("mythical-beast")),
        (path_feat(7), feat("allied-spellcaster")),
        (
            SlotId::Mythic { rank: 8 },
            mythic_feat_for("spell-focus-mythic", evocation()),
        ),
        (
            path_selection(8),
            option("azata/superpowers/favorable-magic"),
        ),
        (
            SlotId::Mythic { rank: 9 },
            ability("greater-abundant-casting"),
        ),
        (SlotId::Mythic { rank: 10 }, mythic_feat("sorcerous-reflex")),
        (
            path_selection(10),
            option("azata/superpowers/marvelous-endurance"),
        ),
    ];
    build.picks.extend(picks);
    build
}

/// The Enforcer: a Half-Elf (+2 Strength) Arcane Enforcer 15 / Sylvan Trickster 4 / Mutation Warrior 1, Chaotic
/// Good and atheist, a rank each in Athletics, Perception and Persuasion a level and the rogue levels' two more in
/// Mobility and Stealth (ticket 88's settled choices), the Trickster's first ascension (ticket 91) and the Azata
/// path, with MR 10 taken at level 20, after Shield Focus (ticket 171). The slot its known gap leaves empty: level
/// 8's Slayer Talents; and level 13's Arcanist Exploits, beside the Greater Exploits slot its exploit goes in (ticket
/// 88). It wears a Dexterity +2 belt (ticket 168), so level 19's Greater Two-Weapon Fighting is legal.
fn enforcer() -> Build {
    let mut build = Build::default();
    let start = &mut build.start;
    start.race = Some(RaceKey::new("half-elf"));
    start.racial_ability = Some(Ability::Strength);
    start.background = Some(BackgroundKey::new("andoran-diplomat"));
    start.deity = Some(Faith::Atheism);
    start.alignment = Some(Alignment::ChaoticGood);
    start.ability_scores = point_buy([16, 16, 14, 10, 8, 12]);
    for (class, archetype) in [
        ("slayer", "slayer/arcane-enforcer"),
        ("rogue", "rogue/sylvan-trickster"),
        ("fighter", "fighter/mutation-warrior"),
    ] {
        build.class_choices.insert(
            ClassKey::new(class),
            ClassChoice {
                archetype: Some(ArchetypeKey::new(archetype)),
                ..ClassChoice::default()
            },
        );
    }
    let skills = ["athletics", "perception", "persuasion"];
    let rogue_skills = [
        "athletics",
        "perception",
        "persuasion",
        "mobility",
        "stealth",
    ];
    let increases = [
        Ability::Strength,
        Ability::Strength,
        Ability::Dexterity,
        Ability::Strength,
        Ability::Strength,
    ];
    for level in 1..=20 {
        let increase = increase(level, increases);
        build.levels.push(match level {
            1..=15 => level_up("slayer", &skills, increase),
            16..=19 => level_up("rogue", &rogue_skills, increase),
            _ => level_up("fighter", &skills, increase),
        });
    }
    build.first_ascension = Some(MythicPathKey::new("trickster"));
    build.mythic_path = Some(MythicPathKey::new("azata"));
    // The write-up names no item. Greater Two-Weapon Fighting at level 19 needs Dex 19 and the Build's own is 17; in
    // the game a belt covers it. The user's choice (ticket 163) is the smallest assumption that makes level 19 legal:
    // a belt with a Dexterity +2 enhancement bonus, the real Belt of Incredible Dexterity +2.
    build.gear.worn.insert(
        GearPlace::Belt,
        GearKey::new("belt-of-incredible-dexterity-2"),
    );
    let battle_axe = || support::weapon("battle-axe");
    let picks = [
        (
            selection(1, 0),
            option("arcanist/arcanist-exploits/wooden-flesh"),
        ),
        (
            feat_slot(1, FeatSource::Race),
            feat_for("skill-focus", skill("persuasion")),
        ),
        (general(1), feat("two-weapon-fighting")),
        (
            selection(2, 0),
            option("slayer/slayer-talents/combat-style-feat-2"),
        ),
        (
            SlotId::OptionSelection { level: 2, index: 0 },
            option("ranger/combat-style-feat/menacing-feats"),
        ),
        (
            SlotId::OptionFeat { level: 2, index: 0 },
            feat("intimidating-prowess"),
        ),
        (general(3), feat("shield-bash")),
        (
            selection(4, 0),
            option("arcanist/arcanist-exploits/armored-mask"),
        ),
        (general(5), feat("outflank")),
        (
            selection(6, 0),
            option("slayer/slayer-talents/combat-style-feat-6"),
        ),
        (
            SlotId::OptionFeat { level: 6, index: 0 },
            feat("shatter-defenses"),
        ),
        (
            selection(7, 0),
            option("arcanist/arcanist-exploits/dimensional-slide"),
        ),
        (general(7), feat("combat-reflexes")),
        (general(9), feat_for("improved-critical", battle_axe())),
        (
            selection(10, 0),
            option("arcanist/arcanist-exploits/spell-resistance"),
        ),
        (general(11), feat("shield-master")),
        (
            selection(12, 0),
            option("slayer/slayer-talents/combat-style-feat-10"),
        ),
        (
            SlotId::OptionFeat {
                level: 12,
                index: 0,
            },
            feat("dreadful-carnage"),
        ),
        (
            selection(13, 1),
            option("arcanist/greater-exploits/greater-spell-resistance"),
        ),
        (general(13), feat("improved-two-weapon-fighting")),
        (
            selection(14, 0),
            option("rogue/rogue-talents/crippling-strike"),
        ),
        (general(15), feat("improved-initiative")),
        (selection(17, 0), option("witch/hex/iceplant")),
        (general(17), feat("bashing-finish")),
        // Rogue level 3: Finesse Training in the light shield's bash (ticket 172).
        (
            selection(18, 0),
            option_for(
                "rogue/finesse-training/weapon",
                support::weapon("light-shield-bash"),
            ),
        ),
        (
            selection(19, 0),
            option("rogue/rogue-talents/double-debilitation"),
        ),
        (general(19), feat("greater-two-weapon-fighting")),
        (feat_slot(20, FeatSource::Class), feat("shield-focus")),
        (SlotId::Mythic { rank: 1 }, ability("defensive-study")),
        (
            SlotId::Mythic { rank: 2 },
            mythic_feat("two-weapon-fighting-mythic"),
        ),
        (SlotId::Mythic { rank: 3 }, ability("inspirational-leader")),
        (
            SlotId::Mythic { rank: 4 },
            mythic_feat_for("improved-critical-mythic", battle_axe()),
        ),
        (
            path_selection(4),
            option("azata/superpowers/incredible-might"),
        ),
        (SlotId::Mythic { rank: 5 }, ability("archmage-armor")),
        (
            SlotId::Mythic { rank: 6 },
            mythic_feat("improved-initiative-mythic"),
        ),
        (path_selection(6), option(LIFE_BONDING_FRIENDSHIP)),
        (path_feat(6), feat("shield-wall")),
        (SlotId::Mythic { rank: 7 }, ability("enduring-spells")),
        (path_feat(7), feat("shake-it-off")),
        (
            SlotId::Mythic { rank: 8 },
            mythic_feat("extra-mythic-ability"),
        ),
        (
            SlotId::MythicExtra { rank: 8 },
            ability("greater-enduring-spells"),
        ),
        (
            path_selection(8),
            option("azata/superpowers/marvelous-endurance"),
        ),
        (SlotId::Mythic { rank: 9 }, ability("ever-ready")),
        (
            SlotId::Mythic { rank: 10 },
            mythic_feat("shield-focus-mythic"),
        ),
        (
            path_selection(10),
            option("azata/superpowers/supersonic-speed"),
        ),
    ];
    build.picks.extend(picks);
    // MR 10 after the last level (ticket 171): its Shield Focus (Mythic) needs Shield Focus, which the write-up
    // takes at level 20, and the rank table's level for MR 10 is 19. "You can select mythic feats after leveling all
    // base levels" (the user, 2026-11-11).
    build.rank_levels.insert(10, 20);
    build
}

const LIFE_BONDING_FRIENDSHIP: &str = "azata/superpowers/life-bonding-friendship";

fn skill(key: &str) -> Parameter {
    support::skill(key)
}

fn path_selection(rank: u8) -> SlotId {
    SlotId::PathSelection { rank, index: 0 }
}

fn path_feat(rank: u8) -> SlotId {
    SlotId::PathFeat { rank, index: 0 }
}

/// The two gold Builds, by their short names: the snapshot corpus takes them too (`snapshots.rs`).
pub fn builds() -> [(&'static str, Build); 2] {
    [("jack", jack()), ("enforcer", enforcer())]
}

#[test]
fn gold_builds_are_possible() {
    let engine = support::engine();
    for (name, build) in builds() {
        assert!(engine.is_possible(&build), "{name}");
        // Every pick is on the sheet, in a slot of its kind.
        let sheet = engine.evaluate(&build);
        for slot in build.picks.keys() {
            assert!(sheet.slot(slot).is_some(), "{name}: {slot:?}");
        }
    }
}

/// Ticket 171 on the real data: at the rank table's level (19), MR 10's Mythic feat slot refuses Shield Focus
/// (Mythic), since Shield Focus comes at level 20; the engine says MR 10 taken at level 20 opens it, the level picker
/// offers that level, and once the feat is taken there the rank can't come back until the feat does.
#[test]
fn the_enforcers_mythic_shield_focus_needs_mr_10_taken_at_level_20() {
    let engine = support::engine();
    let gold = enforcer();
    let slot = SlotId::Mythic { rank: 10 };
    let pick = gold.picks[&slot].clone();
    let mut at_table = gold.clone();
    at_table.rank_levels.clear();
    assert!(!engine.is_possible(&at_table));
    at_table.picks.remove(&slot);
    assert!(engine.is_possible(&at_table));
    let sheet = engine.evaluate(&at_table);
    assert_eq!(sheet.ranks[9].character_level, 19);
    assert_eq!(
        engine.rank_level_that_opens(&at_table, &sheet, &slot, &pick),
        Some((20, vec![]))
    );
    let levels: Vec<(u8, bool, bool)> = (engine.rank_level_options(&at_table, 10).iter())
        .map(|option| (option.level, option.held, option.status.is_ok()))
        .collect();
    assert_eq!(levels, [(19, true, true), (20, false, true)]);

    let back = &engine.rank_level_options(&gold, 10)[0];
    assert_eq!(back.status, Err(rules::Issue::BreaksLater));
    assert_eq!(
        back.reason.as_deref(),
        Some("Shield Focus (Mythic) needs MR 10 later than level 19")
    );
    // Its link holds the move, and reopens to the same Build.
    let code = rules::encode_build(&gold);
    assert_eq!(engine.open_link(&code), Ok(gold));
}

// ---- names ----------------------------------------------------------------------------------------------------

/// The write-up's spellings the planner spells otherwise, each with what the sheet gains for it there and, apart, what
/// it only needs held by then (gained there or before): `(write-up, gained, held)`.
const NAMES: &[(&str, &[&str], &[&str])] = &[
    ("Bypass Epic DR", &["Bypass Epic Damage Reduction"], &[]),
    ("Metamagic (Bolstered)", &["Metamagic (Bolster Spell)"], &[]),
    ("Metamagic (Heighten)", &["Metamagic (Heighten Spell)"], &[]),
    ("Charisma +1", &["+1 Charisma"], &[]),
    ("Draconic Bloodline Arcana", &["Draconic Arcana"], &[]),
    ("Dragon Resistances I", &["Draconic Resistances"], &[]),
    ("Dragon Resistances II", &["Draconic Resistances"], &[]),
    ("Dragon Resistances III", &["Draconic Resistances"], &[]),
    (
        "Soul of the Fey (Power of Wyrms)",
        &["Soul of the Fey", "Power of Wyrms"],
        &[],
    ),
    (
        "Second Bloodline - Bronze Dragon",
        &["Second Bloodline", "Draconic (Bronze)"],
        &[],
    ),
    (
        "Extra Mythic Ability - Abundant Casting",
        &["Extra Mythic Ability", "Abundant Casting"],
        &[],
    ),
    (
        "Extra Mythic Ability - Greater Enduring Spells",
        &["Extra Mythic Ability", "Greater Enduring Spells"],
        &[],
    ),
    // The bracket names the superpower's teamwork feat, at its first rank and its second.
    (
        "Life-Bonding Friendship (Shake It Off)",
        &["Shake It Off"],
        &["Life-Bonding Friendship"],
    ),
    (
        "Life-Bonding Friendship (Allied Spellcaster)",
        &["Allied Spellcaster"],
        &["Life-Bonding Friendship"],
    ),
    (
        "Life-Bonding Friendship (Shield Wall)",
        &["Shield Wall"],
        &["Life-Bonding Friendship"],
    ),
    // The slayer's combat style is the ranger's Menacing Feats, taken through a Combat Style Feat talent.
    (
        "Menacing Style - Intimidating Prowess",
        &["Intimidating Prowess"],
        &["Menacing Feats"],
    ),
    (
        "Menacing Style - Shatter Defenses",
        &["Shatter Defenses"],
        &["Menacing Feats"],
    ),
    (
        "Menacing Style - Dreadful Carnage",
        &["Dreadful Carnage"],
        &["Menacing Feats"],
    ),
    // The game's level-up names each step of Studied Target and of the sneak dice.
    ("Studied Target +1", &["Studied Target"], &[]),
    ("Studied Target +2", &["Studied Target"], &[]),
    ("Studied Target +3", &["Studied Target"], &[]),
    ("Studied Target +4", &["Studied Target"], &[]),
    ("+1d6 sneak", &["Sneak Attack"], &[]),
    ("Mythic Shield Focus", &["Shield Focus (Mythic)"], &[]),
    ("Andoren Diplomat", &["Andoran Diplomat"], &[]),
];

/// The values the planner spells otherwise, in a "Name - Value" name: `(write-up, planner)`.
const VALUES: &[(&str, &str)] = &[
    ("Battleaxe", "Battle Axe"),
    ("Light Shield", "Light Shield Bash"),
];

/// The write-up's name of the path before MR 3's: the first ascension's, shown at MR 1 and 2.
const MYTHIC_HERO: &str = "Mythic Hero";

/// What a gold name asks of the engine's sheet: the planner names gained at its place, and those only held there.
#[derive(Debug, Default)]
struct Expected {
    gained: Vec<String>,
    held: Vec<String>,
}

/// A gold name read as planner names, by [`NAMES`], or else "Name - Value" as the planner's "Name (Value)" (with
/// [`VALUES`]), or else as itself; each must name a planner item ([`Planner::knows`]), else the name that doesn't.
fn expected(planner: &Planner, gold: &str) -> Result<Expected, String> {
    let owned = |names: &[&str]| names.iter().map(|name| (*name).to_owned()).collect();
    let out = match NAMES.iter().find(|(name, ..)| *name == gold) {
        Some((_, gained, held)) => Expected {
            gained: owned(gained),
            held: owned(held),
        },
        None => {
            let name = match gold.rsplit_once(" - ") {
                Some((name, value)) => {
                    let value = VALUES
                        .iter()
                        .find(|(written, _)| *written == value)
                        .map_or(value, |(_, planner)| *planner);
                    format!("{name} ({value})")
                }
                None => gold.to_owned(),
            };
            Expected {
                gained: vec![name],
                held: Vec::new(),
            }
        }
    };
    for name in out.gained.iter().chain(&out.held) {
        if !planner.knows(name) {
            return Err(name.clone());
        }
    }
    Ok(out)
}

/// The planner's names: every item a level or rank can gain, and every value a parameter takes.
struct Planner {
    items: BTreeSet<String>,
    values: BTreeSet<String>,
}

impl Planner {
    fn new() -> Self {
        let mut items: BTreeSet<String> = BTreeSet::new();
        items.extend(DATA.feats.iter().map(|feat| feat.name.clone()));
        items.extend(DATA.class_features().map(|feature| feature.name.clone()));
        items.extend(
            DATA.mythic_paths
                .iter()
                .flat_map(|path| &path.features)
                .map(|feature| feature.name.clone()),
        );
        items.extend(
            DATA.selections()
                .flat_map(|selection| &selection.options)
                .map(|option| option.name.clone()),
        );
        items.extend(DATA.mythic_abilities.iter().map(|a| a.name.clone()));
        items.extend(DATA.mythic_feats.iter().map(|f| f.name.clone()));
        items.extend(AbilityScores::ABILITIES.map(|ability| ability.increase_name()));
        let mut values: BTreeSet<String> = BTreeSet::new();
        values.extend(DATA.skills.iter().map(|skill| skill.name.clone()));
        values.extend(DATA.weapons.iter().map(|weapon| weapon.name.clone()));
        values.extend(SpellSchool::ALL.map(|school| school.name().to_owned()));
        values.extend(Energy::ALL.map(|energy| energy.name().to_owned()));
        values.extend(Element::ALL.map(|element| element.name().to_owned()));
        Self { items, values }
    }

    /// Whether a name is a planner item's, or an item's with a value: "Improved Critical (Battle Axe)".
    fn knows(&self, name: &str) -> bool {
        self.items.contains(name)
            || name
                .strip_suffix(')')
                .and_then(|name| name.rsplit_once(" ("))
                .is_some_and(|(item, value)| {
                    self.items.contains(item) && self.values.contains(value)
                })
    }
}

// ---- the engine's sheet ---------------------------------------------------------------------------------------

/// Where on the sheet: the Build start, a level or a Mythic rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Place {
    Start,
    Level(u8),
    Rank(u8),
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Place::Start => write!(f, "start"),
            Place::Level(level) => write!(f, "level {level}"),
            Place::Rank(rank) => write!(f, "MR {rank}"),
        }
    }
}

/// What the engine's sheet gains at each level and rank, and holds at each level's end, by name.
struct Gains<'a> {
    engine: &'a Engine,
    build: &'a Build,
    sheet: Sheet,
    gained: BTreeMap<Place, BTreeSet<String>>,
    held: BTreeMap<u8, BTreeSet<String>>,
}

/// A feat's names: itself, and with each value it's held for.
fn feat_names(engine: &Engine, state: &CharacterState, feat: &FeatKey) -> Vec<String> {
    let name = engine.feat_name(feat);
    match state.feat_values.get(feat) {
        Some(values) => values
            .keys()
            .map(|value| format!("{name} ({})", engine.parameter_name(value)))
            .collect(),
        None => vec![name.to_owned()],
    }
}

/// The names a state holds: feats (with their values), features (with and without whose they are), options, mythic
/// abilities and mythic feats (with their values).
fn held_names(engine: &Engine, state: &CharacterState) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for feat in state.feats.keys() {
        names.extend(feat_names(engine, state, feat));
    }
    for feature in state.features.keys() {
        names.insert(engine.feature_name(feature).to_owned());
        names.insert(engine.feature_name_apart(feature));
    }
    names.extend(
        state
            .selection_options
            .keys()
            .map(|option| engine.option_name(option).to_owned()),
    );
    names.extend(
        state
            .mythic_abilities
            .iter()
            .map(|ability| engine.mythic_ability_name(ability).to_owned()),
    );
    for feat in &state.mythic_feats {
        let name = engine.mythic_feat_name(feat);
        names.insert(name.to_owned());
        for value in state.mythic_feat_values.get(feat).into_iter().flatten() {
            names.insert(format!("{name} ({})", engine.parameter_name(value)));
        }
    }
    names
}

impl<'a> Gains<'a> {
    fn new(engine: &'a Engine, build: &'a Build) -> Self {
        let sheet = engine.evaluate(build);
        let mut gained: BTreeMap<Place, BTreeSet<String>> = BTreeMap::new();
        let mut held = BTreeMap::new();
        let empty = CharacterState::default();
        let mut before = &empty;
        for (level_up, level) in build.levels.iter().zip(&sheet.levels) {
            let state = &level.state;
            let names = gained
                .entry(Place::Level(level.character_level))
                .or_default();
            names.extend(picked(engine, &level.slots));
            if let Some(ability) = level_up.ability_increase {
                names.insert(ability.increase_name());
            }
            for (feat, times) in &state.feats {
                let new_values: Vec<String> = match state.feat_values.get(feat) {
                    Some(values) => values
                        .iter()
                        .filter(|(value, times)| {
                            let had = before.feat_values.get(feat).and_then(|v| v.get(*value));
                            had.copied().unwrap_or(0) < **times
                        })
                        .map(|(value, _)| {
                            let name = engine.feat_name(feat);
                            format!("{name} ({})", engine.parameter_name(value))
                        })
                        .collect(),
                    None if before.feats.get(feat).copied().unwrap_or(0) < *times => {
                        vec![engine.feat_name(feat).to_owned()]
                    }
                    None => Vec::new(),
                };
                names.extend(new_values);
            }
            for (feature, times) in &state.features {
                if before.feature_count(feature) < *times {
                    names.insert(engine.feature_name(feature).to_owned());
                    names.insert(engine.feature_name_apart(feature));
                }
            }
            for (option, times) in &state.selection_options {
                if before.selection_options.get(option).copied().unwrap_or(0) < *times {
                    names.insert(engine.option_name(option).to_owned());
                }
            }
            held.insert(level.character_level, held_names(engine, state));
            before = state;
        }
        for rank in &sheet.ranks {
            let names = gained.entry(Place::Rank(rank.rank)).or_default();
            names.extend(picked(engine, &rank.slots));
            for feature in engine.rank_features(rank) {
                names.insert(engine.feature_name(feature).to_owned());
                names.insert(engine.feature_name_apart(feature));
            }
        }
        Self {
            engine,
            build,
            sheet,
            gained,
            held,
        }
    }

    /// The character level a place is on: a rank's ascension's.
    fn level_of(&self, place: Place) -> u8 {
        match place {
            Place::Start => 1,
            Place::Level(level) => level,
            Place::Rank(rank) => self.sheet.ranks[usize::from(rank) - 1].character_level,
        }
    }

    fn gains(&self, place: Place, name: &str) -> bool {
        self.gained
            .get(&place)
            .is_some_and(|names| names.contains(name))
    }

    /// Why `name` isn't gained at `place`: why the empty slot there that offers it locks it, or where it's gained
    /// instead, or that nothing gains it.
    fn why_not(&self, place: Place, name: &str) -> String {
        let elsewhere: Vec<String> = self
            .gained
            .iter()
            .filter(|(_, names)| names.contains(name))
            .map(|(at, _)| at.to_string())
            .collect();
        let slots = match place {
            Place::Start => &[][..],
            Place::Level(level) => &self.sheet.levels[usize::from(level) - 1].slots[..],
            Place::Rank(rank) => &self.sheet.ranks[usize::from(rank) - 1].slots[..],
        };
        for slot in slots.iter().filter(|slot| slot.pick.is_none()) {
            let offered = self
                .engine
                .options(self.build, &slot.id)
                .unwrap_or_default();
            let Some(option) = offered
                .into_iter()
                .find(|option| self.engine.pick_name(&option.pick) == name)
            else {
                continue;
            };
            let label = self.engine.slot_label(slot);
            return match option.status {
                Err(issue) => match self.engine.issue_text(&issue) {
                    Some(reason) => format!("locked in the {label} slot, \"{reason}\""),
                    None => format!("locked in the {label} slot: it breaks a later choice"),
                },
                Ok(()) => format!("open in the {label} slot, but the Build doesn't take it"),
            };
        }
        if elsewhere.is_empty() {
            "not gained at any level or rank, nor offered by an empty slot here".to_owned()
        } else {
            format!("not gained here, but at {}", elsewhere.join(", "))
        }
    }

    /// The level whose Mythic rank takes the Second Bloodline, if one does.
    fn second_bloodline_level(&self) -> Option<u8> {
        let second = ability("second-bloodline");
        self.sheet
            .ranks
            .iter()
            .find(|rank| {
                rank.slots
                    .iter()
                    .any(|slot| slot.pick.as_ref() == Some(&second))
            })
            .map(|rank| rank.character_level)
    }
}

/// The names of a list of slots' picks.
fn picked(engine: &Engine, slots: &[Slot]) -> Vec<String> {
    slots
        .iter()
        .filter_map(|slot| slot.pick.as_ref())
        .map(|pick| engine.pick_name(pick))
        .collect()
}

// ---- the comparison -------------------------------------------------------------------------------------------

/// What a disagreement is about: a known gap covers one of these on its line, and no other.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Failure {
    /// A planner item the sheet doesn't gain at the line's place.
    NotGained(String),
    /// A planner item the sheet doesn't hold by the line's place.
    NotHeld(String),
    /// A name that resolves to no planner item.
    NoPlannerItem(String),
    /// The start's, a level's class or a rank's path, or no Second Bloodline for a bracketed line.
    Other,
}

/// One gold line the engine's sheet disagrees with, and on what.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Disagreement {
    build: &'static str,
    place: Place,
    line: String,
    failure: Failure,
    what: String,
}

impl fmt::Display for Disagreement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}: {}: {}",
            self.build, self.place, self.line, self.what
        )
    }
}

/// Every disagreement between a gold file and its Build's sheet.
fn compare(build_name: &'static str, gold: &Gold, build: &Build) -> Vec<Disagreement> {
    let engine = support::engine();
    let planner = Planner::new();
    let gains = Gains::new(&engine, build);
    let mut out = Vec::new();
    let mut disagree = |place: Place, line: &str, failure: Failure, what: String| {
        out.push(Disagreement {
            build: build_name,
            place,
            line: line.to_owned(),
            failure,
            what,
        });
    };
    start(&engine, gold, build, &gains, &mut disagree);
    for level in &gold.levels {
        let place = Place::Level(level.level);
        let class = level_class(&engine, &gains.sheet.levels[usize::from(level.level) - 1]);
        if class != level.class {
            let what = format!("the sheet's class is {class}");
            disagree(place, &level.class, Failure::Other, what);
        }
        for line in &level.gains {
            for (failure, what) in check_line(&planner, &gains, place, line) {
                disagree(place, line, failure, what);
            }
        }
        for spell in &level.bloodline_spells {
            for (failure, what) in check_bloodline_spell(&planner, &gains, level.level, spell) {
                disagree(place, &format!("Bloodline Spell - {spell}"), failure, what);
            }
        }
    }
    for rank in &gold.ranks {
        let place = Place::Rank(rank.rank);
        let sheet = &gains.sheet.ranks[usize::from(rank.rank) - 1];
        let path = sheet.path.as_ref().map(|path| engine.path_name(path));
        // MR 1 and 2 show the first ascension's path (the MR 3 path's without one).
        let first_ascension = build
            .first_ascension
            .as_ref()
            .or(build.mythic_path.as_ref());
        let fits = match rank.path.as_str() {
            MYTHIC_HERO => rank.rank < 3 && sheet.path.as_ref() == first_ascension,
            name => rank.rank >= 3 && path == Some(name),
        };
        if !fits {
            let what = format!("the sheet's path is {path:?}");
            disagree(place, &rank.path, Failure::Other, what);
        }
        for line in &rank.gains {
            for (failure, what) in check_line(&planner, &gains, place, line) {
                disagree(place, line, failure, what);
            }
        }
    }
    out
}

/// A level's class as the write-ups name it: its archetype's name, or the class's without one.
fn level_class<'a>(engine: &'a Engine, level: &'a LevelSheet) -> &'a str {
    let archetype = level
        .state
        .classes
        .iter()
        .find(|taken| taken.class == level.class)
        .and_then(|taken| taken.archetype.as_ref());
    archetype.map_or_else(
        || engine.class_name(&level.class),
        |archetype| engine.archetype_name(archetype),
    )
}

/// What a gold line asks that the sheet doesn't do, one entry per planner name. A line in brackets, "(Dragon
/// Resistances I)", is the write-up's mark for the second bloodline's: each of its items is gained at its level, or at
/// the Second Bloodline's rank when that comes later.
fn check_line(
    planner: &Planner,
    gains: &Gains,
    place: Place,
    line: &str,
) -> Vec<(Failure, String)> {
    let (names, place) = match line.strip_prefix('(').and_then(|l| l.strip_suffix(')')) {
        Some(inner) => {
            let Some(second) = gains.second_bloodline_level() else {
                let what = "no Mythic rank takes the Second Bloodline".to_owned();
                return vec![(Failure::Other, what)];
            };
            let at = Place::Level(gains.level_of(place).max(second));
            (items(inner), at)
        }
        None => (vec![line.to_owned()], place),
    };
    let mut out = Vec::new();
    for name in names {
        let expected = match expected(planner, &name) {
            Ok(expected) => expected,
            Err(unknown) => {
                let what = format!("\"{unknown}\" names no planner item");
                out.push((Failure::NoPlannerItem(unknown), what));
                continue;
            }
        };
        for gained in &expected.gained {
            if !gains.gains(place, gained) {
                let why = gains.why_not(place, gained);
                // The planner's name only where it isn't the line's own.
                let what = match gained.as_str() == line {
                    true => why,
                    false => format!("{gained}: {why}"),
                };
                out.push((Failure::NotGained(gained.clone()), what));
            }
        }
        let level = gains.level_of(place);
        for held in &expected.held {
            if !gains.held[&level].contains(held) {
                let what = format!("{held}: not held by level {level}");
                out.push((Failure::NotHeld(held.clone()), what));
            }
        }
    }
    out
}

/// A level's bloodline spell, "Entangle (+ Mage Armor)": the bloodline's own spell, which the sheet gains as the
/// feature "Bloodline spell - Entangle"; and in brackets the second bloodline's, which the data doesn't hold (spells
/// known aren't Build choices), so it isn't compared. [`the_second_bloodlines_spells_arent_in_the_data`] says when
/// it would be.
fn check_bloodline_spell(
    planner: &Planner,
    gains: &Gains,
    level: u8,
    spell: &str,
) -> Vec<(Failure, String)> {
    let own = spell.split_once(" (+").map_or(spell, |(own, _)| own).trim();
    let name = format!("Bloodline spell - {own}");
    if !planner.knows(&name) {
        let what = format!("\"{name}\" names no planner item");
        return vec![(Failure::NoPlannerItem(name), what)];
    }
    let place = Place::Level(level);
    if gains.gains(place, &name) {
        Vec::new()
    } else {
        let what = format!("{name}: {}", gains.why_not(place, &name));
        vec![(Failure::NotGained(name), what)]
    }
}

/// The second bloodline's spells in a gold file: what each "(+ …)" names.
fn second_bloodline_spells(gold: &Gold) -> Vec<String> {
    gold.levels
        .iter()
        .flat_map(|level| &level.bloodline_spells)
        .filter_map(|spell| spell.split_once("(+"))
        .map(|(_, second)| second.trim_end_matches(')').trim().to_owned())
        .collect()
}

/// The Build start. The race, background, classes and the race's +2 are the Build's own input, compared by name. The
/// engine's part: the attributes are the level-1 scores (the point buy with the engine's racial modifiers) within the
/// point-buy budget; the engine allows the Build at each listed alignment; and its skill points give a rank in each
/// listed skill at every level.
fn start(
    engine: &Engine,
    gold: &Gold,
    build: &Build,
    gains: &Gains,
    disagree: &mut impl FnMut(Place, &str, Failure, String),
) {
    let start = &gold.start;
    let planner_name = |written: &str| -> String {
        NAMES
            .iter()
            .find(|(name, ..)| *name == written)
            .map_or_else(
                || written.to_owned(),
                |(_, planner, _)| planner[0].to_owned(),
            )
    };
    let race = build.start.race.as_ref().map(|race| engine.race_name(race));
    if race != Some(planner_name(&start.race).as_str()) {
        disagree(
            Place::Start,
            &start.race,
            Failure::Other,
            format!("the race is {race:?}"),
        );
    }
    let background = build
        .start
        .background
        .as_ref()
        .map(|key| engine.background_name(key));
    if background != Some(planner_name(&start.background).as_str()) {
        let what = format!("the background is {background:?}");
        disagree(Place::Start, &start.background, Failure::Other, what);
    }
    let mut classes: Vec<(String, u8)> = Vec::new();
    for level in &gains.sheet.levels {
        let name = level_class(engine, level);
        match classes.last_mut() {
            Some((class, count)) if class == name => *count += 1,
            _ => classes.push((name.to_owned(), 1)),
        }
    }
    if classes != start.classes {
        let what = format!("the levels' classes are {classes:?}");
        disagree(Place::Start, "Classes", Failure::Other, what);
    }
    if let Some(bonus) = &start.ability_bonus {
        let chosen = build.start.racial_ability.map(Ability::name);
        if chosen != Some(bonus.as_str()) {
            disagree(
                Place::Start,
                bonus,
                Failure::Other,
                format!("the race's +2 is {chosen:?}"),
            );
        }
    }
    let level_1 = &gains.sheet.levels[0].state;
    for (label, score) in &start.attributes {
        let ability = AbilityScores::ABILITIES
            .into_iter()
            .find(|ability| ability.short_name().eq_ignore_ascii_case(label))
            .unwrap_or_else(|| panic!("no ability {label}"));
        let have = level_1.base_ability_scores.get(ability);
        if have != *score {
            disagree(
                Place::Start,
                label,
                Failure::Other,
                format!("{score} is {have} at level 1"),
            );
        }
    }
    let spent = build.start.ability_scores.cost();
    if i16::from(spent) > i16::from(POINT_BUY_BUDGET) {
        let what = format!("the point buy costs {spent} of {POINT_BUY_BUDGET}");
        disagree(Place::Start, "Attributes", Failure::Other, what);
    }
    for alignment in &start.alignments {
        let Some(chosen) = Alignment::ALL
            .into_iter()
            .find(|candidate| candidate.short_name() == alignment)
        else {
            disagree(
                Place::Start,
                alignment,
                Failure::Other,
                "names no alignment".to_owned(),
            );
            continue;
        };
        let mut aligned = build.clone();
        aligned.start.alignment = Some(chosen);
        if !engine.is_possible(&aligned) {
            disagree(
                Place::Start,
                alignment,
                Failure::Other,
                "the Build isn't possible".to_owned(),
            );
        }
    }
    for written in &start.skills {
        if !DATA.skills.iter().any(|skill| &skill.name == written) {
            disagree(
                Place::Start,
                written,
                Failure::Other,
                "names no skill".to_owned(),
            );
        }
    }
    // The engine's skill points (class, Intelligence, race) cover a rank in each listed skill at every level.
    let listed = start.skills.len();
    for level in &gains.sheet.levels {
        let available = usize::from(level.skill_points.available);
        if available < listed {
            let what = format!(
                "level {} has {available} skill points for {listed} skills",
                level.character_level
            );
            disagree(Place::Start, "Skills", Failure::Other, what);
        }
    }
}

// ---- the known gaps -------------------------------------------------------------------------------------------

/// What a known gap waits on: always a ticket, of one of three kinds.
enum Why {
    /// A bug in the engine or the data.
    Bug(u16),
    /// A map decision that kept the planner from modelling the line, reopened by the user (2026-11-11) as this
    /// ticket.
    Reopened(u16, &'static str),
    /// The write-up against the guide, whose list the data files (the planner's source of truth) follow; the
    /// question is open in this design ticket.
    AgainstTheGuide(u16, &'static str),
}

impl fmt::Display for Why {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Why::Bug(ticket) => write!(f, "a bug, ticket {ticket}"),
            Why::Reopened(ticket, decision) => {
                write!(f, "a reopened map decision, ticket {ticket}: {decision}")
            }
            Why::AgainstTheGuide(ticket, conflict) => {
                write!(
                    f,
                    "the write-up against the guide, ticket {ticket}: {conflict}"
                )
            }
        }
    }
}

/// The one failure on its line that a known gap covers: a planner item not gained there, or a name that resolves to
/// no planner item. Any other failure on the same line is still reported.
#[derive(Debug)]
enum Covers {
    NotGained(&'static str),
    NoPlannerItem(&'static str),
}

impl Covers {
    fn matches(&self, failure: &Failure) -> bool {
        match (self, failure) {
            (Covers::NotGained(item), Failure::NotGained(failed))
            | (Covers::NoPlannerItem(item), Failure::NoPlannerItem(failed)) => item == failed,
            _ => false,
        }
    }
}

/// A gold line the engine disagrees with today, on one failure.
struct Gap {
    build: &'static str,
    place: Place,
    line: &'static str,
    covers: Covers,
    why: Why,
}

const fn gap(
    build: &'static str,
    place: Place,
    line: &'static str,
    covers: Covers,
    why: Why,
) -> Gap {
    Gap {
        build,
        place,
        line,
        covers,
        why,
    }
}

/// The gold lines the engine gets wrong today, each on the one failure it covers. The list only shrinks: a fix
/// deletes its entry.
const KNOWN_GAPS: &[Gap] = &[
    // ---- map decisions the user reopened ----
    gap(
        "jack",
        Place::Level(1),
        "Animal Companion - Elk",
        Covers::NoPlannerItem("Animal Companion (Elk)"),
        Why::Reopened(174, "animal companions"),
    ),
    // ---- the write-up against the guide ----
    gap(
        "enforcer",
        Place::Level(8),
        "Uncanny Dodge",
        Covers::NotGained("Uncanny Dodge"),
        Why::AgainstTheGuide(
            173,
            "the write-up takes Uncanny Dodge as a slayer talent; the guide's slayer talents, which the data \
             follows, have none",
        ),
    ),
];

/// The disagreements no known gap covers, and the known gaps that cover none (they now agree).
fn unexplained_by(disagreements: &[Disagreement], gaps: &[Gap]) -> Vec<String> {
    let covered = |d: &Disagreement, gap: &Gap| {
        gap.build == d.build
            && gap.place == d.place
            && gap.line == d.line
            && gap.covers.matches(&d.failure)
    };
    let mut out: Vec<String> = disagreements
        .iter()
        .filter(|d| !gaps.iter().any(|gap| covered(d, gap)))
        .map(ToString::to_string)
        .collect();
    for gap in gaps {
        if !disagreements.iter().any(|d| covered(d, gap)) {
            out.push(format!(
                "{} {}: {}: listed as a known gap ({:?}; {}) but now agrees: delete its entry",
                gap.build, gap.place, gap.line, gap.covers, gap.why
            ));
        }
    }
    out
}

/// Both gold files against their Builds' sheets.
fn disagreements() -> Vec<Disagreement> {
    let mut disagreements = Vec::new();
    for ((name, build), (short, text)) in builds().into_iter().zip(GOLD) {
        assert_eq!(name, short);
        disagreements.extend(compare(name, &gold(text), &build));
    }
    disagreements.sort();
    disagreements
}

fn unexplained() -> Vec<String> {
    unexplained_by(&disagreements(), KNOWN_GAPS)
}

#[test]
fn gold_builds_gain_what_the_write_ups_list() {
    let unexplained = unexplained();
    assert!(
        unexplained.is_empty(),
        "{} gold lines disagree with the engine and aren't known gaps (gold.rs's KNOWN_GAPS), or are listed and \
         now agree:\n{}",
        unexplained.len(),
        unexplained.join("\n")
    );
}

/// The write-ups' "(+ …)" spells are the Draconic second bloodline's, which the data doesn't hold: it holds bloodline
/// spells only for the archetypes whose tables list them (the Sylvan Sorcerer's among them). When the Draconic
/// bloodline's appear, compare them.
#[test]
fn the_second_bloodlines_spells_arent_in_the_data() {
    let draconic: Vec<&str> = DATA
        .class_features()
        .filter(|feature| {
            feature
                .key
                .as_str()
                .starts_with("sorcerer/bloodline/draconic/")
        })
        .filter(|feature| feature.name.starts_with("Bloodline spell"))
        .map(|feature| feature.name.as_str())
        .collect();
    assert_eq!(
        draconic,
        Vec::<&str>::new(),
        "the data now has the Draconic bloodline's spells: compare the write-ups' (+ …) spells in gold.rs"
    );
    let spells = second_bloodline_spells(&gold(GOLD[0].1));
    assert_eq!(spells.len(), 9, "{spells:?}");
}

/// Every entry of [`NAMES`] and [`VALUES`] reads a name of the gold files, so the tables hold no stale spelling.
#[test]
fn gold_names_tables_are_all_used() {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (_, text) in GOLD {
        let gold = gold(text);
        names.insert(gold.start.background.clone());
        let lines = gold.levels.iter().flat_map(|level| &level.gains);
        for line in lines.chain(gold.ranks.iter().flat_map(|rank| &rank.gains)) {
            match line.strip_prefix('(').and_then(|l| l.strip_suffix(')')) {
                Some(inner) => names.extend(items(inner)),
                None => {
                    names.insert(line.clone());
                }
            }
        }
    }
    for (written, ..) in NAMES {
        assert!(
            names.contains(*written),
            "NAMES: {written} is in no gold file"
        );
    }
    for (written, _) in VALUES {
        let suffix = format!(" - {written}");
        assert!(
            names.iter().any(|name| name.ends_with(&suffix)),
            "VALUES: {written} is in no gold file"
        );
    }
}

/// A known gap covers its one failure, not its line: a Jack whose second bloodline is Elemental (Fire), not Draconic
/// (Bronze), fails "(Dragon Resistances III; Wings)" on Draconic Resistances and on Wings; an entry on Draconic
/// Resistances (made up here: the list holds none) doesn't hide Wings.
#[test]
fn a_known_gap_hides_no_other_failure_on_its_line() {
    let (_, mut jack) = builds().into_iter().next().unwrap();
    jack.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        option("sorcerer/bloodline/elemental-fire"),
    );
    assert!(support::possible(&jack));
    let found = compare("jack", &gold(GOLD[0].1), &jack);
    let line = |d: &&Disagreement| {
        d.place == Place::Level(15) && d.line == "(Dragon Resistances III; Wings)"
    };
    let failures: Vec<&Failure> = found.iter().filter(line).map(|d| &d.failure).collect();
    assert_eq!(
        failures,
        [
            &Failure::NotGained("Draconic Resistances".to_owned()),
            &Failure::NotGained("Wings".to_owned()),
        ]
    );
    let draconic_resistances = [gap(
        "jack",
        Place::Level(15),
        "(Dragon Resistances III; Wings)",
        Covers::NotGained("Draconic Resistances"),
        Why::Bug(0),
    )];
    let unexplained = unexplained_by(&found, &draconic_resistances);
    let at_15: Vec<&String> = unexplained
        .iter()
        .filter(|line| line.starts_with("jack level 15: (Dragon Resistances III; Wings)"))
        .collect();
    assert_eq!(
        at_15,
        [
            "jack level 15: (Dragon Resistances III; Wings): Wings: not gained at any level or rank, nor offered by an \
          empty slot here"
        ]
    );
}
