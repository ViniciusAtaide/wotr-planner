//! Ticket 33: the user's decisions on tickets 30 and 31's open questions about options (2026-11-01, "all
//! recommended"), and the user's fact about Impossible Domain, each built where its owner is.
//!
//! - **Option groups** (30-1): the sorcerer's ten Draconic bloodlines and four Elemental ones each form a group,
//!   and each needs "no other option of this group", so a Crossblooded sorcerer or a Second Bloodline can't take a
//!   second dragon (or element): on the bloodlines before they're given to the class stage
//!   ([`sorcerer_bloodlines`]).
//! - **Crossblooded** (30-2): every bloodline names its arcana, which the second bloodline gives besides its feats;
//!   the archetype's "Bloodline 2 Arcana" cell is that arcana's, so no feature is made of it.
//! - **Daily choices** (30-10): the Unsworn Shaman's Minor Spirit and Wandering Spirit 2; the hunter's Animal Focus
//!   (a feature, not a pick of eight aspects: its heading isn't read as a selection) and its Second Animal Focus.
//!   The Forester's "Animal Focus" is the hunter's.
//! - **Elemental Movement** (30-11): one per element (Air, Fire, Water), none for Earth: on the bloodlines.
//! - **Separatist** (31-1): its second domain, two levels behind (`levels_behind`); the second domain's cells are its
//!   rows: built with the domains ([`option_families`](crate::option_families)' provider).
//! - **Divine Hunter** (31-2): every domain power, by hunter level (no skipped levels): built with the domains.
//! - **Impossible Domain** (31-5, the user's fact): a further domain from the full list, levelled like the domains
//!   held (`LevelFrom::OtherOptions`); a Living Deity may take it: built with the domains.
//! - **Expanded Element** (31-6): the 15th-level element isn't the 7th-level one (`once_per_option`): built with
//!   the elements ([`option_families`](crate::option_families)' provider).
//! - **Revelations** (31-7): Cinder Dance needs no Lame curse; the Dual-Cursed Oracle's revelations offer
//!   Misfortune and Fortune; Maneuver Mastery's maneuver follows its revelation's level and Second Mystery the first
//!   mystery's, so a Living Deity's count arcanist levels: built with the mysteries
//!   ([`option_families`](crate::option_families)' provider).
//! - **Champion of the Faith** (31-8): its alignment blessing is one of his own alignment (`PickedThrough`), a
//!   condition of each alignment blessing beside its lines.
//! - **Signifer** (31-10): its order is a selection sharing the Hellknight's seven: built with the Hellknight's order
//!   ([`option_families`](crate::option_families)' provider).
//! - **The slayer's combat style** (32-2, a scope addition): the slayer picks one of the ranger's combat styles
//!   with his first combat style feat, and each of the three feats comes from that style's list of its stage: built
//!   with the slayer's talents ([`follow_ups`](crate::follow_ups)' provider).
//!
//! Each rule asserts the guide phrase it rests on (the user's decisions and fact aside) when the providers are read;
//! [`check`] asserts what only the finished data shows.

use std::collections::BTreeSet;

use rules::{
    Alignment, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData, OptionGroup,
    Prerequisite,
};

use crate::follow_ups::{says, take_feature};
use crate::guide::Guide;
use crate::option_levels::{Family, feature};
use crate::picks::{Pick, Picks, Rule};

/// Every feature a prerequisite names is still defined (a removed placeholder may have been named).
fn features_named_exist(data: &GameData) {
    let defined: BTreeSet<FeatureKey> = data
        .classes
        .iter()
        .chain(&data.prestige_classes)
        .flat_map(|class| {
            class
                .features
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.features))
        })
        .chain(data.mythic_paths.iter().flat_map(|path| &path.features))
        .map(|feature| feature.key.clone())
        .collect();
    fn named(prerequisite: &Prerequisite, out: &mut Vec<FeatureKey>) {
        match prerequisite {
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                for child in children {
                    named(child, out);
                }
            }
            Prerequisite::Not(child) => named(child, out),
            Prerequisite::Feature { feature, .. } => out.push(feature.clone()),
            _ => {}
        }
    }
    let mut features = Vec::new();
    for prerequisite in crate::references::prerequisites(data).into_iter().flatten() {
        named(prerequisite, &mut features);
    }
    for feature in features {
        assert!(
            defined.contains(&feature),
            "a prerequisite names {feature}, which is gone"
        );
    }
}

/// Adds `leaf` to an option's prerequisite: as its own, or beside what it needs already.
pub(crate) fn and(prerequisite: &mut Option<Prerequisite>, leaf: Prerequisite) {
    *prerequisite = Some(match prerequisite.take() {
        None => leaf,
        Some(Prerequisite::All(mut children)) => {
            children.push(leaf);
            children.sort();
            Prerequisite::All(children)
        }
        Some(other) => {
            let mut children = vec![other, leaf];
            children.sort();
            Prerequisite::All(children)
        }
    });
}

// ---- option groups and arcanas ---------------------------------------------------------------------------------

