//! Options with rows by option level (ticket 30): sorcerer and bloodrager bloodlines, wizard schools, cavalier
//! orders, shaman spirits, the Magic Deceiver's ways and the ranger's combat styles, plus Dragon Disciple, Eldritch
//! Scion and Dragonheir Scion.
//!
//! Each option's rows come from the guide: from its progression table where it has one (the cells every option's
//! table shares at a level are the class's, the rest are the option's; bloodline spells stay out), or from prose
//! where it doesn't (spirits, combat styles). [`provide`] reads every family before any class is built and gives it
//! to the class stage through [`crate::picks`]: its selection, features, lists and nested selections are the owner's
//! from the start, and the class's tables take it as they're read ([`Taking`]):
//! - before a table's rows are read, its cells are planned ([`Taking::plan`]): the choice ("Bloodline", "Order",
//!   "Spirit", "Ways of the Deceiver") and the ability placeholders ("Bloodline Ability", "Order Ability") are
//!   markers, never features (the held option's rows stand there), and so is what a fixed option's row gives where
//!   the archetype's own table lists it; an archetype that takes no family keeps its cells;
//! - once every table of the class is read ([`Taking::take`], on the full rows): the class's row grants the
//!   selection; an option level whose row an archetype's table has no placeholder for (a level its row differs from
//!   its class's at, the markers counted as what they'd be) is skipped, and the archetype's own row stays; an
//!   archetype that skips levels, limits the options or takes only the feat list gets its own selection sharing the
//!   options it allows (one option is granted with no pick); the bonus slots that took the class's merged bloodline
//!   list pick from the held option's list (the lists still count when the class's lists are keyed:
//!   [`crate::feat_lists::PendingLists::stand_in`]); the rows an archetype that takes none shares with its class
//!   follow the class's.
//!
//! The archetype rows are then made sparse as every class's are. Making the decisions on the full rows, before that,
//! gives what deciding on the sparse rows did: a row that matches its class's (its markers included) holds what the
//! class's holds, and the archetype's skipped levels are the ones whose row differs from its class's and has no
//! placeholder.
//!
//! The facts of the families' owners that don't hang on those rows are given with them too: the mythic abilities
//! that pick another option and Bloodline Ascendance, the daily choices, the Primalist's choices, Dragon Disciple,
//! the Eldritch Scion, the Dragonheir Scion, the Elemental Specialist's element and Disruptive's line
//! ([`provide_others`]). [`check`](fn@check) asserts what only the finished data shows.

use std::collections::BTreeMap;

use rules::{
    ClassKey, FeatList, Feature, FeatureKey, FeatureSelection, FeatureSelectionKey, OptionRow,
    SelectionOption, SelectionOptionKey,
};

use crate::guide::Guide;
use crate::names::norm;
use crate::picks::Picks;
use crate::report::Settled;

mod bloodrager;
mod check;
mod class_takes;
mod owner_facts;
mod reading;
mod rows;
mod sorcerer;
mod spirits_and_styles;
mod tabled;
mod taking;

use bloodrager::{BLOODRAGER_TABLES, PRIMAL_CHOICES, bloodrager_bloodlines, primal_choices};
pub(crate) use check::check;
use class_takes::Takes;
use class_takes::{BLOODRAGER, CAVALIER, MAGIC_DECEIVER, RANGER, SHAMAN, SORCERER, WIZARD};
use owner_facts::provide_others;
pub(crate) use reading::{FeatNames, page, section};
pub use reading::{name_list, own_cells};
pub(crate) use rows::{empty_row, feature, list, option, selection};
use sorcerer::{SORCERER_TABLES, sorcerer_bloodlines};
use spirits_and_styles::{ranger_styles, shaman_spirits};
use tabled::{ORDERS, SCHOOLS, WAYS, cavalier_orders, magic_deceiver_ways, wizard_schools};
pub(crate) use taking::{Cell, Planned, TablePlan, Taking};

