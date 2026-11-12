//! Gear in the engine (ticket 166): a Build holds items, their always-on bonuses change the ability scores (and so
//! prerequisites and every derived number, but not a level's skill points) and the other numbers the engine
//! computes, bonuses stack by type, the numbers without gear are one call away, and a choice allowed only because
//! of gear says which items it needs.

use rules::{
    Ability, Address, BonusSource, Build, Choice, Edit, Engine, Failure, GearDependent, GearNeed,
    GearPlace, GearSpot, Issue, Prerequisite, SpellSchool, SpellTraits, Unmet,
};

use crate::support::*;

const GTWF: &str = "greater-two-weapon-fighting";
const SILVER_TONGUE: &str = "silver-tongue";

/// The test world with a feat that needs Dexterity 19, as Greater Two-Weapon Fighting does, and one that needs
/// Charisma 9.
fn engine_with_gtwf() -> Engine {
    let mut data = world();
    let mut gtwf = (data.feats.iter())
        .find(|row| row.key == feat("dodge"))
        .expect("the test world has Dodge")
        .clone();
    gtwf.key = feat(GTWF);
    gtwf.name = "Greater Two-Weapon Fighting".into();
    gtwf.prerequisite = Some(Prerequisite::AbilityScore {
        ability: Ability::Dexterity,
        score: 19,
    });
    let mut silver = gtwf.clone();
    silver.key = feat(SILVER_TONGUE);
    silver.name = "Silver Tongue".into();
    silver.prerequisite = Some(Prerequisite::AbilityScore {
        ability: Ability::Charisma,
        score: 9,
    });
    data.feats.push(gtwf);
    data.feats.push(silver);
    engine_on(data)
}

/// A human of one class with these point-buy scores (Str, Dex, Con, Int, Wis, Cha) and no racial bonus placed.
fn character(scores: [u8; 6], classes: &[&str]) -> Build {
    let mut build = build("human", classes);
    build.start.ability_scores = point_buy(scores);
    build
}

/// A human fighter 1 with Dexterity 17 and Greater Two-Weapon Fighting in the general slot.
fn two_weapon_fighter() -> Build {
    let mut build = character([10, 17, 10, 10, 10, 10], &["fighter"]);
    build.picks.insert(general_slot(1), pick_feat(GTWF));
    build
}

fn belt(build: Build) -> Build {
    wearing(build, GearPlace::Belt, "belt-of-dexterity-2")
}

/// The scores at the end of the Build, in Str, Dex, Con, Int, Wis, Cha order.
fn final_scores(engine: &Engine, build: &Build) -> [u8; 6] {
    let sheet = engine.evaluate(build);
    let scores = sheet.levels.last().expect("a level").state.ability_scores;
    rules::AbilityScores::ABILITIES.map(|ability| scores.get(ability))
}

fn keys(items: &[&str]) -> Vec<rules::GearKey> {
    items.iter().map(|item| gear_item(item)).collect()
}

/// A need of each of these items.
fn all_of(items: &[&str]) -> GearNeed {
    GearNeed {
        all_of: keys(items),
        ..GearNeed::default()
    }
}

fn needs_dex_19() -> Issue {
    Issue::Unmet(Unmet::Condition {
        needs: Prerequisite::AbilityScore {
            ability: Ability::Dexterity,
            score: 19,
        },
        has: Some(17),
    })
}

fn parts(total: &rules::Total) -> Vec<(&str, i16)> {
    (total.parts.iter())
        .map(|part| (part.name.as_str(), part.value))
        .collect()
}

// ---- prerequisites ----------------------------------------------------------------------------------------------

#[test]
fn a_feat_needing_dex_19_is_refused_at_17_allowed_with_a_belt_marked_and_refused_again_without_it()
{
    let engine = engine_with_gtwf();
    let slot = general_slot(1);
    let naked = two_weapon_fighter();

    // Refused at Dexterity 17.
    let refused = vec![Failure {
        choice: Choice::Slot(slot.clone()),
        issue: needs_dex_19(),
    }];
    assert_eq!(engine.failures(&naked), refused);
    assert_eq!(
        engine.issue_text(&needs_dex_19()).as_deref(),
        Some("Needs Dex 19 (have 17)")
    );

    // Allowed with a Dexterity +2 belt, and marked with it.
    let geared = belt(naked.clone());
    assert_eq!(engine.failures(&geared), []);
    assert!(engine.is_possible(&geared));
    let needs = all_of(&["belt-of-dexterity-2"]);
    assert_eq!(
        engine.gear_dependents(&geared),
        [GearDependent {
            choice: Choice::Slot(slot.clone()),
            needs: needs.clone(),
        }]
    );
    assert_eq!(engine.gear_note(&needs), "With Belt of Dexterity +2");

    // The belt can't come off while the feat needs it; without the belt the feat is refused with the usual reason.
    let the_belt = Choice::Gear(GearSpot::Worn(GearPlace::Belt));
    let (edit, removable) = engine
        .withdrawal(&geared, &the_belt)
        .expect("the Build holds the belt");
    assert!(!removable, "the feat needs the belt");
    assert!(!engine.allows(&geared, &edit));
    let mut withdrawn = geared.clone();
    engine.apply(&mut withdrawn, &edit);
    assert_eq!(withdrawn, naked);
    assert_eq!(engine.failures(&withdrawn), refused);
    assert_eq!(engine.gear_dependents(&withdrawn), []);
    // The feat cleared first, the belt comes off.
    assert!(engine.is_removable(&without(&geared, &slot), &the_belt));
}

