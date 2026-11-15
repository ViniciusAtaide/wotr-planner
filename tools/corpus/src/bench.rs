//! The benchmark's Builds (`bench-links.tsv`, ticket 56's `links.json`): one `name<TAB>link` line each, an empty link
//! being the blank Build. The seeds crate's real-data tests read the same file (`support::bench`).

use rules::{Build, Engine, Faith, GameData, LinkKeys, decode_build};

use crate::Named;

const BENCH: &str = include_str!("../bench-links.tsv");

/// The benchmark's Builds, named `bench <name>`, opened as the planner opens a link. One with an alignment and no
/// faith, which opens as a stale link since ticket 94, is an atheist.
pub fn bench_builds(engine: &Engine, data: &GameData) -> Vec<Named> {
    let keys = LinkKeys::new(data);
    BENCH
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (name, link) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("a bench line is `name<TAB>link`: {line:?}"));
            let build = if link.is_empty() {
                Build::default()
            } else {
                engine.open_link(link).unwrap_or_else(|_| {
                    let mut atheist =
                        decode_build(link, &keys).expect("the benchmark's links decode");
                    atheist.start.deity = Some(Faith::Atheism);
                    assert!(
                        engine.is_possible(&atheist),
                        "the benchmark's links open, as atheists if need be: {name}"
                    );
                    atheist
                })
            };
            (format!("bench {name}"), build)
        })
        .collect()
}
