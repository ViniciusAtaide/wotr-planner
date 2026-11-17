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

pub mod carriers;
pub mod facts;
pub mod items;
pub mod log;
pub mod prerequisites;
pub mod selections;
pub mod span;
pub mod tables;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rules::{DataAt, GameData};

use span::{
    ARCHETYPE_FIELDS, CLASS_FIELDS, FEAT_FIELDS, FEATURE_FIELDS, GEAR_FIELDS,
    MYTHIC_ABILITY_FIELDS, MYTHIC_FEAT_FIELDS, OPTION_FIELDS, SELECTION_FIELDS, SpanError, Value,
};

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

/// One whole file to create: an entry the data doesn't have yet (ticket 307a). It is written after the field
/// changes, never over a file that exists, and removed again if anything after it fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewFile {
    pub at: DataAt,
    /// The entry's file, relative to the data folder; [`creatable`] says which folders may take one.
    pub file: PathBuf,
    /// The file's whole text.
    pub text: String,
    /// The check's finding kind this file answers, for the registry.
    pub kind: &'static str,
}

/// An item the importer left unwritten because the game gives a bonus the planner has no word for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unexpressed {
    pub at: DataAt,
    pub what: String,
}

/// What [`apply`] wrote: the files whose text changed, and the files it created, each in path order.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Applied {
    pub files: Vec<PathBuf>,
    pub created: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum WriteError {
    /// A change on a planner-made file, which the importer never writes.
    Planner(PathBuf),
    Missing(PathBuf),
    /// A new file whose path is already taken: the writer never replaces a file it didn't read.
    Exists(PathBuf),
    /// A request the writer can't take: an `--only` key that is no selection.
    Usage(String),
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
            WriteError::Exists(file) => write!(
                f,
                "{}: a new file is never written over one that exists",
                file.display()
            ),
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
            WriteError::Usage(why) => f.write_str(why),
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
        Some("features") => FEATURE_FIELDS,
        Some("feats") => FEAT_FIELDS,
        Some("options") => OPTION_FIELDS,
        Some("mythic") => match subfolder(file) {
            Some("abilities") => MYTHIC_ABILITY_FIELDS,
            Some("feats") => MYTHIC_FEAT_FIELDS,
            _ => &[],
        },
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
        // The carrier-grants target (ticket 307m): the grant components a feature, a feat or an option gains, and
        // an option's rows. It only adds: the read-back refuses a change beyond the field.
        Some("features" | "feats") => &["components"],
        Some("options") => &["components", "levels"],
        // The prerequisites target (ticket 308a): a mythic ability's or a mythic feat's `Requires`.
        Some("mythic") if matches!(subfolder(file), Some("abilities" | "feats")) => &["components"],
        _ => &[],
    }
}

