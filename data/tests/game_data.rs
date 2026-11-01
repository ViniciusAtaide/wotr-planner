//! The real `game-data/` files, read without a database: they pass the data check, and a few facts the planner
//! depends on hold in them.

use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use rules::{
    ArchetypeKey, Build, ClassChoice, ClassKey, Engine, FeatKey, FeatSlot, FeatSource, GameData,
    Issue, LevelUp, Pick, Problem, SelectionOptionKey, SlotId, SlotKind, SlotOption,
};

static DATA: LazyLock<GameData> =
    LazyLock::new(|| data::read_game_data_files(&data::game_data_dir()).expect("game-data reads"));

fn engine() -> Engine {
    Engine::new(DATA.clone())
}

/// A Build of `levels` levels of one class, with no picks.
fn build(class: &str, levels: usize) -> Build {
    let mut build = Build::default();
    build
        .class_choices
        .insert(ClassKey::new(class), ClassChoice::default());
    for _ in 0..levels {
        build.levels.push(LevelUp::new(ClassKey::new(class)));
    }
    build
}

/// A Build of one class taken with an archetype, `levels` levels, with no picks.
fn archetype_build(class: &str, archetype: &str, levels: usize) -> Build {
    let mut build = build(class, levels);
    build.class_choices.insert(
        ClassKey::new(class),
        ClassChoice {
            archetype: Some(ArchetypeKey::new(archetype)),
            ..ClassChoice::default()
        },
    );
    build
}

fn selection_slot(level: u8) -> SlotId {
    SlotId::Selection { level, index: 0 }
}

fn pick(key: &str) -> Pick {
    Pick::SelectionOption(SelectionOptionKey::new(key))
}

/// The options of a selection slot, by key, with their status.
fn offered(build: &Build, level: u8) -> HashMap<String, Result<(), Issue>> {
    engine()
        .options(build, &selection_slot(level))
        .unwrap_or_else(|| panic!("no selection slot at level {level}"))
        .into_iter()
        .map(|SlotOption { pick, status }| (pick.key().to_owned(), status))
        .collect()
}

#[test]
fn the_game_data_passes_the_data_check() {
    assert_eq!(DATA.check(), []);
}

#[test]
fn a_fighter_can_take_shield_focus() {
    // The fighter's first level grants Shield Proficiency, which Shield Focus needs.
    let mut fighter = build("fighter", 1);
    let bonus = SlotId::Feat {
        level: 1,
        source: FeatSource::Class,
        index: 0,
    };
    let shield_focus = Pick::Feat(FeatKey::new("shield-focus"));
    let options = engine().options(&fighter, &bonus).unwrap();
    let status = options
        .iter()
        .find(|option| option.pick == shield_focus)
        .map(|option| option.status.clone());
    assert_eq!(status, Some(Ok(())));

    fighter.picks.insert(bonus, shield_focus);
    assert_eq!(engine().validate(&fighter), []);
}

#[test]
fn a_wizards_scribe_scrolls_is_held_with_no_pick() {
    let sheet = engine().evaluate(&build("wizard", 1));
    assert!(
        sheet
            .at(1)
            .unwrap()
            .feats
            .contains(&FeatKey::new("scribe-scrolls"))
    );
    // No slot at level 1 is a list to pick it from.
    assert!(
        sheet.levels[0]
            .slots
            .iter()
            .all(|slot| !matches!(slot.kind, SlotKind::Feat(FeatSlot::OneOf { .. }))),
        "{:?}",
        sheet.levels[0].slots
    );
}

#[test]
fn every_feat_list_holds_two_feats_or_more() {
    // A lone feat is granted; the data check refuses a shorter list, and the real data has 43.
    assert_eq!(DATA.feat_lists().count(), 43);
    assert!(DATA.feat_lists().all(|list| list.feats.len() >= 2));
}

