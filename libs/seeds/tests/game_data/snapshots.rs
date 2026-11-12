//! The cheap behaviour check (ticket 159): a small corpus of Builds, each projected into flat `path = value` lines
//! from the engine's output and compared with the snapshot committed beside this file (`snapshots/<name>.txt`).
//!
//! The corpus: the bench Builds ([`support::bench`]), every base class at level 20 with no picks, and a fighter 20
//! (Mythic rank 10, or as far as a Legend switch goes) on every mythic path with no picks beyond the path, the last
//! two from the data's own lists. The bench Builds get their sheet; the pick-less ones their sheet and every slot's
//! open options. Prestige classes have no pick-less Build: their entry needs feats and skill ranks.
//!
//! The claim is weaker than the behaviour proof's (`scripts/proof.sh`): nothing changed in these Builds. A difference
//! fails the test with the changes grouped by path ([`message`]); an intended one is accepted by regenerating:
//! `SNAPSHOTS=write cargo test -p seeds --test game_data snapshots`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use rules::{
    AbilityScores, Address, Alignment, Build, CharacterState, Energy, Engine, LevelSheet,
    MythicPathKey, POINT_BUY_BUDGET, Pick, Sheet, Slot, SpellSchool, SpellTraits, Subschool, Total,
    has_ability_increase,
};

use crate::support::{self, DATA};

/// The command that rewrites the snapshot files, the failure message's last line.
const REGENERATE: &str = "SNAPSHOTS=write cargo test -p seeds --test game_data snapshots";

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/game_data/snapshots")
}

// ---- the corpus -----------------------------------------------------------------------------------------------

/// A Build of the corpus, with its snapshot's name and whether the snapshot lists each slot's open options.
struct Subject {
    name: String,
    build: Build,
    options: bool,
}

/// The bench Builds, then one pick-less Build per class and per mythic path, each asserted possible: an impossible
/// Build has no options and would snapshot as empty.
fn corpus() -> Vec<Subject> {
    let mut subjects: Vec<Subject> = support::bench()
        .into_iter()
        .map(|(name, build)| Subject {
            name: format!("bench-{name}"),
            build,
            options: false,
        })
        .collect();
    for class in &DATA.classes {
        let build = possible(support::build(class.key.as_str(), 20));
        subjects.push(Subject {
            name: format!("class-{}", class.key),
            build,
            options: true,
        });
    }
    for path in &DATA.mythic_paths {
        subjects.push(Subject {
            name: format!("path-{}", path.key),
            build: path_build(&path.key),
            options: true,
        });
    }
    let engine = support::engine();
    for subject in &subjects {
        assert!(
            engine.is_possible(&subject.build),
            "{} is impossible",
            subject.name
        );
    }
    subjects
}

/// The Build as it is when possible, else at the first alignment (with a faith that fits it) that makes it so; as it
/// is when none does, for [`corpus`] to name.
fn possible(build: Build) -> Build {
    if support::possible(&build) {
        return build;
    }
    Alignment::ALL
        .into_iter()
        .map(|alignment| support::realigned(build.clone(), alignment))
        .find(support::possible)
        .unwrap_or(build)
}

/// A fighter 20 on `path` with no other pick. A path taken only by a switch (Legend, Devil, Gold Dragon, the Swarm)
/// is switched to from the first path, in key order, it can be switched to from.
fn path_build(path: &MythicPathKey) -> Build {
    let mut build = support::build("fighter", 20);
    let data = DATA
        .mythic_paths
        .iter()
        .find(|candidate| &candidate.key == path)
        .expect("a path of the data");
    match &data.switch {
        None => build.mythic_path = Some(path.clone()),
        Some(switch) => {
            let from = DATA
                .mythic_paths
                .iter()
                .filter(|from| from.switch.is_none())
                .find(|from| switch.from.is_empty() || switch.from.contains(&from.key))
                .expect("a path to switch from");
            build.mythic_path = Some(from.key.clone());
            build.path_switch = Some(path.clone());
        }
    }
    possible(build)
}

// ---- the projection -------------------------------------------------------------------------------------------

/// A Build's lines, `(path, value)` in tree order: the Build start, each level, each Mythic rank, then the last
/// level's totals and caster lines.
type Lines = Vec<(String, String)>;

