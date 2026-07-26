use anyhow::{Context, Result};
use std::sync::Mutex;
use twatch::model::HistoryEntry;
use twatch::traits::StorageService;

pub struct MockStorageService {
    entries: Mutex<Vec<HistoryEntry>>,
}

impl MockStorageService {
    pub const fn new() -> Self {
        Self { entries: Mutex::new(Vec::new()) }
    }
}

impl StorageService for MockStorageService {
    fn history(&self) -> Vec<HistoryEntry> {
        self.entries.lock().unwrap().clone()
    }

    fn add_entry(&self, url: &str, torrent_name: &str) {
        let mut entries = self.entries.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let old = entries.iter().find(|e| e.url == url);

        let custom_name = old.and_then(|e| e.custom_name.clone());
        let watched = old.map(|e| e.watched).unwrap_or(false);
        let watched_files = old.map(|e| e.watched_files.clone()).unwrap_or_default();
        let downloaded = old.map(|e| e.downloaded).unwrap_or(false);
        let downloaded_files = old.map(|e| e.downloaded_files.clone()).unwrap_or_default();

        entries.retain(|e| e.url != url);
        entries.push(HistoryEntry {
            url: url.to_string(),
            custom_name,
            torrent_name: torrent_name.to_string(),
            added_at: now,
            watched,
            watched_files,
            downloaded,
            downloaded_files,
        });
    }

    fn rename_entry(&self, index: usize, new_name: &str) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        let entry = entries.get_mut(index).context("History entry not found")?;
        entry.custom_name = Some(new_name.to_string());
        Ok(())
    }

    fn mark_watched(&self, url: &str, file_name: &str) {
        let mut entries = self.entries.lock().unwrap();
        for e in &mut *entries {
            if e.url == url {
                e.watched = true;
                if !e.watched_files.iter().any(|f| f == file_name) {
                    e.watched_files.push(file_name.to_string());
                }
                break;
            }
        }
    }

    fn mark_downloaded(&self, url: &str, file_name: &str) {
        let mut entries = self.entries.lock().unwrap();
        for e in &mut *entries {
            if e.url == url {
                e.downloaded = true;
                if !e.downloaded_files.iter().any(|f| f == file_name) {
                    e.downloaded_files.push(file_name.to_string());
                }
                break;
            }
        }
    }

    fn mark_deleted(&self, url: &str, file_name: &str) {
        let mut entries = self.entries.lock().unwrap();
        for e in &mut *entries {
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
    }

    fn remove_entry(&self, index: usize) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        if index >= entries.len() {
            anyhow::bail!("History entry not found");
        }
        entries.remove(index);
        Ok(())
    }
}
