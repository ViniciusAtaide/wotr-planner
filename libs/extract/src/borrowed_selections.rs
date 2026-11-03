//! Ticket 60: ticket 54's decision 2 (user, 2026-11-02), applied to finished data. Classes and archetypes that pick
//! from another class's list held a feature with no slot; each becomes a slot, of the existing selection where the
//! guide's list is one, or of a selection sharing the options of the lists it mixes, as ticket 25 made the slayer's.
//!
//! - **Bard and skald talents** (`bard#bard-talents`: "The Bard can select from the Rogue Talent list";
//!   `skald#skald-talents`: "The Skald can select from the Rogue Talent list at 2nd, 7th, 12th and 17th levels"): the
//!   "Bard Talents" and "Skald Talents" features become slots of the rogue's own `rogue/rogue-talents`, where each
//!   table has "Bard Talent" or "Skald Talent" (the Beast Tamer from 6, the Inciter at 4, 9, 14, 19).
//! - **The Archaeologist's Advanced Talents** (`bard#advanced-talents`: "At 10th level, the Archaeologist may select
//!   from the Advanced Rogue Talents"): its feature opens the advanced rogue talents (`rogue#advanced-rogue-talents`)
//!   beside the rogue's and the slayer's, in each one's `Any`.
//! - **The Feyform Shifter's Rogue Talent** (`shifter#rogue-talent`: "At 8th and 14th levels, the shifter can choose a
//!   Rogue Talent. Note that you cannot choose Advanced Talents."): a list the rogue's isn't, so its own selection
//!   `shifter/feyform-shifter/rogue-talent` shares the rogue talents but the advanced ones.
//! - **The Vivisectionist's Medical Discovery** (`alchemist#medical-discovery`: "able to select certain Rogue Talents
//!   (including Advanced Talents from level 10) and Alchemist discoveries", then three named lists): its own selection
//!   `alchemist/vivisectionist/medical-discovery` shares exactly those, and its Advanced Talents feature (at 10) opens
//!   the three advanced talents named.
//! - **The Sanctified Slayer's Slayer Talents** (`inquisitor#slayer-talents`: "At 8th, 16th, 17th and 20th levels, a
//!   Sanctified Slayer can select a Slayer talent"): slots of the slayer's own `slayer/slayer-talents`. Extra Slayer
//!   Talent's "Prerequisites: Slayer Talents" names this section too, so a Sanctified Slayer 8 meets it beside a slayer
//!   2. The combat style feats are the slayer's ("he can only do so ... at certain levels: 2 or 4, 6 or 8 and 10 or
//!   greater"): the 2nd-or-4th one needs slayer level 2, as the other two need theirs.
//! - **The Hexcrafter's Hex Magus and Hex Arcana** (`magus#hex-magus`: "At 4th level, a Hexcrafter may select a Witch
//!   hex"; `magus#hex-arcana`: "Besides the usual Magus Arcana abilities, a Hexcrafter can select a hex. From 12th
//!   level, a Hexcrafter can select major hexes and at 18th level, he can select a grand hex."): Hex Magus is a slot
//!   of the witch's own `witch/hex`, so Extra Hex reaches it; Hex Arcana, where the table has it, is its own selection
//!   `magus/hexcrafter/hex-arcana` sharing the magus arcana and the witch hexes; the major and grand hexes open to a
//!   Hexcrafter of magus 12 and 18.
//!
//! An option held through a selection with option levels gets its rows by the levels of the classes that selection
//! names. Of these lists only Combat Trick has a row, at option level 1, so each selection a new class picks through
//! names that class (the rogue's talents the bard and the skald, the slayer's the inquisitor): a pick there gives
//! Combat Trick's feat slot, and no option's row moves. Extra Rogue Talent's "Prerequisites: Rogue Talents" names the
//! Feyform Shifter's "Rogue Talent" too, so its extra pick may be of that selection (ticket 49's rule).
//!
//! The features that go are needed by nothing. Each rule asserts the guide phrase it rests on and what it finds in
//! the data, so it fails loudly on data it doesn't expect. [`apply`] runs on the finished data of an extraction (after
//! ticket 59's `infusions`) and, through ticket 60's one-off rewrite, on `libs/game-data/`.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArchetypeKey, ClassKey, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData,
    Prerequisite, SelectionOptionKey, effective_row,
};

