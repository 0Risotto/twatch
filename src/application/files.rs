//! File lifecycle use cases for the torrent browser session.

use crate::app::App;
use crate::domain::history::ports::HistoryPort;
use shaku::HasComponent;
use std::fs;
use std::sync::Arc;

/// Delete the selected files from the download directory, syncing the
/// session lists and persistent history.  Returns a status message.
pub fn delete_selected_files(app: &mut App) -> String {
    let indices = app.torrent.selected_indices();
    let download_dir = app.config.download_dir.clone();
    let url = app.torrent.pending_url.clone().unwrap_or_default();
    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    let mut deleted = 0;
    let mut failed: Vec<String> = Vec::new();

    for i in &indices {
        if let Some(f) = app.torrent.tree.files().get(*i) {
            let path = app.torrent.download_path(&download_dir, &f.name);
            match fs::remove_file(&path) {
                Ok(()) => {
                    deleted += 1;
                    storage.mark_deleted(&url, &f.name);
                    app.torrent.watched_files.retain(|wf| wf != &f.name);
                    app.torrent.downloaded_files.retain(|df| df != &f.name);
                    app.torrent.downloading_files.retain(|df| df != &f.name);
                }
                Err(_) => failed.push(f.name.clone()),
            }
        }
    }

    if failed.is_empty() {
        format!("Deleted {} file(s)", deleted)
    } else {
        format!("Deleted {} file(s), {} failed: {}", deleted, failed.len(), failed.join(", "))
    }
}
