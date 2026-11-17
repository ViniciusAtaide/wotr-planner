//! The rogue's, slayer's, alchemist's, magus's, arcanist's and small families' option prerequisites as the game's
//! (ticket 308e, written by `game-check --write prerequisites --only <the families' prefixes>`): one real-data test per
//! pattern the run changed. Uncanny Dodge and the combat style talents have their tests in `borrowed_selections`,
//! `follow_ups`, `negated_prerequisites` and `option_follow_ups`.

use rules::{
    Alignment, ArchetypeKey, ClassKey, FeatKey, FeatureKey, FeatureSelectionKey, Prerequisite,
    SelectionOptionKey,
};

use crate::support::*;

/// An option's prerequisite, if it has one.
fn prerequisite_of(key: &str) -> Option<&'static Prerequisite> {
    selection_option(key).prerequisite.as_ref()
}

/// An option's prerequisite.
fn prerequisite(key: &str) -> &'static Prerequisite {
    prerequisite_of(key).unwrap_or_else(|| panic!("{key}: no prerequisite"))
}

fn class_level(class: &str, level: u8) -> Prerequisite {
    Prerequisite::ClassLevel {
        class: ClassKey::new(class),
        level,
    }
}

/// An archetype's class at a level: `PrerequisiteArchetypeLevel` as the data compiles it.
fn archetype_level(class: &str, archetype: &str, level: u8) -> Prerequisite {
    Prerequisite::All(vec![
        class_level(class, level),
        Prerequisite::Archetype(ArchetypeKey::new(archetype)),
    ])
}

fn not(inner: Prerequisite) -> Prerequisite {
    Prerequisite::Not(Box::new(inner))
}

fn not_archetype(archetype: &str) -> Prerequisite {
    not(Prerequisite::Archetype(ArchetypeKey::new(archetype)))
}

fn has_feature(key: &str) -> Prerequisite {
    Prerequisite::Feature {
        feature: FeatureKey::new(key),
        count: 1,
    }
}

fn has_option(key: &str) -> Prerequisite {
    Prerequisite::SelectionOption(SelectionOptionKey::new(key))
}

fn has_selection(key: &str) -> Prerequisite {
    Prerequisite::Selection(FeatureSelectionKey::new(key))
}

/// The Bloodseeker's nocturnal abilities need the slayer level each blueprint names.
#[test]
fn the_bloodseekers_nocturnal_abilities_need_their_slayer_levels() {
    // Game: each one's PrerequisiteClassLevel SlayerClass: BloodBlasphemyFeature 02f9d14328f64233aaa30f7b493de059 16,
    // BloodEneravtionFeature 9b748f01f281406c8517e907cce551db 10, BloodHasteFeature b2d8de6d91e1426a9cac33e1091f53c0 8,
    // PredatoryMightFeature d5581be0f6124fac900e23398a694750 6, PredatoryProwesFeature 845bc5d567a94b95bebac28bd77dbf93 6,
    // BloodSlowFeature 8adca4a181cc444da922b5c861f3a86b 8, ConjureBloodServantFeature 28294c00a938435b86f6468c49ef3969
    // 8, BloodVampiricShadowShieldFeature 82bd9b37161b4b0493e91856eddfdd03 14, BloodVampiricTouchFeature
    // f707fbd648224498b2d8c36a56045dfe 8, BloodWrackingRayFeature 96089a505a5c4a0fb3ad0a88c9cc1d0a 14.
    for (ability, level) in [
        ("blasphemy", 16),
        ("enervation", 10),
        ("haste", 8),
        ("predatory-might", 6),
        ("predatory-prowess", 6),
        ("slow", 8),
        ("summon-bloody-bones-beast", 8),
        ("vampiric-shadow-shield", 14),
        ("vampiric-touch", 8),
        ("wracking-ray", 14),
    ] {
        assert_eq!(
            prerequisite(&format!("slayer/bloodseeker/nocturnal-abilities/{ability}")),
            &class_level("slayer", level),
            "{ability}"
        );
    }
}

