//! Feat lists. While the data is built, a fixed list of feats is a pending list: [`PendingLists::slot`] files it
//! under a placeholder key, and the slot names that. Once the dangling feats are gone, a list of one feat becomes
//! a granted feat ([`grant_lone_feats`]); once the archetype rows are sparse, the rest get their owner and key
//! ([`name_lists`]).
//!
//! A list belongs to the archetype that alone uses it in its stored rows, else to the class (or the path whose
//! ranks use it). Its key is `<owner>/bonus-feats`, or `<owner>/bonus-feats-<first level>` when the owner has
//! several (`-<level>b`, `c`, … when two start at one level: the base class's first, then the one fewer
//! archetypes use). Its name says the same in words, and its guide link is the owner's: the guide's own heading
//! for each list is a hand fix.

use std::collections::{BTreeMap, BTreeSet};

use rules::{Class, FeatKey, FeatList, FeatListKey, FeatSlot, GameData, GuideRef, MythicPath};

const PENDING: &str = "pending-list-";

/// The fixed lists met so far, each once, by their feats and whether they ignore prerequisites.
#[derive(Clone, Debug, Default)]
pub struct PendingLists {
    lists: Vec<(BTreeSet<FeatKey>, bool)>,
}

impl PendingLists {
    /// A `OneOf` slot for the list, filed under a placeholder key.
    pub fn slot(&mut self, feats: BTreeSet<FeatKey>, ignores_prerequisites: bool) -> FeatSlot {
        let list = (feats, ignores_prerequisites);
        let index = match self.lists.iter().position(|known| known == &list) {
            Some(index) => index,
            None => {
                self.lists.push(list);
                self.lists.len() - 1
            }
        };
        FeatSlot::OneOf {
            list: FeatListKey::new(format!("{PENDING}{index}")),
        }
    }

    fn index(key: &FeatListKey) -> Option<usize> {
        key.as_str().strip_prefix(PENDING)?.parse().ok()
    }

    /// The feats and flag of a pending list.
    pub fn get(&self, key: &FeatListKey) -> Option<&(BTreeSet<FeatKey>, bool)> {
        self.lists.get(Self::index(key)?)
    }

    /// The feats of a pending list, to prune.
    pub fn feats_mut(&mut self, key: &FeatListKey) -> Option<&mut BTreeSet<FeatKey>> {
        let index = Self::index(key)?;
        self.lists.get_mut(index).map(|(feats, _)| feats)
    }
}

/// Moves each list of one feat out of the class and archetype rows' slots into their granted feats.
pub fn grant_lone_feats(data: &mut GameData, lists: &PendingLists) {
    let lone = |slot: &FeatSlot| match slot {
        FeatSlot::OneOf { list } => lists
            .get(list)
            .filter(|(feats, _)| feats.len() == 1)
            .and_then(|(feats, _)| feats.first().cloned()),
        _ => None,
    };
    let grant = |feat_slots: &mut Vec<FeatSlot>, feats: &mut BTreeSet<FeatKey>| {
        feat_slots.retain(|slot| match lone(slot) {
            Some(feat) => {
                feats.insert(feat);
                false
            }
            None => true,
        });
    };
    for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        for row in &mut class.levels {
            grant(&mut row.feat_slots, &mut row.feats);
        }
        for archetype in &mut class.archetypes {
            for row in &mut archetype.levels {
                grant(&mut row.feat_slots, &mut row.feats);
            }
        }
    }
}

/// Who uses one pending list, in one class's stored rows.
#[derive(Default)]
struct Users {
    /// Base class levels that use it.
    base: BTreeSet<u8>,
    /// Archetypes that use it, each with the levels.
    archetypes: BTreeMap<String, BTreeSet<u8>>,
}

/// Gives every pending list its owner, key, name and guide link, and points the slots at the keys.
pub fn name_lists(data: &mut GameData, lists: &PendingLists) {
    for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        name_class_lists(class, lists);
    }
    for path in &mut data.mythic_paths {
        name_path_lists(path, lists);
    }
}

/// The content of a pending list, as lists are told apart: its feats and flag.
type Content = (BTreeSet<FeatKey>, bool);

fn content(lists: &PendingLists, slot: &FeatSlot) -> Option<Content> {
    match slot {
        FeatSlot::OneOf { list } => lists.get(list).cloned(),
        _ => None,
    }
}