#[test]
fn a_pickers_option_open_only_because_of_gear_carries_the_items_note() {
    let engine = engine_with_gtwf();
    let slot = general_slot(1);
    let empty = without(&two_weapon_fighter(), &slot);
    let option = |build: &Build, feat: &str| {
        (engine.options(build, &slot).expect("the slot").into_iter())
            .find(|option| option.pick == pick_feat(feat))
            .expect("the feat is offered")
    };
    let entry = |build: &Build, name: &str| {
        (engine.slot_entries(build, &slot).expect("the slot"))
            .into_iter()
            .find(|entry| entry.name == name)
            .expect("the entry")
    };

    // Without gear the feat is locked, and nothing is marked.
    let locked = option(&empty, GTWF);
    assert_eq!(locked.status, Err(needs_dex_19()));
    assert_eq!(locked.with_gear, None);

    // With the belt it is open, and says so; a feat open anyway says nothing.
    let geared = belt(empty);
    let open = option(&geared, GTWF);
    assert_eq!(open.status, Ok(()));
    assert_eq!(open.with_gear, Some(all_of(&["belt-of-dexterity-2"])));
    assert_eq!(option(&geared, "dodge").with_gear, None);
    let gtwf = entry(&geared, "Greater Two-Weapon Fighting");
    assert!(gtwf.open);
    assert_eq!(gtwf.gear_note.as_deref(), Some("With Belt of Dexterity +2"));
    assert_eq!(entry(&geared, "dodge").gear_note, None);
}

#[test]
fn a_choice_that_either_of_two_items_allows_names_both_as_alternatives() {
    let engine = engine();
    // Dodge needs Dexterity 13; the belt and the manual each take 11 there.
    let mut build = character([10, 11, 10, 10, 10, 10], &["fighter"]);
    build.picks.insert(general_slot(1), pick_feat("dodge"));
    assert!(!engine.is_possible(&build));
    let build = using(belt(build), "manual-of-dexterity-2");
    assert!(engine.is_possible(&build));

    let dependents = engine.gear_dependents(&build);

    let needs = GearNeed {
        one_of: keys(&["belt-of-dexterity-2", "manual-of-dexterity-2"]),
        ..GearNeed::default()
    };
    assert_eq!(
        dependents,
        [GearDependent {
            choice: Choice::Slot(general_slot(1)),
            needs: needs.clone(),
        }]
    );
    assert_eq!(
        engine.gear_note(&needs),
        "With Belt of Dexterity +2 or Manual of Dexterity +2"
    );
}

/// A fighter with this Dexterity and the feat needing 19.
fn needing_dex_19(dexterity: u8) -> Build {
    let mut build = character([10, dexterity, 10, 10, 10, 10], &["fighter"]);
    build.picks.insert(general_slot(1), pick_feat(GTWF));
    build
}

/// The one gear-dependent choice's need, and its note.
fn need_and_note(engine: &Engine, build: &Build) -> (GearNeed, String) {
    assert!(engine.is_possible(build));
    let mut dependents = engine.gear_dependents(build);
    assert_eq!(dependents.len(), 1, "{dependents:?}");
    let needs = dependents.remove(0).needs;
    // The slot's picker says the same of the option.
    let mut empty = build.clone();
    let pick = empty.picks.remove(&general_slot(1)).expect("the pick");
    let option = (engine
        .options(&empty, &general_slot(1))
        .unwrap()
        .into_iter())
    .find(|option| option.pick == pick)
    .expect("offered");
    assert_eq!(option.with_gear.as_ref(), Some(&needs));
    let note = engine.gear_note(&needs);
    (needs, note)
}

#[test]
fn a_choice_that_needs_both_of_two_items_names_both() {
    let engine = engine_with_gtwf();
    // 15 + 2 enhancement + 2 inherent; the cloak has nothing to do with it.
    let mut build = using(belt(needing_dex_19(15)), "manual-of-dexterity-2");
    build = wearing(build, GearPlace::Cloak, "cloak-of-resistance-3");

    let (needs, note) = need_and_note(&engine, &build);

    assert_eq!(
        needs,
        all_of(&["belt-of-dexterity-2", "manual-of-dexterity-2"])
    );
    assert_eq!(note, "With Belt of Dexterity +2 and Manual of Dexterity +2");
}

#[test]
fn a_choice_that_needs_any_two_of_three_items_names_one_set_that_is_enough_and_no_other_item() {
    let engine = engine_with_gtwf();
    // 15 and any two of three +2s of different types (the bracers' is untyped); a cloak beside them.
    let mut build = using(belt(needing_dex_19(15)), "manual-of-dexterity-2");
    build = wearing(build, GearPlace::Bracers, "bracers-of-agility");
    build = wearing(build, GearPlace::Cloak, "cloak-of-resistance-3");

    let (needs, note) = need_and_note(&engine, &build);

    assert_eq!(
        needs,
        GearNeed {
            for_example: keys(&["bracers-of-agility", "manual-of-dexterity-2"]),
            ..GearNeed::default()
        }
    );
    assert_eq!(
        note,
        "With gear such as Bracers of Agility and Manual of Dexterity +2"
    );
}

