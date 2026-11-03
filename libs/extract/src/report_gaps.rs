//! Ticket 43: the extraction report's items that the saved guide pages settle, built where their owners are.
//!
//! - **Totem rage powers** (`barbarian#rage-power`): '"Totem" powers are exclusive. Selecting one "Totem" power
//!   closes off the other categories.' Each of the four lines' powers (Beast, Celestial, Daemon, Fiend: Lesser,
//!   plain, Greater) needs `Not(SelectionOption(..))` of each of the other lines' nine, beside what it needed
//!   already (the Beast chain). A `Not` per power, rather than one over an `Any`, so a lock names the power held.
//!   Stated where the rage powers are built ([`provide`]).
//! - **The Eldritch Scoundrel's sneak attack** (`rogue#sneak-attack-eldritch-scoundrel`): "At 1st level, … She does
//!   an additional 1d6 damage at 3rd level, increasing by 1d6 every four additional class levels (7, 11, 15, 19)."
//!   One die at each of 1, 3, 7, 11, 15 and 19 (`libs/game-data/` had them already; the table names the 1st only):
//!   each of those rows holds one ([`Picks::row_feature`]).
//! - **The shaman's Wandering Hex** (`shaman#wandering-hex`): "a hex slot that can be temporarily filled with a hex
//!   granted by either his main spirit or his current Wandering Spirit": a daily choice, as the Wandering Spirit it
//!   draws on. Built so where the shaman's table defines it ([`provide`]).
//!
//! The report's other items (the sorcerer archetypes' level-1 bonus feat, the kineticist's bonus-feat wild talent,
//! Talented Slayer, the cleric's deity) aren't settled by the saved pages: they were ticket 43's questions. Ticket 44
//! builds the user's decisions on three of them (2026-11-02):
//!
//! - **The sorcerer's level-1 bonus feat** (`sorcerer#crossblooded`, `#empyreal-sorcerer`, `#geomancer`,
//!   `#nine-tailed-heir`, `#sage-sorcerer`, `#sylvan-sorcerer`): every sorcerer table but the Overwhelming Mage's
//!   ("Does not receive bonus feats.") has "Bonus Feat" at 1, in the column of the "Bloodline Feat" at 7, 13 and 19.
//!   Every sorcerer, base included, takes it from the held bloodline's list (`OptionList(selection:
//!   "sorcerer/bloodline")`, as at 7, 13 and 19): Crossblooded from either bloodline's ("at the bonus feat levels,
//!   you may choose a feat from one bloodline or the other"), the Empyreal, Sage and Sylvan from their fixed
//!   bloodline's, the Seeker and Nine Tailed Heir from the one they choose "for the purpose of bonus feats" (the
//!   Seeker's magic feats are its 5th, 10th and 15th). The Geomancer, which "does not gain bloodline spells or
//!   bloodline feats", keeps its General slot; the Overwhelming Mage keeps none. The base sorcerer gains the slot.
//! - **Extra Slayer Talent** (`feats#extra-slayer-talent`): "Slayer Talents or Talented Slayer" reads as slayer
//!   level 2, the level the slayer's table first gives Slayer Talents. Talented Slayer is on no saved page.
//! - **The cleric's deity** (`cleric`): "Alignment must be within 1 place of their chosen deity." Deities are out of
//!   scope; the rule changes no data, and the report item goes.
//!
//! The kineticist's bonus-feat wild talent is ticket 45's: [`crate::wild_talents`] settles it.
//!
//! The sorcerer's level-1 slot is the base table's row's and the archetypes' "Bonus Feat" cell at 1; Extra Slayer
//! Talent's line is stated in place.
//!
//! Each rule asserts the guide phrase it rests on when the providers are read ([`provide`]); [`check`] asserts what
//! only the finished data shows.

use std::collections::BTreeSet;

use rules::{
    ClassKey, DataAt, FeatCategory, FeatSlot, FeatureKey, FeatureSelectionKey, GameData,
    Prerequisite, SelectionOptionKey,
};

use crate::follow_ups::{cell_at, says};
use crate::guide::Guide;
use crate::html::Section;
use crate::option_levels::page;
use crate::picks::{Pick, Picks, Rule};
use crate::report::Settled;

