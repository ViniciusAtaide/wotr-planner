//! The feats' prerequisites as the game's (ticket 308c, written by `game-check --write prerequisites --only 'feat *'`):
//! one real-data test per pattern the run changed.

use rules::{
    Ability, FeatKey, FeatureKey, FeatureSelectionKey, Prerequisite, RaceKey, SelectionOptionKey,
    SpellType,
};

use crate::support::*;

/// A feat's prerequisite.
fn prerequisite(key: &str) -> &'static Prerequisite {
    feat(key)
        .prerequisite
        .as_ref()
        .unwrap_or_else(|| panic!("{key}: no prerequisite"))
}

/// Whether a feat's prerequisite names `leaf` anywhere in its tree.
fn names(key: &str, leaf: &Prerequisite) -> bool {
    prerequisite(key).leaves().any(|held| held == leaf)
}

/// The conditions an `All` prerequisite holds directly (a single condition is its own).
fn all_of(key: &str) -> &'static [Prerequisite] {
    match prerequisite(key) {
        Prerequisite::All(children) => children,
        single => std::slice::from_ref(single),
    }
}

fn has_feat(key: &str) -> Prerequisite {
    Prerequisite::Feat(FeatKey::new(key))
}

fn has_selection(key: &str) -> Prerequisite {
    Prerequisite::Selection(FeatureSelectionKey::new(key))
}

/// The game repeats a tier's leaves in every later tier: each of Cleave's chain needs Strength 13 itself, not only
/// through Cleave.
#[test]
fn each_of_cleaves_chain_needs_strength_13_itself() {
    // Game: PrerequisiteStatValue Strength 13 on CleaveFeature d809b6c4ff2aaff4fa70d712a70f7d7b, GreatCleaveFeature
    // cc9c862ef2e03af4f89be5088851ea35, CleavingFinish 59bd93899149fa44687ff4121389b3a9 and ImprovedCleavingFinish
    // ffa1b373190af4f4db7a5501904a1983; the last two name PowerAttackFeature 9972f33f977fc724c838e59641b2fca5 too.
    let strength_13 = Prerequisite::AbilityScore {
        ability: Ability::Strength,
        score: 13,
    };
    for key in [
        "cleave",
        "great-cleave",
        "cleaving-finish",
        "improved-cleaving-finish",
    ] {
        assert!(all_of(key).contains(&strength_13), "{key}");
    }
    for key in ["cleaving-finish", "improved-cleaving-finish"] {
        assert!(all_of(key).contains(&has_feat("power-attack")), "{key}");
    }
}

/// The combat manoeuvre feats need Intelligence 13 whichever way they're reached: Combat Expertise or the Kinetic
/// Knight's Kinetic Warrior, an `Any` beside the score.
#[test]
fn the_manoeuvre_feats_need_intelligence_13_beside_combat_expertise_or_kinetic_warrior() {
    // Game: ImprovedTrip 0f15c6f70d8fb2b49aa6cc24239cc5fa: PrerequisiteFeature CombatExpertiseFeature
    // 4c44724ffa8844f4d9bedb5bb27d144a (Group Any), KineticWarriorFeature ff14cb2bfab1c0547be66d8aaa7e4ada (Group Any),
    // PrerequisiteStatValue Intelligence 13 (Group All); Feint c610310d31414edabcedf0c8a6fe32c4 the same.
    let either = Prerequisite::Any(vec![
        has_feat("combat-expertise"),
        Prerequisite::Feature {
            feature: FeatureKey::new("kineticist/kinetic-knight/kinetic-warrior"),
            count: 1,
        },
    ]);
    let intelligence_13 = Prerequisite::AbilityScore {
        ability: Ability::Intelligence,
        score: 13,
    };
    for key in ["trip", "feint"] {
        let all = all_of(key);
        assert!(all.contains(&intelligence_13), "{key}");
        assert!(
            all.iter().any(|child| match (child, &either) {
                (Prerequisite::Any(found), Prerequisite::Any(wanted)) =>
                    found.len() == 2 && wanted.iter().all(|leaf| found.contains(leaf)),
                _ => false,
            }),
            "{key}: {all:?}"
        );
    }
}

