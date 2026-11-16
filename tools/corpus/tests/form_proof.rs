//! The form proof (ticket 300): a ticket that changes only the data's or the model's form proves that nothing moved.
//!
//! It records, for every Build of [`corpus::builds::all`] (about 2,000), as text that can be diffed:
//!
//! - `builds.tsv`: the Build's name, its link, whether the link opens, its picks (`Debug`) and the picks its link
//!   decodes to;
//! - `slots.txt`: every slot its walk gives it (level or rank, id, kind, pick);
//! - `sheets.txt`: its `build_sheet`, the link line left out;
//! - `numbers.txt` (ticket 297): at each level, the scores the sheet, prerequisites and full prerequisites read, the
//!   base attack bonus and base saves, the totals with their parts, the caster line for a spell of each school and of
//!   the subschools and energies a bonus names, with and without metamagic, the attack line of the Build's default
//!   weapon and of the longsword, and each held feature's steps;
//! - `summary.txt`: the counts and a hash of each file.
//!
//! Two modes, both ignored (they take minutes in release, and only a ticket that claims no behaviour moves runs them):
//!
//! ```text
//! FORM_PROOF_OUT=<dir>    cargo test --release -p corpus --test form_proof record  -- --ignored --nocapture
//! FORM_PROOF_BEFORE=<dir> cargo test --release -p corpus --test form_proof compare -- --ignored --nocapture
//! ```
//!
//! - **`record`** writes the files to `FORM_PROOF_OUT`. Run it on the ticket's base, in a temporary worktree of the
//!   base commit (`git worktree add`), never in the tree under change.
//! - **`compare`** records again and compares with the files in `FORM_PROOF_BEFORE`, then opens every link the files
//!   hold with this tree's data (the old links against the new keys) and compares what they decode to, the slots
//!   and the sheet. It prints the first differences of each kind, naming the Build and the first line that
//!   differs, and fails on any. The counts it prints go in the ticket's Answer.
//!
//! `FORM_PROOF_THREADS` sets the threads (8 by default). The hashes are `DefaultHasher`'s, so they compare only
//! within one toolchain; `compare` never reads them.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use corpus::{Named, in_parallel};
use rules::{
    Build, CasterPart, Energy, Engine, LinkKeys, SpellSchool, SpellTraits, Subschool, Total,
    WeaponKey, decode_build, encode_build,
};

/// How many differences of each kind `compare` prints (`FORM_PROOF_SHOWN` sets another number).
const SHOWN: usize = 5;

/// What the proof records of one Build.
#[derive(Clone, Default)]
struct Entry {
    name: String,
    link: String,
    opens: bool,
    picks: String,
    /// The picks the link decodes to (`Debug`), or why it doesn't.
    decoded: String,
    slots: String,
    sheet: String,
    numbers: String,
}

fn env_dir(var: &str) -> PathBuf {
    PathBuf::from(std::env::var(var).unwrap_or_else(|_| panic!("{var} names a folder")))
}

/// Every slot of the walk, `L<level>|R<rank> \t id \t kind \t pick`, one per line.
fn slots(engine: &Engine, build: &Build) -> String {
    let sheet = engine.evaluate(build);
    let mut text = String::new();
    for level in &sheet.levels {
        for slot in &level.slots {
            let at = format!("L{}", level.character_level);
            writeln!(
                text,
                "{at}\t{:?}\t{:?}\t{:?}",
                slot.id, slot.kind, slot.pick
            )
            .unwrap();
        }
    }
    for rank in &sheet.ranks {
        for slot in &rank.slots {
            let at = format!("R{}", rank.rank);
            writeln!(
                text,
                "{at}\t{:?}\t{:?}\t{:?}",
                slot.id, slot.kind, slot.pick
            )
            .unwrap();
        }
    }
    text.trim_end().to_owned()
}

/// The sheet with its link line (`#b=…`, which changes with the data's keys) left out.
fn sheet(engine: &Engine, build: &Build) -> String {
    let sheet = engine.build_sheet(build, "");
    let kept: Vec<&str> = sheet
        .lines()
        .filter(|line| !line.starts_with("#b="))
        .collect();
    kept.join("\n")
}

