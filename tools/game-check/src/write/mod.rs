//! The importer's writer (ticket 293, stage S1): it brings the game data's text to the game's values and keeps
//! everything else (importer design §1.2).
//!
//! The data's text is the planner's: keys, names, summaries, guide links, comments, field order and line style. The
//! writer never re-serialises a file. It finds one top-level field of an entry by its span and replaces, inserts or
//! removes only that, so the text outside the span is byte-identical and the comments survive ([`span`]).
//!
//! The scanner reads `//` comments and `"` strings. It does not read `/* */` comments (a text holding `/*` is
//! refused), raw strings, character literals or CRLF line ends; none is in `libs/game-data/` today.
//!
//! [`apply`] writes a list of [`Change`]s to a data folder: it refuses a file under `planner/`, a missing one, a
//! field the importer doesn't own, and a refused span before it writes anything; then it writes each file, reads it
//! back, runs the caller's check of the whole folder, and on any failure puts every written file's bytes back.

pub mod facts;
pub mod items;
pub mod log;
pub mod selections;
pub mod span;
pub mod tables;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rules::{DataAt, GameData};

use span::{ARCHETYPE_FIELDS, CLASS_FIELDS, GEAR_FIELDS, SELECTION_FIELDS, SpanError, Value};

/// One field of one entry's file to write, or to remove when `value` is `None`.
#[derive(Clone, Debug)]
pub struct Change {
    pub at: DataAt,
    /// The entry's file, relative to the data folder ([`entry_file`]).
    pub file: PathBuf,
    pub field: &'static str,
    /// The check's finding kind this change answers, for the registry (293e).
    pub kind: &'static str,
    pub value: Option<Value>,
    /// The field's old and new values in normal form, for the log.
    pub old: String,
    pub new: String,
}

/// An item the importer left unwritten because the game gives a bonus the planner has no word for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unexpressed {
    pub at: DataAt,
    pub what: String,
}

