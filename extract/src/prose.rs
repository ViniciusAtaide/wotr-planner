//! Rules the guide states only in prose, encoded as data where the current types can hold them, and reported
//! where they can't.

use rules::{
    Class, FeatCategory, FeatKey, FeatSlot, FeatureKey, FeatureSelectionKey, MythicPath,
    PathSwitch, Prerequisite, SelectionOptionKey,
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
        "feature:rogue/advanced-talents",
    ),
    // Major hexes from 10th level and grand hexes from 18th become hex options.
    ("witch", "major-hexes", "hex", "feature:witch/major-hexes"),
    ("witch", "grand-hexes", "hex", "feature:witch/grand-hexes"),
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
        "selection:trickster/improved-mythic-trick/perception-ii",
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

/// What each path switch keeps: Legend keeps MR 1-2, ends ascensions and raises the cap to 40 (ticket 04).
/// Gold Dragon, Devil and Swarm That Walks begin at MR 8 and keep the general picks of MR 1-7.
pub fn path_switch(path: &str) -> Option<PathSwitch> {
    match path {
        "legend" => Some(PathSwitch {
            kept_ranks: [1, 2].into(),
            further_ascensions: false,
            level_cap: Some(40),
        }),
        "gold-dragon" | "devil" | "swarm-that-walks" => Some(PathSwitch {
            kept_ranks: (1..=7).collect(),
            further_ascensions: true,
            level_cap: None,
        }),
        _ => None,
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
        builder.report.add(
            Kind::BonusSlot,
            "gold-dragon",
            label,
            "three feats chosen without prerequisites; slots outside a class can't skip prerequisites, so \
             stored as three general slots",
        );
        return Some(vec![
            FeatSlot::General,
            FeatSlot::General,
            FeatSlot::General,
        ]);
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
        return Some(vec![FeatSlot::OneOf {
            feats,
            ignores_prerequisites: false,
        }]);
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
            .find(|class| class.key.as_str() == format!("class:{class_slug}"))
        else {
            continue;
        };
        let selection_key = FeatureSelectionKey::new(format!("selection:{class_slug}/{into}"));
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

    let notes: [(&str, &str); 6] = [
        (
            "barbarian#rage-powers",
            "totem rage powers are exclusive: one totem line per character. The data has no \"not\" \
             prerequisite, so only the Lesser, plain, Greater chain is checked",
        ),
        (
            "other-mythic-paths#devil",
            "the Devil switch is open only to an Aeon or Azata; a path switch has no prerequisite in the data",
        ),
        (
            "gold-dragon",
            "the switch drops the previous path's own powers from MR 3-7 but keeps the mythic abilities and \
             feats; the data keeps whole ranks, so MR 1-7 are kept in full",
        ),
        (
            "other-mythic-paths#swarm-that-walks",
            "the switch drops the previous path's own powers from MR 3-7 but keeps the mythic abilities and \
             feats; the data keeps whole ranks, so MR 1-7 are kept in full",
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
    let Some(trickster) = paths
        .iter()
        .find(|path| path.key.as_str() == "path:trickster")
    else {
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
            .find(|selection| selection.key.as_str() == format!("selection:trickster/{anchor}"))
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
                    key.as_str(),
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
