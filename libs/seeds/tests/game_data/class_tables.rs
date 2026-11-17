//! The class tables the game gives (ticket 293, stage S1): saves, base attack bonus, skill points and class skills of
//! the classes and archetypes the game check found differing (tickets 261 and 262). Each expected value is the
//! game's, read from the class's or archetype's stat progressions and class-skill fields; the run that wrote
//! them is ticket 293l.

use rules::{ArchetypeKey, ClassKey, Save};

use crate::support::*;

/// One save at one class level of a class, taken with an archetype or none, as the data gives it. Prestige classes
/// use the prestige formula.
fn save_of(class: &str, archetype: Option<&str>, save: Save, level: u8) -> u8 {
    let key = ClassKey::new(class);
    let class = DATA
        .class(&key)
        .unwrap_or_else(|| panic!("no class {key:?}"));
    let prestige = DATA
        .prestige_classes
        .iter()
        .any(|prestige| prestige.key == key);
    let archetype = archetype.map(|archetype| {
        let key = ArchetypeKey::new(archetype);
        DATA.archetypes
            .iter()
            .find(|candidate| candidate.key == key)
            .unwrap_or_else(|| panic!("no archetype {key:?}"))
    });
    class.save_bonus(save, level, prestige, archetype)
}

/// The game's save tables: `BlueprintStatProgression.Bonuses` by level, SavesHigh ff4662bde9e75f145853417313842751
/// (0, 2, 3, 3, 4, …, 12 at 20), SavesLow dc0c7c1aba755c54f96c089cdf7d14a3 (0, 0, 0, 1, …, 2 at 7), SavesPrestigeHigh
/// 1f309006cd2855e4e91a6c3707f3f700 (0, 1, 1, 2, 2, …) and SavesPrestigeLow dc5257e1100ad0d48b8f3b9798421c72
/// (0, 0, 1, 1, 1, …); WrathDecompiled ClassData.FortitudeSave, ReflexSave and WillSave (the archetype's when set).
#[test]
fn the_save_tables_are_the_games() {
    use Save::{Fortitude, Reflex, Will};
    // (class, archetype, save, class level, the game's bonus), each row's table cited
    let rows: &[(&str, Option<&str>, Save, u8, u8)] = &[
        // PaladinClass bfa11238e7ae3544bbeb4d0b92e897ec: Will SavesHigh
        ("paladin", None, Will, 20, 12),
        // WarpriestClass 30b5e47d47a0e37438cc5a80c96cfb99: Will SavesHigh
        ("warpriest", None, Will, 20, 12),
        // HunterClass 34ecd1b5e1b90b9498795791b0855239: Will SavesLow
        ("hunter", None, Will, 7, 2),
        // ShifterClass a406d6ebea5c46bba3160246be03e96f: Reflex SavesHigh
        ("shifter", None, Reflex, 2, 3),
        // ArcaneTricksterClass 9c935a076d4fe4d4999fd48d853e3cf3, LoremasterClass 4a7c05adfbaf05446a6bf664d28fb103,
        // MysticTheurgeClass 0920ea7e4fd7a404282e3d8b0ac41838, StudentOfWarClass 46b1fe9759ea01f4d883b23d8f0aecbb,
        // WinterWitchClass eb24ca44debf6714aabe1af1fd905a07: Will SavesPrestigeHigh
        ("arcane-trickster", None, Will, 4, 2),
        ("loremaster", None, Will, 4, 2),
        ("mystic-theurge", None, Will, 4, 2),
        ("student-of-war", None, Will, 4, 2),
        ("winter-witch", None, Will, 4, 2),
        // StalwartDefenderClass d5917881586ff1d4d96d5b7cebda9464: SavesPrestigeHigh, SavesPrestigeLow,
        // SavesPrestigeHigh
        ("stalwart-defender", None, Fortitude, 1, 1),
        ("stalwart-defender", None, Reflex, 1, 0),
        ("stalwart-defender", None, Will, 1, 1),
        // PriestOfBalanceArchetype a4560e3fb5d247d68fb1a2738fcc0855: Fortitude SavesLow
        ("cleric", Some("cleric/priest-of-balance"), Fortitude, 1, 0),
        // DarkElementalistArchetype f12f18ae8842425489d29f302e69134c: SavesLow, SavesLow, SavesHigh
        (
            "kineticist",
            Some("kineticist/dark-elementalist"),
            Fortitude,
            1,
            0,
        ),
        (
            "kineticist",
            Some("kineticist/dark-elementalist"),
            Reflex,
            1,
            0,
        ),
        (
            "kineticist",
            Some("kineticist/dark-elementalist"),
            Will,
            1,
            2,
        ),
        // TraditionalMonk 8f3242d209994884bbb6c0999660f4df: Will SavesHigh
        ("monk", Some("monk/traditional-monk"), Will, 1, 2),
    ];
    for &(class, archetype, save, level, game) in rows {
        assert_eq!(
            save_of(class, archetype, save, level),
            game,
            "{class} {archetype:?} {save:?} at {level}"
        );
    }
}

