//! Options with rows by option level, the second group (ticket 31): domains, oracle mysteries and their revelations,
//! warpriest blessings and kineticist elements, on [`option_levels`](crate::option_levels)'s model.
//!
//! The guide gives these families in prose, so each option's rows are read from its section by a table here that
//! names, for each power, the level and a phrase the section must contain (asserted):
//! - **domains** (`character-creation#domains`, 37): a power at 1 and one at 6 or 8 (the War domain's bonus Combat
//!   feat is a slot), the Animal and Scalykind companion at 4, class skills on their row (tickets 37 and 38: the
//!   Animal domain's on its 4th, with the companion). One selection, `cleric/domain` (cleric levels), which the inquisitor, the Divine Hunter, the druid (through Nature
//!   Bond's domain option), the Blight Druid, the Winter Child, the Hellknight's Pentamic Faith and the Living
//!   Deity's Hollow Spark share through selections of their own, levelled by their own class;
//! - **mysteries** (`oracle#oracle-mysteries`, 9): class skills at 1, the Final Revelation at 20, and a pool of
//!   revelations (`oracle/revelation` offers the held mysteries' pools); revelations that grant later are rows;
//! - **blessings** (`warpriest#blessings`): the minor power at 1 and the major at 10, so the level-10 picks go;
//! - **elements** (`kineticist#elemental-focus`, 4): class skills, the wild-talent bonus feat list, the defensive
//!   talent at 2; Expanded Element at 7 and 15 is a plain pick of an element (which may double the first).
//!
//! The classes' placeholder cells ("Domain Abilities", "Revelation", "Defensive Wild Talent" …) are the held
//! option's rows, and the cells that are a choice are the selection. A scaling number stays in the summary (ticket
//! 29), so a domain's resistance growing to an immunity, or a revelation's uses per day, add no rows.
//!
//! The domains, the mysteries, the blessings, the elements and the Hellknight's order and Pentamic Faith are built
//! with their classes ([`provide`], through [`crate::picks`]): no placeholder feature is made, and [`check`] asserts
//! the rows in force. With the domains: the Separatist's second domain and its alignment domains (tickets 33 and 35),
//! the Divine Hunter's every domain power (ticket 33), Domain Zealot's "Domain" and what Boon Companion's and Mythical
//! Beast's "Animal Companion" also accepts. The Living Deity's mystery, revelations and domains are the Magic
//! Deceiver's selections, which its way's rows grant ([`living_deity_rows`], on the ways before
//! [`option_levels`](crate::option_levels) gives them), and Impossible Domain (tickets 33 and 35) is built beside the
//! domains: its line, resolved once the domains and the Living Deity's way exist, also accepts that way and the
//! Pentamic Faith discipline.

use std::collections::{BTreeMap, BTreeSet};

use rules::{FeatList, Feature, FeatureSelection, GameData, SelectionOptionKey, Skill, SkillKey};

use crate::guide::Guide;
use crate::html::Section;
use crate::names::norm;
use crate::option_levels::{FeatNames, selection};
use crate::picks::Picks;

mod blessings;
mod domain_selections;
mod domains;
mod elements;
mod hellknight;
mod living_deity;
mod mysteries;
mod oracle;

use blessings::{blessings, check_blessings};
use domain_selections::{check_domains, provide_domains};
pub use domains::domain_short;
use domains::domains;
pub use elements::element_feat;
use elements::{check_elements, elements};
use hellknight::{check_hellknight, hellknight};
pub(crate) use living_deity::living_deity_rows;
use living_deity::{impossible_domain, living_deity};
use mysteries::mysteries;
pub use mysteries::revelation_level;
use oracle::{check_mysteries, oracle_mysteries};

/// The families built with their owners, before any class is: the oracle's mysteries, the warpriest's blessings, the
/// kineticist's elements, the domains and the Hellknight's order and Pentamic Faith.
pub(crate) fn provide(guide: &Guide, feats: &FeatNames, skills: &[Skill], picks: &mut Picks) {
    let skills = Skills::of(skills);
    let mysteries = mysteries(guide, &skills, feats);
    oracle_mysteries(guide, &mysteries, picks);
    blessings(guide, picks);
    elements(guide, &skills, feats, picks);
    let domains = domains(guide, &skills);
    provide_domains(guide, &domains, picks);
    hellknight(guide, &domains, picks);
    living_deity(&domains, &mysteries, picks);
    impossible_domain(guide, &domains, picks);
}

/// What the providers can only assert on the finished data: the rows in force. Changes nothing.
pub(crate) fn check(guide: &Guide, data: &GameData) {
    check_mysteries(data);
    check_blessings(guide, data);
    check_elements(data);
    check_hellknight(data);
    check_domains(data);
}

/// Skill names as the guide writes them, to skill keys.
struct Skills(BTreeMap<String, (SkillKey, String)>);

impl Skills {
    fn of(skills: &[Skill]) -> Self {
        Self(
            skills
                .iter()
                .map(|skill| (norm(&skill.name), (skill.key.clone(), skill.name.clone())))
                .collect(),
        )
    }

    /// The skill a guide name means, with its name as the data writes it.
    fn get(&self, name: &str) -> (SkillKey, String) {
        self.0
            .get(&norm(name))
            .cloned()
            .unwrap_or_else(|| panic!("no skill named {name:?}"))
    }
}

/// One family: its selection (options with rows), the features and lists the rows name, and selections the rows
/// grant.
pub struct Family {
    pub selection: FeatureSelection,
    pub features: Vec<Feature>,
    pub lists: Vec<FeatList>,
    pub nested: Vec<FeatureSelection>,
}

impl Family {
    fn keys(&self, short_names: &[&str]) -> BTreeSet<SelectionOptionKey> {
        short_names
            .iter()
            .map(|name| {
                let key = SelectionOptionKey::new(format!("{}/{name}", self.selection.key));
                assert!(
                    self.selection
                        .options
                        .iter()
                        .any(|option| option.key == key),
                    "no option {key}"
                );
                key
            })
            .collect()
    }

    fn all_keys(&self) -> BTreeSet<SelectionOptionKey> {
        self.selection
            .options
            .iter()
            .map(|option| option.key.clone())
            .collect()
    }
}

/// A section's text with its tables' cells too (a wild talent's description is a table row).
fn all_text(section: &Section) -> String {
    let mut text = section.text();
    for table in section.tables() {
        for row in table {
            text.push_str(&row.join(" "));
            text.push(' ');
        }
    }
    text
}

fn contains(text: &str, phrase: &str, at: &str) {
    assert!(text.contains(phrase), "{at}: no {phrase:?} in {text:?}");
}

/// A selection of `owner`'s that offers `options` of a shared family, levelled by `levels`.
fn sharing(
    key: &str,
    name: &str,
    guide: String,
    levels: &[&str],
    options: BTreeSet<SelectionOptionKey>,
) -> FeatureSelection {
    FeatureSelection {
        shared_options: options,
        ..selection(key, name, guide, levels)
    }
}

fn starts(name: &str, prefix: &str) -> bool {
    name.starts_with(&norm(prefix))
}
