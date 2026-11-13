//! What the sheet calls things: names by key, slot labels, picker categories and readable reasons for an Issue.

use std::collections::BTreeSet;

use rules::{
    Ability, Alignment, ArchetypeKey, Engine, FeatCategory, FeatSlot, FeatureSelectionKey, Issue,
    MythicPathKey, OptionGroup, Parameter, Prerequisite, RaceKey, Slot, SlotId, SlotKind,
    SpellType, Story, Unmet,
};

use crate::support::*;

fn needs(prerequisite: Prerequisite, has: Option<u8>) -> Unmet {
    Unmet::Condition {
        needs: prerequisite,
        has,
    }
}

#[test]
fn names_come_from_the_game_data() {
    let engine = engine();
    assert_eq!(engine.class_name(&class("rogue")), "Rogue");
    assert_eq!(
        engine.archetype_name(&ArchetypeKey::new("rogue/eldritch-scoundrel")),
        "Eldritch Scoundrel"
    );
    assert_eq!(engine.race_name(&RaceKey::new("half-elf")), "Half-Elf");
    assert_eq!(engine.skill_name(&skill("mobility")), "Mobility");
    assert_eq!(
        engine.feature_name(&feature("rogue/sneak-attack")),
        "Sneak Attack"
    );
    assert_eq!(
        engine.selection_name(&FeatureSelectionKey::new("rogue/rogue-talents")),
        "Rogue Talents"
    );
    assert_eq!(engine.path_name(&path("legend")), "Legend");
}

#[test]
fn a_key_the_data_lacks_is_named_by_its_key() {
    let engine = engine();
    assert_eq!(engine.class_name(&class("gunslinger")), "gunslinger");
    assert_eq!(engine.feat_name(&feat("no-such-feat")), "no-such-feat");
    assert_eq!(
        engine.path_name(&MythicPathKey::new("no-such-path")),
        "no-such-path"
    );
}

#[test]
fn the_start_pickers_list_the_races_backgrounds_and_skills_in_key_order() {
    let engine = engine();
    let races: Vec<&str> = engine.races().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(races, ["dwarf", "elf", "half-elf", "human"]);
    let subraces: Vec<&str> = engine.races()[2]
        .subraces
        .iter()
        .map(|s| s.key.as_str())
        .collect();
    assert_eq!(subraces, ["half-elf/adaptable", "half-elf/dual-heritage"]);
    let backgrounds: Vec<&str> = engine
        .backgrounds()
        .iter()
        .map(|b| b.key.as_str())
        .collect();
    assert_eq!(
        backgrounds,
        [
            "aldori-swordsman",
            "guard",
            "martial-disciple",
            "none",
            "shackles-corsair"
        ]
    );
    let skills: Vec<&str> = engine.skills().iter().map(|s| s.key.as_str()).collect();
    assert_eq!(
        skills,
        ["athletics", "knowledge-arcana", "mobility", "persuasion"]
    );
}

#[test]
fn a_background_note_and_a_path_summary_come_from_the_game_data() {
    let mut data = world();
    let legend = data
        .mythic_paths
        .iter_mut()
        .find(|p| p.key == path("legend"))
        .unwrap();
    legend.summary = "Raises the level cap to 40.".into();
    let engine = engine_on(data);
    assert_eq!(
        engine.background_note(&rules::BackgroundKey::new("guard")),
        "Shortsword and light crossbow proficiency."
    );
    assert_eq!(
        engine.background_note(&rules::BackgroundKey::new("martial-disciple")),
        ""
    );
    assert_eq!(
        engine.path_summary(&path("legend")),
        "Raises the level cap to 40."
    );
    assert_eq!(engine.path_summary(&path("aeon")), "");
    // A key the data lacks has none.
    assert_eq!(
        engine.background_note(&rules::BackgroundKey::new("no-such-background")),
        ""
    );
    assert_eq!(engine.path_summary(&path("no-such-path")), "");
}

