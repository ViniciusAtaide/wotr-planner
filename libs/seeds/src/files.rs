//! Reading the `libs/game-data/**/*.ron` files into [`GameData`] and hashing them.
//!
//! Layout (relative to the data folder):
//!
//! - `progression.ron`, `bonuses.ron`, `races.ron`, `skills.ron`, `weapons.ron`, `feats.ron`, `backgrounds.ron`,
//!   `deities.ron`
//! - `interactions.ron` (ticket 124) and `gear.ron` (ticket 165): optional, since data from before them (which the
//!   behaviour proof reads) has none
//! - `classes/<class>.ron`, `prestige/<class>.ron`
//! - `mythic/abilities.ron`, `mythic/feats.ron`
//! - `mythic/paths/<path>.ron`
//! - `spells/lists.ron`, and `spells/<page>/level-<n>.ron`: the spells whose blocks sit under the guide page's
//!   "Level <n>" heading (ticket 121), all read into one list. A folder without `spells/` (data from before
//!   ticket 121, which the behaviour proof reads) has no spells and no lists.
//!
//! Any other `.ron` file is an error, so a misplaced file can't be silently skipped. Files with other
//! extensions (a README) are ignored. Order in the files means nothing: the result is
//! [canonical](GameData::canonicalize).

use std::fmt::{self, Write as _};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ron::extensions::Extensions;
use rules::GameData;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

const SINGLE_FILES: [&str; 13] = [
    "progression.ron",
    "bonuses.ron",
    "interactions.ron",
    "gear.ron",
    "races.ron",
    "skills.ron",
    "weapons.ron",
    "feats.ron",
    "backgrounds.ron",
    "deities.ron",
    "mythic/abilities.ron",
    "mythic/feats.ron",
    "spells/lists.ron",
];
const FOLDERS: [&str; 3] = ["classes", "prestige", "mythic/paths"];
/// The folder of the spell files: `lists.ron`, and one folder per guide page holding a `level-<n>.ron` per heading.
const SPELLS: &str = "spells";

/// Why the data files couldn't be read.
#[derive(Debug)]
pub enum FilesError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Parse {
        path: PathBuf,
        source: Box<ron::error::SpannedError>,
    },
    Unexpected {
        path: PathBuf,
    },
}

impl fmt::Display for FilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Parse { path, source } => write!(f, "{}:{source}", path.display()),
            Self::Unexpected { path } => write!(
                f,
                "{}: not part of the game-data layout (see libs/seeds/src/files.rs)",
                path.display()
            ),
        }
    }
}

impl std::error::Error for FilesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source.as_ref()),
            Self::Unexpected { .. } => None,
        }
    }
}

/// Reads every data file under `dir` into one canonical [`GameData`].
pub fn read_game_data_files(dir: &Path) -> Result<GameData, FilesError> {
    for file in ron_files(dir)? {
        if !is_in_layout(&file) {
            return Err(FilesError::Unexpected {
                path: dir.join(file),
            });
        }
    }

    let data = GameData {
        progression: read(&dir.join("progression.ron"))?,
        races: read(&dir.join("races.ron"))?,
        skills: read(&dir.join("skills.ron"))?,
        weapons: read(&dir.join("weapons.ron"))?,
        feats: read(&dir.join("feats.ron"))?,
        classes: read_folder(&dir.join("classes"))?,
        prestige_classes: read_folder(&dir.join("prestige"))?,
        mythic_abilities: read(&dir.join("mythic/abilities.ron"))?,
        mythic_feats: read(&dir.join("mythic/feats.ron"))?,
        bonuses: read(&dir.join("bonuses.ron"))?,
        interactions: read_optional(&dir.join("interactions.ron"))?,
        gear: read_optional(&dir.join("gear.ron"))?,
        mythic_paths: read_folder(&dir.join("mythic/paths"))?,
        backgrounds: read(&dir.join("backgrounds.ron"))?,
        deities: read(&dir.join("deities.ron"))?,
        spells: read_spells(&dir.join(SPELLS))?,
        spell_lists: read_spell_lists(&dir.join(SPELLS))?,
    };
    Ok(data.canonical())
}

