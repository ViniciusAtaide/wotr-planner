//! The build sheet (ticket 87): a Build as plain text a player reads like a build sheet and follows in the game,
//! in the names the sheet shows. The planner copies it to the clipboard or saves it as a `.txt` file.
//!
//! ```text
//! WotR build v1
//! https://…/#b=<code>
//! Name: …                      (when the Build has one)
//! Race: Half-Elf
//! Subrace: Dual Heritage       (when chosen)
//! Racial +2: Strength          (for a race that chooses it)
//! Deity: Balance               (or Atheism)
//! Alignment: True Neutral
//! Background: …
//! Point buy: Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8
//! First ascension: …           (when chosen)
//! Mythic path: Trickster
//! Path switch: Gold Dragon
//!
//! Gear                         (only when the Build holds some, ticket 167)
//!   Belt: Belt of Incredible Dexterity +2   (one line per place worn, in the places' order)
//!   Ring 2: …
//!   Used: Tome of Clear Thought +2          (one line per usable item used, by key)
//!
//! Level 1: Fighter (Armiger)   (the archetype where the class is first taken; a prestige class's ", advancing …")
//!   Ability: +1 Strength       (the ability increase)
//!   Combat feat: Weapon Focus (Longsword)
//!   Skills: Athletics 1, Mobility 1
//!
//! Mythic track
//! MR 3 (level 9)
//!   Mythic ability: …
//!   Trickster Tricks: … # lost to the Path switch
//! MR 10 (level 20, taken later than level 19)   (a rank taken later than the rank table's level, ticket 171)
//! ```
//!
//! The line keys ("Race:", "Deity:", "Racial +2:", "  Ability:" …) are the format's own, fixed with its version:
//! they are not the labels the sheet, the pickers and the MCP server show ([`Engine::choice_label`]), so rewording a
//! label never changes what a v1 export says.
//!
//! The link on the second line always reopens the exact Build; the rest is for reading. Only the choices are data:
//! what's derived from them (here only a pick a Path switch takes away) is a comment, from ` # ` to the end of its
//! line, which a later import ignores. A choice not made yet is left out, a level with nothing chosen names only its
//! class, and a rank with no pick is left out unless it is taken later than the rank table's level: only such a
//! rank's line says so, so a Build that moves no rank gives the text it always did. The same Build always renders the same text: the start, the gear, the
//! levels in order, each level's picks in slot order as the sheet checks them, then the ranks in order.
//!
//! The gear (ticket 167) is a block of its own after the start, written only when the Build holds some, so a Build
//! without gear gives the text it always did. Its keys are the format's own too, and the point buy stays the point
//! buy: no line adds the gear's bonuses in.

use std::fmt::Write as _;

use crate::build::{AbilityScores, Build, GearPlace, GearSpot};
use crate::codec::encode_build;

use super::{Engine, RankSheet, RankStatus, Sheet};

/// The first line: what the text is, and its format's version.
const HEADER: &str = "WotR build v1";

/// The v1 format's line keys, each line's text before its value.
mod key {
    pub const NAME: &str = "Name";
    pub const RACE: &str = "Race";
    pub const SUBRACE: &str = "Subrace";
    pub const RACIAL_ABILITY: &str = "Racial +2";
    pub const FAITH: &str = "Deity";
    pub const ALIGNMENT: &str = "Alignment";
    pub const BACKGROUND: &str = "Background";
    pub const POINT_BUY: &str = "Point buy";
    pub const FIRST_ASCENSION: &str = "First ascension";
    pub const PATH: &str = "Mythic path";
    pub const SWITCH: &str = "Path switch";
    /// A level's line: "Level 4: Fighter".
    pub const LEVEL: &str = "Level";
    /// A level's ability increase: "  Ability: +1 Strength".
    pub const INCREASE: &str = "Ability";
    /// A level's skill ranks: "  Skills: Athletics 1, Mobility 1".
    pub const SKILLS: &str = "Skills";
    /// The Mythic track's heading.
    pub const TRACK: &str = "Mythic track";
    /// A rank's line: "MR 3 (level 9)".
    pub const RANK: &str = "MR";
    /// What follows the level of a rank taken later than the rank table's: "MR 10 (level 20, taken later than level
    /// 19)" (ticket 171).
    pub const LATER_THAN: &str = "taken later than level";
    /// The gear's heading (ticket 167).
    pub const GEAR: &str = "Gear";
    /// A usable item used: "  Used: Tome of Clear Thought +2".
    pub const USED: &str = "Used";
}