#[test]
fn a_class_and_an_archetype_summary_come_from_the_game_data() {
    let mut data = world();
    let sorcerer = data
        .classes
        .iter_mut()
        .find(|c| c.key == class("sorcerer"))
        .unwrap();
    sorcerer.summary = "Spontaneous arcane caster.".into();
    let sage = sorcerer
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == "sorcerer/sage")
        .unwrap();
    sage.summary = "Casts with Intelligence.".into();
    let engine = engine_on(data);
    assert_eq!(
        engine.class_summary(&class("sorcerer")),
        "Spontaneous arcane caster."
    );
    assert_eq!(
        engine.archetype_summary(&rules::ArchetypeKey::new("sorcerer/sage")),
        "Casts with Intelligence."
    );
    assert_eq!(engine.class_summary(&class("wizard")), "");
    // A key the data lacks has none.
    assert_eq!(engine.class_summary(&class("no-such-class")), "");
    assert_eq!(
        engine.archetype_summary(&rules::ArchetypeKey::new("no-such/archetype")),
        ""
    );
}

#[test]
fn a_pick_is_named_with_its_value() {
    let engine = engine();
    assert_eq!(engine.pick_name(&pick_feat("power-attack")), "power-attack");
    assert_eq!(
        engine.pick_name(&pick_feat_for("weapon-focus", weapon("longsword"))),
        "weapon-focus (longsword)"
    );
    assert_eq!(
        engine.pick_name(&pick_feat_for("skill-focus", skill_value("mobility"))),
        "skill-focus (Mobility)"
    );
    assert_eq!(
        engine.pick_name(&pick_option("rogue/rogue-talents/fast-stealth")),
        "fast-stealth"
    );
    assert_eq!(
        engine.parameter_name(&Parameter::SpellSchool(rules::SpellSchool::Evocation)),
        "Evocation"
    );
}

#[test]
fn a_class_value_is_named_with_its_archetype_and_what_it_advances() {
    let engine = engine();
    let armiger = ArchetypeKey::new("fighter/armiger");
    assert_eq!(
        engine.class_value_name(&class("fighter"), None, None),
        "Fighter"
    );
    assert_eq!(
        engine.class_value_name(&class("fighter"), Some(&armiger), None),
        "Armiger (Fighter)"
    );
    let theurge = class("mystic-theurge");
    let both = BTreeSet::from([class("wizard"), class("cleric")]);
    assert_eq!(
        engine.class_value_name(&theurge, None, Some(&both)),
        "Mystic Theurge, advancing Cleric and Wizard"
    );
    assert_eq!(
        engine.class_value_name(&theurge, None, Some(&BTreeSet::new())),
        "Mystic Theurge, advancing nothing"
    );
    // The Build sheet writes the same ", advancing …" after a prestige class it advances something with.
    assert_eq!(engine.advancing(&both), ", advancing Cleric and Wizard");
}

#[test]
fn ability_alignment_and_category_names_are_the_games() {
    assert_eq!(Ability::Strength.name(), "Strength");
    assert_eq!(Ability::Charisma.short_name(), "Cha");
    assert_eq!(Alignment::TrueNeutral.name(), "True Neutral");
    assert_eq!(Alignment::LawfulGood.short_name(), "LG");
    assert_eq!(FeatCategory::ArmourAndShield.name(), "Armour and Shield");
    assert_eq!(FeatCategory::CombatManoeuvres.name(), "Combat Manoeuvres");
}

#[test]
fn numeric_conditions_say_what_the_character_has() {
    let engine = engine();
    let text = |prerequisite, has| reason(&engine, &Issue::Unmet(needs(prerequisite, has)));
    assert_eq!(
        text(Prerequisite::BaseAttackBonus(4), Some(1)),
        "Needs BAB +4 (have +1)"
    );
    assert_eq!(
        text(
            Prerequisite::AbilityScore {
                ability: Ability::Strength,
                score: 13
            },
            Some(10)
        ),
        "Needs Str 13 (have 10)"
    );
    assert_eq!(
        text(
            Prerequisite::ClassLevel {
                class: class("fighter"),
                level: 4
            },
            Some(1)
        ),
        "Needs Fighter level 4 (have 1)"
    );
    assert_eq!(
        text(
            Prerequisite::SkillRanks {
                skill: skill("mobility"),
                ranks: 5
            },
            Some(2)
        ),
        "Needs 5 ranks in Mobility (have 2)"
    );
    assert_eq!(
        text(
            Prerequisite::SpellLevel {
                spell_type: SpellType::Arcane,
                level: 3
            },
            Some(0)
        ),
        "Needs to cast 3rd-level arcane spells (can't cast any)"
    );
    assert_eq!(
        text(
            Prerequisite::SpontaneousSpellLevel {
                spell_type: SpellType::Divine,
                level: 1
            },
            Some(0)
        ),
        "Needs to cast 1st-level divine spells spontaneously (can't cast any)"
    );
    assert_eq!(
        text(Prerequisite::MythicRank(9), Some(3)),
        "Needs Mythic rank 9 (have 3)"
    );
    assert_eq!(
        text(Prerequisite::CharacterLevel(11), Some(9)),
        "Needs character level 11 (have 9)"
    );
    assert_eq!(
        text(Prerequisite::CasterLevel(5), None),
        "Needs caster level 5"
    );
}

