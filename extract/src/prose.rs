//! Rules the guide states only in prose, encoded as data where the current types can hold them, and reported
//! where they can't.

use std::collections::BTreeSet;

use rules::{
    Ability, AbilityBonus, Class, DataAt, FeatCategory, FeatKey, FeatList, FeatListKey, FeatSlot,
    Feature, FeatureKey, FeatureSelectionKey, GuideRef, MythicPath, MythicPathKey, PathSwitch,
    Prerequisite, SelectionOptionKey,
};

use crate::build::{Builder, Scope, section_text};
use crate::html::Section;
use crate::names::norm;
use crate::report::Kind;

/// Option lists that a class's prose folds into another selection once a feature unlocks them:
/// (class, the list's anchor, the selection it joins, the unlocking feature).
const MERGES: [(&str, &str, &str, &str); 3] = [
    // "These unlock at 10th level": advanced talents become rogue talent options from Advanced Talents on.
    (
        "rogue",
        "advanced-rogue-talents",
        "rogue-talents",
        "rogue/advanced-talents",
    ),
    // Major hexes from 10th level and grand hexes from 18th become hex options.
    ("witch", "major-hexes", "hex", "witch/major-hexes"),
    ("witch", "grand-hexes", "hex", "witch/grand-hexes"),
];

/// Whether a heading's options are folded into another selection rather than being one of their own.
pub fn is_merged_list(prefix: &str, anchor: &str) -> bool {
    MERGES
        .iter()
        .any(|(class, list, _, _)| *class == prefix && *list == anchor)
}

/// The Trickster bonus feats open at the Perception II trick.
pub fn trickster_feat_gate() -> Prerequisite {
    Prerequisite::SelectionOption(SelectionOptionKey::new(
        "trickster/improved-mythic-trick/perception-ii",
    ))
}

/// The Trickster bonus feats sit under their own heading, not a feat category: each gets the category of the
/// feats it extends.
pub fn trickster_feat_category(name: &str) -> FeatCategory {
    if name.contains("Critical") {
        FeatCategory::Weapon
    } else if name.starts_with("Metamagic") {
        FeatCategory::Magic
    } else {
        FeatCategory::Miscellaneous
    }
}

/// What each path switch keeps, drops and grants, from the guide's prose (`other-mythic-paths`, `gold-dragon`).
/// The general picks (mythic abilities and feats) and the old path's own grants each have their cut-off.
pub fn path_switch(path: &str) -> Option<PathSwitch> {
    let keys = |items: &[&str]| {
        items
            .iter()
            .map(|key| (*key).to_owned())
            .collect::<Vec<_>>()
    };
    let switch = |kept, path_kept, further_ascensions, level_cap| PathSwitch {
        from: BTreeSet::new(),
        kept_through_rank: kept,
        path_kept_through_rank: path_kept,
        dropped_features: BTreeSet::new(),
        dropped_selections: BTreeSet::new(),
        features: BTreeSet::new(),
        further_ascensions,
        level_cap,
    };
    match path {
        // "you lose ... All rank 3 - 8 mythic abilities and feats ... your rank is stuck at 3"; "Your ability
        // scores are increased by 4. Your level cap is 40".
        "legend" => Some(PathSwitch {
            features: keys(&["legend/legend", "legend/legends-corruption-resistance"])
                .into_iter()
                .map(FeatureKey::new)
                .collect(),
            ..switch(2, 2, false, Some(40))
        }),
        // "You lose any powers you gained at ranks 3 - 7, retaining only your mythic abilities and feats."
        "gold-dragon" | "swarm-that-walks" => Some(switch(7, 2, true, None)),
        // "You must have followed either the Aeon or Azata paths previously. If you were following the Azata
        // path, you lose your dragon companion. If you were following the Aeon path, you will lose your gaze
        // abilities."
        "devil" => Some(PathSwitch {
            from: keys(&["aeon", "azata"])
                .into_iter()
                .map(MythicPathKey::new)
                .collect(),
            dropped_features: keys(&["aeon/enforcing-gaze-extra-gaze", "azata/dragon-companion"])
                .into_iter()
                .map(FeatureKey::new)
                .collect(),
            dropped_selections: [FeatureSelectionKey::new("aeon/enforcing-gaze")].into(),
            ..switch(7, 7, true, None)
        }),
        _ => None,
    }
}

