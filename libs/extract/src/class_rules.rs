//! Class rules from the hand-fix follow-ups (ticket 28), stated in the guide's prose rather than its tables:
//! prestige classes that advance one class, arcane or divine; levels that count as another class's; the druid's
//! choice of an animal companion or a domain (Nature Bond, built with the class: [`provide`], through
//! [`crate::picks`], and [`check`]); the note on the Crusader's bonus lists. The tables quote the guide and are the
//! same as ticket 28's one-off rewrite of `libs/game-data/`.

use std::collections::BTreeSet;

use rules::{
    Advancement, Class, ClassKey, CountsAs, CountsToward, DataAt, FeatKey, FeatureKey,
    FeatureSelection, FeatureSelectionKey, GameData, GuideRef, MythicAbilityKey, OptionLevelRow,
    Prerequisite, Repeat, SelectionOption, SelectionOptionKey,
};

use crate::picks::{Pick, Picks, Rule};

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

/// Advancement and "counts as", each on the prestige class it names, where the prestige classes are built.
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

/// The tables whose level-1 row grants both the companion and the domain: the druid's and those of the archetypes
/// whose table keeps both columns (Defender of the True World, Feyspeaker).
const NATURE_BOND_TABLES: [&str; 3] = [
    "druid",
    "druid/defender-of-the-true-world",
    "druid/feyspeaker",
];

pub const NATURE_BOND: &str = "druid/nature-bond";
const BOND_COMPANION: &str = "druid/nature-bond/animal-companion";
const BOND_DOMAIN: &str = "druid/nature-bond/domain";

/// The druid: "Gains an Animal Companion or may choose a domain instead." Built with the class: in the level-1 rows
/// that grant both, the domain cell is a pick of the Nature Bond and the companion cell is its companion option's
/// (so neither feature is made there); the companion at 4 is the Animal domain's (ticket 31). The domain option
/// grants the druid's domain pick, levelled by druid levels. Boon Companion's and Mythical Beast's "Animal Companion"
/// also accepts the companion option.
pub fn provide(picks: &mut Picks) {
    for table in NATURE_BOND_TABLES {
        picks.rule(Rule {
            table: Some(table.to_owned()),
            levels: Some(vec![1]),
            ..Rule::every(
                &format!("{table}/domain-abilities"),
                Pick::Selection(FeatureSelectionKey::new(NATURE_BOND)),
            )
        });
        picks.rule(Rule {
            table: Some(table.to_owned()),
            levels: Some(vec![1]),
            ..Rule::every(&format!("{table}/animal-companion"), Pick::OptionRow)
        });
    }
    let option = |key: &str, name: &str, levels: Vec<OptionLevelRow>| SelectionOption {
        key: SelectionOptionKey::new(key),
        name: name.into(),
        summary: String::new(),
        guide: GuideRef::new("druid"),
        feats: BTreeSet::new(),
        chosen_feat: None,
        repeat: Repeat::Once,
        prerequisite: None,
        levels,
        pool: None,
        replaces_row_of: None,
        group: None,
        arcana: None,
        parameter: None,
        values: Default::default(),
    };
    picks.selection(
        "druid",
        FeatureSelection {
            key: FeatureSelectionKey::new(NATURE_BOND),
            name: "Nature Bond".into(),
            summary: String::new(),
            guide: GuideRef::new("druid"),
            options: vec![
                option(BOND_COMPANION, "Animal Companion", Vec::new()),
                option(
                    BOND_DOMAIN,
                    "Domain",
                    vec![OptionLevelRow {
                        level: 1,
                        selections: vec![FeatureSelectionKey::new("druid/domain")],
                        ..OptionLevelRow::default()
                    }],
                ),
            ],
            shared_options: BTreeSet::new(),
            option_levels: BTreeSet::from([ClassKey::new("druid")]),
            skipped_levels: BTreeSet::new(),
            feat_lists_only: false,
            only_without_option: false,
            pool_of: None,
            levels_behind: 0,
            level_from: None,
            once_per_option: false,
            level_also_from: None,
        },
    );
    let companion = Prerequisite::SelectionOption(SelectionOptionKey::new(BOND_COMPANION));
    picks.alternatives(
        DataAt::Feat(FeatKey::new("boon-companion")),
        "Animal Companion",
        vec![companion.clone()],
    );
    picks.alternatives(
        DataAt::MythicAbility(MythicAbilityKey::new("mythical-beast")),
        "Animal Companion",
        vec![companion],
    );
}

/// The Nature Bond is the level-1 pick of each table that grants both, and no druid table's domain feature is
/// made. Changes nothing.
pub fn check(data: &GameData) {
    let druid = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "druid")
        .expect("the druid");
    let bond = FeatureSelectionKey::new(NATURE_BOND);
    for table in NATURE_BOND_TABLES {
        let first = if table == "druid" {
            druid
                .levels
                .iter()
                .find(|row| row.level == 1)
                .map(|row| &row.selections)
        } else {
            druid
                .archetypes
                .iter()
                .find(|a| a.key.as_str() == table)
                .and_then(|a| a.levels.iter().find(|row| row.level == 1))
                .map(|row| &row.selections)
        };
        assert_eq!(
            first.map(|selections| selections.iter().filter(|s| **s == bond).count()),
            Some(1),
            "{table}: the Nature Bond at 1"
        );
    }
    let defined = druid
        .features
        .iter()
        .chain(druid.archetypes.iter().flat_map(|a| &a.features));
    for feature in defined {
        assert!(
            !feature.key.as_str().ends_with("/domain-abilities"),
            "{}",
            feature.key
        );
    }
}
