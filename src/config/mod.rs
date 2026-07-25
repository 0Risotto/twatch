//! Application configuration.
//!
//! Defaults use per-user `XDG` directories.
//! Persisted to `$XDG_CONFIG_HOME/twatch/config.json`.

pub mod search;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

fn default_theme() -> String {
    "tokyonight-dark".into()
}

/// Constructed via [`Config::load()`], which merges a JSON config file
/// with these defaults.  Set fields before calling [`crate::app::run`]
/// to further customise behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the tracing log file (default: `$XDG_STATE_HOME/twatch/twatch.log`).
    #[serde(default)]
    pub log_path: PathBuf,
    /// Tracing filter level (default: `"info"`).
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Directory for the librqbit session state (default: `$XDG_CACHE_HOME/twatch/session`).
    #[serde(default)]
    pub session_path: PathBuf,
    /// Directory for persistent config/history data (default: `$XDG_CONFIG_HOME/twatch`).
    #[serde(default)]
    pub config_dir: PathBuf,
    /// Directory for completed downloads (default: `$XDG_DOWNLOAD_DIR/twatch`).
    #[serde(default)]
    pub download_dir: PathBuf,
    /// Base filename for the history JSON file inside `config_dir` (default: `"history.json"`).
    #[serde(default = "default_history_filename")]
    pub history_filename: String,
    /// Theme name in kebab-case (default: `"tokyonight-dark"`).
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_log_level() -> String {
    "info".into()
}

fn default_history_filename() -> String {
    "history.json".into()
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
            theme: "tokyonight-dark".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from `config_dir/config.json`, merging with defaults.
    ///
    /// If the file doesn't exist or is malformed, returns the defaults.
    #[must_use]
    pub fn load() -> Self {
        let default = Self::default();
        let config_path = default.config_dir.join("config.json");

        let loaded = std::fs::read_to_string(&config_path).ok().and_then(|contents| {
            serde_json::from_str::<Config>(&contents)
                .map_err(|e| {
                    tracing::warn!("Config file corrupted, using defaults: {e}");
                    e
                })
                .ok()
        });

        match loaded {
            Some(user) => {
                // Merge user config over defaults for path fields.
                Self {
                    log_path: default.log_path,
                    log_level: user.log_level,
                    session_path: default.session_path,
                    config_dir: default.config_dir,
                    download_dir: default.download_dir,
                    history_filename: default.history_filename,
                    theme: user.theme,
                }
            }
            None => default,
        }
    }

    /// Atomically persist the current configuration to disk.
    ///
    /// Uses the same atomic-write strategy as the history store.
    pub fn save(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config_dir).context("Failed to create config directory")?;

        let path = self.config_dir.join("config.json");
        let json = serde_json::to_string_pretty(self).context("Failed to serialize config")?;

        let tmp = self.config_dir.join("config.tmp");
        {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&tmp)
                .context("Failed to open temp config file")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                f.set_permissions(std::fs::Permissions::from_mode(0o600)).ok();
            }
            f.write_all(json.as_bytes()).context("Failed to write config")?;
            f.flush().context("Failed to flush config")?;
        }
        std::fs::rename(&tmp, &path).context("Failed to rename config file")?;
        Ok(())
    }
}
