//! The test world: an invented game, one file per part, and the helpers its parts share.

mod backgrounds;
mod classes;
mod deities;
mod feats;
mod gear;
mod mythic;
mod nested;
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
    let mut data = GameData {
        progression: progression(),
        races: vec![],
        subraces: vec![],
        feat_lists: vec![],
        archetypes: vec![],
        features: vec![],
        selections: vec![],
        options: vec![],
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
        classes: vec![],
        prestige_classes: vec![],
        mythic_abilities: mythic_abilities(),
        mythic_feats: mythic_feats(),
        interactions: Vec::new(),
        gear: gear::gear(),
        mythic_paths: vec![],
        backgrounds: backgrounds(),
        deities: deities(),
        spells: Vec::new(),
        spell_lists: Vec::new(),
    };
    let nested = nested::Nested {
        races: races(),
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
        mythic_paths: vec![
            trickster(),
            aeon(),
            legend(),
            gold_dragon(),
            swarm_that_walks(),
            devil(),
        ],
    };
    nested::flatten(nested, &mut data);
    attach_bonuses(&mut data);
    data
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

fn guide(anchor: &str) -> Option<GuideRef> {
    Some(GuideRef::new(anchor))
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
        blueprint: None,
        name: key.into(),
        guide: guide(key),
        ignores_prerequisites,
        every_feat: false,
        skips_proficiency: false,
        feats: feats.iter().map(|key| feat(key)).collect(),
    }
}

fn plain_feature(key: &str, name: &str) -> Feature {
    Feature {
        components: Vec::new(),
        key: feature(key),
        blueprint: None,
        name: name.into(),
        summary: String::new(),
        guide: guide(key),
        option_row: None,
        daily_choice: false,
        favoured_weapon_proficiency: false,
        weapons: std::collections::BTreeSet::new(),
    }
}

/// A feature whose components are its numbers (ticket 297): the ability bonuses and floors of the paths' features.
fn bonus_feature(key: &str, components: Vec<Component>) -> Feature {
    Feature {
        components,
        ..plain_feature(key, key)
    }
}

/// `+amount` to one score for each grant, as a feature's ability bonus has always counted: of type `Racial`, so it
/// counts toward ability prerequisites and stacks.
fn fixed(ability: Ability, amount: i16) -> Component {
    Component::StatBonus {
        stat: StatTarget::Ability(ability),
        value: Value::PerRank(amount),
        descriptor: Descriptor::Racial,
        multiplier: 1,
        at_least: None,
        only_if: None,
    }
}

/// `+amount` to every score ([`fixed`] for each).
fn every(amount: i16) -> Vec<Component> {
    AbilityScores::ABILITIES
        .into_iter()
        .map(|ability| fixed(ability, amount))
        .collect()
}

/// The bonuses (ticket 119), as components on their entries (ticket 297): Spell Focus for the school it's held for,
/// doubled by its mythic feat; School Power's two schools; Spell Penetration and the half-elf Dual Heritage's caster
/// level against spell resistance; and the attack line's Weapon Focus, Weapon Specialization and Power Attack.
fn attach_bonuses(data: &mut GameData) {
    let dc = |school, amount| Component::SpellDc {
        scope: SpellScope::School(school),
        value: Value::Flat(amount),
    };
    let penetration = || Component::SpellPenetration {
        value: Value::Flat(2),
        only_while: None,
    };
    for entry in &mut data.feats {
        match entry.key.as_str() {
            "spell-focus" => entry.components.push(Component::SpellFocus {
                bonus: 1,
                mythic_focus: Fact::MythicFeat(MythicFeatKey::new("spell-focus-mythic")),
            }),
            "spell-penetration" => entry.components.push(penetration()),
            // The attack line's kinds (ticket 297).
            "weapon-focus" => entry.components.push(Component::WeaponFocus),
            "weapon-specialization" => entry.components.push(Component::WeaponSpecialization),
            "power-attack" => entry.components.push(Component::PowerAttack),
            _ => {}
        }
    }
    for entry in &mut data.options {
        match entry.key.as_str() {
            "sorcerer/school-power/evocation" => {
                entry.components.push(dc(SpellSchool::Evocation, 2));
            }
            "sorcerer/school-power/necromancy" => {
                entry.components.push(dc(SpellSchool::Necromancy, 2));
            }
            _ => {}
        }
    }
    for entry in &mut data.subraces {
        if entry.key.as_str() == "half-elf/dual-heritage" {
            entry.components.push(penetration());
        }
    }
}
