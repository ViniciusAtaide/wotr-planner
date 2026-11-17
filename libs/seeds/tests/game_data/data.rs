//! The data as a whole: it passes the data check, links stay short, and a few facts of feats, lists and shared
//! talents and hexes hold.

use std::collections::{BTreeSet, HashMap};

use rules::{
    Ability, ClassChoice, ClassKey, FeatKey, FeatSource, Issue, LevelUp, MythicPathKey, Pick,
    SkillKey, SlotId, SlotKind,
};

use crate::support::*;

#[test]
fn the_game_data_passes_the_data_check() {
    assert_eq!(DATA.check(), []);
}

/// A link holds a 24-bit hash of each key, looked up among the keys of its kind (ticket 46). Two keys of one kind
/// with one hash would make every link naming either unreadable: if a data change ever makes two collide, widen the
/// hash or change its offset basis (`rules`' codec docs) and regenerate `V1_LINK`. An option's old keys
/// ([`aliases`](rules::SelectionOption::aliases), ticket 243) count too: one sharing a hash with another key of
/// its kind would make the link naming it unreadable.
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
        let alignment = [rules::Alignment::TrueNeutral, rules::Alignment::LawfulGood]
            .into_iter()
            .find(|&alignment| {
                let mut start = build.clone();
                align(&mut start, alignment);
                engine.is_possible(&start)
            });
        if let Some(alignment) = alignment {
            align(&mut build, alignment);
        }
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
    // Game: FighterProficiencies a23591cc77086494ba20880f87e73970 (ShieldsProficiency); ShieldFocus
    // ac57069b6bf8c904086171683992a92a (a shield proficiency; in FighterFeatSelection)
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
    // Game: WizardProgression 02f3049806dbf62459259ea8cae8f715 (ScribingScrolls a8a385bf53ee3454593ce9054375a2ec at 1)
    assert!(
        sheet
            .at(1)
            .unwrap()
            .has_feat(&FeatKey::new("scribe-scrolls"))
    );
    // No slot at level 1 is a list to pick it from. (The data's slot shape, not a game fact: the game's level-1 bonus
    // feat is WizardFeatSelection 8c3102c2ff3b69444b139a98521a4899, a fixed list the data holds as a category.)
    assert!(
        sheet.levels[0]
            .slots
            .iter()
            .all(|slot| !matches!(&slot.kind, SlotKind::Feat(key) if is_feat_list(any_selection(key.as_str())))),
        "{:?}",
        sheet.levels[0].slots
    );
}