/// A gear place's line key: "  Belt: Belt of Incredible Dexterity +2". The format's own, not the Gear tab's row
/// names ([`GearPlace::name`]), so rewording a row never changes an export.
fn place_key(place: GearPlace) -> &'static str {
    match place {
        GearPlace::Belt => "Belt",
        GearPlace::Boots => "Boots",
        GearPlace::Bracers => "Bracers",
        GearPlace::Cloak => "Cloak",
        GearPlace::Glasses => "Glasses",
        GearPlace::Gloves => "Gloves",
        GearPlace::Headwear => "Headwear",
        GearPlace::Necklace => "Necklace",
        GearPlace::FirstRing => "Ring 1",
        GearPlace::SecondRing => "Ring 2",
        GearPlace::Body => "Body",
        GearPlace::Weapon => "Weapon",
    }
}

/// The comment on a pick a Path switch takes away.
const LOST_TO_THE_SWITCH: &str = "lost to the Path switch";

impl Engine {
    /// The Build as a build sheet. `page` is the planner's address before the fragment (`https://…/planner/`): the
    /// second line is the Build's link, `{page}#b=<code>`.
    pub fn build_sheet(&self, build: &Build, page: &str) -> String {
        let sheet = self.evaluate(build);
        let mut text = String::new();
        // Writing to a String never fails.
        let _ = self.write_start(&mut text, build, page);
        let _ = self.write_gear(&mut text, build);
        let _ = self.write_levels(&mut text, build, &sheet);
        let _ = self.write_track(&mut text, &sheet);
        text
    }

    /// The header, the link, the Build start and the path choices.
    fn write_start(&self, text: &mut String, build: &Build, page: &str) -> std::fmt::Result {
        writeln!(text, "{HEADER}")?;
        writeln!(text, "{page}#b={}", encode_build(build))?;
        if let Some(name) = &build.name {
            writeln!(text, "{}: {name}", key::NAME)?;
        }
        let start = &build.start;
        if let Some(race) = &start.race {
            writeln!(text, "{}: {}", key::RACE, self.race_name(race))?;
        }
        if let Some(subrace) = &start.subrace {
            writeln!(text, "{}: {}", key::SUBRACE, self.subrace_name(subrace))?;
        }
        if let Some(ability) = start.racial_ability {
            writeln!(text, "{}: {}", key::RACIAL_ABILITY, ability.name())?;
        }
        if let Some(faith) = &start.deity {
            writeln!(text, "{}: {}", key::FAITH, self.faith_name(faith))?;
        }
        if let Some(alignment) = start.alignment {
            writeln!(text, "{}: {}", key::ALIGNMENT, alignment.name())?;
        }
        if let Some(background) = &start.background {
            writeln!(
                text,
                "{}: {}",
                key::BACKGROUND,
                self.background_name(background)
            )?;
        }
        let scores: Vec<String> = AbilityScores::ABILITIES
            .iter()
            .map(|&ability| {
                format!(
                    "{} {}",
                    ability.short_name(),
                    start.ability_scores.get(ability)
                )
            })
            .collect();
        writeln!(text, "{}: {}", key::POINT_BUY, scores.join(", "))?;
        if let Some(path) = &build.first_ascension {
            writeln!(
                text,
                "{}: {}",
                key::FIRST_ASCENSION,
                self.first_ascension_name(path)
            )?;
        }
        if let Some(path) = &build.mythic_path {
            writeln!(text, "{}: {}", key::PATH, self.path_name(path))?;
        }
        if let Some(path) = &build.path_switch {
            writeln!(text, "{}: {}", key::SWITCH, self.path_name(path))?;
        }
        Ok(())
    }