/// The Build as lines, with each slot's open options after it when `options`. The shape follows the MCP server's
/// `sheet`, which this test can't call (`mcp` depends on `seeds`).
fn project(engine: &Engine, build: &Build, options: bool) -> Lines {
    let sheet = engine.evaluate(build);
    let mut out = Projection {
        engine,
        build,
        options,
        lines: Vec::new(),
    };
    out.start();
    out.levels(&sheet);
    out.ranks(&sheet);
    if let Some(last) = sheet.levels.last() {
        out.last(last);
    }
    let mut seen = BTreeSet::new();
    for (path, _) in &out.lines {
        assert!(seen.insert(path), "two lines at {path}");
    }
    out.lines
}

struct Projection<'a> {
    engine: &'a Engine,
    build: &'a Build,
    options: bool,
    lines: Lines,
}

/// A value with its name: `key "Name"`.
fn named(key: &str, name: &str) -> String {
    format!("{key} \"{name}\"")
}

/// A pick as an option list names it: `key`, or `key(value)` for a parameterised one.
fn pick_key(pick: &Pick) -> String {
    match pick.parameter() {
        Some(value) => format!("{}({})", pick.key(), value.value_key()),
        None => pick.key().to_owned(),
    }
}

/// A number and what it counts: `5: Dexterity +1, Improved Initiative +4`, or the number alone when it counts nothing.
fn counted<'a>(
    number: impl std::fmt::Display,
    parts: impl IntoIterator<Item = (&'a str, i16)>,
) -> String {
    let parts: Vec<String> = parts
        .into_iter()
        .map(|(name, value)| format!("{name} {value:+}"))
        .collect();
    if parts.is_empty() {
        number.to_string()
    } else {
        format!("{number}: {}", parts.join(", "))
    }
}

fn total(total: &Total) -> String {
    counted(
        total.total,
        total
            .parts
            .iter()
            .map(|part| (part.name.as_str(), part.value)),
    )
}

fn caster_parts(parts: &[rules::CasterPart]) -> impl Iterator<Item = (&str, i16)> {
    parts.iter().map(|part| (part.name.as_str(), part.value))
}

