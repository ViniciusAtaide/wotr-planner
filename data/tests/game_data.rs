//! The real `game-data/` files, read without a database: they pass the data check, and a few facts the planner
//! depends on hold in them.

use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use rules::{
    Ability, Advancement, ArchetypeKey, ArmourType, Build, CharacterState, ClassChoice, ClassKey,
    CountsAs, CountsToward, Engine, FeatKey, FeatSlot, FeatSource, FeatureKey, GameData, Issue,
    LevelUp, MythicPathKey, Parameter, ParameterKind, Pick, Prerequisite, Problem, RankStatus,
    Repeat, SelectionOptionKey, SkillKey, SlotId, SlotKind, SlotOption, SpellSchool, Story, Unmet,
    WeaponKey, WeaponProficiency,
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
    // styles' since ticket 30, 4 the kineticist elements' wild-talent bonus feats since ticket 31, 3 the slayer's
    // combat style stages since ticket 32), Gold Dragon's Dragon Feats (a list of every feat), and the warpriest's
    // Weapon Focus, the Master of All's Skill Focus, the Sword Saint's Weapon Focus and the half-elf's Skill Focus.
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
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.problems, []);
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
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.problems, []);
    let at_17 = sheet.at(17).unwrap();
    assert!(at_17.selection_options.is_empty());
    assert_eq!(
        at_17.feature_count(&FeatureKey::new("aeon/enforcing-gaze-extra-gaze")),
        0
    );
    assert_eq!(at_17.feature_count(&FeatureKey::new("aeon/aeon-bane")), 1);
    assert_eq!(sheet.ranks[2].status, RankStatus::PartlyDropped);
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
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.problems, []);
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
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.problems, []);
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
    let problems = engine().evaluate(&build).problems;
    assert_eq!(
        problems
            .iter()
            .map(|p| (&p.at, &p.issue))
            .collect::<Vec<_>>(),
        [
            (&rules::Choice::Slot(class_slot(2)), &Issue::AlreadyHeld),
            (
                &rules::Choice::Slot(class_slot(4)),
                &Issue::Unmet(Unmet::Condition {
                    needs: Prerequisite::FeatForSameValue(FeatKey::new("weapon-focus")),
                    has: None,
                })
            ),
        ]
    );
    build.picks.insert(
        class_slot(4),
        Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("longsword")),
    );
    build.picks.remove(&class_slot(2));
    assert_eq!(engine().evaluate(&build).problems, []);
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
        assert_eq!(engine().evaluate(&build).problems, [], "{archetype:?}");
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
    assert_eq!(engine().evaluate(&build).problems, []);
    assert_eq!(
        values_at(&build, 15, "skill-focus"),
        BTreeSet::from([skill("mobility"), skill("perception"), skill("persuasion")])
    );
    // Not twice in one skill.
    build.picks.insert(
        class_slot(15),
        Pick::feat_for(FeatKey::new("skill-focus"), skill("mobility")),
    );
    assert_eq!(
        engine()
            .evaluate(&build)
            .problems
            .into_iter()
            .map(|p| p.issue)
            .collect::<Vec<_>>(),
        [Issue::AlreadyHeld]
    );
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
    assert_eq!(engine().evaluate(&build).problems, []);
    assert_eq!(
        values_at(&build, 2, "weapon-focus"),
        BTreeSet::from([weapon("rapier")])
    );
    build
        .picks
        .insert(selection_slot(2), talent("skill-focus", skill("stealth")));
    assert_eq!(engine().evaluate(&build).problems, []);
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
        engine()
            .evaluate(&build)
            .problems
            .into_iter()
            .map(|p| (p.at, p.issue))
            .collect::<Vec<_>>(),
        [(
            rules::Choice::Slot(selection_slot(later[3])),
            Issue::MaxTimes { times: 3 }
        )]
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

