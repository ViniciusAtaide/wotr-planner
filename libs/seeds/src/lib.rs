//! Database access: the SQLite game data database, its embedded migration, and game data in and out of it.
//!
//! The `libs/game-data/**/*.ron` files are the source of truth. [`seed`] builds a new database file
//! from them and renames it over the old one, [`load_game_data`] reads the tables back into
//! [`rules::GameData`], and [`game_data_status`] tells whether the tables still match the files. Every
//! function returns this crate's [`Error`]; sqlx stays an implementation detail.

mod bonuses;
mod columns;
mod files;
mod load;
mod prerequisites;
mod seed;

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub use files::{FilesError, game_data_hash, read_game_data_files};
pub use load::load_game_data;
use rules::DataProblem;
pub use seed::{MissingReference, replace_game_data};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

/// Migrations in `libs/seeds/migrations`, embedded at compile time and run by [`seed`] on each new file.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// A read-only connection pool to the game data database.
#[derive(Clone, Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Opens the database file at `path`, read-only. It must exist ([`seed`] makes it) and come from this
    /// build's migration; otherwise reseed.
    pub async fn open(path: &Path) -> Result<Self, Error> {
        let options = SqliteConnectOptions::new().filename(path).read_only(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await?;
        if !migrated_by_this_build(&pool).await? {
            return Err(Error::OtherMigration);
        }
        Ok(Self { pool })
    }

    fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// For `#[sqlx::test]`s, which hand out an already migrated pool.
impl From<SqlitePool> for Database {
    fn from(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Whether the database's applied migrations are exactly this build's, checksums included. The server
/// opens the database read-only, so it can't run them; a database from an edited migration is reseeded.
async fn migrated_by_this_build(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let applied: Vec<(i64, Vec<u8>)> = sqlx::query_as(
        "SELECT version, checksum FROM _sqlx_migrations WHERE success ORDER BY version",
    )
    .fetch_all(pool)
    .await?;
    let ours = MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .map(|migration| (migration.version, migration.checksum.to_vec()));
    Ok(applied.into_iter().eq(ours))
}

/// The database file: the path in `$DATABASE_URL` (`sqlite://<path>`) if set, otherwise `wotr-planner.db`. A
/// relative path is relative to the workspace root, wherever the binary runs, as it is for the query macros
/// (rustc runs there); the default is git-ignored.
pub fn database_path() -> Result<PathBuf, Error> {
    let path = match std::env::var("DATABASE_URL") {
        Ok(url) => SqliteConnectOptions::from_str(&url)?
            .get_filename()
            .to_owned(),
        Err(_) => PathBuf::from("wotr-planner.db"),
    };
    Ok(workspace_root().join(path))
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the seeds crate sits in libs/ inside the workspace")
}

/// The data folder: `$GAME_DATA_DIR` if set, otherwise `libs/game-data/`.
pub fn game_data_dir() -> PathBuf {
    std::env::var_os("GAME_DATA_DIR")
        .map_or_else(|| workspace_root().join("libs/game-data"), PathBuf::from)
}

/// Reads the data files under `dir` and runs [`rules::GameData::check`] on them, without a database.
/// Returns the data when it has no problems, references to keys that don't exist included: the check catches
/// those before [`seed`]'s foreign keys, which stay as a backstop.
pub fn check_game_data_files(dir: &Path) -> Result<rules::GameData, Error> {
    let data = read_game_data_files(dir)?;
    let problems = data.check();
    if problems.is_empty() {
        Ok(data)
    } else {
        Err(Error::Invalid(problems))
    }
}

/// Reads the data files under `dir`, checks them, and replaces the database at `database` with a new one
/// holding them and the files' hash ([`replace_game_data`]). Returns the hash.
pub async fn seed(database: &Path, dir: &Path) -> Result<String, Error> {
    let hash = game_data_hash(dir)?;
    let data = read_game_data_files(dir)?;
    replace_game_data(database, &data, &hash).await?;
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
    /// The data names keys it doesn't have, every one the foreign keys catch, past
    /// [`check`](rules::GameData::check) (a backstop: the check names them first); nothing was written.
    MissingReferences(Vec<MissingReference>),
    /// The database was built from another migration than this build's: reseed.
    OtherMigration,
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
            Self::MissingReferences(missing) => {
                f.write_str("the game data names keys it doesn't have (foreign key check):")?;
                for reference in missing {
                    write!(f, "\n  {reference}")?;
                }
                Ok(())
            }
            Self::OtherMigration => f.write_str(
                "the game data database was built from another migration than this build's; reseed it",
            ),
            Self::Database(error) => write!(f, "game data database: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Files(error) => Some(error),
            Self::Invalid(_) | Self::MissingReferences(_) | Self::OtherMigration => None,
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
