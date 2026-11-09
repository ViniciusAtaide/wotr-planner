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
//! Level 1: Fighter (Armiger)   (the archetype where the class is first taken; a prestige class's ", advancing …")
//!   Ability: +1 Strength       (the ability increase)
//!   Combat feat: Weapon Focus (Longsword)
//!   Skills: Athletics 1, Mobility 1
//!
//! Mythic track
//! MR 3 (level 9)
//!   Mythic ability: …
//!   Trickster Tricks: … # lost to the Path switch
//! ```
//!
//! The link on the second line always reopens the exact Build; the rest is for reading. Only the choices are data:
//! what's derived from them (here only a pick a Path switch takes away) is a comment, from ` # ` to the end of its
//! line, which a later import ignores. A choice not made yet is left out, a level with nothing chosen names only its
//! class, and a rank with no pick is left out. The same Build always renders the same text: the start, the levels in
//! order, each level's picks in slot order as the sheet checks them, then the ranks in order.

use std::fmt::Write as _;

use crate::build::{AbilityScores, Build};
use crate::codec::encode_build;

use super::{Engine, Sheet};

/// The first line: what the text is, and its format's version.
const HEADER: &str = "WotR build v1";

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
        let _ = self.write_levels(&mut text, build, &sheet);
        let _ = self.write_track(&mut text, &sheet);
        text
    }

    /// The header, the link, the Build start and the path choices.
    fn write_start(&self, text: &mut String, build: &Build, page: &str) -> std::fmt::Result {
        writeln!(text, "{HEADER}")?;
        writeln!(text, "{page}#b={}", encode_build(build))?;
        if let Some(name) = &build.name {
            writeln!(text, "Name: {name}")?;
        }
        let start = &build.start;
        if let Some(race) = &start.race {
            writeln!(text, "Race: {}", self.race_name(race))?;
        }
        if let Some(subrace) = &start.subrace {
            writeln!(text, "Subrace: {}", self.subrace_name(subrace))?;
        }
        if let Some(ability) = start.racial_ability {
            writeln!(text, "Racial +2: {}", ability.name())?;
        }
        if let Some(faith) = &start.deity {
            writeln!(text, "Deity: {}", self.faith_name(faith))?;
        }
        if let Some(alignment) = start.alignment {
            writeln!(text, "Alignment: {}", alignment.name())?;
        }
        if let Some(background) = &start.background {
            writeln!(text, "Background: {}", self.background_name(background))?;
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
        writeln!(text, "Point buy: {}", scores.join(", "))?;
        if let Some(path) = &build.first_ascension {
            writeln!(text, "First ascension: {}", self.first_ascension_name(path))?;
        }
        if let Some(path) = &build.mythic_path {
            writeln!(text, "Mythic path: {}", self.path_name(path))?;
        }
        if let Some(path) = &build.path_switch {
            writeln!(text, "Path switch: {}", self.path_name(path))?;
        }
        Ok(())
    }

    /// Each level after a blank line: its class, ability increase, picks and skill ranks.
    fn write_levels(&self, text: &mut String, build: &Build, sheet: &Sheet) -> std::fmt::Result {
        for (level, level_up) in sheet.levels.iter().zip(&build.levels) {
            write!(
                text,
                "\nLevel {}: {}",
                level.character_level,
                self.class_name(&level.class)
            )?;
            let choice = build.class_choices.get(&level.class);
            if let Some(choice) = choice.filter(|_| level.class_level == 1) {
                if let Some(archetype) = &choice.archetype {
                    write!(text, " ({})", self.archetype_name(archetype))?;
                }
                if !choice.advances.is_empty() {
                    let advances: Vec<&str> = choice
                        .advances
                        .iter()
                        .map(|class| self.class_name(class))
                        .collect();
                    write!(text, ", advancing {}", advances.join(" and "))?;
                }
            }
            writeln!(text)?;
            if let Some(ability) = level_up.ability_increase {
                writeln!(text, "  Ability: +1 {}", ability.name())?;
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
                writeln!(text, "  Skills: {}", ranks.join(", "))?;
            }
        }
        Ok(())
    }

    /// The Mythic track after a blank line: each rank with a pick, its character level and its picks. Nothing when
    /// no rank has one.
    fn write_track(&self, text: &mut String, sheet: &Sheet) -> std::fmt::Result {
        let mut ranks = sheet
            .ranks
            .iter()
            .filter(|rank| rank.slots.iter().any(|slot| slot.pick.is_some()))
            .peekable();
        if ranks.peek().is_some() {
            writeln!(text, "\nMythic track")?;
        }
        for rank in ranks {
            writeln!(text, "MR {} (level {})", rank.rank, rank.character_level)?;
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
