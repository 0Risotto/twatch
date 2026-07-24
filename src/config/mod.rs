//! Application configuration.
//!
//! Defaults use per-user `XDG` directories.

use std::path::PathBuf;

/// Constructed via [`Config::default()`], which picks standard `XDG`
/// directories.  Set fields before calling [`crate::app::run`] to
/// customise behaviour.
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the tracing log file (default: `$XDG_STATE_HOME/twatch/twatch.log`).
    pub log_path: PathBuf,
    /// Tracing filter level (default: `"info"`).
    pub log_level: String,
    /// Directory for the librqbit session state (default: `$XDG_CACHE_HOME/twatch/session`).
    pub session_path: PathBuf,
    /// Directory for persistent config/history data (default: `$XDG_CONFIG_HOME/twatch`).
    pub config_dir: PathBuf,
    /// Directory for completed downloads (default: `$XDG_DOWNLOAD_DIR/twatch`).
    pub download_dir: PathBuf,
    /// Base filename for the history JSON file inside `config_dir` (default: `"history.json"`).
    pub history_filename: String,
}

impl Default for Config {
    fn default() -> Self {
        let default_pb = PathBuf::default();
        let config_dir = dirs::config_dir().unwrap_or_else(|| default_pb.clone()).join("twatch");
        let download_dir =
            dirs::download_dir().unwrap_or_else(|| default_pb.clone()).join("twatch");
        let state_dir = dirs::state_dir().unwrap_or_else(|| default_pb.clone()).join("twatch");
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| default_pb.clone()).join("twatch");

        Self {
            log_path: state_dir.join("twatch.log"),
            log_level: "info".to_string(),
            session_path: cache_dir.join("session"),
            config_dir,
            download_dir,
            history_filename: "history.json".to_string(),
        }
    }
}