/// What a pass changed, one line each, for a one-off rewrite's log.
pub type Log = Vec<String>;

/// The families built with their classes, given to the class stage.
fn provide_families(guide: &Guide, feats: &FeatNames, skills: &[rules::Skill], picks: &mut Picks) {
    let mut orders = cavalier_orders(guide, feats);
    crate::option_class_skills::option_rows(guide, &mut orders, skills);
    give(picks, orders, &CAVALIER);
    give(picks, wizard_schools(guide, feats), &WIZARD);
    let mut ways = magic_deceiver_ways(guide, feats);
    crate::option_families::living_deity_rows(&mut ways);
    crate::follow_ups::razmiri_infiltrator(guide, &mut ways);
    give(picks, ways, &MAGIC_DECEIVER);
    give(picks, shaman_spirits(guide), &SHAMAN);
    give(picks, ranger_styles(guide, feats), &RANGER);
    let mut bloodrager = bloodrager_bloodlines(guide, feats);
    crate::second_follow_ups::bloodrager_groups(&mut bloodrager.selection);
    give(picks, bloodrager, &BLOODRAGER);
    picks.selection("bloodrager/primalist", primal_choices(guide));
    picks.unnamed(PRIMAL_CHOICES);
    let mut sorcerer = sorcerer_bloodlines(guide, feats);
    crate::option_follow_ups::sorcerer_bloodlines(guide, &mut sorcerer, picks);
    crate::option_class_skills::option_rows(guide, &mut sorcerer, skills);
    provide_others(&sorcerer, picks);
    give(picks, sorcerer, &SORCERER);
}

/// Gives a family's definitions to its owner (its selection, or the rows of the guide's selection its headings
/// define; its features, lists and nested selections), and how its class's tables take it.
fn give(picks: &mut Picks, family: Family, takes: &'static Takes) {
    let owner = match family.owner {
        Owner::Class(class) => class,
        Owner::Archetype(_, archetype) => archetype,
    };
    let key = family.selection.key.clone();
    // The ranger's combat styles are the guide's selection already: they get their rows.
    if HEADING_SELECTIONS.contains(&key.as_str()) {
        let classes: Vec<&str> = family
            .selection
            .option_levels
            .iter()
            .map(ClassKey::as_str)
            .collect();
        picks.option_levels(key.as_str(), &classes);
        for option in &family.selection.options {
            picks.option_rows(option.key.as_str(), option.levels.clone());
        }
    } else {
        picks.selection(owner, family.selection.clone());
        // Second Spirit's "Prerequisites: Spirit" means a spirit held; the other families' names are no line's.
        if key.as_str() != "shaman/spirit" {
            picks.unnamed(key.as_str());
        }
    }
    for feature in &family.features {
        picks.feature(owner, feature.clone());
    }
    for list in &family.lists {
        picks.feat_list(owner, list.clone());
    }
    for nested in &family.nested {
        picks.selection(owner, nested.clone());
        picks.unnamed(nested.key.as_str());
    }
    picks.family(Taking { family, takes });
}

/// The families' selections the guide's headings define.
const HEADING_SELECTIONS: [&str; 1] = ["ranger/combat-style-feat"];

/// One family of options: the selection a class grants and the options, features and lists it defines.
#[derive(Clone, Debug)]
pub struct Family {
    /// The class (or, for the Magic Deceiver, the archetype) that defines it.
    pub owner: Owner,
    pub selection: FeatureSelection,
    pub features: Vec<Feature>,
    pub lists: Vec<FeatList>,
    /// Rows from prose that belong with a table row (Strength of the Abyss at 13 and 17 with 9): skipped with it.
    pub ties: BTreeMap<u8, u8>,
    /// Selections the options' rows grant (the Arcane bloodline's school at 15).
    pub nested: Vec<FeatureSelection>,
}

