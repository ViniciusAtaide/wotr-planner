//! Ticket 35: the user's decisions on tickets 32 and 33's open questions (2026-11-01, "all recommended"), applied
//! to finished data.
//!
//! - **Half-elf subraces** (32-1): Dual Heritage and Kindred Raised list no Adaptability, so they get no slot. A
//!   subrace's slots replace the race's (as its ability modifiers do), and the extraction gives a subrace none:
//!   the rule only checks the guide and the data.
//! - **Impossible Domain** (33-2): a Living Deity's extra domain runs on arcanist levels from the pick
//!   (`level_also_from`: the Living Deity's way counts beside the domains held), and a Hellknight with Pentamic
//!   Faith's domain qualifies (levelled by Hellknight levels, as that domain is).
//! - **Signifer** (33-3): its order is offered only to a character holding none, as the Hellknight's: built with the
//!   Hellknight's order ([`option_families`](crate::option_families)' provider).
//! - **Separatist** (33-6): its second domain isn't an alignment domain that conflicts with its alignment.
//! - **Ranger/slayer** (33-7): the slayer's three combat style feats come from the style's list of their window's
//!   stage (2, 6, 10), whoever levels the style (`OptionList.at_level`).
//! - **Bloodrager groups** (33-8): the bloodrager's ten Draconic and four Elemental bloodlines are groups of their
//!   own, so a Second Bloodrager Bloodline can't be a second dragon (or element).
//! - **No druid takes Impossible Domain** (ticket 36, the user following the guide): the druid's page says a druid
//!   who chooses a domain can't, so no druid domain (Nature Bond, Blight Druid, Winter Child) is in its
//!   prerequisite. A guard only: it changes no data.
//!
//! Each rule asserts the guide phrase it rests on (the user's decisions aside) and what it finds in the data, so it
//! fails loudly on data it doesn't expect. [`apply`] runs on the finished data of an extraction and, through ticket
//! 35's one-off rewrite, on `libs/game-data/`.

use rules::{
    Alignment, FeatSlot, FeatureSelectionKey, GameData, LevelFrom, MythicAbilityKey, OptionGroup,
    Prerequisite, SelectionOptionKey,
};

use crate::follow_ups::{class_mut, says};
use crate::guide::Guide;
use crate::option_follow_ups::{and, class_selection};
use crate::option_levels::{Log, page, section};

/// Applies every rule, in the order above.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    half_elf_subraces(guide, data, &mut log);
    impossible_domain(guide, data, &mut log);
    no_druid_impossible_domain(guide, data, &mut log);
    separatist(guide, data, &mut log);
    slayer_window_stages(guide, data, &mut log);
    bloodrager_groups(data, &mut log);
    log
}

