//! `search`: every item the agent may name, matched by name, key or initials and ranked.

use rules::{Engine, GameData, GuideRef, Prerequisite};

use crate::planner::all_selections;
use crate::shapes::{Hit, Kind};

/// How many matches a search returns when it doesn't say, and at most.
const DEFAULT_LIMIT: usize = 10;
const MAX_LIMIT: usize = 50;

/// Every item's [`Hit`], with its name and key ready to match.
#[derive(Debug)]
pub(crate) struct SearchIndex {
    entries: Vec<Entry>,
}

#[derive(Debug)]
struct Entry {
    hit: Hit,
    /// The name in lower case.
    name: String,
    /// The name's words, lower case, apostrophes dropped: "nature's ally" → `natures`, `ally`.
    words: Vec<String>,
    /// The words' first letters: "Brown-Fur Transmuter" → `bft`.
    initials: String,
}

/// How well an entry matches, best first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    /// The whole name, the key, or the key's last segment.
    Exact,
    Initials,
    NameStart,
    /// Every query word starts a word of the name.
    Words,
    /// The name or the key contains the query.
    Contains,
    /// The initials start with the query.
    InitialsStart,
}

impl SearchIndex {
    pub(crate) fn new(data: &GameData, engine: &Engine) -> Self {
        let mut entries = Vec::new();
        let mut add = |hit: Hit| entries.push(Entry::new(hit));
        let prerequisite = |p: &Option<Prerequisite>| {
            p.as_ref()
                .map(|prerequisite| engine.prerequisite_text(prerequisite, None))
        };
        let guide = |guide: &GuideRef| Some(guide.url());

        for (classes, category) in [
            (&data.classes, "Base class"),
            (&data.prestige_classes, "Prestige class"),
        ] {
            for class in classes {
                add(Hit {
                    kind: Kind::Class,
                    key: class.key.as_str().to_owned(),
                    name: class.name.clone(),
                    parent: None,
                    category: Some(category.to_owned()),
                    prerequisite: prerequisite(&class.prerequisite),
                    guide_url: guide(&class.guide),
                });
                for archetype in &class.archetypes {
                    add(Hit {
                        kind: Kind::Archetype,
                        key: archetype.key.as_str().to_owned(),
                        name: archetype.name.clone(),
                        parent: Some(class.key.as_str().to_owned()),
                        category: None,
                        prerequisite: prerequisite(&archetype.prerequisite),
                        guide_url: guide(&archetype.guide),
                    });
                }
            }
        }
        for path in &data.mythic_paths {
            add(Hit {
                kind: Kind::Path,
                key: path.key.as_str().to_owned(),
                name: path.name.clone(),
                parent: None,
                category: None,
                prerequisite: None,
                guide_url: guide(&path.guide),
            });
        }
        for feat in &data.feats {
            add(Hit {
                kind: Kind::Feat,
                key: feat.key.as_str().to_owned(),
                name: feat.name.clone(),
                parent: None,
                category: Some(feat.category.name().to_owned()),
                prerequisite: prerequisite(&feat.prerequisite),
                guide_url: guide(&feat.guide),
            });
        }
        for ability in &data.mythic_abilities {
            add(Hit {
                kind: Kind::MythicAbility,
                key: ability.key.as_str().to_owned(),
                name: ability.name.clone(),
                parent: None,
                category: Some(ability.group.name().to_owned()),
                prerequisite: prerequisite(&ability.prerequisite),
                guide_url: guide(&ability.guide),
            });
        }
        for feat in &data.mythic_feats {
            add(Hit {
                kind: Kind::MythicFeat,
                key: feat.key.as_str().to_owned(),
                name: feat.name.clone(),
                parent: None,
                category: Some(feat.group.name().to_owned()),
                prerequisite: prerequisite(&feat.prerequisite),
                guide_url: guide(&feat.guide),
            });
        }
        for selection in all_selections(data) {
            for option in &selection.options {
                add(Hit {
                    kind: Kind::SelectionOption,
                    key: option.key.as_str().to_owned(),
                    name: option.name.clone(),
                    parent: Some(selection.key.as_str().to_owned()),
                    category: Some(selection.name.clone()),
                    prerequisite: prerequisite(&option.prerequisite),
                    guide_url: guide(&option.guide),
                });
            }
        }
        for race in &data.races {
            add(Hit {
                kind: Kind::Race,
                key: race.key.as_str().to_owned(),
                name: race.name.clone(),
                parent: None,
                category: None,
                prerequisite: None,
                guide_url: guide(&race.guide),
            });
        }
        for deity in &data.deities {
            add(Hit {
                kind: Kind::Deity,
                key: deity.key.as_str().to_owned(),
                name: deity.name.clone(),
                parent: None,
                category: None,
                prerequisite: None,
                guide_url: guide(&deity.guide),
            });
        }
        for background in &data.backgrounds {
            add(Hit {
                kind: Kind::Background,
                key: background.key.as_str().to_owned(),
                name: background.name.clone(),
                parent: None,
                category: background.group.map(|group| group.name().to_owned()),
                prerequisite: None,
                // The user's list, not the guide's: no link.
                guide_url: None,
            });
        }
        Self { entries }
    }