#[test]
fn other_conditions_name_what_is_needed() {
    let engine = engine();
    let text = |prerequisite| reason(&engine, &Issue::Unmet(needs(prerequisite, None)));
    assert_eq!(
        text(Prerequisite::Feat(feat("power-attack"))),
        "Needs power-attack"
    );
    assert_eq!(
        text(Prerequisite::FeatFor {
            feat: feat("weapon-focus"),
            value: weapon("dueling-sword")
        }),
        "Needs weapon-focus (dueling-sword)"
    );
    assert_eq!(
        text(Prerequisite::FeatForSameValue(feat("weapon-focus"))),
        "Needs weapon-focus in the same weapon"
    );
    assert_eq!(
        text(Prerequisite::Feature {
            feature: feature("rogue/sneak-attack"),
            count: 1
        }),
        "Needs Sneak Attack"
    );
    assert_eq!(
        text(Prerequisite::Alignment(Alignment::LawfulGood)),
        "Needs Lawful Good alignment"
    );
    assert_eq!(text(Prerequisite::Race(RaceKey::new("elf"))), "Needs Elf");
    assert_eq!(
        text(Prerequisite::NoOtherOptionOfGroup(OptionGroup::Draconic)),
        "Needs no other Draconic option"
    );
    assert_eq!(
        text(Prerequisite::NoOptionFrom(FeatureSelectionKey::new(
            "rogue/rogue-talents"
        ))),
        "Needs no Rogue Talents yet"
    );
    assert_eq!(
        text(Prerequisite::Story(Story::VowOfDahak)),
        "Needs Vow of Dahak"
    );
}

#[test]
fn options_of_one_name_in_a_group_read_with_their_selections() {
    // Ticket 128: Mythical Beast's prerequisite read "Animal Companion or Animal Companion".
    let engine = engine();
    let fire = |key: &str| Prerequisite::SelectionOption(option(key));
    let named = |selection: &str| {
        format!(
            "fire ({})",
            engine.selection_name(&FeatureSelectionKey::new(selection))
        )
    };
    let domain = named("cleric/domain");
    let element = named("kineticist/element");
    let both = Prerequisite::Any(vec![
        fire("cleric/domain/fire"),
        fire("kineticist/element/fire"),
    ]);
    assert_eq!(
        engine.prerequisite_text(&both, None),
        format!("{domain} or {element}")
    );
    // The same in a reason.
    assert_eq!(
        reason(
            &engine,
            &Issue::Unmet(Unmet::Any(vec![
                needs(fire("cleric/domain/fire"), None),
                needs(fire("kineticist/element/fire"), None),
            ]))
        ),
        format!("Needs {domain} or {element}")
    );
    // An option alone, or with no other of its name, reads by its name alone.
    let water = Prerequisite::Any(vec![
        fire("cleric/domain/fire"),
        Prerequisite::SelectionOption(option("cleric/domain/water")),
    ]);
    assert_eq!(engine.prerequisite_text(&water, None), "fire or water");
}

