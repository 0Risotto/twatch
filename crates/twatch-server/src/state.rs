//! Shared server state and configuration.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use twatch_core::gateway::Gateway;
use twatch_engine::covers::CoverService;
use twatch_engine::stream::StreamService;

/// Runtime configuration for the gateway HTTP server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind: SocketAddr,
    /// Directory containing the built SPA.  `None` falls back to embedded
    /// assets (release feature) or a small "build the UI" page.
    pub web_dir: Option<PathBuf>,
    /// Shared secret for API/auth.  The SPA receives it as an HttpOnly cookie.
    pub token: String,
    /// When false (default), requests with a non-loopback Host header are
    /// rejected.  Set by `--bind 0.0.0.0` deployments that know what they want.
    pub allow_remote: bool,
}

impl ServerConfig {
    #[must_use]
    pub fn local(bind: SocketAddr, token: impl Into<String>) -> Self {
        Self { bind, web_dir: None, token: token.into(), allow_remote: false }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub gateway: Arc<Gateway>,
    pub stream: Option<StreamService>,
    pub covers: Arc<CoverService>,
    pub web_dir: Option<Arc<PathBuf>>,
    pub token: Arc<str>,
    pub allow_remote: bool,
}

impl AppState {
    #[must_use]
    pub fn new(
        gateway: Arc<Gateway>,
        stream: Option<StreamService>,
        covers: Arc<CoverService>,
        config: &ServerConfig,
    ) -> Self {
        Self {
            gateway,
            stream,
            covers,
            web_dir: config.web_dir.clone().map(Arc::new),
            token: Arc::from(config.token.as_str()),
            allow_remote: config.allow_remote,
        }
    }
}
