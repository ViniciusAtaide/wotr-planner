//! The game data files: reading the `libs/game-data/**/*.ron` files, the source of truth, into a
//! [`rules::GameData`] and checking it.
//!
//! [`check_game_data_files`] is what the web server, the MCP server and the `check-data` binary load; the tests
//! read the files with [`read_game_data_files`]. There is no database (ticket 187): the server serialises the data
//! it reads here to JSON for the browser.

mod files;

use std::fmt;
use std::path::{Path, PathBuf};

pub use files::{FilesError, read_game_data_files};
use rules::DataProblem;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the seeds crate sits in libs/ inside the workspace")
}

/// The data folder: `$GAME_DATA_DIR` if set, otherwise `libs/game-data/` in the workspace this crate was compiled
/// in.
pub fn game_data_dir() -> PathBuf {
    std::env::var_os("GAME_DATA_DIR")
        .map_or_else(|| workspace_root().join("libs/game-data"), PathBuf::from)
}

/// Reads the data files under `dir` and runs [`rules::GameData::check`] on them. Returns the data, checked, when it
/// has no problems; references to keys the data doesn't have are problems too.
pub fn check_game_data_files(dir: &Path) -> Result<rules::CheckedGameData, Error> {
    read_game_data_files(dir)?.checked().map_err(Error::Invalid)
}

/// Why the game data couldn't be loaded.
#[derive(Debug)]
pub enum Error {
    Files(FilesError),
    /// The data fails [`rules::GameData::check`].
    Invalid(Vec<DataProblem>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Files(error) => write!(f, "reading the game data files: {error}"),
            Self::Invalid(problems) => {
                f.write_str("the game data has problems:")?;
                for problem in problems {
                    write!(f, "\n  {problem}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Files(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<FilesError> for Error {
    fn from(error: FilesError) -> Self {
        Self::Files(error)
    }
}
