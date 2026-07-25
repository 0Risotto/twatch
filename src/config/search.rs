//! User-configurable JSON search API.
//!
//! Persisted to `$XDG_CONFIG_HOME/twatch/search.json`.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

fn default_name() -> String {
    "name".into()
}
fn default_info_hash() -> String {
    "info_hash".into()
}
fn default_seeders() -> String {
    "seeders".into()
}
fn default_leechers() -> String {
    "leechers".into()
}
fn default_size() -> String {
    "size".into()
}
fn default_username() -> String {
    "username".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMap {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_info_hash")]
    pub info_hash: String,
    #[serde(default = "default_seeders")]
    pub seeders: String,
    #[serde(default = "default_leechers")]
    pub leechers: String,
    #[serde(default = "default_size")]
    pub size: String,
    #[serde(default = "default_username")]
    pub username: String,
}

impl Default for FieldMap {
    fn default() -> Self {
        Self {
            name: default_name(),
            info_hash: default_info_hash(),
            seeders: default_seeders(),
            leechers: default_leechers(),
            size: default_size(),
            username: default_username(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub field_map: FieldMap,
    #[serde(default)]
    pub page: u32,
}

impl SearchConfig {
    /// Load search config from `config_dir/search.json`, or return defaults.
    #[must_use]
    pub fn load(config_dir: &Path) -> Self {
        let path = config_dir.join("search.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|contents| {
                serde_json::from_str(&contents)
                    .map_err(|e| tracing::warn!("search.json corrupted: {e}"))
                    .ok()
            })
            .unwrap_or_default()
    }

    /// Save search config to disk.
    pub fn save(&self, config_dir: &Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(config_dir).context("Failed to create config dir")?;
        let path = config_dir.join("search.json");
        let json = serde_json::to_string_pretty(self).context("Failed to serialize")?;
        let tmp = config_dir.join("search.tmp");
        {
            let mut f =
                std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(&tmp)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                f.set_permissions(std::fs::Permissions::from_mode(0o600)).ok();
            }
            f.write_all(json.as_bytes())?;
            f.flush()?;
        }
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }
}