/// "Adaptability: Half-elves receive Skill Focus as a bonus feat at 1st level" is the half-elf's trait; the two
/// subraces' own trait tables don't list it.
fn half_elf_subraces(guide: &Guide, data: &mut GameData, log: &mut Log) {
    let creation = page(guide, "character-creation");
    let race = data
        .races
        .iter()
        .find(|race| race.key.as_str() == "half-elf")
        .expect("the half-elf");
    assert_eq!(
        race.feat_slots,
        [FeatSlot::OneOf {
            list: rules::FeatListKey::new("half-elf/adaptability")
        }]
    );
    for (key, anchor) in [
        ("half-elf/dual-heritage", "half-elf-dual-heritage"),
        ("half-elf/kindred-raised", "half-elf-kindred-raised"),
    ] {
        let text: String = section(creation, anchor)
            .tables()
            .flatten()
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(text.contains("Racial Traits"), "{anchor}: {text:?}");
        assert!(!text.contains("Adaptability"), "{anchor}: {text:?}");
        let subrace = race
            .subraces
            .iter()
            .find(|subrace| subrace.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"));
        assert_eq!(subrace.feat_slots, [], "{key}");
        log.push(format!("{key}: no slot of its own, so no Adaptability"));
    }
}

/// "Prerequisites: Domain" and "You may select any domain."; the Hellknight's Pentamic Faith domain: "She uses her
/// class level to determine which domain abilities she can use." The Living Deity's levels are the user's decision.
fn impossible_domain(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(guide, "mythic-paths", "impossible-domain", "Domain");
    says(
        guide,
        "prestige-classes",
        "pentamic-faith",
        "She uses her class level to determine which domain abilities she can use.",
    );
    let living_deity =
        SelectionOptionKey::new("arcanist/magic-deceiver/ways-of-the-deceiver/living-deity");
    let key = "cleric/impossible-domain";
    let selection = class_selection(data, "cleric", key);
    assert_eq!(selection.level_from, Some(LevelFrom::OtherOptions), "{key}");
    assert_eq!(selection.level_also_from, None, "{key}");
    selection.level_also_from = Some(living_deity.clone());
    log.push(format!(
        "{key}: the Living Deity's way counts beside the domains held (arcanist levels from the pick)"
    ));
    // The Pentamic Faith discipline's row grants the domain.
    let discipline = "hellknight/disciplines/pentamic-faith";
    let faith = class_selection(data, "hellknight", "hellknight/disciplines")
        .options
        .iter()
        .find(|option| option.key.as_str() == discipline)
        .expect("Pentamic Faith");
    assert_eq!(
        faith.levels.first().map(|row| &row.selections[..]),
        Some(&[FeatureSelectionKey::new("hellknight/pentamic-faith")][..])
    );
    let ability = data
        .mythic_abilities
        .iter_mut()
        .find(|ability| ability.key == MythicAbilityKey::new("impossible-domain"))
        .expect("Impossible Domain");
    let Some(Prerequisite::Any(children)) = &mut ability.prerequisite else {
        panic!("Impossible Domain: {:?}", ability.prerequisite);
    };
    assert!(children.contains(&Prerequisite::SelectionOption(living_deity)));
    children.push(Prerequisite::SelectionOption(SelectionOptionKey::new(
        discipline,
    )));
    children.sort();
    log.push(format!(
        "impossible-domain: a Hellknight with Pentamic Faith's domain ({discipline}) may take it"
    ));
}

/// "If you choose a domain, you can't take the Impossible Domain mythic ability." (the druid's characteristics,
/// after its Nature Bond domains; the Blight Druid's and Winter Child's domains are the druid's under an
/// archetype). Asserts the phrase and that no druid source is among Impossible Domain's prerequisites; changes no
/// data. Public so ticket 36's check runs it alone over `libs/game-data/`.
pub fn no_druid_impossible_domain(guide: &Guide, data: &GameData, log: &mut Log) {
    let phrase = "If you choose a domain, you can't take the Impossible Domain mythic ability.";
    let druid = page(guide, "druid")
        .root
        .walk()
        .into_iter()
        .find(|section| section.name == "Druid")
        .expect("the druid's section");
    let text = druid.text();
    assert!(text.contains(phrase), "druid: no {phrase:?} in {text:?}");
    let ability = data
        .mythic_abilities
        .iter()
        .find(|ability| ability.key == MythicAbilityKey::new("impossible-domain"))
        .expect("Impossible Domain");
    let Some(Prerequisite::Any(children)) = &ability.prerequisite else {
        panic!("Impossible Domain: {:?}", ability.prerequisite);
    };
    let druids = |child: &&Prerequisite| match child {
        Prerequisite::ClassLevel { class, .. } => class.as_str() == "druid",
        Prerequisite::Archetype(archetype) => archetype.as_str().starts_with("druid/"),
        Prerequisite::SelectionOption(option) => option.as_str().starts_with("druid/"),
        _ => false,
    };
    let found: Vec<_> = children.iter().filter(druids).collect();
    assert!(
        found.is_empty(),
        "impossible-domain: druid sources {found:?}"
    );
    log.push(
        "impossible-domain: no druid domain qualifies (Nature Bond, Blight Druid, Winter Child)"
            .to_owned(),
    );
}

/// "This second domain cannot be an alignment domain that conflicts with the cleric's own alignment." Each
/// alignment domain, picked through the Separatist's second-domain selection, needs an alignment off the opposite
/// side of its axis.
fn separatist(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "cleric",
        "forbidden-rites",
        "This second domain cannot be an alignment domain that conflicts with the cleric's own alignment.",
    );
    use Alignment::*;
    let second = FeatureSelectionKey::new("cleric/separatist/domain");
    assert!(
        class_selection(data, "cleric", second.as_str())
            .shared_options
            .len()
            == 37
    );
    let domains = class_selection(data, "cleric", "cleric/domain");
    for (domain, alignments) in [
        (
            "chaos",
            [
                NeutralGood,
                ChaoticGood,
                TrueNeutral,
                ChaoticNeutral,
                NeutralEvil,
                ChaoticEvil,
            ],
        ),
        (
            "evil",
            [
                LawfulNeutral,
                TrueNeutral,
                ChaoticNeutral,
                LawfulEvil,
                NeutralEvil,
                ChaoticEvil,
            ],
        ),
        (
            "good",
            [
                LawfulGood,
                NeutralGood,
                ChaoticGood,
                LawfulNeutral,
                TrueNeutral,
                ChaoticNeutral,
            ],
        ),
        (
            "law",
            [
                LawfulGood,
                NeutralGood,
                LawfulNeutral,
                TrueNeutral,
                LawfulEvil,
                NeutralEvil,
            ],
        ),
    ] {
        let key = format!("cleric/domain/{domain}");
        let option = domains
            .options
            .iter_mut()
            .find(|option| option.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"));
        assert_eq!(option.prerequisite, None, "{key}");
        let mut children: Vec<Prerequisite> = alignments.map(Prerequisite::Alignment).into();
        children.push(Prerequisite::Not(Box::new(Prerequisite::PickedThrough(
            second.clone(),
        ))));
        children.sort();
        option.prerequisite = Some(Prerequisite::Any(children));
        log.push(format!(
            "{key}: as the Separatist's second domain, no conflicting alignment"
        ));
    }
}

/// "He can only do so three times in total and at certain levels: 2 or 4, 6 or 8 and 10 or greater": each combat
/// style talent's feat comes from the style's list of its window's stage, whichever class levels the style (a
/// ranger/slayer shares the ranger's: the user's decision).
fn slayer_window_stages(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "slayer",
        "slayer-camouflage",
        "he can only do so three times in total and at certain levels: 2 or 4, 6 or 8 and 10 or greater",
    );
    let style = FeatureSelectionKey::new("slayer/combat-style");
    let talents = class_selection(data, "slayer", "slayer/slayer-talents");
    for stage in [2, 6, 10] {
        let talent = format!("slayer/slayer-talents/combat-style-feat-{stage}");
        let option = talents
            .options
            .iter_mut()
            .find(|option| option.key.as_str() == talent)
            .unwrap_or_else(|| panic!("no {talent}"));
        let [row] = &mut option.levels[..] else {
            panic!("{talent}: one row");
        };
        let unfixed = FeatSlot::OptionList {
            selection: style.clone(),
            at_level: None,
            label: None,
        };
        assert_eq!(row.feat_slots, [unfixed], "{talent}");
        row.feat_slots = vec![FeatSlot::OptionList {
            selection: style.clone(),
            at_level: Some(stage),
            label: None,
        }];
        log.push(format!(
            "{talent}: the feat from the style's list at option level {stage}"
        ));
    }
    // The style's own levels no longer choose a list: still slayer levels, as its selection names.
    assert!(
        class_mut(data, "slayer")
            .selections
            .iter()
            .any(|selection| selection.key == style)
    );
}

/// The bloodrager's Draconic and Elemental bloodlines (the user's decision; the guide states the rule for the
/// sorcerer's only): one of each group at most, apart from the sorcerer's groups.
fn bloodrager_groups(data: &mut GameData, log: &mut Log) {
    let bloodline = class_selection(data, "bloodrager", "bloodrager/bloodline");
    for (prefix, group, count) in [
        (
            "bloodrager/bloodline/draconic-",
            OptionGroup::BloodragerDraconic,
            10,
        ),
        (
            "bloodrager/bloodline/elemental-",
            OptionGroup::BloodragerElemental,
            4,
        ),
    ] {
        let mut found = 0;
        for option in bloodline
            .options
            .iter_mut()
            .filter(|option| option.key.as_str().starts_with(prefix))
        {
            assert_eq!(option.group, None, "{}", option.key);
            option.group = Some(group);
            and(
                &mut option.prerequisite,
                Prerequisite::NoOtherOptionOfGroup(group),
            );
            log.push(format!("{}: one of the {} group", option.key, group.name()));
            found += 1;
        }
        assert_eq!(found, count, "{prefix}");
    }
}