use crate::follow_ups::{class_mut, says};
use crate::guide::Guide;
use crate::html::Page;
use crate::mercies_and_exploits::{levels_in_tables, replace_leaf};
use crate::option_levels::{Log, page, section, selection};
use crate::wild_talents::{needing, take};

const ROGUE_TALENTS: &str = "rogue/rogue-talents";
const ROGUE_ADVANCED: &str = "rogue/advanced-talents";
const SLAYER_TALENTS: &str = "slayer/slayer-talents";
const WITCH_HEX: &str = "witch/hex";
const MAGUS_ARCANA: &str = "magus/magus-arcana";

const ARCHAEOLOGIST_ADVANCED: &str = "bard/archaeologist/advanced-talents";
const FEYFORM: &str = "shifter/feyform-shifter";
const FEYFORM_TALENT: &str = "shifter/feyform-shifter/rogue-talent";
const VIVISECTIONIST: &str = "alchemist/vivisectionist";
const MEDICAL_DISCOVERY: &str = "alchemist/vivisectionist/medical-discovery";
const VIVISECTIONIST_ADVANCED: &str = "alchemist/vivisectionist/advanced-talents";
const SANCTIFIED: &str = "inquisitor/sanctified-slayer";
const SANCTIFIED_FEATURE: &str = "inquisitor/sanctified-slayer/slayer-talents";
/// The Sanctified Slayer's first Slayer Talent.
const SANCTIFIED_FIRST: u8 = 8;
const HEXCRAFTER: &str = "magus/hexcrafter";
const HEX_MAGUS: &str = "magus/hexcrafter/hex-magus";
const HEX_ARCANA: &str = "magus/hexcrafter/hex-arcana";
/// "From 12th level, a Hexcrafter can select major hexes and at 18th level, he can select a grand hex."
const HEXCRAFTER_MAJOR: u8 = 12;
const HEXCRAFTER_GRAND: u8 = 18;

const EXTRA_ROGUE_TALENT: &str = "extra-rogue-talent";
const EXTRA_SLAYER_TALENT: &str = "extra-slayer-talent";
const COMBAT_STYLE_2: &str = "slayer/slayer-talents/combat-style-feat-2";

/// Makes the borrowed lists picks.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    let (talents, advanced) = rogue_talents(guide, data);
    bards_and_skalds(guide, data, &advanced, &mut log);
    feyform(guide, data, &talents, &advanced, &mut log);
    vivisectionist(guide, data, &mut log);
    sanctified_slayer(guide, data, &mut log);
    hexcrafter(guide, data, &mut log);
    log
}

/// The rogue talents and the advanced ones among them, by key: the guide's two lists (`rogue#rogue-talents`,
/// `rogue#advanced-rogue-talents`), the advanced ones exactly those the data gates on the rogue's Advanced Talents.
fn rogue_talents(
    guide: &Guide,
    data: &GameData,
) -> (BTreeSet<SelectionOptionKey>, BTreeSet<SelectionOptionKey>) {
    says(
        guide,
        "rogue",
        "advanced-rogue-talents",
        "These unlock at 10th level.",
    );
    let rogue_page = page(guide, "rogue");
    let names = |anchor: &str| -> BTreeSet<String> {
        section(rogue_page, anchor)
            .children
            .iter()
            .map(|s| s.name.clone())
            .collect()
    };
    let (plain, advanced_names) = (names("rogue-talents"), names("advanced-rogue-talents"));
    let selection = find_selection(data, ROGUE_TALENTS);
    let by_name: BTreeMap<&str, &SelectionOptionKey> = selection
        .options
        .iter()
        .map(|o| (o.name.as_str(), &o.key))
        .collect();
    assert_eq!(
        by_name
            .keys()
            .map(|n| n.to_string())
            .collect::<BTreeSet<_>>(),
        &plain | &advanced_names,
        "{ROGUE_TALENTS}: the guide's talents"
    );
    let gated: BTreeSet<SelectionOptionKey> = selection
        .options
        .iter()
        .filter(|o| {
            o.prerequisite
                .as_ref()
                .is_some_and(|p| names_feature(p, ROGUE_ADVANCED))
        })
        .map(|o| o.key.clone())
        .collect();
    let advanced: BTreeSet<SelectionOptionKey> = advanced_names
        .iter()
        .map(|n| by_name[n.as_str()].clone())
        .collect();
    assert_eq!(gated, advanced, "the talents gated on {ROGUE_ADVANCED}");
    assert_eq!(
        (plain.len(), advanced.len()),
        (20, 16),
        "the guide's talents"
    );
    (by_name.values().map(|k| (*k).clone()).collect(), advanced)
}