/// The game's Feyspeaker has BABLow 0538081888b2d8c41893d25d098dee99 (0, 0, 1, 1, 2, …: 1 at 3) where the druid has BABMedium
/// 4c936de4249b61e419a3fb775b9f2581 (0, 0, 1, 2, 3, …: 2 at 3); DruidClass 610d836f3a3a9ed42a4349b62f002e96,
/// FeyspeakerArchetype da69747aa3dd0044ebff5f3d701cdde3 (`m_BaseAttackBonus` BABLow).
#[test]
fn the_feyspeakers_bab_is_low() {
    let druid = DATA.class(&ClassKey::new("druid")).expect("the druid");
    let feyspeaker = DATA
        .archetypes
        .iter()
        .find(|archetype| archetype.key == ArchetypeKey::new("druid/feyspeaker"))
        .expect("the Feyspeaker");
    assert_eq!(druid.base_attack_bonus(3, None), 2);
    assert_eq!(druid.base_attack_bonus(3, Some(feyspeaker)), 1);
}

/// The skill points per level: ArcaneTricksterClass 9c935a076d4fe4d4999fd48d853e3cf3 (`m_SkillPoints` 3), and the
/// fighter's 2 (FighterClass) plus `AddSkillPoints` 1 on DefenderOfTheHearthArchetype
/// 6c96dc1f5573487399336cd351ef8f2c.
#[test]
fn skill_points_are_the_games() {
    let trickster = DATA
        .class(&ClassKey::new("arcane-trickster"))
        .expect("the Arcane Trickster");
    assert_eq!(trickster.skill_points, 3);
    let fighter = DATA.class(&ClassKey::new("fighter")).expect("the fighter");
    let defender = DATA
        .archetypes
        .iter()
        .find(|archetype| archetype.key == ArchetypeKey::new("fighter/defender-of-the-hearth"))
        .expect("the Defender of the Hearth");
    assert_eq!(defender.skill_points.unwrap_or(fighter.skill_points), 3);
}

/// An entry's class skills as the data has them: an archetype's own list when it replaces its class's, else the
/// class's.
fn class_skills_of(entry: &str) -> Vec<String> {
    let skills = match DATA
        .archetypes
        .iter()
        .find(|archetype| archetype.key == ArchetypeKey::new(entry))
    {
        Some(archetype) => {
            let class = DATA.class(&archetype.class).expect("the archetype's class");
            archetype
                .class_skills
                .as_ref()
                .unwrap_or(&class.class_skills)
        }
        None => {
            &DATA
                .class(&ClassKey::new(entry))
                .unwrap_or_else(|| panic!("no entry {entry}"))
                .class_skills
        }
    };
    skills.iter().map(ToString::to_string).collect()
}

