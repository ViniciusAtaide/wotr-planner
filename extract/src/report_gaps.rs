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
//! Talented Slayer, the cleric's deity) aren't settled by the saved pages: they are ticket 43's questions.
//!
//! Each rule asserts the guide phrase it rests on and what it finds in the data, so it fails loudly on data it
//! doesn't expect. [`apply`] runs on the finished data of an extraction and, through ticket 43's one-off rewrite, on
//! `game-data/`.

use std::collections::BTreeSet;

use rules::{FeatureKey, GameData, Prerequisite, SelectionOptionKey};

use crate::follow_ups::{archetype_mut, class_mut, says};
use crate::guide::Guide;
use crate::option_levels::Log;

/// Applies every rule, in the order above.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    totems(guide, data, &mut log);
    eldritch_scoundrel(guide, data, &mut log);
    wandering_hex(guide, data, &mut log);
    log
}

/// The extraction report's items this module settles: the totems' prose rule.
pub fn settled(at: &str, text: &str) -> bool {
    at == "barbarian#rage-powers" && text.is_empty()
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
