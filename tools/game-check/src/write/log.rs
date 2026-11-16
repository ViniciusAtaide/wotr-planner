//! The change log (importer design §1.2, point 6): one tab-separated row per changed field, the review's input.

use std::collections::BTreeSet;

use super::Change;
use crate::Report;

/// The log: a header, then `entry`, `field`, `old`, `new`, `findings removed`, `fact`, a row per change in entry then
/// field order.
///
/// A finding removed is a group of the report (`area / kind`) that named the entry before the write and no longer
/// does after it; `-` for none. `fact` is `-` until the registry of the user's facts (293e) fills it.
pub fn tsv(changes: &[Change], before: &Report, after: &Report) -> String {
    let (before, after) = (named(before), named(after));
    let mut sorted: Vec<&Change> = changes.iter().collect();
    sorted.sort_by_key(|change| (change.at.to_string(), change.field));
    let mut out = String::from("entry\tfield\told\tnew\tfindings removed\tfact\n");
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
        out.push_str(&format!(
            "{entry}\t{}\t{}\t{}\t{removed}\t-\n",
            change.field,
            flat(&change.old),
            flat(&change.new)
        ));
    }
    out
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