/// The extraction report's items this module settles: the totems' prose rule (ticket 43); the Geomancer's level-1
/// "Bonus Feat" and the cleric's deity (ticket 44). The other sorcerer archetypes' "Bonus Feat" and Extra Slayer
/// Talent's line are no longer reported (ticket 71 reads the cell and states the line), so nothing settles them.
pub fn settles() -> Vec<Settled> {
    vec![
        Settled::is("barbarian#rage-powers", ""),
        Settled::is("sorcerer#geomancer", "Bonus Feat"),
        Settled::is("cleric", DEITY),
    ]
}

/// Whether a report item is one this module settles ([`settles`]).
pub fn settled(at: &str, text: &str) -> bool {
    settles().iter().any(|entry| entry.matches(at, text))
}

// ---- totem rage powers ----------------------------------------------------------------------------------------

const RAGE_POWER: &str = "barbarian/rage-power";

/// Each totem power needs none of the other lines' powers.
/// The totem rage powers, stated where the barbarian's rage powers are built: each needs none of the other lines'
/// powers, beside what its own line names.
fn totems(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "barbarian",
        "rage-power",
        "\"Totem\" powers are exclusive. Selecting one \"Totem\" power closes off the other categories.",
    );
    let heading = crate::option_levels::section(page(guide, "barbarian"), "rage-power");
    // The lines, from the powers' names: "Beast Totem, Lesser", "Beast Totem", "Beast Totem, Greater".
    let line_of = |name: &str| -> Option<String> {
        let line = name
            .strip_suffix(", Lesser")
            .or_else(|| name.strip_suffix(", Greater"))
            .unwrap_or(name);
        line.strip_suffix(" Totem").map(str::to_owned)
    };
    let totems: Vec<(String, SelectionOptionKey)> = heading
        .children
        .iter()
        .filter_map(|power| {
            Some((
                line_of(&power.name)?,
                SelectionOptionKey::new(format!("{RAGE_POWER}/{}", power.anchor)),
            ))
        })
        .collect();
    let lines: BTreeSet<&str> = totems.iter().map(|(line, _)| line.as_str()).collect();
    assert_eq!(
        lines,
        BTreeSet::from(["Beast", "Celestial", "Daemon", "Fiend"]),
        "the totem lines"
    );
    assert_eq!(totems.len(), 12, "three powers a line");
    for (line, key) in &totems {
        for (_, other) in totems.iter().filter(|(other, _)| other != line) {
            picks.stated(
                DataAt::SelectionOption(key.clone()),
                Prerequisite::Not(Box::new(Prerequisite::SelectionOption(other.clone()))),
            );
        }
    }
}

// ---- the Eldritch Scoundrel's sneak attack --------------------------------------------------------------------

const ELDRITCH_SCOUNDREL: &str = "rogue/eldritch-scoundrel";
const ELDRITCH_SNEAK_ATTACK: &str = "rogue/eldritch-scoundrel/sneak-attack-eldritch-scoundrel";

/// The levels of the archetype's own sneak attack's dice: 1, 3 and each level the guide's brackets list.
fn eldritch_scoundrel_levels(guide: &Guide) -> Vec<u8> {
    let first = "At 1st level, the Eldritch Scoundrel deals extra damage when her opponent is denied Dexterity bonus \
                 or when she flanks her opponent.";
    let later = "She does an additional 1d6 damage at 3rd level, increasing by 1d6 every four additional class \
                 levels (7, 11, 15, 19).";
    says(guide, "rogue", "sneak-attack-eldritch-scoundrel", first);
    says(guide, "rogue", "sneak-attack-eldritch-scoundrel", later);
    let bracketed: Vec<u8> = later
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'))
        .map(|(list, _)| list.split(", ").map(|n| n.parse().unwrap()).collect())
        .unwrap();
    let levels: Vec<u8> = [1, 3].into_iter().chain(bracketed).collect();
    assert_eq!(levels, [1, 3, 7, 11, 15, 19]);
    levels
}

