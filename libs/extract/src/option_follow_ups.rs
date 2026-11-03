//! Ticket 33: the user's decisions on tickets 30 and 31's open questions about options (2026-11-01, "all
//! recommended"), and the user's fact about Impossible Domain, applied to finished data.
//!
//! - **Option groups** (30-1): the sorcerer's ten Draconic bloodlines and four Elemental ones each form a group,
//!   and each needs "no other option of this group", so a Crossblooded sorcerer or a Second Bloodline can't take a
//!   second dragon (or element).
//! - **Crossblooded** (30-2): every bloodline names its arcana, which the second bloodline gives besides its feats;
//!   the archetype's "Bloodline 2 Arcana" placeholder goes.
//! - **Daily choices** (30-10): the Unsworn Shaman's Minor Spirit and Wandering Spirit 2; the hunter's Animal Focus
//!   (a feature now, not a pick of eight aspects) and its Second Animal Focus.
//! - **Elemental Movement** (30-11): one per element (Air, Fire, Water), none for Earth.
//! - **Separatist** (31-1): its second domain, two levels behind (`levels_behind`); the second domain's cells are its
//!   rows: built with the domains ([`option_families`](crate::option_families)' provider).
//! - **Divine Hunter** (31-2): every domain power, by hunter level (no skipped levels): built with the domains.
//! - **Impossible Domain** (31-5, the user's fact): a further domain from the full list, levelled like the domains
//!   held (`LevelFrom::OtherOptions`); a Living Deity may take it.
//! - **Expanded Element** (31-6): the 15th-level element isn't the 7th-level one (`once_per_option`): built with
//!   the elements ([`option_families`](crate::option_families)' provider).
//! - **Revelations** (31-7): Cinder Dance needs no Lame curse; the Dual-Cursed Oracle's revelations offer
//!   Misfortune and Fortune; Maneuver Mastery's maneuver follows its revelation's level and Second Mystery the first
//!   mystery's, so a Living Deity's count arcanist levels: built with the mysteries
//!   ([`option_families`](crate::option_families)' provider).
//! - **Champion of the Faith** (31-8): its alignment blessing is one of his own alignment (`PickedThrough`).
//! - **Signifer** (31-10): its order is a selection sharing the Hellknight's seven: built with the Hellknight's order
//!   ([`option_families`](crate::option_families)' provider).
//! - **The slayer's combat style** (32-2, a scope addition): the slayer picks one of the ranger's combat styles
//!   with his first combat style feat, and each of the three feats comes from that style's list of its stage.
//!
//! Each rule asserts the guide phrase it rests on (the user's decisions and fact aside) and what it finds in the
//! data, so it fails loudly on data it doesn't expect. [`apply`] runs on the finished data of an extraction and,
//! through ticket 33's one-off rewrite, on `libs/game-data/`.

use std::collections::BTreeSet;

use rules::{
    Alignment, FeatSlot, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData, LevelFrom,
    MythicAbilityKey, OptionGroup, Prerequisite, SelectionOptionKey,
};

use crate::follow_ups::{archetype_mut, class_mut, drop_feature, says, take_feature};
use crate::guide::Guide;
use crate::option_levels::{Log, feature, selection};
use crate::picks::Picks;

/// Applies every rule, in the order above.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    option_groups(guide, data, &mut log);
    crossblooded(guide, data, &mut log);
    daily_choices(guide, data, &mut log);
    elemental_movement(guide, data, &mut log);
    impossible_domain(guide, data, &mut log);
    champion_of_the_faith(guide, data, &mut log);
    slayer_combat_style(guide, data, &mut log);
    normalise(data, &mut log);
    features_named_exist(data);
    log
}

/// Every feature a prerequisite names is still defined (a removed placeholder may have been named).
fn features_named_exist(data: &mut GameData) {
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
    for prerequisite in crate::references::prerequisites_mut(data)
        .into_iter()
        .flatten()
    {
        named(prerequisite, &mut features);
    }
    for feature in features {
        assert!(
            defined.contains(&feature),
            "a prerequisite names {feature}, which is gone"
        );
    }
}

