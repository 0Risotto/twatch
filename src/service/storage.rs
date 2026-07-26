//! Default [`StorageService`](crate::traits::StorageService) implementation.
//!
//! Persists a torrent history list to a JSON file inside the config
//! directory.  Every mutation is written to disk immediately.

use crate::model::HistoryEntry;
use crate::traits::StorageService;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use shaku::{Component, Module, ModuleBuildContext};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Maximum number of history entries before older ones are evicted.
const MAX_HISTORY: usize = 1000;

/// Serialisable container for the full history list.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Store {
    history: Vec<HistoryEntry>,
}

/// Encapsulates the on-disk path and in-memory store.
pub struct StorageState {
    path: PathBuf,
    data: Store,
}

impl StorageState {
    /// Read existing history from `config_dir/history_filename`, or
    /// seed an empty store if the file does not exist yet.
    pub fn new(config_dir: PathBuf, history_filename: &str) -> Result<Self> {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        let path = config_dir.join(history_filename);

        let data = if path.exists() {
            let contents = fs::read_to_string(&path).context("Failed to read history file")?;
            serde_json::from_str(&contents).unwrap_or_else(|e| {
                tracing::warn!("History file corrupted, starting fresh: {e}");
                Store::default()
            })
        } else {
            Store::default()
        };

        Ok(Self { path, data })
    }

    fn save(&self) -> Result<()> {
        // Evict oldest entries if we exceed the cap.
        let mut data = &self.data;
        let mut owned;
        if data.history.len() > MAX_HISTORY {
            owned = Store { history: data.history.clone() };
            let excess = owned.history.len() - MAX_HISTORY;
            owned.history.drain(0..excess);
            data = &owned;
        }

        let json = serde_json::to_string_pretty(&data).context("Failed to serialize")?;

        // Write atomically: first to a temp file, then rename.
        // Use 0o600 to prevent other users from reading history.
        let tmp = self.path.with_extension("tmp");
        {
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&tmp)
                .context("Failed to open temp history file")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                f.set_permissions(std::fs::Permissions::from_mode(0o600)).ok();
            }
            f.write_all(json.as_bytes()).context("Failed to write history")?;
            f.flush().context("Failed to flush history")?;
        }
        fs::rename(&tmp, &self.path).context("Failed to rename history file")?;
        Ok(())
    }
}

/// Production [`StorageService`] backed by a JSON file.
pub struct RealStorage {
    pub state: Mutex<StorageState>,
}

/// Shaku parameters for [`RealStorage`].
///
/// `state` must be set via [`AppModule::builder`].
#[derive(Default)]
pub struct RealStorageParameters {
    pub state: Option<Mutex<StorageState>>,
}

impl<M: Module> Component<M> for RealStorage {
    type Interface = dyn StorageService;
    type Parameters = RealStorageParameters;

    #[allow(clippy::expect_used)]
    fn build(
        _context: &mut ModuleBuildContext<M>,
        params: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(Self { state: params.state.expect("RealStorage: state parameter required") })
    }
}

impl StorageService for RealStorage {
    fn history(&self) -> Vec<HistoryEntry> {
        super::lock_state(&self.state).data.history.clone()
    }

    fn add_entry(&self, url: &str, torrent_name: &str) {
        let mut state = super::lock_state(&self.state);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let old = state.data.history.iter().find(|e| e.url == url);

        let custom_name = old.and_then(|e| e.custom_name.clone());
        let watched = old.map(|e| e.watched).unwrap_or(false);
        let watched_files = old.map(|e| e.watched_files.clone()).unwrap_or_default();
        let downloaded = old.map(|e| e.downloaded).unwrap_or(false);
        let downloaded_files = old.map(|e| e.downloaded_files.clone()).unwrap_or_default();

        state.data.history.retain(|e| e.url != url);

        state.data.history.push(HistoryEntry {
            url: url.to_string(),
            custom_name,
            torrent_name: torrent_name.to_string(),
            added_at: now,
            watched,
            watched_files,
            downloaded,
            downloaded_files,
        });

        let _ = state.save();
    }

    fn rename_entry(&self, index: usize, new_name: &str) -> Result<()> {
        let mut state = super::lock_state(&self.state);
        let entry = state.data.history.get_mut(index).context("History entry not found")?;
        entry.custom_name = Some(new_name.to_string());
        state.save()
    }

    fn mark_watched(&self, url: &str, file_name: &str) {
        let mut state = super::lock_state(&self.state);
        for e in &mut state.data.history {
            if e.url == url {
                e.watched = true;
                if !e.watched_files.iter().any(|f| f == file_name) {
                    e.watched_files.push(file_name.to_string());
                }
                break;
            }
        }
        let _ = state.save();
    }

    fn mark_downloaded(&self, url: &str, file_name: &str) {
        let mut state = super::lock_state(&self.state);
        for e in &mut state.data.history {
            if e.url == url {
                e.downloaded = true;
                if !e.downloaded_files.iter().any(|f| f == file_name) {
                    e.downloaded_files.push(file_name.to_string());
                }
                break;
            }
        }
        let _ = state.save();
    }

    fn mark_deleted(&self, url: &str, file_name: &str) {
        let mut state = super::lock_state(&self.state);
        for e in &mut state.data.history {
            if e.url == url {
                e.watched_files.retain(|f| f != file_name);
                e.downloaded_files.retain(|f| f != file_name);
                if e.watched_files.is_empty() {
                    e.watched = false;
                }
                if e.downloaded_files.is_empty() {
                    e.downloaded = false;
                }
                break;
            }
        }
        let _ = state.save();
    }

    fn remove_entry(&self, index: usize) -> Result<()> {
        let mut state = super::lock_state(&self.state);
        if index >= state.data.history.len() {
            anyhow::bail!("History entry not found");
        }
        state.data.history.remove(index);
        state.save()
    }
}
