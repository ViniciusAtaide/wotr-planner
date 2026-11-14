//! A Build's facts in clear (ticket 203): what a record hashes, one fact per line, so that two dumps line up by
//! `(build, path)` and [`crate::explain`] can say what changed rather than only where.
//!
//! One line is `<build><TAB><path><TAB><value>`; a Build's lines are sorted by path (a path twice gets `#2`, `#3`…),
//! and the Builds by name, so two dumps stream side by side. Paths are dot-separated like the snapshot files' (a
//! number between dots is a level, a rank or an index, which `explain` wildcards), with `/` before an option's key:
//!
//! - `build.who`, `build.levels`, `build.picks`, `build.possible`;
//! - `sheet.…`: the sheet ([`fingerprint::sheet_text`], so without what a record's `sheet` hash leaves out), its Debug
//!   text read into fields: `sheet.levels.3.state.bab`, a map by its keys, a list by its elements' `id`, `rank`… or
//!   their index;
//! - `removable/<choice>` = `yes`;
//! - `slot.<id>.kind`, `.open` (how many options are open), `.total`, `.option/<pick>` = `open` or `locked: <the
//!   lock reason's text>` (the issue's Debug text where it has no text), `.order-hash` (the options' order, hashed:
//!   a reorder alone shows as this line) and `.reason-hash` (the issues' Debug texts, hashed: a lock reason that
//!   changes without changing its text shows as this line). `<id>` is the slot id's name and values joined by dots
//!   (`Feat.1.General.0`);
//! - `classes.<level>.option/<class>+<archetype>`, `.order-hash`; `paths.option/<path>`, `paths.order-hash`;
//!   `switches.option/<path>`, `switches.order-hash`: the pickers, as the slots;
//! - `allows/<edit>` = `0` or `1` for each edit of the battery (not in a light dump);
//! - `totals.<level>.…` and `caster.<level>.<spell>.…`: each level's totals and caster lines, read as the sheet.
//!
//! The lines are the same facts a record hashes, with one exception: a pick's *edit* isn't hashed either. Everything
//! reads the engine through the calls the fingerprint makes, and a Debug text where it needs the shape of a type, so
//! the harness still compiles against the crates of an older ref.

use std::collections::BTreeMap;

use rules::{Build, Engine, Issue, Pick, SlotOption};

use crate::fingerprint::{self, Mode};
use crate::{Named, in_parallel};

/// A Debug text read into a tree.
#[derive(Debug, PartialEq)]
enum Node {
    Str(String),
    Atom(String),
    Struct(String, Vec<(String, Node)>),
    Tuple(String, Vec<Node>),
    List(Vec<Node>),
    Map(Vec<(Node, Node)>),
    Set(Vec<Node>),
}

