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
        // An entry of the map's text starts after `{` or `, `: `Selection { .. }: ` is also the end of
        // `OptionSelection { .. }: `.
        let starts: Vec<usize> = (old_picks.match_indices(&at))
            .map(|(start, _)| start)
            .filter(|&start| {
                let before = &old_picks[..start];
                before.ends_with('{') || before.ends_with(", ")
            })
            .collect();
        if (starts.iter())
            .any(|&start| old_picks[start + at.len()..].starts_with(&format!("{pick:?}")))
        {
            continue;
        }
        // The old pick at this slot: its key is the first quoted text after the slot.
        let old =
            (starts.first()).and_then(|&start| old_picks[start + at.len()..].split('"').nth(1));
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

#[cfg(test)]
mod tests {
    use super::*;
    use rules::SelectionOptionKey;

    fn option(key: &str) -> Pick {
        Pick::SelectionOption {
            option: SelectionOptionKey::new(key),
            parameter: None,
        }
    }

    /// `Selection { level: 5, index: 0 }: ` ends `OptionSelection { level: 5, index: 0 }: `, and the old text of
    /// the second must not stand for the first.
    #[test]
    fn a_selection_slot_is_not_matched_inside_an_option_selection_slot() {
        let selection = SlotId::Selection { level: 5, index: 0 };
        let inner = SlotId::OptionSelection { level: 5, index: 0 };
        let mut old = Build::default();
        old.picks.insert(selection.clone(), option("old-key"));
        old.picks.insert(inner.clone(), option("kept"));
        let old = format!("{:?}", old.picks);

        let mut now = Build::default();
        now.picks.insert(selection.clone(), option("heir-key"));
        now.picks.insert(inner, option("kept"));
        assert_eq!(
            moved_since(&old, &now),
            Fate::Heirs(vec![(
                selection,
                "old-key".to_owned(),
                "heir-key".to_owned()
            )])
        );
    }
}
