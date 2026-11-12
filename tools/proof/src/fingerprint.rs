//! A Build's fingerprint: what the planner answers about it, hashed where it's long. Two sides that answer alike
//! give the same line.
//!
//! The fields, per Build:
//!
//! - `build`: its name;
//! - `who`, `levels`, `picks`: what it is (its classes, each with its archetype; its level and pick counts), for
//!   filtering a comparison;
//! - `possible`: [`Engine::is_possible`];
//! - `sheet`: the sheet ([`Engine::evaluate`]), its Debug text hashed (without a state's empty faith and empty option
//!   values, and without its bonuses in force, which `totals` and `caster_lines` read);
//! - `removable`: [`Engine::removable`], listed;
//! - `slots`: per slot of the sheet, by its id's Debug text: `options` (the picker's options with their statuses,
//!   hashed, without their edits: [`Hashed`]), `open` (how many are open), `total`, `kind`, and the lock-reason texts: `reasons` (those of options
//!   whose issue holds no `Not`, hashed with their keys) and `negated` (the others, listed in full: ticket 65 read
//!   each "Needs not" text, and the split stays so older records still compare);
//! - `classes`: the class picker at each level and one past the end, hashed (without the edits); `class_reasons`: its
//!   lock-reason texts, hashed;
//! - `paths`, `switches`: the path and switch pickers (Debug, without the edits); `path_reasons`, `switch_reasons`:
//!   their lock-reason texts, hashed;
//! - `allows`: [`Engine::allows`] for each edit of [`battery`], a `0` or `1` each (a light record has none, and
//!   `light: true` instead: see [`Mode`]);
//! - `totals`: [`Engine::totals`] at each level of the sheet, its Debug text hashed (ticket 151);
//! - `caster_lines`: [`Engine::caster_line`] at each level of the sheet for each spell of [`spells`], its Debug text
//!   hashed (ticket 151).
//!
//! Ticket 151 changed what the two lines take (a Build start and a state, then the state, then a level of the
//! sheet), and gave each state its bonuses in force (`CharacterState::bonuses_in_force`). The harness calls the lines
//! whichever way (`cfg(lines_take_start)`, `cfg(lines_take_state)`, set by its build script from the functions'
//! parameters), and the sheet hashes without the bonuses in force, so records from before it still compare.
//!
//! Recording also checks (not in a light record), panicking on a difference: [`Engine::is_removable`] and [`Engine::withdrawal`] against
//! `removable` for every choice the Build holds ([`Engine::held_choices`]) and some it doesn't,
//! [`Engine::removable_classes`] against `removable`'s class levels, each Build start and level option and step's
//! status against `allows` of its edit (ticket 96: the pickers share one walk where `allows` walks each), and, on
//! the Builds [`SHORTCUT_CHECKED`] names, ticket 48's picker shortcut against full walks.
//!
//! Ticket 96 gave every picker option its edit. The fingerprint hashes the options as they were before ([`Hashed`]:
//! the same Debug text), so records from before it still compare.
//!
//! Ticket 125 gave a mythic ability pick a value: every Debug text hashed writes one with no value as before
//! ([`as_before_ticket_125`]), so records from before it still compare. And `record --fold-values <key>,…` folds the
//! options of those items in each slot into one with no value, open when any value is ([`fold_values`]): a side
//! where they take a value lists one option per value where the other lists one.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use rules::{
    ArchetypeKey, Build, Choice, ChoiceOption, ClassKey, Edit, Energy, Engine, Issue,
    MythicPathKey, Pick, SlotId, SlotKind, SlotOption, SpellSchool, SpellTraits, Stepper,
    Subschool,
};
use serde_json::{Value, json};

