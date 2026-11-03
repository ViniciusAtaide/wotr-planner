//! Shared selection options, from the guide's prose (the tables can't say a list is another class's). A selection
//! the guide says draws on another's list shares those options instead of copying them: an option keeps the one
//! key of the selection that defines it, so a character holds it once whichever slot took it. Where an option's
//! prerequisite names the defining class's feature or level, the sharing class's stands in too, with an `Any`
//! (decision S4, option (a)). A pick the guide describes over two lists (Fey Tricks, the Underground Chemist's
//! Discovery) is its archetype's own selection, sharing both, and takes over the rogue-talent slots it describes.
//! Runs on full rows, before the archetype rows are made sparse.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArchetypeKey, ClassKey, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData, GuideRef,
    Prerequisite, SelectionOptionKey,
};

use crate::references::prerequisites_mut;

/// One selection that shares other selections' options.
#[derive(Clone, Copy, Debug)]
pub struct Share {
    pub selection: &'static str,
    /// Set when the selection is new: the guide describes a pick, not a list of its own.
    pub new: Option<NewSelection>,
    /// The selections it shares from, and which of their options.
    pub sources: &'static [(&'static str, Options)],
    /// A gate in a shared option's prerequisite (the defining class's), and the sharing class's that also opens it.
    pub gates: &'static [(Gate, Gate)],
    /// For a new selection: which of its archetype's rows it fills.
    pub slots: Option<SlotRule>,
}

/// A new selection, owned by an archetype.
#[derive(Clone, Copy, Debug)]
pub struct NewSelection {
    pub archetype: &'static str,
    pub name: &'static str,
    pub guide: &'static str,
}

/// Which of a source selection's options are shared.
#[derive(Clone, Copy, Debug)]
pub enum Options {
    All,
    /// The options with these guide anchors.
    Anchors(&'static [&'static str]),
}

