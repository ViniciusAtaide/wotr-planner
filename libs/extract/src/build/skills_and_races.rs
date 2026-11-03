//! The skills, with each class's and archetype's skill points and class skills from the skills table, and the races
//! with their subraces and ability modifiers.

use std::collections::BTreeSet;

use rules::{
    Ability, AbilityModifier, FeatSlot, GuideRef, Race, RaceKey, Skill, SkillKey, Subrace,
    SubraceKey,
};

use crate::html::{Page, Section};
use crate::names::{norm, slug};
use crate::report::Kind;

use super::Builder;

/// The skills table's column abbreviations.
const SKILL_COLUMNS: [(&str, &str); 11] = [
    ("ATH", "Athletics"),
    ("MOB", "Mobility"),
    ("TRI", "Trickery"),
    ("STE", "Stealth"),
    ("KNA", "Knowledge (Arcana)"),
    ("KNW", "Knowledge (World)"),
    ("LON", "Lore (Nature)"),
    ("LOR", "Lore (Religion)"),
    ("PCP", "Perception"),
    ("PER", "Persuasion"),
    ("UMD", "Use Magic Device"),
];

impl<'g> Builder<'g> {
    pub(super) fn skills(&mut self) -> Vec<Skill> {
        let page = self.guide.page("character-creation");
        let Some(section) = page.root.find("skills") else {
            self.report
                .add(Kind::Cell, page.guide("skills"), "", "no skills section");
            return Vec::new();
        };
        let mut skills = Vec::new();
        for rows in section.tables() {
            let Some(header) = rows.first() else { continue };
            if header.first().map(String::as_str) != Some("Skill") {
                continue;
            }
            let abilities = [
                ("STR", Ability::Strength),
                ("DEX", Ability::Dexterity),
                ("CON", Ability::Constitution),
                ("INT", Ability::Intelligence),
                ("WIS", Ability::Wisdom),
                ("CHA", Ability::Charisma),
            ];
            for row in &rows[1..] {
                let name = row[0].clone();
                let ability = header.iter().zip(row).find_map(|(column, cell)| {
                    (cell.contains('✓'))
                        .then(|| abilities.iter().find(|(abbr, _)| abbr == column))
                        .flatten()
                        .map(|(_, ability)| *ability)
                });
                let Some(ability) = ability else {
                    self.report
                        .add(Kind::Cell, page.guide("skills"), name, "no ability ticked");
                    continue;
                };
                let key = SkillKey::new(slug(&name));
                self.catalog.skills.insert(norm(&name), key.clone());
                skills.push(Skill { key, name, ability });
            }
        }
        skills
    }

    /// The skills table of classes and archetypes: name, skill points, class skills.
    pub(super) fn class_skill_rows(&self) -> Vec<(String, u8, BTreeSet<SkillKey>)> {
        let page = self.guide.page("character-creation");
        let Some(section) = page.root.find("skills") else {
            return Vec::new();
        };
        let mut rows_out = Vec::new();
        for rows in section.tables() {
            let Some(header) = rows.first() else { continue };
            if header.get(1).map(String::as_str) != Some("/ lvl") {
                continue;
            }
            for row in &rows[1..] {
                let Ok(points) = row[1].parse() else { continue };
                let skills = header
                    .iter()
                    .zip(row)
                    .filter(|(_, cell)| cell.contains('✓'))
                    .filter_map(|(column, _)| {
                        SKILL_COLUMNS
                            .iter()
                            .find(|(abbr, _)| abbr == column)
                            .map(|(_, name)| SkillKey::new(slug(name)))
                    })
                    .collect();
                rows_out.push((row[0].clone(), points, skills));
            }
        }
        rows_out
    }