#[test]
fn every_feat_list_holds_two_feats_or_more_or_every_feat() {
    // A lone feat is granted; the data check refuses a shorter list unless its one feat takes a value (a bonus feat
    // of the player's choosing). The real data has 84: 79 named lists (34 of them the bloodlines' and the ranger
    // styles' since ticket 30, 4 the kineticist elements' wild-talent bonus feats since ticket 31; ticket 32's three
    // slayer stage lists went in ticket 33, the slayer picking a ranger style), Gold Dragon's Dragon Feats (a list of
    // every feat), and the warpriest's Weapon Focus, the Master of All's Skill Focus, the Sword Saint's Weapon Focus
    // and the half-elf's Skill Focus. Ticket 61 adds the Titan Fighter's three Massive Attacks lists.
    // Ticket 301: each a selection of feats, every feat named. Ticket 306l: the magus's, the Arcane Rider's and the
    // warpriest's bonus lists (three more). Ticket 306m: the cavalier's, its Tactician's and the Gendarme's (three more).
    assert_eq!(feat_lists().count(), 84);
    let every: Vec<_> = feat_lists()
        .filter(|list| list.feats.len() + 3 == DATA.feats.len())
        .map(|list| {
            (
                list.key.as_str(),
                list.ignores_prerequisites,
                list.feats.len(),
            )
        })
        .collect();
    // Game: DragonLevel2FeatSelection a21acdafc0169f5488a9bd3256e2e65b (IgnorePrerequisites; its list, which the data
    // holds as every feat, is the game check's). Tickets 306q and 306r: every feat but Advanced Weapon Training 2 to 4,
    // which the game's list lacks (the 13 feats they added: the list holds the other 10).
    assert_eq!(every, [("gold-dragon/dragon-feats", true, 273)]);
    let one_choice: Vec<_> = feat_lists()
        .filter(|list| list.feats.len() < 2)
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
            // Game: Adaptability 26a668c5a8c22354bac67bcd42e09a3f (the 11 Skill Focus feats; it doesn't ignore
            // prerequisites, but Skill Focus has none: equivalent model)
            (
                "half-elf/adaptability",
                true,
                BTreeSet::from([FeatKey::new("skill-focus")])
            ),
            // Game: SwordSaintChosenWeaponSelection 3130e3083845b7b4fa2ac5b6f1a1ae6e (SwordSaintChosenWeaponFeature
            // c0b4ec0175e3ff940a45fc21f318a39a gives Weapon Focus in the chosen weapon, unchecked: equivalent model)
            (
                "magus/sword-saint/chosen-weapon",
                true,
                BTreeSet::from([FeatKey::new("weapon-focus")])
            ),
            // Game: MasterOfAllSkillFocus f2d2c1702d8a4cc6adfcbd4ebff8eee4 (SkillFocusSelection
            // c9629ef9eebb88b479b2fbc5e836656a; `IgnorePrerequisites` false, as for Adaptability)
            (
                "rogue/master-of-all/skill-focus",
                false,
                BTreeSet::from([FeatKey::new("skill-focus")])
            ),
            // Game: WarpriestWeaponFocusSelection ac384183dbfbbd7499410a21d749bef1 (Weapon Focus, IgnorePrerequisites)
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
    for selection in &DATA.selections {
        // An option the guide gives no heading of its own links to the page (Nature Bond's two): no anchor to copy.
        // The ten dragons and four elements share their bloodline's heading, and name it ("Draconic (Black)"); the
        // Primalist's two choices are described under their selection's heading.
        for option in DATA.options_of(selection).filter(|o| {
            o.guide.as_ref().unwrap().0.contains('#')
                // The shaman's eleven copies of witch hexes keep the witch's anchor (306j, design decision 6).
                && !(o.key.as_str().starts_with("shaman/hex/") && !o.aliases.is_empty())
                && !o.name.ends_with(')')
                && o.guide != selection.guide
        }) {
            if let Some(other) =
                anchors.insert(option.guide.as_ref().unwrap().0.clone(), option.key.clone())
            {
                panic!(
                    "{} and {other} are both {}",
                    option.key,
                    option.guide.as_ref().unwrap().0
                );
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
    // Game: FastStealth 97a6aa2b64dd21a4fac67658a91067d7 (Ranks 1); BlueprintFeatureSelection.CanSelect, WrathDecompiled
    // 6ce4a6b
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
    // Game: CripplingStrike b696bd7cb38da194fa3404032483d1db (Sneak Attack and AdvanceTalents, slayer 10:
    // SlayerProgression 1f35d11937a06624b8a60db68c7869a1); SlayerCamouflage fd374a48fd0f5e94a841cc3bf408c432
    // (TerrainMastery and AdvanceTalents)
    assert_eq!(at_10["rogue/rogue-talents/crippling-strike"], Ok(()));
    // Slayer Camouflage needs Terrain Mastery, now the rogue's talent.
    assert!(at_10["slayer/slayer-talents/slayer-camouflage"].is_err());
    assert!(offered(&slayer, 8)["rogue/rogue-talents/crippling-strike"].is_err());
}

/// The slayer's talent list at 10 is the game's 39: three combat styles, Slayer Camouflage, Slayer Trapfinding, 32 of
/// the rogue's talents and two archetypes' talents.
// Game: SlayerTalentSelection10 913b9cf25c9536949b43a2651b7ffb66 (39 entries; not the rogue's Intimidating Prowess,
// Weapon Snatcher, Skill Focus, Weapon Focus, Emboldening Strike or Petrifying Strike; ExecutionerPainfulStrike and
// VanguardTacticianExtraUseTalent in)
#[test]
fn a_slayers_talent_list_at_10_is_the_games_39() {
    assert_eq!(offered(&build("slayer", 10), 10).len(), 39);
}

#[test]
fn a_shaman_has_its_own_eleven_copies_of_witch_hexes() {
    let shaman = build("shaman", 2);
    let hexes = offered(&shaman, 2);
    // Game: ShamanHexSelection 4223fe18c75d4d14787af196a04e14e7 (the shaman's own hex blueprints, 11 sharing a witch
    // hex's name: Healing among them); ShamanHexEvilEyeFeature 890cd9309dc515c4199d811c36760d17 (no prerequisite).
    // Ticket 306j: each is the shaman's own entry (`shaman/hex/<name>`), the witch's key an alias; no `witch/hex/` key
    // is offered.
    assert!(!hexes.keys().any(|key| key.starts_with("witch/hex/")));
    assert_eq!(hexes["shaman/hex/healing-hex"], Ok(()));
    assert_eq!(hexes["shaman/hex/evil-eye"], Ok(()));
}

#[test]
fn a_sylvan_trickster_picks_a_rogue_talent_or_a_witch_hex() {
    let trickster = archetype_build("rogue", "rogue/sylvan-trickster", 10);
    let at_2 = offered(&trickster, 2);
    assert_eq!(at_2["rogue/rogue-talents/combat-trick"], Ok(()));
    assert_eq!(at_2["witch/hex/evil-eye"], Ok(()));
    // Major hexes from 10th level, grand hexes from 18th.
    // Game: SylvanTricksterArchetype 490394869f666c141bf8647b1a365220 (SylvanTricksterTalentSelection
    // 290bbcc3c3bb92144b853fd8fb8ff452 at every even level); WitchHexAgonyFeature de5f107ec5a340d4dbc333fd86a3cc06
    // (WitchMajorHex, at 10); WitchHexDeathCurseFeature b7cd7ba71151e354d9c8a1bff959d789 (WitchGrandHex, at 18)
    assert!(at_2["witch/hex/agony"].is_err());
    let at_10 = offered(&trickster, 10);
    assert_eq!(at_10["witch/hex/agony"], Ok(()));
    assert!(at_10["witch/hex/death-curse"].is_err());
    let at_18 = offered(&archetype_build("rogue", "rogue/sylvan-trickster", 18), 18);
    assert_eq!(at_18["witch/hex/death-curse"], Ok(()));
}

/// The game's Fey Tricks list has 64 options: 37 rogue talents (the rogue's 38 less Weapon Snatcher) and 27 hexes
/// (the witch's 26 and the Hex Channeler's channel).
// Game: SylvanTricksterTalentSelection 290bbcc3c3bb92144b853fd8fb8ff452
#[test]
fn a_sylvan_tricksters_fey_tricks_are_the_games_64() {
    let trickster = archetype_build("rogue", "rogue/sylvan-trickster", 2);
    assert_eq!(offered(&trickster, 2).len(), 64);
}

#[test]
fn an_underground_chemist_may_take_discoveries_in_place_of_rogue_talents_from_level_10() {
    let chemist = archetype_build("rogue", "rogue/underground-chemist", 12);
    // Game: UndergroundChemistArchetype b4f5741d42c4cb04abf5ab3fe33f6fc5 (UndergroundChemistSelection
    // c3d08fdb481aa2e4a9f71c4349141826 at 10 only: EnhancePotion and CripplingStrike among its options)
    assert!(!offered(&chemist, 8).contains_key("alchemist/discovery/enhance-potion"));
    let offered = offered(&chemist, 10);
    assert_eq!(offered["alchemist/discovery/enhance-potion"], Ok(()));
    assert_eq!(offered["rogue/rogue-talents/crippling-strike"], Ok(()));
}

/// The game gives the Underground Chemist its discovery choice at 10 only, of 29 options (6 discoveries and 23 rogue
/// talents); at 12 the rogue's own talents, no discovery, so no Mummification there after Preserve Organs at 10
/// (Mummification's own prerequisite, Preserve Organs and alchemist 10 or Underground Chemist rogue 10, agrees).
// Game: UndergroundChemistSelection c3d08fdb481aa2e4a9f71c4349141826 (29 options); UndergroundChemistArchetype
// b4f5741d42c4cb04abf5ab3fe33f6fc5 (adds it at 10 only); Mummification daf854d84d442e941aa3a2fdc041b37c
#[test]
fn an_underground_chemist_takes_a_discovery_at_10_only() {
    let mut chemist = archetype_build("rogue", "rogue/underground-chemist", 12);
    assert_eq!(offered(&chemist, 10).len(), 29);
    assert!(!offered(&chemist, 12).contains_key("alchemist/discovery/enhance-potion"));
    chemist.picks.insert(
        selection_slot(10),
        pick("alchemist/discovery/preserve-organs"),
    );
    assert!(!offered(&chemist, 12).contains_key("alchemist/discovery/mummification"));
}