/// A prerequisite leaf that gates an option on a class.
#[derive(Clone, Copy, Debug)]
pub enum Gate {
    Feature(&'static str),
    ClassLevel(&'static str, u8),
}

impl Gate {
    fn prerequisite(self) -> Prerequisite {
        match self {
            Gate::Feature(feature) => Prerequisite::Feature {
                feature: FeatureKey::new(feature),
                count: 1,
            },
            Gate::ClassLevel(class, level) => Prerequisite::ClassLevel {
                class: ClassKey::new(class),
                level,
            },
        }
    }
}

/// The archetype rows a new selection fills: one `rogue/rogue-talents` slot becomes it, or it's added to a row
/// that has none.
#[derive(Clone, Copy, Debug)]
pub enum SlotRule {
    /// Every row granting this feature.
    WithFeature(&'static str),
    /// From this level: every row with a rogue-talent slot or granting this feature.
    FromLevel { level: u8, feature: &'static str },
}

const ROGUE_TALENTS: &str = "rogue/rogue-talents";

/// The classes whose levels level a new selection's options (ticket 47: Combat Trick's row, which the rogue talents
/// they share hold), as the rogue talents' are.
const NEW_LEVELS: [(&str, &str); 2] = [
    ("rogue/sylvan-trickster/fey-tricks", "rogue"),
    ("rogue/underground-chemist/discovery", "rogue"),
];

/// What the guide shares (`slayer#slayer-talents`, `shaman#hex`, `rogue#fey-tricks`, `rogue#discovery`).
pub const SHARES: &[Share] = &[
    // "From 2nd level, the Slayer may select abilities from Rogue Talents, counting his Slayer class level as his
    // Rogue level. [...] He expands the Rogue Talent list with two additional abilities"
    Share {
        selection: "slayer/slayer-talents",
        new: None,
        sources: &[(ROGUE_TALENTS, Options::All)],
        gates: &[(
            Gate::Feature("rogue/advanced-talents"),
            Gate::Feature("slayer/advanced-talents"),
        )],
        slots: None,
    },
    // "The following hexes are shared with the Witch: Ameliorating, Aura of Purity, Evil Eye, Fortune, Healing,
    // Iceplant, Misfortune, Nails, Protective Luck, Slumber and Ward."
    Share {
        selection: "shaman/hex",
        new: None,
        sources: &[(
            "witch/hex",
            Options::Anchors(&[
                "witch#ameliorating",
                "witch#aura-of-purity",
                "witch#evil-eye",
                "witch#fortune",
                "witch#healing-hex",
                "witch#iceplant",
                "witch#misfortune",
                "witch#nails",
                "witch#protective-luck",
                "witch#slumber",
                "witch#ward",
            ]),
        )],
        gates: &[],
        slots: None,
    },
    // "Besides the normal array of Rogue Talents, a Sylvan Trickster can select a hex from the Witch's list. From
    // 10th level, she can additionally select major hexes and at 18th level, can select grand hexes." Her table has
    // Fey Tricks where a rogue's has Rogue Talent.
    Share {
        selection: "rogue/sylvan-trickster/fey-tricks",
        new: Some(NewSelection {
            archetype: "rogue/sylvan-trickster",
            name: "Fey Tricks",
            guide: "rogue#fey-tricks",
        }),
        sources: &[(ROGUE_TALENTS, Options::All), ("witch/hex", Options::All)],
        gates: &[
            (
                Gate::Feature("witch/major-hexes"),
                Gate::Feature("rogue/sylvan-trickster/major-hexes"),
            ),
            (
                Gate::Feature("witch/grand-hexes"),
                Gate::Feature("rogue/sylvan-trickster/grand-hex"),
            ),
        ],
        slots: Some(SlotRule::WithFeature("rogue/sylvan-trickster/fey-tricks")),
    },
    // "At 10th level, an Underground Chemist unlocks certain discoveries that he can take in place of Rogue
    // Talents: Enhance Potion, Extend Potion, Nauseating Flesh, Preserve Organs, Spontaneous Healing and
    // Mummification (requires Preserve Organs)." Her table has Discovery where a rogue's has Rogue Talent at 10.
    Share {
        selection: "rogue/underground-chemist/discovery",
        new: Some(NewSelection {
            archetype: "rogue/underground-chemist",
            name: "Discovery",
            guide: "rogue#discovery",
        }),
        sources: &[
            (ROGUE_TALENTS, Options::All),
            (
                "alchemist/discovery",
                Options::Anchors(&[
                    "alchemist#enhance-potion",
                    "alchemist#extend-potion",
                    "alchemist#nauseating-flesh",
                    "alchemist#preserve-organs",
                    "alchemist#spontaneous-healing",
                    "alchemist#mummification",
                ]),
            ),
        ],
        // Mummification needs alchemist level 10; the chemist's Discovery, at her 10th level, opens it.
        gates: &[(
            Gate::ClassLevel("alchemist", 10),
            Gate::Feature("rogue/underground-chemist/discovery"),
        )],
        slots: Some(SlotRule::FromLevel {
            level: 10,
            feature: "rogue/underground-chemist/discovery",
        }),
    },
];

/// Applies [`SHARES`].
pub fn apply(data: &mut GameData) {
    apply_shares(data, SHARES);
}

/// Applies each share whose selection (or, for a new one, archetype) and sources the data has; the others change
/// nothing.
pub fn apply_shares(data: &mut GameData, shares: &[Share]) {
    for share in shares {
        apply_share(data, share);
    }
}

fn apply_share(data: &mut GameData, share: &Share) {
    let Some(shared) = shared_options(data, share) else {
        return;
    };
    let exists = match share.new {
        None => selection_mut(data, share.selection).is_some(),
        Some(new) => archetype_exists(data, new.archetype),
    };
    if !exists {
        return;
    }

    // Copies: the selection's own options with a shared option's anchor go, and references to them point at the
    // original.
    let by_anchor: BTreeMap<&str, &SelectionOptionKey> = shared
        .iter()
        .map(|(key, guide)| (guide.0.as_str(), key))
        .collect();
    let mut redirect = BTreeMap::new();
    if let Some(selection) = selection_mut(data, share.selection) {
        selection
            .options
            .retain(|option| match by_anchor.get(option.guide.0.as_str()) {
                Some(original) => {
                    redirect.insert(option.key.clone(), (*original).clone());
                    false
                }
                None => true,
            });
    }
    if !redirect.is_empty() {
        for prerequisite in prerequisites_mut(data).into_iter().flatten() {
            point_at_originals(prerequisite, &redirect);
        }
    }

    // Gates.
    let keys: BTreeSet<SelectionOptionKey> = shared.into_iter().map(|(key, _)| key).collect();
    let gates: Vec<(Prerequisite, Prerequisite)> = share
        .gates
        .iter()
        .map(|(gate, alternative)| (gate.prerequisite(), alternative.prerequisite()))
        .collect();
    for selection in selections_mut(data) {
        for option in &mut selection.options {
            if keys.contains(&option.key)
                && let Some(prerequisite) = &mut option.prerequisite
            {
                widen(prerequisite, &gates);
            }
        }
    }

    // The selection, and for a new one its slots.
    match share.new {
        None => {
            if let Some(selection) = selection_mut(data, share.selection) {
                selection.shared_options.extend(keys);
            }
        }
        Some(new) => {
            let key = FeatureSelectionKey::new(share.selection);
            let Some(archetype) = data
                .classes
                .iter_mut()
                .flat_map(|class| &mut class.archetypes)
                .find(|archetype| archetype.key == ArchetypeKey::new(new.archetype))
            else {
                return;
            };
            archetype.selections.push(FeatureSelection {
                key: key.clone(),
                name: new.name.to_owned(),
                summary: String::new(),
                guide: GuideRef::new(new.guide),
                options: Vec::new(),
                shared_options: keys,
                option_levels: NEW_LEVELS
                    .iter()
                    .filter(|(selection, _)| *selection == share.selection)
                    .map(|(_, class)| ClassKey::new(*class))
                    .collect(),
                skipped_levels: BTreeSet::new(),
                feat_lists_only: false,
                only_without_option: false,
                pool_of: None,
                levels_behind: 0,
                level_from: None,
                once_per_option: false,
                level_also_from: None,
            });
            let Some(rule) = share.slots else {
                return;
            };
            let talents = FeatureSelectionKey::new(ROGUE_TALENTS);
            for row in &mut archetype.levels {
                let picked = match rule {
                    SlotRule::WithFeature(feature) => {
                        row.features.contains(&FeatureKey::new(feature))
                    }
                    SlotRule::FromLevel { level, feature } => {
                        row.level >= level
                            && (row.selections.contains(&talents)
                                || row.features.contains(&FeatureKey::new(feature)))
                    }
                };
                if !picked {
                    continue;
                }
                match row.selections.iter().position(|s| s == &talents) {
                    Some(index) => row.selections[index] = key.clone(),
                    None => row.selections.push(key.clone()),
                }
            }
        }
    }
}

/// The options a share takes from its sources, with their guide links; `None` when a source is missing.
fn shared_options(data: &GameData, share: &Share) -> Option<Vec<(SelectionOptionKey, GuideRef)>> {
    let mut shared = Vec::new();
    for (source, options) in share.sources {
        let source = selections(data).find(|s| s.key.as_str() == *source)?;
        for option in &source.options {
            let wanted = match options {
                Options::All => true,
                Options::Anchors(anchors) => anchors.contains(&option.guide.0.as_str()),
            };
            if wanted {
                shared.push((option.key.clone(), option.guide.clone()));
            }
        }
    }
    Some(shared)
}

fn selections(data: &GameData) -> impl Iterator<Item = &FeatureSelection> {
    data.classes
        .iter()
        .chain(&data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .chain(data.mythic_paths.iter().flat_map(|path| &path.selections))
}

fn selections_mut(data: &mut GameData) -> impl Iterator<Item = &mut FeatureSelection> {
    data.classes
        .iter_mut()
        .chain(&mut data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter_mut()
                .chain(class.archetypes.iter_mut().flat_map(|a| &mut a.selections))
        })
        .chain(
            data.mythic_paths
                .iter_mut()
                .flat_map(|path| &mut path.selections),
        )
}

fn selection_mut<'a>(data: &'a mut GameData, key: &str) -> Option<&'a mut FeatureSelection> {
    selections_mut(data).find(|selection| selection.key.as_str() == key)
}

fn archetype_exists(data: &GameData, key: &str) -> bool {
    data.archetypes()
        .any(|archetype| archetype.key.as_str() == key)
}

fn point_at_originals(
    prerequisite: &mut Prerequisite,
    redirect: &BTreeMap<SelectionOptionKey, SelectionOptionKey>,
) {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => {
            for child in children {
                point_at_originals(child, redirect);
            }
        }
        Prerequisite::SelectionOption(key) => {
            if let Some(original) = redirect.get(key) {
                *key = original.clone();
            }
        }
        _ => {}
    }
}

/// Each leaf that is a gate becomes `Any([gate, alternative])`, in canonical order.
fn widen(prerequisite: &mut Prerequisite, gates: &[(Prerequisite, Prerequisite)]) {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => {
            for child in children {
                widen(child, gates);
            }
        }
        leaf => {
            if let Some((gate, alternative)) = gates.iter().find(|(gate, _)| gate == leaf) {
                let mut either = Prerequisite::Any(vec![gate.clone(), alternative.clone()]);
                either.canonicalize();
                *leaf = either;
            }
        }
    }
}