/// A hex SHA-256 over every `.ron` file under `dir`: their paths relative to `dir` and their contents.
pub fn game_data_hash(dir: &Path) -> Result<String, FilesError> {
    let mut hasher = Sha256::new();
    for file in ron_files(dir)? {
        let path = dir.join(&file);
        let bytes = fs::read(&path).map_err(|source| FilesError::Io { path, source })?;
        hasher.update(file.as_bytes());
        hasher.update([0]);
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    let mut hex = String::with_capacity(64);
    for byte in hasher.finalize().iter() {
        write!(hex, "{byte:02x}").expect("writing to a String can't fail");
    }
    Ok(hex)
}

fn is_in_layout(file: &str) -> bool {
    SINGLE_FILES.contains(&file)
        || FOLDERS.iter().any(|folder| {
            file.strip_prefix(folder)
                .and_then(|rest| rest.strip_prefix('/'))
                .is_some_and(|name| !name.contains('/'))
        })
        || is_spell_level_file(file)
}

/// `spells/<page>/level-<n>.ron`, `<n>` a number.
fn is_spell_level_file(file: &str) -> bool {
    let Some((page, name)) = file
        .strip_prefix(SPELLS)
        .and_then(|rest| rest.strip_prefix('/'))
        .and_then(|rest| rest.split_once('/'))
    else {
        return false;
    };
    let level = name
        .strip_prefix("level-")
        .and_then(|rest| rest.strip_suffix(".ron"));
    !page.is_empty()
        && level.is_some_and(|level| !level.is_empty() && level.bytes().all(|b| b.is_ascii_digit()))
}

/// `spells/lists.ron`, required once the folder exists; no lists without the folder.
fn read_spell_lists(folder: &Path) -> Result<Vec<rules::SpellList>, FilesError> {
    if folder.exists() {
        read(&folder.join("lists.ron"))
    } else {
        Ok(Vec::new())
    }
}

/// Every spell of every `spells/<page>/level-<n>.ron` file, in path order. A missing folder reads as empty.
fn read_spells(folder: &Path) -> Result<Vec<rules::Spell>, FilesError> {
    if !folder.exists() {
        return Ok(Vec::new());
    }
    let mut spells = Vec::new();
    for file in ron_files(folder)? {
        if file.contains('/') {
            spells.extend(read::<Vec<rules::Spell>>(&folder.join(file))?);
        }
    }
    Ok(spells)
}

/// A file that may be missing, which reads as empty.
fn read_optional<T: DeserializeOwned + Default>(path: &Path) -> Result<T, FilesError> {
    if path.exists() {
        read(path)
    } else {
        Ok(T::default())
    }
}

fn read<T: DeserializeOwned>(path: &Path) -> Result<T, FilesError> {
    let text = fs::read_to_string(path).map_err(|source| FilesError::Io {
        path: path.to_owned(),
        source,
    })?;
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .from_str(&text)
        .map_err(|source| FilesError::Parse {
            path: path.to_owned(),
            source: Box::new(source),
        })
}

/// One value per `.ron` file in `folder`, in file-name order. A missing folder reads as empty.
fn read_folder<T: DeserializeOwned>(folder: &Path) -> Result<Vec<T>, FilesError> {
    if !folder.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in read_dir(folder)? {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "ron") {
            paths.push(path);
        }
    }
    paths.sort();
    paths.iter().map(|path| read(path)).collect()
}

/// Every `.ron` file under `dir`, as sorted `/`-separated paths relative to `dir`.
fn ron_files(dir: &Path) -> Result<Vec<String>, FilesError> {
    let mut files = Vec::new();
    collect_ron_files(dir, "", &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_ron_files(dir: &Path, prefix: &str, files: &mut Vec<String>) -> Result<(), FilesError> {
    for entry in read_dir(dir)? {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let relative = format!("{prefix}{name}");
        let file_type = entry.file_type().map_err(|source| FilesError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            collect_ron_files(&path, &format!("{relative}/"), files)?;
        } else if path.extension().is_some_and(|ext| ext == "ron") {
            files.push(relative);
        }
    }
    Ok(())
}

fn read_dir(dir: &Path) -> Result<Vec<fs::DirEntry>, FilesError> {
    let io_error = |source| FilesError::Io {
        path: dir.to_owned(),
        source,
    };
    fs::read_dir(dir)
        .map_err(io_error)?
        .collect::<Result<_, _>>()
        .map_err(io_error)
}
