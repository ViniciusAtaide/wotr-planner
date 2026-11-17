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
//! - **Unset, `record` and `compare` print a skip line and pass**, so `cargo test --release -p corpus -- --ignored`
//!   passes.
//!
//! `FORM_PROOF_THREADS` sets the threads (8 by default). The hashes are `DefaultHasher`'s, so they compare only
//! within one toolchain; `compare` never reads them.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use corpus::links::{Fate, fate};
use corpus::{Named, in_parallel};
use rules::{
    Build, CasterPart, Energy, Engine, GameData, LinkKeys, SpellSchool, SpellTraits, Subschool,
    Total, WeaponKey, decode_build, encode_build,
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
fn offers(engine: &Engine, build: &Build) -> Vec<Offer> {
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
            let kind = match &option.status {
                Ok(()) => "ok".to_owned(),
                Err(issue) => status_kind(&format!("{issue:?}")).to_owned(),
            };
            found.push(Offer {
                slot: slot_text.clone(),
                pick: format!("{:?}", option.pick),
                kind,
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
        text
    }
}

/// The offers of each Build, `threads` Builds at a time, calling `sink` on each in the Builds' order: memory holds a
/// handful of Builds' options, never all.
fn offers_pass(
    engine: &Engine,
    builds: &[Named],
    threads: usize,
    mut sink: impl FnMut(&Named, &[Offer]),
) {
    for batch in builds.chunks(threads * 4) {
        let found = in_parallel(batch, threads, |(_, build)| offers(engine, build));
        for (build, offers) in batch.iter().zip(&found) {
            sink(build, offers);
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

/// Records the offers of every corpus Build to `offers.bin` and `offers-kinds.txt` in `out`; one summary line.
fn write_offers(out: &Path) -> String {
    let (engine, data) = corpus::engine().expect("the game data loads");
    let threads = corpus::threads("FORM_PROOF_THREADS");
    let builds = corpus::builds::all(&engine, &data, threads);
    let file = std::fs::File::create(out.join(OFFERS_FILE)).unwrap();
    let mut writer = OffersWriter::new(BufWriter::new(file));
    offers_pass(&engine, &builds, threads, |(name, _), offers| {
        writer.build(name, offers).unwrap();
    });
    let options = writer.options;
    let kinds = writer.finish().unwrap();
    std::fs::write(out.join(KINDS_FILE), &kinds).unwrap();
    let bytes = std::fs::metadata(out.join(OFFERS_FILE)).unwrap().len();
    format!(
        "offers {options} options, {} status kinds, {bytes} bytes\n",
        kinds.lines().count()
    )
}

/// The offers record in `dir` against this tree's offers for `builds`, a Build at a time. The record may hold Builds
/// `builds` doesn't (an old link that no longer decodes); it must hold every Build of `builds`, in the same order.
fn compare_offers(engine: &Engine, builds: &[Named], threads: usize, dir: &Path) -> OfferChanges {
    let kinds: Vec<String> = std::fs::read_to_string(dir.join(KINDS_FILE))
        .expect("the offers record has its kinds table")
        .lines()
        .map(str::to_owned)
        .collect();
    let file = std::fs::File::open(dir.join(OFFERS_FILE)).unwrap();
    let mut reader = OffersReader {
        input: BufReader::with_capacity(1 << 20, file),
    };
    let mut changes = OfferChanges::default();
    offers_pass(engine, builds, threads, |(name, _), now| {
        let before = loop {
            match reader.next_build().unwrap() {
                Some((recorded, options)) if recorded == *name => break options,
                Some(_) => {}
                None => panic!(
                    "the offers record holds no Build {name}, or lists the Builds in another order"
                ),
            }
        };
        changes.build(name, &before, &kinds, now);
    });
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
    if offers_wanted(std::env::var("FORM_PROOF_OFFERS").ok().as_deref()) {
        summary += &write_offers(&out);
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
    let changes = if before_dir.join(OFFERS_FILE).exists() {
        let changes = compare_offers(&engine, &builds, threads, &before_dir);
        println!("{}", changes.report());
        Some(changes)
    } else {
        println!(
            "offers: not recorded in FORM_PROOF_BEFORE ({OFFERS_FILE} is missing), not compared"
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
        "{} Builds compared ({} open), {} old links reopened, {} differences, {offer_changes} offer differences, {mapped} slot lines mapped by FORM_PROOF_KIND_MAP",
        before.len(),
        before.iter().filter(|e| e.opens).count(),
        before.len() - not_reopened,
        differences.total()
    );
    assert_eq!(
        (differences.total() + unattributed) as u64 + offer_changes,
        0,
        "the form moved something:\n{}{}",
        differences.report(),
        changes.map_or(String::new(), |changes| changes.report())
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
    refused: usize,
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
            self.refused,
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
                census.refused += 1;
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
}
