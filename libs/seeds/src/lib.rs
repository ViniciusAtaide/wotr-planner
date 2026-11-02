//! Database access: the Postgres connection, embedded migrations, and game data in and out of Postgres.
//!
//! The `libs/game-data/**/*.ron` files are the source of truth. [`seed`] loads them into the game
//! tables, [`load_game_data`] reads the tables back into [`rules::GameData`], and
//! [`game_data_status`] tells whether the tables still match the files. Every function returns this
//! crate's [`Error`]; sqlx stays an implementation detail.

mod columns;
mod files;
mod load;
mod prerequisites;
mod seed;

use std::fmt;
use std::path::{Path, PathBuf};

pub use files::{FilesError, game_data_hash, read_game_data_files};
pub use load::load_game_data;
use rules::DataProblem;
pub use seed::replace_game_data;

/// Migrations in `libs/seeds/migrations`, embedded at compile time and run by [`Database::connect`].
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// A connection pool to the game data database.
#[derive(Clone, Debug)]
pub struct Database {
    pool: sqlx::PgPool,
}

impl Database {
    /// Connects to Postgres and applies any pending migrations.
    pub async fn connect(url: &str) -> Result<Self, Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;
        MIGRATOR.run(&pool).await.map_err(sqlx::Error::from)?;
        Ok(Self { pool })
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

/// For `#[sqlx::test]`s, which hand out an already migrated pool.
impl From<sqlx::PgPool> for Database {
    fn from(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

/// The data folder: `$GAME_DATA_DIR` if set, otherwise `libs/game-data/`.
pub fn game_data_dir() -> PathBuf {
    std::env::var_os("GAME_DATA_DIR").map_or_else(
        || {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("the data crate sits in libs/ inside the workspace")
                .join("game-data")
        },
        PathBuf::from,
    )
}

/// Reads the data files under `dir` and runs [`rules::GameData::check`] on them, without a database.
/// Returns the data when it has no problems. References to keys that don't exist are left to the
/// database's foreign keys, so only [`seed`] catches those.
pub fn check_game_data_files(dir: &Path) -> Result<rules::GameData, Error> {
    let data = read_game_data_files(dir)?;
    let problems = data.check();
    if problems.is_empty() {
        Ok(data)
    } else {
        Err(Error::Invalid(problems))
    }
}

/// Reads the data files under `dir`, checks them, and replaces every game table with them in one
/// transaction, storing the files' hash. Returns the hash.
pub async fn seed(db: &Database, dir: &Path) -> Result<String, Error> {
    let hash = game_data_hash(dir)?;
    let data = read_game_data_files(dir)?;
    replace_game_data(db, &data, &hash).await?;
    Ok(hash)
}

/// Whether the game tables match the data files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameDataStatus {
    /// Seeded from files with the same hash as the current ones.
    Current,
    /// Seeded from different files; reseed to pick up the changes.
    Stale { seeded: String, files: String },
    /// Never seeded.
    NeverSeeded,
}

/// Compares the hash stored by the last [`seed`] with the hash of the files under `dir`.
pub async fn game_data_status(db: &Database, dir: &Path) -> Result<GameDataStatus, Error> {
    let files = game_data_hash(dir)?;
    let seeded = sqlx::query_scalar!("SELECT hash FROM game_data_seed")
        .fetch_optional(db.pool())
        .await?;
    Ok(match seeded {
        None => GameDataStatus::NeverSeeded,
        Some(seeded) if seeded == files => GameDataStatus::Current,
        Some(seeded) => GameDataStatus::Stale { seeded, files },
    })
}

/// Why reading, seeding or loading the game data failed.
#[derive(Debug)]
pub enum Error {
    Files(FilesError),
    /// The data fails [`rules::GameData::check`]; nothing was written.
    Invalid(Vec<DataProblem>),
    Database(DatabaseError),
}

/// A database failure: connecting, a query, a constraint, or rows that don't fit the `rules` types.
#[derive(Debug)]
pub struct DatabaseError(sqlx::Error);

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
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
            Self::Database(error) => write!(f, "game data database: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Files(error) => Some(error),
            Self::Invalid(_) => None,
            Self::Database(error) => Some(error),
        }
    }
}

impl From<FilesError> for Error {
    fn from(error: FilesError) -> Self {
        Self::Files(error)
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(DatabaseError(error))
    }
}
