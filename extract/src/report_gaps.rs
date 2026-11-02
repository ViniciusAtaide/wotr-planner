//! Ticket 43: the extraction report's items that the saved guide pages settle, applied to finished data.
//!
//! - **Totem rage powers** (`barbarian#rage-power`): '"Totem" powers are exclusive. Selecting one "Totem" power
//!   closes off the other categories.' Each of the four lines' powers (Beast, Celestial, Daemon, Fiend: Lesser,
//!   plain, Greater) needs `Not(SelectionOption(..))` of each of the other lines' nine, beside what it needed
//!   already (the Beast chain). A `Not` per power, rather than one over an `Any`, so a lock names the power held.
//! - **The Eldritch Scoundrel's sneak attack** (`rogue#sneak-attack-eldritch-scoundrel`): "At 1st level, … She does
//!   an additional 1d6 damage at 3rd level, increasing by 1d6 every four additional class levels (7, 11, 15, 19)."
//!   One die at each of 1, 3, 7, 11, 15 and 19 (`game-data/` had them already; an extraction has the 1st only).
//! - **The shaman's Wandering Hex** (`shaman#wandering-hex`): "a hex slot that can be temporarily filled with a hex
//!   granted by either his main spirit or his current Wandering Spirit": a daily choice, as the Wandering Spirit it
//!   draws on.
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
//! The kineticist's bonus-feat wild talent is ticket 45's ([`moved`]).
//!
//! Each rule asserts the guide phrase it rests on and what it finds in the data, so it fails loudly on data it
//! doesn't expect. [`apply`] runs on the finished data of an extraction and, through ticket 43's one-off rewrite, on
//! `game-data/`.

use std::collections::BTreeSet;

use rules::{
    ClassKey, FeatCategory, FeatSlot, FeatureKey, FeatureSelectionKey, GameData, Prerequisite,
    SelectionOptionKey,
};

use crate::follow_ups::{archetype_mut, cell_at, class_mut, says};
use crate::guide::Guide;
use crate::html::Section;
use crate::option_levels::{Log, page};

/// Applies every rule, in the order above: ticket 43's, then [`decisions`].
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    totems(guide, data, &mut log);
    eldritch_scoundrel(guide, data, &mut log);
    wandering_hex(guide, data, &mut log);
    log.extend(decisions(guide, data));
    log
}

/// Ticket 44's rules alone (the user's decisions on ticket 43's questions), which its one-off rewrite applies to
/// `game-data/`, where ticket 43's are in already.
pub fn decisions(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    sorcerer_level_1_feat(guide, data, &mut log);
    extra_slayer_talent(guide, data, &mut log);
    deity(guide);
    log
}

/// The extraction report's items this module settles: the totems' prose rule (ticket 43); the sorcerer archetypes'
/// level-1 "Bonus Feat", Extra Slayer Talent's "Talented Slayer" and the cleric's deity (ticket 44).
pub fn settled(at: &str, text: &str) -> bool {
    match text {
        "" => at == "barbarian#rage-powers",
        "Bonus Feat" => SORCERER_ARCHETYPES_GENERAL_AT_1
            .iter()
            .any(|archetype| at == format!("sorcerer#{archetype}")),
        "Slayer Talents or Talented Slayer" => at == "feats#extra-slayer-talent",
        DEITY => at == "cleric",
        _ => false,
    }
}

/// The extraction report's items a follow-up ticket carries, left out of the report: the kineticist's bonus-feat
/// wild talent, which [ticket 45] builds once wild talents are a selection.
///
/// [ticket 45]: ../../.scratch/wotr-planner/issues/45-kineticist-wild-talents.md
pub fn moved(at: &str, text: &str) -> bool {
    at == "kineticist#kineticist-bonus-feat" && text.is_empty()
}

// ---- totem rage powers ----------------------------------------------------------------------------------------

const RAGE_POWER: &str = "barbarian/rage-power";