/// Each of those rows in force holds one die, and no other row holds one.
fn check_eldritch_scoundrel(data: &GameData) {
    let levels = [1, 3, 7, 11, 15, 19];
    let feature = FeatureKey::new(ELDRITCH_SNEAK_ATTACK);
    let archetype = data
        .classes
        .iter()
        .flat_map(|class| &class.archetypes)
        .find(|a| a.key.as_str() == ELDRITCH_SCOUNDREL)
        .expect("the Eldritch Scoundrel");
    assert!(
        archetype.features.iter().any(|f| f.key == feature),
        "{ELDRITCH_SNEAK_ATTACK} isn't defined"
    );
    for row in &archetype.levels {
        let held = row.features.iter().filter(|f| **f == feature).count();
        let expected = usize::from(levels.contains(&row.level));
        assert_eq!(
            held, expected,
            "{ELDRITCH_SCOUNDREL} {}: {held} dice",
            row.level
        );
    }
    for level in levels {
        assert!(
            archetype.levels.iter().any(|row| row.level == level),
            "{ELDRITCH_SCOUNDREL} has no row {level}"
        );
    }
}

// ---- the shaman's Wandering Hex -------------------------------------------------------------------------------

const WANDERING_HEX: &str = "shaman/wandering-hex";

/// The Wandering Hex is a daily choice.
/// The totem rage powers' exclusion, and the Wandering Hex a daily choice, built where their owners are.
pub(crate) fn provide(guide: &Guide, picks: &mut Picks) {
    totems(guide, picks);
    for level in eldritch_scoundrel_levels(guide) {
        picks.row_feature(ELDRITCH_SCOUNDREL, level, ELDRITCH_SNEAK_ATTACK);
    }
    sorcerer_level_1_feat(guide, picks);
    extra_slayer_talent(guide, picks);
    deity(guide);
    says(
        guide,
        "shaman",
        "wandering-hex",
        "At 6th and 14th levels, a Shaman gains a hex slot that can be temporarily filled with a hex granted by \
         either his main spirit or his current Wandering Spirit.",
    );
    picks.daily_choice(WANDERING_HEX);
}

/// What the owners [`provide`] gave facts to hold once built: the Eldritch Scoundrel's dice, the sorcerers' level-1
/// slots, Extra Slayer Talent's level, and the shaman's rows grant the Wandering Hex at 6 and 14, which draws on the
/// Wandering Spirit, a daily choice too. Changes nothing.
pub(crate) fn check(data: &GameData) {
    check_eldritch_scoundrel(data);
    check_sorcerer_level_1_feat(data);
    check_extra_slayer_talent(data);
    let shaman = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "shaman")
        .expect("the shaman");
    let granted: Vec<u8> = shaman
        .levels
        .iter()
        .filter(|row| row.features.iter().any(|f| f.as_str() == WANDERING_HEX))
        .map(|row| row.level)
        .collect();
    assert_eq!(granted, [6, 14], "{WANDERING_HEX}'s rows");
    for key in ["shaman/wandering-spirit", WANDERING_HEX] {
        let feature = shaman
            .features
            .iter()
            .find(|f| f.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"));
        assert!(feature.daily_choice, "{key} is a daily choice");
    }
}

// ---- the sorcerer's level-1 bonus feat ------------------------------------------------------------------------

const SORCERER_BLOODLINE: &str = "sorcerer/bloodline";

/// The archetypes whose level-1 "Bonus Feat" an extraction stores as a general slot (and reports).
const SORCERER_ARCHETYPES_GENERAL_AT_1: [&str; 6] = [
    "crossblooded",
    "empyreal-sorcerer",
    "geomancer",
    "nine-tailed-heir",
    "sage-sorcerer",
    "sylvan-sorcerer",
];

