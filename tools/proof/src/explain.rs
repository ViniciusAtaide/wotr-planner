//! Two dumps ([`crate::dump`]) compared and the differences explained (ticket 203): grouped as the snapshot check
//! groups its message (`libs/seeds/tests/game_data/snapshots.rs`), by path with its levels, ranks and indexes
//! wildcarded and by the change, `old → new`, largest group first.
//!
//! Each dump is sorted by `(build, path)`, so the two are read side by side and one Build's lines are all that is held
//! at a time (the groups are held too: a count, three example Builds and the change's text each).
//!
//! - A **group** is one wildcarded path with one change: `a → b` (the line differs), `+ b` (a line only in B), `- a`
//!   (only in A). It counts the Builds and the lines it covers, and names three of its Builds.
//! - A **slot only on one side** (a `slot.<id>.kind` line only in one dump) is one group of its own, `slot only in
//!   A (kind)`, whose lines are the slot's own: its options aren't listed again as lines only in A.
//! - A **Build only on one side** is a group of its own, `Build only in A`.
//! - A value longer than [`WIDTH`] is cut in its group's line: a change of a long text shows the part where the two
//!   differ, with `…` for the rest, and the whole value is in the list at the end, each as `[n]`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::{self, BufRead};

/// How many characters of a value a group's line shows.
pub const WIDTH: usize = 100;

/// How many groups the text lists unless told otherwise.
pub const DEFAULT_TOP: usize = 400;

/// One line of a dump.
struct Line {
    build: String,
    path: String,
    value: String,
}

/// A dump read line by line, checked to be in order.
struct Dump<R> {
    lines: io::Lines<R>,
    name: &'static str,
    number: usize,
    last: Option<(String, String)>,
    /// The next line, read ahead.
    head: Option<Line>,
}

impl<R: BufRead> Dump<R> {
    fn new(reader: R, name: &'static str) -> Result<Self, String> {
        let mut dump = Dump {
            lines: reader.lines(),
            name,
            number: 0,
            last: None,
            head: None,
        };
        dump.advance()?;
        Ok(dump)
    }

    fn advance(&mut self) -> Result<(), String> {
        self.head = None;
        while let Some(line) = self.lines.next() {
            self.number += 1;
            let at = |why: &str| format!("dump {}, line {}: {why}", self.name, self.number);
            let line = line.map_err(|error| at(&error.to_string()))?;
            if line.is_empty() {
                continue;
            }
            let mut parts = line.splitn(3, '\t');
            let (Some(build), Some(path), Some(value)) = (parts.next(), parts.next(), parts.next())
            else {
                return Err(at("not `<build>\\t<path>\\t<value>`"));
            };
            let key = (build.to_owned(), path.to_owned());
            if self.last.as_ref().is_some_and(|last| *last >= key) {
                return Err(at(
                    "out of order, or a path twice (a dump is sorted by build, then path)",
                ));
            }
            self.head = Some(Line {
                build: key.0.clone(),
                path: key.1.clone(),
                value: value.to_owned(),
            });
            self.last = Some(key);
            return Ok(());
        }
        Ok(())
    }

    /// The name of the Build the next line is of.
    fn next_build(&self) -> Option<&str> {
        self.head.as_ref().map(|line| line.build.as_str())
    }

