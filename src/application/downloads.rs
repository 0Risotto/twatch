//! Live download tracking and completion notifications.
//!
//! The tracker polls engine stats and fires [`CompletionObserver`]s
//! when a non-streaming download finishes.  New completion side
//! effects are added by registering a new observer — the polling
//! loop never changes.

use crate::app::App;
use crate::domain::history::ports::HistoryPort;
use crate::domain::torrent::ports::TorrentPort;
use crate::model::ActiveDownload;
use shaku::HasComponent;
use std::sync::Arc;

/// Notified whenever a non-streaming download reaches 100%.
pub trait CompletionObserver: Send + Sync {
    /// `dl` is the download that just completed.
    fn on_completed(&self, app: &mut App, dl: &ActiveDownload);
}

/// Polls active downloads and notifies completion observers.
#[derive(Default)]
pub struct DownloadTracker {
    observers: Vec<Box<dyn CompletionObserver>>,
}

impl DownloadTracker {
    /// Create a tracker with the built-in observers:
    /// history sync, browser badge sync, and status message.
    #[must_use]
    pub fn new() -> Self {
        Self {
            observers: vec![
                Box::new(HistorySyncObserver),
                Box::new(BrowserStatusObserver),
                Box::new(StatusMessageObserver),
            ],
        }
    }

    /// Poll stats for every active download, notify observers on
    /// completion, keep the single-torrent stats fresh, and prune
    /// finished non-streaming downloads.
    pub fn tick(&self, app: &mut App) {
        let torrent: Arc<dyn TorrentPort> = app.module.resolve();
        let mut completed = Vec::new();

        for dl in &mut app.active_downloads {
            if dl.progress >= 1.0 {
                continue;
            }
            let before = dl.progress;
            if let Ok(stats) = torrent.get_stats(&dl.torrent_id) {
                dl.progress = stats.progress;
                dl.download_speed = stats.download_speed;
                dl.total_size = stats.total_size;
                dl.downloaded = stats.downloaded;
            }
            if before < 1.0 && dl.progress >= 1.0 && !dl.is_streaming {
                completed.push(dl.clone());
            }
        }

        for dl in &completed {
            for observer in &self.observers {
                observer.on_completed(app, dl);
            }
        }

        // Also keep the single-torrent stats for the Player screen.
        if let Some(id) = &app.torrent_id
            && let Ok(stats) = torrent.get_stats(id)
        {
            app.stats = Some(stats);
        }

        app.active_downloads.retain(|dl| dl.is_streaming || dl.progress < 1.0);
        if app.active_downloads.is_empty() {
            app.torrent_id = None;
        }
    }
}

/// Records the completed download in persistent history.
pub struct HistorySyncObserver;

impl CompletionObserver for HistorySyncObserver {
    fn on_completed(&self, app: &mut App, dl: &ActiveDownload) {
        let storage: Arc<dyn HistoryPort> = app.module.resolve();
        for file_name in &dl.file_names {
            storage.mark_downloaded(&dl.url, file_name);
        }
    }
}

/// Updates the session's downloaded/downloading lists.
pub struct BrowserStatusObserver;

impl CompletionObserver for BrowserStatusObserver {
    fn on_completed(&self, app: &mut App, dl: &ActiveDownload) {
        for file_name in &dl.file_names {
            app.torrent.downloading_files.retain(|f| f != file_name);
            if !app.torrent.downloaded_files.iter().any(|f| f == file_name) {
                app.torrent.downloaded_files.push(file_name.clone());
            }
        }
    }
}

/// Shows a completion status message.
pub struct StatusMessageObserver;

impl CompletionObserver for StatusMessageObserver {
    fn on_completed(&self, app: &mut App, dl: &ActiveDownload) {
        app.status_message = format!("Download complete: {}", dl.file_name);
    }
}