/// The racial feats name the game's races: Ironguts a dwarf or a half-orc, Blood Feaster a dhampir, Murmurs of Earth
/// an oread.
#[test]
fn the_racial_feats_name_the_games_races() {
    // Game: Ironguts 90f7164ce17840c68c1ceadcf5d10297 (DwarfRace c4faf439f0e70bd40b5e36ee80d06be7 or HalfOrcRace
    // 1dc20e195581a804890ddc74218bfd8e, Group Any); BloodFeaster e78b4b66db382aa4199f5f2b7da6e5ea (DhampirRace
    // 64e8b7d5f1ae91d45bbf1e56a3fdff01); MurmursOfEarth 94be54cd152d1c94396754de7bf0105f (OreadRace
    // 4d4555326b9b7144f93be1ea61337cd7).
    let race = |key: &str| Prerequisite::Race(RaceKey::new(key));
    match prerequisite("ironguts") {
        Prerequisite::Any(races) => {
            assert_eq!(races.len(), 2);
            assert!(races.contains(&race("dwarf")) && races.contains(&race("half-orc")));
        }
        other => panic!("ironguts: {other:?}"),
    }
    assert!(all_of("blood-feaster").contains(&race("dhampir")));
    assert!(all_of("murmurs-of-earth").contains(&race("oread")));
}

/// The base attack bonus the game names: Ranged Feint BAB 2, Bashing Finish BAB 11.
#[test]
fn the_feats_name_the_games_base_attack_bonus() {
    // Game: RangedFeint a2e947d6be234abba7c3ac0bd5dc9b1d (Feint, PrerequisiteStatValue BaseAttackBonus 2);
    // ImprovedCleavingFinish ffa1b373190af4f4db7a5501904a1983 (BaseAttackBonus 6); Manyshot
    // adf54af2a681792489826f7fd1b62889 (RapidShotFeature and BaseAttackBonus 6, no Dexterity, no Point-Blank Shot).
    assert!(all_of("ranged-feint").contains(&Prerequisite::BaseAttackBonus(2)));
    assert!(all_of("improved-cleaving-finish").contains(&Prerequisite::BaseAttackBonus(6)));
    let manyshot = all_of("manyshot");
    assert_eq!(manyshot.len(), 2, "{manyshot:?}");
    assert!(manyshot.contains(&has_feat("rapid-shot")));
    assert!(manyshot.contains(&Prerequisite::BaseAttackBonus(6)));
}

