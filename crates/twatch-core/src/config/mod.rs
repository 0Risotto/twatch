//! Application configuration.
//!
//! Defaults use per-user `XDG` directories.
//! Persisted to `$XDG_CONFIG_HOME/twatch/config.json`.

pub mod search;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

/// Constructed via [`Config::load()`], which merges a JSON config file
/// with these defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
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
    /// Theme name for the web UI (default: `"mono-dark"`).
    #[serde(default = "default_web_theme")]
    pub web_theme: String,
}

fn default_history_filename() -> String {
    "history.json".into()
}

fn default_web_theme() -> String {
    "mono-dark".into()
}

impl Default for Config {
    fn default() -> Self {
        let default_pb = PathBuf::default();
        let config_dir = dirs::config_dir().unwrap_or_else(|| default_pb.clone()).join("twatch");
        let download_dir =
            dirs::download_dir().unwrap_or_else(|| default_pb.clone()).join("twatch");
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| default_pb.clone()).join("twatch");

        Self {
            session_path: cache_dir.join("session"),
            config_dir,
            download_dir,
            history_filename: "history.json".to_string(),
            web_theme: "mono-dark".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from the default config dir, merging with
    /// built-in defaults.  If the file doesn't exist or is malformed,
    /// returns the defaults.
    #[must_use]
    pub fn load() -> Self {
        Self::load_from(&Self::default().config_dir)
    }

    /// Load configuration from `config_dir/config.json`, merging with
    /// built-in defaults.  Explicitly user-set fields win; fields left
    /// empty (or unset) fall back to the built-in defaults.
    #[must_use]
    pub fn load_from(config_dir: &std::path::Path) -> Self {
        let default = Self::default();
        let config_path = config_dir.join("config.json");

        let loaded = std::fs::read_to_string(&config_path).ok().and_then(|contents| {
            serde_json::from_str::<Config>(&contents)
                .map_err(|e| {
                    tracing::warn!("Config file corrupted, using defaults: {e}");
                    e
                })
                .ok()
        });

        match loaded {
            Some(user) => merge_config(default, user),
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

/// Merge a user-supplied config over the built-in defaults: an empty
/// path or string means "not set" and keeps the default.
fn merge_config(default: Config, user: Config) -> Config {
    Config {
        session_path: or_default_path(&user.session_path, &default.session_path),
        config_dir: or_default_path(&user.config_dir, &default.config_dir),
        download_dir: or_default_path(&user.download_dir, &default.download_dir),
        history_filename: or_default_str(&user.history_filename, &default.history_filename),
        web_theme: or_default_str(&user.web_theme, &default.web_theme),
    }
}

fn or_default_path(user: &std::path::Path, default: &std::path::Path) -> PathBuf {
    if user.as_os_str().is_empty() { default.to_path_buf() } else { user.to_path_buf() }
}

fn or_default_str(user: &str, default: &str) -> String {
    if user.is_empty() { default.to_string() } else { user.to_string() }
}
