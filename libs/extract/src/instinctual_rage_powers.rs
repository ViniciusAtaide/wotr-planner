//! Ticket 66: ticket 58's decision 3 (user, 2026-11-02). The Instinctual Warrior, a barbarian archetype, may take monk
//! abilities and ki powers as rage powers (`barbarian#rage-power-instinctual-warrior`: "Besides the normal Barbarian
//! rage powers, an Instinctual Warrior can select a number of Monk abilities and Ki powers: Deflect Arrows (requires
//! Improved Unarmed Strike), Purity of Body (requires 8th level), …").
//!
//! - **Five options** of the barbarian's own `barbarian/rage-power`, one per power the sentence names, each needing
//!   the archetype and what its brackets require: a level is a barbarian level (`ClassLevel`), as every rage power's
//!   is; a name is the feat of that name. They are options of their own, not the monk's ki powers, since the levels
//!   differ (the user's decision). Each is named "… (Instinctual Warrior)" after the heading that lists them, and
//!   links to it. The class stage gives them to the selection where it reads the rage powers ([`provide`]).
//! - **Where they're picked**: the archetype's rage power slots (its table's "Rage Power" cells: 4, 8, 10, 12, 16, 18
//!   and 20), and Extra Rage Power's, which offers what the character's rage power slots offer. A character without
//!   the archetype has them locked.
//! - **Extra Rage Power** ("Prerequisites: Rage Power") is held in the data as "has taken one of the rage powers", so
//!   its `Any` holds the five too: an Instinctual Warrior holding only Deflect Arrows has a rage power.
//!
//! Each rule asserts the guide phrase it rests on where the options are read, and [`check`] what it finds in the
//! finished data, so it fails loudly on data it doesn't expect.

use std::collections::BTreeSet;

use rules::{ArchetypeKey, ClassKey, GameData, Prerequisite, SelectionOptionKey};

use crate::borrowed_extra_picks::slot_levels;
use crate::follow_ups::says;
use crate::guide::Guide;
use crate::html::Page;
use crate::mercies_and_exploits::levels_in_tables;
use crate::names::slug;
use crate::option_levels::{FeatNames, option, page, section};
use crate::picks::Picks;

const BARBARIAN: &str = "barbarian";
const ARCHETYPE: &str = "barbarian/instinctual-warrior";
const RAGE_POWER: &str = "barbarian/rage-power";
const EXTRA_RAGE_POWER: &str = "extra-rage-power";
/// The section that lists the powers.
const ANCHOR: &str = "rage-power-instinctual-warrior";
const LEAD: &str = "Besides the normal Barbarian rage powers, an Instinctual Warrior can select a number of Monk \
                    abilities and Ki powers: ";
/// The archetype's rage power levels, by its table.
const LEVELS: [u8; 7] = [4, 8, 10, 12, 16, 18, 20];

/// What one power's brackets require.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Needs {
    /// "requires 8th level".
    Level(u8),
    /// "requires Improved Unarmed Strike": a feat, by its name.
    Feat(String),
}

/// One power the section names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonkPower {
    pub name: String,
    pub needs: Needs,
}

/// The powers `barbarian#rage-power-instinctual-warrior` names, in its order: "Name (requires …)", separated by
/// commas and a last "and", up to the sentence's full stop.
pub fn read_monk_powers(page: &Page) -> Vec<MonkPower> {
    let text = section(page, ANCHOR).text();
    let rest = text
        .split_once(LEAD)
        .unwrap_or_else(|| panic!("{}#{ANCHOR}: no {LEAD:?} in {text:?}", page.slug))
        .1;
    let list = rest
        .split_once(").")
        .map(|(list, _)| format!("{list})"))
        .unwrap_or_else(|| panic!("{}#{ANCHOR}: the list doesn't end", page.slug));
    list.split(", ")
        .flat_map(|item| item.split(" and "))
        .map(|item| {
            let (name, needs) = item
                .strip_suffix(')')
                .and_then(|item| item.split_once(" (requires "))
                .unwrap_or_else(|| panic!("{}#{ANCHOR}: {item:?}", page.slug));
            let level = needs
                .strip_suffix(" level")
                .and_then(|ordinal| {
                    ordinal
                        .strip_suffix("st")
                        .or_else(|| ordinal.strip_suffix("nd"))
                        .or_else(|| ordinal.strip_suffix("rd"))
                        .or_else(|| ordinal.strip_suffix("th"))
                })
                .map(|number| {
                    number
                        .parse()
                        .unwrap_or_else(|_| panic!("{}#{ANCHOR}: level {needs:?}", page.slug))
                });
            MonkPower {
                name: name.to_owned(),
                needs: match level {
                    Some(level) => Needs::Level(level),
                    None => Needs::Feat(needs.to_owned()),
                },
            }
        })
        .collect()
}