/// The FNV-1a hash of a text.
pub fn fnv(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// A value's Debug text, hashed (as before ticket 125: [`as_before_ticket_125`]).
fn hashed(value: &impl std::fmt::Debug) -> String {
    format!("{:016x}", fnv(&as_before_ticket_125(&format!("{value:?}"))))
}

/// A Debug text with each mythic ability pick taking no value written as before ticket 125:
/// `MythicAbility { ability: MythicAbilityKey("k"), parameter: None }` as `MythicAbility(MythicAbilityKey("k"))`.
fn as_before_ticket_125(text: &str) -> String {
    const NEW: &str = "MythicAbility { ability: ";
    const NO_VALUE: &str = ", parameter: None }";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(NEW) {
        out.push_str(&rest[..at]);
        let after = &rest[at + NEW.len()..];
        // The key, `MythicAbilityKey("…")`: a key holds no bracket.
        match after.find(')').map(|end| end + 1) {
            Some(end) if after[end..].starts_with(NO_VALUE) => {
                out.push_str("MythicAbility(");
                out.push_str(&after[..end]);
                out.push(')');
                rest = &after[end + NO_VALUE.len()..];
            }
            _ => {
                out.push_str(NEW);
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The keys whose options a slot's picker folds into one (`record --fold-values`).
static FOLDED: OnceLock<BTreeSet<String>> = OnceLock::new();

/// Folds, in every slot's options, the options of these keys into one each (see the module docs). Set once, before
/// recording.
pub fn fold_values(keys: BTreeSet<String>) {
    FOLDED
        .set(keys)
        .expect("the folded keys are set once, before recording");
}

/// A slot's options with the options of each folded key as one: the first, with no value, open when any of them is.
fn folded(options: Vec<SlotOption>) -> Vec<SlotOption> {
    let Some(keys) = FOLDED.get().filter(|keys| !keys.is_empty()) else {
        return options;
    };
    let mut out: Vec<SlotOption> = Vec::with_capacity(options.len());
    for option in options {
        if !keys.contains(option.pick.key()) {
            out.push(option);
            continue;
        }
        let pick = without_value(&option.pick);
        match out.last_mut() {
            Some(last) if last.pick == pick => {
                if option.status.is_ok() {
                    last.status = Ok(());
                }
            }
            _ => {
                let mut option = option;
                option.pick = pick;
                out.push(option);
            }
        }
    }
    out
}

/// A pick without its value, through serde so the harness compiles against every pick shape.
fn without_value(pick: &Pick) -> Pick {
    let mut value = serde_json::to_value(pick).expect("a pick serialises");
    if let Some(fields) = value
        .as_object_mut()
        .and_then(|variant| variant.values_mut().next())
        .and_then(Value::as_object_mut)
        .filter(|fields| fields.contains_key("parameter"))
    {
        fields.insert("parameter".to_owned(), Value::Null);
    }
    serde_json::from_value(value).expect("a pick deserialises")
}

/// The picker options as the fingerprint hashes them: what they were before ticket 96 gave each its edit, with the
/// same Debug text (each mirror is named and laid out as the option it stands for), so older records still compare.
mod hashed {
    use super::{ArchetypeKey, ClassKey, Issue, MythicPathKey, Pick};

    #[derive(Debug)]
    pub struct SlotOption<'a> {
        pub pick: &'a Pick,
        pub status: &'a Result<(), Issue>,
    }

    #[derive(Debug)]
    pub struct ClassOption<'a> {
        pub class: &'a ClassKey,
        pub archetype: &'a Option<ArchetypeKey>,
        pub status: &'a Result<(), Issue>,
    }

    #[derive(Debug)]
    pub struct PathOption<'a> {
        pub path: &'a MythicPathKey,
        pub status: &'a Result<(), Issue>,
    }
}

/// The options of a slot, class or path picker without their edits (see [`hashed`]).
pub trait Hashed {
    type Mirror<'a>: std::fmt::Debug
    where
        Self: 'a;
    fn hashed(&self) -> Self::Mirror<'_>;
}

impl Hashed for SlotOption {
    type Mirror<'a> = hashed::SlotOption<'a>;
    fn hashed(&self) -> hashed::SlotOption<'_> {
        hashed::SlotOption {
            pick: &self.pick,
            status: &self.status,
        }
    }
}

impl Hashed for rules::ClassOption {
    type Mirror<'a> = hashed::ClassOption<'a>;
    fn hashed(&self) -> hashed::ClassOption<'_> {
        hashed::ClassOption {
            class: &self.class,
            archetype: &self.archetype,
            status: &self.status,
        }
    }
}