/// The classes whose archetype rows the rules edit.
const TOUCHED: [&str; 2] = ["cleric", "oracle"];

pub(crate) fn class_selection<'d>(
    data: &'d mut GameData,
    class: &str,
    key: &str,
) -> &'d mut FeatureSelection {
    let class = class_mut(data, class);
    class
        .selections
        .iter_mut()
        .chain(class.archetypes.iter_mut().flat_map(|a| &mut a.selections))
        .find(|selection| selection.key.as_str() == key)
        .unwrap_or_else(|| panic!("no selection {key}"))
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

/// Every option `selection` defines or shares, as a set of keys (for another selection to share them all).
fn offered(data: &mut GameData, class: &str, key: &str) -> BTreeSet<SelectionOptionKey> {
    class_selection(data, class, key)
        .offered()
        .into_iter()
        .cloned()
        .collect()
}

// ---- option groups and arcanas ---------------------------------------------------------------------------------

fn option_groups(guide: &Guide, data: &mut GameData, log: &mut Log) {
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
    let bloodline = class_selection(data, "sorcerer", "sorcerer/bloodline");
    for (prefix, group, count) in [
        ("sorcerer/bloodline/draconic-", OptionGroup::Draconic, 10),
        ("sorcerer/bloodline/elemental-", OptionGroup::Elemental, 4),
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

/// "Gains the arcana and spells from both bloodlines": every bloodline's arcana is the feature of its first row
/// whose name ends in "Arcana".
fn crossblooded(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "sorcerer",
        "crossblooded",
        "Gains the arcana and spells from both bloodlines.",
    );
    let sorcerer = class_mut(data, "sorcerer");
    let names: std::collections::HashMap<FeatureKey, String> = sorcerer
        .features
        .iter()
        .map(|feature| (feature.key.clone(), feature.name.clone()))
        .collect();
    let bloodline = sorcerer
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == "sorcerer/bloodline")
        .expect("the sorcerer's bloodline");
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
        log.push(format!("{}: arcana {arcana}", option.key));
    }
    let crossblooded = archetype_mut(data, "sorcerer", "sorcerer/crossblooded");
    let first = crossblooded
        .levels
        .iter_mut()
        .find(|row| row.level == 1)
        .expect("the Crossblooded's first row");
    take_feature(
        &mut first.features,
        "sorcerer/crossblooded/bloodline-2-arcana",
        "crossblooded 1",
    );
    drop_feature(
        data,
        "sorcerer",
        Some("sorcerer/crossblooded"),
        "sorcerer/crossblooded/bloodline-2-arcana",
        log,
    );
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
}

/// The hunter's Animal Focus: a daily choice of an aspect, so a feature rather than a pick of eight. Kept a pass on
/// the finished rows: the guide's "Animal Focus" heading lists the eight aspects, so the class stage reads it as a
/// selection, and the cells naming it as its slots.
fn daily_choices(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "hunter",
        "animal-focus",
        "A Hunter can grant an animal aspect to her companion.",
    );
    let key = "hunter/animal-focus";
    let hunter = class_mut(data, "hunter");
    let before = hunter.selections.len();
    hunter
        .selections
        .retain(|selection| selection.key.as_str() != key);
    assert_eq!(hunter.selections.len(), before - 1, "no selection {key}");
    let mut grants = 0;
    let rows = hunter
        .levels
        .iter_mut()
        .map(|row| (&mut row.selections, &mut row.features))
        .chain(
            hunter
                .archetypes
                .iter_mut()
                .flat_map(|archetype| &mut archetype.levels)
                .map(|row| (&mut row.selections, &mut row.features)),
        );
    for (selections, features) in rows {
        if let Some(at) = selections.iter().position(|s| s.as_str() == key) {
            selections.remove(at);
            features.push(FeatureKey::new(key));
            features.sort();
            grants += 1;
        }
    }
    assert_eq!(
        grants, 5,
        "the hunter's and its four archetypes' Animal Focus"
    );
    hunter.features.push(rules::Feature {
        daily_choice: true,
        ..feature(
            &FeatureKey::new(key),
            "Animal Focus",
            "hunter#animal-focus".into(),
        )
    });
    log.push(format!(
        "{key}: a daily-choice feature instead of a selection of 8 aspects ({grants} rows)"
    ));
}

