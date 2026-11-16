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

use std::collections::BTreeSet;

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
    data.selections.extend(pools(&data.feats));
    data.selections.extend(granted_selections());
    list_feats_in_pools(&mut data);
    attach_bonuses(&mut data);
    data
}

/// Lists `data`'s feats as they are now in the general and category selections ([`pools`]) and Gold Dragon's Dragon
/// Feats (every feat), and its mythic abilities in Extra Mythic Ability's selection (ticket 302), so a feat or mythic
/// ability a test adds to the world is offered as the world's own are.
pub fn list_feats_in_pools(data: &mut GameData) {
    let mut pools = pools(&data.feats);
    pools.push(FeatureSelection {
        feats: data.feats.iter().map(|feat| feat.key.clone()).collect(),
        ..feat_selection("gold-dragon/dragon-feats", "")
    });
    let abilities: BTreeSet<MythicAbilityKey> = (data.mythic_abilities.iter())
        .map(|ability| ability.key.clone())
        .collect();
    for selection in &mut data.selections {
        if let Some(pool) = pools.iter().find(|pool| pool.key == selection.key) {
            selection.feats.clone_from(&pool.feats);
        }
        if selection.key.as_str() == EXTRA_MYTHIC_ABILITY {
            selection.mythic_abilities.clone_from(&abilities);
        }
    }
}

/// The selection of mythic abilities Extra Mythic Ability grants, listing every one (filled by
/// [`list_feats_in_pools`]).
const EXTRA_MYTHIC_ABILITY: &str = "mythic/extra-mythic-ability";

/// The selections the Extra feats and Extra Mythic Ability grant (ticket 302): the feats' name the selections a
/// character must have had a slot of, and give a slot of the first it had; Extra Mythic Ability's lists every mythic
/// ability.
fn granted_selections() -> Vec<FeatureSelection> {
    let first_had = |key: &str, of: &[&str]| FeatureSelection {
        needs_slot_of: of.iter().map(|key| feat_list(key)).collect(),
        ..feat_selection(key, "")
    };
    vec![
        first_had(
            "rogue/extra-rogue-talent",
            &["rogue/rogue-talents", "slayer/slayer-talents"],
        ),
        first_had("witch/extra-hex", &["witch/hex"]),
        first_had("slayer/slayer-trick", &["slayer/slayer-talents"]),
        FeatureSelection {
            band: Band::MythicAbility,
            ..feat_selection(EXTRA_MYTHIC_ABILITY, "Mythic ability")
        },
    ]
}

/// The selections of feats of the general slot and the category slots, each listing the feats it offers (ticket
/// 301): every feat, the Combat feats, the Magic feats and the Teamwork feats.
fn pools(feats: &[Feat]) -> Vec<FeatureSelection> {
    let pool = |key: &str, name: &str, offers: &dyn Fn(&Feat) -> bool| FeatureSelection {
        feats: (feats.iter())
            .filter(|feat| offers(feat))
            .map(|feat| feat.key.clone())
            .collect(),
        ..feat_selection(key, name)
    };
    vec![
        pool(BASIC_FEAT_SELECTION, "Feat", &|_| true),
        pool("combat-feats", "Combat feat", &|feat| {
            feat.category.is_combat()
        }),
        pool("magic-feats", "Magic feat", &|feat| {
            feat.category == FeatCategory::Magic
        }),
        pool("teamwork-feats", "Teamwork feat", &|feat| {
            feat.category == FeatCategory::Teamwork
        }),
    ]
}

/// A selection of feats with nothing set, named `name`.
fn feat_selection(key: &str, name: &str) -> FeatureSelection {
    FeatureSelection {
        components: Vec::new(),
        key: feat_list(key),
        blueprint: None,
        name: name.into(),
        summary: String::new(),
        guide: guide(key),
        options: BTreeSet::new(),
        feats: BTreeSet::new(),
        mythic_abilities: BTreeSet::new(),
        needs_slot_of: BTreeSet::new(),
        ignores_prerequisites: false,
        skips_proficiency: false,
        option_lists: None,
        shared_options: BTreeSet::new(),
        option_levels: BTreeSet::new(),
        option_level_archetypes: std::collections::BTreeMap::new(),
        half_other_classes: false,
        skipped_levels: BTreeSet::new(),
        replaced_by: std::collections::BTreeMap::new(),
        feat_lists_only: false,
        only_without_option: false,
        filtered_by: None,
        later_rows_behind: 0,
        level_from: None,
        level_also_from: None,
        once_per_option: false,
        deity_domains: None,
        band: Band::Features,
    }
}

/// A selection of feats whose lists are those of the options held that `of` offers ([`FeatureSelection::option_lists`]),
/// at `at_level` or the option's own level, named as the option-list slots were: `of`'s name and "feat".
fn option_list(key: &str, of: &str, at_level: Option<u8>) -> FeatureSelection {
    FeatureSelection {
        option_lists: Some(OptionLists {
            selection: FeatureSelectionKey::new(of),
            at_level,
        }),
        ..feat_selection(key, &format!("{of} feat"))
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

fn list(key: &str, ignores_prerequisites: bool, feats: &[&str]) -> FeatureSelection {
    FeatureSelection {
        ignores_prerequisites,
        feats: feats.iter().map(|key| feat(key)).collect(),
        ..feat_selection(key, key)
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
