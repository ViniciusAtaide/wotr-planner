//! The unparsed report: every line, cell or prose rule the extractor couldn't turn into data, with where it is.
//!
//! Items are grouped by [`Category`], one section per later hand-fix pass, then by [`Kind`].

use std::collections::BTreeMap;
use std::fmt::Write as _;

use rules::DataAt;

use crate::guide::PATH_PAGES;

/// The part of the data an item belongs to: each has its own section in the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Feats,
    BaseClasses,
    PrestigeClasses,
    Mythic,
    RacesAndSkills,
}

impl Category {
    pub const ALL: [Category; 5] = [
        Category::Feats,
        Category::BaseClasses,
        Category::PrestigeClasses,
        Category::Mythic,
        Category::RacesAndSkills,
    ];

    fn heading(self) -> &'static str {
        match self {
            Category::Feats => "Feats",
            Category::BaseClasses => "Base classes",
            Category::PrestigeClasses => "Prestige classes",
            Category::Mythic => "Mythic",
            Category::RacesAndSkills => "Races and skills",
        }
    }

    /// The category of a `page#anchor` link.
    pub fn of(at: &str) -> Category {
        let (page, anchor) = at.split_once('#').unwrap_or((at, ""));
        match page {
            "feats" => Category::Feats,
            "prestige-classes" => Category::PrestigeClasses,
            "mythic-paths" | "other-mythic-paths" | "progression" => Category::Mythic,
            // Sorcerer and bloodrager bloodlines are described on the character creation page.
            "character-creation" if anchor == "bloodlines" => Category::BaseClasses,
            "character-creation" => Category::RacesAndSkills,
            _ if PATH_PAGES.contains(&page) => Category::Mythic,
            // The base class pages.
            _ => Category::BaseClasses,
        }
    }

    /// The category of a data entity, for an item about the data rather than a guide page. Keys carry no type
    /// prefix, so the kind comes from [`DataAt`]. A class may be a prestige class; it's filed with the base classes.
    pub fn of_data(at: &DataAt) -> Category {
        match at {
            DataAt::Feat(_) => Category::Feats,
            // Lists belong to classes; a path's would be filed with them too.
            DataAt::FeatList(_) => Category::BaseClasses,
            DataAt::Race(_) | DataAt::Subrace(_) | DataAt::Skill(_) | DataAt::Background(_) => {
                Category::RacesAndSkills
            }
            DataAt::Progression
            | DataAt::MythicAbility(_)
            | DataAt::MythicFeat(_)
            | DataAt::MythicPath(_) => Category::Mythic,
            DataAt::Class(_)
            | DataAt::Archetype(_)
            | DataAt::Feature(_)
            | DataAt::FeatureSelection(_)
            | DataAt::SelectionOption(_) => Category::BaseClasses,
        }
    }
}

/// What kind of thing couldn't be turned into data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// A prerequisite line that didn't parse, or named something the data doesn't have. It is left out, so it
    /// doesn't block any pick.
    Prerequisite,
    /// A prestige class or archetype entry requirement.
    Requirement,
    /// A progression-table cell that couldn't be placed.
    Cell,
    /// A bonus feat slot whose list or prerequisite rule is unclear.
    BonusSlot,
    /// A rule the guide states only in prose that the data types can't express.
    ProseRule,
}

impl Kind {
    fn heading(self) -> &'static str {
        match self {
            Kind::Prerequisite => "Prerequisite lines",
            Kind::Requirement => "Entry requirements",
            Kind::Cell => "Progression table cells",
            Kind::BonusSlot => "Bonus feat slots",
            Kind::ProseRule => "Prose-only rules",
        }
    }

    fn explanation(self) -> &'static str {
        match self {
            Kind::Prerequisite => {
                "Each line is left out of its prerequisite, so it doesn't block any pick. A group (`or`, \"One of\") \
                 with an unresolved member is left out whole."
            }
            Kind::Requirement => "Left out of the class's or archetype's prerequisite.",
            Kind::Cell => "The cell's text, and how it was stored instead.",
            Kind::BonusSlot => "The slot as stored, and why it needs a look.",
            Kind::ProseRule => "Rules stated only in prose that the current data types can't hold.",
        }
    }
}

/// One item.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    pub kind: Kind,
    /// `page#anchor`, or a data entity (`feat dodge`).
    pub at: String,
    pub category: Category,
    /// The guide's text for the item (a prerequisite line or cell; short facts, never prose).
    pub text: String,
    pub why: String,
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub items: Vec<Item>,
}

impl Report {
    /// An item at a `page#anchor` link.
    pub fn add(
        &mut self,
        kind: Kind,
        at: impl Into<String>,
        text: impl Into<String>,
        why: impl Into<String>,
    ) {
        let at = at.into();
        self.items.push(Item {
            kind,
            category: Category::of(&at),
            at,
            text: text.into(),
            why: why.into(),
        });
    }

    /// An item about a data entity, such as a data-check problem: filed by the entity's kind.
    pub fn add_about(
        &mut self,
        kind: Kind,
        at: &DataAt,
        text: impl Into<String>,
        why: impl Into<String>,
    ) {
        self.items.push(Item {
            kind,
            at: at.to_string(),
            category: Category::of_data(at),
            text: text.into(),
            why: why.into(),
        });
    }

    /// The distinct items, each once, in order.
    fn distinct(&self) -> Vec<&Item> {
        let mut items: Vec<&Item> = self.items.iter().collect();
        items.sort();
        items.dedup();
        items
    }

    /// How many distinct items there are: an item repeated (an archetype copying its class's slot) counts once.
    pub fn len(&self) -> usize {
        self.distinct().len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The report as Markdown: a section per [`Category`], each split by [`Kind`].
    pub fn markdown(&self, saved_at: &str) -> String {
        let items = self.distinct();
        let mut by_category: BTreeMap<Category, BTreeMap<Kind, Vec<&Item>>> = BTreeMap::new();
        for item in &items {
            by_category
                .entry(item.category)
                .or_default()
                .entry(item.kind)
                .or_default()
                .push(item);
        }
        let mut out = String::new();
        out.push_str("# Extraction report\n\n");
        let _ = writeln!(
            out,
            "What the extractor couldn't turn into data from the guide pages saved on {saved_at}, with where \
             each item is (`page#anchor`). Fix an item by editing the `.ron` files, check with \
             `cargo run -p data --bin seed -- --check-only`, then delete its line here. {} items.\n",
            items.len()
        );
        for category in Category::ALL {
            let kinds = by_category.remove(&category).unwrap_or_default();
            let count: usize = kinds.values().map(Vec::len).sum();
            let _ = writeln!(out, "## {} ({count})\n", category.heading());
            if count == 0 {
                out.push_str("Nothing left.\n\n");
            }
            for (kind, items) in kinds {
                let _ = writeln!(out, "### {} ({})\n", kind.heading(), items.len());
                let _ = writeln!(out, "{}\n", kind.explanation());
                for item in items {
                    let _ = writeln!(out, "- `{}`: {} ({})", item.at, quote(&item.text), item.why);
                }
                out.push('\n');
            }
        }
        out
    }
}

fn quote(text: &str) -> String {
    if text.is_empty() {
        String::new()
    } else {
        format!("\"{}\"", text.replace('\n', " / "))
    }
}