/// The magus arcana named by level need that magus level, and four of them refuse the Armored Battlemage.
#[test]
fn the_magus_arcana_need_their_magus_level_and_four_refuse_the_armored_battlemage() {
    // Game: PrerequisiteClassLevel MagusClass at DimensionStrikeFeature cb6916027e3c25e4185de068249254dc 9,
    // EmpoweredArcanaFeature 4cab1e1a9d8e68f41a48777196c65501 6, HastedAssaultFeature 85c05a8120e3e9f4e8ae01625038809a
    // 9, MaximizedArcanaFeature 6c44c7fe4cf35644ea5ed25e48aa6db6 12, PrescientAttackFeature
    // a3909a7293533fe49a2d7cfe051f17e4 6, QuickenedArcanaFeature 10a75ca624a7d3242a05003d22bc38f0 15.
    for (arcana, level) in [
        ("dimension-strike-level-9", 9),
        ("empowered-magic-level-6", 6),
        ("hasted-assault-level-9", 9),
        ("maximized-magic-level-12", 12),
        ("prescient-attack-level-6", 6),
        ("quickened-magic-level-15", 15),
    ] {
        assert_eq!(
            prerequisite(&format!("magus/magus-arcana/{arcana}")),
            &class_level("magus", level),
            "{arcana}"
        );
    }
    // Game: BaneBladeFeature a2e0691dcfda2374e84d8bbf480e06a0 (15), DevotedBladeFeature 4be0bb10e110a35419e406da767bd1e3
    // (12), EnduringBladeFeature 7a73bf165e8eda6478b4419f857d1ab5 (6) and GhostBladeFeature
    // 8896f327c59569c4eaf129bf35b96c1f (9): the magus level and PrerequisiteNoArchetype ArmoredBattlemageArchetype
    // 67ec8dcae6fb3d3439e5ae874ddc7b9b.
    for (arcana, level) in [
        ("bane-blade-level-15", 15),
        ("devoted-blade-level-12", 12),
        ("enduring-blade-level-6", 6),
        ("ghost-blade-level-9", 9),
    ] {
        assert_eq!(
            prerequisite(&format!("magus/magus-arcana/{arcana}")),
            &Prerequisite::All(vec![
                not_archetype("magus/armored-battlemage"),
                class_level("magus", level),
            ]),
            "{arcana}"
        );
    }
}

/// The discoveries, exploits and talents whose blueprints refuse an archetype refuse it.
#[test]
fn the_options_whose_blueprints_refuse_an_archetype_refuse_it() {
    // Game: CognatogenFeature e3f460ea61fcc504183c7d6818bbbf7a: PrerequisiteNoArchetype IncenseSynthesizerArchetype
    // 6faf49766d8156c419d08f556a40374f.
    assert_eq!(
        prerequisite("alchemist/discovery/cognatogen"),
        &not_archetype("alchemist/incense-synthesizer")
    );
    // Game: FastBombsFeature 128c5fccec5ca724281a4907b1f0ac83: PrerequisiteNoArchetype VivisectionistArchetype
    // 68cbcd9fbf1fb1d489562f829bb97e38 and MetamorphArchetype ac242ddeaa05af449bfc5230231fab84, and alchemist 8.
    assert_eq!(
        prerequisite("alchemist/discovery/fast-bombs"),
        &Prerequisite::All(vec![
            not_archetype("alchemist/metamorph"),
            not_archetype("alchemist/vivisectionist"),
            class_level("alchemist", 8),
        ])
    );
    // Game: PreserveOrgans 76b4bb8e54f3f5c418f421684c76ef4e: PrerequisiteNoArchetype MetamorphArchetype.
    assert_eq!(
        prerequisite("alchemist/discovery/preserve-organs"),
        &not_archetype("alchemist/metamorph")
    );
    // Game: ArcanistExploitPotentMagicFeature 995110cc948d5164a820403a9e903151: PrerequisiteNoArchetype
    // PhantasmalMageArchetype e9d0ee69305049fe8400a066010dbcd1.
    assert_eq!(
        prerequisite("arcanist/arcanist-exploits/potent-magic"),
        &not_archetype("arcanist/phantasmal-mage")
    );
    // Game: MysterySeeker 04f8c258a47a2de4faac28be313b1621: PrerequisiteNoArchetype MasterOfAllArchetype
    // bd4e70bfb89a452b876713d61b9b8eb2; Survivalist 9836ea18e56af8249bdda4a4b11ea78c: it and SylvanTricksterArchetype
    // 490394869f666c141bf8647b1a365220.
    assert_eq!(
        prerequisite("rogue/rogue-talents/mystery-seeker"),
        &not_archetype("rogue/master-of-all")
    );
    assert_eq!(
        prerequisite("rogue/rogue-talents/survivalist"),
        &Prerequisite::All(vec![
            not_archetype("rogue/master-of-all"),
            not_archetype("rogue/sylvan-trickster"),
        ])
    );
}

