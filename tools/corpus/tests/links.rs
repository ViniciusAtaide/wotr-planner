//! Every link keeps opening (ticket 298): each corpus Build's link and each benchmark link opens with the same picks.
//! A pick may come back under another key only through the link migration ([`rules::Engine::open_link`]): the key of
//! an option or feat whose `aliases` name the key the link holds. A ticket that renames or splits a key a link names
//! gives the new entry the old key as an alias, and this test then reads the old links through it.

use corpus::links::{Fate, assert_same_picks, fate};
use rules::{Engine, FeatKey, GameData, LinkKeys, Pick, StaleLink, decode_build, encode_build};

/// The benchmark's links as they were made, by name; `blank`, which has none, left out.
fn bench_links() -> impl Iterator<Item = (&'static str, &'static str)> {
    corpus::bench::links().filter(|(_, link)| !link.is_empty())
}

#[test]
fn every_corpus_and_bench_link_opens_with_the_same_picks() {
    let (engine, data) = corpus::engine().expect("the game data loads");

    // The corpus's Builds (its own, the benchmark's as opened and the full ones): each one's link opens as it, but
    // for those that aren't possible (161 at 7a793c6a, as the form proof's record has it; some stale on purpose),
    // whose links open nothing, as before.
    let threads = corpus::threads("LINKS_THREADS");
    let builds = corpus::builds::all(&engine, &data, threads);
    let opened = corpus::in_parallel(&builds, threads, |(_, build)| {
        engine.open_link(&encode_build(build))
    });
    let mut stale = 0;
    for ((name, build), opened) in builds.iter().zip(&opened) {
        match opened {
            Ok(opened) => assert_eq!(opened, build, "{name}"),
            Err(StaleLink) => {
                assert!(!engine.is_possible(build), "{name}");
                stale += 1;
            }
        }
    }
    println!(
        "{} corpus Builds, {stale} of them not possible",
        builds.len()
    );
    assert!(builds.len() > 2_000, "{}", builds.len());

    // The benchmark's links as they were made: each opens with the picks it decodes to.
    let keys = LinkKeys::new(&data);
    let mut links = 0;
    for (name, link) in bench_links() {
        let (linked, opened) = corpus::bench::open(&engine, &keys, name, link);
        let opened = opened.unwrap_or_else(|_| panic!("{name}: opens"));
        assert_same_picks(&data, name, &linked, &opened);
        links += 1;
    }
    // `start` to `legend40`.
    assert_eq!(links, 5);
}

/// The engine on the game data with `feat` renamed, the old key an alias when `alias`, with that data and its link
/// keys, if the data still passes the data check (nothing but the selections that list it names the feat).
fn renamed(data: &GameData, feat: &FeatKey, alias: bool) -> Option<(Engine, GameData, LinkKeys)> {
    let mut data = data.clone();
    let new = FeatKey::new(format!("{feat}-renamed"));
    let entry = data.feats.iter_mut().find(|entry| entry.key == *feat)?;
    entry.key = new.clone();
    if alias {
        entry.aliases.insert(feat.clone());
    }
    // The selections of feats that list it list it by its new key (ticket 301: the general one lists every feat).
    for selection in &mut data.selections {
        if selection.feats.remove(feat) {
            selection.feats.insert(new.clone());
        }
    }
    let checked = data.checked().ok()?;
    let data = GameData::clone(&checked);
    let keys = LinkKeys::new(&data);
    Some((Engine::new(checked), data, keys))
}

/// The test above passes a rename that keeps its old key as an alias, and fails one that doesn't: `legend40`'s first
/// feat that nothing else names, renamed in the real game data.
#[test]
fn a_planted_rename_opens_through_its_alias_and_is_refused_without_one() {
    let data = GameData::clone(
        &seeds::check_game_data_files(&seeds::game_data_dir()).expect("the game data loads"),
    );
    let (name, link) = bench_links()
        .find(|(name, _)| *name == "legend40")
        .expect("the benchmark has legend40");
    let decoded = decode_build(link, &LinkKeys::new(&data)).expect("legend40 decodes");
    let (slot, feat, (engine, renamed_data, keys)) = (decoded.picks.iter())
        .find_map(|(slot, pick)| match pick {
            Pick::Feat { feat, .. } => Some((slot, feat, renamed(&data, feat, true)?)),
            _ => None,
        })
        .expect("legend40 holds a feat nothing else, fighter20 included, names");

    let (linked, opened) = corpus::bench::open(&engine, &keys, name, link);
    let opened = opened.expect("the old link opens through the alias");
    assert_eq!(opened.picks[slot].key(), format!("{feat}-renamed"));
    assert_same_picks(&renamed_data, name, &linked, &opened);

    let (engine, _, keys) = renamed(&data, feat, false).expect("the rename checks");
    assert!(decode_build(link, &keys).is_err());
    assert!(engine.open_link(link).is_err());
}

