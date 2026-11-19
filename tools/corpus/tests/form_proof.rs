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
//! - `offers.bin` and `offers-kinds.txt` (ticket 311; only with `FORM_PROOF_OFFERS=1`): every option the slot pickers offer at every slot of the walk, as
//!   9 bytes an option (a hash of its slot and pick, and the index of its status's kind: `ok`, `Unmet`, `Taken`, …),
//!   about 160 MB for the corpus, written and read a Build at a time;
//! - `reasons.bin` and `reasons-texts.txt` (ticket 331; only with `FORM_PROOF_REASONS=1`, which records the offers
//!   too): the text of every refused option's reason and of every failed choice, 12 bytes an entry and a table of
//!   the distinct texts, about 180 MB for the corpus;
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
//! - **The offers** are for a ticket that changes what a picker offers or how often an entry repeats (as 303 did):
//!   `record` writes them only with `FORM_PROOF_OFFERS=1` (it takes about 8 times as long, 47 s against 6 s), and
//!   `compare` checks them only when the recorded folder has them. They compare as the record: for each option, its status kind before and now. `compare` prints each
//!   change as `count pick key: kind before -> kind now (n Builds)` (the first 40, `FORM_PROOF_OFFERS_SHOWN` for
//!   another number), with the options that appeared or disappeared, and fails on any. It judges the Builds the old
//!   links open to. A record without `offers.bin` (an older one) compares everything else and says offers were not
//!   recorded. The options' names are those of the tree now: an option that disappeared is counted, not named.
//!   `compare` also counts the Builds a status changed in, and `FORM_PROOF_OFFER_BUILDS_OUT=<file>` writes their names.
//! - **The reasons** (ticket 331) are for a ticket that changes reason texts (a wording ticket) or must keep them (a
//!   form-only ticket that touches what a text reads): `record` writes them only with `FORM_PROOF_REASONS=1`, which
//!   records the offers too (one shared pass), as `reasons.bin` and `reasons-texts.txt`: for each refused option
//!   `reason_text`'s text (the engine's one-line text, `BreaksLater` where it has none), and for each choice of
//!   `Engine::failures` its text. A Build at a time, 12 bytes an entry (a hash of the slot and pick, or of the
//!   choice, and an index into the table of distinct texts): 183 MB and 4,258 texts (293 KB) for the corpus's 15.2
//!   million refused options and 220 failures. `compare` reads them whenever the record has them, and compares the
//!   options refused on both sides (a kind change is the offers') and the failures by choice. `FORM_PROOF_REASONS=1`
//!   on `compare` fails on a record without `reasons.bin`, with or without the offers; one without says reasons were
//!   not recorded. It groups each changed text by its **edit**, both texts with their common prefix and suffix
//!   removed (`Needs 1 more of: A` to `Needs A` is `1 more of: ` to nothing), and prints `reasons: N compared, N
//!   changed in N edits` (the compared count holds the options and the failures present on both sides), then each
//!   edit, most frequent first (`FORM_PROOF_REASONS_SHOWN` sets how many, 40 by default): its count, its Builds, how
//!   many distinct full texts it covered, and one full example. The edit's sides print with `{:?}`, so spaces and
//!   tabs show. It fails on each edit unless `FORM_PROOF_REASONS_ACCEPT=<file>` lists it, on a listed edit no change
//!   made, and on a failure that appeared or went.
//!   - **The accept file** has one edit a line, `before<TAB>now` (the prefix- and suffix-free parts); `\n`, `\r`, `\t`,
//!     `\\` and a leading `\#` escape a newline, a carriage return, a tab, a backslash and a `#`; `#` comments and
//!     blank lines are skipped, a side's spaces kept. `FORM_PROOF_REASONS_ACCEPT_OUT=<file>` writes the edits no
//!     accept line covers in that format, ready to paste into the accept file after the ticket's author has read
//!     them.
//!   - **With `FORM_PROOF_CENSUS`**, the reasons split as the statuses do, one unit throughout (an option or a
//!     failure): a changed option text on a pick whose key the log names counts under `reasons census:` and is not an
//!     edit; so does any changed, appeared or gone failure in a Build that holds a logged entry (the census's own
//!     Build-level rule, `held_with_picks`: a choice's `Debug` names no pick key, so failures have no finer rule),
//!     and any option or failure in a Build whose old link is refused (those links that opened with the base's
//!     data: a Build that already failed on the base is not among them). Only the rest fails.
//!   - **Measured** (the same tree, `record` with `FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`): 45 s for the test and
//!     48 s for `compare` at a load of 4 to 13. The offers alone took 49 s at a load over 23, so what the reasons add
//!     to the offers' time is unmeasured (the design estimated 50 %).
//!
//!   `reason_text` is the one function that makes the text, so a ticket that words refusals another way (336) changes
//!   it and `compare` lists what it moved.
//! - **The link census** (ticket 306c): `compare` opens each old link that opened with this tree's data and prints
//!   how many opened with the same picks, through an heir (a pick carried to the entry whose `aliases` name its key)
//!   or refused (`links: N the same, N through an heir, N refused, N changed`); one that opens as another Build is a
//!   difference. `FORM_PROOF_LINKS_OUT=<file>` writes `name<TAB>slot<TAB>key<TAB>kind` of each heir (`old=>new`)
//!   and each refused pick. `tests/links.rs` pins a stage's base links and refusals.
//! - **Lost Builds** (ticket 306af): `compare` matches the record's Builds with the corpus's by name, and lists each
//!   Build the record holds that the corpus doesn't (`lost Build`), each new one, and each that opened on the base and
//!   not on the tip, with the generator lines that can make it: a string literal of `tools/corpus/src/builds/` or
//!   `generate.rs` whose fixed text the name has (the best fits first), or its line of `bench-links.tsv`. A name
//!   built from data may match none. The stage's Answer explains each or fixes the generator.
//! - **`FORM_PROOF_KIND_MAP=<file>`** is for a ticket that renames a slot kind: a file of `old kind<TAB>new kind` lines
//!   (the `Debug` text of the kind, as `slots.txt` holds it; `#` comments). `compare` accepts a slot line that
//!   differs only by its kind through the map, and prints how many it mapped apart from the differences.
//!
//! - **`FORM_PROOF_CENSUS=<change log>`** (ticket 306ae) is for a stage that changes values, the TSV `game-check --write
//!   --log` writes. `compare` then attributes each differing line of `builds.tsv`, `slots.txt`, `sheets.txt` and
//!   `numbers.txt` (and the reopened Builds' slots and sheets) instead of failing on it. A line is attributed when the
//!   Build holds an entry the log has a row for, and that row's field is one that moves the line's kind (`FIELDS`: a save
//!   field the header and the save totals, `class_skills` the skill lines, `bab` the header and the attack lines,
//!   `prerequisite` (ticket 308b) every kind: the entry may be a class, an archetype or a pick key of the base or the
//!   tip, and a change-log entry of two words, `selection option a/b`, reads as its last word…). A
//!   field that gates a pick or sizes the generator's draws (`bab`, `skill_points`) also accounts for every line of a
//!   Build whose link or slots changed. `compare` prints `census <field>: N lines in N Builds (per file)` and each
//!   unattributed line (the first `FORM_PROOF_SHOWN`), and fails on any. A log field `FIELDS` lacks is refused: the
//!   stage adds its row, with what the field moves. It replaces a per-ticket classifier.
//!
//!   With the offers recorded too, `compare` sizes the change (ticket 322): for each log entry, the option statuses
//!   that went from an open kind to `Unmet` or `BreaksLater` in Builds whose old link isn't refused (`census narrowing
//!   <entry>: N statuses … in N Builds (<before> -> <now> N, …)`, the most first, then how many entries narrowed none);
//!   and every status change split into those on options the log names (the pick's key is a log entry's), those in
//!   Builds whose links are refused (the link census's), and the rest, each named (`ELSE <Build>: <key>: <before> ->
//!   <now>`, the first `FORM_PROOF_SHOWN`). A census line attributes every line of a Build holding the entry, so it
//!   can't tell a move toward the game from one away: this says how many picks the change refuses, and where.
//! - **Unset, `record` and `compare` print a skip line and pass**, so `cargo test --release -p corpus -- --ignored`
//!   passes.
//!
//! `FORM_PROOF_THREADS` sets the threads (8 by default). The hashes are `DefaultHasher`'s, so they compare only
//! within one toolchain; `compare` never reads them.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use corpus::links::{Fate, fate};
use corpus::{Named, in_parallel};
use rules::{
    Build, CasterPart, Energy, Engine, GameData, Issue, LinkKeys, SpellSchool, SpellTraits,
    Subschool, Total, WeaponKey, decode_build, encode_build,
};

/// How many differences of each kind `compare` prints (`FORM_PROOF_SHOWN` sets another number).
const SHOWN: usize = 5;

/// What the proof records of one Build.
#[derive(Clone, Default)]
struct Entry {
    name: String,
    /// The keys of the classes and archetypes the Build holds (not recorded: read from this tree's Builds, for the
    /// census).
    held: BTreeSet<String>,
    link: String,
    opens: bool,
    picks: String,
    /// The picks the link decodes to (`Debug`), or why it doesn't.
    decoded: String,
    slots: String,
    sheet: String,
    numbers: String,
}

/// The folder `var` names, or the line to print when it's unset: the mode was not asked for, so its test passes.
fn dir_or_skip(var: &str, value: Option<std::ffi::OsString>) -> Result<PathBuf, String> {
    value.map(PathBuf::from).ok_or_else(|| {
        format!("{var} is not set: skipped (the form proof runs only when asked for)")
    })
}