/// Whether the importer may create this file: `features/<key>.ron`, and no other folder yet (a later slice gives
/// a folder its place here).
pub fn creatable(file: &Path) -> bool {
    let mut parts = file.components();
    let (Some(first), Some(name), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    first.as_os_str() == "features"
        && Path::new(name.as_os_str())
            .extension()
            .is_some_and(|e| e == "ron")
}

fn folder(file: &Path) -> Option<&str> {
    file.components()
        .next()
        .and_then(|part| part.as_os_str().to_str())
}

/// The second component of a path whose file is below it: `abilities` of `mythic/abilities/<key>.ron`.
fn subfolder(file: &Path) -> Option<&str> {
    let mut parts = file.components();
    let (_, Some(second), Some(_)) = (parts.next(), parts.next(), parts.next()) else {
        return None;
    };
    second.as_os_str().to_str()
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
/// `new_files` are created after the field changes: a path under `planner/`, one that exists, or one [`creatable`]
/// refuses stops the run with nothing written. After writing, each is read back: the disk holds its text, which
/// parses as the folder's type, with its key the path's stem and a blueprint set. A failure puts every field
/// file back and removes every file and folder the run created.
///
/// The set of files is not atomic: a kill between two files leaves the folder half written, and git is the
/// recovery. Each file is replaced atomically ([`atomic_write`]), so none is left truncated.
pub fn apply(
    dir: &Path,
    changes: &[Change],
    new_files: &[NewFile],
    validate: impl Fn(&Path) -> Result<(), String>,
) -> Result<Applied, WriteError> {
    apply_with(dir, changes, new_files, validate, atomic_write)
}

/// [`apply`] with the function that writes a file, so a test can make a write fail or land wrong. Restoring always
/// uses [`atomic_write`].
pub fn apply_with(
    dir: &Path,
    changes: &[Change],
    new_files: &[NewFile],
    validate: impl Fn(&Path) -> Result<(), String>,
    write: impl Fn(&Path, &str) -> io::Result<()>,
) -> Result<Applied, WriteError> {
    let plan = plan_files(dir, changes)?;
    let creating = plan_new(dir, new_files)?;

    let mut written: Vec<usize> = Vec::new();
    let mut made = Made::default();
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
        for new in &creating {
            made.create(dir, new)?;
            read_back_new(dir, new)?;
        }
        validate(dir).map_err(WriteError::Invalid)
    })();
    if let Err(cause) = result {
        let mut failed: Vec<PathBuf> = written
            .into_iter()
            .filter(|at| atomic_write(&dir.join(&plan[*at].file), &plan[*at].before).is_err())
            .map(|at| plan[at].file.clone())
            .collect();
        failed.extend(made.undo(dir));
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
        created: creating.iter().map(|new| new.file.clone()).collect(),
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
    /// The hidden features a prerequisite names, and the chains that hold them, as new `features/` files.
    Carriers,
    /// The grants an existing entry's blueprint makes of a carrier, as components and option rows (ticket 307m).
    CarrierGrants,
    /// An entry's `Requires` components, from its blueprint's `Prerequisite*` components (ticket 308a).
    Prerequisites,
}

impl WriteArea {
    /// The area `--write` names.
    pub fn parse(name: &str) -> Option<WriteArea> {
        match name {
            "items" => Some(WriteArea::Items),
            "class-tables" => Some(WriteArea::ClassTables),
            "option-lists" => Some(WriteArea::OptionLists),
            "carriers" => Some(WriteArea::Carriers),
            "carrier-grants" => Some(WriteArea::CarrierGrants),
            "prerequisites" => Some(WriteArea::Prerequisites),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WriteArea::Items => "items",
            WriteArea::ClassTables => "class-tables",
            WriteArea::OptionLists => "option-lists",
            WriteArea::Carriers => "carriers",
            WriteArea::CarrierGrants => "carrier-grants",
            WriteArea::Prerequisites => "prerequisites",
        }
    }

    /// The check's area, which the registry names it by.
    pub fn area(self) -> crate::Area {
        match self {
            WriteArea::Items => crate::Area::Items,
            WriteArea::ClassTables => crate::Area::ClassTables,
            WriteArea::OptionLists => crate::Area::OptionLists,
            WriteArea::Carriers | WriteArea::CarrierGrants | WriteArea::Prerequisites => {
                crate::Area::Prerequisites
            }
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
            WriteArea::Carriers => &[carriers::KIND],
            WriteArea::CarrierGrants => &[carriers::GRANT_KIND],
            WriteArea::Prerequisites => &[prerequisites::KIND],
        }
    }
}

/// What `run` planned and wrote.
#[derive(Debug)]
pub struct Outcome {
    /// The changes written, or planned with `dry_run`.
    pub changes: Vec<Change>,
    /// The files created, or planned with `dry_run`.
    pub created: Vec<NewFile>,
    /// The user's facts the registry holds: not written, and the findings stay.
    pub to_the_user: Vec<Change>,
    /// The ticket of each written change the registry has `applied`, by entry and field (the log's `fact` column).
    pub facts: BTreeMap<(String, &'static str), String>,
    pub unexpressed: Vec<Unexpressed>,
    pub applied: Applied,
}

/// [`run_with`] allowing no narrowing of a prerequisite.
pub fn run(
    dump: &crate::Dump,
    dir: &Path,
    areas: &[WriteArea],
    only: Option<&BTreeSet<String>>,
    dry_run: bool,
    facts: &facts::Facts,
    validate: impl Fn(&Path) -> Result<(), String>,
) -> Result<Outcome, WriteError> {
    let none = prerequisites::Narrowings::default();
    run_with(dump, dir, areas, only, dry_run, facts, &none, validate)
}

/// Plans the changes of `areas` from the data under `dir` and the dump, and [`apply`]s them, or with `dry_run` only
/// [`plan`]s them (the guards run, nothing is written). `only` limits the option-lists area to the selections it
/// names, the carriers areas to GUIDs, and the prerequisites area, run alone, to the entries it names as the report
/// does (`feat extra-hex`; a trailing `*` takes every entry the rest begins: `selection option rogue/*`). The data is
/// read with `seeds::read_game_data_files`, the model as the files hold it, not compiled. `narrowings` are the
/// prerequisite rewrites that narrow who qualifies and may be written all the same
/// ([`prerequisites::Narrowings`]).
#[allow(clippy::too_many_arguments)]
pub fn run_with(
    dump: &crate::Dump,
    dir: &Path,
    areas: &[WriteArea],
    only: Option<&BTreeSet<String>>,
    dry_run: bool,
    facts: &facts::Facts,
    narrowings: &prerequisites::Narrowings,
    validate: impl Fn(&Path) -> Result<(), String>,
) -> Result<Outcome, WriteError> {
    let data =
        seeds::read_game_data_files(dir).map_err(|error| WriteError::Invalid(error.to_string()))?;
    let stale = facts.stale(&data);
    if !stale.is_empty() {
        return Err(WriteError::StaleFacts(stale));
    }
    let lists = areas.contains(&WriteArea::OptionLists);
    let carriers =
        areas.contains(&WriteArea::Carriers) || areas.contains(&WriteArea::CarrierGrants);
    if only.is_some() && lists && carriers {
        return Err(WriteError::Usage(
            "--only takes selection keys for option-lists and GUIDs for carriers and carrier-grants: not both at once".to_owned(),
        ));
    }
    if only.is_some() && areas.contains(&WriteArea::Prerequisites) && areas.len() > 1 {
        return Err(WriteError::Usage(
            "--only takes entry names for prerequisites, which then runs alone".to_owned(),
        ));
    }
    if let Some(unknown) = only
        .filter(|_| areas.contains(&WriteArea::Prerequisites))
        .and_then(|only| prerequisites::unmatched(&data, only))
    {
        return Err(WriteError::Usage(format!(
            "--only names no entry the prerequisites area writes: {unknown}"
        )));
    }
    if let Some(unknown) = only.filter(|_| lists).and_then(|only| {
        only.iter().find(|key| {
            !data
                .selections
                .iter()
                .any(|s| s.key.as_str() == key.as_str())
        })
    }) {
        return Err(WriteError::Usage(format!(
            "--only names no selection: {unknown}"
        )));
    }
    if let Some(unknown) = only.filter(|_| carriers).and_then(|only| {
        only.iter().find(|guid| {
            guid.len() != 32 || !guid.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        })
    }) {
        return Err(WriteError::Usage(format!(
            "--only names no GUID (32 lowercase hex digits): {unknown}"
        )));
    }
    let planner = crate::planner::Planner::new(&data);
    let ctx = crate::areas::Ctx {
        dump,
        planner: &planner,
    };
    let mut created: Vec<NewFile> = Vec::new();
    let mut changes = Vec::new();
    let mut unexpressed = Vec::new();
    let mut to_the_user = Vec::new();
    let mut applied_facts = BTreeMap::new();
    // The grants come after the carriers they name, whatever order the areas were given in.
    let mut ordered = areas.to_vec();
    ordered.sort_by_key(|area| *area == WriteArea::CarrierGrants);
    for area in &ordered {
        let (more, left) = match area {
            WriteArea::Items => items::changes(&ctx),
            WriteArea::ClassTables => tables::changes(&ctx, facts),
            WriteArea::OptionLists => selections::changes(&ctx, only),
            WriteArea::Carriers => {
                let (files, left) = carriers::changes(&ctx, only);
                created.extend(files);
                (Vec::new(), left)
            }
            WriteArea::CarrierGrants => carriers::attachments(&ctx, &created, dry_run, only),
            WriteArea::Prerequisites => prerequisites::changes(&ctx, only, narrowings),
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
        plan(dir, &changes, &created)?
    } else {
        apply(dir, &changes, &created, validate)?
    };
    Ok(Outcome {
        changes,
        created,
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
pub fn plan(dir: &Path, changes: &[Change], new_files: &[NewFile]) -> Result<Applied, WriteError> {
    let creating = plan_new(dir, new_files)?;
    Ok(Applied {
        created: creating.iter().map(|new| new.file.clone()).collect(),
        files: plan_files(dir, changes)?
            .into_iter()
            .filter(|file| file.before != file.after)
            .map(|file| file.file)
            .collect(),
    })
}

/// The new files' guards, run before any write: a file under `planner/`, one that exists (or a path two new files
/// share) and one [`creatable`] refuses. In path order.
fn plan_new<'a>(dir: &Path, new_files: &'a [NewFile]) -> Result<Vec<&'a NewFile>, WriteError> {
    let mut sorted: Vec<&NewFile> = new_files.iter().collect();
    sorted.sort_by_key(|new| (new.file.to_string_lossy().to_lowercase(), new.file.clone()));
    for (at, new) in sorted.iter().enumerate() {
        if new.file.starts_with("planner") {
            return Err(WriteError::Planner(new.file.clone()));
        }
        if dir.join(&new.file).symlink_metadata().is_ok()
            || at > 0 && same_path(&sorted[at - 1].file, &new.file)
        {
            return Err(WriteError::Exists(new.file.clone()));
        }
        if !creatable(&new.file) {
            return Err(WriteError::NotOwned {
                file: new.file.clone(),
                field: "(new file)",
            });
        }
    }
    Ok(sorted)
}

/// Two paths a case-insensitive disk (macOS) would make one.
fn same_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
}

/// What a run created, to take away again: the files in order, and the folders it made, outermost first.
#[derive(Default)]
struct Made {
    files: Vec<PathBuf>,
    folders: Vec<PathBuf>,
}

impl Made {
    /// Creates the new file's folders and the file itself, whole; a folder is noted before it is made, so a failure
    /// half way is undone too.
    fn create(&mut self, dir: &Path, new: &NewFile) -> Result<(), WriteError> {
        let io = |error| WriteError::Io {
            file: new.file.clone(),
            error,
        };
        let mut missing: Vec<PathBuf> = Vec::new();
        let mut parent = new.file.parent();
        while let Some(folder) = parent.filter(|p| !p.as_os_str().is_empty()) {
            if dir.join(folder).exists() {
                break;
            }
            missing.push(folder.to_owned());
            parent = folder.parent();
        }
        for folder in missing.into_iter().rev() {
            self.folders.push(folder.clone());
            fs::create_dir(dir.join(folder)).map_err(io)?;
        }
        // `create_new` fails on a path taken since the plan, which a rename would have replaced; the file is
        // noted only once it is ours, so an undo never removes one that was not.
        let mut open = fs::OpenOptions::new();
        let mut handle = match open.write(true).create_new(true).open(dir.join(&new.file)) {
            Ok(handle) => handle,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                return Err(WriteError::Exists(new.file.clone()));
            }
            Err(error) => return Err(io(error)),
        };
        self.files.push(new.file.clone());
        io::Write::write_all(&mut handle, new.text.as_bytes()).map_err(io)
    }

    /// Removes what was created, files then folders (innermost first); the paths that couldn't be removed.
    fn undo(self, dir: &Path) -> Vec<PathBuf> {
        let gone = |result: io::Result<()>| {
            result.is_ok() || result.is_err_and(|e| e.kind() == io::ErrorKind::NotFound)
        };
        let files = self
            .files
            .into_iter()
            .filter(|file| !gone(fs::remove_file(dir.join(file))));
        let folders = self
            .folders
            .into_iter()
            .rev()
            .filter(|folder| !gone(fs::remove_dir(dir.join(folder))));
        files.chain(folders).collect()
    }
}

/// A created file read back: the disk holds the text meant, it parses as the folder's type, its key is the path's
/// stem and it names a blueprint.
fn read_back_new(dir: &Path, new: &NewFile) -> Result<(), WriteError> {
    let wrong = || WriteError::ReadBack {
        file: new.file.clone(),
        field: "(new file)",
    };
    let disk = fs::read_to_string(dir.join(&new.file)).map_err(|error| WriteError::Io {
        file: new.file.clone(),
        error,
    })?;
    if disk != new.text {
        return Err(wrong());
    }
    let feature: rules::Feature = ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&disk)
        .map_err(|_| wrong())?;
    let stem = new.file.file_stem().and_then(|stem| stem.to_str());
    if Some(feature.key.as_str()) != stem || feature.blueprint.is_none() {
        return Err(wrong());
    }
    Ok(())
}