/// The mutagen discoveries accept the Mutation Warrior's mutagen and fighter levels beside the alchemist's.
#[test]
fn the_mutagen_discoveries_accept_the_mutation_warrior() {
    let mutation_warrior = |level| archetype_level("fighter", "fighter/mutation-warrior", level);
    // Game: FeralMutagenFeature fd5f7b37ab4301c48a88cc196ee5f0ce: PrerequisiteFeature MutagenFeature or
    // MutationWarriorMutagen, Group Any.
    assert_eq!(
        prerequisite("alchemist/discovery/feral-mutagen"),
        &Prerequisite::Any(vec![
            has_feature("alchemist/mutagen"),
            has_feature("fighter/mutation-warrior/mutagen"),
        ])
    );
    // Game: GreaterMutagenFeature 76c61966afdd82048911f3d63c6fe0bc: alchemist 12 or a Mutation Warrior fighter 12
    // (PrerequisiteArchetypeLevel), and one of the two mutagens (PrerequisiteFeaturesFromList).
    assert_eq!(
        prerequisite("alchemist/discovery/greater-mutagen"),
        &Prerequisite::All(vec![
            Prerequisite::Any(vec![mutation_warrior(12), class_level("alchemist", 12)]),
            Prerequisite::AtLeast(
                1,
                vec![
                    has_feature("alchemist/mutagen"),
                    has_feature("fighter/mutation-warrior/mutagen"),
                ]
            ),
        ])
    );
    // Game: GrandMutagenFeature 6f5cb651e26bd97428523061b07ffc85 (16, after Greater Mutagen) and FeralWings
    // 78197196e096c6e4eaed5c62fa108b52 (6, after Feral Mutagen).
    assert_eq!(
        prerequisite("alchemist/discovery/grand-mutagen"),
        &Prerequisite::All(vec![
            Prerequisite::Any(vec![mutation_warrior(16), class_level("alchemist", 16)]),
            has_option("alchemist/discovery/greater-mutagen"),
        ])
    );
    assert_eq!(
        prerequisite("alchemist/discovery/feral-wings"),
        &Prerequisite::All(vec![
            Prerequisite::Any(vec![mutation_warrior(6), class_level("alchemist", 6)]),
            has_option("alchemist/discovery/feral-mutagen"),
        ])
    );
    // Game: NauseatingFlesh de112af9600e0e446af0c7ef0707e1c8: alchemist 12, a Mutation Warrior fighter 12 or an
    // Underground Chemist rogue 12.
    assert_eq!(
        prerequisite("alchemist/discovery/nauseating-flesh"),
        &Prerequisite::Any(vec![
            mutation_warrior(12),
            archetype_level("rogue", "rogue/underground-chemist", 12),
            class_level("alchemist", 12),
        ])
    );
}

/// Mummification needs an Underground Chemist's rogue 10, not its discovery feature.
#[test]
fn mummification_needs_an_underground_chemist_rogue_10() {
    // Game: Mummification daf854d84d442e941aa3a2fdc041b37c: PrerequisiteArchetypeLevel UndergroundChemistArchetype
    // b4f5741d42c4cb04abf5ab3fe33f6fc5 rogue 10 or alchemist 10, Group Any, and Preserve Organs.
    assert_eq!(
        prerequisite("alchemist/discovery/mummification"),
        &Prerequisite::All(vec![
            Prerequisite::Any(vec![
                archetype_level("rogue", "rogue/underground-chemist", 10),
                class_level("alchemist", 10),
            ]),
            has_option("alchemist/discovery/preserve-organs"),
        ])
    );
}