impl Hashed for rules::PathOption {
    type Mirror<'a> = hashed::PathOption<'a>;
    fn hashed(&self) -> hashed::PathOption<'_> {
        hashed::PathOption {
            path: &self.path,
            status: &self.status,
        }
    }
}

/// A picker's options (if any) without their edits, for hashing.
fn mirrors<T: Hashed>(options: Option<&Vec<T>>) -> Option<Vec<T::Mirror<'_>>> {
    options.map(|options| options.iter().map(Hashed::hashed).collect())
}

/// The name prefixes of the Builds whose feat pickers are checked against full walks (ticket 48's shortcut), each
/// also with `full ` before it. The Builds added since ticket 55 (the checks are slow, so not on every Build).
pub const SHORTCUT_CHECKED: [&str; 13] = [
    "t55 ", "t57 ", "t58 ", "t59 ", "t60 ", "t61 ", "t62 ", "t63 ", "t64 ", "t65 ", "t66 ", "t67 ",
    "t86 ",
];

static SHORTCUT_BUILDS: AtomicUsize = AtomicUsize::new(0);
static SHORTCUT_OPTIONS: AtomicUsize = AtomicUsize::new(0);

/// How many Builds and feat options the shortcut checks covered so far, for the log.
pub fn shortcut_counts() -> (usize, usize) {
    (
        SHORTCUT_BUILDS.load(Ordering::Relaxed),
        SHORTCUT_OPTIONS.load(Ordering::Relaxed),
    )
}

static HELD: Mutex<BTreeMap<String, usize>> = Mutex::new(BTreeMap::new());

/// How many held choices the consistency checks covered so far, by the kind of choice (`Deity`, `Slot`…), for the
/// log.
pub fn held_counts() -> BTreeMap<String, usize> {
    HELD.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// How much of a fingerprint a record holds (ticket 162).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Every field, after the consistency and shortcut checks.
    Full,
    /// Without the `allows` battery and without the consistency and shortcut checks: about a third of a record's cost.
    /// Each line says so (`"light": true`), so `compare` refuses a light record against a full one.
    Light,
}