fn bards_and_skalds(
    guide: &Guide,
    data: &mut GameData,
    advanced: &BTreeSet<SelectionOptionKey>,
    log: &mut Log,
) {
    says(
        guide,
        "bard",
        "bard-talents",
        "The Bard can select from the Rogue Talent list.",
    );
    says(
        guide,
        "skald",
        "skald-talents",
        "The Skald can select from the Rogue Talent list at 2nd, 7th, 12th and 17th levels.",
    );
    for (class, feature, cell) in [
        ("bard", "bard/bard-talents", "Bard Talent"),
        ("skald", "skald/skald-talents", "Skald Talent"),
    ] {
        let levels = replace(data, class, None, feature, ROGUE_TALENTS, log);
        assert!(!levels.is_empty(), "{feature}: no rows");
        assert_in_force(guide, data, class, ROGUE_TALENTS, cell);
        add_option_level(data, ROGUE_TALENTS, class, log);
    }

    says(
        guide,
        "bard",
        "advanced-talents",
        "At 10th level, the Archaeologist may select from the Advanced Rogue Talents.",
    );
    assert_eq!(
        levels_in_tables(page(guide, "bard"), "Advanced Talents"),
        BTreeMap::from([("archaeologist".to_owned(), vec![10])]),
        "bard: the tables' Advanced Talents"
    );
    open_advanced(data, advanced, ARCHAEOLOGIST_ADVANCED, log);
}

fn feyform(
    guide: &Guide,
    data: &mut GameData,
    talents: &BTreeSet<SelectionOptionKey>,
    advanced: &BTreeSet<SelectionOptionKey>,
    log: &mut Log,
) {
    says(
        guide,
        "shifter",
        "rogue-talent",
        "At 8th and 14th levels, the shifter can choose a Rogue Talent. Note that you cannot choose Advanced Talents.",
    );
    let shifter_page = page(guide, "shifter");
    let mut chosen = selection(
        FEYFORM_TALENT,
        &section(shifter_page, "rogue-talent").name,
        shifter_page.guide("rogue-talent"),
        &["shifter"],
    );
    chosen.shared_options = talents - advanced;
    add_selection(data, "shifter", FEYFORM, chosen, log);
    replace(
        data,
        "shifter",
        Some(FEYFORM),
        FEYFORM_TALENT,
        FEYFORM_TALENT,
        log,
    );
    assert_in_force(guide, data, "shifter", FEYFORM_TALENT, "Rogue Talent");

    // "Prerequisites: Rogue Talents or Fey Tricks": the Feyform Shifter's Rogue Talent is one.
    says(
        guide,
        "feats",
        EXTRA_ROGUE_TALENT,
        "Prerequisites: Rogue Talents or Fey Tricks",
    );
    let feat = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == EXTRA_ROGUE_TALENT)
        .expect("Extra Rogue Talent");
    assert_eq!(
        feat.extra_pick_of,
        BTreeSet::from([
            FeatureSelectionKey::new(ROGUE_TALENTS),
            FeatureSelectionKey::new("rogue/sylvan-trickster/fey-tricks"),
        ]),
        "{EXTRA_ROGUE_TALENT}'s extra pick"
    );
    feat.extra_pick_of
        .insert(FeatureSelectionKey::new(FEYFORM_TALENT));
    log.push(format!(
        "{EXTRA_ROGUE_TALENT}: its extra pick may be of {FEYFORM_TALENT}"
    ));
}