/// The three mercies and Sudden Speed have no prerequisite: their selections' levels gate them.
#[test]
fn the_three_mercies_and_sudden_speed_name_no_class_level() {
    // Game: MercyFatigued e5aa306af9b91974a9b2f2cbe702f562, MercyShaken c4732ca5b0af7214a98f9b4dafe87da9,
    // MercySickened 7ee2ef06226a4884f80b7647a2aa2dee and KiSuddenSpeedFeature 96d96f5b6f71004418e7a7e42a81c94d hold
    // no prerequisite component.
    for key in [
        "paladin/mercy/fatigued",
        "paladin/mercy/shaken",
        "paladin/mercy/sickened",
        "monk/ki-power/sudden-speed",
    ] {
        assert_eq!(prerequisite_of(key), None, "{key}");
    }
}

/// The Stalwart Defender's powers take the Stonelord's paladin levels, and Clear Mind is the rage power's.
#[test]
fn the_stalwart_defenders_powers_take_the_games_levels() {
    let stonelord = |level| archetype_level("paladin", "paladin/stonelord", level);
    // Game: FearlessDefenseDefensivePower 2c13bd43a7ed4844b9f4dcc919fd74f8: StalwartDefenderClass 4 or a Stonelord
    // paladin 8; IncreasedDamageReductionDefensivePower d10496e92d0799a40bb3930b8f4fda0d: 6 or 12, Group Any.
    assert_eq!(
        prerequisite("stalwart-defender/defensive-powers/fearless-defense"),
        &Prerequisite::Any(vec![stonelord(8), class_level("stalwart-defender", 4)])
    );
    assert_eq!(
        prerequisite("stalwart-defender/defensive-powers/increased-damage-reduction"),
        &Prerequisite::Any(vec![stonelord(12), class_level("stalwart-defender", 6)])
    );
    // Game: ClearMindFeature 4a236ae23d5a4f76b3616ddfa337e724 (the rage power's blueprint, which
    // StalwartDefenderDefensivePowerSelection 2cd91c501bda80b47ac2df0d51b02973 doesn't list: known.txt, 287): barbarian,
    // skald or shifter 8, or a Primalist bloodrager 8.
    assert_eq!(
        prerequisite("stalwart-defender/defensive-powers/clear-mind"),
        &Prerequisite::Any(vec![
            archetype_level("bloodrager", "bloodrager/primalist", 8),
            class_level("barbarian", 8),
            class_level("shifter", 8),
            class_level("skald", 8),
        ])
    );
}

/// The Divine Hunter's otherworldly companions name the alignments their blueprints allow.
#[test]
fn the_otherworldly_companions_name_their_alignments() {
    let any_of = |alignments: [Alignment; 6]| {
        Prerequisite::Any(alignments.map(Prerequisite::Alignment).to_vec())
    };
    // Game: OtherworldlyCompanionCelestial 3db2fd3394613b4438d3c844a0c034ca: PrerequisiteAlignment, any good or
    // neutral; OtherworldlyCompanionFiendish 4d7607a0155af7d43b49b785f2051e21: any neutral or evil.
    assert_eq!(
        prerequisite(
            "hunter/divine-hunter/otherworldly-companion/otherworldly-companion-celestial"
        ),
        &any_of([
            Alignment::LawfulGood,
            Alignment::NeutralGood,
            Alignment::ChaoticGood,
            Alignment::LawfulNeutral,
            Alignment::TrueNeutral,
            Alignment::ChaoticNeutral,
        ])
    );
    assert_eq!(
        prerequisite("hunter/divine-hunter/otherworldly-companion/otherworldly-companion-fiendish"),
        &any_of([
            Alignment::LawfulNeutral,
            Alignment::TrueNeutral,
            Alignment::ChaoticNeutral,
            Alignment::LawfulEvil,
            Alignment::NeutralEvil,
            Alignment::ChaoticEvil,
        ])
    );
}

