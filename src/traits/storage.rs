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

    /// Mark the entry matching `url` as watched, recording the file name.
    fn mark_watched(&self, url: &str, file_name: &str);

    /// Mark the entry matching `url` as downloaded, recording the file name.
    fn mark_downloaded(&self, url: &str, file_name: &str);

    /// Remove `file_name` from watched/downloaded files on the entry matching `url`.
    fn mark_deleted(&self, url: &str, file_name: &str);

    /// Remove the entry at `index`.
    fn remove_entry(&self, index: usize) -> Result<()>;
}
