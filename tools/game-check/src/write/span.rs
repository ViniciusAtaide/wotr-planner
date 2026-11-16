//! One top-level field of an entry's RON text: its span, and writing it (replace, insert, remove) with every other
//! byte kept. Lifted from ticket 296's rewrite crate.

use std::fmt;
use std::ops::Range;

use serde::Serialize;

/// The fields of a class in the order `rules::Class` declares them (`libs/rules/src/game_data.rs`).
pub const CLASS_FIELDS: &[&str] = &[
    "key",
    "blueprint",
    "name",
    "summary",
    "guide",
    "skill_points",
    "bab",
    "fort",
    "reflex",
    "will",
    "save_exceptions",
    "class_skills",
    "prerequisite",
    "spontaneous",
    "casting_ability",
    "advances",
    "counts_as",
    "levels",
    "components",
];

/// The fields of an archetype in the order `rules::Archetype` declares them, with `bab` between `casting_ability`
/// and `fort` (the field is new, 293f).
pub const ARCHETYPE_FIELDS: &[&str] = &[
    "key",
    "class",
    "blueprint",
    "name",
    "summary",
    "guide",
    "prerequisite",
    "skill_points",
    "class_skills",
    "spontaneous",
    "casting_ability",
    "bab",
    "fort",
    "reflex",
    "will",
    "levels",
    "components",
];

/// The fields of a gear item in the order `rules::GearItem` declares them (`libs/rules/src/gear.rs`).
pub const GEAR_FIELDS: &[&str] = &[
    "key",
    "blueprint",
    "name",
    "slot",
    "guide",
    "components",
    "always_on",
    "other_effects",
];

/// The fields of a feature selection in the order `rules::FeatureSelection` declares them
/// (`libs/rules/src/game_data.rs`).
pub const SELECTION_FIELDS: &[&str] = &[
    "key",
    "blueprint",
    "name",
    "summary",
    "guide",
    "options",
    "feats",
    "mythic_abilities",
    "needs_slot_of",
    "ignores_prerequisites",
    "skips_proficiency",
    "shared_options",
    "option_levels",
    "option_level_archetypes",
    "option_headers_first",
    "skipped_levels",
    "replaced_by",
    "feat_lists_only",
    "only_without_option",
    "filtered_by",
    "later_rows_behind",
    "level_from",
    "level_also_from",
    "once_per_option",
    "deity_domains",
    "band",
    "components",
];

/// A field's new value as RON text: one value, or a list of elements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    One(String),
    List(Vec<String>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SpanError {
    /// The field's span holds a `//` comment: replacing it would lose the comment.
    CommentInside { field: &'static str },
    /// No field before it in the declared order is in the text, so there is nowhere to put it.
    NoAnchor { field: &'static str },
    /// The text holds a `/*` block comment, which the writer does not scan.
    BlockComment,
}

impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpanError::CommentInside { field } => write!(f, "a comment inside `{field}`"),
            SpanError::NoAnchor { field } => {
                write!(f, "no field before `{field}` to put it after")
            }
            SpanError::BlockComment => write!(f, "a `/*` block comment in the text"),
        }
    }
}

impl std::error::Error for SpanError {}

/// RON on one line, as the files write it: `name: value`, `a, b`, no `Some` (`implicit_some`).
pub fn ron_text<T: Serialize>(value: &T) -> String {
    let config = ron::ser::PrettyConfig::new()
        .new_line("")
        .indentor("")
        .separator(" ")
        .compact_arrays(true)
        .compact_structs(true)
        .compact_maps(true)
        .extensions(ron::extensions::Extensions::IMPLICIT_SOME);
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .to_string_pretty(value, config)
        .expect("a value writes as RON")
}