fn advances_problems(build: &Build, prestige: &str) -> Vec<Issue> {
    engine()
        .validate(build)
        .into_iter()
        .filter(|p| {
            p.at == rules::Choice::Advances {
                class: ClassKey::new(prestige),
            }
        })
        .map(|p| p.issue)
        .collect()
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
    choose(&mut both, &["wizard", "cleric"]);
    assert_eq!(advances_problems(&both, "loremaster"), [Issue::NotAllowed]);
    choose(&mut both, &["cleric"]);
    assert_eq!(advances_problems(&both, "loremaster"), []);
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

    let cleric = std::fs::read_to_string(data::game_data_dir().join("classes/cleric.ron")).unwrap();
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
    assert_eq!(engine().validate(&build), []);
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
                selection: rules::FeatureSelectionKey::new("sorcerer/bloodline")
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
    assert_eq!(engine().validate(&build), []);
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
    let entry: Vec<Problem> = engine()
        .validate(&build)
        .into_iter()
        .filter(|p| p.at == rules::Choice::Class { level: 6 })
        .collect();
    assert_eq!(
        entry,
        [Problem {
            at: rules::Choice::Class { level: 6 },
            issue: Issue::Unmet(Unmet::Condition {
                needs: Prerequisite::CharacterLevel(6),
                has: Some(5),
            }),
        }]
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
    assert_eq!(engine().validate(&build), []);
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
    assert_eq!(engine().validate(&build), []);
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
    assert_eq!(engine().validate(&twice).len(), 1);
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
    assert_eq!(engine().validate(&nature), []);
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
    assert_eq!(engine().validate(&weather), []);
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
    assert_eq!(engine().validate(&blight), []);
    assert!(!holds(
        &at(&blight, 4),
        "cleric/domain/animal/animal-companion"
    ));
    assert!(holds(
        &at(&blight, 1),
        "cleric/domain/animal/class-skill-lore-nature"
    ));

    let winter = druid(Some("druid/winter-child"), 8);
    assert_eq!(engine().validate(&winter), []);
    assert!(holds(&at(&winter, 1), "cleric/domain/ice/icicle"));
    from_level(&winter, 8, "cleric/domain/ice/body-of-ice");
}

#[test]
fn an_inquisitors_domain_runs_on_inquisitor_levels() {
    let build = with_option(build("inquisitor", 6), 1, 0, "cleric/domain/luck");
    assert_eq!(engine().validate(&build), []);
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
    assert_eq!(engine().validate(&early).len(), 1);
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
    assert_eq!(engine().validate(&build), []);
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
    assert!(holds(
        &at(&build, 1),
        "oracle/mystery/battle/class-skill-perception"
    ));
    from_level(&build, 20, "oracle/mystery/battle/battle-final-revelation");
    // "he does not gain additional class skills from his choice of mystery" (Divine Herbalist).
    let herbalist = with_option(
        archetype_build("oracle", "oracle/divine-herbalist", 1),
        1,
        0,
        "oracle/mystery/battle",
    );
    assert_eq!(engine().validate(&herbalist), []);
    assert!(!holds(
        &at(&herbalist, 1),
        "oracle/mystery/battle/class-skill-perception"
    ));
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
    assert_eq!(engine().validate(&build), []);
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
    assert_eq!(engine().validate(&feral), []);
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
    assert_eq!(engine().validate(&build), []);
    let one = at(&build, 1);
    assert!(holds(
        &one,
        "kineticist/element/earth/class-skill-athletics"
    ));
    assert!(holds(
        &one,
        "kineticist/element/earth/class-skill-knowledge-world"
    ));
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
    assert_eq!(engine().validate(&knight), []);
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
    assert_eq!(engine().validate(&build), []);
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
    assert_eq!(engine().validate(&build).len(), 1, "a third Fire");
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
    let problems: Vec<Problem> = engine().validate(&build);
    assert_eq!(problems, []);
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

fn problems(build: &Build) -> Vec<Problem> {
    engine().evaluate(build).problems
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

fn unmet(slot: SlotId, needs: Prerequisite, has: Option<u8>) -> Problem {
    Problem {
        at: rules::Choice::Slot(slot),
        issue: Issue::Unmet(Unmet::Condition { needs, has }),
    }
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
    assert_eq!(problems(&build), []);
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
    assert_eq!(problems(&build), []);
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
    assert_eq!(problems(&fighter), []);
    assert_eq!(
        values_at(&fighter, 10, "greater-weapon-focus"),
        BTreeSet::from([weapon("dagger"), weapon("longsword")])
    );
    // Not twice in one weapon.
    fighter.picks.insert(
        class_slot(10),
        feat_pick("greater-weapon-focus", weapon("longsword")),
    );
    assert_eq!(
        problems(&fighter),
        [Problem {
            at: rules::Choice::Slot(class_slot(10)),
            issue: Issue::AlreadyHeld,
        }]
    );
}

#[test]
fn exotic_weapon_proficiency_can_take_the_sawtooth_saber() {
    // Exotic Weapon Proficiency can take the Sawtooth Saber.
    let mut fighter = build("fighter", 1);
    fighter.picks.insert(
        general_slot(1),
        feat_pick("exotic-weapon-proficiency", weapon("sawtooth-saber")),
    );
    assert_eq!(problems(&fighter), []);
}

#[test]
fn armor_focus_heavy_needs_heavy_armour_proficiency() {
    use rules::ArmourType as A;
    let focus = |armour| feat_pick("armor-focus", Parameter::ArmourType(armour));
    // A fighter has every armour proficiency.
    let mut fighter = build("fighter", 1);
    fighter.picks.insert(class_slot(1), focus(A::Heavy));
    assert_eq!(problems(&fighter), []);
    // A cleric has light and medium only.
    let mut cleric = build("cleric", 3);
    cleric.picks.insert(general_slot(3), focus(A::Heavy));
    assert_eq!(
        problems(&cleric),
        [unmet(
            general_slot(3),
            Prerequisite::FeatIfValue {
                value: Parameter::ArmourType(A::Heavy),
                feat: FeatKey::new("armor-proficiency-heavy-armor"),
            },
            None
        )]
    );
    cleric.picks.insert(general_slot(3), focus(A::Medium));
    assert_eq!(problems(&cleric), []);
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
    assert_eq!(problems(&fighter), []);
    fighter.picks.insert(
        mythic.clone(),
        Pick::mythic_feat_for(
            rules::MythicFeatKey::new("improved-critical-mythic"),
            weapon("dagger"),
        ),
    );
    assert_eq!(
        problems(&fighter),
        [unmet(
            mythic,
            Prerequisite::FeatForSameValue(FeatKey::new("improved-critical")),
            None
        )]
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
    let zealot = archetype_build("warpriest", "warpriest/mantis-zealot", 12);
    assert_eq!(problems(&zealot), []);
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
    assert_eq!(problems(&archer), []);
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
    assert_eq!(problems(&shaman), []);
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
    assert_eq!(problems(&slayer), []);
    // A fourth: each window's talent is taken once, and the first window is past.
    slayer.picks.insert(selection_slot(12), style(10));
    assert_eq!(
        problems(&slayer),
        [Problem {
            at: rules::Choice::Slot(selection_slot(12)),
            issue: Issue::AlreadyHeld,
        }]
    );
    // And the first window is past at 6.
    let mut late = build("slayer", 6);
    late.picks.insert(selection_slot(6), style(2));
    assert_eq!(
        problems(&late),
        [unmet(
            selection_slot(6),
            Prerequisite::Not(Box::new(Prerequisite::ClassLevel {
                class: ClassKey::new("slayer"),
                level: 6,
            })),
            None
        )]
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
    assert_eq!(problems(&dwarf), []);
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
