//! What an old link becomes (tickets 298 and 306c): it opens with the same picks, or through the link migration
//! with some picks carried to the entry whose `aliases` name their key (an heir), or it is refused.
//!
//! The form proof's `compare` counts the three over the recorded Builds, and `tests/links.rs` pins a stage's
//! refusals with the same [`fate`].

use rules::{Build, Choice, Engine, GameData, LinkKeys, Pick, SlotId, decode_build};

/// Whether `opened` is `linked`, or the same pick of an entry whose `aliases` name `linked`'s key.
pub fn carried(data: &GameData, linked: &Pick, opened: &Pick) -> bool {
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

/// The slots whose pick `opened` holds under another key than `linked`, each with the old and the new key, when
/// `opened` is `linked` but for picks the migration carried; else why it is not.
pub fn heirs(
    data: &GameData,
    linked: &Build,
    opened: &Build,
) -> Result<Vec<(SlotId, String, String)>, String> {
    if !opened.picks.keys().eq(linked.picks.keys()) {
        return Err("not the same slots".to_owned());
    }
    let mut moved = Vec::new();
    for (slot, pick) in &linked.picks {
        let now = &opened.picks[slot];
        if !carried(data, pick, now) {
            return Err(format!("{slot:?} holds {pick:?}, opened as {now:?}"));
        }
        if pick != now {
            moved.push((slot.clone(), pick.key().to_owned(), now.key().to_owned()));
        }
    }
    let without_picks = |build: &Build| Build {
        picks: Default::default(),
        ..build.clone()
    };
    if without_picks(opened) != without_picks(linked) {
        return Err("the Build differs beyond its picks".to_owned());
    }
    Ok(moved)
}

/// `opened` is `linked` but for picks the migration carried to another key.
pub fn assert_same_picks(data: &GameData, name: &str, linked: &Build, opened: &Build) {
    if let Err(why) = heirs(data, linked, opened) {
        panic!("{name}: {why}");
    }
}

/// What opening an old link comes to.
#[derive(Debug, PartialEq, Eq)]
pub enum Fate {
    /// It opens with the picks it decodes to.
    Same,
    /// It opens, with these picks carried to another key: slot, old key, new key.
    Heirs(Vec<(SlotId, String, String)>),
    /// It doesn't open: the picks that fail, as slot and key (`-` for a choice that is no slot, or a link that
    /// doesn't decode).
    Refused(Vec<(String, String)>),
    /// It opens as something else than it decodes to, beyond the migration.
    Changed(String),
}

/// Opens `link` with `engine` (its `data` and link `keys`) and says what became of it. `old_picks` is the `Debug`
/// text of the picks the link held when it was made (the form proof records it): decoding already carries a pick
/// whose old key has one heir, so only the old text tells that a pick moved.
pub fn fate(
    engine: &Engine,
    data: &GameData,
    keys: &LinkKeys,
    link: &str,
    old_picks: &str,
) -> Fate {
    let Ok(decoded) = decode_build(link, keys) else {
        return Fate::Refused(vec![("-".to_owned(), "does not decode".to_owned())]);
    };
    match engine.open_link(link) {
        Ok(opened) => match heirs(data, &decoded, &opened) {
            Ok(_) => moved_since(old_picks, &opened),
            Err(why) => Fate::Changed(why),
        },
        Err(_) => Fate::Refused(
            (engine.failures(&decoded).into_iter())
                .map(|failure| match failure.choice {
                    Choice::Slot(slot) => {
                        let key = decoded.picks.get(&slot).map_or("-", Pick::key);
                        (format!("{slot:?}"), key.to_owned())
                    }
                    other => (format!("{other:?}"), "-".to_owned()),
                })
                .collect(),
        ),
    }
}

/// The picks of `opened` that differ from `old_picks`, the `Debug` text of the picks the link held: the same, or
/// heirs, each with the key it held.
fn moved_since(old_picks: &str, opened: &Build) -> Fate {
    let mut moved = Vec::new();
    for (slot, pick) in &opened.picks {
        let at = format!("{slot:?}: ");
        if old_picks.contains(&format!("{at}{pick:?}")) {
            continue;
        }
        // The old pick at this slot: its key is the first quoted text after the slot.
        let old = old_picks
            .split_once(&at)
            .and_then(|(_, rest)| rest.split('"').nth(1));
        match old {
            Some(old) => moved.push((slot.clone(), old.to_owned(), pick.key().to_owned())),
            None => return Fate::Changed(format!("{slot:?} is not a slot of the old picks")),
        }
    }
    if moved.is_empty() {
        Fate::Same
    } else {
        Fate::Heirs(moved)
    }
}
