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

pub fn pick_feat(key: &str) -> Pick {
    Pick::Feat(feat(key))
}
pub fn pick_option(key: &str) -> Pick {
    Pick::SelectionOption(option(key))
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
        feats: feats(),
        classes: vec![fighter(), rogue(), slayer(), wizard(), cleric(), monk()],
        prestige_classes: vec![arcane_trickster(), mystic_theurge()],
        mythic_abilities: mythic_abilities(),
        mythic_feats: mythic_feats(),
        mythic_paths: vec![trickster(), legend(), gold_dragon()],
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

fn feat_row(key: &str, category: FeatCategory, prerequisite: Option<Prerequisite>) -> Feat {
    Feat {
        key: feat(key),
        name: key.into(),
        category,
        summary: String::new(),
        guide: guide(&format!("feats#{key}")),
        prerequisite,
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
        feat_row("weapon-focus", Weapon, Some(BaseAttackBonus(1))),
        feat_row("skill-focus", Skill, None),
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
        feats: feats.iter().map(|key| feat(key)).collect(),
    }
}

fn plain_feature(key: &str, name: &str) -> Feature {
    Feature {
        key: feature(key),
        name: name.into(),
        summary: String::new(),
        guide: guide(key),
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
        feat,
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
            ],
        }],
        feat_lists: vec![],
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
        features: vec![plain_feature(
            "trickster/trickster-tricks",
            "Trickster Tricks",
        )],
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
            ],
        }],
        feat_lists: vec![list(
            "trickster/bonus-feats",
            false,
            &["dodge", "toughness"],
        )],
        ranks: vec![
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

fn legend() -> MythicPath {
    MythicPath {
        key: path("legend"),
        name: "Legend".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#legend"),
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        ranks: vec![],
        switch: Some(PathSwitch {
            kept_ranks: BTreeSet::from([1, 2]),
            further_ascensions: false,
            level_cap: Some(40),
        }),
    }
}

/// A switch path that keeps everything and carries on ascending.
fn gold_dragon() -> MythicPath {
    MythicPath {
        key: path("gold-dragon"),
        name: "Gold Dragon".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#gold-dragon"),
        features: vec![plain_feature("gold-dragon/breath", "Breath")],
        selections: vec![],
        feat_lists: vec![],
        ranks: vec![MythicRankRow {
            rank: 8,
            features: vec![feature("gold-dragon/breath")],
            feat_slots: vec![],
            selections: vec![],
        }],
        switch: Some(PathSwitch {
            kept_ranks: (1..=7).collect(),
            further_ascensions: true,
            level_cap: None,
        }),
    }
}