impl Projection<'_> {
    /// A line; an empty value is `-`, so no line ends in a space.
    fn line(&mut self, path: impl Into<String>, value: impl Into<String>) {
        let value = value.into();
        let value = if value.is_empty() { "-".into() } else { value };
        self.lines.push((path.into(), value));
    }

    fn choice(&mut self, address: Address, value: Option<String>) {
        self.line(address.to_string(), value.unwrap_or_else(|| "-".into()));
    }

    fn start(&mut self) {
        let (engine, build) = (self.engine, self.build);
        let start = &build.start;
        if let Some(name) = &build.name {
            self.line("name", name.clone());
        }
        let race = start
            .race
            .as_ref()
            .map(|race| named(race.as_str(), engine.race_name(race)));
        self.choice(Address::Race, race);
        if let Some(subrace) = &start.subrace {
            let value = named(subrace.as_str(), engine.subrace_name(subrace));
            self.choice(Address::Subrace, Some(value));
        }
        if engine.chooses_racial_ability(start) {
            let value = start
                .racial_ability
                .map(|ability| named(ability.key(), ability.name()));
            self.choice(Address::RacialAbility, value);
        }
        let faith = start
            .deity
            .as_ref()
            .map(|faith| named(faith.key(), engine.faith_name(faith)));
        self.choice(Address::Faith, faith);
        let alignment = start
            .alignment
            .map(|alignment| named(alignment.key(), alignment.name()));
        self.choice(Address::Alignment, alignment);
        let background = start
            .background
            .as_ref()
            .map(|key| named(key.as_str(), engine.background_name(key)));
        self.choice(Address::Background, background);
        for ability in AbilityScores::ABILITIES {
            let score = start.ability_scores.get(ability).to_string();
            self.choice(Address::Score(ability), Some(score));
        }
        let spent = start.ability_scores.cost();
        self.line("start.point-buy", format!("{spent} of {POINT_BUY_BUDGET}"));
        let path = |key: &Option<MythicPathKey>| {
            key.as_ref()
                .map(|key| named(key.as_str(), engine.path_name(key)))
        };
        self.choice(Address::FirstAscension, path(&build.first_ascension));
        self.choice(Address::Path, path(&build.mythic_path));
        self.choice(Address::Switch, path(&build.path_switch));
    }

    /// Each level: its class, numbers and choices, then the feats and features it gains (the state at its end
    /// against the one before, so one new feature is one line).
    fn levels(&mut self, sheet: &Sheet) {
        let (engine, build) = (self.engine, self.build);
        self.line("level-cap", sheet.level_cap.to_string());
        let empty = CharacterState::default();
        let mut before = &empty;
        for (level_up, level) in build.levels.iter().zip(&sheet.levels) {
            let n = level.character_level;
            let state = &level.state;
            let at = |what: &str| format!("level.{n}.{what}");
            let archetype = state
                .classes
                .iter()
                .find(|taken| taken.class == level.class)
                .and_then(|taken| taken.archetype.as_ref());
            let advances = engine.asks_advances(build, n, &level.class).then(|| {
                build
                    .class_choices
                    .get(&level.class)
                    .map(|choice| choice.advances.clone())
                    .unwrap_or_default()
            });
            let mut class = level.class.to_string();
            if let Some(archetype) = archetype {
                class += &format!(" + {archetype}");
            }
            if let Some(advances) = &advances {
                let keys: Vec<String> = advances.iter().map(ToString::to_string).collect();
                class += &format!(" advancing [{}]", keys.join(", "));
            }
            let name = engine.class_value_name(&level.class, archetype, advances.as_ref());
            self.choice(Address::Class(n), Some(named(&class, &name)));
            self.line(at("class-level"), level.class_level.to_string());
            self.line(at("bab"), state.bab.to_string());
            self.line(at("saves.fort"), state.fort.to_string());
            self.line(at("saves.reflex"), state.reflex.to_string());
            self.line(at("saves.will"), state.will.to_string());
            for ability in AbilityScores::ABILITIES {
                let score = state.ability_scores.get(ability).to_string();
                self.line(at(&format!("scores.{}", ability.key())), score);
            }
            let points = level.skill_points;
            self.line(at("skill-points.available"), points.available.to_string());
            self.line(at("skill-points.spent"), points.spent.to_string());
            if has_ability_increase(n) {
                let value = level_up
                    .ability_increase
                    .map(|ability| named(ability.key(), &ability.increase_name()));
                self.choice(Address::Increase(n), value);
            }
            for (skill, ranks) in &level_up.skill_ranks {
                self.choice(Address::Skill(n, skill.clone()), Some(ranks.to_string()));
            }
            for slot in &level.slots {
                self.slot(slot);
            }
            for (feat, name) in new_feats(engine, before, state) {
                self.line(at(&format!("feat/{feat}")), name);
            }
            for feature in state
                .features
                .iter()
                .filter(|(feature, times)| before.feature_count(feature) < **times)
                .map(|(feature, _)| feature)
            {
                let name = engine.feature_name_apart(feature);
                self.line(at(&format!("feature/{feature}")), name);
            }
            before = state;
        }
    }

    /// A slot's line, `Label: key "Name"` or `Label: -` while empty, then its open options when asked.
    fn slot(&mut self, slot: &Slot) {
        let engine = self.engine;
        let address = Address::Slot(slot.id.clone());
        let label = engine.choice_label(&address, Some(slot));
        let held = slot.pick.as_ref().map_or_else(
            || "-".to_owned(),
            |pick| named(&pick_key(pick), &engine.pick_name(pick)),
        );
        self.line(address.to_string(), format!("{label}: {held}"));
        if self.options {
            let open: BTreeSet<String> = engine
                .options(self.build, &slot.id)
                .expect("the sheet's slot has options")
                .into_iter()
                .filter(|option| option.status.is_ok())
                .map(|option| pick_key(&option.pick))
                .collect();
            let open: Vec<String> = open.into_iter().collect();
            self.line(format!("{address}.open"), open_list(&open));
        }
    }

    /// Each Mythic rank, reached or not: its level, status and path, its slots and the features it gives.
    fn ranks(&mut self, sheet: &Sheet) {
        let engine = self.engine;
        for rank in &sheet.ranks {
            let at = |what: &str| format!("rank.{}.{what}", rank.rank);
            self.line(at("character-level"), rank.character_level.to_string());
            self.line(at("status"), format!("{:?}", rank.status));
            if let Some(path) = &rank.path {
                self.line(at("path"), named(path.as_str(), engine.path_name(path)));
            }
            for slot in &rank.slots {
                self.slot(slot);
            }
            // A feature the rank gives twice (the Trickster's Sneak Attack) is one line, `×2`.
            let mut features: Vec<(&rules::FeatureKey, usize)> = Vec::new();
            for feature in engine.rank_features(rank) {
                match features.iter_mut().find(|(seen, _)| *seen == feature) {
                    Some((_, times)) => *times += 1,
                    None => features.push((feature, 1)),
                }
            }
            for (feature, times) in features {
                let mut name = engine.feature_name_apart(feature);
                if times > 1 {
                    name += &format!(" ×{times}");
                }
                self.line(at(&format!("feature/{feature}")), name);
            }
            if !rank.dropped_slots.is_empty() {
                let dropped: Vec<String> = rank
                    .dropped_slots
                    .iter()
                    .map(|slot| Address::Slot(slot.clone()).to_string())
                    .collect();
                self.line(at("dropped"), dropped.join(", "));
            }
        }
    }

    /// The last level's spellcasting, class skills, totals and caster lines.
    fn last(&mut self, last: &LevelSheet) {
        let engine = self.engine;
        let state = &last.state;
        self.line("last.caster-level", state.caster_level.to_string());
        self.line(
            "last.spell-level.arcane",
            state.arcane_spell_level.to_string(),
        );
        self.line(
            "last.spell-level.divine",
            state.divine_spell_level.to_string(),
        );
        let skills: Vec<&str> = state.class_skills.iter().map(|s| s.as_str()).collect();
        self.line("last.class-skills", skills.join(", "));
        let totals = engine.totals(last);
        self.line("totals.initiative", total(&totals.initiative));
        self.line("totals.fortitude", total(&totals.fortitude));
        self.line("totals.reflex", total(&totals.reflex));
        self.line("totals.will", total(&totals.will));
        for skill in &totals.skills {
            self.line(format!("totals.skill.{}", skill.skill), total(&skill.total));
        }
        self.caster(last);
    }

    /// The caster lines: the casting class's numbers, then each school's DCs, and Evocation's for each energy type
    /// and with metamagic, and Enchantment's for charm and compulsion (the subschools a DC source names).
    fn caster(&mut self, last: &LevelSheet) {
        let engine = self.engine;
        let traits = |school, subschool, energy, metamagic| SpellTraits {
            school,
            subschool,
            energy,
            metamagic,
        };
        let mut spells: Vec<(String, SpellTraits)> = SpellSchool::ALL
            .into_iter()
            .map(|school| (lower(school.name()), traits(school, None, None, false)))
            .collect();
        for energy in Energy::ALL {
            let name = format!("evocation.{}", lower(energy.name()));
            spells.push((
                name,
                traits(SpellSchool::Evocation, None, Some(energy), false),
            ));
        }
        for subschool in [Subschool::Charm, Subschool::Compulsion] {
            let name = format!("enchantment.{}", lower(subschool.name()));
            let spell = traits(SpellSchool::Enchantment, Some(subschool), None, false);
            spells.push((name, spell));
        }
        spells.push((
            "evocation.metamagic".into(),
            traits(SpellSchool::Evocation, None, None, true),
        ));
        let Some(line) = engine.caster_line(last, spells[0].1) else {
            return;
        };
        let class = named(line.class.as_str(), engine.class_name(&line.class));
        self.line("caster.class", class);
        self.line("caster.ability", line.ability.name());
        self.line("caster.level", line.caster_level.to_string());
        let penetration = counted(
            line.spell_penetration,
            caster_parts(&line.spell_penetration_parts),
        );
        self.line("caster.spell-penetration", penetration);
        let concentration = counted(line.concentration, caster_parts(&line.concentration_parts));
        self.line("caster.concentration", concentration);
        for (name, spell) in spells {
            let line = engine
                .caster_line(last, spell)
                .expect("a caster line for every spell");
            let dcs: Vec<String> = line.dcs.iter().map(|dc| dc.dc.to_string()).collect();
            let mut value = counted(dcs.join(" "), caster_parts(&line.dc_parts));
            if let Some(heightened) = line.heightened {
                value += &format!(" (heightened {})", heightened.dc);
            }
            self.line(format!("caster.dc.{name}"), value);
        }
    }
}