/// [`dir_or_skip`] on the environment, printing the skip line.
fn env_dir(var: &str) -> Option<PathBuf> {
    match dir_or_skip(var, std::env::var_os(var)) {
        Ok(dir) => Some(dir),
        Err(skipped) => {
            println!("{skipped}");
            None
        }
    }
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
        let steps: Vec<String> = (state.features().map(|(feature, _)| feature))
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
    let classes = build.levels.iter().map(|level| level.class.as_str());
    let archetypes = (build.class_choices.values()).filter_map(|choice| {
        choice
            .archetype
            .as_ref()
            .map(|archetype| archetype.as_str())
    });
    Entry {
        name: name.to_string(),
        held: classes.chain(archetypes).map(str::to_owned).collect(),
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
    no_duplicate_names(builds.iter().map(|(name, _)| name.as_str()), "the corpus");
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
    let entries: Vec<Entry> = rows
        .iter()
        .zip(slot_sections)
        .zip(sheet_sections)
        .zip(number_sections)
        .map(|(((row, (name, slots)), (_, sheet)), (_, numbers))| {
            let mut parts = row.splitn(5, '\t');
            let mut next = || parts.next().expect("five columns").to_owned();
            let (_, link, opens, picks, decoded) = (next(), next(), next(), next(), next());
            Entry {
                held: BTreeSet::new(),
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
        .collect();
    no_duplicate_names(entries.iter().map(|e| e.name.as_str()), "the record");
    entries
}

/// Panics naming each Build name `names` holds twice: the proof matches Builds by name, so a shared name would hide
/// one of them. `whose` says which list it is.
fn no_duplicate_names<'a>(names: impl Iterator<Item = &'a str>, whose: &str) {
    let mut seen = BTreeSet::new();
    let duplicates: BTreeSet<&str> = names.filter(|name| !seen.insert(*name)).collect();
    assert!(
        duplicates.is_empty(),
        "{whose} holds Builds that share a name (the proof matches by name): {duplicates:?}"
    );
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

    /// A difference that is its own text (a Build the corpus lost, with its generator lines), counted and shown.
    fn note(&mut self, kind: &'static str, text: String) {
        let (count, shown) = self.0.entry(kind).or_default();
        *count += 1;
        let most = std::env::var("FORM_PROOF_SHOWN")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(SHOWN);
        if shown.len() < most {
            shown.push(text);
        }
    }

    /// A slots text compared with the kind map applied to the recorded side: the lines it maps and finds equal are
    /// counted in `mapped`, apart from the differences.
    fn add_slots(
        &mut self,
        kind: &'static str,
        build: &str,
        (before, now): (&str, &str),
        map: &BTreeMap<String, String>,
        mapped: &mut usize,
    ) {
        let (rewritten, lines) = map_slot_kinds(before, map);
        if rewritten == now {
            *mapped += lines;
        }
        self.add(kind, build, &rewritten, now);
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

// ---- The offers record (ticket 311) ---------------------------------------------------------------------------

/// The offers record's files: every option's status kind at every slot of every Build, in binary (9 bytes an option),
/// and the table of status kinds it indexes.
const OFFERS_FILE: &str = "offers.bin";
const KINDS_FILE: &str = "offers-kinds.txt";

/// How many offer changes `compare` prints (`FORM_PROOF_OFFERS_SHOWN` sets another number).
const OFFERS_SHOWN: usize = 40;

/// One option offered at a slot: the slot and the pick as `Debug` text, and the status's kind (`ok`, or the text of
/// its `Issue` up to the first space, `(` or `{`).
struct Offer {
    slot: String,
    pick: String,
    kind: String,
    /// A refused option's reason (`reason_text`), only with `FORM_PROOF_REASONS=1`.
    reason: Option<String>,
}

/// FNV-1a over the parts, a separator byte between them: the same in every run and toolchain.
fn fnv(parts: &[&str]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in parts {
        for byte in part.bytes().chain(std::iter::once(0xff)) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}

/// A status's kind: the text before its first space, `(` or `{` (`Unmet(Condition …)` is `Unmet`).
fn status_kind(status: &str) -> &str {
    status
        .find([' ', '(', '{'])
        .map_or(status, |i| &status[..i])
}

/// The key a pick names: the first `feat: …Key("x")`, `ability: …Key("x")` or `option: …Key("x")`, so a feat's
/// parameter (a weapon's key) is not taken for it.
fn pick_key(pick: &str) -> Option<&str> {
    const OPENER: &str = "Key(\"";
    let mut from = 0;
    while let Some(found) = pick[from..].find(OPENER) {
        let at = from + found;
        let word_start = pick[..at]
            .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
            .map_or(0, |i| i + 1);
        let named = word_start < at
            && ["feat: ", "ability: ", "option: "]
                .iter()
                .any(|p| pick[..word_start].ends_with(p));
        let value_start = at + OPENER.len();
        if named && let Some(len) = pick[value_start..].find('"') {
            return Some(&pick[value_start..value_start + len]);
        }
        from = value_start;
    }
    None
}

/// Every key the picks' `Debug` text names (`pick_key`'s scan, repeated past each key found).
fn pick_keys(picks: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut rest = picks;
    while let Some(key) = pick_key(rest) {
        keys.insert(key.to_owned());
        let end = key.as_ptr() as usize - rest.as_ptr() as usize + key.len();
        rest = &rest[end..];
    }
    keys
}

/// The entries a Build holds for the census (ticket 308b): its classes and archetypes, and the pick keys of the base
/// and the tip, so a prerequisite change attributes the Build that held the entry on either side.
fn held_with_picks(held: &BTreeSet<String>, picks: &[&str]) -> BTreeSet<String> {
    let mut all = held.clone();
    for picks in picks {
        all.extend(pick_keys(picks));
    }
    all
}

/// Every option of every slot of a Build's walk, with its status's kind.
fn offers(engine: &Engine, build: &Build, reasons: bool) -> Vec<Offer> {
    let sheet = engine.evaluate(build);
    let slots = (sheet.levels.iter())
        .flat_map(|level| &level.slots)
        .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots));
    let mut found = Vec::new();
    for slot in slots {
        let Some(options) = engine.options(build, &slot.id) else {
            continue;
        };
        let slot_text = format!("{:?}", slot.id);
        for option in options {
            let (kind, reason) = match &option.status {
                Ok(()) => ("ok".to_owned(), None),
                Err(issue) => (
                    status_kind(&format!("{issue:?}")).to_owned(),
                    reasons.then(|| reason_text(engine, issue)),
                ),
            };
            found.push(Offer {
                slot: slot_text.clone(),
                pick: format!("{:?}", option.pick),
                kind,
                reason,
            });
        }
    }
    found
}

/// Writes the record one Build at a time: `u32 name length, name, u32 options`, then per option `u64 hash of (slot,
/// pick)` and `u8` index of its status kind in the kinds table. 9 bytes an option (about 160 MB for the corpus's
/// 18 million), never held whole.
struct OffersWriter<W: Write> {
    out: W,
    kinds: Vec<String>,
    options: u64,
}

impl<W: Write> OffersWriter<W> {
    fn new(out: W) -> Self {
        Self {
            out,
            kinds: Vec::new(),
            options: 0,
        }
    }

    fn build(&mut self, name: &str, offers: &[Offer]) -> io::Result<()> {
        self.out.write_all(&(name.len() as u32).to_le_bytes())?;
        self.out.write_all(name.as_bytes())?;
        self.out.write_all(&(offers.len() as u32).to_le_bytes())?;
        for offer in offers {
            let index = match self.kinds.iter().position(|k| *k == offer.kind) {
                Some(index) => index,
                None => {
                    self.kinds.push(offer.kind.clone());
                    self.kinds.len() - 1
                }
            };
            let index = u8::try_from(index).expect("under 256 status kinds");
            self.out
                .write_all(&fnv(&[&offer.slot, &offer.pick]).to_le_bytes())?;
            self.out.write_all(&[index])?;
        }
        self.options += offers.len() as u64;
        Ok(())
    }

    /// The kinds table, one per line, in the order the indices name them.
    fn finish(mut self) -> io::Result<String> {
        self.out.flush()?;
        Ok(self.kinds.join("\n") + "\n")
    }
}

/// Reads the record back a Build at a time: its name and `hash → kind index`.
struct OffersReader<R: Read> {
    input: R,
}

impl<R: Read> OffersReader<R> {
    fn next_build(&mut self) -> io::Result<Option<(String, HashMap<u64, u8>)>> {
        let mut head = [0u8; 4];
        if self.input.read(&mut head[..1])? == 0 {
            return Ok(None);
        }
        self.input.read_exact(&mut head[1..])?;
        let mut name = vec![0u8; u32::from_le_bytes(head) as usize];
        self.input.read_exact(&mut name)?;
        self.input.read_exact(&mut head)?;
        let mut body = vec![0u8; u32::from_le_bytes(head) as usize * 9];
        self.input.read_exact(&mut body)?;
        let (chunks, _) = body.as_chunks::<9>();
        let options = (chunks.iter())
            .map(|option| {
                let hash = u64::from_le_bytes(option[..8].try_into().expect("8 bytes"));
                (hash, option[8])
            })
            .collect();
        let name = String::from_utf8(name).map_err(io::Error::other)?;
        Ok(Some((name, options)))
    }
}

const REASONS_FILE: &str = "reasons.bin";
const TEXTS_FILE: &str = "reasons-texts.txt";

/// How many edits `compare` prints (`FORM_PROOF_REASONS_SHOWN` sets another number).
const REASONS_SHOWN: usize = 40;

/// The text the reasons record holds for a refusal: the one function that makes it, so a ticket that words the
/// refusals another way (336) changes this and `compare` lists what moved. Today it is the engine's one-line text, and
/// `BreaksLater` (which has none: the sheet leaves it grey) is named.
fn reason_text(engine: &Engine, issue: &Issue) -> String {
    engine
        .issue_text(issue)
        .unwrap_or_else(|| "BreaksLater".to_owned())
}

/// What one Build's walk gives the record: every option of every slot, and (with the reasons) each failed choice
/// with its text: `Debug` of the choice, and the reason.
struct Found {
    offers: Vec<Offer>,
    failures: Vec<(String, String)>,
}

/// Text on one line: a backslash, a newline, a carriage return and a tab escaped, and a leading `#` (which would
/// make an accept line a comment).
fn escape(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    match escaped.strip_prefix('#') {
        Some(rest) => format!("\\#{rest}"),
        None => escaped,
    }
}

/// [`escape`]'s inverse.
fn unescape(text: &str) -> String {
    let mut plain = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            plain.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => plain.push('\n'),
            Some('t') => plain.push('\t'),
            Some('r') => plain.push('\r'),
            Some('#') => plain.push('#'),
            Some('\\') | None => plain.push('\\'),
            Some(other) => {
                plain.push('\\');
                plain.push(other);
            }
        }
    }
    plain
}

/// The texts table: each text once, on a line of its own.
fn read_texts(text: &str) -> Vec<String> {
    text.lines().map(unescape).collect()
}

/// What a changed text is: both texts with their common prefix and suffix removed (`Needs 1 more of: A` and `Needs A`
/// are the edit `1 more of: ` to nothing), cut on character boundaries.
fn edit_of<'a>(before: &'a str, now: &'a str) -> (&'a str, &'a str) {
    let mut prefix = 0;
    for ((i, a), b) in before.char_indices().zip(now.chars()) {
        if a != b {
            break;
        }
        prefix = i + a.len_utf8();
    }
    let (before, now) = (&before[prefix..], &now[prefix..]);
    let mut suffix = 0;
    for (a, b) in before.chars().rev().zip(now.chars().rev()) {
        if a != b {
            break;
        }
        suffix += a.len_utf8();
    }
    (&before[..before.len() - suffix], &now[..now.len() - suffix])
}

/// An edit as the line that accepts it in `FORM_PROOF_REASONS_ACCEPT`: `before<TAB>now`, each escaped.
fn edit_line(before: &str, now: &str) -> String {
    format!("{}\t{}", escape(before), escape(now))
}

/// `FORM_PROOF_REASONS_ACCEPT`'s file: one edit a line, `before<TAB>now` (`\n`, `\r`, `\t`, `\\` and a leading `\#`
/// escape a newline, a carriage return, a tab, a backslash and a `#`), the texts with their common prefix and suffix removed, either side empty. Blank lines and lines
/// starting with `#` are skipped; a side's spaces are kept.
fn read_accept(text: &str) -> BTreeSet<(String, String)> {
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (before, now) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("an accepted edit is `before<TAB>now`: {line:?}"));
            (unescape(before), unescape(now))
        })
        .collect()
}

/// Writes the reasons record one Build at a time: `u32 name length, name, u32 refused options`, per option `u64 hash of
/// (slot, pick)` (as the offers' record) and `u32` index in the texts table, then `u32 failures` and per failure `u64
/// hash of the choice` and `u32` index. 12 bytes an entry; the writer keeps only the distinct texts.
struct ReasonsWriter<W: Write> {
    out: W,
    index: HashMap<String, u32>,
    texts: Vec<String>,
    options: u64,
    failures: u64,
}

impl<W: Write> ReasonsWriter<W> {
    fn new(out: W) -> Self {
        Self {
            out,
            index: HashMap::new(),
            texts: Vec::new(),
            options: 0,
            failures: 0,
        }
    }

    fn intern(&mut self, text: &str) -> u32 {
        if let Some(&index) = self.index.get(text) {
            return index;
        }
        let index = u32::try_from(self.texts.len()).expect("under 2^32 distinct texts");
        self.index.insert(text.to_owned(), index);
        self.texts.push(text.to_owned());
        index
    }

    fn entry(&mut self, hash: u64, text: &str) -> io::Result<()> {
        let index = self.intern(text);
        self.out.write_all(&hash.to_le_bytes())?;
        self.out.write_all(&index.to_le_bytes())
    }

    fn build(&mut self, name: &str, found: &Found) -> io::Result<()> {
        self.out.write_all(&(name.len() as u32).to_le_bytes())?;
        self.out.write_all(name.as_bytes())?;
        let refused: Vec<(&Offer, &String)> = (found.offers.iter())
            .filter_map(|offer| offer.reason.as_ref().map(|reason| (offer, reason)))
            .collect();
        self.out.write_all(&(refused.len() as u32).to_le_bytes())?;
        for (offer, reason) in refused {
            self.entry(fnv(&[&offer.slot, &offer.pick]), reason)?;
        }
        self.out
            .write_all(&(found.failures.len() as u32).to_le_bytes())?;
        for (choice, reason) in &found.failures {
            self.entry(fnv(&[choice]), reason)?;
        }
        self.options += found.offers.iter().filter(|o| o.reason.is_some()).count() as u64;
        self.failures += found.failures.len() as u64;
        Ok(())
    }

    /// The texts table, one escaped text a line, in the order the indices name them.
    fn finish(mut self) -> io::Result<String> {
        self.out.flush()?;
        Ok(self.texts.iter().map(|text| escape(text) + "\n").collect())
    }
}

/// One Build as the record holds it: `hash -> text index` of its refused options and of its failed choices.
#[derive(Default)]
struct Recorded {
    options: HashMap<u64, u32>,
    failures: HashMap<u64, u32>,
}

/// Reads the reasons record back a Build at a time.
struct ReasonsReader<R: Read> {
    input: R,
}

impl<R: Read> ReasonsReader<R> {
    fn count(&mut self) -> io::Result<usize> {
        let mut head = [0u8; 4];
        self.input.read_exact(&mut head)?;
        Ok(u32::from_le_bytes(head) as usize)
    }

    fn entries(&mut self) -> io::Result<HashMap<u64, u32>> {
        let mut body = vec![0u8; self.count()? * 12];
        self.input.read_exact(&mut body)?;
        let (chunks, _) = body.as_chunks::<12>();
        Ok((chunks.iter())
            .map(|entry| {
                let hash = u64::from_le_bytes(entry[..8].try_into().expect("8 bytes"));
                (
                    hash,
                    u32::from_le_bytes(entry[8..].try_into().expect("4 bytes")),
                )
            })
            .collect())
    }

    fn next_build(&mut self) -> io::Result<Option<(String, Recorded)>> {
        let mut first = [0u8; 1];
        if self.input.read(&mut first)? == 0 {
            return Ok(None);
        }
        let mut rest = [0u8; 3];
        self.input.read_exact(&mut rest)?;
        let length = u32::from_le_bytes([first[0], rest[0], rest[1], rest[2]]) as usize;
        let mut name = vec![0u8; length];
        self.input.read_exact(&mut name)?;
        let options = self.entries()?;
        let failures = self.entries()?;
        let name = String::from_utf8(name).map_err(io::Error::other)?;
        Ok(Some((name, Recorded { options, failures })))
    }
}

/// One edit's changes: how many, in which Builds, how many distinct full texts, and the first full example.
struct EditGroup {
    count: u64,
    builds: BTreeSet<String>,
    /// The distinct (before, now) texts the edit covered: `2 -> 3` may be three different texts.
    pairs: BTreeSet<(String, String)>,
    example: String,
}

/// What `FORM_PROOF_CENSUS` gives the reasons: the log's entries, the Builds whose old links are refused, and the
/// Builds that hold a logged entry (the census's own Build-level rule, `held_with_picks`).
#[derive(Default)]
struct ReasonCensus {
    log: BTreeSet<String>,
    refused: BTreeSet<String>,
    holders: BTreeSet<String>,
}

/// Whether a record that was asked for (`FORM_PROOF_REASONS=1`) has its reasons: a record without `reasons.bin` fails,
/// whether or not it has the offers.
fn reasons_check(asked: bool, recorded: bool) -> Result<(), String> {
    if asked && !recorded {
        Err(format!(
            "FORM_PROOF_REASONS=1, but {REASONS_FILE} is missing from FORM_PROOF_BEFORE: record with FORM_PROOF_REASONS=1"
        ))
    } else {
        Ok(())
    }
}