#[test]
fn a_choice_that_needs_one_item_and_either_of_two_others_says_so() {
    let engine = engine_with_gtwf();
    // 13 + the gloves' 4, then 2 more from the bracers or the manual; a cloak beside them.
    let mut build = wearing(
        needing_dex_19(13),
        GearPlace::Gloves,
        "gloves-of-dexterity-4",
    );
    build = wearing(build, GearPlace::Bracers, "bracers-of-agility");
    build = wearing(build, GearPlace::Cloak, "cloak-of-resistance-3");
    build = using(build, "manual-of-dexterity-2");

    let (needs, note) = need_and_note(&engine, &build);

    assert_eq!(
        needs,
        GearNeed {
            all_of: keys(&["gloves-of-dexterity-4"]),
            one_of: keys(&["bracers-of-agility", "manual-of-dexterity-2"]),
            ..GearNeed::default()
        }
    );
    assert_eq!(
        note,
        "With Gloves of Dexterity +4 and one of Bracers of Agility or Manual of Dexterity +2"
    );
    // Three further items with none enough alone and one needed for sure.
    let example = GearNeed {
        all_of: keys(&["gloves-of-dexterity-4"]),
        for_example: keys(&["bracers-of-agility", "manual-of-dexterity-2"]),
        ..GearNeed::default()
    };
    assert_eq!(
        engine.gear_note(&example),
        "With Gloves of Dexterity +4 and more gear, such as Bracers of Agility and Manual of Dexterity +2"
    );
}

#[test]
fn a_choice_needing_a_gear_dependent_one_depends_on_the_same_item() {
    let engine = engine();
    // Power Attack needs Strength 13 (the tome's +4 takes 10 there); Cleave needs Power Attack.
    let mut build = character([10, 10, 10, 10, 10, 10], &["fighter"]);
    build.picks.insert(race_slot(0), pick_feat("power-attack"));
    build.picks.insert(general_slot(1), pick_feat("cleave"));
    let build = using(build, "tome-of-perfection-4");
    assert!(engine.is_possible(&build));

    let dependents = engine.gear_dependents(&build);

    let choices: Vec<&Choice> = dependents.iter().map(|d| &d.choice).collect();
    assert_eq!(
        choices,
        [&Choice::Slot(race_slot(0)), &Choice::Slot(general_slot(1))]
    );
    for dependent in &dependents {
        assert_eq!(dependent.needs, all_of(&["tome-of-perfection-4"]));
    }
}

// ---- stacking ---------------------------------------------------------------------------------------------------

#[test]
fn two_enhancement_bonuses_to_one_score_give_the_higher_and_an_enhancement_and_an_inherent_add() {
    let engine = engine();
    let naked = build("human", &["fighter"]);
    assert_eq!(final_scores(&engine, &naked), [16, 14, 12, 13, 10, 8]);

    // Enhancement +2 (belt) and +4 (gloves) to Dexterity: +4.
    let both = wearing(
        belt(naked.clone()),
        GearPlace::Gloves,
        "gloves-of-dexterity-4",
    );
    assert_eq!(final_scores(&engine, &both)[1], 18);

    // Enhancement +2 (belt) and inherent +2 (manual): +4.
    let mixed = using(belt(naked.clone()), "manual-of-dexterity-2");
    assert_eq!(final_scores(&engine, &mixed)[1], 18);

    // Enhancement +4 and inherent +2: +6.
    let most = using(both, "manual-of-dexterity-2");
    assert_eq!(final_scores(&engine, &most)[1], 20);
}

#[test]
fn two_inherent_bonuses_to_one_score_give_the_higher() {
    let engine = engine();
    let build = using(
        using(build("human", &["fighter"]), "manual-of-dexterity-2"),
        "tome-of-perfection-4",
    );
    // The tome's +4 to every score; the manual's +2 Dexterity is hidden by it.
    assert_eq!(final_scores(&engine, &build), [20, 18, 16, 17, 14, 12]);
}

#[test]
fn an_untyped_bonus_always_adds_and_a_penalty_always_applies() {
    let engine = engine();
    let naked = build("human", &["fighter"]);
    // Constitution: +2 enhancement (belt) and +2 untyped (the plate).
    let sturdy = wearing(
        wearing(naked.clone(), GearPlace::Belt, "belt-of-constitution-2"),
        GearPlace::Body,
        "plate-of-vigor",
    );
    assert_eq!(final_scores(&engine, &sturdy)[2], 16);
    // Charisma: the headband's −2 beside the robe's +2 enhancement; Intelligence +2.
    let scholar = wearing(naked.clone(), GearPlace::Headwear, "scholars-headband");
    assert_eq!(final_scores(&engine, &scholar), [16, 14, 12, 15, 10, 6]);
    let robed = wearing(scholar, GearPlace::Body, "robe-of-the-mage");
    assert_eq!(final_scores(&engine, &robed)[5], 8);
}