#[test]
fn a_short_prerequisite_counts_a_long_groups_alternatives() {
    let engine = engine();
    let feats = Prerequisite::Any(
        ["power-attack", "dodge", "toughness", "weapon-focus"]
            .map(|key| Prerequisite::Feat(feat(key)))
            .to_vec(),
    );
    let all = Prerequisite::All(vec![feats.clone(), Prerequisite::BaseAttackBonus(4)]);
    // At most four alternatives, as the full text.
    assert_eq!(
        engine.short_prerequisite_text(&all, 4),
        engine.prerequisite_text(&all, None)
    );
    // More, as their count, their noun and the first three.
    assert_eq!(
        engine.short_prerequisite_text(&all, 3),
        "(one of 4 feats (power-attack, dodge, toughness, …)), BAB +4"
    );
    // Options of one name: that name, in the plural, each told apart by its selection.
    let fires = Prerequisite::Any(vec![
        Prerequisite::SelectionOption(option("cleric/domain/fire")),
        Prerequisite::SelectionOption(option("kineticist/element/fire")),
    ]);
    assert!(
        engine
            .short_prerequisite_text(&fires, 1)
            .starts_with("one of 2 fires (fire ("),
        "{}",
        engine.short_prerequisite_text(&fires, 1)
    );
    // An "at least" group too.
    let two = Prerequisite::AtLeast(
        2,
        ["power-attack", "dodge", "toughness", "weapon-focus"]
            .map(|key| Prerequisite::Feat(feat(key)))
            .to_vec(),
    );
    assert_eq!(
        engine.short_prerequisite_text(&two, 3),
        "2 of 4 feats (power-attack, dodge, toughness, …)"
    );
    // Mixed kinds are alternatives.
    let mixed = Prerequisite::Any(vec![
        Prerequisite::Feat(feat("dodge")),
        Prerequisite::BaseAttackBonus(4),
    ]);
    assert_eq!(
        engine.short_prerequisite_text(&mixed, 1),
        "one of 2 alternatives (dodge, BAB +4)"
    );
}

fn not(prerequisite: Prerequisite) -> Prerequisite {
    Prerequisite::Not(Box::new(prerequisite))
}

/// A negated condition that fails says what rules the option out, in words of its own rather than "Needs not …"
/// (ticket 65): "Not for" an archetype, race or alignment, "Not with" what the character holds, "Not as" the slot it
/// fills, "Only below" a level or score, "Not for a caster of" a spell level.
#[test]
fn a_negated_condition_says_what_rules_the_option_out() {
    let engine = engine();
    let text = |prerequisite| reason(&engine, &Issue::Unmet(needs(not(prerequisite), None)));
    let scoundrel = || Prerequisite::Archetype(ArchetypeKey::new("rogue/eldritch-scoundrel"));
    assert_eq!(
        text(scoundrel()),
        "Not for the Eldritch Scoundrel archetype"
    );
    assert_eq!(
        text(Prerequisite::Race(RaceKey::new("elf"))),
        "Not for the Elf race"
    );
    assert_eq!(
        text(Prerequisite::Alignment(Alignment::LawfulGood)),
        "Not for a Lawful Good character"
    );
    assert_eq!(
        text(Prerequisite::SpellLevel {
            spell_type: SpellType::Arcane,
            level: 3
        }),
        "Not for a caster of 3rd-level arcane spells"
    );
    assert_eq!(
        text(Prerequisite::SpontaneousSpellLevel {
            spell_type: SpellType::Divine,
            level: 1
        }),
        "Not for a spontaneous caster of 1st-level divine spells"
    );
    assert_eq!(
        text(Prerequisite::Feat(feat("power-attack"))),
        "Not with power-attack"
    );
    assert_eq!(
        text(Prerequisite::SelectionOption(option(
            "rogue/rogue-talents/fast-stealth"
        ))),
        "Not with fast-stealth"
    );
    assert_eq!(
        text(Prerequisite::Feature {
            feature: feature("rogue/sneak-attack"),
            count: 1
        }),
        "Not with Sneak Attack"
    );
    assert_eq!(
        text(Prerequisite::Story(Story::VowOfDahak)),
        "Not with Vow of Dahak"
    );
    assert_eq!(
        text(Prerequisite::NoOptionFrom(FeatureSelectionKey::new(
            "rogue/rogue-talents"
        ))),
        "Not with a Rogue Talents pick"
    );
    assert_eq!(
        text(Prerequisite::NoOtherOptionOfGroup(OptionGroup::Draconic)),
        "Not with another Draconic option"
    );
    assert_eq!(
        text(Prerequisite::PickedThrough(FeatureSelectionKey::new(
            "rogue/rogue-talents"
        ))),
        "Not as Rogue Talents"
    );
    assert_eq!(
        text(Prerequisite::ClassLevel {
            class: class("slayer"),
            level: 6
        }),
        "Only below Slayer level 6"
    );
    assert_eq!(text(Prerequisite::BaseAttackBonus(4)), "Only below BAB +4");
    assert_eq!(
        text(Prerequisite::AbilityScore {
            ability: Ability::Strength,
            score: 13
        }),
        "Only below Str 13"
    );
    assert_eq!(
        text(Prerequisite::MythicRank(3)),
        "Only below Mythic rank 3"
    );
    assert_eq!(
        text(Prerequisite::All(vec![
            Prerequisite::Feat(feat("dodge")),
            Prerequisite::Feat(feat("toughness"))
        ])),
        "Not with (dodge, toughness)"
    );
    // Two negations cancel.
    assert_eq!(
        text(not(Prerequisite::Feat(feat("power-attack")))),
        "Needs power-attack"
    );
}