/// What [`apply`] wrote: the files whose text changed, in path order.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Applied {
    pub files: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum WriteError {
    /// A change on a planner-made file, which the importer never writes.
    Planner(PathBuf),
    Missing(PathBuf),
    /// A change to a field the importer doesn't own in that folder (design §1.2's table).
    NotOwned {
        file: PathBuf,
        field: &'static str,
    },
    Span {
        file: PathBuf,
        field: &'static str,
        error: SpanError,
    },
    /// The file as written differs from the file before in more than the written field.
    ReadBack {
        file: PathBuf,
        field: &'static str,
    },
    /// The caller's check of the written folder failed.
    Invalid(String),
    Io {
        file: PathBuf,
        error: io::Error,
    },
    /// Lines of the registry of the user's facts that name an entry the data doesn't have.
    StaleFacts(Vec<String>),
    /// The run failed (`cause`) and putting the files back failed too: these files are left as written.
    RestoreFailed {
        cause: Box<WriteError>,
        files: Vec<PathBuf>,
    },
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::Planner(file) => {
                write!(
                    f,
                    "{}: a planner-made file is never written",
                    file.display()
                )
            }
            WriteError::Missing(file) => write!(f, "{}: no such entry file", file.display()),
            WriteError::NotOwned { file, field } => write!(
                f,
                "{}: `{field}` is not a field the importer writes in this folder",
                file.display()
            ),
            WriteError::Span { file, field, error } => {
                write!(f, "{}: `{field}`: {error}", file.display())
            }
            WriteError::ReadBack { file, field } => write!(
                f,
                "{}: after writing `{field}` the file differs in more than the written field",
                file.display()
            ),
            WriteError::Invalid(why) => write!(f, "the written data fails its check: {why}"),
            WriteError::Io { file, error } => write!(f, "{}: {error}", file.display()),
            WriteError::StaleFacts(lines) => write!(
                f,
                "user-facts.txt names entries the data doesn't have: {}",
                lines.join("; ")
            ),
            WriteError::RestoreFailed { cause, files } => {
                let files: Vec<_> = files
                    .iter()
                    .map(|file| file.display().to_string())
                    .collect();
                write!(
                    f,
                    "{cause}; and these files could not be put back and are left as written: {}",
                    files.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for WriteError {}

/// The entry's file under the data folder: the folder of its kind (`prestige/` for a prestige class), then
/// `<key>.ron`; the mythic ascension table and the skills are single files.
///
/// It does not detect a planner-made entry, whose file is `planner/<folder>/<key>.ron`: it has no blueprint, so the
/// importer never reaches it by a blueprint, and a caller that holds such an entry passes the path it knows.
/// [`apply`] refuses any `planner/` path.
pub fn entry_file(data: &GameData, at: &DataAt) -> PathBuf {
    let folder = match at {
        DataAt::Progression => return PathBuf::from("progression.ron"),
        DataAt::Skill(_) => return PathBuf::from("skills.ron"),
        DataAt::Race(_) => "races",
        DataAt::Subrace(_) => "subraces",
        DataAt::Feat(_) => "feats",
        DataAt::Class(key) if data.prestige_classes.iter().any(|class| class.key == *key) => {
            "prestige"
        }
        DataAt::Class(_) => "classes",
        DataAt::Archetype(_) => "archetypes",
        DataAt::Feature(_) => "features",
        DataAt::FeatureSelection(_) => "selections",
        DataAt::SelectionOption(_) => "options",
        DataAt::MythicAbility(_) => "mythic/abilities",
        DataAt::MythicFeat(_) => "mythic/feats",
        DataAt::MythicPath(_) => "mythic/paths",
        DataAt::Background(_) => "backgrounds",
        DataAt::Deity(_) => "deities",
        DataAt::Weapon(_) => "weapons",
        DataAt::Spell(_) => "spells",
        DataAt::SpellList(_) => "spell-lists",
        DataAt::Gear(_) => "gear",
    };
    let key = at.key().expect("every kind but the progression has a key");
    Path::new(folder).join(format!("{key}.ron"))
}

/// The declared field order of the kind a file is of, for an insert's anchor.
fn declared(file: &Path) -> &'static [&'static str] {
    match folder(file) {
        Some("classes" | "prestige") => CLASS_FIELDS,
        Some("archetypes") => ARCHETYPE_FIELDS,
        Some("gear") => GEAR_FIELDS,
        Some("selections") => SELECTION_FIELDS,
        _ => &[],
    }
}

/// The fields the importer writes in a folder (design §1.2's table): nothing else is changed, and a folder with no
/// list here has none, until a later slice gives it one.
fn owned(file: &Path) -> &'static [&'static str] {
    match folder(file) {
        Some("classes" | "prestige") => &[
            "skill_points",
            "bab",
            "fort",
            "reflex",
            "will",
            "save_exceptions",
            "class_skills",
            "levels",
            "components",
        ],
        Some("archetypes") => &[
            "skill_points",
            "class_skills",
            "bab",
            "fort",
            "reflex",
            "will",
            "levels",
            "components",
        ],
        Some("gear") => &["components"],
        Some("selections") => &[
            "options",
            "shared_options",
            "feats",
            "mythic_abilities",
            "ignores_prerequisites",
        ],
        _ => &[],
    }
}

fn folder(file: &Path) -> Option<&str> {
    file.components()
        .next()
        .and_then(|part| part.as_os_str().to_str())
}

/// Writes `text` to `path` through a temp file in the same folder and a rename over it, so a kill in the middle
/// never leaves a truncated file.
pub fn atomic_write(path: &Path, text: &str) -> io::Result<()> {
    let name = path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let temp = path.with_file_name(format!(".{name}.tmp"));
    fs::write(&temp, text)?;
    fs::rename(&temp, path).inspect_err(|_| {
        let _ = fs::remove_file(&temp);
    })
}

/// One file's plan: its text before and after, and each change's step (the text before it, and after it).
struct FilePlan<'a> {
    file: PathBuf,
    before: String,
    after: String,
    steps: Vec<(String, String, &'a Change)>,
}

/// Writes the changes under `dir`.
///
/// Every file is checked and every new text built before any is written: a file under `planner/`, a missing one, a
/// field outside [`owned`] or a refused span stops the run with nothing written. A file's changes apply in the order
/// given, so of two changes to one field the later wins. After writing, each file is read back from the disk: it
/// must hold the text built, and each change's step must differ from the text before it only in its field and the
/// separators the writer adds or drops with it ([`read_back`]). Then `validate` runs on `dir`. A failure of either
/// puts every written file's bytes back; one that can't be put back is named in [`WriteError::RestoreFailed`].
///
/// The set of files is not atomic: a kill between two files leaves the folder half written, and git is the
/// recovery. Each file is replaced atomically ([`atomic_write`]), so none is left truncated.
pub fn apply(
    dir: &Path,
    changes: &[Change],
    validate: impl Fn(&Path) -> Result<(), String>,
) -> Result<Applied, WriteError> {
    apply_with(dir, changes, validate, atomic_write)
}

/// [`apply`] with the function that writes a file, so a test can make a write fail or land wrong. Restoring always
/// uses [`atomic_write`].
pub fn apply_with(
    dir: &Path,
    changes: &[Change],
    validate: impl Fn(&Path) -> Result<(), String>,
    write: impl Fn(&Path, &str) -> io::Result<()>,
) -> Result<Applied, WriteError> {
    let plan = plan_files(dir, changes)?;

    let mut written: Vec<usize> = Vec::new();
    let result = (|| {
        for (at, file) in plan.iter().enumerate() {
            if file.before == file.after {
                continue;
            }
            written.push(at);
            let path = dir.join(&file.file);
            write(&path, &file.after).map_err(|error| WriteError::Io {
                file: file.file.clone(),
                error,
            })?;
            let first = file.steps.first().map_or("", |step| step.2.field);
            match fs::read_to_string(&path) {
                Ok(disk) if disk == file.after => {}
                Ok(_) => {
                    return Err(WriteError::ReadBack {
                        file: file.file.clone(),
                        field: first,
                    });
                }
                Err(error) => {
                    return Err(WriteError::Io {
                        file: file.file.clone(),
                        error,
                    });
                }
            }
            for (before, after, change) in &file.steps {
                read_back(&file.file, before, after, change)?;
            }
        }
        validate(dir).map_err(WriteError::Invalid)
    })();
    if let Err(cause) = result {
        let failed: Vec<PathBuf> = written
            .into_iter()
            .filter(|at| atomic_write(&dir.join(&plan[*at].file), &plan[*at].before).is_err())
            .map(|at| plan[at].file.clone())
            .collect();
        return Err(if failed.is_empty() {
            cause
        } else {
            WriteError::RestoreFailed {
                cause: Box::new(cause),
                files: failed,
            }
        });
    }
    Ok(Applied {
        files: plan
            .into_iter()
            .filter(|file| file.before != file.after)
            .map(|file| file.file)
            .collect(),
    })
}

fn parse(text: &str) -> Option<ron::Value> {
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(text)
        .ok()
}

fn fields(value: &ron::Value) -> Option<BTreeMap<String, &ron::Value>> {
    let ron::Value::Map(map) = value else {
        return None;
    };
    map.iter()
        .map(|(key, value)| match key {
            ron::Value::String(name) => Some((name.clone(), value)),
            _ => None,
        })
        .collect()
}

/// One change's step: `after` is `before` with `change`'s field written, and nothing else moved.
///
/// - The two texts parse, every other field holds the same value and the written one holds the meant value
///   (`ron::Value` reads every unit variant alike, so this checks structure only).
/// - The written field's span is the change's text, up to white space and trailing commas.
/// - The bytes between the fields are the same: the two texts are equal with the field's span cut out, and for an
///   insert or a removal with the separators the writer adds or drops with it cut too. A dropped comment, a changed
///   comma or changed white space fails.
pub fn read_back(
    file: &Path,
    before_text: &str,
    after_text: &str,
    change: &Change,
) -> Result<(), WriteError> {
    let field = change.field;
    let wrong = || WriteError::ReadBack {
        file: file.to_owned(),
        field,
    };
    let (Some(before), Some(after)) = (parse(before_text), parse(after_text)) else {
        return Err(wrong());
    };
    let (Some(mut expected), Some(after)) = (fields(&before), fields(&after)) else {
        return Err(wrong());
    };
    expected.remove(field);
    let meant = match &change.value {
        None => None,
        Some(value) => {
            let parsed = parse(&format!("({})", render_one(field, value))).ok_or_else(wrong)?;
            let held = fields(&parsed).and_then(|map| map.get(field).map(|v| (*v).clone()));
            Some(held.ok_or_else(wrong)?)
        }
    };
    if after.get(field).copied() != meant.as_ref() {
        return Err(wrong());
    }
    if after
        .keys()
        .any(|name| name != field && !expected.contains_key(name))
        || expected
            .iter()
            .any(|(name, value)| after.get(name) != Some(value))
    {
        return Err(wrong());
    }
    let before_span = span::field_span(before_text, field);
    let after_span = span::field_span(after_text, field);
    let meant_text = change
        .value
        .as_ref()
        .map(|value| normal(&render_one(field, value)));
    if after_span.as_ref().map(|s| normal(&after_text[s.clone()])) != meant_text {
        return Err(wrong());
    }
    let same_between = match (before_span, after_span) {
        (Some(b), Some(a)) => {
            format!("{}{}", &before_text[..b.start], &before_text[b.end..])
                == format!("{}{}", &after_text[..a.start], &after_text[a.end..])
        }
        (None, Some(a)) => cuts(after_text, &a).iter().any(|text| text == before_text),
        (Some(b), None) => cuts(before_text, &b).iter().any(|text| text == after_text),
        (None, None) => before_text == after_text,
    };
    if same_between { Ok(()) } else { Err(wrong()) }
}

/// The text with a field's span cut out together with what the writer adds or drops beside it: a trailing comma, the
/// blanks after it, its line, the newline before it, the comma and blanks before it.
fn cuts(text: &str, span: &std::ops::Range<usize>) -> Vec<String> {
    let bytes = text.as_bytes();
    let blank = |b: u8| b == b' ' || b == b'\t';
    let line_start = text[..span.start].rfind('\n').map_or(0, |at| at + 1);
    let alone_before = bytes[line_start..span.start].iter().all(|b| blank(*b));
    let comma = (bytes.get(span.end) == Some(&b',')).then_some(span.end + 1);
    let after_comma = comma.unwrap_or(span.end);
    let blanks = bytes[after_comma..]
        .iter()
        .take_while(|b| blank(**b))
        .count();
    let line_end = text[after_comma..]
        .find('\n')
        .map(|at| after_comma + at + 1)
        .filter(|end| text[after_comma..*end].trim().is_empty());
    let before_ws = text[..span.start].trim_end().len();
    let mut starts = vec![span.start];
    if alone_before {
        starts.push(line_start);
        if line_start > 0 {
            starts.push(line_start - 1);
        }
    }
    if before_ws > 0 && bytes[before_ws - 1] == b',' {
        starts.push(before_ws - 1);
    }
    let mut ends = vec![span.end, after_comma, after_comma + blanks];
    ends.extend(line_end);
    let mut out = Vec::new();
    for start in &starts {
        for end in &ends {
            let cut = format!("{}{}", &text[..*start], &text[*end..]);
            if !out.contains(&cut) {
                out.push(cut);
            }
        }
    }
    out
}

/// A change's field as the text `name: value` the writer meant, on one line.
fn render_one(field: &str, value: &Value) -> String {
    match value {
        Value::One(one) => format!("{field}: {one}"),
        Value::List(items) => format!("{field}: [{}]", items.join(", ")),
    }
}

/// A span without its white space outside strings and without a trailing comma before a closing bracket, so a
/// list on one line and one element a line compare equal.
fn normal(text: &str) -> String {
    let mut out = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for c in text.chars() {
        if in_string {
            out.push(c);
            match (escaped, c) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => {}
            }
        } else if c == '"' {
            in_string = true;
            out.push(c);
        } else if matches!(c, ']' | ')') && out.ends_with(',') {
            out.pop();
            out.push(c);
        } else if !c.is_whitespace() {
            out.push(c);
        }
    }
    out
}