#[test]
fn gears_bonuses_to_a_save_initiative_and_a_skill_stack_by_type_among_themselves() {
    let engine = engine();
    let naked = build("human", &["fighter"]);
    let mut build = wearing(naked.clone(), GearPlace::Cloak, "cloak-of-resistance-3");
    build = wearing(build, GearPlace::FirstRing, "ring-of-fortitude");
    build = wearing(build, GearPlace::SecondRing, "ring-of-luck");
    build = wearing(build, GearPlace::Headwear, "scholars-headband");
    let totals = engine.totals(&last_level(&engine, &build));
    let before = engine.totals(&last_level(&engine, &naked));

    // Fortitude: the ring's +5 resistance hides the cloak's +3; Reflex and Will get the cloak's.
    assert_eq!(
        parts(&totals.fortitude),
        [
            ("Base save", 2),
            ("Constitution", 1),
            ("Ring of Fortitude", 5)
        ]
    );
    assert_eq!(totals.reflex.total, before.reflex.total + 3);
    assert_eq!(totals.will.total, before.will.total + 3);
    // Initiative: two luck bonuses, the higher.
    assert_eq!(
        parts(&totals.initiative),
        [("Dexterity", 2), ("Ring of Fortitude", 2)]
    );
    // A competence bonus to one skill, and a penalty to another (with the headband's Charisma −2).
    let mobility = totals.skill(&skill("mobility")).unwrap();
    assert_eq!(parts(&mobility.total).last(), Some(&("Ring of Luck", 4)));
    let persuasion = totals.skill(&skill("persuasion")).unwrap();
    assert_eq!(
        parts(&persuasion.total),
        [("Ranks", 0), ("Charisma", -2), ("Scholar's Headband", -3)]
    );
}

#[test]
fn gears_bonuses_in_force_follow_the_item_bonuses_at_every_level_with_their_type_and_item() {
    let engine = engine();
    let cloaked = wearing(
        build("human", &times("fighter", 3)),
        GearPlace::Cloak,
        "cloak-of-resistance-3",
    );
    let sheet = walked(&engine, &cloaked);
    for level in &sheet.levels {
        let last = level.state.bonuses_in_force.last().expect("the cloak's");
        assert_eq!(
            last.source,
            BonusSource::Gear(gear_item("cloak-of-resistance-3"))
        );
        assert_eq!(last.kind, rules::BonusType::Resistance);
        assert_eq!(
            (last.name.as_str(), last.amount),
            ("Cloak of Resistance +3", 3)
        );
    }
    // An ability score bonus goes through the scores, never the list.
    let belted = belt(build("human", &["fighter"]));
    assert!(
        walked(&engine, &belted).levels[0]
            .state
            .bonuses_in_force
            .is_empty()
    );
}

// ---- the order: gear before the features ------------------------------------------------------------------------

#[test]
fn gloves_that_make_another_score_the_highest_move_a_highest_score_features_bonus() {
    let engine = engine();
    // A fighter 17 switched to Gold Dragon: every score at least 18, and the highest gets +6.
    let mut dragon = build("human", &times("fighter", 17));
    dragon.mythic_path = Some(path("trickster"));
    dragon.path_switch = Some(path("gold-dragon"));
    // Strength 16 is the highest: 24, and Dexterity 18.
    assert_eq!(final_scores(&engine, &dragon)[..2], [24, 18]);

    // Dexterity 14 + 4 is now the highest: it gets the +6, and Strength only the floor.
    let gloved = wearing(dragon, GearPlace::Gloves, "gloves-of-dexterity-4");
    assert_eq!(final_scores(&engine, &gloved)[..2], [18, 24]);
}

// ---- skill points -----------------------------------------------------------------------------------------------

#[test]
fn an_intelligence_item_changes_the_score_and_no_levels_skill_points() {
    let engine = engine();
    let naked = build("human", &times("wizard", 8));
    let geared = wearing(naked.clone(), GearPlace::Weapon, "mage-staff");
    let (before, after) = (engine.evaluate(&naked), engine.evaluate(&geared));

    for (before, after) in before.levels.iter().zip(&after.levels) {
        assert_eq!(before.state.ability_scores.intelligence, 13);
        assert_eq!(after.state.ability_scores.intelligence, 19);
        assert_eq!(before.skill_points, after.skill_points);
    }
    // The staff buys no rank: one point more than the level has is refused, staff or not.
    let available = after.levels[0].skill_points.available;
    let mut overspent = geared.clone();
    for level in overspent.levels.iter_mut().take(usize::from(available) + 1) {
        level.skill_ranks.insert(skill("athletics"), 1);
    }
    overspent.levels[0]
        .skill_ranks
        .insert(skill("mobility"), available);
    assert!(!engine.is_possible(&overspent));
}

// ---- the caster line --------------------------------------------------------------------------------------------

