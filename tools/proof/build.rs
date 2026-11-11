//! Ticket 151 changed what `Engine::totals` and `Engine::caster_line` take, twice: from a Build start and a state, to
//! the state alone (its first commit), to a level of the sheet (its review). The harness compiles against the crates
//! on every side of that (a `rules` proof builds it against a ref's crates too), so this reads which one it's built
//! against from the two functions' parameters in the rules crate beside it, wherever in its sources they're defined:
//!
//! - `cfg(lines_take_start)`: `(&self, start: &BuildStart, state: &CharacterState, …)`, before ticket 151;
//! - `cfg(lines_take_state)`: `(&self, state: &CharacterState, …)`, ticket 151's first commits;
//! - neither: `(&self, level: &LevelSheet, …)`, from ticket 151's review.
//!
//! Anything else (no such function: a ref from before ticket 120, which brought the totals; or a shape none of these)
//! fails the build with what it found.

use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Start,
    State,
    Level,
}

fn main() {
    println!("cargo::rustc-check-cfg=cfg(lines_take_start)");
    println!("cargo::rustc-check-cfg=cfg(lines_take_state)");
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir");
    let sources = Path::new(&manifest).join("../../libs/rules/src");
    println!("cargo::rerun-if-changed={}", sources.display());
    let mut files = Vec::new();
    rust_files(&sources, &mut files);
    let text: String = files
        .iter()
        .map(|file| std::fs::read_to_string(file).unwrap_or_default())
        .collect();

    let totals = shape(&text, "totals");
    let caster_line = shape(&text, "caster_line");
    assert_eq!(
        totals,
        caster_line,
        "`Engine::totals` and `Engine::caster_line` take different things in {}",
        sources.display()
    );
    match totals {
        Shape::Start => println!("cargo::rustc-cfg=lines_take_start"),
        Shape::State => println!("cargo::rustc-cfg=lines_take_state"),
        Shape::Level => {}
    }
}

/// Every `.rs` file under `dir`.
fn rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        panic!(
            "the proof harness needs the rules crate's sources at {}",
            dir.display()
        );
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

/// What the public function `name` takes, from its parameter list (whatever its layout).
fn shape(text: &str, name: &str) -> Shape {
    let head = format!("pub fn {name}(");
    let Some(at) = text.find(&head) else {
        panic!(
            "the rules crate has no `pub fn {name}`: the proof harness reads it from ticket 120 on"
        );
    };
    let rest = &text[at + head.len()..];
    let parameters: String = rest[..rest.find(')').unwrap_or(rest.len())]
        .split_whitespace()
        .collect();
    let takes = |type_name: &str| parameters.contains(&format!(":&{type_name}"));
    match (
        takes("BuildStart"),
        takes("CharacterState"),
        takes("LevelSheet"),
    ) {
        (true, true, false) => Shape::Start,
        (false, true, false) => Shape::State,
        (false, false, true) => Shape::Level,
        _ => panic!(
            "the proof harness doesn't know `{name}({parameters})`: teach `tools/proof/build.rs` it"
        ),
    }
}
