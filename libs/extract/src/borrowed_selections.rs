//! Ticket 60: ticket 54's decision 2 (user, 2026-11-02). Classes and archetypes that pick from another class's list
//! pick there: each such cell is a slot, of the existing selection where the guide's list is one, or of a selection
//! sharing the options of the lists it mixes, as ticket 25 made the slayer's. The class stage reads them so
//! ([`provide`]).
//!
//! - **Bard and skald talents** (`bard#bard-talents`: "The Bard can select from the Rogue Talent list";
//!   `skald#skald-talents`: "The Skald can select from the Rogue Talent list at 2nd, 7th, 12th and 17th levels"): the
//!   "Bard Talent" and "Skald Talent" cells are slots of the rogue's own `rogue/rogue-talents` (the Beast Tamer from 6,
//!   the Inciter at 4, 9, 14, 19).
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
//! What widens another owner's facts (the rogue's and the slayer's option levels, the advanced talents' and the major
//! and grand hexes' prerequisites, Extra Rogue Talent's and Extra Slayer Talent's, the slayer's combat style feat) is
//! [`apply`], on the finished data: tickets 44 and 47 set those facts there first, and this widens what they set.
//!
//! Each rule asserts the guide phrase it rests on where it's read, and [`check`] what it finds in the finished data,
//! so it fails loudly on data it doesn't expect.

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
use crate::picks::{Pick, Picks, Rule};

const ROGUE_TALENTS: &str = "rogue/rogue-talents";
const ROGUE_ADVANCED: &str = "rogue/advanced-talents";
const SLAYER_TALENTS: &str = "slayer/slayer-talents";
const WITCH_HEX: &str = "witch/hex";
const MAGUS_ARCANA: &str = "magus/magus-arcana";
const DISCOVERY: &str = "alchemist/discovery";

const ARCHAEOLOGIST_ADVANCED: &str = "bard/archaeologist/advanced-talents";
const FEYFORM: &str = "shifter/feyform-shifter";
const FEYFORM_TALENT: &str = "shifter/feyform-shifter/rogue-talent";
const VIVISECTIONIST: &str = "alchemist/vivisectionist";
const MEDICAL_DISCOVERY: &str = "alchemist/vivisectionist/medical-discovery";
const VIVISECTIONIST_ADVANCED: &str = "alchemist/vivisectionist/advanced-talents";
const SANCTIFIED_FEATURE: &str = "inquisitor/sanctified-slayer/slayer-talents";
const SANCTIFIED: &str = "inquisitor/sanctified-slayer";
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

/// The borrowed lists' cells are picks, and the archetypes that mix lists have their own sharing selections.
pub fn provide(guide: &Guide, picks: &mut Picks) {
    let rogue = RogueTalents::read(guide);
    bards_and_skalds(guide, picks);
    feyform(guide, picks, &rogue);
    vivisectionist(guide, picks, &rogue);
    sanctified_slayer(guide, picks);
    hexcrafter(guide, picks);
}

/// Widens the facts of other owners the borrowed picks reach: the selections' option levels, the advanced talents'
/// and the major and grand hexes' prerequisites, and the Extra feats.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    let rogue = RogueTalents::read(guide);
    for class in ["bard", "skald"] {
        add_option_level(data, ROGUE_TALENTS, class, &mut log);
    }
    open_advanced(data, &rogue.advanced, ARCHAEOLOGIST_ADVANCED, &mut log);
    extra_rogue_talent(guide, data, &mut log);
    open_advanced(
        data,
        &medical_discoveries(guide, &rogue).2,
        VIVISECTIONIST_ADVANCED,
        &mut log,
    );
    add_option_level(data, SLAYER_TALENTS, "inquisitor", &mut log);
    extra_slayer_talent(guide, data, &mut log);
    combat_style(guide, data, &mut log);
    open_hexes(data, &mut log);
    log
}

