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
            skill_row("skill:athletics", "Athletics", Ability::Strength),
            skill_row("skill:mobility", "Mobility", Ability::Dexterity),
            skill_row(
                "skill:knowledge-arcana",
                "Knowledge (Arcana)",
                Ability::Intelligence,
            ),
            skill_row("skill:persuasion", "Persuasion", Ability::Charisma),
        ],
        feats: feats(),
        classes: vec![fighter(), rogue(), wizard(), cleric()],
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
            key: RaceKey::new("race:human"),
            name: "Human".into(),
            summary: String::new(),
            guide: guide("character-creation#human"),
            ability_modifiers: vec![AbilityModifier::Chosen { amount: 2 }],
            feat_slots: vec![FeatSlot::General],
            subraces: vec![],
        },
        Race {
            key: RaceKey::new("race:elf"),
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
                key: SubraceKey::new("subrace:elf/blightborn"),
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
            key: RaceKey::new("race:dwarf"),
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
        name: key.trim_start_matches("feat:").into(),
        category,
        summary: String::new(),
        guide: guide(&format!("feats#{}", key.trim_start_matches("feat:"))),
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
            "feat:power-attack",
            Weapon,
            Some(needs_score(Ability::Strength, 13)),
        ),
        feat_row(
            "feat:cleave",
            Weapon,
            Some(All(vec![
                needs_feat("feat:power-attack"),
                BaseAttackBonus(1),
            ])),
        ),
        feat_row(
            "feat:great-cleave",
            Weapon,
            Some(All(vec![needs_feat("feat:cleave"), BaseAttackBonus(4)])),
        ),
        // Invented third link so a chain of two dependants can be tested.
        feat_row(
            "feat:cleaving-finish",
            Weapon,
            Some(needs_feat("feat:great-cleave")),
        ),
        feat_row(
            "feat:combat-expertise",
            CombatManoeuvres,
            Some(needs_score(Ability::Intelligence, 13)),
        ),
        feat_row(
            "feat:dodge",
            Miscellaneous,
            Some(needs_score(Ability::Dexterity, 13)),
        ),
        feat_row("feat:toughness", Miscellaneous, None),
        feat_row("feat:weapon-focus", Weapon, Some(BaseAttackBonus(1))),
        feat_row("feat:skill-focus", Skill, None),
        feat_row(
            "feat:arcane-strike",
            Magic,
            Some(SpellLevel {
                spell_type: SpellType::Arcane,
                level: 1,
            }),
        ),
        feat_row("feat:spell-penetration", Magic, Some(CasterLevel(5))),
        feat_row(
            "feat:outflank",
            Teamwork,
            Some(All(vec![
                BaseAttackBonus(4),
                Alignment(rules::Alignment::LawfulGood),
            ])),
        ),
        // "Combat Expertise, or Sneak Attack and Intelligence 13."
        feat_row(
            "feat:dirty-trick",
            CombatManoeuvres,
            Some(Any(vec![
                needs_feat("feat:combat-expertise"),
                All(vec![
                    Feature {
                        feature: feature("feature:rogue/sneak-attack"),
                        count: 1,
                    },
                    needs_score(Ability::Intelligence, 13),
                ]),
            ])),
        ),
        // "Two of: Power Attack, Dirty Trick, (Cleave or Rogue 3)", and Fighter 2.
        feat_row(
            "feat:critical-mastery",
            Weapon,
            Some(All(vec![
                AtLeast(
                    2,
                    vec![
                        needs_feat("feat:power-attack"),
                        needs_feat("feat:dirty-trick"),
                        Any(vec![
                            needs_feat("feat:cleave"),
                            ClassLevel {
                                class: class("class:rogue"),
                                level: 3,
                            },
                        ]),
                    ],
                ),
                ClassLevel {
                    class: class("class:fighter"),
                    level: 2,
                },
            ])),
        ),
        feat_row(
            "feat:breadth-of-experience",
            Skill,
            Some(All(vec![
                Any(vec![
                    Race(RaceKey::new("race:elf")),
                    Race(RaceKey::new("race:dwarf")),
                ]),
                CharacterLevel(2),
            ])),
        ),
        feat_row(
            "feat:deft-maneuvers",
            CombatManoeuvres,
            Some(SkillRanks {
                skill: skill("skill:mobility"),
                ranks: 3,
            }),
        ),
    ]
}