/// The reasons that changed between a record and a tree, grouped by their edit (ticket 331). Options are compared
/// where they are refused on both sides (a status that changed kind is the offers' change), failures by their choice.
#[derive(Default)]
struct ReasonChanges {
    compared: u64,
    changed: u64,
    edits: BTreeMap<(String, String), EditGroup>,
    failures_appeared: u64,
    failures_gone: u64,
    /// With the census. A changed option text on a pick the log names, or in a Build whose link is refused, and any
    /// changed, appeared or gone failure in a Build that holds a logged entry (or whose link is refused), is counted
    /// here, one for each option or failure, and is not an edit.
    census: Option<ReasonCensus>,
    named: u64,
    in_refused: u64,
}

impl ReasonChanges {
    fn new(census: Option<ReasonCensus>) -> Self {
        Self {
            census,
            ..Self::default()
        }
    }

    /// Whether the census accounts for a change of an option in `build` on a pick holding `key`.
    fn option_accounted(&mut self, build: &str, key: Option<&str>) -> bool {
        let Some(census) = &self.census else {
            return false;
        };
        if key.is_some_and(|key| census.log.contains(key)) {
            self.named += 1;
            true
        } else if census.refused.contains(build) {
            self.in_refused += 1;
            true
        } else {
            false
        }
    }

    /// Whether the census accounts for a failure (changed, appeared or gone) in `build`.
    fn failure_accounted(&mut self, build: &str) -> bool {
        let Some(census) = &self.census else {
            return false;
        };
        if census.holders.contains(build) {
            self.named += 1;
            true
        } else if census.refused.contains(build) {
            self.in_refused += 1;
            true
        } else {
            false
        }
    }

    /// One text that changed (not accounted for by the census), described by `what` for the example.
    fn edit(&mut self, build: &str, what: &str, (before, now): (&str, &str)) {
        let (from, to) = edit_of(before, now);
        let group = self
            .edits
            .entry((from.to_owned(), to.to_owned()))
            .or_insert_with(|| EditGroup {
                count: 0,
                builds: BTreeSet::new(),
                pairs: BTreeSet::new(),
                example: format!("{build}: {what}: {before:?} -> {now:?}"),
            });
        group.count += 1;
        group.builds.insert(build.to_owned());
        group.pairs.insert((before.to_owned(), now.to_owned()));
    }

    /// One Build's recorded reasons (`texts` names their indices) against what it gives now.
    fn build(&mut self, name: &str, before: &Recorded, texts: &[String], now: &Found) {
        let text = |index: u32| texts.get(index as usize).map_or("?", String::as_str);
        for offer in &now.offers {
            let Some(reason) = &offer.reason else {
                continue;
            };
            let Some(&was) = before.options.get(&fnv(&[&offer.slot, &offer.pick])) else {
                continue;
            };
            self.compared += 1;
            if text(was) != reason {
                self.changed += 1;
                let key = pick_key(&offer.pick);
                if !self.option_accounted(name, key) {
                    let what = format!("{} {}", offer.slot, key.unwrap_or(&offer.pick));
                    self.edit(name, &what, (text(was), reason));
                }
            }
        }
        let mut seen = HashSet::new();
        for (choice, reason) in &now.failures {
            let hash = fnv(&[choice]);
            seen.insert(hash);
            let Some(&was) = before.failures.get(&hash) else {
                if !self.failure_accounted(name) {
                    self.failures_appeared += 1;
                }
                continue;
            };
            self.compared += 1;
            if text(was) != reason {
                self.changed += 1;
                if !self.failure_accounted(name) {
                    self.edit(name, choice, (text(was), reason));
                }
            }
        }
        let gone = before
            .failures
            .keys()
            .filter(|hash| !seen.contains(hash))
            .count();
        for _ in 0..gone {
            if !self.failure_accounted(name) {
                self.failures_gone += 1;
            }
        }
    }

    /// What fails the proof: each edit nobody accepted, each accepted edit no change made, and failures that appeared
    /// or went.
    fn problems(&self, accept: &BTreeSet<(String, String)>) -> Vec<String> {
        let mut problems = Vec::new();
        for (before, now) in self.edits.keys() {
            if !accept.contains(&(before.clone(), now.clone())) {
                problems.push(format!("reasons: unaccepted edit {before:?} -> {now:?}"));
            }
        }
        for (before, now) in accept {
            if !self.edits.contains_key(&(before.clone(), now.clone())) {
                problems.push(format!(
                    "reasons: stale accepted edit {before:?} -> {now:?} (no change made it)"
                ));
            }
        }
        if self.failures_appeared > 0 {
            problems.push(format!("failures: {} appeared", self.failures_appeared));
        }
        if self.failures_gone > 0 {
            problems.push(format!("failures: {} gone", self.failures_gone));
        }
        problems
    }

    /// The unaccepted edits as the accept file's lines, ready to paste (`FORM_PROOF_REASONS_ACCEPT_OUT`).
    fn unaccepted(&self, accept: &BTreeSet<(String, String)>) -> String {
        (self.edits.keys())
            .filter(|(before, now)| !accept.contains(&(before.clone(), now.clone())))
            .map(|(before, now)| edit_line(before, now) + "\n")
            .collect()
    }

    fn report(&self, accept: &BTreeSet<(String, String)>) -> String {
        let mut text = format!(
            "reasons: {} compared (options and failures present on both sides), {} changed in {} edits\n",
            self.compared,
            self.changed,
            self.edits.len()
        );
        if self.census.is_some() {
            writeln!(
                text,
                "reasons census: {} on a pick or in a Build holding a logged entry, {} in Builds whose links are refused (each option or failure counted once)",
                self.named, self.in_refused
            )
            .unwrap();
        }
        writeln!(
            text,
            "failures: {} appeared, {} gone",
            self.failures_appeared, self.failures_gone
        )
        .unwrap();
        let mut edits: Vec<_> = self.edits.iter().collect();
        edits.sort_unstable_by(|(a, x), (b, y)| y.count.cmp(&x.count).then_with(|| a.cmp(b)));
        let shown = std::env::var("FORM_PROOF_REASONS_SHOWN")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(REASONS_SHOWN);
        for ((before, now), group) in edits.into_iter().take(shown) {
            let accepted = if accept.contains(&(before.clone(), now.clone())) {
                " (accepted)"
            } else {
                ""
            };
            writeln!(
                text,
                "  {:6} {before:?} -> {now:?} ({} Builds, {} distinct texts){accepted}\n         e.g. {}",
                group.count,
                group.builds.len(),
                group.pairs.len(),
                group.example
            )
            .unwrap();
        }
        text
    }
}

/// The reasons record of `compare`: its reader, its texts and what changed.
struct ReasonsCompare {
    reader: ReasonsReader<BufReader<std::fs::File>>,
    texts: Vec<String>,
    changes: ReasonChanges,
}

impl ReasonsCompare {
    /// The record in `dir`, if it has one.
    fn open(dir: &Path, census: Option<ReasonCensus>) -> Option<Self> {
        let file = std::fs::File::open(dir.join(REASONS_FILE)).ok()?;
        let texts = std::fs::read_to_string(dir.join(TEXTS_FILE))
            .expect("the reasons record has its texts table");
        Some(Self {
            reader: ReasonsReader {
                input: BufReader::with_capacity(1 << 20, file),
            },
            texts: read_texts(&texts),
            changes: ReasonChanges::new(census),
        })
    }

    fn build(&mut self, name: &str, now: &Found) {
        let before = loop {
            match self.reader.next_build().unwrap() {
                Some((recorded, found)) if recorded == name => break found,
                Some(_) => {}
                None => panic!(
                    "the reasons record holds no Build {name}, or lists the Builds in another order"
                ),
            }
        };
        self.changes.build(name, &before, &self.texts, now);
    }
}

/// The offers that changed between a record and a tree: (pick key, kind before, kind after) → how many options, and
/// the Builds each pick key changed in.
#[derive(Default)]
struct OfferChanges {
    before: u64,
    after: u64,
    only_before: u64,
    only_after: u64,
    changes: BTreeMap<(String, String, String), u64>,
    builds: BTreeMap<String, BTreeSet<String>>,
    /// With the census: every change, split by why (ticket 322).
    split: Option<StatusSplit>,
}

impl OfferChanges {
    /// One Build's recorded options (`kinds` names their indices) against the options now.
    fn build(&mut self, name: &str, before: &HashMap<u64, u8>, kinds: &[String], now: &[Offer]) {
        self.before += before.len() as u64;
        self.after += now.len() as u64;
        let mut seen = 0;
        for offer in now {
            let Some(&was) = before.get(&fnv(&[&offer.slot, &offer.pick])) else {
                self.only_after += 1;
                continue;
            };
            seen += 1;
            let was = kinds.get(usize::from(was)).map_or("?", String::as_str);
            if was != offer.kind {
                let key = pick_key(&offer.pick)
                    .map_or_else(|| offer.pick.chars().take(60).collect(), str::to_owned);
                self.builds
                    .entry(key.clone())
                    .or_default()
                    .insert(name.to_owned());
                if let Some(split) = &mut self.split {
                    split.change(name, &key, was, &offer.kind);
                }
                let change = (key, was.to_owned(), offer.kind.clone());
                *self.changes.entry(change).or_default() += 1;
            }
        }
        self.only_before += (before.len() - seen) as u64;
    }

    /// The options whose status kind changed, appeared or disappeared.
    fn total(&self) -> u64 {
        self.changes.values().sum::<u64>() + self.only_before + self.only_after
    }

    fn report(&self) -> String {
        let mut text = format!(
            "offers: {} options recorded, {} now; only recorded, only now: ({}, {})\n",
            self.before, self.after, self.only_before, self.only_after
        );
        let mut changes: Vec<_> = self.changes.iter().collect();
        changes.sort_unstable_by(|(a, n), (b, m)| m.cmp(n).then_with(|| a.cmp(b)));
        let shown = std::env::var("FORM_PROOF_OFFERS_SHOWN")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(OFFERS_SHOWN);
        writeln!(
            text,
            "offers: {} statuses changed in {} (pick, kinds) groups, the first {}:",
            self.changes.values().sum::<u64>(),
            changes.len(),
            shown.min(changes.len())
        )
        .unwrap();
        for ((key, from, to), n) in changes.into_iter().take(shown) {
            let builds = self.builds.get(key).map_or(0, BTreeSet::len);
            writeln!(text, "  {n:6} {key}: {from} -> {to} ({builds} Builds)").unwrap();
        }
        // The Builds a status changed in, once each; `FORM_PROOF_OFFER_BUILDS_OUT=<file>` writes their names.
        let changed: BTreeSet<&String> = self.builds.values().flatten().collect();
        writeln!(text, "offers: statuses changed in {} Builds", changed.len()).unwrap();
        if let Some(file) = std::env::var_os("FORM_PROOF_OFFER_BUILDS_OUT") {
            let names: Vec<&str> = changed.iter().map(|name| name.as_str()).collect();
            std::fs::write(file, names.join("\n"))
                .expect("FORM_PROOF_OFFER_BUILDS_OUT is writable");
        }
        if let Some(split) = &self.split {
            text.push_str(&split.report());
        }
        text
    }
}

/// The status kinds that refuse a pick: a status that goes to one narrows who may take the option.
const REFUSING: [&str; 2] = ["Unmet", "BreaksLater"];

/// A log entry's narrowing: (kind before, kind now) → options, and the Builds.
type Narrowing = (BTreeMap<(String, String), u64>, BTreeSet<String>);

/// Every offer status that changed, split by why (ticket 322, with `FORM_PROOF_CENSUS`): on an option the change log
/// names (its pick's key is a log entry's), else in a Build whose old link is refused (the link census's), else
/// neither, each named. And per log entry, the statuses that went from an open kind to a refusing one in Builds whose
/// link isn't refused: the size of the narrowing, which the census's line counts don't show.
struct StatusSplit {
    log: BTreeSet<String>,
    refused: BTreeSet<String>,
    narrowed: BTreeMap<String, Narrowing>,
    named: u64,
    in_refused: u64,
    /// `Build: pick key: before -> now` of each other change.
    rest: Vec<String>,
}

impl StatusSplit {
    fn new(log: BTreeSet<String>, refused: BTreeSet<String>) -> Self {
        Self {
            log,
            refused,
            narrowed: BTreeMap::new(),
            named: 0,
            in_refused: 0,
            rest: Vec::new(),
        }
    }

    /// One option of `build` whose status went from `was` to `now`; `key` is its pick's key.
    fn change(&mut self, build: &str, key: &str, was: &str, now: &str) {
        let named = self.log.contains(key);
        let refused = self.refused.contains(build);
        if named && !refused && REFUSING.contains(&now) && !REFUSING.contains(&was) {
            let (kinds, builds) = self.narrowed.entry(key.to_owned()).or_default();
            *kinds.entry((was.to_owned(), now.to_owned())).or_default() += 1;
            builds.insert(build.to_owned());
        }
        if named {
            self.named += 1;
        } else if refused {
            self.in_refused += 1;
        } else {
            self.rest.push(format!("{build}: {key}: {was} -> {now}"));
        }
    }