/// The sorcerer's bloodlines as the user decided (30-1, 30-2, 30-11), on the family before it's given to the class
/// stage ([`crate::option_levels`]): the groups, each bloodline's arcana and the elements' Elemental Movement.
pub(crate) fn sorcerer_bloodlines(guide: &Guide, bloodlines: &mut Family, picks: &mut Picks) {
    option_groups(guide, &mut bloodlines.selection);
    crossblooded(guide, bloodlines, picks);
    elemental_movement(guide, bloodlines);
}

fn option_groups(guide: &Guide, bloodline: &mut FeatureSelection) {
    says(
        guide,
        "character-creation",
        "draconic-bloodline",
        "You can only take a single draconic bloodline",
    );
    says(
        guide,
        "mythic-paths",
        "second-bloodline",
        "You cannot use this ability to select multiple Draconic bloodlines.",
    );
    says(
        guide,
        "character-creation",
        "elemental-bloodline",
        "Every Elemental bloodline has certain characteristics in common",
    );
    for (prefix, group, count) in [
        ("sorcerer/bloodline/draconic-", OptionGroup::Draconic, 10),
        ("sorcerer/bloodline/elemental-", OptionGroup::Elemental, 4),
    ] {
        group_options(bloodline, prefix, group, count);
    }
}

/// The options of `selection` whose key starts with `prefix` (`count` of them) form `group`, each needing no other
/// option of it.
pub(crate) fn group_options(
    selection: &mut FeatureSelection,
    prefix: &str,
    group: OptionGroup,
    count: usize,
) {
    let mut found = 0;
    for option in selection
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
        found += 1;
    }
    assert_eq!(found, count, "{prefix}");
}

/// "Gains the arcana and spells from both bloodlines": every bloodline's arcana is the feature of its first row
/// whose name ends in "Arcana". The archetype's "Bloodline 2 Arcana" cell is that arcana's: no feature is made.
fn crossblooded(guide: &Guide, bloodlines: &mut Family, picks: &mut Picks) {
    says(
        guide,
        "sorcerer",
        "crossblooded",
        "Gains the arcana and spells from both bloodlines.",
    );
    let names: std::collections::HashMap<FeatureKey, String> = bloodlines
        .features
        .iter()
        .map(|feature| (feature.key.clone(), feature.name.clone()))
        .collect();
    let bloodline = &mut bloodlines.selection;
    assert_eq!(bloodline.options.len(), 21, "the 21 bloodlines");
    for option in &mut bloodline.options {
        let first = option
            .levels
            .iter()
            .find(|row| row.level == 1)
            .unwrap_or_else(|| panic!("{} has no first row", option.key));
        let arcanas: Vec<&FeatureKey> = first
            .features
            .iter()
            .filter(|key| names.get(*key).is_some_and(|name| name.ends_with("Arcana")))
            .collect();
        let [arcana] = arcanas[..] else {
            panic!("{}: arcanas {arcanas:?}", option.key);
        };
        option.arcana = Some(arcana.clone());
    }
    picks.rule(Rule {
        feature: FeatureKey::new("sorcerer/crossblooded/bloodline-2-arcana"),
        table: Some("sorcerer/crossblooded".into()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::OptionRow,
    });
}

// ---- daily choices and Elemental Movement --------------------------------------------------------------------

/// The daily choices the tables define, built so where they're defined: the Unsworn Shaman's Minor Spirit and
/// Wandering Spirit 2, and the hunter's Second Animal Focus.
pub(crate) fn provide(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "shaman",
        "minor-spirit",
        "forms a temporary bond with a minor spirit each day",
    );
    says(
        guide,
        "shaman",
        "wandering-spirit-2",
        "gains access to a second Wandering Spirit at 6th level",
    );
    says(
        guide,
        "hunter",
        "second-animal-focus",
        "whenever a Hunter uses her Animal Focus ability, she can choose two aspects",
    );
    for key in [
        "shaman/unsworn-shaman/minor-spirit",
        "shaman/unsworn-shaman/wandering-spirit-2",
        "hunter/second-animal-focus",
    ] {
        picks.daily_choice(key);
    }
    // "A Hunter can grant an animal aspect to her companion": a daily choice of an aspect, so a feature rather than a
    // pick of eight, though the guide's "Animal Focus" heading lists them.
    says(
        guide,
        "hunter",
        "animal-focus",
        "A Hunter can grant an animal aspect to her companion.",
    );
    picks.not_selection(ANIMAL_FOCUS);
    picks.daily_choice(ANIMAL_FOCUS);
    // The Forester's "Animal Focus" is the hunter's (its table's cell named the hunter's heading before its own).
    picks.rule(Rule {
        feature: FeatureKey::new("hunter/forester/animal-focus-forester"),
        table: Some("hunter/forester".into()),
        cell: None,
        levels: None,
        pick: Pick::Feature(rules::Feature {
            daily_choice: true,
            ..feature(
                &FeatureKey::new(ANIMAL_FOCUS),
                "Animal Focus",
                "hunter#animal-focus".into(),
            )
        }),
    });
    champion_of_the_faith(guide, picks);
}