fn vivisectionist(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "alchemist",
        "medical-discovery",
        "The Vivisectionist is able to select certain Rogue Talents (including Advanced Talents from level 10) and \
         Alchemist discoveries",
    );
    let alchemist_page = page(guide, "alchemist");
    assert_eq!(
        levels_in_tables(alchemist_page, "Advanced Talents"),
        BTreeMap::from([("vivisectionist".to_owned(), vec![10])]),
        "alchemist: the tables' Advanced Talents"
    );
    let lists = read_medical_discoveries(alchemist_page);
    let headings: Vec<&str> = lists.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(
        headings,
        ["Discoveries", "Rogue Talents", "Advanced Rogue Talents"],
        "alchemist#medical-discovery's lists"
    );
    let resolve = |selection_key: &str, names: &[String]| -> BTreeSet<SelectionOptionKey> {
        let selection = find_selection(data, selection_key);
        names
            .iter()
            .flat_map(|name| {
                let found: Vec<SelectionOptionKey> = selection
                    .options
                    .iter()
                    .filter(|o| {
                        o.name == *name
                            // "Focusing Attacks": the three "Focusing Attack - …".
                            || (name.ends_with('s')
                                && o.name.starts_with(&format!("{} - ", &name[..name.len() - 1])))
                    })
                    .map(|o| o.key.clone())
                    .collect();
                assert!(!found.is_empty(), "{selection_key}: no {name:?}");
                found
            })
            .collect()
    };
    let discoveries = resolve("alchemist/discovery", &lists[0].1);
    let talents = resolve(ROGUE_TALENTS, &lists[1].1);
    let advanced = resolve(ROGUE_TALENTS, &lists[2].1);
    assert_eq!(
        (discoveries.len(), talents.len(), advanced.len()),
        (14, 8, 3),
        "the Vivisectionist's lists"
    );
    for talent in &advanced {
        let prerequisite = find_option_prerequisite(data, talent);
        assert!(
            names_feature(prerequisite, ROGUE_ADVANCED),
            "{talent}: not advanced"
        );
    }
    for talent in &talents {
        assert!(
            !find_option_prerequisite_opt(data, talent)
                .is_some_and(|p| names_feature(p, ROGUE_ADVANCED)),
            "{talent}: advanced"
        );
    }

    let mut chosen = selection(
        MEDICAL_DISCOVERY,
        &section(alchemist_page, "medical-discovery").name,
        alchemist_page.guide("medical-discovery"),
        &["alchemist"],
    );
    chosen.shared_options = discoveries
        .iter()
        .chain(&talents)
        .chain(&advanced)
        .cloned()
        .collect();
    add_selection(data, "alchemist", VIVISECTIONIST, chosen, log);
    replace(
        data,
        "alchemist",
        Some(VIVISECTIONIST),
        MEDICAL_DISCOVERY,
        MEDICAL_DISCOVERY,
        log,
    );
    assert_in_force(
        guide,
        data,
        "alchemist",
        MEDICAL_DISCOVERY,
        "Medical Discovery",
    );
    open_advanced(data, &advanced, VIVISECTIONIST_ADVANCED, log);
}