/// The guide's lists and the data's agree, and the rows in force hold each pick where the tables have its cell.
pub fn check(guide: &Guide, data: &GameData) {
    let rogue = RogueTalents::read(guide);
    rogue.check(data);
    for (class, cell) in [("bard", "Bard Talent"), ("skald", "Skald Talent")] {
        assert_in_force(guide, data, class, ROGUE_TALENTS, cell);
    }
    assert_in_force(guide, data, "shifter", FEYFORM_TALENT, "Rogue Talent");
    assert_eq!(
        find_selection(data, FEYFORM_TALENT).shared_options,
        &rogue.all - &rogue.advanced,
        "{FEYFORM_TALENT}: the rogue talents but the advanced ones"
    );
    let (discoveries, talents, advanced) = medical_discoveries(guide, &rogue);
    let alchemist: BTreeSet<SelectionOptionKey> = find_selection(data, DISCOVERY)
        .options
        .iter()
        .map(|o| o.key.clone())
        .collect();
    assert!(
        discoveries.is_subset(&alchemist),
        "{DISCOVERY}: the Vivisectionist's"
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
    assert_in_force(
        guide,
        data,
        "alchemist",
        MEDICAL_DISCOVERY,
        "Medical Discovery",
    );
    assert_in_force(guide, data, "inquisitor", SLAYER_TALENTS, "Slayer Talent");
    let first = effective_row_levels(data, "inquisitor", SANCTIFIED, SLAYER_TALENTS);
    assert_eq!(
        first.first(),
        Some(&SANCTIFIED_FIRST),
        "{SANCTIFIED}'s first"
    );

    assert_in_force(guide, data, "magus", WITCH_HEX, "Hex Magus");
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
    let shared: BTreeSet<SelectionOptionKey> = arcana
        .options
        .iter()
        .chain(&hexes.options)
        .map(|o| o.key.clone())
        .collect();
    assert_eq!(
        find_selection(data, HEX_ARCANA).shared_options,
        shared,
        "{HEX_ARCANA}: the magus arcana and the witch hexes"
    );
    assert_in_force(guide, data, "magus", HEX_ARCANA, "Hex Arcana");
    // The Hexcrafter's arcana are all its Hex Arcana: the rows in force hold no magus arcana slot.
    assert_in_force(guide, data, "magus", MAGUS_ARCANA, "Magus Arcana");
}

/// The rogue talents and the advanced ones among them, by the keys the class stage gives the guide's two lists
/// (`rogue#rogue-talents`, and `rogue#advanced-rogue-talents`, which joins it), with their names.
struct RogueTalents {
    all: BTreeSet<SelectionOptionKey>,
    advanced: BTreeSet<SelectionOptionKey>,
    by_name: BTreeMap<String, SelectionOptionKey>,
}

impl RogueTalents {
    fn read(guide: &Guide) -> Self {
        says(
            guide,
            "rogue",
            "advanced-rogue-talents",
            "These unlock at 10th level.",
        );
        let rogue_page = page(guide, "rogue");
        let plain = option_keys(rogue_page, "rogue-talents", ROGUE_TALENTS);
        let advanced = option_keys(rogue_page, "advanced-rogue-talents", ROGUE_TALENTS);
        assert_eq!(
            (plain.len(), advanced.len()),
            (20, 16),
            "the guide's talents"
        );
        let by_name: BTreeMap<String, SelectionOptionKey> =
            plain.clone().into_iter().chain(advanced.clone()).collect();
        Self {
            all: by_name.values().cloned().collect(),
            advanced: advanced.into_values().collect(),
            by_name,
        }
    }

    /// The data's rogue talents are these, and the advanced ones exactly those it gates on the rogue's Advanced
    /// Talents.
    fn check(&self, data: &GameData) {
        let selection = find_selection(data, ROGUE_TALENTS);
        let keys: BTreeSet<SelectionOptionKey> =
            selection.options.iter().map(|o| o.key.clone()).collect();
        assert_eq!(keys, self.all, "{ROGUE_TALENTS}: the guide's talents");
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
        assert_eq!(
            gated, self.advanced,
            "the talents gated on {ROGUE_ADVANCED}"
        );
    }
}

/// The options the class stage makes of a heading's children in `selection`, by name: `<selection>/<anchor>`.
fn option_keys(page: &Page, anchor: &str, selection: &str) -> BTreeMap<String, SelectionOptionKey> {
    section(page, anchor)
        .children
        .iter()
        .map(|child| {
            (
                child.name.clone(),
                SelectionOptionKey::new(format!("{selection}/{}", child.anchor)),
            )
        })
        .collect()
}

/// `feature`'s cells, in every table naming it, are slots of `selection`.
fn slots(picks: &mut Picks, feature: &str, selection: &str) {
    picks.rule(Rule::every(
        feature,
        Pick::Selection(FeatureSelectionKey::new(selection)),
    ));
}

fn bards_and_skalds(guide: &Guide, picks: &mut Picks) {
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
    slots(picks, "bard/bard-talents", ROGUE_TALENTS);
    slots(picks, "skald/skald-talents", ROGUE_TALENTS);

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
}

fn feyform(guide: &Guide, picks: &mut Picks, rogue: &RogueTalents) {
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
    chosen.shared_options = &rogue.all - &rogue.advanced;
    add_selection(picks, FEYFORM, chosen);
    slots(picks, FEYFORM_TALENT, FEYFORM_TALENT);
}

/// "Prerequisites: Rogue Talents or Fey Tricks": the Feyform Shifter's Rogue Talent is one.
fn extra_rogue_talent(guide: &Guide, data: &mut GameData, log: &mut Log) {
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

/// The Vivisectionist's three lists, by the keys the class stage gives them: the discoveries, the rogue talents and
/// the advanced ones.
fn medical_discoveries(
    guide: &Guide,
    rogue: &RogueTalents,
) -> (
    BTreeSet<SelectionOptionKey>,
    BTreeSet<SelectionOptionKey>,
    BTreeSet<SelectionOptionKey>,
) {
    let alchemist_page = page(guide, "alchemist");
    let lists = read_medical_discoveries(alchemist_page);
    let headings: Vec<&str> = lists.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(
        headings,
        ["Discoveries", "Rogue Talents", "Advanced Rogue Talents"],
        "alchemist#medical-discovery's lists"
    );
    let discoveries = option_keys(alchemist_page, "discovery", DISCOVERY);
    let resolve = |options: &BTreeMap<String, SelectionOptionKey>,
                   names: &[String]|
     -> BTreeSet<SelectionOptionKey> {
        names
            .iter()
            .flat_map(|name| {
                let found: Vec<SelectionOptionKey> = options
                    .iter()
                    .filter(|(option, _)| {
                        *option == name
                            // "Focusing Attacks": the three "Focusing Attack - …".
                            || (name.ends_with('s')
                                && option.starts_with(&format!("{} - ", &name[..name.len() - 1])))
                    })
                    .map(|(_, key)| key.clone())
                    .collect();
                assert!(!found.is_empty(), "no {name:?}");
                found
            })
            .collect()
    };
    let discoveries = resolve(&discoveries, &lists[0].1);
    let talents = resolve(&rogue.by_name, &lists[1].1);
    let advanced = resolve(&rogue.by_name, &lists[2].1);
    assert_eq!(
        (discoveries.len(), talents.len(), advanced.len()),
        (14, 8, 3),
        "the Vivisectionist's lists"
    );
    assert!(advanced.is_subset(&rogue.advanced), "advanced talents");
    assert!(talents.is_disjoint(&rogue.advanced), "plain talents");
    (discoveries, talents, advanced)
}

fn vivisectionist(guide: &Guide, picks: &mut Picks, rogue: &RogueTalents) {
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
    let (discoveries, talents, advanced) = medical_discoveries(guide, rogue);
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
    add_selection(picks, VIVISECTIONIST, chosen);
    slots(picks, MEDICAL_DISCOVERY, MEDICAL_DISCOVERY);
}

fn sanctified_slayer(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "inquisitor",
        "slayer-talents",
        "At 8th, 16th, 17th and 20th levels, a Sanctified Slayer can select a Slayer talent.",
    );
    let name = &section(page(guide, "inquisitor"), "slayer-talents").name;
    assert_eq!(name, "Slayer Talents", "inquisitor#slayer-talents");
    slots(picks, SANCTIFIED_FEATURE, SLAYER_TALENTS);
}

/// "Prerequisites: Slayer Talents or Talented Slayer": the slayer's from 2 (ticket 44), the Sanctified Slayer's from 8.
fn extra_slayer_talent(guide: &Guide, data: &mut GameData, log: &mut Log) {
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
        slayer_2,
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
}

/// The combat style feats are the slayer's own windows by slayer level (the note under Slayer Camouflage).
fn combat_style(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "slayer",
        "slayer-camouflage",
        "he can only do so three times in total and at certain levels: 2 or 4, 6 or 8 and 10 or greater.",
    );
    let slayer_2 = Prerequisite::ClassLevel {
        class: ClassKey::new("slayer"),
        level: 2,
    };
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

fn hexcrafter(guide: &Guide, picks: &mut Picks) {
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
    slots(picks, HEX_MAGUS, WITCH_HEX);

    let mut chosen = selection(
        HEX_ARCANA,
        &section(magus_page, "hex-arcana").name,
        magus_page.guide("hex-arcana"),
        &[],
    );
    let witch_page = page(guide, "witch");
    chosen.shared_options = option_keys(magus_page, "magus-arcana", MAGUS_ARCANA)
        .into_values()
        .chain(
            ["hex", "major-hexes", "grand-hexes"]
                .into_iter()
                .flat_map(|anchor| option_keys(witch_page, anchor, WITCH_HEX).into_values()),
        )
        .collect();
    add_selection(picks, HEXCRAFTER, chosen);
    slots(picks, HEX_ARCANA, HEX_ARCANA);
    assert!(
        !levels_in_tables(magus_page, "Magus Arcana").contains_key("hexcrafter"),
        "magus#hexcrafter: Magus Arcana"
    );
}

/// The major and grand hexes open to a Hexcrafter of magus 12 and 18, beside the witch's and the Sylvan Trickster's
/// own features.
fn open_hexes(data: &mut GameData, log: &mut Log) {
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

fn add_selection(picks: &mut Picks, archetype: &str, chosen: FeatureSelection) {
    assert!(
        chosen.options.is_empty(),
        "{}: options of its own",
        chosen.key
    );
    picks.selection(archetype, chosen);
}

/// The levels whose row in force for the archetype holds one slot of `selection`.
fn effective_row_levels(data: &GameData, class: &str, archetype: &str, selection: &str) -> Vec<u8> {
    let class = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == class)
        .unwrap_or_else(|| panic!("no class {class}"));
    let archetype = class
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == archetype)
        .unwrap_or_else(|| panic!("no {archetype}"));
    let selection = FeatureSelectionKey::new(selection);
    (1..=20)
        .filter(|level| {
            effective_row(class, Some(archetype), *level)
                .is_some_and(|row| row.selections.iter().filter(|s| **s == selection).count() == 1)
        })
        .collect()
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
