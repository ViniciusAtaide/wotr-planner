//! The test world's gear: a few invented items, enough for every place kind, each bonus type's stacking and every
//! kind of number an item changes.

use crate::*;

use crate::test_world::builds::skill;

fn item(key: &str, name: &str, slot: GearSlot, bonuses: Vec<GearBonus>) -> GearItem {
    GearItem {
        key: GearKey::new(key),
        name: name.into(),
        slot,
        guide: None,
        bonuses,
        always_on: Vec::new(),
        other_effects: Vec::new(),
    }
}

fn bonus(target: GearTarget, amount: i8, kind: BonusType) -> GearBonus {
    GearBonus {
        target,
        amount,
        kind,
    }
}

fn score(ability: Ability, amount: i8, kind: BonusType) -> GearBonus {
    bonus(GearTarget::Ability(ability), amount, kind)
}

pub(super) fn gear() -> Vec<GearItem> {
    use Ability::*;
    use BonusType::*;
    vec![
        item(
            "belt-of-dexterity-2",
            "Belt of Dexterity +2",
            GearSlot::Belt,
            vec![score(Dexterity, 2, Enhancement)],
        ),
        item(
            "belt-of-constitution-2",
            "Belt of Constitution +2",
            GearSlot::Belt,
            vec![score(Constitution, 2, Enhancement)],
        ),
        item(
            "bracers-of-agility",
            "Bracers of Agility",
            GearSlot::Bracers,
            vec![score(Dexterity, 2, Untyped)],
        ),
        item(
            "gloves-of-dexterity-4",
            "Gloves of Dexterity +4",
            GearSlot::Gloves,
            vec![score(Dexterity, 4, Enhancement)],
        ),
        // A penalty to a score and one to a skill, both untyped, as the Bookworm's Headband has.
        item(
            "scholars-headband",
            "Scholar's Headband",
            GearSlot::Headwear,
            vec![
                score(Intelligence, 2, Enhancement),
                score(Charisma, -2, Untyped),
                bonus(GearTarget::Skill(skill("persuasion")), -3, Untyped),
            ],
        ),
        item(
            "cloak-of-resistance-3",
            "Cloak of Resistance +3",
            GearSlot::Cloak,
            vec![bonus(GearTarget::AllSaves, 3, Resistance)],
        ),
        item(
            "ring-of-fortitude",
            "Ring of Fortitude",
            GearSlot::Ring,
            vec![
                bonus(GearTarget::Save(Save::Fortitude), 5, Resistance),
                bonus(GearTarget::Initiative, 2, Luck),
            ],
        ),
        item(
            "ring-of-luck",
            "Ring of Luck",
            GearSlot::Ring,
            vec![
                bonus(GearTarget::Initiative, 1, Luck),
                bonus(GearTarget::Skill(skill("mobility")), 4, Competence),
            ],
        ),
        item(
            "robe-of-the-mage",
            "Robe of the Mage",
            GearSlot::Robe,
            vec![
                score(Charisma, 2, Enhancement),
                bonus(GearTarget::SpellDc(SpellScope::All), 1, Untyped),
            ],
        ),
        item(
            "plate-of-vigor",
            "Plate of Vigor",
            GearSlot::Armour,
            vec![score(Constitution, 2, Untyped)],
        ),
        item(
            "mage-staff",
            "Mage Staff",
            GearSlot::Weapon,
            vec![
                score(Intelligence, 6, Untyped),
                bonus(GearTarget::CasterLevel, 3, Untyped),
                bonus(GearTarget::SpellPenetration, 2, Competence),
                bonus(GearTarget::Concentration, 2, Competence),
            ],
        ),
        item(
            "manual-of-dexterity-2",
            "Manual of Dexterity +2",
            GearSlot::Usable,
            vec![score(Dexterity, 2, Inherent)],
        ),
        item(
            "tome-of-perfection-4",
            "Tome of Perfection +4",
            GearSlot::Usable,
            vec![
                score(Strength, 4, Inherent),
                score(Dexterity, 4, Inherent),
                score(Constitution, 4, Inherent),
                score(Intelligence, 4, Inherent),
                score(Wisdom, 4, Inherent),
                score(Charisma, 4, Inherent),
            ],
        ),
    ]
}