fn lower(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

/// The feats held at the end of a level and not before it, as `(key, name)`: a parameterised one once per new value
/// (`weapon-focus(longsword)`, "Weapon Focus (Longsword)").
fn new_feats(
    engine: &Engine,
    before: &CharacterState,
    after: &CharacterState,
) -> Vec<(String, String)> {
    let mut feats = Vec::new();
    for feat in after.feats.keys() {
        let name = engine.feat_name(feat);
        match after.feat_values.get(feat) {
            Some(values) => {
                for value in values
                    .keys()
                    .filter(|value| !before.has_feat_for(feat, value))
                {
                    let key = format!("{feat}({})", value.value_key());
                    feats.push((key, format!("{name} ({})", engine.parameter_name(value))));
                }
            }
            None if !before.feats.contains_key(feat) => feats.push((feat.to_string(), name.into())),
            None => {}
        }
    }
    feats
}

// ---- the files ------------------------------------------------------------------------------------------------

/// An `.open` line's value: its keys, or `-` for none.
fn open_list(keys: &[String]) -> String {
    if keys.is_empty() {
        "-".into()
    } else {
        keys.join(", ")
    }
}

fn render(lines: &Lines) -> String {
    lines
        .iter()
        .map(|(path, value)| format!("{path} = {value}\n"))
        .collect()
}

fn parse(text: &str) -> Lines {
    text.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (path, value) = line
                .split_once(" = ")
                .unwrap_or_else(|| panic!("a snapshot line is `path = value`: {line}"));
            (path.to_owned(), value.to_owned())
        })
        .collect()
}