    /// The matches for a query, best first, limited to `kinds` when given. `None` for a query with nothing to
    /// match.
    pub(crate) fn search(
        &self,
        query: &str,
        kinds: Option<&[Kind]>,
        limit: Option<u32>,
    ) -> Option<Vec<Hit>> {
        let query = Query::new(query)?;
        let limit = limit.map_or(DEFAULT_LIMIT, |limit| {
            usize::try_from(limit).unwrap_or(MAX_LIMIT).min(MAX_LIMIT)
        });
        let mut found: Vec<(Rank, &Entry)> = self
            .entries
            .iter()
            .filter(|entry| kinds.is_none_or(|kinds| kinds.contains(&entry.hit.kind)))
            .filter_map(|entry| Some((entry.rank(&query)?, entry)))
            .collect();
        found.sort_by(|(a_rank, a), (b_rank, b)| {
            (a_rank, a.hit.kind, a.name.len(), &a.name, &a.hit.key).cmp(&(
                b_rank,
                b.hit.kind,
                b.name.len(),
                &b.name,
                &b.hit.key,
            ))
        });
        Some(
            found
                .into_iter()
                .take(limit)
                .map(|(_, entry)| entry.hit.clone())
                .collect(),
        )
    }

    /// The item of a kind with a key.
    pub(crate) fn get(&self, kind: Kind, key: &str) -> Option<&Hit> {
        self.entries
            .iter()
            .map(|entry| &entry.hit)
            .find(|hit| hit.kind == kind && hit.key == key)
    }
}

impl Entry {
    fn new(hit: Hit) -> Self {
        let words = words(&hit.name);
        Self {
            name: hit.name.to_lowercase(),
            initials: words
                .iter()
                .filter_map(|word| word.chars().next())
                .collect(),
            words,
            hit,
        }
    }

    fn rank(&self, query: &Query) -> Option<Rank> {
        let key = self.hit.key.as_str();
        let last_segment = key.rsplit('/').next().unwrap_or(key);
        if self.name == query.text || key == query.text || last_segment == query.text {
            Some(Rank::Exact)
        } else if query.initials.as_deref() == Some(self.initials.as_str()) {
            Some(Rank::Initials)
        } else if self.name.starts_with(&query.text) {
            Some(Rank::NameStart)
        } else if query.words.iter().all(|word| {
            self.words
                .iter()
                .any(|name_word| name_word.starts_with(word.as_str()))
        }) {
            Some(Rank::Words)
        } else if self.name.contains(&query.text) || key.contains(&query.text) {
            Some(Rank::Contains)
        } else if query
            .initials
            .as_deref()
            .is_some_and(|initials| self.initials.starts_with(initials))
        {
            Some(Rank::InitialsStart)
        } else {
            None
        }
    }
}

/// A query, ready to match.
struct Query {
    /// Lower case, its spaces collapsed.
    text: String,
    words: Vec<String>,
    /// The query as initials: one word of two or more letters ("bft").
    initials: Option<String>,
}

impl Query {
    fn new(query: &str) -> Option<Self> {
        let text = query
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let words = words(&text);
        if words.is_empty() {
            return None;
        }
        let initials = match words.as_slice() {
            [word] if word.chars().count() >= 2 && !text.contains(' ') => Some(word.clone()),
            _ => None,
        };
        Some(Self {
            text,
            words,
            initials,
        })
    }
}

/// A name's words, lower case: split at anything but letters, digits and apostrophes, which are dropped.
fn words(name: &str) -> Vec<String> {
    name.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '’'))
        .map(|word| {
            word.chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .filter(|word| !word.is_empty())
        .collect()
}