fn sanctified_slayer(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "inquisitor",
        "slayer-talents",
        "At 8th, 16th, 17th and 20th levels, a Sanctified Slayer can select a Slayer talent.",
    );
    let name = &section(page(guide, "inquisitor"), "slayer-talents").name;
    assert_eq!(name, "Slayer Talents", "inquisitor#slayer-talents");
    let levels = replace(
        data,
        "inquisitor",
        Some(SANCTIFIED),
        SANCTIFIED_FEATURE,
        SLAYER_TALENTS,
        log,
    );
    assert_eq!(
        levels.first(),
        Some(&SANCTIFIED_FIRST),
        "{SANCTIFIED}'s first"
    );
    assert_in_force(guide, data, "inquisitor", SLAYER_TALENTS, "Slayer Talent");
    add_option_level(data, SLAYER_TALENTS, "inquisitor", log);

    // "Prerequisites: Slayer Talents or Talented Slayer": the slayer's from 2 (ticket 44), the Sanctified Slayer's
    // from 8.
    says(
        guide,
        "feats",
        EXTRA_SLAYER_TALENT,
        "Prerequisites: Slayer Talents or Talented Slayer",
    );
    let slayer_2 = Prerequisite::ClassLevel {
        class: ClassKey::new("slayer"),
        level: 2,
    };
    let feat = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == EXTRA_SLAYER_TALENT)
        .expect("Extra Slayer Talent");
    assert_eq!(
        feat.prerequisite.as_ref(),
        Some(&slayer_2),
        "{EXTRA_SLAYER_TALENT}"
    );
    let mut prerequisite = Prerequisite::Any(vec![
        slayer_2.clone(),
        Prerequisite::All(vec![
            Prerequisite::Archetype(ArchetypeKey::new(SANCTIFIED)),
            Prerequisite::ClassLevel {
                class: ClassKey::new("inquisitor"),
                level: SANCTIFIED_FIRST,
            },
        ]),
    ]);
    prerequisite.canonicalize();
    feat.prerequisite = Some(prerequisite);
    log.push(format!(
        "{EXTRA_SLAYER_TALENT}: a slayer 2 or a Sanctified Slayer {SANCTIFIED_FIRST}"
    ));

    // The combat style feats are the slayer's own windows by slayer level (the note under Slayer Camouflage).
    says(
        guide,
        "slayer",
        "slayer-camouflage",
        "he can only do so three times in total and at certain levels: 2 or 4, 6 or 8 and 10 or greater.",
    );
    let style = find_option_mut(data, SLAYER_TALENTS, COMBAT_STYLE_2);
    let not_6 = Prerequisite::Not(Box::new(Prerequisite::ClassLevel {
        class: ClassKey::new("slayer"),
        level: 6,
    }));
    assert_eq!(
        style.prerequisite.as_ref(),
        Some(&not_6),
        "{COMBAT_STYLE_2}"
    );
    let mut prerequisite = Prerequisite::All(vec![slayer_2, not_6]);
    prerequisite.canonicalize();
    style.prerequisite = Some(prerequisite);
    log.push(format!("{COMBAT_STYLE_2}: needs slayer level 2"));
}

