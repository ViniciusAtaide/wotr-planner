//! `wotr-mcp`: the MCP server, over stdio or streamable HTTP.
//!
//!   [WOTR_MCP_PORT=<port>] wotr-mcp [--data <dir>]
//!
//! Reads and checks the game data files under `<dir>` (default `$GAME_DATA_DIR`, else `libs/game-data/`), then
//! serves MCP:
//! - with `WOTR_MCP_PORT` set, as streamable HTTP at `http://localhost:<port>/mcp` ([`mcp::http`]), for a client
//!   that connects to a running server (`.mcp.json`), until SIGINT or SIGTERM. The app's server serves the same
//!   endpoint, so this is for running it without the app;
//! - otherwise on stdin and stdout, for a client that starts the server itself; stdout carries only the protocol.
//!
//! stderr (Claude Code keeps a stdio server's in its MCP logs, ticket 118) gets one line each for:
//! - the start: version, pid and the binary;
//! - a warning when a source file the binary is built from is newer than the binary (`.mcp.json` runs the built
//!   binary, so a change needs `scripts/build-mcp.sh`);
//! - the game data: the file the site would serve it as (`game-data/<hash>.json`);
//! - over stdio, the client once it has initialized; over HTTP, the URL it serves;
//! - the exit and its reason (stdin closed, a signal, cancelled, an error), with the time served.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Instant, SystemTime};

use mcp::{Planner, Server, http};
use rmcp::ServiceExt;
use rmcp::service::QuitReason;

const USAGE: &str = "usage: [WOTR_MCP_PORT=<port>] wotr-mcp [--data <dir>]";

/// The folders, under the workspace root, of the crates the binary is built from.
const SOURCES: [&str; 3] = ["apps/mcp/src", "libs/rules/src", "libs/seeds/src"];

#[tokio::main]
async fn main() -> ExitCode {
    let started = Instant::now();
    let exe = std::env::current_exe().ok();
    eprintln!(
        "wotr-mcp {}: started (pid {}) from {}",
        env!("CARGO_PKG_VERSION"),
        std::process::id(),
        exe.as_deref().map_or_else(
            || "an unknown path".to_owned(),
            |exe| exe.display().to_string()
        )
    );
    if let Some(newer) = exe.as_deref().and_then(newer_source) {
        eprintln!(
            "wotr-mcp: warning: {} is newer than this binary; rebuild it with scripts/build-mcp.sh",
            newer.display()
        );
    }
    let (dir, port) = match data_dir(std::env::args().skip(1))
        .and_then(|dir| Ok((dir, http::port_from_env()?)))
    {
        Ok(found) => found,
        Err(message) => {
            eprintln!("wotr-mcp: {message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let planner = match Planner::load(&dir) {
        Ok(planner) => planner,
        Err(error) => {
            eprintln!("wotr-mcp: exiting: {}: {error}", dir.display());
            return ExitCode::FAILURE;
        }
    };
    eprintln!(
        "wotr-mcp: game data from {}, which the site serves as {}",
        dir.display(),
        planner.site_file()
    );
    let server = Server::new(planner);
    match port {
        Some(port) => serve_http(server, port, started).await,
        None => serve_stdio(server, started).await,
    }
}

/// Serves one client on stdin and stdout until it closes stdin.
async fn serve_stdio(server: Server, started: Instant) -> ExitCode {
    let running = match server.serve(rmcp::transport::stdio()).await {
        Ok(running) => running,
        Err(error) => {
            eprintln!("wotr-mcp: exiting before the client initialized: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(client) = running.peer().peer_info() {
        eprintln!(
            "wotr-mcp: serving {} {} (protocol {}) after {} ms",
            client.client_info.name,
            client.client_info.version,
            client.protocol_version,
            started.elapsed().as_millis()
        );
    }
    let quit = running.waiting().await;
    let served = started.elapsed().as_secs();
    match quit {
        Ok(QuitReason::Closed) => {
            eprintln!("wotr-mcp: exiting after {served} s: the client closed stdin");
            ExitCode::SUCCESS
        }
        Ok(QuitReason::Cancelled) => {
            eprintln!("wotr-mcp: exiting after {served} s: cancelled");
            ExitCode::SUCCESS
        }
        Ok(QuitReason::JoinError(error)) | Err(error) => {
            eprintln!("wotr-mcp: exiting after {served} s: the service task failed: {error}");
            ExitCode::FAILURE
        }
        Ok(other) => {
            eprintln!("wotr-mcp: exiting after {served} s: {other:?}");
            ExitCode::SUCCESS
        }
    }
}

/// Serves streamable HTTP at `http://localhost:<port>/mcp` until SIGINT or SIGTERM.
async fn serve_http(server: Server, port: u16, started: Instant) -> ExitCode {
    let listener = match http::Listener::bind(server, port).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("wotr-mcp: exiting: can't listen on port {port}: {error}");
            return ExitCode::FAILURE;
        }
    };
    eprintln!(
        "wotr-mcp: serving MCP at {} after {} ms",
        http::url(port),
        started.elapsed().as_millis()
    );
    let served = listener
        .serve(async {
            let signal = stop_signal().await;
            eprintln!("wotr-mcp: {signal}: closing the sessions");
        })
        .await;
    let seconds = started.elapsed().as_secs();
    match served {
        Ok(()) => {
            eprintln!("wotr-mcp: exiting after {seconds} s");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("wotr-mcp: exiting after {seconds} s: the HTTP server failed: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Waits for SIGINT (Ctrl-C) or, on Unix, SIGTERM, and names the one that came.
async fn stop_signal() -> &'static str {
    let interrupt = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let terminate = async {
            match signal(SignalKind::terminate()) {
                Ok(mut terminate) => {
                    terminate.recv().await;
                }
                Err(_) => std::future::pending::<()>().await,
            }
        };
        tokio::select! {
            () = interrupt => "SIGINT",
            () = terminate => "SIGTERM",
        }
    }
    #[cfg(not(unix))]
    {
        interrupt.await;
        "Ctrl-C"
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

/// A source file under [`SOURCES`] modified after the binary `exe`, if any: then the binary may lag the source.
/// Only a hint, read from the checkout the binary was built in; game data is read at start, so it never lags.
fn newer_source(exe: &Path) -> Option<PathBuf> {
    let built = modified(exe)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2)?;
    let mut folders: Vec<PathBuf> = SOURCES.iter().map(|folder| root.join(folder)).collect();
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(&folder).ok()?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                folders.push(path);
            } else if modified(&path).is_some_and(|time| time > built) {
                return Some(path);
            }
        }
    }
    None
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
}
