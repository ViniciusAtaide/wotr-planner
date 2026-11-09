//! The server over streamable HTTP, at `http://localhost:<port>/mcp`, for a client that connects to a running
//! server rather than starting one. Both `wotr-mcp` (with [`PORT_VAR`] set) and the app's server serve it.
//!
//! It listens on the loopback address only and accepts only loopback `Host` headers (rmcp's guard against DNS
//! rebinding). Each client gets its own session and its own copy of the [`Server`], which shares the planner.

use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};

use crate::Server;

/// The environment variable that names the port.
pub const PORT_VAR: &str = "WOTR_MCP_PORT";

/// The port the app's server uses when [`PORT_VAR`] is unset: the dev server is on 3900 and cargo-leptos's live
/// reload on 3901.
pub const DEFAULT_PORT: u16 = 3902;

/// The path the server answers on.
pub const PATH: &str = "/mcp";

/// The port [`PORT_VAR`] names: `None` when it's unset or empty, an error when it isn't a port number.
pub fn port_from_env() -> Result<Option<u16>, String> {
    match std::env::var(PORT_VAR) {
        Ok(value) if value.trim().is_empty() => Ok(None),
        Ok(value) => value
            .trim()
            .parse()
            .map(Some)
            .map_err(|_| format!("{PORT_VAR} must be a port number, not `{value}`")),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(format!("{PORT_VAR} isn't valid Unicode")),
    }
}

/// The URL a client connects to for `port`.
pub fn url(port: u16) -> String {
    format!("http://localhost:{port}{PATH}")
}

/// A server listening on a loopback port, not yet serving.
pub struct Listener {
    listener: tokio::net::TcpListener,
    server: Server,
}

impl Listener {
    /// Binds `127.0.0.1:<port>`, so a busy port is reported before anything is served.
    pub async fn bind(server: Server, port: u16) -> std::io::Result<Self> {
        let listener =
            tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port))).await?;
        Ok(Self { listener, server })
    }

    /// Serves until `stop` completes, then ends every session and returns once their connections close.
    pub async fn serve(
        self,
        stop: impl Future<Output = ()> + Send + 'static,
    ) -> std::io::Result<()> {
        let config = StreamableHttpServerConfig::default();
        let sessions_end = config.cancellation_token.clone();
        let server = self.server;
        let service = StreamableHttpService::new(
            move || Ok(server.clone()),
            Arc::new(LocalSessionManager::default()),
            config,
        );
        let router = axum::Router::new().nest_service(PATH, service);
        axum::serve(self.listener, router)
            .with_graceful_shutdown(async move {
                stop.await;
                sessions_end.cancel();
            })
            .await
    }
}