fn hexcrafter(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "magus",
        "hex-magus",
        "At 4th level, a Hexcrafter may select a Witch hex",
    );
    says(
        guide,
        "magus",
        "hex-arcana",
        "Besides the usual Magus Arcana abilities, a Hexcrafter can select a hex. From 12th level, a Hexcrafter can \
         select major hexes and at 18th level, he can select a grand hex.",
    );
    let magus_page = page(guide, "magus");
    replace(data, "magus", Some(HEXCRAFTER), HEX_MAGUS, WITCH_HEX, log);
    assert_in_force(guide, data, "magus", WITCH_HEX, "Hex Magus");

    let mut chosen = selection(
        HEX_ARCANA,
        &section(magus_page, "hex-arcana").name,
        magus_page.guide("hex-arcana"),
        &[],
    );
    let arcana = find_selection(data, MAGUS_ARCANA);
    let hexes = find_selection(data, WITCH_HEX);
    for list in [arcana, hexes] {
        assert!(
            list.shared_options.is_empty() && list.option_levels.is_empty(),
            "{}: shares or has option levels",
            list.key
        );
        assert!(
            list.options.iter().all(|o| o.levels.is_empty()),
            "{}: options with rows",
            list.key
        );
    }
    chosen.shared_options = arcana
        .options
        .iter()
        .chain(&hexes.options)
        .map(|o| o.key.clone())
        .collect();
    add_selection(data, "magus", HEXCRAFTER, chosen, log);
    replace(data, "magus", Some(HEXCRAFTER), HEX_ARCANA, HEX_ARCANA, log);
    assert_in_force(guide, data, "magus", HEX_ARCANA, "Hex Arcana");
    // The Hexcrafter's arcana are all its Hex Arcana: the rows in force hold no magus arcana slot.
    let tables = levels_in_tables(magus_page, "Magus Arcana");
    assert!(
        !tables.contains_key("hexcrafter"),
        "magus#hexcrafter: Magus Arcana"
    );
    assert_in_force(guide, data, "magus", MAGUS_ARCANA, "Magus Arcana");

    // The major and grand hexes open to a Hexcrafter of magus 12 and 18, beside the witch's and the Sylvan
    // Trickster's own features.
    for (feature, level, count) in [
        ("witch/major-hexes", HEXCRAFTER_MAJOR, 9),
        ("witch/grand-hexes", HEXCRAFTER_GRAND, 4),
    ] {
        let leaf = Prerequisite::Feature {
            feature: FeatureKey::new(feature),
            count: 1,
        };
        let with = Prerequisite::Any(vec![
            leaf.clone(),
            Prerequisite::All(vec![
                Prerequisite::Archetype(ArchetypeKey::new(HEXCRAFTER)),
                Prerequisite::ClassLevel {
                    class: ClassKey::new("magus"),
                    level,
                },
            ]),
        ]);
        let witch = class_mut(data, "witch");
        let hexes = witch
            .selections
            .iter_mut()
            .find(|s| s.key.as_str() == WITCH_HEX)
            .expect("the witch's hexes");
        let mut opened = 0;
        for option in &mut hexes.options {
            let Some(prerequisite) = option.prerequisite.as_mut() else {
                continue;
            };
            if replace_leaf(prerequisite, &leaf, &with) > 0 {
                prerequisite.canonicalize();
                opened += 1;
                log.push(format!(
                    "{}: also a Hexcrafter of magus {level}",
                    option.key
                ));
            }
        }
        assert_eq!(opened, count, "the hexes needing {feature}");
    }
}

/// `alchemist#medical-discovery`'s lists: each item's bold heading and the names on its next line, with "Greater /
/// Grand Mutagen" read as both.
pub fn read_medical_discoveries(page: &Page) -> Vec<(String, Vec<String>)> {
    section(page, "medical-discovery")
        .list_items()
        .map(|item| {
            let (heading, names) = item
                .split_once('\n')
                .unwrap_or_else(|| panic!("alchemist#medical-discovery: item {item:?}"));
            let names = names
                .trim_end_matches('.')
                .split(", ")
                .flat_map(|name| match name.split_once(" / ") {
                    // "Greater / Grand Mutagen": the first word of each, then the rest.
                    Some((first, rest)) => {
                        let (second, tail) = rest
                            .split_once(' ')
                            .unwrap_or_else(|| panic!("alchemist#medical-discovery: {name:?}"));
                        vec![format!("{first} {tail}"), format!("{second} {tail}")]
                    }
                    None => vec![name.to_owned()],
                })
                .collect();
            (heading.to_owned(), names)
        })
        .collect()
}

/// Takes `feature` off the class's (or the archetype's) features and every row of the class and its archetypes that
/// held it, putting a slot of `selection` on each; the base rows' levels, or the archetype's.
fn replace(
    data: &mut GameData,
    class: &str,
    archetype: Option<&str>,
    feature: &str,
    selection_key: &str,
    log: &mut Log,
) -> Vec<u8> {
    let feature = FeatureKey::new(feature);
    assert_eq!(
        needing(data, &feature),
        Vec::<String>::new(),
        "what needs {feature}"
    );
    let selection_key = FeatureSelectionKey::new(selection_key);
    let class_data = class_mut(data, class);
    let base = put(
        &feature,
        &selection_key,
        log,
        class,
        class_data
            .levels
            .iter_mut()
            .map(|r| (r.level, &mut r.features, &mut r.selections)),
    );
    let mut own = Vec::new();
    for archetype_data in &mut class_data.archetypes {
        let key = archetype_data.key.to_string();
        let levels = put(
            &feature,
            &selection_key,
            log,
            &key,
            archetype_data
                .levels
                .iter_mut()
                .map(|r| (r.level, &mut r.features, &mut r.selections)),
        );
        if Some(key.as_str()) == archetype {
            own = levels;
        } else if archetype.is_some() {
            assert!(levels.is_empty(), "{key} holds {feature}");
        }
    }
    let defined = match archetype {
        None => {
            assert!(!base.is_empty(), "the base {class} holds no {feature}");
            &mut class_data.features
        }
        Some(archetype) => {
            assert!(base.is_empty(), "the base {class} holds {feature}");
            &mut class_data
                .archetypes
                .iter_mut()
                .find(|a| a.key.as_str() == archetype)
                .unwrap_or_else(|| panic!("no {archetype}"))
                .features
        }
    };
    let before = defined.len();
    defined.retain(|f| f.key != feature);
    assert_eq!(defined.len(), before - 1, "{feature}: defined");
    if archetype.is_none() { base } else { own }
}