    pub(super) fn races(&mut self) -> Vec<Race> {
        let page = self.guide.page("character-creation");
        let Some(section) = page.root.find("race") else {
            return Vec::new();
        };
        let mut races: Vec<Race> = Vec::new();
        for entry in section.children.iter().filter(|s| s.level == 4) {
            let (modifiers, feat_slots) = self.racial_table(page, entry);
            // "Aasimar (Angelkin)" as its own H4 is a subrace of the race before it.
            if let Some((race_name, sub)) = entry.name.split_once(" (")
                && let Some(race) = races
                    .iter_mut()
                    .find(|race| norm(&race.name) == norm(race_name))
            {
                race.subraces
                    .push(subrace(page, race, entry, sub, modifiers));
                continue;
            }
            let key = RaceKey::new(entry.anchor.clone());
            self.catalog.races.insert(norm(&entry.name), key.clone());
            // A bonus feat at 1st level the guide names: a pick of its one-feat list.
            let (feat_slots, feat_lists) = match self.picks.race_list_of(key.as_str()) {
                None => (feat_slots, Vec::new()),
                Some(list) => {
                    assert_eq!(feat_slots, [FeatSlot::General], "{key}");
                    (
                        vec![FeatSlot::OneOf {
                            list: list.key.clone(),
                        }],
                        vec![list.clone()],
                    )
                }
            };
            let mut race = Race {
                key,
                name: entry.name.clone(),
                summary: String::new(),
                guide: GuideRef::new(page.guide(&entry.anchor)),
                ability_modifiers: modifiers,
                feat_slots,
                subraces: Vec::new(),
                feat_lists,
            };
            for child in &entry.children {
                let (modifiers, _) = self.racial_table(page, child);
                let sub = child
                    .name
                    .split_once(" (")
                    .map_or(child.name.as_str(), |(_, sub)| sub);
                let made = subrace(page, &race, child, sub, modifiers);
                race.subraces.push(made);
            }
            races.push(race);
        }
        races
    }

    fn racial_table(
        &mut self,
        page: &Page,
        entry: &Section,
    ) -> (Vec<AbilityModifier>, Vec<FeatSlot>) {
        let mut modifiers = Vec::new();
        let mut slots = Vec::new();
        for rows in entry.tables() {
            for row in rows {
                let text = row.join(" ");
                if row.first().map(String::as_str) == Some("Ability Modifiers") {
                    match ability_modifiers(row.get(1).map_or("", String::as_str)) {
                        Some(parsed) => modifiers = parsed,
                        None => self.report.add(
                            Kind::Cell,
                            page.guide(&entry.anchor),
                            text.clone(),
                            "ability modifiers not understood",
                        ),
                    }
                }
                if text
                    .to_ascii_lowercase()
                    .contains("bonus feat at 1st level")
                {
                    slots.push(FeatSlot::General);
                }
            }
        }
        (modifiers, slots)
    }
}

fn subrace(
    page: &Page,
    race: &Race,
    entry: &Section,
    sub: &str,
    modifiers: Vec<AbilityModifier>,
) -> Subrace {
    let race_slug = race.key.as_str();
    let sub_slug = slug(sub.trim_end_matches(')'));
    let sub_slug = sub_slug
        .strip_prefix(&format!("{race_slug}-"))
        .unwrap_or(&sub_slug)
        .to_owned();
    Subrace {
        key: SubraceKey::new(format!("{race_slug}/{sub_slug}")),
        name: entry.name.clone(),
        summary: String::new(),
        guide: GuideRef::new(page.guide(&entry.anchor)),
        ability_modifiers: modifiers,
        feat_slots: vec![],
    }
}

/// "+2 Dexterity, Intelligence; -2 Constitution" or "+2 any".
fn ability_modifiers(text: &str) -> Option<Vec<AbilityModifier>> {
    let mut out = Vec::new();
    for group in text.split(';') {
        let group = group.trim().trim_end_matches('|').trim();
        if group.is_empty() {
            continue;
        }
        let (amount, abilities) = group.split_once(' ')?;
        let amount: i8 = amount.trim_start_matches('+').parse().ok()?;
        for name in abilities.split(',') {
            let name = name.trim().to_ascii_lowercase();
            if name == "any" {
                out.push(AbilityModifier::Chosen { amount });
                continue;
            }
            let ability = match name.as_str() {
                "strength" => Ability::Strength,
                "dexterity" => Ability::Dexterity,
                "constitution" => Ability::Constitution,
                "intelligence" => Ability::Intelligence,
                "wisdom" => Ability::Wisdom,
                "charisma" => Ability::Charisma,
                _ => return None,
            };
            out.push(AbilityModifier::Fixed { ability, amount });
        }
    }
    (!out.is_empty()).then_some(out)
}
