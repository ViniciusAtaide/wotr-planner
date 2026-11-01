//! A small hand-built game world for the rules engine tests, plus helpers for writing Builds.
//!
//! Invented and simplified: the numbers follow Pathfinder's usual tables, but nothing here is checked against the
//! guide. Full 20-level tables are generated so mythic ascensions and Legend can be exercised.

#![allow(dead_code)]

use std::collections::BTreeSet;

use rules::*;

// ---- keys -----------------------------------------------------------------------------------------------------

pub fn feat(key: &str) -> FeatKey {
    FeatKey::new(key)
}
pub fn class(key: &str) -> ClassKey {
    ClassKey::new(key)
}
pub fn skill(key: &str) -> SkillKey {
    SkillKey::new(key)
}
pub fn path(key: &str) -> MythicPathKey {
    MythicPathKey::new(key)
}
pub fn feature(key: &str) -> FeatureKey {
    FeatureKey::new(key)
}
pub fn option(key: &str) -> SelectionOptionKey {
    SelectionOptionKey::new(key)
}
pub fn feat_list(key: &str) -> FeatListKey {
    FeatListKey::new(key)
}

pub fn weapon(key: &str) -> Parameter {
    Parameter::Weapon(WeaponKey::new(key))
}
pub fn skill_value(key: &str) -> Parameter {
    Parameter::Skill(skill(key))
}

pub fn pick_feat(key: &str) -> Pick {
    Pick::feat(feat(key))
}
/// A parameterised feat taken for `value`.
pub fn pick_feat_for(key: &str, value: Parameter) -> Pick {
    Pick::feat_for(feat(key), value)
}
pub fn pick_option(key: &str) -> Pick {
    Pick::option(option(key))
}
/// An option whose chosen feat is taken for `value`.
pub fn pick_option_for(key: &str, value: Parameter) -> Pick {
    Pick::option_for(option(key), value)
}
pub fn pick_mythic_ability(key: &str) -> Pick {
    Pick::MythicAbility(MythicAbilityKey::new(key))
}
pub fn pick_mythic_feat(key: &str) -> Pick {
    Pick::MythicFeat(MythicFeatKey::new(key))
}

// ---- slots ----------------------------------------------------------------------------------------------------

pub fn general_slot(level: u8) -> SlotId {
    SlotId::Feat {
        level,
        source: FeatSource::General,
        index: 0,
    }
}
pub fn race_slot(index: u8) -> SlotId {
    SlotId::Feat {
        level: 1,
        source: FeatSource::Race,
        index,
    }
}
pub fn class_slot(level: u8, index: u8) -> SlotId {
    SlotId::Feat {
        level,
        source: FeatSource::Class,
        index,
    }
}
pub fn selection_slot(level: u8, index: u8) -> SlotId {
    SlotId::Selection { level, index }
}

// ---- builds ---------------------------------------------------------------------------------------------------

/// Ability scores in the order Str, Dex, Con, Int, Wis, Cha.
pub fn scores(values: [u8; 6]) -> AbilityScores {
    let [
        strength,
        dexterity,
        constitution,
        intelligence,
        wisdom,
        charisma,
    ] = values;
    AbilityScores {
        strength,
        dexterity,
        constitution,
        intelligence,
        wisdom,
        charisma,
    }
}

/// A Build of the given race with Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8 (21 of 25 points), true neutral,
/// and one Level-up per class key.
pub fn build(race: &str, classes: &[&str]) -> Build {
    let mut build = Build {
        start: BuildStart {
            race: Some(RaceKey::new(race)),
            alignment: Some(Alignment::TrueNeutral),
            ability_scores: scores([16, 14, 12, 13, 10, 8]),
            ..BuildStart::default()
        },
        ..Build::default()
    };
    for key in classes {
        build.levels.push(LevelUp::new(class(key)));
    }
    build
}

/// `n` copies of one class key.
pub fn times(key: &'static str, n: usize) -> Vec<&'static str> {
    vec![key; n]
}

pub fn engine() -> Engine {
    Engine::new(world())
}

// ---- the world ------------------------------------------------------------------------------------------------

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
        ],
        prestige_classes: vec![
            arcane_trickster(),
            mystic_theurge(),
            eldritch_knight(),
            winter_witch(),
        ],
        mythic_abilities: mythic_abilities(),
        mythic_feats: mythic_feats(),
        mythic_paths: vec![
            trickster(),
            aeon(),
            legend(),
            gold_dragon(),
            swarm_that_walks(),
            devil(),
        ],
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