/// The Build's fingerprint (see the module docs), after its consistency checks unless the mode is light.
pub fn fingerprint(engine: &Engine, name: &str, build: &Build, mode: Mode) -> Value {
    let light = mode == Mode::Light;
    let sheet = engine.evaluate(build);
    let removable = engine.removable(build);
    let unfull = name.strip_prefix("full ").unwrap_or(name);
    let shortcut_checked = !light
        && SHORTCUT_CHECKED
            .iter()
            .any(|prefix| unfull.starts_with(prefix));
    // The feat slots' options, unfolded, for the shortcut checks: the walks the slots below already pay.
    let mut feat_options: Vec<(&SlotId, Vec<SlotOption>)> = Vec::new();
    let mut slots = BTreeMap::new();
    for slot in sheet
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
    {
        let options = engine.options(build, &slot.id);
        if shortcut_checked && matches!(slot.kind, SlotKind::Feat(_)) {
            feat_options.push((&slot.id, options.clone().unwrap_or_default()));
        }
        let options = options.map(folded);
        let open = options
            .iter()
            .flatten()
            .filter(|option| option.status.is_ok())
            .count();
        let total = options.as_ref().map_or(0, Vec::len);
        let mut reasons = Vec::new();
        let mut negated = Vec::new();
        for option in options.iter().flatten() {
            let Err(issue) = &option.status else { continue };
            let text = engine.issue_text(issue);
            if format!("{issue:?}").contains("Not(") {
                negated.push(format!(
                    "{}: {}",
                    option.pick.key(),
                    text.unwrap_or_default()
                ));
            } else {
                reasons.push((option.pick.key().to_owned(), text));
            }
        }
        slots.insert(
            format!("{:?}", slot.id),
            json!({
                "options": hashed(&mirrors(options.as_ref())),
                "open": open,
                "total": total,
                "kind": format!("{:?}", slot.kind),
                "reasons": hashed(&reasons),
                "negated": negated,
            }),
        );
    }
    let past_end = u8::try_from(build.levels.len() + 1).expect("at most 254 levels");
    // Each level's class picker once: the hashed options and their lock reasons both read it.
    let class_options: Vec<Option<Vec<rules::ClassOption>>> = (1..=past_end)
        .map(|level| engine.class_options(build, level))
        .collect();
    let classes: Vec<String> = class_options
        .iter()
        .map(|options| hashed(&mirrors(options.as_ref())))
        .collect();
    let reason = |status: &Result<(), Issue>| {
        status
            .as_ref()
            .err()
            .and_then(|issue| engine.issue_text(issue))
    };
    let class_reasons: Vec<Vec<Option<String>>> = class_options
        .iter()
        .map(|options| {
            options
                .iter()
                .flatten()
                .map(|option| reason(&option.status))
                .collect()
        })
        .collect();
    let paths = engine.path_options(build);
    let switches = engine.switch_options(build);
    let path_reasons: Vec<Option<String>> =
        paths.iter().map(|option| reason(&option.status)).collect();
    let switch_reasons: Vec<Option<String>> = switches
        .iter()
        .map(|option| reason(&option.status))
        .collect();
    // The battery and the checks: about a third of a full record's cost, and left out of a light one.
    let allows = (!light).then(|| {
        let controls = controls(engine, build);
        let edits = battery(engine, build, &controls);
        let allows: Vec<bool> = edits
            .iter()
            .map(|edit| engine.allows(build, edit))
            .collect();
        consistency_checks(engine, build, &removable, &controls, &allows);
        if shortcut_checked {
            shortcut_checks(engine, build, feat_options);
        }
        allows
            .iter()
            .map(|allowed| if *allowed { '1' } else { '0' })
            .collect::<String>()
    });

    let mut record = json!({
        "build": name,
        "who": who(build),
        "levels": build.levels.len(),
        "picks": build.picks.len(),
        "possible": engine.is_possible(build),
        "sheet": sheet_hash(&sheet),
        "removable": removable.iter().map(|choice| format!("{choice:?}")).collect::<Vec<_>>(),
        "slots": slots,
        "classes": classes,
        "class_reasons": hashed(&class_reasons),
        "paths": format!("{:?}", mirrors(Some(&paths)).unwrap_or_default()),
        "path_reasons": hashed(&path_reasons),
        "switches": format!("{:?}", mirrors(Some(&switches)).unwrap_or_default()),
        "switch_reasons": hashed(&switch_reasons),
        "totals": totals_hash(engine, build, &sheet),
        "caster_lines": caster_lines_hash(engine, build, &sheet),
    });
    // A full record keeps its fields and their order as before (`allows` sorts among them: the maps are ordered).
    match allows {
        Some(allows) => record["allows"] = Value::String(allows),
        None => record["light"] = Value::Bool(true),
    }
    record
}

/// Each level's totals, their Debug texts hashed together. The "leaves out" texts aren't hashed: they are wording,
/// which ticket 166 rewrote for every Build, and hashing them hid whether any number had changed.
fn totals_hash(engine: &Engine, build: &Build, sheet: &rules::Sheet) -> String {
    let totals: Vec<rules::Totals> = sheet
        .levels
        .iter()
        .map(|level| {
            let mut totals = totals(engine, build, level);
            totals.leaves_out = &[];
            totals
        })
        .collect();
    hashed(&totals)
}

/// Each level's caster line for each spell of [`spells`], their Debug texts hashed together, without the "leaves
/// out" texts (as [`totals_hash`]).
fn caster_lines_hash(engine: &Engine, build: &Build, sheet: &rules::Sheet) -> String {
    let spells = spells();
    let lines: Vec<Option<rules::CasterLine>> = sheet
        .levels
        .iter()
        .flat_map(|level| {
            spells.iter().map(|spell| {
                let mut line = caster_line(engine, build, level, *spell);
                if let Some(line) = &mut line {
                    line.leaves_out = &[];
                }
                line
            })
        })
        .collect();
    hashed(&lines)
}

/// [`Engine::totals`] at a level of the Build's sheet, whatever the `rules` beside the harness takes (its build
/// script: before ticket 151, the Build start and the level's state; its first commits, the state; then the level).
fn totals(engine: &Engine, build: &Build, level: &rules::LevelSheet) -> rules::Totals {
    // The Build is read only before ticket 151.
    let _ = build;
    #[cfg(lines_take_start)]
    return engine.totals(&build.start, &level.state);
    #[cfg(lines_take_state)]
    return engine.totals(&level.state);
    #[cfg(not(any(lines_take_start, lines_take_state)))]
    engine.totals(level)
}