/// A stage's base links and the refusals it lists (ticket 306c): `tests/links/<stage>-base.tsv` holds `name<TAB>link`
/// of each Build that opened on the stage's base, and `<stage>-refused.txt` the names, one a line, of those the
/// stage's data refuses on purpose (the game's lists no longer offer a pick).
struct Stage {
    base: &'static str,
    refused: &'static str,
}

/// `old` is the link keys the links were made with.
/// What is wrong with the links of `stage` on this engine, one line each: a link that no longer opens as it did and
/// isn't listed, a listed one that opens, a listed name that is no link. Else how many opened the same, through an
/// heir and as listed refusals.
fn problems(
    engine: &Engine,
    data: &GameData,
    old: &LinkKeys,
    stage: &Stage,
) -> Result<(usize, usize, usize), Vec<String>> {
    let keys = LinkKeys::new(data);
    let listed: Vec<&str> = stage.refused.lines().filter(|l| !l.is_empty()).collect();
    let mut found = Vec::new();
    let (mut same, mut heirs, mut refused) = (0, 0, 0);
    let names: Vec<&str> = (stage.base.lines())
        .filter_map(|line| line.split_once('\t').map(|(name, _)| name))
        .collect();
    for name in &listed {
        if !names.contains(name) {
            found.push(format!("{name}: listed as refused, and no base link"));
        }
    }
    for line in stage.base.lines().filter(|line| !line.is_empty()) {
        let (name, link) = line
            .split_once('\t')
            .expect("a base line is `name<TAB>link`");
        let old_picks = decode_build(link, old)
            .map_or_else(|_| String::new(), |build| format!("{:?}", build.picks));
        match (
            fate(engine, data, &keys, link, &old_picks),
            listed.contains(&name),
        ) {
            (Fate::Same, false) => same += 1,
            (Fate::Heirs(_), false) => heirs += 1,
            (Fate::Refused(_), true) => refused += 1,
            (Fate::Changed(why), _) => found.push(format!("{name}: opens as another Build: {why}")),
            (Fate::Refused(picks), false) => {
                found.push(format!("{name}: refused, and not listed: {picks:?}"))
            }
            (_, true) => found.push(format!("{name}: listed as refused, and it opens")),
        }
    }
    if found.is_empty() {
        Ok((same, heirs, refused))
    } else {
        Err(found)
    }
}

const FIXTURE: Stage = Stage {
    base: include_str!("links/fixture-base.tsv"),
    refused: include_str!("links/fixture-refused.txt"),
};

/// Ticket 306's stage S2: its base links, and the refusals its slices list as they make them.
const S2: Stage = Stage {
    base: include_str!("links/306-base.tsv"),
    refused: include_str!("links/306-refused.txt"),
};

/// The fixture's three links (the benchmark's `start`, `fighter20` and `legend40` as the corpus opens them) on the real game data: all three open the same;
/// with `legend40`'s feat renamed and the old key an alias, `legend40` opens through its heir; with the rename and no
/// alias it is refused, and then it must be listed (`fixture-refused.txt`), as it is.
#[test]
fn a_base_link_opens_the_same_through_an_heir_or_is_a_listed_refusal() {
    let (engine, data) = corpus::engine().expect("the game data loads");
    let unlisted = Stage {
        refused: "",
        ..FIXTURE
    };
    let old = LinkKeys::new(&data);
    assert_eq!(problems(&engine, &data, &old, &unlisted), Ok((3, 0, 0)));

    // A feat of legend40 that nothing else names, fighter20 included, so the rename touches that one link.
    let decoded = |name: &str| {
        let link = (FIXTURE.base.lines())
            .find_map(|line| line.strip_prefix(&format!("{name}\t")))
            .expect("the fixture has the link");
        decode_build(link, &old).expect("the fixture's links decode")
    };
    let (legend, fighter) = (decoded("legend40"), decoded("fighter20"));
    let feat = (legend.picks.values())
        .find_map(|pick| match pick {
            Pick::Feat { feat, .. }
                if !(fighter.picks.values()).any(|held| held.key() == feat.as_str()) =>
            {
                renamed(&data, feat, true).map(|_| feat.clone())
            }
            _ => None,
        })
        .expect("legend40 holds a feat nothing else names");

    let (aliased, aliased_data, _) = renamed(&data, &feat, true).expect("the rename checks");
    assert_eq!(
        problems(&aliased, &aliased_data, &old, &unlisted),
        Ok((2, 1, 0))
    );

    let (engine, data, _) = renamed(&data, &feat, false).expect("the rename checks");
    assert_eq!(problems(&engine, &data, &old, &FIXTURE), Ok((2, 0, 1)));
    let problems = problems(&engine, &data, &old, &unlisted).unwrap_err();
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].starts_with("legend40: refused, and not listed"));
}

