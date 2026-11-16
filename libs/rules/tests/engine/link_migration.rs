//! The link migration (ticket 298): an option's or feat's `aliases` name keys it had before a rename or a split. A
//! link's pick that no longer holds at its slot is read as the one live entry whose alias names its key and which
//! holds there; with none or several, the pick is refused as any stale pick is. 243's aliases (an old key no entry
//! has any more, read as the one option naming it) keep working, and an alias may now name a key still live (a split
//! that keeps the old key in one slot, as 294's domains will) or be shared by the copies of a split (as 293's
//! per-class selections will).

use rules::{Build, Engine, GameData, Prerequisite, StaleLink, encode_build};

use crate::support::*;

const FAST_STEALTH: &str = "rogue/rogue-talents/fast-stealth";
/// The slayer's own copy of Fast Stealth, in a split.
const SLAYER_STEALTH: &str = "slayer/slayer-talents/fast-stealth";
/// The rogue's copy of Fast Stealth, in a split that retires the old key.
const ROGUE_STEALTH: &str = "rogue/rogue-talents/quick-stealth";

fn open(engine: &Engine, build: &Build) -> Result<Build, StaleLink> {
    engine.open_link(&encode_build(build))
}

/// Rogue 2 with Fast Stealth as its talent.
fn rogue() -> Build {
    with(
        &build("elf", &times("rogue", 2)),
        selection_slot(2, 0),
        pick_option(FAST_STEALTH),
    )
}

/// Slayer 2 with the shared Fast Stealth as its talent.
fn slayer() -> Build {
    with(
        &build("elf", &times("slayer", 2)),
        selection_slot(2, 0),
        pick_option(FAST_STEALTH),
    )
}

/// The slayer's talents hold their own copy of Fast Stealth, naming the old key, in place of the rogue's.
fn split_for_the_slayer(data: &mut GameData) {
    let mut copy = option_mut(data, FAST_STEALTH).clone();
    copy.key = option(SLAYER_STEALTH);
    copy.aliases.insert(option(FAST_STEALTH));
    data.options.push(copy);
    let talents = selection_mut(data, "slayer/slayer-talents");
    talents.shared_options.remove(&option(FAST_STEALTH));
    talents.options.insert(option(SLAYER_STEALTH));
    // Slayer Camouflage needed the rogue's; it now needs the slayer's.
    option_mut(data, "slayer/slayer-talents/slayer-camouflage").prerequisite =
        Some(Prerequisite::SelectionOption(option(SLAYER_STEALTH)));
}

/// The rogue's Fast Stealth renamed: no entry has the old key any more. With `alias`, the new key names it.
fn rename_the_rogues(data: &mut GameData, alias: bool) {
    let stealth = option_mut(data, FAST_STEALTH);
    stealth.key = option(ROGUE_STEALTH);
    if alias {
        stealth.aliases.insert(option(FAST_STEALTH));
    }
    let talents = selection_mut(data, "rogue/rogue-talents");
    talents.options.remove(&option(FAST_STEALTH));
    talents.options.insert(option(ROGUE_STEALTH));
    let slayer = selection_mut(data, "slayer/slayer-talents");
    if slayer.shared_options.remove(&option(FAST_STEALTH)) {
        slayer.shared_options.insert(option(ROGUE_STEALTH));
    }
    let camouflage = option_mut(data, "slayer/slayer-talents/slayer-camouflage");
    if camouflage.prerequisite == Some(Prerequisite::SelectionOption(option(FAST_STEALTH))) {
        camouflage.prerequisite = Some(Prerequisite::SelectionOption(option(ROGUE_STEALTH)));
    }
}

// ---- options -----------------------------------------------------------------------------------------------------

#[test]
fn a_renamed_options_old_key_opens_on_its_new_key_with_an_alias_and_is_refused_without() {
    for slayer_too in [false, true] {
        let mut data = world();
        rename_the_rogues(&mut data, true);
        let engine = engine_on(data);
        let old = if slayer_too { slayer() } else { rogue() };
        let new = with(&old, selection_slot(2, 0), pick_option(ROGUE_STEALTH));
        assert_eq!(open(&engine, &old), Ok(new), "slayer: {slayer_too}");
    }

    let mut data = world();
    rename_the_rogues(&mut data, false);
    let engine = engine_on(data);
    assert_eq!(open(&engine, &rogue()), Err(StaleLink));
    assert_eq!(open(&engine, &slayer()), Err(StaleLink));
}

