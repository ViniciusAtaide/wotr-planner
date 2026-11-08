//! The game data the server answers from, loaded once at startup.

use std::path::Path;
use std::sync::Arc;

use rules::{DataProblem, Engine, FeatureSelection, GameData};
use sha2::{Digest, Sha256};

use crate::search::SearchIndex;

/// The checked game data, the engine built from it, and the search index. Cheap to clone: the tools take a copy
/// onto a blocking thread.
#[derive(Clone, Debug)]
pub struct Planner {
    pub(crate) data: Arc<GameData>,
    pub(crate) engine: Arc<Engine>,
    pub(crate) index: Arc<SearchIndex>,
    site_file: String,
}

impl Planner {
    /// Reads the `.ron` files under `dir` and runs [`GameData::check`] on them.
    pub fn load(dir: &Path) -> Result<Self, seeds::Error> {
        Ok(Self::new(seeds::check_game_data_files(dir)?))
    }

    /// Game data already in memory (the tests' `rules::test_world`), after [`GameData::check`] finds no problem.
    pub fn from_data(data: GameData) -> Result<Self, Vec<DataProblem>> {
        let problems = data.check();
        if problems.is_empty() {
            Ok(Self::new(data))
        } else {
            Err(problems)
        }
    }

    fn new(data: GameData) -> Self {
        let site_file = site_file(&data);
        let engine = Engine::new(data.clone());
        let index = SearchIndex::new(&data, &engine);
        Self {
            data: Arc::new(data),
            engine: Arc::new(engine),
            index: Arc::new(index),
            site_file,
        }
    }

    /// The file the site serves this data as, `game-data/<hash>.json`: when it differs from the deployed site's,
    /// a link valid here may be stale there.
    pub fn site_file(&self) -> &str {
        &self.site_file
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
