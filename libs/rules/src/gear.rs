//! Gear (ticket 165): the named items a character wears or wields, each with the bonuses it always gives, and a
//! summary of them generated from the data. `gear.ron` lists them. Nothing in a Build names an item yet: the engine
//! reads them in ticket 166.
//!
//! A bonus has a [`GearTarget`] (an ability score, or a number the planner already computes), an amount and a
//! [`BonusType`]. What an item does besides (a bonus in some circumstance, a use per day, who may wear it) is a short
//! line of the planner's own words in [`GearItem::other_effects`].

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::bonus::{BonusTarget, BonusType, SpellScope};
use crate::game_data::{Ability, GuideRef, Save, Skill};
use crate::keys::{GearKey, SkillKey};

/// A named item: where it is worn, what it always gives, and what else it does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearItem {
    pub key: GearKey,
    pub name: String,
    /// The guide section it is listed in.
    pub slot: GearSlot,
    /// `page#section`: the guide has no anchor per item. `None` for an item the guide doesn't list (the generic
    /// stat belts and headbands, from the user).
    #[serde(default)]
    pub guide: Option<GuideRef>,
    /// Its always-on bonuses, one per number changed.
    pub bonuses: Vec<GearBonus>,
    /// Everything else it does, one short line each, in the planner's own words.
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

/// One always-on bonus of an item.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearBonus {
    pub target: GearTarget,
    /// Never 0. Negative for a penalty, which is always [`BonusType::Untyped`].
    pub amount: i8,
    /// Left out in the file: untyped.
    #[serde(default = "untyped")]
    pub kind: BonusType,
}

fn untyped() -> BonusType {
    BonusType::Untyped
}

/// What a [`GearBonus`] adds to. Declared in the order the summary lists them.
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
    /// The [`BonusTarget`] it adds to; `None` for an ability score, which goes through the scores, and for the
    /// caster level, which the caster line reads on its own.
    pub fn bonus_target(&self) -> Option<BonusTarget> {
        match self {
            Self::Ability(_) | Self::CasterLevel => None,
            Self::AllSaves => Some(BonusTarget::AllSaves),
            Self::Save(save) => Some(BonusTarget::Save(*save)),
            Self::Initiative => Some(BonusTarget::Initiative),
            Self::Skill(skill) => Some(BonusTarget::Skill(skill.clone())),
            Self::SpellDc(scope) => Some(BonusTarget::SpellDc(*scope)),
            Self::SpellPenetration => Some(BonusTarget::SpellPenetration),
            Self::Concentration => Some(BonusTarget::Concentration),
        }
    }

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
    /// The summary of its bonuses, generated so the web app and the MCP show one text: the bonuses in
    /// [`GearTarget`]'s order, those sharing an amount and a type grouped ("Str, Dex, Con +6 enhancement"), groups
    /// joined by "; ", an untyped bonus with no type, no full stop. `skills` gives a skill target its name.
    pub fn summary(&self, skills: &[Skill]) -> String {
        let mut bonuses: Vec<&GearBonus> = self.bonuses.iter().collect();
        bonuses.sort_by(|a, b| a.target.cmp(&b.target));
        let mut groups: Vec<(i8, BonusType, Vec<String>)> = Vec::new();
        for bonus in bonuses {
            let name = bonus.target.name(skills);
            match groups
                .iter_mut()
                .find(|(amount, kind, _)| *amount == bonus.amount && *kind == bonus.kind)
            {
                Some((_, _, names)) => names.push(name),
                None => groups.push((bonus.amount, bonus.kind, vec![name])),
            }
        }
        let mut text = String::new();
        for (index, (amount, kind, names)) in groups.iter().enumerate() {
            if index > 0 {
                text.push_str("; ");
            }
            let _ = write!(text, "{} {amount:+}", names.join(", "));
            if *kind != BonusType::Untyped {
                let _ = write!(text, " {}", kind.name());
            }
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

    fn bonus(target: GearTarget, amount: i8, kind: BonusType) -> GearBonus {
        GearBonus {
            target,
            amount,
            kind,
        }
    }

    fn item(bonuses: Vec<GearBonus>) -> GearItem {
        GearItem {
            key: GearKey::new("test"),
            name: "Test".to_owned(),
            slot: GearSlot::Belt,
            guide: None,
            bonuses,
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
            BonusType::Enhancement,
        )]);
        assert_eq!(item.summary(&skills()), "Dex +6 enhancement");
    }

    #[test]
    fn three_scores_are_grouped() {
        let item = item(
            [Ability::Constitution, Ability::Strength, Ability::Dexterity]
                .map(|a| bonus(ability(a), 6, BonusType::Enhancement))
                .to_vec(),
        );
        assert_eq!(item.summary(&skills()), "Str, Dex, Con +6 enhancement");
    }

    #[test]
    fn a_cloak() {
        let item = item(vec![
            bonus(GearTarget::AllSaves, 7, BonusType::Resistance),
            bonus(ability(Ability::Charisma), 2, BonusType::Circumstance),
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
                BonusType::Competence,
            ),
            bonus(
                GearTarget::Skill(SkillKey::new("knowledge-world")),
                5,
                BonusType::Competence,
            ),
            bonus(ability(Ability::Charisma), 2, BonusType::Enhancement),
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
                BonusType::Untyped,
            ),
            bonus(ability(Ability::Charisma), -2, BonusType::Untyped),
            bonus(ability(Ability::Intelligence), 2, BonusType::Enhancement),
        ]);
        assert_eq!(
            item.summary(&skills()),
            "Int +2 enhancement; Cha -2; Persuasion -3"
        );
    }

    #[test]
    fn a_scoped_dc_and_the_caster_level() {
        let item = item(vec![
            bonus(GearTarget::CasterLevel, 3, BonusType::Untyped),
            bonus(
                GearTarget::SpellDc(SpellScope::School(crate::SpellSchool::Evocation)),
                1,
                BonusType::Untyped,
            ),
        ]);
        assert_eq!(
            item.summary(&skills()),
            "Spell DC (Evocation) +1; Caster level +3"
        );
    }
}
