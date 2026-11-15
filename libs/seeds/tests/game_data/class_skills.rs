//! Options' class skills (ticket 37) and the class-skill cleanup (ticket 38).

use std::collections::{BTreeSet, HashMap};

use rules::{
    Build, CharacterState, Engine, FeatKey, FeatSource, FeatureKey, GameData, Pick, Prerequisite,
    SkillKey, SlotId,
};

use crate::support::*;

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
    // Game: OracleClass 20ce9bf8af32bee4c8557a045ab499b1
    assert_eq!(class_skills_at_1("oracle", None, None), oracle);
    // Each mystery's AddClassSkill list, less the oracle's own skills:
    for (mystery, skills) in [
        ("battle", &["perception"][..]), // Game: OracleBattleMysteryFeature 6b8e676943cb91648b21b7ca72fbb833
        ("bones", &["stealth"]), // Game: OracleBonesMysteryFeature 067b175d3df0d1a408efd7eee2b36b9b
        ("flame", &["athletics", "mobility"]), // Game: OracleFlameMysteryFeature 3b68909df737cd4458509d7f3a9c3706
        ("nature", &["athletics"]), // Game: OracleNatureMysteryFeature 69579bfe28e15b942af0722021d8725c
        ("stone", &["athletics"]), // Game: OracleStoneMysteryFeature 7bb4bb3e7fd26f34e8ca035a27e03e85
        ("waves", &["athletics", "mobility"]), // Game: OracleWavesMysteryFeature e2f8bd7c45dfb954c8c42b168505c783
        ("wind", &["mobility", "stealth"]), // Game: OracleWindMysteryFeature 7c1fdd831af747b47bb2cce0051f309b
        ("ancestor", &[]), // Game: OracleAncestorsMysteryFeature 9936554e9372dab4a874c1dd165fb6f8
        ("life", &[]),     // Game: OracleLifeMysteryFeature a5458a1c316d1f24e8d9770f4fc179fa
    ] {
        let held = class_skills_at_1("oracle", None, Some(&format!("oracle/mystery/{mystery}")));
        let expected: BTreeSet<String> = oracle.iter().cloned().chain(skill_set(skills)).collect();
        assert_eq!(held, expected, "{mystery}");
    }
    // A Divine Herbalist gains no class skills from its mystery.
    // Game: DivineHerbalistArchetype 204117c48168f2a41a8e155419e6e880 (DivineHerbalistMysterySelection
    // c11ff5dbd8518c941849b3112d4d6b68's features have no AddClassSkill)
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
    // Game: KineticistClass 42a455d9ec1ad924d889272429eb8391
    assert_eq!(class_skills_at_1("kineticist", None, None), kineticist);
    for (element, skills) in [
        ("air", &["lore-nature"][..]), // Game: AirClassSkills 5f6197b9fde96d840882b858208d2809
        ("earth", &["athletics", "knowledge-world"]), // Game: EarthClassSkills 5c0916592017fbd419856df54db630f7
        ("fire", &["lore-nature"]), // Game: FireClassSkills e76a453abe7a7304cbde17a8c123ad76
        ("water", &["athletics", "lore-nature"]), // Game: WaterClassSkills 96e9396ffce22de498a0d29a0f14117b
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
    // Game: CavalierClass 3adc3439f98cb534ba98df59838f02c7
    assert_eq!(class_skills_at_1("cavalier", None, None), cavalier);
    for (order, skill) in [
        ("order-of-the-lion", "knowledge-world"), // Game: CavalierLionSkills 7c00cd81a50bd4944bce094d870c8f5e
        ("order-of-the-shroud", "lore-religion"), // Game: CavalierShroudSkills 3fc4db417af250f4ea0b81db21f2f7e0
        ("order-of-the-star", "lore-religion"), // Game: CavalierStarSkills 30cab0aee32cd86429f5bdf304291f3a
        ("order-of-the-sword", "lore-religion"), // Game: CavalierSwordSkills 0e333ad0b9537d0478c771cd0fe38e66
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
    // Calistria's (ticket 86).
    let cleric = faithful(cleric);
    assert!(engine().is_possible(&cleric));
    // Game: ClericClass 67819271767a9dd4fbfd4ae700befea0; KnowledgeDomainBaseFeature 5335f015063776d429a0b5eab97eb060;
    // TrickeryDomainBaseFeature cd1f4a784e0820647a34fe9bd5ffa770
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
    // Game: SorcererClass b3a505fb61437dc4097f43c3f8f9a4cf
    assert_eq!(class_skills_at_1("sorcerer", None, None), sorcerer);
    for (bloodline, skill) in [
        ("abyssal", "athletics"), // Game: BloodlineAbyssalClassSkill 58e97e23349f115408d768ff7ad55c43
        ("celestial", "lore-religion"), // Game: BloodlineCelestialClassSkill b0520c35f31ba544189d3293ed14dd67
        ("draconic-red", "perception"), // Game: BloodlineDraconicClassSkill fb4615bd462372a44b778ac9f035675f
        ("elemental-fire", "mobility"), // Game: BloodlineElementalClassSkill 582a0ee43b0a38c4581c96cec8ddcdfd
        ("fey", "lore-nature"), // Game: BloodlineFeyClassSkill 243f474ce797acb4086c2dbc58660c4a
        ("infernal", "knowledge-world"), // Game: BloodlineInfernalClassSkill f07a37a5b245304429530842cb65e213
        ("serpentine", "stealth"), // Game: BloodlineSerpentineClassSkill 70e2242c7d52a1b428938d406adfd8ef
        ("undead", "lore-religion"), // Game: BloodlineUndeadClassSkill 461a5980bfe5579468ba99d369f75308
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
    for selection in DATA.class_selections() {
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
    rows.sort();
    rows
}

/// Every option row's class skills are on its first row. The Animal domain's, which the data has on its 4th, wait on a
/// ticket (`the_animal_domains_lore_nature_comes_with_the_domain_at_its_first_level`).
#[test]
fn options_class_skills_are_on_their_first_row() {
    let rows: Vec<_> = option_row_class_skills()
        .into_iter()
        .filter(|(option, ..)| option != "cleric/domain/animal")
        .collect();
    let mut expected: Vec<(String, u8, Vec<String>)> = Vec::new();
    let mut add = |option: String, skills: &[&str]| {
        expected.push((option, 1, skills.iter().map(|s| (*s).to_owned()).collect()));
    };
    // Game: the bloodlines', mysteries', domains', elements' and orders' class skill features cited in the tests above
    // (the draconic and elemental colours share BloodlineDraconicClassSkill fb4615bd462372a44b778ac9f035675f and
    // BloodlineElementalClassSkill 582a0ee43b0a38c4581c96cec8ddcdfd), each at the option's first level
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
        (
            "knowledge",
            &[
                "knowledge-arcana",
                "knowledge-world",
                "lore-nature",
                "lore-religion",
            ][..],
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
    // Game: DemonicCurseFeatureLevel1 b256f446392d1bc489ed9c1bc79e89b7 and HellboundCurseFeatureLevel1
    // 0ff5141fe79c43345a1c8ff72b75923d, each `AddClassSkill(SkillPersuasion)`, at the curse progression's level 1.
    add("oracle/oracles-curse/demonic".into(), &["persuasion"]);
    add("oracle/oracles-curse/hellbound".into(), &["persuasion"]);
    expected.sort();
    assert_eq!(rows, expected);
}

/// The Animal domain's Lore (Nature) is on the domain's base feature, given at the domain's first level, not with the
/// companion at its 4th.
// Game: AnimalDomainProgression 23d2f87aa54c89f418e68e790dba11e0 (AnimalDomainBaseFeature
// d577aba79b5727a4ab74627c4c6ba23c at level 1, with AddClassSkill LoreNature)
#[test]
#[ignore = "waits on ticket 261: the game gives the Animal domain's Lore (Nature) at the domain's level 1"]
fn the_animal_domains_class_skill_is_on_its_first_row() {
    let animal: Vec<_> = option_row_class_skills()
        .into_iter()
        .filter(|(option, ..)| option == "cleric/domain/animal")
        .collect();
    assert_eq!(
        animal,
        [(
            "cleric/domain/animal".to_owned(),
            1,
            vec!["lore-nature".to_owned()]
        )]
    );
}

#[test]
fn no_feature_stands_for_a_fixed_class_skill() {
    let features: Vec<(&str, &str)> = DATA
        .class_features()
        .map(|feature| (feature.key.as_str(), feature.name.as_str()))
        .filter(|(_, name)| name.starts_with("Class Skill") || *name == "Skills")
        .collect();
    // What's left is a choice of one of three skills (ticket 37's first open question, left as features).
    // Game: BloodlineArcaneClassSkillSelection 19ab16dba857d1a4ba617074f203f975 (Lore (Religion), Knowledge (World) or
    // Lore (Nature)), which SageBloodlineProgression 7d990675841a7354c957689a6707c6c2 holds too; that the data keeps
    // them as features is its shape, not a game fact
    assert_eq!(
        features,
        [
            ("sorcerer/bloodline/arcane/class-skill", "Class Skill"),
            ("sorcerer/sage-sorcerer/class-skill", "Class Skill"),
        ]
    );
    // The archetypes whose feature went hold the skill through their own list.
    // Game: SoheiArchetype fad7c56737ed12e42aacc330acc86428 (SoheiLoreNatureFeature d59fe46a1aa1ace4e987d843c8e309ff);
    // EmpyrealBloodlineProgression 8a95d80a3162d274896d50c2f18bb6b1 (BloodlineCelestialClassSkill);
    // SylvanBloodlineProgression a46d4bd93601427409d034a997673ece (BloodlineFeyClassSkill)
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

/// The bloodlines a sorcerer archetype's class skills are checked with, and each one's class skill.
const BLOODLINE_SKILLS: [(&str, &str); 8] = [
    ("abyssal", "athletics"),
    ("celestial", "lore-religion"),
    ("draconic-gold", "perception"),
    ("elemental-water", "mobility"),
    ("fey", "lore-nature"),
    ("infernal", "knowledge-world"),
    ("serpentine", "stealth"),
    ("undead", "lore-religion"),
];

/// A sorcerer 1 of `archetype` with `bloodline`, at level 1.
fn archetype_with_bloodline(archetype: &str, bloodline: &str) -> CharacterState {
    let build = with_option(
        archetype_build("sorcerer", archetype, 1),
        1,
        0,
        &format!("sorcerer/bloodline/{bloodline}"),
    );
    assert!(possible(&build), "{archetype} {bloodline}");
    at(&build, 1)
}

fn class_skills_of(state: &CharacterState) -> BTreeSet<String> {
    state.class_skills.iter().map(|s| s.to_string()).collect()
}

/// A Seeker Sorcerer's Trapfinding makes Trickery a class skill, beside the sorcerer's and the bloodline's.
// Game: SeekerArchetype 7229db6fc0b07af4180e783eed43c4d9 adds SorcererSeekerTrapfinding
// b1c4f14bc87534e4da7ecd8e0b0658d2 at 1 (AddClassSkill Thievery, the planner's Trickery); its bloodlines (e.g.
// SeekerBloodlineAbyssalProgression 17b752be1e0f4a34e8914df52eebeb75) keep the bloodline's class skill
#[test]
#[ignore = "waits on ticket 261: the game gives a Seeker Sorcerer Trickery as a class skill"]
fn a_seeker_sorcerer_holds_trickery_and_its_bloodlines_class_skill() {
    let sorcerer = skill_set(&["knowledge-arcana", "persuasion", "use-magic-device"]);
    for (bloodline, skill) in BLOODLINE_SKILLS {
        let held = class_skills_of(&archetype_with_bloodline(
            "sorcerer/seeker-sorcerer",
            bloodline,
        ));
        let expected: BTreeSet<String> = sorcerer
            .iter()
            .cloned()
            .chain([skill.to_owned(), "trickery".to_owned()])
            .collect();
        assert_eq!(held, expected, "{bloodline}");
    }
}

#[test]
fn an_overwhelming_mage_and_a_seeker_hold_their_bloodlines_class_skill_and_no_class_skill_feature()
{
    let sorcerer = skill_set(&["knowledge-arcana", "persuasion", "use-magic-device"]);
    for archetype in ["sorcerer/overwhelming-mage", "sorcerer/seeker-sorcerer"] {
        for (bloodline, skill) in BLOODLINE_SKILLS {
            let state = archetype_with_bloodline(archetype, bloodline);
            let held = class_skills_of(&state);
            let expected: BTreeSet<String> =
                sorcerer.iter().cloned().chain([skill.to_owned()]).collect();
            // Game: OverwhelimgMageArchetype 734917ec8b8542378c1d35519439d67f (keeps SorcererBloodlineSelection, adds
            // no class skill). The Seeker holds these and, in the game, Trickery too, which waits on a ticket
            // (`a_seeker_sorcerer_holds_trickery_and_its_bloodlines_class_skill`).
            if archetype == "sorcerer/overwhelming-mage" {
                assert_eq!(held, expected, "{archetype} {bloodline}");
            } else {
                assert!(held.is_superset(&expected), "{archetype} {bloodline}");
            }
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
        align(&mut monk, rules::Alignment::LawfulNeutral);
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
    // Game: MonkClass e8f21e5b58e0569468e420ebea456124; SoheiLoreNatureFeature d59fe46a1aa1ace4e987d843c8e309ff
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
    let edited = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == class)
        .unwrap();
    assert!(edited.class_skills.remove(&SkillKey::new(skill)));
    let checked = data.checked().unwrap_or_else(|problems| {
        panic!("the game data with {skill} taken off {class}'s list has problems: {problems:#?}")
    });
    Engine::new(checked)
}

/// A cleric 4 of the Animal and Fire domains.
fn animal_cleric() -> Build {
    cleric(["animal", "fire"], 4)
}

/// A Separatist 6 of Fire, with Animal as the second domain.
fn animal_separatist() -> Build {
    with_option(
        with_option(
            archetype_build("cleric", "cleric/separatist", 6),
            1,
            0,
            "cleric/domain/fire",
        ),
        1,
        1,
        "cleric/domain/animal",
    )
}

/// A druid 4 whose Nature Bond is the Animal domain.
fn animal_druid() -> Build {
    with_pick(
        with_option(druid(None, 4), 1, 0, "druid/nature-bond/domain"),
        option_slot(1, 0),
        "cleric/domain/animal",
    )
}

/// An inquisitor 4 of the Animal domain.
fn animal_inquisitor() -> Build {
    with_option(build("inquisitor", 4), 1, 0, "cleric/domain/animal")
}

/// The Animal domain's companion comes at the holder's 4th domain level; a druid's and an inquisitor's own lists hold
/// Lore (Nature) from level 1. When the domain's Lore (Nature) comes waits on a ticket
/// (`the_animal_domains_lore_nature_comes_with_the_domain_at_its_first_level`).
#[test]
fn the_animal_domains_companion_comes_at_the_holders_4th_domain_level() {
    let animal = animal_cleric();
    assert!(engine().is_possible(&animal));
    // Game: AnimalDomainProgression 23d2f87aa54c89f418e68e790dba11e0 (the companion selection at 4)
    from_level(&animal, 4, "cleric/domain/animal/animal-companion");
    // Game: DruidClass 610d836f3a3a9ed42a4349b62f002e96 and InquisitorClass f1a70d9e1b0b41e49874e1fa9052a1ce (Lore
    // (Nature) on their own lists)
    let druid = animal_druid();
    assert!(engine().is_possible(&druid));
    assert!(has_class_skill(&at(&druid, 1), "lore-nature"));
    let inquisitor = animal_inquisitor();
    assert!(engine().is_possible(&inquisitor));
    assert!(has_class_skill(&at(&inquisitor, 1), "lore-nature"));
}

/// The Animal domain's Lore (Nature) comes with the domain's base feature at the domain's first level: a cleric's
/// from cleric 1, a Separatist's second domain's from cleric 1 too, and a druid's and an inquisitor's from their level
/// 1 (seen with Lore (Nature) taken off their own lists).
// Game: AnimalDomainProgression 23d2f87aa54c89f418e68e790dba11e0, AnimalDomainProgressionSeparatist
// e3d58ae932554987a6e247e281efc2f0 and AnimalDomainProgressionDruid a75ad4936e099c54881cf553e2110703 (each gives
// AnimalDomainBaseFeature d577aba79b5727a4ab74627c4c6ba23c, AddClassSkill LoreNature, at level 1)
#[test]
#[ignore = "waits on ticket 261: the game gives the Animal domain's Lore (Nature) at its first level, cleric 1 for a Separatist too"]
fn the_animal_domains_lore_nature_comes_with_the_domain_at_its_first_level() {
    assert!(has_class_skill(&at(&animal_cleric(), 1), "lore-nature"));
    assert!(has_class_skill(&at(&animal_separatist(), 1), "lore-nature"));
    for (class, build) in [
        ("druid", animal_druid()),
        ("inquisitor", animal_inquisitor()),
    ] {
        let engine = engine_without_class_skill(class, "lore-nature");
        let sheet = engine.evaluate(&build);
        assert!(
            has_class_skill(sheet.at(1).unwrap(), "lore-nature"),
            "{class}"
        );
    }
}

/// Every feature a prerequisite names in the real data, wherever the prerequisite sits.
fn prerequisite_features(data: &GameData) -> BTreeSet<FeatureKey> {
    data_prerequisites(data)
        .into_iter()
        .flat_map(Prerequisite::leaves)
        .filter_map(|leaf| match leaf {
            Prerequisite::Feature { feature, .. } => Some(feature.clone()),
            _ => None,
        })
        .collect()
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
    // The reason names only the alternatives the character's classes give, all of them when none does. The fighter's
    // agrees with the game (AdvancedWeaponTraining1 3aa4cbdd4af5ba54888b0dc7f07f80c4: WeaponTrainingSelection and
    // Fighter 5); the Sohei's and the Disciple of the Pike's alternatives wait on a ticket
    // (`advanced_weapon_training_needs_the_fighters_weapon_training`). The texts are the engine's.
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
}

/// Advanced Weapon Training needs the fighter's Weapon Training and fighter level 5: the Sohei's and the Disciple of
/// the Pike's weapon training are other blueprints and don't meet it.
// Game: AdvancedWeaponTraining1 3aa4cbdd4af5ba54888b0dc7f07f80c4 (WeaponTrainingSelection, Fighter 5);
// SoheiWeaponTrainingSelection 5f08ae15929f02341a3f328449d443f3 and DIscipleOfThePikeWeaponTrainingSelection
// bc050eac79b34d4abf63e2f1152badad aren't named
#[test]
#[ignore = "waits on ticket 263: the game's Advanced Weapon Training needs the fighter's Weapon Training and fighter 5 only"]
fn advanced_weapon_training_needs_the_fighters_weapon_training() {
    let prerequisite = feat("advanced-weapon-training")
        .prerequisite
        .clone()
        .unwrap();
    let features: BTreeSet<&str> = prerequisite
        .leaves()
        .filter_map(|leaf| match leaf {
            Prerequisite::Feature { feature, .. } => Some(feature.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(features, BTreeSet::from(["fighter/weapon-training"]));
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
    // Game: CleaveFeature d809b6c4ff2aaff4fa70d712a70f7d7b (Power Attack and Str 13; the fighter 20 has Str 13)
    assert_eq!(
        engine.issue_text(&cleave.status.unwrap_err()).as_deref(),
        Some("Needs Power Attack")
    );
}
