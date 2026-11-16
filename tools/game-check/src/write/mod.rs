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
//! [`apply`] writes a list of [`Change`]s to a data folder all or nothing: it refuses a file under `planner/` or a
//! missing one, builds every new text before writing any, reads each written file back, runs the caller's check of
//! the whole folder, and on any failure puts every touched file's bytes back.

pub mod span;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rules::{DataAt, GameData};

use span::{ARCHETYPE_FIELDS, CLASS_FIELDS, GEAR_FIELDS, SpanError, Value};

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
    Span {
        file: PathBuf,
        field: &'static str,
        error: SpanError,
    },
    /// The file as written differs from the file before in more than the written fields.
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
            WriteError::Span { file, field, error } => {
                write!(f, "{}: `{field}`: {error}", file.display())
            }
            WriteError::ReadBack { file, field } => write!(
                f,
                "{}: after writing `{field}` the file differs in more than the written fields",
                file.display()
            ),
            WriteError::Invalid(why) => write!(f, "the written data fails its check: {why}"),
            WriteError::Io { file, error } => write!(f, "{}: {error}", file.display()),
        }
    }
}

impl std::error::Error for WriteError {}

/// The entry's file under the data folder: the folder of its kind (`prestige/` for a prestige class), then
/// `<key>.ron`; the mythic ascension table and the skills are single files.
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
    match file
        .components()
        .next()
        .and_then(|part| part.as_os_str().to_str())
    {
        Some("classes" | "prestige") => CLASS_FIELDS,
        Some("archetypes") => ARCHETYPE_FIELDS,
        Some("gear") => GEAR_FIELDS,
        _ => &[],
    }
}

/// Writes the changes under `dir`, all or nothing.
///
/// Every file is checked and every new text built before any is written; a file under `planner/`, a missing one or
/// a refused span stops the run with nothing written. After writing, each file is parsed as a `ron::Value` before
/// and after: only the written fields may differ, and they must hold the written values. Then `validate` runs on
/// `dir`. A failure of either restores every touched file's original bytes.
pub fn apply(
    dir: &Path,
    changes: &[Change],
    validate: impl Fn(&Path) -> Result<(), String>,
) -> Result<Applied, WriteError> {
    let mut by_file: BTreeMap<&Path, Vec<&Change>> = BTreeMap::new();
    for change in changes {
        by_file.entry(&change.file).or_default().push(change);
    }
    let mut plan: Vec<(PathBuf, String, String, Vec<&Change>)> = Vec::new();
    for (file, changes) in by_file {
        if file.starts_with("planner") {
            return Err(WriteError::Planner(file.to_owned()));
        }
        let path = dir.join(file);
        if !path.is_file() {
            return Err(WriteError::Missing(file.to_owned()));
        }
        let before = fs::read_to_string(&path).map_err(|error| WriteError::Io {
            file: file.to_owned(),
            error,
        })?;
        let mut after = before.clone();
        for change in &changes {
            after = span::write_field(&after, change.field, change.value.as_ref(), declared(file))
                .map_err(|error| WriteError::Span {
                    file: file.to_owned(),
                    field: change.field,
                    error,
                })?;
        }
        plan.push((file.to_owned(), before, after, changes));
    }

    let mut written: Vec<usize> = Vec::new();
    let result = (|| {
        for (at, (file, before, after, changes)) in plan.iter().enumerate() {
            if before == after {
                continue;
            }
            written.push(at);
            fs::write(dir.join(file), after).map_err(|error| WriteError::Io {
                file: file.clone(),
                error,
            })?;
            read_back(file, before, after, changes)?;
        }
        validate(dir).map_err(WriteError::Invalid)
    })();
    if let Err(error) = result {
        for at in written {
            let (file, before, ..) = &plan[at];
            // A restore that fails too is the lesser error: the one that led here is reported.
            let _ = fs::write(dir.join(file), before);
        }
        return Err(error);
    }
    Ok(Applied {
        files: plan
            .into_iter()
            .filter(|(_, before, after, _)| before != after)
            .map(|(file, ..)| file)
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

/// The written text parsed, against the text before: the written fields hold the written values (or are gone) and
/// every other field is as it was.
pub fn read_back(
    file: &Path,
    before_text: &str,
    after_text: &str,
    changes: &[&Change],
) -> Result<(), WriteError> {
    let wrong = |field| WriteError::ReadBack {
        file: file.to_owned(),
        field,
    };
    let first = changes.first().map_or("", |change| change.field);
    let (Some(before), Some(after)) = (parse(before_text), parse(after_text)) else {
        return Err(wrong(first));
    };
    let (Some(mut expected), Some(after)) = (fields(&before), fields(&after)) else {
        return Err(wrong(first));
    };
    let expected_names: Vec<String> = expected.keys().cloned().collect();
    // The last change to a field decides what it holds.
    let mut owned: BTreeMap<&'static str, Option<ron::Value>> = BTreeMap::new();
    for change in changes {
        let value = match &change.value {
            None => None,
            Some(value) => {
                let text = match value {
                    Value::One(one) => one.clone(),
                    Value::List(items) => format!("[{}]", items.join(", ")),
                };
                let parsed = parse(&format!("({}: {text})", change.field))
                    .ok_or_else(|| wrong(change.field))?;
                let held = fields(&parsed)
                    .and_then(|map| map.get(change.field).map(|value| (*value).clone()));
                Some(held.ok_or_else(|| wrong(change.field))?)
            }
        };
        owned.insert(change.field, value);
    }
    let mut expected_owned: BTreeMap<String, Option<&ron::Value>> = BTreeMap::new();
    for (field, value) in &owned {
        expected.remove(*field);
        expected_owned.insert((*field).to_owned(), value.as_ref());
    }
    for (field, value) in &expected {
        if after.get(field) != Some(value) {
            return Err(wrong(owned.keys().next().copied().unwrap_or(first)));
        }
    }
    for (field, value) in &owned {
        if after.get(*field).copied() != value.as_ref() {
            return Err(wrong(field));
        }
    }
    // `ron::Value` reads every unit variant alike, so the texts are compared too: an untouched field's span is
    // the same bytes, a written one is the text the change meant, up to white space and trailing commas.
    for name in expected_names {
        if owned.contains_key(name.as_str()) {
            continue;
        }
        let same = span::field_span(before_text, &name).map(|s| &before_text[s])
            == span::field_span(after_text, &name).map(|s| &after_text[s]);
        if !same {
            return Err(wrong(owned.keys().next().copied().unwrap_or(first)));
        }
    }
    for change in changes {
        let meant = change
            .value
            .as_ref()
            .map(|value| render_one(change.field, value));
        let held = span::field_span(after_text, change.field).map(|s| normal(&after_text[s]));
        if held != meant.map(|text| normal(&text)) {
            return Err(wrong(change.field));
        }
    }
    let extra = after
        .keys()
        .any(|name| !expected.contains_key(name) && !owned.contains_key(name.as_str()));
    if extra {
        return Err(wrong(first));
    }
    Ok(())
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
