//! The known differences (`known.txt`): each a group, or one entry of a group, with the ticket that will remove it.
//!
//! One line each, `#` starting a comment:
//!
//! ```text
//! <area> -> <ticket>
//! <area> / <kind> -> <ticket>
//! <area> / <kind> / <entry> -> <ticket>
//! <area> / <kind> / <entry> / <value> -> <ticket>
//! ```
//!
//! A kind of `*` is any kind; a value is the finding's game or planner value, whole. The ticket is its number or,
//! before it has one, its file name without `.md`. A finding takes the most specific line that matches it. Known
//! findings are reported in groups of their own and don't fail the check; a line that matches no finding does.

use crate::Error;
use crate::report::{Area, Finding};

#[derive(Debug, Default)]
pub struct Known {
    lines: Vec<Line>,
}

#[derive(Debug)]
struct Line {
    number: usize,
    area: Area,
    kind: Option<String>,
    entry: Option<String>,
    value: Option<String>,
    ticket: String,
}

impl Known {
    /// The repo's own list, `tools/game-check/known.txt`.
    pub fn embedded() -> Result<Known, Error> {
        Known::parse(include_str!("../known.txt"))
    }

    pub fn parse(text: &str) -> Result<Known, Error> {
        let mut lines = Vec::new();
        for (index, line) in text.lines().enumerate() {
            let number = index + 1;
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let bad = |why: &str| Error::Known(format!("line {number}: {why}"));
            let (what, ticket) = line
                .split_once(" -> ")
                .ok_or_else(|| bad("no ` -> <ticket>`"))?;
            let mut parts = what.split(" / ").map(str::trim);
            let area = parts.next().unwrap_or("");
            let area = Area::parse(area).ok_or_else(|| bad(&format!("no area `{area}`")))?;
            let kind = parts.next().map(str::to_owned);
            let entry = parts.next().map(str::to_owned);
            let value = parts.next().map(str::to_owned);
            if parts.next().is_some() {
                return Err(bad("more than area, kind, entry and value"));
            }
            let ticket = ticket.trim();
            if ticket.is_empty() {
                return Err(bad("no ticket"));
            }
            lines.push(Line {
                number,
                area,
                kind,
                entry,
                value,
                ticket: ticket.to_owned(),
            });
        }
        Ok(Known { lines })
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// The most specific line that covers `finding`: its index and its ticket.
    pub fn ticket_for(&self, finding: &Finding) -> Option<(usize, &str)> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                line.area == finding.area
                    && line
                        .kind
                        .as_deref()
                        .is_none_or(|kind| kind == "*" || kind == finding.kind)
                    && line
                        .entry
                        .as_deref()
                        .is_none_or(|entry| entry == finding.entry)
                    && line
                        .value
                        .as_deref()
                        .is_none_or(|value| value == finding.game || value == finding.planner)
            })
            .max_by_key(|(index, line)| {
                (
                    line.value.is_some(),
                    line.entry.is_some(),
                    line.kind.as_deref().is_some_and(|kind| kind != "*"),
                    std::cmp::Reverse(*index),
                )
            })
            .map(|(index, line)| (index, line.ticket.as_str()))
    }

    /// Each line as the report names it: `line 12: progression rows / in game only -> 240`.
    pub fn lines(&self) -> impl Iterator<Item = String> + '_ {
        self.lines.iter().map(|line| {
            let mut what = line.area.name().to_owned();
            for part in [&line.kind, &line.entry, &line.value].into_iter().flatten() {
                what.push_str(" / ");
                what.push_str(part);
            }
            format!("line {}: {what} -> {}", line.number, line.ticket)
        })
    }
}