/// Puts a slot of `selection` on each row holding `feature`, in its place; the levels of those rows.
fn put<'r>(
    feature: &FeatureKey,
    selection: &FeatureSelectionKey,
    log: &mut Log,
    owner: &str,
    rows: impl Iterator<
        Item = (
            u8,
            &'r mut Vec<FeatureKey>,
            &'r mut Vec<FeatureSelectionKey>,
        ),
    >,
) -> Vec<u8> {
    let mut levels = Vec::new();
    for (level, features, selections) in rows {
        if take(features, feature) {
            assert!(
                !selections.contains(selection),
                "{owner} {level}: a {selection} slot already"
            );
            selections.push(selection.clone());
            selections.sort();
            levels.push(level);
        }
    }
    if !levels.is_empty() {
        log.push(format!(
            "{owner} {levels:?}: a slot of {selection} for {feature}"
        ));
    }
    levels
}

/// The rows in force hold one slot of `selection` exactly where each of the class's tables has `cell`, and nowhere
/// else (the base class under the page's own anchor, `""`).
fn assert_in_force(guide: &Guide, data: &GameData, class: &str, selection_key: &str, cell: &str) {
    let tables = levels_in_tables(page(guide, class), cell);
    let class_data = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == class)
        .unwrap_or_else(|| panic!("no class {class}"));
    let selection_key = FeatureSelectionKey::new(selection_key);
    let held = |archetype| -> Vec<u8> {
        (1..=20)
            .filter(|level| {
                effective_row(class_data, archetype, *level).is_some_and(|row| {
                    row.selections
                        .iter()
                        .filter(|s| **s == selection_key)
                        .count()
                        == 1
                })
            })
            .collect()
    };
    let mut found = BTreeMap::new();
    let base = held(None);
    if !base.is_empty() {
        found.insert(String::new(), base);
    }
    for archetype in &class_data.archetypes {
        let levels = held(Some(archetype));
        if !levels.is_empty() {
            let anchor = archetype
                .guide
                .0
                .strip_prefix(&format!("{class}#"))
                .unwrap_or_else(|| panic!("{}: guide {}", archetype.key, archetype.guide.0));
            found.insert(anchor.to_owned(), levels);
        }
    }
    assert_eq!(
        found, tables,
        "{class}: {selection_key} where the tables have {cell:?}"
    );
}