/// Blinding Strike refuses Blinding Critical, Improved Evasion needs Evasion, and Swift Consume arcanist 1.
#[test]
fn blinding_strike_improved_evasion_and_swift_consume_take_the_games_conditions() {
    // Game: BlindingStrike fd01c15d9e74cd247b1fdbd6eb4d4713: PrerequisiteNoFeature BlindingCriticalFeature
    // 787e56055e3ef864d9c78a3ec21e56be, beside Advance Talents and character level 15.
    assert_eq!(
        prerequisite("rogue/rogue-talents/blinding-strike"),
        &Prerequisite::All(vec![
            not(Prerequisite::Feat(FeatKey::new("blinding-critical"))),
            has_feature("advanced-talents"),
            Prerequisite::CharacterLevel(15),
        ])
    );
    // Game: ImprovedEvasion 0d35d6c4d5eef8d4790d09bd9a874e57: PrerequisiteFeature Evasion
    // 576933720c440aa4d8d42b0c54b77e80, which the class feature and the rogue talent share, and Advance Talents.
    assert_eq!(
        prerequisite("rogue/rogue-talents/improved-evasion"),
        &Prerequisite::All(vec![
            Prerequisite::Any(vec![
                has_feature("evasion"),
                has_option("rogue/rogue-talents/evasion"),
            ]),
            has_feature("advanced-talents"),
        ])
    );
    // Game: ArcanistExploitSwiftConsumeFeature eaec53cbf34042f4e9f0d5451f9529d1: PrerequisiteClassLevel ArcanistClass 1.
    assert_eq!(
        prerequisite("arcanist/arcanist-exploits/swift-consume"),
        &class_level("arcanist", 1)
    );
}

/// The ranger's 6th-level style lists need the 2nd-level style in any of the planner's entries of its one blueprint.
/// (The 10th-level lists stay as they were: the 6th-level list they name is never held, 308e's note.)
#[test]
fn the_rangers_6th_level_styles_need_the_2nd_level_style() {
    // Game: RangerStyleArcherySelection6 6c799d09d5b93f344b9ade0e0c765c2d: PrerequisiteFeature
    // RangerStyleArcherySelection2, which is the ranger's Archery option, the Stormwalker's and the Nomad's level-2
    // archery feats and the nested Archery list at 2 in the planner.
    assert_eq!(
        prerequisite("ranger/combat-style-feats-6/archery"),
        &Prerequisite::Any(vec![
            has_option("ranger/combat-style-feat/archery-feats"),
            has_selection("ranger/bonus-feats-2b"),
            has_selection("ranger/combat-style-feat/archery-feats-2"),
        ])
    );
    // Game: RangerStyleMenacingSelection6 a9998f05a0472bb4f8950304fe26ec0c: RangerStyleMenacingSelection2
    // 1c5988e90e0657645972e7b28ba4ce8b.
    assert_eq!(
        prerequisite("ranger/combat-style-feats-6/menacing"),
        &Prerequisite::Any(vec![
            has_option("ranger/combat-style-feat/menacing-feats"),
            has_selection("ranger/combat-style-feat/menacing-feats-2"),
        ])
    );
}

/// The slayer's 6th- and 10th-level combat style talents need the style before and the level of a ranger, a slayer or
/// a Sanctified Slayer.
#[test]
fn the_slayers_later_combat_style_talents_need_the_style_before_and_the_level() {
    let level = |n| {
        Prerequisite::Any(vec![
            archetype_level("inquisitor", "inquisitor/sanctified-slayer", n),
            class_level("ranger", n),
            class_level("slayer", n),
        ])
    };
    // Game: RangerStyleSelection6 61f82ba786fe05643beb3cd3910233a8: PrerequisiteFeature RangerStyleSelection2
    // c6d0da9124735a44f93ac31df803b9a9, ranger, slayer or Sanctified Slayer 6 (Group Any), and its own NoFeature.
    assert_eq!(
        prerequisite("slayer/slayer-talents/combat-style-feat-6"),
        &Prerequisite::All(vec![
            level(6),
            Prerequisite::Any(vec![
                has_option("slayer/slayer-talents/combat-style-feat-2"),
                has_selection("ranger/combat-style-feat"),
            ]),
            not(Prerequisite::Any(vec![
                has_option("slayer/slayer-talents/combat-style-feat-6"),
                has_selection("ranger/combat-style-feats-6"),
            ])),
        ])
    );
    // Game: RangerStyleSelection10 78177315fc63b474ea3cbb8df38fafcd: RangerStyleSelection6, level 10.
    assert_eq!(
        prerequisite("slayer/slayer-talents/combat-style-feat-10"),
        &Prerequisite::All(vec![
            level(10),
            Prerequisite::Any(vec![
                has_option("slayer/slayer-talents/combat-style-feat-6"),
                has_selection("ranger/combat-style-feats-6"),
            ]),
            not(Prerequisite::Any(vec![
                has_option("slayer/slayer-talents/combat-style-feat-10"),
                has_selection("ranger/combat-style-feats-10"),
            ])),
        ])
    );
}