fn name_class_lists(class: &mut Class, lists: &PendingLists) {
    let mut users: BTreeMap<Content, Users> = BTreeMap::new();
    for row in &class.levels {
        for list in row.feat_slots.iter().filter_map(|s| content(lists, s)) {
            users.entry(list).or_default().base.insert(row.level);
        }
    }
    for archetype in &class.archetypes {
        for row in &archetype.levels {
            for list in row.feat_slots.iter().filter_map(|s| content(lists, s)) {
                users
                    .entry(list)
                    .or_default()
                    .archetypes
                    .entry(archetype.key.to_string())
                    .or_default()
                    .insert(row.level);
            }
        }
    }

    // Owner and first level of each list.
    let mut by_owner: BTreeMap<String, Vec<(Content, u8)>> = BTreeMap::new();
    for (list, used) in &users {
        let (owner, first) = match (used.base.is_empty(), used.archetypes.len()) {
            (true, 1) => {
                let (archetype, levels) = used.archetypes.iter().next().expect("one archetype");
                (archetype.clone(), *levels.first().expect("a level"))
            }
            _ => {
                let first = used
                    .base
                    .iter()
                    .chain(used.archetypes.values().flatten())
                    .min()
                    .copied()
                    .expect("a level");
                (class.key.to_string(), first)
            }
        };
        by_owner
            .entry(owner)
            .or_default()
            .push((list.clone(), first));
    }

    let mut keys: BTreeMap<Content, FeatListKey> = BTreeMap::new();
    let mut defined: BTreeMap<String, Vec<FeatList>> = BTreeMap::new();
    for (owner, mut owned) in by_owner {
        let guide = if owner == class.key.as_str() {
            class.guide.clone()
        } else {
            class
                .archetypes
                .iter()
                .find(|a| a.key.as_str() == owner)
                .map(|a| a.guide.clone())
                .expect("an archetype of the class")
        };
        owned.sort_by_cached_key(|(list, first)| {
            let used = &users[list];
            let archetypes: Vec<&String> = used.archetypes.keys().collect();
            (
                *first,
                used.base.is_empty(),
                used.archetypes.len(),
                archetypes.into_iter().cloned().collect::<Vec<_>>(),
                list.0.clone(),
            )
        });
        let named = owned.len() > 1;
        let mut at_level: BTreeMap<u8, usize> = BTreeMap::new();
        for (list, first) in owned {
            let n = at_level.entry(first).or_default();
            let suffix = if *n == 0 {
                String::new()
            } else {
                char::from(b'a' + u8::try_from(*n).expect("a few lists per level")).to_string()
            };
            *n += 1;
            let (key, name) = if named {
                (
                    format!("{owner}/bonus-feats-{first}{suffix}"),
                    format!("Bonus Feats (from level {first})"),
                )
            } else {
                (format!("{owner}/bonus-feats"), "Bonus Feats".to_owned())
            };
            let key = FeatListKey::new(key);
            keys.insert(list.clone(), key.clone());
            defined.entry(owner.clone()).or_default().push(FeatList {
                key,
                name,
                guide: guide.clone(),
                ignores_prerequisites: list.1,
                feats: list.0,
            });
        }
    }

    let rename = |slots: &mut Vec<FeatSlot>| {
        for slot in slots {
            if let Some(list) = content(lists, slot) {
                *slot = FeatSlot::OneOf {
                    list: keys[&list].clone(),
                };
            }
        }
    };
    for row in &mut class.levels {
        rename(&mut row.feat_slots);
    }
    for archetype in &mut class.archetypes {
        for row in &mut archetype.levels {
            rename(&mut row.feat_slots);
        }
        archetype.feat_lists = defined.remove(archetype.key.as_str()).unwrap_or_default();
    }
    class.feat_lists = defined.remove(class.key.as_str()).unwrap_or_default();
}

fn name_path_lists(path: &mut MythicPath, lists: &PendingLists) {
    let mut first: BTreeMap<Content, u8> = BTreeMap::new();
    for row in &path.ranks {
        for list in row.feat_slots.iter().filter_map(|s| content(lists, s)) {
            let rank = first.entry(list).or_insert(row.rank);
            *rank = (*rank).min(row.rank);
        }
    }
    let named = first.len() > 1;
    let mut keys = BTreeMap::new();
    for (list, rank) in first {
        let (key, name) = if named {
            (
                format!("{}/bonus-feats-{rank}", path.key),
                format!("Bonus Feats (from rank {rank})"),
            )
        } else {
            (
                format!("{}/bonus-feats", path.key),
                "Bonus Feats".to_owned(),
            )
        };
        let key = FeatListKey::new(key);
        keys.insert(list.clone(), key.clone());
        path.feat_lists.push(FeatList {
            key,
            name,
            guide: GuideRef::new(path.guide.0.clone()),
            ignores_prerequisites: list.1,
            feats: list.0,
        });
    }
    for row in &mut path.ranks {
        for slot in &mut row.feat_slots {
            if let Some(list) = content(lists, slot) {
                *slot = FeatSlot::OneOf {
                    list: keys[&list].clone(),
                };
            }
        }
    }
}