/// Beside other unmet conditions, what rules an option out comes first, each kind once ("Not with A or B"), then what
/// it needs. An "or" with a negated branch reads as the case the others answer: "As …, needs …". Deeper inside a
/// group, a negated condition reads in line.
#[test]
fn negated_conditions_lead_a_reason_and_read_in_line_inside_a_group() {
    let engine = engine();
    let text = |unmet| reason(&engine, &Issue::Unmet(unmet));
    let scoundrel = || {
        not(Prerequisite::Archetype(ArchetypeKey::new(
            "rogue/eldritch-scoundrel",
        )))
    };
    let all = Unmet::All(vec![
        needs(scoundrel(), None),
        needs(
            Prerequisite::ClassLevel {
                class: class("fighter"),
                level: 4,
            },
            Some(1),
        ),
        needs(not(Prerequisite::Feat(feat("dodge"))), None),
        needs(Prerequisite::Feat(feat("power-attack")), None),
        needs(
            not(Prerequisite::SelectionOption(option(
                "rogue/rogue-talents/fast-stealth",
            ))),
            None,
        ),
        needs(
            not(Prerequisite::ClassLevel {
                class: class("slayer"),
                level: 6,
            }),
            None,
        ),
    ]);
    assert_eq!(
        text(all),
        "Not for the Eldritch Scoundrel archetype; not with dodge or fast-stealth; only below Slayer level 6; \
         needs Fighter level 4 (have 1), power-attack"
    );
    // Only negations.
    assert_eq!(
        text(Unmet::All(vec![
            needs(not(Prerequisite::Feat(feat("dodge"))), None),
            needs(not(Prerequisite::Feat(feat("toughness"))), None),
        ])),
        "Not with dodge or toughness"
    );
    // One group left beside the negation reads without brackets.
    assert_eq!(
        text(Unmet::All(vec![
            needs(scoundrel(), None),
            Unmet::Any(vec![
                needs(Prerequisite::Race(RaceKey::new("human")), None),
                needs(Prerequisite::Race(RaceKey::new("elf")), None),
            ]),
        ])),
        "Not for the Eldritch Scoundrel archetype; needs Human or Elf"
    );
    // An "or" whose negated branch fails: the others are what that case needs.
    let through = || {
        needs(
            not(Prerequisite::PickedThrough(FeatureSelectionKey::new(
                "rogue/rogue-talents",
            ))),
            None,
        )
    };
    assert_eq!(
        text(Unmet::Any(vec![
            through(),
            needs(Prerequisite::Alignment(Alignment::LawfulGood), None),
            needs(Prerequisite::Alignment(Alignment::NeutralGood), None),
        ])),
        "As Rogue Talents, needs Lawful Good alignment or Neutral Good alignment"
    );
    assert_eq!(
        text(Unmet::Any(vec![
            needs(scoundrel(), None),
            through(),
            needs(Prerequisite::Feat(feat("dodge")), None),
        ])),
        "For the Eldritch Scoundrel archetype and as Rogue Talents, needs dodge"
    );
    // An "or" of negations only fails when every one holds.
    assert_eq!(
        text(Unmet::Any(vec![
            needs(not(Prerequisite::Feat(feat("dodge"))), None),
            needs(not(Prerequisite::Feat(feat("toughness"))), None),
        ])),
        "Not with dodge and with toughness together"
    );
    // Inside a group, in line.
    assert_eq!(
        text(Unmet::All(vec![
            needs(Prerequisite::BaseAttackBonus(1), Some(0)),
            Unmet::Any(vec![
                needs(Prerequisite::Race(RaceKey::new("human")), None),
                needs(scoundrel(), None),
                needs(
                    not(Prerequisite::ClassLevel {
                        class: class("slayer"),
                        level: 6,
                    }),
                    None,
                ),
            ]),
        ])),
        "Needs BAB +1 (have +0), (Human or not for the Eldritch Scoundrel archetype or below Slayer level 6)"
    );
    // A negation's own text is the in-line one.
    assert_eq!(
        engine.prerequisite_text(&scoundrel(), None),
        "not for the Eldritch Scoundrel archetype"
    );
}