/// 294's shape: the old key stays live in one slot, and a copy naming it takes its place in another.
#[test]
fn a_split_that_keeps_the_old_key_opens_each_slot_on_the_entry_it_offers() {
    let mut data = world();
    split_for_the_slayer(&mut data);
    let engine = engine_on(data);
    assert_eq!(open(&engine, &rogue()), Ok(rogue()));
    assert_eq!(
        open(&engine, &slayer()),
        Ok(with(
            &slayer(),
            selection_slot(2, 0),
            pick_option(SLAYER_STEALTH)
        ))
    );

    // Rogue 2 then slayer 2, Fast Stealth in each class's slot: the slayer's slot reads its copy, the rogue's keeps
    // the old key.
    let both = with(
        &with(
            &build("elf", &["rogue", "rogue", "slayer", "slayer"]),
            selection_slot(2, 0),
            pick_option(FAST_STEALTH),
        ),
        selection_slot(4, 0),
        pick_option(FAST_STEALTH),
    );
    assert!(!engine.is_possible(&both));
    assert_eq!(
        open(&engine, &both),
        Ok(with(
            &both,
            selection_slot(4, 0),
            pick_option(SLAYER_STEALTH)
        ))
    );
}

/// 293's shape: the old key is gone and each class's copy names it; each slot reads the copy it offers.
#[test]
fn an_old_key_two_copies_name_opens_each_slot_on_the_copy_it_offers() {
    let mut data = world();
    split_for_the_slayer(&mut data);
    rename_the_rogues(&mut data, true);
    let engine = engine_on(data);
    assert_eq!(
        open(&engine, &rogue()),
        Ok(with(
            &rogue(),
            selection_slot(2, 0),
            pick_option(ROGUE_STEALTH)
        ))
    );
    assert_eq!(
        open(&engine, &slayer()),
        Ok(with(
            &slayer(),
            selection_slot(2, 0),
            pick_option(SLAYER_STEALTH)
        ))
    );
}

#[test]
fn a_pick_two_live_aliases_hold_at_is_refused() {
    let mut data = world();
    split_for_the_slayer(&mut data);
    rename_the_rogues(&mut data, true);
    // The slayer's talents offer the rogue's copy too: both copies hold at a slayer's slot.
    selection_mut(&mut data, "slayer/slayer-talents")
        .shared_options
        .insert(option(ROGUE_STEALTH));
    let engine = engine_on(data);
    for copy in [ROGUE_STEALTH, SLAYER_STEALTH] {
        let held = with(&slayer(), selection_slot(2, 0), pick_option(copy));
        assert!(engine.is_possible(&held), "{copy}");
    }
    assert_eq!(open(&engine, &slayer()), Err(StaleLink));
    // The rogue's slot offers only one of them.
    assert_eq!(
        open(&engine, &rogue()),
        Ok(with(
            &rogue(),
            selection_slot(2, 0),
            pick_option(ROGUE_STEALTH)
        ))
    );
}

#[test]
fn a_pick_no_alias_holds_at_is_refused() {
    let mut data = world();
    split_for_the_slayer(&mut data);
    // The slayer's copy needs rogue levels: it doesn't hold at a plain slayer's slot.
    option_mut(&mut data, SLAYER_STEALTH).prerequisite = Some(Prerequisite::ClassLevel {
        class: class("rogue"),
        level: 1,
    });
    let engine = engine_on(data);
    let copy = with(&slayer(), selection_slot(2, 0), pick_option(SLAYER_STEALTH));
    assert!(!engine.is_possible(&copy));
    assert_eq!(open(&engine, &slayer()), Err(StaleLink));
}

// ---- feats -------------------------------------------------------------------------------------------------------

/// Fighter 1 with `feat` in its general slot.
fn fighter_with(feat: &str) -> Build {
    with(
        &build("elf", &["fighter"]),
        general_slot(1),
        pick_feat(feat),
    )
}

/// The test world with a copy of Toughness under each of `keys`, each naming `quick-hands` as an alias when `alias`.
fn with_feats(keys: &[&str], alias: bool) -> GameData {
    let mut data = world();
    for key in keys {
        let mut copy = feat_mut(&mut data, "toughness").clone();
        copy.key = feat(key);
        if alias {
            copy.aliases.insert(feat("quick-hands"));
        }
        data.feats.push(copy);
    }
    data
}

#[test]
fn a_renamed_feats_old_key_opens_on_its_new_key_with_an_alias_and_is_refused_without() {
    let engine = engine_on(with_feats(&["nimble-hands"], true));
    assert_eq!(
        open(&engine, &fighter_with("quick-hands")),
        Ok(fighter_with("nimble-hands"))
    );
    let engine = engine_on(with_feats(&["nimble-hands"], false));
    assert_eq!(open(&engine, &fighter_with("quick-hands")), Err(StaleLink));
}

#[test]
fn a_split_feats_old_key_opens_on_the_one_copy_that_holds_at_its_slot() {
    let mut data = with_feats(&["nimble-hands", "rogues-hands"], true);
    feat_mut(&mut data, "rogues-hands").prerequisite = Some(Prerequisite::ClassLevel {
        class: class("rogue"),
        level: 1,
    });
    let engine = engine_on(data);
    assert_eq!(
        open(&engine, &fighter_with("quick-hands")),
        Ok(fighter_with("nimble-hands"))
    );

    // Two copies that both hold: refused.
    let engine = engine_on(with_feats(&["nimble-hands", "rogues-hands"], true));
    assert!(engine.is_possible(&fighter_with("rogues-hands")));
    assert_eq!(open(&engine, &fighter_with("quick-hands")), Err(StaleLink));
}
