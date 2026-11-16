//! The comparisons, one module per area, and what they share: the context, the game's names for the planner's
//! stats, and the blueprints the check names by GUID.

use rules::{Ability, BlueprintGuid, SkillKey};

use serde_json::Value;

use crate::Dump;
use crate::planner::Planner;
use crate::report::{Area, Out};

pub mod class_tables;
pub mod headers;
pub mod items;
pub mod mapping;
pub mod options;
pub mod prerequisites;
pub mod rows;

/// What every comparison reads.
pub struct Ctx<'a> {
    pub dump: &'a Dump,
    pub planner: &'a Planner<'a>,
}

impl<'a> Ctx<'a> {
    /// A blueprint's asset name, for a finding.
    pub fn name(&self, guid: &BlueprintGuid) -> String {
        self.dump.name(guid)
    }

    /// An entry's blueprint and its JSON; when it has none or the dump lacks it, `None`, counted in `area`.
    pub fn blueprint<'g>(
        &self,
        out: &mut Out,
        area: Area,
        guid: Option<&'g BlueprintGuid>,
        kind: &str,
    ) -> Option<(&'g BlueprintGuid, &'a Value)> {
        let Some(guid) = guid else {
            out.skip(area, format!("{kind} with no blueprint"));
            return None;
        };
        let Some(json) = self.dump.json(guid) else {
            out.skip(area, "a blueprint not in the dump");
            return None;
        };
        Some((guid, json))
    }
}

/// Blueprints the check names itself, each cited by its asset name.
pub mod fixed {
    /// `BasicFeatSelection`: the feat a character picks at every odd level.
    pub const BASIC_FEAT_SELECTION: &str = "247a4068296e8be42890143f451b4b45";
    /// The game's selections of a feat group that the planner's general and category selections name, and the
    /// sorcerer's bloodline feat's (ticket 301): `BasicFeatSelection`, `FighterFeatSelection`, `WizardFeatSelection`,
    /// the inquisitor's `TeamworkFeat`, `CavalierBonusFeatSelection` and `SorcererFeatSelection`. The rows compare a
    /// grant of one, on either side, as a pick of its group, as they compared the planner's feat slots before.
    pub const FEAT_POOLS: [&str; 6] = [
        BASIC_FEAT_SELECTION,
        "41c8486641f7d6d4283ca9dae4147a9f",
        "8c3102c2ff3b69444b139a98521a4899",
        "d87e2f6a9278ac04caeb0f93eff95fcb",
        "dd17090d14958ef48ba601688b611970",
        "3a60f0c0442acfb419b0c03b584e1394",
    ];
    /// `MythicAbilitySelection`: a Mythic rank's general ability pick.
    pub const MYTHIC_ABILITY_SELECTION: &str = "ba0e5a900b775be4a99702f1ed08914d";
    /// `MythicFeatSelection`: a Mythic rank's general feat pick.
    pub const MYTHIC_FEAT_SELECTION: &str = "9ee0f6745f555484299b0a1563b99d81";
    /// `FirstAscensionSelection`: the first rank's choice of path.
    pub const FIRST_ASCENSION_SELECTION: &str = "1421e0034a3afac458de08648d06faf0";
    /// `MythicStartingClass`: the mythic class of the ranks before a path is taken.
    pub const MYTHIC_STARTING_CLASS: &str = "247aa787806d5da4f89cfc3dff0b217f";
    /// `MythicStartingProgression`: `MythicStartingClass`'s rows.
    pub const MYTHIC_STARTING_PROGRESSION: &str = "af4ee0acb9114e544bf02f39027966b0";
    /// `AnimalClass`: every class's prerequisite names it, to keep animal companions out.
    pub const ANIMAL_CLASS: &str = "4cd1757a0eea7694ba5c933729a53920";
    /// `AnimalCompanionClass`: a pet's class, which no Build has.
    pub const ANIMAL_COMPANION_CLASS: &str = "26b10d4340839004f960f9816f6109fe";
    /// `MonstrousHumanoidClass`: a pet's class, which no Build has.
    pub const MONSTROUS_HUMANOID_CLASS: &str = "8a3c86893f383214da070e9c84c1e95b";
    /// `SubtypeElemental`: a creature's subtype, which no Build has.
    pub const SUBTYPE_ELEMENTAL: &str = "198fd8924dabcb5478d0f78bd453c586";
    /// `BlizzardServantLockEquipmentFeature`: the Winter Child's servant's lock on its equipment, which no Build has.
    pub const BLIZZARD_SERVANT_LOCK: &str = "daf893d14cc54e98a319fb121d7ac4d9";
    /// Each path's rows before its class is taken (`<Path>FirstLevelsProgression`), by the path's class. The story
    /// grants them, so no blueprint leads from the class to them.
    pub const FIRST_LEVELS: [(&str, &str); 6] = [
        // AeonMythicClass, AeonFirstLevelsProgression
        (
            "15a85e67b7d69554cab9ed5830d0268e",
            "46df0532714a9454eb5fbad64ce6c14f",
        ),
        // AngelMythicClass, AngelFirstLevelsProgression
        (
            "a5a9fe8f663d701488bd1db8ea40484e",
            "b236003be2b9400498b9dd0f07b0c93c",
        ),
        // AzataMythicClass, AzataFirstLevelsProgression
        (
            "9a3b2c63afa79744cbca46bea0da9a16",
            "132afc1a28bd9d442821385f7cbf1c05",
        ),
        // DemonMythicClass, DemonFirstLevelsProgression
        (
            "8e19495ea576a8641964102d177e34b7",
            "8afd697daf0d47a4883759a6bc1aff88",
        ),
        // LichMythicClass, LichFirstLevelsProgression
        (
            "5d501618a28bdc24c80007a5c937dcb7",
            "a796af14198bf5e45b63f056c32107a2",
        ),
        // TricksterMythicClass, TricksterFirstLevelsProgression
        (
            "8df873a8c6e48294abdb78c45834aa0a",
            "11ba180ac736c894e937602e54f7320c",
        ),
    ];

    pub fn guid(text: &str) -> rules::BlueprintGuid {
        rules::BlueprintGuid::new(text)
    }
}

/// The game's stat for an ability score (`StatType`, WrathDecompiled 6ce4a6b): the ability's own name.
pub fn ability_stat(ability: Ability) -> &'static str {
    ability.name()
}

/// The game's stat for one of the planner's skills (`StatType`): Trickery is `SkillThievery`.
pub fn skill_stat(skill: &SkillKey) -> String {
    match skill.as_str() {
        "athletics" => "SkillAthletics",
        "knowledge-arcana" => "SkillKnowledgeArcana",
        "knowledge-world" => "SkillKnowledgeWorld",
        "lore-nature" => "SkillLoreNature",
        "lore-religion" => "SkillLoreReligion",
        "mobility" => "SkillMobility",
        "perception" => "SkillPerception",
        "persuasion" => "SkillPersuasion",
        "stealth" => "SkillStealth",
        "trickery" => "SkillThievery",
        "use-magic-device" => "SkillUseMagicDevice",
        other => return format!("(no game skill) {other}"),
    }
    .to_owned()
}

/// Whether a game stat is an ability score.
pub fn is_ability_stat(stat: &str) -> bool {
    matches!(
        stat,
        "Strength" | "Dexterity" | "Constitution" | "Intelligence" | "Wisdom" | "Charisma"
    )
}