fn row(level: u8, bab: u8, fort: u8, reflex: u8, will: u8) -> ClassLevelRow {
    ClassLevelRow {
        level,
        bab,
        fort,
        reflex,
        will,
        arcane_spell_level: 0,
        divine_spell_level: 0,
        advances_arcane: false,
        advances_divine: false,
        features: vec![],
        feat_slots: vec![],
        selections: vec![],
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

/// Fighter: full BAB, good Fortitude, a Combat feat bonus slot at 1 and every even level.
fn fighter() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l, l, 2 + l / 2, l / 3, l / 3);
            if l == 1 || l % 2 == 0 {
                r.feat_slots.push(FeatSlot::CombatFeat);
            }
            if l % 4 == 2 {
                r.features.push(feature("feature:fighter/bravery"));
            }
            r
        })
        .collect();
    // Armiger: no Combat feat slots; an order selection at 1 and a fixed-list bonus slot at 2.
    let armiger_levels = (1..=20)
        .map(|l| {
            let mut r = row(l, l, 2 + l / 2, l / 3, l / 3);
            if l == 1 {
                r.selections
                    .push(FeatureSelectionKey::new("selection:fighter/armiger/order"));
            }
            if l == 2 {
                r.feat_slots.push(FeatSlot::OneOf(BTreeSet::from([
                    feat("feat:power-attack"),
                    feat("feat:combat-expertise"),
                ])));
            }
            r
        })
        .collect();
    Class {
        key: class("class:fighter"),
        name: "Fighter".into(),
        summary: String::new(),
        guide: guide("fighter"),
        skill_points: 2,
        class_skills: BTreeSet::from([skill("skill:athletics")]),
        prerequisite: None,
        features: vec![plain_feature("feature:fighter/bravery", "Bravery")],
        selections: vec![],
        levels,
        archetypes: vec![Archetype {
            key: ArchetypeKey::new("archetype:fighter/armiger"),
            name: "Armiger".into(),
            summary: String::new(),
            guide: guide("fighter#armiger"),
            prerequisite: None,
            skill_points: Some(4),
            class_skills: None,
            features: vec![],
            selections: vec![FeatureSelection {
                key: FeatureSelectionKey::new("selection:fighter/armiger/order"),
                name: "Order".into(),
                summary: String::new(),
                guide: guide("fighter#order"),
                options: vec![
                    selection_option("selection:fighter/armiger/order/cockatrice", None, None),
                    selection_option("selection:fighter/armiger/order/dragon", None, None),
                ],
            }],
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
    let talents = FeatureSelectionKey::new("selection:rogue/rogue-talents");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l, l * 3 / 4, l / 3, 2 + l / 2, l / 3);
            if l % 2 == 1 {
                r.features.push(feature("feature:rogue/sneak-attack"));
            } else {
                r.selections.push(talents.clone());
            }
            r
        })
        .collect();
    // Eldritch Scoundrel: arcane spells from level 1, the first Sneak Attack moves to level 2.
    let scoundrel_levels = (1..=20)
        .map(|l| {
            let mut r = row(l, l * 3 / 4, l / 3, 2 + l / 2, l / 3);
            r.arcane_spell_level = l.div_ceil(3).min(6);
            if l == 2 || (l % 2 == 1 && l > 1) {
                r.features.push(feature("feature:rogue/sneak-attack"));
            }
            if l % 2 == 0 {
                r.selections.push(talents.clone());
            }
            r
        })
        .collect();
    Class {
        key: class("class:rogue"),
        name: "Rogue".into(),
        summary: String::new(),
        guide: guide("rogue"),
        skill_points: 8,
        class_skills: BTreeSet::from([
            skill("skill:athletics"),
            skill("skill:mobility"),
            skill("skill:persuasion"),
        ]),
        prerequisite: None,
        features: vec![plain_feature("feature:rogue/sneak-attack", "Sneak Attack")],
        selections: vec![FeatureSelection {
            key: talents.clone(),
            name: "Rogue Talents".into(),
            summary: String::new(),
            guide: guide("rogue#rogue-talents"),
            options: vec![
                selection_option(
                    "selection:rogue/rogue-talents/combat-trick",
                    Some(feat("feat:combat-expertise")),
                    None,
                ),
                selection_option(
                    "selection:rogue/rogue-talents/crippling-strike",
                    None,
                    Some(Prerequisite::All(vec![
                        Prerequisite::ClassLevel {
                            class: class("class:rogue"),
                            level: 10,
                        },
                        Prerequisite::SelectionOption(option(
                            "selection:rogue/rogue-talents/combat-trick",
                        )),
                    ])),
                ),
                selection_option("selection:rogue/rogue-talents/fast-stealth", None, None),
            ],
        }],
        levels,
        archetypes: vec![Archetype {
            key: ArchetypeKey::new("archetype:rogue/eldritch-scoundrel"),
            name: "Eldritch Scoundrel".into(),
            summary: String::new(),
            guide: guide("rogue#eldritch-scoundrel"),
            prerequisite: Some(Prerequisite::Any(vec![
                Prerequisite::Race(RaceKey::new("race:human")),
                Prerequisite::Race(RaceKey::new("race:elf")),
            ])),
            skill_points: Some(4),
            class_skills: None,
            features: vec![],
            selections: vec![],
            levels: scoundrel_levels,
        }],
    }
}