/// The stored snapshots by name.
fn read_snapshots(dir: &Path) -> BTreeMap<String, Lines> {
    let mut stored = BTreeMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return stored;
    };
    for entry in entries {
        let path = entry.expect("a snapshot directory entry").path();
        if path.extension().is_some_and(|ext| ext == "txt") {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(&path).expect("a snapshot reads");
            stored.insert(name, parse(&text));
        }
    }
    stored
}

/// Every Build of the corpus projected, by name, on as many threads as the machine has.
fn current() -> BTreeMap<String, Lines> {
    let subjects = corpus();
    let threads = std::thread::available_parallelism().map_or(4, usize::from);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let projected = std::sync::Mutex::new(BTreeMap::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                let engine = support::engine();
                loop {
                    let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(subject) = subjects.get(index) else {
                        return;
                    };
                    let lines = project(&engine, &subject.build, subject.options);
                    projected
                        .lock()
                        .unwrap()
                        .insert(subject.name.clone(), lines);
                }
            });
        }
    });
    projected.into_inner().unwrap()
}

#[test]
fn snapshots_match_the_engine() {
    let dir = snapshot_dir();
    let current = current();
    match std::env::var("SNAPSHOTS").as_deref() {
        Ok("write") => {
            std::fs::create_dir_all(&dir).expect("the snapshot directory");
            for (name, _) in read_snapshots(&dir) {
                if !current.contains_key(&name) {
                    std::fs::remove_file(dir.join(format!("{name}.txt")))
                        .expect("a stale snapshot goes");
                }
            }
            for (name, lines) in &current {
                std::fs::write(dir.join(format!("{name}.txt")), render(lines))
                    .expect("a snapshot writes");
            }
        }
        Ok(other) => panic!("SNAPSHOTS={other}: the only value is `write`"),
        Err(_) => {
            let stored = read_snapshots(&dir);
            if let Some(message) = message::message(&stored, &current) {
                panic!("{message}");
            }
        }
    }
}

// ---- the message ----------------------------------------------------------------------------------------------

/// The failure message: what changed between the stored and the current lines, grouped by path.
mod message {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fmt::Write;