    /// The gear after a blank line: the item worn in each place, in the places' order, then each usable item used,
    /// by key (the order the link writes them in). Nothing for a Build without gear.
    fn write_gear(&self, text: &mut String, build: &Build) -> std::fmt::Result {
        if build.gear.is_empty() {
            return Ok(());
        }
        writeln!(text, "\n{}", key::GEAR)?;
        for (spot, item) in build.gear.items() {
            let key = match spot {
                GearSpot::Worn(place) => place_key(place),
                GearSpot::Used(_) => key::USED,
            };
            writeln!(text, "  {key}: {}", self.gear_name(item))?;
        }
        Ok(())
    }

    /// Each level after a blank line: its class, ability increase, picks and skill ranks.
    fn write_levels(&self, text: &mut String, build: &Build, sheet: &Sheet) -> std::fmt::Result {
        for (level, level_up) in sheet.levels.iter().zip(&build.levels) {
            write!(
                text,
                "\n{} {}: {}",
                key::LEVEL,
                level.character_level,
                self.class_name(&level.class)
            )?;
            let choice = build.class_choices.get(&level.class);
            if let Some(choice) = choice.filter(|_| level.class_level == 1) {
                if let Some(archetype) = &choice.archetype {
                    write!(text, " ({})", self.archetype_name(archetype))?;
                }
                if !choice.advances.is_empty() {
                    write!(text, "{}", self.advancing(&choice.advances))?;
                }
            }
            writeln!(text)?;
            if let Some(ability) = level_up.ability_increase {
                writeln!(text, "  {}: +1 {}", key::INCREASE, ability.name())?;
            }
            for slot in &level.slots {
                if let Some(pick) = &slot.pick {
                    writeln!(
                        text,
                        "  {}: {}",
                        self.slot_label(slot),
                        self.pick_name(pick)
                    )?;
                }
            }
            if !level_up.skill_ranks.is_empty() {
                let ranks: Vec<String> = level_up
                    .skill_ranks
                    .iter()
                    .map(|(skill, ranks)| format!("{} {ranks}", self.skill_name(skill)))
                    .collect();
                writeln!(text, "  {}: {}", key::SKILLS, ranks.join(", "))?;
            }
        }
        Ok(())
    }

    /// The Mythic track after a blank line: each rank with a pick, its character level and its picks, and each
    /// reached rank taken later than the rank table's level, which its line says. Nothing when there is none.
    fn write_track(&self, text: &mut String, sheet: &Sheet) -> std::fmt::Result {
        let later = |rank: &RankSheet| {
            (self.rank_table_level(rank.rank)).filter(|earliest| rank.character_level > *earliest)
        };
        let mut ranks = sheet
            .ranks
            .iter()
            .filter(|rank| {
                rank.slots.iter().any(|slot| slot.pick.is_some())
                    || (rank.status != RankStatus::NotReached && later(rank).is_some())
            })
            .peekable();
        if ranks.peek().is_some() {
            writeln!(text, "\n{}", key::TRACK)?;
        }
        for rank in ranks {
            write!(
                text,
                "{} {} (level {}",
                key::RANK,
                rank.rank,
                rank.character_level
            )?;
            if let Some(earliest) = later(rank) {
                write!(text, ", {} {earliest}", key::LATER_THAN)?;
            }
            writeln!(text, ")")?;
            for slot in &rank.slots {
                let Some(pick) = &slot.pick else { continue };
                write!(
                    text,
                    "  {}: {}",
                    self.slot_label(slot),
                    self.pick_name(pick)
                )?;
                if rank.dropped_slots.contains(&slot.id) {
                    write!(text, " # {LOST_TO_THE_SWITCH}")?;
                }
                writeln!(text)?;
            }
        }
        Ok(())
    }
}
