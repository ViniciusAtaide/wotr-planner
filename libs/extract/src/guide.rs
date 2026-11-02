//! The saved guide pages, loaded and parsed.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::html::{Page, parse_page};

/// The base classes, each on its own page.
pub const BASE_CLASSES: [&str; 26] = [
    "alchemist",
    "arcanist",
    "barbarian",
    "bard",
    "bloodrager",
    "cavalier",
    "cleric",
    "druid",
    "fighter",
    "hunter",
    "inquisitor",
    "kineticist",
    "magus",
    "monk",
    "oracle",
    "paladin",
    "ranger",
    "rogue",
    "shaman",
    "shifter",
    "skald",
    "slayer",
    "sorcerer",
    "warpriest",
    "witch",
    "wizard",
];

/// The mythic paths with their own page.
pub const PATH_PAGES: [&str; 7] = [
    "aeon",
    "angel",
    "azata",
    "demon",
    "lich",
    "trickster",
    "gold-dragon",
];

/// The paths on the `other-mythic-paths` page, by anchor.
pub const OTHER_PATHS: [&str; 3] = ["legend", "devil", "swarm-that-walks"];

/// Every page the extractor reads.
pub fn page_slugs() -> Vec<&'static str> {
    let mut slugs = vec![
        "character-creation",
        "feats",
        "prestige-classes",
        "mythic-paths",
        "other-mythic-paths",
    ];
    slugs.extend(BASE_CLASSES);
    slugs.extend(PATH_PAGES);
    slugs
}

/// The parsed pages and the date they were saved.
pub struct Guide {
    pub pages: BTreeMap<String, Page>,
    pub saved_at: String,
}

impl Guide {
    /// Loads `<dir>/<slug>.html` for every page, checking each page's canonical link.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let mut pages = BTreeMap::new();
        for slug in page_slugs() {
            let path = dir.join(format!("{slug}.html"));
            let html = fs::read_to_string(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            pages.insert(slug.to_owned(), parse_page(slug, &html)?);
        }
        let saved_at = fs::read_to_string(dir.join("SAVED_AT"))
            .map(|text| text.trim().to_owned())
            .unwrap_or_else(|_| "an unknown date".to_owned());
        Ok(Self { pages, saved_at })
    }

    pub fn page(&self, slug: &str) -> &Page {
        &self.pages[slug]
    }
}