/// [`Engine::caster_line`] at a level of the Build's sheet, whatever the `rules` beside the harness takes (as
/// [`totals`]).
fn caster_line(
    engine: &Engine,
    build: &Build,
    level: &rules::LevelSheet,
    spell: SpellTraits,
) -> Option<rules::CasterLine> {
    let _ = build;
    #[cfg(lines_take_start)]
    return engine.caster_line(&build.start, &level.state, spell);
    #[cfg(lines_take_state)]
    return engine.caster_line(&level.state, spell);
    #[cfg(not(any(lines_take_start, lines_take_state)))]
    engine.caster_line(level, spell)
}

/// The spells the caster lines are recorded for: one of each school, one of each subschool, one of each energy type
/// (an evocation) and one with metamagic, so every bonus scope of the data is met.
fn spells() -> Vec<SpellTraits> {
    let plain = |school| SpellTraits {
        school,
        subschool: None,
        energy: None,
        metamagic: false,
    };
    let mut spells: Vec<SpellTraits> = SpellSchool::ALL.into_iter().map(plain).collect();
    spells.extend(Subschool::ALL.into_iter().map(|subschool| SpellTraits {
        subschool: Some(subschool),
        ..plain(subschool.school())
    }));
    spells.extend(Energy::ALL.into_iter().map(|energy| SpellTraits {
        energy: Some(energy),
        ..plain(SpellSchool::Evocation)
    }));
    spells.push(SpellTraits {
        metamagic: true,
        ..plain(SpellSchool::Evocation)
    });
    spells
}

/// The sheet's Debug text, hashed, without a state's empty faith (ticket 86's `faith: None, deity_domains: {}`): a
/// Build without one walks to the same sheet as on a side before deities, whose states have no such fields. And
/// without a state's bonuses in force (ticket 151), which a side before them doesn't have, nor its empty option values
/// (ticket 172's `option_values: {}`), for the same reason: a state that holds one differs.
fn sheet_hash(sheet: &rules::Sheet) -> String {
    format!("{:016x}", fnv(&sheet_text(&format!("{sheet:?}"))))
}

/// A sheet's (or a state's) Debug text as [`sheet_hash`] hashes it.
fn sheet_text(debug: &str) -> String {
    let text = as_before_ticket_125(debug)
        .replace("faith: None, deity_domains: {}, ", "")
        .replace("option_values: {}, ", "");
    without_bonuses_in_force(&text)
}

/// A Debug text without its states' bonuses in force: each `, bonuses_in_force: […]` left out.
fn without_bonuses_in_force(text: &str) -> String {
    const FIELD: &str = ", bonuses_in_force: [";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(FIELD) {
        out.push_str(&rest[..at]);
        let list = &rest[at + FIELD.len()..];
        rest = &list[list_end(list)..];
    }
    out.push_str(rest);
    out
}

/// Where a Debug list opened just before `text` ends: just past its `]`, skipping the brackets of the lists in it
/// and of quoted text.
fn list_end(text: &str) -> usize {
    let (mut depth, mut quoted, mut escaped) = (1_usize, false, false);
    for (at, c) in text.char_indices() {
        if quoted {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => quoted = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return at + 1;
                }
            }
            _ => {}
        }
    }
    text.len()
}

/// The Build's classes, each with its archetype ("slayer:slayer/arcane-enforcer").
fn who(build: &Build) -> Vec<String> {
    build
        .class_choices
        .iter()
        .map(|(class, choice)| match &choice.archetype {
            Some(archetype) => format!("{class}:{archetype}"),
            None => class.to_string(),
        })
        .collect()
}