struct Parser<'a> {
    text: &'a str,
    at: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.text[self.at..].chars().next()
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.at += 1;
        }
    }

    fn eat(&mut self, c: char) -> bool {
        self.skip_space();
        if self.peek() == Some(c) {
            self.at += c.len_utf8();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.eat(c) {
            Ok(())
        } else {
            Err(format!("expected {c:?} at {}", self.at))
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.at += 1;
        let start = self.at;
        let mut escaped = false;
        for (offset, c) in self.text[start..].char_indices() {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => {
                    self.at = start + offset + 1;
                    return Ok(self.text[start..start + offset].to_owned());
                }
                _ => {}
            }
        }
        Err("unterminated string".to_owned())
    }

    /// A name or number: up to a delimiter.
    fn token(&mut self) -> String {
        let start = self.at;
        while let Some(c) = self.peek() {
            if c.is_whitespace() || ",()[]{}:\"".contains(c) {
                break;
            }
            self.at += c.len_utf8();
        }
        self.text[start..self.at].to_owned()
    }

    fn items(&mut self, close: char) -> Result<Vec<Node>, String> {
        let mut items = Vec::new();
        while !self.eat(close) {
            items.push(self.value()?);
            self.eat(',');
        }
        Ok(items)
    }

    fn value(&mut self) -> Result<Node, String> {
        self.skip_space();
        match self.peek().ok_or("the text ends early")? {
            '"' => self.string().map(Node::Str),
            '[' => {
                self.at += 1;
                self.items(']').map(Node::List)
            }
            '(' => {
                self.at += 1;
                let items = self.items(')')?;
                Ok(if items.is_empty() {
                    Node::Atom("()".to_owned())
                } else {
                    Node::Tuple(String::new(), items)
                })
            }
            '{' => {
                self.at += 1;
                if self.eat('}') {
                    return Ok(Node::Map(Vec::new()));
                }
                let first = self.value()?;
                if self.eat(':') {
                    let mut pairs = vec![(first, self.value()?)];
                    self.eat(',');
                    while !self.eat('}') {
                        let key = self.value()?;
                        self.expect(':')?;
                        pairs.push((key, self.value()?));
                        self.eat(',');
                    }
                    Ok(Node::Map(pairs))
                } else {
                    self.eat(',');
                    let mut items = vec![first];
                    items.extend(self.items('}')?);
                    Ok(Node::Set(items))
                }
            }
            '\'' => {
                // A char: up to its closing quote.
                let rest = &self.text[self.at + 1..];
                let end = rest
                    .char_indices()
                    .skip(1)
                    .find(|(_, c)| *c == '\'')
                    .map(|(at, _)| at)
                    .ok_or("unterminated char")?;
                let text = self.text[self.at..self.at + end + 2].to_owned();
                self.at += end + 2;
                Ok(Node::Atom(text))
            }
            _ => {
                let name = self.token();
                if name.is_empty() {
                    return Err(format!("unexpected {:?} at {}", self.peek(), self.at));
                }
                if self.peek() == Some('(') {
                    self.at += 1;
                    return Ok(Node::Tuple(name, self.items(')')?));
                }
                if self.text[self.at..].starts_with(" {") {
                    self.at += 2;
                    let mut fields = Vec::new();
                    while !self.eat('}') {
                        self.skip_space();
                        let field = self.token();
                        self.expect(':')?;
                        fields.push((field, self.value()?));
                        self.eat(',');
                    }
                    return Ok(Node::Struct(name, fields));
                }
                Ok(Node::Atom(name))
            }
        }
    }
}

/// A Debug text (compact, as `{:?}` writes it) read into a tree; `Err` where it isn't one this reader knows.
fn parse(text: &str) -> Result<Node, String> {
    let mut parser = Parser { text, at: 0 };
    let node = parser.value()?;
    parser.skip_space();
    if parser.at == text.len() {
        Ok(node)
    } else {
        Err(format!("text after the value, at {}", parser.at))
    }
}