/// What `run` brings to the game's values. Each slice that gives the writer a target adds a variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteArea {
    /// A gear item's always-on bonuses (`GearStat` components).
    Items,
    /// A class's and an archetype's BAB and saves, and the save exceptions.
    ClassTables,
    /// A selection's feats or mythic abilities, and whether its picks ignore prerequisites.
    OptionLists,
}

impl WriteArea {
    /// The area `--write` names.
    pub fn parse(name: &str) -> Option<WriteArea> {
        match name {
            "items" => Some(WriteArea::Items),
            "class-tables" => Some(WriteArea::ClassTables),
            "option-lists" => Some(WriteArea::OptionLists),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WriteArea::Items => "items",
            WriteArea::ClassTables => "class-tables",
            WriteArea::OptionLists => "option-lists",
        }
    }

    /// The check's area, which the registry names it by.
    pub fn area(self) -> crate::Area {
        match self {
            WriteArea::Items => crate::Area::Items,
            WriteArea::ClassTables => crate::Area::ClassTables,
            WriteArea::OptionLists => crate::Area::OptionLists,
        }
    }

    /// The `kind`s of the changes this area plans (`Change::kind`).
    pub fn kinds(self) -> &'static [&'static str] {
        match self {
            WriteArea::Items => &["bonuses"],
            WriteArea::ClassTables => &[
                "BAB differs",
                "Fortitude differs",
                "Reflex differs",
                "Will differs",
                "skill points differ (the user's facts)",
                "class skills differ (the user's facts)",
            ],
            WriteArea::OptionLists => &["list"],
        }
    }
}

