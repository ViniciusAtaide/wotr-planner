//! Reading the `libs/game-data/**/*.ron` files into [`GameData`].
//!
//! Layout (relative to the data folder, ticket 244): one file per entry, in a folder by kind, named by its key
//! (`features/rogue/sneak-attack.ron` holds the feature keyed `rogue/sneak-attack`):
//!
//! - `races/`, `subraces/`, `weapons/`, `feats/`, `feat-lists/`, `classes/` (base classes), `prestige/` (prestige
//!   classes), `archetypes/`, `features/`, `selections/`, `options/` (selection options), `mythic/abilities/`,
//!   `mythic/feats/`, `mythic/paths/`, `backgrounds/`, `deities/`, `gear/`, `spells/`, `spell-lists/`;
//! - `planner/<kind folder>/`: the entries the planner made, with no game blueprint behind them, each written as
//!   `(why: "…", entry: …)`, the reason no blueprint stands behind it and the entry;
//! - the single files `progression.ron`, `skills.ron` (the game's skills are a stat, with no blueprint),
//!   `bonuses.ron`, and `interactions.ron` (optional, since data from before ticket 124 has none).
//!
//! A missing folder reads as empty. Any other `.ron` file is an error, so a misplaced file can't be silently
//! skipped, and so is a file whose path isn't its entry's key. Files with other extensions (a README) are ignored.
//! Order in the files means nothing: the result is [canonical](GameData::canonicalize).

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ron::extensions::Extensions;
use rules::GameData;
use serde::Deserialize;
use serde::de::DeserializeOwned;

/// The single files, and whether each must be there.
const SINGLE_FILES: [(&str, bool); 4] = [
    ("progression.ron", true),
    ("skills.ron", true),
    ("bonuses.ron", true),
    ("interactions.ron", false),
];

/// The folders of one file per entry, by kind.
const FOLDERS: [&str; 19] = [
    "races",
    "subraces",
    "weapons",
    "feats",
    "feat-lists",
    "classes",
    "prestige",
    "archetypes",
    "features",
    "selections",
    "options",
    "mythic/abilities",
    "mythic/feats",
    "mythic/paths",
    "backgrounds",
    "deities",
    "gear",
    "spells",
    "spell-lists",
];

/// The folder of the planner-made entries, which holds a folder per kind as the data folder does.
const PLANNER: &str = "planner";

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
    /// An entry's file whose path (under its kind's folder, without `.ron`) isn't the entry's key.
    Misnamed {
        path: PathBuf,
        key: String,
    },
    /// A planner-made entry that doesn't say why no blueprint stands behind it.
    NoReason {
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
            Self::Misnamed { path, key } => write!(
                f,
                "{}: holds the entry keyed {key}; an entry's file is named by its key",
                path.display()
            ),
            Self::NoReason { path } => write!(
                f,
                "{}: a planner-made entry says why no blueprint stands behind it",
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
            Self::Unexpected { .. } | Self::Misnamed { .. } | Self::NoReason { .. } => None,
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
        skills: read(&dir.join("skills.ron"))?,
        bonuses: read(&dir.join("bonuses.ron"))?,
        interactions: read_optional(&dir.join("interactions.ron"))?,
        races: entries(dir, "races", |r: &rules::Race| r.key.as_str())?,
        subraces: entries(dir, "subraces", |s: &rules::Subrace| s.key.as_str())?,
        weapons: entries(dir, "weapons", |w: &rules::Weapon| w.key.as_str())?,
        feats: entries(dir, "feats", |f: &rules::Feat| f.key.as_str())?,
        feat_lists: entries(dir, "feat-lists", |l: &rules::FeatList| l.key.as_str())?,
        classes: entries(dir, "classes", |c: &rules::Class| c.key.as_str())?,
        prestige_classes: entries(dir, "prestige", |c: &rules::Class| c.key.as_str())?,
        archetypes: entries(dir, "archetypes", |a: &rules::Archetype| a.key.as_str())?,
        features: entries(dir, "features", |f: &rules::Feature| f.key.as_str())?,
        selections: entries(dir, "selections", |s: &rules::FeatureSelection| {
            s.key.as_str()
        })?,
        options: entries(dir, "options", |o: &rules::SelectionOption| o.key.as_str())?,
        mythic_abilities: entries(dir, "mythic/abilities", |a: &rules::MythicAbility| {
            a.key.as_str()
        })?,
        mythic_feats: entries(dir, "mythic/feats", |f: &rules::MythicFeat| f.key.as_str())?,
        mythic_paths: entries(dir, "mythic/paths", |p: &rules::MythicPath| p.key.as_str())?,
        backgrounds: entries(dir, "backgrounds", |b: &rules::Background| b.key.as_str())?,
        deities: entries(dir, "deities", |d: &rules::Deity| d.key.as_str())?,
        gear: entries(dir, "gear", |g: &rules::GearItem| g.key.as_str())?,
        spells: entries(dir, "spells", |s: &rules::Spell| s.key.as_str())?,
        spell_lists: entries(dir, "spell-lists", |l: &rules::SpellList| l.key.as_str())?,
    };
    Ok(data.canonical())
}

/// A single file, or a file under a kind's folder, directly or under `planner/`.
fn is_in_layout(file: &str) -> bool {
    let entry = file.strip_prefix("planner/").unwrap_or(file);
    SINGLE_FILES.iter().any(|(name, _)| *name == file)
        || FOLDERS.iter().any(|folder| {
            entry
                .strip_prefix(folder)
                .is_some_and(|rest| rest.starts_with('/'))
        })
}

/// A planner-made entry: the reason no game blueprint stands behind it, and the entry.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Planner<T> {
    why: String,
    entry: T,
}

/// Every entry of one kind: one per file under `folder` and under `planner/<folder>`, each file named by its
/// entry's key (`key`).
fn entries<T: DeserializeOwned>(
    dir: &Path,
    folder: &str,
    key: impl Fn(&T) -> &str,
) -> Result<Vec<T>, FilesError> {
    let mut out = Vec::new();
    let named = |path: &Path, name: &str, entry: &T| {
        let entry_key = key(entry);
        if name.strip_suffix(".ron") == Some(entry_key) {
            Ok(())
        } else {
            Err(FilesError::Misnamed {
                path: path.to_owned(),
                key: entry_key.to_owned(),
            })
        }
    };
    let own = dir.join(folder);
    if own.exists() {
        for name in ron_files(&own)? {
            let path = own.join(&name);
            let entry: T = read(&path)?;
            named(&path, &name, &entry)?;
            out.push(entry);
        }
    }
    let planner = dir.join(PLANNER).join(folder);
    if planner.exists() {
        for name in ron_files(&planner)? {
            let path = planner.join(&name);
            let Planner { why, entry } = read::<Planner<T>>(&path)?;
            if why.trim().is_empty() {
                return Err(FilesError::NoReason { path });
            }
            named(&path, &name, &entry)?;
            out.push(entry);
        }
    }
    Ok(out)
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