    /// Each narrowed log entry, the most statuses first, with its kinds; the split; and the first `FORM_PROOF_SHOWN`
    /// changes neither the log nor a refused link accounts for.
    fn report(&self) -> String {
        let mut text = String::new();
        let mut narrowed: Vec<_> = (self.narrowed.iter())
            .map(|(entry, (kinds, builds))| (kinds.values().sum::<u64>(), entry, kinds, builds))
            .collect();
        narrowed.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
        for (count, entry, kinds, builds) in narrowed {
            let kinds: Vec<String> = (kinds.iter())
                .map(|((was, now), n)| format!("{was} -> {now} {n}"))
                .collect();
            writeln!(
                text,
                "census narrowing {entry}: {count} statuses to Unmet or BreaksLater in {} Builds ({})",
                builds.len(),
                kinds.join(", ")
            )
            .unwrap();
        }
        writeln!(
            text,
            "census narrowing: {} other log entries narrowed no status",
            self.log.len() - self.narrowed.len()
        )
        .unwrap();
        let most = std::env::var("FORM_PROOF_SHOWN")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(SHOWN);
        writeln!(
            text,
            "census statuses: {} changed: {} on options the log names, {} in Builds whose links are refused, {} else",
            self.named + self.in_refused + self.rest.len() as u64,
            self.named,
            self.in_refused,
            self.rest.len()
        )
        .unwrap();
        for line in self.rest.iter().take(most) {
            writeln!(text, "  ELSE {line}").unwrap();
        }
        text
    }
}

/// The offers of each Build, `threads` Builds at a time, calling `sink` on each in the Builds' order: memory holds a
/// handful of Builds' options, never all.
fn offers_pass(
    engine: &Engine,
    builds: &[Named],
    threads: usize,
    reasons: bool,
    mut sink: impl FnMut(&Named, &Found),
) {
    for batch in builds.chunks(threads * 4) {
        let found = in_parallel(batch, threads, |(_, build)| Found {
            offers: offers(engine, build, reasons),
            failures: if reasons {
                (engine.failures(build).iter())
                    .map(|f| (format!("{:?}", f.choice), reason_text(engine, &f.issue)))
                    .collect()
            } else {
                Vec::new()
            },
        });
        for (build, found) in batch.iter().zip(&found) {
            sink(build, found);
        }
    }
}

/// `FORM_PROOF_KIND_MAP`'s file: `old kind <TAB> new kind` per line (`#` starts a comment), the slot kinds (`Debug`
/// text, as `slots.txt` holds them) a ticket that renames a kind accepts as equal.
fn read_kind_map(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (old, new) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("a kind map line is `old<TAB>new`: {line:?}"));
            (old.trim().to_owned(), new.trim().to_owned())
        })
        .collect()
}

/// `slots.txt`'s text with each line's kind (its third column) that the map names rewritten, and how many lines
/// were rewritten.
fn map_slot_kinds(slots: &str, map: &BTreeMap<String, String>) -> (String, usize) {
    if map.is_empty() {
        return (slots.to_owned(), 0);
    }
    let mut mapped = 0;
    let lines: Vec<String> = slots
        .lines()
        .map(|line| {
            let mut columns: Vec<&str> = line.splitn(4, '\t').collect();
            if let Some(new) = columns.get(2).and_then(|kind| map.get(*kind)) {
                columns[2] = new;
                mapped += 1;
                return columns.join("\t");
            }
            line.to_owned()
        })
        .collect();
    (lines.join("\n"), mapped)
}

/// Whether `record` writes the offers: only with `FORM_PROOF_OFFERS=1`, since they roughly octuple its time.
fn offers_wanted(variable: Option<&str>) -> bool {
    variable == Some("1")
}

/// Records the offers of every corpus Build to `offers.bin` and `offers-kinds.txt` in `out`, and with `reasons` their
/// reasons to `reasons.bin` and `reasons-texts.txt`, in the one pass; a summary line each.
fn write_offers(out: &Path, reasons: bool) -> String {
    let (engine, data) = corpus::engine().expect("the game data loads");
    let threads = corpus::threads("FORM_PROOF_THREADS");
    let builds = corpus::builds::all(&engine, &data, threads);
    let file = std::fs::File::create(out.join(OFFERS_FILE)).unwrap();
    let mut writer = OffersWriter::new(BufWriter::new(file));
    let mut reasons_writer = reasons.then(|| {
        let file = std::fs::File::create(out.join(REASONS_FILE)).unwrap();
        ReasonsWriter::new(BufWriter::new(file))
    });
    offers_pass(&engine, &builds, threads, reasons, |(name, _), found| {
        writer.build(name, &found.offers).unwrap();
        if let Some(reasons) = &mut reasons_writer {
            reasons.build(name, found).unwrap();
        }
    });
    let options = writer.options;
    let kinds = writer.finish().unwrap();
    std::fs::write(out.join(KINDS_FILE), &kinds).unwrap();
    let bytes = std::fs::metadata(out.join(OFFERS_FILE)).unwrap().len();
    let mut summary = format!(
        "offers {options} options, {} status kinds, {bytes} bytes\n",
        kinds.lines().count()
    );
    if let Some(writer) = reasons_writer {
        let (options, failures) = (writer.options, writer.failures);
        let texts = writer.finish().unwrap();
        std::fs::write(out.join(TEXTS_FILE), &texts).unwrap();
        let bytes = std::fs::metadata(out.join(REASONS_FILE)).unwrap().len();
        summary += &format!(
            "reasons {options} options, {failures} failures, {} texts, {bytes} bytes\n",
            texts.lines().count()
        );
    }
    summary
}

/// The offers record in `dir` against this tree's offers for `builds`, a Build at a time. The record may hold Builds
/// `builds` doesn't (an old link that no longer decodes); it must hold every Build of `builds`, in the same order.
/// `split`, with the census, splits each change by why.
fn compare_offers(
    engine: &Engine,
    builds: &[Named],
    threads: usize,
    dir: &Path,
    split: Option<StatusSplit>,
    mut reasons: Option<&mut ReasonsCompare>,
) -> OfferChanges {
    let kinds: Vec<String> = std::fs::read_to_string(dir.join(KINDS_FILE))
        .expect("the offers record has its kinds table")
        .lines()
        .map(str::to_owned)
        .collect();
    let file = std::fs::File::open(dir.join(OFFERS_FILE)).unwrap();
    let mut reader = OffersReader {
        input: BufReader::with_capacity(1 << 20, file),
    };
    let mut changes = OfferChanges {
        split,
        ..OfferChanges::default()
    };
    offers_pass(
        engine,
        builds,
        threads,
        reasons.is_some(),
        |(name, _), now| {
            let before = loop {
                match reader.next_build().unwrap() {
                    Some((recorded, options)) if recorded == *name => break options,
                    Some(_) => {}
                    None => panic!(
                        "the offers record holds no Build {name}, or lists the Builds in another order"
                    ),
                }
            };
            changes.build(name, &before, &kinds, &now.offers);
            if let Some(reasons) = &mut reasons {
                reasons.build(name, now);
            }
        },
    );
    changes
}

#[test]
#[ignore = "records about 2,000 real-data Builds: a minute in release"]
fn record() {
    let Some(out) = env_dir("FORM_PROOF_OUT") else {
        return;
    };
    let entries = record_all();
    let mut summary = write_entries(&out, &entries);
    // The reasons share the offers' pass, so asking for them records the offers too.
    let reasons = offers_wanted(std::env::var("FORM_PROOF_REASONS").ok().as_deref());
    if reasons || offers_wanted(std::env::var("FORM_PROOF_OFFERS").ok().as_deref()) {
        summary += &write_offers(&out, reasons);
    }
    println!("{summary}");
}

#[test]
#[ignore = "records about 2,000 real-data Builds and compares: a minute in release"]
fn compare() {
    let Some(before_dir) = env_dir("FORM_PROOF_BEFORE") else {
        return;
    };
    let before = read_entries(&before_dir);
    let mut attribution = std::env::var_os("FORM_PROOF_CENSUS").map(|path| {
        let log = std::fs::read_to_string(&path).expect("FORM_PROOF_CENSUS is readable");
        Attribution::new(read_change_log(&log))
    });
    let kind_map = std::env::var("FORM_PROOF_KIND_MAP")
        .map(|path| {
            read_kind_map(&std::fs::read_to_string(path).expect("the kind map is readable"))
        })
        .unwrap_or_default();
    let mut mapped = 0;
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
    // The Builds are matched by name: one the corpus lost, or gained, shifts every later one by index. Each is
    // listed with the generator lines that can make it (ticket 306af).
    let sources = generator_sources();
    let (lost, gained) = lost_and_gained(&before, &now, &sources);
    for text in lost {
        differences.note("lost Build (in the record, not in the corpus)", text);
    }
    for text in gained {
        differences.note("new Build (in the corpus, not in the record)", text);
    }
    let by_name: HashMap<&str, &Entry> = now.iter().map(|n| (n.name.as_str(), n)).collect();
    for (b, n) in before
        .iter()
        .filter_map(|b| by_name.get(b.name.as_str()).map(|n| (b, *n)))
    {
        let build = n.name.as_str();
        if b.opens && !n.opens {
            let note = generator_note(build, &sources);
            differences.note(
                "opened on the base, not on the tip",
                format!("{build}\n{note}"),
            );
        } else {
            differences.add("opens", build, &b.opens.to_string(), &n.opens.to_string());
        }
        match attribution.as_mut() {
            // The census attributes each differing line of the Build's files to a change-log row.
            Some(attribution) => {
                let (slots, lines) = map_slot_kinds(&b.slots, &kind_map);
                if slots == n.slots {
                    mapped += lines;
                }
                let row = |e: &Entry| format!("{}\t{}\t{}", e.link, e.picks, e.decoded);
                let regenerated = row(b) != row(n) || slots != n.slots;
                let files: [(&str, Classify, String, &str); 4] = [
                    ("builds.tsv", link_kind, row(b), &row(n)),
                    ("slots.txt", slot_kind, slots, &n.slots),
                    ("sheets.txt", sheet_kind, b.sheet.clone(), &n.sheet),
                    ("numbers.txt", numbers_kind, b.numbers.clone(), &n.numbers),
                ];
                let held_set = held_with_picks(&n.held, &[&b.picks, &n.picks]);
                for (file, classify, before, now) in files {
                    let held = (&held_set, regenerated);
                    attribution.lines(file, build, held, (&before, now), classify);
                }
            }
            None => {
                differences.add("link", build, &b.link, &n.link);
                differences.add("picks", build, &b.picks, &n.picks);
                differences.add("decoded picks", build, &b.decoded, &n.decoded);
                differences.add_slots(
                    "slots",
                    build,
                    (b.slots.as_str(), n.slots.as_str()),
                    &kind_map,
                    &mut mapped,
                );
                differences.add("sheet", build, &b.sheet, &n.sheet);
                differences.add("numbers", build, &b.numbers, &n.numbers);
            }
        }
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
                let slots = (b.slots.as_str(), r.slots.as_str());
                match attribution.as_mut() {
                    Some(attribution) => {
                        let held_set = held_with_picks(&r.held, &[&b.picks, &r.picks]);
                        let held = (&held_set, false);
                        let (before, _) = map_slot_kinds(slots.0, &kind_map);
                        attribution.lines(
                            "reopened slots",
                            &b.name,
                            held,
                            (&before, slots.1),
                            slot_kind,
                        );
                        let sheets = (b.sheet.as_str(), r.sheet.as_str());
                        attribution.lines("reopened sheet", &b.name, held, sheets, sheet_kind);
                    }
                    None => {
                        let name = &b.name;
                        differences.add_slots(
                            "reopened slots",
                            name,
                            slots,
                            &kind_map,
                            &mut mapped,
                        );
                        differences.add("reopened sheet", name, &b.sheet, &r.sheet);
                    }
                }
            }
            None => {
                not_reopened += 1;
                differences.add("reopened link", &b.name, "opens", "does not decode");
            }
        }
    }

    // The link census: each old link that opened, opened with this tree's data: the same picks, through an heir (a
    // pick carried to the entry whose `aliases` name its key), or refused. A link that opens as another Build is a
    // difference.
    let census = link_census(&engine, &data, &keys, &before);
    println!("{}", census.summary());
    for (name, why) in &census.changed {
        differences.add("opened link", name, "the same picks or an heir", why);
    }
    if let Some(file) = std::env::var_os("FORM_PROOF_LINKS_OUT") {
        std::fs::write(&file, &census.lines).expect("FORM_PROOF_LINKS_OUT is writable");
    }

    // The offers: every option's status kind at every slot of the old links' Builds, against the record, streamed.
    // The reasons ride the offers' pass (ticket 331), so they are compared when the record has them. A record asked
    // to hold them (`FORM_PROOF_REASONS=1`) that doesn't fails, with or without the offers.
    let reasons_recorded = before_dir.join(REASONS_FILE).exists();
    let asked = offers_wanted(std::env::var("FORM_PROOF_REASONS").ok().as_deref());
    if let Err(why) = reasons_check(asked, reasons_recorded) {
        panic!("{why}");
    }
    let mut reasons_problems = Vec::new();
    let changes = if before_dir.join(OFFERS_FILE).exists() {
        // With the census, each change is split by why, and each logged entry's narrowing sized (ticket 322).
        let split = (attribution.as_ref())
            .map(|a| StatusSplit::new(a.log.keys().cloned().collect(), census.refused.clone()));
        let census_reasons = attribution.as_ref().map(|a| {
            let log: BTreeSet<String> = a.log.keys().cloned().collect();
            // The Builds that hold a logged entry, by the census's own rule (ticket 308b).
            let holders = (before.iter())
                .filter_map(|b| by_name.get(b.name.as_str()).map(|n| (b, *n)))
                .filter(|(b, n)| {
                    let held = held_with_picks(&n.held, &[&b.picks, &n.picks]);
                    held.iter().any(|key| log.contains(key))
                })
                .map(|(b, _)| b.name.clone())
                .collect();
            ReasonCensus {
                log,
                refused: census.refused.clone(),
                holders,
            }
        });
        let mut reasons = ReasonsCompare::open(&before_dir, census_reasons);
        let changes = compare_offers(
            &engine,
            &builds,
            threads,
            &before_dir,
            split,
            reasons.as_mut(),
        );
        println!("{}", changes.report());
        match reasons {
            Some(reasons) => {
                let accept = std::env::var_os("FORM_PROOF_REASONS_ACCEPT")
                    .map(|path| {
                        let text = std::fs::read_to_string(path)
                            .expect("FORM_PROOF_REASONS_ACCEPT is readable");
                        read_accept(&text)
                    })
                    .unwrap_or_default();
                println!("{}", reasons.changes.report(&accept));
                if let Some(file) = std::env::var_os("FORM_PROOF_REASONS_ACCEPT_OUT") {
                    std::fs::write(file, reasons.changes.unaccepted(&accept))
                        .expect("FORM_PROOF_REASONS_ACCEPT_OUT is writable");
                }
                reasons_problems = reasons.changes.problems(&accept);
            }
            None => println!(
                "reasons: not recorded in FORM_PROOF_BEFORE ({REASONS_FILE} is missing), not compared"
            ),
        }
        Some(changes)
    } else {
        println!(
            "offers: not recorded in FORM_PROOF_BEFORE ({OFFERS_FILE} is missing), not compared"
        );
        println!(
            "reasons: not recorded in FORM_PROOF_BEFORE ({REASONS_FILE} is missing), not compared"
        );
        None
    };
    let offer_changes = changes.as_ref().map_or(0, OfferChanges::total);

    // The census: each differing line, attributed to the change-log fields that move it.
    let unattributed = attribution.as_ref().map_or(0, |a| a.unattributed.len());
    if let Some(attribution) = &attribution {
        println!("{}", attribution.report());
    }

    println!(
        "{} Builds compared ({} open), {} old links reopened, {} differences, {offer_changes} offer differences, {} reason problems, {mapped} slot lines mapped by FORM_PROOF_KIND_MAP",
        before.len(),
        before.iter().filter(|e| e.opens).count(),
        before.len() - not_reopened,
        differences.total(),
        reasons_problems.len()
    );
    assert_eq!(
        (differences.total() + unattributed + reasons_problems.len()) as u64 + offer_changes,
        0,
        "the form moved something:\n{}{}{}",
        differences.report(),
        changes.map_or(String::new(), |changes| changes.report()),
        reasons_problems.join("\n")
    );
}

