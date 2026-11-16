//! The benchmark's Builds (`bench-links.tsv`, ticket 56's `links.json`): one `name<TAB>link` line each, an empty link
//! being the blank Build. The seeds crate's real-data tests read the same file (`support::bench`).

use rules::{Build, Engine, Faith, GameData, LinkKeys, StaleLink, decode_build, encode_build};

use crate::Named;

const BENCH: &str = include_str!("../bench-links.tsv");

/// The benchmark's lines, `(name, link)`, in file order; `blank`'s link is empty.
pub fn links() -> impl Iterator<Item = (&'static str, &'static str)> {
    BENCH.lines().filter(|line| !line.is_empty()).map(|line| {
        line.split_once('\t')
            .unwrap_or_else(|| panic!("a bench line is `name<TAB>link`: {line:?}"))
    })
}

/// A benchmark link's Build as it decodes, and as it opens. One with an alignment and no faith, which opens as a stale
/// link since ticket 94, is read as an atheist: it decodes as one here, and opens as one.
pub fn open(
    engine: &Engine,
    keys: &LinkKeys,
    name: &str,
    link: &str,
) -> (Build, Result<Build, StaleLink>) {
    let mut linked =
        decode_build(link, keys).unwrap_or_else(|_| panic!("the benchmark's links decode: {name}"));
    let opened = engine.open_link(link).or_else(|stale| {
        if linked.start.deity.is_some() {
            return Err(stale);
        }
        linked.start.deity = Some(Faith::Atheism);
        engine.open_link(&encode_build(&linked))
    });
    (linked, opened)
}

/// The benchmark's Builds, named `bench <name>`, opened as the planner opens a link, as atheists if need be
/// ([`open`]).
pub fn bench_builds(engine: &Engine, data: &GameData) -> Vec<Named> {
    let keys = LinkKeys::new(data);
    links()
        .map(|(name, link)| {
            let build = if link.is_empty() {
                Build::default()
            } else {
                open(engine, &keys, name, link).1.unwrap_or_else(|_| {
                    panic!("the benchmark's links open, as atheists if need be: {name}")
                })
            };
            (format!("bench {name}"), build)
        })
        .collect()
}