/// The level-1 bonus feat of the base sorcerer and each archetype that has it, from the held bloodline's list: the
/// base sorcerer's row gains the slot, and the archetypes' "Bonus Feat" cell at 1 is it (the Geomancer's stays a
/// general slot; the Overwhelming Mage has none).
fn sorcerer_level_1_feat(guide: &Guide, picks: &mut Picks) {
    for anchor in [
        "abyssal",
        "arcane",
        "celestial",
        "draconic",
        "elemental",
        "fey",
        "infernal",
        "serpentine",
        "undead",
        "crossblooded",
        "empyreal-sorcerer",
        "geomancer",
        "nine-tailed-heir",
        "sage-sorcerer",
        "seeker-sorcerer",
        "sylvan-sorcerer",
    ] {
        cell_at(guide, "sorcerer", anchor, "1", "Bonus Feat");
    }
    says(
        guide,
        "sorcerer",
        "bloodline-2",
        "Note that at the bonus feat levels, you may choose a feat from one bloodline or the other, not both.",
    );
    says(
        guide,
        "sorcerer",
        "nine-tailed-heir",
        "A Nine Tailed Heir chooses a standard bloodline for the purpose of bonus feats and abilities.",
    );
    says(
        guide,
        "sorcerer",
        "seeker-sorcerer",
        "A Seeker chooses a standard bloodline for the purpose of bonus feats and spells.",
    );
    says(
        guide,
        "sorcerer",
        "bonus-feat",
        "At 5th, 10th and 15th levels, a Seeker gains a bonus magic feat.",
    );
    says(
        guide,
        "sorcerer",
        "geomancer",
        "Does not gain bloodline spells or bloodline feats.",
    );
    says(
        guide,
        "sorcerer",
        "overwhelming-mage",
        "Does not receive bonus feats.",
    );
    cell_at(
        guide,
        "sorcerer",
        "overwhelming-mage",
        "1",
        "Overwhelming Penetration",
    );
    let slot = bloodline_slot();
    picks.row_slot("sorcerer", 1, slot.clone());
    for archetype in SORCERER_ARCHETYPES_GENERAL_AT_1
        .into_iter()
        .chain(["seeker-sorcerer"])
        .filter(|archetype| *archetype != "geomancer")
    {
        let table = format!("sorcerer/{archetype}");
        picks.rule(Rule {
            feature: FeatureKey::new(format!("{table}/bonus-feat")),
            table: Some(table),
            cell: None,
            levels: Some(vec![1]),
            pick: Pick::Slots(vec![slot.clone()]),
        });
    }
}

fn bloodline_slot() -> FeatSlot {
    FeatSlot::OptionList {
        selection: FeatureSelectionKey::new(SORCERER_BLOODLINE),
        at_level: None,
        label: None,
    }
}

/// Every sorcerer with the level-1 bonus feat picks it from the held bloodline's list, and holds a bloodline there:
/// the base sorcerer, and each archetype but the Geomancer (a general slot) and the Overwhelming Mage (none). The
/// Seeker keeps its magic feats at 5, 10 and 15.
fn check_sorcerer_level_1_feat(data: &GameData) {
    let slot = bloodline_slot();
    let sorcerer = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "sorcerer")
        .expect("the sorcerer");
    let bloodline = sorcerer
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == SORCERER_BLOODLINE)
        .unwrap_or_else(|| panic!("no {SORCERER_BLOODLINE}"));
    let bloodlines: Vec<SelectionOptionKey> = bloodline
        .options
        .iter()
        .map(|option| option.key.clone())
        .collect();
    let archetypes: Vec<&str> = sorcerer
        .archetypes
        .iter()
        .map(|archetype| archetype.key.as_str())
        .collect();
    assert_eq!(
        archetypes,
        [
            "sorcerer/crossblooded",
            "sorcerer/empyreal-sorcerer",
            "sorcerer/geomancer",
            "sorcerer/nine-tailed-heir",
            "sorcerer/overwhelming-mage",
            "sorcerer/sage-sorcerer",
            "sorcerer/seeker-sorcerer",
            "sorcerer/sylvan-sorcerer",
        ],
        "the sorcerer's archetypes"
    );
    let row = sorcerer
        .levels
        .iter()
        .find(|row| row.level == 1)
        .expect("the sorcerer's row 1");
    assert_eq!(row.feat_slots, std::slice::from_ref(&slot), "sorcerer 1");
    assert!(
        row.selections
            .iter()
            .any(|selection| selection.as_str() == SORCERER_BLOODLINE),
        "sorcerer 1 picks no bloodline"
    );
    for archetype in &sorcerer.archetypes {
        let name = archetype.key.as_str().strip_prefix("sorcerer/").unwrap();
        let own_bloodline = |key: &FeatureSelectionKey| {
            key.as_str() == SORCERER_BLOODLINE
                || archetype.selections.iter().any(|selection| {
                    selection.key == *key
                        && selection.options.is_empty()
                        && !selection.shared_options.is_empty()
                        && selection
                            .shared_options
                            .iter()
                            .all(|option| bloodlines.contains(option))
                })
        };
        let row = archetype
            .levels
            .iter()
            .find(|row| row.level == 1)
            .unwrap_or_else(|| panic!("{name} has no row 1"));
        let expected = match name {
            "overwhelming-mage" => vec![],
            "geomancer" => vec![FeatSlot::General],
            _ => {
                assert!(
                    row.selections.iter().any(own_bloodline),
                    "{name} 1 holds no bloodline"
                );
                vec![slot.clone()]
            }
        };
        assert_eq!(row.feat_slots, expected, "{name} 1's slots");
        if name == "seeker-sorcerer" {
            for level in [5, 10, 15] {
                let row = archetype
                    .levels
                    .iter()
                    .find(|row| row.level == level)
                    .unwrap_or_else(|| panic!("the Seeker has no row {level}"));
                assert_eq!(
                    row.feat_slots,
                    [FeatSlot::Category(FeatCategory::Magic)],
                    "the Seeker's magic feat at {level}"
                );
            }
        }
    }
}

