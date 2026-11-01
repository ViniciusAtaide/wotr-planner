//! The real `game-data/` files, read without a database: they pass the data check, and a few facts the planner
//! depends on hold in them.

use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use rules::{
    Ability, ArchetypeKey, ArmourType, Build, ClassChoice, ClassKey, Engine, FeatKey, FeatSlot,
    FeatSource, FeatureKey, GameData, Issue, LevelUp, MythicPathKey, Parameter, ParameterKind,
    Pick, Prerequisite, Problem, RankStatus, Repeat, SelectionOptionKey, SkillKey, SlotId,
    SlotKind, SlotOption, SpellSchool, Unmet, WeaponKey, WeaponProficiency,
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
    // of the player's choosing). The real data has 46: 43 named lists, Gold Dragon's Dragon Feats (a list of every
    // feat), and the warpriest's Weapon Focus and the Master of All's Skill Focus.
    assert_eq!(DATA.feat_lists().count(), 46);
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
    assert_eq!(exotic.len(), 16);
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
    assert_eq!(count(Some(WeaponProficiency::Exotic)), 16);
    // Sawtooth Saber (Weapon Finesse's list), unarmed strike, natural weapons, touch attacks and rays.
    let others: BTreeSet<&str> = DATA
        .weapons
        .iter()
        .filter(|w| w.proficiency.is_none())
        .map(|w| w.key.as_str())
        .collect();
    assert_eq!(
        others,
        BTreeSet::from([
            "natural-weapons",
            "rays",
            "sawtooth-saber",
            "touch-attacks",
            "unarmed-strike"
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
        (Some(K::ExoticWeapon), Repeat::Once)
    );
    assert_eq!(shape("spell-focus"), (Some(K::SpellSchool), Repeat::Once));
    assert_eq!(shape("elemental-focus"), (Some(K::Energy), Repeat::Once));
    assert_eq!(shape("armor-focus"), (Some(K::ArmourType), Repeat::Once));
    assert_eq!(shape("improved-critical"), (Some(K::Weapon), Repeat::Once));
    assert_eq!(shape("extra-hex"), (None, Repeat::Times(3)));
    assert_eq!(shape("extra-ki"), (None, Repeat::Unlimited));
    assert_eq!(shape("magical-tail"), (None, Repeat::Times(8)));
    let repeating: Vec<&str> = DATA
        .feats
        .iter()
        .filter(|f| f.repeat != Repeat::Once)
        .map(|f| f.key.as_str())
        .collect();
    // Weapon Focus, Skill Focus, Magical Tail and the 18 Extra feats the guide says repeat (not Extra Bane
    // Weapon: it doesn't say).
    assert_eq!(repeating.len(), 21, "{repeating:?}");
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
