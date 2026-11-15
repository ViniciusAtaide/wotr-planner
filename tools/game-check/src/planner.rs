//! The planner's side: every entry that can name a blueprint ([`GameData::blueprints`]), by entry and by GUID.

use std::collections::{BTreeMap, BTreeSet};

use rules::{BlueprintGuid, DataAt, GameData};

pub struct Planner<'a> {
    pub data: &'a GameData,
    guids: BTreeMap<DataAt, &'a BlueprintGuid>,
    mapped: BTreeSet<&'a BlueprintGuid>,
    /// Every entry of the kinds that can name a blueprint, with the one it names.
    pub entries: Vec<(DataAt, Option<&'a BlueprintGuid>)>,
}

impl<'a> Planner<'a> {
    pub fn new(data: &'a GameData) -> Planner<'a> {
        let entries = data.blueprints();
        let guids = entries
            .iter()
            .filter_map(|(at, guid)| Some((at.clone(), (*guid)?)))
            .collect();
        let mapped = entries.iter().filter_map(|(_, guid)| *guid).collect();
        Planner {
            data,
            guids,
            mapped,
            entries,
        }
    }

    /// The blueprint an entry names.
    pub fn guid(&self, at: DataAt) -> Option<&'a BlueprintGuid> {
        self.guids.get(&at).copied()
    }

    /// Whether some entry names this blueprint.
    pub fn maps(&self, guid: &BlueprintGuid) -> bool {
        self.mapped.contains(guid)
    }

    /// How many entries name a blueprint.
    pub fn with_blueprint(&self) -> usize {
        self.entries
            .iter()
            .filter(|(_, guid)| guid.is_some())
            .count()
    }
}