/// The Extra feats need the selection their pick stands for, held whatever was picked in it, not one of its options.
#[test]
fn the_extra_feats_need_the_selections_the_game_names() {
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a (RagePowerSelection
    // 28710502f46848d48b3f0d6132817c4e, InstinctualWarriorRagePowerSelection 609f0e5336084442a0dafa3abd4d31c5 or
    // SkaldRagePowerSelection 2476514e31791394fa140f1a07941c96); ExtraArcanistExploitSelection
    // 0a7065e13f4449fa81bfb33199bf7f6a (ArcanistExploitSelection b8bf3d5023f2d8c428fdf6438cecaea7 or
    // ExploiterExploitSelection 2ba8a0040e0149e9ae9bfcb01a8ff01d); ExtraMercySelection
    // 8a49abed5be9473da1e1fd1e2457562e (SelectionMercy 02b187038a8dce545bb34bbfb346428d);
    // ExtraVivsectionistDiscoverySelection 10287e7b8cee479e82ea88bd6d2d4dae (VivsectionistDiscoverySelection
    // 67f499218a0e22944abab6fe1c9eaeee); ExtraArcanaSelection 00727883edf145e2a6bce9ad176ecfd8 (MagusArcanaSelection
    // e9dc4dfc73eaaf94aae27e0ed6cc9ada or HexcrafterMagusHexArcanaSelection ad6b9cecb5286d841a66e23cea3ef7bf; the
    // Eldritch Scion's EldritchMagusArcanaSelection d4b54d9db4932454ab2899f931c2042c has no entry, research/308
    // decisions.md item 2); ExtraShamanHexSelection d0b4c8245d504b8c9c6d3fccc1f8c5b6 (ShamanHexSelection
    // 4223fe18c75d4d14787af196a04e14e7; WinterWitchShamanHexSelection 4e760c5034fafb2438993d8a192150b9 has none).
    for (feat_key, selections) in [
        (
            "extra-rage-power",
            &[
                "barbarian/rage-power",
                "barbarian/instinctual-warrior/rage-power",
                "barbarian/skald-rage-power",
            ][..],
        ),
        (
            "extra-arcanist-exploit",
            &["arcanist/arcanist-exploits", "arcanist/exploiter-exploits"],
        ),
        ("extra-mercy", &["paladin/mercy"]),
        (
            "extra-discovery-vivisectionist",
            &["alchemist/vivisectionist/medical-discovery"],
        ),
        (
            "extra-arcana",
            &["magus/magus-arcana", "magus/hexcrafter/hex-arcana"],
        ),
        ("extra-hex-shaman", &["shaman/hex"]),
    ] {
        for selection in selections {
            assert!(
                names(feat_key, &has_selection(selection)),
                "{feat_key}: {selection}"
            );
        }
        assert!(
            !prerequisite(feat_key)
                .leaves()
                .any(|leaf| matches!(leaf, Prerequisite::SelectionOption(_))),
            "{feat_key} names an option"
        );
    }
}

/// Where the planner holds the game's blueprint under two entries, a feat accepts either: Dazzling Display, Shatter
/// Defenses and the two Penetrating Strikes accept the rogue talent's Weapon Focus as the feat's.
#[test]
fn the_weapon_focus_feats_accept_the_rogue_talents_weapon_focus() {
    // Game: DazzlingDisplayFeature bcbd674ec70ff6f4894bb5f07b6f4095, ShatterDefenses 61a17ccbbb3d79445b0926347ec07577,
    // PenetratingStrike 308cd7dc4f10efd428f531bbf4f2823d and GreaterPenetratingStrike eb6eb946c68ef094f89c7633f5bfdc9b
    // name WeaponFocus 1e1f627d26ad36f43bbd26cc2bf8ac7e, which the feat weapon-focus and the option
    // rogue/rogue-talents/weapon-focus both map (one blueprint, held once: research/308/slices.md §1.1).
    let talent =
        Prerequisite::SelectionOption(SelectionOptionKey::new(rogue_talent("weapon-focus")));
    for key in [
        "dazzling-display",
        "shatter-defenses",
        "penetrating-strike",
        "greater-penetrating-strike",
    ] {
        assert!(names(key, &has_feat("weapon-focus")), "{key}");
        assert!(names(key, &talent), "{key}");
    }
}

/// Arcane Strike needs an arcane caster, not an arcane spell level: a bloodrager 1, who casts no spell yet, meets it.
#[test]
fn arcane_strike_needs_an_arcane_caster_type() {
    // Game: ArcaneStrikeFeature 0ab2f21a922feee4dab116238e3150b4: PrerequisiteCasterType IsArcane (and Intelligence 3,
    // which no Build fails); Blueprints/Classes/Prerequisites/PrerequisiteCasterType.cs CheckInternal, WrathDecompiled
    // 6ce4a6b: a class with an arcane spellbook, whatever level it casts yet.
    assert_eq!(
        prerequisite("arcane-strike"),
        &Prerequisite::CasterType(SpellType::Arcane)
    );
    assert_eq!(
        feat_status(&build("bloodrager", 1), &general_slot(1), "arcane-strike"),
        Ok(())
    );
}

/// Natural Charmer and Perfect Strike read the full score (the permanent one plus feat bonuses), as the game's
/// `PrerequisiteFullStatValue`.
#[test]
fn natural_charmer_reads_the_full_charisma() {
    // Game: NaturalCharmer 0ebbc4a0ecf03844384a177e9f1f9a11: DhampirRace and PrerequisiteFullStatValue Charisma 17;
    // PerfectStrikeFeature 7477e2e0b72f4ce4fb674f4b21d5e81d: PrerequisiteFullStatValue Dexterity 13 and Wisdom 13.
    assert!(
        all_of("natural-charmer").contains(&Prerequisite::FullAbilityScore {
            ability: Ability::Charisma,
            score: 17,
        })
    );
    for ability in [Ability::Dexterity, Ability::Wisdom] {
        assert!(
            all_of("perfect-strike")
                .contains(&Prerequisite::FullAbilityScore { ability, score: 13 }),
            "{ability:?}"
        );
    }
}

/// Critical Mastery counts two of the game's five critical feats; Flaying and Stunning Critical don't count.
#[test]
fn critical_mastery_counts_two_of_the_games_five_critical_feats() {
    // Game: CriticalMastery fc85925cd241dd0408ab2f4cb171b7e3: PrerequisiteFeaturesFromList Amount 2 of
    // BlindingCriticalFeature 787e56055e3ef864d9c78a3ec21e56be, ExhaustingCriticalFeature
    // 7c492212d25d8f04fbd43eb99d780b1e, SickeningCriticalFeature 4c7205d859a1e114895e798af383d76a,
    // StaggeringCriticalFeature 055fb8dcaf6be334ca13172bafa9e782 and TiringCriticalFeature
    // 12c1b556df3b667458ae200bfb38ccb8.
    let counted = Prerequisite::AtLeast(
        2,
        [
            "blinding-critical",
            "exhausting-critical",
            "sickening-critical",
            "staggering-critical",
            "tiring-critical",
        ]
        .map(has_feat)
        .to_vec(),
    );
    assert!(all_of("critical-mastery").contains(&counted));
    for absent in ["flaying-critical", "stunning-critical"] {
        assert!(!names("critical-mastery", &has_feat(absent)), "{absent}");
    }
}