/// Each totem power needs none of the other lines' powers.
fn totems(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "barbarian",
        "rage-power",
        "\"Totem\" powers are exclusive. Selecting one \"Totem\" power closes off the other categories.",
    );
    let selection = class_mut(data, "barbarian")
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == RAGE_POWER)
        .unwrap_or_else(|| panic!("no {RAGE_POWER}"));
    // The lines, from the powers' names: "Beast Totem, Lesser", "Beast Totem", "Beast Totem, Greater".
    let line_of = |name: &str| -> Option<String> {
        let line = name
            .strip_suffix(", Lesser")
            .or_else(|| name.strip_suffix(", Greater"))
            .unwrap_or(name);
        line.strip_suffix(" Totem").map(str::to_owned)
    };
    let totems: Vec<(String, SelectionOptionKey)> = selection
        .options
        .iter()
        .filter_map(|option| Some((line_of(&option.name)?, option.key.clone())))
        .collect();
    let lines: BTreeSet<&str> = totems.iter().map(|(line, _)| line.as_str()).collect();
    assert_eq!(
        lines,
        BTreeSet::from(["Beast", "Celestial", "Daemon", "Fiend"]),
        "the totem lines"
    );
    assert_eq!(totems.len(), 12, "three powers a line");
    for option in &mut selection.options {
        let Some(line) = line_of(&option.name) else {
            continue;
        };
        assert!(
            !has_not(option.prerequisite.as_ref()),
            "{}: a Not already",
            option.key
        );
        let nots: Vec<Prerequisite> = totems
            .iter()
            .filter(|(other, _)| *other != line)
            .map(|(_, key)| Prerequisite::Not(Box::new(Prerequisite::SelectionOption(key.clone()))))
            .collect();
        let count = nots.len();
        option.prerequisite = Some(match option.prerequisite.take() {
            None => Prerequisite::All(nots),
            Some(Prerequisite::All(mut children)) => {
                children.extend(nots);
                Prerequisite::All(children)
            }
            Some(other) => Prerequisite::All(std::iter::once(other).chain(nots).collect()),
        });
        log.push(format!(
            "{}: needs none of the other lines' {count} totem powers",
            option.key
        ));
    }
}

fn has_not(prerequisite: Option<&Prerequisite>) -> bool {
    match prerequisite {
        Some(Prerequisite::Not(_)) => true,
        Some(Prerequisite::All(children) | Prerequisite::Any(children))
        | Some(Prerequisite::AtLeast(_, children)) => {
            children.iter().any(|child| has_not(Some(child)))
        }
        _ => false,
    }
}

// ---- the Eldritch Scoundrel's sneak attack --------------------------------------------------------------------

const ELDRITCH_SCOUNDREL: &str = "rogue/eldritch-scoundrel";
const ELDRITCH_SNEAK_ATTACK: &str = "rogue/eldritch-scoundrel/sneak-attack-eldritch-scoundrel";

