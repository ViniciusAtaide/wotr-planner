//! Ticket 35: the user's decisions on tickets 32 and 33's open questions (2026-11-01, "all recommended"), each built
//! where its owner is.
//!
//! - **Half-elf subraces** (32-1): Dual Heritage and Kindred Raised list no Adaptability, so they get no slot. A
//!   subrace's slots replace the race's (as its ability modifiers do), and the extraction gives a subrace none:
//!   the rule only checks the guide and the data ([`check`]).
//! - **Impossible Domain** (33-2): a Living Deity's extra domain runs on arcanist levels from the pick
//!   (`level_also_from`: the Living Deity's way counts beside the domains held), and a Hellknight with Pentamic
//!   Faith's domain qualifies (levelled by Hellknight levels, as that domain is): built with the domains
//!   ([`option_families`](crate::option_families)' provider).
//! - **Signifer** (33-3): its order is offered only to a character holding none, as the Hellknight's: built with the
//!   Hellknight's order ([`option_families`](crate::option_families)' provider).
//! - **Separatist** (33-6): its second domain isn't an alignment domain that conflicts with its alignment: built
//!   with the domains ([`option_families`](crate::option_families)' provider).
//! - **Ranger/slayer** (33-7): the slayer's three combat style feats come from the style's list of their window's
//!   stage (2, 6, 10), whoever levels the style (`OptionList.at_level`): built with the slayer's talents
//!   ([`follow_ups`](crate::follow_ups)' provider).
//! - **Bloodrager groups** (33-8): the bloodrager's ten Draconic and four Elemental bloodlines are groups of their
//!   own, so a Second Bloodrager Bloodline can't be a second dragon (or element): on the bloodlines before they're
//!   given to the class stage ([`bloodrager_groups`]).
//! - **No druid takes Impossible Domain** (ticket 36, the user following the guide): the druid's page says a druid
//!   who chooses a domain can't, so no druid domain (Nature Bond, Blight Druid, Winter Child) is in its
//!   prerequisite. A guard only ([`check`]): it changes no data.

use rules::{FeatSlot, GameData, MythicAbilityKey, OptionGroup, Prerequisite};

use crate::guide::Guide;
use crate::option_levels::{Log, page, section};

/// The guards of [`half_elf_subraces`] and [`no_druid_impossible_domain`], on the finished data. Changes nothing.
pub(crate) fn check(guide: &Guide, data: &GameData) {
    half_elf_subraces(guide, data);
    pentamic_faith_domain(data);
    no_druid_impossible_domain(guide, data, &mut Log::new());
}
/// "Adaptability: Half-elves receive Skill Focus as a bonus feat at 1st level" is the half-elf's trait; the two
/// subraces' own trait tables don't list it.
fn half_elf_subraces(guide: &Guide, data: &GameData) {
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
    }
}

/// The Pentamic Faith discipline's row grants the domain that lets a Hellknight take Impossible Domain.
fn pentamic_faith_domain(data: &GameData) {
    let discipline = "hellknight/disciplines/pentamic-faith";
    let faith = data
        .prestige_classes
        .iter()
        .find(|class| class.key.as_str() == "hellknight")
        .and_then(|class| {
            class
                .selections
                .iter()
                .find(|s| s.key.as_str() == "hellknight/disciplines")
        })
        .and_then(|selection| {
            selection
                .options
                .iter()
                .find(|option| option.key.as_str() == discipline)
        })
        .expect("Pentamic Faith");
    assert_eq!(
        faith.levels.first().map(|row| row
            .selections
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()),
        Some(vec!["hellknight/pentamic-faith"])
    );
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

/// The bloodrager's Draconic and Elemental bloodlines (the user's decision; the guide states the rule for the
/// sorcerer's only): one of each group at most, apart from the sorcerer's groups. On the family before it's given
/// to the class stage ([`crate::option_levels`]).
pub(crate) fn bloodrager_groups(bloodline: &mut rules::FeatureSelection) {
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
        crate::option_follow_ups::group_options(bloodline, prefix, group, count);
    }
}