#[test]
fn groups_of_unmet_conditions_read_as_a_list() {
    let engine = engine();
    let all = Unmet::All(vec![
        needs(Prerequisite::Feat(feat("power-attack")), None),
        needs(Prerequisite::BaseAttackBonus(1), Some(0)),
    ]);
    assert_eq!(
        reason(&engine, &Issue::Unmet(all)),
        "Needs power-attack, BAB +1 (have +0)"
    );
    let any = Unmet::Any(vec![
        needs(Prerequisite::Race(RaceKey::new("human")), None),
        needs(Prerequisite::Race(RaceKey::new("elf")), None),
    ]);
    assert_eq!(
        reason(&engine, &Issue::Unmet(any.clone())),
        "Needs Human or Elf"
    );
    // An `Any` inside a longer list is bracketed so it reads unambiguously.
    let nested = Unmet::All(vec![needs(Prerequisite::BaseAttackBonus(1), Some(0)), any]);
    assert_eq!(
        reason(&engine, &Issue::Unmet(nested)),
        "Needs BAB +1 (have +0), (Human or Elf)"
    );
    let at_least = Unmet::AtLeast {
        needed: 2,
        met: 1,
        failed: vec![
            needs(Prerequisite::Feat(feat("dodge")), None),
            needs(Prerequisite::Feat(feat("toughness")), None),
        ],
    };
    assert_eq!(
        reason(&engine, &Issue::Unmet(at_least)),
        "Needs 1 more of: dodge, toughness"
    );
}

#[test]
fn the_other_issues_read_plainly() {
    let engine = engine();
    assert_eq!(reason(&engine, &Issue::AlreadyHeld), "Already taken");
    assert_eq!(
        reason(&engine, &Issue::MaxTimes { times: 3 }),
        "Already taken 3 times"
    );
    assert_eq!(
        reason(&engine, &Issue::ClassLevelCap { cap: 10 }),
        "All 10 levels taken"
    );
    assert_eq!(
        reason(
            &engine,
            &Issue::NotFromPath {
                from: BTreeSet::from([path("aeon"), path("trickster")])
            }
        ),
        "Only from Aeon or Trickster"
    );
    // An option that would break a later choice stays grey, with no reason.
    assert_eq!(engine.issue_text(&Issue::BreaksLater), None);
    assert_eq!(reason(&engine, &Issue::NotAllowed), "Not allowed here");
}

#[test]
fn a_picker_reason_comes_from_the_engine() {
    // Cleave at level 1 without Power Attack: the picker's reason reads as the game would say it.
    let engine = engine();
    let build = build("elf", &["fighter"]);
    let cleave = engine
        .options(&build, &general_slot(1))
        .unwrap()
        .into_iter()
        .find(|option| option.pick == pick_feat("cleave"))
        .unwrap();
    assert_eq!(
        reason(&engine, &cleave.status.unwrap_err()),
        "Needs power-attack"
    );
}

#[test]
fn slots_are_labelled_by_what_they_take() {
    let engine = engine();
    let sheet = engine.evaluate(&build("human", &["fighter", "rogue", "rogue"]));
    let labels: Vec<String> = sheet.levels[0]
        .slots
        .iter()
        .map(|slot| engine.slot_label(slot))
        .collect();
    // Class slots, then the race's, then the general slot.
    assert_eq!(labels, ["Combat feat", "Bonus feat", "Feat"]);
    // A rogue's talent at its 2nd class level, before the general slot.
    let labels: Vec<String> = sheet.levels[2]
        .slots
        .iter()
        .map(|slot| engine.slot_label(slot))
        .collect();
    assert_eq!(labels, ["Rogue Talents", "Feat"]);
}

#[test]
fn an_option_list_slot_is_named_after_its_selection_unless_it_has_a_label() {
    // A sorcerer's "Bloodline feat"; the kineticist's bonus-feat talent's "Wild talent feat" (ticket 47), though it
    // offers the Elemental Focus's lists.
    let engine = engine();
    let slot = |label: Option<&str>| Slot {
        id: SlotId::OptionFeat { level: 2, index: 0 },
        kind: SlotKind::Feat(FeatSlot::OptionList {
            selection: FeatureSelectionKey::new("kineticist/element"),
            at_level: None,
            label: label.map(str::to_owned),
        }),
        pick: None,
    };
    assert_eq!(engine.slot_label(&slot(None)), "kineticist/element feat");
    assert_eq!(
        engine.slot_label(&slot(Some("Wild talent feat"))),
        "Wild talent feat"
    );
}