/// Every option and step of the Build start's and the levels' pickers and steppers, in order, each with its status
/// and edit: race and subrace, faith, alignment, background, the racial ability, the point buy's steps, then each
/// level's ability increase and skill steps (ticket 96: the engine's, so a new start choice is covered here without
/// editing the proof).
pub fn controls(engine: &Engine, build: &Build) -> Vec<(Result<(), Issue>, Edit)> {
    fn options<T>(
        options: Vec<ChoiceOption<T>>,
    ) -> impl Iterator<Item = (Result<(), Issue>, Edit)> {
        options
            .into_iter()
            .map(|option| (option.status, option.edit))
    }
    fn steps<T>(steppers: Vec<Stepper<T>>) -> impl Iterator<Item = (Result<(), Issue>, Edit)> {
        steppers
            .into_iter()
            .flat_map(|stepper| [stepper.down, stepper.up])
            .flatten()
            .map(|step| (step.status, step.edit))
    }
    let mut controls: Vec<(Result<(), Issue>, Edit)> = Vec::new();
    controls.extend(options(engine.race_options(build)));
    controls.extend(options(engine.faith_options(build)));
    controls.extend(options(engine.alignment_options(build)));
    controls.extend(options(engine.background_options(build)));
    controls.extend(options(engine.racial_ability_options(build)));
    controls.extend(steps(engine.point_buy_steps(build)));
    for index in 0..build.levels.len() {
        let level = u8::try_from(index + 1).expect("at most 255 levels");
        let increases = engine.ability_increase_options(build, level);
        controls.extend(options(increases.unwrap_or_default()));
        controls.extend(steps(engine.skill_steps(build, level).unwrap_or_default()));
    }
    controls
}

/// The edits whose `allows` the fingerprint holds: every option and step of the start and level pickers
/// ([`controls`], passed in: each of their statuses costs a walk, which the fingerprint has already paid), each
/// first ascension, each held choice's withdrawal ([`Engine::held_choices`]), and the edits past the end.
pub fn battery(
    engine: &Engine,
    build: &Build,
    controls: &[(Result<(), Issue>, Edit)],
) -> Vec<Edit> {
    let mut edits: Vec<Edit> = controls.iter().map(|(_, edit)| edit.clone()).collect();
    edits.extend(
        engine
            .first_ascension_options(build)
            .into_iter()
            .map(|option| option.edit),
    );
    edits.extend(
        engine
            .held_choices(build)
            .into_iter()
            .map(|withdrawal| withdrawal.edit),
    );
    edits.push(Edit::RemoveLevel);
    edits.push(Edit::SetName(Some("x".into())));
    edits
}

/// Some choices a Build may not hold, for `is_removable` and `withdrawal` beside the held ones: every Build start
/// choice and Mythic one, each level's ability increase, skill points, archetype and advanced classes, and the class
/// and the character level one past the end.
fn unheld_choices(build: &Build) -> Vec<Choice> {
    let mut choices = vec![
        Choice::Race,
        Choice::Subrace,
        Choice::Alignment,
        Choice::Background,
        Choice::Deity,
        Choice::FirstAscension,
        Choice::MythicPath,
        Choice::PathSwitch,
    ];
    for (index, level_up) in build.levels.iter().enumerate() {
        let level = u8::try_from(index + 1).expect("at most 255 levels");
        choices.push(Choice::AbilityIncrease { level });
        choices.push(Choice::SkillPoints { level });
        choices.push(Choice::CharacterLevel { level });
        choices.push(Choice::Archetype {
            class: level_up.class.clone(),
        });
        choices.push(Choice::Advances {
            class: level_up.class.clone(),
        });
    }
    let past = u8::try_from(build.levels.len() + 1).expect("at most 254 levels");
    choices.push(Choice::Class { level: past });
    choices.push(Choice::CharacterLevel { level: past });
    choices
}

