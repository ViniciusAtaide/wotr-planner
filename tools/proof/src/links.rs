//! Builds as share links, one `name<TAB>link` line each: the benchmark's links (`bench-links.tsv`), and the files
//! `export` writes and `record --links` reads, so two revisions record the same Builds.

use rules::{Build, Engine, GameData, LinkKeys, decode_build, encode_build};

use crate::Named;

/// The benchmark's Builds (ticket 56's `links.json`), by name and link; an empty link is the blank Build.
const BENCH: &str = include_str!("../bench-links.tsv");

/// The benchmark's Builds, named `bench <name>`, opened as the planner opens a link.
pub fn bench_builds(engine: &Engine) -> Vec<Named> {
    parse(BENCH)
        .map(|(name, link)| {
            let build = if link.is_empty() {
                Build::default()
            } else {
                engine.open_link(link).expect("the benchmark's links open")
            };
            (format!("bench {name}"), build)
        })
        .collect()
}

/// The `name<TAB>link` lines of a file, skipping blank ones.
fn parse(text: &str) -> impl Iterator<Item = (&str, &str)> {
    text.lines().filter(|line| !line.is_empty()).map(|line| {
        line.split_once('\t')
            .unwrap_or_else(|| panic!("a links line is `name<TAB>link`: {line:?}"))
    })
}

/// The Builds of a links file, in its order, decoded with the game data's keys whether or not they're possible (a
/// Build no sheet holds is recorded too). Returns the names of the links that don't decode apart: those Builds
/// can't be recorded on this side, so a comparison lists them as only in the other.
pub fn read(data: &GameData, text: &str) -> (Vec<Named>, Vec<String>) {
    let keys = LinkKeys::new(data);
    let mut builds = Vec::new();
    let mut failed = Vec::new();
    for (name, link) in parse(text) {
        if link.is_empty() {
            builds.push((name.to_owned(), Build::default()));
            continue;
        }
        match decode_build(link, &keys) {
            Ok(build) => builds.push((name.to_owned(), build)),
            Err(error) => failed.push(format!("{name}: {error:?}")),
        }
    }
    (builds, failed)
}

/// The Builds as a links file. Returns the names of the Builds whose link doesn't decode back to the same Build with
/// this game data's keys; they're left out.
pub fn export(data: &GameData, builds: &[Named]) -> (String, Vec<String>) {
    let keys = LinkKeys::new(data);
    let mut text = String::new();
    let mut skipped = Vec::new();
    for (name, build) in builds {
        assert!(
            !name.contains(['\t', '\n']),
            "a Build name holds no tab or line break: {name:?}"
        );
        let link = encode_build(build);
        if decode_build(&link, &keys).is_ok_and(|decoded| decoded == *build) {
            text.push_str(name);
            text.push('\t');
            text.push_str(&link);
            text.push('\n');
        } else {
            skipped.push(name.clone());
        }
    }
    (text, skipped)
}
