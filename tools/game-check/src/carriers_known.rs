//! The known carrier hits (`carriers-known.txt`, ticket 322): each hit of the carrier audit
//! ([`prerequisites::audit`](crate::write::prerequisites::audit), 308h) with the ticket that will remove it. The plain
//! game check runs the audit and compares, as it does its findings with `known.txt`: a hit no line lists fails it, and
//! so does a line no hit matches.
//!
//! One line a hit, `#` starting a comment line:
//!
//! ```text
//! <entry> | holds: <fact> granted by <grantor> <GUID>, which <planner entry> doesn't grant -> <ticket>
//! <entry> | left whole: <fact> granted by <grantor> <GUID>, which <planner entry> doesn't grant -> <ticket>
//! ```
//!
//! A line matches the hit whose text before its last ` -> ` (the hit's own owner, 307 for a carrier) it repeats, so its
//! ticket is the main session's to set. [`KnownCarriers::regenerate`] writes the file again from the audit
//! (`game-check --carrier-audit --known-format`), keeping its header and each listed hit's ticket.

use crate::Error;

/// The list's lines, each a hit and its ticket.
#[derive(Debug, Default)]
pub struct KnownCarriers {
    lines: Vec<Line>,
}

#[derive(Debug)]
struct Line {
    number: usize,
    /// The hit's text before its owner.
    hit: String,
    ticket: String,
}

/// The audit's hits against the list: the hits no line lists, and the lines no hit matches (`line <n>: <line>`).
#[derive(Debug, Default)]
pub struct CarrierCheck {
    pub hits: usize,
    pub unlisted: Vec<String>,
    pub stale: Vec<String>,
}

/// The repo's own list.
const EMBEDDED: &str = include_str!("../carriers-known.txt");

/// A hit's text before its owner, and its owner (307 when it names none).
fn split(hit: &str) -> (&str, &str) {
    hit.rsplit_once(" -> ").unwrap_or((hit, "307"))
}

impl KnownCarriers {
    /// The repo's own list, `tools/game-check/carriers-known.txt`.
    pub fn embedded() -> Result<KnownCarriers, Error> {
        KnownCarriers::parse(EMBEDDED)
    }

    /// The list's text: a line without ` -> <ticket>`, or a hit listed twice, is refused.
    pub fn parse(text: &str) -> Result<KnownCarriers, Error> {
        let mut lines: Vec<Line> = Vec::new();
        for (index, line) in text.lines().enumerate() {
            let number = index + 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let bad = |why: &str| Error::Known(format!("carriers-known.txt line {number}: {why}"));
            let (hit, ticket) = line
                .rsplit_once(" -> ")
                .filter(|(_, ticket)| !ticket.trim().is_empty())
                .ok_or_else(|| bad("no ` -> <ticket>`"))?;
            if lines.iter().any(|known| known.hit == hit) {
                return Err(bad("a hit listed twice"));
            }
            lines.push(Line {
                number,
                hit: hit.to_owned(),
                ticket: ticket.trim().to_owned(),
            });
        }
        Ok(KnownCarriers { lines })
    }

    fn ticket_for(&self, hit: &str) -> Option<&Line> {
        let (hit, _) = split(hit);
        self.lines.iter().find(|line| line.hit == hit)
    }

    /// The audit's `hits` against the list.
    pub fn compare(&self, hits: &[String]) -> CarrierCheck {
        let mut used = vec![false; self.lines.len()];
        let mut unlisted = Vec::new();
        for hit in hits {
            let (text, _) = split(hit);
            match self.lines.iter().position(|line| line.hit == text) {
                Some(index) => used[index] = true,
                None => unlisted.push(hit.clone()),
            }
        }
        let stale = (self.lines.iter().zip(&used))
            .filter(|(_, used)| !**used)
            .map(|(line, _)| format!("line {}: {} -> {}", line.number, line.hit, line.ticket))
            .collect();
        CarrierCheck {
            hits: hits.len(),
            unlisted,
            stale,
        }
    }

    /// The list written again from the audit's `hits`: `text`'s header (its comment and blank lines before the first
    /// hit), then each hit in the audit's order, with the ticket its listed line names or else its own owner.
    pub fn regenerate(text: &str, hits: &[String]) -> Result<String, Error> {
        let known = KnownCarriers::parse(text)?;
        let mut out: String = (text.lines())
            .take_while(|line| line.trim().is_empty() || line.trim_start().starts_with('#'))
            .map(|line| format!("{line}\n"))
            .collect();
        for hit in hits {
            let (text, owner) = split(hit);
            let ticket = known
                .ticket_for(hit)
                .map_or(owner, |line| line.ticket.as_str());
            out.push_str(&format!("{text} -> {ticket}\n"));
        }
        Ok(out)
    }

    /// [`regenerate`](Self::regenerate) on the repo's own list.
    pub fn regenerate_embedded(hits: &[String]) -> Result<String, Error> {
        KnownCarriers::regenerate(EMBEDDED, hits)
    }
}