fn races() -> Vec<Race> {
    let fixed = |ability, amount| AbilityModifier::Fixed { ability, amount };
    vec![
        Race {
            key: RaceKey::new("human"),
            name: "Human".into(),
            summary: String::new(),
            guide: guide("character-creation#human"),
            ability_modifiers: vec![AbilityModifier::Chosen { amount: 2 }],
            feat_slots: vec![FeatSlot::General],
            subraces: vec![],
        },
        Race {
            key: RaceKey::new("elf"),
            name: "Elf".into(),
            summary: String::new(),
            guide: guide("character-creation#elf"),
            ability_modifiers: vec![
                fixed(Ability::Dexterity, 2),
                fixed(Ability::Intelligence, 2),
                fixed(Ability::Constitution, -2),
            ],
            feat_slots: vec![],
            subraces: vec![Subrace {
                key: SubraceKey::new("elf/blightborn"),
                name: "Blightborn".into(),
                summary: String::new(),
                guide: guide("character-creation#elf-blightborn"),
                ability_modifiers: vec![
                    fixed(Ability::Dexterity, 2),
                    fixed(Ability::Wisdom, 2),
                    fixed(Ability::Constitution, -2),
                ],
            }],
        },
        Race {
            key: RaceKey::new("dwarf"),
            name: "Dwarf".into(),
            summary: String::new(),
            guide: guide("character-creation#dwarf"),
            ability_modifiers: vec![
                fixed(Ability::Constitution, 2),
                fixed(Ability::Wisdom, 2),
                fixed(Ability::Charisma, -2),
            ],
            feat_slots: vec![],
            subraces: vec![],
        },
    ]
}

/// Five weapons, one per kind of entry: simple, martial, two exotic, and one no proficiency list names.
fn weapons() -> Vec<Weapon> {
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

fn needs_feat(key: &str) -> Prerequisite {
    Prerequisite::Feat(feat(key))
}

fn needs_score(ability: Ability, score: u8) -> Prerequisite {
    Prerequisite::AbilityScore { ability, score }
}

fn feats() -> Vec<Feat> {
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
        repeat_feat_row("extra-rogue-talent", Class, Repeat::Times(3)),
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
    ]
}

fn row(level: u8) -> ClassLevelRow {
    ClassLevelRow {
        level,
        arcane_spell_level: 0,
        divine_spell_level: 0,
        advances_arcane: false,
        advances_divine: false,
        features: vec![],
        feats: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
    }
}

/// An archetype's row at one level: no grants and the base class's spell levels until filled in.
fn archetype_row(level: u8) -> ArchetypeLevelRow {
    ArchetypeLevelRow {
        level,
        spell_levels: None,
        features: vec![],
        feats: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
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
    }
}

fn bonus_feature(key: &str, ability_bonuses: Vec<AbilityBonus>) -> Feature {
    Feature {
        ability_bonuses,
        ..plain_feature(key, key)
    }
}

/// Fighter: full BAB, good Fortitude, Shield and Tower Shield Proficiency granted at 1, a Combat feat bonus slot
/// at 1 and every even level.
fn fighter() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 1 {
                r.feats =
                    BTreeSet::from([feat("shield-proficiency"), feat("tower-shield-proficiency")]);
            }
            if l == 1 || l % 2 == 0 {
                r.feat_slots.push(FeatSlot::CombatFeat);
            }
            if l % 4 == 2 {
                r.features.push(feature("fighter/bravery"));
            }
            r
        })
        .collect();
    // Armiger: no Combat feat slots, no Bravery and no granted proficiencies, so a row at 1 and every even level
    // (empty ones take the fighter's grants away); an order selection at 1 and a fixed-list bonus slot at 2. Odd
    // levels from 3 are the fighter's.
    let armiger_levels = (1..=20)
        .filter(|l| l % 2 == 0 || *l == 1)
        .map(|l| {
            let mut r = archetype_row(l);
            if l == 1 {
                r.selections
                    .push(FeatureSelectionKey::new("fighter/armiger/order"));
            }
            if l == 2 {
                r.feat_slots.push(FeatSlot::OneOf {
                    list: feat_list("fighter/armiger/bonus-feats"),
                });
            }
            r
        })
        .collect();
    Class {
        key: class("fighter"),
        name: "Fighter".into(),
        summary: String::new(),
        guide: guide("fighter"),
        skill_points: 2,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("athletics")]),
        prerequisite: None,
        features: vec![plain_feature("fighter/bravery", "Bravery")],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![Archetype {
            key: ArchetypeKey::new("fighter/armiger"),
            name: "Armiger".into(),
            summary: String::new(),
            guide: guide("fighter#armiger"),
            prerequisite: None,
            skill_points: Some(4),
            class_skills: None,
            features: vec![],
            selections: vec![FeatureSelection {
                key: FeatureSelectionKey::new("fighter/armiger/order"),
                name: "Order".into(),
                summary: String::new(),
                guide: guide("fighter#order"),
                shared_options: BTreeSet::new(),
                options: vec![
                    selection_option("fighter/armiger/order/cockatrice", None, None),
                    selection_option("fighter/armiger/order/dragon", None, None),
                ],
            }],
            feat_lists: vec![list(
                "fighter/armiger/bonus-feats",
                false,
                &["combat-expertise", "power-attack"],
            )],
            levels: armiger_levels,
        }],
    }
}