/// Wizard: 1/2 BAB, good Will, full arcane spell-level access, a Magic bonus slot every 5 levels.
fn wizard() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l, l / 2, l / 3, l / 3, 2 + l / 2);
            r.arcane_spell_level = l.div_ceil(2).min(9);
            if l % 5 == 0 {
                r.feat_slots.push(FeatSlot::Category(FeatCategory::Magic));
            }
            r
        })
        .collect();
    Class {
        key: class("class:wizard"),
        name: "Wizard".into(),
        summary: String::new(),
        guide: guide("wizard"),
        skill_points: 2,
        class_skills: BTreeSet::from([skill("skill:knowledge-arcana")]),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Cleric: 3/4 BAB, good Fortitude and Will, full divine spell-level access.
fn cleric() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l, l * 3 / 4, 2 + l / 2, l / 3, 2 + l / 2);
            r.divine_spell_level = l.div_ceil(2).min(9);
            r
        })
        .collect();
    Class {
        key: class("class:cleric"),
        name: "Cleric".into(),
        summary: String::new(),
        guide: guide("cleric"),
        skill_points: 2,
        class_skills: BTreeSet::from([skill("skill:persuasion")]),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Mystic Theurge: every level advances both the chosen arcane class and the chosen divine class.
fn mystic_theurge() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l, l / 2, l / 3, l / 3, 2 + l / 2);
            r.advances_arcane = true;
            r.advances_divine = true;
            r
        })
        .collect();
    Class {
        key: class("class:mystic-theurge"),
        name: "Mystic Theurge".into(),
        summary: String::new(),
        guide: guide("prestige-classes#mystic-theurge"),
        skill_points: 2,
        class_skills: BTreeSet::from([skill("skill:knowledge-arcana")]),
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
        levels,
        archetypes: vec![],
    }
}

/// Arcane Trickster: every level advances the chosen arcane class; Sneak Attack at even levels.
fn arcane_trickster() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l, l / 2, l / 3, 2 + l / 2, 2 + l / 2);
            r.advances_arcane = true;
            if l % 2 == 0 {
                r.features.push(feature("feature:rogue/sneak-attack"));
            }
            r
        })
        .collect();
    Class {
        key: class("class:arcane-trickster"),
        name: "Arcane Trickster".into(),
        summary: String::new(),
        guide: guide("prestige-classes#arcane-trickster"),
        skill_points: 4,
        class_skills: BTreeSet::from([skill("skill:mobility")]),
        prerequisite: Some(Prerequisite::All(vec![
            Prerequisite::SkillRanks {
                skill: skill("skill:mobility"),
                ranks: 4,
            },
            Prerequisite::SpellLevel {
                spell_type: SpellType::Arcane,
                level: 2,
            },
            Prerequisite::Feature {
                feature: feature("feature:rogue/sneak-attack"),
                count: 2,
            },
        ])),
        features: vec![],
        selections: vec![],
        levels,
        archetypes: vec![],
    }
}

