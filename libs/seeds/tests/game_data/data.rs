//! The data as a whole: it passes the data check, links stay short, and a few facts of feats, lists and shared
//! talents and hexes hold.

use std::collections::{BTreeSet, HashMap};

use rules::{
    Ability, ClassChoice, ClassKey, FeatKey, FeatSlot, FeatSource, Issue, LevelUp, MythicPathKey,
    Pick, SkillKey, SlotId, SlotKind,
};

use crate::support::*;

#[test]
fn the_game_data_passes_the_data_check() {
    assert_eq!(DATA.check(), []);
}

/// A link holds a 24-bit hash of each key, looked up among the keys of its kind (ticket 46). Two keys of one kind
/// with one hash would make every link naming either unreadable: if a data change ever makes two collide, widen the
/// hash or change its offset basis (`rules`' codec docs) and regenerate `V1_LINK`.
#[test]
fn no_two_keys_of_one_kind_share_a_link_hash() {
    assert_eq!(rules::LinkKeys::new(&DATA).collisions(), []);
}

/// Every base class to 20, every slot filled with its first open option, two skill ranks a level, a background and
/// a path (ticket 46's measurement; true neutral, or lawful good for the monk and the paladin): each opens from its
/// link unchanged, and the link is short. Before ticket 46 these links were 629 to 894 characters.
#[test]
fn a_full_build_of_every_base_class_opens_from_a_short_link() {
    let engine = engine();
    let skills: Vec<SkillKey> = DATA.skills.iter().take(2).map(|s| s.key.clone()).collect();
    for class in &DATA.classes {
        let mut build = build(class.key.as_str(), 20);
        build.start.race = Some(rules::RaceKey::new("human"));
        build.start.alignment = [rules::Alignment::TrueNeutral, rules::Alignment::LawfulGood]
            .into_iter()
            .find(|&alignment| {
                let mut start = build.clone();
                start.start.alignment = Some(alignment);
                engine.is_possible(&start)
            });
        build.start.background = Some(rules::BackgroundKey::new("mercenary"));
        build.mythic_path = Some(MythicPathKey::new("angel"));
        for (index, level) in build.levels.iter_mut().enumerate() {
            if (index + 1) % 4 == 0 {
                level.ability_increase = Some(Ability::Strength);
            }
            for skill in &skills {
                level.skill_ranks.insert(skill.clone(), 1);
            }
        }
        // Picks open slots (selections grant more), so fill until nothing changes.
        loop {
            let sheet = engine.evaluate(&build);
            let open: Vec<SlotId> = sheet
                .levels
                .iter()
                .flat_map(|level| &level.slots)
                .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
                .map(|slot| slot.id.clone())
                .filter(|id| !build.picks.contains_key(id))
                .collect();
            let before = build.picks.len();
            for id in open {
                let first = engine
                    .options(&build, &id)
                    .unwrap_or_default()
                    .into_iter()
                    .find(|option| option.status.is_ok());
                if let Some(option) = first {
                    build.picks.insert(id, option.pick);
                }
            }
            if build.picks.len() == before {
                break;
            }
        }
        assert!(engine.is_possible(&build), "{}", class.key);
        let code = rules::encode_build(&build);
        assert!(
            code.len() <= 400,
            "{}: {} characters",
            class.key,
            code.len()
        );
        assert_eq!(engine.open_link(&code), Ok(build), "{}", class.key);
    }
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
    let shield_focus = Pick::feat(FeatKey::new("shield-focus"));
    let options = engine().options(&fighter, &bonus).unwrap();
    let status = options
        .iter()
        .find(|option| option.pick == shield_focus)
        .map(|option| option.status.clone());
    assert_eq!(status, Some(Ok(())));

    fighter.picks.insert(bonus, shield_focus);
    assert!(engine().is_possible(&fighter));
}

#[test]
fn a_wizards_scribe_scrolls_is_held_with_no_pick() {
    let sheet = engine().evaluate(&build("wizard", 1));
    assert!(
        sheet
            .at(1)
            .unwrap()
            .feats
            .contains_key(&FeatKey::new("scribe-scrolls"))
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
fn every_feat_list_holds_two_feats_or_more_or_every_feat() {
    // A lone feat is granted; the data check refuses a shorter list unless its one feat takes a value (a bonus feat
    // of the player's choosing). The real data has 78: 73 named lists (34 of them the bloodlines' and the ranger
    // styles' since ticket 30, 4 the kineticist elements' wild-talent bonus feats since ticket 31; ticket 32's three
    // slayer stage lists went in ticket 33, the slayer picking a ranger style), Gold Dragon's Dragon Feats (a list of
    // every feat), and the warpriest's Weapon Focus, the Master of All's Skill Focus, the Sword Saint's Weapon Focus
    // and the half-elf's Skill Focus. Ticket 61 adds the Titan Fighter's three Massive Attacks lists.
    assert_eq!(DATA.feat_lists().count(), 78);
    let every: Vec<_> = DATA
        .feat_lists()
        .filter(|list| list.every_feat)
        .map(|list| {
            (
                list.key.as_str(),
                list.ignores_prerequisites,
                list.feats.len(),
            )
        })
        .collect();
    assert_eq!(every, [("gold-dragon/dragon-feats", true, 0)]);
    let one_choice: Vec<_> = DATA
        .feat_lists()
        .filter(|list| !list.every_feat && list.feats.len() < 2)
        .map(|list| {
            (
                list.key.as_str(),
                list.ignores_prerequisites,
                list.feats.clone(),
            )
        })
        .collect();
    assert_eq!(
        one_choice,
        [
            (
                "half-elf/adaptability",
                true,
                BTreeSet::from([FeatKey::new("skill-focus")])
            ),
            (
                "magus/sword-saint/chosen-weapon",
                true,
                BTreeSet::from([FeatKey::new("weapon-focus")])
            ),
            (
                "rogue/master-of-all/skill-focus",
                true,
                BTreeSet::from([FeatKey::new("skill-focus")])
            ),
            (
                "warpriest/weapon-focus",
                true,
                BTreeSet::from([FeatKey::new("weapon-focus")])
            ),
        ]
    );
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
        // An option the guide gives no heading of its own links to the page (Nature Bond's two): no anchor to copy.
        // The ten dragons and four elements share their bloodline's heading, and name it ("Draconic (Black)"); the
        // Primalist's two choices are described under their selection's heading.
        for option in selection.options.iter().filter(|o| {
            o.guide.0.contains('#') && !o.name.ends_with(')') && o.guide != selection.guide
        }) {
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
    // Not Combat Trick, which repeats (ticket 47).
    let talent = pick("rogue/rogue-talents/fast-stealth");
    build.picks.insert(selection_slot(2), talent.clone());
    assert_eq!(
        offered(&build, 4)["rogue/rogue-talents/fast-stealth"],
        Err(Issue::AlreadyHeld)
    );
    build.picks.insert(selection_slot(4), talent);
    assert_eq!(
        why_not(&build, &selection_slot(4)),
        Some(Issue::AlreadyHeld)
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
    // The slayer's talents are the 36 rogue talents plus its own two and the three combat style feats (ticket 32).
    assert_eq!(at_10.len(), 41);
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
