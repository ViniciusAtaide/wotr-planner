//! The change log (importer design §1.2, point 6): one tab-separated row per changed field, the review's input.

use std::collections::{BTreeMap, BTreeSet};

use super::{Change, NewFile};
use crate::Report;

/// The log: a header, then `entry`, `field`, `old`, `new`, `findings removed`, `fact`, a row per change in entry then
/// field order.
///
/// A finding removed is a group of the report (`area / kind`) that named the entry before the write and no longer
/// does after it; `-` for none. That is read per (`area / kind`, entry): a finding whose text changed, or one of two in
/// the same group, shows `-`. `fact` is `applied <ticket>` when the registry of the user's facts has the change applied, else `-`.
pub fn tsv(
    changes: &[Change],
    created: &[NewFile],
    facts: &BTreeMap<(String, &'static str), String>,
    before: &Report,
    after: &Report,
) -> String {
    let (before, after) = (named(before), named(after));
    let mut sorted: Vec<&Change> = changes.iter().collect();
    sorted.sort_by_key(|change| (change.at.to_string(), change.field));
    let mut out = String::from("entry\tfield\told\tnew\tfindings removed\tfact\n");
    // A created file has no old value; its new one is its key and blueprint, and no finding is read for it.
    let mut made: Vec<(String, String)> = created
        .iter()
        .map(|new| (new.at.to_string(), new_cell(new)))
        .collect();
    made.sort();
    for (entry, new) in made {
        out.push_str(&format!("{entry}\t(new file)\t-\t{new}\t-\t-\n"));
    }
    for change in sorted {
        let entry = change.at.to_string();
        let removed: Vec<&str> = before
            .iter()
            .filter(|(group, at)| *at == entry && !after.contains(&(group.clone(), at.clone())))
            .map(|(group, _)| group.as_str())
            .collect();
        let removed = if removed.is_empty() {
            "-".to_owned()
        } else {
            removed.join("; ")
        };
        // A prerequisites change is the census's `prerequisite` row (308b), apart from a carrier grant's `components`.
        let field = if change.kind == super::prerequisites::KIND {
            "prerequisite"
        } else {
            change.field
        };
        out.push_str(&format!(
            "{entry}\t{field}\t{}\t{}\t{removed}\t{}\n",
            flat(&change.old),
            flat(&change.new),
            facts
                .get(&(entry.clone(), change.field))
                .map_or("-", String::as_str)
        ));
    }
    out
}

/// A created file's new cell: `<key> <blueprint>`, read from its text.
fn new_cell(new: &NewFile) -> String {
    let field = |name: &str| {
        let span = super::span::field_span(&new.text, name)?;
        Some(
            new.text[span]
                .trim_end_matches(',')
                .split_once(':')?
                .1
                .trim()
                .trim_matches('"')
                .to_owned(),
        )
    };
    format!(
        "{} {}",
        field("key").unwrap_or_default(),
        field("blueprint").unwrap_or_default()
    )
}

/// A cell on one line.
fn flat(text: &str) -> String {
    text.replace(['\t', '\n'], " ")
}

/// The report's `(group, entry)` pairs: a group is the header's `area / kind` without its known ticket and count.
fn named(report: &Report) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let mut group: Option<String> = None;
    for line in report.text().lines() {
        if let Some(entry) = line.strip_prefix("  ") {
            if let (Some(group), Some((entry, _))) = (&group, entry.split_once(" | ")) {
                out.insert((group.clone(), entry.to_owned()));
            }
        } else if line.contains(" / ") && line.contains(": ") {
            let head = line.split(" [known").next().unwrap_or(line);
            let head = head.rsplit_once(": ").map_or(head, |(head, _)| head);
            group = Some(head.to_owned());
        } else {
            group = None;
        }
    }
    out
}
