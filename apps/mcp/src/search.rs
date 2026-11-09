//! `search`: every item the agent may name, matched by name, key or initials, or by the words of its summary
//! (ticket 123), and ranked.

use rules::{Engine, GameData, GuideRef, Prerequisite};

use crate::planner::all_selections;
use crate::shapes::{Hit, Kind, Matched, SearchResult};
use crate::similar::{FLOOR, closeness, words};

/// How many matches a search returns when it doesn't say (for a query, and for an empty one listing what the filters
/// allow), and at most.
const DEFAULT_LIMIT: usize = 10;
const EMPTY_QUERY_LIMIT: usize = 50;
const MAX_LIMIT: usize = 100;

/// A prerequisite text longer than this, in characters, is shortened in `search`'s hits: each group of more than
/// [`MOST_ALTERNATIVES`] alternatives reads "one of 49 hexes (Air Barrier, Battle Master, Battle Ward, …)"
/// (ticket 128). `describe` gives the whole text.
const LONG_PREREQUISITE: usize = 200;
const MOST_ALTERNATIVES: usize = 6;

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
    /// The summary's words, lower case, apostrophes dropped.
    summary_words: Vec<String>,
    /// The prerequisite shortened, when its whole text is long ([`LONG_PREREQUISITE`]).
    short_prerequisite: Option<String>,
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
    /// Some query word is a whole word of the summary; `missing` counts the query words it lacks, so a summary
    /// holding every word comes first.
    Summary {
        missing: usize,
    },
}