/// The span of a top-level field `name: value` (from the name to the end of the value, before its comma) in an
/// entry's text, outside strings and comments. A planner-made file's entry is nested one deeper and is never passed
/// here.
pub fn field_span(text: &str, name: &str) -> Option<Range<usize>> {
    let bytes = text.as_bytes();
    let name = name.as_bytes();
    let mut depth = 0usize;
    let mut at = 0usize;
    let mut found = None;
    while at < bytes.len() {
        match bytes[at] {
            b'"' => {
                at = skip_string(bytes, at);
                continue;
            }
            b'/' if bytes.get(at + 1) == Some(&b'/') => {
                while at < bytes.len() && bytes[at] != b'\n' {
                    at += 1;
                }
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                if let Some(start) = found
                    && depth == 0
                {
                    return Some(start..trim_end(text, at));
                }
            }
            b',' if depth == 1 => {
                if let Some(start) = found {
                    return Some(start..trim_end(text, at));
                }
            }
            _ if depth == 1
                && found.is_none()
                && bytes[at..].starts_with(name)
                && (at == 0 || !is_ident(bytes[at - 1]))
                && next_byte(bytes, at + name.len()) == Some(b':') =>
            {
                found = Some(at);
            }
            _ => {}
        }
        at += 1;
    }
    None
}

/// Replaces, inserts or removes the top-level field `name` of `text`, keeping every other byte.
///
/// - `Some(value)` with the field present replaces its span: a value that spans lines is written one element a line,
///   at the field's indent plus four with the `]` at the field's indent; one that is on one line stays on one.
/// - `Some(value)` with the field missing inserts it after the last field of `declared` before it that is present:
///   on a line of its own in a file whose fields are indented, after a comma in a one-line file. It is one line,
///   except a `components` list in an indented file, one element a line.
/// - `None` removes the field: its whole line in an indented file, `name: value, ` or `, name: value` in a one-line
///   one. A field that isn't there stays unwritten.
///
/// A span that holds a `//` comment is refused.
pub fn write_field(
    text: &str,
    name: &'static str,
    value: Option<&Value>,
    declared: &[&str],
) -> Result<String, SpanError> {
    if text.contains("/*") {
        return Err(SpanError::BlockComment);
    }
    match (field_span(text, name), value) {
        (Some(span), Some(value)) => {
            refuse_comment(text, &span, name)?;
            let multiline = text[span.clone()].contains('\n');
            let rendered = render(name, value, indent_of(text, span.start), multiline);
            Ok(format!(
                "{}{rendered}{}",
                &text[..span.start],
                &text[span.end..]
            ))
        }
        (Some(span), None) => {
            refuse_comment(text, &span, name)?;
            Ok(remove(text, &span))
        }
        (None, Some(value)) => insert(text, name, value, declared),
        (None, None) => Ok(text.to_owned()),
    }
}

/// `name: value`, the value in the style the field has or is given.
fn render(name: &str, value: &Value, indent: Option<&str>, multiline: bool) -> String {
    match value {
        Value::One(one) => format!("{name}: {one}"),
        Value::List(items) if multiline && !items.is_empty() => {
            let indent = indent.unwrap_or("");
            let mut out = format!("{name}: [\n");
            for item in items {
                out.push_str(&format!("{indent}    {item},\n"));
            }
            out.push_str(&format!("{indent}]"));
            out
        }
        Value::List(items) => format!("{name}: [{}]", items.join(", ")),
    }
}

/// The indent of the line a field starts, when only blanks precede it there: the sign of a pretty file. A field
/// at the start of a line has the empty indent.
fn indent_of(text: &str, start: usize) -> Option<&str> {
    let line_start = text[..start].rfind('\n').map_or(0, |at| at + 1);
    let before = &text[line_start..start];
    before
        .chars()
        .all(|c| c == ' ' || c == '\t')
        .then_some(before)
}

fn insert(
    text: &str,
    name: &'static str,
    value: &Value,
    declared: &[&str],
) -> Result<String, SpanError> {
    let position = declared
        .iter()
        .position(|field| *field == name)
        .ok_or(SpanError::NoAnchor { field: name })?;
    let anchor = declared[..position]
        .iter()
        .rev()
        .find_map(|field| field_span(text, field).map(|span| (*field, span)))
        .ok_or(SpanError::NoAnchor { field: name })?;
    let (_, span) = anchor;
    refuse_comment(text, &span, name)?;
    // The file is pretty when its first line holds no field: its fields are one a line, at the indent of the first.
    match pretty_indent(text) {
        Some(indent) => {
            let indent = indent.as_str();
            let mut at = span.end;
            if text.as_bytes().get(at) == Some(&b',') {
                at += 1;
                at = past_line_comment(text, at);
                let line = render(name, value, Some(indent), name == "components");
                Ok(format!("{}\n{indent}{line},{}", &text[..at], &text[at..]))
            } else {
                let line = render(name, value, Some(indent), name == "components");
                Ok(format!(
                    "{},\n{indent}{line}{}",
                    &text[..span.end],
                    &text[span.end..]
                ))
            }
        }
        None => {
            let line = render(name, value, None, false);
            Ok(format!(
                "{}, {line}{}",
                &text[..span.end],
                &text[span.end..]
            ))
        }
    }
}

