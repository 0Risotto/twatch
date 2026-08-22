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
            if !path.starts_with(app.torrent.download_root_dir(&download_dir)) {
                failed.push(f.name.clone());
                continue;
            }
            match fs::remove_file(&path) {
                Ok(()) => {
                    deleted += 1;
                    storage.mark_deleted(&url, &f.name);
                    app.torrent.watched_files.retain(|wf| wf != &f.name);
                    app.torrent.downloaded_files.retain(|df| df != &f.name);
                    app.torrent.downloading_files.retain(|df| df != &f.name);
                }
                Err(e) => {
                    tracing::warn!("Failed to delete {}: {e}", f.name);
                    failed.push(f.name.clone());
                }
            }
        }
    }

    if failed.is_empty() {
        format!("Deleted {} file(s)", deleted)
    } else {
        format!("Deleted {} file(s), {} failed: {}", deleted, failed.len(), failed.join(", "))
    }
}

/// Rename a browser file on disk and sync the session lists plus
/// persistent history.  Returns a user-facing status message, or an
/// error string when the rename must not proceed.
pub fn rename_browser_file(
    app: &mut App,
    file_index: usize,
    new_name: &str,
) -> Result<String, String> {
    let Some(old) = app.torrent.tree.files().get(file_index).cloned() else {
        return Err("File not found".into());
    };

    // Refuse names that could escape the download directory.
    if new_name.split('/').any(|seg| seg.is_empty() || seg == "." || seg == "..") {
        return Err(format!("Invalid file name: {new_name}"));
    }

    let download_dir = app.config.download_dir.clone();
    let old_path = app.torrent.download_path(&download_dir, &old.name);
    let new_path = app.torrent.download_path(&download_dir, new_name);
    if !new_path.starts_with(app.torrent.download_root_dir(&download_dir)) {
        return Err(format!("Invalid file name: {new_name}"));
    }

    if old_path.exists() {
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
        }
        fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename on disk: {e}"))?;
    }

    if !app.torrent.rename_file(file_index, new_name) {
        return Err("File not found".into());
    }

    for list in [
        &mut app.torrent.watched_files,
        &mut app.torrent.downloaded_files,
        &mut app.torrent.downloading_files,
    ] {
        for name in list.iter_mut() {
            if *name == old.name {
                *name = new_name.to_string();
            }
        }
    }

    let url = app.torrent.pending_url.clone().unwrap_or_default();
    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    storage.rename_file(&url, &old.name, new_name);

    Ok(format!("Renamed to: {new_name}"))
}
