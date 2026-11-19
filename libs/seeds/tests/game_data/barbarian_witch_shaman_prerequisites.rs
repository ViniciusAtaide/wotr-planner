//! The barbarian's, witch's, shaman's, kineticist's, aeon's and Hellknight's option prerequisites as the game's (ticket
//! 308f, written by `game-check --write prerequisites --only <the families' prefixes>`): one real-data test per
//! pattern the run changed. The rage powers' level gates, the Instinctual Warrior's monk powers and Flame Shield have
//! their tests in `skald_rage_powers`, `instinctual_rage_powers` and `wild_talents`, the totems' exclusion in
//! `report_gaps`.

use rules::{
    ArchetypeKey, ClassKey, FeatKey, FeatureKey, FeatureSelectionKey, Prerequisite,
    SelectionOptionKey,
};

use crate::support::*;

/// An option's prerequisite, if it has one.
fn prerequisite_of(key: &str) -> Option<&'static Prerequisite> {
    selection_option(key).prerequisite.as_ref()
}

/// The tree in the data's canonical order.
fn canonical(mut prerequisite: Prerequisite) -> Prerequisite {
    prerequisite.canonicalize();
    prerequisite
}

fn assert_prerequisite(key: &str, expected: Prerequisite) {
    assert_eq!(prerequisite_of(key), Some(&canonical(expected)), "{key}");
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

fn has_option(key: &str) -> Prerequisite {
    Prerequisite::SelectionOption(SelectionOptionKey::new(key))
}

fn has_selection(key: &str) -> Prerequisite {
    Prerequisite::Selection(FeatureSelectionKey::new(key))
}

fn has_feature(key: &str) -> Prerequisite {
    Prerequisite::Feature {
        feature: FeatureKey::new(key),
        count: 1,
    }
}

/// A rage power's level gate: barbarian, skald or shifter at `level`, or a Primalist bloodrager at it.
fn rage_gate(level: u8) -> Prerequisite {
    Prerequisite::Any(vec![
        archetype_level("bloodrager", "bloodrager/primalist", level),
        class_level("barbarian", level),
        class_level("shifter", level),
        class_level("skald", level),
    ])
}

/// The planner's two entries of ElementalFocusSelection 1f3a15a3ae8a5524ab8b97f469bf4e3d: the kineticist's element
/// and the Blood Kineticist's.
fn element_selection() -> Prerequisite {
    Prerequisite::Any(vec![
        has_selection("kineticist/blood-kineticist/element"),
        has_selection("kineticist/element"),
    ])
}

/// The plain and Greater totems need the totem before them and the level, and refuse no other line; the Lesser totems
/// keep refusing the other lines, which is how the game keeps one line per character.
#[test]
fn the_totems_need_the_one_before_them_and_refuse_no_line() {
    // Game: BeastTotemFeature c085888db293f2741b881cc989a2ab14, CelestialTotemFeature
    // 5156331dc888e9347ae6fc81ad3f3cec, DaemonTotemFeature d673c30720e8e7c4bb0903dc3c9ab649: PrerequisiteFeature of
    // their Lesser totem and the level gate of 6 (PrerequisiteClassLevel BarbarianClass, SkaldClass, ShifterClass;
    // PrerequisiteArchetypeLevel BloodragerClass PrimalistArchetype), no PrerequisiteNoFeature. BeastTotemGreaterFeature
    // 602d2684add008b4ba748af95c3f8b2e, CelestialTotemGreaterFeature 774f79845d1683a43aa42ebd2a549497,
    // DaemonTotemGreaterFeature 9a2f0ffe517d221459640a4ad85710d7: the plain totem and 10.
    for line in ["beast", "celestial", "daemon"] {
        let lesser = format!("barbarian/rage-power/{line}-totem-lesser");
        let plain = format!("barbarian/rage-power/{line}-totem");
        assert_prerequisite(
            &plain,
            Prerequisite::All(vec![rage_gate(6), has_option(&lesser)]),
        );
        assert_prerequisite(
            &format!("{plain}-greater"),
            Prerequisite::All(vec![rage_gate(10), has_option(&plain)]),
        );
    }
    // Game: FiendTotemFeature ce449404eeb4a7c499fbe0248056174f needs FiendTotemLesserFeature
    // 76437492f801f054ba536473ad2fde79; FiendTotemGreaterFeature 1105632657d94d940a43707a3a57b006 needs
    // FiendTotemFeature, which the Demon Dancer's row grants too (its `skald/demon-dancer/fiend-totem`).
    assert_prerequisite(
        "barbarian/rage-power/fiend-totem",
        Prerequisite::All(vec![
            rage_gate(6),
            has_option("barbarian/rage-power/fiend-totem-lesser"),
        ]),
    );
    assert_prerequisite(
        "barbarian/rage-power/fiend-totem-greater",
        Prerequisite::All(vec![
            rage_gate(10),
            Prerequisite::Any(vec![
                has_feature("skald/demon-dancer/fiend-totem"),
                has_option("barbarian/rage-power/fiend-totem"),
            ]),
        ]),
    );
    // Game: BeastTotemLesserFeature d99dfc9238a8a6646b32be09057c1729 refuses FiendTotemLesserFeature,
    // CelestialTotemLesserFeature aba61e0b0e66bf3439cc247ee89fddae and DaemonTotemLesserFeature
    // 45102fd7aab96f94d81ec80768549e12 (PrerequisiteNoFeature); so does each Lesser of the other three.
    for line in ["beast", "celestial", "daemon", "fiend"] {
        let lesser = format!("barbarian/rage-power/{line}-totem-lesser");
        for other in ["beast", "celestial", "daemon", "fiend"] {
            if other == line {
                continue;
            }
            let refused = not(has_option(&format!(
                "barbarian/rage-power/{other}-totem-lesser"
            )));
            assert!(
                matches!(prerequisite_of(&lesser), Some(Prerequisite::All(children)) if children.contains(&refused)),
                "{lesser} refuses {other}"
            );
        }
    }
}

/// Increased Damage Reduction refuses the Inciter, and it and Protect Vitals name no shifter level.
#[test]
fn increased_damage_reduction_refuses_the_inciter_and_names_no_shifter() {
    let no_shifter = |level| {
        Prerequisite::Any(vec![
            archetype_level("bloodrager", "bloodrager/primalist", level),
            class_level("barbarian", level),
            class_level("skald", level),
        ])
    };
    // Game: IncreasedDamageReductionFeature ddaee203ee4dcb24c880d633fbd77db6: barbarian, skald or Primalist bloodrager
    // 8, and PrerequisiteNoArchetype ProvocateurArchetype a1f5ab7eb78942f6889304acbd92e945 (the planner's Inciter).
    assert_prerequisite(
        "barbarian/rage-power/increased-damage-reduction",
        Prerequisite::All(vec![
            no_shifter(8),
            not(Prerequisite::Archetype(ArchetypeKey::new("skald/inciter"))),
        ]),
    );
    // Game: ProtectVitalsFeature efd53fe2887c3a54d86b99f4bba61dd6: GuardedStanceFeature
    // efb97e482f53f064dab85a9eeaf01085 and barbarian, skald or Primalist bloodrager 8.
    assert_prerequisite(
        "barbarian/rage-power/protect-vitals",
        Prerequisite::All(vec![
            no_shifter(8),
            has_option("barbarian/rage-power/guarded-stance"),
        ]),
    );
}

/// A shifter's levels open a rage power's gate: a shifter 4 with barbarian 2 takes Renewed Vigor in the barbarian's
/// rage power slot, and a fighter 4 with barbarian 2 can't. Before, only barbarian and skald levels counted.
#[test]
fn a_shifter_4_is_offered_renewed_vigor_through_its_level_gate() {
    // Game: RenewedVigorFeature aad496a7c9bee224cbd01a3cf8e42061: PrerequisiteClassLevel ShifterClass 4 among its
    // Group Any gates; ComeAndGetMeFeature 032402bde3fb47e39e6ebb6724ae9c22 the same gates at 12.
    assert_prerequisite("barbarian/rage-power/renewed-vigor", rage_gate(4));
    assert_prerequisite("barbarian/rage-power/come-and-get-me", rage_gate(12));
    let renewed_vigor = "barbarian/rage-power/renewed-vigor";
    let shifter = multiclass(&[("shifter", None, 4), ("barbarian", None, 2)]);
    let slot = slot_of(&shifter, 6, "barbarian/rage-power");
    assert_eq!(offered_in(&shifter, &slot)[renewed_vigor], Ok(()));
    let fighter = multiclass(&[("fighter", None, 4), ("barbarian", None, 2)]);
    let slot = slot_of(&fighter, 6, "barbarian/rage-power");
    assert!(offered_in(&fighter, &slot)[renewed_vigor].is_err());
}

/// The infusions take the levels and the element selection their blueprints name.
#[test]
fn the_infusions_take_the_levels_and_the_element_selection_their_blueprints_name() {
    let sharpshooter = |level| archetype_level("kineticist", SHARPSHOOTER, level);
    // Game: KineticBladeInfusion 9ff81732daddb174aa8138ad1297c787: PrerequisiteNoArchetype KineticSharpshooterArchetype
    // e0684d0a6ef44cae9d095f5318618a85 alone, no kineticist level.
    assert_prerequisite(
        &infusion("kinetic-blade"),
        not(Prerequisite::Archetype(ArchetypeKey::new(SHARPSHOOTER))),
    );
    // Game: ExtendedRangeInfusion cb2d9e6355dd33940b2bef49e544b0bf: ElementalFocusSelection
    // 1f3a15a3ae8a5524ab8b97f469bf4e3d and not the Kinetic Sharpshooter.
    assert_prerequisite(
        &infusion("extended-range"),
        Prerequisite::All(vec![
            element_selection(),
            not(Prerequisite::Archetype(ArchetypeKey::new(SHARPSHOOTER))),
        ]),
    );
    // Game: SpindleInfusion c4f4a62a325f7c14dbcace3ce34782b5 (4) and WallInfusion c684335918896ce4ab13e96cec929796
    // (10): kineticist level and ElementalFocusSelection.
    for (name, level) in [("spindle", 4), ("wall", 10)] {
        assert_prerequisite(
            &infusion(name),
            Prerequisite::All(vec![element_selection(), class_level("kineticist", level)]),
        );
    }
    // Game: ExplodingArrowsInfusion 7c8ded7015114e4086b3dbdd16388b29: kineticist 3, a Kinetic Sharpshooter 3 and
    // ElementalFocusSelection; ClusterArrowsInfusion 8896ac251d394dceaf650793aa85e177 (the planner's Rending Arrows):
    // kineticist 11 and a Kinetic Sharpshooter 11.
    assert_prerequisite(
        &infusion("exploding-arrows"),
        Prerequisite::All(vec![
            sharpshooter(3),
            element_selection(),
            class_level("kineticist", 3),
        ]),
    );
    // ArmorPiercingInfusion cf531860c9fb44c09bda9d8904657314 (the planner's Honed Infusion) and ChainArrowsInfusion
    // 0ff81c5691c64c4aa8b2ecc35c885901: the same pair at 12 and 16.
    for (name, level) in [
        ("rending-arrows", 11),
        ("honed-infusion", 12),
        ("chain-arrows", 16),
    ] {
        assert_prerequisite(
            &infusion(name),
            Prerequisite::All(vec![sharpshooter(level), class_level("kineticist", level)]),
        );
    }
    // Game: GrapplingInfusionFeature e2d70b95e80549b439d30df29a79cb58: PrerequisiteFeaturesFromList Amount 1 of
    // WallInfusion, DeadlyEarthInfusion 061f5d7e659432b478668b70f6d4caae and CloudInfusion
    // 3c53ee4965a13d74e81b37ae34f0861b, and kineticist 10.
    assert_prerequisite(
        &infusion("grappling-infusion"),
        Prerequisite::All(vec![
            Prerequisite::AtLeast(
                1,
                vec![
                    has_option(&infusion("cloud")),
                    has_option(&infusion("deadly-earth")),
                    has_option(&infusion("wall")),
                ],
            ),
            class_level("kineticist", 10),
        ]),
    );
}

/// The wild talents name their blueprints' features and levels, not an element: Skill Focus, Skilled Kineticist and
/// Elemental Whispers have no level (their slot's level gates them), the healing talents need Kinetic Healer, and
/// Jagged Flesh Flesh of Stone.
#[test]
fn the_wild_talents_name_their_features_and_levels_not_an_element() {
    // Game: SkillFocusSelection c9629ef9eebb88b479b2fbc5e836656a has no prerequisite; SkilledKineticistFeature
    // fd13e9efee08db448974fe0263eb96c8 and ElementalWhispersSelection f02525006521bee4eb90ab26b7b9db24 only their own
    // PrerequisiteNoFeature.
    assert_eq!(prerequisite_of(&talent("skill-focus")), None);
    assert_eq!(prerequisite_of(&talent("skilled-kineticist")), None);
    assert_prerequisite(
        &talent("elemental-whispers"),
        not(has_option(&talent("elemental-whispers"))),
    );
    // Game: HealingBUrstFeature c73b37aaa2b82b44686c56db8ce14e7f (8) and KineticRevificationFeature
    // 0377fcf4c10871f4187809d273af7f5d (12): PrerequisiteFeaturesFromList Amount 1 of KineticHealerFeature
    // 3ef666973adfa8f40af6c0679bd98ba5 and the kineticist level, no element.
    for (name, level) in [("healing-burst", 8), ("kinetic-revivification", 12)] {
        assert_prerequisite(
            &talent(name),
            Prerequisite::All(vec![
                has_option(&talent("kinetic-healer")),
                class_level("kineticist", level),
            ]),
        );
    }
    // Game: JaggedFleshFeature 94064ed53b1020247941ac70313b439d: FleshOfStoneFeature a275b35f282601944a97e694f6bc79f8
    // and kineticist 6, no element.
    assert_prerequisite(
        &talent("jagged-flesh"),
        Prerequisite::All(vec![
            class_level("kineticist", 6),
            has_feature("kineticist/element/earth/flesh-of-stone"),
        ]),
    );
}

/// The witch's and the shaman's Iceplant refuse each other, and Intimidating Display refuses Dazzling Display.
#[test]
fn the_hexes_refuse_their_twins() {
    // Game: WitchHexIceplantFeature 9af6dc2697c8ef24eab8ce1c697b05c4 and ShamanHexIceplantFeature
    // 999e367bbabc42d46bf864d3f4bda86d: each a PrerequisiteNoFeature of the other.
    assert_prerequisite("witch/hex/iceplant", not(has_option("shaman/hex/iceplant")));
    assert_prerequisite("shaman/hex/iceplant", not(has_option("witch/hex/iceplant")));
    // Game: ShamanHexIntimidatingDisplayFeature 2eacbef4be7c6434e9cfaa9632fdc429: PrerequisiteNoFeature
    // DazzlingDisplayFeature bcbd674ec70ff6f4894bb5f07b6f4095.
    assert_prerequisite(
        "shaman/hex/intimidating-display",
        not(Prerequisite::Feat(FeatKey::new("dazzling-display"))),
    );
}

/// The Prophet of Pestilence's plague mutations need the archetype's shaman level, and those naming a list need one
/// of it.
#[test]
fn the_plague_mutations_need_the_prophet_of_pestilences_shaman_level() {
    let mutation = |name: &str| {
        format!("shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-{name}")
    };
    let prophet = |level| archetype_level("shaman", "shaman/prophet-of-pestilence", level);
    // Game: PlagueHexAreaFeature 738a731ca6be428c9d49f0335da36312 (10), PlagueHexIncreaseDamageStepFeature
    // 505648df95f5489fa13ddbd1af9473fd (10), PlagueHexNegativeLevelFeature a9184917ef1c4b358ba03da5f6b03931 (14),
    // PlagueHexIncreaseDamageDiceFeature c0e30cfef3994c7e992dd8146139868f (18): PrerequisiteArchetypeLevel ShamanClass
    // ProphetOfPestilenceArchetype bcd758a75fb54651a7f668fe2661a307 alone.
    for (name, level) in [
        ("contagiousness", 10),
        ("new-strain", 10),
        ("withering", 14),
        ("virulence", 18),
    ] {
        assert_prerequisite(&mutation(name), prophet(level));
    }
    // Game: PlagueHexStrengthWeakenedFeature c6c6c70d65e94606932f13904dddec41: the level 10 and
    // PlagueHexStrengthDamageFeature ed39cb73232b4473934fe932bd7c0d58.
    assert_prerequisite(
        &mutation("muscle-feebleness"),
        Prerequisite::All(vec![prophet(10), has_option(&mutation("strength-damage"))]),
    );
    // Game: PlagueHexPhysicalImpairedFeature 811e379c217b4caa9b229fed765c977d: the level 14 and
    // PrerequisiteFeaturesFromList Amount 1 of the Strength, Dexterity and Constitution damage, not all three.
    assert_prerequisite(
        &mutation("erode-body"),
        Prerequisite::All(vec![
            prophet(14),
            Prerequisite::AtLeast(
                1,
                vec![
                    has_option(&mutation("constitution-damage")),
                    has_option(&mutation("dexterity-damage")),
                    has_option(&mutation("strength-damage")),
                ],
            ),
        ]),
    );
}

/// The Aeon's gazes name no mythic rank: the gaze lists of ranks 5 and 8 gate them.
#[test]
fn the_aeons_gazes_name_no_rank() {
    // Game: AeonAreaEffectsGazeFeature 39d8ae51c0034adbb8394210c51901e2, AeonTemporalJumpGazeFeature
    // 72514d0dcd054d1f8bb9abd4ddb9711c and the seven others have no Prerequisite component; AeonGazeSecondSelection
    // 7d498e034d18ca94baea19d7edee7403 lists them from rank 5.
    for gaze in [
        "area-effects",
        "concentration",
        "hit-points",
        "normalize-size",
        "power-of-law",
        "primordial-magic",
        "relativity-of-space",
        "summoners-gaze",
        "temporal-jump",
    ] {
        assert_eq!(
            prerequisite_of(&format!("aeon/enforcing-gaze/enforcing-gaze-{gaze}")),
            None,
            "{gaze}"
        );
    }
    // The rank-5 list isn't offered at MR 4.
    let build = mythic("aeon", 14);
    let offered: Vec<String> = engine()
        .options(&build, &path_selection(4))
        .unwrap()
        .into_iter()
        .map(|option| option.pick.key().to_owned())
        .collect();
    assert!(!offered.contains(&"aeon/enforcing-gaze/enforcing-gaze-area-effects".to_owned()));
}

/// Pentamic Faith refuses itself and the planner's selection of its blueprint, and needs the Order of the Godclaw.
#[test]
fn pentamic_faith_refuses_itself_and_its_selection() {
    // Game: HellknightDisciplinePentamicFaith b9750875e9d7454e85347d739a1bc894: PrerequisiteFeature
    // HellKnightOrderOfTheGodclaw 5636564c278583342aec54eb2b409029 and its own PrerequisiteNoFeature, a blueprint the
    // planner maps to the option and to the selection `hellknight/pentamic-faith`.
    assert_prerequisite(
        "hellknight/disciplines/pentamic-faith",
        Prerequisite::All(vec![
            not(Prerequisite::Any(vec![
                has_option("hellknight/disciplines/pentamic-faith"),
                has_selection("hellknight/pentamic-faith"),
            ])),
            has_option("fighter/armiger/order/order-of-the-godclaw"),
        ]),
    );
}