/// "Air: …, Earth: None, Fire: …, Water: …": the 15th-level row of each Elemental bloodline.
fn elemental_movement(guide: &Guide, data: &mut GameData, log: &mut Log) {
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
    let sorcerer = class_mut(data, "sorcerer");
    let bloodline = sorcerer
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == "sorcerer/bloodline")
        .expect("the sorcerer's bloodline");
    let mut new = Vec::new();
    for element in ["air", "earth", "fire", "water"] {
        let key = format!("sorcerer/bloodline/elemental-{element}");
        let option = bloodline
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
            log.push(format!("{key}: no Elemental Movement"));
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
            log.push(format!("{key}: {own}"));
        }
    }
    sorcerer.features.extend(new);
    sorcerer.features.sort_by(|a, b| a.key.cmp(&b.key));
    let before = sorcerer.features.len();
    sorcerer
        .features
        .retain(|feature| feature.key.as_str() != shared);
    assert_eq!(sorcerer.features.len(), before - 1, "no feature {shared}");
    log.push(format!("{shared}: feature removed"));
}

// ---- domains --------------------------------------------------------------------------------------------------

/// "You may select any domain" (the guide), levelled by the class that gives the character's other domains (the
/// user's fact): a selection of every domain levelled like the domains held. A Living Deity, whose way gives
/// domains, may take it too.
fn impossible_domain(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "mythic-paths",
        "impossible-domain",
        "You may select any domain.",
    );
    let domains = offered(data, "cleric", "cleric/domain");
    let key = "cleric/impossible-domain";
    class_mut(data, "cleric").selections.push(FeatureSelection {
        shared_options: domains,
        level_from: Some(LevelFrom::OtherOptions),
        ..selection(
            key,
            "Impossible Domain",
            "mythic-paths#impossible-domain".into(),
            &[],
        )
    });
    let ability = data
        .mythic_abilities
        .iter_mut()
        .find(|ability| ability.key == MythicAbilityKey::new("impossible-domain"))
        .expect("Impossible Domain");
    assert!(ability.selections.is_empty());
    ability.selections.push(FeatureSelectionKey::new(key));
    let living_deity = Prerequisite::SelectionOption(SelectionOptionKey::new(
        "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity",
    ));
    ability.prerequisite = Some(match ability.prerequisite.take() {
        Some(Prerequisite::Any(mut children)) => {
            children.push(living_deity);
            children.sort();
            Prerequisite::Any(children)
        }
        Some(other) => {
            let mut children = vec![other, living_deity];
            children.sort();
            Prerequisite::Any(children)
        }
        None => panic!("Impossible Domain has no prerequisite"),
    });
    log.push(format!(
        "impossible-domain: picks one of {key}, levelled like the domains held; a Living Deity may take it"
    ));
}

// ---- the Champion of the Faith ---------------------------------------------------------------------------------

/// "The chosen alignment must correspond to his own alignment": each alignment blessing, picked through the
/// Champion's Chosen Alignment, needs one of its alignments.
fn champion_of_the_faith(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "warpriest",
        "chosen-alignment",
        "The chosen alignment must correspond to his own alignment",
    );
    use Alignment::*;
    let chosen = FeatureSelectionKey::new("warpriest/champion-of-the-faith/chosen-alignment");
    let blessings = class_selection(data, "warpriest", "warpriest/blessings");
    for (blessing, alignments) in [
        ("chaos", [ChaoticGood, ChaoticNeutral, ChaoticEvil]),
        ("evil", [LawfulEvil, NeutralEvil, ChaoticEvil]),
        ("good", [LawfulGood, NeutralGood, ChaoticGood]),
        ("law", [LawfulGood, LawfulNeutral, LawfulEvil]),
    ] {
        let key = format!("warpriest/blessings/{blessing}-domain");
        let option = blessings
            .options
            .iter_mut()
            .find(|option| option.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"));
        assert_eq!(option.prerequisite, None, "{key}");
        let mut children: Vec<Prerequisite> = alignments.map(Prerequisite::Alignment).into();
        children.push(Prerequisite::Not(Box::new(Prerequisite::PickedThrough(
            chosen.clone(),
        ))));
        children.sort();
        option.prerequisite = Some(Prerequisite::Any(children));
        log.push(format!("{key}: the Champion's own alignment"));
    }
}

