//! `search`: every item the agent may name, matched by name, key or initials and ranked.

use rules::{Engine, GameData, GuideRef, Prerequisite};

use crate::planner::all_selections;
use crate::shapes::{Hit, Kind, SearchResult};
use crate::similar::{FLOOR, closeness, words};

/// How many matches a search returns when it doesn't say (for a query, and for an empty one listing what the filters
/// allow), and at most.
const DEFAULT_LIMIT: usize = 10;
const EMPTY_QUERY_LIMIT: usize = 50;
const MAX_LIMIT: usize = 100;

/// How many near names or categories a note names.
const NEAR: usize = 5;

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
        let summary = |text: &str| (!text.is_empty()).then(|| text.to_owned());

        for (classes, category) in [
            (&data.classes, "Base class"),
            (&data.prestige_classes, "Prestige class"),
        ] {
            for class in classes {
                add(Hit {
                    kind: Kind::Class,
                    key: class.key.as_str().to_owned(),
                    name: class.name.clone(),
                    summary: summary(&class.summary),
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
                        summary: summary(&archetype.summary),
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
                summary: summary(&path.summary),
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
                summary: summary(&feat.summary),
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
                summary: summary(&ability.summary),
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
                summary: summary(&feat.summary),
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
                    summary: summary(&option.summary),
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
                summary: summary(&race.summary),
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
                summary: None,
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
                summary: summary(&background.note),
                parent: None,
                category: background.group.map(|group| group.name().to_owned()),
                prerequisite: None,
                // The user's list, not the guide's: no link.
                guide_url: None,
            });
        }
        Self { entries }
    }

    /// The matches for a query, best first, limited to `kinds` and `categories` when given. An empty query matches
    /// every item the filters allow, by kind and then name. When nothing matches, the result's note says so and names
    /// the near names and categories ([`similar::FLOOR`]). An error for an empty query without a filter.
    pub(crate) fn search(
        &self,
        query: &str,
        kinds: Option<&[Kind]>,
        categories: Option<&[String]>,
        limit: Option<u32>,
    ) -> Result<SearchResult, String> {
        let given = query.trim();
        let query = Query::new(given);
        if query.is_none() && kinds.is_none() && categories.is_none() {
            return Err("The query is empty: give a name, a key or initials, or filter by `kinds` or `category` \
                        to list every item they allow."
                .to_owned());
        }
        let allowed = |entry: &&Entry| {
            kinds.is_none_or(|kinds| kinds.contains(&entry.hit.kind))
                && categories.is_none_or(|categories| entry.in_category(categories))
        };
        let default_limit = match query {
            Some(_) => DEFAULT_LIMIT,
            None => EMPTY_QUERY_LIMIT,
        };
        let limit = limit.map_or(default_limit, |limit| {
            usize::try_from(limit).unwrap_or(MAX_LIMIT).min(MAX_LIMIT)
        });
        let mut found: Vec<(Rank, &Entry)> = self
            .entries
            .iter()
            .filter(allowed)
            .filter_map(|entry| match &query {
                Some(query) => Some((entry.rank(query)?, entry)),
                None => Some((Rank::Exact, entry)),
            })
            .collect();
        match query {
            Some(_) => found.sort_by(|(a_rank, a), (b_rank, b)| {
                (a_rank, a.hit.kind, a.name.len(), &a.name, &a.hit.key).cmp(&(
                    b_rank,
                    b.hit.kind,
                    b.name.len(),
                    &b.name,
                    &b.hit.key,
                ))
            }),
            None => found.sort_by(|(_, a), (_, b)| {
                (a.hit.kind, &a.name, &a.hit.key).cmp(&(b.hit.kind, &b.name, &b.hit.key))
            }),
        }
        let total = u32::try_from(found.len()).unwrap_or(u32::MAX);
        let note = found.is_empty().then(|| {
            let mut note = match query {
                Some(_) => format!("No item in the planner's data matches `{given}`"),
                None => "No item in the planner's data matches".to_owned(),
            };
            if kinds.is_some() || categories.is_some() {
                note.push_str(" with these filters");
            }
            note.push_str(
                ". The data holds what the guide lists: an item that isn't there can't be taken in a Build.",
            );
            if query.is_some() {
                let near = self.near_names(given, &allowed);
                if !near.is_empty() {
                    note.push_str(&format!(" Near names: {}.", near.join(", ")));
                }
            }
            for category in categories.into_iter().flatten() {
                if let Some(near) = self.near_categories(category) {
                    note.push_str(&near);
                }
            }
            note
        });
        Ok(SearchResult {
            matches: found
                .into_iter()
                .take(limit)
                .map(|(_, entry)| entry.hit.clone())
                .collect(),
            total,
            note,
        })
    }

    /// The names the filters allow that are close to a query that matched nothing, closest first, as
    /// "Power Attack (feat `power-attack`)".
    fn near_names(&self, given: &str, allowed: &impl Fn(&&Entry) -> bool) -> Vec<String> {
        let mut near: Vec<(f64, &Entry)> = self
            .entries
            .iter()
            .filter(allowed)
            .filter_map(|entry| {
                let closeness =
                    closeness(given, &entry.hit.name).max(closeness(given, &entry.hit.key));
                (closeness >= FLOOR).then_some((closeness, entry))
            })
            .collect();
        near.sort_by(|(a_closeness, a), (b_closeness, b)| {
            b_closeness
                .total_cmp(a_closeness)
                .then_with(|| (a.hit.kind, &a.name).cmp(&(b.hit.kind, &b.name)))
        });
        near.into_iter()
            .take(NEAR)
            .map(|(_, entry)| {
                format!(
                    "{} ({} `{}`)",
                    entry.hit.name,
                    entry.hit.kind.noun(),
                    entry.hit.key
                )
            })
            .collect()
    }

    /// For a category no item has: says so, with the categories close to it.
    fn near_categories(&self, category: &str) -> Option<String> {
        if self
            .entries
            .iter()
            .any(|entry| entry.in_category(&[category.to_owned()]))
        {
            return None;
        }
        let mut known: Vec<&str> = self
            .entries
            .iter()
            .filter_map(|entry| entry.hit.category.as_deref())
            .collect();
        known.sort_unstable();
        known.dedup();
        let mut near: Vec<(f64, &str)> = known
            .into_iter()
            .map(|known| (closeness(category, known), known))
            .filter(|(closeness, _)| *closeness >= FLOOR)
            .collect();
        near.sort_by(|(a, a_name), (b, b_name)| b.total_cmp(a).then_with(|| a_name.cmp(b_name)));
        let near: Vec<String> = near
            .into_iter()
            .take(NEAR)
            .map(|(_, known)| format!("\"{known}\""))
            .collect();
        Some(match near.is_empty() {
            true => format!(" No item has the category \"{category}\"."),
            false => format!(
                " No item has the category \"{category}\"; close ones: {}.",
                near.join(", ")
            ),
        })
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

    /// Whether its category is one of these (case and spacing don't count), or, for an option, its Feature
    /// selection's key is.
    fn in_category(&self, categories: &[String]) -> bool {
        let same = |a: &str, b: &str| {
            a.split_whitespace()
                .map(str::to_lowercase)
                .eq(b.split_whitespace().map(str::to_lowercase))
        };
        categories.iter().any(|category| {
            self.hit
                .category
                .as_deref()
                .is_some_and(|own| same(own, category))
                || (self.hit.kind == Kind::SelectionOption
                    && self
                        .hit
                        .parent
                        .as_deref()
                        .is_some_and(|parent| same(parent, category)))
        })
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
