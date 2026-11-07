//! The test world's weapons and feats.

use std::collections::BTreeSet;

use rules::*;

use crate::support::builds::{class, feat, feature, option, skill, skill_value, weapon};
use crate::support::world::guide;

/// Five weapons, one per kind of entry: simple, martial, two exotic, and one no proficiency list names.
pub(super) fn weapons() -> Vec<Weapon> {
    let weapon = |key: &str, proficiency| Weapon {
        key: WeaponKey::new(key),
        name: key.into(),
        proficiency,
    };
    vec![
        weapon("dagger", Some(WeaponProficiency::Simple)),
        weapon("longsword", Some(WeaponProficiency::Martial)),
        weapon("dueling-sword", Some(WeaponProficiency::Exotic)),
        weapon("bastard-sword", Some(WeaponProficiency::Exotic)),
        weapon("unarmed-strike", None),
    ]
}

fn feat_row(key: &str, category: FeatCategory, prerequisite: Option<Prerequisite>) -> Feat {
    Feat {
        key: feat(key),
        name: key.into(),
        category,
        summary: String::new(),
        guide: guide(&format!("feats#{key}")),
        parameter: None,
        repeat: Repeat::Once,
        extra_pick_of: BTreeSet::new(),
        prerequisite,
    }
}

/// A parameterised feat.
fn param_feat_row(
    key: &str,
    category: FeatCategory,
    parameter: ParameterKind,
    repeat: Repeat,
    prerequisite: Option<Prerequisite>,
) -> Feat {
    Feat {
        parameter: Some(parameter),
        repeat,
        ..feat_row(key, category, prerequisite)
    }
}

/// A feat taken more than once, for nothing in particular.
fn repeat_feat_row(key: &str, category: FeatCategory, repeat: Repeat) -> Feat {
    Feat {
        repeat,
        ..feat_row(key, category, None)
    }
}

/// A feat taken more than once that gives one more slot of one of `selections` each time.
fn extra_pick_row(key: &str, category: FeatCategory, repeat: Repeat, selections: &[&str]) -> Feat {
    Feat {
        extra_pick_of: selections
            .iter()
            .map(|key| FeatureSelectionKey::new(*key))
            .collect(),
        ..repeat_feat_row(key, category, repeat)
    }
}

pub(super) fn needs_feat(key: &str) -> Prerequisite {
    Prerequisite::Feat(feat(key))
}

fn needs_score(ability: Ability, score: u8) -> Prerequisite {
    Prerequisite::AbilityScore { ability, score }
}