#[test]
fn a_caster_level_item_raises_the_lines_caster_level_spell_penetration_and_concentration() {
    let engine = engine();
    let naked = build("human", &times("sorcerer", 5));
    let spell = SpellTraits {
        school: SpellSchool::Evocation,
        subschool: None,
        energy: None,
        metamagic: false,
    };
    let line = |build: &Build| {
        let level = last_level(&engine, build);
        (
            engine.caster_line(&level, spell).expect("a caster"),
            level.state.caster_level,
        )
    };
    let (before, _) = line(&naked);
    assert_eq!(before.caster_level, 5);

    // The staff: caster level +3, spell penetration and concentration +2 competence each.
    let (staff, own) = line(&wearing(naked.clone(), GearPlace::Weapon, "mage-staff"));
    assert_eq!(staff.caster_level, 8);
    assert_eq!(own, 5, "prerequisites read the class's caster level");
    assert_eq!(staff.spell_penetration, before.spell_penetration + 3 + 2);
    assert_eq!(staff.concentration, before.concentration + 3 + 2);
    assert_eq!(staff.dcs, before.dcs);

    // The robe: Charisma +2 (one more to every DC) and +1 to every spell's DC.
    let (robe, _) = line(&wearing(naked, GearPlace::Body, "robe-of-the-mage"));
    assert_eq!(robe.dcs[0].dc, before.dcs[0].dc + 2);
    assert_eq!(
        robe.dc_parts
            .last()
            .map(|part| (part.name.as_str(), part.value)),
        Some(("Robe of the Mage", 1))
    );
    assert_eq!(robe.concentration, before.concentration + 1);
}

// ---- both numbers -----------------------------------------------------------------------------------------------

#[test]
fn the_naked_numbers_of_a_build_with_gear_equal_the_numbers_of_the_same_build_without_it() {
    let engine = engine();
    let without = build("human", &times("sorcerer", 9));
    let mut with = wearing(without.clone(), GearPlace::Weapon, "mage-staff");
    with = wearing(with, GearPlace::Cloak, "cloak-of-resistance-3");
    with = using(belt(with), "tome-of-perfection-4");

    let naked = engine.naked_sheet(&with).expect("the Build has gear");

    let own = engine.evaluate(&without);
    assert_eq!(naked, own);
    assert_ne!(engine.evaluate(&with), own);
    let (naked, own) = (naked.levels.last().unwrap(), own.levels.last().unwrap());
    assert_eq!(engine.totals(naked), engine.totals(own));
    let spell = SpellTraits {
        school: SpellSchool::Evocation,
        subschool: None,
        energy: None,
        metamagic: false,
    };
    assert_eq!(
        engine.caster_line(naked, spell),
        engine.caster_line(own, spell)
    );
    // A Build without gear is its own naked Build: nothing more to walk.
    assert_eq!(engine.naked_sheet(&without), None);
    assert_eq!(engine.gear_dependents(&without), []);
}

// ---- places, options and edits ----------------------------------------------------------------------------------

#[test]
fn a_place_offers_none_and_every_item_of_its_slots() {
    let engine = engine();
    let build = build("human", &["fighter"]);
    let keys = |place| -> Vec<String> {
        (engine.gear_options(&build, place).into_iter())
            .map(|option| {
                option
                    .item
                    .map_or("none".to_owned(), |item| item.as_str().to_owned())
            })
            .collect()
    };
    assert_eq!(
        keys(GearPlace::Belt),
        ["none", "belt-of-constitution-2", "belt-of-dexterity-2"]
    );
    // A robe and an armour share the body; both ring places take a ring.
    assert_eq!(
        keys(GearPlace::Body),
        ["none", "plate-of-vigor", "robe-of-the-mage"]
    );
    assert_eq!(
        keys(GearPlace::FirstRing),
        ["none", "ring-of-fortitude", "ring-of-luck"]
    );
    assert_eq!(keys(GearPlace::SecondRing), keys(GearPlace::FirstRing));
    // Nothing needs anything: every option is open, and "None" is what the empty place holds.
    for place in GearPlace::ALL {
        let options = engine.gear_options(&build, place);
        assert!(options.iter().all(|option| option.status.is_ok()));
        assert!(options.iter().all(|option| option.reason.is_none()));
        let held: Vec<bool> = options.iter().map(|option| option.held).collect();
        assert_eq!(held.iter().filter(|held| **held).count(), 1);
        assert!(held[0], "{place:?}");
    }
}