    use super::{Lines, REGENERATE};

    /// How many lines each grouping shows before saying how many more there are.
    const SHOWN: usize = 60;

    /// One Build's lines by path, with each `.open` list expanded into one path per key and no value
    /// (`slot.feat.5.general.0.open/power-attack`), so an option is one line.
    fn expand(lines: &Lines) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for (path, value) in lines {
            if path.ends_with(".open") {
                for key in value.split(", ").filter(|key| *key != "-") {
                    out.insert(format!("{path}/{key}"), String::new());
                }
            } else {
                out.insert(path.clone(), value.clone());
            }
        }
        out
    }

    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Change {
        Added(String),
        Removed(String),
        Changed(String, String),
    }

    impl Change {
        /// The change after its path: `+`, `- Bravery`, `4 → 5`.
        fn text(&self) -> String {
            let signed = |sign: &str, value: &str| {
                if value.is_empty() {
                    sign.to_owned()
                } else {
                    format!("{sign} {value}")
                }
            };
            match self {
                Change::Added(value) => signed("+", value),
                Change::Removed(value) => signed("-", value),
                Change::Changed(before, after) => format!("{before} → {after}"),
            }
        }

        /// The change with its path: `+ path`, `- path = value`, `path: 4 → 5`.
        fn with_path(&self, path: &str) -> String {
            let line = |sign: &str, value: &str| {
                if value.is_empty() {
                    format!("{sign} {path}")
                } else {
                    format!("{sign} {path} = {value}")
                }
            };
            match self {
                Change::Added(value) => line("+", value),
                Change::Removed(value) => line("-", value),
                Change::Changed(before, after) => format!("{path}: {before} → {after}"),
            }
        }
    }

    /// The path with its level or rank, its first number, wildcarded: `slot.feat.*.general.0.open/power-attack`.
    fn wildcard(path: &str) -> String {
        let (head, tail) = path.split_once('/').unwrap_or((path, ""));
        let mut done = false;
        let head: Vec<&str> = head
            .split('.')
            .map(|segment| {
                if !done && !segment.is_empty() && segment.bytes().all(|b| b.is_ascii_digit()) {
                    done = true;
                    "*"
                } else {
                    segment
                }
            })
            .collect();
        let head = head.join(".");
        if tail.is_empty() && !path.contains('/') {
            head
        } else {
            format!("{head}/{tail}")
        }
    }

    /// The first three names, and `…` after them when there are more.
    fn first_names<'a>(names: impl IntoIterator<Item = &'a String>) -> String {
        let names: Vec<&String> = names.into_iter().collect();
        let mut shown: Vec<&str> = names.iter().take(3).map(|name| name.as_str()).collect();
        if names.len() > 3 {
            shown.push("…");
        }
        shown.join(", ")
    }

    /// `1 Build`, `2 Builds`.
    fn count(count: usize, noun: &str) -> String {
        if count == 1 {
            format!("1 {noun}")
        } else {
            format!("{count} {noun}s")
        }
    }

    /// Writes up to [`SHOWN`] lines, then how many more there were.
    fn section(out: &mut String, title: &str, lines: Vec<String>) {
        writeln!(out, "\n{title}").unwrap();
        for line in lines.iter().take(SHOWN) {
            writeln!(out, "  {line}").unwrap();
        }
        if lines.len() > SHOWN {
            writeln!(out, "  … and {} more", lines.len() - SHOWN).unwrap();
        }
    }

    /// What differs between the stored snapshots and the current lines, by Build name; `None` when nothing does.
    pub(super) fn message(
        stored: &BTreeMap<String, Lines>,
        current: &BTreeMap<String, Lines>,
    ) -> Option<String> {
        let missing: Vec<&String> = current
            .keys()
            .filter(|name| !stored.contains_key(*name))
            .collect();
        let extra: Vec<&String> = stored
            .keys()
            .filter(|name| !current.contains_key(*name))
            .collect();
        // (path, change) → the Builds it's in; (wildcarded path, change) → the Builds and the paths it's at.
        let mut by_path: BTreeMap<(String, Change), BTreeSet<&String>> = BTreeMap::new();
        let mut by_shape: BTreeMap<(String, Change), (BTreeSet<&String>, usize)> = BTreeMap::new();
        let mut changed = 0;
        for (name, lines) in current {
            let Some(before) = stored.get(name) else {
                continue;
            };
            let (before, after) = (expand(before), expand(lines));
            let paths: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
            let mut differs = false;
            for path in paths {
                let change = match (before.get(path), after.get(path)) {
                    (Some(a), Some(b)) if a == b => continue,
                    (Some(a), Some(b)) => Change::Changed(a.clone(), b.clone()),
                    (None, Some(b)) => Change::Added(b.clone()),
                    (Some(a), None) => Change::Removed(a.clone()),
                    (None, None) => unreachable!(),
                };
                differs = true;
                by_path
                    .entry((path.clone(), change.clone()))
                    .or_default()
                    .insert(name);
                let shape = by_shape.entry((wildcard(path), change)).or_default();
                shape.0.insert(name);
                shape.1 += 1;
            }
            changed += usize::from(differs);
        }
        if missing.is_empty() && extra.is_empty() && by_path.is_empty() {
            return None;
        }
        let mut out = format!(
            "the engine's output differs from the snapshots in {} of {} Builds\n",
            changed,
            current.len()
        );
        if !by_path.is_empty() {
            let mut lines: Vec<(usize, String)> = by_path
                .iter()
                .map(|((path, change), names)| {
                    let line = format!(
                        "{} in {}: {}",
                        change.with_path(path),
                        count(names.len(), "Build"),
                        first_names(names.iter().copied())
                    );
                    (names.len(), line)
                })
                .collect();
            lines.sort_by_key(|(count, _)| std::cmp::Reverse(*count));
            section(
                &mut out,
                "Changes, each with the Builds it's in:",
                lines.into_iter().map(|(_, line)| line).collect(),
            );
            let mut lines: Vec<(usize, String)> = by_shape
                .iter()
                .map(|((path, change), (names, places))| {
                    let place = if path.contains(".open/") {
                        "slot"
                    } else {
                        "line"
                    };
                    let line = format!(
                        "{path}: {} in {}, {}",
                        change.text(),
                        count(names.len(), "Build"),
                        count(*places, place)
                    );
                    (*places, line)
                })
                .collect();
            lines.sort_by_key(|(count, _)| std::cmp::Reverse(*count));
            section(
                &mut out,
                "The same, levels and ranks wildcarded:",
                lines.into_iter().map(|(_, line)| line).collect(),
            );
        }
        if !missing.is_empty() {
            let names = missing
                .iter()
                .map(|name| format!("{name}.txt: no snapshot file (a new Build)"));
            section(&mut out, "Missing snapshot files:", names.collect());
        }
        if !extra.is_empty() {
            let names = extra
                .iter()
                .map(|name| format!("{name}.txt: no Build of the corpus"));
            section(&mut out, "Extra snapshot files:", names.collect());
        }
        writeln!(
            out,
            "\nIf every change is intended, regenerate the snapshots:\n{REGENERATE}"
        )
        .unwrap();
        Some(out)
    }

    // ---- the message's own tests --------------------------------------------------------------------------------

    fn lines(pairs: &[(&str, &str)]) -> Lines {
        pairs
            .iter()
            .map(|(path, value)| (path.to_string(), value.to_string()))
            .collect()
    }

    fn snapshots(builds: &[(&str, &[(&str, &str)])]) -> BTreeMap<String, Lines> {
        builds
            .iter()
            .map(|(name, pairs)| (name.to_string(), lines(pairs)))
            .collect()
    }

    #[test]
    fn snapshots_message_none_when_nothing_differs() {
        let same = snapshots(&[(
            "a",
            &[("level.1.bab", "1"), ("x.open", "dodge, power-attack")],
        )]);
        assert_eq!(message(&same, &same), None);
    }

    #[test]
    fn snapshots_message_groups_a_changed_value_by_build() {
        let stored = snapshots(&[
            ("a", &[("level.5.saves.will", "4")]),
            ("b", &[("level.5.saves.will", "4")]),
            ("c", &[("level.5.saves.will", "4")]),
            ("d", &[("level.5.saves.will", "4")]),
            ("e", &[("level.5.saves.will", "1")]),
        ]);
        let current = snapshots(&[
            ("a", &[("level.5.saves.will", "5")]),
            ("b", &[("level.5.saves.will", "5")]),
            ("c", &[("level.5.saves.will", "5")]),
            ("d", &[("level.5.saves.will", "5")]),
            ("e", &[("level.5.saves.will", "1")]),
        ]);
        let message = message(&stored, &current).unwrap();
        assert!(
            message
                .starts_with("the engine's output differs from the snapshots in 4 of 5 Builds\n")
        );
        assert!(
            message.contains("\n  level.5.saves.will: 4 → 5 in 4 Builds: a, b, c, …\n"),
            "{message}"
        );
        assert!(
            message.contains("\n  level.*.saves.will: 4 → 5 in 4 Builds, 4 lines\n"),
            "{message}"
        );
    }

    #[test]
    fn snapshots_message_expands_options_and_wildcards_levels() {
        let stored = snapshots(&[
            (
                "fighter",
                &[
                    ("slot.feat.1.general.0.open", "cleave, dodge"),
                    ("slot.feat.3.general.0.open", "cleave, dodge"),
                ],
            ),
            ("rogue", &[("slot.feat.1.general.0.open", "-")]),
        ]);
        let current = snapshots(&[
            (
                "fighter",
                &[
                    ("slot.feat.1.general.0.open", "dodge, power-attack"),
                    ("slot.feat.3.general.0.open", "dodge, power-attack"),
                ],
            ),
            ("rogue", &[("slot.feat.1.general.0.open", "power-attack")]),
        ]);
        let message = message(&stored, &current).unwrap();
        for line in [
            "  + slot.feat.1.general.0.open/power-attack in 2 Builds: fighter, rogue",
            "  + slot.feat.3.general.0.open/power-attack in 1 Build: fighter",
            "  - slot.feat.1.general.0.open/cleave in 1 Build: fighter",
            "  slot.feat.*.general.0.open/power-attack: + in 2 Builds, 3 slots",
            "  slot.feat.*.general.0.open/cleave: - in 1 Build, 2 slots",
        ] {
            assert!(
                message.contains(&format!("{line}\n")),
                "{line} in:\n{message}"
            );
        }
    }

    #[test]
    fn snapshots_message_names_added_and_removed_lines_with_their_values() {
        let stored = snapshots(&[("a", &[("level.2.feature/x", "X")])]);
        let current = snapshots(&[("a", &[("level.3.feature/y", "Y")])]);
        let message = message(&stored, &current).unwrap();
        assert!(
            message.contains("\n  - level.2.feature/x = X in 1 Build: a\n"),
            "{message}"
        );
        assert!(
            message.contains("\n  + level.3.feature/y = Y in 1 Build: a\n"),
            "{message}"
        );
        assert!(
            message.contains("\n  level.*.feature/y: + Y in 1 Build, 1 line\n"),
            "{message}"
        );
    }

    #[test]
    fn snapshots_message_names_missing_and_extra_files_and_ends_with_the_command() {
        let stored = snapshots(&[("a", &[("x", "1")]), ("gone", &[("x", "1")])]);
        let current = snapshots(&[("a", &[("x", "1")]), ("class-new", &[("x", "1")])]);
        let message = message(&stored, &current).unwrap();
        assert!(
            message.contains("\n  class-new.txt: no snapshot file (a new Build)\n"),
            "{message}"
        );
        assert!(
            message.contains("\n  gone.txt: no Build of the corpus\n"),
            "{message}"
        );
        assert_eq!(message.lines().last(), Some(REGENERATE));
    }

    #[test]
    fn snapshots_message_wildcards_only_the_level_or_rank() {
        assert_eq!(
            wildcard("slot.feat.5.general.0.open/power-attack"),
            "slot.feat.*.general.0.open/power-attack"
        );
        assert_eq!(
            wildcard("level.12.feat/weapon-focus(longsword)"),
            "level.*.feat/weapon-focus(longsword)"
        );
        assert_eq!(
            wildcard("rank.3.feature/azata/x2"),
            "rank.*.feature/azata/x2"
        );
        assert_eq!(wildcard("totals.skill.athletics"), "totals.skill.athletics");
    }
}
