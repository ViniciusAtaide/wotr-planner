//! Writing the data files. The extractor runs once: it never overwrites data.

use std::fs;
use std::path::{Path, PathBuf};

/// Writes `files` (paths relative to `dir`) into `dir`.
///
/// Refuses, writing nothing, when `dir` already holds any `.ron` file or a file about to be written. A folder
/// with only other files in it (a README) needs `force`; a missing or empty folder doesn't.
pub fn write_files(dir: &Path, files: &[(PathBuf, String)], force: bool) -> Result<(), String> {
    let mut existing = Vec::new();
    if dir.exists() {
        collect(dir, dir, &mut existing)?;
    }
    let data: Vec<&PathBuf> = existing
        .iter()
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "ron")
                || files.iter().any(|(file, _)| file == *path)
        })
        .collect();
    if let Some(first) = data.first() {
        return Err(format!(
            "{} already holds data ({} and {} more); the extractor never overwrites it",
            dir.display(),
            first.display(),
            data.len() - 1
        ));
    }
    if !existing.is_empty() && !force {
        return Err(format!(
            "{} is not empty ({}); pass --force to write next to those files",
            dir.display(),
            existing[0].display()
        ));
    }
    for (file, text) in files {
        let path = dir.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(())
}

/// Every file under `dir`, relative to `root`.
fn collect(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("{}: {error}", dir.display()))?
            .path();
        if path.is_dir() {
            collect(root, &path, files)?;
        } else if let Ok(relative) = path.strip_prefix(root) {
            files.push(relative.to_owned());
        }
    }
    files.sort();
    Ok(())
}