/// The end of the line when only a `//` comment follows `at` on it, else `at`.
fn past_line_comment(text: &str, at: usize) -> usize {
    let rest = &text[at..];
    let line_end = rest.find('\n').unwrap_or(rest.len());
    if rest[..line_end].trim_start().starts_with("//") {
        at + line_end
    } else {
        at
    }
}

fn remove(text: &str, span: &Range<usize>) -> String {
    let bytes = text.as_bytes();
    let comma = (bytes.get(span.end) == Some(&b',')).then_some(span.end + 1);
    let after = comma.unwrap_or(span.end);
    let line_start = text[..span.start].rfind('\n').map_or(0, |at| at + 1);
    let alone_before = text[line_start..span.start]
        .chars()
        .all(|c| c == ' ' || c == '\t');
    let line_end = text[after..].find('\n').map(|at| after + at + 1);
    let alone_after = line_end.is_some_and(|end| text[after..end].trim().is_empty());
    if alone_before && alone_after {
        let end = line_end.expect("alone_after has a line end");
        return format!("{}{}", &text[..line_start], &text[end..]);
    }
    if let Some(comma) = comma {
        let blanks = text[comma..]
            .bytes()
            .take_while(|b| *b == b' ' || *b == b'\t')
            .count();
        return format!("{}{}", &text[..span.start], &text[comma + blanks..]);
    }
    // The last field of a line: the comma before it goes too.
    let kept = text[..span.start].trim_end();
    let from = kept.strip_suffix(',').map_or(span.start, str::len);
    format!("{}{}", &text[..from], &text[span.end..])
}

fn refuse_comment(text: &str, span: &Range<usize>, field: &'static str) -> Result<(), SpanError> {
    let bytes = &text.as_bytes()[span.clone()];
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'"' => {
                at = skip_string(bytes, at);
                continue;
            }
            b'/' if bytes.get(at + 1) == Some(&b'/') => {
                return Err(SpanError::CommentInside { field });
            }
            _ => {}
        }
        at += 1;
    }
    Ok(())
}

/// The first byte at or after `at` that isn't white space.
fn next_byte(bytes: &[u8], mut at: usize) -> Option<u8> {
    while let Some(byte) = bytes.get(at) {
        if !byte.is_ascii_whitespace() {
            return Some(*byte);
        }
        at += 1;
    }
    None
}

fn trim_end(text: &str, end: usize) -> usize {
    text[..end].trim_end().len()
}

fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The index just past the string starting at `at`.
fn skip_string(bytes: &[u8], mut at: usize) -> usize {
    at += 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            b'"' => return at + 1,
            _ => at += 1,
        }
    }
    at
}

/// The indent of a pretty file's fields: the file is pretty when the line of its opening `(` holds nothing else, and
/// its fields are at the indent of the first line after it that isn't blank or a comment.
fn pretty_indent(text: &str) -> Option<String> {
    let mut lines = text.lines().peekable();
    // Skip leading comments and blanks to the line holding the opening bracket.
    let open = loop {
        let line = lines.next()?;
        let trimmed = line.trim_start();
        if !(trimmed.is_empty() || trimmed.starts_with("//")) {
            break trimmed;
        }
    };
    let after = open.find('(').map(|at| &open[at + 1..])?;
    let after = after.trim();
    if !(after.is_empty() || after.starts_with("//")) {
        return None;
    }
    lines
        .find(|line| {
            let trimmed = line.trim_start();
            !(trimmed.is_empty() || trimmed.starts_with("//"))
        })
        .map(|line| line[..line.len() - line.trim_start().len()].to_owned())
        .filter(|indent| !indent.is_empty())
}
