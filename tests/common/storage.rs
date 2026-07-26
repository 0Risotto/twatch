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

        entries.retain(|e| e.url != url);
        entries.push(HistoryEntry {
            url: url.to_string(),
            custom_name: None,
            torrent_name: torrent_name.to_string(),
            added_at: now,
            watched: false,
            watched_files: vec![],
            downloaded: false,
            downloaded_files: vec![],
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

    fn remove_entry(&self, index: usize) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        if index >= entries.len() {
            anyhow::bail!("History entry not found");
        }
        entries.remove(index);
        Ok(())
    }
}
