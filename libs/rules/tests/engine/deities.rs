//! Deities (ticket 86): a Build start follows a deity or chooses atheism. The deity limits the alignment, the
//! domains of a deity-limited selection (only the deity's, or only the others), and gives its favoured weapons to a
//! feature that grants them; prerequisites ask for a deity, a given deity, atheism, or a deity offering a domain.

use rules::{
    Alignment, Build, ClassChoice, DeityDomains, DeityKey, Edit, Engine, Faith, Issue,
    Prerequisite, Unmet, encode_build,
};

use crate::support::*;

fn deity(key: &str) -> Option<Faith> {
    Some(Faith::Deity(DeityKey::new(key)))
}

/// A lawful neutral human of `faith` taking `classes` (every deity of the world allows Lawful Neutral).
fn follower(faith: Option<Faith>, classes: &[&str]) -> Build {
    let mut build = build("human", classes);
    build.start.alignment = Some(Alignment::LawfulNeutral);
    build.start.deity = faith;
    build
}

/// A Priest of the deity, `levels` levels.
fn priest(key: &str, levels: usize) -> Build {
    follower(deity(key), &times("priest", levels))
}

fn with_archetype(mut build: Build, class: &str, archetype: &str) -> Build {
    build.class_choices.insert(
        crate::support::class(class),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new(archetype)),
            ..ClassChoice::default()
        },
    );
    build
}

/// The status of each option of a slot, by the option's key.
fn statuses(build: &Build, slot: &rules::SlotId) -> Vec<(String, Result<(), Issue>)> {
    engine()
        .options(build, slot)
        .unwrap()
        .into_iter()
        .map(|option| (option.pick.key().to_owned(), option.status))
        .collect()
}

/// The keys a slot's picker leaves open.
fn open(build: &Build, slot: &rules::SlotId) -> Vec<String> {
    statuses(build, slot)
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .collect()
}

fn text(issue: &Issue) -> String {
    engine().issue_text(issue).unwrap()
}

/// The class picker's status for a class (with an archetype) at level 1.
fn class_status(build: &Build, class: &str, archetype: Option<&str>) -> Result<(), Issue> {
    engine()
        .class_options(build, 1)
        .unwrap()
        .into_iter()
        .find(|option| {
            option.class.as_str() == class
                && option.archetype.as_ref().map(|a| a.as_str()) == archetype
        })
        .unwrap()
        .status
}

fn needs(prerequisite: Prerequisite) -> Result<(), Issue> {
    Err(Issue::Unmet(Unmet::Condition {
        needs: prerequisite,
        has: None,
    }))
}

// ---- the start check -------------------------------------------------------------------------------------------

#[test]
fn a_deity_limits_the_alignment_to_its_own() {
    let mut build = follower(deity("lawgiver"), &[]);
    assert!(engine().is_possible(&build));
    build.start.alignment = Some(Alignment::TrueNeutral);
    assert!(!engine().is_possible(&build));
    build.start.alignment = None;
    assert!(engine().is_possible(&build), "no alignment chosen yet");
    // Atheism allows every alignment.
    for alignment in Alignment::ALL {
        build.start.alignment = Some(alignment);
        build.start.deity = Some(Faith::Atheism);
        assert!(engine().is_possible(&build), "{alignment:?}");
    }
}

#[test]
fn the_start_pickers_lock_a_misfit_with_the_deitys_alignments() {
    let engine = engine();
    let mut start = follower(deity("lawgiver"), &[]).start;
    assert_eq!(engine.faith_fits(&start), Ok(()));
    start.alignment = Some(Alignment::ChaoticNeutral);
    let misfit = engine.faith_fits(&start).unwrap_err();
    assert_eq!(
        misfit,
        Issue::DeityAlignment {
            deity: DeityKey::new("lawgiver")
        }
    );
    assert_eq!(
        text(&misfit),
        "Lawgiver allows Lawful Good, Neutral Good or Lawful Neutral"
    );
    start.deity = Some(Faith::Atheism);
    assert_eq!(engine.faith_fits(&start), Ok(()));
    start.deity = None;
    assert_eq!(engine.faith_fits(&start), Ok(()));
}