/// The kind of a differing line: what quantity it holds, which a change-log field moves or doesn't.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineKind {
    /// `numbers.txt`: a level's header (scores, base attack bonus, base saves, caster level, mythic rank).
    Header,
    /// `numbers.txt`: the initiative and the three save totals.
    Totals,
    /// `numbers.txt`: a skill's ranks and total.
    Skill,
    /// `numbers.txt`: a caster line.
    Spell,
    /// `numbers.txt`: an attack line.
    Attack,
    /// `numbers.txt`: the steps of the held features.
    Steps,
    /// `sheets.txt`: a line of the sheet.
    Sheet,
    /// `slots.txt`: a slot and its pick.
    Slot,
    /// `builds.tsv`: the Build's link and picks.
    Link,
}

/// The corpus's generated Builds are named `full <own Build>`; only they redraw their picks.
const GENERATED: &str = "full ";

/// What a change-log field moves (ticket 306ae): the line kinds it moves directly, and whether it also regenerates the
/// corpus's full Builds. A full Build's picks, ranks and ability increases come from a generator that draws from
/// what the data offers, so a field that gates a pick (a base attack bonus a feat needs) or sizes a draw (the skill
/// points) can change the Build itself; then every line of that Build is its consequence. A Build whose link, picks or
/// slots did not change, and every Build that is not generated (a hand-made or bench Build never redraws), moves only
/// through the direct kinds.
struct FieldMoves {
    field: &'static str,
    kinds: &'static [LineKind],
    regenerates: bool,
}

/// The change log's fields (the `field` column of `game-check --write --log`) and what each moves. A stage that
/// changes another field adds its row here, with what that field can move: a field absent from the table is refused.
const FIELDS: &[FieldMoves] = &[
    // The base attack bonus: the header's, each attack line's, and the feats it gates.
    FieldMoves {
        field: "bab",
        kinds: &[LineKind::Header, LineKind::Attack],
        regenerates: true,
    },
    // A base save: the header's saves and the save totals.
    FieldMoves {
        field: "fort",
        kinds: &[LineKind::Header, LineKind::Totals],
        regenerates: false,
    },
    FieldMoves {
        field: "reflex",
        kinds: &[LineKind::Header, LineKind::Totals],
        regenerates: false,
    },
    FieldMoves {
        field: "will",
        kinds: &[LineKind::Header, LineKind::Totals],
        regenerates: false,
    },
    // A save a class gives at one level: the same lines as the saves.
    FieldMoves {
        field: "save_exceptions",
        kinds: &[LineKind::Header, LineKind::Totals],
        regenerates: false,
    },
    // Which skills are class skills: the skill lines' totals.
    FieldMoves {
        field: "class_skills",
        kinds: &[LineKind::Skill],
        regenerates: false,
    },
    // The ranks a level gives: the skill lines, and the generator's draws.
    FieldMoves {
        field: "skill_points",
        kinds: &[LineKind::Skill],
        regenerates: true,
    },
    // A prerequisite (ticket 308b): a refused or newly allowed pick can move every line of a Build that holds the
    // entry, at the base or the tip.
    FieldMoves {
        field: "prerequisite",
        kinds: &[
            LineKind::Header,
            LineKind::Totals,
            LineKind::Skill,
            LineKind::Spell,
            LineKind::Attack,
            LineKind::Steps,
            LineKind::Sheet,
            LineKind::Slot,
            LineKind::Link,
        ],
        regenerates: true,
    },
];

/// The `FIELDS` row of a change-log field.
fn field_moves(field: &str) -> &'static FieldMoves {
    FIELDS.iter().find(|row| row.field == field).unwrap_or_else(|| {
        panic!(
            "the change log's field `{field}` is not in the census table (`FIELDS` in form_proof.rs): add the line kinds it moves"
        )
    })
}

/// A change log (`game-check --write --log`: a header, then `entry<TAB>field<TAB>…`): each entry's key (`class x`
/// and `archetype a/b` read as `x` and `a/b`) with the fields it changed. A field the table lacks is refused.
fn read_change_log(text: &str) -> BTreeMap<String, BTreeSet<&'static str>> {
    let mut rows: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
    for line in text.lines().skip(1).filter(|line| !line.is_empty()) {
        let mut columns = line.split('\t');
        let (entry, field) = (columns.next().unwrap_or(""), columns.next());
        let field =
            field.unwrap_or_else(|| panic!("a change-log line has a field column: {line:?}"));
        // `class x`, `selection option a/b` and `mythic ability x` read as their keys.
        let key = entry.rsplit_once(' ').map_or(entry, |(_, key)| key);
        rows.entry(key.to_owned())
            .or_default()
            .insert(field_moves(field).field);
    }
    rows
}

/// The kind of a line of a file: the file's one kind, or a `numbers.txt` line's by its shape.
type Classify = fn(&str) -> LineKind;

fn link_kind(_: &str) -> LineKind {
    LineKind::Link
}

fn slot_kind(_: &str) -> LineKind {
    LineKind::Slot
}

fn sheet_kind(_: &str) -> LineKind {
    LineKind::Sheet
}

/// The kind of a `numbers.txt` line, by its shape: `L<n> scores …` (the header), `  initiative …`, `  steps …`,
/// `  SpellTraits { … }` (a caster line), `  <skill> <ranks> <total> […]`; any other is an attack line
/// (`  <weapon> <attacks> <damage> …`).
fn numbers_kind(line: &str) -> LineKind {
    let words: Vec<&str> = line.split_whitespace().collect();
    let number = |word: Option<&&str>, sign: bool| {
        word.is_some_and(|word| {
            let digits = word.strip_prefix('-').filter(|_| sign).unwrap_or(word);
            !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
        })
    };
    match words.first().copied().unwrap_or("") {
        first if !line.starts_with(' ') && first.starts_with('L') => LineKind::Header,
        "initiative" => LineKind::Totals,
        "steps" => LineKind::Steps,
        "SpellTraits" => LineKind::Spell,
        _ if number(words.get(1), false) && number(words.get(2), true) => LineKind::Skill,
        _ => LineKind::Attack,
    }
}

/// The census of a stage's differing lines (ticket 306ae): each is attributed to a field of the change log that the
/// Build holds an entry of, when that field moves the line's kind, or to a regenerating field when the Build itself
/// changed. A line no field accounts for is unattributed, and `compare` fails on it.
struct Attribution {
    log: BTreeMap<String, BTreeSet<&'static str>>,
    /// Per field: lines attributed, by file, and the Builds.
    counts: BTreeMap<&'static str, (BTreeMap<String, usize>, BTreeSet<String>)>,
    /// `file: Build: kind: line` of each differing line no field accounts for.
    unattributed: Vec<String>,
}

impl Attribution {
    fn new(log: BTreeMap<String, BTreeSet<&'static str>>) -> Self {
        Self {
            log,
            counts: BTreeMap::new(),
            unattributed: Vec::new(),
        }
    }

    /// The lines of `file` for `build` that differ between `before` and `now` (a line one side lacks differs),
    /// attributed. `held` is the entries the Build holds and whether the Build itself changed.
    fn lines(
        &mut self,
        file: &str,
        build: &str,
        (held, regenerated): (&BTreeSet<String>, bool),
        (before, now): (&str, &str),
        classify: Classify,
    ) {
        if before == now {
            return;
        }
        let names: BTreeSet<&'static str> = (held.iter())
            .filter_map(|entry| self.log.get(entry))
            .flatten()
            .copied()
            .collect();
        let fields: Vec<&'static FieldMoves> = names.into_iter().map(field_moves).collect();
        let (old, new): (Vec<&str>, Vec<&str>) = (before.lines().collect(), now.lines().collect());
        for at in 0..old.len().max(new.len()) {
            let (was, is) = (old.get(at), new.get(at));
            if was == is {
                continue;
            }
            let line = is.or(was).copied().unwrap_or("");
            let kind = classify(line);
            let generated = build.starts_with(GENERATED);
            let moves = |row: &&&'static FieldMoves| {
                row.kinds.contains(&kind) || (regenerated && generated && row.regenerates)
            };
            match fields.iter().find(moves) {
                Some(row) => {
                    let (lines, builds) = self.counts.entry(row.field).or_default();
                    *lines.entry(file.to_owned()).or_default() += 1;
                    builds.insert(build.to_owned());
                }
                None => {
                    let shown: String = line.chars().take(110).collect();
                    self.unattributed
                        .push(format!("{file}: {build}: {kind:?}: {shown}"));
                }
            }
        }
    }

    /// The attributed counts per field, and the unattributed lines (the first `FORM_PROOF_SHOWN`).
    fn report(&self) -> String {
        let mut text = String::new();
        for (field, (lines, builds)) in &self.counts {
            let files: Vec<String> = (lines.iter())
                .map(|(file, count)| format!("{file} {count}"))
                .collect();
            let total: usize = lines.values().sum();
            writeln!(
                text,
                "census {field}: {total} lines in {} Builds ({})",
                builds.len(),
                files.join(", ")
            )
            .unwrap();
        }
        let most = std::env::var("FORM_PROOF_SHOWN")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(SHOWN);
        for line in self.unattributed.iter().take(most) {
            writeln!(text, "  UNATTRIBUTED {line}").unwrap();
        }
        write!(text, "census: {} unattributed", self.unattributed.len()).unwrap();
        text
    }
}

/// What the old links that opened came to with this tree's data (ticket 306c).
#[derive(Default)]
struct LinkCensus {
    same: usize,
    heirs: usize,
    /// The Builds whose links are refused.
    refused: BTreeSet<String>,
    /// The links that open as another Build, by Build and how.
    changed: Vec<(String, String)>,
    /// `name<TAB>slot<TAB>key<TAB>kind` of each heir (`key` is `old=>new`) and each refused pick.
    lines: String,
}

impl LinkCensus {
    fn summary(&self) -> String {
        format!(
            "links: {} the same, {} through an heir, {} refused, {} changed",
            self.same,
            self.heirs,
            self.refused.len(),
            self.changed.len()
        )
    }
}

/// Opens the link of each recorded Build that opened, and counts what became of it.
fn link_census(engine: &Engine, data: &GameData, keys: &LinkKeys, before: &[Entry]) -> LinkCensus {
    let mut census = LinkCensus::default();
    for e in before.iter().filter(|e| e.opens) {
        match fate(engine, data, keys, &e.link, &e.picks) {
            Fate::Same => census.same += 1,
            Fate::Heirs(moved) => {
                census.heirs += 1;
                for (slot, old, new) in moved {
                    let name = &e.name;
                    writeln!(census.lines, "{name}\t{slot:?}\t{old}=>{new}\their").unwrap();
                }
            }
            Fate::Refused(picks) => {
                census.refused.insert(e.name.clone());
                for (slot, key) in picks {
                    writeln!(census.lines, "{}\t{slot}\t{key}\trefused", e.name).unwrap();
                }
            }
            Fate::Changed(why) => census.changed.push((e.name.clone(), why)),
        }
    }
    census
}

// ---- The Builds the regenerated corpus lost (ticket 306af) ----------------------------------------------------

/// A file the corpus's names come from: its path under `tools/corpus/` and its text.
struct Source {
    path: String,
    text: String,
}

