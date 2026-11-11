//! The game data the server answers from, loaded once at startup.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use rules::{CheckedGameData, DataProblem, Engine, FeatureKey, FeatureSelection, GameData};
use sha2::{Digest, Sha256};

use crate::search::SearchIndex;

/// The checked game data, the engine built from it, and the search index. Cheap to clone: the tools take a copy
/// onto a blocking thread.
#[derive(Clone, Debug)]
pub struct Planner {
    pub(crate) data: Arc<GameData>,
    pub(crate) engine: Arc<Engine>,
    pub(crate) index: Arc<SearchIndex>,
    /// Every feature's summary that isn't empty, by key: a class's, an archetype's or a path's features.
    feature_summaries: Arc<HashMap<FeatureKey, String>>,
    site_file: String,
}

impl Planner {
    /// Reads the `.ron` files under `dir` and runs [`GameData::check`] on them.
    pub fn load(dir: &Path) -> Result<Self, seeds::Error> {
        Ok(Self::new(seeds::check_game_data_files(dir)?))
    }

    /// Game data already in memory (the tests' `rules::test_world`), after [`GameData::check`] finds no problem.
    pub fn from_data(data: GameData) -> Result<Self, Vec<DataProblem>> {
        Ok(Self::new(data.checked()?))
    }

    fn new(data: CheckedGameData) -> Self {
        let site_file = site_file(&data);
        let engine = Engine::new(data.clone());
        let data = GameData::clone(&data);
        let index = SearchIndex::new(&data, &engine);
        let classes = data.classes.iter().chain(&data.prestige_classes);
        let feature_summaries = classes
            .clone()
            .flat_map(|class| &class.features)
            .chain(
                classes
                    .flat_map(|class| &class.archetypes)
                    .flat_map(|archetype| &archetype.features),
            )
            .chain(data.mythic_paths.iter().flat_map(|path| &path.features))
            .filter(|feature| !feature.summary.is_empty())
            .map(|feature| (feature.key.clone(), feature.summary.clone()))
            .collect();
        Self {
            data: Arc::new(data),
            engine: Arc::new(engine),
            index: Arc::new(index),
            feature_summaries: Arc::new(feature_summaries),
            site_file,
        }
    }

    /// The file the site serves this data as, `game-data/<hash>.json`: when it differs from the deployed site's,
    /// a link valid here may be stale there.
    pub fn site_file(&self) -> &str {
        &self.site_file
    }

    /// A feature's summary, when the data has one.
    pub(crate) fn feature_summary(&self, key: &FeatureKey) -> Option<String> {
        self.feature_summaries.get(key).cloned()
    }

    /// An item's summary by kind and key, as `search` gives it ([`crate::shapes::Hit::summary`]).
    pub(crate) fn summary(&self, kind: crate::shapes::Kind, key: &str) -> Option<String> {
        self.index
            .get(kind, key)
            .and_then(|hit| hit.summary.clone())
    }

    /// Every Feature selection, wherever it's defined: a class's, an archetype's, a prestige class's or a path's.
    pub(crate) fn selections(&self) -> impl Iterator<Item = &FeatureSelection> {
        all_selections(&self.data)
    }
}

pub(crate) fn all_selections(data: &GameData) -> impl Iterator<Item = &FeatureSelection> {
    let classes = data.classes.iter().chain(&data.prestige_classes);
    classes
        .clone()
        .flat_map(|class| &class.selections)
        .chain(
            classes
                .flat_map(|class| &class.archetypes)
                .flat_map(|archetype| &archetype.selections),
        )
        .chain(data.mythic_paths.iter().flat_map(|path| &path.selections))
}

/// `game-data/<hash>.json`, named as the server names the file it serves (`apps/backend/src/game_data.rs`): the
/// first 16 hex digits of the SHA-256 of the data's JSON. The site's data comes from the database, which loads
/// back equal to the files (`seeds`' round-trip test), so the JSON and the hash are the same.
fn site_file(data: &GameData) -> String {
    let json = serde_json::to_vec(data).expect("the game data serialises to JSON");
    let hash: String = Sha256::digest(&json)[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("game-data/{hash}.json")
}