/// Features with ability bonuses, as the guide states them (`swarm size` starts at 0, so the Swarm's are +4).
const ABILITY_BONUSES: [(&str, &[AbilityBonus]); 6] = [
    // "Your ability scores are increased by 4."
    ("legend/legend", &[AbilityBonus::Every { amount: 4 }]),
    // "All of your attributes are at least 14 and your highest ability score gets a +4 increase."
    (
        "gold-dragon/perfect-body",
        &[
            AbilityBonus::Highest { amount: 4 },
            AbilityBonus::AtLeast { score: 14 },
        ],
    ),
    // "All of your attributes are at least 18 and your highest ability score gets an additional +2 increase."
    (
        "gold-dragon/perfect-mind",
        &[
            AbilityBonus::Highest { amount: 2 },
            AbilityBonus::AtLeast { score: 18 },
        ],
    ),
    // "your highest ability score gets an additional +2 bonus (for a total of +8)"
    (
        "gold-dragon/perfect-soul",
        &[AbilityBonus::Highest { amount: 2 }],
    ),
    // "Your Strength and Dexterity ability scores increase by 4 + your swarm size."
    (
        "swarm-that-walks/eye-of-the-swarm",
        &[
            AbilityBonus::Fixed {
                ability: Ability::Strength,
                amount: 4,
            },
            AbilityBonus::Fixed {
                ability: Ability::Dexterity,
                amount: 4,
            },
        ],
    ),
    // "your Intelligence, Wisdom and Charisma ability scores increase by 4 + your swarm size."
    (
        "swarm-that-walks/the-swarm",
        &[
            AbilityBonus::Fixed {
                ability: Ability::Intelligence,
                amount: 4,
            },
            AbilityBonus::Fixed {
                ability: Ability::Wisdom,
                amount: 4,
            },
            AbilityBonus::Fixed {
                ability: Ability::Charisma,
                amount: 4,
            },
        ],
    ),
];

/// "You also become proficient with all types of weapons and armour" (`azata#all-skilled`). Barding is the
/// companion's ("Your companion becomes proficient with ... Barding"), so it's left out.
const ALL_SKILLED: [&str; 8] = [
    "armor-proficiency-heavy-armor",
    "armor-proficiency-light-armor",
    "armor-proficiency-medium-armor",
    "exotic-weapon-proficiency",
    "martial-weapons-proficiency",
    "shield-proficiency",
    "simple-weapon-proficiency",
    "tower-shield-proficiency",
];

const DRAGON_FEATS: &str = "gold-dragon/dragon-feats";

/// "At 9th rank, you can select any three feats without having to meet the prerequisites": three slots on the
/// Gold Dragon's own list of every feat.
pub fn dragon_feats_slots() -> Vec<FeatSlot> {
    vec![
        FeatSlot::OneOf {
            list: FeatListKey::new(DRAGON_FEATS),
        };
        3
    ]
}