/// A die of the archetype's own sneak attack at 1, 3 and each level the guide's brackets list.
fn eldritch_scoundrel(guide: &Guide, data: &mut GameData, log: &mut Log) {
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
    let feature = FeatureKey::new(ELDRITCH_SNEAK_ATTACK);
    let archetype = archetype_mut(data, "rogue", ELDRITCH_SCOUNDREL);
    assert!(
        archetype.features.iter().any(|f| f.key == feature),
        "{ELDRITCH_SNEAK_ATTACK} isn't defined"
    );
    for row in &mut archetype.levels {
        let held = row.features.iter().filter(|f| **f == feature).count();
        if levels.contains(&row.level) {
            assert!(held <= 1, "{ELDRITCH_SCOUNDREL} {}: {held} dice", row.level);
            if held == 0 {
                row.features.push(feature.clone());
                log.push(format!(
                    "{ELDRITCH_SCOUNDREL} {}: a die of {ELDRITCH_SNEAK_ATTACK}",
                    row.level
                ));
            }
        } else {
            assert_eq!(
                held, 0,
                "{ELDRITCH_SCOUNDREL} {}: a die off the guide's",
                row.level
            );
        }
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
fn wandering_hex(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "shaman",
        "wandering-hex",
        "At 6th and 14th levels, a Shaman gains a hex slot that can be temporarily filled with a hex granted by \
         either his main spirit or his current Wandering Spirit.",
    );
    let shaman = class_mut(data, "shaman");
    let granted: Vec<u8> = shaman
        .levels
        .iter()
        .filter(|row| row.features.iter().any(|f| f.as_str() == WANDERING_HEX))
        .map(|row| row.level)
        .collect();
    assert_eq!(granted, [6, 14], "{WANDERING_HEX}'s rows");
    let spirit = shaman
        .features
        .iter()
        .find(|f| f.key.as_str() == "shaman/wandering-spirit")
        .expect("the Wandering Spirit");
    assert!(
        spirit.daily_choice,
        "the Wandering Spirit is a daily choice"
    );
    let feature = shaman
        .features
        .iter_mut()
        .find(|f| f.key.as_str() == WANDERING_HEX)
        .unwrap_or_else(|| panic!("no {WANDERING_HEX}"));
    assert!(
        !feature.daily_choice,
        "{WANDERING_HEX}: a daily choice already"
    );
    feature.daily_choice = true;
    log.push(format!("{WANDERING_HEX}: a daily choice"));
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

/// The level-1 bonus feat of the base sorcerer and each archetype that has it, from the held bloodline's list.
fn sorcerer_level_1_feat(guide: &Guide, data: &mut GameData, log: &mut Log) {
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

    let slot = FeatSlot::OptionList {
        selection: FeatureSelectionKey::new(SORCERER_BLOODLINE),
        at_level: None,
    };
    let sorcerer = class_mut(data, "sorcerer");
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

    // The base sorcerer: no slot at 1 yet; it picks its bloodline there.
    let row = sorcerer
        .levels
        .iter_mut()
        .find(|row| row.level == 1)
        .expect("the sorcerer's row 1");
    assert_eq!(row.feat_slots, [], "sorcerer 1: a feat slot already");
    assert!(
        row.selections
            .iter()
            .any(|selection| selection.as_str() == SORCERER_BLOODLINE),
        "sorcerer 1 picks no bloodline"
    );
    row.feat_slots.push(slot.clone());
    log.push("sorcerer 1: a bonus feat from the held bloodline's list".to_owned());

    for archetype in &mut sorcerer.archetypes {
        let name = archetype
            .key
            .as_str()
            .strip_prefix("sorcerer/")
            .unwrap()
            .to_owned();
        // A selection the archetype's bloodline comes from: the sorcerer's, or the archetype's own over the
        // sorcerer's bloodlines (the Empyreal's Celestial, the Sage's Arcane, the Sylvan's Fey, the Seeker's any).
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
        let picks_bloodline = archetype
            .levels
            .iter()
            .find(|row| row.level == 1)
            .is_some_and(|row| row.selections.iter().any(own_bloodline));
        let row = archetype
            .levels
            .iter_mut()
            .find(|row| row.level == 1)
            .unwrap_or_else(|| panic!("{name} has no row 1"));
        let before = match name.as_str() {
            "overwhelming-mage" => vec![],
            "seeker-sorcerer" => vec![FeatSlot::Category(FeatCategory::Magic)],
            _ => vec![FeatSlot::General],
        };
        assert_eq!(row.feat_slots, before, "{name} 1's slots");
        match name.as_str() {
            "overwhelming-mage" | "geomancer" => continue,
            _ => {}
        }
        assert!(picks_bloodline, "{name} 1 holds no bloodline");
        row.feat_slots = vec![slot.clone()];
        log.push(format!(
            "{} 1: the bonus feat from the held bloodline's list (was {:?})",
            archetype.key, before[0]
        ));
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

/// Extra Slayer Talent needs slayer level 2, the level the slayer's table first gives Slayer Talents.
fn extra_slayer_talent(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "feats",
        "extra-slayer-talent",
        "Slayer Talents or Talented Slayer",
    );
    // The slayer's tables: saves and BAB, then the features by level.
    let first = named_section(guide, "slayer", "Slayer")
        .tables()
        .flatten()
        .find(|row| row.iter().skip(1).any(|cell| cell == "Slayer Talents"))
        .and_then(|row| row.first())
        .expect("Slayer Talents in the slayer's table");
    assert_eq!(first, "2", "the slayer's first Slayer Talents");
    let level: u8 = first.parse().unwrap();
    let slayer = class_mut(data, "slayer");
    let first_row = slayer
        .levels
        .iter()
        .find(|row| {
            row.selections
                .iter()
                .any(|selection| selection.as_str() == "slayer/slayer-talents")
        })
        .map(|row| row.level);
    assert_eq!(
        first_row,
        Some(level),
        "the slayer's first slayer-talents row"
    );
    let feat = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "extra-slayer-talent")
        .expect("Extra Slayer Talent");
    assert_eq!(
        feat.prerequisite, None,
        "extra-slayer-talent: a prerequisite already"
    );
    feat.prerequisite = Some(Prerequisite::ClassLevel {
        class: ClassKey::new("slayer"),
        level,
    });
    log.push(format!("extra-slayer-talent: needs slayer level {level}"));
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
