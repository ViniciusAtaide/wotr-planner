//! The real `libs/game-data/` files, read without a database: they pass the data check, and a few facts the planner
//! depends on hold in them.

use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use rules::{
    Ability, Advancement, ArchetypeKey, ArmourType, Build, CharacterState, ClassChoice, ClassKey,
    CountsAs, CountsToward, Engine, FeatKey, FeatSlot, FeatSource, FeatureKey, GameData, Issue,
    LevelUp, MythicPathKey, Parameter, ParameterKind, Pick, Prerequisite, RankStatus, Repeat,
    SelectionOptionKey, SkillKey, SlotId, SlotKind, SlotOption, SpellSchool, Story, Unmet,
    WeaponKey, WeaponProficiency,
};

static DATA: LazyLock<GameData> = LazyLock::new(|| {
    seeds::read_game_data_files(&seeds::game_data_dir()).expect("game-data reads")
});

fn engine() -> Engine {
    Engine::new(DATA.clone())
}

/// Why the pick in `slot` can't be taken there, for a Build that only that pick makes impossible (it's possible
/// without it): what the slot's picker says of it, or `None` when the picker doesn't offer it at all.
fn why_not(build: &Build, slot: &SlotId) -> Option<Issue> {
    let engine = engine();
    assert!(!engine.is_possible(build), "the Build is possible");
    let mut rest = build.clone();
    let pick = rest.picks.remove(slot).expect("a pick in the slot");
    assert!(engine.is_possible(&rest), "impossible without {slot:?} too");
    let status = engine
        .options(&rest, slot)?
        .into_iter()
        .find(|option| option.pick == pick)?
        .status;
    Some(status.expect_err("the picker offers it"))
}

/// The Build without the pick in `slot`.
fn without(build: &Build, slot: &SlotId) -> Build {
    let mut build = build.clone();
    build.picks.remove(slot);
    build
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
    Pick::option(SelectionOptionKey::new(key))
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

// ---- Mythic path rules (ticket 26) ------------------------------------------------------------------------------

/// `levels` fighter levels on `path` from MR 3, with no picks.
fn mythic(path: &str, levels: usize) -> Build {
    let mut build = build("fighter", levels);
    build.mythic_path = Some(MythicPathKey::new(path));
    build
}

fn path_selection(rank: u8) -> SlotId {
    SlotId::PathSelection { rank, index: 0 }
}

#[test]
fn choosing_a_path_grants_its_mr_1_and_2_rows_twelve_rows_and_thirty_features() {
    let rows: Vec<_> = DATA
        .mythic_paths
        .iter()
        .flat_map(|path| &path.ranks)
        .filter(|row| row.rank < 3)
        .collect();
    assert_eq!(rows.len(), 12);
    assert_eq!(rows.iter().map(|row| row.features.len()).sum::<usize>(), 30);

    let sheet = engine().evaluate(&mythic("aeon", 9));
    let force_reality = FeatureKey::new("aeon/force-reality");
    // Granted at MR 1 and again at MR 2: held twice from MR 3 on, never before.
    assert_eq!(sheet.at(8).unwrap().feature_count(&force_reality), 0);
    assert_eq!(sheet.at(9).unwrap().feature_count(&force_reality), 2);
    assert_eq!(sheet.ranks[0].path, Some(MythicPathKey::new("aeon")));
}

#[test]
fn an_aeon_gaze_needing_rank_5_opens_at_mr_5() {
    let build = mythic("aeon", 14);
    let area_effects = pick("aeon/enforcing-gaze/enforcing-gaze-area-effects");
    let status = |rank| {
        engine()
            .options(&build, &path_selection(rank))
            .unwrap()
            .into_iter()
            .find(|option| option.pick == area_effects)
            .unwrap()
            .status
    };
    assert_eq!(
        status(4),
        Err(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::MythicRank(5),
            has: Some(4)
        }))
    );
    assert_eq!(status(5), Ok(()));
}

#[test]
fn a_legend_has_4_more_in_every_score_from_the_switch() {
    let mut build = mythic("trickster", 18);
    build.path_switch = Some(MythicPathKey::new("legend"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let (before, after) = (sheet.at(16).unwrap(), sheet.at(17).unwrap());
    for ability in [Ability::Strength, Ability::Charisma] {
        assert_eq!(
            after.ability_scores.get(ability),
            before.ability_scores.get(ability) + 4
        );
    }
    assert_eq!(after.mythic_rank, 3);
    // The Trickster's MR 1-2 rows stay, its MR 3-7 ones go.
    assert_eq!(
        after.feature_count(&FeatureKey::new("trickster/bit-of-fun")),
        1
    );
    assert_eq!(
        after.feature_count(&FeatureKey::new("trickster/sneaky-quack")),
        0
    );
}

#[test]
fn the_devil_switch_is_open_from_aeon_or_azata_only() {
    let open = |path: &str| {
        engine()
            .switch_options(&mythic(path, 0))
            .into_iter()
            .find(|option| option.path.as_str() == "devil")
            .unwrap()
            .status
            .is_ok()
    };
    assert!(open("aeon") && open("azata"));
    assert!(!open("trickster") && !open("lich"));
}

#[test]
fn a_devil_from_aeon_loses_the_gaze_picks_and_keeps_the_rest() {
    let mut build = mythic("aeon", 17);
    build.picks.insert(
        path_selection(3),
        pick("aeon/enforcing-gaze/enforcing-gaze-attack"),
    );
    build.path_switch = Some(MythicPathKey::new("devil"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let at_17 = sheet.at(17).unwrap();
    assert!(at_17.selection_options.is_empty());
    assert_eq!(
        at_17.feature_count(&FeatureKey::new("aeon/enforcing-gaze-extra-gaze")),
        0
    );
    assert_eq!(at_17.feature_count(&FeatureKey::new("aeon/aeon-bane")), 1);
    assert_eq!(sheet.ranks[2].status, RankStatus::PartlyDropped);
    assert_eq!(sheet.ranks[2].dropped_slots, [path_selection(3)]);
}

#[test]
fn a_gold_dragon_takes_three_dragon_feats_ignoring_prerequisites_and_rises_in_every_score() {
    let mut build = mythic("trickster", 19);
    build.path_switch = Some(MythicPathKey::new("gold-dragon"));
    // Improved Critical is taken for a weapon.
    for (index, pick) in [
        Pick::feat_for(FeatKey::new("improved-critical"), weapon("longsword")),
        Pick::feat(FeatKey::new("great-cleave")),
        Pick::feat(FeatKey::new("stat-focus")),
    ]
    .into_iter()
    .enumerate()
    {
        build.picks.insert(
            SlotId::PathFeat {
                rank: 9,
                index: index as u8,
            },
            pick,
        );
    }
    // Annihilating Breath needs MR 9: open at the MR 9 pick, not at MR 8's, though both come at 17.
    let annihilating = "gold-dragon/improved-gold-dragon-breath/annihilating-breath";
    build.picks.insert(path_selection(9), pick(annihilating));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let status = engine()
        .options(&build, &path_selection(8))
        .unwrap()
        .into_iter()
        .find(|option| option.pick == pick(annihilating))
        .unwrap()
        .status;
    assert!(status.is_err());
    // Perfect Body and Mind at 17 (every score at least 18), Perfect Soul at 19.
    let at_19 = sheet.at(19).unwrap();
    assert!(
        rules::AbilityScores::ABILITIES
            .iter()
            .all(|ability| at_19.ability_scores.get(*ability) >= 18)
    );
}

#[test]
fn all_skilled_grants_every_weapon_armour_and_shield_proficiency() {
    let mut build = mythic("azata", 11);
    build
        .picks
        .insert(path_selection(4), pick("azata/superpowers/all-skilled"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let feats = &sheet.at(11).unwrap().feats;
    for feat in [
        "simple-weapon-proficiency",
        "martial-weapons-proficiency",
        "exotic-weapon-proficiency",
        "armor-proficiency-light-armor",
        "armor-proficiency-medium-armor",
        "armor-proficiency-heavy-armor",
        "shield-proficiency",
        "tower-shield-proficiency",
    ] {
        assert!(feats.contains_key(&FeatKey::new(feat)), "{feat}");
    }
    assert!(!feats.contains_key(&FeatKey::new("armor-proficiency-light-barding")));
    // "Proficient with all types of weapons": Exotic Weapon Proficiency in every exotic weapon.
    let exotic = DATA
        .weapons
        .iter()
        .filter(|w| w.proficiency == Some(WeaponProficiency::Exotic))
        .map(|w| Parameter::Weapon(w.key.clone()))
        .collect::<BTreeSet<_>>();
    // The guide's 16 and the Sawtooth Saber (ticket 32).
    assert_eq!(exotic.len(), 17);
    assert_eq!(values_at(&build, 11, "exotic-weapon-proficiency"), exotic);
}

// ---- parameterised and repeatable feats (ticket 27) -----------------------------------------------------------

fn feat(key: &str) -> &'static rules::Feat {
    DATA.feats
        .iter()
        .find(|feat| feat.key.as_str() == key)
        .unwrap_or_else(|| panic!("no feat {key}"))
}

fn weapon(key: &str) -> Parameter {
    Parameter::Weapon(WeaponKey::new(key))
}

fn skill(key: &str) -> Parameter {
    Parameter::Skill(SkillKey::new(key))
}

fn feat_for(feat: &str, value: Parameter) -> Prerequisite {
    Prerequisite::FeatFor {
        feat: FeatKey::new(feat),
        value,
    }
}

fn general_slot(level: u8) -> SlotId {
    SlotId::Feat {
        level,
        source: FeatSource::General,
        index: 0,
    }
}

fn class_slot(level: u8) -> SlotId {
    SlotId::Feat {
        level,
        source: FeatSource::Class,
        index: 0,
    }
}

/// The values a feat is held for at the end of a level.
fn values_at(build: &Build, level: u8, key: &str) -> BTreeSet<Parameter> {
    engine()
        .evaluate(build)
        .at(level)
        .unwrap()
        .feat_values
        .get(&FeatKey::new(key))
        .map(|values| values.keys().cloned().collect())
        .unwrap_or_default()
}

#[test]
fn the_weapons_are_the_guides_proficiency_lists_and_what_weapon_focus_names() {
    let count = |proficiency| {
        DATA.weapons
            .iter()
            .filter(|w| w.proficiency == proficiency)
            .count()
    };
    assert_eq!(count(Some(WeaponProficiency::Simple)), 15);
    assert_eq!(count(Some(WeaponProficiency::Martial)), 27);
    // The Exotic Weapon Proficiency list and the Sawtooth Saber (Weapon Finesse's exotic list; ticket 32).
    assert_eq!(count(Some(WeaponProficiency::Exotic)), 17);
    // Unarmed strike, touch attacks and rays, the natural attacks the guide names (ticket 32: one value each),
    // Kinetic Blast and Bomb.
    let others: BTreeSet<&str> = DATA
        .weapons
        .iter()
        .filter(|w| w.proficiency.is_none())
        .map(|w| w.key.as_str())
        .collect();
    assert_eq!(
        others,
        BTreeSet::from([
            "bite",
            "bomb",
            "claw",
            "gore",
            "hoof",
            "kinetic-blast",
            "rays",
            "slam",
            "spike",
            "tail",
            "talon",
            "touch-attacks",
            "unarmed-strike",
            "wing"
        ])
    );
}

#[test]
fn feats_repeat_and_take_values_as_the_guide_says() {
    use ParameterKind as K;
    let shape = |key| (feat(key).parameter, feat(key).repeat);
    assert_eq!(shape("weapon-focus"), (Some(K::Weapon), Repeat::PerValue));
    assert_eq!(shape("skill-focus"), (Some(K::Skill), Repeat::PerValue));
    assert_eq!(
        shape("exotic-weapon-proficiency"),
        (Some(K::ExoticWeapon), Repeat::PerValue)
    );
    assert_eq!(
        shape("spell-focus"),
        (Some(K::SpellSchool), Repeat::PerValue)
    );
    assert_eq!(
        shape("elemental-focus"),
        (Some(K::Energy), Repeat::PerValue)
    );
    assert_eq!(
        shape("armor-focus"),
        (Some(K::ArmourType), Repeat::PerValue)
    );
    assert_eq!(
        shape("improved-critical"),
        (Some(K::Weapon), Repeat::PerValue)
    );
    assert_eq!(shape("extra-hex"), (None, Repeat::Times(3)));
    assert_eq!(shape("extra-ki"), (None, Repeat::Unlimited));
    assert_eq!(shape("magical-tail"), (None, Repeat::Times(8)));
    let repeating: Vec<&str> = DATA
        .feats
        .iter()
        .filter(|f| f.repeat != Repeat::Once)
        .map(|f| f.key.as_str())
        .collect();
    // The 15 parameterised feats once per value (ticket 32), Magical Tail and the 18 Extra feats the guide says
    // repeat (not Extra Bane Weapon: it doesn't say).
    assert_eq!(repeating.len(), 34, "{repeating:?}");
}

#[test]
fn weapon_focus_is_taken_once_per_weapon_and_specialization_needs_the_same_weapon() {
    let mut build = build("fighter", 4);
    build.picks.insert(
        class_slot(1),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword")),
    );
    build.picks.insert(
        general_slot(1),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("dagger")),
    );
    build.picks.insert(
        class_slot(2),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("dagger")),
    );
    build.picks.insert(
        class_slot(4),
        Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("rapier")),
    );
    assert_eq!(
        why_not(&without(&build, &class_slot(4)), &class_slot(2)),
        Some(Issue::AlreadyHeld)
    );
    assert_eq!(
        why_not(&without(&build, &class_slot(2)), &class_slot(4)),
        Some(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::FeatForSameValue(FeatKey::new("weapon-focus")),
            has: None,
        }))
    );
    build.picks.insert(
        class_slot(4),
        Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("longsword")),
    );
    build.picks.remove(&class_slot(2));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 4, "weapon-focus"),
        BTreeSet::from([weapon("dagger"), weapon("longsword")])
    );
}

#[test]
fn prerequisites_name_the_values_the_guide_names() {
    let has = |key: &str, needs: &Prerequisite| {
        let mut found = false;
        let mut stack: Vec<&Prerequisite> = feat(key).prerequisite.iter().collect();
        while let Some(prerequisite) = stack.pop() {
            found |= prerequisite == needs;
            if let Prerequisite::All(c) | Prerequisite::Any(c) | Prerequisite::AtLeast(_, c) =
                prerequisite
            {
                stack.extend(c);
            }
        }
        found
    };
    let same = |key: &str| Prerequisite::FeatForSameValue(FeatKey::new(key));
    assert!(has(
        "aldori-dueling-mastery",
        &feat_for("weapon-focus", weapon("dueling-sword"))
    ));
    assert!(has(
        "acrobatic-spellcaster",
        &feat_for("skill-focus", skill("mobility"))
    ));
    assert!(has(
        "slayers-feint",
        &feat_for("skill-focus", skill("mobility"))
    ));
    assert!(has(
        "augment-summoning",
        &feat_for(
            "spell-focus",
            Parameter::SpellSchool(SpellSchool::Conjuration)
        )
    ));
    for weapon_ in [
        "dagger",
        "estoc",
        "punching-dagger",
        "rapier",
        "sai",
        "starknife",
    ] {
        assert!(has(
            "fencing-grace",
            &feat_for("weapon-focus", weapon(weapon_))
        ));
    }
    assert!(has("greater-weapon-focus", &same("weapon-focus")));
    assert!(has("weapon-specialization", &same("weapon-focus")));
    assert!(has(
        "greater-weapon-specialization",
        &same("weapon-specialization")
    ));
    assert!(has("greater-spell-focus", &same("spell-focus")));
    assert!(has("greater-elemental-focus", &same("elemental-focus")));

    // The mythic Armor Focus feats name their armour type.
    let light = DATA
        .mythic_feats
        .iter()
        .find(|f| f.key.as_str() == "mythic-armor-focus-light-armor-assault")
        .unwrap();
    assert_eq!(
        light.prerequisite,
        Some(feat_for(
            "armor-focus",
            Parameter::ArmourType(ArmourType::Light)
        ))
    );
}

#[test]
fn an_aldori_swordlord_needs_weapon_focus_in_the_dueling_sword() {
    let swordlord = DATA
        .prestige_classes
        .iter()
        .find(|c| c.key.as_str() == "aldori-swordlord")
        .unwrap();
    let Some(Prerequisite::All(lines)) = &swordlord.prerequisite else {
        panic!("{:?}", swordlord.prerequisite)
    };
    assert!(lines.contains(&feat_for("weapon-focus", weapon("dueling-sword"))));
    assert!(!lines.contains(&Prerequisite::Feat(FeatKey::new("weapon-focus"))));
}

#[test]
fn a_warpriest_takes_weapon_focus_in_a_weapon_of_his_choosing() {
    // Base, Champion of the Faith and Feral Champion: a bonus slot of Weapon Focus alone, prerequisites ignored
    // (a warpriest 1 has BAB 0).
    for archetype in [
        None,
        Some("warpriest/champion-of-the-faith"),
        Some("warpriest/feral-champion"),
    ] {
        let mut build = match archetype {
            None => build("warpriest", 1),
            Some(archetype) => archetype_build("warpriest", archetype, 1),
        };
        let slot = class_slot(1);
        let options = engine().options(&build, &slot).unwrap();
        assert!(
            options
                .iter()
                .all(|option| option.pick.key() == "weapon-focus" && option.status.is_ok()),
            "{archetype:?}"
        );
        assert_eq!(options.len(), DATA.weapons.len());
        build.picks.insert(
            slot,
            Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword")),
        );
        assert!(engine().is_possible(&build), "{archetype:?}");
        assert_eq!(
            values_at(&build, 1, "weapon-focus"),
            BTreeSet::from([weapon("longsword")])
        );
    }
    // A Cult Leader doesn't get it.
    let build = archetype_build("warpriest", "warpriest/cult-leader", 1);
    assert_eq!(engine().options(&build, &class_slot(1)), None);
}

#[test]
fn a_master_of_all_takes_three_skill_focus_in_three_skills() {
    let mut build = archetype_build("rogue", "rogue/master-of-all", 15);
    // "Prerequisites: Half-Elf".
    build.start.race = Some(rules::RaceKey::new("half-elf"));
    for (level, skill_) in [(3, "mobility"), (9, "perception"), (15, "persuasion")] {
        build.picks.insert(
            class_slot(level),
            Pick::feat_for(FeatKey::new("skill-focus"), skill(skill_)),
        );
    }
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 15, "skill-focus"),
        BTreeSet::from([skill("mobility"), skill("perception"), skill("persuasion")])
    );
    // Not twice in one skill.
    build.picks.insert(
        class_slot(15),
        Pick::feat_for(FeatKey::new("skill-focus"), skill("mobility")),
    );
    assert_eq!(why_not(&build, &class_slot(15)), Some(Issue::AlreadyHeld));
}

#[test]
fn the_rogue_talents_weapon_focus_and_skill_focus_name_their_value() {
    let mut build = build("rogue", 2);
    let talent = |key: &str, value| {
        Pick::option_for(
            SelectionOptionKey::new(format!("rogue/rogue-talents/{key}")),
            value,
        )
    };
    build
        .picks
        .insert(selection_slot(2), talent("weapon-focus", weapon("rapier")));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 2, "weapon-focus"),
        BTreeSet::from([weapon("rapier")])
    );
    build
        .picks
        .insert(selection_slot(2), talent("skill-focus", skill("stealth")));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 2, "skill-focus"),
        BTreeSet::from([skill("stealth")])
    );
}

#[test]
fn rage_powers_discoveries_incense_and_mutations_repeat_as_the_guide_says() {
    let options: HashMap<&str, Repeat> = DATA
        .classes
        .iter()
        .chain(&DATA.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .flat_map(|selection| &selection.options)
        .filter(|option| option.repeat != Repeat::Once)
        .map(|option| (option.key.as_str(), option.repeat))
        .collect();
    let incense =
        |key: &str| format!("alchemist/incense-synthesizer/incense-fog/incense-fog-{key}");
    let mut expected: HashMap<String, Repeat> = [
        (
            "barbarian/rage-power/increased-damage-reduction",
            Repeat::Times(3),
        ),
        ("barbarian/rage-power/swift-foot", Repeat::Times(3)),
        (
            "stalwart-defender/defensive-powers/increased-damage-reduction",
            Repeat::Times(2),
        ),
        ("alchemist/discovery/preserve-organs", Repeat::Times(3)),
        (
            "shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-new-strain",
            Repeat::Unlimited,
        ),
        // "At 4th level and every four levels thereafter, a Primalist can choose ... two Rage Powers."
        (
            "bloodrager/primalist/primal-choices/rage-powers",
            Repeat::Unlimited,
        ),
    ]
    .into_iter()
    .map(|(key, repeat)| (key.to_owned(), repeat))
    .collect();
    for key in [
        "clarifying-incense",
        "neutralizing-incense",
        "rallying-incense",
        "toxic-incense",
        "unsettling-fog",
    ] {
        expected.insert(incense(key), Repeat::Times(3));
    }
    expected.insert(incense("improved-incense"), Repeat::Times(2));
    // "the Kineticist may choose a second Elemental Focus or increase her primary focus", "there isn't a third
    // level of specialization" (ticket 31).
    for element in ["air", "earth", "fire", "water"] {
        expected.insert(format!("kineticist/element/{element}"), Repeat::Times(2));
    }
    // The user's (ticket 47): the Skill Focus talents once per skill, Combat Trick any number of times.
    expected.insert(
        "kineticist/wild-talents/skill-focus".into(),
        Repeat::PerValue,
    );
    expected.insert("rogue/rogue-talents/skill-focus".into(), Repeat::PerValue);
    expected.insert("rogue/rogue-talents/combat-trick".into(), Repeat::Unlimited);
    let expected: HashMap<&str, Repeat> = expected
        .iter()
        .map(|(key, repeat)| (key.as_str(), *repeat))
        .collect();
    assert_eq!(options, expected);

    // A barbarian takes Increased Damage Reduction three times, not four.
    let mut build = build("barbarian", 16);
    build.start.alignment = Some(rules::Alignment::ChaoticNeutral);
    let rage_power_levels: Vec<u8> = (1..=16)
        .filter(|level| {
            engine()
                .evaluate(&build)
                .levels
                .get(usize::from(*level) - 1)
                .is_some_and(|l| {
                    l.slots.iter().any(|slot| {
                        slot.kind
                            == SlotKind::Selection(rules::FeatureSelectionKey::new(
                                "barbarian/rage-power",
                            ))
                    })
                })
        })
        .collect();
    let reduction = || pick("barbarian/rage-power/increased-damage-reduction");
    let later: Vec<u8> = rage_power_levels
        .iter()
        .copied()
        .filter(|level| *level >= 8)
        .take(4)
        .collect();
    assert_eq!(later.len(), 4, "{rage_power_levels:?}");
    for level in &later {
        build.picks.insert(selection_slot(*level), reduction());
    }
    assert_eq!(
        why_not(&build, &selection_slot(later[3])),
        Some(Issue::MaxTimes { times: 3 })
    );
}

// ---- class rules (ticket 28) ----------------------------------------------------------------------------------

fn class(key: &str) -> &'static rules::Class {
    DATA.classes
        .iter()
        .chain(&DATA.prestige_classes)
        .find(|class| class.key.as_str() == key)
        .unwrap_or_else(|| panic!("no class {key}"))
}

/// One Level-up per class key, with no picks.
fn classes(keys: &[&str]) -> Build {
    let mut build = Build::default();
    for key in keys {
        build.levels.push(LevelUp::new(ClassKey::new(*key)));
    }
    build
}

#[test]
fn signifer_loremaster_and_winter_witch_advance_one_class_arcane_or_divine() {
    let either: BTreeSet<&str> = DATA
        .classes
        .iter()
        .chain(&DATA.prestige_classes)
        .filter(|class| class.advances == Advancement::Either)
        .map(|class| class.key.as_str())
        .collect();
    assert_eq!(
        either,
        BTreeSet::from(["hellknight-signifer", "loremaster", "winter-witch"])
    );
    assert_eq!(class("mystic-theurge").advances, Advancement::Each);

    let mut both = classes(&[
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "cleric",
        "cleric",
        "cleric",
        "cleric",
        "cleric",
        "loremaster",
    ]);
    let choose = |build: &mut Build, advances: &[&str]| {
        build.class_choices.insert(
            ClassKey::new("loremaster"),
            ClassChoice {
                archetype: None,
                advances: advances.iter().map(|key| ClassKey::new(*key)).collect(),
            },
        );
    };
    // What the Loremaster's entry needs: Skill Focus (Knowledge (Arcana)), a metamagic feat and 3rd-level spells.
    both.picks.insert(
        general_slot(1),
        feat_pick(
            "skill-focus",
            Parameter::Skill(SkillKey::new("knowledge-arcana")),
        ),
    );
    both.picks.insert(
        general_slot(3),
        Pick::feat(FeatKey::new("metamagic-extend-spell")),
    );
    choose(&mut both, &["wizard", "cleric"]);
    assert!(!engine().is_possible(&both));
    choose(&mut both, &["cleric"]);
    assert!(engine().is_possible(&both));
}

#[test]
fn eldritch_knight_levels_count_as_fighter_levels_for_feats() {
    assert_eq!(
        class("eldritch-knight").counts_as,
        BTreeSet::from([CountsAs {
            class: ClassKey::new("fighter"),
            toward: CountsToward::FeatPrerequisites,
        }])
    );
    // Wizard 5, Fighter 1, Eldritch Knight 3: fighter level 4 for feats at character level 9.
    let mut build = classes(&[
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "fighter",
        "eldritch-knight",
        "eldritch-knight",
        "eldritch-knight",
    ]);
    build.picks.insert(
        class_slot(6),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword")),
    );
    let specialization = Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("longsword"));
    let status = |level| {
        engine()
            .options(&build, &general_slot(level))
            .unwrap()
            .into_iter()
            .find(|option| option.pick == specialization)
            .unwrap()
            .status
    };
    assert!(status(7).is_err(), "fighter 1 / Eldritch Knight 1");
    assert_eq!(status(9), Ok(()));
}

#[test]
fn winter_witch_levels_stack_with_witch_levels_for_major_and_grand_hexes() {
    let toward = |feature: &str| CountsAs {
        class: ClassKey::new("witch"),
        toward: CountsToward::Feature(FeatureKey::new(feature)),
    };
    assert_eq!(
        class("winter-witch").counts_as,
        BTreeSet::from([toward("witch/major-hexes"), toward("witch/grand-hexes")])
    );
    let mut keys = vec!["witch"; 9];
    keys.push("winter-witch");
    let sheet = engine().evaluate(&classes(&keys));
    let major = FeatureKey::new("witch/major-hexes");
    assert_eq!(sheet.at(9).unwrap().feature_count(&major), 0);
    assert_eq!(sheet.at(10).unwrap().feature_count(&major), 1);
}

#[test]
fn the_gold_dragon_breaths_story_conditions_are_notes_that_never_block() {
    let engine = engine();
    let breath = |key: &str| pick(&format!("gold-dragon/improved-gold-dragon-breath/{key}"));
    for (key, story) in [
        ("annihilating-breath", Story::VowOfDahak),
        ("decaying-breath", Story::VowOfDahak),
        ("enlightening-breath", Story::Uncorrupted),
        ("rejuvenating-breath", Story::Uncorrupted),
    ] {
        assert_eq!(
            engine.story_conditions(&breath(key)),
            BTreeSet::from([story]),
            "{key}"
        );
    }
    let mut build = mythic("trickster", 17);
    build.path_switch = Some(MythicPathKey::new("gold-dragon"));
    let options = engine.options(&build, &path_selection(9)).unwrap();
    for key in ["annihilating-breath", "enlightening-breath"] {
        let option = options.iter().find(|o| o.pick == breath(key)).unwrap();
        assert_eq!(option.status, Ok(()), "{key}");
    }
}

#[test]
fn a_crusaders_bonus_feats_still_skip_every_prerequisite_and_the_data_says_so() {
    // Weapon Focus needs BAB 1; a crusader cleric 1 has 0.
    let crusader = archetype_build("cleric", "cleric/crusader", 1);
    let focus = Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword"));
    let status = engine()
        .options(&crusader, &class_slot(1))
        .unwrap()
        .into_iter()
        .find(|option| option.pick == focus)
        .unwrap()
        .status;
    assert_eq!(status, Ok(()));

    let cleric =
        std::fs::read_to_string(seeds::game_data_dir().join("classes/cleric.ron")).unwrap();
    assert!(
        cleric.contains("// Crusader Bonus Feats:"),
        "the note on the Crusader's lists"
    );
}

#[test]
fn an_assassin_is_true_neutral_or_evil() {
    use rules::Alignment::*;
    let assassin = class("assassin").prerequisite.as_ref().unwrap();
    let allowed = [
        LawfulGood,
        NeutralGood,
        ChaoticGood,
        LawfulNeutral,
        TrueNeutral,
        ChaoticNeutral,
        LawfulEvil,
        NeutralEvil,
        ChaoticEvil,
    ]
    .into_iter()
    .filter(|alignment| {
        let state = CharacterState {
            alignment: Some(*alignment),
            character_level: 6,
            skill_ranks: [(SkillKey::new("stealth"), 5)].into(),
            ..CharacterState::default()
        };
        state.check(assassin).is_ok()
    })
    .collect::<Vec<_>>();
    assert_eq!(allowed, [TrueNeutral, LawfulEvil, NeutralEvil, ChaoticEvil]);
}

fn nature_bond(archetype: Option<&str>, option: Option<&str>) -> CharacterState {
    let mut druid = match archetype {
        Some(archetype) => archetype_build("druid", archetype, 1),
        None => build("druid", 1),
    };
    if let Some(option) = option {
        druid.picks.insert(
            selection_slot(1),
            pick(&format!("druid/nature-bond/{option}")),
        );
    }
    engine().evaluate(&druid).at(1).cloned().unwrap()
}

#[test]
fn a_druid_chooses_an_animal_companion_or_a_domain_at_level_1() {
    let offered: BTreeSet<String> = offered(&build("druid", 1), 1).into_keys().collect();
    assert_eq!(
        offered,
        BTreeSet::from([
            "druid/nature-bond/animal-companion".to_owned(),
            "druid/nature-bond/domain".to_owned()
        ])
    );
    // The archetypes whose table keeps both columns choose too; the others have their companion, their domain
    // or neither.
    let chooses = |archetype: &str| {
        let row = class("druid")
            .archetypes
            .iter()
            .find(|a| a.key.as_str() == archetype)
            .unwrap()
            .levels
            .iter()
            .find(|row| row.level == 1)
            .unwrap();
        row.selections
            .contains(&rules::FeatureSelectionKey::new("druid/nature-bond"))
    };
    for archetype in ["druid/defender-of-the-true-world", "druid/feyspeaker"] {
        assert!(chooses(archetype), "{archetype}");
    }
    for archetype in [
        "druid/blight-druid",
        "druid/drovier",
        "druid/elemental-rampager",
        "druid/primal-druid",
        "druid/winter-child",
    ] {
        assert!(!chooses(archetype), "{archetype}");
    }
}

#[test]
fn domain_zealot_needs_a_druids_domain_and_boon_companion_its_companion() {
    let mythic_ability = |key: &str| {
        DATA.mythic_abilities
            .iter()
            .find(|a| a.key.as_str() == key)
            .unwrap()
            .prerequisite
            .clone()
            .unwrap()
    };
    let domain_zealot = mythic_ability("domain-zealot");
    let boon_companion = feat("boon-companion").prerequisite.clone().unwrap();

    // Ticket 31: Domain Zealot needs a domain held, so the domain option needs its domain picked.
    assert!(
        nature_bond(None, Some("domain"))
            .check(&domain_zealot)
            .is_err()
    );
    let mut weather = build("druid", 1);
    weather
        .picks
        .insert(selection_slot(1), pick("druid/nature-bond/domain"));
    weather.picks.insert(
        SlotId::OptionSelection { level: 1, index: 0 },
        pick("cleric/domain/weather"),
    );
    let domain = engine().evaluate(&weather).at(1).cloned().unwrap();
    assert_eq!(domain.check(&domain_zealot), Ok(()));
    assert!(domain.check(&boon_companion).is_err());

    let companion = nature_bond(None, Some("animal-companion"));
    assert!(companion.check(&domain_zealot).is_err());
    assert_eq!(companion.check(&boon_companion), Ok(()));

    assert!(nature_bond(None, None).check(&domain_zealot).is_err());
    // A Blight Druid picks a domain and a Winter Child always has one; a Primal Druid never does.
    let mut blight = archetype_build("druid", "druid/blight-druid", 1);
    blight
        .picks
        .insert(selection_slot(1), pick("cleric/domain/death"));
    assert_eq!(
        engine()
            .evaluate(&blight)
            .at(1)
            .unwrap()
            .check(&domain_zealot),
        Ok(())
    );
    assert_eq!(
        nature_bond(Some("druid/winter-child"), None).check(&domain_zealot),
        Ok(())
    );
    assert!(
        nature_bond(Some("druid/primal-druid"), None)
            .check(&domain_zealot)
            .is_err()
    );
}

// ---- options with rows by option level (ticket 30) ------------------------------------------------------------

/// `build` with `option` picked at the selection slot `index` of `level`.
fn with_option(mut build: Build, level: u8, index: u8, option: &str) -> Build {
    build
        .picks
        .insert(SlotId::Selection { level, index }, pick(option));
    build
}

fn at(build: &Build, level: u8) -> CharacterState {
    engine().evaluate(build).at(level).cloned().unwrap()
}

fn holds(state: &CharacterState, feature: &str) -> bool {
    state.feature_count(&FeatureKey::new(feature)) > 0
}

/// The feats a feat slot offers, by key.
fn feats_offered(build: &Build, slot: &SlotId) -> BTreeSet<String> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .map(|option| option.pick.key().to_owned())
        .collect()
}

const ABYSSAL: &str = "sorcerer/bloodline/abyssal";

#[test]
fn a_sorcerer_holds_the_bloodlines_powers_at_1_3_9_15_and_20() {
    let build = with_option(build("sorcerer", 20), 1, 0, ABYSSAL);
    assert!(engine().is_possible(&build));
    let powers = [
        (1, "sorcerer/bloodline/abyssal/grow-claws"),
        (3, "sorcerer/bloodline/abyssal/demon-resistances"),
        (9, "sorcerer/bloodline/abyssal/strength-of-the-abyss"),
        (15, "sorcerer/bloodline/abyssal/added-summonings"),
        (20, "sorcerer/bloodline/abyssal/demonic-might"),
    ];
    for (level, power) in powers {
        if level > 1 {
            assert!(
                !holds(&at(&build, level - 1), power),
                "{power} before {level}"
            );
        }
        assert!(holds(&at(&build, level), power), "{power} at {level}");
    }
    // Wings at 15, and Strength of the Abyss +2 at 9, 13 and 17.
    assert!(at(&build, 15).feats.contains_key(&FeatKey::new("wings")));
    assert_eq!(at(&build, 8).ability_scores.strength, 10);
    assert_eq!(at(&build, 9).ability_scores.strength, 12);
    assert_eq!(at(&build, 17).ability_scores.strength, 16);
    assert_eq!(
        at(&build, 20).option_level(&SelectionOptionKey::new(ABYSSAL)),
        20
    );
}

#[test]
fn a_sorcerer_picks_the_bloodlines_feats_at_7_13_and_19() {
    let build = with_option(build("sorcerer", 19), 1, 0, ABYSSAL);
    let sheet = engine().evaluate(&build);
    for level in [7, 13, 19] {
        let slot = sheet
            .slot(&class_slot(level))
            .expect("a bloodline feat slot");
        assert_eq!(
            slot.kind,
            SlotKind::Feat(FeatSlot::OptionList {
                selection: rules::FeatureSelectionKey::new("sorcerer/bloodline"),
                at_level: None,
                label: None,
            })
        );
    }
    // "Augment Summoning, Bull Rush, Cleave, Empower Spell, Great Fortitude, Power Attack, Skill Focus (Lore:
    // Religion), Sunder Armor."
    let feats = feats_offered(&build, &class_slot(7));
    assert!(feats.contains("augment-summoning") && feats.contains("metamagic-empower-spell"));
    assert!(
        !feats.contains("combat-casting"),
        "an Arcane bloodline feat"
    );
}

#[test]
fn a_seeker_skips_the_bloodline_rows_at_3_9_and_15() {
    let build = with_option(
        archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 20),
        1,
        0,
        ABYSSAL,
    );
    assert!(engine().is_possible(&build));
    let at_20 = at(&build, 20);
    assert!(holds(&at_20, "sorcerer/bloodline/abyssal/grow-claws"));
    assert!(holds(&at_20, "sorcerer/bloodline/abyssal/demonic-might"));
    for skipped in [
        "sorcerer/bloodline/abyssal/demon-resistances",
        "sorcerer/bloodline/abyssal/strength-of-the-abyss",
        "sorcerer/bloodline/abyssal/added-summonings",
    ] {
        assert!(!holds(&at_20, skipped), "{skipped}");
    }
}

#[test]
fn a_sorcerer_5_dragon_disciple_5_has_bloodline_level_10() {
    let red = "sorcerer/bloodline/draconic-red";
    let mut build = build("sorcerer", 5);
    for _ in 0..5 {
        build
            .levels
            .push(LevelUp::new(ClassKey::new("dragon-disciple")));
    }
    for level in &mut build.levels[..5] {
        level
            .skill_ranks
            .insert(SkillKey::new("knowledge-arcana"), 1);
    }
    let build = with_option(build, 1, 0, red);
    // Only the minimum character level stands in the way of entering at 6 (it's checked before the level).
    let entry = engine()
        .class_options(&build, 6)
        .unwrap()
        .into_iter()
        .find(|option| option.class == ClassKey::new("dragon-disciple"))
        .unwrap()
        .status;
    assert_eq!(
        entry,
        Err(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::CharacterLevel(6),
            has: Some(5),
        }))
    );
    let at_10 = at(&build, 10);
    assert_eq!(at_10.option_level(&SelectionOptionKey::new(red)), 10);
    assert!(holds(&at_10, "sorcerer/bloodline/draconic/grow-claws"));
    // The Dragon Disciple's Strength +2 at its 2nd and 4th levels.
    assert_eq!(at_10.ability_scores.strength, 14);
}

#[test]
fn a_wizard_holds_the_schools_first_level_power_at_1() {
    let build = with_option(build("wizard", 1), 1, 0, "wizard/school/universalist");
    assert!(engine().is_possible(&build));
    assert!(holds(
        &at(&build, 1),
        "wizard/school/universalist/hand-of-the-apprentice"
    ));
}

#[test]
fn a_fixed_bloodline_is_held_with_no_pick() {
    let build = archetype_build("sorcerer", "sorcerer/empyreal-sorcerer", 3);
    let sheet = engine().evaluate(&build);
    assert!(
        sheet.levels[0]
            .slots
            .iter()
            .all(|slot| !matches!(slot.kind, SlotKind::Selection(_)))
    );
    let at_3 = at(&build, 3);
    assert_eq!(
        at_3.option_count(&SelectionOptionKey::new("sorcerer/bloodline/celestial")),
        1
    );
    assert!(holds(
        &at_3,
        "sorcerer/bloodline/celestial/celestial-resistances"
    ));
    // Its own Empyreal Arcana stands in for the Celestial's arcana.
    assert!(!holds(&at_3, "sorcerer/bloodline/celestial/angelic-arcana"));
}

#[test]
fn the_winter_witch_needs_a_shamans_frost_spirit() {
    let shaman = |spirit: &str| {
        let mut build = build("shaman", 6);
        for level in &mut build.levels {
            level
                .skill_ranks
                .insert(SkillKey::new("knowledge-arcana"), 1);
            level.skill_ranks.insert(SkillKey::new("lore-nature"), 1);
        }
        with_option(build, 1, 0, spirit)
    };
    let witch = |build: &Build| -> Result<(), Issue> {
        engine()
            .class_options(build, 7)
            .unwrap()
            .into_iter()
            .find(|o| o.class == ClassKey::new("winter-witch"))
            .unwrap()
            .status
    };
    assert_eq!(witch(&shaman("shaman/spirit/frost")), Ok(()));
    assert!(witch(&shaman("shaman/spirit/battle")).is_err());
}

#[test]
fn a_rangers_style_slot_picks_from_the_held_styles_list() {
    let build = with_option(
        build("ranger", 6),
        2,
        0,
        "ranger/combat-style-feat/archery-feats",
    );
    let at_2 = feats_offered(&build, &class_slot(2));
    assert!(at_2.contains("rapid-shot") && at_2.contains("dodge"));
    assert!(!at_2.contains("manyshot") && !at_2.contains("power-attack"));
    assert!(feats_offered(&build, &class_slot(6)).contains("manyshot"));
}

#[test]
fn a_hag_riven_picks_from_its_four_bloodlines() {
    let build = archetype_build("bloodrager", "bloodrager/hag-riven", 1);
    let offered: BTreeSet<String> = offered(&build, 1).into_keys().collect();
    assert!(offered.contains("bloodrager/bloodline/hag"));
    assert!(offered.contains("bloodrager/bloodline/elemental-fire"));
    assert!(!offered.contains("bloodrager/bloodline/abyssal"));
    assert_eq!(offered.len(), 7);
}

#[test]
fn a_cavalier_of_the_cockatrice_gets_dazzling_display_at_2() {
    let build = with_option(
        build("cavalier", 2),
        1,
        0,
        "cavalier/order/order-of-the-cockatrice",
    );
    assert!(
        at(&build, 2)
            .feats
            .contains_key(&FeatKey::new("dazzling-display"))
    );
}

// ---- domains, mysteries, blessings and elements (ticket 31) -------------------------------------------------

fn option_slot(level: u8, index: u8) -> SlotId {
    SlotId::OptionSelection { level, index }
}

fn with_pick(mut build: Build, slot: SlotId, option: &str) -> Build {
    build.picks.insert(slot, pick(option));
    build
}

/// The options a slot offers, by key, each once.
fn offered_at(build: &Build, slot: &SlotId) -> BTreeSet<String> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .map(|option| option.pick.key().to_owned())
        .collect()
}

fn has_class_skill(state: &CharacterState, skill: &str) -> bool {
    state.class_skills.contains(&SkillKey::new(skill))
}

/// Holds `skill` as a class skill from `level` on, not before.
fn class_skill_from_level(build: &Build, level: u8, skill: &str) {
    assert!(
        !has_class_skill(&at(build, level - 1), skill),
        "{skill} before {level}"
    );
    assert!(
        has_class_skill(&at(build, level), skill),
        "{skill} at {level}"
    );
}

/// Holds `feature` from `level` on, not before.
fn from_level(build: &Build, level: u8, feature: &str) {
    assert!(
        !holds(&at(build, level - 1), feature),
        "{feature} before {level}"
    );
    assert!(holds(&at(build, level), feature), "{feature} at {level}");
}

fn cleric(domains: [&str; 2], levels: usize) -> Build {
    let build = build("cleric", levels);
    let build = with_option(build, 1, 0, &format!("cleric/domain/{}", domains[0]));
    with_option(build, 1, 1, &format!("cleric/domain/{}", domains[1]))
}

#[test]
fn a_cleric_8_holds_both_domains_second_powers() {
    let build = cleric(["fire", "nobility"], 8);
    assert!(engine().is_possible(&build));
    for power in [
        "cleric/domain/fire/fire-bolt",
        "cleric/domain/nobility/inspiring-word",
    ] {
        assert!(holds(&at(&build, 1), power), "{power}");
    }
    from_level(&build, 6, "cleric/domain/fire/fire-resistance");
    from_level(&build, 8, "cleric/domain/nobility/inspiring-command");
    // Two picks of one domain aren't two domains.
    let twice = cleric(["fire", "fire"], 1);
    assert!(why_not(&twice, &SlotId::Selection { level: 1, index: 1 }).is_some());
}

#[test]
fn a_cleric_archetype_that_may_only_select_a_single_domain_picks_one() {
    let crusader = archetype_build("cleric", "cleric/crusader", 1);
    let sheet = engine().evaluate(&crusader);
    let domain_slots = sheet.levels[0]
        .slots
        .iter()
        .filter(|slot| {
            slot.kind == SlotKind::Selection(rules::FeatureSelectionKey::new("cleric/domain"))
        })
        .count();
    assert_eq!(domain_slots, 1);
    let ecclesitheurge = engine().evaluate(&archetype_build("cleric", "cleric/ecclesitheurge", 1));
    assert_eq!(
        ecclesitheurge.levels[0]
            .slots
            .iter()
            .filter(|slot| slot.kind
                == SlotKind::Selection(rules::FeatureSelectionKey::new("cleric/domain")))
            .count(),
        2
    );
}

#[test]
fn the_war_domain_grants_a_bonus_combat_feat_at_8() {
    let build = cleric(["war", "fire"], 8);
    let slot = SlotId::OptionFeat { level: 8, index: 0 };
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet.slot(&slot).expect("the War domain's slot").kind,
        SlotKind::Feat(FeatSlot::CombatFeat)
    );
    let feats = feats_offered(&build, &slot);
    assert!(feats.contains("power-attack"));
    assert!(!feats.contains("skill-focus"), "not a combat feat");
    assert!(
        engine()
            .evaluate(&cleric(["war", "fire"], 7))
            .slot(&SlotId::OptionFeat { level: 7, index: 0 })
            .is_none()
    );
}

#[test]
fn the_animal_domains_companion_comes_at_4_and_opens_boon_companion() {
    let boon_companion = feat("boon-companion").prerequisite.clone().unwrap();
    let animal = cleric(["animal", "fire"], 4);
    from_level(&animal, 4, "cleric/domain/animal/animal-companion");
    assert!(at(&animal, 3).check(&boon_companion).is_err());
    assert_eq!(at(&animal, 4).check(&boon_companion), Ok(()));
    // Another domain has none.
    assert!(
        at(&cleric(["fire", "luck"], 4), 4)
            .check(&boon_companion)
            .is_err()
    );
    // The Nature mystery's Animal Companion revelation.
    let nature = with_option(build("oracle", 1), 1, 0, "oracle/mystery/nature");
    let nature = with_pick(
        nature,
        revelation_slot(1),
        "oracle/revelations/nature/animal-companion",
    );
    assert!(engine().is_possible(&nature));
    assert_eq!(at(&nature, 1).check(&boon_companion), Ok(()));
}

/// A true neutral druid.
fn druid(archetype: Option<&str>, levels: usize) -> Build {
    let mut druid = match archetype {
        Some(archetype) => archetype_build("druid", archetype, levels),
        None => build("druid", levels),
    };
    druid.start.alignment = Some(rules::Alignment::TrueNeutral);
    druid
}

#[test]
fn a_druid_with_the_weather_domain_holds_lightning_lord_at_8_and_others_dont() {
    let domain = |which: &str, levels| {
        let druid = with_option(druid(None, levels), 1, 0, "druid/nature-bond/domain");
        with_pick(druid, option_slot(1, 0), &format!("cleric/domain/{which}"))
    };
    assert_eq!(
        offered_at(&domain("weather", 1), &option_slot(1, 0)).len(),
        8,
        "Air, Animal, Earth, Fire, Ice, Plant, Water and Weather"
    );
    let weather = domain("weather", 8);
    assert!(engine().is_possible(&weather));
    from_level(&weather, 8, "cleric/domain/weather/lightning-lord");
    // A druid with an animal companion holds no domain powers and no second companion at 4.
    let companion = with_option(
        build("druid", 8),
        1,
        0,
        "druid/nature-bond/animal-companion",
    );
    let state = at(&companion, 8);
    assert!(
        state
            .features
            .keys()
            .all(|f| !f.as_str().starts_with("cleric/domain/"))
    );
}

#[test]
fn a_blight_druid_skips_the_animal_domains_companion_and_a_winter_child_holds_ice_with_no_pick() {
    let blight = with_option(
        druid(Some("druid/blight-druid"), 4),
        1,
        0,
        "cleric/domain/animal",
    );
    assert!(engine().is_possible(&blight));
    assert!(!holds(
        &at(&blight, 4),
        "cleric/domain/animal/animal-companion"
    ));
    // The domain's Lore (Nature) sits on that skipped row too; a druid has it anyway.
    assert!(has_class_skill(&at(&blight, 4), "lore-nature"));

    let winter = druid(Some("druid/winter-child"), 8);
    assert!(engine().is_possible(&winter));
    assert!(holds(&at(&winter, 1), "cleric/domain/ice/icicle"));
    from_level(&winter, 8, "cleric/domain/ice/body-of-ice");
}

#[test]
fn an_inquisitors_domain_runs_on_inquisitor_levels() {
    let build = with_option(build("inquisitor", 6), 1, 0, "cleric/domain/luck");
    assert!(engine().is_possible(&build));
    from_level(&build, 6, "cleric/domain/luck/divine-fortune");
    let huntsmaster = archetype_build("inquisitor", "inquisitor/sacred-huntsmaster", 1);
    let domains = offered(&huntsmaster, 1);
    assert!(!domains.contains_key("cleric/domain/animal"));
    assert!(!domains.contains_key("cleric/domain/scalykind"));
    assert!(domains.contains_key("cleric/domain/luck"));
    // "Cannot select a domain."
    let faith_hunter =
        engine().evaluate(&archetype_build("inquisitor", "inquisitor/faith-hunter", 1));
    assert!(faith_hunter.levels[0].slots.iter().all(
        |slot| !matches!(&slot.kind, SlotKind::Selection(s) if s.as_str().ends_with("/domain"))
    ));
}

fn oracle(mystery: &str, levels: usize) -> Build {
    with_option(
        build("oracle", levels),
        1,
        0,
        &format!("oracle/mystery/{mystery}"),
    )
}

/// The oracle's level-1 slots: mystery, curse, revelation.
fn revelation_slot(level: u8) -> SlotId {
    SlotId::Selection {
        level,
        index: if level == 1 { 2 } else { 0 },
    }
}

#[test]
fn an_oracles_revelation_slot_offers_only_its_mysterys_revelations() {
    let battle = offered_at(&oracle("battle", 1), &revelation_slot(1));
    assert!(battle.contains("oracle/revelations/battle/weapon-mastery"));
    assert!(
        battle
            .iter()
            .all(|key| key.starts_with("oracle/revelations/battle/")),
        "{battle:?}"
    );
    let flame = offered_at(&oracle("flame", 1), &revelation_slot(1));
    assert!(
        flame
            .iter()
            .all(|key| key.starts_with("oracle/revelations/flame/")),
        "{flame:?}"
    );
    // "Prerequisites: Level 7"
    let mut early = oracle("battle", 3);
    early.picks.insert(
        revelation_slot(3),
        pick("oracle/revelations/battle/combat-healer"),
    );
    assert!(why_not(&early, &revelation_slot(3)).is_some());
}

#[test]
fn weapon_mastery_grants_improved_critical_at_8_in_its_weapon() {
    let mut build = oracle("battle", 12);
    build.picks.insert(
        revelation_slot(1),
        Pick::option_for(
            SelectionOptionKey::new("oracle/revelations/battle/weapon-mastery"),
            weapon("longsword"),
        ),
    );
    assert!(engine().is_possible(&build));
    assert!(at(&build, 1).has_feat_for(&FeatKey::new("weapon-focus"), &weapon("longsword")));
    let critical = FeatKey::new("improved-critical");
    assert!(!at(&build, 7).has_feat_for(&critical, &weapon("longsword")));
    assert!(at(&build, 8).has_feat_for(&critical, &weapon("longsword")));
    assert!(!at(&build, 8).has_feat_for(&critical, &weapon("dagger")));
    assert!(
        at(&build, 12).has_feat_for(&FeatKey::new("greater-weapon-focus"), &weapon("longsword"))
    );
}

#[test]
fn an_oracle_holds_the_mysterys_class_skill_and_final_revelation() {
    let build = oracle("battle", 20);
    assert!(has_class_skill(&at(&build, 1), "perception"));
    from_level(&build, 20, "oracle/mystery/battle/battle-final-revelation");
    // "he does not gain additional class skills from his choice of mystery" (Divine Herbalist).
    let herbalist = with_option(
        archetype_build("oracle", "oracle/divine-herbalist", 1),
        1,
        0,
        "oracle/mystery/battle",
    );
    assert!(engine().is_possible(&herbalist));
    assert!(!has_class_skill(&at(&herbalist, 1), "perception"));
    // "Must select the Wind Mystery": held with no pick.
    let whisperer = archetype_build("oracle", "oracle/wind-whisperer", 1);
    assert_eq!(
        at(&whisperer, 1).option_count(&SelectionOptionKey::new("oracle/mystery/wind")),
        1
    );
}

#[test]
fn a_warpriest_10_holds_both_major_blessings() {
    let build = build("warpriest", 10);
    let build = with_option(build, 1, 0, "warpriest/blessings/fire");
    let build = with_option(build, 1, 1, "warpriest/blessings/luck");
    assert!(engine().is_possible(&build));
    for blessing in ["fire", "luck"] {
        assert!(holds(
            &at(&build, 1),
            &format!("warpriest/blessings/{blessing}/minor-blessing")
        ));
        from_level(
            &build,
            10,
            &format!("warpriest/blessings/{blessing}/major-blessing"),
        );
    }
    // No more blessing picks at 10.
    let sheet = engine().evaluate(&build);
    assert!(sheet.levels[9].slots.iter().all(
        |slot| !matches!(&slot.kind, SlotKind::Selection(s) if s.as_str() == "warpriest/blessings")
    ));
    // "A Feral Champion must take the Animal Domain blessing and does not gain a second."
    let feral = archetype_build("warpriest", "warpriest/feral-champion", 10);
    assert!(engine().is_possible(&feral));
    from_level(
        &feral,
        10,
        "warpriest/blessings/animal-domain/major-blessing",
    );
}

fn kineticist(element: &str, levels: usize) -> Build {
    with_option(
        build("kineticist", levels),
        1,
        0,
        &format!("kineticist/element/{element}"),
    )
}

#[test]
fn a_kineticist_holds_the_elements_class_skills_list_and_defensive_talent() {
    let build = kineticist("earth", 2);
    assert!(engine().is_possible(&build));
    let one = at(&build, 1);
    assert!(has_class_skill(&one, "athletics"));
    assert!(has_class_skill(&one, "knowledge-world"));
    assert_eq!(
        one.option_feat_lists
            .get(&SelectionOptionKey::new("kineticist/element/earth")),
        Some(&rules::FeatListKey::new(
            "kineticist/element/earth/wild-talent-feats"
        ))
    );
    from_level(&build, 2, "kineticist/element/earth/flesh-of-stone");
    // The Kinetic Knight's comes at 4.
    let knight = with_option(
        archetype_build("kineticist", "kineticist/kinetic-knight", 4),
        1,
        0,
        "kineticist/element/earth",
    );
    assert!(engine().is_possible(&knight));
    from_level(&knight, 4, "kineticist/element/earth/flesh-of-stone");
}

#[test]
fn expanded_element_takes_a_second_element_or_the_first_again_but_not_three_times() {
    let mut build = kineticist("fire", 15);
    assert_eq!(offered_at(&build, &selection_slot(7)).len(), 4);
    build
        .picks
        .insert(selection_slot(7), pick("kineticist/element/fire"));
    build
        .picks
        .insert(selection_slot(15), pick("kineticist/element/water"));
    assert!(engine().is_possible(&build));
    assert_eq!(
        at(&build, 7).option_count(&SelectionOptionKey::new("kineticist/element/fire")),
        2
    );
    // The second element gives its list, not its defensive talent.
    assert!(!holds(
        &at(&build, 15),
        "kineticist/element/water/shroud-of-water"
    ));
    build
        .picks
        .insert(selection_slot(15), pick("kineticist/element/fire"));
    assert!(
        why_not(&build, &selection_slot(15)).is_some(),
        "a third Fire"
    );
}

#[test]
fn a_living_deity_takes_a_mystery_revelations_and_two_domains_at_20() {
    let deceiver = "arcanist/magic-deceiver";
    let build = archetype_build("arcanist", deceiver, 20);
    let build = with_option(
        build,
        1,
        0,
        "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity",
    );
    let build = with_pick(build, option_slot(1, 0), "oracle/mystery/battle");
    let build = with_pick(
        build,
        option_slot(1, 1),
        "oracle/revelations/battle/battlecry",
    );
    let build = with_pick(build, option_slot(1, 2), "oracle/oracles-curse/plagued");
    let build = with_pick(build, option_slot(20, 0), "cleric/domain/fire");
    let build = with_pick(build, option_slot(20, 1), "cleric/domain/luck");
    assert!(engine().is_possible(&build));
    from_level(&build, 20, "oracle/mystery/battle/battle-final-revelation");
    let twenty = at(&build, 20);
    assert!(holds(&twenty, "cleric/domain/fire/fire-resistance"));
    assert!(holds(&twenty, "cleric/domain/luck/divine-fortune"));
}

#[test]
fn domain_zealot_needs_a_domain_held_whoever_gave_it() {
    let domain_zealot = DATA
        .mythic_abilities
        .iter()
        .find(|a| a.key.as_str() == "domain-zealot")
        .unwrap()
        .prerequisite
        .clone()
        .unwrap();
    assert!(
        at(&build("cleric", 1), 1).check(&domain_zealot).is_err(),
        "no domain picked yet"
    );
    assert_eq!(
        at(&cleric(["fire", "luck"], 1), 1).check(&domain_zealot),
        Ok(())
    );
    let faith_hunter = archetype_build("inquisitor", "inquisitor/faith-hunter", 1);
    assert!(at(&faith_hunter, 1).check(&domain_zealot).is_err());
    assert_eq!(
        at(&archetype_build("druid", "druid/winter-child", 1), 1).check(&domain_zealot),
        Ok(())
    );
}

#[test]
fn a_hellknight_picks_an_order_unless_held_and_pentamic_faith_gives_a_domain() {
    let hellknight = class("hellknight");
    let order = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight/order")
        .expect("the Hellknight's order");
    assert!(order.only_without_option);
    assert_eq!(order.offered().len(), 7);
    assert!(hellknight.levels[0].selections.contains(&order.key));
    let faith = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight/disciplines")
        .unwrap()
        .options
        .iter()
        .find(|o| o.key.as_str() == "hellknight/disciplines/pentamic-faith")
        .unwrap();
    assert_eq!(
        faith.levels[0].selections,
        [rules::FeatureSelectionKey::new("hellknight/pentamic-faith")]
    );
    let domains = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight/pentamic-faith")
        .unwrap();
    assert_eq!(domains.offered().len(), 11);
    assert_eq!(
        domains.option_levels,
        BTreeSet::from([ClassKey::new("hellknight")])
    );
}

// ---- feat and rule follow-ups (ticket 32) ---------------------------------------------------------------------

fn race_slot() -> SlotId {
    SlotId::Feat {
        level: 1,
        source: FeatSource::Race,
        index: 0,
    }
}

fn possible(build: &Build) -> bool {
    engine().is_possible(build)
}

fn feat_pick(key: &str, value: Parameter) -> Pick {
    Pick::feat_for(FeatKey::new(key), value)
}

fn mythic_feat(key: &str) -> &'static rules::MythicFeat {
    DATA.mythic_feats
        .iter()
        .find(|feat| feat.key.as_str() == key)
        .unwrap_or_else(|| panic!("no mythic feat {key}"))
}

fn unmet(needs: Prerequisite, has: Option<u8>) -> Option<Issue> {
    Some(Issue::Unmet(Unmet::Condition { needs, has }))
}

#[test]
fn a_human_warpriest_takes_dazzling_display_at_1_on_the_class_slots_weapon_focus() {
    let mut build = build("warpriest", 1);
    build.start.race = Some(rules::RaceKey::new("human"));
    build.picks.insert(
        class_slot(1),
        feat_pick("weapon-focus", weapon("longsword")),
    );
    build
        .picks
        .insert(race_slot(), Pick::feat(FeatKey::new("dazzling-display")));
    assert!(possible(&build));
}

#[test]
fn a_human_druid_with_a_companion_takes_boon_companion_at_1() {
    let mut build = build("druid", 1);
    build.start.race = Some(rules::RaceKey::new("human"));
    build.start.alignment = Some(rules::Alignment::TrueNeutral);
    build.picks.insert(
        selection_slot(1),
        pick("druid/nature-bond/animal-companion"),
    );
    build
        .picks
        .insert(race_slot(), Pick::feat(FeatKey::new("boon-companion")));
    assert!(possible(&build));
}

#[test]
fn every_parameterised_feat_repeats_once_per_value() {
    let parameterised: Vec<&rules::Feat> = DATA
        .feats
        .iter()
        .filter(|f| f.parameter.is_some())
        .collect();
    assert_eq!(parameterised.len(), 15);
    for feat in parameterised {
        assert_eq!(feat.repeat, Repeat::PerValue, "{}", feat.key);
    }
}

#[test]
fn weapon_focus_and_greater_weapon_focus_each_in_two_weapons() {
    let mut fighter = build("fighter", 10);
    for (slot, key, value) in [
        (class_slot(1), "weapon-focus", "longsword"),
        (general_slot(1), "weapon-focus", "dagger"),
        (class_slot(8), "greater-weapon-focus", "longsword"),
        (class_slot(10), "greater-weapon-focus", "dagger"),
    ] {
        fighter.picks.insert(slot, feat_pick(key, weapon(value)));
    }
    assert!(possible(&fighter));
    assert_eq!(
        values_at(&fighter, 10, "greater-weapon-focus"),
        BTreeSet::from([weapon("dagger"), weapon("longsword")])
    );
    // Not twice in one weapon.
    fighter.picks.insert(
        class_slot(10),
        feat_pick("greater-weapon-focus", weapon("longsword")),
    );
    assert_eq!(why_not(&fighter, &class_slot(10)), Some(Issue::AlreadyHeld));
}

#[test]
fn exotic_weapon_proficiency_can_take_the_sawtooth_saber() {
    // Exotic Weapon Proficiency can take the Sawtooth Saber.
    let mut fighter = build("fighter", 1);
    fighter.picks.insert(
        general_slot(1),
        feat_pick("exotic-weapon-proficiency", weapon("sawtooth-saber")),
    );
    assert!(possible(&fighter));
}

#[test]
fn armor_focus_heavy_needs_heavy_armour_proficiency() {
    use rules::ArmourType as A;
    let focus = |armour| feat_pick("armor-focus", Parameter::ArmourType(armour));
    // A fighter has every armour proficiency.
    let mut fighter = build("fighter", 1);
    fighter.picks.insert(class_slot(1), focus(A::Heavy));
    assert!(possible(&fighter));
    // A cleric has light and medium only.
    let mut cleric = build("cleric", 3);
    cleric.picks.insert(general_slot(3), focus(A::Heavy));
    assert_eq!(
        why_not(&cleric, &general_slot(3)),
        unmet(
            Prerequisite::FeatIfValue {
                value: Parameter::ArmourType(A::Heavy),
                feat: FeatKey::new("armor-proficiency-heavy-armor"),
            },
            None
        )
    );
    cleric.picks.insert(general_slot(3), focus(A::Medium));
    assert!(possible(&cleric));
}

#[test]
fn improved_critical_mythic_in_a_longsword_needs_improved_critical_in_a_longsword() {
    let mut fighter = build("fighter", 11);
    fighter.picks.insert(
        class_slot(8),
        feat_pick("improved-critical", weapon("longsword")),
    );
    let mythic = SlotId::Mythic { rank: 4 };
    fighter.picks.insert(
        mythic.clone(),
        Pick::mythic_feat_for(
            rules::MythicFeatKey::new("improved-critical-mythic"),
            weapon("longsword"),
        ),
    );
    assert!(possible(&fighter));
    fighter.picks.insert(
        mythic.clone(),
        Pick::mythic_feat_for(
            rules::MythicFeatKey::new("improved-critical-mythic"),
            weapon("dagger"),
        ),
    );
    assert_eq!(
        why_not(&fighter, &mythic),
        unmet(
            Prerequisite::FeatForSameValue(FeatKey::new("improved-critical")),
            None
        )
    );
}

#[test]
fn the_mythic_feats_that_pick_a_value_take_one_once_per_value() {
    use ParameterKind as K;
    for (key, kind, base) in [
        (
            "improved-critical-mythic",
            K::Weapon,
            Some("improved-critical"),
        ),
        ("spell-focus-mythic", K::SpellSchool, Some("spell-focus")),
        ("skill-focus-mythic", K::Skill, Some("skill-focus")),
        ("expanded-arsenal", K::SpellSchool, None),
        ("school-mastery", K::SpellSchool, None),
    ] {
        let feat = mythic_feat(key);
        assert_eq!(
            (feat.parameter, feat.repeat),
            (Some(kind), Repeat::PerValue),
            "{key}"
        );
        assert_eq!(
            feat.prerequisite,
            base.map(|base| Prerequisite::FeatForSameValue(FeatKey::new(base))),
            "{key}"
        );
    }
    // The Trickster's chain: each needs the one before in the same weapon.
    for (key, base) in [
        ("improved-improved-critical", "improved-critical"),
        (
            "improved-improved-improved-critical",
            "improved-improved-critical",
        ),
    ] {
        let Some(Prerequisite::All(children)) = &feat(key).prerequisite else {
            panic!("{key}");
        };
        assert!(children.contains(&Prerequisite::FeatForSameValue(FeatKey::new(base))));
        assert_eq!(feat(key).parameter, Some(K::Weapon));
    }
}

#[test]
fn a_mantis_zealot_holds_its_sawtooth_saber_feats_at_1_4_8_and_12() {
    // Lawful neutral: a Mantis Zealot is LE, LN or NE (ticket 63).
    let mut zealot = archetype_build("warpriest", "warpriest/mantis-zealot", 12);
    zealot.start.alignment = Some(rules::Alignment::LawfulNeutral);
    assert!(possible(&zealot));
    let saber = || BTreeSet::from([weapon("sawtooth-saber")]);
    for (level, key) in [
        (1, "weapon-focus"),
        (1, "exotic-weapon-proficiency"),
        (4, "weapon-specialization"),
        (8, "greater-weapon-focus"),
        (12, "greater-weapon-specialization"),
    ] {
        assert_eq!(values_at(&zealot, level, key), saber(), "{key} at {level}");
        if level > 1 {
            assert_eq!(
                values_at(&zealot, level - 1, key),
                BTreeSet::new(),
                "{key} before {level}"
            );
        }
    }
}

#[test]
fn the_chirurgeon_and_the_enlightened_philosopher_hold_skill_focus_in_their_skill() {
    let chirurgeon = archetype_build("alchemist", "alchemist/chirurgeon", 5);
    assert_eq!(values_at(&chirurgeon, 4, "skill-focus"), BTreeSet::new());
    assert_eq!(
        values_at(&chirurgeon, 5, "skill-focus"),
        BTreeSet::from([skill("lore-religion")])
    );
    let philosopher = archetype_build("oracle", "oracle/enlightened-philosopher", 1);
    assert_eq!(
        values_at(&philosopher, 1, "skill-focus"),
        BTreeSet::from([skill("knowledge-world")])
    );
}

#[test]
fn a_zen_archers_way_of_the_bow_is_weapon_focus_in_a_bow_and_specialization_in_it_at_6() {
    let way = "monk/zen-archer/way-of-the-bow/bow";
    let mut archer = archetype_build("monk", "monk/zen-archer", 6);
    archer.start.alignment = Some(rules::Alignment::LawfulNeutral);
    let offered: Vec<Pick> = engine()
        .options(&archer, &selection_slot(2))
        .expect("a pick at 2")
        .into_iter()
        .map(|option| option.pick)
        .collect();
    assert_eq!(
        offered,
        [
            Pick::option_for(SelectionOptionKey::new(way), weapon("long-bow")),
            Pick::option_for(SelectionOptionKey::new(way), weapon("short-bow")),
        ]
    );
    archer.picks.insert(
        selection_slot(2),
        Pick::option_for(SelectionOptionKey::new(way), weapon("short-bow")),
    );
    assert!(possible(&archer));
    assert_eq!(
        values_at(&archer, 2, "weapon-focus"),
        BTreeSet::from([weapon("short-bow")])
    );
    assert_eq!(
        values_at(&archer, 5, "weapon-specialization"),
        BTreeSet::new()
    );
    assert_eq!(
        values_at(&archer, 6, "weapon-specialization"),
        BTreeSet::from([weapon("short-bow")])
    );
}

#[test]
fn a_battle_master_shaman_specialises_at_8_and_greater_focuses_at_16_in_one_weapon() {
    let mut shaman = build("shaman", 16);
    shaman
        .picks
        .insert(selection_slot(1), pick("shaman/spirit/battle"));
    shaman.picks.insert(
        selection_slot(2),
        Pick::option_for(
            SelectionOptionKey::new("shaman/hex/battle-master"),
            weapon("spear"),
        ),
    );
    assert!(possible(&shaman));
    assert_eq!(
        values_at(&shaman, 7, "weapon-specialization"),
        BTreeSet::new()
    );
    assert_eq!(
        values_at(&shaman, 8, "weapon-specialization"),
        BTreeSet::from([weapon("spear")])
    );
    assert_eq!(
        values_at(&shaman, 16, "greater-weapon-focus"),
        BTreeSet::from([weapon("spear")])
    );
}

#[test]
fn a_sword_saint_takes_weapon_focus_in_a_weapon_of_his_choosing_at_1() {
    let saint = archetype_build("magus", "magus/sword-saint", 1);
    let options = engine().options(&saint, &class_slot(1)).unwrap();
    assert_eq!(options.len(), DATA.weapons.len());
    assert!(
        options
            .iter()
            .all(|option| option.pick.key() == "weapon-focus" && option.status.is_ok())
    );
}

#[test]
fn a_half_elfs_race_slot_is_skill_focus_in_a_skill_of_choice() {
    let mut half_elf = build("fighter", 1);
    half_elf.start.race = Some(rules::RaceKey::new("half-elf"));
    let options = engine().options(&half_elf, &race_slot()).unwrap();
    assert_eq!(options.len(), DATA.skills.len());
    assert!(
        options
            .iter()
            .all(|option| option.pick.key() == "skill-focus")
    );
}

#[test]
fn a_slayer_takes_three_combat_style_feats_and_not_a_fourth() {
    let style = |level: u8| pick(&format!("slayer/slayer-talents/combat-style-feat-{level}"));
    let option_feat = |level: u8| SlotId::OptionFeat { level, index: 0 };
    let mut slayer = build("slayer", 12);
    for (level, stage, feat_) in [(4, 2, "dodge"), (8, 6, "toughness"), (10, 10, "iron-will")] {
        slayer.picks.insert(selection_slot(level), style(stage));
        slayer
            .picks
            .insert(option_feat(level), Pick::feat(FeatKey::new(feat_)));
    }
    // The style with the first (ticket 33).
    slayer.picks.insert(
        SlotId::OptionSelection { level: 4, index: 0 },
        pick("ranger/combat-style-feat/archery-feats"),
    );
    assert!(possible(&slayer));
    // A fourth: each window's talent is taken once, and the first window is past.
    slayer.picks.insert(selection_slot(12), style(10));
    assert_eq!(
        why_not(&slayer, &selection_slot(12)),
        Some(Issue::AlreadyHeld)
    );
    // And the first window is past at 6.
    let mut late = build("slayer", 6);
    late.picks.insert(selection_slot(6), style(2));
    assert_eq!(
        why_not(&late, &selection_slot(6)),
        unmet(
            Prerequisite::Not(Box::new(Prerequisite::ClassLevel {
                class: ClassKey::new("slayer"),
                level: 6,
            })),
            None
        )
    );
}

#[test]
fn perfect_mind_gives_bab_15_and_skill_ranks_at_the_mythic_rank_for_prerequisites() {
    let mut wizard = mythic("trickster", 17);
    wizard.levels = (0..17)
        .map(|_| LevelUp::new(ClassKey::new("wizard")))
        .collect();
    wizard.class_choices.clear();
    wizard.path_switch = Some(MythicPathKey::new("gold-dragon"));
    let sheet = engine().evaluate(&wizard);
    let state = sheet.at(17).unwrap();
    assert_eq!(state.bab, 8);
    assert_eq!(state.check(&Prerequisite::BaseAttackBonus(15)), Ok(()));
    assert_eq!(
        state.check(&Prerequisite::SkillRanks {
            skill: SkillKey::new("perception"),
            ranks: 9,
        }),
        Ok(())
    );
    assert!(
        sheet
            .at(16)
            .unwrap()
            .check(&Prerequisite::BaseAttackBonus(15))
            .is_err()
    );
}

#[test]
fn destiny_beyond_birth_removes_a_dwarfs_charisma_penalty() {
    let mut dwarf = build("fighter", 7);
    dwarf.start.race = Some(rules::RaceKey::new("dwarf"));
    dwarf.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new("destiny-beyond-birth")),
    );
    assert!(possible(&dwarf));
    let sheet = engine().evaluate(&dwarf);
    assert_eq!(sheet.at(6).unwrap().ability_scores.charisma, 8);
    assert_eq!(sheet.at(7).unwrap().ability_scores.charisma, 10);
    assert!(mythic_feat("destiny-beyond-birth").removes_racial_penalty);
}

#[test]
fn the_razmiri_infiltrator_is_true_neutral_or_evil() {
    let way = DATA
        .classes
        .iter()
        .flat_map(|c| &c.archetypes)
        .flat_map(|a| &a.selections)
        .flat_map(|s| &s.options)
        .find(|o| {
            o.key.as_str() == "arcanist/magic-deceiver/ways-of-the-deceiver/razmiri-infiltrator"
        })
        .unwrap();
    let Some(Prerequisite::Any(alignments)) = &way.prerequisite else {
        panic!("{:?}", way.prerequisite);
    };
    use rules::Alignment as A;
    assert_eq!(
        alignments.iter().cloned().collect::<BTreeSet<_>>(),
        [
            A::TrueNeutral,
            A::LawfulEvil,
            A::NeutralEvil,
            A::ChaoticEvil
        ]
        .into_iter()
        .map(Prerequisite::Alignment)
        .collect()
    );
}

#[test]
fn the_tortured_crusader_gets_3_skill_points_and_its_own_class_skills() {
    let crusader = archetype_build("paladin", "paladin/tortured-crusader", 1);
    let sheet = engine().evaluate(&crusader);
    // Intelligence 10: no modifier.
    assert_eq!(sheet.levels[0].skill_points.available, 3);
    let archetype = class("paladin")
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == "paladin/tortured-crusader")
        .unwrap();
    assert_eq!(
        archetype.class_skills,
        Some(
            [
                "athletics",
                "knowledge-arcana",
                "knowledge-world",
                "lore-nature",
                "lore-religion",
                "perception",
                "persuasion"
            ]
            .into_iter()
            .map(SkillKey::new)
            .collect()
        )
    );
}

#[test]
fn the_arcane_trickster_lacks_use_magic_device_and_the_dragon_disciple_has_knowledge_world() {
    assert!(
        !class("arcane-trickster")
            .class_skills
            .contains(&SkillKey::new("use-magic-device"))
    );
    assert!(
        class("dragon-disciple")
            .class_skills
            .contains(&SkillKey::new("knowledge-world"))
    );
}

// ---- option follow-ups (ticket 33) ----------------------------------------------------------------------------

fn crossblooded(first: &str, second: &str) -> Build {
    let build = archetype_build("sorcerer", "sorcerer/crossblooded", 1);
    let build = with_option(build, 1, 0, &format!("sorcerer/bloodline/{first}"));
    with_option(build, 1, 1, &format!("sorcerer/bloodline/{second}"))
}

fn one_of_group(group: rules::OptionGroup) -> Issue {
    Issue::Unmet(Unmet::Condition {
        needs: Prerequisite::NoOtherOptionOfGroup(group),
        has: None,
    })
}

#[test]
fn a_crossblooded_sorcerer_cant_take_red_after_blue() {
    let blue_red = crossblooded("draconic-blue", "draconic-red");
    let second = SlotId::Selection { level: 1, index: 1 };
    assert_eq!(
        why_not(&blue_red, &second),
        Some(one_of_group(rules::OptionGroup::Draconic))
    );
    // Two Elemental bloodlines neither; a dragon and an element are fine ("Elemental and Draconic bloodlines").
    assert_eq!(
        why_not(&crossblooded("elemental-air", "elemental-fire"), &second),
        Some(one_of_group(rules::OptionGroup::Elemental))
    );
    assert!(possible(&crossblooded("draconic-blue", "elemental-fire")));
    // Nor through Second Bloodline.
    let mut second = with_option(
        build("sorcerer", 6),
        1,
        0,
        "sorcerer/bloodline/draconic-blue",
    );
    second.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::MythicAbility(rules::MythicAbilityKey::new("second-bloodline")),
    );
    let slot = SlotId::MythicSelection { rank: 1, index: 0 };
    let options: HashMap<String, Result<(), Issue>> = engine()
        .options(&second, &slot)
        .unwrap()
        .into_iter()
        .map(|SlotOption { pick, status }| (pick.key().to_owned(), status))
        .collect();
    assert_eq!(
        options["sorcerer/bloodline/draconic-red"],
        Err(one_of_group(rules::OptionGroup::Draconic))
    );
    assert_eq!(options[ABYSSAL], Ok(()));
}

#[test]
fn a_crossblooded_sorcerers_second_bloodline_gives_its_arcana() {
    let build = crossblooded("arcane", "abyssal");
    assert!(possible(&build));
    let one = at(&build, 1);
    assert!(holds(&one, "sorcerer/bloodline/arcane/arcane-arcana"));
    assert!(holds(&one, "sorcerer/bloodline/abyssal/demonic-arcana"));
    assert!(!holds(&one, "sorcerer/bloodline/abyssal/grow-claws"));
    assert!(!holds(&one, "sorcerer/crossblooded/bloodline-2-arcana"));
    // Every sorcerer bloodline names its arcana.
    let bloodlines = &class("sorcerer")
        .selections
        .iter()
        .find(|s| s.key.as_str() == "sorcerer/bloodline")
        .unwrap()
        .options;
    assert_eq!(bloodlines.len(), 21);
    assert!(bloodlines.iter().all(|o| o.arcana.is_some()));
}

#[test]
fn the_unsworn_shamans_spirits_and_the_hunters_animal_focus_are_daily_choices() {
    let unsworn = at(&archetype_build("shaman", "shaman/unsworn-shaman", 6), 6);
    let daily: Vec<String> = engine()
        .daily_choices(&unsworn)
        .iter()
        .map(|key| key.to_string())
        .collect();
    assert!(daily.contains(&"shaman/unsworn-shaman/minor-spirit".to_owned()));
    assert!(daily.contains(&"shaman/unsworn-shaman/wandering-spirit-2".to_owned()));
    // The hunter's Animal Focus is a note, not a pick.
    let hunter = build("hunter", 8);
    let sheet = engine().evaluate(&hunter);
    assert!(
        sheet.levels[0]
            .slots
            .iter()
            .all(|slot| !matches!(slot.kind, SlotKind::Selection(_))),
        "{:?}",
        sheet.levels[0].slots
    );
    let daily = engine().daily_choices(sheet.at(8).unwrap());
    assert!(daily.contains(&FeatureKey::new("hunter/animal-focus")));
    assert!(daily.contains(&FeatureKey::new("hunter/second-animal-focus")));
}

#[test]
fn elemental_movement_follows_the_element_and_earth_has_none() {
    let element = |element: &str| {
        at(
            &with_option(
                build("sorcerer", 15),
                1,
                0,
                &format!("sorcerer/bloodline/elemental-{element}"),
            ),
            15,
        )
    };
    for name in ["air", "fire", "water"] {
        assert!(
            holds(
                &element(name),
                &format!("sorcerer/bloodline/elemental/elemental-movement-{name}")
            ),
            "{name}"
        );
    }
    let earth = element("earth");
    assert!(
        earth
            .features
            .keys()
            .all(|key| !key.as_str().contains("elemental-movement")),
        "{:?}",
        earth.features
    );
}

fn separatist(levels: usize) -> Build {
    let build = archetype_build("cleric", "cleric/separatist", levels);
    let build = with_option(build, 1, 0, "cleric/domain/fire");
    with_option(build, 1, 1, "cleric/domain/luck")
}

#[test]
fn a_separatists_second_domain_comes_two_levels_late() {
    let one = separatist(1);
    assert!(possible(&one));
    let at_1 = at(&one, 1);
    assert!(holds(&at_1, "cleric/domain/fire/fire-bolt"));
    assert!(!holds(&at_1, "cleric/domain/luck/bit-of-luck"));
    let three = separatist(3);
    let at_3 = at(&three, 3);
    assert!(holds(&at_3, "cleric/domain/fire/fire-bolt"));
    assert!(holds(&at_3, "cleric/domain/luck/bit-of-luck"));
    // Luck's second power at 6, two levels late: cleric 8.
    from_level(&separatist(8), 8, "cleric/domain/luck/divine-fortune");
    from_level(&separatist(8), 6, "cleric/domain/fire/fire-resistance");
}

#[test]
fn a_divine_hunter_8_holds_its_domains_level_8_power() {
    let build = with_option(
        archetype_build("hunter", "hunter/divine-hunter", 8),
        1,
        0,
        "cleric/domain/nobility",
    );
    assert!(possible(&build));
    from_level(&build, 8, "cleric/domain/nobility/inspiring-command");
}

/// A Living Deity with the Battle mystery and the Plagued curse, arcanist `levels`, its two domains at 20.
fn living_deity(levels: usize) -> Build {
    let build = archetype_build("arcanist", "arcanist/magic-deceiver", levels);
    let build = with_option(
        build,
        1,
        0,
        "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity",
    );
    let build = with_pick(build, option_slot(1, 0), "oracle/mystery/battle");
    let build = with_pick(build, option_slot(1, 2), "oracle/oracles-curse/plagued");
    let build = with_pick(build, option_slot(20, 0), "cleric/domain/fire");
    with_pick(build, option_slot(20, 1), "cleric/domain/luck")
}

/// `build` with a mythic ability at MR 1 (character level 6) and `option` in its selection slot.
fn with_mythic_selection(mut build: Build, ability: &str, option: &str) -> Build {
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::MythicAbility(rules::MythicAbilityKey::new(ability)),
    );
    build
        .picks
        .insert(SlotId::MythicSelection { rank: 1, index: 0 }, pick(option));
    build
}

#[test]
fn a_living_deity_with_impossible_domain_holds_a_third_domain() {
    let build = with_pick(
        living_deity(20),
        option_slot(1, 1),
        "oracle/revelations/battle/battlecry",
    );
    let build = with_mythic_selection(build, "impossible-domain", "cleric/domain/nobility");
    assert!(possible(&build));
    let twenty = at(&build, 20);
    for domain in ["fire", "luck", "nobility"] {
        assert_eq!(
            twenty.option_count(&SelectionOptionKey::new(format!("cleric/domain/{domain}"))),
            1,
            "{domain}"
        );
    }
    // Levelled by arcanist levels (ticket 35): 20 here.
    assert!(holds(&twenty, "cleric/domain/nobility/inspiring-command"));
    // A cleric's third domain runs on cleric levels.
    let cleric = with_mythic_selection(
        cleric(["fire", "luck"], 8),
        "impossible-domain",
        "cleric/domain/nobility",
    );
    assert!(possible(&cleric));
    from_level(&cleric, 6, "cleric/domain/nobility/inspiring-word");
    from_level(&cleric, 8, "cleric/domain/nobility/inspiring-command");
}

#[test]
fn cinder_dance_needs_no_lame_curse() {
    let flame = |curse: &str| {
        let build = oracle("flame", 1);
        let build = with_option(build, 1, 1, &format!("oracle/oracles-curse/{curse}"));
        with_option(build, 1, 2, "oracle/revelations/flame/cinder-dance")
    };
    assert!(possible(&flame("plagued")));
    assert!(!possible(&flame("lame")));
}

#[test]
fn a_dual_cursed_oracle_is_offered_misfortune_and_fortune() {
    // Its level-1 slots in key order: its own revelation, the mystery, the curse.
    let build = with_option(
        archetype_build("oracle", "oracle/dual-cursed-oracle", 13),
        1,
        1,
        "oracle/mystery/battle",
    );
    for level in [1, 3, 5, 7, 11, 13] {
        let offered = offered_at(&build, &selection_slot(level));
        assert!(
            offered.contains("oracle/dual-cursed-oracle/revelation/misfortune")
                && offered.contains("oracle/dual-cursed-oracle/revelation/fortune")
                && offered.contains("oracle/revelations/battle/weapon-mastery"),
            "{level}: {offered:?}"
        );
    }
    // A plain oracle isn't.
    assert!(
        !offered_at(&oracle("battle", 1), &revelation_slot(1))
            .contains("oracle/dual-cursed-oracle/revelation/misfortune")
    );
}

#[test]
fn a_living_deitys_maneuver_mastery_and_second_mystery_count_arcanist_levels() {
    // Maneuver Mastery's Trip at the revelation's 7th level: arcanist 7.
    let build = with_pick(
        living_deity(7),
        option_slot(1, 1),
        "oracle/revelations/battle/maneuver-mastery",
    );
    let build = with_pick(build, option_slot(1, 3), "oracle/maneuver-mastery/trip");
    let trip = FeatKey::new("trip");
    assert_eq!(at(&build, 6).feat_count(&trip), 0);
    assert_eq!(at(&build, 7).feat_count(&trip), 1);
    // The second mystery's class skill from the pick (arcanist 6), its Final Revelation at 20.
    let build = with_pick(
        living_deity(20),
        option_slot(1, 1),
        "oracle/revelations/battle/battlecry",
    );
    let build = with_mythic_selection(build, "second-mystery", "oracle/mystery/flame");
    assert!(possible(&build));
    class_skill_from_level(&build, 6, "athletics");
    from_level(&build, 20, "oracle/mystery/flame/flame-final-revelation");
}

#[test]
fn a_champion_of_the_faiths_alignment_blessing_is_his_own_alignment() {
    let champion = |alignment: rules::Alignment, blessing: &str| {
        let mut build = archetype_build("warpriest", "warpriest/champion-of-the-faith", 1);
        build.start.alignment = Some(alignment);
        let slot = engine().evaluate(&build).levels[0]
            .slots
            .iter()
            .find(|slot| {
                matches!(&slot.kind, SlotKind::Selection(s)
                    if s.as_str() == "warpriest/champion-of-the-faith/chosen-alignment")
            })
            .unwrap()
            .id
            .clone();
        build.picks.insert(
            slot,
            pick(&format!("warpriest/blessings/{blessing}-domain")),
        );
        possible(&build)
    };
    assert!(champion(rules::Alignment::LawfulGood, "good"));
    assert!(champion(rules::Alignment::LawfulGood, "law"));
    assert!(!champion(rules::Alignment::LawfulGood, "evil"));
    assert!(!champion(rules::Alignment::TrueNeutral, "chaos"));
    assert!(champion(rules::Alignment::ChaoticEvil, "chaos"));
    // A warpriest's own blessings don't check it.
    let blessings = with_option(
        build("warpriest", 1),
        1,
        0,
        "warpriest/blessings/evil-domain",
    );
    assert_eq!(blessings.start.alignment, None);
    assert!(possible(&blessings));
}

#[test]
fn a_signifer_picks_one_of_the_seven_hellknight_orders() {
    let signifer = class("hellknight-signifer");
    let order = signifer
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight-signifer/order")
        .expect("the Signifer's order");
    assert_eq!(order.offered().len(), 7);
    assert!(signifer.levels[0].selections.contains(&order.key));
    assert!(
        signifer
            .features
            .iter()
            .all(|f| f.key.as_str() != "hellknight-signifer/order")
    );
}

#[test]
fn expanded_element_at_15_isnt_the_7th_level_element() {
    let mut build = kineticist("fire", 15);
    build
        .picks
        .insert(selection_slot(7), pick("kineticist/element/water"));
    build
        .picks
        .insert(selection_slot(15), pick("kineticist/element/water"));
    assert!(!possible(&build), "the 7th-level element again");
    build
        .picks
        .insert(selection_slot(15), pick("kineticist/element/fire"));
    assert!(possible(&build));
}

#[test]
fn a_slayer_who_picked_archery_cant_take_two_weapon_fighting_through_a_combat_style_talent() {
    let mut slayer = build("slayer", 6);
    slayer.picks.insert(
        selection_slot(2),
        pick("slayer/slayer-talents/combat-style-feat-2"),
    );
    let style = SlotId::OptionSelection { level: 2, index: 0 };
    // The five ranger styles.
    assert_eq!(offered_at(&slayer, &style).len(), 5);
    slayer
        .picks
        .insert(style, pick("ranger/combat-style-feat/archery-feats"));
    let feat_slot = |level| SlotId::OptionFeat { level, index: 0 };
    let offered = feats_offered(&slayer, &feat_slot(2));
    assert!(offered.contains("rapid-shot") && !offered.contains("two-weapon-fighting"));
    slayer.picks.insert(
        feat_slot(2),
        Pick::feat(FeatKey::new("two-weapon-fighting")),
    );
    assert_eq!(why_not(&slayer, &feat_slot(2)), None);
    // The second talent picks no style; its feat comes from Archery's list of the second stage.
    slayer
        .picks
        .insert(feat_slot(2), Pick::feat(FeatKey::new("rapid-shot")));
    slayer.picks.insert(
        selection_slot(6),
        pick("slayer/slayer-talents/combat-style-feat-6"),
    );
    let sheet = engine().evaluate(&slayer);
    assert!(
        sheet
            .slot(&SlotId::OptionSelection { level: 6, index: 0 })
            .is_none()
    );
    assert!(feats_offered(&slayer, &feat_slot(6)).contains("manyshot"));
    slayer
        .picks
        .insert(feat_slot(6), Pick::feat(FeatKey::new("manyshot")));
    assert!(possible(&slayer));
}

// ---- second follow-ups (ticket 35) ----------------------------------------------------------------------------

#[test]
fn a_dual_heritage_or_kindred_raised_half_elf_has_no_adaptability_pick() {
    let half_elf = |subrace: Option<&str>| {
        let mut build = build("fighter", 1);
        build.start.race = Some(rules::RaceKey::new("half-elf"));
        build.start.subrace = subrace.map(rules::SubraceKey::new);
        build
    };
    let race_slots = |build: &Build| {
        engine().evaluate(build).levels[0]
            .slots
            .iter()
            .filter(|slot| slot.id == race_slot())
            .count()
    };
    assert_eq!(race_slots(&half_elf(None)), 1);
    for subrace in ["half-elf/dual-heritage", "half-elf/kindred-raised"] {
        assert_eq!(race_slots(&half_elf(Some(subrace))), 0, "{subrace}");
    }
    // No subrace in the data has slots of its own: the guide lists none.
    assert!(
        DATA.races
            .iter()
            .flat_map(|race| &race.subraces)
            .all(|subrace| subrace.feat_slots.is_empty())
    );
}

/// A Living Deity with the Battle mystery and the Plagued curse, arcanist `levels` (below 20: no domains of its
/// own yet), with Impossible Domain's Nobility domain at MR 1 (character level 6).
fn living_deity_with_impossible_domain(levels: usize) -> Build {
    let build = archetype_build("arcanist", "arcanist/magic-deceiver", levels);
    let build = with_option(
        build,
        1,
        0,
        "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity",
    );
    let build = with_pick(build, option_slot(1, 0), "oracle/mystery/battle");
    let build = with_pick(build, option_slot(1, 2), "oracle/oracles-curse/plagued");
    with_mythic_selection(build, "impossible-domain", "cleric/domain/nobility")
}

#[test]
fn a_living_deity_with_impossible_domain_holds_the_domains_level_1_power_before_arcanist_20() {
    let build = living_deity_with_impossible_domain(8);
    assert!(possible(&build));
    // Arcanist levels from the pick: its 1st-level power at 6, its 8th-level one at 8.
    from_level(&build, 6, "cleric/domain/nobility/inspiring-word");
    from_level(&build, 8, "cleric/domain/nobility/inspiring-command");
    assert_eq!(
        at(&build, 8).option_level(&SelectionOptionKey::new("cleric/domain/nobility")),
        8
    );
}

/// Fighter 6, then Hellknight 3 (lawful neutral, with the entry's skill ranks): the Order of the Godclaw at
/// Hellknight 1, Pentamic Faith's Law domain at 3 (character level 9, Mythic rank 3).
fn pentamic_hellknight() -> Build {
    let mut build = classes(&[
        "fighter",
        "fighter",
        "fighter",
        "fighter",
        "fighter",
        "fighter",
        "hellknight",
        "hellknight",
        "hellknight",
    ]);
    build.start.alignment = Some(rules::Alignment::LawfulNeutral);
    for (level, ranks) in build.levels.iter_mut().zip([
        [("knowledge-arcana", 1), ("persuasion", 1)],
        [("knowledge-arcana", 1), ("persuasion", 1)],
        [("persuasion", 1), ("perception", 1)],
        [("persuasion", 1), ("perception", 1)],
        [("persuasion", 1), ("perception", 1)],
    ]) {
        level.skill_ranks = ranks
            .into_iter()
            .map(|(skill, ranks)| (SkillKey::new(skill), ranks))
            .collect();
    }
    let build = with_option(build, 7, 0, "fighter/armiger/order/order-of-the-godclaw");
    let build = with_option(build, 9, 0, "hellknight/disciplines/pentamic-faith");
    with_pick(build, option_slot(9, 0), "cleric/domain/law")
}

#[test]
fn a_hellknight_3_with_pentamic_faiths_domain_can_take_impossible_domain() {
    let build = pentamic_hellknight();
    assert!(possible(&build));
    let mythic = SlotId::Mythic { rank: 3 };
    let status = engine()
        .options(&build, &mythic)
        .unwrap()
        .into_iter()
        .find(|option| option.pick.key() == "impossible-domain")
        .map(|option| option.status);
    assert_eq!(status, Some(Ok(())));
    // Its domain runs on Hellknight levels, like the Pentamic Faith domain.
    let mut build = build;
    build.picks.insert(
        mythic,
        Pick::MythicAbility(rules::MythicAbilityKey::new("impossible-domain")),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 3, index: 0 },
        pick("cleric/domain/war"),
    );
    assert!(possible(&build));
    let nine = at(&build, 9);
    assert_eq!(
        nine.option_level(&SelectionOptionKey::new("cleric/domain/war")),
        3
    );
    assert!(holds(&nine, "cleric/domain/war/battle-rage"));
}

#[test]
fn a_hellknight_who_becomes_a_signifer_gets_no_second_order() {
    let order_slots = |build: &Build, level: u8| {
        engine().evaluate(build).levels[usize::from(level) - 1]
            .slots
            .iter()
            .filter(|slot| {
                matches!(&slot.kind, SlotKind::Selection(s) if s.as_str() == "hellknight-signifer/order")
            })
            .count()
    };
    let mut hellknight = with_option(
        classes(&[
            "fighter",
            "fighter",
            "fighter",
            "fighter",
            "fighter",
            "fighter",
            "hellknight",
            "hellknight-signifer",
        ]),
        7,
        0,
        "fighter/armiger/order/order-of-the-chain",
    );
    hellknight.start.alignment = Some(rules::Alignment::LawfulNeutral);
    assert_eq!(order_slots(&hellknight, 8), 0);
    // Anyone holding no order picks one.
    let mut wizard = classes(&[
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "hellknight-signifer",
    ]);
    wizard.start.alignment = Some(rules::Alignment::LawfulNeutral);
    assert_eq!(order_slots(&wizard, 6), 1);
}

#[test]
fn a_lawful_good_separatist_cant_take_chaos_or_evil_as_his_second_domain() {
    let separatist = |alignment: rules::Alignment, domain: &str| {
        let mut build = with_option(
            with_option(
                archetype_build("cleric", "cleric/separatist", 1),
                1,
                0,
                "cleric/domain/fire",
            ),
            1,
            1,
            &format!("cleric/domain/{domain}"),
        );
        build.start.alignment = Some(alignment);
        possible(&build)
    };
    let lawful_good = rules::Alignment::LawfulGood;
    assert!(!separatist(lawful_good, "chaos"));
    assert!(!separatist(lawful_good, "evil"));
    assert!(separatist(lawful_good, "good"));
    assert!(separatist(lawful_good, "law"));
    // A true neutral has none that conflicts; a chaotic evil can't take Law or Good.
    for domain in ["chaos", "evil", "good", "law"] {
        assert!(
            separatist(rules::Alignment::TrueNeutral, domain),
            "{domain}"
        );
    }
    assert!(!separatist(rules::Alignment::ChaoticEvil, "law"));
    assert!(!separatist(rules::Alignment::ChaoticEvil, "good"));
    // His first domain, a cleric's, isn't checked against it here.
    let mut first = with_option(
        archetype_build("cleric", "cleric/separatist", 1),
        1,
        0,
        "cleric/domain/chaos",
    );
    first.start.alignment = Some(lawful_good);
    assert!(possible(&first));
}

#[test]
fn a_ranger_10_slayer_2_takes_the_slayers_2_or_4_feat_from_the_first_stage() {
    let mut keys = vec!["ranger"; 10];
    keys.extend(["slayer", "slayer"]);
    let build = with_option(
        classes(&keys),
        2,
        0,
        "ranger/combat-style-feat/archery-feats",
    );
    let build = with_option(build, 12, 0, "slayer/slayer-talents/combat-style-feat-2");
    // One shared style: the slayer picks none.
    assert!(
        engine()
            .evaluate(&build)
            .slot(&option_slot(12, 0))
            .is_none()
    );
    let feat_slot = SlotId::OptionFeat {
        level: 12,
        index: 0,
    };
    let offered = feats_offered(&build, &feat_slot);
    assert!(offered.contains("rapid-shot"), "{offered:?}");
    assert!(
        !offered.contains("manyshot") && !offered.contains("improved-critical"),
        "{offered:?}"
    );
    // The ranger's own slot at 10 is the style's 10th-level list.
    assert!(
        feats_offered(
            &build,
            &SlotId::Feat {
                level: 10,
                source: FeatSource::Class,
                index: 0,
            }
        )
        .contains("improved-critical")
    );
}

#[test]
fn a_bloodrager_with_a_dragon_bloodline_cant_take_a_second_dragon_through_second_bloodline() {
    let second = |first: &str| {
        let mut build = with_option(
            build("bloodrager", 6),
            1,
            0,
            &format!("bloodrager/bloodline/{first}"),
        );
        build.picks.insert(
            SlotId::Mythic { rank: 1 },
            Pick::MythicAbility(rules::MythicAbilityKey::new("second-bloodrager-bloodline")),
        );
        let options: HashMap<String, Result<(), Issue>> = engine()
            .options(&build, &SlotId::MythicSelection { rank: 1, index: 0 })
            .unwrap()
            .into_iter()
            .map(|SlotOption { pick, status }| (pick.key().to_owned(), status))
            .collect();
        options
    };
    let red = second("draconic-red");
    assert_eq!(
        red["bloodrager/bloodline/draconic-blue"],
        Err(one_of_group(rules::OptionGroup::BloodragerDraconic))
    );
    assert_eq!(red["bloodrager/bloodline/abyssal"], Ok(()));
    assert_eq!(red["bloodrager/bloodline/elemental-fire"], Ok(()));
    let air = second("elemental-air");
    assert_eq!(
        air["bloodrager/bloodline/elemental-fire"],
        Err(one_of_group(rules::OptionGroup::BloodragerElemental))
    );
    assert_eq!(air["bloodrager/bloodline/draconic-red"], Ok(()));
    // Apart from the sorcerer's: a sorcerer's dragon doesn't stop a bloodrager's.
    let mut both = with_option(
        build("sorcerer", 1),
        1,
        0,
        "sorcerer/bloodline/draconic-red",
    );
    both.levels.push(LevelUp::new(ClassKey::new("bloodrager")));
    let both = with_option(both, 2, 0, "bloodrager/bloodline/draconic-blue");
    assert!(possible(&both));
}

// ---- Impossible Domain and the druid (ticket 36) ---------------------------------------------------------------

/// `levels` of one class (and an archetype), then fighter 3: Mythic rank 1 at character level 6.
fn then_fighter_3(mut build: Build) -> Build {
    build
        .class_choices
        .insert(ClassKey::new("fighter"), ClassChoice::default());
    for _ in 0..3 {
        build.levels.push(LevelUp::new(ClassKey::new("fighter")));
    }
    build
}

/// Impossible Domain's status in the Mythic rank 1 slot.
fn impossible_domain_at_mr_1(build: &Build) -> Option<Result<(), Issue>> {
    engine()
        .options(build, &SlotId::Mythic { rank: 1 })
        .unwrap()
        .into_iter()
        .find(|option| option.pick.key() == "impossible-domain")
        .map(|option| option.status)
}

#[test]
fn no_druid_can_take_impossible_domain_whatever_its_domain() {
    // "If you choose a domain, you can't take the Impossible Domain mythic ability." (the druid's page)
    let nature_bond = with_pick(
        with_option(druid(None, 3), 1, 0, "druid/nature-bond/domain"),
        option_slot(1, 0),
        "cleric/domain/weather",
    );
    let blight = with_option(
        druid(Some("druid/blight-druid"), 3),
        1,
        0,
        "cleric/domain/death",
    );
    let winter = druid(Some("druid/winter-child"), 3);
    let companion = with_option(druid(None, 3), 1, 0, "druid/nature-bond/animal-companion");
    for (name, build, domain) in [
        ("Nature Bond", nature_bond, Some("weather")),
        ("Blight Druid", blight, Some("death")),
        ("Winter Child", winter, Some("ice")),
        ("animal companion", companion, None),
    ] {
        let build = then_fighter_3(build);
        assert!(possible(&build), "{name}");
        // Each holds its druid's domain at druid level 3, so only being a druid's keeps it out.
        if let Some(domain) = domain {
            assert_eq!(
                at(&build, 6)
                    .option_level(&SelectionOptionKey::new(format!("cleric/domain/{domain}"))),
                3,
                "{name}"
            );
        }
        let status = impossible_domain_at_mr_1(&build);
        assert!(matches!(status, Some(Err(_))), "{name}: {status:?}");
    }
    // A cleric 3 / fighter 3 can.
    assert_eq!(
        impossible_domain_at_mr_1(&then_fighter_3(cleric(["air", "fire"], 3))),
        Some(Ok(()))
    );
}

// ---- backgrounds (ticket 34) ----------------------------------------------------------------------------------

/// A wizard 1 (no class skill or proficiency a background gives) with a background.
fn wizard_with(background: Option<&str>) -> Build {
    let mut build = build("wizard", 1);
    build.start.background = background.map(rules::BackgroundKey::new);
    build
}

fn general_slot_1() -> SlotId {
    SlotId::Feat {
        level: 1,
        source: FeatSource::General,
        index: 0,
    }
}

/// The status of a feat in level 1's general slot.
fn general_feat_status(build: &Build, feat: &str) -> Result<(), Issue> {
    engine()
        .options(build, &general_slot_1())
        .unwrap()
        .into_iter()
        .find(|option| option.pick.key() == feat)
        .unwrap_or_else(|| panic!("{feat} isn't offered"))
        .status
}

#[test]
fn the_users_38_backgrounds_are_there_in_their_groups() {
    use rules::BackgroundGroup::*;
    assert_eq!(DATA.backgrounds.len(), 38);
    let in_group = |group| {
        DATA.backgrounds
            .iter()
            .filter(|b| b.group == Some(group))
            .count()
    };
    assert_eq!(
        [
            Craftsman,
            Noble,
            Oblate,
            Regional,
            Scholar,
            StreetUrchin,
            Wanderer,
            Warrior
        ]
        .map(in_group),
        [4, 3, 3, 15, 3, 3, 3, 4]
    );
    // No background stands for "none": a Build without one is the Build start's own state (user, 2026-11-02).
    // So every background has a group and makes at least one skill a class skill.
    assert!(
        DATA.backgrounds
            .iter()
            .all(|b| b.group.is_some() && !b.class_skills.is_empty())
    );
}

#[test]
fn only_the_decided_backgrounds_grant_feats() {
    let granted: Vec<(String, Vec<String>)> = DATA
        .backgrounds
        .iter()
        .filter(|b| !b.feats.is_empty() || !b.feats_for.is_empty())
        .map(|b| {
            let feats = b.feats.iter().map(|f| f.to_string()).chain(
                b.feats_for
                    .iter()
                    .map(|g| format!("{} ({})", g.feat, g.value.key())),
            );
            (b.key.to_string(), feats.collect())
        })
        .collect();
    let light = || vec!["armor-proficiency-light-armor".to_owned()];
    assert_eq!(
        granted,
        [
            (
                "aldori-swordsman".to_owned(),
                vec!["exotic-weapon-proficiency (dueling-sword)".to_owned()]
            ),
            ("arcane".to_owned(), vec!["scribe-scrolls".to_owned()]),
            ("bounty-hunter".to_owned(), light()),
            ("divine".to_owned(), vec!["brew-potions".to_owned()]),
            ("gladiator".to_owned(), light()),
            ("guard".to_owned(), light()),
            (
                "martial-disciple".to_owned(),
                vec!["improved-unarmed-strike".to_owned()]
            ),
            ("mercenary".to_owned(), light()),
        ]
    );
}

#[test]
fn a_shackles_corsair_holds_athletics_and_mobility_as_class_skills() {
    let skills = |build: &Build| -> BTreeSet<SkillKey> {
        engine().evaluate(build).at(1).unwrap().class_skills.clone()
    };
    let athletics = SkillKey::new("athletics");
    let mobility = SkillKey::new("mobility");
    let plain = skills(&wizard_with(None));
    assert!(!plain.contains(&athletics) && !plain.contains(&mobility));
    let corsair = skills(&wizard_with(Some("shackles-corsair")));
    assert!(corsair.contains(&athletics) && corsair.contains(&mobility));
    assert_eq!(
        corsair,
        plain.iter().cloned().chain([athletics, mobility]).collect()
    );
}

#[test]
fn an_aldori_swordsman_holds_exotic_weapon_proficiency_in_the_dueling_sword() {
    let sheet = engine().evaluate(&wizard_with(Some("aldori-swordsman")));
    let state = sheet.at(1).unwrap();
    let proficiency = FeatKey::new("exotic-weapon-proficiency");
    let weapon = |key: &str| Parameter::Weapon(WeaponKey::new(key));
    assert!(state.has_feat_for(&proficiency, &weapon("dueling-sword")));
    assert!(!state.has_feat_for(&proficiency, &weapon("bastard-sword")));
    assert!(engine().is_possible(&wizard_with(Some("aldori-swordsman"))));
}

#[test]
fn a_guard_can_take_armor_proficiency_medium_armor_at_level_1() {
    let medium = "armor-proficiency-medium-armor";
    assert!(general_feat_status(&wizard_with(None), medium).is_err());
    let mut guard = wizard_with(Some("guard"));
    assert_eq!(general_feat_status(&guard, medium), Ok(()));
    guard
        .picks
        .insert(general_slot_1(), Pick::feat(FeatKey::new(medium)));
    assert!(engine().is_possible(&guard));
}

#[test]
fn a_martial_disciples_improved_unarmed_strike_meets_stunning_fists_prerequisite() {
    let iua = Prerequisite::Feat(FeatKey::new("improved-unarmed-strike"));
    let unmet = general_feat_status(&wizard_with(None), "stunning-fist").unwrap_err();
    let Issue::Unmet(unmet) = unmet else {
        panic!("{unmet:?}")
    };
    assert!(unmet.conditions().iter().any(|(needs, _)| **needs == iua));
    assert_eq!(
        general_feat_status(&wizard_with(Some("martial-disciple")), "stunning-fist"),
        Ok(())
    );
}

// ---- options' class skills (ticket 37) ------------------------------------------------------------------------

/// The class skills at level 1 of a Build of `class` (with `archetype`) picking `option` in its first selection slot.
fn class_skills_at_1(
    class: &str,
    archetype: Option<&str>,
    option: Option<&str>,
) -> BTreeSet<String> {
    let mut build = match archetype {
        Some(archetype) => archetype_build(class, archetype, 1),
        None => build(class, 1),
    };
    if let Some(option) = option {
        build.picks.insert(selection_slot(1), pick(option));
    }
    assert!(
        engine().is_possible(&build),
        "{class} {archetype:?} {option:?}"
    );
    let sheet = engine().evaluate(&build);
    sheet
        .at(1)
        .unwrap()
        .class_skills
        .iter()
        .map(|skill| skill.to_string())
        .collect()
}

fn skill_set(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|key| (*key).to_owned()).collect()
}

#[test]
fn an_oracle_holds_its_mysterys_class_skills_and_one_without_it_doesnt() {
    let oracle = skill_set(&[
        "knowledge-arcana",
        "knowledge-world",
        "lore-nature",
        "lore-religion",
        "persuasion",
    ]);
    assert_eq!(class_skills_at_1("oracle", None, None), oracle);
    for (mystery, skills) in [
        ("battle", &["perception"][..]),
        ("bones", &["stealth"]),
        ("flame", &["athletics", "mobility"]),
        ("nature", &["athletics"]),
        ("stone", &["athletics"]),
        ("waves", &["athletics", "mobility"]),
        ("wind", &["mobility", "stealth"]),
        ("ancestor", &[]),
        ("life", &[]),
    ] {
        let held = class_skills_at_1("oracle", None, Some(&format!("oracle/mystery/{mystery}")));
        let expected: BTreeSet<String> = oracle.iter().cloned().chain(skill_set(skills)).collect();
        assert_eq!(held, expected, "{mystery}");
    }
    // A Divine Herbalist "does not gain additional class skills from his choice of mystery".
    assert_eq!(
        class_skills_at_1(
            "oracle",
            Some("oracle/divine-herbalist"),
            Some("oracle/mystery/flame")
        ),
        oracle
    );
}

#[test]
fn a_kineticist_holds_its_elements_class_skills_and_one_without_it_doesnt() {
    let kineticist = skill_set(&[
        "mobility",
        "perception",
        "persuasion",
        "stealth",
        "use-magic-device",
    ]);
    assert_eq!(class_skills_at_1("kineticist", None, None), kineticist);
    for (element, skills) in [
        ("air", &["lore-nature"][..]),
        ("earth", &["athletics", "knowledge-world"]),
        ("fire", &["lore-nature"]),
        ("water", &["athletics", "lore-nature"]),
    ] {
        let held = class_skills_at_1(
            "kineticist",
            None,
            Some(&format!("kineticist/element/{element}")),
        );
        let expected: BTreeSet<String> = kineticist
            .iter()
            .cloned()
            .chain(skill_set(skills))
            .collect();
        assert_eq!(held, expected, "{element}");
    }
}

#[test]
fn a_cavalier_holds_its_orders_class_skill() {
    let cavalier = skill_set(&["athletics", "mobility", "persuasion"]);
    assert_eq!(class_skills_at_1("cavalier", None, None), cavalier);
    for (order, skill) in [
        ("order-of-the-lion", "knowledge-world"),
        ("order-of-the-shroud", "lore-religion"),
        ("order-of-the-star", "lore-religion"),
        ("order-of-the-sword", "lore-religion"),
    ] {
        let held = class_skills_at_1("cavalier", None, Some(&format!("cavalier/order/{order}")));
        let expected: BTreeSet<String> =
            cavalier.iter().cloned().chain([skill.to_owned()]).collect();
        assert_eq!(held, expected, "{order}");
    }
}

#[test]
fn a_cleric_holds_both_domains_class_skills() {
    let mut cleric = build("cleric", 1);
    cleric
        .picks
        .insert(selection_slot(1), pick("cleric/domain/knowledge"));
    cleric.picks.insert(
        SlotId::Selection { level: 1, index: 1 },
        pick("cleric/domain/trickery"),
    );
    assert!(engine().is_possible(&cleric));
    let sheet = engine().evaluate(&cleric);
    let held: BTreeSet<String> = sheet
        .at(1)
        .unwrap()
        .class_skills
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        held,
        skill_set(&[
            "knowledge-arcana",
            "knowledge-world",
            "lore-nature",
            "lore-religion",
            "persuasion",
            "stealth",
            "trickery",
        ])
    );
}

#[test]
fn a_sorcerer_holds_its_bloodlines_class_skill() {
    let sorcerer = skill_set(&["knowledge-arcana", "persuasion", "use-magic-device"]);
    assert_eq!(class_skills_at_1("sorcerer", None, None), sorcerer);
    for (bloodline, skill) in [
        ("abyssal", "athletics"),
        ("celestial", "lore-religion"),
        ("draconic-red", "perception"),
        ("elemental-fire", "mobility"),
        ("fey", "lore-nature"),
        ("infernal", "knowledge-world"),
        ("serpentine", "stealth"),
        ("undead", "lore-religion"),
    ] {
        let held = class_skills_at_1(
            "sorcerer",
            None,
            Some(&format!("sorcerer/bloodline/{bloodline}")),
        );
        let expected: BTreeSet<String> =
            sorcerer.iter().cloned().chain([skill.to_owned()]).collect();
        assert_eq!(held, expected, "{bloodline}");
    }
}

/// Every option row's class skills, as `option level: skills`.
fn option_row_class_skills() -> Vec<(String, u8, Vec<String>)> {
    let mut rows = Vec::new();
    for class in DATA.classes.iter().chain(&DATA.prestige_classes) {
        let selections = class
            .selections
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.selections));
        for selection in selections {
            for option in &selection.options {
                for row in &option.levels {
                    if !row.class_skills.is_empty() {
                        rows.push((
                            option.key.to_string(),
                            row.level,
                            row.class_skills.iter().map(|s| s.to_string()).collect(),
                        ));
                    }
                }
            }
        }
    }
    rows.sort();
    rows
}

#[test]
fn options_class_skills_are_the_guides_on_their_first_row_and_the_animal_domains_on_its_4th() {
    let rows = option_row_class_skills();
    let mut expected: Vec<(String, u8, Vec<String>)> = Vec::new();
    let mut add = |option: String, skills: &[&str]| {
        // The Animal domain's comes with its companion: "At 4th level, you gain an Animal Companion. … Lore (Nature)
        // becomes a Class Skill."
        let level = if option == "cleric/domain/animal" {
            4
        } else {
            1
        };
        expected.push((
            option,
            level,
            skills.iter().map(|s| (*s).to_owned()).collect(),
        ));
    };
    for (bloodline, skill) in [
        ("abyssal", "athletics"),
        ("celestial", "lore-religion"),
        ("fey", "lore-nature"),
        ("infernal", "knowledge-world"),
        ("serpentine", "stealth"),
        ("undead", "lore-religion"),
    ] {
        add(format!("sorcerer/bloodline/{bloodline}"), &[skill]);
    }
    for colour in [
        "black", "blue", "brass", "bronze", "copper", "gold", "green", "red", "silver", "white",
    ] {
        add(
            format!("sorcerer/bloodline/draconic-{colour}"),
            &["perception"],
        );
    }
    for element in ["air", "earth", "fire", "water"] {
        add(
            format!("sorcerer/bloodline/elemental-{element}"),
            &["mobility"],
        );
    }
    for (mystery, skills) in [
        ("battle", &["perception"][..]),
        ("bones", &["stealth"]),
        ("flame", &["athletics", "mobility"]),
        ("nature", &["athletics"]),
        ("stone", &["athletics"]),
        ("waves", &["athletics", "mobility"]),
        ("wind", &["mobility", "stealth"]),
    ] {
        add(format!("oracle/mystery/{mystery}"), skills);
    }
    for (domain, skills) in [
        ("animal", &["lore-nature"][..]),
        (
            "knowledge",
            &[
                "knowledge-arcana",
                "knowledge-world",
                "lore-nature",
                "lore-religion",
            ],
        ),
        ("trickery", &["stealth", "trickery"]),
    ] {
        add(format!("cleric/domain/{domain}"), skills);
    }
    for (element, skills) in [
        ("air", &["lore-nature"][..]),
        ("earth", &["athletics", "knowledge-world"]),
        ("fire", &["lore-nature"]),
        ("water", &["athletics", "lore-nature"]),
    ] {
        add(format!("kineticist/element/{element}"), skills);
    }
    add(
        "cavalier/order/order-of-the-lion".into(),
        &["knowledge-world"],
    );
    for order in ["shroud", "star", "sword"] {
        add(
            format!("cavalier/order/order-of-the-{order}"),
            &["lore-religion"],
        );
    }
    expected.sort();
    assert_eq!(rows, expected);
}

#[test]
fn no_feature_stands_for_a_fixed_class_skill() {
    let features: Vec<(&str, &str)> = DATA
        .classes
        .iter()
        .chain(&DATA.prestige_classes)
        .flat_map(|class| {
            class
                .features
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.features))
        })
        .map(|feature| (feature.key.as_str(), feature.name.as_str()))
        .filter(|(_, name)| name.starts_with("Class Skill") || *name == "Skills")
        .collect();
    // What's left is a choice of one of three skills (ticket 37's first open question, left as features).
    assert_eq!(
        features,
        [
            ("sorcerer/bloodline/arcane/class-skill", "Class Skill"),
            ("sorcerer/sage-sorcerer/class-skill", "Class Skill"),
        ]
    );
    // The archetypes whose feature went hold the skill through their own list.
    for (class, archetype, skill) in [
        ("monk", "monk/sohei", "lore-nature"),
        ("sorcerer", "sorcerer/empyreal-sorcerer", "lore-religion"),
        ("sorcerer", "sorcerer/sylvan-sorcerer", "lore-nature"),
    ] {
        let sheet = engine().evaluate(&archetype_build(class, archetype, 1));
        assert!(
            sheet
                .at(1)
                .unwrap()
                .class_skills
                .contains(&SkillKey::new(skill)),
            "{archetype}"
        );
    }
}

// ---- class-skill cleanup (ticket 38) --------------------------------------------------------------------------

#[test]
fn an_overwhelming_mage_and_a_seeker_hold_their_bloodlines_class_skill_and_no_class_skill_feature()
{
    let sorcerer = skill_set(&["knowledge-arcana", "persuasion", "use-magic-device"]);
    for archetype in ["sorcerer/overwhelming-mage", "sorcerer/seeker-sorcerer"] {
        for (bloodline, skill) in [
            ("abyssal", "athletics"),
            ("celestial", "lore-religion"),
            ("draconic-gold", "perception"),
            ("elemental-water", "mobility"),
            ("fey", "lore-nature"),
            ("infernal", "knowledge-world"),
            ("serpentine", "stealth"),
            ("undead", "lore-religion"),
        ] {
            let build = with_option(
                archetype_build("sorcerer", archetype, 1),
                1,
                0,
                &format!("sorcerer/bloodline/{bloodline}"),
            );
            assert!(engine().is_possible(&build), "{archetype} {bloodline}");
            let sheet = engine().evaluate(&build);
            let state = sheet.at(1).unwrap();
            let held: BTreeSet<String> = state.class_skills.iter().map(|s| s.to_string()).collect();
            let expected: BTreeSet<String> =
                sorcerer.iter().cloned().chain([skill.to_owned()]).collect();
            assert_eq!(held, expected, "{archetype} {bloodline}");
            let class_skill_features: Vec<&FeatureKey> = state
                .features
                .keys()
                .filter(|feature| feature.as_str().ends_with("/class-skill"))
                .collect();
            assert_eq!(
                class_skill_features,
                [] as [&FeatureKey; 0],
                "{archetype} {bloodline}"
            );
        }
        let defined = DATA
            .classes
            .iter()
            .flat_map(|class| class.archetypes.iter())
            .flat_map(|a| &a.features)
            .any(|feature| feature.key.as_str() == format!("{archetype}/class-skill"));
        assert!(!defined, "{archetype}/class-skill is still defined");
    }
}

#[test]
fn a_sohei_holds_lore_nature_as_a_class_skill_and_a_plain_monk_doesnt() {
    // A lawful neutral monk, plain or Sohei, at 1.
    let monk = |archetype: Option<&str>| {
        let mut monk = match archetype {
            Some(archetype) => archetype_build("monk", archetype, 1),
            None => build("monk", 1),
        };
        monk.start.alignment = Some(rules::Alignment::LawfulNeutral);
        assert!(engine().is_possible(&monk), "{archetype:?}");
        let sheet = engine().evaluate(&monk);
        sheet.at(1).cloned().unwrap()
    };
    let skills = |state: &CharacterState| -> BTreeSet<String> {
        state.class_skills.iter().map(|s| s.to_string()).collect()
    };
    let plain = skill_set(&[
        "athletics",
        "knowledge-world",
        "lore-religion",
        "mobility",
        "perception",
        "persuasion",
        "stealth",
    ]);
    assert_eq!(skills(&monk(None)), plain);
    let sohei = monk(Some("monk/sohei"));
    let expected: BTreeSet<String> = plain
        .iter()
        .cloned()
        .chain(["lore-nature".to_owned()])
        .collect();
    assert_eq!(skills(&sohei), expected);
    assert!(!holds(&sohei, "monk/sohei/skills"));
}

/// The real data with `skill` taken off `class`'s own list, so that the skill an option row gives shows.
fn engine_without_class_skill(class: &str, skill: &str) -> Engine {
    let mut data = DATA.clone();
    let class = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == class)
        .unwrap();
    assert!(class.class_skills.remove(&SkillKey::new(skill)));
    Engine::new(data)
}

/// `engine` gives `skill` as a class skill from `level` on, not before.
fn class_skill_from_level_by(engine: &Engine, build: &Build, level: u8, skill: &str) {
    assert!(engine.is_possible(build));
    let sheet = engine.evaluate(build);
    let has = |level: u8| {
        sheet
            .at(level)
            .unwrap()
            .class_skills
            .contains(&SkillKey::new(skill))
    };
    assert!(!has(level - 1), "{skill} before {level}");
    assert!(has(level), "{skill} at {level}");
}

#[test]
fn the_animal_domains_lore_nature_comes_with_its_companion_at_the_holders_4th_domain_level() {
    let animal = cleric(["animal", "fire"], 4);
    assert!(engine().is_possible(&animal));
    class_skill_from_level(&animal, 4, "lore-nature");
    from_level(&animal, 4, "cleric/domain/animal/animal-companion");
    // A Separatist's second domain runs two levels behind: cleric 6.
    let separatist = with_option(
        with_option(
            archetype_build("cleric", "cleric/separatist", 6),
            1,
            0,
            "cleric/domain/fire",
        ),
        1,
        1,
        "cleric/domain/animal",
    );
    class_skill_from_level(&separatist, 6, "lore-nature");
    // A druid's and an inquisitor's own lists hold Lore (Nature) from level 1; without it there, the domain's shows
    // by their class levels.
    let druid = with_pick(
        with_option(druid(None, 4), 1, 0, "druid/nature-bond/domain"),
        option_slot(1, 0),
        "cleric/domain/animal",
    );
    assert!(engine().is_possible(&druid));
    assert!(has_class_skill(&at(&druid, 1), "lore-nature"));
    class_skill_from_level_by(
        &engine_without_class_skill("druid", "lore-nature"),
        &druid,
        4,
        "lore-nature",
    );
    let inquisitor = with_option(build("inquisitor", 4), 1, 0, "cleric/domain/animal");
    assert!(engine().is_possible(&inquisitor));
    class_skill_from_level_by(
        &engine_without_class_skill("inquisitor", "lore-nature"),
        &inquisitor,
        4,
        "lore-nature",
    );
}

/// Every feature a prerequisite names in the real data, wherever the prerequisite sits.
fn prerequisite_features(data: &GameData) -> BTreeSet<FeatureKey> {
    fn collect(prerequisite: &Prerequisite, out: &mut BTreeSet<FeatureKey>) {
        match prerequisite {
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                children.iter().for_each(|child| collect(child, out));
            }
            Prerequisite::Not(child) => collect(child, out),
            Prerequisite::Feature { feature, .. } => {
                out.insert(feature.clone());
            }
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    for prerequisite in data_prerequisites(data) {
        collect(prerequisite, &mut out);
    }
    out
}

/// Every prerequisite in the real data: feats', classes', archetypes', selection options', mythic abilities' and
/// mythic feats'.
fn data_prerequisites(data: &GameData) -> Vec<&Prerequisite> {
    let selections = data
        .classes
        .iter()
        .chain(&data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .chain(data.mythic_paths.iter().flat_map(|path| &path.selections));
    let prerequisites = data
        .feats
        .iter()
        .filter_map(|feat| feat.prerequisite.as_ref())
        .chain(
            data.classes
                .iter()
                .chain(&data.prestige_classes)
                .filter_map(|class| class.prerequisite.as_ref()),
        )
        .chain(data.archetypes().filter_map(|a| a.prerequisite.as_ref()))
        .chain(
            selections
                .flat_map(|selection| &selection.options)
                .filter_map(|option| option.prerequisite.as_ref()),
        )
        .chain(
            data.mythic_abilities
                .iter()
                .filter_map(|a| a.prerequisite.as_ref()),
        )
        .chain(
            data.mythic_feats
                .iter()
                .filter_map(|f| f.prerequisite.as_ref()),
        );
    prerequisites.collect()
}

/// A picker's reasons name features by their names: every feature a prerequisite needs has one the sheet knows,
/// so no reason shows a bare key.
#[test]
fn every_feature_a_prerequisite_needs_is_named() {
    let engine = engine();
    let features = prerequisite_features(&DATA);
    assert!(!features.is_empty());
    let unnamed: Vec<&FeatureKey> = features
        .iter()
        .filter(|feature| engine.feature_name(feature) == feature.as_str())
        .collect();
    assert!(unnamed.is_empty(), "unnamed: {unnamed:?}");
}

/// Features that share a name read apart in a reason: each one a prerequisite needs reads its own way, and one whose
/// name another feature shares says whose it is ("Weapon Training (Fighter) or Weapon Training (Sohei)"). Of the
/// alternatives, a reason names those the character's classes give.
#[test]
fn features_sharing_a_name_read_apart_in_a_reason() {
    let engine = engine();
    let mut seen: HashMap<String, &FeatureKey> = HashMap::new();
    let features = prerequisite_features(&DATA);
    for feature in &features {
        let text = engine.prerequisite_text(
            &Prerequisite::Feature {
                feature: feature.clone(),
                count: 1,
            },
            None,
        );
        if let Some(other) = seen.insert(text.clone(), feature) {
            panic!("{other} and {feature} both read {text:?}");
        }
    }
    // The reason names only the alternatives the character's classes give, all of them when none does.
    let reason = |build: &Build| {
        let advanced = engine
            .options(
                build,
                &SlotId::Feat {
                    level: 1,
                    source: FeatSource::General,
                    index: 0,
                },
            )
            .unwrap()
            .into_iter()
            .find(|option| option.pick == Pick::feat(FeatKey::new("advanced-weapon-training")))
            .unwrap();
        engine.issue_text(&advanced.status.unwrap_err()).unwrap()
    };
    assert_eq!(
        reason(&build("fighter", 1)),
        "Needs Weapon Training (Fighter), Fighter level 5 (have 1)"
    );
    assert_eq!(
        reason(&archetype_build("monk", "monk/sohei", 1)),
        "Needs Weapon Training (Sohei), Fighter level 5 (have 0)"
    );
    assert_eq!(
        reason(&build("wizard", 1)),
        "Needs (Weapon Training (Spears / Polearms) or Weapon Training (Fighter) or Weapon Training (Sohei)), \
         Fighter level 5 (have 0)"
    );
}

/// Every slot of a long real Build has a label, and a reason reads as the game says it.
#[test]
fn the_sheet_labels_a_real_build() {
    let engine = engine();
    let build = build("fighter", 20);
    let sheet = engine.evaluate(&build);
    for slot in sheet.levels.iter().flat_map(|level| &level.slots) {
        let label = engine.slot_label(slot);
        assert!(!label.is_empty() && !label.contains('/'), "{label}");
    }
    let cleave = engine
        .options(
            &build,
            &SlotId::Feat {
                level: 1,
                source: FeatSource::General,
                index: 0,
            },
        )
        .unwrap()
        .into_iter()
        .find(|option| option.pick == Pick::feat(FeatKey::new("cleave")))
        .unwrap();
    assert_eq!(
        engine.issue_text(&cleave.status.unwrap_err()).as_deref(),
        Some("Needs Power Attack")
    );
}

// ---- the extraction report's gaps (ticket 43) -----------------------------------------------------------------

/// The barbarian's four totem lines, each Lesser, plain and Greater.
const TOTEM_LINES: [&str; 4] = ["beast", "celestial", "daemon", "fiend"];

fn totem_options(line: &str) -> [String; 3] {
    [
        format!("barbarian/rage-power/{line}-totem-lesser"),
        format!("barbarian/rage-power/{line}-totem"),
        format!("barbarian/rage-power/{line}-totem-greater"),
    ]
}

fn selection_option(key: &str) -> &'static rules::SelectionOption {
    DATA.classes
        .iter()
        .chain(&DATA.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .flat_map(|selection| &selection.options)
        .find(|option| option.key.as_str() == key)
        .unwrap_or_else(|| panic!("no option {key}"))
}

/// The options a prerequisite rules out with a `Not(SelectionOption(..))` in its top-level `All` (or as itself).
fn ruled_out(prerequisite: Option<&Prerequisite>) -> BTreeSet<String> {
    let children = match prerequisite {
        Some(Prerequisite::All(children)) => children.iter().collect(),
        Some(other) => vec![other],
        None => Vec::new(),
    };
    children
        .into_iter()
        .filter_map(|child| match child {
            Prerequisite::Not(inner) => match inner.as_ref() {
                Prerequisite::SelectionOption(option) => Some(option.to_string()),
                other => panic!("an unexpected Not: {other:?}"),
            },
            _ => None,
        })
        .collect()
}

fn barbarian(levels: usize) -> Build {
    let mut build = build("barbarian", levels);
    build.start.alignment = Some(rules::Alignment::ChaoticNeutral);
    build
}

fn not_option(key: &str) -> Prerequisite {
    Prerequisite::Not(Box::new(Prerequisite::SelectionOption(
        SelectionOptionKey::new(key),
    )))
}

/// '"Totem" powers are exclusive. Selecting one "Totem" power closes off the other categories.' (barbarian#rage-power)
#[test]
fn the_totem_rage_powers_are_exclusive_one_line_per_character() {
    // Each totem power rules out the other three lines' nine powers; no other rage power rules anything out.
    for line in TOTEM_LINES {
        let others: BTreeSet<String> = TOTEM_LINES
            .iter()
            .filter(|other| **other != line)
            .flat_map(|other| totem_options(other))
            .collect();
        for key in totem_options(line) {
            assert_eq!(
                ruled_out(selection_option(&key).prerequisite.as_ref()),
                others,
                "{key}"
            );
        }
    }
    let rage_power = DATA
        .classes
        .iter()
        .find(|class| class.key.as_str() == "barbarian")
        .unwrap()
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == "barbarian/rage-power")
        .unwrap();
    for option in &rage_power.options {
        if !option.key.as_str().contains("-totem") {
            assert_eq!(ruled_out(option.prerequisite.as_ref()), BTreeSet::new());
        }
    }

    // A barbarian with Beast Totem, Lesser at 2 is offered no other line's power, and his own line's chain.
    let mut build = barbarian(10);
    build.picks.insert(
        selection_slot(2),
        pick("barbarian/rage-power/beast-totem-lesser"),
    );
    let at_4 = offered(&build, 4);
    let beast_lesser = not_option("barbarian/rage-power/beast-totem-lesser");
    for line in ["celestial", "daemon", "fiend"] {
        for key in totem_options(line) {
            let status = at_4[&key].clone();
            match status {
                Err(Issue::Unmet(Unmet::Condition { needs, .. })) => {
                    assert_eq!(needs, beast_lesser)
                }
                Err(Issue::Unmet(Unmet::All(children))) => assert!(
                    children.contains(&Unmet::Condition {
                        needs: beast_lesser.clone(),
                        has: None
                    }),
                    "{key}: {children:?}"
                ),
                other => panic!("{key}: {other:?}"),
            }
        }
    }
    build
        .picks
        .insert(selection_slot(6), pick("barbarian/rage-power/beast-totem"));
    assert!(possible(&build));
    assert_eq!(
        offered(&build, 10)["barbarian/rage-power/beast-totem-greater"],
        Ok(())
    );

    // Two lines can't be held: Fiend Totem, Lesser after Beast Totem, Lesser, and the reason says why.
    let mut two = barbarian(4);
    two.picks.insert(
        selection_slot(2),
        pick("barbarian/rage-power/beast-totem-lesser"),
    );
    two.picks.insert(
        selection_slot(4),
        pick("barbarian/rage-power/fiend-totem-lesser"),
    );
    let issue = why_not(&two, &selection_slot(4)).expect("offered, locked");
    assert_eq!(issue, unmet(beast_lesser, None).unwrap());
    assert_eq!(
        engine().issue_text(&issue).as_deref(),
        Some("Not with Beast Totem, Lesser")
    );
    // Lines other than the one held stay open to a barbarian with none.
    let none = offered(&barbarian(2), 2);
    for line in TOTEM_LINES {
        assert_eq!(
            none[&format!("barbarian/rage-power/{line}-totem-lesser")],
            Ok(())
        );
    }
}

/// "At 1st level, the Eldritch Scoundrel deals extra damage … She does an additional 1d6 damage at 3rd level,
/// increasing by 1d6 every four additional class levels (7, 11, 15, 19)." (rogue#sneak-attack-eldritch-scoundrel)
#[test]
fn an_eldritch_scoundrels_sneak_attack_dice_come_at_1_3_7_11_15_and_19() {
    let sheet = engine().evaluate(&archetype_build("rogue", "rogue/eldritch-scoundrel", 20));
    let dice: Vec<u8> = (1..=20)
        .map(|level| {
            sheet.at(level).unwrap().feature_count(&FeatureKey::new(
                "rogue/eldritch-scoundrel/sneak-attack-eldritch-scoundrel",
            ))
        })
        .collect();
    assert_eq!(
        dice,
        [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6]
    );
    // It is the archetype's only sneak attack.
    assert_eq!(
        sheet
            .at(20)
            .unwrap()
            .feature_count(&FeatureKey::new("rogue/sneak-attack")),
        0
    );
}

/// "At 6th and 14th levels, a Shaman gains a hex slot that can be temporarily filled with a hex granted by either
/// his main spirit or his current Wandering Spirit." (shaman#wandering-hex): a daily choice, as the Wandering Spirit
/// it draws on.
#[test]
fn a_shamans_wandering_hex_is_a_daily_choice_from_6() {
    let wandering_hex = FeatureKey::new("shaman/wandering-hex");
    let daily = |build: &Build, level: u8| engine().daily_choices(&at(build, level));
    let shaman = build("shaman", 14);
    assert!(!daily(&shaman, 5).contains(&wandering_hex));
    assert!(daily(&shaman, 6).contains(&wandering_hex));
    assert!(daily(&shaman, 6).contains(&FeatureKey::new("shaman/wandering-spirit")));
    // The Possessed Shaman has Wandering Skills at 6 and the Wandering Hex at 14 only.
    let possessed = archetype_build("shaman", "shaman/possessed-shaman", 14);
    assert!(!daily(&possessed, 13).contains(&wandering_hex));
    assert!(daily(&possessed, 14).contains(&wandering_hex));
}

// ---- the sorcerer's level-1 feat and Extra Slayer Talent (ticket 44) ------------------------------------------

/// The slot a sorcerer's bloodline feats take: the held bloodline's list.
fn bloodline_feat_slot() -> SlotKind {
    SlotKind::Feat(FeatSlot::OptionList {
        selection: rules::FeatureSelectionKey::new("sorcerer/bloodline"),
        at_level: None,
        label: None,
    })
}

/// A sorcerer bloodline's bonus-feat list, by key: `bloodline_feats(&["abyssal"])`; several lists together.
fn bloodline_feats(bloodlines: &[&str]) -> BTreeSet<String> {
    bloodlines
        .iter()
        .flat_map(|bloodline| {
            let key = format!("sorcerer/bloodline/{bloodline}/feats");
            class("sorcerer")
                .feat_lists
                .iter()
                .find(|list| list.key.as_str() == key)
                .unwrap_or_else(|| panic!("no list {key}"))
                .feats
                .iter()
                .map(|feat| feat.to_string())
        })
        .collect()
}

/// The kind of a Build's class feat slot at `level`, if it has one.
fn class_slot_kind(build: &Build, level: u8) -> Option<SlotKind> {
    engine()
        .evaluate(build)
        .slot(&class_slot(level))
        .map(|slot| slot.kind.clone())
}

/// The status of `feat` in a feat slot.
fn feat_status(build: &Build, slot: &SlotId, feat: &str) -> Result<(), Issue> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .find(|option| option.pick.key() == feat)
        .unwrap_or_else(|| panic!("{feat} isn't offered in {slot:?}"))
        .status
}

/// Every sorcerer table has "Bonus Feat" at 1, in the column of the "Bloodline Feat" at 7, 13 and 19 (ticket 43's
/// decision 1): the base sorcerer's level-1 bonus feat is from the held bloodline's list.
#[test]
fn a_sorcerer_1_picks_a_bonus_feat_from_the_held_bloodlines_list() {
    let draconic = with_option(
        build("sorcerer", 1),
        1,
        0,
        "sorcerer/bloodline/draconic-red",
    );
    assert!(possible(&draconic));
    assert_eq!(class_slot_kind(&draconic, 1), Some(bloodline_feat_slot()));
    // "Blind-Fight, Great Fortitude, Improved Initiative, Quicken Spell, Power Attack, Skill Focus, Toughness": the
    // Draconic list and nothing else. Its Power Attack is offered but locked at sorcerer 1 (BAB +1).
    let offered = feats_offered(&draconic, &class_slot(1));
    assert_eq!(offered, bloodline_feats(&["draconic"]));
    assert!(!offered.contains("dodge") && !offered.contains("combat-casting"));
    assert!(feat_status(&draconic, &class_slot(1), "power-attack").is_err());
    assert_eq!(feat_status(&draconic, &class_slot(1), "toughness"), Ok(()));
    let mut toughness = draconic.clone();
    toughness
        .picks
        .insert(class_slot(1), Pick::feat(FeatKey::new("toughness")));
    assert!(possible(&toughness));
    // An Abyssal sorcerer 1 is offered the Abyssal list, Power Attack among them, and not Toughness.
    let abyssal = with_option(build("sorcerer", 1), 1, 0, ABYSSAL);
    let offered = feats_offered(&abyssal, &class_slot(1));
    assert_eq!(offered, bloodline_feats(&["abyssal"]));
    assert!(!offered.contains("toughness"));
}

/// "Note that at the bonus feat levels, you may choose a feat from one bloodline or the other, not both."
/// (sorcerer#crossblooded)
#[test]
fn a_crossblooded_sorcerer_1_picks_from_either_bloodlines_list() {
    let build = crossblooded("arcane", "abyssal");
    assert_eq!(class_slot_kind(&build, 1), Some(bloodline_feat_slot()));
    let offered = feats_offered(&build, &class_slot(1));
    assert_eq!(offered, bloodline_feats(&["arcane", "abyssal"]));
    assert!(offered.contains("combat-casting") && offered.contains("cleave"));
    assert!(!offered.contains("toughness"));
}

/// The Empyreal, Sage and Sylvan Sorcerers' fixed bloodlines (Celestial, Arcane, Fey) give their level-1 list.
#[test]
fn a_fixed_bloodline_sorcerer_1_picks_from_its_bloodlines_list() {
    for (archetype, bloodline) in [
        ("sorcerer/empyreal-sorcerer", "celestial"),
        ("sorcerer/sage-sorcerer", "arcane"),
        ("sorcerer/sylvan-sorcerer", "fey"),
    ] {
        let build = archetype_build("sorcerer", archetype, 1);
        assert!(possible(&build), "{archetype}");
        assert_eq!(
            class_slot_kind(&build, 1),
            Some(bloodline_feat_slot()),
            "{archetype}"
        );
        let offered = feats_offered(&build, &class_slot(1));
        assert_eq!(offered, bloodline_feats(&[bloodline]), "{archetype}");
        assert!(!offered.contains("power-attack"), "{archetype}");
    }
}

/// "A Seeker chooses a standard bloodline for the purpose of bonus feats and spells." and "At 5th, 10th and 15th
/// levels, a Seeker gains a bonus magic feat." (sorcerer#seeker-sorcerer): its level-1 feat is the bloodline's, its
/// magic feats come at 5, 10 and 15. A Nine Tailed Heir "chooses a standard bloodline for the purpose of bonus feats".
#[test]
fn a_seeker_and_a_nine_tailed_heir_1_pick_from_the_chosen_bloodlines_list() {
    let seeker = with_option(
        archetype_build("sorcerer", "sorcerer/seeker-sorcerer", 15),
        1,
        0,
        ABYSSAL,
    );
    assert!(possible(&seeker));
    assert_eq!(class_slot_kind(&seeker, 1), Some(bloodline_feat_slot()));
    let offered = feats_offered(&seeker, &class_slot(1));
    assert_eq!(offered, bloodline_feats(&["abyssal"]));
    // No longer any Magic feat: Spell Focus is one, and not on the Abyssal list.
    assert!(!offered.contains("spell-focus"));
    for level in [5, 10, 15] {
        assert_eq!(
            class_slot_kind(&seeker, level),
            Some(SlotKind::Feat(FeatSlot::Category(
                rules::FeatCategory::Magic
            ))),
            "{level}"
        );
    }
    // The Nine Tailed Heir is the kitsune's.
    let mut heir = with_option(
        archetype_build("sorcerer", "sorcerer/nine-tailed-heir", 1),
        1,
        0,
        "sorcerer/bloodline/fey",
    );
    heir.start.race = Some(rules::RaceKey::new("kitsune"));
    assert!(possible(&heir));
    assert_eq!(class_slot_kind(&heir, 1), Some(bloodline_feat_slot()));
    assert_eq!(
        feats_offered(&heir, &class_slot(1)),
        bloodline_feats(&["fey"])
    );
}

/// "Does not gain bloodline spells or bloodline feats." (sorcerer#geomancer): its level-1 bonus feat stays any feat.
/// "Does not receive bonus feats." (sorcerer#overwhelming-mage): no slot at 1.
#[test]
fn a_geomancer_1_still_takes_any_feat_and_an_overwhelming_mage_none() {
    let geomancer = with_option(
        archetype_build("sorcerer", "sorcerer/geomancer", 1),
        1,
        0,
        ABYSSAL,
    );
    assert_eq!(
        class_slot_kind(&geomancer, 1),
        Some(SlotKind::Feat(FeatSlot::General))
    );
    let offered = feats_offered(&geomancer, &class_slot(1));
    assert!(offered.contains("power-attack") && offered.contains("dodge"));
    let mage = with_option(
        archetype_build("sorcerer", "sorcerer/overwhelming-mage", 1),
        1,
        0,
        ABYSSAL,
    );
    assert_eq!(class_slot_kind(&mage, 1), None);
}

/// The sheet names the level-1 slot as it names the 7th's: "Bloodline feat" (a Geomancer's: "Bonus feat").
#[test]
fn the_sheet_labels_a_sorcerers_level_1_feat_bloodline_feat() {
    let engine = engine();
    let label = |build: &Build, level: u8| {
        let sheet = engine.evaluate(build);
        engine.slot_label(sheet.slot(&class_slot(level)).expect("a class feat slot"))
    };
    let base = with_option(build("sorcerer", 20), 1, 0, ABYSSAL);
    assert_eq!(label(&base, 1), "Bloodline feat");
    assert_eq!(label(&base, 7), "Bloodline feat");
    for archetype in [
        "sorcerer/empyreal-sorcerer",
        "sorcerer/sage-sorcerer",
        "sorcerer/sylvan-sorcerer",
    ] {
        assert_eq!(
            label(&archetype_build("sorcerer", archetype, 1), 1),
            "Bloodline feat",
            "{archetype}"
        );
    }
    assert_eq!(
        label(&crossblooded("arcane", "abyssal"), 1),
        "Bloodline feat"
    );
    let geomancer = archetype_build("sorcerer", "sorcerer/geomancer", 1);
    assert_eq!(label(&geomancer, 1), "Bonus feat");
    let sheet = engine.evaluate(&base);
    for slot in sheet.levels.iter().flat_map(|level| &level.slots) {
        let label = engine.slot_label(slot);
        assert!(!label.is_empty() && !label.contains('/'), "{label}");
    }
}

/// "Prerequisites: Slayer Talents or Talented Slayer" (feats#extra-slayer-talent), read as slayer level 2, the level
/// the slayer's table first gives Slayer Talents (ticket 43's decision 3; Talented Slayer is on no saved page), or
/// the Sanctified Slayer's own "Slayer Talents" from inquisitor 8 (ticket 60).
#[test]
fn extra_slayer_talent_needs_slayer_level_2() {
    assert_eq!(
        feat("extra-slayer-talent").prerequisite,
        Some(Prerequisite::Any(vec![
            Prerequisite::All(vec![
                Prerequisite::ClassLevel {
                    class: ClassKey::new("inquisitor"),
                    level: 8,
                },
                Prerequisite::Archetype(ArchetypeKey::new("inquisitor/sanctified-slayer")),
            ]),
            Prerequisite::ClassLevel {
                class: ClassKey::new("slayer"),
                level: 2,
            },
        ]))
    );
    let status = feat_status(&build("slayer", 1), &general_slot(1), "extra-slayer-talent");
    assert_eq!(
        engine().issue_text(&status.unwrap_err()).as_deref(),
        Some("Needs Slayer level 2 (have 1)")
    );
    // A fighter 1 / slayer 2 takes it at character level 3.
    let slayer_2 = classes(&["fighter", "slayer", "slayer"]);
    assert_eq!(
        feat_status(&slayer_2, &general_slot(3), "extra-slayer-talent"),
        Ok(())
    );
    let rogue = feat_status(&build("rogue", 1), &general_slot(1), "extra-slayer-talent");
    assert!(
        engine()
            .issue_text(&rogue.unwrap_err())
            .unwrap()
            .ends_with("or Slayer level 2 (have 0)"),
    );
}

// ---- the kineticist's wild talents (ticket 45) -----------------------------------------------------------------

const WILD_TALENTS: &str = "kineticist/wild-talents";

fn talent(name: &str) -> String {
    format!("{WILD_TALENTS}/{name}")
}

/// Each wild talent the kineticist picks (the four defensive ones are each element's 2nd-level grant), with its
/// level, its elements (none: Universal or All) and what else the guide's "Prerequisites" names.
const TALENTS: [(&str, u8, &[&str], Option<&str>); 22] = [
    ("aerial-adaptation", 1, &["air"], None),
    ("cold-adaptation", 1, &["fire", "water"], None),
    ("elemental-whispers", 1, &[], None),
    ("heat-adaptation", 1, &["fire", "water"], None),
    ("fires-fury", 1, &["fire"], None),
    (
        "flame-shield",
        1,
        &["fire"],
        Some("kineticist/element/fire/searing-flesh"),
    ),
    ("kinetic-healer", 1, &["water"], None),
    ("kineticist-bonus-feat", 1, &[], None),
    ("skill-focus", 1, &[], None),
    ("skilled-kineticist", 1, &[], None),
    ("slick", 1, &["water"], None),
    ("foxfire", 2, &["fire"], None),
    (
        "aerial-evasion",
        3,
        &["air"],
        Some("kineticist/element/air/enveloping-winds"),
    ),
    ("celerity", 3, &["air"], None),
    (
        "jagged-flesh",
        3,
        &["earth"],
        Some("kineticist/element/earth/flesh-of-stone"),
    ),
    ("kinetic-restoration", 3, &[], None),
    ("enduring-earth", 4, &["earth"], None),
    ("expanded-defense", 4, &[], None),
    (
        "healing-burst",
        4,
        &["water"],
        Some("kineticist/wild-talents/kinetic-healer"),
    ),
    (
        "kinetic-revivification",
        6,
        &["water"],
        Some("kineticist/wild-talents/kinetic-healer"),
    ),
    ("tremor-sense", 6, &["earth"], None),
    ("tidal-wave", 9, &["water"], None),
];

/// What a talent of `level` with these elements and that other prerequisite needs: the kineticist level twice the
/// talent's, one of its elements, and the other (a Universal talent: the level alone).
fn talent_prerequisite(level: u8, elements: &[&str], other: Option<&str>) -> Prerequisite {
    let element = |e: &&str| {
        Prerequisite::SelectionOption(SelectionOptionKey::new(format!("kineticist/element/{e}")))
    };
    let mut all = vec![Prerequisite::ClassLevel {
        class: ClassKey::new("kineticist"),
        level: 2 * level,
    }];
    match elements {
        [] => {}
        [one] => all.push(element(one)),
        several => all.push(Prerequisite::Any(several.iter().map(element).collect())),
    }
    if let Some(other) = other {
        all.push(match other.strip_prefix("kineticist/wild-talents/") {
            Some(_) => Prerequisite::SelectionOption(SelectionOptionKey::new(other)),
            None => Prerequisite::Feature {
                feature: FeatureKey::new(other),
                count: 1,
            },
        });
    }
    let mut prerequisite = match all.len() {
        1 => all.pop().unwrap(),
        _ => Prerequisite::All(all),
    };
    prerequisite.canonicalize();
    prerequisite
}

/// A kineticist of `element` with `levels` levels and these picks at the selection slots of their levels (a wild
/// talent at an even level, Expanded Element at 7 and 15).
fn kineticist_with(element: &str, levels: usize, picks: &[(u8, &str)]) -> Build {
    let mut build = kineticist(element, levels);
    for (level, key) in picks {
        build.picks.insert(selection_slot(*level), pick(key));
    }
    build
}

/// The wild talents a slot leaves open, by key.
fn open_talents(build: &Build, level: u8) -> BTreeSet<String> {
    offered(build, level)
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .collect()
}

fn talent_reason(build: &Build, level: u8, name: &str) -> String {
    let status = offered(build, level)
        .remove(&talent(name))
        .unwrap_or_else(|| panic!("{name} isn't offered at {level}"));
    engine().issue_text(&status.expect_err("locked")).unwrap()
}

fn bonus_feat_slot(level: u8) -> SlotId {
    SlotId::OptionFeat { level, index: 0 }
}

/// An element's five wild-talent bonus feats, by key.
fn element_feats(elements: &[&str]) -> BTreeSet<String> {
    elements
        .iter()
        .flat_map(|element| {
            let key = format!("kineticist/element/{element}/wild-talent-feats");
            class("kineticist")
                .feat_lists
                .iter()
                .find(|list| list.key.as_str() == key)
                .unwrap_or_else(|| panic!("no list {key}"))
                .feats
                .iter()
                .map(|feat| feat.to_string())
        })
        .collect()
}

/// "At 2nd level and every subsequent even-numbered level, the Kineticist selects a Utility Wild Talent." The guide's
/// 26 talents less the four defensive ones, each with its level and element conditions.
#[test]
fn wild_talents_are_a_selection_of_22_talents_with_their_level_and_elements() {
    let kineticist = class("kineticist");
    let selection = kineticist
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == WILD_TALENTS)
        .expect("the wild talents selection");
    assert_eq!(selection.name, "Wild Talents");
    let keys: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    let expected: BTreeSet<String> = TALENTS.iter().map(|(name, ..)| talent(name)).collect();
    assert_eq!(keys, expected);
    for (name, level, elements, other) in TALENTS {
        assert_eq!(
            selection_option(&talent(name)).prerequisite,
            Some(talent_prerequisite(level, elements, other)),
            "{name}"
        );
    }
    // No feature stands for the pick any more.
    assert!(
        kineticist
            .features
            .iter()
            .all(|feature| feature.key.as_str() != "kineticist/wild-talents")
    );
    let levels: Vec<u8> = kineticist
        .levels
        .iter()
        .filter(|row| row.selections.iter().any(|s| s.as_str() == WILD_TALENTS))
        .map(|row| row.level)
        .collect();
    assert_eq!(levels, [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
}

/// The ticket's: a kineticist 2 picks a talent of level 1 at most, from its element's and the Universal ones.
#[test]
fn a_fire_kineticist_2_is_offered_its_elements_and_universal_level_1_talents() {
    let build = kineticist("fire", 2);
    assert!(possible(&build));
    let open: BTreeSet<String> = [
        "cold-adaptation",
        "elemental-whispers",
        "heat-adaptation",
        "fires-fury",
        "flame-shield",
        "kineticist-bonus-feat",
        "skill-focus",
        "skilled-kineticist",
    ]
    .iter()
    .map(|name| talent(name))
    .collect();
    assert_eq!(open_talents(&build, 2), open);
    // A Water-only talent is locked, by its element.
    assert_eq!(
        talent_reason(&build, 2, "kinetic-healer"),
        "Needs Elemental Focus - Water"
    );
    // Foxfire is a level-2 Fire talent: from kineticist 4.
    assert_eq!(
        talent_reason(&build, 2, "foxfire"),
        "Needs Kineticist level 4 (have 2)"
    );
    let water = kineticist("water", 2);
    assert!(open_talents(&water, 2).contains(&talent("kinetic-healer")));
    assert!(!open_talents(&water, 2).contains(&talent("fires-fury")));
}

/// "The Kineticist's class level must be at least twice the level of the Wild Talent to unlock it."
#[test]
fn a_level_3_talent_opens_at_kineticist_6() {
    let build = kineticist("air", 6);
    assert_eq!(
        talent_reason(&build, 4, "celerity"),
        "Needs Kineticist level 6 (have 4)"
    );
    assert!(open_talents(&build, 6).contains(&talent("celerity")));
    let celerity = kineticist_with("air", 6, &[(6, &talent("celerity"))]);
    assert!(possible(&celerity));
    let early = kineticist_with("air", 6, &[(4, &talent("celerity"))]);
    assert!(!possible(&early));
}

/// Flame Shield needs Searing Flesh, Fire's defensive talent: a Kinetic Knight has it from 4, not 2.
#[test]
fn a_talent_needing_a_defensive_talent_waits_for_it() {
    let knight = with_option(
        archetype_build("kineticist", "kineticist/kinetic-knight", 4),
        1,
        0,
        "kineticist/element/fire",
    );
    assert_eq!(
        talent_reason(&knight, 2, "flame-shield"),
        "Needs Searing Flesh"
    );
    assert!(open_talents(&knight, 4).contains(&talent("flame-shield")));
    // Healing Burst needs Kinetic Healer.
    let water = kineticist_with("water", 8, &[(2, &talent("kinetic-healer"))]);
    assert!(open_talents(&water, 8).contains(&talent("healing-burst")));
    assert_eq!(
        talent_reason(&kineticist("water", 8), 8, "healing-burst"),
        "Needs Kinetic Healer"
    );
}

/// "Select a bonus feat associated with an Elemental Focus. You do not need to meet the normal prerequisites for the
/// feat you select." One slot, from the held elements' lists, taken once.
#[test]
fn the_bonus_feat_talent_grants_one_slot_from_the_held_elements_lists() {
    let build = kineticist_with("fire", 2, &[(2, &talent("kineticist-bonus-feat"))]);
    assert!(possible(&build));
    assert_eq!(
        feats_offered(&build, &bonus_feat_slot(2)),
        element_feats(&["fire"])
    );
    // Improved Iron Will needs Iron Will, but the list ignores prerequisites.
    assert_eq!(
        feat_status(&build, &bonus_feat_slot(2), "improved-iron-will"),
        Ok(())
    );
    let mut with_feat = build.clone();
    with_feat.picks.insert(
        bonus_feat_slot(2),
        Pick::feat(FeatKey::new("improved-iron-will")),
    );
    assert!(possible(&with_feat));
    assert!(
        at(&with_feat, 2)
            .feats
            .contains_key(&FeatKey::new("improved-iron-will"))
    );
    // Not twice.
    let twice = kineticist_with("fire", 4, &[(2, &talent("kineticist-bonus-feat"))]);
    assert_eq!(
        offered(&twice, 4).remove(&talent("kineticist-bonus-feat")),
        Some(Err(Issue::AlreadyHeld))
    );
    // With Expanded Element (Water) at 7, the talent at 8 offers both elements' lists.
    let expanded = kineticist_with(
        "fire",
        8,
        &[
            (7, "kineticist/element/water"),
            (8, &talent("kineticist-bonus-feat")),
        ],
    );
    assert!(possible(&expanded));
    assert_eq!(
        feats_offered(&expanded, &bonus_feat_slot(8)),
        element_feats(&["fire", "water"])
    );
    // A feat off the lists isn't allowed.
    let mut off = build.clone();
    off.picks
        .insert(bonus_feat_slot(2), Pick::feat(FeatKey::new("power-attack")));
    assert!(!possible(&off));
}

/// "You unlock the Defensive Wild Talent for any element you selected with Expanded Element."
#[test]
fn expanded_defense_unlocks_the_expanded_elements_defensive_talent() {
    let without = kineticist_with("fire", 10, &[(7, "kineticist/element/air")]);
    assert!(!holds(
        &at(&without, 10),
        "kineticist/element/air/enveloping-winds"
    ));
    assert_eq!(
        talent_reason(&without, 10, "aerial-evasion"),
        "Needs Enveloping Winds"
    );
    let with = kineticist_with(
        "fire",
        10,
        &[
            (7, "kineticist/element/air"),
            (8, &talent("expanded-defense")),
        ],
    );
    assert!(possible(&with));
    from_level(&with, 8, "kineticist/element/air/enveloping-winds");
    assert!(open_talents(&with, 10).contains(&talent("aerial-evasion")));
    // An element taken at 15, after the talent, gets its defensive talent too.
    let later = kineticist_with(
        "fire",
        15,
        &[
            (7, "kineticist/element/air"),
            (8, &talent("expanded-defense")),
            (15, "kineticist/element/water"),
        ],
    );
    assert!(possible(&later));
    from_level(&later, 15, "kineticist/element/water/shroud-of-water");
}

/// The Skill Focus talent: "Identical to the feat", for a skill of the player's choosing.
#[test]
fn the_skill_focus_talent_is_skill_focus_in_a_skill() {
    let mut build = kineticist("earth", 2);
    build.picks.insert(
        selection_slot(2),
        Pick::option_for(
            SelectionOptionKey::new(talent("skill-focus")),
            skill("perception"),
        ),
    );
    assert!(possible(&build));
    assert!(at(&build, 2).has_feat_for(&FeatKey::new("skill-focus"), &skill("perception")));
}

/// The archetypes keep the wild talents at every even level: the Kinetic Knight's 4th and the Blood Kineticist's
/// 20th rows, which differ from the base's, pick one too.
#[test]
fn every_kineticist_archetype_picks_wild_talents_at_even_levels() {
    for archetype in &class("kineticist").archetypes {
        let build = archetype_build("kineticist", archetype.key.as_str(), 20);
        let sheet = engine().evaluate(&build);
        let levels: Vec<u8> = (1..=20)
            .filter(|&level| {
                sheet.slot(&selection_slot(level)).is_some_and(|slot| {
                    slot.kind == SlotKind::Selection(rules::FeatureSelectionKey::new(WILD_TALENTS))
                })
            })
            .collect();
        assert_eq!(
            levels,
            [2, 4, 6, 8, 10, 12, 14, 16, 18, 20],
            "{}",
            archetype.key
        );
    }
}

/// The sheet names the wild talent slot after the selection, "Wild Talents", and the bonus-feat talent's slot "Wild
/// talent feat" (ticket 47; the user's name for it, not the Elemental Focus's whose lists it offers).
#[test]
fn the_sheet_labels_the_wild_talent_and_its_bonus_feat() {
    let engine = engine();
    let build = kineticist_with(
        "fire",
        20,
        &[
            (2, &talent("kineticist-bonus-feat")),
            (7, "kineticist/element/water"),
        ],
    );
    assert!(possible(&build));
    let sheet = engine.evaluate(&build);
    let label = |slot: &SlotId| engine.slot_label(sheet.slot(slot).expect("a slot"));
    assert_eq!(label(&selection_slot(2)), "Wild Talents");
    assert_eq!(label(&selection_slot(20)), "Wild Talents");
    assert_eq!(label(&bonus_feat_slot(2)), "Wild talent feat");
    for slot in sheet.levels.iter().flat_map(|level| &level.slots) {
        let label = engine.slot_label(slot);
        assert!(!label.is_empty() && !label.contains('/'), "{label}");
    }
}

/// "Prerequisites: Wild Talents" (feats#extra-wild-talent): the feature the even rows held is a pick now, so the
/// feat needs the level of the first one, kineticist 2, as Extra Slayer Talent needs slayer 2.
#[test]
fn extra_wild_talent_needs_kineticist_level_2() {
    assert_eq!(
        feat("extra-wild-talent").prerequisite,
        Some(Prerequisite::ClassLevel {
            class: ClassKey::new("kineticist"),
            level: 2,
        })
    );
    let status = feat_status(
        &kineticist("fire", 1),
        &general_slot(1),
        "extra-wild-talent",
    );
    assert_eq!(
        engine().issue_text(&status.unwrap_err()).as_deref(),
        Some("Needs Kineticist level 2 (have 1)")
    );
    assert_eq!(
        feat_status(
            &kineticist("fire", 3),
            &general_slot(3),
            "extra-wild-talent"
        ),
        Ok(())
    );
}

// ---- talents that repeat (ticket 47) ---------------------------------------------------------------------------

const ROGUE_TALENTS: &str = "rogue/rogue-talents";

fn rogue_talent(name: &str) -> String {
    format!("{ROGUE_TALENTS}/{name}")
}

/// A pick of an option for a value (Skill Focus (Perception)).
fn pick_for(key: &str, value: Parameter) -> Pick {
    Pick::option_for(SelectionOptionKey::new(key), value)
}

/// The status of each pick a slot offers, by pick (an option once for each value it may name).
fn statuses(build: &Build, slot: &SlotId) -> HashMap<Pick, Result<(), Issue>> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .map(|SlotOption { pick, status }| (pick, status))
        .collect()
}

/// A Build of one class with these picks at the selection slots of their levels.
fn with_picks(mut build: Build, picks: &[(u8, Pick)]) -> Build {
    for (level, pick) in picks {
        build.picks.insert(selection_slot(*level), pick.clone());
    }
    build
}

/// A feat slot an option's row grants: the first one of its level.
fn option_feat_slot(level: u8) -> SlotId {
    SlotId::OptionFeat { level, index: 0 }
}

/// The Skill Focus wild talent and the rogue talent Skill Focus repeat once per skill (user, 2026-11-02): the option
/// takes the skill of its chosen feat, Skill Focus, once for each.
#[test]
fn the_skill_focus_talents_repeat_once_per_skill_and_weapon_focus_doesnt() {
    for key in [talent("skill-focus"), rogue_talent("skill-focus")] {
        let option = selection_option(&key);
        assert_eq!(
            option.chosen_feat,
            Some(FeatKey::new("skill-focus")),
            "{key}"
        );
        assert_eq!(option.repeat, Repeat::PerValue, "{key}");
    }
    let weapon_focus = selection_option(&rogue_talent("weapon-focus"));
    assert_eq!(weapon_focus.chosen_feat, Some(FeatKey::new("weapon-focus")));
    assert_eq!(weapon_focus.repeat, Repeat::Once);
}

/// A kineticist takes Skill Focus (Perception) at 2 and Skill Focus (Stealth) at 4; Skill Focus (Perception) again at
/// 6 is refused, another skill is open.
#[test]
fn a_kineticist_takes_the_skill_focus_talent_for_two_skills_but_not_one_twice() {
    let focus = talent("skill-focus");
    let build = with_picks(
        kineticist("earth", 6),
        &[
            (2, pick_for(&focus, skill("perception"))),
            (4, pick_for(&focus, skill("stealth"))),
        ],
    );
    assert!(possible(&build));
    let state = at(&build, 4);
    for value in ["perception", "stealth"] {
        assert!(state.has_feat_for(&FeatKey::new("skill-focus"), &skill(value)));
    }
    let at_6 = statuses(&build, &selection_slot(6));
    assert_eq!(
        at_6[&pick_for(&focus, skill("perception"))],
        Err(Issue::AlreadyHeld)
    );
    assert_eq!(
        at_6[&pick_for(&focus, skill("stealth"))],
        Err(Issue::AlreadyHeld)
    );
    assert_eq!(at_6[&pick_for(&focus, skill("athletics"))], Ok(()));
    let again = with_picks(build, &[(6, pick_for(&focus, skill("perception")))]);
    assert_eq!(
        why_not(&again, &selection_slot(6)),
        Some(Issue::AlreadyHeld)
    );
}

/// A rogue takes the Skill Focus talent for two different skills and is refused a second one for the same skill; a
/// slayer, who shares the rogue talents, too.
#[test]
fn a_rogue_and_a_slayer_take_the_skill_focus_talent_for_two_skills_but_not_one_twice() {
    let focus = rogue_talent("skill-focus");
    for class in ["rogue", "slayer"] {
        let build = with_picks(
            build(class, 6),
            &[
                (2, pick_for(&focus, skill("perception"))),
                (4, pick_for(&focus, skill("stealth"))),
            ],
        );
        assert!(possible(&build), "{class}");
        assert_eq!(
            statuses(&build, &selection_slot(6))[&pick_for(&focus, skill("trickery"))],
            Ok(()),
            "{class}"
        );
        let again = with_picks(build, &[(6, pick_for(&focus, skill("stealth")))]);
        assert_eq!(
            why_not(&again, &selection_slot(6)),
            Some(Issue::AlreadyHeld),
            "{class}"
        );
    }
}

/// The rogue talent Weapon Focus stays taken once (user, 2026-11-02), whatever the weapon.
#[test]
fn the_rogue_weapon_focus_talent_cant_be_taken_twice() {
    let focus = rogue_talent("weapon-focus");
    let build = with_picks(
        build("rogue", 4),
        &[
            (2, pick_for(&focus, weapon("longsword"))),
            (4, pick_for(&focus, weapon("dagger"))),
        ],
    );
    assert_eq!(
        why_not(&build, &selection_slot(4)),
        Some(Issue::AlreadyHeld)
    );
}

/// "Combat Trick: Any combat feat that the character qualifies for." Taken any number of times, each time a bonus
/// Combat feat slot whose feat's prerequisites are checked (user, 2026-11-02).
#[test]
fn combat_trick_repeats_and_grants_a_combat_feat_slot_each_time() {
    let trick = selection_option(&rogue_talent("combat-trick"));
    assert_eq!(trick.repeat, Repeat::Unlimited);
    assert_eq!(
        trick
            .levels
            .iter()
            .map(|row| (row.level, row.feat_slots.clone()))
            .collect::<Vec<_>>(),
        [(1, vec![FeatSlot::CombatFeat])]
    );
    let engine = engine();
    let build = with_picks(
        build("rogue", 4),
        &[
            (2, pick(&rogue_talent("combat-trick"))),
            (4, pick(&rogue_talent("combat-trick"))),
        ],
    );
    assert!(possible(&build));
    let sheet = engine.evaluate(&build);
    for level in [2, 4] {
        let slot = sheet
            .slot(&option_feat_slot(level))
            .expect("a Combat Trick slot");
        assert_eq!(slot.kind, SlotKind::Feat(FeatSlot::CombatFeat));
        assert_eq!(engine.slot_label(slot), "Combat feat");
    }
    // Its feat's prerequisites are checked; a feat that isn't a Combat feat isn't offered.
    let offered = feats_offered(&build, &option_feat_slot(2));
    assert!(offered.contains("weapon-focus"));
    assert!(!offered.contains("skill-focus"));
    let cleave = feat_status(&build, &option_feat_slot(2), "cleave");
    assert_eq!(
        engine.issue_text(&cleave.unwrap_err()).as_deref(),
        Some("Needs Power Attack")
    );
}

/// A rogue takes Combat Trick twice and Weapon Focus (Longsword) in one slot, Weapon Focus (Dagger) in the other;
/// Weapon Focus (Longsword) twice is refused (the feat's own once per weapon). The link holds it all.
#[test]
fn a_rogue_takes_weapon_focus_in_two_weapons_through_two_combat_tricks() {
    let engine = engine();
    let focus = |value: &str| Pick::feat_for(FeatKey::new("weapon-focus"), weapon(value));
    let mut build = with_picks(
        build("rogue", 4),
        &[
            (2, pick(&rogue_talent("combat-trick"))),
            (4, pick(&rogue_talent("combat-trick"))),
        ],
    );
    build.picks.insert(option_feat_slot(2), focus("longsword"));
    build.picks.insert(option_feat_slot(4), focus("dagger"));
    assert!(possible(&build));
    let state = at(&build, 4);
    for value in ["longsword", "dagger"] {
        assert!(state.has_feat_for(&FeatKey::new("weapon-focus"), &weapon(value)));
    }
    assert_eq!(
        engine.open_link(&rules::encode_build(&build)),
        Ok(build.clone())
    );
    assert_eq!(
        statuses(&build, &option_feat_slot(4))[&focus("longsword")],
        Err(Issue::AlreadyHeld)
    );
    let mut twice = build.clone();
    twice.picks.insert(option_feat_slot(4), focus("longsword"));
    assert_eq!(
        why_not(&twice, &option_feat_slot(4)),
        Some(Issue::AlreadyHeld)
    );
    // The rogue talent Weapon Focus beside them: another weapon is open, Longsword isn't.
    let mut rogue_6 = build.clone();
    rogue_6.levels.extend([
        LevelUp::new(ClassKey::new("rogue")),
        LevelUp::new(ClassKey::new("rogue")),
    ]);
    let at_6 = statuses(&rogue_6, &selection_slot(6));
    assert_eq!(
        at_6[&pick_for(&rogue_talent("weapon-focus"), weapon("longsword"))],
        Err(Issue::AlreadyHeld)
    );
    assert_eq!(
        at_6[&pick_for(&rogue_talent("weapon-focus"), weapon("rapier"))],
        Ok(())
    );
}

/// Combat Trick grants its slot through every selection that offers it: the slayer's talents, the Sylvan Trickster's
/// Fey Tricks and the Underground Chemist's Discovery (from rogue 10).
#[test]
fn combat_trick_grants_its_slot_through_every_selection_offering_it() {
    let offering: BTreeSet<String> = DATA
        .classes
        .iter()
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .filter(|selection| {
            selection.offers(&SelectionOptionKey::new(rogue_talent("combat-trick")))
        })
        .map(|selection| selection.key.to_string())
        .collect();
    assert_eq!(
        offering,
        BTreeSet::from(
            [
                "alchemist/vivisectionist/medical-discovery",
                "rogue/rogue-talents",
                "rogue/sylvan-trickster/fey-tricks",
                "rogue/underground-chemist/discovery",
                "shifter/feyform-shifter/rogue-talent",
                "slayer/slayer-talents",
            ]
            .map(String::from)
        )
    );
    let trick = pick(&rogue_talent("combat-trick"));
    for (build, level) in [
        (build("slayer", 2), 2),
        (archetype_build("rogue", "rogue/sylvan-trickster", 2), 2),
        (
            archetype_build("rogue", "rogue/underground-chemist", 10),
            10,
        ),
        // Ticket 60: through the rogue's and the slayer's selections on another class's row, and the archetypes' own.
        (build("bard", 2), 2),
        (build("skald", 2), 2),
        (
            chaotic(archetype_build("shifter", "shifter/feyform-shifter", 8)),
            8,
        ),
        (
            archetype_build("alchemist", "alchemist/vivisectionist", 2),
            2,
        ),
        (
            archetype_build("inquisitor", "inquisitor/sanctified-slayer", 8),
            8,
        ),
    ] {
        let build = with_picks(build, &[(level, trick.clone())]);
        assert!(possible(&build), "{:?}", build.class_choices);
        assert_eq!(
            engine()
                .evaluate(&build)
                .slot(&option_feat_slot(level))
                .map(|slot| slot.kind.clone()),
            Some(SlotKind::Feat(FeatSlot::CombatFeat)),
            "{:?}",
            build.class_choices
        );
    }
}

// ---- Extra feats' extra picks (ticket 49) ---------------------------------------------------------------------

/// The "Extra …" feats whose guide entry gives one more pick of a class selection ("You gain one additional hex"),
/// with the selections that pick may be of: each one the guide's "Prerequisites" name. Extra Mercy's since ticket 54
/// made the paladin's mercies a selection; the Vivisectionist's Medical Discovery and the Hexcrafter's Hex Arcana since
/// ticket 64.
const EXTRA_PICKS: [(&str, &[&str]); 9] = [
    (
        "extra-arcana",
        &["magus/hexcrafter/hex-arcana", "magus/magus-arcana"],
    ),
    ("extra-arcanist-exploit", &["arcanist/arcanist-exploits"]),
    (
        "extra-discovery",
        &[
            "alchemist/discovery",
            "alchemist/vivisectionist/medical-discovery",
            "rogue/underground-chemist/discovery",
        ],
    ),
    ("extra-hex", &["shaman/hex", "witch/hex"]),
    ("extra-mercy", &["paladin/mercy"]),
    ("extra-rage-power", &["barbarian/rage-power"]),
    (
        "extra-rogue-talent",
        &[
            "rogue/rogue-talents",
            "rogue/sylvan-trickster/fey-tricks",
            "shifter/feyform-shifter/rogue-talent",
        ],
    ),
    ("extra-slayer-talent", &["slayer/slayer-talents"]),
    ("extra-wild-talent", &["kineticist/wild-talents"]),
];

fn extra_slot(level: u8, index: u8) -> SlotId {
    SlotId::FeatSelection { level, index }
}

fn selection_kind(key: &str) -> SlotKind {
    SlotKind::Selection(rules::FeatureSelectionKey::new(key))
}

/// A chaotic neutral Build taking `classes`, one per level, with no picks.
fn classes_build(classes: &[&str]) -> Build {
    let mut build = Build::default();
    build.start.alignment = Some(rules::Alignment::ChaoticNeutral);
    for class in classes {
        build
            .class_choices
            .entry(ClassKey::new(*class))
            .or_default();
        build.levels.push(LevelUp::new(ClassKey::new(*class)));
    }
    build
}

/// The Build with every empty Feature selection slot filled, in slot order, with the first option its picker
/// leaves open.
fn with_selections_filled(mut build: Build) -> Build {
    let engine = engine();
    loop {
        let sheet = engine.evaluate(&build);
        let empty = sheet
            .levels
            .iter()
            .flat_map(|level| &level.slots)
            .filter(|slot| {
                matches!(slot.kind, SlotKind::Selection(_)) && !build.picks.contains_key(&slot.id)
            })
            .map(|slot| slot.id.clone())
            .find(|id| {
                engine
                    .options(&build, id)
                    .is_some_and(|options| options.iter().any(|option| option.status.is_ok()))
            });
        let Some(id) = empty else {
            return build;
        };
        let open = engine
            .options(&build, &id)
            .unwrap()
            .into_iter()
            .find(|option| option.status.is_ok())
            .unwrap();
        build.picks.insert(id, open.pick);
    }
}

/// The slot kind the Build's sheet has at `slot`, `None` when it has none.
fn kind_at(build: &Build, slot: &SlotId) -> Option<SlotKind> {
    engine()
        .evaluate(build)
        .slot(slot)
        .map(|slot| slot.kind.clone())
}

#[test]
fn the_extra_feats_that_give_a_pick_name_their_selections() {
    let named: BTreeSet<(String, BTreeSet<String>)> = DATA
        .feats
        .iter()
        .filter(|feat| !feat.extra_pick_of.is_empty())
        .map(|feat| {
            (
                feat.key.to_string(),
                feat.extra_pick_of.iter().map(|s| s.to_string()).collect(),
            )
        })
        .collect();
    let expected: BTreeSet<(String, BTreeSet<String>)> = EXTRA_PICKS
        .iter()
        .map(|(feat, selections)| {
            (
                feat.to_string(),
                selections.iter().map(|s| s.to_string()).collect(),
            )
        })
        .collect();
    assert_eq!(named, expected);
    // The resource ones give no pick.
    for key in [
        "extra-arcane-pool",
        "extra-bane-weapon",
        "extra-bloodrage",
        "extra-bombs",
        "extra-channel",
        "extra-ki",
        "extra-lay-on-hands",
        "extra-performance",
        "extra-rage",
        "extra-reservoir",
    ] {
        assert!(feat(key).extra_pick_of.is_empty(), "{key}");
    }
}

#[test]
fn a_barbarian_with_extra_rage_power_at_3_gets_a_rage_power_slot_there() {
    let engine = engine();
    let mut build = with_selections_filled(barbarian(3));
    build
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    assert!(possible(&build));
    assert_eq!(
        kind_at(&build, &extra_slot(3, 0)),
        Some(selection_kind("barbarian/rage-power"))
    );
    // Its options are the barbarian's rage powers, as the rage power slot at 2 offers them.
    let keys = |slot: &SlotId| -> BTreeSet<String> {
        engine
            .options(&build, slot)
            .unwrap()
            .into_iter()
            .map(|option| option.pick.key().to_owned())
            .collect()
    };
    assert_eq!(keys(&extra_slot(3, 0)), keys(&selection_slot(2)));
    // 33, and the Instinctual Warrior's five monk powers (ticket 66), locked here.
    assert_eq!(keys(&extra_slot(3, 0)).len(), 38);
    let sheet = engine.evaluate(&build);
    let slot = sheet.slot(&extra_slot(3, 0)).unwrap();
    assert_eq!(engine.slot_label(slot), "Rage Power");

    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(3, 0)));
    assert!(possible(&filled));
}

/// An unparameterised feat.
fn feat_pick_plain(key: &str) -> Pick {
    Pick::feat(FeatKey::new(key))
}

#[test]
fn extra_rage_power_taken_again_gives_a_second_slot() {
    let mut build = with_selections_filled(barbarian(5));
    for level in [3, 5] {
        build
            .picks
            .insert(general_slot(level), feat_pick_plain("extra-rage-power"));
    }
    let build = with_selections_filled(build);
    assert!(possible(&build));
    for level in [3, 5] {
        assert_eq!(
            kind_at(&build, &extra_slot(level, 0)),
            Some(selection_kind("barbarian/rage-power")),
            "at {level}"
        );
        assert!(build.picks.contains_key(&extra_slot(level, 0)));
    }
}

#[test]
fn extra_rage_power_isnt_removable_while_its_slots_pick_depends_on_it_and_takes_the_slot_with_it() {
    let engine = engine();
    let mut build = with_selections_filled(barbarian(3));
    build
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    let mut build = with_selections_filled(build);
    assert!(build.picks.contains_key(&extra_slot(3, 0)));
    let feat = rules::Choice::Slot(general_slot(3));
    assert!(!engine.is_removable(&build, &feat));
    assert!(!engine.removable(&build).contains(&feat));
    assert!(engine.is_removable(&build, &rules::Choice::Slot(extra_slot(3, 0))));

    build.apply(&rules::Edit::Clear(extra_slot(3, 0)));
    assert!(engine.is_removable(&build, &feat));
    build.apply(&rules::Edit::Clear(general_slot(3)));
    assert!(possible(&build));
    assert_eq!(kind_at(&build, &extra_slot(3, 0)), None);
}

#[test]
fn a_witchs_extra_hex_gives_a_hex_slot() {
    let mut build = with_selections_filled(classes_build(&["witch"]));
    build
        .picks
        .insert(general_slot(1), feat_pick_plain("extra-hex"));
    assert!(possible(&build));
    assert_eq!(
        kind_at(&build, &extra_slot(1, 0)),
        Some(selection_kind("witch/hex"))
    );
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(1, 0)));
    assert!(possible(&filled));
}

/// Each feat gives its slot to a character of the class whose selection it names: the feat in the general slot at
/// `level`, after the class's earlier picks.
#[test]
fn each_extra_feat_gives_a_slot_of_its_classs_selection() {
    let cases: [(Build, &str, u8, &str); 22] = [
        (barbarian(3), "extra-rage-power", 3, "barbarian/rage-power"),
        (classes_build(&["witch"]), "extra-hex", 1, "witch/hex"),
        (
            classes_build(&times("shaman", 3)),
            "extra-hex",
            3,
            "shaman/hex",
        ),
        (
            classes_build(&times("magus", 3)),
            "extra-arcana",
            3,
            "magus/magus-arcana",
        ),
        (
            classes_build(&["arcanist"]),
            "extra-arcanist-exploit",
            1,
            "arcanist/arcanist-exploits",
        ),
        (
            classes_build(&times("alchemist", 3)),
            "extra-discovery",
            3,
            "alchemist/discovery",
        ),
        (
            chaotic(archetype_build("rogue", "rogue/underground-chemist", 11)),
            "extra-discovery",
            11,
            "rogue/underground-chemist/discovery",
        ),
        (
            classes_build(&times("rogue", 3)),
            "extra-rogue-talent",
            3,
            "rogue/rogue-talents",
        ),
        (
            chaotic(archetype_build("rogue", "rogue/sylvan-trickster", 3)),
            "extra-rogue-talent",
            3,
            "rogue/sylvan-trickster/fey-tricks",
        ),
        (
            classes_build(&times("slayer", 3)),
            "extra-slayer-talent",
            3,
            "slayer/slayer-talents",
        ),
        (
            classes_build(&times("kineticist", 3)),
            "extra-wild-talent",
            3,
            "kineticist/wild-talents",
        ),
        // A witch who becomes a shaman: her Extra Hex is a witch's hex, the selection she had a slot of first.
        (
            classes_build(&["witch", "shaman", "shaman"]),
            "extra-hex",
            3,
            "witch/hex",
        ),
        // Ticket 54: the paladin's mercies and the Arcane Enforcer's exploits are picks.
        (paladin(3), "extra-mercy", 3, "paladin/mercy"),
        (
            enforcer(1),
            "extra-arcanist-exploit",
            1,
            "arcanist/arcanist-exploits",
        ),
        // Ticket 57: the Exploiter Wizard's exploits are picks too.
        (
            exploiter(1),
            "extra-arcanist-exploit",
            1,
            "arcanist/arcanist-exploits",
        ),
        // Ticket 60: talents and hexes taken through another class.
        (
            classes_build(&times("bard", 3)),
            "extra-rogue-talent",
            3,
            "rogue/rogue-talents",
        ),
        (
            classes_build(&times("skald", 3)),
            "extra-rogue-talent",
            3,
            "rogue/rogue-talents",
        ),
        (
            chaotic(archetype_build("shifter", "shifter/feyform-shifter", 9)),
            "extra-rogue-talent",
            9,
            "shifter/feyform-shifter/rogue-talent",
        ),
        (
            chaotic(archetype_build(
                "inquisitor",
                "inquisitor/sanctified-slayer",
                9,
            )),
            "extra-slayer-talent",
            9,
            "slayer/slayer-talents",
        ),
        (
            chaotic(archetype_build("magus", "magus/hexcrafter", 5)),
            "extra-hex",
            5,
            "witch/hex",
        ),
        // Ticket 64: the Vivisectionist's Medical Discovery and the Hexcrafter's Hex Arcana.
        (
            chaotic(archetype_build("alchemist", "alchemist/vivisectionist", 3)),
            "extra-discovery",
            3,
            "alchemist/vivisectionist/medical-discovery",
        ),
        (
            chaotic(archetype_build("magus", "magus/hexcrafter", 7)),
            "extra-arcana",
            7,
            "magus/hexcrafter/hex-arcana",
        ),
    ];
    for (build, feat, level, selection) in cases {
        let mut build = with_selections_filled(build);
        build
            .picks
            .insert(general_slot(level), feat_pick_plain(feat));
        assert!(possible(&build), "{feat} at {level}");
        assert_eq!(
            kind_at(&build, &extra_slot(level, 0)),
            Some(selection_kind(selection)),
            "{feat} at {level}"
        );
        let filled = with_selections_filled(build);
        assert!(
            filled.picks.contains_key(&extra_slot(level, 0)),
            "{feat} at {level}: nothing open"
        );
        assert!(possible(&filled), "{feat} at {level}");
    }
}

fn times(class: &'static str, n: usize) -> Vec<&'static str> {
    vec![class; n]
}

fn chaotic(mut build: Build) -> Build {
    build.start.alignment = Some(rules::Alignment::ChaoticNeutral);
    build
}

// ---- Paladin mercies and the Arcane Enforcer's exploits (ticket 54) -------------------------------------------

const MERCY: &str = "paladin/mercy";
const EXPLOITS: &str = "arcanist/arcanist-exploits";

/// The guide's mercies (`paladin#mercy`): each one's key, the paladin level it opens at and the mercy it needs.
const MERCIES: [(&str, u8, Option<&str>); 15] = [
    ("fatigued", 3, None),
    ("shaken", 3, None),
    ("sickened", 3, None),
    ("dazed", 6, None),
    ("diseased", 6, None),
    ("staggered", 6, None),
    ("confused", 9, None),
    ("cursed", 9, None),
    ("exhausted", 9, Some("fatigued")),
    ("frightened", 9, Some("shaken")),
    ("nauseated", 9, Some("sickened")),
    ("poisoned", 9, None),
    ("blinded", 12, None),
    ("paralyzed", 12, None),
    ("stunned", 12, None),
];

fn mercy(name: &str) -> String {
    format!("{MERCY}/{name}")
}

/// A lawful good paladin of `levels` levels, with no picks.
fn paladin(levels: usize) -> Build {
    let mut build = build("paladin", levels);
    build.start.alignment = Some(rules::Alignment::LawfulGood);
    build
}

/// A lawful good paladin of `archetype` (a dwarf: the Stonelord's race), `levels` levels, with no picks.
fn paladin_of(archetype: &str, levels: usize) -> Build {
    let mut build = archetype_build("paladin", archetype, levels);
    build.start.alignment = Some(rules::Alignment::LawfulGood);
    build.start.race = Some(rules::RaceKey::new("dwarf"));
    build
}

/// A chaotic neutral Arcane Enforcer of `levels` levels, with no picks.
fn enforcer(levels: usize) -> Build {
    chaotic(archetype_build("slayer", "slayer/arcane-enforcer", levels))
}

/// The levels at which the Build's sheet has a slot of `selection`, in order.
fn levels_with_slot_of(build: &Build, selection: &str) -> Vec<u8> {
    let sheet = engine().evaluate(build);
    sheet
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .filter(|slot| slot.kind == selection_kind(selection))
        .map(|slot| match slot.id {
            SlotId::Selection { level, .. } => level,
            ref other => panic!("a {selection} slot at {other:?}"),
        })
        .collect()
}

/// The options the selection slot of `level` leaves open, by key.
fn open_at(build: &Build, level: u8) -> BTreeSet<String> {
    offered(build, level)
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .collect()
}

/// Why the selection slot of `level` locks `option`, as the sheet says it.
fn lock_reason(build: &Build, level: u8, option: &str) -> String {
    let status = offered(build, level)
        .remove(option)
        .unwrap_or_else(|| panic!("{option} isn't offered at {level}"));
    engine().issue_text(&status.expect_err("locked")).unwrap()
}

/// "At 3rd level and every three levels thereafter, the Paladin may select a Mercy": the guide's 15 mercies, each
/// needing its level in paladin and the mercy its table names; no feature stands for the pick any more.
#[test]
fn mercies_are_a_selection_of_15_with_their_level_and_prerequisite() {
    let paladin = class("paladin");
    let selection = paladin
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == MERCY)
        .expect("the mercy selection");
    assert_eq!(selection.name, "Mercy");
    let keys: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    let expected: BTreeSet<String> = MERCIES.iter().map(|(name, ..)| mercy(name)).collect();
    assert_eq!(keys, expected);
    for (name, level, needs) in MERCIES {
        let at_level = Prerequisite::ClassLevel {
            class: ClassKey::new("paladin"),
            level,
        };
        let mut expected = match needs {
            None => at_level,
            Some(other) => Prerequisite::All(vec![
                at_level,
                Prerequisite::SelectionOption(SelectionOptionKey::new(mercy(other))),
            ]),
        };
        expected.canonicalize();
        assert_eq!(
            selection_option(&mercy(name)).prerequisite,
            Some(expected),
            "{name}"
        );
    }
    assert!(
        paladin
            .features
            .iter()
            .all(|feature| feature.key.as_str() != MERCY)
    );
    let levels: Vec<u8> = paladin
        .levels
        .iter()
        .filter(|row| row.selections.iter().any(|s| s.as_str() == MERCY))
        .map(|row| row.level)
        .collect();
    assert_eq!(levels, [3, 6, 9, 12, 15, 18]);
}

/// Each paladin archetype picks a mercy where its table has "Mercy": the Divine Hunter not at 6 (Distant Mercy) or
/// 18, the Stonelord only at 6, 12 and 18.
#[test]
fn every_paladin_archetype_picks_a_mercy_at_its_tables_levels() {
    let every = vec![3, 6, 9, 12, 15, 18];
    assert_eq!(levels_with_slot_of(&paladin(20), MERCY), every);
    for (archetype, levels) in [
        ("paladin/divine-guardian", every.clone()),
        ("paladin/divine-hunter-paladin", vec![3, 9, 12, 15]),
        ("paladin/divine-scion", every.clone()),
        ("paladin/hospitaler", every.clone()),
        ("paladin/martyr", every.clone()),
        ("paladin/stonelord", vec![6, 12, 18]),
        ("paladin/tortured-crusader", every.clone()),
        ("paladin/warrior-of-the-holy-light", every.clone()),
    ] {
        let build = paladin_of(archetype, 20);
        assert!(possible(&build), "{archetype}");
        assert_eq!(levels_with_slot_of(&build, MERCY), levels, "{archetype}");
    }
    assert_eq!(class("paladin").archetypes.len(), 8);
}

/// The ticket's: a paladin 3 picks one mercy, from those of level 3 only; the slot reads "Mercy".
#[test]
fn a_paladin_3_picks_one_mercy_from_those_of_level_3() {
    let engine = engine();
    let build = paladin(3);
    assert_eq!(levels_with_slot_of(&build, MERCY), [3]);
    let open: BTreeSet<String> = ["fatigued", "shaken", "sickened"]
        .iter()
        .map(|name| mercy(name))
        .collect();
    assert_eq!(open_at(&build, 3), open);
    assert_eq!(offered(&build, 3).len(), 15);
    let sheet = engine.evaluate(&build);
    assert_eq!(
        engine.slot_label(sheet.slot(&selection_slot(3)).unwrap()),
        "Mercy"
    );
    let mut picked = build.clone();
    picked
        .picks
        .insert(selection_slot(3), pick(&mercy("shaken")));
    assert!(possible(&picked));
    // Paladin levels 1 and 2 pick none.
    assert_eq!(levels_with_slot_of(&paladin(2), MERCY), Vec::<u8>::new());
}

/// The ticket's: a mercy of level 6 is locked at 3 with a reason and open at 6. A mercy needing another waits for it.
#[test]
fn a_level_6_mercy_is_locked_at_3_with_a_reason_and_open_at_6() {
    let build = paladin(6);
    assert_eq!(
        lock_reason(&build, 3, &mercy("dazed")),
        "Needs Paladin level 6 (have 3)"
    );
    assert!(open_at(&build, 6).contains(&mercy("dazed")));
    let mut early = build.clone();
    early.picks.insert(selection_slot(3), pick(&mercy("dazed")));
    assert_eq!(
        why_not(&early, &selection_slot(3)).and_then(|issue| engine().issue_text(&issue)),
        Some("Needs Paladin level 6 (have 3)".to_owned())
    );
    let mut at_6 = build.clone();
    at_6.picks.insert(selection_slot(6), pick(&mercy("dazed")));
    assert!(possible(&at_6));

    // Exhausted (level 9) needs Fatigued.
    let nine = paladin(9);
    assert_eq!(lock_reason(&nine, 9, &mercy("exhausted")), "Needs Fatigued");
    let mut fatigued = nine.clone();
    fatigued
        .picks
        .insert(selection_slot(3), pick(&mercy("fatigued")));
    assert!(open_at(&fatigued, 9).contains(&mercy("exhausted")));
    // A Divine Hunter's first slot after 3 is at 9.
    let hunter = paladin_of("paladin/divine-hunter-paladin", 9);
    assert!(open_at(&hunter, 9).contains(&mercy("dazed")));
}

/// The ticket's: Extra Mercy gives a paladin one more mercy slot at the level it's taken, and needs a mercy.
#[test]
fn extra_mercy_gives_a_paladin_one_more_mercy_slot() {
    let engine = engine();
    let mut build = paladin(3);
    build
        .picks
        .insert(selection_slot(3), pick(&mercy("fatigued")));
    build
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-mercy"));
    assert!(possible(&build));
    assert_eq!(
        kind_at(&build, &extra_slot(3, 0)),
        Some(selection_kind(MERCY))
    );
    let sheet = engine.evaluate(&build);
    assert_eq!(
        engine.slot_label(sheet.slot(&extra_slot(3, 0)).unwrap()),
        "Mercy"
    );
    // Its options are the paladin's mercies, the one held already taken.
    let options = engine.options(&build, &extra_slot(3, 0)).unwrap();
    assert_eq!(options.len(), 15);
    let open: BTreeSet<String> = options
        .iter()
        .filter(|option| option.status.is_ok())
        .map(|option| option.pick.key().to_owned())
        .collect();
    assert_eq!(
        open,
        ["shaken", "sickened"]
            .iter()
            .map(|name| mercy(name))
            .collect()
    );
    let mut filled = build.clone();
    filled
        .picks
        .insert(extra_slot(3, 0), pick(&mercy("shaken")));
    assert!(possible(&filled));

    // "Prerequisites: Mercy": a paladin holding no mercy can't take it (a Stonelord before 6).
    let mut stonelord = paladin_of("paladin/stonelord", 3);
    stonelord
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-mercy"));
    assert!(!possible(&stonelord));
    assert!(why_not(&stonelord, &general_slot(3)).is_some());
}

/// "An Arcane Enforcer can choose an Arcanist Exploit at level 1 and every 3 subsequent levels": a slot of the
/// arcanist's own exploits (the guide gives the Enforcer no list of its own) at 1, 4, 7, …, 19; the feature is gone.
#[test]
fn an_arcane_enforcer_picks_an_arcanist_exploit_at_1_4_7_and_every_3_levels() {
    let engine = engine();
    let build = enforcer(20);
    assert!(possible(&build));
    assert_eq!(
        levels_with_slot_of(&build, EXPLOITS),
        [1, 4, 7, 10, 13, 16, 19]
    );
    let enforcer_archetype = class("slayer")
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == "slayer/arcane-enforcer")
        .unwrap();
    assert!(
        enforcer_archetype
            .features
            .iter()
            .all(|f| f.key.as_str() != "slayer/arcane-enforcer/arcanist-exploits")
    );
    // The arcanist's 16 exploits, Holy Water Jet locked for a chaotic neutral Enforcer by its alignment.
    let arcanist_exploits: BTreeSet<String> = class("arcanist")
        .selections
        .iter()
        .find(|s| s.key.as_str() == EXPLOITS)
        .unwrap()
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    assert_eq!(arcanist_exploits.len(), 16);
    let offered_keys: BTreeSet<String> = offered(&build, 1).into_keys().collect();
    assert_eq!(offered_keys, arcanist_exploits);
    let mut open = arcanist_exploits.clone();
    open.remove("arcanist/arcanist-exploits/holy-water-jet");
    assert_eq!(open_at(&build, 1), open);
    let sheet = engine.evaluate(&build);
    assert_eq!(
        engine.slot_label(sheet.slot(&selection_slot(1)).unwrap()),
        "Arcanist Exploits"
    );
    // Filled at every level, the Build holds seven different exploits.
    let filled = with_selections_filled(build);
    assert!(possible(&filled));
    for level in [1, 4, 7, 10, 13, 16, 19] {
        assert!(filled.picks.contains_key(&selection_slot(level)), "{level}");
    }
    // An exploit taken once isn't offered again.
    let mut twice = enforcer(4);
    twice.picks.insert(
        selection_slot(1),
        pick("arcanist/arcanist-exploits/wooden-flesh"),
    );
    assert_eq!(
        lock_reason(&twice, 4, "arcanist/arcanist-exploits/wooden-flesh"),
        "Already taken"
    );
}

/// The ticket's: Extra Arcanist Exploit gives an Arcane Enforcer one more exploit slot; its prerequisite is an
/// exploit held (the Enforcer's feature is gone).
#[test]
fn extra_arcanist_exploit_gives_an_arcane_enforcer_one_more_exploit_slot() {
    let engine = engine();
    let mut build = enforcer(1);
    build.picks.insert(
        selection_slot(1),
        pick("arcanist/arcanist-exploits/armored-mask"),
    );
    build
        .picks
        .insert(general_slot(1), feat_pick_plain("extra-arcanist-exploit"));
    assert!(possible(&build));
    assert_eq!(
        kind_at(&build, &extra_slot(1, 0)),
        Some(selection_kind(EXPLOITS))
    );
    let sheet = engine.evaluate(&build);
    assert_eq!(
        engine.slot_label(sheet.slot(&extra_slot(1, 0)).unwrap()),
        "Arcanist Exploits"
    );
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(1, 0)));
    assert!(possible(&filled));

    let expected = Prerequisite::Any(
        class("arcanist")
            .selections
            .iter()
            .find(|s| s.key.as_str() == EXPLOITS)
            .unwrap()
            .options
            .iter()
            .map(|o| Prerequisite::SelectionOption(o.key.clone()))
            .collect(),
    );
    let mut actual = feat("extra-arcanist-exploit").prerequisite.clone().unwrap();
    actual.canonicalize();
    let mut expected = expected;
    expected.canonicalize();
    assert_eq!(actual, expected);
}

// ---- Mythic Extra Feat and Extra Mythic Ability give their pick (ticket 55) ----------------------------------------

/// The slot a rank's general mythic feat gives.
fn mythic_extra(rank: u8) -> SlotId {
    SlotId::MythicExtra { rank }
}

/// A Build of `levels` levels of `class` with `mythic_feat` as MR 2's general pick (MR 2 is at level 7).
fn with_mythic_feat(class: &str, levels: usize, mythic_feat: &str) -> Build {
    let mut build = build(class, levels);
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new(mythic_feat)),
    );
    build
}

/// Every option a slot's picker offers, with its status.
fn slot_options(build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
}

/// "You can select any normal feat." / "You can select a mythic ability." (mythic-paths#extra-feat,
/// mythic-paths#extra-mythic-ability): each gives one pick, and only these two; taken once, as the guide says no
/// more.
#[test]
fn extra_feat_and_extra_mythic_ability_are_the_mythic_feats_that_give_a_pick() {
    let giving: Vec<(String, rules::MythicExtraPick, Repeat)> = DATA
        .mythic_feats
        .iter()
        .filter_map(|feat| Some((feat.key.to_string(), feat.extra_pick?, feat.repeat)))
        .collect();
    assert_eq!(
        giving,
        [
            (
                "extra-feat".to_owned(),
                rules::MythicExtraPick::Feat,
                Repeat::Once
            ),
            (
                "extra-mythic-ability".to_owned(),
                rules::MythicExtraPick::MythicAbility,
                Repeat::Once
            ),
        ]
    );
}

#[test]
fn extra_feat_at_mr_2_gives_a_feat_slot_at_level_7_offering_what_the_general_slot_there_does() {
    let engine = engine();
    let build = with_mythic_feat("fighter", 7, "extra-feat");
    let sheet = engine.evaluate(&build);
    let rank_2: Vec<(SlotId, SlotKind)> = sheet.ranks[1]
        .slots
        .iter()
        .map(|slot| (slot.id.clone(), slot.kind.clone()))
        .collect();
    assert_eq!(
        rank_2,
        [
            (SlotId::Mythic { rank: 2 }, SlotKind::MythicFeat),
            (mythic_extra(2), SlotKind::Feat(FeatSlot::General)),
        ]
    );
    assert_eq!(sheet.ranks[1].character_level, 7);
    assert_eq!(engine.slot_label(&sheet.ranks[1].slots[1]), "Feat");
    // Every feat, each open or locked as in level 7's general slot.
    let extra = slot_options(&build, &mythic_extra(2));
    assert_eq!(extra.len(), slot_options(&build, &general_slot(7)).len());
    assert_eq!(extra, slot_options(&build, &general_slot(7)));
    assert!(extra.iter().any(|option| option.status.is_ok()));

    // A pick there is held from level 7; its prerequisites are checked.
    let mut with_pick = build.clone();
    with_pick
        .picks
        .insert(mythic_extra(2), feat_pick_plain("toughness"));
    assert!(possible(&with_pick));
    assert!(
        engine
            .evaluate(&with_pick)
            .at(7)
            .unwrap()
            .feats
            .contains_key(&FeatKey::new("toughness"))
    );
    let mut unmet = build;
    unmet
        .picks
        .insert(mythic_extra(2), feat_pick_plain("great-cleave"));
    assert!(matches!(
        why_not(&unmet, &mythic_extra(2)),
        Some(Issue::Unmet(_))
    ));
}

#[test]
fn extra_mythic_ability_gives_a_mythic_ability_slot_offering_what_a_ranks_does() {
    let engine = engine();
    let build = with_mythic_feat("fighter", 7, "extra-mythic-ability");
    let sheet = engine.evaluate(&build);
    let slot = sheet.slot(&mythic_extra(2)).expect("the extra slot");
    assert_eq!(slot.kind, SlotKind::MythicAbility);
    assert_eq!(engine.slot_label(slot), "Mythic ability");
    let picks = |slot: &SlotId| -> BTreeSet<Pick> {
        slot_options(&build, slot)
            .into_iter()
            .map(|option| option.pick)
            .collect()
    };
    assert_eq!(picks(&mythic_extra(2)), picks(&SlotId::Mythic { rank: 1 }));
    assert_eq!(picks(&mythic_extra(2)).len(), DATA.mythic_abilities.len());

    // A sorcerer's Second Bloodline there gives another bloodline at MR 2.
    let mut sorcerer = with_mythic_feat("sorcerer", 7, "extra-mythic-ability");
    sorcerer.picks.insert(
        mythic_extra(2),
        Pick::MythicAbility(rules::MythicAbilityKey::new("second-bloodline")),
    );
    let second = SlotId::MythicSelection { rank: 2, index: 0 };
    assert_eq!(
        kind_at(&sorcerer, &second),
        Some(selection_kind("sorcerer/bloodline"))
    );
    let open = slot_options(&sorcerer, &second)
        .into_iter()
        .find(|option| option.status.is_ok())
        .expect("an open bloodline");
    sorcerer.picks.insert(second, open.pick);
    assert!(possible(&with_selections_filled(sorcerer)));
}

#[test]
fn removing_extra_feat_or_extra_mythic_ability_is_refused_while_its_slot_holds_a_pick() {
    let engine = engine();
    for (mythic_feat, pick) in [
        ("extra-feat", feat_pick_plain("toughness")),
        (
            "extra-mythic-ability",
            Pick::MythicAbility(rules::MythicAbilityKey::new("unstoppable")),
        ),
    ] {
        let mut build = with_mythic_feat("fighter", 7, mythic_feat);
        build.picks.insert(mythic_extra(2), pick);
        assert!(possible(&build), "{mythic_feat}");
        let feat = rules::Choice::Slot(SlotId::Mythic { rank: 2 });
        assert!(!engine.is_removable(&build, &feat), "{mythic_feat}");
        assert!(!engine.removable(&build).contains(&feat));
        assert!(engine.is_removable(&build, &rules::Choice::Slot(mythic_extra(2))));

        build.apply(&rules::Edit::Clear(mythic_extra(2)));
        assert!(engine.is_removable(&build, &feat), "{mythic_feat}");
        build.apply(&rules::Edit::Clear(SlotId::Mythic { rank: 2 }));
        assert!(possible(&build));
        assert_eq!(kind_at(&build, &mythic_extra(2)), None);
    }
}

#[test]
fn extra_rage_power_through_extra_feat_gives_a_rage_power_slot_at_the_ranks_level() {
    let mut build = with_selections_filled(barbarian(7));
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new("extra-feat")),
    );
    build
        .picks
        .insert(mythic_extra(2), feat_pick_plain("extra-rage-power"));
    assert_eq!(
        kind_at(&build, &extra_slot(7, 0)),
        Some(selection_kind("barbarian/rage-power"))
    );
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(7, 0)));
    assert!(possible(&filled));
}

// ---- Greater exploits, and the Exploiter Wizard's exploits (ticket 57) ---------------------------------------------

const GREATER: &str = "arcanist/greater-exploits";

/// A chaotic neutral Exploiter Wizard of `levels` levels, with no picks.
fn exploiter(levels: usize) -> Build {
    chaotic(archetype_build("wizard", "wizard/exploiter-wizard", levels))
}

/// The one slot of `selection` the Build's sheet has at `level`.
fn slot_of(build: &Build, level: u8, selection: &str) -> SlotId {
    let found: Vec<SlotId> = engine()
        .evaluate(build)
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .filter(|slot| {
            slot.kind == selection_kind(selection)
                && matches!(slot.id, SlotId::Selection { level: at, .. } if at == level)
        })
        .map(|slot| slot.id.clone())
        .collect();
    let [slot] = found.as_slice() else {
        panic!("{} slots of {selection} at {level}", found.len())
    };
    slot.clone()
}

/// The options `slot`'s picker offers, by key, with their status.
fn offered_in(build: &Build, slot: &SlotId) -> HashMap<String, Result<(), Issue>> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .map(|SlotOption { pick, status }| (pick.key().to_owned(), status))
        .collect()
}

/// Why `slot`'s picker locks `option`, as the sheet says it.
fn reason_in(build: &Build, slot: &SlotId, option: &str) -> String {
    let status = offered_in(build, slot)
        .remove(option)
        .unwrap_or_else(|| panic!("{option} isn't offered in {slot:?}"));
    engine().issue_text(&status.expect_err("locked")).unwrap()
}

/// The keys of the arcanist's own selection `selection`.
fn arcanist_options(selection: &str) -> BTreeSet<String> {
    class("arcanist")
        .selections
        .iter()
        .find(|s| s.key.as_str() == selection)
        .unwrap_or_else(|| panic!("no {selection}"))
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect()
}

/// The features of `class`'s archetype `archetype`, by key.
fn archetype_features(class_key: &str, archetype: &str) -> BTreeSet<String> {
    class(class_key)
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == archetype)
        .unwrap_or_else(|| panic!("no {archetype}"))
        .features
        .iter()
        .map(|f| f.key.to_string())
        .collect()
}

/// The ticket's: "From 13th level, he gains access to Greater Exploits" (`slayer#arcanist-exploits`), read as the
/// arcanist's own "Greater Exploits" cell at 11 is: one slot of the arcanist's greater exploits at 13, in place of the
/// feature. An Enforcer 12 has none.
#[test]
fn an_arcane_enforcer_13_has_one_greater_exploit_slot_and_an_enforcer_12_none() {
    let engine = engine();
    assert_eq!(levels_with_slot_of(&enforcer(20), GREATER), [13]);
    assert_eq!(levels_with_slot_of(&enforcer(13), GREATER), [13]);
    assert_eq!(
        levels_with_slot_of(&enforcer(12), GREATER),
        Vec::<u8>::new()
    );
    assert!(
        !archetype_features("slayer", "slayer/arcane-enforcer")
            .contains("slayer/arcane-enforcer/greater-exploits")
    );
    // Its 13th level also keeps its arcanist exploit.
    assert_eq!(
        levels_with_slot_of(&enforcer(13), EXPLOITS),
        [1, 4, 7, 10, 13]
    );

    // Flame Arc at 1 opens the two greater exploits that extend it; the others wait for theirs.
    let mut build = enforcer(13);
    build.picks.insert(
        selection_slot(1),
        pick("arcanist/arcanist-exploits/flame-arc"),
    );
    let greater = slot_of(&build, 13, GREATER);
    let sheet = engine.evaluate(&build);
    assert_eq!(
        engine.slot_label(sheet.slot(&greater).unwrap()),
        "Greater Exploits"
    );
    let offered = offered_in(&build, &greater);
    assert_eq!(
        offered.keys().cloned().collect::<BTreeSet<_>>(),
        arcanist_options(GREATER)
    );
    assert_eq!(offered.len(), 6);
    let open: BTreeSet<&str> = offered
        .iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key.as_str())
        .collect();
    assert_eq!(
        open,
        BTreeSet::from([
            "arcanist/greater-exploits/burning-flame",
            "arcanist/greater-exploits/hellfire-ray",
        ])
    );
    assert_eq!(
        reason_in(&build, &greater, "arcanist/greater-exploits/icy-tomb"),
        "Needs Ice Missile"
    );
    let mut picked = build.clone();
    picked.picks.insert(
        greater.clone(),
        pick("arcanist/greater-exploits/burning-flame"),
    );
    assert!(possible(&picked));
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&greater));
    assert!(possible(&filled));
}

/// The ticket's: "At 1st level and every four levels thereafter, an Exploiter Wizard may choose an Arcanist
/// Exploit. From 13th level, he can choose from the list of Greater Exploits." (`wizard#exploiter-exploits`): slots
/// of the arcanist's own exploits at 1, 5, 9, 13 and 17, and of its greater exploits where the table has "Greater
/// Exploits", at 13. Both features are gone.
#[test]
fn an_exploiter_wizard_picks_an_arcanist_exploit_at_1_5_9_13_and_17_and_a_greater_exploit_at_13() {
    let engine = engine();
    let wizard = exploiter(20);
    assert!(possible(&wizard));
    assert_eq!(levels_with_slot_of(&wizard, EXPLOITS), [1, 5, 9, 13, 17]);
    assert_eq!(levels_with_slot_of(&wizard, GREATER), [13]);
    assert_eq!(
        levels_with_slot_of(&exploiter(12), GREATER),
        Vec::<u8>::new()
    );
    let features = archetype_features("wizard", "wizard/exploiter-wizard");
    assert!(!features.contains("wizard/exploiter-wizard/exploiter-exploits"));
    assert!(!features.contains("wizard/exploiter-wizard/greater-exploits"));
    // A wizard of no archetype picks none.
    assert_eq!(
        levels_with_slot_of(&build("wizard", 20), EXPLOITS),
        Vec::<u8>::new()
    );

    // The arcanist's 16 exploits, Holy Water Jet locked for a chaotic neutral wizard by its alignment.
    let first = slot_of(&wizard, 1, EXPLOITS);
    let offered = offered_in(&wizard, &first);
    let exploits = arcanist_options(EXPLOITS);
    assert_eq!(exploits.len(), 16);
    assert_eq!(offered.keys().cloned().collect::<BTreeSet<_>>(), exploits);
    let open: BTreeSet<String> = offered
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .collect();
    let mut expected = exploits;
    expected.remove("arcanist/arcanist-exploits/holy-water-jet");
    assert_eq!(open, expected);
    let sheet = engine.evaluate(&wizard);
    assert_eq!(
        engine.slot_label(sheet.slot(&first).unwrap()),
        "Arcanist Exploits"
    );
    let greater = slot_of(&wizard, 13, GREATER);
    assert_eq!(
        engine.slot_label(sheet.slot(&greater).unwrap()),
        "Greater Exploits"
    );
    assert_eq!(
        offered_in(&wizard, &greater)
            .into_keys()
            .collect::<BTreeSet<_>>(),
        arcanist_options(GREATER)
    );

    // Filled, the Build holds five exploits and a greater exploit.
    let filled = with_selections_filled(wizard.clone());
    assert!(possible(&filled));
    for level in [1, 5, 9, 13, 17] {
        assert!(
            filled
                .picks
                .contains_key(&slot_of(&wizard, level, EXPLOITS)),
            "{level}"
        );
    }
    assert!(filled.picks.contains_key(&greater));
}

/// The ticket's: Extra Arcanist Exploit gives an Exploiter Wizard one more exploit slot (ticket 49's rule: the
/// selection it had a slot of first). Holding no exploit, it can't take the feat.
#[test]
fn extra_arcanist_exploit_gives_an_exploiter_wizard_one_more_exploit_slot() {
    let engine = engine();
    let mut wizard = exploiter(1);
    let first = slot_of(&wizard, 1, EXPLOITS);
    wizard
        .picks
        .insert(first, pick("arcanist/arcanist-exploits/potent-magic"));
    wizard
        .picks
        .insert(general_slot(1), feat_pick_plain("extra-arcanist-exploit"));
    assert!(possible(&wizard));
    assert_eq!(
        kind_at(&wizard, &extra_slot(1, 0)),
        Some(selection_kind(EXPLOITS))
    );
    let sheet = engine.evaluate(&wizard);
    assert_eq!(
        engine.slot_label(sheet.slot(&extra_slot(1, 0)).unwrap()),
        "Arcanist Exploits"
    );
    assert_eq!(offered_in(&wizard, &extra_slot(1, 0)).len(), 16);
    assert_eq!(
        reason_in(
            &wizard,
            &extra_slot(1, 0),
            "arcanist/arcanist-exploits/potent-magic"
        ),
        "Already taken"
    );
    let filled = with_selections_filled(wizard);
    assert!(filled.picks.contains_key(&extra_slot(1, 0)));
    assert!(possible(&filled));

    let mut none = exploiter(1);
    none.picks
        .insert(general_slot(1), feat_pick_plain("extra-arcanist-exploit"));
    assert!(!possible(&none));
    assert!(why_not(&none, &general_slot(1)).is_some());
}

/// The ticket's: an exploit already taken is not offered again, in the Exploiter Wizard's later slots, nor in
/// another class's slot of the same selection (an Enforcer who becomes an Exploiter Wizard).
#[test]
fn an_exploit_already_taken_is_not_offered_again() {
    let mut wizard = exploiter(17);
    wizard.picks.insert(
        slot_of(&wizard, 1, EXPLOITS),
        pick("arcanist/arcanist-exploits/wooden-flesh"),
    );
    assert!(possible(&wizard));
    for level in [5, 9, 13, 17] {
        assert_eq!(
            reason_in(
                &wizard,
                &slot_of(&wizard, level, EXPLOITS),
                "arcanist/arcanist-exploits/wooden-flesh"
            ),
            "Already taken",
            "{level}"
        );
    }

    let mut both = classes_build(&["slayer", "wizard"]);
    for (class, archetype) in [
        ("slayer", "slayer/arcane-enforcer"),
        ("wizard", "wizard/exploiter-wizard"),
    ] {
        build_archetype(&mut both, class, archetype);
    }
    assert_eq!(levels_with_slot_of(&both, EXPLOITS), [1, 2]);
    both.picks.insert(
        selection_slot(1),
        pick("arcanist/arcanist-exploits/wooden-flesh"),
    );
    assert_eq!(
        reason_in(
            &both,
            &slot_of(&both, 2, EXPLOITS),
            "arcanist/arcanist-exploits/wooden-flesh"
        ),
        "Already taken"
    );
    let mut twice = both.clone();
    twice.picks.insert(
        slot_of(&both, 2, EXPLOITS),
        pick("arcanist/arcanist-exploits/wooden-flesh"),
    );
    assert!(!possible(&twice));
}

/// Takes `class` with `archetype` in the Build.
fn build_archetype(build: &mut Build, class: &str, archetype: &str) {
    build
        .class_choices
        .get_mut(&ClassKey::new(class))
        .unwrap_or_else(|| panic!("no {class}"))
        .archetype = Some(ArchetypeKey::new(archetype));
}

// ---- Monk ki powers (ticket 58) -----------------------------------------------------------------------------------

const KI_POWER: &str = "monk/ki-power";

/// The guide's ki powers (`monk#ki-powers`): each one's key and the monk level it unlocks at.
const KI_POWERS: [(&str, u8); 14] = [
    ("barkskin", 4),
    ("scorching-ray", 4),
    ("sudden-speed", 4),
    ("true-strike", 4),
    ("abundant-step", 8),
    ("poison", 8),
    ("restoration", 8),
    ("wholeness-of-body", 8),
    ("spit-venom", 10),
    ("diamond-body", 12),
    ("cold-ice-strike", 14),
    ("diamond-soul", 14),
    ("shout", 14),
    ("quivering-palm", 16),
];

fn ki_power(name: &str) -> String {
    format!("{KI_POWER}/{name}")
}

/// A lawful neutral monk of `levels` levels, with no picks.
fn monk(levels: usize) -> Build {
    let mut build = build("monk", levels);
    build.start.alignment = Some(rules::Alignment::LawfulNeutral);
    build
}

/// A monk of `archetype`, `levels` levels, with no picks: lawful neutral, but chaotic neutral for the Drunken Master
/// ("Cannot be Lawful"), and an oread for the Student of Stone ("Race must be Oread").
fn monk_of(archetype: &str, levels: usize) -> Build {
    let mut build = archetype_build("monk", archetype, levels);
    build.start.alignment = Some(match archetype {
        "monk/drunken-master" => rules::Alignment::ChaoticNeutral,
        _ => rules::Alignment::LawfulNeutral,
    });
    if archetype == "monk/student-of-stone" {
        build.start.race = Some(rules::RaceKey::new("oread"));
    }
    build
}

/// "At 4th level and every 2 levels thereafter a Monk may select a ki power": the guide's 14 ki powers, each needing
/// the monk level its table gives; no feature stands for the pick any more, and no feat gives one.
#[test]
fn ki_powers_are_a_selection_of_14_with_their_level() {
    let monk = class("monk");
    let selection = monk
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == KI_POWER)
        .expect("the ki power selection");
    assert_eq!(selection.name, "Ki Power");
    let keys: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    let expected: BTreeSet<String> = KI_POWERS.iter().map(|(name, _)| ki_power(name)).collect();
    assert_eq!(keys, expected);
    for (name, level) in KI_POWERS {
        assert_eq!(
            selection_option(&ki_power(name)).prerequisite,
            Some(Prerequisite::ClassLevel {
                class: ClassKey::new("monk"),
                level,
            }),
            "{name}"
        );
    }
    assert!(
        monk.features
            .iter()
            .all(|feature| feature.key.as_str() != KI_POWER)
    );
    let feature = FeatureKey::new(KI_POWER);
    assert!(
        monk.levels
            .iter()
            .all(|row| !row.features.contains(&feature))
            && monk
                .archetypes
                .iter()
                .flat_map(|a| &a.levels)
                .all(|row| !row.features.contains(&feature))
    );
    let levels: Vec<u8> = monk
        .levels
        .iter()
        .filter(|row| row.selections.iter().any(|s| s.as_str() == KI_POWER))
        .map(|row| row.level)
        .collect();
    assert_eq!(levels, [4, 6, 8, 10, 12, 14, 16, 18, 20]);
    // Extra Ki and Abundant Ki Pool grow the pool; no feat gives a ki power.
    assert!(
        DATA.feats
            .iter()
            .all(|feat| feat.extra_pick_of.iter().all(|s| s.as_str() != KI_POWER))
    );
}

/// The ticket's: each monk archetype picks a ki power where its table has "Ki Power": the Scaled Fist not at 12
/// (Dragon Breath), the Sohei from 6, the Student of Stone not at 6 or 12, the Zen Archer not at 10, and the
/// Traditional Monk never (its ki powers are fixed). Filled, each Build is possible, the Drunken Master's too since
/// its "Cannot be Lawful" replaces the monk's "Lawful" (ticket 63).
#[test]
fn every_monk_archetype_picks_a_ki_power_at_its_tables_levels() {
    let every = vec![4, 6, 8, 10, 12, 14, 16, 18, 20];
    assert_eq!(levels_with_slot_of(&monk(20), KI_POWER), every);
    for (archetype, levels) in [
        ("monk/drunken-master", every.clone()),
        ("monk/quarterstaff-master", every.clone()),
        ("monk/scaled-fist", vec![4, 6, 8, 10, 14, 16, 18, 20]),
        ("monk/sensei", every.clone()),
        ("monk/sohei", vec![6, 8, 10, 12, 14, 16, 18, 20]),
        ("monk/student-of-stone", vec![4, 8, 10, 14, 16, 18, 20]),
        ("monk/traditional-monk", vec![]),
        ("monk/zen-archer", vec![4, 6, 8, 12, 14, 16, 18, 20]),
    ] {
        let build = monk_of(archetype, 20);
        assert_eq!(levels_with_slot_of(&build, KI_POWER), levels, "{archetype}");
        assert!(possible(&build), "{archetype}");
        let filled = with_selections_filled(build);
        assert!(possible(&filled), "{archetype} filled");
    }
    assert_eq!(class("monk").archetypes.len(), 8);
}

/// The ticket's: a monk 4 has one ki power slot, "Ki Power", offering the 14 and leaving open only the four of level
/// 4. A monk 3 has none.
#[test]
fn a_monk_4_has_one_ki_power_slot_offering_only_the_powers_of_level_4() {
    let engine = engine();
    let build = monk(4);
    assert_eq!(levels_with_slot_of(&build, KI_POWER), [4]);
    let open: BTreeSet<String> = KI_POWERS
        .iter()
        .filter(|(_, level)| *level <= 4)
        .map(|(name, _)| ki_power(name))
        .collect();
    assert_eq!(open.len(), 4);
    assert_eq!(open_at(&build, 4), open);
    assert_eq!(offered(&build, 4).len(), 14);
    let sheet = engine.evaluate(&build);
    assert_eq!(
        engine.slot_label(sheet.slot(&selection_slot(4)).unwrap()),
        "Ki Power"
    );
    let mut picked = build.clone();
    picked
        .picks
        .insert(selection_slot(4), pick(&ki_power("sudden-speed")));
    assert!(possible(&picked));
    assert_eq!(levels_with_slot_of(&monk(3), KI_POWER), Vec::<u8>::new());
}

/// The ticket's: a ki power of a higher level is locked with a reason and opens at its level; one taken isn't offered
/// again.
#[test]
fn a_higher_level_ki_power_is_locked_with_a_reason_and_opens_at_its_level() {
    let build = monk(16);
    assert_eq!(
        lock_reason(&build, 4, &ki_power("abundant-step")),
        "Needs Monk level 8 (have 4)"
    );
    assert_eq!(
        lock_reason(&build, 6, &ki_power("abundant-step")),
        "Needs Monk level 8 (have 6)"
    );
    assert!(open_at(&build, 8).contains(&ki_power("abundant-step")));
    assert_eq!(
        lock_reason(&build, 14, &ki_power("quivering-palm")),
        "Needs Monk level 16 (have 14)"
    );
    assert!(open_at(&build, 16).contains(&ki_power("quivering-palm")));
    // Each opens at its own level.
    for (name, level) in KI_POWERS {
        for at in (4..=16).step_by(2) {
            assert_eq!(
                open_at(&build, at).contains(&ki_power(name)),
                level <= at,
                "{name} at {at}"
            );
        }
    }

    let mut early = monk(4);
    early
        .picks
        .insert(selection_slot(4), pick(&ki_power("diamond-body")));
    assert_eq!(
        why_not(&early, &selection_slot(4)).and_then(|issue| engine().issue_text(&issue)),
        Some("Needs Monk level 12 (have 4)".to_owned())
    );

    let mut taken = monk(6);
    taken
        .picks
        .insert(selection_slot(4), pick(&ki_power("barkskin")));
    assert_eq!(
        lock_reason(&taken, 6, &ki_power("barkskin")),
        "Already taken"
    );
    let mut twice = taken.clone();
    twice
        .picks
        .insert(selection_slot(6), pick(&ki_power("barkskin")));
    assert!(!possible(&twice));
}

/// "Gains fixed Ki Powers": a Traditional Monk picks no ki power, and holds the five its table names from their
/// levels (Wholeness of Body at 6, Abundant Step at 8, Diamond Body at 10, Diamond Soul at 12, Quivering Palm at
/// 14), each a feature of its own.
#[test]
fn a_traditional_monk_gains_its_fixed_ki_powers_and_picks_none() {
    let build = monk_of("monk/traditional-monk", 20);
    assert!(possible(&build));
    assert_eq!(levels_with_slot_of(&build, KI_POWER), Vec::<u8>::new());
    let sheet = engine().evaluate(&build);
    for (feature, from) in [
        ("monk/traditional-monk/ki-power-wholeness-of-body", 6),
        ("monk/traditional-monk/ki-power-abundant-step", 8),
        ("monk/traditional-monk/ki-power-diamond-body", 10),
        ("monk/traditional-monk/ki-power-diamond-soul", 12),
        ("monk/traditional-monk/ki-power-quivering-palm", 14),
    ] {
        assert!(!holds(sheet.at(from - 1).unwrap(), feature), "{feature}");
        assert!(holds(sheet.at(from).unwrap(), feature), "{feature}");
        assert!(holds(sheet.at(20).unwrap(), feature), "{feature}");
    }
    // The rest of those rows is the base class's: the bonus feats at 6, 10 and 14, Stunning Fist: Sicken at 8 and
    // Ki Strike - Lawful at 10.
    assert!(holds(sheet.at(8).unwrap(), "monk/stunning-fist-sicken"));
    assert!(holds(sheet.at(10).unwrap(), "monk/ki-strike-lawful"));
    let traditional = class("monk")
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == "monk/traditional-monk")
        .unwrap();
    for level in [6, 10, 14] {
        let row = rules::effective_row(class("monk"), Some(traditional), level).unwrap();
        assert_eq!(row.feat_slots.len(), 1, "{level}");
    }
}

// ---- the kineticist's infusions (ticket 59) --------------------------------------------------------------------

const INFUSIONS_SELECTION: &str = "kineticist/infusions";
const SHARPSHOOTER: &str = "kineticist/kinetic-sharpshooter";

fn infusion(name: &str) -> String {
    format!("{INFUSIONS_SELECTION}/{name}")
}

/// Who may take an infusion, by the Kinetic Sharpshooter's section: anyone, only the Sharpshooter ("exclusive to the
/// Sharpshooter"), or anyone but the Sharpshooter ("infusions that the Sharpshooter cannot use").
#[derive(Clone, Copy)]
enum Sharpshooter {
    Any,
    Only,
    Not,
}

/// Each infusion of the guide's list (`kineticist#infusions`), with its level, its elements (none: Universal), the
/// infusions its "Prerequisites" names (one of them) and who may take it.
#[allow(clippy::type_complexity)]
const INFUSIONS: [(&str, u8, &[&str], &[&str], Sharpshooter); 32] = [
    ("burning-infusion", 1, &["fire"], &[], Sharpshooter::Any),
    ("dazzling-infusion", 1, &["fire"], &[], Sharpshooter::Any),
    ("exploding-arrows", 1, &[], &[], Sharpshooter::Only),
    ("extended-range", 1, &[], &[], Sharpshooter::Not),
    ("fan-of-flames", 1, &["fire"], &[], Sharpshooter::Any),
    ("kinetic-blade", 1, &[], &[], Sharpshooter::Not),
    (
        "pushing-infusion",
        1,
        &["air", "earth", "water"],
        &[],
        Sharpshooter::Any,
    ),
    ("bowling-infusion", 2, &["earth"], &[], Sharpshooter::Any),
    (
        "entangling-infusion",
        2,
        &["earth", "water"],
        &[],
        Sharpshooter::Any,
    ),
    (
        "magnetic-infusion",
        2,
        &["air", "earth"],
        &[],
        Sharpshooter::Any,
    ),
    ("spindle", 2, &[], &[], Sharpshooter::Any),
    (
        "eruption",
        3,
        &["fire"],
        &["extended-range"],
        Sharpshooter::Not,
    ),
    (
        "foxfire-infusion",
        3,
        &["fire"],
        &["kineticist/wild-talents/foxfire"],
        Sharpshooter::Any,
    ),
    ("rare-metal-infusion", 3, &["earth"], &[], Sharpshooter::Any),
    ("synaptic-infusion", 3, &["air"], &[], Sharpshooter::Any),
    (
        "torrent",
        3,
        &["air", "fire", "water"],
        &[],
        Sharpshooter::Any,
    ),
    ("cyclone", 4, &["air"], &[], Sharpshooter::Any),
    ("detonation", 4, &["fire"], &[], Sharpshooter::Any),
    ("flash-infusion", 4, &["fire"], &[], Sharpshooter::Any),
    ("spray", 4, &["water"], &[], Sharpshooter::Any),
    (
        "blade-whirlwind",
        5,
        &[],
        &["kinetic-blade"],
        Sharpshooter::Not,
    ),
    ("chilling-infusion", 5, &["water"], &[], Sharpshooter::Any),
    (
        "grappling-infusion",
        5,
        &[],
        &["cloud", "deadly-earth", "wall"],
        Sharpshooter::Any,
    ),
    ("rending-arrows", 5, &[], &[], Sharpshooter::Only),
    ("unraveling-infusion", 5, &["fire"], &[], Sharpshooter::Any),
    ("wall", 5, &[], &[], Sharpshooter::Any),
    (
        "deadly-earth",
        6,
        &["earth"],
        &["extended-range"],
        Sharpshooter::Not,
    ),
    ("honed-infusion", 6, &[], &[], Sharpshooter::Only),
    (
        "cloud",
        7,
        &["air", "water"],
        &["extended-range"],
        Sharpshooter::Any,
    ),
    (
        "fragmentation",
        7,
        &["earth", "water"],
        &["extended-range"],
        Sharpshooter::Not,
    ),
    ("pure-flame-infusion", 7, &["fire"], &[], Sharpshooter::Any),
    ("chain-arrows", 8, &[], &[], Sharpshooter::Only),
];

/// What an infusion of `level` needs: the kineticist level twice its own, one of its elements, one of the options its
/// "Prerequisites" names (an infusion, or the wild talent the key names), and the Sharpshooter's archetype or its
/// absence.
fn infusion_prerequisite(
    level: u8,
    elements: &[&str],
    needs: &[&str],
    sharpshooter: Sharpshooter,
) -> Prerequisite {
    let one_of = |keys: Vec<Prerequisite>| match keys.len() {
        1 => keys.into_iter().next().unwrap(),
        _ => Prerequisite::Any(keys),
    };
    let mut all = vec![Prerequisite::ClassLevel {
        class: ClassKey::new("kineticist"),
        level: 2 * level,
    }];
    if !elements.is_empty() {
        all.push(one_of(
            elements
                .iter()
                .map(|e| {
                    Prerequisite::SelectionOption(SelectionOptionKey::new(format!(
                        "kineticist/element/{e}"
                    )))
                })
                .collect(),
        ));
    }
    if !needs.is_empty() {
        all.push(one_of(
            needs
                .iter()
                .map(|key| {
                    let key = match key.contains('/') {
                        true => (*key).to_owned(),
                        false => infusion(key),
                    };
                    Prerequisite::SelectionOption(SelectionOptionKey::new(key))
                })
                .collect(),
        ));
    }
    let archetype = || Prerequisite::Archetype(ArchetypeKey::new(SHARPSHOOTER));
    match sharpshooter {
        Sharpshooter::Any => {}
        Sharpshooter::Only => all.push(archetype()),
        Sharpshooter::Not => all.push(Prerequisite::Not(Box::new(archetype()))),
    }
    let mut prerequisite = match all.len() {
        1 => all.pop().unwrap(),
        _ => Prerequisite::All(all),
    };
    prerequisite.canonicalize();
    prerequisite
}

fn infusion_reason(build: &Build, level: u8, name: &str) -> String {
    lock_reason(build, level, &infusion(name))
}

/// The infusions a slot leaves open, by name.
fn open_infusions(build: &Build, level: u8) -> BTreeSet<String> {
    open_at(build, level)
        .into_iter()
        .map(|key| {
            key.strip_prefix(&format!("{INFUSIONS_SELECTION}/"))
                .unwrap_or_else(|| panic!("{key}: not an infusion"))
                .to_owned()
        })
        .collect()
}

/// "The Kineticist may select any Infusion for which she qualifies. She gains new Infusions at 3rd, 5th, 9th, 11th,
/// 13th, 17th and 19th levels." The guide's 32 infusions, each with its level, elements and prerequisites; the
/// "Infusion" feature stays only on the 1st-level rows whose table has it, and no feat gives an infusion.
#[test]
fn infusions_are_a_selection_of_32_with_their_level_and_elements() {
    let kineticist = class("kineticist");
    let selection = kineticist
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == INFUSIONS_SELECTION)
        .expect("the infusion selection");
    assert_eq!(selection.name, "Infusion");
    let keys: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    let expected: BTreeSet<String> = INFUSIONS.iter().map(|(name, ..)| infusion(name)).collect();
    assert_eq!(keys, expected);
    for (name, level, elements, needs, sharpshooter) in INFUSIONS {
        assert_eq!(
            selection_option(&infusion(name)).prerequisite,
            Some(infusion_prerequisite(level, elements, needs, sharpshooter)),
            "{name}"
        );
    }
    // The guide's "Dazziing Infusion" is named as the game names it; its link is the guide's anchor.
    let dazzling = selection_option(&infusion("dazzling-infusion"));
    assert_eq!(dazzling.name, "Dazzling Infusion");
    assert_eq!(dazzling.guide.0, "kineticist#dazziing-infusion");

    let levels: Vec<u8> = kineticist
        .levels
        .iter()
        .filter(|row| {
            row.selections
                .iter()
                .any(|s| s.as_str() == INFUSIONS_SELECTION)
        })
        .map(|row| row.level)
        .collect();
    assert_eq!(levels, [3, 5, 9, 11, 13, 17, 19]);
    let feature = FeatureKey::new("kineticist/infusion");
    let rows_with_feature: Vec<u8> = kineticist
        .levels
        .iter()
        .filter(|row| row.features.contains(&feature))
        .map(|row| row.level)
        .collect();
    assert_eq!(rows_with_feature, [1]);
    assert!(
        kineticist
            .archetypes
            .iter()
            .flat_map(|a| &a.levels)
            .all(|row| row.level == 1 || !row.features.contains(&feature))
    );
    // The Sharpshooter's "Infusion (Kinetic Sharpshooter)" was its pick; it's gone.
    assert!(kineticist.archetypes.iter().all(|a| {
        a.features.iter().all(|f| {
            f.key.as_str() != "kineticist/kinetic-sharpshooter/infusion-kinetic-sharpshooter"
        })
    }));
    assert!(DATA.feats.iter().all(|feat| {
        feat.extra_pick_of
            .iter()
            .all(|s| s.as_str() != INFUSIONS_SELECTION)
    }));
}

/// The ticket's: a kineticist 3 has one infusion slot, "Infusion", offering every infusion and leaving open only its
/// element's and the Universal ones of level 1 (but the Sharpshooter's own). A kineticist 2 has none.
#[test]
fn a_kineticist_3_has_one_infusion_slot_offering_its_elements_and_universal_infusions_of_level_1() {
    let engine = engine();
    let build = kineticist("fire", 3);
    assert!(possible(&build));
    assert_eq!(levels_with_slot_of(&build, INFUSIONS_SELECTION), [3]);
    assert_eq!(offered(&build, 3).len(), 32);
    let fire: BTreeSet<String> = [
        "burning-infusion",
        "dazzling-infusion",
        "extended-range",
        "fan-of-flames",
        "kinetic-blade",
    ]
    .iter()
    .map(|name| (*name).to_owned())
    .collect();
    assert_eq!(open_infusions(&build, 3), fire);
    let earth: BTreeSet<String> = ["extended-range", "kinetic-blade", "pushing-infusion"]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    assert_eq!(open_infusions(&kineticist("earth", 3), 3), earth);
    let sheet = engine.evaluate(&build);
    assert_eq!(
        engine.slot_label(sheet.slot(&selection_slot(3)).unwrap()),
        "Infusion"
    );
    let picked = kineticist_with("fire", 3, &[(3, &infusion("extended-range"))]);
    assert!(possible(&picked));
    assert_eq!(
        levels_with_slot_of(&kineticist("fire", 2), INFUSIONS_SELECTION),
        Vec::<u8>::new()
    );
    // The 1st-level "Infusion" stays a feature.
    assert!(holds(&at(&build, 1), "kineticist/infusion"));
}

/// The ticket's: "The minimum Kineticist level to use a given Infusion is twice the Infusion's level": one of a higher
/// level is locked with a reason and opens at its level; one taken isn't offered again.
#[test]
fn a_higher_level_infusion_is_locked_with_a_reason_and_opens_at_its_level() {
    let build = kineticist("fire", 19);
    assert_eq!(
        infusion_reason(&build, 3, "detonation"),
        "Needs Kineticist level 8 (have 3)"
    );
    assert_eq!(
        infusion_reason(&build, 5, "detonation"),
        "Needs Kineticist level 8 (have 5)"
    );
    assert!(open_infusions(&build, 9).contains("detonation"));
    assert_eq!(
        infusion_reason(&build, 13, "pure-flame-infusion"),
        "Needs Kineticist level 14 (have 13)"
    );
    assert!(open_infusions(&build, 17).contains("pure-flame-infusion"));
    // Each Fire or Universal infusion with nothing else to need opens at its own level.
    for (name, level, elements, needs, sharpshooter) in INFUSIONS {
        if !needs.is_empty()
            || !matches!(sharpshooter, Sharpshooter::Any)
            || !(elements.is_empty() || elements.contains(&"fire"))
        {
            continue;
        }
        for at in [3, 5, 9, 11, 13, 17, 19] {
            assert_eq!(
                open_infusions(&build, at).contains(name),
                2 * level <= at,
                "{name} at {at}"
            );
        }
    }

    let early = kineticist_with("fire", 3, &[(3, &infusion("detonation"))]);
    assert_eq!(
        why_not(&early, &selection_slot(3)).and_then(|issue| engine().issue_text(&issue)),
        Some("Needs Kineticist level 8 (have 3)".to_owned())
    );
    let taken = kineticist_with("fire", 5, &[(3, &infusion("extended-range"))]);
    assert_eq!(
        infusion_reason(&taken, 5, "extended-range"),
        "Already taken"
    );
    let twice = kineticist_with(
        "fire",
        5,
        &[
            (3, &infusion("extended-range")),
            (5, &infusion("extended-range")),
        ],
    );
    assert!(!possible(&twice));
}

/// The ticket's: a Fire kineticist isn't offered a Water-only infusion, until Expanded Element gives it Water.
#[test]
fn a_fire_kineticist_isnt_offered_a_water_only_infusion() {
    let build = kineticist("fire", 9);
    assert_eq!(
        infusion_reason(&build, 9, "spray"),
        "Needs Elemental Focus - Water"
    );
    assert!(!open_infusions(&build, 9).contains("pushing-infusion"));
    assert!(open_infusions(&build, 9).contains("torrent"));
    let expanded = kineticist_with("fire", 9, &[(7, "kineticist/element/water")]);
    assert!(possible(&expanded));
    assert!(open_infusions(&expanded, 9).contains("spray"));
    assert!(open_infusions(&expanded, 9).contains("pushing-infusion"));
}

/// An infusion's "Prerequisites": Eruption needs Extended Range, Foxfire Infusion the Foxfire wild talent, Blade
/// Whirlwind Kinetic Blade, and Grappling Infusion one of Cloud, Deadly Earth and Wall.
#[test]
fn an_infusion_waits_for_the_infusion_or_talent_it_needs() {
    let build = kineticist("fire", 11);
    assert_eq!(
        infusion_reason(&build, 9, "eruption"),
        "Needs Extended Range"
    );
    let ranged = kineticist_with("fire", 11, &[(3, &infusion("extended-range"))]);
    assert!(open_infusions(&ranged, 9).contains("eruption"));
    assert_eq!(
        infusion_reason(&build, 9, "foxfire-infusion"),
        "Needs Foxfire"
    );
    let foxfire = kineticist_with("fire", 11, &[(4, &talent("foxfire"))]);
    assert!(possible(&foxfire));
    assert!(open_infusions(&foxfire, 9).contains("foxfire-infusion"));
    assert!(!open_infusions(&build, 11).contains("blade-whirlwind"));
    let blade = kineticist_with("fire", 11, &[(3, &infusion("kinetic-blade"))]);
    assert!(open_infusions(&blade, 11).contains("blade-whirlwind"));
    assert!(!open_infusions(&build, 11).contains("grappling-infusion"));
    let wall = kineticist_with("fire", 13, &[(11, &infusion("wall"))]);
    assert!(possible(&wall));
    assert!(open_infusions(&wall, 13).contains("grappling-infusion"));
}

/// The Kinetic Sharpshooter's section: "A number of infusions are exclusive to the Sharpshooter" (Exploding Arrows,
/// Rending Arrows, Honed Infusion, Chain Arrows) and "a number of infusions that the Sharpshooter cannot use"
/// (Extended Range, Kinetic Blade, Eruption, Blade Whirlwind, Deadly Earth, Fragmentation).
#[test]
fn the_kinetic_sharpshooter_has_its_own_infusions_and_not_the_ones_it_cannot_use() {
    let sharpshooter = with_option(
        archetype_build("kineticist", SHARPSHOOTER, 3),
        1,
        0,
        "kineticist/element/fire",
    );
    assert!(possible(&sharpshooter));
    let open = open_infusions(&sharpshooter, 3);
    assert!(open.contains("exploding-arrows"));
    assert!(!open.contains("extended-range"));
    assert!(!open.contains("kinetic-blade"));
    assert!(open.contains("burning-infusion"));
    let base = kineticist("fire", 3);
    assert!(!open_infusions(&base, 3).contains("exploding-arrows"));
    assert_eq!(
        infusion_reason(&base, 3, "exploding-arrows"),
        "Needs the Kinetic Sharpshooter archetype"
    );
    let picked = with_option(sharpshooter, 3, 0, &infusion("exploding-arrows"));
    assert!(possible(&picked));
}

/// The ticket's: each kineticist archetype picks an infusion where its table has "Infusion" after 1st level (the
/// Blood Kineticist's own infusions at 5, 9 and 11 are fixed; the Kinetic Knight has none at 3 or 9), and holds the
/// "Infusion" feature at 1 where its table has it there. Filled, each Build is possible.
#[test]
fn every_kineticist_archetype_picks_infusions_at_its_tables_levels() {
    let every = vec![3, 5, 9, 11, 13, 17, 19];
    assert_eq!(
        levels_with_slot_of(&build("kineticist", 20), INFUSIONS_SELECTION),
        every
    );
    for (archetype, levels, feature_at_1) in [
        ("kineticist/blood-kineticist", vec![3, 13, 17, 19], false),
        ("kineticist/dark-elementalist", every.clone(), true),
        ("kineticist/elemental-engine", every.clone(), true),
        ("kineticist/kinetic-knight", vec![5, 11, 13, 17, 19], false),
        (SHARPSHOOTER, every.clone(), false),
        ("kineticist/overwhelming-soul", every.clone(), true),
        ("kineticist/psychokineticist", every.clone(), true),
    ] {
        let build = archetype_build("kineticist", archetype, 20);
        assert_eq!(
            levels_with_slot_of(&build, INFUSIONS_SELECTION),
            levels,
            "{archetype}"
        );
        assert_eq!(
            holds(&at(&build, 1), "kineticist/infusion"),
            feature_at_1,
            "{archetype}"
        );
        let filled = with_selections_filled(build);
        assert!(possible(&filled), "{archetype} filled");
        assert_eq!(
            levels.len(),
            levels
                .iter()
                .filter(|level| filled.picks.contains_key(&selection_slot(**level)))
                .count(),
            "{archetype}: every slot filled"
        );
    }
    assert_eq!(class("kineticist").archetypes.len(), 7);
}

// ---- talents, hexes and discoveries taken through another class (ticket 60) -------------------------------------

const SLAYER_TALENTS: &str = "slayer/slayer-talents";
const WITCH_HEX: &str = "witch/hex";
const FEYFORM: &str = "shifter/feyform-shifter";
const FEYFORM_TALENT: &str = "shifter/feyform-shifter/rogue-talent";
const VIVISECTIONIST: &str = "alchemist/vivisectionist";
const MEDICAL_DISCOVERY: &str = "alchemist/vivisectionist/medical-discovery";
const SANCTIFIED: &str = "inquisitor/sanctified-slayer";
const HEXCRAFTER: &str = "magus/hexcrafter";
const HEX_ARCANA: &str = "magus/hexcrafter/hex-arcana";
const ARCHAEOLOGIST: &str = "bard/archaeologist";

/// The guide's Advanced Rogue Talents (`rogue#advanced-rogue-talents`: "These unlock at 10th level").
const ADVANCED_TALENTS: [&str; 16] = [
    "blinding-strike",
    "confounding-blades",
    "crippling-strike",
    "dispelling-attack",
    "double-debilitation",
    "familiar",
    "hunters-surprise",
    "improved-evasion",
    "opportunist",
    "petrifying-strike",
    "skill-mastery-knowledge",
    "skill-mastery-lore",
    "skill-mastery-stealth-and-trickery",
    "slippery-mind",
    "weapon-snatcher",
    "wearying-strike",
];

/// `alchemist#medical-discovery`'s three lists, by key.
const MEDICAL_DISCOVERIES: [&str; 25] = [
    "alchemist/discovery/cognatogen",
    "alchemist/discovery/enhance-potion",
    "alchemist/discovery/extend-potion",
    "alchemist/discovery/feral-mutagen",
    "alchemist/discovery/infusion",
    "alchemist/discovery/preserve-organs",
    "alchemist/discovery/spontaneous-healing",
    "alchemist/discovery/feral-wings",
    "alchemist/discovery/mummification",
    "alchemist/discovery/nauseating-flesh",
    "alchemist/discovery/greater-mutagen",
    "alchemist/discovery/grand-mutagen",
    "alchemist/discovery/greater-cognatogen",
    "alchemist/discovery/grand-cognatogen",
    "rogue/rogue-talents/combat-trick",
    "rogue/rogue-talents/fast-stealth",
    "rogue/rogue-talents/focusing-attack-confused",
    "rogue/rogue-talents/focusing-attack-shaken",
    "rogue/rogue-talents/focusing-attack-sickened",
    "rogue/rogue-talents/iron-guts",
    "rogue/rogue-talents/slow-reactions",
    "rogue/rogue-talents/weakening-wound",
    "rogue/rogue-talents/confounding-blades",
    "rogue/rogue-talents/crippling-strike",
    "rogue/rogue-talents/dispelling-attack",
];

/// The features the new slots replace: each was a label for a pick.
const BORROWED_FEATURES: [(&str, &str); 7] = [
    ("bard", "bard/bard-talents"),
    ("skald", "skald/skald-talents"),
    ("shifter", FEYFORM_TALENT),
    ("alchemist", MEDICAL_DISCOVERY),
    ("inquisitor", "inquisitor/sanctified-slayer/slayer-talents"),
    ("magus", "magus/hexcrafter/hex-magus"),
    ("magus", HEX_ARCANA),
];

/// A chaotic neutral Build taking each class in turn for its levels, with its archetype if any, with no picks.
fn multiclass(parts: &[(&str, Option<&str>, usize)]) -> Build {
    let mut build = Build::default();
    build.start.alignment = Some(rules::Alignment::ChaoticNeutral);
    for (class, archetype, levels) in parts {
        build.class_choices.insert(
            ClassKey::new(*class),
            ClassChoice {
                archetype: archetype.map(ArchetypeKey::new),
                ..ClassChoice::default()
            },
        );
        for _ in 0..*levels {
            build.levels.push(LevelUp::new(ClassKey::new(*class)));
        }
    }
    build
}

/// A chaotic neutral Build of one class (with its archetype if any), `levels` levels, with no picks.
fn borrower(class: &str, archetype: Option<&str>, levels: usize) -> Build {
    multiclass(&[(class, archetype, levels)])
}

/// Every selection of the data, by key.
fn any_selection(key: &str) -> &'static rules::FeatureSelection {
    DATA.classes
        .iter()
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .find(|selection| selection.key.as_str() == key)
        .unwrap_or_else(|| panic!("no selection {key}"))
}

fn option_prerequisite(key: &str) -> &'static Prerequisite {
    DATA.classes
        .iter()
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .flat_map(|selection| &selection.options)
        .find(|option| option.key.as_str() == key)
        .unwrap_or_else(|| panic!("no option {key}"))
        .prerequisite
        .as_ref()
        .unwrap_or_else(|| panic!("{key}: no prerequisite"))
}

fn names_feature(prerequisite: &Prerequisite, feature: &str) -> bool {
    match prerequisite {
        Prerequisite::All(children) | Prerequisite::Any(children) => {
            children.iter().any(|child| names_feature(child, feature))
        }
        Prerequisite::Feature { feature: f, .. } => f.as_str() == feature,
        _ => false,
    }
}

fn keys(selection: &rules::FeatureSelection) -> BTreeSet<String> {
    selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .chain(selection.shared_options.iter().map(|k| k.to_string()))
        .collect()
}

/// The one selection slot of the Build at character level `level`.
fn the_slot_at(build: &Build, level: u8) -> SlotId {
    let found: Vec<SlotId> = engine()
        .evaluate(build)
        .levels
        .iter()
        .flat_map(|l| &l.slots)
        .filter(|slot| {
            matches!(slot.kind, SlotKind::Selection(_))
                && matches!(slot.id, SlotId::Selection { level: at, .. } if at == level)
        })
        .map(|slot| slot.id.clone())
        .collect();
    let [slot] = found.as_slice() else {
        panic!("{} selection slots at {level}", found.len())
    };
    slot.clone()
}

/// The one selection slot of the Build at character level `level` whose selection offers `option`.
fn slot_offering(build: &Build, level: u8, option: &str) -> SlotId {
    let found: Vec<SlotId> = engine()
        .evaluate(build)
        .levels
        .iter()
        .flat_map(|l| &l.slots)
        .filter(|slot| {
            matches!(&slot.kind, SlotKind::Selection(selection)
                if any_selection(selection.as_str()).offers(&SelectionOptionKey::new(option)))
                && matches!(slot.id, SlotId::Selection { level: at, .. } if at == level)
        })
        .map(|slot| slot.id.clone())
        .collect();
    let [slot] = found.as_slice() else {
        panic!("{} slots offering {option} at {level}", found.len())
    };
    slot.clone()
}

/// The guide's lists are features in the data no more: each borrowed pick is a slot, and the feature that stood for
/// it is gone from its class's (or archetype's) features and from every row. The Advanced Talents features of the
/// Archaeologist and the Vivisectionist stay: they open the advanced rogue talents.
#[test]
fn the_borrowed_picks_replace_their_features() {
    for (class_key, feature) in BORROWED_FEATURES {
        let class = class(class_key);
        let defined = class
            .features
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.features))
            .any(|f| f.key.as_str() == feature);
        assert!(!defined, "{feature} is still a feature");
        let on_a_row = class
            .levels
            .iter()
            .map(|row| &row.features)
            .chain(
                class
                    .archetypes
                    .iter()
                    .flat_map(|a| a.levels.iter().map(|row| &row.features)),
            )
            .flatten()
            .any(|f| f.as_str() == feature);
        assert!(!on_a_row, "{feature} is still on a row");
    }
    for (class_key, archetype, feature) in [
        ("bard", ARCHAEOLOGIST, "bard/archaeologist/advanced-talents"),
        (
            "alchemist",
            VIVISECTIONIST,
            "alchemist/vivisectionist/advanced-talents",
        ),
    ] {
        assert!(
            archetype_features(class_key, archetype).contains(feature),
            "{feature}"
        );
    }
}

/// Each list is an existing selection where the guide's list is one ("The Bard can select from the Rogue Talent
/// list", "a Sanctified Slayer can select a Slayer talent", "a Hexcrafter may select a Witch hex"), or a selection
/// sharing the options of the lists it mixes, as ticket 25 made the slayer's (the Feyform Shifter's rogue talents but
/// the advanced ones; the Vivisectionist's discoveries and talents; the Hexcrafter's arcana and hexes). Combat Trick's
/// row needs an option level, so the classes that pick through each name it.
#[test]
fn the_borrowed_selections_offer_the_guides_lists() {
    let rogue = any_selection(ROGUE_TALENTS);
    let advanced: BTreeSet<String> = ADVANCED_TALENTS.iter().map(|t| rogue_talent(t)).collect();
    let all_talents = keys(rogue);
    assert_eq!(all_talents.len(), 36);
    assert!(advanced.is_subset(&all_talents));
    let classes = |selection: &rules::FeatureSelection| -> BTreeSet<String> {
        selection
            .option_levels
            .iter()
            .map(|c| c.to_string())
            .collect()
    };
    let set =
        |items: &[&str]| -> BTreeSet<String> { items.iter().map(|s| s.to_string()).collect() };
    assert_eq!(classes(rogue), set(&["bard", "rogue", "skald"]));
    assert_eq!(
        classes(any_selection(SLAYER_TALENTS)),
        set(&["inquisitor", "slayer"])
    );

    let feyform = any_selection(FEYFORM_TALENT);
    assert_eq!(feyform.name, "Rogue Talent");
    assert!(feyform.options.is_empty());
    assert_eq!(keys(feyform), &all_talents - &advanced);
    assert_eq!(keys(feyform).len(), 20);
    assert_eq!(classes(feyform), set(&["shifter"]));

    let medical = any_selection(MEDICAL_DISCOVERY);
    assert_eq!(medical.name, "Medical Discovery");
    assert!(medical.options.is_empty());
    assert_eq!(keys(medical), set(&MEDICAL_DISCOVERIES));
    assert_eq!(classes(medical), set(&["alchemist"]));

    let hex_arcana = any_selection(HEX_ARCANA);
    assert_eq!(hex_arcana.name, "Hex Arcana");
    assert!(hex_arcana.options.is_empty());
    let arcana_and_hexes: BTreeSet<String> =
        &keys(any_selection("magus/magus-arcana")) | &keys(any_selection(WITCH_HEX));
    assert_eq!(arcana_and_hexes.len(), 15 + 26);
    assert_eq!(keys(hex_arcana), arcana_and_hexes);
    assert!(hex_arcana.option_levels.is_empty());

    // "At 10th level, the Archaeologist may select from the Advanced Rogue Talents"; the Vivisectionist only its
    // three, "including Advanced Talents from level 10".
    for talent in &advanced {
        let prerequisite = option_prerequisite(talent);
        assert!(
            names_feature(prerequisite, "bard/archaeologist/advanced-talents"),
            "{talent}"
        );
        assert_eq!(
            names_feature(prerequisite, "alchemist/vivisectionist/advanced-talents"),
            MEDICAL_DISCOVERIES.contains(&talent.as_str()),
            "{talent}"
        );
    }
}

/// Each class or archetype picks where its table says: Bard Talent, Skald Talent, the Feyform Shifter's Rogue
/// Talent, Medical Discovery, the Sanctified Slayer's Slayer Talent, Hex Magus and Hex Arcana. Filled, each is possible.
#[test]
fn each_borrowing_class_picks_at_its_tables_levels() {
    let bard = vec![2, 6, 10, 14, 18];
    let skald = vec![2, 7, 12, 17];
    let mut cases: Vec<(Build, &str, Vec<u8>)> = vec![
        (borrower("bard", None, 20), ROGUE_TALENTS, bard.clone()),
        (borrower("skald", None, 20), ROGUE_TALENTS, skald.clone()),
        (
            borrower("bard", Some("bard/beast-tamer"), 20),
            ROGUE_TALENTS,
            vec![6, 14, 18],
        ),
        (
            borrower("skald", Some("skald/inciter"), 20),
            ROGUE_TALENTS,
            vec![4, 9, 14, 19],
        ),
        (
            borrower("shifter", Some(FEYFORM), 20),
            FEYFORM_TALENT,
            vec![8, 14],
        ),
        (
            borrower("alchemist", Some(VIVISECTIONIST), 20),
            MEDICAL_DISCOVERY,
            vec![2, 4, 6, 8, 10, 12, 14, 16, 18],
        ),
        (
            borrower("inquisitor", Some(SANCTIFIED), 20),
            SLAYER_TALENTS,
            vec![8, 16, 17, 20],
        ),
        (borrower("magus", Some(HEXCRAFTER), 20), WITCH_HEX, vec![4]),
        (
            borrower("magus", Some(HEXCRAFTER), 20),
            HEX_ARCANA,
            vec![6, 9, 12, 15, 18],
        ),
    ];
    for archetype in [
        ARCHAEOLOGIST,
        "bard/chelaxian-diva",
        "bard/dirge-bard",
        "bard/flame-dancer",
        "bard/thundercaller",
        "bard/tranquil-whisperer",
    ] {
        cases.push((
            borrower("bard", Some(archetype), 20),
            ROGUE_TALENTS,
            bard.clone(),
        ));
    }
    for archetype in [
        "skald/battle-scion",
        "skald/battle-singer",
        "skald/court-poet",
        "skald/demon-dancer",
        "skald/herald-of-the-horn",
        "skald/hunt-caller",
    ] {
        cases.push((
            borrower("skald", Some(archetype), 20),
            ROGUE_TALENTS,
            skald.clone(),
        ));
    }
    assert_eq!(class("bard").archetypes.len(), 7);
    assert_eq!(class("skald").archetypes.len(), 7);
    for (build, selection, levels) in cases {
        let who = format!("{:?} {selection}", build.class_choices);
        assert_eq!(levels_with_slot_of(&build, selection), levels, "{who}");
        let filled = with_selections_filled(build);
        assert!(possible(&filled), "{who}: filled");
        assert!(
            levels.iter().all(|level| filled
                .picks
                .contains_key(&slot_of(&filled, *level, selection))),
            "{who}: every slot filled"
        );
    }
    // The Vivisectionist's Medical Discovery is its discovery: no alchemist discovery slot is left, and the Hexcrafter
    // has no magus arcana slot of its own.
    assert_eq!(
        levels_with_slot_of(
            &borrower("alchemist", Some(VIVISECTIONIST), 20),
            "alchemist/discovery"
        ),
        Vec::<u8>::new()
    );
    assert_eq!(
        levels_with_slot_of(
            &borrower("magus", Some(HEXCRAFTER), 20),
            "magus/magus-arcana"
        ),
        Vec::<u8>::new()
    );
    // Other archetypes pick none of them.
    for (class_key, selection) in [
        ("inquisitor", SLAYER_TALENTS),
        ("magus", WITCH_HEX),
        ("shifter", FEYFORM_TALENT),
        ("alchemist", MEDICAL_DISCOVERY),
    ] {
        assert_eq!(
            levels_with_slot_of(&borrower(class_key, None, 20), selection),
            Vec::<u8>::new(),
            "{class_key}"
        );
    }
}

/// A bard and a skald pick from the rogue talents; the advanced ones need Advanced Talents, which an Archaeologist
/// has from 10.
#[test]
fn a_bard_takes_rogue_talents_and_an_archaeologist_advanced_ones_from_10() {
    let crippling = rogue_talent("crippling-strike");
    for build in [borrower("bard", None, 2), borrower("skald", None, 2)] {
        let offered = offered(&build, 2);
        assert_eq!(offered.len(), 36, "{:?}", build.levels[0].class);
        assert_eq!(offered[&rogue_talent("fast-stealth")], Ok(()));
        assert!(offered[&crippling].is_err());
    }
    assert!(offered(&borrower("bard", None, 10), 10)[&crippling].is_err());
    assert_eq!(
        lock_reason(&borrower("bard", Some(ARCHAEOLOGIST), 6), 6, &crippling),
        "Needs Advanced Talents (Archaeologist)"
    );
    let archaeologist = borrower("bard", Some(ARCHAEOLOGIST), 10);
    assert_eq!(offered(&archaeologist, 10)[&crippling], Ok(()));
    let with = with_picks(archaeologist, &[(10, pick(&crippling))]);
    assert!(possible(&with));
}

/// "At 8th and 14th levels, the shifter can choose a Rogue Talent. Note that you cannot choose Advanced Talents":
/// the advanced ones aren't offered, even to a rogue 10.
#[test]
fn a_feyform_shifter_picks_a_rogue_talent_at_8_and_14_but_no_advanced_one() {
    let feyform = borrower("shifter", Some(FEYFORM), 8);
    let offered = offered(&feyform, 8);
    assert_eq!(offered.len(), 20);
    assert_eq!(offered[&rogue_talent("combat-trick")], Ok(()));
    let rogue_then_feyform = multiclass(&[("rogue", None, 10), ("shifter", Some(FEYFORM), 8)]);
    let slot = the_slot_at(&rogue_then_feyform, 18);
    let offered = offered_in(&rogue_then_feyform, &slot);
    assert!(
        ADVANCED_TALENTS
            .iter()
            .all(|t| !offered.contains_key(&rogue_talent(t)))
    );
    let with = with_picks(feyform, &[(8, pick(&rogue_talent("fast-stealth")))]);
    assert!(possible(&with));
}

/// "The Vivisectionist is able to select certain Rogue Talents (including Advanced Talents from level 10) and
/// Alchemist discoveries": its lists, with Crippling Strike open from 10.
#[test]
fn a_vivisectionist_picks_its_discoveries_and_talents_and_advanced_ones_from_10() {
    let at_2 = borrower("alchemist", Some(VIVISECTIONIST), 2);
    let offered_2 = offered(&at_2, 2);
    assert_eq!(
        offered_2.keys().cloned().collect::<BTreeSet<_>>(),
        MEDICAL_DISCOVERIES.iter().map(|k| k.to_string()).collect()
    );
    for open in [
        "alchemist/discovery/cognatogen",
        "rogue/rogue-talents/fast-stealth",
        "rogue/rogue-talents/combat-trick",
    ] {
        assert_eq!(offered_2[open], Ok(()), "{open}");
    }
    let crippling = rogue_talent("crippling-strike");
    assert_eq!(
        lock_reason(&at_2, 2, &crippling),
        "Needs Advanced Talents (Vivisectionist)"
    );
    assert!(offered_2["alchemist/discovery/greater-mutagen"].is_err());
    let at_10 = borrower("alchemist", Some(VIVISECTIONIST), 10);
    assert_eq!(offered(&at_10, 10)[&crippling], Ok(()));
    // Blinding Strike is an advanced talent the list doesn't name.
    assert!(!offered(&at_10, 10).contains_key(&rogue_talent("blinding-strike")));
    let with = with_picks(
        at_10,
        &[
            (2, pick("alchemist/discovery/feral-mutagen")),
            (6, pick("alchemist/discovery/feral-wings")),
            (10, pick(&crippling)),
        ],
    );
    assert!(possible(&with));
}

/// "At 8th, 16th, 17th and 20th levels, a Sanctified Slayer can select a Slayer talent": the slayer's talents, its
/// combat style feats the slayer's own (they need slayer levels), and Extra Slayer Talent from its first talent.
#[test]
fn a_sanctified_slayer_picks_slayer_talents_at_8_16_17_and_20() {
    let at_8 = borrower("inquisitor", Some(SANCTIFIED), 8);
    let offered_8 = offered(&at_8, 8);
    assert_eq!(offered_8.len(), keys(any_selection(SLAYER_TALENTS)).len());
    for open in [
        "slayer/slayer-talents/trapfinding",
        "rogue/rogue-talents/combat-trick",
        "rogue/rogue-talents/fast-stealth",
    ] {
        assert_eq!(offered_8[open], Ok(()), "{open}");
    }
    assert_eq!(
        lock_reason(&at_8, 8, "slayer/slayer-talents/combat-style-feat-2"),
        "Needs Slayer level 2 (have 0)"
    );
    assert!(offered_8[&rogue_talent("crippling-strike")].is_err());
    // A slayer 2 still takes its 2nd-or-4th combat style feat.
    assert_eq!(
        offered(&build("slayer", 2), 2)["slayer/slayer-talents/combat-style-feat-2"],
        Ok(())
    );
    // "Prerequisites: Slayer Talents": the Sanctified Slayer's Slayer Talents from 8.
    assert!(
        feat_status(
            &borrower("inquisitor", Some(SANCTIFIED), 7),
            &general_slot(7),
            "extra-slayer-talent"
        )
        .is_err()
    );
    let mut at_9 = with_selections_filled(borrower("inquisitor", Some(SANCTIFIED), 9));
    assert_eq!(
        feat_status(&at_9, &general_slot(9), "extra-slayer-talent"),
        Ok(())
    );
    at_9.picks
        .insert(general_slot(9), feat_pick_plain("extra-slayer-talent"));
    assert!(possible(&at_9));
    assert_eq!(
        kind_at(&at_9, &extra_slot(9, 0)),
        Some(selection_kind(SLAYER_TALENTS))
    );
}

/// "At 4th level, a Hexcrafter may select a Witch hex"; "Besides the usual Magus Arcana abilities, a Hexcrafter can
/// select a hex. From 12th level, a Hexcrafter can select major hexes and at 18th level, he can select a grand hex."
#[test]
fn a_hexcrafter_picks_a_witch_hex_at_4_and_an_arcana_or_hex_from_6() {
    let at_4 = borrower("magus", Some(HEXCRAFTER), 4);
    let hex_magus = offered(&at_4, 4);
    assert_eq!(hex_magus.len(), 26);
    assert_eq!(hex_magus["witch/hex/cackle"], Ok(()));
    assert!(hex_magus["witch/hex/agony"].is_err());

    let at_6 = borrower("magus", Some(HEXCRAFTER), 6);
    let hex_arcana = offered(&at_6, 6);
    assert_eq!(hex_arcana.len(), 41);
    for open in ["magus/magus-arcana/arcane-accuracy", "witch/hex/cackle"] {
        assert_eq!(hex_arcana[open], Ok(()), "{open}");
    }
    let at_9 = borrower("magus", Some(HEXCRAFTER), 9);
    assert_eq!(
        lock_reason(&at_9, 9, "witch/hex/agony"),
        "Needs Magus level 12 (have 9)"
    );
    assert_eq!(
        offered(&borrower("magus", Some(HEXCRAFTER), 12), 12)["witch/hex/agony"],
        Ok(())
    );
    let at_15 = borrower("magus", Some(HEXCRAFTER), 15);
    assert_eq!(
        lock_reason(&at_15, 15, "witch/hex/death-curse"),
        "Needs Magus level 18 (have 15)"
    );
    let at_18 = borrower("magus", Some(HEXCRAFTER), 18);
    assert_eq!(offered(&at_18, 18)["witch/hex/death-curse"], Ok(()));
    let with = with_picks(
        at_18,
        &[
            (4, pick("witch/hex/slumber")),
            (6, pick("magus/magus-arcana/arcane-accuracy")),
            (12, pick("witch/hex/restless-slumber")),
            (18, pick("witch/hex/death-curse")),
        ],
    );
    assert!(possible(&with));
    // A magus of no archetype still can't.
    assert!(!offered(&borrower("magus", None, 12), 12).contains_key("witch/hex/agony"));
}

/// An option is one key whichever class takes it: taken through one class's slot, it's "Already taken" through
/// another's, and twice is impossible.
#[test]
fn an_option_taken_through_one_class_isnt_offered_again_through_another() {
    let fast_stealth = rogue_talent("fast-stealth");
    let cases: [(Build, u8, &str, u8); 7] = [
        (
            multiclass(&[("rogue", None, 2), ("bard", None, 2)]),
            2,
            &fast_stealth,
            4,
        ),
        (
            multiclass(&[("bard", None, 2), ("skald", None, 2)]),
            2,
            &fast_stealth,
            4,
        ),
        (
            multiclass(&[("rogue", None, 2), ("shifter", Some(FEYFORM), 8)]),
            2,
            &fast_stealth,
            10,
        ),
        (
            multiclass(&[("rogue", None, 2), ("alchemist", Some(VIVISECTIONIST), 2)]),
            2,
            &fast_stealth,
            4,
        ),
        (
            multiclass(&[("slayer", None, 2), ("inquisitor", Some(SANCTIFIED), 8)]),
            2,
            "slayer/slayer-talents/trapfinding",
            10,
        ),
        (
            multiclass(&[("witch", None, 1), ("magus", Some(HEXCRAFTER), 4)]),
            1,
            "witch/hex/cackle",
            5,
        ),
        (
            multiclass(&[("witch", None, 1), ("magus", Some(HEXCRAFTER), 6)]),
            1,
            "witch/hex/cackle",
            7,
        ),
    ];
    for (build, first, option, then) in cases {
        let who = format!("{:?}", build.class_choices);
        let first_slot = slot_offering(&build, first, option);
        let mut build = build.clone();
        build.picks.insert(first_slot, pick(option));
        assert!(possible(&build), "{who}");
        let slot = slot_offering(&build, then, option);
        assert_eq!(reason_in(&build, &slot, option), "Already taken", "{who}");
        build.picks.insert(slot, pick(option));
        assert!(!possible(&build), "{who}: twice");
    }
}

// ---- Extra Discovery and Extra Arcana for the Vivisectionist and the Hexcrafter (ticket 64) -------------------------

/// The keys a slot's picker offers.
fn offered_keys(build: &Build, slot: &SlotId) -> BTreeSet<String> {
    offered_in(build, slot).into_keys().collect()
}

/// The sheet's label of `slot`.
fn label_at(build: &Build, slot: &SlotId) -> String {
    let engine = engine();
    let sheet = engine.evaluate(build);
    engine.slot_label(
        sheet
            .slot(slot)
            .unwrap_or_else(|| panic!("no slot {slot:?}")),
    )
}

/// Ticket 60's decision 2: Extra Discovery gives a Vivisectionist one more Medical Discovery, at the level it's taken
/// (ticket 49's rule: the Vivisectionist has no slot of the alchemist's Discovery, so its first is Medical Discovery's).
#[test]
fn a_vivisectionists_extra_discovery_gives_a_medical_discovery_slot_at_the_level_its_taken() {
    let mut build = with_selections_filled(borrower("alchemist", Some(VIVISECTIONIST), 5));
    for level in [3, 5] {
        build
            .picks
            .insert(general_slot(level), feat_pick_plain("extra-discovery"));
    }
    assert!(possible(&build));
    for level in [3, 5] {
        let slot = extra_slot(level, 0);
        assert_eq!(
            kind_at(&build, &slot),
            Some(selection_kind(MEDICAL_DISCOVERY)),
            "at {level}"
        );
        assert_eq!(label_at(&build, &slot), "Medical Discovery");
        // What the Vivisectionist's own slot at 2 offers: its discoveries and talents.
        assert_eq!(
            offered_keys(&build, &slot),
            offered_keys(&build, &selection_slot(2))
        );
        assert_eq!(offered_keys(&build, &slot).len(), MEDICAL_DISCOVERIES.len());
    }
    assert_eq!(kind_at(&build, &extra_slot(4, 0)), None);
    let filled = with_selections_filled(build);
    for level in [3, 5] {
        assert!(
            filled.picks.contains_key(&extra_slot(level, 0)),
            "at {level}"
        );
    }
    assert!(possible(&filled));
}

/// Ticket 60's decision 3: Extra Arcana gives a Hexcrafter one more Hex Arcana, the arcana and the hexes, a major hex
/// from magus 12 as in its own slots.
#[test]
fn a_hexcrafters_extra_arcana_gives_a_hex_arcana_slot() {
    let mut build = with_selections_filled(borrower("magus", Some(HEXCRAFTER), 7));
    build
        .picks
        .insert(general_slot(7), feat_pick_plain("extra-arcana"));
    assert!(possible(&build));
    let slot = extra_slot(7, 0);
    assert_eq!(kind_at(&build, &slot), Some(selection_kind(HEX_ARCANA)));
    assert_eq!(label_at(&build, &slot), "Hex Arcana");
    assert_eq!(
        offered_keys(&build, &slot),
        offered_keys(&build, &selection_slot(6))
    );
    assert_eq!(offered_keys(&build, &slot).len(), 41);
    assert_eq!(
        reason_in(&build, &slot, "witch/hex/agony"),
        "Needs Magus level 12 (have 7)"
    );
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&slot));
    assert!(possible(&filled));
}

/// "Prerequisites: Dicovery" and "Prerequisites: Magus Arcana" name the class feature, and the archetypes' are their
/// version of it (Medical Discovery where the alchemist has Discovery; Hex Arcana, "Besides the usual Magus Arcana
/// abilities"): a Vivisectionist from alchemist 2 and a Hexcrafter from magus 6 meet them whatever they picked there,
/// as a Sanctified Slayer 8 meets Extra Slayer Talent's (ticket 60). Before, neither does.
#[test]
fn extra_discovery_and_extra_arcana_reach_a_vivisectionist_and_a_hexcrafter_whatever_they_picked() {
    let fast_stealth = rogue_talent("fast-stealth");
    let vivisectionist = with_picks(
        borrower("alchemist", Some(VIVISECTIONIST), 3),
        &[(2, pick(&fast_stealth))],
    );
    assert!(possible(&vivisectionist));
    assert_eq!(
        feat_status(&vivisectionist, &general_slot(3), "extra-discovery"),
        Ok(())
    );
    let hexcrafter = with_picks(
        borrower("magus", Some(HEXCRAFTER), 7),
        &[
            (4, pick("witch/hex/slumber")),
            (6, pick("witch/hex/cackle")),
        ],
    );
    assert!(possible(&hexcrafter));
    assert_eq!(
        feat_status(&hexcrafter, &general_slot(7), "extra-arcana"),
        Ok(())
    );
    // Before their first slot, neither.
    assert!(
        feat_status(
            &borrower("alchemist", Some(VIVISECTIONIST), 1),
            &general_slot(1),
            "extra-discovery"
        )
        .is_err()
    );
    let hexcrafter_5 = with_picks(
        borrower("magus", Some(HEXCRAFTER), 5),
        &[(4, pick("witch/hex/slumber"))],
    );
    assert!(feat_status(&hexcrafter_5, &general_slot(5), "extra-arcana").is_err());
    // Another class's archetype doesn't: a rogue 2 holding Fast Stealth, then alchemist 1.
    let rogue_then_vivisectionist = with_selections_filled(multiclass(&[
        ("rogue", None, 2),
        ("alchemist", Some(VIVISECTIONIST), 1),
    ]));
    assert!(
        feat_status(
            &rogue_then_vivisectionist,
            &general_slot(3),
            "extra-discovery"
        )
        .is_err()
    );
}

/// A plain alchemist and magus are as before: their Extra feat needs a discovery or an arcana held, and gives a slot of
/// the alchemist's Discovery or the magus's Magus Arcana, offering what their own slots offer.
#[test]
fn a_plain_alchemists_and_magus_extra_picks_are_unchanged() {
    for (class, feat, own, selection, label) in [
        (
            "alchemist",
            "extra-discovery",
            2,
            "alchemist/discovery",
            "Discovery",
        ),
        (
            "magus",
            "extra-arcana",
            3,
            "magus/magus-arcana",
            "Magus Arcana",
        ),
    ] {
        let empty = borrower(class, None, 3);
        assert!(
            feat_status(&empty, &general_slot(3), feat).is_err(),
            "{feat}: nothing held"
        );
        let mut build = with_selections_filled(empty);
        assert_eq!(
            feat_status(&build, &general_slot(3), feat),
            Ok(()),
            "{feat}"
        );
        build.picks.insert(general_slot(3), feat_pick_plain(feat));
        assert!(possible(&build), "{feat}");
        let slot = extra_slot(3, 0);
        assert_eq!(kind_at(&build, &slot), Some(selection_kind(selection)));
        assert_eq!(label_at(&build, &slot), label);
        assert_eq!(
            offered_keys(&build, &slot),
            offered_keys(&build, &selection_slot(own)),
            "{feat}"
        );
    }
}

// ---- archetype bonus-feat choices (ticket 61) ---------------------------------------------------------------------

const TITAN: &str = "fighter/titan-fighter";
const URBAN_HUNTER: &str = "hunter/urban-hunter";
const STORMWALKER: &str = "ranger/stormwalker";

/// The features that stood for the four bonus-feat choices: each is a slot of its list now, as the Divine
/// Guardian's bonus feats are (a list and its slots, no feature).
const BONUS_FEAT_FEATURES: [(&str, &str); 4] = [
    ("fighter", "fighter/titan-fighter/massive-attacks"),
    ("hunter", "hunter/precise-companion"),
    ("hunter", "hunter/urban-hunter/captor"),
    ("ranger", "ranger/stormwalker/archery"),
];

/// Each choice's lists: the slot at each level picks from the list of the last threshold reached.
const MASSIVE_ATTACKS: [&str; 3] = [
    "fighter/titan-fighter/bonus-feats-2",
    "fighter/titan-fighter/bonus-feats-6",
    "fighter/titan-fighter/bonus-feats-10",
];
const PRECISE_COMPANION: &str = "hunter/bonus-feats-2";
const CAPTOR: [&str; 2] = [
    "hunter/urban-hunter/bonus-feats-6",
    "hunter/urban-hunter/bonus-feats-12",
];
const STORMWALKER_ARCHERY: [&str; 3] = [
    "ranger/bonus-feats-2b",
    "ranger/bonus-feats-6b",
    "ranger/bonus-feats-10b",
];

/// Every feat list of the data, by key.
fn any_feat_list(key: &str) -> &'static rules::FeatList {
    DATA.classes
        .iter()
        .flat_map(|class| {
            class
                .feat_lists
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.feat_lists))
        })
        .find(|list| list.key.as_str() == key)
        .unwrap_or_else(|| panic!("no feat list {key}"))
}

fn list_feats(key: &str) -> BTreeSet<String> {
    any_feat_list(key)
        .feats
        .iter()
        .map(|f| f.to_string())
        .collect()
}

fn feat_set(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|k| k.to_string()).collect()
}

/// The `OneOf` lists of the rows in force for `class` (with `archetype`) at levels 1 to 20, by level.
fn one_of_lists(class_key: &str, archetype: Option<&str>) -> Vec<(u8, String)> {
    let class = class(class_key);
    let archetype = archetype.map(|key| {
        class
            .archetypes
            .iter()
            .find(|a| a.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"))
    });
    (1..=20)
        .filter_map(|level| rules::effective_row(class, archetype, level))
        .flat_map(|row| {
            row.feat_slots
                .iter()
                .filter_map(move |slot| match slot {
                    FeatSlot::OneOf { list } => Some((row.level, list.to_string())),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The rows in force hold `lists` (level, list) among their `OneOf` slots, and no other slot of those lists.
fn holds_lists(class_key: &str, archetype: Option<&str>, lists: &[(u8, &str)]) {
    let names: BTreeSet<&str> = lists.iter().map(|(_, list)| *list).collect();
    let found: Vec<(u8, String)> = one_of_lists(class_key, archetype)
        .into_iter()
        .filter(|(_, list)| names.contains(list.as_str()))
        .collect();
    let expected: Vec<(u8, String)> = lists
        .iter()
        .map(|(level, list)| (*level, list.to_string()))
        .collect();
    assert_eq!(found, expected, "{class_key} {archetype:?}");
}

/// The guide's sentences, as slots: the bonus-feat choice is a pick, and the feature that stood for it is gone from
/// its class's (or archetype's) features and from every row. Nothing needed one.
#[test]
fn the_bonus_feat_choices_replace_their_features() {
    for (class_key, feature) in BONUS_FEAT_FEATURES {
        let class = class(class_key);
        let defined = class
            .features
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.features))
            .any(|f| f.key.as_str() == feature);
        assert!(!defined, "{feature} is still a feature");
        let on_a_row = class
            .levels
            .iter()
            .map(|row| &row.features)
            .chain(
                class
                    .archetypes
                    .iter()
                    .flat_map(|a| a.levels.iter().map(|row| &row.features)),
            )
            .flatten()
            .any(|f| f.as_str() == feature);
        assert!(!on_a_row, "{feature} is still on a row");
    }
}

/// Each choice is a slot where its table has the cell: Massive Attacks at 2 and every four levels (in place of the
/// fighter's Bonus Combat Feat there), Precise Companion at 2 for the hunter and every archetype but the Forester,
/// Captor at 6 and every three levels, the Stormwalker's Archery at 2, 6, 10, 14 and 18.
#[test]
fn each_bonus_feat_choice_is_a_slot_at_its_tables_levels() {
    holds_lists(
        "fighter",
        Some(TITAN),
        &[
            (2, MASSIVE_ATTACKS[0]),
            (6, MASSIVE_ATTACKS[1]),
            (10, MASSIVE_ATTACKS[2]),
            (14, MASSIVE_ATTACKS[2]),
            (18, MASSIVE_ATTACKS[2]),
        ],
    );
    let titan_combat: Vec<u8> = {
        let class = class("fighter");
        let titan = class.archetypes.iter().find(|a| a.key.as_str() == TITAN);
        (1..=20)
            .filter(|level| {
                rules::effective_row(class, titan, *level)
                    .is_some_and(|row| row.feat_slots.contains(&FeatSlot::CombatFeat))
            })
            .collect()
    };
    assert_eq!(titan_combat, [1, 4, 8, 12, 16, 20]);

    holds_lists("hunter", None, &[(2, PRECISE_COMPANION)]);
    for archetype in &class("hunter").archetypes {
        let expected: &[(u8, &str)] = if archetype.key.as_str() == "hunter/forester" {
            &[]
        } else {
            &[(2, PRECISE_COMPANION)]
        };
        holds_lists("hunter", Some(archetype.key.as_str()), expected);
    }
    holds_lists(
        "hunter",
        Some(URBAN_HUNTER),
        &[
            (6, CAPTOR[0]),
            (9, CAPTOR[0]),
            (12, CAPTOR[1]),
            (15, CAPTOR[1]),
            (18, CAPTOR[1]),
        ],
    );
    holds_lists(
        "ranger",
        Some(STORMWALKER),
        &[
            (2, STORMWALKER_ARCHERY[0]),
            (6, STORMWALKER_ARCHERY[1]),
            (10, STORMWALKER_ARCHERY[2]),
            (14, STORMWALKER_ARCHERY[2]),
            (18, STORMWALKER_ARCHERY[2]),
        ],
    );
    // Massive Attacks is the Titan Fighter's alone.
    for (class_key, archetype) in std::iter::once(("fighter", None)).chain(
        class("fighter")
            .archetypes
            .iter()
            .filter(|a| a.key.as_str() != TITAN)
            .map(|a| ("fighter", Some(a.key.as_str()))),
    ) {
        holds_lists(class_key, archetype, &[]);
        assert!(
            one_of_lists(class_key, archetype)
                .iter()
                .all(|(_, list)| !MASSIVE_ATTACKS.contains(&list.as_str())),
            "{archetype:?}"
        );
    }
}

/// The lists are the guide's, each ignoring prerequisites. Massive Attacks: "a feat from the Ranger's Two-Weapon
/// Combat Style feats" (Double Slice, Shield Bash and Two-Weapon Fighting; Improved Two-Weapon Fighting from 6;
/// Greater Two-Weapon Fighting from 10), the style feats without the extras every ranger style offers, as the
/// Stormwalker's archery lists are the archery style's. Precise Companion: Precise Shot or Outflank. Captor: four
/// feats, six more from 12.
#[test]
fn the_bonus_feat_lists_are_the_guides() {
    let extras = feat_set(&[
        "dodge",
        "great-fortitude",
        "iron-will",
        "lightning-reflexes",
        "skill-focus",
        "toughness",
    ]);
    let two_weapon = [
        feat_set(&["double-slice", "shield-bash", "two-weapon-fighting"]),
        feat_set(&[
            "double-slice",
            "improved-two-weapon-fighting",
            "shield-bash",
            "two-weapon-fighting",
        ]),
        feat_set(&[
            "double-slice",
            "greater-two-weapon-fighting",
            "improved-two-weapon-fighting",
            "shield-bash",
            "two-weapon-fighting",
        ]),
    ];
    let names = [
        "Massive Attacks",
        "Massive Attacks (from level 6)",
        "Massive Attacks (from level 10)",
    ];
    for (i, level) in [2, 6, 10].into_iter().enumerate() {
        let list = any_feat_list(MASSIVE_ATTACKS[i]);
        assert_eq!(list_feats(MASSIVE_ATTACKS[i]), two_weapon[i], "{level}");
        assert_eq!(list.name, names[i]);
        assert_eq!(list.guide.0, "fighter#massive-attacks");
        assert!(list.ignores_prerequisites && !list.every_feat);
        let ranger = format!("ranger/combat-style-feat/two-weapon-combat-feats-{level}");
        assert_eq!(&list_feats(&ranger) - &extras, two_weapon[i], "{ranger}");
        let archery = format!("ranger/combat-style-feat/archery-feats-{level}");
        assert_eq!(
            &list_feats(&archery) - &extras,
            list_feats(STORMWALKER_ARCHERY[i]),
            "{archery}"
        );
        assert!(any_feat_list(STORMWALKER_ARCHERY[i]).ignores_prerequisites);
    }
    assert_eq!(
        archetype_feat_lists("fighter", TITAN),
        feat_set(&MASSIVE_ATTACKS)
    );

    let precise = any_feat_list(PRECISE_COMPANION);
    assert_eq!(
        list_feats(PRECISE_COMPANION),
        feat_set(&["outflank", "precise-shot"])
    );
    assert_eq!(precise.name, "Precise Companion");
    assert!(precise.ignores_prerequisites);

    let captor_6 = feat_set(&["bull-rush", "dirty-trick", "disarm", "trip"]);
    let captor_12 = &captor_6
        | &feat_set(&[
            "combat-reflexes",
            "greater-bull-rush",
            "greater-dirty-trick",
            "greater-disarm",
            "greater-trip",
            "lunge",
        ]);
    assert_eq!(list_feats(CAPTOR[0]), captor_6);
    assert_eq!(list_feats(CAPTOR[1]), captor_12);
    assert_eq!(any_feat_list(CAPTOR[0]).name, "Captor");
    assert_eq!(any_feat_list(CAPTOR[1]).name, "Captor (from level 12)");
    assert!(
        CAPTOR
            .iter()
            .all(|key| any_feat_list(key).ignores_prerequisites)
    );
}

/// The keys of the feat lists `archetype` defines.
fn archetype_feat_lists(class_key: &str, archetype: &str) -> BTreeSet<String> {
    class(class_key)
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == archetype)
        .unwrap_or_else(|| panic!("no {archetype}"))
        .feat_lists
        .iter()
        .map(|l| l.key.to_string())
        .collect()
}

/// A Titan Fighter's Massive Attacks slots offer exactly their list, open whatever the feats' prerequisites (Two-Weapon
/// Fighting needs Dexterity 15, Greater Two-Weapon Fighting Improved Two-Weapon Fighting), and a filled Titan 18 is
/// possible.
#[test]
fn a_titan_fighter_picks_massive_attacks_from_the_two_weapon_style_feats_ignoring_prerequisites() {
    let titan = archetype_build("fighter", TITAN, 18);
    for (level, list) in [
        (2, MASSIVE_ATTACKS[0]),
        (6, MASSIVE_ATTACKS[1]),
        (10, MASSIVE_ATTACKS[2]),
        (14, MASSIVE_ATTACKS[2]),
        (18, MASSIVE_ATTACKS[2]),
    ] {
        assert_eq!(
            feats_offered(&titan, &class_slot(level)),
            list_feats(list),
            "{level}"
        );
        for feat in list_feats(list) {
            assert_eq!(
                feat_status(&titan, &class_slot(level), &feat),
                Ok(()),
                "{feat} at {level}"
            );
        }
    }
    let mut filled = titan.clone();
    for (level, feat) in [
        (2, "two-weapon-fighting"),
        (6, "double-slice"),
        (10, "greater-two-weapon-fighting"),
        (14, "improved-two-weapon-fighting"),
        (18, "shield-bash"),
    ] {
        filled
            .picks
            .insert(class_slot(level), feat_pick_plain(feat));
    }
    assert!(possible(&filled));
    // Greater Two-Weapon Fighting isn't on the list before 10.
    let mut early = titan.clone();
    early.picks.insert(
        class_slot(6),
        feat_pick_plain("greater-two-weapon-fighting"),
    );
    assert!(!possible(&early));
}

/// A feat held is not offered again unless it repeats: Two-Weapon Fighting taken at 2 is "Already taken" at 6, and
/// Precise Shot taken through Precise Companion is in a hunter's teamwork slot at 3. Improved Critical repeats once
/// per weapon, so a Stormwalker takes it at 10 and again for another weapon at 14.
#[test]
fn a_bonus_feat_choice_offers_a_held_feat_again_only_when_it_repeats() {
    let mut titan = archetype_build("fighter", TITAN, 6);
    titan
        .picks
        .insert(class_slot(2), feat_pick_plain("two-weapon-fighting"));
    assert!(possible(&titan));
    assert_eq!(
        feat_status(&titan, &class_slot(6), "two-weapon-fighting"),
        Err(Issue::AlreadyHeld)
    );
    titan
        .picks
        .insert(class_slot(6), feat_pick_plain("two-weapon-fighting"));
    assert_eq!(why_not(&titan, &class_slot(6)), Some(Issue::AlreadyHeld));

    let mut hunter = build("hunter", 3);
    hunter
        .picks
        .insert(class_slot(2), feat_pick_plain("precise-shot"));
    assert!(possible(&hunter));
    assert_eq!(
        feat_status(&hunter, &class_slot(3), "precise-shot"),
        Err(Issue::AlreadyHeld)
    );

    let mut stormwalker = archetype_build("ranger", STORMWALKER, 14);
    stormwalker.picks.insert(
        class_slot(10),
        feat_pick("improved-critical", weapon("long-bow")),
    );
    stormwalker.picks.insert(
        class_slot(14),
        feat_pick("improved-critical", weapon("longsword")),
    );
    assert!(possible(&stormwalker));
    stormwalker.picks.insert(
        class_slot(14),
        feat_pick("improved-critical", weapon("long-bow")),
    );
    assert_eq!(
        why_not(&stormwalker, &class_slot(14)),
        Some(Issue::AlreadyHeld)
    );
}

/// The hunter's, the Urban Hunter's and the Stormwalker's slots offer exactly their lists, prerequisites ignored as
/// their sections say ("She does not need to meet the prerequisites", "Neither she nor her companion need to meet the
/// prerequisites", the ranger's style feats), and the Forester has no Precise Companion.
#[test]
fn the_hunters_and_the_stormwalkers_choices_offer_their_lists_ignoring_prerequisites() {
    let cases: [(Build, &[(u8, &str)]); 3] = [
        (build("hunter", 2), &[(2, PRECISE_COMPANION)]),
        (
            archetype_build("hunter", URBAN_HUNTER, 18),
            &[
                (6, CAPTOR[0]),
                (9, CAPTOR[0]),
                (12, CAPTOR[1]),
                (15, CAPTOR[1]),
                (18, CAPTOR[1]),
            ],
        ),
        (
            archetype_build("ranger", STORMWALKER, 18),
            &[
                (2, STORMWALKER_ARCHERY[0]),
                (6, STORMWALKER_ARCHERY[1]),
                (10, STORMWALKER_ARCHERY[2]),
                (14, STORMWALKER_ARCHERY[2]),
                (18, STORMWALKER_ARCHERY[2]),
            ],
        ),
    ];
    for (build, slots) in cases {
        for (level, list) in slots {
            assert_eq!(
                feats_offered(&build, &class_slot(*level)),
                list_feats(list),
                "{list} at {level}"
            );
            for option in engine().options(&build, &class_slot(*level)).unwrap() {
                assert_eq!(
                    option.status,
                    Ok(()),
                    "{list} at {level}: {:?}",
                    option.pick
                );
            }
        }
    }
    let forester = archetype_build("hunter", "hunter/forester", 2);
    assert!(
        !feats_offered(&forester, &class_slot(2)).contains("outflank"),
        "the Forester's level-2 slot is a combat feat"
    );
}

/// The sheet labels each new slot by its list's name.
#[test]
fn the_bonus_feat_slots_read_their_lists_names() {
    let engine = engine();
    for (build, labels) in [
        (
            archetype_build("fighter", TITAN, 18),
            vec![
                (2, "Massive Attacks"),
                (6, "Massive Attacks (from level 6)"),
                (10, "Massive Attacks (from level 10)"),
                (18, "Massive Attacks (from level 10)"),
                (4, "Combat feat"),
            ],
        ),
        (build("hunter", 2), vec![(2, "Precise Companion")]),
        (
            archetype_build("hunter", URBAN_HUNTER, 12),
            vec![(6, "Captor"), (12, "Captor (from level 12)")],
        ),
        (
            archetype_build("ranger", STORMWALKER, 10),
            vec![
                (2, "Archery Feats"),
                (6, "Archery Feats (from level 6)"),
                (10, "Archery Feats (from level 10)"),
            ],
        ),
    ] {
        let sheet = engine.evaluate(&build);
        for (level, label) in labels {
            assert_eq!(
                engine.slot_label(sheet.slot(&class_slot(level)).expect("a class slot")),
                label,
                "{level}"
            );
        }
    }
}

// ---- Witch Wandering Hex and Beneficial Curse give their pick (ticket 62) -----------------------------------------

const WITCH_WANDERING_HEX: &str = "witch-wandering-hex";
const BENEFICIAL_CURSE: &str = "beneficial-curse";
const CURSES: &str = "oracle/oracles-curse";

fn mythic_ability_data(key: &str) -> &'static rules::MythicAbility {
    DATA.mythic_abilities
        .iter()
        .find(|ability| ability.key.as_str() == key)
        .unwrap_or_else(|| panic!("no mythic ability {key}"))
}

/// `build` with `ability` as MR 1's general pick (MR 1 is at level 6).
fn with_mr_1_ability(mut build: Build, ability: &str) -> Build {
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::MythicAbility(rules::MythicAbilityKey::new(ability)),
    );
    build
}

/// The slot MR 1's mythic ability grants.
fn mr_1_selection() -> SlotId {
    SlotId::MythicSelection { rank: 1, index: 0 }
}

/// What MR 1's mythic ability picker says of `ability`.
fn mr_1_status(build: &Build, ability: &str) -> Result<(), Issue> {
    slot_options(build, &SlotId::Mythic { rank: 1 })
        .into_iter()
        .find(|option| option.pick.key() == ability)
        .unwrap_or_else(|| panic!("MR 1 doesn't offer {ability}"))
        .status
}

/// "Witch Wandering Hex. Prequisites: Hex. You can select any hex for which you meet the prerequisites." and
/// "Beneficial Curse. Prerequisites: Oracle's Curse. You gain an additional Curse with no penalties."
/// (mythic-paths): one slot of the witch's hexes, needing a hex held as Extra Hex does ("Prerequisites: Hex"), and one
/// of the oracle's curses, needing a curse held.
#[test]
fn witch_wandering_hex_and_beneficial_curse_give_a_slot_of_their_selection() {
    let hex = mythic_ability_data(WITCH_WANDERING_HEX);
    assert_eq!(
        hex.selections,
        [rules::FeatureSelectionKey::new("witch/hex")]
    );
    let mut needs = hex.prerequisite.clone().expect("a prerequisite");
    needs.canonicalize();
    let mut extra_hex = feat("extra-hex").prerequisite.clone().unwrap();
    extra_hex.canonicalize();
    assert_eq!(needs, extra_hex);

    let curse = mythic_ability_data(BENEFICIAL_CURSE);
    assert_eq!(curse.selections, [rules::FeatureSelectionKey::new(CURSES)]);
    let held: BTreeSet<String> = match curse.prerequisite.as_ref() {
        Some(Prerequisite::Any(children)) => children
            .iter()
            .map(|child| match child {
                Prerequisite::SelectionOption(option) => option.to_string(),
                other => panic!("{other:?}"),
            })
            .collect(),
        other => panic!("{other:?}"),
    };
    let curses: BTreeSet<String> = class("oracle")
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == CURSES)
        .unwrap()
        .options
        .iter()
        .map(|option| option.key.to_string())
        .collect();
    assert_eq!(held, curses);
    assert_eq!(curses.len(), 9);

    // The mythic abilities that give a pick: these two and ticket 55's five.
    let giving: BTreeSet<&str> = DATA
        .mythic_abilities
        .iter()
        .filter(|ability| !ability.selections.is_empty())
        .map(|ability| ability.key.as_str())
        .collect();
    assert_eq!(
        giving,
        BTreeSet::from([
            BENEFICIAL_CURSE,
            "impossible-domain",
            "second-bloodline",
            "second-bloodrager-bloodline",
            "second-mystery",
            "second-spirit",
            WITCH_WANDERING_HEX,
        ])
    );
}

#[test]
fn witch_wandering_hex_is_locked_without_a_hex() {
    let fighter = build("fighter", 6);
    assert!(matches!(
        mr_1_status(&fighter, WITCH_WANDERING_HEX),
        Err(Issue::Unmet(_))
    ));
    assert!(matches!(
        why_not(
            &with_mr_1_ability(fighter, WITCH_WANDERING_HEX),
            &SlotId::Mythic { rank: 1 }
        ),
        Some(Issue::Unmet(_))
    ));
    // A witch's hex or a shaman's opens it.
    assert_eq!(
        mr_1_status(
            &with_selections_filled(build("witch", 6)),
            WITCH_WANDERING_HEX
        ),
        Ok(())
    );
    let shaman = with_selections_filled(build("shaman", 6));
    assert!(
        at(&shaman, 6)
            .selection_options
            .keys()
            .any(|option| option.as_str().starts_with("shaman/hex/"))
    );
    assert_eq!(mr_1_status(&shaman, WITCH_WANDERING_HEX), Ok(()));
}

#[test]
fn witch_wandering_hex_gives_a_witch_hex_slot_offering_hexes_whose_prerequisites_are_met() {
    let engine = engine();
    let witch = with_mr_1_ability(
        with_selections_filled(build("witch", 6)),
        WITCH_WANDERING_HEX,
    );
    assert!(possible(&witch));
    let sheet = engine.evaluate(&witch);
    let slot = sheet.slot(&mr_1_selection()).expect("the hex slot");
    assert_eq!(slot.kind, selection_kind("witch/hex"));
    assert_eq!(engine.slot_label(slot), "Hex");
    // The witch's hexes, each open as the witch's own slot at 6 would leave it, but the ones held.
    let hexes = offered_in(&witch, &mr_1_selection());
    let held: BTreeSet<String> = witch
        .picks
        .values()
        .map(|pick| pick.key().to_owned())
        .filter(|key| key.starts_with("witch/hex/"))
        .collect();
    assert_eq!(held.len(), 4);
    for hex in &held {
        assert_eq!(hexes[hex], Err(Issue::AlreadyHeld), "{hex}");
    }
    let free = first_open_in_mr_1_selection(&witch);
    assert!(free.starts_with("witch/hex/") && !held.contains(&free));
    assert!(matches!(hexes["witch/hex/agony"], Err(Issue::Unmet(_))));
    // At MR 5 (level 14) a witch has her major hexes (from 10).
    let mut witch_14 = with_selections_filled(build("witch", 14));
    witch_14.mythic_path = Some(MythicPathKey::new("trickster"));
    witch_14.picks.insert(
        SlotId::Mythic { rank: 5 },
        Pick::MythicAbility(rules::MythicAbilityKey::new(WITCH_WANDERING_HEX)),
    );
    let rank_5 = SlotId::MythicSelection { rank: 5, index: 0 };
    let major = offered_in(&witch_14, &rank_5);
    assert!(
        !witch_14
            .picks
            .values()
            .any(|pick| pick.key() == "witch/hex/major-healing")
    );
    assert_eq!(major["witch/hex/major-healing"], Ok(()));
    assert!(matches!(
        hexes["witch/hex/major-healing"],
        Err(Issue::Unmet(_))
    ));

    // A pick there is held from MR 1's level; a hex not open there is refused.
    let mut third = witch.clone();
    third.picks.insert(mr_1_selection(), pick(&free));
    assert!(possible(&third));
    assert_eq!(
        at(&third, 6).option_count(&SelectionOptionKey::new(&free)),
        1
    );
    let mut agony = witch;
    agony
        .picks
        .insert(mr_1_selection(), pick("witch/hex/agony"));
    assert!(matches!(
        why_not(&agony, &mr_1_selection()),
        Some(Issue::Unmet(_))
    ));

    // A shaman's slot is a witch hex slot too; a witch hex the shaman took through her own hexes is held.
    let mut shaman = with_selections_filled(build("shaman", 6));
    shaman.picks.insert(
        SlotId::Selection { level: 2, index: 0 },
        pick("witch/hex/evil-eye"),
    );
    let shaman = with_mr_1_ability(shaman, WITCH_WANDERING_HEX);
    assert!(possible(&shaman));
    assert_eq!(
        kind_at(&shaman, &mr_1_selection()),
        Some(selection_kind("witch/hex"))
    );
    let hexes = offered_in(&shaman, &mr_1_selection());
    assert_eq!(hexes["witch/hex/evil-eye"], Err(Issue::AlreadyHeld));
    assert_eq!(hexes["witch/hex/cackle"], Ok(()));
}

/// The first option, in key order, open in MR 1's selection slot.
fn first_open_in_mr_1_selection(build: &Build) -> String {
    offered_in(build, &mr_1_selection())
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .min()
        .expect("an open option")
}

#[test]
fn beneficial_curse_gives_an_oracle_curse_slot_offering_only_curses_not_held() {
    let engine = engine();
    let oracle = with_selections_filled(build("oracle", 6));
    assert_eq!(mr_1_status(&oracle, BENEFICIAL_CURSE), Ok(()));
    assert!(matches!(
        mr_1_status(&build("fighter", 6), BENEFICIAL_CURSE),
        Err(Issue::Unmet(_))
    ));
    let oracle = with_mr_1_ability(oracle, BENEFICIAL_CURSE);
    assert!(possible(&oracle));
    let sheet = engine.evaluate(&oracle);
    let slot = sheet.slot(&mr_1_selection()).expect("the curse slot");
    assert_eq!(slot.kind, selection_kind(CURSES));
    assert_eq!(engine.slot_label(slot), "Oracle's Curse");
    let held = oracle
        .picks
        .values()
        .map(|pick| pick.key().to_owned())
        .find(|key| key.starts_with("oracle/oracles-curse/"))
        .expect("a curse held");
    let curses = offered_in(&oracle, &mr_1_selection());
    assert_eq!(curses.len(), 9);
    for (curse, status) in &curses {
        if *curse == held {
            assert_eq!(*status, Err(Issue::AlreadyHeld));
        } else {
            assert_eq!(*status, Ok(()), "{curse}");
        }
    }
    let other = curses.keys().find(|curse| **curse != held).unwrap().clone();
    let mut second = oracle.clone();
    second.picks.insert(mr_1_selection(), pick(&other));
    assert!(possible(&second));
    assert_eq!(
        at(&second, 6).option_count(&SelectionOptionKey::new(&other)),
        1
    );
    let mut twice = oracle;
    twice.picks.insert(mr_1_selection(), pick(&held));
    assert_eq!(why_not(&twice, &mr_1_selection()), Some(Issue::AlreadyHeld));
}

#[test]
fn removing_witch_wandering_hex_or_beneficial_curse_is_refused_while_its_slot_holds_a_pick() {
    let engine = engine();
    for (class, ability) in [("witch", WITCH_WANDERING_HEX), ("oracle", BENEFICIAL_CURSE)] {
        let mut build = with_mr_1_ability(with_selections_filled(build(class, 6)), ability);
        let option = first_open_in_mr_1_selection(&build);
        build.picks.insert(mr_1_selection(), pick(&option));
        assert!(possible(&build), "{ability}");
        let choice = rules::Choice::Slot(SlotId::Mythic { rank: 1 });
        assert!(!engine.is_removable(&build, &choice), "{ability}");
        assert!(!engine.removable(&build).contains(&choice));
        assert!(engine.is_removable(&build, &rules::Choice::Slot(mr_1_selection())));

        build.apply(&rules::Edit::Clear(mr_1_selection()));
        assert!(engine.is_removable(&build, &choice), "{ability}");
        build.apply(&rules::Edit::Clear(SlotId::Mythic { rank: 1 }));
        assert!(possible(&build));
        assert_eq!(kind_at(&build, &mr_1_selection()), None);
    }
}

// ---- ticket 63: an archetype's alignment requirement replaces its class's ---------------------------------------

/// Every archetype with an alignment rule, with what the rule allows, and a race it needs besides (the Reformed
/// Fiend's tiefling). The Mantis Zealot's is the user's fact; the others are the guide's.
const ALIGNMENT_RULES: [(&str, &[rules::Alignment], Option<&str>); 7] = {
    use rules::Alignment::*;
    [
        (
            "bloodrager/reformed-fiend",
            &[LawfulGood, NeutralGood, ChaoticGood],
            Some("tiefling"),
        ),
        (
            "cleric/angelfire-apostle",
            &[LawfulGood, NeutralGood, ChaoticGood],
            None,
        ),
        (
            "cleric/priest-of-balance",
            &[LawfulNeutral, TrueNeutral, ChaoticNeutral],
            None,
        ),
        (
            "monk/drunken-master",
            &[
                NeutralGood,
                ChaoticGood,
                TrueNeutral,
                ChaoticNeutral,
                NeutralEvil,
                ChaoticEvil,
            ],
            None,
        ),
        (
            "shaman/prophet-of-pestilence",
            &[TrueNeutral, NeutralEvil, ChaoticEvil],
            None,
        ),
        (
            "slayer/bloodseeker",
            &[LawfulEvil, NeutralEvil, ChaoticEvil],
            None,
        ),
        (
            "warpriest/mantis-zealot",
            &[LawfulNeutral, LawfulEvil, NeutralEvil],
            None,
        ),
    ]
};

/// A Build of `levels` levels of `archetype`'s class taken with it, at `alignment`, with no picks.
fn aligned(archetype: &str, alignment: rules::Alignment, levels: usize) -> Build {
    let class = archetype.split('/').next().unwrap();
    let mut build = archetype_build(class, archetype, levels);
    build.start.alignment = Some(alignment);
    build
}

/// The level-1 class picker's status for `archetype` (a class key alone: the class without one) in a Build like
/// `build`, before its first level.
fn entry_status(build: &Build, archetype: &str) -> Result<(), Issue> {
    let (class, archetype) = match archetype.split_once('/') {
        Some((class, _)) => (class, Some(ArchetypeKey::new(archetype))),
        None => (archetype, None),
    };
    let mut before = build.clone();
    before.levels.clear();
    before.class_choices.clear();
    engine()
        .class_options(&before, 1)
        .unwrap()
        .into_iter()
        .find(|option| option.class == ClassKey::new(class) && option.archetype == archetype)
        .unwrap_or_else(|| panic!("no class option {class} {archetype:?}"))
        .status
}

/// Whether an `Alignment` appears anywhere in a prerequisite.
fn names_alignment(prerequisite: &Prerequisite) -> bool {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => children.iter().any(names_alignment),
        Prerequisite::Not(child) => names_alignment(child),
        Prerequisite::Alignment(_) => true,
        _ => false,
    }
}

/// What an `Any` of alignments reads as when none holds.
fn none_of(alignments: &[rules::Alignment]) -> Issue {
    Issue::Unmet(Unmet::Any(
        alignments
            .iter()
            .map(|alignment| Unmet::Condition {
                needs: Prerequisite::Alignment(*alignment),
                has: None,
            })
            .collect(),
    ))
}

/// The ticket's: a Chaotic Neutral monk 1 can take Drunken Master, and so can a monk 20 of any non-lawful alignment.
#[test]
fn a_chaotic_neutral_monk_1_can_take_drunken_master() {
    let build = aligned("monk/drunken-master", rules::Alignment::ChaoticNeutral, 1);
    assert!(possible(&build));
    assert_eq!(entry_status(&build, "monk/drunken-master"), Ok(()));
    let filled = with_selections_filled(aligned(
        "monk/drunken-master",
        rules::Alignment::NeutralEvil,
        20,
    ));
    assert!(possible(&filled));
}

/// The ticket's: a Lawful Neutral one can't, and the reason is the archetype's alignment, not the monk's.
#[test]
fn a_lawful_neutral_drunken_master_is_locked_by_the_archetypes_alignment() {
    use rules::Alignment::*;
    let build = aligned("monk/drunken-master", LawfulNeutral, 1);
    assert!(!possible(&build));
    let status = entry_status(&build, "monk/drunken-master");
    let not_lawful = [
        NeutralGood,
        ChaoticGood,
        TrueNeutral,
        ChaoticNeutral,
        NeutralEvil,
        ChaoticEvil,
    ];
    assert_eq!(status, Err(none_of(&not_lawful)));
    assert_eq!(
        engine().issue_text(&status.unwrap_err()).unwrap(),
        "Needs Neutral Good alignment or Chaotic Good alignment or True Neutral alignment or Chaotic Neutral \
         alignment or Neutral Evil alignment or Chaotic Evil alignment"
    );
}

/// The ticket's: a monk without an archetype, or with one stating no alignment, still needs a lawful one.
#[test]
fn a_plain_monk_still_needs_a_lawful_alignment() {
    use rules::Alignment::*;
    let lawful = [LawfulGood, LawfulNeutral, LawfulEvil];
    for alignment in rules::Alignment::ALL {
        let mut plain = monk(1);
        plain.start.alignment = Some(alignment);
        let zen = aligned("monk/zen-archer", alignment, 1);
        if lawful.contains(&alignment) {
            assert!(possible(&plain) && possible(&zen), "{alignment:?}");
            assert_eq!(entry_status(&plain, "monk"), Ok(()));
        } else {
            assert!(!possible(&plain) && !possible(&zen), "{alignment:?}");
            assert_eq!(entry_status(&plain, "monk"), Err(none_of(&lawful)));
            assert_eq!(entry_status(&zen, "monk/zen-archer"), Err(none_of(&lawful)));
        }
    }
}

/// The ticket's: a Bloodseeker ("Alignment must be evil") needs an evil alignment; the slayer has no rule of its own.
#[test]
fn a_bloodseeker_needs_an_evil_alignment() {
    use rules::Alignment::*;
    let evil = [LawfulEvil, NeutralEvil, ChaoticEvil];
    for alignment in rules::Alignment::ALL {
        let bloodseeker = aligned("slayer/bloodseeker", alignment, 1);
        let mut slayer = build("slayer", 1);
        slayer.start.alignment = Some(alignment);
        assert!(possible(&slayer), "{alignment:?}");
        if evil.contains(&alignment) {
            assert!(possible(&bloodseeker), "{alignment:?}");
            assert_eq!(entry_status(&bloodseeker, "slayer/bloodseeker"), Ok(()));
        } else {
            assert!(!possible(&bloodseeker), "{alignment:?}");
            assert_eq!(
                entry_status(&bloodseeker, "slayer/bloodseeker"),
                Err(none_of(&evil))
            );
        }
    }
}

/// The ticket's, the user's fact: a Mantis Zealot is Lawful Evil, Lawful Neutral or Neutral Evil (one step from
/// Achaekek), so open at those and locked at the rest (Chaotic Evil and Lawful Good among them).
#[test]
fn a_mantis_zealot_is_open_at_le_ln_and_ne_and_locked_at_ce_and_lg() {
    use rules::Alignment::*;
    let allowed = [LawfulNeutral, LawfulEvil, NeutralEvil];
    let zealot = class("warpriest")
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == "warpriest/mantis-zealot")
        .unwrap();
    assert_eq!(
        zealot.prerequisite,
        Some(Prerequisite::Any(
            allowed.map(Prerequisite::Alignment).to_vec()
        ))
    );
    for alignment in rules::Alignment::ALL {
        let build = aligned("warpriest/mantis-zealot", alignment, 1);
        let status = entry_status(&build, "warpriest/mantis-zealot");
        if allowed.contains(&alignment) {
            assert!(possible(&build), "{alignment:?}");
            assert_eq!(status, Ok(()), "{alignment:?}");
        } else {
            assert!(!possible(&build), "{alignment:?}");
            assert_eq!(status, Err(none_of(&allowed)), "{alignment:?}");
        }
    }
    for alignment in [ChaoticEvil, LawfulGood] {
        assert!(!possible(&aligned("warpriest/mantis-zealot", alignment, 1)));
    }
}

/// The ticket's: the archetypes with an alignment rule are these seven; only the Drunken Master's clashes with its
/// class's (the monk's Lawful), and each, the Drunken Master too, is open exactly at the alignments its own rule
/// allows and buildable to 20 there.
#[test]
fn every_archetype_with_an_alignment_rule_is_buildable_at_an_alignment_it_allows() {
    let with_rule: BTreeSet<&str> = DATA
        .classes
        .iter()
        .flat_map(|class| &class.archetypes)
        .filter(|archetype| archetype.prerequisite.as_ref().is_some_and(names_alignment))
        .map(|archetype| archetype.key.as_str())
        .collect();
    let expected: BTreeSet<&str> = ALIGNMENT_RULES.iter().map(|(key, ..)| *key).collect();
    assert_eq!(with_rule, expected);

    let mut clashing = Vec::new();
    for (archetype, allowed, race) in ALIGNMENT_RULES {
        let class_key = archetype.split('/').next().unwrap();
        // The alignments the class alone allows.
        let class_allows: Vec<rules::Alignment> = rules::Alignment::ALL
            .into_iter()
            .filter(|alignment| {
                let state = CharacterState {
                    alignment: Some(*alignment),
                    ..CharacterState::default()
                };
                class(class_key)
                    .prerequisite
                    .as_ref()
                    .is_none_or(|p| state.check(p).is_ok())
            })
            .collect();
        if !allowed.iter().any(|a| class_allows.contains(a)) {
            clashing.push(archetype);
        }
        for alignment in rules::Alignment::ALL {
            let mut build = aligned(archetype, alignment, 1);
            build.start.race = race.map(rules::RaceKey::new);
            let open = entry_status(&build, archetype).is_ok();
            assert_eq!(
                open,
                allowed.contains(&alignment),
                "{archetype} {alignment:?}"
            );
            assert_eq!(possible(&build), open, "{archetype} {alignment:?}");
        }
        let mut full = aligned(archetype, allowed[0], 20);
        full.start.race = race.map(rules::RaceKey::new);
        assert!(possible(&full), "{archetype}");
        assert!(
            possible(&with_selections_filled(full)),
            "{archetype} filled"
        );
    }
    assert_eq!(clashing, ["monk/drunken-master"]);
}

// ---- reasons for negated prerequisites (ticket 65) ----------------------------------------------------------------

/// Ticket 59's decision 5: an infusion the Kinetic Sharpshooter "cannot use" reads "Not for the Kinetic Sharpshooter
/// archetype", alone or ahead of what else it needs; the base class's reason for one of the Sharpshooter's own is
/// unchanged.
#[test]
fn a_kinetic_sharpshooters_closed_infusion_reads_not_for_its_archetype() {
    let sharpshooter = with_option(
        archetype_build("kineticist", SHARPSHOOTER, 3),
        1,
        0,
        "kineticist/element/fire",
    );
    for name in ["extended-range", "kinetic-blade"] {
        assert_eq!(
            infusion_reason(&sharpshooter, 3, name),
            "Not for the Kinetic Sharpshooter archetype",
            "{name}"
        );
    }
    assert_eq!(
        infusion_reason(&sharpshooter, 3, "eruption"),
        "Not for the Kinetic Sharpshooter archetype; needs Kineticist level 6 (have 3), Extended Range"
    );
    assert_eq!(
        infusion_reason(&kineticist("fire", 3), 3, "exploding-arrows"),
        "Needs the Kinetic Sharpshooter archetype"
    );
}

/// Every negated prerequisite in the data reads as what rules the option out, never "Needs not …": the four kinds
/// the data negates (an archetype, an option held, a class level, the slot picked through), each in a real reason.
#[test]
fn every_negated_prerequisite_in_the_data_reads_as_what_rules_the_option_out() {
    fn negations<'a>(prerequisite: &'a Prerequisite, out: &mut Vec<&'a Prerequisite>) {
        match prerequisite {
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                children.iter().for_each(|child| negations(child, out));
            }
            Prerequisite::Not(child) => out.push(child),
            _ => {}
        }
    }
    let engine = engine();
    let mut held = Vec::new();
    for prerequisite in data_prerequisites(&DATA) {
        negations(prerequisite, &mut held);
    }
    let mut kinds = BTreeSet::new();
    for child in held {
        let text = engine
            .issue_text(&Issue::Unmet(Unmet::Condition {
                needs: Prerequisite::Not(Box::new(child.clone())),
                has: None,
            }))
            .unwrap();
        let (kind, lead) = match child {
            Prerequisite::Archetype(_) => ("archetype", "Not for the "),
            Prerequisite::SelectionOption(_) => ("option", "Not with "),
            Prerequisite::ClassLevel { .. } => ("class level", "Only below "),
            Prerequisite::PickedThrough(_) => ("picked through", "Not as "),
            other => panic!("a new negated kind: {other:?} reads {text:?}"),
        };
        assert!(text.starts_with(lead), "{child:?} reads {text:?}");
        kinds.insert(kind);
    }
    assert_eq!(
        kinds,
        BTreeSet::from(["archetype", "class level", "option", "picked through"])
    );

    // A slayer's 2nd-level combat style talent at slayer 6.
    assert_eq!(
        lock_reason(
            &build("slayer", 6),
            6,
            "slayer/slayer-talents/combat-style-feat-2"
        ),
        "Only below Slayer level 6"
    );
    // A second totem line, ahead of what else it needs: Beast Totem for a barbarian of the Celestial line.
    let mut totems = barbarian(6);
    totems.picks.insert(
        selection_slot(2),
        pick("barbarian/rage-power/celestial-totem-lesser"),
    );
    totems.picks.insert(
        selection_slot(4),
        pick("barbarian/rage-power/celestial-totem"),
    );
    assert!(possible(&totems));
    assert_eq!(
        lock_reason(&totems, 6, "barbarian/rage-power/beast-totem"),
        "Not with Celestial Totem or Celestial Totem, Lesser; needs Beast Totem, Lesser"
    );
    // Cinder Dance with the Lame curse.
    let lame = with_option(oracle("flame", 1), 1, 1, "oracle/oracles-curse/lame");
    let lame = with_option(lame, 1, 2, "oracle/revelations/flame/cinder-dance");
    assert_eq!(
        why_not(&lame, &SlotId::Selection { level: 1, index: 2 })
            .and_then(|issue| engine.issue_text(&issue))
            .as_deref(),
        Some("Not with Lame")
    );
    // A Lawful Good Champion of the Faith's evil blessing as his chosen alignment, and a Lawful Good Separatist's
    // Chaos Domain as his own: the alignments that case allows.
    let mut champion = archetype_build("warpriest", "warpriest/champion-of-the-faith", 1);
    champion.start.alignment = Some(rules::Alignment::LawfulGood);
    let chosen = slot_of(
        &champion,
        1,
        "warpriest/champion-of-the-faith/chosen-alignment",
    );
    champion
        .picks
        .insert(chosen.clone(), pick("warpriest/blessings/evil-domain"));
    assert_eq!(
        why_not(&champion, &chosen)
            .and_then(|issue| engine.issue_text(&issue))
            .as_deref(),
        Some(
            "As Chosen Alignment Blessing, needs Lawful Evil alignment or Neutral Evil alignment or Chaotic Evil \
             alignment"
        )
    );
    let mut separatist = separatist(3);
    separatist.start.alignment = Some(rules::Alignment::LawfulGood);
    let own = slot_of(&separatist, 1, "cleric/separatist/domain");
    separatist
        .picks
        .insert(own.clone(), pick("cleric/domain/chaos"));
    assert_eq!(
        why_not(&separatist, &own)
            .and_then(|issue| engine.issue_text(&issue))
            .as_deref(),
        Some(
            "As Domain (Separatist), needs Neutral Good alignment or Chaotic Good alignment or True Neutral \
             alignment or Chaotic Neutral alignment or Neutral Evil alignment or Chaotic Evil alignment"
        )
    );
}

// ---- the Instinctual Warrior's monk rage powers (ticket 66) -------------------------------------------------------

const RAGE_POWER: &str = "barbarian/rage-power";
const INSTINCTUAL_WARRIOR: &str = "barbarian/instinctual-warrior";

/// The guide's monk abilities and ki powers an Instinctual Warrior may take as rage powers
/// (`barbarian#rage-power-instinctual-warrior`): each one's key in the rage power selection, its name, and the
/// barbarian level it needs (`None`: Deflect Arrows, which needs Improved Unarmed Strike instead).
const MONK_RAGE_POWERS: [(&str, &str, Option<u8>); 5] = [
    (
        "barbarian/rage-power/deflect-arrows-instinctual-warrior",
        "Deflect Arrows (Instinctual Warrior)",
        None,
    ),
    (
        "barbarian/rage-power/purity-of-body-instinctual-warrior",
        "Purity of Body (Instinctual Warrior)",
        Some(8),
    ),
    (
        "barbarian/rage-power/diamond-body-instinctual-warrior",
        "Diamond Body (Instinctual Warrior)",
        Some(12),
    ),
    (
        "barbarian/rage-power/diamond-soul-instinctual-warrior",
        "Diamond Soul (Instinctual Warrior)",
        Some(16),
    ),
    (
        "barbarian/rage-power/perfect-self-instinctual-warrior",
        "Perfect Self (Instinctual Warrior)",
        Some(20),
    ),
];

/// The Instinctual Warrior's rage power levels, by its table.
const INSTINCTUAL_RAGE_POWER_LEVELS: [u8; 7] = [4, 8, 10, 12, 16, 18, 20];

/// A chaotic neutral Instinctual Warrior of `levels` levels, with Improved Unarmed Strike at 1 when `unarmed`.
fn instinctual_warrior(levels: usize, unarmed: bool) -> Build {
    let mut build = chaotic(archetype_build("barbarian", INSTINCTUAL_WARRIOR, levels));
    if unarmed {
        build
            .picks
            .insert(general_slot(1), feat_pick_plain("improved-unarmed-strike"));
    }
    build
}

/// Whether a monk power needing barbarian level `needs` is open in a slot at `level`: Deflect Arrows (`None`) from its
/// first rage power, the others from the level they need.
fn opens_at(needs: Option<u8>, level: u8) -> bool {
    needs.is_none_or(|needs| needs <= level)
}

/// "Besides the normal Barbarian rage powers, an Instinctual Warrior can select a number of Monk abilities and Ki
/// powers": five options of the barbarian's rage power selection, each needing the archetype and the level (or, for
/// Deflect Arrows, the feat) the guide gives. Extra Rage Power's "Rage Power" is met by holding any of them too.
#[test]
fn the_instinctual_warriors_monk_powers_are_rage_power_options_for_it_alone() {
    let archetype = Prerequisite::Archetype(ArchetypeKey::new(INSTINCTUAL_WARRIOR));
    for (key, name, level) in MONK_RAGE_POWERS {
        let option = selection_option(key);
        assert_eq!(option.name, name);
        assert_eq!(option.guide.0, "barbarian#rage-power-instinctual-warrior");
        assert_eq!(option.repeat, Repeat::Once, "{name}");
        let needs = match level {
            Some(level) => Prerequisite::ClassLevel {
                class: ClassKey::new("barbarian"),
                level,
            },
            None => Prerequisite::Feat(FeatKey::new("improved-unarmed-strike")),
        };
        let mut expected = Prerequisite::All(vec![archetype.clone(), needs]);
        expected.canonicalize();
        assert_eq!(option.prerequisite, Some(expected), "{name}");
    }
    let selection = class("barbarian")
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == RAGE_POWER)
        .expect("the rage power selection");
    assert_eq!(selection.options.len(), 38);
    let Some(Prerequisite::Any(children)) = &feat("extra-rage-power").prerequisite else {
        panic!("Extra Rage Power's prerequisite isn't an Any");
    };
    let named: BTreeSet<String> = children
        .iter()
        .map(|child| match child {
            Prerequisite::SelectionOption(key) => key.to_string(),
            other => panic!("Extra Rage Power needs {other:?}"),
        })
        .collect();
    let every: BTreeSet<String> = selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .collect();
    assert_eq!(named, every);
}

/// The ticket's: an Instinctual Warrior's rage power picker offers each monk power from its level and not before,
/// with a reason; Deflect Arrows needs Improved Unarmed Strike. One taken at its level is possible, one taken early
/// isn't, and the sheet shows it picked.
#[test]
fn an_instinctual_warrior_is_offered_each_monk_power_from_its_level_and_not_before() {
    let engine = engine();
    let build = instinctual_warrior(20, true);
    assert!(possible(&build));
    assert_eq!(
        levels_with_slot_of(&build, RAGE_POWER),
        INSTINCTUAL_RAGE_POWER_LEVELS
    );
    for level in INSTINCTUAL_RAGE_POWER_LEVELS {
        let open = open_at(&build, level);
        for (key, name, needs) in MONK_RAGE_POWERS {
            assert_eq!(
                open.contains(key),
                opens_at(needs, level),
                "{name} at {level}"
            );
        }
    }
    assert_eq!(
        lock_reason(&build, 4, MONK_RAGE_POWERS[1].0),
        "Needs Barbarian level 8 (have 4)"
    );
    assert_eq!(
        lock_reason(&build, 10, MONK_RAGE_POWERS[2].0),
        "Needs Barbarian level 12 (have 10)"
    );
    assert_eq!(
        lock_reason(&build, 12, MONK_RAGE_POWERS[3].0),
        "Needs Barbarian level 16 (have 12)"
    );
    assert_eq!(
        lock_reason(&build, 18, MONK_RAGE_POWERS[4].0),
        "Needs Barbarian level 20 (have 18)"
    );
    // Without Improved Unarmed Strike, Deflect Arrows is locked.
    let bare = instinctual_warrior(4, false);
    assert_eq!(
        lock_reason(&bare, 4, MONK_RAGE_POWERS[0].0),
        "Needs Improved Unarmed Strike"
    );

    // Each taken at its level: possible, and the sheet shows it in its slot.
    let mut taken = build.clone();
    for (slot, (key, _, _)) in [4, 8, 12, 16, 20].into_iter().zip(MONK_RAGE_POWERS) {
        taken.picks.insert(selection_slot(slot), pick(key));
    }
    assert!(possible(&taken));
    let sheet = engine.evaluate(&taken);
    let slot = sheet.slot(&selection_slot(8)).unwrap();
    assert_eq!(engine.slot_label(slot), "Rage Power");
    assert_eq!(
        engine.option_name(&SelectionOptionKey::new(MONK_RAGE_POWERS[1].0)),
        "Purity of Body (Instinctual Warrior)"
    );
    // A power taken is "Already taken" in a later slot.
    assert_eq!(
        lock_reason(&taken, 10, MONK_RAGE_POWERS[0].0),
        "Already taken"
    );

    // Taken before its level: impossible, for that reason.
    let mut early = instinctual_warrior(8, true);
    early
        .picks
        .insert(selection_slot(8), pick(MONK_RAGE_POWERS[2].0));
    assert_eq!(
        why_not(&early, &selection_slot(8)).and_then(|issue| engine.issue_text(&issue)),
        Some("Needs Barbarian level 12 (have 8)".to_owned())
    );
}

/// The ticket's: a barbarian without the archetype (plain, or another archetype) has them locked in every rage power
/// slot, needing the archetype (and the level, before it), and can't take one.
#[test]
fn a_barbarian_without_the_archetype_is_not_offered_the_monk_powers() {
    for archetype in [None, Some("barbarian/armored-hulk")] {
        let mut build = match archetype {
            Some(archetype) => chaotic(archetype_build("barbarian", archetype, 20)),
            None => barbarian(20),
        };
        build
            .picks
            .insert(general_slot(1), feat_pick_plain("improved-unarmed-strike"));
        assert!(possible(&build), "{archetype:?}");
        let levels = levels_with_slot_of(&build, RAGE_POWER);
        assert!(!levels.is_empty());
        for level in levels {
            let open = open_at(&build, level);
            for (key, name, needs) in MONK_RAGE_POWERS {
                assert!(!open.contains(key), "{name} at {level} ({archetype:?})");
                let reason = match needs {
                    Some(needs) if needs > level => format!(
                        "Needs Barbarian level {needs} (have {level}), the Instinctual Warrior archetype"
                    ),
                    _ => "Needs the Instinctual Warrior archetype".to_owned(),
                };
                assert_eq!(
                    lock_reason(&build, level, key),
                    reason,
                    "{name} at {level} ({archetype:?})"
                );
            }
        }
        let mut taken = build.clone();
        taken
            .picks
            .insert(selection_slot(20), pick(MONK_RAGE_POWERS[4].0));
        assert!(!possible(&taken), "{archetype:?}");
    }
}

/// The ticket's: Extra Rage Power gives an Instinctual Warrior a rage power slot offering the monk powers as its own
/// slots do, from their levels. Holding only a monk power meets the feat's "Rage Power".
#[test]
fn extra_rage_power_offers_an_instinctual_warrior_its_monk_powers() {
    let engine = engine();
    let mut build = instinctual_warrior(9, true);
    build
        .picks
        .insert(selection_slot(4), pick(MONK_RAGE_POWERS[0].0));
    build
        .picks
        .insert(general_slot(5), feat_pick_plain("extra-rage-power"));
    assert!(possible(&build));
    build
        .picks
        .insert(general_slot(9), feat_pick_plain("extra-rage-power"));
    assert!(possible(&build));
    for level in [5, 9] {
        assert_eq!(
            kind_at(&build, &extra_slot(level, 0)),
            Some(selection_kind(RAGE_POWER)),
            "at {level}"
        );
    }
    let keys = |slot: &SlotId| -> BTreeSet<String> {
        engine
            .options(&build, slot)
            .unwrap()
            .into_iter()
            .map(|option| option.pick.key().to_owned())
            .collect()
    };
    assert_eq!(keys(&extra_slot(9, 0)), keys(&selection_slot(8)));
    let open = |slot: &SlotId| -> BTreeSet<String> {
        engine
            .options(&build, slot)
            .unwrap()
            .into_iter()
            .filter(|option| option.status.is_ok())
            .map(|option| option.pick.key().to_owned())
            .collect()
    };
    // Deflect Arrows is taken at 4; Purity of Body opens at 8, so in the slot at 9 but not at 5.
    for (slot, level) in [(extra_slot(5, 0), 5), (extra_slot(9, 0), 9)] {
        let open = open(&slot);
        assert!(!open.contains(MONK_RAGE_POWERS[0].0), "at {level}");
        for (key, name, needs) in &MONK_RAGE_POWERS[1..] {
            assert_eq!(
                open.contains(*key),
                opens_at(*needs, level),
                "{name} at {level}"
            );
        }
    }
    let mut filled = build.clone();
    filled
        .picks
        .insert(extra_slot(9, 0), pick(MONK_RAGE_POWERS[1].0));
    let filled = with_selections_filled(filled);
    assert!(possible(&filled));

    // A plain barbarian's Extra Rage Power slot has them locked.
    let mut plain = with_selections_filled(barbarian(3));
    plain
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    let locked: BTreeSet<String> = engine
        .options(&plain, &extra_slot(3, 0))
        .unwrap()
        .into_iter()
        .filter(|option| option.status.is_err())
        .map(|option| option.pick.key().to_owned())
        .collect();
    for (key, name, _) in MONK_RAGE_POWERS {
        assert!(locked.contains(key), "{name}");
    }
}

// ---- the Dual-Cursed Oracle's second curse (ticket 67) -----------------------------------------------------------

const DUAL_CURSED: &str = "oracle/dual-cursed-oracle";

/// The Dual-Cursed Oracle's level-1 slots in key order: its own revelation, the mystery, and the two curses.
const FIRST_CURSE: SlotId = SlotId::Selection { level: 1, index: 2 };
const SECOND_CURSE: SlotId = SlotId::Selection { level: 1, index: 3 };

/// "The Oracle must choose a second curse" (oracle#second-curse): its table's "Second Curse" at 1 is a second slot of
/// the oracle's curses in the rows in force, not a feature; the plain oracle and the other archetypes keep one.
#[test]
fn the_dual_cursed_oracles_second_curse_is_a_slot_of_the_oracles_curses() {
    let oracle = class("oracle");
    let curses = rules::FeatureSelectionKey::new(CURSES);
    let curse_slots = |archetype: Option<&rules::Archetype>, level| {
        rules::effective_row(oracle, archetype, level).map_or(0, |row| {
            row.selections.iter().filter(|s| **s == curses).count()
        })
    };
    let dual = oracle
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == DUAL_CURSED)
        .expect("the Dual-Cursed Oracle");
    assert_eq!(curse_slots(Some(dual), 1), 2);
    assert!((2..=20).all(|level| curse_slots(Some(dual), level) == 0));
    assert_eq!(curse_slots(None, 1), 1);
    for archetype in oracle
        .archetypes
        .iter()
        .filter(|a| a.key.as_str() != DUAL_CURSED)
    {
        assert!(curse_slots(Some(archetype), 1) <= 1, "{}", archetype.key);
    }
    let second = FeatureKey::new("oracle/dual-cursed-oracle/second-curse");
    assert!(dual.features.iter().all(|feature| feature.key != second));
    assert!(
        dual.levels
            .iter()
            .all(|row| !row.features.contains(&second))
    );
}

#[test]
fn a_dual_cursed_oracle_1_picks_two_curses_the_second_not_offering_the_first() {
    let engine = engine();
    let build = archetype_build("oracle", DUAL_CURSED, 1);
    let sheet = engine.evaluate(&build);
    for id in [&FIRST_CURSE, &SECOND_CURSE] {
        let slot = sheet.slot(id).expect("a curse slot");
        assert_eq!(slot.kind, selection_kind(CURSES), "{id:?}");
        assert_eq!(engine.slot_label(slot), "Oracle's Curse");
    }
    let curse_slots = |build: &Build| {
        engine.evaluate(build).levels[0]
            .slots
            .iter()
            .filter(|slot| slot.kind == selection_kind(CURSES))
            .count()
    };
    assert_eq!(curse_slots(&build), 2);
    assert_eq!(curse_slots(&self::build("oracle", 1)), 1);
    assert!(!holds(
        &at(&build, 1),
        "oracle/dual-cursed-oracle/second-curse"
    ));

    // Either slot offers the nine curses, the one taken in the other locked: "Already taken" in the second slot, and
    // in the first, breaking the second's pick (as a cleric's two domain slots answer).
    let first = "oracle/oracles-curse/plagued";
    let second = "oracle/oracles-curse/lame";
    for (taken_in, other, issue) in [
        (&FIRST_CURSE, &SECOND_CURSE, Issue::AlreadyHeld),
        (&SECOND_CURSE, &FIRST_CURSE, Issue::BreaksLater),
    ] {
        let one = with_pick(build.clone(), taken_in.clone(), first);
        let curses = offered_in(&one, other);
        assert_eq!(curses.len(), 9);
        for (curse, status) in &curses {
            if curse == first {
                assert_eq!(*status, Err(issue.clone()), "{other:?}");
            } else {
                assert_eq!(*status, Ok(()), "{curse}");
            }
        }
    }
    let both = with_pick(
        with_pick(build.clone(), FIRST_CURSE, first),
        SECOND_CURSE,
        second,
    );
    assert!(possible(&both));
    let state = at(&both, 1);
    for curse in [first, second] {
        assert_eq!(state.option_count(&SelectionOptionKey::new(curse)), 1);
    }
    let twice = with_pick(with_pick(build, FIRST_CURSE, first), SECOND_CURSE, first);
    assert_eq!(why_not(&twice, &SECOND_CURSE), Some(Issue::AlreadyHeld));
}

#[test]
fn beneficial_curse_offers_a_dual_cursed_oracle_neither_of_her_curses() {
    let oracle = with_selections_filled(archetype_build("oracle", DUAL_CURSED, 6));
    let held: BTreeSet<String> = [&FIRST_CURSE, &SECOND_CURSE]
        .into_iter()
        .map(|slot| oracle.picks[slot].key().to_owned())
        .collect();
    assert_eq!(held.len(), 2);
    let oracle = with_mr_1_ability(oracle, BENEFICIAL_CURSE);
    assert!(possible(&oracle));
    let curses = offered_in(&oracle, &mr_1_selection());
    assert_eq!(curses.len(), 9);
    for (curse, status) in &curses {
        if held.contains(curse) {
            assert_eq!(*status, Err(Issue::AlreadyHeld), "{curse}");
        } else {
            assert_eq!(*status, Ok(()), "{curse}");
        }
    }
    for curse in &held {
        let again = with_pick(oracle.clone(), mr_1_selection(), curse);
        assert_eq!(why_not(&again, &mr_1_selection()), Some(Issue::AlreadyHeld));
    }
    let third = curses.keys().find(|curse| !held.contains(*curse)).unwrap();
    assert!(possible(&with_pick(oracle, mr_1_selection(), third)));
}
