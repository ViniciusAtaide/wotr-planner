//! Gear (ticket 165): the named items a character wears or wields, each with the bonuses it always gives, and a
//! summary of them generated from the data. `gear/` holds them, one file each. A Build holds them in its gear
//! ([`Build::gear`](crate::Build::gear), ticket 166), and the engine counts what they always give.
//!
//! An item's bonuses are its components (ticket 297), each a [`GearStat`](Component::GearStat): a [`GearTarget`] (an
//! ability score, or a number the planner already computes), an amount and a [`Descriptor`]. What an item does besides
//! (a bonus in some circumstance, a use per day, who may wear it) is a short line of the planner's own words in
//! [`GearItem::other_effects`], and what it does while worn without being a counted bonus is a line in
//! [`GearItem::always_on`], which the summary appends to the bonuses.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::bonus::SpellScope;
use crate::component::{Component, Descriptor};
use crate::game_data::{Ability, GuideRef, Save, Skill};
use crate::keys::{BlueprintGuid, GearKey, SkillKey};

/// A named item: where it is worn, what it always gives, and what else it does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearItem {
    pub key: GearKey,
    /// The game blueprint behind it, if any ([`BlueprintGuid`]).
    #[serde(default, skip_serializing)]
    pub blueprint: Option<BlueprintGuid>,
    pub name: String,
    /// The guide section it is listed in.
    pub slot: GearSlot,
    /// `page#section`: the guide has no anchor per item. `None` for an item the guide doesn't list (the generic
    /// stat belts and headbands, from the user).
    #[serde(default)]
    pub guide: Option<GuideRef>,
    /// Its always-on bonuses, one [`GearStat`](Component::GearStat) per number changed (ticket 297; the check refuses
    /// any other kind).
    pub components: Vec<Component>,
    /// What it also does while worn that no number counts (ticket 176): damage reduction, an immunity, an AC or
    /// speed bonus, the weapon's or armour's own enhancement. One short line each, in the planner's own words, in
    /// the order [`GearItem::summary`] appends them.
    #[serde(default)]
    pub always_on: Vec<String>,
    /// What it does only in some circumstance or by use: on a critical hit, while raging, against a kind of enemy,
    /// a number of times a day, with another item, or only for some wearers. One short line each, in the planner's
    /// own words, shown in the detail and not in the summary.
    #[serde(default)]
    pub other_effects: Vec<String>,
}

/// The guide's item sections, plus the armour and weapon an item may sit in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GearSlot {
    Belt,
    Boots,
    Bracers,
    Cloak,
    Glasses,
    Gloves,
    Headwear,
    Necklace,
    Ring,
    Robe,
    Armour,
    Weapon,
    /// The guide's "Equipable and Consumable" section.
    Usable,
}

impl GearSlot {
    pub const ALL: [Self; 13] = [
        Self::Belt,
        Self::Boots,
        Self::Bracers,
        Self::Cloak,
        Self::Glasses,
        Self::Gloves,
        Self::Headwear,
        Self::Necklace,
        Self::Ring,
        Self::Robe,
        Self::Armour,
        Self::Weapon,
        Self::Usable,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Belt => "Belt",
            Self::Boots => "Boots",
            Self::Bracers => "Bracers",
            Self::Cloak => "Cloak",
            Self::Glasses => "Glasses",
            Self::Gloves => "Gloves",
            Self::Headwear => "Headwear",
            Self::Necklace => "Necklace",
            Self::Ring => "Ring",
            Self::Robe => "Robe",
            Self::Armour => "Armour",
            Self::Weapon => "Weapon",
            Self::Usable => "Usable",
        }
    }
}

/// What a [`GearStat`](Component::GearStat) adds to. Declared in the order the summary lists them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GearTarget {
    Ability(Ability),
    AllSaves,
    Save(Save),
    Initiative,
    Skill(SkillKey),
    /// Never [`SpellScope::Value`].
    SpellDc(SpellScope),
    /// The caster level, and with it caster level checks and concentration. Only an unscoped bonus is one.
    CasterLevel,
    SpellPenetration,
    Concentration,
}