/// The class skills of the 13 entries ticket 262 lists, as the game has them: the archetype's `ReplaceClassSkills`
/// list when it sets one, else its class's `ClassSkills` (the Metamorph, Sohei, Empyreal Sorcerer and Sylvan Sorcerer
/// set none: their blueprints' `ReplaceClassSkills` is false, and the skill each gives back comes through its own
/// feature). Skill names are the planner's keys (SkillThievery is "trickery").
#[test]
fn class_skills_are_the_games() {
    let alchemist = &[
        "knowledge-arcana",
        "knowledge-world",
        "lore-nature",
        "perception",
        "trickery",
        "use-magic-device",
    ];
    let monk = &[
        "athletics",
        "knowledge-world",
        "lore-religion",
        "mobility",
        "perception",
        "persuasion",
        "stealth",
    ];
    let sorcerer = &["knowledge-arcana", "persuasion", "use-magic-device"];
    let rows: &[(&str, &[&str])] = &[
        // MetamorphArchetype ac242ddeaa05af449bfc5230231fab84: no replacement; AlchemistClass
        // 0937bec61c0dabc468428f496580c721
        ("alchemist/metamorph", alchemist),
        // MagicDeceiver 5c77110cd0414e7eb4c2e485659c9a46
        (
            "arcanist/magic-deceiver",
            &[
                "knowledge-arcana",
                "knowledge-world",
                "lore-nature",
                "lore-religion",
                "persuasion",
                "use-magic-device",
            ],
        ),
        // DivineHunterArchetype f996f0a18e5d945459e710ee3a6dd485
        (
            "hunter/divine-hunter",
            &[
                "athletics",
                "knowledge-arcana",
                "lore-nature",
                "lore-religion",
                "perception",
                "persuasion",
                "stealth",
            ],
        ),
        // SpellDancerArchetype 1125145639129cf45b6b9b674cbd62b1
        (
            "magus/spell-dancer",
            &[
                "knowledge-arcana",
                "knowledge-world",
                "mobility",
                "persuasion",
                "use-magic-device",
            ],
        ),
        // SoheiArchetype fad7c56737ed12e42aacc330acc86428: no replacement; MonkClass
        // e8f21e5b58e0569468e420ebea456124
        ("monk/sohei", monk),
        // SeekerArchetype c539d65b7682c8d469e6f122922f1dee
        (
            "oracle/seeker",
            &[
                "knowledge-arcana",
                "knowledge-world",
                "lore-nature",
                "lore-religion",
                "persuasion",
            ],
        ),
        // WIndWhispererArchetype 77769e43e63647c2b32298efcc577aca
        (
            "oracle/wind-whisperer",
            &[
                "knowledge-arcana",
                "knowledge-world",
                "lore-nature",
                "lore-religion",
                "persuasion",
            ],
        ),
        // SableMarineArchetype 475117e837fa461589005a5339f653a0
        (
            "ranger/sable-company-marine",
            &[
                "athletics",
                "knowledge-world",
                "lore-nature",
                "mobility",
                "perception",
                "persuasion",
                "stealth",
            ],
        ),
        // SylvanTricksterArchetype 490394869f666c141bf8647b1a365220
        (
            "rogue/sylvan-trickster",
            &[
                "athletics",
                "lore-nature",
                "mobility",
                "perception",
                "persuasion",
                "stealth",
                "trickery",
                "use-magic-device",
            ],
        ),
        // EmpyrealSorcererArchetype aa00d945f7cf6c34c909a29a25f2df38 and SylvanSorcererArchetype
        // 711d5024ecc75f346b9cda609c3a1f83: no replacement; SorcererClass b3a505fb61437dc4097f43c3f8f9a4cf
        ("sorcerer/empyreal-sorcerer", sorcerer),
        ("sorcerer/sylvan-sorcerer", sorcerer),
        // WitchOfTheVeilArchetype 08045a390a764d71b2150fd122e05e19
        (
            "witch/witch-of-the-veil",
            &[
                "knowledge-arcana",
                "knowledge-world",
                "lore-nature",
                "persuasion",
                "use-magic-device",
            ],
        ),
        // StalwartDefenderClass d5917881586ff1d4d96d5b7cebda9464: Athletics, Mobility, Perception and the check
        // Intimidate. The planner has no Intimidate skill key, so the list stays without it (gap G4, ticket 312).
        (
            "stalwart-defender",
            &["athletics", "mobility", "perception"],
        ),
    ];
    for &(entry, game) in rows {
        let mut game: Vec<String> = game.iter().map(|skill| (*skill).to_owned()).collect();
        game.sort();
        let mut data = class_skills_of(entry);
        data.sort();
        assert_eq!(data, game, "{entry}");
    }
}