/// The path rules on the built paths: Legend's two features (the switch grants them; the path has no rank
/// table), the features' ability bonuses, All-skilled's feats and the Gold Dragon's Dragon Feats list.
pub fn path_rules(paths: &mut [MythicPath]) {
    for path in paths.iter_mut() {
        match path.key.as_str() {
            "legend" => {
                let feature = |key: &str, name: &str, anchor: &str| Feature {
                    key: FeatureKey::new(key),
                    name: name.to_owned(),
                    summary: String::new(),
                    guide: GuideRef::new(format!("other-mythic-paths#{anchor}")),
                    ability_bonuses: Vec::new(),
                };
                path.features
                    .push(feature("legend/legend", "Legend", "legend"));
                path.features.push(feature(
                    "legend/legends-corruption-resistance",
                    "Legend's Corruption Resistance",
                    "legends-corruption-resistance",
                ));
            }
            "gold-dragon" => path.feat_lists.push(FeatList {
                key: FeatListKey::new(DRAGON_FEATS),
                name: "Dragon Feats".to_owned(),
                guide: GuideRef::new("gold-dragon#dragon-feats"),
                ignores_prerequisites: true,
                every_feat: true,
                feats: BTreeSet::new(),
            }),
            _ => {}
        }
        for feature in &mut path.features {
            if let Some((_, bonuses)) = ABILITY_BONUSES
                .iter()
                .find(|(key, _)| *key == feature.key.as_str())
            {
                feature.ability_bonuses = bonuses.to_vec();
            }
        }
        for option in path.selections.iter_mut().flat_map(|s| &mut s.options) {
            if option.key.as_str() == "azata/superpowers/all-skilled" {
                option.feats = ALL_SKILLED.iter().map(|key| FeatKey::new(*key)).collect();
            }
        }
    }
}

/// A path table's cell as the heading that describes it is named: five paths print "Bypass Epic DR" for the
/// `mythic-paths` page's Bypass Epic Damage Reduction.
pub fn path_cell(cell: &str) -> &str {
    match cell {
        "Bypass Epic DR" => "Bypass Epic Damage Reduction",
        other => other,
    }
}

/// A name for matching across the guide's spellings ("Eldritich Scoundrel").
pub fn fix_name(name: &str) -> String {
    norm(name).replace("eldritich", "eldritch")
}

/// Which spellcasting a prestige class's `CL` column advances: what its table and requirements name.
pub fn prestige_advances(section: &Section) -> (bool, bool) {
    let text = section_text(section).to_ascii_lowercase();
    let arcane = text.contains("arcane spellcasting") || text.contains("arcane spells");
    let divine = text.contains("divine spellcasting") || text.contains("divine spells");
    if !arcane && !divine {
        // Winter Witch's entry is a witch, druid or shaman: either kind.
        return (true, true);
    }
    (arcane, divine)
}

/// Bonus slots whose list the generic rule can't find.
pub fn special_bonus_slots(
    builder: &mut Builder,
    scope: &Scope,
    label: &str,
    _level: u8,
) -> Option<Vec<FeatSlot>> {
    let label_norm = norm(label);
    if label_norm == "dragon-feats" {
        return Some(dragon_feats_slots());
    }
    if label_norm == "bloodline-feat" {
        let feats = bloodline_feats(builder);
        builder.report.add(
            Kind::BonusSlot,
            "character-creation#bloodlines",
            label,
            "each bloodline has its own list; stored as one list of every bloodline's feats, checking \
             prerequisites",
        );
        let _ = scope;
        return Some(vec![builder.lists.slot(feats, false)]);
    }
    None
}

/// Every feat on a bloodline's bonus list: the bloodrager's "Bonus Feats (…)" entries and the sorcerer
/// bloodlines' "Bonus Feats:" bullets.
fn bloodline_feats(builder: &Builder) -> std::collections::BTreeSet<FeatKey> {
    let mut feats = std::collections::BTreeSet::new();
    let bloodrager = builder.guide.page("bloodrager");
    for section in bloodrager.root.walk() {
        if section.name.starts_with("Bonus Feats (") {
            feats.extend(
                builder
                    .feats_in(&section.text())
                    .into_iter()
                    .map(|(_, f)| f),
            );
        }
    }
    let creation = builder.guide.page("character-creation");
    if let Some(bloodlines) = creation.root.find("bloodlines") {
        for section in bloodlines.walk() {
            for item in section.list_items() {
                if let Some(list) = item.strip_prefix("Bonus Feats:") {
                    feats.extend(builder.feats_in(list).into_iter().map(|(_, f)| f));
                }
            }
        }
    }
    feats
}

