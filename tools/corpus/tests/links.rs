//! Every link keeps opening (ticket 298): each corpus Build's link and each benchmark link opens with the same picks.
//! A pick may come back under another key only through the link migration ([`rules::Engine::open_link`]): the key of
//! an option or feat whose `aliases` name the key the link holds. A ticket that renames or splits a key a link names
//! gives the new entry the old key as an alias, and this test then reads the old links through it.

use rules::{
    Build, Engine, FeatKey, GameData, LinkKeys, Pick, StaleLink, decode_build, encode_build,
};

/// Whether `opened` is `linked`, or the same pick of an entry whose `aliases` name `linked`'s key.
fn carried(data: &GameData, linked: &Pick, opened: &Pick) -> bool {
    if linked == opened {
        return true;
    }
    let aliases: Vec<&str> = match (linked, opened) {
        (Pick::Feat { .. }, Pick::Feat { feat, .. }) => (data.feats.iter())
            .filter(|entry| entry.key == *feat)
            .flat_map(|entry| entry.aliases.iter().map(|key| key.as_str()))
            .collect(),
        (Pick::SelectionOption { .. }, Pick::SelectionOption { option, .. }) => {
            (data.options.iter())
                .filter(|entry| entry.key == *option)
                .flat_map(|entry| entry.aliases.iter().map(|key| key.as_str()))
                .collect()
        }
        _ => Vec::new(),
    };
    linked.parameter() == opened.parameter() && aliases.contains(&linked.key())
}

/// `opened` is `linked` but for picks the migration carried to another key.
fn assert_same_picks(data: &GameData, name: &str, linked: &Build, opened: &Build) {
    let slots = |build: &Build| build.picks.keys().cloned().collect::<Vec<_>>();
    assert_eq!(slots(opened), slots(linked), "{name}: the same slots");
    for (slot, pick) in &linked.picks {
        assert!(
            carried(data, pick, &opened.picks[slot]),
            "{name}: {slot:?} holds {pick:?}, opened as {:?}",
            opened.picks[slot]
        );
    }
    let without_picks = |build: &Build| Build {
        picks: Default::default(),
        ..build.clone()
    };
    assert_eq!(without_picks(opened), without_picks(linked), "{name}");
}

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
        .expect("legend40 holds a feat nothing else names");

    let (linked, opened) = corpus::bench::open(&engine, &keys, name, link);
    let opened = opened.expect("the old link opens through the alias");
    assert_eq!(opened.picks[slot].key(), format!("{feat}-renamed"));
    assert_same_picks(&renamed_data, name, &linked, &opened);

    let (engine, _, keys) = renamed(&data, feat, false).expect("the rename checks");
    assert!(decode_build(link, &keys).is_err());
    assert!(engine.open_link(link).is_err());
}