impl GearTarget {
    /// The name the summary prints; `skills` gives a skill its name.
    fn name(&self, skills: &[Skill]) -> String {
        match self {
            Self::Ability(ability) => ability.short_name().to_owned(),
            Self::AllSaves => "Saves".to_owned(),
            Self::Save(Save::Fortitude) => "Fortitude".to_owned(),
            Self::Save(Save::Reflex) => "Reflex".to_owned(),
            Self::Save(Save::Will) => "Will".to_owned(),
            Self::Initiative => "Initiative".to_owned(),
            Self::Skill(key) => skills
                .iter()
                .find(|skill| skill.key == *key)
                .map_or_else(|| key.to_string(), |skill| skill.name.clone()),
            Self::SpellDc(SpellScope::All) => "Spell DC".to_owned(),
            Self::SpellDc(scope) => {
                let scope = match scope {
                    SpellScope::All | SpellScope::Value => "",
                    SpellScope::School(school) => school.name(),
                    SpellScope::Subschool(subschool) => subschool.name(),
                    SpellScope::Energy(energy) => energy.name(),
                    SpellScope::Metamagic => "metamagic",
                };
                format!("Spell DC ({scope})")
            }
            Self::CasterLevel => "Caster level".to_owned(),
            Self::SpellPenetration => "Spell penetration".to_owned(),
            Self::Concentration => "Concentration".to_owned(),
        }
    }
}

impl GearItem {
    /// Its always-on bonuses ([`GearStat`](Component::GearStat)), each with its target, amount and type.
    pub fn stats(&self) -> impl Iterator<Item = (&GearTarget, i8, Descriptor)> {
        self.components
            .iter()
            .filter_map(|component| match component {
                Component::GearStat {
                    stat,
                    value,
                    descriptor,
                } => Some((stat, *value, *descriptor)),
                _ => None,
            })
    }

    /// Whether it gives a bonus or a penalty to an ability score.
    pub fn changes(&self, ability: Ability) -> bool {
        self.stats()
            .any(|(target, ..)| *target == GearTarget::Ability(ability))
    }

