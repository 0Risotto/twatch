//! Persistent history interface.

use crate::model::HistoryEntry;
use anyhow::Result;
use shaku::Interface;

/// `CRUD` operations for a JSON-backed torrent history store.
pub trait StorageService: Interface {
    /// Return a snapshot of every history entry (newest last).
    fn history(&self) -> Vec<HistoryEntry>;

    /// Add (or move to front) a history entry for `url`.
    fn add_entry(&self, url: &str, torrent_name: &str);

    /// Set a custom display name for an entry at `index`.
    fn rename_entry(&self, index: usize, new_name: &str) -> Result<()>;

    /// Remove the entry at `index`.
    fn remove_entry(&self, index: usize) -> Result<()>;
}