/// `barbarian/rage-power/purity-of-body-instinctual-warrior`, "Purity of Body (Instinctual Warrior)".
fn key_and_name(power: &MonkPower) -> (String, String) {
    let name = format!("{} (Instinctual Warrior)", power.name);
    (format!("{RAGE_POWER}/{}", slug(&name)), name)
}

/// The Instinctual Warrior's monk powers are options of the rage powers.
pub(crate) fn provide(guide: &Guide, feats: &FeatNames, picks: &mut Picks) {
    let barbarian_page = page(guide, BARBARIAN);
    says(
        guide,
        BARBARIAN,
        "instinctual-warrior",
        "Gains fewer Rage Powers",
    );
    says(
        guide,
        "feats",
        EXTRA_RAGE_POWER,
        "Prerequisites: Rage Power",
    );
    says(
        guide,
        "feats",
        EXTRA_RAGE_POWER,
        "gain one additional rage power.",
    );
    // The archetype's table: where it picks a rage power.
    let anchor = ARCHETYPE.strip_prefix("barbarian/").expect("its class");
    assert_eq!(
        levels_in_tables(barbarian_page, "Rage Power").get(anchor),
        Some(&LEVELS.to_vec()),
        "{ARCHETYPE}: its table's Rage Power"
    );

    let powers = read_monk_powers(barbarian_page);
    let level = |name: &str, level: u8| MonkPower {
        name: name.to_owned(),
        needs: Needs::Level(level),
    };
    assert_eq!(
        powers,
        [
            MonkPower {
                name: "Deflect Arrows".to_owned(),
                needs: Needs::Feat("Improved Unarmed Strike".to_owned()),
            },
            level("Purity of Body", 8),
            level("Diamond Body", 12),
            level("Diamond Soul", 16),
            level("Perfect Self", 20),
        ],
        "{BARBARIAN}#{ANCHOR}: its powers"
    );

    let archetype = Prerequisite::Archetype(ArchetypeKey::new(ARCHETYPE));
    for power in &powers {
        let needs = match &power.needs {
            Needs::Level(level) => Prerequisite::ClassLevel {
                class: ClassKey::new(BARBARIAN),
                level: *level,
            },
            Needs::Feat(name) => Prerequisite::Feat(
                feats
                    .get(name)
                    .unwrap_or_else(|| panic!("{}: no feat {name:?}", power.name)),
            ),
        };
        let (key, name) = key_and_name(power);
        let mut one = option(&key, &name, barbarian_page.guide(ANCHOR), Vec::new());
        let mut prerequisite = Prerequisite::All(vec![archetype.clone(), needs]);
        prerequisite.canonicalize();
        one.prerequisite = Some(prerequisite);
        picks.option(RAGE_POWER, one);
    }
}

/// The archetype's rage power slots in force are its table's; its powers are new to the selection, and Extra Rage
/// Power is met by holding any rage power, its five too.
pub fn check(guide: &Guide, data: &GameData) {
    let barbarian = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == BARBARIAN)
        .expect("the barbarian");
    assert_eq!(
        slot_levels(barbarian, Some(ARCHETYPE), RAGE_POWER),
        LEVELS,
        "{ARCHETYPE}: its rage power slots"
    );
    let selection = barbarian
        .selections
        .iter()
        .find(|s| s.key.as_str() == RAGE_POWER)
        .unwrap_or_else(|| panic!("no {RAGE_POWER}"));
    for power in read_monk_powers(page(guide, BARBARIAN)) {
        let (key, name) = key_and_name(&power);
        assert!(
            selection.options.iter().any(|o| o.key.as_str() == key),
            "{key}: not an option"
        );
        let plain = name.trim_end_matches(" (Instinctual Warrior)");
        assert!(
            selection.options.iter().all(|o| o.name != plain),
            "{RAGE_POWER}: {plain} already"
        );
    }
    let all: BTreeSet<SelectionOptionKey> =
        selection.options.iter().map(|o| o.key.clone()).collect();
    let feat = data
        .feats
        .iter()
        .find(|feat| feat.key.as_str() == EXTRA_RAGE_POWER)
        .unwrap_or_else(|| panic!("no feat {EXTRA_RAGE_POWER}"));
    let Some(Prerequisite::Any(children)) = &feat.prerequisite else {
        panic!("{EXTRA_RAGE_POWER}: its prerequisite isn't an Any");
    };
    let held: BTreeSet<SelectionOptionKey> = children
        .iter()
        .map(|child| match child {
            Prerequisite::SelectionOption(key) => key.clone(),
            other => panic!("{EXTRA_RAGE_POWER} needs {other:?}"),
        })
        .collect();
    assert_eq!(held, all, "{EXTRA_RAGE_POWER}: the rage powers");
}