/// The spells a caster line is recorded for: each school, then the subschools and energies a bonus names, and an
/// Enchantment with metamagic.
fn spells() -> Vec<SpellTraits> {
    let spell = |school, subschool, energy, metamagic| SpellTraits {
        school,
        subschool,
        energy,
        metamagic,
    };
    let mut spells: Vec<SpellTraits> = (SpellSchool::ALL.into_iter())
        .map(|school| spell(school, None, None, false))
        .collect();
    for subschool in [Subschool::Charm, Subschool::Compulsion] {
        spells.push(spell(
            SpellSchool::Enchantment,
            Some(subschool),
            None,
            false,
        ));
    }
    for energy in Energy::ALL {
        spells.push(spell(SpellSchool::Evocation, None, Some(energy), false));
    }
    spells.push(spell(
        SpellSchool::Conjuration,
        None,
        Some(Energy::Acid),
        false,
    ));
    spells.push(spell(SpellSchool::Enchantment, None, None, true));
    spells
}

fn caster_parts(parts: &[CasterPart]) -> String {
    let each: Vec<String> = (parts.iter())
        .map(|part| format!("{}={}", part.name, part.value))
        .collect();
    each.join(",")
}

fn total(total: &Total) -> String {
    let each: Vec<String> = (total.parts.iter())
        .map(|part| format!("{}={}", part.name, part.value))
        .collect();
    format!("{} [{}]", total.total, each.join(","))
}

/// The numbers of a Build at each level (`numbers.txt`).
fn numbers(engine: &Engine, build: &Build) -> String {
    let sheet = engine.evaluate(build);
    let spells = spells();
    let mut weapons: Vec<WeaponKey> = engine.default_attack_weapon(&sheet).into_iter().collect();
    weapons.push(WeaponKey::new("longsword"));
    let mut text = String::new();
    for level in &sheet.levels {
        let state = &level.state;
        writeln!(
            text,
            "L{} scores {:?} permanent {:?} full {:?} bab {} saves {}/{}/{} caster level {} mythic {}",
            level.character_level,
            state.ability_scores,
            state.permanent_ability_scores,
            state.full_ability_scores,
            state.bab,
            state.fort,
            state.reflex,
            state.will,
            state.caster_level,
            state.mythic_rank,
        )
        .unwrap();
        let totals = engine.totals(level);
        writeln!(
            text,
            "  initiative {} fortitude {} reflex {} will {}",
            total(&totals.initiative),
            total(&totals.fortitude),
            total(&totals.reflex),
            total(&totals.will)
        )
        .unwrap();
        for skill in &totals.skills {
            writeln!(
                text,
                "  {} {} {}",
                skill.skill,
                skill.ranks,
                total(&skill.total)
            )
            .unwrap();
        }
        for spell in &spells {
            let Some(line) = engine.caster_line(level, *spell) else {
                continue;
            };
            let dc = line.dcs.last().map(|dc| (dc.spell_level, dc.dc));
            writeln!(
                text,
                "  {:?} {} {:?} cl {} dc {:?} heightened {:?} [{}] sp {} [{}] concentration {} [{}]",
                spell,
                line.class,
                line.ability,
                line.caster_level,
                dc,
                line.heightened.map(|dc| dc.dc),
                caster_parts(&line.dc_parts),
                line.spell_penetration,
                caster_parts(&line.spell_penetration_parts),
                line.concentration,
                caster_parts(&line.concentration_parts),
            )
            .unwrap();
        }
        for weapon in &weapons {
            if let Some(line) = engine.attack_line(state, weapon) {
                let parts: Vec<String> = (line.parts.iter())
                    .map(|part| format!("{}={}/{}", part.name, part.attack, part.damage))
                    .collect();
                writeln!(
                    text,
                    "  {weapon} {} {} {} {:?} [{}]",
                    line.attacks_text(),
                    line.damage_text(),
                    line.critical_text(),
                    line.power_attack,
                    parts.join(",")
                )
                .unwrap();
            }
        }
        let steps: Vec<String> = (state.features.keys())
            .filter_map(|feature| {
                let step = engine.steps_reached(state, feature);
                (step > 0).then(|| format!("{feature}={step}"))
            })
            .collect();
        if !steps.is_empty() {
            writeln!(text, "  steps {}", steps.join(",")).unwrap();
        }
    }
    text.trim_end().to_owned()
}

fn record_entry(engine: &Engine, keys: &LinkKeys, (name, build): &Named) -> Entry {
    let link = encode_build(build);
    let decoded = match decode_build(&link, keys) {
        Ok(build) => format!("{:?}", build.picks),
        Err(error) => format!("does not decode: {error:?}"),
    };
    Entry {
        name: name.to_string(),
        opens: engine.open_link(&link).is_ok(),
        picks: format!("{:?}", build.picks),
        decoded,
        slots: slots(engine, build),
        sheet: sheet(engine, build),
        numbers: numbers(engine, build),
        link,
    }
}