/// What `run` planned and wrote.
#[derive(Debug)]
pub struct Outcome {
    /// The changes written, or planned with `dry_run`.
    pub changes: Vec<Change>,
    /// The user's facts the registry holds: not written, and the findings stay.
    pub to_the_user: Vec<Change>,
    /// The ticket of each written change the registry has `applied`, by entry and field (the log's `fact` column).
    pub facts: BTreeMap<(String, &'static str), String>,
    pub unexpressed: Vec<Unexpressed>,
    pub applied: Applied,
}

/// Plans the changes of `areas` from the data under `dir` and the dump, and [`apply`]s them, or with `dry_run` only
/// [`plan`]s them (the guards run, nothing is written). `only` limits the option-lists area to the selections it names. The data is read with
/// `seeds::read_game_data_files`, the model as the files hold it, not compiled.
pub fn run(
    dump: &crate::Dump,
    dir: &Path,
    areas: &[WriteArea],
    only: Option<&BTreeSet<String>>,
    dry_run: bool,
    facts: &facts::Facts,
    validate: impl Fn(&Path) -> Result<(), String>,
) -> Result<Outcome, WriteError> {
    let data =
        seeds::read_game_data_files(dir).map_err(|error| WriteError::Invalid(error.to_string()))?;
    let stale = facts.stale(&data);
    if !stale.is_empty() {
        return Err(WriteError::StaleFacts(stale));
    }
    if let Some(unknown) = only.and_then(|only| {
        only.iter().find(|key| {
            !data
                .selections
                .iter()
                .any(|s| s.key.as_str() == key.as_str())
        })
    }) {
        return Err(WriteError::Invalid(format!(
            "--only names no selection: {unknown}"
        )));
    }
    let planner = crate::planner::Planner::new(&data);
    let ctx = crate::areas::Ctx {
        dump,
        planner: &planner,
    };
    let mut changes = Vec::new();
    let mut unexpressed = Vec::new();
    let mut to_the_user = Vec::new();
    let mut applied_facts = BTreeMap::new();
    for area in areas {
        let (more, left) = match area {
            WriteArea::Items => items::changes(&ctx),
            WriteArea::ClassTables => tables::changes(&ctx, facts),
            WriteArea::OptionLists => selections::changes(&ctx, only),
        };
        for change in more {
            let entry = change.at.to_string();
            match facts.decide(area.area(), change.kind, &entry) {
                facts::Decision::Held => to_the_user.push(change),
                facts::Decision::Applied(ticket) => {
                    applied_facts.insert((entry, change.field), format!("applied {ticket}"));
                    changes.push(change);
                }
                facts::Decision::Free => changes.push(change),
            }
        }
        unexpressed.extend(left);
    }
    let applied = if dry_run {
        plan(dir, &changes)?
    } else {
        apply(dir, &changes, validate)?
    };
    Ok(Outcome {
        changes,
        to_the_user,
        facts: applied_facts,
        unexpressed,
        applied,
    })
}

/// Every file's text built and every guard run, nothing written.
fn plan_files<'a>(dir: &Path, changes: &'a [Change]) -> Result<Vec<FilePlan<'a>>, WriteError> {
    let mut by_file: BTreeMap<&Path, Vec<&Change>> = BTreeMap::new();
    for change in changes {
        by_file.entry(&change.file).or_default().push(change);
    }
    let mut plan: Vec<FilePlan> = Vec::new();
    for (file, changes) in by_file {
        if file.starts_with("planner") {
            return Err(WriteError::Planner(file.to_owned()));
        }
        let path = dir.join(file);
        if !path.is_file() {
            return Err(WriteError::Missing(file.to_owned()));
        }
        if let Some(change) = changes.iter().find(|c| !owned(file).contains(&c.field)) {
            return Err(WriteError::NotOwned {
                file: file.to_owned(),
                field: change.field,
            });
        }
        let before = fs::read_to_string(&path).map_err(|error| WriteError::Io {
            file: file.to_owned(),
            error,
        })?;
        let mut after = before.clone();
        let mut steps = Vec::new();
        for change in changes {
            let next =
                span::write_field(&after, change.field, change.value.as_ref(), declared(file))
                    .map_err(|error| WriteError::Span {
                        file: file.to_owned(),
                        field: change.field,
                        error,
                    })?;
            steps.push((after, next.clone(), change));
            after = next;
        }
        plan.push(FilePlan {
            file: file.to_owned(),
            before,
            after,
            steps,
        });
    }
    Ok(plan)
}

/// What [`apply`] would do: the guards and the span writer run on every change and the files whose text would change
/// are listed, with nothing written.
pub fn plan(dir: &Path, changes: &[Change]) -> Result<Applied, WriteError> {
    Ok(Applied {
        files: plan_files(dir, changes)?
            .into_iter()
            .filter(|file| file.before != file.after)
            .map(|file| file.file)
            .collect(),
    })
}