pub(super) fn feats() -> Vec<Feat> {
    use FeatCategory::*;
    use Prerequisite::*;
    vec![
        feat_row(
            "power-attack",
            Weapon,
            Some(needs_score(Ability::Strength, 13)),
        ),
        feat_row(
            "cleave",
            Weapon,
            Some(All(vec![needs_feat("power-attack"), BaseAttackBonus(1)])),
        ),
        feat_row(
            "great-cleave",
            Weapon,
            Some(All(vec![needs_feat("cleave"), BaseAttackBonus(4)])),
        ),
        // Invented third link so a chain of two dependants can be tested.
        feat_row("cleaving-finish", Weapon, Some(needs_feat("great-cleave"))),
        feat_row(
            "combat-expertise",
            CombatManoeuvres,
            Some(needs_score(Ability::Intelligence, 13)),
        ),
        feat_row(
            "dodge",
            Miscellaneous,
            Some(needs_score(Ability::Dexterity, 13)),
        ),
        feat_row("toughness", Miscellaneous, None),
        // Once per weapon, as the guide says.
        param_feat_row(
            "weapon-focus",
            Weapon,
            ParameterKind::Weapon,
            Repeat::PerValue,
            Some(BaseAttackBonus(1)),
        ),
        // "Choose a single weapon for which you have already selected Weapon Focus."
        param_feat_row(
            "weapon-specialization",
            Weapon,
            ParameterKind::Weapon,
            Repeat::PerValue,
            Some(All(vec![
                FeatForSameValue(feat("weapon-focus")),
                ClassLevel {
                    class: class("fighter"),
                    level: 4,
                },
            ])),
        ),
        feat_row(
            "aldori-dueling-mastery",
            Weapon,
            Some(FeatFor {
                feat: feat("weapon-focus"),
                value: weapon("dueling-sword"),
            }),
        ),
        param_feat_row(
            "exotic-weapon-proficiency",
            Weapon,
            ParameterKind::ExoticWeapon,
            Repeat::PerValue,
            Some(BaseAttackBonus(1)),
        ),
        param_feat_row(
            "skill-focus",
            Skill,
            ParameterKind::Skill,
            Repeat::PerValue,
            None,
        ),
        feat_row(
            "acrobatic-spellcaster",
            Magic,
            Some(FeatFor {
                feat: feat("skill-focus"),
                value: skill_value("mobility"),
            }),
        ),
        // Parameterised but taken once: the guide doesn't say it repeats.
        param_feat_row(
            "spell-focus",
            Magic,
            ParameterKind::SpellSchool,
            Repeat::Once,
            None,
        ),
        feat_row(
            "augment-summoning",
            Magic,
            Some(FeatFor {
                feat: feat("spell-focus"),
                value: Parameter::SpellSchool(SpellSchool::Conjuration),
            }),
        ),
        // One more rogue talent, or slayer talent for a character who had a slayer talent slot first (as Extra Hex
        // names the witch's and the shaman's hexes).
        extra_pick_row(
            "extra-rogue-talent",
            Class,
            Repeat::Times(3),
            &["rogue/rogue-talents", "slayer/slayer-talents"],
        ),
        extra_pick_row("extra-hex", Class, Repeat::Times(3), &["witch/hex"]),
        // Invented: a Combat feat with an extra pick, so a fighter's bonus slot can hold one before the general
        // slot's at the same level.
        extra_pick_row(
            "slayer-trick",
            Weapon,
            Repeat::Unlimited,
            &["slayer/slayer-talents"],
        ),
        repeat_feat_row("extra-ki", Class, Repeat::Unlimited),
        feat_row(
            "arcane-strike",
            Magic,
            Some(SpellLevel {
                spell_type: SpellType::Arcane,
                level: 1,
            }),
        ),
        feat_row("spell-penetration", Magic, Some(CasterLevel(5))),
        feat_row(
            "outflank",
            Teamwork,
            Some(All(vec![
                BaseAttackBonus(4),
                Alignment(rules::Alignment::LawfulGood),
            ])),
        ),
        // "Combat Expertise, or Sneak Attack and Intelligence 13."
        feat_row(
            "dirty-trick",
            CombatManoeuvres,
            Some(Any(vec![
                needs_feat("combat-expertise"),
                All(vec![
                    Feature {
                        feature: feature("rogue/sneak-attack"),
                        count: 1,
                    },
                    needs_score(Ability::Intelligence, 13),
                ]),
            ])),
        ),
        // "Two of: Power Attack, Dirty Trick, (Cleave or Rogue 3)", and Fighter 2.
        feat_row(
            "critical-mastery",
            Weapon,
            Some(All(vec![
                AtLeast(
                    2,
                    vec![
                        needs_feat("power-attack"),
                        needs_feat("dirty-trick"),
                        Any(vec![
                            needs_feat("cleave"),
                            ClassLevel {
                                class: class("rogue"),
                                level: 3,
                            },
                        ]),
                    ],
                ),
                ClassLevel {
                    class: class("fighter"),
                    level: 2,
                },
            ])),
        ),
        feat_row(
            "breadth-of-experience",
            Skill,
            Some(All(vec![
                Any(vec![Race(RaceKey::new("elf")), Race(RaceKey::new("dwarf"))]),
                CharacterLevel(2),
            ])),
        ),
        feat_row(
            "deft-maneuvers",
            CombatManoeuvres,
            Some(SkillRanks {
                skill: skill("mobility"),
                ranks: 3,
            }),
        ),
        // Simplified: the game's Crane Style also needs Improved Unarmed Strike.
        feat_row("crane-style", UnarmedCombat, Some(needs_feat("dodge"))),
        // Invented prerequisite, so a monk's grant of it shows that grants skip prerequisites.
        feat_row(
            "improved-unarmed-strike",
            UnarmedCombat,
            Some(needs_score(Ability::Dexterity, 13)),
        ),
        // Simplified: the game's Stunning Fist also needs Dex 13, Wis 13 and BAB +8.
        feat_row(
            "stunning-fist",
            UnarmedCombat,
            Some(needs_feat("improved-unarmed-strike")),
        ),
        feat_row("scribe-scrolls", Magic, Some(CasterLevel(1))),
        feat_row("shield-proficiency", ArmourAndShield, None),
        feat_row(
            "tower-shield-proficiency",
            ArmourAndShield,
            Some(needs_feat("shield-proficiency")),
        ),
        feat_row(
            "shield-focus",
            ArmourAndShield,
            Some(needs_feat("shield-proficiency")),
        ),
        // The Trickster's Perception II unlocks it, like the guide's Stat Focus.
        feat_row(
            "stat-focus",
            Miscellaneous,
            Some(SelectionOption(option("trickster/tricks/perception-2"))),
        ),
        feat_row(
            "crane-wing",
            UnarmedCombat,
            Some(All(vec![needs_feat("crane-style"), needs_feat("dodge")])),
        ),
        // Weapon Focus in any weapon, as the guide's Dazzling Display needs.
        feat_row("dazzling-display", Weapon, Some(needs_feat("weapon-focus"))),
        // The druid's companion through Nature Bond.
        feat_row(
            "boon-companion",
            Miscellaneous,
            Some(SelectionOption(option(
                "druid/nature-bond/animal-companion",
            ))),
        ),
        feat_row("armor-proficiency-light", ArmourAndShield, None),
        feat_row("armor-proficiency-heavy", ArmourAndShield, None),
        // "Armor Proficiency (Light, Medium or Heavy)": the one of the armour type taken.
        param_feat_row(
            "armor-focus",
            ArmourAndShield,
            ParameterKind::ArmourType,
            Repeat::PerValue,
            Some(All(vec![
                FeatIfValue {
                    value: Parameter::ArmourType(ArmourType::Light),
                    feat: feat("armor-proficiency-light"),
                },
                FeatIfValue {
                    value: Parameter::ArmourType(ArmourType::Heavy),
                    feat: feat("armor-proficiency-heavy"),
                },
            ])),
        ),
    ]
}