#[test]
fn a_deity_the_game_data_lacks_makes_a_stale_link() {
    let engine = engine();
    let gone = follower(deity("cut-in-a-later-patch"), &["fighter"]);
    assert!(!engine.is_possible(&gone));
    assert!(engine.open_link(&encode_build(&gone)).is_err());
    let misfit = Build {
        start: rules::BuildStart {
            alignment: Some(Alignment::ChaoticEvil),
            ..follower(deity("lawgiver"), &[]).start
        },
        ..Build::default()
    };
    assert!(engine.open_link(&encode_build(&misfit)).is_err());
    let fits = follower(deity("fist"), &["fighter"]);
    assert_eq!(engine.open_link(&encode_build(&fits)), Ok(fits));
}

// ---- the prerequisites -----------------------------------------------------------------------------------------

#[test]
fn a_class_serving_a_deity_needs_one() {
    let none = follower(None, &[]);
    assert_eq!(
        class_status(&none, "priest", None),
        needs(Prerequisite::Worships)
    );
    let atheist = follower(Some(Faith::Atheism), &[]);
    assert_eq!(
        class_status(&atheist, "priest", None),
        needs(Prerequisite::Worships)
    );
    assert_eq!(
        class_status(&follower(deity("balance"), &[]), "priest", None),
        Ok(())
    );
    assert!(!engine().is_possible(&follower(None, &["priest"])));
    assert!(engine().is_possible(&priest("balance", 1)));
}

#[test]
fn an_archetype_may_need_one_deity() {
    let zealot =
        |key: &str| class_status(&follower(deity(key), &[]), "priest", Some("priest/zealot"));
    assert_eq!(zealot("lawgiver"), Ok(()));
    assert_eq!(
        zealot("balance"),
        needs(Prerequisite::Deity(DeityKey::new("lawgiver")))
    );
}

#[test]
fn an_archetype_may_need_a_deity_offering_a_domain() {
    let apostle =
        |key: &str| class_status(&follower(deity(key), &[]), "priest", Some("priest/apostle"));
    assert_eq!(apostle("lawgiver"), Ok(()));
    assert_eq!(
        apostle("fist"),
        needs(Prerequisite::DeityOffers(option("cleric/domain/fire")))
    );
}

#[test]
fn an_option_may_need_atheism_or_a_deity() {
    let slot = selection_slot(1, 0);
    assert_eq!(
        open(&follower(Some(Faith::Atheism), &["seeker"]), &slot),
        ["seeker/creed/godless"]
    );
    assert_eq!(
        open(&follower(deity("fist"), &["seeker"]), &slot),
        ["seeker/creed/devout"]
    );
    assert!(open(&follower(None, &["seeker"]), &slot).is_empty());
}

#[test]
fn the_new_conditions_read_as_the_sheet_says_them() {
    let engine = engine();
    let reason = |prerequisite: Prerequisite| text(&needs(prerequisite).unwrap_err());
    assert_eq!(reason(Prerequisite::Worships), "Needs a deity");
    assert_eq!(reason(Prerequisite::Atheist), "Needs atheism");
    assert_eq!(
        reason(Prerequisite::Deity(DeityKey::new("lawgiver"))),
        "Needs Lawgiver as deity"
    );
    assert_eq!(
        reason(Prerequisite::DeityOffers(option("cleric/domain/fire"))),
        "Needs a deity offering the fire"
    );
    // Refused, as a `Not`: what rules the option out.
    assert_eq!(
        reason(Prerequisite::Not(Box::new(Prerequisite::DeityOffers(
            option("cleric/domain/fire")
        )))),
        "Not for a deity offering the fire"
    );
    assert_eq!(engine.deity_name(&DeityKey::new("fist")), "Fist");
    assert_eq!(engine.faith_name(&Faith::Atheism), "Atheism");
}

// ---- deity-limited selections ----------------------------------------------------------------------------------