const ANIMAL_FOCUS: &str = "hunter/animal-focus";

/// What [`provide`]'s owners hold once built: the hunter's and its four archetypes' rows in force hold the Animal Focus
/// feature, and each alignment blessing needs what the Champion's rule says. Changes nothing.
pub(crate) fn check(data: &GameData) {
    let hunter = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "hunter")
        .expect("the hunter");
    let focus = FeatureKey::new(ANIMAL_FOCUS);
    assert!(
        hunter
            .features
            .iter()
            .any(|f| f.key == focus && f.daily_choice),
        "{ANIMAL_FOCUS}: a daily-choice feature"
    );
    let rows = hunter
        .levels
        .iter()
        .map(|row| &row.features)
        .chain(
            hunter
                .archetypes
                .iter()
                .flat_map(|a| &a.levels)
                .map(|row| &row.features),
        )
        .filter(|features| features.contains(&focus))
        .count();
    assert_eq!(
        rows, 5,
        "the hunter's and its four archetypes' Animal Focus"
    );
    features_named_exist(data);
    let blessings = hunter_free_blessings(data);
    for key in ["chaos", "evil", "good", "law"] {
        let key = format!("warpriest/blessings/{key}-domain");
        let option = blessings
            .iter()
            .find(|option| option.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"));
        assert!(
            matches!(&option.prerequisite, Some(Prerequisite::Any(children)) if children.len() == 4),
            "{key}: {:?}",
            option.prerequisite
        );
    }
}

/// The warpriest's blessings.
fn hunter_free_blessings(data: &GameData) -> &[rules::SelectionOption] {
    &data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "warpriest")
        .and_then(|class| {
            class
                .selections
                .iter()
                .find(|s| s.key.as_str() == "warpriest/blessings")
        })
        .expect("the warpriest's blessings")
        .options
}

/// "Air: …, Earth: None, Fire: …, Water: …": the 15th-level row of each Elemental bloodline.
fn elemental_movement(guide: &Guide, bloodlines: &mut Family) {
    for phrase in [
        "At 15th level, you gain a movement ability related your bloodline's element",
        "Air: immune to entanglement",
        "Earth: None",
        "Fire: +30' base speed",
        "Water: Immune to combat maneuvers",
    ] {
        says(guide, "character-creation", "elemental-movement", phrase);
    }
    let shared = "sorcerer/bloodline/elemental/elemental-movement";
    let mut new = Vec::new();
    for element in ["air", "earth", "fire", "water"] {
        let key = format!("sorcerer/bloodline/elemental-{element}");
        let option = bloodlines
            .selection
            .options
            .iter_mut()
            .find(|option| option.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"));
        let at = option
            .levels
            .iter()
            .position(|row| row.level == 15)
            .unwrap_or_else(|| panic!("{key} has no row 15"));
        let row = &mut option.levels[at];
        take_feature(&mut row.features, shared, &key);
        if element == "earth" {
            let empty = rules::OptionLevelRow {
                level: 15,
                ..rules::OptionLevelRow::default()
            };
            assert_eq!(row, &empty, "Earth's row 15 holds only Elemental Movement");
            option.levels.remove(at);
        } else {
            let own = FeatureKey::new(format!("{shared}-{element}"));
            row.features.push(own.clone());
            row.features.sort();
            let name = format!(
                "Elemental Movement ({}{})",
                element[..1].to_uppercase(),
                &element[1..]
            );
            new.push(feature(
                &own,
                &name,
                "character-creation#elemental-movement".into(),
            ));
        }
    }
    let before = bloodlines.features.len();
    bloodlines
        .features
        .retain(|feature| feature.key.as_str() != shared);
    assert_eq!(bloodlines.features.len(), before - 1, "no feature {shared}");
    bloodlines.features.extend(new);
}

// ---- the Champion of the Faith ---------------------------------------------------------------------------------

/// "The chosen alignment must correspond to his own alignment": each alignment blessing, picked through the
/// Champion's Chosen Alignment, needs one of its alignments; picked otherwise, nothing (its lines name none).
fn champion_of_the_faith(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "warpriest",
        "chosen-alignment",
        "The chosen alignment must correspond to his own alignment",
    );
    use Alignment::*;
    let chosen = FeatureSelectionKey::new("warpriest/champion-of-the-faith/chosen-alignment");
    for (blessing, alignments) in [
        ("chaos", [ChaoticGood, ChaoticNeutral, ChaoticEvil]),
        ("evil", [LawfulEvil, NeutralEvil, ChaoticEvil]),
        ("good", [LawfulGood, NeutralGood, ChaoticGood]),
        ("law", [LawfulGood, LawfulNeutral, LawfulEvil]),
    ] {
        let mut children: Vec<Prerequisite> = alignments.map(Prerequisite::Alignment).into();
        children.push(Prerequisite::Not(Box::new(Prerequisite::PickedThrough(
            chosen.clone(),
        ))));
        children.sort();
        picks.option_condition(
            &format!("warpriest/blessings/{blessing}-domain"),
            Prerequisite::Any(children),
        );
    }
}