fn selection_option(
    key: &str,
    feat: Option<FeatKey>,
    prerequisite: Option<Prerequisite>,
) -> SelectionOption {
    SelectionOption {
        key: option(key),
        name: key.rsplit('/').next().unwrap_or(key).into(),
        summary: String::new(),
        guide: guide(key),
        feats: feat.into_iter().collect(),
        chosen_feat: None,
        repeat: Repeat::Once,
        prerequisite,
    }
}

/// Rogue: 3/4 BAB, good Reflex, Sneak Attack at odd levels, a rogue talent at even levels.
fn rogue() -> Class {
    let talents = FeatureSelectionKey::new("rogue/rogue-talents");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l % 2 == 1 {
                r.features.push(feature("rogue/sneak-attack"));
            } else {
                r.selections.push(talents.clone());
            }
            if l == 3 {
                r.feat_slots.push(FeatSlot::OneOf {
                    list: feat_list("rogue/skill-focus"),
                });
            }
            r
        })
        .collect();
    // Eldritch Scoundrel: arcane spells from level 1 (so a row at every level), the first Sneak Attack moves to
    // level 2.
    let scoundrel_levels = (1..=20)
        .map(|l| {
            let mut r = archetype_row(l);
            r.spell_levels = Some(SpellLevels {
                arcane: l.div_ceil(3).min(6),
                divine: 0,
            });
            if l == 2 || (l % 2 == 1 && l > 1) {
                r.features.push(feature("rogue/sneak-attack"));
            }
            if l % 2 == 0 {
                r.selections.push(talents.clone());
            }
            r
        })
        .collect();
    Class {
        key: class("rogue"),
        name: "Rogue".into(),
        summary: String::new(),
        guide: guide("rogue"),
        skill_points: 8,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("athletics"), skill("mobility"), skill("persuasion")]),
        prerequisite: None,
        features: vec![plain_feature("rogue/sneak-attack", "Sneak Attack")],
        selections: vec![FeatureSelection {
            key: talents.clone(),
            name: "Rogue Talents".into(),
            summary: String::new(),
            guide: guide("rogue#rogue-talents"),
            shared_options: BTreeSet::new(),
            options: vec![
                selection_option(
                    "rogue/rogue-talents/combat-trick",
                    Some(feat("combat-expertise")),
                    None,
                ),
                selection_option(
                    "rogue/rogue-talents/crippling-strike",
                    None,
                    Some(Prerequisite::All(vec![
                        Prerequisite::ClassLevel {
                            class: class("rogue"),
                            level: 10,
                        },
                        Prerequisite::SelectionOption(option("rogue/rogue-talents/combat-trick")),
                    ])),
                ),
                selection_option("rogue/rogue-talents/fast-stealth", None, None),
                // Weapon Focus in a weapon of the player's choosing.
                SelectionOption {
                    chosen_feat: Some(feat("weapon-focus")),
                    ..selection_option("rogue/rogue-talents/weapon-focus", None, None)
                },
                // Invented, like Increased Damage Reduction: taken up to twice.
                SelectionOption {
                    repeat: Repeat::Times(2),
                    ..selection_option("rogue/rogue-talents/resiliency", None, None)
                },
            ],
        }],
        // A Skill Focus of the player's choosing at 3, like the Master of All's.
        feat_lists: vec![list("rogue/skill-focus", true, &["skill-focus"])],
        levels,
        archetypes: vec![Archetype {
            key: ArchetypeKey::new("rogue/eldritch-scoundrel"),
            name: "Eldritch Scoundrel".into(),
            summary: String::new(),
            guide: guide("rogue#eldritch-scoundrel"),
            prerequisite: Some(Prerequisite::Any(vec![
                Prerequisite::Race(RaceKey::new("human")),
                Prerequisite::Race(RaceKey::new("elf")),
            ])),
            skill_points: Some(4),
            class_skills: None,
            features: vec![],
            selections: vec![],
            feat_lists: vec![],
            levels: scoundrel_levels,
        }],
    }
}