/// The shortcuts against the full answers they stand for: `is_removable` against `removable`, `removable_classes`
/// against `removable`'s class levels, and each start and level option and step's status against `allows` of its
/// edit (the battery's first edits are theirs): open exactly when allowed, unless a faith and alignment that don't
/// fit lock it. Panics on a difference.
fn consistency_checks(
    engine: &Engine,
    build: &Build,
    removable: &BTreeSet<Choice>,
    controls: &[(Result<(), Issue>, Edit)],
    allows: &[bool],
) {
    let held = engine.held_choices(build);
    for withdrawal in &held {
        let choice = &withdrawal.choice;
        let removable = removable.contains(choice);
        assert_eq!(engine.is_removable(build, choice), removable, "{choice:?}");
        assert_eq!(
            engine.withdrawal(build, choice),
            Some((withdrawal.edit.clone(), removable)),
            "{choice:?}"
        );
    }
    let unheld = unheld_choices(build);
    for choice in unheld
        .iter()
        .filter(|choice| !held.iter().any(|held| &held.choice == *choice))
    {
        assert!(!engine.is_removable(build, choice), "{choice:?}");
        assert_eq!(engine.withdrawal(build, choice), None, "{choice:?}");
    }
    assert!(
        removable
            .iter()
            .all(|choice| held.iter().any(|held| &held.choice == choice)),
        "a removable choice the Build doesn't hold"
    );
    let mut counts = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    for withdrawal in &held {
        let debug = format!("{:?}", withdrawal.choice);
        let kind: String = debug
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect();
        *counts.entry(kind).or_default() += 1;
    }
    drop(counts);
    let levels: BTreeSet<u8> = removable
        .iter()
        .filter_map(|choice| match choice {
            Choice::Class { level } => Some(*level),
            _ => None,
        })
        .collect();
    assert_eq!(engine.removable_classes(build), levels);
    for ((status, edit), allowed) in controls.iter().zip(allows) {
        match status {
            Ok(()) | Err(Issue::BreaksLater) => assert_eq!(status.is_ok(), *allowed, "{edit:?}"),
            Err(Issue::NeedsFaith | Issue::DeityAlignment { .. }) => {}
            Err(issue) => panic!("{issue:?} for {edit:?}"),
        }
    }
}

/// Ticket 48's picker shortcut, checked: every feat a feat slot accepts is open, not `BreaksLater`, exactly when
/// `allows` keeps the Build with it picked there (a full walk of the later choices; the shortcut gives a feat no later
/// check names the slot-left-empty answer). `feat_options` is each feat slot's options, in the sheet's order. Panics
/// on a difference.
fn shortcut_checks(engine: &Engine, build: &Build, feat_options: Vec<(&SlotId, Vec<SlotOption>)>) {
    for (slot, options) in feat_options {
        for option in options {
            // Only a feat the slot accepts: the shortcut answers whether it breaks a later choice.
            if !matches!(option.pick, Pick::Feat { .. })
                || !matches!(option.status, Ok(()) | Err(Issue::BreaksLater))
            {
                continue;
            }
            SHORTCUT_OPTIONS.fetch_add(1, Ordering::Relaxed);
            let allowed = engine.allows(
                build,
                &Edit::Pick {
                    slot: slot.clone(),
                    pick: option.pick.clone(),
                },
            );
            assert_eq!(
                option.status.is_ok(),
                allowed,
                "{:?} {:?}: {:?}",
                slot,
                option.pick,
                option.status
            );
        }
    }
    SHORTCUT_BUILDS.fetch_add(1, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rules::{CharacterState, Parameter, SelectionOptionKey, WeaponKey};

    use super::sheet_text;

    /// Ticket 172's strip: a state's empty `option_values` is left out of the text the sheet hashes, as a side before
    /// the field has none; one that holds a value stays.
    #[test]
    fn the_sheet_text_drops_an_empty_option_values_and_keeps_a_held_one() {
        let empty = format!("{:?}", CharacterState::default());
        assert!(empty.contains("option_values: {}, "), "{empty}");
        let text = sheet_text(&empty);
        assert!(!text.contains("option_values"), "{text}");
        // Only the field is gone: the fields either side of it stay.
        assert!(
            text.contains("options_through: {}, option_groups: {}"),
            "{text}"
        );

        let mut held = CharacterState::default();
        held.option_values.insert(
            SelectionOptionKey::new("rogue/finesse-training/weapon"),
            BTreeMap::from([(Parameter::Weapon(WeaponKey::new("dagger")), 1)]),
        );
        let text = sheet_text(&format!("{held:?}"));
        assert!(
            text.contains("option_values: {SelectionOptionKey("),
            "{text}"
        );
        assert!(text.contains("WeaponKey(\"dagger\")"), "{text}");
    }
}
