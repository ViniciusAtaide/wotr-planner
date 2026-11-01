//! `cargo run -p extract -- [--guide DIR] [--out DIR] [--force]`
//!
//! Reads the saved guide pages (default: `guide-cache/` at the repo root) and writes the game data files and
//! `EXTRACTION_REPORT.md` (default: `$GAME_DATA_DIR` or `game-data/`). It never overwrites data: a folder that
//! already holds `.ron` files is refused, and a folder with other files in it (a README) needs `--force`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use extract::build::extract;
use extract::guide::Guide;
use extract::output::write_files;
use extract::ron_out;

fn main() -> ExitCode {
    let mut guide_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the extract crate sits inside the workspace")
        .join("guide-cache");
    let mut out = data::game_data_dir();
    let mut force = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--force" => force = true,
            "--guide" => match args.next() {
                Some(dir) => guide_dir = PathBuf::from(dir),
                None => return usage(),
            },
            "--out" => match args.next() {
                Some(dir) => out = PathBuf::from(dir),
                None => return usage(),
            },
            _ => return usage(),
        }
    }

    let guide = match Guide::load(&guide_dir) {
        Ok(guide) => guide,
        Err(error) => {
            eprintln!("extract: {error}");
            return ExitCode::FAILURE;
        }
    };
    let extraction = extract(&guide);
    let mut files = ron_out::files(&extraction.data);
    files.push((
        "EXTRACTION_REPORT.md".into(),
        extraction.report.markdown(&guide.saved_at),
    ));
    if let Err(error) = write_files(&out, &files, force) {
        eprintln!("extract: {error}");
        return ExitCode::FAILURE;
    }

    // The files must read back as exactly what was built.
    match data::read_game_data_files(&out) {
        Ok(read) if read == extraction.data => {}
        Ok(_) => {
            eprintln!("extract: the files don't read back as the extracted data");
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("extract: reading the files back: {error}");
            return ExitCode::FAILURE;
        }
    }
    let counts: Vec<String> = extraction
        .data
        .counts()
        .iter()
        .map(|(label, count)| format!("{count} {}", label.to_lowercase()))
        .collect();
    println!("extracted into {}: {}", out.display(), counts.join(", "));
    println!(
        "report: {} items in EXTRACTION_REPORT.md; data check: {} problems",
        extraction.report.len(),
        extraction.data.check().len()
    );
    ExitCode::SUCCESS
}

fn usage() -> ExitCode {
    eprintln!("usage: extract [--guide DIR] [--out DIR] [--force]");
    ExitCode::FAILURE
}