/// Slayer: full BAB, good Fortitude and Reflex, a slayer talent at even levels. Slayer talents share the rogue's
/// three talents and add Slayer Camouflage, which needs the rogue's Fast Stealth.
fn slayer() -> Class {
    let talents = FeatureSelectionKey::new("slayer/slayer-talents");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l % 2 == 0 {
                r.selections.push(talents.clone());
            }
            r
        })
        .collect();
    Class {
        key: class("slayer"),
        name: "Slayer".into(),
        summary: String::new(),
        guide: guide("slayer"),
        skill_points: 6,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("athletics"), skill("mobility")]),
        prerequisite: None,
        features: vec![],
        selections: vec![FeatureSelection {
            key: talents,
            name: "Slayer Talents".into(),
            summary: String::new(),
            guide: guide("slayer#slayer-talents"),
            options: vec![selection_option(
                "slayer/slayer-talents/slayer-camouflage",
                None,
                Some(Prerequisite::SelectionOption(option(
                    "rogue/rogue-talents/fast-stealth",
                ))),
            )],
            shared_options: [
                "rogue/rogue-talents/combat-trick",
                "rogue/rogue-talents/crippling-strike",
                "rogue/rogue-talents/fast-stealth",
            ]
            .into_iter()
            .map(option)
            .collect(),
        }],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Wizard: 1/2 BAB, good Will, full arcane spell-level access, Scribe Scrolls granted at 1, a Magic bonus slot
/// every 5 levels.
fn wizard() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            r.arcane_spell_level = l.div_ceil(2).min(9);
            if l == 1 {
                r.feats.insert(feat("scribe-scrolls"));
            }
            if l % 5 == 0 {
                r.feat_slots.push(FeatSlot::Category(FeatCategory::Magic));
            }
            r
        })
        .collect();
    Class {
        key: class("wizard"),
        name: "Wizard".into(),
        summary: String::new(),
        guide: guide("wizard"),
        skill_points: 2,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("knowledge-arcana")]),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Monk: 3/4 BAB, good saves, Improved Unarmed Strike granted at 1, a bonus-list slot (Crane Style or Dodge,
/// prerequisites ignored) at 1 and at 2, 6, 10, 14 and 18.
fn monk() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l == 1 {
                r.feats.insert(feat("improved-unarmed-strike"));
            }
            if l == 1 || l % 4 == 2 {
                r.feat_slots.push(FeatSlot::OneOf {
                    list: feat_list("monk/bonus-feats"),
                });
            }
            r
        })
        .collect();
    Class {
        key: class("monk"),
        name: "Monk".into(),
        summary: String::new(),
        guide: guide("monk"),
        skill_points: 4,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Good,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("mobility")]),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![list("monk/bonus-feats", true, &["crane-style", "dodge"])],
        levels,
        archetypes: vec![],
    }
}

/// Cleric: 3/4 BAB, good Fortitude and Will, full divine spell-level access.
fn cleric() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            r.divine_spell_level = l.div_ceil(2).min(9);
            r
        })
        .collect();
    Class {
        key: class("cleric"),
        name: "Cleric".into(),
        summary: String::new(),
        guide: guide("cleric"),
        skill_points: 2,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("persuasion")]),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Mystic Theurge: every level advances both the chosen arcane class and the chosen divine class.