fn mythic_abilities() -> Vec<MythicAbility> {
    let ability = |key: &str, prerequisite| MythicAbility {
        key: MythicAbilityKey::new(key),
        name: key.into(),
        group: String::new(),
        summary: String::new(),
        guide: guide(key),
        prerequisite,
    };
    vec![
        ability("mythic-ability:mythic-charge", None),
        ability(
            "mythic-ability:limitless-smite",
            Some(Prerequisite::ClassLevel {
                class: class("class:fighter"),
                level: 1,
            }),
        ),
        ability("mythic-ability:unstoppable", None),
        ability("mythic-ability:inspired-strike", None),
        ability("mythic-ability:dimensional-retribution", None),
        ability("mythic-ability:abundant-casting", None),
    ]
}

fn mythic_feats() -> Vec<MythicFeat> {
    let mythic_feat = |key: &str, prerequisite| MythicFeat {
        key: MythicFeatKey::new(key),
        name: key.into(),
        group: String::new(),
        summary: String::new(),
        guide: guide(key),
        prerequisite,
    };
    vec![
        mythic_feat(
            "mythic-feat:power-attack-mythic",
            Some(needs_feat("feat:power-attack")),
        ),
        mythic_feat(
            "mythic-feat:toughness-mythic",
            Some(needs_feat("feat:toughness")),
        ),
        mythic_feat("mythic-feat:extra-mythic-ability", None),
        mythic_feat("mythic-feat:improved-critical-mythic", None),
        mythic_feat("mythic-feat:dodge-mythic", None),
        mythic_feat("mythic-feat:cleave-mythic", Some(needs_feat("feat:cleave"))),
    ]
}

fn trickster() -> MythicPath {
    let tricks = FeatureSelectionKey::new("selection:trickster/tricks");
    let rank = |rank, features: Vec<FeatureKey>, feat_slots, selections| MythicRankRow {
        rank,
        features,
        feat_slots,
        selections,
    };
    MythicPath {
        key: path("path:trickster"),
        name: "Trickster".into(),
        summary: String::new(),
        guide: guide("trickster"),
        features: vec![plain_feature(
            "feature:trickster/trickster-tricks",
            "Trickster Tricks",
        )],
        selections: vec![FeatureSelection {
            key: tricks.clone(),
            name: "Trickster Tricks".into(),
            summary: String::new(),
            guide: guide("trickster#tricks"),
            options: vec![
                selection_option("selection:trickster/tricks/perception-1", None, None),
                selection_option(
                    "selection:trickster/tricks/perception-2",
                    None,
                    Some(Prerequisite::SelectionOption(option(
                        "selection:trickster/tricks/perception-1",
                    ))),
                ),
                selection_option("selection:trickster/tricks/athletics-1", None, None),
                selection_option("selection:trickster/tricks/mobility-1", None, None),
                selection_option("selection:trickster/tricks/persuasion-1", None, None),
            ],
        }],
        ranks: vec![
            rank(
                3,
                vec![feature("feature:trickster/trickster-tricks")],
                vec![],
                vec![tricks.clone()],
            ),
            rank(
                4,
                vec![],
                vec![FeatSlot::OneOf(BTreeSet::from([
                    feat("feat:toughness"),
                    feat("feat:dodge"),
                ]))],
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
        key: path("path:legend"),
        name: "Legend".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#legend"),
        features: vec![],
        selections: vec![],
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
        key: path("path:gold-dragon"),
        name: "Gold Dragon".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#gold-dragon"),
        features: vec![plain_feature("feature:gold-dragon/breath", "Breath")],
        selections: vec![],
        ranks: vec![MythicRankRow {
            rank: 8,
            features: vec![feature("feature:gold-dragon/breath")],
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