#[test]
fn a_deity_limited_domain_is_one_of_the_deitys() {
    let build = priest("lawgiver", 1);
    assert_eq!(
        statuses(&build, &selection_slot(1, 0)),
        [
            ("cleric/domain/fire".to_owned(), Ok(())),
            ("cleric/domain/war".to_owned(), Ok(())),
            (
                "cleric/domain/water".to_owned(),
                Err(Issue::DeityDomain {
                    deity: DeityKey::new("lawgiver"),
                    limit: DeityDomains::Offered,
                })
            ),
        ]
    );
    let not_offered = Issue::DeityDomain {
        deity: DeityKey::new("lawgiver"),
        limit: DeityDomains::Offered,
    };
    assert_eq!(text(&not_offered), "Not a domain of Lawgiver");
}

#[test]
fn a_heresy_is_a_domain_the_deity_does_not_offer() {
    let build = priest("lawgiver", 1);
    let issue = Issue::DeityDomain {
        deity: DeityKey::new("lawgiver"),
        limit: DeityDomains::NotOffered,
    };
    let offered = Err(issue.clone());
    assert_eq!(
        statuses(&build, &selection_slot(1, 1)),
        [
            ("cleric/domain/fire".to_owned(), offered.clone()),
            ("cleric/domain/war".to_owned(), offered.clone()),
            ("cleric/domain/water".to_owned(), Ok(())),
        ]
    );
    assert_eq!(text(&issue), "A domain of Lawgiver");
}

#[test]
fn a_blessing_is_checked_by_its_domain() {
    let build = priest("balance", 2);
    assert_eq!(
        open(&build, &selection_slot(2, 0)),
        ["priest/blessing/water"]
    );
    let build = priest("lawgiver", 2);
    assert_eq!(
        open(&build, &selection_slot(2, 0)),
        ["priest/blessing/fire"]
    );
}

#[test]
fn a_pick_outside_the_deitys_domains_makes_the_build_impossible() {
    let mut build = priest("lawgiver", 1);
    build
        .picks
        .insert(selection_slot(1, 0), pick_option("cleric/domain/water"));
    assert_eq!(
        why_not(&build, &selection_slot(1, 0)),
        Some(Issue::DeityDomain {
            deity: DeityKey::new("lawgiver"),
            limit: DeityDomains::Offered,
        })
    );
}

#[test]
fn the_deity_does_not_limit_a_selection_without_a_limit() {
    // The cleric's own domain selection names no limit: every domain, whatever the deity.
    let build = follower(deity("balance"), &["cleric"]);
    assert_eq!(
        open(&build, &selection_slot(1, 0)),
        [
            "cleric/domain/fire",
            "cleric/domain/war",
            "cleric/domain/water"
        ]
    );
}

// ---- favoured weapons ------------------------------------------------------------------------------------------

#[test]
fn an_exotic_favoured_weapon_is_held_as_its_proficiency() {
    let state = engine().evaluate(&priest("lawgiver", 1)).levels[0]
        .state
        .clone();
    assert!(state.has_feat_for(&feat("exotic-weapon-proficiency"), &weapon("dueling-sword")));
    // A simple or martial weapon needs no feat.
    let state = engine().evaluate(&priest("balance", 1)).levels[0]
        .state
        .clone();
    assert!(!state.feats.contains_key(&feat("exotic-weapon-proficiency")));
}

#[test]
fn the_unarmed_strike_is_held_as_improved_unarmed_strike() {
    let state = |build: &Build| engine().evaluate(build).levels[0].state.clone();
    assert_eq!(
        state(&priest("fist", 1)).feat_count(&feat("improved-unarmed-strike")),
        1
    );
    // Only with a feature that grants the favoured weapons: a fighter of the Fist holds nothing.
    assert_eq!(
        state(&follower(deity("fist"), &["fighter"])).feat_count(&feat("improved-unarmed-strike")),
        0
    );
    // With another deity it goes.
    assert_eq!(
        state(&priest("balance", 1)).feat_count(&feat("improved-unarmed-strike")),
        0
    );
    // It counts for prerequisites: Stunning Fist at level 1.
    let mut build = priest("fist", 1);
    build
        .picks
        .insert(general_slot(1), pick_feat("stunning-fist"));
    assert!(engine().is_possible(&build));
}