/// A node as text, on one line: a key newtype as its key, `Some(x)` as `x`, `None` as `-`.
fn compact(node: &Node) -> String {
    let list = |items: &[Node]| items.iter().map(compact).collect::<Vec<_>>().join(", ");
    match node {
        Node::Str(text) => format!("\"{text}\""),
        Node::Atom(text) if text == "None" => "-".to_owned(),
        Node::Atom(text) => text.clone(),
        Node::Tuple(name, items) => match (name.as_str(), items.as_slice()) {
            ("Some", [item]) => compact(item),
            (name, [Node::Str(key)]) if name.ends_with("Key") => key.clone(),
            ("", items) => format!("({})", list(items)),
            (name, items) => format!("{name}({})", list(items)),
        },
        Node::Struct(name, fields) => {
            let fields: Vec<String> = fields
                .iter()
                .map(|(field, value)| format!("{field}: {}", compact(value)))
                .collect();
            format!("{name} {{ {} }}", fields.join(", "))
        }
        Node::List(items) => format!("[{}]", list(items)),
        Node::Set(items) => format!("{{{}}}", list(items)),
        Node::Map(pairs) => {
            let pairs: Vec<String> = pairs
                .iter()
                .map(|(key, value)| format!("{}: {}", compact(key), compact(value)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
    }
}

/// A node as one path segment: a struct as its name and values joined by dots (`Feat.1.General.0`), a key newtype as its
/// key, anything else as [`compact`].
fn segment(node: &Node) -> String {
    match node {
        Node::Struct(name, fields) => std::iter::once(name.clone())
            .chain(fields.iter().map(|(_, value)| segment(value)))
            .collect::<Vec<_>>()
            .join("."),
        Node::Tuple(name, items) => match (name.as_str(), items.as_slice()) {
            ("Some", [item]) => segment(item),
            (_, [Node::Str(_)]) => compact(node),
            (_, [item]) => segment(item),
            _ => compact(node),
        },
        Node::Str(text) => text.clone(),
        _ => compact(node),
    }
}

/// Whether a node is one value on a line, not worth splitting: a text, an atom, a newtype or tuple of those, an empty
/// container or a list of those.
fn leafy(node: &Node) -> bool {
    match node {
        Node::Str(_) | Node::Atom(_) => true,
        Node::Tuple(_, items) => items.iter().all(leafy),
        Node::List(items) | Node::Set(items) => items.iter().all(leafy),
        Node::Map(pairs) => pairs.is_empty(),
        Node::Struct(..) => false,
    }
}

/// What names a list's element in a path: the value of its first `id`, `character_level`, `rank`, `key` or `name`
/// field, so an element inserted before it doesn't rename the others.
fn element_name(node: &Node) -> Option<(&'static str, String)> {
    let Node::Struct(_, fields) = node else {
        return None;
    };
    ["id", "character_level", "rank", "key", "name"]
        .into_iter()
        .find_map(|wanted| {
            fields
                .iter()
                .find(|(field, _)| field == wanted)
                .map(|(_, value)| (wanted, segment(value)))
        })
}

fn join(path: &str, child: &str) -> String {
    if path.is_empty() {
        child.to_owned()
    } else {
        format!("{path}.{child}")
    }
}

/// A node as `path = value` lines.
fn flatten(path: &str, node: &Node, out: &mut Vec<(String, String)>) {
    if leafy(node) {
        out.push((path.to_owned(), compact(node)));
        return;
    }
    match node {
        Node::Struct(_, fields) => {
            for (field, value) in fields {
                flatten(&join(path, field), value, out);
            }
        }
        Node::Tuple(name, items) => match (name.as_str(), items.as_slice()) {
            ("Some", [item]) => flatten(path, item, out),
            (name, [item]) if !name.is_empty() => flatten(&join(path, name), item, out),
            (_, items) => {
                for (index, item) in items.iter().enumerate() {
                    flatten(&join(path, &index.to_string()), item, out);
                }
            }
        },
        Node::List(items) | Node::Set(items) => {
            let mut seen: BTreeMap<String, usize> = BTreeMap::new();
            for (index, item) in items.iter().enumerate() {
                let (named_by, mut name) = element_name(item)
                    .map_or((None, index.to_string()), |(field, name)| {
                        (Some(field), name)
                    });
                let times = seen.entry(name.clone()).or_default();
                *times += 1;
                if *times > 1 {
                    name = format!("{name}#{times}");
                }
                let path = join(path, &name);
                match (named_by, item) {
                    // The field that names the element is in its path: not a line of its own.
                    (Some(field), Node::Struct(_, fields)) => {
                        let mut any = false;
                        for (name, value) in fields.iter().filter(|(name, _)| name != field) {
                            any = true;
                            flatten(&join(&path, name), value, out);
                        }
                        if !any {
                            out.push((path, "-".to_owned()));
                        }
                    }
                    _ => flatten(&path, item, out),
                }
            }
        }
        Node::Map(pairs) => {
            for (key, value) in pairs {
                flatten(&join(path, &segment(key)), value, out);
            }
        }
        Node::Str(_) | Node::Atom(_) => unreachable!("a leaf"),
    }
}

/// A value's Debug text as the fingerprint reads it (as before ticket 125), read into lines under `path`. A text this
/// reader can't read is one line, as it is.
fn debug_lines(path: &str, text: &str, out: &mut Vec<(String, String)>) {
    let text = fingerprint::as_before_ticket_125(text);
    match parse(&text) {
        Ok(node) => flatten(path, &node, out),
        Err(_) => out.push((path.to_owned(), text)),
    }
}

/// A key, class, path or spell's Debug text as a path segment.
fn key_text(debug: &str) -> String {
    parse(&fingerprint::as_before_ticket_125(debug))
        .map_or_else(|_| debug.to_owned(), |node| segment(&node))
}

/// A pick as an option list names it: its key, with its value (if it takes one) in brackets.
fn pick_label(pick: &Pick) -> String {
    let parameter = serde_json::to_value(pick)
        .ok()
        .and_then(|value| {
            value
                .as_object()?
                .values()
                .next()?
                .as_object()?
                .get("parameter")
                .filter(|parameter| !parameter.is_null())
                .map(ToString::to_string)
        })
        .map(|text| text.replace('"', ""));
    match parameter {
        Some(parameter) => format!("{}({parameter})", pick.key()),
        None => pick.key().to_owned(),
    }
}

/// A picker option's status as a value: `open`, or `locked: <reason>`.
fn status_value(engine: &Engine, status: &Result<(), Issue>) -> String {
    match status {
        Ok(()) => "open".to_owned(),
        Err(issue) => format!(
            "locked: {}",
            engine
                .issue_text(issue)
                .unwrap_or_else(|| format!("{issue:?}"))
        ),
    }
}

/// An option of a picker: its label, its value and its issue's Debug text when locked.
type Entry = (String, String, Option<String>);

fn entry(engine: &Engine, label: String, status: &Result<(), Issue>) -> Entry {
    (
        label,
        status_value(engine, status),
        status.as_ref().err().map(|issue| format!("{issue:?}")),
    )
}

/// A picker's lines: each option's status under `<path>.option/<label>`, and the order and the issues hashed.
fn picker(path: &str, options: impl IntoIterator<Item = Entry>, lines: &mut Vec<(String, String)>) {
    let mut labels = String::new();
    let mut issues = String::new();
    for (label, value, issue) in options {
        labels.push_str(&label);
        labels.push('\n');
        if let Some(issue) = issue {
            issues.push_str(&format!("{label} {issue}\n"));
        }
        lines.push((format!("{path}.option/{label}"), value));
    }
    lines.push((
        format!("{path}.order-hash"),
        format!("{:016x}", fingerprint::fnv(&labels)),
    ));
    lines.push((
        format!("{path}.reason-hash"),
        format!("{:016x}", fingerprint::fnv(&issues)),
    ));
}

/// A value on one line: tabs and newlines written out.
fn one_line(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// One Build's facts as `(path, value)`, sorted by path, a repeated path numbered.
pub fn dump_build(engine: &Engine, build: &Build, mode: Mode) -> Vec<(String, String)> {
    let mut lines: Vec<(String, String)> = Vec::new();
    let sheet = engine.evaluate(build);
    let removable = engine.removable(build);
    let mut put = |path: &str, value: String| lines.push((path.to_owned(), value));
    put("build.who", fingerprint::who(build).join(", "));
    put("build.levels", build.levels.len().to_string());
    put("build.picks", build.picks.len().to_string());
    put("build.possible", engine.is_possible(build).to_string());
    debug_lines(
        "sheet",
        &fingerprint::sheet_text(&format!("{sheet:?}")),
        &mut lines,
    );
    for choice in &removable {
        lines.push((format!("removable/{choice:?}"), "yes".to_owned()));
    }
    for slot in sheet
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
    {
        let options: Option<Vec<SlotOption>> =
            engine.options(build, &slot.id).map(fingerprint::folded);
        let path = format!("slot.{}", key_text(&format!("{:?}", slot.id)));
        let open = options
            .iter()
            .flatten()
            .filter(|option| option.status.is_ok())
            .count();
        lines.push((format!("{path}.kind"), format!("{:?}", slot.kind)));
        lines.push((format!("{path}.open"), open.to_string()));
        lines.push((
            format!("{path}.total"),
            options.as_ref().map_or(0, Vec::len).to_string(),
        ));
        let options = options.unwrap_or_default();
        picker(
            &path,
            options
                .iter()
                .map(|option| entry(engine, pick_label(&option.pick), &option.status)),
            &mut lines,
        );
    }
    let past_end = u8::try_from(build.levels.len() + 1).expect("at most 254 levels");
    for level in 1..=past_end {
        let options = engine.class_options(build, level).unwrap_or_default();
        picker(
            &format!("classes.{level}"),
            options.iter().map(|option| {
                let class = key_text(&format!("{:?}", option.class));
                let label = match &option.archetype {
                    Some(archetype) => format!("{class}+{}", key_text(&format!("{archetype:?}"))),
                    None => class,
                };
                entry(engine, label, &option.status)
            }),
            &mut lines,
        );
    }
    let paths = engine.path_options(build);
    picker(
        "paths",
        paths.iter().map(|option| {
            entry(
                engine,
                key_text(&format!("{:?}", option.path)),
                &option.status,
            )
        }),
        &mut lines,
    );
    let switches = engine.switch_options(build);
    picker(
        "switches",
        switches.iter().map(|option| {
            entry(
                engine,
                key_text(&format!("{:?}", option.path)),
                &option.status,
            )
        }),
        &mut lines,
    );
    if mode == Mode::Full {
        let controls = fingerprint::controls(engine, build);
        let edits = fingerprint::battery(engine, build, &controls);
        for edit in &edits {
            lines.push((
                format!("allows/{edit:?}"),
                u8::from(engine.allows(build, edit)).to_string(),
            ));
        }
    }
    for (index, totals) in fingerprint::level_totals(engine, build, &sheet)
        .iter()
        .enumerate()
    {
        debug_lines(
            &format!("totals.{}", index + 1),
            &format!("{totals:?}"),
            &mut lines,
        );
    }
    let spells = fingerprint::spells();
    for (index, line) in fingerprint::level_caster_lines(engine, build, &sheet)
        .iter()
        .enumerate()
    {
        let (level, spell) = (index / spells.len() + 1, &spells[index % spells.len()]);
        let path = format!("caster.{level}.{}", key_text(&format!("{spell:?}")));
        match line {
            Some(line) => debug_lines(&path, &format!("{line:?}"), &mut lines),
            None => lines.push((path, "-".to_owned())),
        }
    }
    // Sorted by path, and a path twice numbered: one path names one fact.
    lines.sort_by(|a, b| a.0.cmp(&b.0));
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (path, _) in &mut lines {
        let times = seen.entry(path.clone()).or_default();
        *times += 1;
        if *times > 1 {
            *path = format!("{path}#{times}");
        }
    }
    // A numbered path can sort out of place (`x#2` after `x.y`): sort again.
    lines.sort_by(|a, b| a.0.cmp(&b.0));
    lines
}

/// Dumps `builds`, sorted by name, to `write` one text at a time: each chunk of Builds in parallel, so only a chunk's
/// lines are held at once.
pub fn dump(
    engine: &Engine,
    mut builds: Vec<Named>,
    threads: usize,
    mode: Mode,
    mut write: impl FnMut(&str),
) {
    builds.sort_by(|a, b| a.0.cmp(&b.0));
    for chunk in builds.chunks(threads.max(1) * 4) {
        let texts = in_parallel(chunk, threads, |(name, build)| {
            let mut text = String::new();
            for (path, value) in dump_build(engine, build, mode) {
                text.push_str(&format!(
                    "{}\t{path}\t{}\n",
                    one_line(name),
                    one_line(&value)
                ));
            }
            text
        });
        for text in texts {
            write(&text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        let mut out = Vec::new();
        debug_lines("x", text, &mut out);
        out.into_iter().map(|(p, v)| format!("{p} = {v}")).collect()
    }

    #[test]
    fn a_struct_is_split_into_its_fields_and_a_map_by_its_keys() {
        let text = r#"State { bab: 3, faith: None, features: {FeatureKey("bravery"): 2, FeatureKey("x"): 1}, picks: [Feat(FeatKey("a")), Feat(FeatKey("b"))] }"#;
        assert_eq!(
            lines(text),
            [
                "x.bab = 3",
                "x.faith = -",
                "x.features.bravery = 2",
                "x.features.x = 1",
                "x.picks = [Feat(a), Feat(b)]",
            ]
        );
    }

    #[test]
    fn a_list_of_structs_is_named_by_its_ids_not_its_positions() {
        let text = r#"[Slot { id: Feat { level: 1, source: General, index: 0 }, pick: None }, Slot { id: Feat { level: 3, source: General, index: 0 }, pick: Some(Feat { feat: FeatKey("a"), parameter: None }) }]"#;
        assert_eq!(
            lines(text),
            [
                "x.Feat.1.General.0.pick = -",
                "x.Feat.3.General.0.pick.feat = a",
                "x.Feat.3.General.0.pick.parameter = -",
            ]
        );
    }

    #[test]
    fn a_text_the_reader_does_not_know_is_one_line() {
        assert_eq!(lines("Weird <1 2>"), ["x = Weird <1 2>"]);
    }

    #[test]
    fn a_picker_gives_each_option_a_line_and_hashes_the_order() {
        let facts = |labels: &[&str]| {
            let mut out = Vec::new();
            let options = labels
                .iter()
                .map(|label| ((*label).to_owned(), "open".to_owned(), None));
            picker("p", options, &mut out);
            out
        };
        let (ab, ba) = (facts(&["a", "b"]), facts(&["b", "a"]));
        assert_eq!(ab[0], ("p.option/a".to_owned(), "open".to_owned()));
        assert_eq!(ab.len(), 4);
        assert_ne!(ab[2], ba[2], "a reorder changes the order hash");
        assert_eq!(ab[3], ba[3], "the issues are the same");
    }
}