/// The generators' files: `src/builds/*.rs`, `src/generate.rs` and the benchmark's `bench-links.tsv`.
fn generator_sources() -> Vec<Source> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths: Vec<PathBuf> = std::fs::read_dir(root.join("src/builds"))
        .expect("src/builds is readable")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    paths.sort();
    paths.push(root.join("src/generate.rs"));
    paths.push(root.join("bench-links.tsv"));
    paths
        .into_iter()
        .map(|path| Source {
            text: std::fs::read_to_string(&path).expect("a generator file is readable"),
            path: format!(
                "tools/corpus/{}",
                path.strip_prefix(root).unwrap().display()
            ),
        })
        .collect()
}

/// The string literals of a line of Rust, each as its fixed pieces: the text between its `{…}` placeholders.
fn literal_pieces(line: &str) -> Vec<Vec<String>> {
    let mut literals = Vec::new();
    let mut parts = line.split('"');
    parts.next();
    let mut inside = true;
    for part in parts {
        if inside && !part.ends_with('\\') {
            let mut pieces = vec![String::new()];
            let mut depth = 0;
            for c in part.chars() {
                match c {
                    '{' => {
                        depth += 1;
                        pieces.push(String::new());
                    }
                    '}' if depth > 0 => depth -= 1,
                    _ if depth > 0 => {}
                    _ => pieces.last_mut().unwrap().push(c),
                }
            }
            literals.push(pieces);
        }
        inside = !inside;
    }
    literals
}

/// Whether `name` is a text with `pieces` fixed and the rest filled in: it starts with the first piece, ends with the
/// last and holds the others in order.
fn fits(name: &str, pieces: &[String]) -> bool {
    let (Some(first), Some(last)) = (pieces.first(), pieces.last()) else {
        return false;
    };
    if pieces.len() == 1 {
        return name == first;
    }
    let Some(mut rest) = name.strip_prefix(first.as_str()) else {
        return false;
    };
    for middle in &pieces[1..pieces.len() - 1] {
        match rest.find(middle.as_str()) {
            Some(at) => rest = &rest[at + middle.len()..],
            None => return false,
        }
    }
    rest.ends_with(last.as_str())
}

/// The lines (`path:line: text`) of the generators that can make a Build named `name`, the best fit first: a string
/// literal whose fixed text the name has around its placeholders, the more fixed text the better (at most
/// `shown`). A `full <name>` Build also names the line of the Build it fills. A benchmark Build is its line of
/// `bench-links.tsv`.
fn generator_lines(name: &str, sources: &[Source], shown: usize) -> Vec<String> {
    let mut names = vec![name];
    if let Some(base) = name.strip_prefix("full ") {
        names.push(base);
        if let Some((base, _)) = base.rsplit_once(" / ") {
            names.push(base);
        }
    }
    let mut found: Vec<(usize, String)> = Vec::new();
    for source in sources {
        for (index, line) in source.text.lines().enumerate() {
            let at = format!("{}:{}", source.path, index + 1);
            if source.path.ends_with(".tsv") {
                let bench = line.split('\t').next().unwrap_or_default();
                if name.strip_prefix("bench ") == Some(bench) {
                    found.push((usize::MAX, format!("{at}: {bench}")));
                }
            } else if !line.trim_start().starts_with("//") {
                for pieces in literal_pieces(line) {
                    let fixed: usize = pieces.iter().map(String::len).sum();
                    if fixed >= 3 && names.iter().any(|name| fits(name, &pieces)) {
                        found.push((fixed, format!("{at}: {}", line.trim())));
                    }
                }
            }
        }
    }
    found.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    found.dedup_by(|a, b| a.1 == b.1);
    found
        .into_iter()
        .take(shown)
        .map(|(_, line)| line)
        .collect()
}