// ---- Extra Slayer Talent --------------------------------------------------------------------------------------

/// Extra Slayer Talent needs slayer level 2, the level the slayer's table first gives Slayer Talents: stated in
/// place of its line, whose "Talented Slayer" names nothing.
fn extra_slayer_talent(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "feats",
        "extra-slayer-talent",
        "Slayer Talents or Talented Slayer",
    );
    picks.prerequisite(
        DataAt::Feat(rules::FeatKey::new("extra-slayer-talent")),
        "Slayer Talents or Talented Slayer",
        Prerequisite::ClassLevel {
            class: ClassKey::new("slayer"),
            level: first_slayer_talents(guide),
        },
    );
}

/// The level the slayer's table first gives Slayer Talents (its tables: saves and BAB, then the features by level).
fn first_slayer_talents(guide: &Guide) -> u8 {
    let first = named_section(guide, "slayer", "Slayer")
        .tables()
        .flatten()
        .find(|row| row.iter().skip(1).any(|cell| cell == "Slayer Talents"))
        .and_then(|row| row.first())
        .expect("Slayer Talents in the slayer's table");
    assert_eq!(first, "2", "the slayer's first Slayer Talents");
    first.parse().unwrap()
}

/// The slayer's first slayer-talents row is the level Extra Slayer Talent needs.
fn check_extra_slayer_talent(data: &GameData) {
    let slayer = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "slayer")
        .expect("the slayer");
    let first_row = slayer
        .levels
        .iter()
        .find(|row| {
            row.selections
                .iter()
                .any(|selection| selection.as_str() == "slayer/slayer-talents")
        })
        .map(|row| row.level);
    let feat = data
        .feats
        .iter()
        .find(|feat| feat.key.as_str() == "extra-slayer-talent")
        .expect("Extra Slayer Talent");
    let level = first_row.expect("the slayer's slayer-talents rows");
    let leaf = Prerequisite::ClassLevel {
        class: ClassKey::new("slayer"),
        level,
    };
    // The Sanctified Slayer's slayer talents widen it later (ticket 60).
    let holds = match &feat.prerequisite {
        Some(Prerequisite::Any(children)) => children.contains(&leaf),
        Some(one) => *one == leaf,
        None => false,
    };
    assert!(holds, "extra-slayer-talent: {:?}", feat.prerequisite);
}

// ---- the cleric's deity ---------------------------------------------------------------------------------------

const DEITY: &str = "Alignment must be within 1 place of their chosen deity.";

/// Deities are out of scope: asserts the sentence the report item quotes and changes no data.
fn deity(guide: &Guide) {
    let text = named_section(guide, "cleric", "Cleric").text();
    assert!(text.contains(DEITY), "cleric: no {DEITY:?}");
}

/// The section of a page with this heading (a class's own section, which has no anchor).
fn named_section<'g>(guide: &'g Guide, slug: &str, name: &str) -> &'g Section {
    page(guide, slug)
        .root
        .walk()
        .into_iter()
        .find(|section| section.name == name)
        .unwrap_or_else(|| panic!("{slug}: no section {name:?}"))
}