    /// All the lines of the Build `name`, as `(path, value)`.
    fn take_build(&mut self, name: &str) -> Result<Vec<(String, String)>, String> {
        let mut lines = Vec::new();
        while self.next_build() == Some(name) {
            let line = self.head.take().expect("a head");
            lines.push((line.path, line.value));
            self.advance()?;
        }
        Ok(lines)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Change {
    Added(String),
    Removed(String),
    Changed(String, String),
    /// A `-hash` line that changed: its values say nothing, so they don't split the group.
    Rehashed,
    SlotOnly(char, String),
    BuildOnly(char),
}

/// What a group covers.
#[derive(Default)]
struct Group {
    builds: usize,
    lines: usize,
    examples: Vec<String>,
    last_build: String,
}

/// The path with each segment that is a number, wildcarded: `slot.Feat.*.General.*.option/power-attack`. A number
/// after the `/` (part of an option's key) stays.
pub fn wildcard(path: &str) -> String {
    let (head, tail) = path
        .split_once('/')
        .map_or((path, None), |(head, tail)| (head, Some(tail)));
    let head: Vec<&str> = head
        .split('.')
        .map(|segment| {
            let numbered = segment.split('#').next().unwrap_or(segment);
            if !numbered.is_empty() && numbered.bytes().all(|b| b.is_ascii_digit()) {
                "*"
            } else {
                segment
            }
        })
        .collect();
    match tail {
        Some(tail) => format!("{}/{tail}", head.join(".")),
        None => head.join("."),
    }
}

/// The values of a change shown at most [`WIDTH`] long: the part where two long values differ when both are long,
/// else the start of each. Each cut value is kept whole in `full`, and shown with its `[n]`.
struct Shown {
    full: Vec<String>,
}

impl Shown {
    fn reference(&mut self, value: &str) -> usize {
        match self.full.iter().position(|full| full == value) {
            Some(at) => at + 1,
            None => {
                self.full.push(value.to_owned());
                self.full.len()
            }
        }
    }

    fn one(&mut self, value: &str) -> String {
        if value.chars().count() <= WIDTH {
            return value.to_owned();
        }
        let cut: String = value.chars().take(WIDTH).collect();
        format!("{cut}…[{}]", self.reference(value))
    }

    /// Two values: as [`Shown::one`] when both are short. When one is long and both are lists of alternatives
    /// (`Needs a or b or c`), the heads of the two and what the lists add and drop; else the part where they differ.
    fn two(&mut self, a: &str, b: &str) -> String {
        if a.chars().count() <= WIDTH && b.chars().count() <= WIDTH {
            return format!("{a} → {b}");
        }
        let (items_a, items_b): (Vec<&str>, Vec<&str>) =
            (a.split(" or ").collect(), b.split(" or ").collect());
        if items_a.len() >= 3 && items_b.len() >= 3 {
            let dropped = items_a.iter().filter(|item| !items_b.contains(item));
            let added = items_b.iter().filter(|item| !items_a.contains(item));
            let note: Vec<String> = added
                .map(|item| format!("+ {item}"))
                .chain(dropped.map(|item| format!("- {item}")))
                .collect();
            if !note.is_empty() {
                let (shown_a, shown_b) = (self.one(a), self.one(b));
                return format!(
                    "{shown_a} → {shown_b} (the alternatives: {})",
                    note.join(", ")
                );
            }
        }
        let (a_chars, b_chars): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
        let prefix = a_chars
            .iter()
            .zip(&b_chars)
            .take_while(|(x, y)| x == y)
            .count();
        let suffix = a_chars[prefix..]
            .iter()
            .rev()
            .zip(b_chars[prefix..].iter().rev())
            .take_while(|(x, y)| x == y)
            .count();
        let window = |chars: &[char]| {
            const CONTEXT: usize = 24;
            let from = prefix.saturating_sub(CONTEXT);
            let mut to = (chars.len() - suffix + CONTEXT).min(chars.len());
            to = to.min(from + WIDTH);
            let text: String = chars[from..to].iter().collect();
            let start = if from > 0 { "…" } else { "" };
            let end = if to < chars.len() { "…" } else { "" };
            format!("{start}{text}{end}")
        };
        let (shown_a, shown_b) = (window(&a_chars), window(&b_chars));
        let (n_a, n_b) = (self.reference(a), self.reference(b));
        format!("{shown_a}[{n_a}] → {shown_b}[{n_b}]")
    }
}

fn count(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

/// The groups the two dumps' differences make, and what is needed to write them.
struct Findings {
    groups: BTreeMap<(String, Change), Group>,
    builds: usize,
    differing: usize,
    only_a: usize,
    only_b: usize,
}

impl Findings {
    fn add(&mut self, build: &str, path: String, change: Change, lines: usize) {
        let group = self.groups.entry((path, change)).or_default();
        if group.last_build != build {
            group.builds += 1;
            group.last_build = build.to_owned();
            if group.examples.len() < 3 {
                group.examples.push(build.to_owned());
            }
        }
        group.lines += lines;
    }
}

/// Compares the Build `name`'s lines in A and in B (each sorted by path).
fn compare_build(
    findings: &mut Findings,
    name: &str,
    a: &[(String, String)],
    b: &[(String, String)],
) -> bool {
    let (mut i, mut j) = (0, 0);
    let mut differs = false;
    // The slot, as `(its path prefix, the side it is only on)`, whose lines aren't listed one by one.
    let mut slot_only: Option<(String, char, String)> = None;
    // What a `-hash` line that changed would add to what the option lines say: held back, and given only for a slot
    // (or picker) none of whose other lines differ (a reorder alone).
    let mut explained: BTreeSet<String> = BTreeSet::new();
    let mut rehashed: Vec<String> = Vec::new();
    while i < a.len() || j < b.len() {
        let order = match (a.get(i), b.get(j)) {
            (Some(x), Some(y)) => x.0.cmp(&y.0),
            (Some(_), None) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Greater,
        };
        let (side, path, value) = match order {
            std::cmp::Ordering::Equal => {
                let (path, before) = &a[i];
                let after = &b[j].1;
                (i, j) = (i + 1, j + 1);
                if before != after {
                    differs = true;
                    if path.ends_with("-hash") {
                        rehashed.push(path.clone());
                    } else {
                        explained.insert(owner(path).to_owned());
                        let change = Change::Changed(before.clone(), after.clone());
                        findings.add(name, wildcard(path), change, 1);
                    }
                }
                continue;
            }
            std::cmp::Ordering::Less => {
                i += 1;
                ('A', &a[i - 1].0, &a[i - 1].1)
            }
            std::cmp::Ordering::Greater => {
                j += 1;
                ('B', &b[j - 1].0, &b[j - 1].1)
            }
        };
        differs = true;
        explained.insert(owner(path).to_owned());
        if let Some((prefix, on, kind)) = &slot_only
            && *on == side
            && path.starts_with(prefix.as_str())
        {
            let group = wildcard(&prefix[..prefix.len() - 1]);
            findings.add(name, group, Change::SlotOnly(side, kind.clone()), 1);
            continue;
        }
        if let Some(prefix) = path
            .strip_suffix(".kind")
            .filter(|_| path.starts_with("slot."))
        {
            // The head line is the group's first line; the kind names the group.
            slot_only = Some((format!("{prefix}."), side, value.clone()));
            findings.add(
                name,
                wildcard(prefix),
                Change::SlotOnly(side, value.clone()),
                1,
            );
            continue;
        }
        let change = if side == 'A' {
            Change::Removed(value.clone())
        } else {
            Change::Added(value.clone())
        };
        findings.add(name, wildcard(path), change, 1);
    }
    for path in rehashed {
        if !explained.contains(owner(&path)) {
            findings.add(name, wildcard(&path), Change::Rehashed, 1);
        }
    }
    differs
}

/// What a line belongs to: a slot or picker, the path without its last part (and without an option's key).
fn owner(path: &str) -> &str {
    if let Some((owner, _)) = path.split_once(".option/") {
        return owner;
    }
    path.rsplit_once('.').map_or(path, |(owner, _)| owner)
}

/// The differences between dump `a` and dump `b`, explained: see the module docs. `top` is how many groups to list
/// (0 for all).
pub fn explain(a: impl BufRead, b: impl BufRead, top: usize) -> Result<String, String> {
    let (mut a, mut b) = (Dump::new(a, "A")?, Dump::new(b, "B")?);
    let mut findings = Findings {
        groups: BTreeMap::new(),
        builds: 0,
        differing: 0,
        only_a: 0,
        only_b: 0,
    };
    loop {
        let name = match (a.next_build(), b.next_build()) {
            (None, None) => break,
            (Some(x), Some(y)) => x.min(y).to_owned(),
            (Some(x), None) | (None, Some(x)) => x.to_owned(),
        };
        let (lines_a, lines_b) = (a.take_build(&name)?, b.take_build(&name)?);
        findings.builds += 1;
        match (lines_a.is_empty(), lines_b.is_empty()) {
            (false, true) => {
                findings.only_a += 1;
                findings.add(
                    &name,
                    "Build".to_owned(),
                    Change::BuildOnly('A'),
                    lines_a.len(),
                );
            }
            (true, false) => {
                findings.only_b += 1;
                findings.add(
                    &name,
                    "Build".to_owned(),
                    Change::BuildOnly('B'),
                    lines_b.len(),
                );
            }
            _ => {
                if compare_build(&mut findings, &name, &lines_a, &lines_b) {
                    findings.differing += 1;
                }
            }
        }
    }
    Ok(write(findings, top))
}

fn write(findings: Findings, top: usize) -> String {
    let mut out = format!(
        "{} of {} Builds differ ({} in both dumps, {} only in A, {} only in B)\n",
        findings.differing + findings.only_a + findings.only_b,
        findings.builds,
        findings.differing,
        findings.only_a,
        findings.only_b,
    );
    if findings.groups.is_empty() {
        out.push_str("\nNo differences.\n");
        return out;
    }
    let mut shown = Shown { full: Vec::new() };
    let mut groups: Vec<(&(String, Change), &Group)> = findings.groups.iter().collect();
    // Largest first: by Builds, then lines, then the order of the keys.
    groups.sort_by(|x, y| {
        (y.1.builds, y.1.lines)
            .cmp(&(x.1.builds, x.1.lines))
            .then_with(|| x.0.cmp(y.0))
    });
    let listed = if top == 0 {
        groups.len()
    } else {
        top.min(groups.len())
    };
    let _ = writeln!(
        out,
        "\n{} of differences, levels, ranks and indexes wildcarded, largest first (a value is cut at {WIDTH} characters, `[n]` is its full text at the end):",
        count(groups.len(), "group")
    );
    for (key, group) in &groups[..listed] {
        let (path, change) = key;
        let text = match change {
            Change::Added(value) => format!("{path}: + {}", shown.one(value)),
            Change::Removed(value) => format!("{path}: - {}", shown.one(value)),
            Change::Rehashed => format!("{path}: changed"),
            Change::Changed(a, b) => format!("{path}: {}", shown.two(a, b)),
            Change::SlotOnly(side, kind) => {
                format!("{path}: slot only in {side} ({})", shown.one(kind))
            }
            Change::BuildOnly(side) => format!("{path}: only in {side}"),
        };
        let _ = writeln!(
            out,
            "  {text}  [{}, {}; {}{}]",
            count(group.builds, "Build"),
            count(group.lines, "line"),
            group.examples.join(", "),
            if group.builds > group.examples.len() {
                ", …"
            } else {
                ""
            },
        );
    }
    if listed < groups.len() {
        let _ = writeln!(
            out,
            "  … and {} more groups (`explain --top 0` lists all)",
            groups.len() - listed
        );
    }
    if !shown.full.is_empty() {
        out.push_str("\nValues in full:\n");
        for (at, value) in shown.full.iter().enumerate() {
            let _ = writeln!(out, "  [{}] {value}", at + 1);
        }
    }
    out
}
