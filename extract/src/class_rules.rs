//! Class rules from the hand-fix follow-ups (ticket 28), stated in the guide's prose rather than its tables:
//! prestige classes that advance one class, arcane or divine; levels that count as another class's; the druid's
//! choice of an animal companion or a domain (Nature Bond); the note on the Crusader's bonus lists. The tables quote
//! the guide and are the same as ticket 28's one-off rewrite of `game-data/`.

use std::collections::BTreeSet;

use rules::{
    Advancement, Class, ClassKey, CountsAs, CountsToward, FeatureKey, FeatureSelection,
    FeatureSelectionKey, GameData, GuideRef, Prerequisite, Repeat, SelectionOption,
    SelectionOptionKey,
};

/// "Advances one class, arcane or divine": each of these gains spells "as though she had also gained a level in her
/// previous class" whichever kind that class casts (prestige-classes#hellknight-signifer, #loremaster,
/// #winter-witch). Their rows advance both spell types.
const EITHER: [&str; 3] = ["hellknight-signifer", "loremaster", "winter-witch"];

/// "Counts as" facts: (class, the class it counts as, the feature it counts toward; none: feat prerequisites).
/// - Diverse Training: "Eldritch Knight levels count as Fighter levels for the purposes of unlocking feats." (Its
///   caster-level half names no class, and no feat checks a caster level.)
/// - Frost Power: "Winter Witch levels stack ... with Witch levels to determine the power of Witch hexes, access to
///   major and grand hexes". (With shaman levels for power only: nothing a Build holds.)
const COUNTS_AS: [(&str, &str, Option<&str>); 3] = [
    ("eldritch-knight", "fighter", None),
    ("winter-witch", "witch", Some("witch/major-hexes")),
    ("winter-witch", "witch", Some("witch/grand-hexes")),
];

/// The note above the Crusader's bonus lists: "The Crusader does not need to meet the BAB or Fighter-level
/// prerequisites for any of these", but a list skips all or none (user, ticket 21 R5: keep skipping all).
pub const CRUSADER_NOTE: &str = "// Crusader Bonus Feats: the guide skips only their BAB and fighter-level \
prerequisites; they skip every prerequisite here (user, ticket 21 R5).";

/// The comment the files write above an owner's feat lists, if any.
pub fn feat_lists_note(owner: &str) -> Option<&'static str> {
    (owner == "cleric/crusader").then_some(CRUSADER_NOTE)
}

/// Every rule, on full rows.
pub fn apply(data: &mut GameData) {
    apply_prestige(&mut data.prestige_classes);
    nature_bond(data);
}

/// Advancement and "counts as", each on the prestige class it names.
pub fn apply_prestige(classes: &mut [Class]) {
    fn class<'c>(classes: &'c mut [Class], key: &str) -> &'c mut Class {
        classes
            .iter_mut()
            .find(|class| class.key.as_str() == key)
            .unwrap_or_else(|| panic!("no prestige class {key}"))
    }
    for key in EITHER {
        let class = class(classes, key);
        assert!(
            class
                .levels
                .iter()
                .all(|row| row.advances_arcane && row.advances_divine),
            "{key} advances both spell types at every level"
        );
        class.advances = Advancement::Either;
    }
    for (key, as_class, feature) in COUNTS_AS {
        class(classes, key).counts_as.insert(CountsAs {
            class: ClassKey::new(as_class),
            toward: feature.map_or(CountsToward::FeatPrerequisites, |feature| {
                CountsToward::Feature(FeatureKey::new(feature))
            }),
        });
    }
}

/// The druid: "Gains an Animal Companion or may choose a domain instead." The level 1 rows that grant both (the
/// druid's and those of the archetypes whose table keeps both columns: Defender of the True World, Feyspeaker) pick
/// one instead; the domain feature, granted nowhere else, goes. The companion at 4 stays: the Animal domain's
/// (ticket 31). A companion option counts as a companion wherever a companion feature does.
pub fn nature_bond(data: &mut GameData) {
    const OWNERS: [&str; 3] = [
        "druid",
        "druid/defender-of-the-true-world",
        "druid/feyspeaker",
    ];
    let selection = FeatureSelectionKey::new("druid/nature-bond");
    let druid = data
        .classes
        .iter_mut()
        .find(|class| class.key.as_str() == "druid")
        .expect("the druid");
    let mut changed = BTreeSet::new();
    let mut choose = |features: &mut Vec<FeatureKey>, selections: &mut Vec<FeatureSelectionKey>| {
        for owner in OWNERS {
            let companion = FeatureKey::new(format!("{owner}/animal-companion"));
            let domain = FeatureKey::new(format!("{owner}/domain-abilities"));
            if features.contains(&companion) && features.contains(&domain) {
                features.retain(|feature| feature != &companion && feature != &domain);
                selections.push(selection.clone());
                changed.insert(owner);
            }
        }
    };
    if let Some(first) = druid.levels.iter_mut().find(|row| row.level == 1) {
        choose(&mut first.features, &mut first.selections);
    }
    for archetype in &mut druid.archetypes {
        if let Some(first) = archetype.levels.iter_mut().find(|row| row.level == 1) {
            choose(&mut first.features, &mut first.selections);
        }
    }
    assert_eq!(changed, BTreeSet::from(OWNERS), "the rows granting both");

    let domains: BTreeSet<FeatureKey> = OWNERS
        .iter()
        .map(|owner| FeatureKey::new(format!("{owner}/domain-abilities")))
        .collect();
    let granted = druid
        .levels
        .iter()
        .flat_map(|row| &row.features)
        .chain(
            druid
                .archetypes
                .iter()
                .flat_map(|a| a.levels.iter().flat_map(|row| &row.features)),
        )
        .any(|feature| domains.contains(feature));
    assert!(!granted, "a domain feature granted past level 1");
    druid.features.retain(|f| !domains.contains(&f.key));
    for archetype in &mut druid.archetypes {
        archetype.features.retain(|f| !domains.contains(&f.key));
    }

    let option = |key: &str, name: &str| SelectionOption {
        key: SelectionOptionKey::new(format!("druid/nature-bond/{key}")),
        name: name.into(),
        summary: String::new(),
        guide: GuideRef::new("druid"),
        feats: BTreeSet::new(),
        chosen_feat: None,
        repeat: Repeat::Once,
        prerequisite: None,
    };
    druid.selections.push(FeatureSelection {
        key: selection,
        name: "Nature Bond".into(),
        summary: String::new(),
        guide: GuideRef::new("druid"),
        options: vec![
            option("animal-companion", "Animal Companion"),
            option("domain", "Domain"),
        ],
        shared_options: BTreeSet::new(),
    });

    // Boon Companion and Mythical Beast: "Animal Companion".
    let companion = Prerequisite::SelectionOption(SelectionOptionKey::new(
        "druid/nature-bond/animal-companion",
    ));
    let druid_companion = Prerequisite::Feature {
        feature: FeatureKey::new("druid/animal-companion"),
        count: 1,
    };
    let add = |prerequisite: &mut Option<Prerequisite>, owner: &str| {
        let Some(Prerequisite::Any(children)) = prerequisite else {
            panic!("{owner} needs any of the companions");
        };
        assert!(children.contains(&druid_companion), "{owner}");
        children.push(companion.clone());
    };
    let feat = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "boon-companion")
        .expect("Boon Companion");
    add(&mut feat.prerequisite, "Boon Companion");
    let ability = data
        .mythic_abilities
        .iter_mut()
        .find(|ability| ability.key.as_str() == "mythical-beast")
        .expect("Mythical Beast");
    add(&mut ability.prerequisite, "Mythical Beast");
}
