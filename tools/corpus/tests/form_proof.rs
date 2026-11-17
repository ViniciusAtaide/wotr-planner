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
//! - **The link census** (ticket 306c): `compare` opens each old link that opened with this tree's data and prints
//!   how many opened with the same picks, through an heir (a pick carried to the entry whose `aliases` name its key)
//!   or refused (`links: N the same, N through an heir, N refused, N changed`); one that opens as another Build is a
//!   difference. `FORM_PROOF_LINKS_OUT=<file>` writes `name<TAB>slot<TAB>key<TAB>kind` of each heir (`old=>new`)
//!   and each refused pick. `tests/links.rs` pins a stage's base links and refusals.
//! - **`FORM_PROOF_KIND_MAP=<file>`** is for a ticket that renames a slot kind: a file of `old kind<TAB>new kind` lines
//!   (the `Debug` text of the kind, as `slots.txt` holds it; `#` comments). `compare` accepts a slot line that
//!   differs only by its kind through the map, and prints how many it mapped apart from the differences.
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
    let entries = record_all();
    let out = env_dir("FORM_PROOF_OUT");
    let mut summary = write_entries(&out, &entries);
    if offers_wanted(std::env::var("FORM_PROOF_OFFERS").ok().as_deref()) {
        summary += &write_offers(&out);
    }
    println!("{summary}");
}

#[test]
#[ignore = "records about 2,000 real-data Builds and compares: a minute in release"]
fn compare() {
    let before_dir = env_dir("FORM_PROOF_BEFORE");
    let before = read_entries(&before_dir);
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
    for (b, n) in before.iter().zip(&now) {
        let build = n.name.as_str();
        differences.add("name", &b.name, &b.name, build);
        differences.add("link", build, &b.link, &n.link);
        differences.add("opens", build, &b.opens.to_string(), &n.opens.to_string());
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
                differences.add_slots("reopened slots", &b.name, slots, &kind_map, &mut mapped);
                differences.add("reopened sheet", &b.name, &b.sheet, &r.sheet);
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

    println!(
        "{} Builds compared ({} open), {} old links reopened, {} differences, {offer_changes} offer differences, {mapped} slot lines mapped by FORM_PROOF_KIND_MAP",
        before.len(),
        before.iter().filter(|e| e.opens).count(),
        before.len() - not_reopened,
        differences.total()
    );
    assert_eq!(
        differences.total() as u64 + offer_changes,
        0,
        "the form moved something:\n{}{}",
        differences.report(),
        changes.map_or(String::new(), |changes| changes.report())
    );
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