/// "Alternatively, he may choose from a Ranger's Combat Style Feats" (32-2, the user's decision: one style, as a
/// ranger picks): a selection sharing the ranger's five styles, levelled by slayer levels so each window's feat comes
/// from the style's list of its stage (2 or 4: the 2nd; 6 or 8: the 6th; 10+: the 10th), picked by whichever combat
/// style talent comes first (only by a character holding no style). Ticket 32's three lists of every style's feats go.
fn slayer_combat_style(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "slayer",
        "slayer-talents",
        "Alternatively, he may choose from a Ranger's Combat Style Feats.",
    );
    let styles = offered(data, "ranger", "ranger/combat-style-feat");
    assert_eq!(styles.len(), 5, "the ranger's five styles");
    let key = FeatureSelectionKey::new("slayer/combat-style");
    let slayer = class_mut(data, "slayer");
    slayer.selections.push(FeatureSelection {
        shared_options: styles,
        only_without_option: true,
        ..selection(
            key.as_str(),
            "Combat Style (Slayer)",
            "slayer#slayer-talents".into(),
            &["slayer"],
        )
    });
    let talents = slayer
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == "slayer/slayer-talents")
        .expect("the slayer's talents");
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
        let list = format!("slayer/combat-style-feats-{stage}");
        assert_eq!(
            row.feat_slots,
            [FeatSlot::OneOf {
                list: rules::FeatListKey::new(list.clone())
            }],
            "{talent}"
        );
        assert!(row.selections.is_empty(), "{talent}");
        row.feat_slots = vec![FeatSlot::OptionList {
            selection: key.clone(),
            at_level: None,
            label: None,
        }];
        row.selections = vec![key.clone()];
        log.push(format!(
            "{talent}: the style's pick (if none held) and a feat from its list"
        ));
        let before = slayer.feat_lists.len();
        slayer.feat_lists.retain(|l| l.key.as_str() != list);
        assert_eq!(slayer.feat_lists.len(), before - 1, "no list {list}");
        log.push(format!("{list}: list removed"));
    }
}

// ---- tidying ---------------------------------------------------------------------------------------------------

/// An archetype row the edits made the same as the class's row is the class's row.
fn normalise(data: &mut GameData, log: &mut Log) {
    for class in data.classes.iter_mut() {
        if !TOUCHED.contains(&class.key.as_str()) {
            continue;
        }
        let base = class.levels.clone();
        for archetype in &mut class.archetypes {
            let key = archetype.key.clone();
            archetype.levels.retain(|row| {
                let Some(base) = base.iter().find(|b| b.level == row.level) else {
                    return true;
                };
                let sorted = |mut items: Vec<String>| {
                    items.sort();
                    items
                };
                let features =
                    |items: &[FeatureKey]| sorted(items.iter().map(|k| k.to_string()).collect());
                let slots = |items: &[rules::FeatSlot]| {
                    sorted(items.iter().map(|s| format!("{s:?}")).collect())
                };
                let picks = |items: &[FeatureSelectionKey]| {
                    sorted(items.iter().map(|k| k.to_string()).collect())
                };
                let same = row
                    .spell_levels
                    .is_none_or(|spells| spells == base.spell_levels())
                    && features(&row.features) == features(&base.features)
                    && row.feats == base.feats
                    && row.feats_for == base.feats_for
                    && slots(&row.feat_slots) == slots(&base.feat_slots)
                    && picks(&row.selections) == picks(&base.selections);
                if same {
                    log.push(format!("{key}: row {} is the class's now", row.level));
                }
                !same
            });
        }
    }
}