    /// The summary of its bonuses, generated so the web app and the MCP show one text: the bonuses in
    /// [`GearTarget`]'s order, those sharing an amount and a type grouped ("Str, Dex, Con +6 enhancement"), then
    /// the [`always_on`](Self::always_on) lines as written, all joined by "; ", an untyped bonus with no type, no
    /// full stop ("Str, Dex, Con +4 enhancement; DR 3/-"). `skills` gives a skill target its name.
    pub fn summary(&self, skills: &[Skill]) -> String {
        let mut bonuses: Vec<(&GearTarget, i8, Descriptor)> = self.stats().collect();
        bonuses.sort_by(|a, b| a.0.cmp(b.0));
        let mut groups: Vec<(i8, Descriptor, Vec<String>)> = Vec::new();
        for (target, amount, descriptor) in bonuses {
            let name = target.name(skills);
            match groups
                .iter_mut()
                .find(|(of, kind, _)| *of == amount && *kind == descriptor)
            {
                Some((_, _, names)) => names.push(name),
                None => groups.push((amount, descriptor, vec![name])),
            }
        }
        let mut text = String::new();
        for (index, (amount, descriptor, names)) in groups.iter().enumerate() {
            if index > 0 {
                text.push_str("; ");
            }
            let _ = write!(text, "{} {amount:+}", names.join(", "));
            if !descriptor.name().is_empty() {
                let _ = write!(text, " {}", descriptor.name());
            }
        }
        for line in &self.always_on {
            if !text.is_empty() {
                text.push_str("; ");
            }
            text.push_str(line);
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(key: &str, name: &str) -> Skill {
        Skill {
            key: SkillKey::new(key),
            name: name.to_owned(),
            ability: Ability::Charisma,
        }
    }

    fn skills() -> Vec<Skill> {
        vec![
            skill("knowledge-world", "Knowledge (World)"),
            skill("persuasion", "Persuasion"),
        ]
    }

    fn bonus(stat: GearTarget, value: i8, descriptor: Descriptor) -> Component {
        Component::GearStat {
            stat,
            value,
            descriptor,
        }
    }

    fn item(components: Vec<Component>) -> GearItem {
        GearItem {
            key: GearKey::new("test"),
            blueprint: None,
            name: "Test".to_owned(),
            slot: GearSlot::Belt,
            guide: None,
            components,
            always_on: Vec::new(),
            other_effects: Vec::new(),
        }
    }

    fn ability(ability: Ability) -> GearTarget {
        GearTarget::Ability(ability)
    }

    #[test]
    fn one_score() {
        let item = item(vec![bonus(
            ability(Ability::Dexterity),
            6,
            Descriptor::Enhancement,
        )]);
        assert_eq!(item.summary(&skills()), "Dex +6 enhancement");
    }

    #[test]
    fn three_scores_are_grouped() {
        let item = item(
            [Ability::Constitution, Ability::Strength, Ability::Dexterity]
                .map(|a| bonus(ability(a), 6, Descriptor::Enhancement))
                .to_vec(),
        );
        assert_eq!(item.summary(&skills()), "Str, Dex, Con +6 enhancement");
    }

    #[test]
    fn a_cloak() {
        let item = item(vec![
            bonus(GearTarget::AllSaves, 7, Descriptor::Resistance),
            bonus(ability(Ability::Charisma), 2, Descriptor::Circumstance),
        ]);
        assert_eq!(
            item.summary(&skills()),
            "Cha +2 circumstance; Saves +7 resistance"
        );
    }

    #[test]
    fn skills_are_named_and_grouped() {
        let item = item(vec![
            bonus(
                GearTarget::Skill(SkillKey::new("persuasion")),
                5,
                Descriptor::Competence,
            ),
            bonus(
                GearTarget::Skill(SkillKey::new("knowledge-world")),
                5,
                Descriptor::Competence,
            ),
            bonus(ability(Ability::Charisma), 2, Descriptor::Enhancement),
        ]);
        assert_eq!(
            item.summary(&skills()),
            "Cha +2 enhancement; Knowledge (World), Persuasion +5 competence"
        );
    }

    #[test]
    fn penalties_are_untyped_and_signed() {
        let item = item(vec![
            bonus(
                GearTarget::Skill(SkillKey::new("persuasion")),
                -3,
                Descriptor::Untyped,
            ),
            bonus(ability(Ability::Charisma), -2, Descriptor::Untyped),
            bonus(ability(Ability::Intelligence), 2, Descriptor::Enhancement),
        ]);
        assert_eq!(
            item.summary(&skills()),
            "Int +2 enhancement; Cha -2; Persuasion -3"
        );
    }

    #[test]
    fn a_scoped_dc_and_the_caster_level() {
        let item = item(vec![
            bonus(GearTarget::CasterLevel, 3, Descriptor::Untyped),
            bonus(
                GearTarget::SpellDc(SpellScope::School(crate::SpellSchool::Evocation)),
                1,
                Descriptor::Untyped,
            ),
        ]);
        assert_eq!(
            item.summary(&skills()),
            "Spell DC (Evocation) +1; Caster level +3"
        );
    }

    #[test]
    fn always_on_lines_follow_the_bonuses() {
        let mut belt = item(
            [Ability::Strength, Ability::Dexterity, Ability::Constitution]
                .map(|a| bonus(ability(a), 4, Descriptor::Enhancement))
                .to_vec(),
        );
        belt.always_on = vec!["DR 3/-".to_owned(), "Immune to acid".to_owned()];
        assert_eq!(
            belt.summary(&skills()),
            "Str, Dex, Con +4 enhancement; DR 3/-; Immune to acid"
        );
    }
}
