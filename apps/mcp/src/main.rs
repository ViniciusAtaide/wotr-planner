//! `wotr-mcp`: the MCP server over stdio.
//!
//!   wotr-mcp [--data <dir>]
//!
//! Reads and checks the game data files under `<dir>` (default `$GAME_DATA_DIR`, else `libs/game-data/`), prints
//! to stderr the file the site would serve them as (`game-data/<hash>.json`), then serves MCP on stdin and stdout.
//! stdout carries only the protocol.

use std::path::PathBuf;
use std::process::ExitCode;

use mcp::{Planner, Server};
use rmcp::ServiceExt;

const USAGE: &str = "usage: wotr-mcp [--data <dir>]";

#[tokio::main]
async fn main() -> ExitCode {
    let dir = match data_dir(std::env::args().skip(1)) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("wotr-mcp: {message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let planner = match Planner::load(&dir) {
        Ok(planner) => planner,
        Err(error) => {
            eprintln!("wotr-mcp: {}: {error}", dir.display());
            return ExitCode::FAILURE;
        }
    };
    eprintln!(
        "wotr-mcp: game data from {}, which the site serves as {}",
        dir.display(),
        planner.site_file()
    );
    let served = async {
        Server::new(planner)
            .serve(rmcp::transport::stdio())
            .await?
            .waiting()
            .await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    };
    match served.await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("wotr-mcp: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The data folder: `--data <dir>`, else [`seeds::game_data_dir`].
fn data_dir(mut args: impl Iterator<Item = String>) -> Result<PathBuf, String> {
    let mut dir = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data" => dir = Some(PathBuf::from(args.next().ok_or("--data needs a folder")?)),
            "-h" | "--help" => return Err("the MCP server over the WotR rules engine".to_owned()),
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(dir.unwrap_or_else(seeds::game_data_dir))
}