/// Names `class` among the classes whose levels make the option level of an option held through `selection`. Its
/// options' rows are all at option level 1, so this only lets a pick on that class's rows have its row.
fn add_option_level(data: &mut GameData, selection_key: &str, class: &str, log: &mut Log) {
    let selection = find_selection_mut(data, selection_key);
    assert!(
        selection.level_from.is_none() && selection.levels_behind == 0,
        "{selection_key}: levelled otherwise"
    );
    let shared = selection.shared_options.clone();
    let own_rows: Vec<u8> = selection
        .options
        .iter()
        .flat_map(|o| o.levels.iter().map(|r| r.level))
        .collect();
    assert!(
        selection.option_levels.insert(ClassKey::new(class)),
        "{selection_key}: {class} already"
    );
    let shared_rows: Vec<u8> = shared
        .iter()
        .flat_map(|key| {
            find_option(data, key)
                .levels
                .iter()
                .map(|r| r.level)
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        own_rows.iter().chain(&shared_rows).all(|level| *level == 1),
        "{selection_key}: an option row past level 1"
    );
    log.push(format!("{selection_key}: levelled by {class} too"));
}

/// Opens each of `talents`, which needs the rogue's Advanced Talents, to `feature` too.
fn open_advanced(
    data: &mut GameData,
    talents: &BTreeSet<SelectionOptionKey>,
    feature: &str,
    log: &mut Log,
) {
    let leaf = Prerequisite::Feature {
        feature: FeatureKey::new(ROGUE_ADVANCED),
        count: 1,
    };
    let with = Prerequisite::Any(vec![
        leaf.clone(),
        Prerequisite::Feature {
            feature: FeatureKey::new(feature),
            count: 1,
        },
    ]);
    let rogue = find_selection_mut(data, ROGUE_TALENTS);
    for option in &mut rogue.options {
        if !talents.contains(&option.key) {
            continue;
        }
        let prerequisite = option
            .prerequisite
            .as_mut()
            .unwrap_or_else(|| panic!("{}: no prerequisite", option.key));
        assert_eq!(
            replace_leaf(prerequisite, &leaf, &with),
            1,
            "{}: {ROGUE_ADVANCED} once",
            option.key
        );
        prerequisite.canonicalize();
    }
    log.push(format!(
        "{} advanced rogue talents: also {feature}",
        talents.len()
    ));
}

fn add_selection(
    data: &mut GameData,
    class: &str,
    archetype: &str,
    chosen: FeatureSelection,
    log: &mut Log,
) {
    assert!(
        all_selections(data).all(|s| s.key != chosen.key),
        "{}: a selection already",
        chosen.key
    );
    log.push(format!(
        "{}: \"{}\", sharing {} options",
        chosen.key,
        chosen.name,
        chosen.shared_options.len()
    ));
    class_mut(data, class)
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == archetype)
        .unwrap_or_else(|| panic!("no {archetype}"))
        .selections
        .push(chosen);
}

fn all_selections(data: &GameData) -> impl Iterator<Item = &FeatureSelection> {
    data.classes
        .iter()
        .chain(&data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
}

fn find_selection<'d>(data: &'d GameData, key: &str) -> &'d FeatureSelection {
    all_selections(data)
        .find(|s| s.key.as_str() == key)
        .unwrap_or_else(|| panic!("no selection {key}"))
}

fn find_selection_mut<'d>(data: &'d mut GameData, key: &str) -> &'d mut FeatureSelection {
    data.classes
        .iter_mut()
        .chain(&mut data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter_mut()
                .chain(class.archetypes.iter_mut().flat_map(|a| &mut a.selections))
        })
        .find(|s| s.key.as_str() == key)
        .unwrap_or_else(|| panic!("no selection {key}"))
}

fn find_option<'d>(data: &'d GameData, key: &SelectionOptionKey) -> &'d rules::SelectionOption {
    all_selections(data)
        .flat_map(|s| &s.options)
        .find(|o| &o.key == key)
        .unwrap_or_else(|| panic!("no option {key}"))
}

fn find_option_mut<'d>(
    data: &'d mut GameData,
    selection_key: &str,
    key: &str,
) -> &'d mut rules::SelectionOption {
    find_selection_mut(data, selection_key)
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == key)
        .unwrap_or_else(|| panic!("no option {key}"))
}

fn find_option_prerequisite_opt<'d>(
    data: &'d GameData,
    key: &SelectionOptionKey,
) -> Option<&'d Prerequisite> {
    find_option(data, key).prerequisite.as_ref()
}

fn find_option_prerequisite<'d>(data: &'d GameData, key: &SelectionOptionKey) -> &'d Prerequisite {
    find_option_prerequisite_opt(data, key).unwrap_or_else(|| panic!("{key}: no prerequisite"))
}

fn names_feature(prerequisite: &Prerequisite, feature: &str) -> bool {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => children.iter().any(|c| names_feature(c, feature)),
        Prerequisite::Not(child) => names_feature(child, feature),
        Prerequisite::Feature { feature: f, .. } => f.as_str() == feature,
        _ => false,
    }
}
