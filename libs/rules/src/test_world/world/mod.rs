//! The test world: an invented game, one file per part, and the helpers its parts share.

mod backgrounds;
mod classes;
mod deities;
mod feats;
mod mythic;
mod paths;
mod races;
mod selections;

use crate::*;

use self::backgrounds::backgrounds;
use self::classes::{
    arcane_trickster, bard, cleric, dragon_disciple, druid, eldritch_knight, fighter, kineticist,
    monk, mystic_theurge, ranger, rogue, slayer, sorcerer, warpriest, winter_witch, witch, wizard,
};
use self::deities::{deities, priest, seeker};
use self::feats::{feats, weapons};
use self::mythic::{mythic_abilities, mythic_feats};
use self::paths::{aeon, devil, gold_dragon, legend, swarm_that_walks, trickster};
use self::races::races;
use crate::test_world::builds::{feat, feat_list, feature, skill};

pub fn world() -> GameData {
    GameData {
        progression: progression(),
        races: races(),
        skills: vec![
            skill_row("athletics", "Athletics", Ability::Strength),
            skill_row("mobility", "Mobility", Ability::Dexterity),
            skill_row(
                "knowledge-arcana",
                "Knowledge (Arcana)",
                Ability::Intelligence,
            ),
            skill_row("persuasion", "Persuasion", Ability::Charisma),
        ],
        weapons: weapons(),
        feats: feats(),
        classes: vec![
            fighter(),
            rogue(),
            slayer(),
            wizard(),
            cleric(),
            monk(),
            witch(),
            sorcerer(),
            bard(),
            ranger(),
            warpriest(),
            druid(),
            kineticist(),
            priest(),
            seeker(),
        ],
        prestige_classes: vec![
            arcane_trickster(),
            mystic_theurge(),
            eldritch_knight(),
            winter_witch(),
            dragon_disciple(),
        ],
        mythic_abilities: mythic_abilities(),
        mythic_feats: mythic_feats(),
        bonuses: bonuses(),
        interactions: Vec::new(),
        mythic_paths: vec![
            trickster(),
            aeon(),
            legend(),
            gold_dragon(),
            swarm_that_walks(),
            devil(),
        ],
        backgrounds: backgrounds(),
        deities: deities(),
        spells: Vec::new(),
        spell_lists: Vec::new(),
    }
}

fn progression() -> Progression {
    let ascension = |rank, character_level| MythicAscension {
        rank,
        character_level,
    };
    Progression {
        mythic_ascensions: vec![
            ascension(1, 6),
            ascension(2, 7),
            ascension(3, 9),
            ascension(4, 11),
            ascension(5, 14),
            ascension(6, 15),
            ascension(7, 16),
            ascension(8, 17),
            ascension(9, 17),
            ascension(10, 19),
        ],
    }
}

fn guide(anchor: &str) -> GuideRef {
    GuideRef::new(anchor)
}

fn skill_row(key: &str, name: &str, ability: Ability) -> Skill {
    Skill {
        key: skill(key),
        name: name.into(),
        ability,
    }
}

fn list(key: &str, ignores_prerequisites: bool, feats: &[&str]) -> FeatList {
    FeatList {
        key: feat_list(key),
        name: key.into(),
        guide: guide(key),
        ignores_prerequisites,
        every_feat: false,
        feats: feats.iter().map(|key| feat(key)).collect(),
    }
}

fn plain_feature(key: &str, name: &str) -> Feature {
    Feature {
        key: feature(key),
        name: name.into(),
        summary: String::new(),
        guide: guide(key),
        ability_bonuses: vec![],
        option_row: None,
        daily_choice: false,
        bab_floor: None,
        skill_ranks_at_least_mythic_rank: false,
        favoured_weapon_proficiency: false,
    }
}

fn bonus_feature(key: &str, ability_bonuses: Vec<AbilityBonus>) -> Feature {
    Feature {
        ability_bonuses,
        ..plain_feature(key, key)
    }
}

/// The flat bonuses (ticket 119): Spell Focus and its mythic feat for the school they're held for, School Power's
/// two schools, Spell Penetration and the half-elf Dual Heritage's caster level against spell resistance.
fn bonuses() -> Vec<ItemBonus> {
    let bonus = |item, target, amount| ItemBonus {
        item,
        target,
        amount: BonusAmount::Flat(amount),
        requires: None,
    };
    let dc = |scope| BonusTarget::SpellDc(scope);
    vec![
        bonus(
            BonusItem::Feat(FeatKey::new("spell-focus")),
            dc(SpellScope::Value),
            1,
        ),
        bonus(
            BonusItem::MythicFeat(MythicFeatKey::new("spell-focus-mythic")),
            dc(SpellScope::Value),
            1,
        ),
        bonus(
            BonusItem::SelectionOption(SelectionOptionKey::new("sorcerer/school-power/evocation")),
            dc(SpellScope::School(SpellSchool::Evocation)),
            2,
        ),
        bonus(
            BonusItem::SelectionOption(SelectionOptionKey::new("sorcerer/school-power/necromancy")),
            dc(SpellScope::School(SpellSchool::Necromancy)),
            2,
        ),
        bonus(
            BonusItem::Feat(FeatKey::new("spell-penetration")),
            BonusTarget::SpellPenetration,
            2,
        ),
        bonus(
            BonusItem::Subrace(SubraceKey::new("half-elf/dual-heritage")),
            BonusTarget::SpellPenetration,
            2,
        ),
    ]
}