/// Who defines a family.
#[derive(Clone, Debug)]
pub enum Owner {
    Class(&'static str),
    Archetype(&'static str, &'static str),
}

impl Family {
    fn option(&self, key: &str) -> &SelectionOption {
        self.selection
            .options
            .iter()
            .find(|option| option.key.as_str() == key)
            .unwrap_or_else(|| panic!("no option {key}"))
    }

    fn option_keys(&self) -> Vec<SelectionOptionKey> {
        self.selection
            .options
            .iter()
            .map(|option| option.key.clone())
            .collect()
    }

    fn feature_name(&self, key: &FeatureKey) -> Option<&str> {
        self.features
            .iter()
            .find(|feature| &feature.key == key)
            .map(|feature| feature.name.as_str())
    }
}

/// The facts of the families' mythic abilities and daily choices, built where their owners are: Second Bloodline
/// ("You can select a second bloodline, gaining all its benefits"), Second Bloodrager Bloodline and Second Spirit
/// ("You gain a second Shaman Spirit") pick another option; Bloodline Ascendance ("You gain the 20th level ability of your bloodline") grants the held bloodline's row
/// 20. Mixed Bloodline ("Once per day, a Mixed-Blood Rager chooses a bloodline") and the shaman's Wandering Spirit
/// ("can be changed each time you rest") are daily choices.
pub(crate) fn provide(
    guide: &Guide,
    feats: &FeatNames,
    skills: &[rules::Skill],
    picks: &mut Picks,
) {
    provide_families(guide, feats, skills, picks);
    let grants = [
        ("second-bloodline", "sorcerer/bloodline"),
        ("second-bloodrager-bloodline", "bloodrager/bloodline"),
        ("second-spirit", "shaman/spirit"),
    ];
    for (ability, selection) in grants {
        picks.mythic_selection(ability, FeatureSelectionKey::new(selection));
    }
    let ascendance = "sorcerer/bloodline-ascendance";
    picks.feature(
        "sorcerer",
        Feature {
            option_row: Some(OptionRow {
                selection: FeatureSelectionKey::new("sorcerer/bloodline"),
                level: 20,
            }),
            ..feature(
                &FeatureKey::new(ascendance),
                "Bloodline Ascendance",
                "mythic-paths#bloodline-ascendance".into(),
            )
        },
    );
    picks.mythic_feature("bloodline-ascendance", ascendance);
    for daily in [
        "bloodrager/mixed-blood-rager/mixed-bloodline",
        "bloodrager/mixed-blood-rager/second-mixed-bloodline",
        "shaman/wandering-spirit",
    ] {
        picks.daily_choice(daily);
    }
}

/// The report items this module settles: the option tables it reads, and the lines it can now state.
pub fn settles() -> Vec<Settled> {
    let tables = SORCERER_TABLES
        .iter()
        .map(|t| format!("sorcerer#{t}"))
        .chain(
            BLOODRAGER_TABLES
                .iter()
                .map(|t| format!("bloodrager#{t}-bloodrager")),
        )
        .chain(SCHOOLS.iter().map(|t| format!("wizard#{t}")))
        .chain(ORDERS.iter().map(|t| format!("cavalier#{t}")))
        .chain(WAYS.iter().map(|t| format!("arcanist#{t}")));
    tables
        .map(Settled::any)
        .chain([
            Settled::any("character-creation#bloodlines"),
            Settled::containing("prestige-classes#dragon-disciple", "Draconic bloodline"),
            Settled::containing("prestige-classes#dragon-disciple", "spontaneously"),
        ])
        .collect()
}

/// Whether a report item is one this module settles ([`settles`]).
pub fn settled(at: &str, text: &str) -> bool {
    settles().iter().any(|entry| entry.matches(at, text))
}

/// A name without its parenthesised tail, for comparing: "Manifestation (Battle)" is "Manifestation".
pub(crate) fn bare(name: &str) -> String {
    norm(name.split(" (").next().unwrap_or(name))
}
