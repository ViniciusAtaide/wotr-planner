//! The user's facts in the fields the importer writes (importer design §3.2): a registry in `known.txt`'s line
//! grammar, so a run writes a change to one only under an `applied` line and otherwise hands it to the user.
//!
//! One line each, `#` starting a comment:
//!
//! ```text
//! <area> / <kind> / <entry> -> applied <ticket>
//! <area> / <kind> -> held
//! <area> -> held
//! ```
//!
//! A kind of `*` is any kind. A change takes the most specific line that matches it (an entry beats a kind beats the
//! area). `held`: not written, listed "to the user", the finding stays. `applied <ticket>`: the user decided and the
//! ticket applied it, so the change is written and the log says so. A change no line names is free. A line naming an
//! entry the data doesn't have is stale and fails the run ([`Facts::stale`]), as a stale `known.txt` line fails the
//! check. The grammar's optional value has no meaning for a change and is refused.

use std::collections::BTreeSet;

use rules::GameData;

use crate::Error;
use crate::report::Area;

#[derive(Debug, Default)]
pub struct Facts {
    lines: Vec<Line>,
}

#[derive(Debug)]
struct Line {
    number: usize,
    area: Area,
    kind: Option<String>,
    entry: Option<String>,
    decision: Decision,
}

/// What the registry says of a change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    /// No line names it.
    Free,
    /// The user decided it, and this ticket applied it.
    Applied(String),
    /// The user's fact, not applied: it goes to the user.
    Held,
}

impl Facts {
    /// The repo's own registry, `tools/game-check/user-facts.txt`.
    pub fn embedded() -> Result<Facts, Error> {
        Facts::parse(include_str!("../../user-facts.txt"))
    }

    pub fn parse(text: &str) -> Result<Facts, Error> {
        let mut lines = Vec::new();
        for (index, line) in text.lines().enumerate() {
            let number = index + 1;
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let bad = |why: &str| Error::Known(format!("user-facts.txt line {number}: {why}"));
            let (what, decision) = line
                .split_once(" -> ")
                .ok_or_else(|| bad("no ` -> applied <ticket>` or ` -> held`"))?;
            let decision = match decision.trim().split_once(' ') {
                None if decision.trim() == "held" => Decision::Held,
                Some(("applied", ticket)) if !ticket.trim().is_empty() => {
                    Decision::Applied(ticket.trim().to_owned())
                }
                _ => return Err(bad("the right side is `applied <ticket>` or `held`")),
            };
            let mut parts = what.split(" / ").map(str::trim);
            let area = parts.next().unwrap_or("");
            let area = Area::parse(area).ok_or_else(|| bad(&format!("no area `{area}`")))?;
            let kind = parts.next().map(str::to_owned);
            let entry = parts.next().map(str::to_owned);
            if parts.next().is_some() {
                return Err(bad("more than area, kind and entry"));
            }
            // Two lines of one specificity for one thing (`area` and `area / *` are one) would leave the winner to their order.
            let same = |other: &Line| {
                other.area == area
                    && other.entry == entry
                    && other.kind.as_deref().unwrap_or("*") == kind.as_deref().unwrap_or("*")
            };
            if let Some(first) = lines.iter().find(|other| same(other)) {
                return Err(bad(&format!("the same as line {}", first.number)));
            }
            lines.push(Line {
                number,
                area,
                kind,
                entry,
                decision,
            });
        }
        Ok(Facts { lines })
    }

    /// The area and kind of every line that names a kind (not `*`), for a test that each is one the check emits.
    pub fn kinds(&self) -> Vec<(Area, String)> {
        self.lines
            .iter()
            .filter_map(|line| {
                line.kind
                    .as_ref()
                    .filter(|kind| *kind != "*")
                    .map(|kind| (line.area, kind.clone()))
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// What the registry says of a change in `area` of `kind` to `entry` (`gear item cloak-of-test`): the most
    /// specific matching line's decision.
    pub fn decide(&self, area: Area, kind: &str, entry: &str) -> Decision {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                line.area == area
                    && line.kind.as_deref().is_none_or(|k| k == "*" || k == kind)
                    && line.entry.as_deref().is_none_or(|e| e == entry)
            })
            .max_by_key(|(index, line)| {
                (
                    line.entry.is_some(),
                    line.kind.as_deref().is_some_and(|k| k != "*"),
                    std::cmp::Reverse(*index),
                )
            })
            .map_or(Decision::Free, |(_, line)| line.decision.clone())
    }

    /// The lines that name an entry the data doesn't have, as `line 12: class tables / * / class x -> held`.
    pub fn stale(&self, data: &GameData) -> Vec<String> {
        let entries: BTreeSet<String> = data
            .blueprints()
            .into_iter()
            .map(|(at, _)| at.to_string())
            .collect();
        self.lines
            .iter()
            .filter(|line| {
                line.entry
                    .as_ref()
                    .is_some_and(|entry| !entries.contains(entry))
            })
            .map(|line| {
                let mut what = line.area.name().to_owned();
                for part in [&line.kind, &line.entry].into_iter().flatten() {
                    what.push_str(" / ");
                    what.push_str(part);
                }
                format!("line {}: {what}", line.number)
            })
            .collect()
    }
}