fn mystic_theurge() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l);
            r.advances_arcane = true;
            r.advances_divine = true;
            r
        })
        .collect();
    Class {
        key: class("mystic-theurge"),
        name: "Mystic Theurge".into(),
        summary: String::new(),
        guide: guide("prestige-classes#mystic-theurge"),
        skill_points: 2,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("knowledge-arcana")]),
        prerequisite: Some(Prerequisite::All(vec![
            Prerequisite::SpellLevel {
                spell_type: SpellType::Arcane,
                level: 2,
            },
            Prerequisite::SpellLevel {
                spell_type: SpellType::Divine,
                level: 2,
            },
        ])),
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Arcane Trickster: every level advances the chosen arcane class; Sneak Attack at even levels.
fn arcane_trickster() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l);
            r.advances_arcane = true;
            if l % 2 == 0 {
                r.features.push(feature("rogue/sneak-attack"));
            }
            r
        })
        .collect();
    Class {
        key: class("arcane-trickster"),
        name: "Arcane Trickster".into(),
        summary: String::new(),
        guide: guide("prestige-classes#arcane-trickster"),
        skill_points: 4,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Good,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("mobility")]),
        prerequisite: Some(Prerequisite::All(vec![
            Prerequisite::SkillRanks {
                skill: skill("mobility"),
                ranks: 4,
            },
            Prerequisite::SpellLevel {
                spell_type: SpellType::Arcane,
                level: 2,
            },
            Prerequisite::Feature {
                feature: feature("rogue/sneak-attack"),
                count: 2,
            },
        ])),
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Witch: 1/2 BAB, good Will, full arcane spell-level access like the wizard, a hex at 1 and every even level,
/// Major Hexes at 10 and Grand Hexes at 18 (which the hexes that need them check).
fn witch() -> Class {
    let hex = FeatureSelectionKey::new("witch/hex");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            r.arcane_spell_level = l.div_ceil(2).min(9);
            if l == 1 || l % 2 == 0 {
                r.selections.push(hex.clone());
            }
            if l == 10 {
                r.features.push(feature("witch/major-hexes"));
            }
            if l == 18 {
                r.features.push(feature("witch/grand-hexes"));
            }
            r
        })
        .collect();
    let needs_feature = |key: &str| {
        Some(Prerequisite::Feature {
            feature: feature(key),
            count: 1,
        })
    };
    Class {
        key: class("witch"),
        name: "Witch".into(),
        summary: String::new(),
        guide: guide("witch"),
        skill_points: 2,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("knowledge-arcana")]),
        prerequisite: None,
        features: vec![
            plain_feature("witch/major-hexes", "Major Hexes"),
            plain_feature("witch/grand-hexes", "Grand Hexes"),
        ],
        selections: vec![FeatureSelection {
            key: hex,
            name: "Hex".into(),
            summary: String::new(),
            guide: guide("witch#hex"),
            shared_options: BTreeSet::new(),
            options: vec![
                selection_option("witch/hex/slumber", None, None),
                selection_option("witch/hex/misfortune", None, None),
                selection_option("witch/hex/agony", None, needs_feature("witch/major-hexes")),
                selection_option(
                    "witch/hex/life-giver",
                    None,
                    needs_feature("witch/grand-hexes"),
                ),
            ],
        }],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Eldritch Knight: full BAB, good Fortitude, a Combat feat bonus slot at 1, 5 and 9, the arcane class advanced
