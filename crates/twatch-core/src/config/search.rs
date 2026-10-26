//! Persistence for the user-configurable JSON search API.
//!
//! The [`SearchConfig`] value object lives in the domain layer; this
//! module only knows how to load it from and save it to disk at
//! `$XDG_CONFIG_HOME/twatch/search.json`.

use crate::domain::search::SearchConfig;
use anyhow::Context;
use std::io::Write;
use std::path::Path;

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