/// A Build's generator lines as the lines a difference prints, or one saying none was found.
fn generator_note(name: &str, sources: &[Source]) -> String {
    let lines = generator_lines(name, sources, 3);
    if lines.is_empty() {
        return "    generator: none found by name (a name built from data)".to_owned();
    }
    lines
        .iter()
        .map(|line| format!("    generator: {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The Builds of the record the corpus no longer holds, and those it holds that the record didn't, as the notes the
/// differences print: each name with the generator lines that can make it.
fn lost_and_gained(
    before: &[Entry],
    now: &[Entry],
    sources: &[Source],
) -> (Vec<String>, Vec<String>) {
    let names = |entries: &[Entry]| -> BTreeSet<String> {
        entries.iter().map(|e| e.name.clone()).collect()
    };
    let (old, new) = (names(before), names(now));
    let note =
        |name: &String, sources: &[Source]| format!("{name}\n{}", generator_note(name, sources));
    let lost = old
        .difference(&new)
        .map(|name| note(name, sources))
        .collect();
    let gained = new
        .difference(&old)
        .map(|name| note(name, sources))
        .collect();
    (lost, gained)
}

/// The unit tests of the record's and the compare's pieces (they run with `cargo test -p corpus`).
mod pieces {
    use super::*;

    fn offer(slot: &str, pick: &str, kind: &str) -> Offer {
        Offer {
            slot: slot.to_owned(),
            pick: pick.to_owned(),
            kind: kind.to_owned(),
            reason: None,
        }
    }

    /// A Build read back from a record: its name and `hash -> kind index`.
    type ReadBack = (String, HashMap<u64, u8>);

    const EXTRA: &str =
        r#"Feat { feat: FeatKey("extra-feat"), parameter: Some(Weapon(WeaponKey("longsword"))) }"#;
    const OTHER: &str = r#"Option { option: SelectionOptionKey("rogue/x") }"#;

    /// A record of the Builds, read back: each Build's name and its options.
    fn recorded(builds: &[(&str, Vec<Offer>)]) -> (Vec<ReadBack>, Vec<String>) {
        let mut writer = OffersWriter::new(Vec::new());
        for (name, offers) in builds {
            writer.build(name, offers).unwrap();
        }
        assert_eq!(
            writer.out.len() as u64,
            builds
                .iter()
                .map(|(name, offers)| 8 + name.len() as u64 + 9 * offers.len() as u64)
                .sum::<u64>(),
            "9 bytes an option"
        );
        let bytes = std::mem::take(&mut writer.out);
        let kinds: Vec<String> = writer
            .finish()
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();
        let mut reader = OffersReader {
            input: bytes.as_slice(),
        };
        let mut found = Vec::new();
        while let Some(build) = reader.next_build().unwrap() {
            found.push(build);
        }
        (found, kinds)
    }

    const LOG: &str = "entry\tfield\told\tnew\tfindings removed\tfact\n\
        class stalwart-defender\tfort\tPoor\tGood\tx\t-\n\
        class stalwart-defender\tclass_skills\ta\tb\tx\t262\n\
        archetype druid/feyspeaker\tbab\tthe class's\tHalf\tx\t-\n";
    const HEADER: &str =
        "L1 scores AbilityScores { strength: 10 } bab 0 saves 2/2/0 caster level 0 mythic 0";
    const TOTALS: &str = "  initiative 0 [Dexterity=0] fortitude 2 [Base save=2] reflex 2 [Base save=2] will 0 [Base save=0]";
    const SKILL: &str = "  athletics 0 0 [Ranks=0,Strength=0]";
    const ATTACK: &str = "  longsword +0 1d8 19–20/×2 None [Strength=0/0]";

    fn held(keys: &[&str]) -> BTreeSet<String> {
        keys.iter().map(|key| (*key).to_owned()).collect()
    }

    fn entry(name: &str, opens: bool) -> Entry {
        Entry {
            name: name.to_owned(),
            opens,
            ..Entry::default()
        }
    }

    const PLANTED: &str = "fn old() {\n    // \"t86 {name} / empty\" in a comment\n    let a = format!(\"t86 {} / empty\", key);\n    let b = format!(\n        \"t90 {name} 20\",\n    );\n}\n";

    fn planted() -> Vec<Source> {
        vec![Source {
            path: "tools/corpus/src/builds/planted.rs".to_owned(),
            text: PLANTED.to_owned(),
        }]
    }

    #[test]
    #[should_panic(expected = "share a name (the proof matches by name): {\"t90 witch 20\"}")]
    fn two_builds_with_one_name_are_refused() {
        no_duplicate_names(
            ["t90 witch 20", "t91 new", "t90 witch 20"].into_iter(),
            "the corpus",
        );
    }

    #[test]
    fn a_lost_build_is_named_with_its_generator_line() {
        let before = [entry("t86 monk / empty", true), entry("t90 witch 20", true)];
        let now = [entry("t90 witch 20", true), entry("t91 new", true)];
        let (lost, gained) = lost_and_gained(&before, &now, &planted());
        assert_eq!(
            lost,
            [
                "t86 monk / empty\n    generator: tools/corpus/src/builds/planted.rs:3: \
                 let a = format!(\"t86 {} / empty\", key);"
            ]
        );
        assert_eq!(gained.len(), 1);
        assert!(gained[0].starts_with("t91 new\n    generator: none found by name"));
        // A Build that no longer opens is named the same way.
        assert_eq!(
            generator_note("t90 witch 20", &planted()),
            "    generator: tools/corpus/src/builds/planted.rs:5: \"t90 {name} 20\","
        );
    }

    #[test]
    fn a_full_build_names_the_generator_of_the_build_it_fills() {
        let lines = generator_lines("full t86 monk / empty / ki", &planted(), 3);
        assert!(lines[0].contains("planted.rs:3:"), "{lines:?}");
        let benches = [Source {
            path: "tools/corpus/bench-links.tsv".to_owned(),
            text: "blank\t\nmonk 5\tabc\n".to_owned(),
        }];
        assert_eq!(
            generator_lines("bench monk 5", &benches, 3),
            ["tools/corpus/bench-links.tsv:2: monk 5"]
        );
    }

    #[test]
    fn the_real_generators_are_found_by_a_build_s_name() {
        let lines = generator_lines("t55 fighter some-key / empty", &generator_sources(), 3);
        assert!(
            lines[0].starts_with("tools/corpus/src/builds/mythic.rs:")
                && lines[0].contains("t55 fighter {key} / empty"),
            "{lines:?}"
        );
    }

    #[test]
    fn a_missing_variable_skips_the_mode_and_a_set_one_names_its_folder() {
        let skipped = dir_or_skip("FORM_PROOF_OUT", None).unwrap_err();
        assert!(
            skipped.contains("FORM_PROOF_OUT is not set: skipped"),
            "{skipped}"
        );
        let dir = dir_or_skip("FORM_PROOF_OUT", Some("/x/y".into()));
        assert_eq!(dir, Ok(PathBuf::from("/x/y")));
    }

    #[test]
    fn a_numbers_line_is_classified_by_its_shape() {
        assert_eq!(numbers_kind(HEADER), LineKind::Header);
        assert_eq!(numbers_kind(TOTALS), LineKind::Totals);
        assert_eq!(numbers_kind(SKILL), LineKind::Skill);
        assert_eq!(
            numbers_kind("  trident +8/+3 1d8 ×3 None []"),
            LineKind::Attack
        );
        assert_eq!(numbers_kind(ATTACK), LineKind::Attack);
        assert_eq!(numbers_kind("  steps rage=2,bloodrage=1"), LineKind::Steps);
        assert_eq!(
            numbers_kind("  SpellTraits { school: Evocation } wizard Some(Intelligence) cl 1"),
            LineKind::Spell
        );
        assert_eq!(
            numbers_kind("  stealth 3 -1 [Ranks=3,Dexterity=-4]"),
            LineKind::Skill
        );
    }

    #[test]
    fn the_change_log_reads_each_entry_key_with_its_fields() {
        let log = read_change_log(LOG);
        assert_eq!(
            log["stalwart-defender"],
            BTreeSet::from(["fort", "class_skills"])
        );
        assert_eq!(log["druid/feyspeaker"], BTreeSet::from(["bab"]));
    }

    #[test]
    #[should_panic(expected = "`new_field` is not in the census table")]
    fn a_log_field_the_table_lacks_is_refused() {
        read_change_log("entry\tfield\nclass x\tnew_field\n");
    }

    #[test]
    fn a_line_a_held_field_moves_is_attributed_and_an_unrelated_one_is_not() {
        let mut attribution = Attribution::new(read_change_log(LOG));
        let held = (&held(&["stalwart-defender"]), false);
        // The save totals move with a save field; the skill line with class skills; both held.
        let before = format!("{HEADER}\n{TOTALS}\n{SKILL}");
        let now = format!(
            "{}\n{}\n{}",
            HEADER.replace("saves 2/2/0", "saves 3/2/0"),
            TOTALS.replace("fortitude 2", "fortitude 3"),
            SKILL.replace("Strength=0", "Strength=0,Class skill=3")
        );
        attribution.lines("numbers.txt", "b", held, (&before, &now), numbers_kind);
        assert!(
            attribution.unattributed.is_empty(),
            "{:?}",
            attribution.unattributed
        );
        assert_eq!(attribution.counts["fort"].0["numbers.txt"], 2);
        assert_eq!(attribution.counts["class_skills"].0["numbers.txt"], 1);
        // An attack line moves with neither: it's unattributed, in a Build that holds a changed class.
        attribution.lines(
            "numbers.txt",
            "b",
            held,
            (ATTACK, &ATTACK.replace("+0", "+1")),
            numbers_kind,
        );
        assert_eq!(attribution.unattributed.len(), 1);
        assert!(attribution.unattributed[0].contains("Attack"));
        assert!(attribution.report().contains("census: 1 unattributed"));
        // A Build that holds no changed entry has no attributed line at all.
        let mut other = Attribution::new(read_change_log(LOG));
        other.lines(
            "numbers.txt",
            "c",
            (&held_none(), false),
            (&before, &now),
            numbers_kind,
        );
        assert_eq!(other.unattributed.len(), 3);
    }

    fn held_none() -> BTreeSet<String> {
        held(&["fighter"])
    }

    #[test]
    fn a_regenerated_build_attributes_all_its_lines_to_the_field_that_regenerates() {
        let mut attribution = Attribution::new(read_change_log(LOG));
        let (feyspeaker, other) = (held(&["druid/feyspeaker"]), held(&["stalwart-defender"]));
        let (before, now) = ("L1\tS\tK\tp1\nL2\tS\tK\tp2", "L1\tS\tK\tp1\nL2\tS\tK\tp3");
        attribution.lines(
            "slots.txt",
            "full b",
            (&feyspeaker, true),
            (before, now),
            slot_kind,
        );
        assert_eq!(attribution.counts["bab"].0["slots.txt"], 1);
        // The same lines, unchanged Build: a slot line is no save field's, and no regenerating field is held.
        attribution.lines(
            "slots.txt",
            "b",
            (&feyspeaker, false),
            (before, now),
            slot_kind,
        );
        attribution.lines(
            "slots.txt",
            "full b",
            (&other, true),
            (before, now),
            slot_kind,
        );
        // A hand-made Build never redraws: its changed slot is a real difference, though it holds a changed class.
        attribution.lines(
            "slots.txt",
            "bench b",
            (&feyspeaker, true),
            (before, now),
            slot_kind,
        );
        assert_eq!(attribution.unattributed.len(), 3);
        assert!(attribution.unattributed[2].contains("bench b"));
    }

    #[test]
    fn a_prerequisite_row_attributes_every_line_of_a_build_holding_the_entry_at_the_base() {
        let log = "entry\tfield\told\tnew\tfindings removed\tfact\n\
            feat extra-feat\tprerequisite\ta\tb\tx\t-\n";
        let mut attribution = Attribution::new(read_change_log(log));
        // The base's picks name the entry; the tip's (a refused pick) do not.
        let held = held_with_picks(&held(&["fighter"]), &[EXTRA, ""]);
        assert!(held.contains("extra-feat"));
        let files: [(&str, Classify, &str); 10] = [
            ("builds.tsv", link_kind, "x"),
            ("slots.txt", slot_kind, "x"),
            ("sheets.txt", sheet_kind, "x"),
            ("numbers.txt", numbers_kind, HEADER),
            ("numbers.txt", numbers_kind, TOTALS),
            ("numbers.txt", numbers_kind, SKILL),
            ("numbers.txt", numbers_kind, ATTACK),
            ("numbers.txt", numbers_kind, "  steps 1,2"),
            ("numbers.txt", numbers_kind, "  caster arcane 1"),
            ("numbers.txt", numbers_kind, "  other"),
        ];
        for (file, classify, line) in files {
            attribution.lines(file, "b", (&held, false), (line, "changed"), classify);
        }
        let attributed = &attribution.counts["prerequisite"];
        assert_eq!(attributed.0["numbers.txt"], 7);
        assert_eq!(attributed.1.len(), 1);
        assert!(attribution.unattributed.is_empty());
        // A Build holding no entry of the row is not attributed.
        attribution.lines(
            "slots.txt",
            "c",
            (&held_none(), false),
            ("x", "y"),
            slot_kind,
        );
        assert_eq!(attribution.unattributed.len(), 1);
    }

    #[test]
    fn a_change_log_entry_with_a_two_word_kind_reads_as_its_key() {
        let log = "entry\tfield\told\tnew\tfindings removed\tfact\n\
            selection option rogue/rogue-talents/uncanny-dodge\tprerequisite\ta\tb\tx\t-\n\
            mythic ability impossible-domain\tprerequisite\ta\tb\tx\t-\n\
            class stalwart-defender\tfort\ta\tb\tx\t-\n";
        let rows = read_change_log(log);
        let keys: Vec<&str> = rows.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "impossible-domain",
                "rogue/rogue-talents/uncanny-dodge",
                "stalwart-defender"
            ]
        );
    }

    #[test]
    fn a_line_one_side_lacks_differs() {
        let mut attribution = Attribution::new(read_change_log(LOG));
        let held = held(&["stalwart-defender"]);
        attribution.lines(
            "numbers.txt",
            "b",
            (&held, false),
            (TOTALS, &format!("{TOTALS}\n{ATTACK}")),
            numbers_kind,
        );
        assert_eq!(attribution.unattributed.len(), 1);
    }

    #[test]
    fn the_offers_are_recorded_only_when_asked_for() {
        assert!(!offers_wanted(None));
        assert!(!offers_wanted(Some("0")));
        assert!(offers_wanted(Some("1")));
    }

    #[test]
    fn fnv_is_the_fixed_hash_of_its_parts_with_a_separator() {
        // The FNV-1a 64 offset basis for no parts, and a separator that keeps ("a", "b") from ("ab", "").
        assert_eq!(fnv(&[]), 0xcbf2_9ce4_8422_2325);
        assert_ne!(fnv(&["a", "b"]), fnv(&["ab", ""]));
        assert_eq!(fnv(&["L1", "x"]), fnv(&["L1", "x"]));
    }

    #[test]
    fn pick_key_reads_the_named_key_and_skips_a_parameter() {
        assert_eq!(pick_key(EXTRA), Some("extra-feat"));
        assert_eq!(pick_key(OTHER), Some("rogue/x"));
        assert_eq!(pick_key(r#"Weapon(WeaponKey("longsword"))"#), None);
    }

    #[test]
    fn status_kind_cuts_at_the_first_space_paren_or_brace() {
        assert_eq!(status_kind("Unmet(Condition { needs: x })"), "Unmet");
        assert_eq!(status_kind("Taken {rank: 2}"), "Taken");
        assert_eq!(status_kind("BreaksLater"), "BreaksLater");
    }

    #[test]
    fn a_record_reads_back_its_builds_in_order() {
        let (found, kinds) = recorded(&[
            (
                "x",
                vec![offer("S1", EXTRA, "ok"), offer("S2", EXTRA, "Taken")],
            ),
            ("y", vec![offer("S1", OTHER, "Unmet")]),
        ]);
        assert_eq!(kinds, ["ok", "Taken", "Unmet"]);
        let names: Vec<&str> = found.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["x", "y"]);
        assert_eq!(found[0].1.len(), 2);
        assert_eq!(found[0].1[&fnv(&["S2", EXTRA])], 1);
    }

    #[test]
    fn a_changed_status_is_reported_once_per_option_with_its_pick_kinds_and_builds() {
        let before = [
            (
                "x",
                vec![
                    offer("S1", EXTRA, "ok"),
                    offer("S2", EXTRA, "Taken"),
                    offer("S3", OTHER, "Unmet"),
                ],
            ),
            ("y", vec![offer("S1", EXTRA, "ok")]),
        ];
        let (found, kinds) = recorded(&before);
        let now = [
            // S2 Taken -> ok; S3 Unmet -> Taken; S9 is new.
            vec![
                offer("S1", EXTRA, "ok"),
                offer("S2", EXTRA, "ok"),
                offer("S3", OTHER, "Taken"),
                offer("S9", EXTRA, "ok"),
            ],
            // Unchanged.
            vec![offer("S1", EXTRA, "ok")],
        ];
        let mut changes = OfferChanges::default();
        for ((name, options), now) in found.iter().zip(&now) {
            changes.build(name, options, &kinds, now);
        }
        let key =
            |pick: &str, from: &str, to: &str| (pick.to_owned(), from.to_owned(), to.to_owned());
        assert_eq!(changes.changes[&key("extra-feat", "Taken", "ok")], 1);
        assert_eq!(changes.changes[&key("rogue/x", "Unmet", "Taken")], 1);
        assert_eq!(changes.changes.len(), 2);
        assert_eq!(changes.builds["extra-feat"].len(), 1);
        assert_eq!(
            (
                changes.before,
                changes.after,
                changes.only_before,
                changes.only_after
            ),
            (4, 5, 0, 1)
        );
        assert_eq!(changes.total(), 3);
        let report = changes.report();
        assert!(
            report.contains("1 extra-feat: Taken -> ok (1 Builds)"),
            "{report}"
        );
    }

    /// 322: with the census, each status change is split: on an option the log names (Extra Feat, `extra-feat`), in
    /// a Build whose old link is refused (`refused`), or else, named. Per log entry, the statuses that went to `Unmet`
    /// or `BreaksLater` in Builds whose link isn't refused size the narrowing.
    #[test]
    fn the_census_splits_the_status_changes_and_sizes_each_logged_entrys_narrowing() {
        const MISSED: &str = r#"Feat { feat: FeatKey("missed-feat"), parameter: None }"#;
        let before = [
            (
                "x",
                vec![
                    offer("S1", EXTRA, "ok"),
                    offer("S2", EXTRA, "ok"),
                    offer("S3", EXTRA, "Unmet"),
                    offer("S4", OTHER, "Unmet"),
                ],
            ),
            ("y", vec![offer("S1", EXTRA, "ok")]),
            (
                "refused",
                vec![offer("S1", EXTRA, "ok"), offer("S2", MISSED, "ok")],
            ),
        ];
        let (found, kinds) = recorded(&before);
        let now = [
            // Two narrowings of the logged feat, one widening of it, and a change the log doesn't account for.
            vec![
                offer("S1", EXTRA, "Unmet"),
                offer("S2", EXTRA, "BreaksLater"),
                offer("S3", EXTRA, "ok"),
                offer("S4", OTHER, "Taken"),
            ],
            vec![offer("S1", EXTRA, "Unmet")],
            // The refused link's Build: its logged feat is named, not a narrowing; the other is the refusal's.
            vec![
                offer("S1", EXTRA, "Unmet"),
                offer("S2", MISSED, "BreaksLater"),
            ],
        ];
        let log: BTreeSet<String> = ["extra-feat", "quiet-feat"].map(str::to_owned).into();
        let refused: BTreeSet<String> = ["refused".to_owned()].into();
        let mut changes = OfferChanges {
            split: Some(StatusSplit::new(log, refused)),
            ..OfferChanges::default()
        };
        for ((name, options), now) in found.iter().zip(&now) {
            changes.build(name, options, &kinds, now);
        }
        let split = changes.split.as_ref().unwrap();
        assert_eq!((split.named, split.in_refused), (5, 1));
        assert_eq!(split.rest, ["x: rogue/x: Unmet -> Taken"]);
        let report = changes.report();
        assert!(
            report.contains(
                "census narrowing extra-feat: 3 statuses to Unmet or BreaksLater in 2 Builds (ok -> BreaksLater 1, ok -> Unmet 2)\n"
            ),
            "{report}"
        );
        assert!(
            report.contains("census narrowing: 1 other log entries narrowed no status\n"),
            "{report}"
        );
        assert!(
            report.contains(
                "census statuses: 7 changed: 5 on options the log names, 1 in Builds whose links are refused, 1 else\n  ELSE x: rogue/x: Unmet -> Taken\n"
            ),
            "{report}"
        );
    }

    #[test]
    fn an_option_that_disappears_is_counted_and_an_equal_tree_has_no_differences() {
        let before = [(
            "x",
            vec![offer("S1", EXTRA, "ok"), offer("S2", EXTRA, "Taken")],
        )];
        let (found, kinds) = recorded(&before);
        let mut gone = OfferChanges::default();
        gone.build("x", &found[0].1, &kinds, &[offer("S1", EXTRA, "ok")]);
        assert_eq!((gone.only_before, gone.total()), (1, 1));
        let mut same = OfferChanges::default();
        same.build("x", &found[0].1, &kinds, &before[0].1);
        assert_eq!(same.total(), 0);
    }

    #[test]
    fn the_kind_map_rewrites_only_the_kind_column_and_counts_the_lines() {
        let map = read_kind_map("# comment\n\nOldKind\tNewKind\n");
        let slots = "L1\tSlotId\tOldKind\tPick(OldKind)\nL2\tSlotId\tOther\tPick\nL3\tSlotId\tOldKind\tPick";
        let (mapped, lines) = map_slot_kinds(slots, &map);
        assert_eq!(
            mapped,
            "L1\tSlotId\tNewKind\tPick(OldKind)\nL2\tSlotId\tOther\tPick\nL3\tSlotId\tNewKind\tPick"
        );
        assert_eq!(lines, 2);
        assert_eq!(
            map_slot_kinds(slots, &BTreeMap::new()),
            (slots.to_owned(), 0)
        );
    }

    #[test]
    fn mapped_slot_lines_are_counted_apart_and_a_real_difference_still_fails() {
        let map = read_kind_map("OldKind\tNewKind");
        let (mut differences, mut mapped) = (Differences::default(), 0);
        let before = "L1\tS\tOldKind\tp1\nL2\tS\tOldKind\tp2";
        differences.add_slots(
            "slots",
            "b",
            (before, "L1\tS\tNewKind\tp1\nL2\tS\tNewKind\tp2"),
            &map,
            &mut mapped,
        );
        assert_eq!((differences.total(), mapped), (0, 2));
        // A pick that changed with the kind is a difference, and its lines are not counted as mapped.
        differences.add_slots(
            "slots",
            "b",
            (before, "L1\tS\tNewKind\tp1\nL2\tS\tNewKind\tCHANGED"),
            &map,
            &mut mapped,
        );
        assert_eq!((differences.total(), mapped), (1, 2));
    }

    // The reasons record (ticket 331).

    /// An option refused with `reason`.
    fn refused(slot: &str, pick: &str, reason: &str) -> Offer {
        Offer {
            reason: Some(reason.to_owned()),
            ..offer(slot, pick, "Unmet")
        }
    }

    fn found(offers: Vec<Offer>, failures: &[(&str, &str)]) -> Found {
        Found {
            offers,
            failures: (failures.iter())
                .map(|(choice, text)| ((*choice).to_owned(), (*text).to_owned()))
                .collect(),
        }
    }

    /// Builds written and read back, with the texts table and the bytes written.
    fn reasons_recorded(builds: &[(&str, &Found)]) -> (Vec<(String, Recorded)>, Vec<String>, u64) {
        let mut writer = ReasonsWriter::new(Vec::new());
        for (name, found) in builds {
            writer.build(name, found).unwrap();
        }
        let bytes = std::mem::take(&mut writer.out);
        let size = bytes.len() as u64;
        let texts = read_texts(&writer.finish().unwrap());
        let mut reader = ReasonsReader {
            input: bytes.as_slice(),
        };
        let mut read = Vec::new();
        while let Some(build) = reader.next_build().unwrap() {
            read.push(build);
        }
        (read, texts, size)
    }

    /// `now` against the record of `before`, for Build `name`.
    fn reasons_changed(changes: &mut ReasonChanges, name: &str, before: &Found, now: &Found) {
        let (mut read, texts, _) = reasons_recorded(&[(name, before)]);
        let (_, recorded) = read.remove(0);
        changes.build(name, &recorded, &texts, now);
    }

    const NEEDS_BEFORE: &str = "Needs 1 more of: Kinetic Healer";
    const NEEDS_NOW: &str = "Needs Kinetic Healer";

    #[test]
    fn an_edit_is_the_texts_with_their_common_prefix_and_suffix_removed() {
        assert_eq!(edit_of(NEEDS_BEFORE, NEEDS_NOW), ("1 more of: ", ""));
        assert_eq!(edit_of("same", "same"), ("", ""));
        assert_eq!(edit_of("a", "ab"), ("", "b"));
        // The cut falls on a character boundary, and the suffix never reaches into the prefix.
        assert_eq!(edit_of("é1é", "é2é"), ("1", "2"));
        assert_eq!(edit_of("aa", "aaa"), ("", "a"));
    }

    #[test]
    fn the_accept_file_holds_edits_tab_separated_with_comments_and_blank_lines_skipped() {
        let text = "# why\n\n1 more of: \t\nBreaks\tTabs\\there\\nnew\n";
        let accepted = read_accept(text);
        assert_eq!(accepted.len(), 2);
        assert!(accepted.contains(&("1 more of: ".to_owned(), String::new())));
        assert!(accepted.contains(&("Breaks".to_owned(), "Tabs\there\nnew".to_owned())));
        // An edit prints as the line that accepts it.
        assert_eq!(
            read_accept(&edit_line("a\tb\n", "c\\")),
            BTreeSet::from([("a\tb\n".to_owned(), "c\\".to_owned())])
        );
    }

    #[test]
    fn the_texts_table_keeps_each_text_once_and_survives_newlines() {
        let one = found(
            vec![refused("s", "p1", "Needs A"), refused("s", "p2", "Needs A")],
            &[("Race", "Not allowed here")],
        );
        let two = found(vec![refused("s", "p1", "line\nbreak \\ end")], &[]);
        let (read, texts, size) = reasons_recorded(&[("b1", &one), ("build 2", &two)]);
        assert_eq!(texts, ["Needs A", "Not allowed here", "line\nbreak \\ end"]);
        // Per Build: the name's length and the name, an option count and 12 bytes an option, a failure count and
        // 12 bytes a failure.
        assert_eq!(size, (4 + 2 + 4 + 24 + 4 + 12) + (4 + 7 + 4 + 12 + 4));
        assert_eq!(read[0].0, "b1");
        assert_eq!(read[0].1.options.len(), 2);
        assert_eq!(read[0].1.failures.len(), 1);
        assert_eq!(read[1].1.options[&fnv(&["s", "p1"])], 2);
    }

    #[test]
    fn an_ok_option_is_not_recorded() {
        let open = found(vec![offer("s", "p", "ok")], &[]);
        let (read, texts, _) = reasons_recorded(&[("b", &open)]);
        assert!(read[0].1.options.is_empty() && texts.is_empty());
    }

    #[test]
    fn changed_reasons_group_by_their_edit_across_options_and_builds() {
        let mut changes = ReasonChanges::new(None);
        let before = found(
            vec![
                refused("s", "p1", NEEDS_BEFORE),
                refused("s", "p2", "Needs 1 more of: Dodge"),
                refused("s", "p3", "Already taken"),
                offer("s", "p4", "ok"),
            ],
            &[],
        );
        // p4 was open and is refused now, p5 is new: kinds are the offers' job, not compared here.
        let now = found(
            vec![
                refused("s", "p1", NEEDS_NOW),
                refused("s", "p2", "Needs Dodge"),
                refused("s", "p3", "Already taken"),
                refused("s", "p4", "Not allowed here"),
                refused("s", "p5", "Not allowed here"),
            ],
            &[],
        );
        reasons_changed(&mut changes, "one", &before, &now);
        reasons_changed(&mut changes, "two", &before, &now);
        assert_eq!((changes.compared, changes.changed), (6, 4));
        let edit = ("1 more of: ".to_owned(), String::new());
        assert_eq!(changes.edits.len(), 1);
        assert_eq!(changes.edits[&edit].count, 4);
        assert_eq!(changes.edits[&edit].builds.len(), 2);
        let report = changes.report(&BTreeSet::new());
        assert!(
            report.contains("reasons: 6 compared (options and failures present on both sides), 4 changed in 1 edits"),
            "{report}"
        );
        assert!(report.contains(NEEDS_BEFORE) && report.contains(NEEDS_NOW));
    }

    #[test]
    fn an_edit_fails_unless_it_is_accepted_and_an_accepted_edit_nothing_used_fails() {
        let before = found(vec![refused("s", "p", NEEDS_BEFORE)], &[]);
        let now = found(vec![refused("s", "p", NEEDS_NOW)], &[]);
        let mut changes = ReasonChanges::new(None);
        reasons_changed(&mut changes, "b", &before, &now);
        assert_eq!(changes.problems(&BTreeSet::new()).len(), 1);
        let accept = read_accept("1 more of: \t");
        assert!(changes.problems(&accept).is_empty());
        let stale = read_accept("1 more of: \t\nBreaks\tx");
        assert_eq!(changes.problems(&stale).len(), 1);
        // Nothing changed: the same record passes, and an accept line is stale.
        let mut same = ReasonChanges::new(None);
        reasons_changed(&mut same, "b", &before, &before);
        assert!(same.problems(&BTreeSet::new()).is_empty());
        assert_eq!(same.problems(&accept).len(), 1);
        assert!(same.report(&accept).contains(
            "reasons: 1 compared (options and failures present on both sides), 0 changed in 0 edits"
        ));
    }

    #[test]
    fn a_failure_is_compared_by_its_choice_and_one_that_appears_or_goes_fails() {
        let before = found(
            vec![],
            &[("Race", "Not allowed here"), ("Feat(a)", "Already taken")],
        );
        let now = found(
            vec![],
            &[("Race", "Not allowed here"), ("Feat(b)", "Already taken")],
        );
        let mut changes = ReasonChanges::new(None);
        reasons_changed(&mut changes, "b", &before, &now);
        assert_eq!((changes.failures_appeared, changes.failures_gone), (1, 1));
        assert_eq!(changes.problems(&BTreeSet::new()).len(), 2);
        let report = changes.report(&BTreeSet::new());
        assert!(report.contains("failures: 1 appeared, 1 gone"), "{report}");
        // A failure whose text changed groups under its edit with an option's of the same edit.
        let before = found(
            vec![refused("s", "p", "Needs here")],
            &[("Race", "Not allowed here"), ("Feat(a)", "Already taken")],
        );
        let now = found(
            vec![refused("s", "p", "Needs")],
            &[("Race", "Not allowed"), ("Feat(a)", "Already taken")],
        );
        let mut changes = ReasonChanges::new(None);
        reasons_changed(&mut changes, "b", &before, &now);
        let group = &changes.edits[&(" here".to_owned(), String::new())];
        assert_eq!((group.count, group.pairs.len()), (2, 2));
        assert_eq!(
            changes
                .report(&BTreeSet::new())
                .matches("2 distinct texts")
                .count(),
            1
        );
    }

    #[test]
    fn with_a_census_only_changes_the_log_and_refused_links_do_not_name_fail() {
        let log = BTreeSet::from(["extra-feat".to_owned()]);
        let refused_builds = BTreeSet::from(["refused".to_owned()]);
        let census = ReasonCensus {
            log,
            refused: refused_builds,
            holders: BTreeSet::new(),
        };
        let mut changes = ReasonChanges::new(Some(census));
        let before = found(
            vec![
                refused("s", EXTRA, "Needs A"),
                refused("s", OTHER, "Needs A"),
            ],
            &[],
        );
        let now = found(
            vec![
                refused("s", EXTRA, "Needs B"),
                refused("s", OTHER, "Needs B"),
            ],
            &[],
        );
        reasons_changed(&mut changes, "kept", &before, &now);
        reasons_changed(&mut changes, "refused", &before, &now);
        assert_eq!((changes.named, changes.in_refused), (2, 1));
        // The other option of the kept Build is the only one left to fail.
        assert_eq!(changes.edits[&("A".to_owned(), "B".to_owned())].count, 1);
        assert_eq!(changes.problems(&BTreeSet::new()).len(), 1);
    }

    #[test]
    fn the_reason_is_the_engines_text_and_breaks_later_has_none() {
        let (engine, _) = corpus::engine().expect("the game data loads");
        assert_eq!(reason_text(&engine, &Issue::AlreadyHeld), "Already taken");
        assert_eq!(reason_text(&engine, &Issue::BreaksLater), "BreaksLater");
    }

    #[test]
    fn under_the_census_a_failure_in_a_build_holding_a_logged_entry_is_excused_one_by_one() {
        let census = ReasonCensus {
            log: BTreeSet::from(["x".to_owned()]),
            refused: BTreeSet::from(["refused".to_owned()]),
            holders: BTreeSet::from(["held".to_owned()]),
        };
        let mut changes = ReasonChanges::new(Some(census));
        let before = found(
            vec![],
            &[
                ("A", "Needs 1"),
                ("B", "Needs 1"),
                ("C", "Needs 1"),
                ("D", "Needs 1"),
            ],
        );
        // Two gone, one text changed, one new, in each Build.
        let now = found(
            vec![],
            &[("A", "Needs 1"), ("C", "Needs 2"), ("E", "Needs 1")],
        );
        for build in ["held", "refused", "other"] {
            reasons_changed(&mut changes, build, &before, &now);
        }
        // `held`: 4 failures counted (2 gone, 1 changed, 1 new); `refused`: 4 more; `other`: all fail.
        assert_eq!((changes.named, changes.in_refused), (4, 4));
        assert_eq!((changes.failures_appeared, changes.failures_gone), (1, 2));
        assert_eq!(changes.edits[&("1".to_owned(), "2".to_owned())].count, 1);
        assert_eq!(changes.changed, 3);
        let report = changes.report(&BTreeSet::new());
        assert!(report.contains("reasons census: 4 on a pick"), "{report}");
    }

    #[test]
    fn a_record_asked_to_hold_reasons_without_them_fails_with_or_without_offers() {
        assert!(reasons_check(false, false).is_ok());
        assert!(reasons_check(true, true).is_ok());
        let why = reasons_check(true, false).unwrap_err();
        assert!(why.contains("reasons.bin is missing"), "{why}");
        // A folder with neither file: the check reads only reasons.bin, so the offers' absence can't hide it.
        let dir = std::env::temp_dir().join("form-proof-331-empty-record");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!dir.join(OFFERS_FILE).exists());
        assert!(reasons_check(true, dir.join(REASONS_FILE).exists()).is_err());
        assert!(ReasonsCompare::open(&dir, None).is_none());
    }

    #[test]
    fn edits_print_with_debug_and_the_unaccepted_ones_write_as_accept_lines() {
        let before = found(vec![refused("s", "p", " Needs\tA ")], &[]);
        let now = found(vec![refused("s", "p", " Needs\tB ")], &[]);
        let mut changes = ReasonChanges::new(None);
        reasons_changed(&mut changes, "b", &before, &now);
        let problems = changes.problems(&BTreeSet::new());
        assert_eq!(problems, ["reasons: unaccepted edit \"A\" -> \"B\""]);
        let line = changes.unaccepted(&BTreeSet::new());
        assert_eq!(line, "A\tB\n");
        assert_eq!(
            read_accept(&line),
            BTreeSet::from([("A".to_owned(), "B".to_owned())])
        );
        assert_eq!(changes.unaccepted(&read_accept(&line)), "");
        // A tab in an edit shows as `\t` in the report and in the file.
        let before = found(vec![refused("s", "p", "x\ta")], &[]);
        let now = found(vec![refused("s", "p", "x\tb")], &[]);
        let mut changes = ReasonChanges::new(None);
        reasons_changed(&mut changes, "b", &before, &now);
        assert!(changes.report(&BTreeSet::new()).contains("\"a\" -> \"b\""));
    }

    #[test]
    fn a_leading_hash_and_a_carriage_return_survive_the_accept_file() {
        let line = edit_line("# first", "\r\nsecond");
        assert_eq!(line, "\\# first\t\\r\\nsecond");
        assert!(!line.starts_with('#'));
        assert_eq!(
            read_accept(&format!("{line}\n")),
            BTreeSet::from([("# first".to_owned(), "\r\nsecond".to_owned())])
        );
        assert_eq!(read_texts(&escape("#a\r")), ["#a\r"]);
    }
}