/// from 2; its levels count as fighter levels for feat prerequisites (Diverse Training).
fn eldritch_knight() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l);
            r.advances_arcane = l >= 2;
            if l % 4 == 1 {
                r.feat_slots.push(FeatSlot::CombatFeat);
            }
            r
        })
        .collect();
    Class {
        key: class("eldritch-knight"),
        name: "Eldritch Knight".into(),
        summary: String::new(),
        guide: guide("prestige-classes#eldritch-knight"),
        skill_points: 2,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::from([CountsAs {
            class: class("fighter"),
            toward: CountsToward::FeatPrerequisites,
        }]),
        class_skills: BTreeSet::from([skill("athletics")]),
        prerequisite: Some(Prerequisite::SpellLevel {
            spell_type: SpellType::Arcane,
            level: 2,
        }),
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Winter Witch: every level advances one class, arcane or divine; its levels stack with witch levels for Major
/// and Grand Hexes (Frost Power).
fn winter_witch() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l);
            r.advances_arcane = true;
            r.advances_divine = true;
            r
        })
        .collect();
    let toward_witch = |key: &str| CountsAs {
        class: class("witch"),
        toward: CountsToward::Feature(feature(key)),
    };
    Class {
        key: class("winter-witch"),
        name: "Winter Witch".into(),
        summary: String::new(),
        guide: guide("prestige-classes#winter-witch"),
        skill_points: 2,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Either,
        counts_as: BTreeSet::from([
            toward_witch("witch/major-hexes"),
            toward_witch("witch/grand-hexes"),
        ]),
        class_skills: BTreeSet::from([skill("knowledge-arcana")]),
        prerequisite: Some(Prerequisite::Any(vec![
            Prerequisite::SpellLevel {
                spell_type: SpellType::Arcane,
                level: 2,
            },
            Prerequisite::SpellLevel {
                spell_type: SpellType::Divine,
                level: 2,
            },
        ])),
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

fn mythic_abilities() -> Vec<MythicAbility> {
    let ability = |key: &str, prerequisite| MythicAbility {
        key: MythicAbilityKey::new(key),
        name: key.into(),
        group: MythicAbilityGroup::Melee,
        summary: String::new(),
        guide: guide(key),
        prerequisite,
    };
    vec![
        ability("mythic-charge", None),
        ability(
            "limitless-smite",
            Some(Prerequisite::ClassLevel {
                class: class("fighter"),
                level: 1,
            }),
        ),
        ability("unstoppable", None),
        ability("inspired-strike", None),
        ability("dimensional-retribution", None),
        ability("abundant-casting", None),
    ]
}

fn mythic_feats() -> Vec<MythicFeat> {
    let mythic_feat = |key: &str, prerequisite| MythicFeat {
        key: MythicFeatKey::new(key),
        name: key.into(),
        group: MythicFeatGroup::Melee,
        summary: String::new(),
        guide: guide(key),
        prerequisite,
    };
    vec![
        mythic_feat("power-attack-mythic", Some(needs_feat("power-attack"))),
        mythic_feat("toughness-mythic", Some(needs_feat("toughness"))),
        mythic_feat("extra-mythic-ability", None),
        mythic_feat("improved-critical-mythic", None),
        mythic_feat("dodge-mythic", None),
        mythic_feat("cleave-mythic", Some(needs_feat("cleave"))),
    ]
}

fn trickster() -> MythicPath {
    let tricks = FeatureSelectionKey::new("trickster/tricks");
    let rank = |rank, features: Vec<FeatureKey>, feat_slots, selections| MythicRankRow {
        rank,
        features,
        feat_slots,
        selections,
    };
    MythicPath {
        key: path("trickster"),
        name: "Trickster".into(),
        summary: String::new(),
        guide: guide("trickster"),
        features: vec![
            plain_feature("trickster/trickster-tricks", "Trickster Tricks"),
            plain_feature("trickster/bit-of-fun", "Bit of Fun"),
            plain_feature("trickster/hard-to-kill", "Hard to Kill"),
        ],
        selections: vec![FeatureSelection {
            key: tricks.clone(),
            name: "Trickster Tricks".into(),
            summary: String::new(),
            guide: guide("trickster#tricks"),
            shared_options: BTreeSet::new(),
            options: vec![
                selection_option("trickster/tricks/perception-1", None, None),
                selection_option(
                    "trickster/tricks/perception-2",
                    None,
                    Some(Prerequisite::SelectionOption(option(
                        "trickster/tricks/perception-1",
                    ))),
                ),
                selection_option("trickster/tricks/athletics-1", None, None),
                selection_option("trickster/tricks/mobility-1", None, None),
                selection_option("trickster/tricks/persuasion-1", None, None),
                // Invented, like Azata's All-skilled: one option granting several feats.
                SelectionOption {
                    feats: BTreeSet::from([
                        feat("shield-proficiency"),
                        feat("tower-shield-proficiency"),
                    ]),
                    ..selection_option("trickster/tricks/all-armour", None, None)
                },
                // Like All-skilled's Exotic Weapon Proficiency: every value of a parameterised feat.
                selection_option(
                    "trickster/tricks/all-weapons",
                    Some(feat("exotic-weapon-proficiency")),
                    None,
                ),
            ],
        }],
        feat_lists: vec![list(
            "trickster/bonus-feats",
            false,
            &["dodge", "toughness"],
        )],
        ranks: vec![
            rank(1, vec![feature("trickster/bit-of-fun")], vec![], vec![]),
            rank(2, vec![feature("trickster/hard-to-kill")], vec![], vec![]),
            rank(
                3,
                vec![feature("trickster/trickster-tricks")],
                vec![],
                vec![tricks.clone()],
            ),
            rank(
                4,
                vec![],
                vec![FeatSlot::OneOf {
                    list: feat_list("trickster/bonus-feats"),
                }],
                vec![],
            ),
            rank(5, vec![], vec![], vec![tricks.clone()]),
            rank(7, vec![], vec![], vec![tricks.clone()]),
            rank(9, vec![], vec![], vec![tricks.clone()]),
        ],
        switch: None,
    }
}

/// A path's rank row.
fn rank_row(
    rank: u8,
    features: &[&str],
    feat_slots: Vec<FeatSlot>,
    selections: &[&str],
) -> MythicRankRow {
    MythicRankRow {
        rank,
        features: features.iter().map(|key| feature(key)).collect(),
        feat_slots,
        selections: selections
            .iter()
            .map(|key| FeatureSelectionKey::new(*key))
            .collect(),
    }
}

/// Legend: keeps MR 1-2 (general picks and the old path's grants), stays at MR 3, raises the cap to 40, and the
/// switch grants +4 to every score.
fn legend() -> MythicPath {
    MythicPath {
        key: path("legend"),
        name: "Legend".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#legend"),
        features: vec![bonus_feature(
            "legend/legend",
            vec![AbilityBonus::Every { amount: 4 }],
        )],
        selections: vec![],
        feat_lists: vec![],
        ranks: vec![],
        switch: Some(PathSwitch {
            from: BTreeSet::new(),
            kept_through_rank: 2,
            path_kept_through_rank: 2,
            dropped_features: BTreeSet::new(),
            dropped_selections: BTreeSet::new(),
            features: BTreeSet::from([feature("legend/legend")]),
            further_ascensions: false,
            level_cap: Some(40),
        }),
    }
}

/// Gold Dragon: keeps the general picks of MR 1-7 but the old path's grants of MR 1-2 only, and carries on
/// ascending. Perfect Body, Mind and Soul raise the scores; Dragon Feats are three picks of any feat ignoring
/// prerequisites; two breaths need MR 9.
fn gold_dragon() -> MythicPath {
    let dragon_feats = || FeatSlot::OneOf {
        list: feat_list("gold-dragon/dragon-feats"),
    };
    MythicPath {
        key: path("gold-dragon"),
        name: "Gold Dragon".into(),
        summary: String::new(),
        guide: guide("gold-dragon"),
        features: vec![
            plain_feature("gold-dragon/breath", "Breath"),
            bonus_feature(
                "gold-dragon/perfect-body",
                vec![
                    AbilityBonus::AtLeast { score: 14 },
                    AbilityBonus::Highest { amount: 4 },
                ],
            ),
            bonus_feature(
                "gold-dragon/perfect-mind",
                vec![
                    AbilityBonus::AtLeast { score: 18 },
                    AbilityBonus::Highest { amount: 2 },
                ],
            ),
            bonus_feature(
                "gold-dragon/perfect-soul",
                vec![AbilityBonus::Highest { amount: 2 }],
            ),
        ],
        selections: vec![FeatureSelection {
            key: FeatureSelectionKey::new("gold-dragon/breaths"),
            name: "Breaths".into(),
            summary: String::new(),
            guide: guide("gold-dragon#breaths"),
            shared_options: BTreeSet::new(),
            options: vec![
                selection_option("gold-dragon/breaths/empowered", None, None),
                // The guide's "Mythic Rank: 9" and "Vow of Dahak": the story condition never blocks.
                selection_option(
                    "gold-dragon/breaths/annihilating",
                    None,
                    Some(Prerequisite::All(vec![
                        Prerequisite::MythicRank(9),
                        Prerequisite::Story(Story::VowOfDahak),
                    ])),
                ),
            ],
        }],
        feat_lists: vec![FeatList {
            every_feat: true,
            ..list("gold-dragon/dragon-feats", true, &[])
        }],
        ranks: vec![
            rank_row(
                8,
                &["gold-dragon/breath", "gold-dragon/perfect-body"],
                vec![],
                &["gold-dragon/breaths"],
            ),
            rank_row(
                9,
                &["gold-dragon/perfect-mind"],
                vec![dragon_feats(), dragon_feats(), dragon_feats()],
                &["gold-dragon/breaths"],
            ),
            rank_row(10, &["gold-dragon/perfect-soul"], vec![], &[]),
        ],
        switch: Some(PathSwitch {
            from: BTreeSet::new(),
            kept_through_rank: 7,
            path_kept_through_rank: 2,
            dropped_features: BTreeSet::new(),
            dropped_selections: BTreeSet::new(),
            features: BTreeSet::new(),
            further_ascensions: true,
            level_cap: None,
        }),
    }
}

/// Swarm That Walks: Gold Dragon's keeps; Eye of the Swarm at MR 9 raises Strength and Dexterity by 4.
fn swarm_that_walks() -> MythicPath {
    MythicPath {
        key: path("swarm-that-walks"),
        name: "Swarm That Walks".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#swarm-that-walks"),
        features: vec![bonus_feature(
            "swarm-that-walks/eye-of-the-swarm",
            vec![
                AbilityBonus::Fixed {
                    ability: Ability::Strength,
                    amount: 4,
                },
                AbilityBonus::Fixed {
                    ability: Ability::Dexterity,
                    amount: 4,
                },
            ],
        )],
        selections: vec![],
        feat_lists: vec![],
        ranks: vec![rank_row(
            9,
            &["swarm-that-walks/eye-of-the-swarm"],
            vec![],
            &[],
        )],
        switch: Some(PathSwitch {
            from: BTreeSet::new(),
            kept_through_rank: 7,
            path_kept_through_rank: 2,
            dropped_features: BTreeSet::new(),
            dropped_selections: BTreeSet::new(),
            features: BTreeSet::new(),
            further_ascensions: true,
            level_cap: None,
        }),
    }
}

/// Aeon: MR 1-2 features, Aeon Bane at 4, an Enforcing Gaze pick at MR 3-7 (Area Effects needs MR 5) and Extra
/// Gaze at 6.
fn aeon() -> MythicPath {
    MythicPath {
        key: path("aeon"),
        name: "Aeon".into(),
        summary: String::new(),
        guide: guide("aeon"),
        features: vec![
            plain_feature("aeon/force-reality", "Force Reality"),
            plain_feature("aeon/hard-to-kill", "Hard to Kill"),
            plain_feature("aeon/aeon-bane", "Aeon Bane"),
            plain_feature("aeon/enforcing-gaze-extra-gaze", "Extra Gaze"),
        ],
        selections: vec![FeatureSelection {
            key: FeatureSelectionKey::new("aeon/enforcing-gaze"),
            name: "Enforcing Gaze".into(),
            summary: String::new(),
            guide: guide("aeon#enforcing-gaze"),
            shared_options: BTreeSet::new(),
            options: vec![
                selection_option("aeon/enforcing-gaze/attack", None, None),
                selection_option("aeon/enforcing-gaze/damage", None, None),
                selection_option("aeon/enforcing-gaze/save-dc", None, None),
                selection_option(
                    "aeon/enforcing-gaze/area-effects",
                    None,
                    Some(Prerequisite::MythicRank(5)),
                ),
            ],
        }],
        feat_lists: vec![],
        ranks: vec![
            rank_row(1, &["aeon/force-reality"], vec![], &[]),
            rank_row(2, &["aeon/hard-to-kill"], vec![], &[]),
            rank_row(3, &[], vec![], &["aeon/enforcing-gaze"]),
            rank_row(4, &["aeon/aeon-bane"], vec![], &["aeon/enforcing-gaze"]),
            rank_row(5, &[], vec![], &["aeon/enforcing-gaze"]),
            rank_row(
                6,
                &["aeon/enforcing-gaze-extra-gaze"],
                vec![],
                &["aeon/enforcing-gaze"],
            ),
            rank_row(7, &[], vec![], &["aeon/enforcing-gaze"]),
        ],
        switch: None,
    }
}

/// Devil: only from Aeon; keeps MR 1-7 but loses the Aeon's gaze (its picks and Extra Gaze).
fn devil() -> MythicPath {
    MythicPath {
        key: path("devil"),
        name: "Devil".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#devil"),
        features: vec![plain_feature("devil/hells-dominance", "Hell's Dominance")],
        selections: vec![],
        feat_lists: vec![],
        ranks: vec![rank_row(8, &["devil/hells-dominance"], vec![], &[])],
        switch: Some(PathSwitch {
            from: BTreeSet::from([path("aeon")]),
            kept_through_rank: 7,
            path_kept_through_rank: 7,
            dropped_features: BTreeSet::from([feature("aeon/enforcing-gaze-extra-gaze")]),
            dropped_selections: BTreeSet::from([FeatureSelectionKey::new("aeon/enforcing-gaze")]),
            features: BTreeSet::new(),
            further_ascensions: true,
            level_cap: None,
        }),
    }
}