/// A refusal listed for a Build whose link opens fails the check, and so does a listed name that is no link.
#[test]
fn a_listed_refusal_that_opens_fails() {
    let (engine, data) = corpus::engine().expect("the game data loads");
    let listed = Stage {
        refused: "legend40\nnot-a-build\n",
        ..FIXTURE
    };
    let old = LinkKeys::new(&data);
    let problems = problems(&engine, &data, &old, &listed).unwrap_err();
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].contains("not-a-build: listed as refused, and no base link"));
    assert!(problems[1].contains("legend40: listed as refused, and it opens"));
}

/// Ticket 306's stage S2: each link of its base opens with the same picks or through an heir, or is a listed
/// refusal, and no listed one opens. The heir count is of no use here: the keys are the current data's, which
/// already carry a pick with a single heir, so it counts as the same; only `compare`, with the base's picks, sees it. The base is the links of the corpus's open Builds when 306c was built, before
/// any slice of S2 changed the data (`FORM_PROOF_LINKS_OUT` of `compare` lists the heirs and the refusals).
#[test]
fn every_s2_base_link_opens_the_same_or_through_an_heir_or_is_listed() {
    let (engine, data) = corpus::engine().expect("the game data loads");
    let (same, heirs, refused) = problems(&engine, &data, &LinkKeys::new(&data), &S2)
        .unwrap_or_else(|found| {
            panic!("{} links:\n{}", found.len(), found.join("\n"));
        });
    println!("S2's base links: {same} the same, {heirs} through an heir, {refused} refused");
    assert!(same + heirs + refused > 1_800, "{}", same + heirs + refused);
}

/// The twelve rage powers the game's Battle Prowess list lacks (ticket 306h's review): a Battle Scion's refused pick of
/// any other rage power is a bug.
// Game: BattleProwessSelection 29b480a26a88f9e47a10d8c9fab84ee6 offers 21 of RagePowerSelection
// 28710502f46848d48b3f0d6132817c4e's 33; these are the other 12.
const BATTLE_PROWESS_LACKS: [&str; 12] = [
    "barbarian/rage-power/celestial-totem",
    "barbarian/rage-power/celestial-totem-greater",
    "barbarian/rage-power/celestial-totem-lesser",
    "barbarian/rage-power/clear-mind",
    "barbarian/rage-power/come-and-get-me",
    "barbarian/rage-power/crippling-blows",
    "barbarian/rage-power/daemon-totem",
    "barbarian/rage-power/daemon-totem-greater",
    "barbarian/rage-power/daemon-totem-lesser",
    "barbarian/rage-power/powerful-stance",
    "barbarian/rage-power/rolling-dodge",
    "barbarian/rage-power/wrecking-blows",
];

/// Ticket 306e: a Battle Scion's link from S2's base still opens with its picks. Its level-12 row holds the skald's
/// talents and its own selection, numbered by key, and the talents' key keeps its place in that order.
#[test]
fn a_battle_scion_link_from_the_s2_base_still_opens() {
    let (engine, data) = corpus::engine().expect("the game data loads");
    let keys = LinkKeys::new(&data);
    let mut scions = 0;
    for line in S2.base.lines().filter(|line| line.contains("battle-scion")) {
        let (name, link) = line.split_once('\t').expect("`name<TAB>link`");
        let old = decode_build(link, &keys)
            .map_or_else(|_| String::new(), |build| format!("{:?}", build.picks));
        // A pick the game's lists lack may be refused (listed in `306-refused.txt`): a rogue talent the skald's list
        // lacks, or a rage power Battle Prowess lacks (306h). A teamwork feat never is.
        match fate(&engine, &data, &keys, link, &old) {
            Fate::Same | Fate::Heirs(_) => {}
            Fate::Refused(picks) => assert!(
                picks
                    .iter()
                    .all(|(_, key)| key.starts_with("rogue/rogue-talents/")
                        || BATTLE_PROWESS_LACKS.contains(&key.as_str())),
                "{name}: {picks:?}"
            ),
            Fate::Changed(why) => panic!("{name}: {why}"),
        }
        scions += 1;
    }
    assert!(scions >= 5, "{scions}");
}