#[test]
fn no_selection_option_is_a_copy_of_another() {
    // A copy had its own key and the original's guide anchor; a selection shares the original instead.
    let mut anchors = HashMap::new();
    for selection in DATA
        .classes
        .iter()
        .chain(&DATA.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .chain(DATA.mythic_paths.iter().flat_map(|path| &path.selections))
    {
        for option in &selection.options {
            if let Some(other) = anchors.insert(option.guide.0.clone(), option.key.clone()) {
                panic!("{} and {other} are both {}", option.key, option.guide.0);
            }
        }
    }
}

#[test]
fn a_rogue_slayer_cannot_take_one_talent_twice() {
    let mut build = build("rogue", 2);
    build
        .class_choices
        .insert(ClassKey::new("slayer"), ClassChoice::default());
    build.levels.push(LevelUp::new(ClassKey::new("slayer")));
    build.levels.push(LevelUp::new(ClassKey::new("slayer")));
    let talent = pick("rogue/rogue-talents/combat-trick");
    build.picks.insert(selection_slot(2), talent.clone());
    assert_eq!(
        offered(&build, 4)["rogue/rogue-talents/combat-trick"],
        Err(Issue::AlreadyHeld)
    );
    build.picks.insert(selection_slot(4), talent);
    assert_eq!(
        engine().validate(&build),
        [Problem {
            at: rules::Choice::Slot(selection_slot(4)),
            issue: Issue::AlreadyHeld,
        }]
    );
}

#[test]
fn a_slayer_takes_rogue_talents_and_advanced_ones_from_slayer_10() {
    let slayer = build("slayer", 10);
    let at_10 = offered(&slayer, 10);
    assert_eq!(at_10["rogue/rogue-talents/crippling-strike"], Ok(()));
    // Slayer Camouflage needs Terrain Mastery, now the rogue's talent.
    assert!(at_10["slayer/slayer-talents/slayer-camouflage"].is_err());
    assert!(offered(&slayer, 8)["rogue/rogue-talents/crippling-strike"].is_err());
    // The slayer's talents are the 36 rogue talents plus its own two.
    assert_eq!(at_10.len(), 38);
}

#[test]
fn a_shaman_shares_eleven_witch_hexes() {
    let shaman = build("shaman", 2);
    let hexes = offered(&shaman, 2);
    let shared: BTreeSet<_> = hexes
        .keys()
        .filter(|key| key.starts_with("witch/hex/"))
        .cloned()
        .collect();
    assert_eq!(shared.len(), 11);
    assert!(shared.contains("witch/hex/healing-hex"));
    assert_eq!(hexes["witch/hex/evil-eye"], Ok(()));
}

#[test]
fn a_sylvan_trickster_picks_a_rogue_talent_or_a_witch_hex() {
    let trickster = archetype_build("rogue", "rogue/sylvan-trickster", 10);
    let at_2 = offered(&trickster, 2);
    assert_eq!(at_2["rogue/rogue-talents/combat-trick"], Ok(()));
    assert_eq!(at_2["witch/hex/evil-eye"], Ok(()));
    // Major hexes from 10th level, grand hexes from 18th.
    assert!(at_2["witch/hex/agony"].is_err());
    let at_10 = offered(&trickster, 10);
    assert_eq!(at_10["witch/hex/agony"], Ok(()));
    assert!(at_10["witch/hex/death-curse"].is_err());
    let at_18 = offered(&archetype_build("rogue", "rogue/sylvan-trickster", 18), 18);
    assert_eq!(at_18["witch/hex/death-curse"], Ok(()));
    // Every even level is a Fey Tricks pick: 36 rogue talents and 26 witch hexes.
    assert_eq!(at_2.len(), 62);
}

#[test]
fn an_underground_chemist_may_take_discoveries_in_place_of_rogue_talents_from_level_10() {
    let chemist = archetype_build("rogue", "rogue/underground-chemist", 12);
    assert!(!offered(&chemist, 8).contains_key("alchemist/discovery/enhance-potion"));
    for level in [10, 12] {
        let offered = offered(&chemist, level);
        assert_eq!(offered["alchemist/discovery/enhance-potion"], Ok(()));
        assert_eq!(offered["rogue/rogue-talents/crippling-strike"], Ok(()));
        // 36 rogue talents and 6 discoveries.
        assert_eq!(offered.len(), 42);
    }
    // Mummification needs Preserve Organs, and the chemist's Discovery stands in for alchemist level 10.
    let mut chemist = chemist;
    chemist.picks.insert(
        selection_slot(10),
        pick("alchemist/discovery/preserve-organs"),
    );
    assert_eq!(
        offered(&chemist, 12)["alchemist/discovery/mummification"],
        Ok(())
    );
}