/// Applies the prose rules after every class and path is built.
pub fn apply(
    builder: &mut Builder,
    classes: &mut [Class],
    _prestige: &mut [Class],
    paths: &mut [MythicPath],
) {
    for (class_slug, list, into, gate) in MERGES {
        let page = builder.guide.page(class_slug);
        let Some(heading) = page.root.find(list) else {
            builder.report.add(
                Kind::ProseRule,
                page.guide(list),
                "",
                "the list to fold in is missing",
            );
            continue;
        };
        let gate = FeatureKey::new(gate);
        if !builder.features.contains_key(&gate) {
            builder.report.add(
                Kind::ProseRule,
                page.guide(list),
                "",
                format!("the unlocking feature {gate} isn't granted by the class table"),
            );
            continue;
        }
        let Some(class) = classes
            .iter_mut()
            .find(|class| class.key.as_str() == class_slug)
        else {
            continue;
        };
        let selection_key = FeatureSelectionKey::new(format!("{class_slug}/{into}"));
        let class_key = class.key.clone();
        let Some(selection) = class
            .selections
            .iter_mut()
            .find(|selection| selection.key == selection_key)
        else {
            builder.report.add(
                Kind::ProseRule,
                page.guide(into),
                "",
                format!("{selection_key} isn't a selection, so {list} can't join it"),
            );
            continue;
        };
        builder.add_options(
            page,
            heading,
            selection,
            Some(&class_key),
            vec![Prerequisite::Feature {
                feature: gate,
                count: 1,
            }],
        );
    }

    trickster_tiers(builder, paths);

    let notes: [(&str, &str); 3] = [
        (
            "barbarian#rage-powers",
            "totem rage powers are exclusive: one totem line per character. The data has no \"not\" \
             prerequisite, so only the Lesser, plain, Greater chain is checked",
        ),
        (
            "slayer#advanced-talents",
            "from 10th level slayer talents may be advanced rogue talents; the slayer talent list is prose, \
             not option headings, so the selection isn't built",
        ),
        (
            "kineticist#kineticist-bonus-feat",
            "a wild talent that grants a bonus feat slot; a selection option can't grant a slot",
        ),
    ];
    for (at, why) in notes {
        builder.report.add(Kind::ProseRule, at, "", why);
    }
}

/// Trickster tiers: an Improved trick needs the plain one for the same skill, a Greater trick the Improved one.
fn trickster_tiers(builder: &mut Builder, paths: &[MythicPath]) {
    let Some(trickster) = paths.iter().find(|path| path.key.as_str() == "trickster") else {
        return;
    };
    let tiers = [
        "mythic-trick",
        "improved-mythic-trick",
        "greater-mythic-trick",
    ];
    let options = |anchor: &str| -> Vec<(String, SelectionOptionKey, String)> {
        trickster
            .selections
            .iter()
            .find(|selection| selection.key.as_str() == format!("trickster/{anchor}"))
            .map(|selection| {
                selection
                    .options
                    .iter()
                    .map(|option| {
                        (
                            trick_skill(&option.name),
                            option.key.clone(),
                            option.guide.0.clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    for pair in tiers.windows(2) {
        let lower = options(pair[0]);
        for (skill, key, guide) in options(pair[1]) {
            match lower
                .iter()
                .find(|(lower_skill, _, _)| *lower_skill == skill)
            {
                Some((_, previous, _)) => builder.add_prerequisite(
                    &DataAt::SelectionOption(key.clone()),
                    guide,
                    vec![Prerequisite::SelectionOption(previous.clone())],
                ),
                None => builder.report.add(
                    Kind::ProseRule,
                    guide,
                    "",
                    "no lower-tier trick for the same skill found",
                ),
            }
        }
    }
}

/// The skill a mythic trick belongs to: "Athletics II" is Athletics; the Use Magic Device tricks are named
/// Infuse, Reuse and Use.
fn trick_skill(name: &str) -> String {
    let name = norm(name).replace("knowlege", "knowledge");
    let base = name
        .trim_end_matches("-iii")
        .trim_end_matches("-ii")
        .trim_end_matches("-i")
        .to_owned();
    match base.as_str() {
        "infuse-magic-device" | "reuse-magic-device" | "use-magic-device" => {
            "use-magic-device".to_owned()
        }
        _ => base,
    }
}