#[test]
fn the_sheet_notes_the_favoured_weapons() {
    let engine = engine();
    let note = |build: &Build| {
        let state = engine.evaluate(build).levels[0].state.clone();
        engine.favoured_weapons_note(&state)
    };
    assert_eq!(
        note(&priest("lawgiver", 1)).as_deref(),
        Some("Proficient with dueling-sword (Lawgiver)")
    );
    assert_eq!(note(&follower(deity("fist"), &["fighter"])), None);
}

// ---- withdrawing the deity --------------------------------------------------------------------------------------

/// The deity edit of a Build start.
fn to_deity(build: &Build, faith: Option<Faith>) -> Edit {
    Edit::Start(rules::BuildStart {
        deity: faith,
        ..build.start.clone()
    })
}

#[test]
fn a_held_domain_locks_the_deities_that_do_not_offer_it() {
    let engine = engine();
    let mut build = priest("lawgiver", 1);
    build
        .picks
        .insert(selection_slot(1, 0), pick_option("cleric/domain/war"));
    assert!(engine.allows(&build, &to_deity(&build, deity("fist"))));
    assert!(!engine.allows(&build, &to_deity(&build, deity("balance"))));
    // Taken back, any deity fits again.
    let build = without(&build, &selection_slot(1, 0));
    assert!(engine.allows(&build, &to_deity(&build, deity("balance"))));
}

#[test]
fn a_pick_needing_a_favoured_weapon_locks_the_deity() {
    let engine = engine();
    let mut build = priest("fist", 1);
    build
        .picks
        .insert(general_slot(1), pick_feat("stunning-fist"));
    assert!(!engine.allows(&build, &to_deity(&build, deity("balance"))));
    let build = without(&build, &general_slot(1));
    assert!(engine.allows(&build, &to_deity(&build, deity("balance"))));
}

#[test]
fn the_deity_comes_off_once_nothing_needs_it() {
    let engine = engine();
    let mut devout = follower(deity("fist"), &["seeker"]);
    assert!(engine.is_removable(&devout, &rules::Choice::Deity));
    devout
        .picks
        .insert(selection_slot(1, 0), pick_option("seeker/creed/devout"));
    assert!(!engine.is_removable(&devout, &rules::Choice::Deity));
    // A Priest's class needs one while it's held.
    assert!(!engine.is_removable(&priest("fist", 1), &rules::Choice::Deity));
    assert!(
        !engine
            .removable(&follower(None, &["fighter"]))
            .contains(&rules::Choice::Deity),
        "nothing to remove"
    );
}

#[test]
fn every_deity_is_listed_by_name() {
    let engine = Engine::new(world());
    let names: Vec<&str> = engine.deities().iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, ["Balance", "Fist", "Lawgiver"]);
}

#[test]
fn an_archetype_needing_a_deity_is_impossible_without_it() {
    let zealot = with_archetype(priest("balance", 1), "priest", "priest/zealot");
    assert!(!engine().is_possible(&zealot));
    let zealot = with_archetype(priest("lawgiver", 1), "priest", "priest/zealot");
    assert!(engine().is_possible(&zealot));
}

/// The Champion of the Faith's alignment blessing (question 13): an alignment and a deity offering its domain, each
/// as the case its slot answers. Both failing read as that case once.
#[test]
fn conditions_of_one_case_read_as_that_case_once() {
    let as_heresy = || Unmet::Condition {
        needs: Prerequisite::Not(Box::new(Prerequisite::PickedThrough(
            rules::FeatureSelectionKey::new("priest/heresy"),
        ))),
        has: None,
    };
    let condition = |needs| Unmet::Condition { needs, has: None };
    let unmet = Unmet::All(vec![
        Unmet::Any(vec![
            as_heresy(),
            condition(Prerequisite::Alignment(Alignment::LawfulGood)),
            condition(Prerequisite::Alignment(Alignment::NeutralGood)),
        ]),
        Unmet::Any(vec![
            as_heresy(),
            condition(Prerequisite::DeityOffers(option("cleric/domain/fire"))),
        ]),
    ]);
    assert_eq!(
        text(&Issue::Unmet(unmet)),
        "As priest/heresy, needs Lawful Good alignment or Neutral Good alignment, and a deity offering the fire"
    );
}
