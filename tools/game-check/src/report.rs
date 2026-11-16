//! The findings and the report: each area's counts, then the differences grouped by area and kind, largest group
//! first. Everything is sorted, so the same dump and data give the same bytes.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use rules::BlueprintGuid;

use crate::Pins;
use crate::known::Known;

/// What the check compares, in the order the report lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Area {
    /// Each entry's blueprint exists and is of a type its kind can be.
    Mapping,
    Prerequisites,
    /// Skill points, class skills, BAB and saves.
    ClassTables,
    /// Each level's grants, for classes, archetypes and prestige classes.
    ProgressionRows,
    /// A selection's options or feats, and the general and mythic pools.
    OptionLists,
    MythicRanks,
    /// Gear's always-on bonuses and slot.
    Items,
}

impl Area {
    pub const ALL: [Area; 7] = [
        Area::Mapping,
        Area::Prerequisites,
        Area::ClassTables,
        Area::ProgressionRows,
        Area::OptionLists,
        Area::MythicRanks,
        Area::Items,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Area::Mapping => "mapping",
            Area::Prerequisites => "prerequisites",
            Area::ClassTables => "class tables",
            Area::ProgressionRows => "progression rows",
            Area::OptionLists => "option lists",
            Area::MythicRanks => "mythic ranks",
            Area::Items => "items",
        }
    }

    pub fn parse(name: &str) -> Option<Area> {
        Area::ALL.into_iter().find(|area| area.name() == name)
    }
}

/// One difference: the entry, its blueprint, and the two values. Ordered by area and kind, then within a group by
/// entry, GUID and the two values.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub area: Area,
    pub kind: &'static str,
    /// The planner's entry: `feat power-attack`.
    pub entry: String,
    /// The entry's blueprint, by GUID and asset name.
    pub guid: BlueprintGuid,
    pub blueprint: String,
    pub game: String,
    pub planner: String,
}

/// What the areas found: their findings, and per area how many entries they compared, how many agreed, and what
/// they left out and why.
#[derive(Debug, Default)]
pub struct Out {
    pub findings: Vec<Finding>,
    tallies: BTreeMap<Area, Tally>,
}

#[derive(Debug, Default)]
struct Tally {
    compared: usize,
    agree: usize,
    skipped: BTreeMap<String, usize>,
}

impl Out {
    /// Where the next entry's findings start: pass it to [`compared`](Self::compared) once the entry is done.
    pub fn mark(&self) -> usize {
        self.findings.len()
    }

    /// One entry compared in `area`; it agrees when it added no finding since `mark`.
    pub fn compared(&mut self, area: Area, mark: usize) {
        let tally = self.tallies.entry(area).or_default();
        tally.compared += 1;
        if self.findings.len() == mark {
            tally.agree += 1;
        }
    }

    /// Something `area` left out, and why.
    pub fn skip(&mut self, area: Area, reason: impl Into<String>) {
        *self
            .tallies
            .entry(area)
            .or_default()
            .skipped
            .entry(reason.into())
            .or_default() += 1;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn finding(
        &mut self,
        area: Area,
        kind: &'static str,
        entry: impl ToString,
        guid: &BlueprintGuid,
        blueprint: &str,
        game: impl Into<String>,
        planner: impl Into<String>,
    ) {
        self.findings.push(Finding {
            area,
            kind,
            entry: entry.to_string(),
            guid: guid.clone(),
            blueprint: blueprint.to_owned(),
            game: game.into(),
            planner: planner.into(),
        });
    }
}

/// The check's report.
#[derive(Debug)]
pub struct Report {
    text: String,
    failed: bool,
}

impl Report {
    pub fn build(
        pins: &Pins,
        blueprints: usize,
        entries: (usize, usize),
        out: Out,
        known: &Known,
    ) -> Report {
        let mut text = String::new();
        let (all, with_blueprint) = entries;
        let _ = writeln!(text, "The game check (ticket 238)");
        let _ = writeln!(
            text,
            "Blueprints: {}, SHA-256 {}, {blueprints} blueprints.",
            pins.asset, pins.sha256
        );
        let _ = writeln!(
            text,
            "Game data: {all} entries of the kinds that can name a blueprint, {with_blueprint} naming one."
        );
        let _ = writeln!(text, "\nAreas:");
        for area in Area::ALL {
            let Some(tally) = out.tallies.get(&area) else {
                let _ = writeln!(text, "  {}: nothing compared", area.name());
                continue;
            };
            let _ = write!(
                text,
                "  {}: {} compared, {} agree, {} differ",
                area.name(),
                tally.compared,
                tally.agree,
                tally.compared - tally.agree
            );
            if !tally.skipped.is_empty() {
                let skipped: Vec<String> = tally
                    .skipped
                    .iter()
                    .map(|(reason, count)| format!("{count} {reason}"))
                    .collect();
                let _ = write!(text, "; left out: {}", skipped.join(", "));
            }
            let _ = writeln!(text);
        }

        let mut groups: BTreeMap<(Area, &'static str, Option<&str>), Vec<&Finding>> =
            BTreeMap::new();
        let mut used = vec![false; known.len()];
        for finding in &out.findings {
            let ticket = known.ticket_for(finding).map(|(line, ticket)| {
                used[line] = true;
                ticket
            });
            groups
                .entry((finding.area, finding.kind, ticket))
                .or_default()
                .push(finding);
        }
        let mut groups: Vec<_> = groups.into_iter().collect();
        groups.sort_by(|(a_key, a), (b_key, b)| b.len().cmp(&a.len()).then(a_key.cmp(b_key)));

        let known_count: usize = groups
            .iter()
            .filter(|((_, _, ticket), _)| ticket.is_some())
            .map(|(_, findings)| findings.len())
            .sum();
        let known_groups = groups
            .iter()
            .filter(|((_, _, ticket), _)| ticket.is_some())
            .count();
        let _ = writeln!(
            text,
            "\nDifferences: {} in {} groups; {} not known in {} groups, {known_count} known in {known_groups} groups.",
            out.findings.len(),
            groups.len(),
            out.findings.len() - known_count,
            groups.len() - known_groups,
        );
        for ((area, kind, ticket), mut findings) in groups {
            findings.sort();
            let known = ticket.map_or_else(String::new, |ticket| format!(" [known: {ticket}]"));
            let _ = writeln!(
                text,
                "\n{} / {kind}{known}: {}",
                area.name(),
                findings.len()
            );
            for finding in findings {
                let _ = writeln!(
                    text,
                    "  {} | {} {} | game: {} | planner: {}",
                    finding.entry, finding.blueprint, finding.guid, finding.game, finding.planner
                );
            }
        }

        let stale: Vec<_> = known
            .lines()
            .zip(&used)
            .filter(|(_, used)| !**used)
            .collect();
        if !stale.is_empty() {
            let _ = writeln!(text, "\nKnown lines that match no difference:");
            for (line, _) in &stale {
                let _ = writeln!(text, "  {line}");
            }
        }
        let failed = known_count < out.findings.len() || !stale.is_empty();
        Report { text, failed }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether a difference is not known, or a known line matches none: the check's exit status is then 1.
    pub fn failed(&self) -> bool {
        self.failed
    }
}