impl SearchIndex {
    pub(crate) fn new(data: &GameData, engine: &Engine) -> Self {
        let mut entries = Vec::new();
        // An item and its prerequisite: the whole text on the hit, and a shortened one for a long text.
        let mut add = |hit: Hit, needs: &Option<Prerequisite>| {
            let short = needs.as_ref().and_then(|needs| {
                let full = hit.prerequisite.as_ref()?.as_deref()?;
                (full.chars().count() > LONG_PREREQUISITE)
                    .then(|| engine.short_prerequisite_text(needs, MOST_ALTERNATIVES))
                    .filter(|short| short != full)
            });
            entries.push(Entry::new(hit, short));
        };
        let prerequisite = |p: &Option<Prerequisite>| {
            Some(
                p.as_ref()
                    .map(|prerequisite| engine.prerequisite_text(prerequisite, None)),
            )
        };
        let guide = |guide: &GuideRef| Some(guide.url());
        let summary = |text: &str| (!text.is_empty()).then(|| text.to_owned());

        for (classes, category) in [
            (&data.classes, "Base class"),
            (&data.prestige_classes, "Prestige class"),
        ] {
            for class in classes {
                add(
                    Hit {
                        kind: Kind::Class,
                        key: class.key.as_str().to_owned(),
                        name: class.name.clone(),
                        summary: summary(&class.summary),
                        parent: None,
                        category: Some(category.to_owned()),
                        prerequisite: prerequisite(&class.prerequisite),
                        guide_url: guide(&class.guide),
                        matched: None,
                    },
                    &class.prerequisite,
                );
                for archetype in &class.archetypes {
                    add(
                        Hit {
                            kind: Kind::Archetype,
                            key: archetype.key.as_str().to_owned(),
                            name: archetype.name.clone(),
                            summary: summary(&archetype.summary),
                            parent: Some(class.key.as_str().to_owned()),
                            category: None,
                            prerequisite: prerequisite(&archetype.prerequisite),
                            guide_url: guide(&archetype.guide),
                            matched: None,
                        },
                        &archetype.prerequisite,
                    );
                }
            }
        }
        for path in &data.mythic_paths {
            add(
                Hit {
                    kind: Kind::Path,
                    key: path.key.as_str().to_owned(),
                    name: path.name.clone(),
                    summary: summary(&path.summary),
                    parent: None,
                    category: None,
                    prerequisite: Some(None),
                    guide_url: guide(&path.guide),
                    matched: None,
                },
                &None,
            );
        }
        for feat in &data.feats {
            add(
                Hit {
                    kind: Kind::Feat,
                    key: feat.key.as_str().to_owned(),
                    name: feat.name.clone(),
                    summary: summary(&feat.summary),
                    parent: None,
                    category: Some(feat.category.name().to_owned()),
                    prerequisite: prerequisite(&feat.prerequisite),
                    guide_url: guide(&feat.guide),
                    matched: None,
                },
                &feat.prerequisite,
            );
        }
        for ability in &data.mythic_abilities {
            add(
                Hit {
                    kind: Kind::MythicAbility,
                    key: ability.key.as_str().to_owned(),
                    name: ability.name.clone(),
                    summary: summary(&ability.summary),
                    parent: None,
                    category: Some(ability.group.name().to_owned()),
                    prerequisite: prerequisite(&ability.prerequisite),
                    guide_url: guide(&ability.guide),
                    matched: None,
                },
                &ability.prerequisite,
            );
        }
        for feat in &data.mythic_feats {
            add(
                Hit {
                    kind: Kind::MythicFeat,
                    key: feat.key.as_str().to_owned(),
                    name: feat.name.clone(),
                    summary: summary(&feat.summary),
                    parent: None,
                    category: Some(feat.group.name().to_owned()),
                    prerequisite: prerequisite(&feat.prerequisite),
                    guide_url: guide(&feat.guide),
                    matched: None,
                },
                &feat.prerequisite,
            );
        }
        for selection in all_selections(data) {
            for option in &selection.options {
                add(
                    Hit {
                        kind: Kind::SelectionOption,
                        key: option.key.as_str().to_owned(),
                        name: option.name.clone(),
                        summary: summary(&option.summary),
                        parent: Some(selection.key.as_str().to_owned()),
                        category: Some(selection.name.clone()),
                        prerequisite: prerequisite(&option.prerequisite),
                        guide_url: guide(&option.guide),
                        matched: None,
                    },
                    &option.prerequisite,
                );
            }
        }
        for race in &data.races {
            add(
                Hit {
                    kind: Kind::Race,
                    key: race.key.as_str().to_owned(),
                    name: race.name.clone(),
                    summary: summary(&race.summary),
                    parent: None,
                    category: None,
                    prerequisite: Some(None),
                    guide_url: guide(&race.guide),
                    matched: None,
                },
                &None,
            );
        }
        for deity in &data.deities {
            add(
                Hit {
                    kind: Kind::Deity,
                    key: deity.key.as_str().to_owned(),
                    name: deity.name.clone(),
                    summary: None,
                    parent: None,
                    category: None,
                    prerequisite: Some(None),
                    guide_url: guide(&deity.guide),
                    matched: None,
                },
                &None,
            );
        }
        for background in &data.backgrounds {
            add(
                Hit {
                    kind: Kind::Background,
                    key: background.key.as_str().to_owned(),
                    name: background.name.clone(),
                    summary: summary(&background.note),
                    parent: None,
                    category: background.group.map(|group| group.name().to_owned()),
                    prerequisite: Some(None),
                    // The user's list, not the guide's: no link.
                    guide_url: None,
                    matched: None,
                },
                &None,
            );
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
        compact: bool,
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
        let by_summary = |found: &[(Rank, &Entry)]| {
            found
                .iter()
                .filter(|(rank, _)| matches!(rank, Rank::Summary { .. }))
                .count()
        };
        let mut notes = Vec::new();
        if found.is_empty() {
            notes.push(self.no_match_note(given, query.is_some(), kinds, categories, &allowed));
        } else if by_summary(&found) == found.len() {
            notes.push(format!(
                "No item's name, key or initials match `{given}`: every match is by its summary's words \
                 (`matched: \"summary\"`)."
            ));
        }
        let hidden = by_summary(found.get(limit..).unwrap_or_default());
        if hidden > 0 {
            notes.push(format!(
                "`limit` left out {hidden} more {} by summary (`total` counts {}): raise `limit` to see {}.",
                plural(hidden, "match", "matches"),
                plural(hidden, "it", "them"),
                plural(hidden, "it", "them"),
            ));
        }
        let note = (!notes.is_empty()).then(|| notes.join(" "));
        Ok(SearchResult {
            matches: found
                .into_iter()
                .take(limit)
                .map(|(rank, entry)| entry.found(rank, compact))
                .collect(),
            total,
            note,
        })
    }

    /// The note for a search that matched nothing: says so, and names the near names and categories.
    fn no_match_note(
        &self,
        given: &str,
        has_query: bool,
        kinds: Option<&[Kind]>,
        categories: Option<&[String]>,
        allowed: &impl Fn(&&Entry) -> bool,
    ) -> String {
        let mut note = match has_query {
            true => format!("No item in the planner's data matches `{given}`"),
            false => "No item in the planner's data matches".to_owned(),
        };
        if kinds.is_some() || categories.is_some() {
            note.push_str(" with these filters");
        }
        note.push_str(
            ". The data holds what the guide lists: an item that isn't there can't be taken in a Build.",
        );
        if has_query {
            let near = self.near_names(given, allowed);
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
    fn new(hit: Hit, short_prerequisite: Option<String>) -> Self {
        let words = words(&hit.name);
        Self {
            short_prerequisite,
            summary_words: hit.summary.as_deref().map(self::words).unwrap_or_default(),
            name: hit.name.to_lowercase(),
            initials: words
                .iter()
                .filter_map(|word| word.chars().next())
                .collect(),
            words,
            hit,
        }
    }

    /// Its hit as `search` gives it: marked when it matched by its summary, its long prerequisite shortened, and
    /// `compact`, only its kind, key, name and category.
    fn found(&self, rank: Rank, compact: bool) -> Hit {
        let matched = matches!(rank, Rank::Summary { .. }).then_some(Matched::Summary);
        match compact {
            true => Hit {
                kind: self.hit.kind,
                key: self.hit.key.clone(),
                name: self.hit.name.clone(),
                summary: None,
                parent: None,
                category: self.hit.category.clone(),
                prerequisite: None,
                guide_url: None,
                matched,
            },
            false => Hit {
                matched,
                prerequisite: match &self.short_prerequisite {
                    Some(short) => Some(Some(short.clone())),
                    None => self.hit.prerequisite.clone(),
                },
                ..self.hit.clone()
            },
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
            let missing = query
                .words
                .iter()
                .filter(|word| {
                    !self
                        .summary_words
                        .iter()
                        .any(|summary_word| same_word(word, summary_word))
                })
                .count();
            (missing < query.words.len()).then_some(Rank::Summary { missing })
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

/// Whether a summary's word is a query's word, as a whole word: the same, or one is the other's plural ("DC" and
/// "DCs", "hex" and "hexes"). A word inside another ("DC" in "DCrossbow") isn't.
fn same_word(a: &str, b: &str) -> bool {
    let plural_of = |plural: &str, single: &str| {
        plural.strip_suffix('s') == Some(single) || plural.strip_suffix("es") == Some(single)
    };
    a == b || plural_of(a, b) || plural_of(b, a)
}

/// One word or the other, by a count.
fn plural(count: usize, one: &'static str, many: &'static str) -> &'static str {
    match count {
        1 => one,
        _ => many,
    }
}