#[test]
fn a_table_rows_feat_slot_is_labelled_as_its_slot_would_be() {
    // A class, archetype or path row's slot before any Build holds it (the MCP server's `describe`): a general one
    // there is a bonus feat, as on the sheet.
    let engine = engine();
    let label = |slot: FeatSlot| engine.feat_slot_label(&slot);
    assert_eq!(label(FeatSlot::General), "Bonus feat");
    assert_eq!(label(FeatSlot::CombatFeat), "Combat feat");
    assert_eq!(
        label(FeatSlot::Category(FeatCategory::Teamwork)),
        "Teamwork feat"
    );
    assert_eq!(
        label(FeatSlot::OneOf {
            list: feat_list("rogue/skill-focus")
        }),
        "rogue/skill-focus"
    );
    assert_eq!(
        label(FeatSlot::OptionList {
            selection: FeatureSelectionKey::new("kineticist/element"),
            at_level: None,
            label: Some("Wild talent feat".to_owned()),
        }),
        "Wild talent feat"
    );
}

/// The reason a locked option shows, for one that has one.
fn reason(engine: &Engine, issue: &Issue) -> String {
    engine.issue_text(issue).expect("a reason")
}

#[test]
fn an_and_of_two_texts_that_read_alike_reads_once_and_an_or_keeps_every_alternative() {
    // Ticket 192: the Dragon Disciple's "no Bloodline yet, no Bloodline yet" (the Sorcerer's and the Bloodrager's
    // selections are both named "Bloodline").
    let engine = engine();
    let none = || Prerequisite::NoOptionFrom(FeatureSelectionKey::new("rogue/rogue-talents"));
    let fire = Prerequisite::SelectionOption(option("cleric/domain/fire"));
    let either = Prerequisite::Any(vec![Prerequisite::All(vec![none(), none()]), fire.clone()]);
    let fire_name = engine.prerequisite_text(&fire, None);
    assert_eq!(
        engine.prerequisite_text(&either, None),
        format!("no Rogue Talents yet or {fire_name}")
    );
    // Shortened for `search`, with the same inner text.
    assert_eq!(
        engine.short_prerequisite_text(&either, 1),
        format!("one of 2 alternatives (no Rogue Talents yet, {fire_name})")
    );
    // An "and" of two different texts keeps its brackets; an "or" never merges alternatives.
    let mixed = Prerequisite::Any(vec![
        Prerequisite::All(vec![none(), fire.clone()]),
        fire.clone(),
    ]);
    assert_eq!(
        engine.prerequisite_text(&mixed, None),
        format!("(no Rogue Talents yet, {fire_name}) or {fire_name}")
    );
    let twice = Prerequisite::Any(vec![none(), none()]);
    assert_eq!(
        engine.prerequisite_text(&twice, None),
        "no Rogue Talents yet or no Rogue Talents yet"
    );
    // The same in a reason.
    let unmet = |prerequisite: Prerequisite| needs(prerequisite, None);
    assert_eq!(
        reason(
            &engine,
            &Issue::Unmet(Unmet::Any(vec![
                Unmet::All(vec![unmet(none()), unmet(none())]),
                unmet(fire.clone()),
            ]))
        ),
        format!("Needs no Rogue Talents yet or {fire_name}")
    );
}

#[test]
fn a_selection_named_x_labels_its_option_list_feat_slot_x_feat() {
    // Ticket 192: the ranger's selection was named "Combat Style Feat" and its feat slot read "Combat Style Feat
    // feat"; the data names it "Combat Style" so the slot reads "Combat Style feat".
    let engine = engine();
    let slot = Slot {
        id: SlotId::OptionFeat { level: 2, index: 0 },
        kind: SlotKind::Feat(FeatSlot::OptionList {
            selection: FeatureSelectionKey::new("rogue/rogue-talents"),
            at_level: None,
            label: None,
        }),
        pick: None,
    };
    assert_eq!(engine.slot_label(&slot), "Rogue Talents feat");
}