fn record_all() -> Vec<Entry> {
    let (engine, data) = corpus::engine().expect("the game data loads");
    let threads = corpus::threads("FORM_PROOF_THREADS");
    let builds = corpus::builds::all(&engine, &data, threads);
    let keys = LinkKeys::new(&data);
    in_parallel(&builds, threads, |build| {
        record_entry(&engine, &keys, build)
    })
}

/// The four files' texts, `summary.txt` last.
fn texts(entries: &[Entry]) -> [(&'static str, String); 5] {
    let (mut builds, mut slots, mut sheets) = (String::new(), String::new(), String::new());
    let mut numbers = String::new();
    for e in entries {
        let Entry {
            name,
            link,
            opens,
            picks,
            decoded,
            ..
        } = e;
        writeln!(builds, "{name}\t{link}\t{opens}\t{picks}\t{decoded}").unwrap();
        writeln!(slots, "===== {name}\n{}", e.slots).unwrap();
        writeln!(sheets, "===== {name}\n{}", e.sheet).unwrap();
        writeln!(numbers, "===== {name}\n{}", e.numbers).unwrap();
    }
    let hash = |text: &str| {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    };
    let summary = format!(
        "builds {}\nopen {}\nbuilds.tsv {:016x}\nslots.txt {:016x}\nsheets.txt {:016x}\nnumbers.txt {:016x}\n",
        entries.len(),
        entries.iter().filter(|e| e.opens).count(),
        hash(&builds),
        hash(&slots),
        hash(&sheets),
        hash(&numbers),
    );
    [
        ("builds.tsv", builds),
        ("slots.txt", slots),
        ("sheets.txt", sheets),
        ("numbers.txt", numbers),
        ("summary.txt", summary),
    ]
}

fn write_entries(out: &Path, entries: &[Entry]) -> String {
    std::fs::create_dir_all(out).unwrap();
    let mut summary = String::new();
    for (file, text) in texts(entries) {
        std::fs::write(out.join(file), &text).unwrap();
        if file == "summary.txt" {
            summary = text;
        }
    }
    summary
}

/// The sections of a `===== name` file, in order.
fn sections(text: &str) -> Vec<(&str, String)> {
    let mut found: Vec<(&str, Vec<&str>)> = Vec::new();
    for line in text.lines() {
        match line.strip_prefix("===== ") {
            Some(name) => found.push((name, Vec::new())),
            None => found.last_mut().expect("a file of sections").1.push(line),
        }
    }
    // A section is its lines, with the blank line that ends it dropped; a sheet has none of its own at the end.
    found
        .into_iter()
        .map(|(name, mut lines)| {
            while lines.last() == Some(&"") {
                lines.pop();
            }
            (name, lines.join("\n"))
        })
        .collect()
}

fn read_entries(dir: &Path) -> Vec<Entry> {
    let read = |file: &str| {
        std::fs::read_to_string(dir.join(file))
            .unwrap_or_else(|error| panic!("{}: {error}", dir.join(file).display()))
    };
    let (slot_text, sheet_text) = (read("slots.txt"), read("sheets.txt"));
    let (slot_sections, sheet_sections) = (sections(&slot_text), sections(&sheet_text));
    let number_text = read("numbers.txt");
    let number_sections = sections(&number_text);
    let builds = read("builds.tsv");
    let rows: Vec<&str> = builds.lines().collect();
    assert!(
        rows.len() == slot_sections.len()
            && rows.len() == sheet_sections.len()
            && rows.len() == number_sections.len(),
        "the recorded files hold {}, {}, {} and {} Builds",
        rows.len(),
        slot_sections.len(),
        sheet_sections.len(),
        number_sections.len()
    );
    rows.iter()
        .zip(slot_sections)
        .zip(sheet_sections)
        .zip(number_sections)
        .map(|(((row, (name, slots)), (_, sheet)), (_, numbers))| {
            let mut parts = row.splitn(5, '\t');
            let mut next = || parts.next().expect("five columns").to_owned();
            let (_, link, opens, picks, decoded) = (next(), next(), next(), next(), next());
            Entry {
                name: name.to_owned(),
                link,
                opens: opens == "true",
                picks,
                decoded,
                slots,
                sheet,
                numbers,
            }
        })
        .collect()
}

/// The differences by kind: how many, and the first few.
#[derive(Default)]
struct Differences(BTreeMap<&'static str, (usize, Vec<String>)>);

impl Differences {
    fn add(&mut self, kind: &'static str, build: &str, before: &str, now: &str) {
        if before == now {
            return;
        }
        let (count, shown) = self.0.entry(kind).or_default();
        *count += 1;
        let most = std::env::var("FORM_PROOF_SHOWN")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(SHOWN);
        if shown.len() < most {
            shown.push(format!("{build}: {}", first_difference(before, now)));
        }
    }

    fn total(&self) -> usize {
        self.0.values().map(|(count, _)| count).sum()
    }

    fn report(&self) -> String {
        let mut text = String::new();
        for (kind, (count, shown)) in &self.0 {
            writeln!(
                text,
                "{kind}: {count} Builds differ, the first {}:",
                shown.len()
            )
            .unwrap();
            for line in shown {
                writeln!(text, "  {line}").unwrap();
            }
        }
        text
    }
}

/// The first line where two texts differ (or the one that ends first), both sides, cut short.
fn first_difference(before: &str, now: &str) -> String {
    let cut = |text: &str| text.chars().take(240).collect::<String>();
    let (mut a, mut b) = (before.lines(), now.lines());
    for number in 1.. {
        match (a.next(), b.next()) {
            (None, None) => break,
            (x, y) if x == y => {}
            (x, y) => {
                return format!(
                    "line {number}\n    before: {}\n    now:    {}",
                    x.map_or("(ends)".to_owned(), cut),
                    y.map_or("(ends)".to_owned(), cut)
                );
            }
        }
    }
    "(equal)".to_owned()
}

#[test]
#[ignore = "records about 2,000 real-data Builds: a minute in release"]
fn record() {
    let entries = record_all();
    let summary = write_entries(&env_dir("FORM_PROOF_OUT"), &entries);
    println!("{summary}");
}

#[test]
#[ignore = "records about 2,000 real-data Builds and compares: a minute in release"]
fn compare() {
    let before = read_entries(&env_dir("FORM_PROOF_BEFORE"));
    let now = record_all();
    let mut differences = Differences::default();
    if before.len() != now.len() {
        differences.add(
            "builds",
            "the corpus",
            &format!("{} Builds", before.len()),
            &format!("{} Builds", now.len()),
        );
    }
    for (b, n) in before.iter().zip(&now) {
        let build = n.name.as_str();
        differences.add("name", &b.name, &b.name, build);
        differences.add("link", build, &b.link, &n.link);
        differences.add("opens", build, &b.opens.to_string(), &n.opens.to_string());
        differences.add("picks", build, &b.picks, &n.picks);
        differences.add("decoded picks", build, &b.decoded, &n.decoded);
        differences.add("slots", build, &b.slots, &n.slots);
        differences.add("sheet", build, &b.sheet, &n.sheet);
        differences.add("numbers", build, &b.numbers, &n.numbers);
    }

    // The old links against this tree's keys: what they decode to must be what they decoded to before, and walk
    // and print the same.
    let (engine, data) = corpus::engine().expect("the game data loads");
    let keys = LinkKeys::new(&data);
    let threads = corpus::threads("FORM_PROOF_THREADS");
    let builds: Vec<Named> = before
        .iter()
        .filter_map(|e| {
            decode_build(&e.link, &keys)
                .ok()
                .map(|b| (e.name.clone(), b))
        })
        .collect();
    let reopened: BTreeMap<&str, Entry> = in_parallel(&builds, threads, |build| {
        record_entry(&engine, &keys, build)
    })
    .into_iter()
    .zip(&builds)
    .map(|(entry, (name, _))| (name.as_ref(), entry))
    .collect();
    let mut not_reopened = 0;
    for b in &before {
        match reopened.get(b.name.as_str()) {
            Some(r) => {
                differences.add("reopened picks", &b.name, &b.picks, &r.picks);
                differences.add("reopened slots", &b.name, &b.slots, &r.slots);
                differences.add("reopened sheet", &b.name, &b.sheet, &r.sheet);
            }
            None => {
                not_reopened += 1;
                differences.add("reopened link", &b.name, "opens", "does not decode");
            }
        }
    }

    println!(
        "{} Builds compared ({} open), {} old links reopened, {} differences",
        before.len(),
        before.iter().filter(|e| e.opens).count(),
        before.len() - not_reopened,
        differences.total()
    );
    assert_eq!(
        differences.total(),
        0,
        "the form moved something:\n{}",
        differences.report()
    );
}