#[test]
fn an_item_a_pick_needs_cant_be_removed_or_replaced_until_the_pick_is_cleared() {
    let engine = engine_with_gtwf();
    let geared = belt(two_weapon_fighter());
    let slot = Choice::Slot(general_slot(1));
    let reason = "Greater Two-Weapon Fighting needs Belt of Dexterity +2";

    let options = engine.gear_options(&geared, GearPlace::Belt);

    let shown: Vec<(Option<&str>, bool, bool, Option<&str>)> = (options.iter())
        .map(|option| {
            (
                option.item.as_ref().map(|item| item.as_str()),
                option.held,
                option.status.is_ok(),
                option.reason.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            // "None" and the other belt are locked, naming the feat; the belt worn stays open.
            (None, false, false, Some(reason)),
            (Some("belt-of-constitution-2"), false, false, Some(reason)),
            (Some("belt-of-dexterity-2"), true, true, None),
        ]
    );
    for option in &options[..2] {
        assert_eq!(option.status, Err(Issue::BreaksLater));
        assert_eq!(option.needed_by, std::slice::from_ref(&slot));
        // The edit is refused as any edit that breaks a choice is.
        assert!(!engine.allows(&geared, &option.edit));
    }
    // An item elsewhere that keeps the feat legal is open.
    let gloves = engine.gear_options(&geared, GearPlace::Gloves);
    assert!(gloves.iter().all(|option| option.status.is_ok()));
    // With the gloves' +4 too, the belt is no longer needed: it comes off or is replaced.
    let both = wearing(geared.clone(), GearPlace::Gloves, "gloves-of-dexterity-4");
    let options = engine.gear_options(&both, GearPlace::Belt);
    assert!(options.iter().all(|option| option.status.is_ok()));
    // And with the feat cleared, everything is open again.
    let cleared = without(&geared, &general_slot(1));
    let options = engine.gear_options(&cleared, GearPlace::Belt);
    assert!(options.iter().all(|option| option.status.is_ok()));
    let mut bare = cleared.clone();
    engine.apply(&mut bare, &options[0].edit);
    assert!(bare.gear.is_empty());
}

#[test]
fn an_item_whose_own_penalty_would_refuse_a_pick_is_locked_and_says_so() {
    let engine = engine_with_gtwf();
    // Silver Tongue needs Charisma 9; the Scholar's Headband takes 10 to 8.
    let mut build = character([10, 10, 10, 10, 10, 10], &["fighter"]);
    build
        .picks
        .insert(general_slot(1), pick_feat(SILVER_TONGUE));
    assert!(engine.is_possible(&build));

    let options = engine.gear_options(&build, GearPlace::Headwear);

    let headband = (options.iter())
        .find(|option| option.item == Some(gear_item("scholars-headband")))
        .expect("the headband");
    assert_eq!(headband.status, Err(Issue::BreaksLater));
    assert_eq!(
        headband.reason.as_deref(),
        Some("Scholar's Headband would refuse Silver Tongue")
    );
    assert!(options[0].held && options[0].status.is_ok());
}

#[test]
fn usable_items_share_no_place_each_is_a_tick_and_a_needed_one_stays_ticked() {
    let engine = engine();
    let mut build = build("human", &["fighter"]);
    let ticks = |build: &Build| -> Vec<(String, bool)> {
        (engine.usable_options(build).into_iter())
            .map(|option| {
                (
                    option.item.expect("an item").as_str().to_owned(),
                    option.held,
                )
            })
            .collect()
    };
    assert_eq!(
        ticks(&build),
        [
            ("manual-of-dexterity-2".to_owned(), false),
            ("tome-of-perfection-4".to_owned(), false)
        ]
    );
    // Each tick uses its item; both are held at once.
    for option in engine.usable_options(&build) {
        assert_eq!(option.status, Ok(()));
        engine.apply(&mut build, &option.edit);
    }
    assert_eq!(build.gear.used.len(), 2);
    assert!(engine.is_possible(&build));
    assert!(ticks(&build).iter().all(|(_, held)| *held));
    // A held tick's edit stops using it.
    let manual = &engine.usable_options(&build)[0];
    assert_eq!(manual.status, Ok(()));
    let mut fewer = build.clone();
    engine.apply(&mut fewer, &manual.edit);
    assert_eq!(
        fewer.gear.used.iter().collect::<Vec<_>>(),
        [&gear_item("tome-of-perfection-4")]
    );
    let withdrawal = Choice::Gear(GearSpot::Used(gear_item("manual-of-dexterity-2")));
    assert_eq!(
        engine.withdrawal(&build, &withdrawal),
        Some((manual.edit.clone(), true))
    );

    // Dodge needs Dexterity 13: at 11, only the manual's +2 gives it, so its tick is locked.
    let mut dodger = character([10, 11, 10, 10, 10, 10], &["fighter"]);
    dodger.picks.insert(general_slot(1), pick_feat("dodge"));
    let dodger = using(dodger, "manual-of-dexterity-2");
    let manual = &engine.usable_options(&dodger)[0];
    assert!(manual.held);
    assert_eq!(manual.status, Err(Issue::BreaksLater));
    assert_eq!(
        manual.reason.as_deref(),
        Some("dodge needs Manual of Dexterity +2")
    );
    assert!(!engine.allows(&dodger, &manual.edit));
}

#[test]
fn a_ring_worn_in_the_other_ring_place_moves() {
    let engine = engine();
    let mut build = wearing(
        build("human", &["fighter"]),
        GearPlace::FirstRing,
        "ring-of-luck",
    );
    let edit = Edit::Wear {
        place: GearPlace::SecondRing,
        item: Some(gear_item("ring-of-luck")),
    };
    engine.apply(&mut build, &edit);
    assert_eq!(
        build.gear.worn.iter().collect::<Vec<_>>(),
        [(&GearPlace::SecondRing, &gear_item("ring-of-luck"))]
    );
}

#[test]
fn an_item_the_data_lacks_in_the_wrong_place_or_worn_twice_fails_and_counts_for_nothing() {
    let engine = engine();
    let naked = build("human", &["fighter"]);
    let failure = |build: &Build| -> Vec<(Address, Issue)> {
        (engine.failures(build).into_iter())
            .map(|failure| (Address::from(&failure.choice), failure.issue))
            .collect()
    };
    let at = |text: &str| text.parse::<Address>().unwrap();

    let unknown = wearing(naked.clone(), GearPlace::Belt, "no-such-belt");
    assert_eq!(failure(&unknown), [(at("gear.belt"), Issue::UnknownKey)]);
    // A belt on the hands; a belt used; a manual worn.
    let misplaced = wearing(naked.clone(), GearPlace::Gloves, "belt-of-dexterity-2");
    assert_eq!(
        failure(&misplaced),
        [(at("gear.gloves"), Issue::NotAllowed)]
    );
    assert_eq!(final_scores(&engine, &misplaced)[1], 14);
    let used = using(naked.clone(), "belt-of-dexterity-2");
    assert_eq!(
        failure(&used),
        [(at("gear.usable.belt-of-dexterity-2"), Issue::NotAllowed)]
    );
    let worn = wearing(naked.clone(), GearPlace::Belt, "manual-of-dexterity-2");
    assert_eq!(failure(&worn), [(at("gear.belt"), Issue::NotAllowed)]);
    // One ring on both hands: the second fails, and its bonuses count once.
    let twice = wearing(
        wearing(naked, GearPlace::FirstRing, "ring-of-luck"),
        GearPlace::SecondRing,
        "ring-of-luck",
    );
    assert_eq!(failure(&twice), [(at("gear.ring.1"), Issue::AlreadyHeld)]);
    assert!(engine.open_link(&rules::encode_build(&twice)).is_err());
}

#[test]
fn gear_has_a_place_of_its_own_and_its_labels() {
    let engine = engine();
    let belt: Address = "gear.belt".parse().unwrap();
    assert_eq!(belt.place(), Some(rules::Place::Gear));
    assert_eq!(engine.placed_label(&belt, None), "Gear: Belt");
    let ring: Address = "gear.ring.1".parse().unwrap();
    assert_eq!(engine.placed_label(&ring, None), "Gear: Ring 2");
    let manual: Address = "gear.usable.manual-of-dexterity-2".parse().unwrap();
    assert_eq!(
        engine.placed_label(&manual, None),
        "Gear: Usable: Manual of Dexterity +2"
    );
    assert_eq!(
        engine.gear_summary(&gear_item("belt-of-dexterity-2")),
        "Dex +2 enhancement"
    );
    for text in [
        "gear",
        "gear.ring",
        "gear.ring.2",
        "gear.usable",
        "gear.robe",
    ] {
        assert!(text.parse::<Address>().is_err(), "{text}");
    }
}

// ---- the point buy and the withdrawals --------------------------------------------------------------------------

#[test]
fn a_point_buy_step_that_only_gear_keeps_legal_is_open_and_says_it_leans_on_the_gear() {
    let engine = engine_with_gtwf();
    // Dexterity 18 and the feat needing 19, under gloves of +4: 22.
    let mut build = character([10, 18, 10, 10, 10, 10], &["fighter"]);
    build.start.racial_ability = Some(Ability::Dexterity);
    build.picks.insert(general_slot(1), pick_feat(GTWF));
    assert!(engine.is_possible(&build), "20 without gear");
    let gloved = wearing(build.clone(), GearPlace::Gloves, "gloves-of-dexterity-4");
    let down = |build: &Build| {
        (engine.point_buy_steps(build).into_iter())
            .find(|stepper| stepper.value == Ability::Dexterity)
            .and_then(|stepper| stepper.down)
            .expect("a step down")
    };

    // 18 to 17 keeps 19 without gear: safe either way.
    let safe = down(&gloved);
    assert_eq!((safe.status.clone(), safe.leans_on_gear.len()), (Ok(()), 0));
    // 17 to 16 makes 18 without the gloves: refused without them, open with them but not safe.
    let mut lower = build.clone();
    engine.apply(&mut lower, &safe.edit);
    assert_eq!(down(&lower).status, Err(Issue::BreaksLater));
    let leaning = down(&wearing(lower, GearPlace::Gloves, "gloves-of-dexterity-4"));
    assert_eq!(leaning.status, Ok(()));
    assert_eq!(leaning.leans_on_gear, [Choice::Slot(general_slot(1))]);
    assert_eq!(
        engine.newly_gear_dependent(&gloved, &leaning.edit),
        [Choice::Slot(general_slot(1))]
    );
}

#[test]
fn the_withdrawals_see_the_gear_and_an_item_something_needs_is_not_removable() {
    let engine = engine_with_gtwf();
    // Dexterity 17 + 2 is exactly the 19 the feat needs.
    let geared = belt(two_weapon_fighter());
    let dexterity = Choice::AbilityScore {
        ability: Ability::Dexterity,
    };
    let the_belt = Choice::Gear(GearSpot::Worn(GearPlace::Belt));
    let removable = engine.removable(&geared);
    assert!(!removable.contains(&dexterity));
    assert!(!removable.contains(&the_belt));
    assert!(removable.contains(&Choice::Slot(general_slot(1))));
    // Once the feat is gone, the belt comes off like any other choice.
    let bare = without(&geared, &general_slot(1));
    assert!(engine.is_removable(&bare, &the_belt));
    // A withdrawal that the gear alone keeps legal: the level-4 increase to Dexterity, under the gloves.
    let mut build = character([10, 18, 10, 10, 10, 10], &times("fighter", 5));
    build.levels[3].ability_increase = Some(Ability::Dexterity);
    build.picks.insert(general_slot(5), pick_feat(GTWF));
    assert!(engine.is_possible(&build));
    let gloved = wearing(build.clone(), GearPlace::Gloves, "gloves-of-dexterity-4");
    let increase = Choice::AbilityIncrease { level: 4 };
    assert!(!engine.is_removable(&build, &increase));
    let (edit, removable) = engine.withdrawal(&gloved, &increase).expect("held");
    assert!(removable);
    assert_eq!(
        engine.newly_gear_dependent(&gloved, &edit),
        [Choice::Slot(general_slot(5))]
    );
}

// ---- the other pickers that change a score or a class -----------------------------------------------------------

#[test]
fn moving_the_racial_bonus_off_a_score_a_feat_needs_is_open_under_a_belt_and_says_what_leans_on_it()
{
    let engine = engine_with_gtwf();
    // Dexterity 17 + the human's 2: 19 without gear.
    let mut build = two_weapon_fighter();
    build.start.racial_ability = Some(Ability::Dexterity);
    assert!(engine.is_possible(&build));
    let option = |build: &Build, ability: Ability| {
        (engine.racial_ability_options(build).into_iter())
            .find(|option| option.value == ability)
            .expect("the ability")
    };
    // Without the belt the move is refused.
    assert_eq!(
        option(&build, Ability::Strength).status,
        Err(Issue::BreaksLater)
    );

    let geared = belt(build);
    let strength = option(&geared, Ability::Strength);
    assert_eq!(strength.status, Ok(()));
    assert_eq!(strength.leans_on_gear, [Choice::Slot(general_slot(1))]);
    // The ability it is on leans on nothing.
    let dexterity = option(&geared, Ability::Dexterity);
    assert_eq!(
        (dexterity.status, dexterity.leans_on_gear),
        (Ok(()), vec![])
    );
    // Nor does any option once the feat needs the belt already.
    let leaning = belt(two_weapon_fighter());
    assert!(
        engine
            .racial_ability_options(&leaning)
            .iter()
            .all(|option| option.leans_on_gear.is_empty())
    );
    // The race picker, which moves the bonus too, marks each open race as the engine's own answer does.
    let races = engine.race_options(&geared);
    assert!(races.iter().any(|option| !option.leans_on_gear.is_empty()));
    for race in races.iter().filter(|option| option.status.is_ok()) {
        assert_eq!(
            race.leans_on_gear,
            engine.newly_gear_dependent(&geared, &race.edit),
            "{:?}",
            race.value
        );
    }
}

#[test]
fn a_levels_ability_increase_moved_off_a_score_a_feat_needs_says_what_leans_on_the_gear() {
    let engine = engine_with_gtwf();
    // Dexterity 18, the level-4 increase on it, and the feat at 5: 19 without gear.
    let mut build = character([10, 18, 10, 10, 10, 10], &times("fighter", 5));
    build.levels[3].ability_increase = Some(Ability::Dexterity);
    build.picks.insert(general_slot(5), pick_feat(GTWF));
    assert!(engine.is_possible(&build));
    let strength = |build: &Build| {
        (engine
            .ability_increase_options(build, 4)
            .expect("level 4")
            .into_iter())
        .find(|option| option.value == Ability::Strength)
        .expect("Strength")
    };
    assert_eq!(strength(&build).status, Err(Issue::BreaksLater));

    let gloved = wearing(build, GearPlace::Gloves, "gloves-of-dexterity-4");
    let option = strength(&gloved);
    assert_eq!(option.status, Ok(()));
    assert_eq!(option.leans_on_gear, [Choice::Slot(general_slot(5))]);
}

#[test]
fn a_class_whose_entry_only_the_gear_meets_is_open_and_says_it_leans_on_the_gear() {
    // The monk asks for Dexterity 19 here.
    let mut data = world();
    let monk = (data.classes.iter_mut())
        .find(|row| row.key == class("monk"))
        .expect("the test world has a monk");
    monk.prerequisite = Some(Prerequisite::AbilityScore {
        ability: Ability::Dexterity,
        score: 19,
    });
    let engine = engine_on(data);
    let build = character([10, 17, 10, 10, 10, 10], &["fighter"]);
    let option = |build: &Build, level: u8, key: &str| {
        (engine
            .class_options(build, level)
            .expect("the level")
            .into_iter())
        .find(|option| option.class == class(key) && option.archetype.is_none())
        .expect("the class")
    };
    assert!(option(&build, 2, "monk").status.is_err());

    let geared = belt(build);
    // One past the last level, and at a level the Build has.
    let monk = option(&geared, 2, "monk");
    assert_eq!(monk.status, Ok(()));
    assert_eq!(monk.leans_on_gear, [Choice::Class { level: 2 }]);
    let fighter = option(&geared, 2, "fighter");
    assert_eq!((fighter.status, fighter.leans_on_gear), (Ok(()), vec![]));
    let first = option(&geared, 1, "monk");
    assert_eq!(first.status, Ok(()));
    assert_eq!(first.leans_on_gear, [Choice::Class { level: 1 }]);
    // Taken, the class is reported as the Build's other gear-dependent choices are, and locks the belt.
    let mut taken = geared.clone();
    engine.apply(&mut taken, &monk.edit);
    assert_eq!(
        engine.gear_dependents(&taken),
        [GearDependent {
            choice: Choice::Class { level: 2 },
            needs: all_of(&["belt-of-dexterity-2"]),
        }]
    );
    let none = &engine.gear_options(&taken, GearPlace::Belt)[0];
    assert_eq!(
        none.reason.as_deref(),
        Some("Level 2: Monk needs Belt of Dexterity +2")
    );
}
