//! Torrent metadata preview flow.

use crate::app::App;
use crate::app::screens::Transition;
use crate::app::screens::registry;
use crate::domain::common::spawner::TaskSpawner;
use crate::domain::history::ports::HistoryPort;
use crate::domain::torrent::ports::TorrentPort;
use crate::model::TorrentInfo;
use shaku::HasComponent;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Events produced by preview background tasks.
pub enum PreviewEvent {
    Ready { url: String, info: TorrentInfo },
    Failed(String),
}

/// Owns the preview channel and background-task lifecycle.
pub struct PreviewFlow {
    tx: mpsc::UnboundedSender<PreviewEvent>,
    rx: mpsc::UnboundedReceiver<PreviewEvent>,
}

impl PreviewFlow {
    /// Create an idle flow with no queued events.
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }

    /// Spawn a background task that fetches torrent metadata for `url`.
    pub fn spawn(&self, app: &App, url: String) {
        let torrent: Arc<dyn TorrentPort> = app.module.resolve();
        let spawner: Arc<dyn TaskSpawner> = app.module.resolve();
        let tx = self.tx.clone();

        spawner.spawn(Box::pin(async move {
            match torrent.preview(&url).await {
                Ok(info) => {
                    let _ = tx.send(PreviewEvent::Ready { url, info });
                }
                Err(e) => {
                    let _ = tx.send(PreviewEvent::Failed(format!("Failed: {e}")));
                }
            }
        }));
    }

    /// Remove all currently queued events.
    pub fn drain(&mut self) -> Vec<PreviewEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            events.push(event);
        }
        events
    }
}

impl Default for PreviewFlow {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply a preview event to the application state.
pub fn apply(app: &mut App, event: PreviewEvent) {
    match event {
        PreviewEvent::Ready { url, info } => {
            let storage: Arc<dyn HistoryPort> = app.module.resolve();
            storage.add_entry(&url, &info.name);
            let file_count = info.files.len();
            app.torrent.set_torrent(info.name, url, info.files);
            app.torrent.watched_files = storage
                .history()
                .iter()
                .find(|e| e.url == app.torrent.pending_url.as_deref().unwrap_or_default())
                .map(|e| e.watched_files.clone())
                .unwrap_or_default();
            app.torrent.downloaded_files = storage
                .history()
                .iter()
                .find(|e| e.url == app.torrent.pending_url.as_deref().unwrap_or_default())
                .map(|e| e.downloaded_files.clone())
                .unwrap_or_default();
            // Prune statuses whose files no longer exist on disk.
            let root = app.torrent.download_root_dir(&app.config.download_dir);
            app.torrent.watched_files.retain(|f| root.join(f).exists());
            app.torrent.downloaded_files.retain(|f| root.join(f).exists());
            app.status_message = format!("{file_count} file(s) found — w: watch, d: download");
            app.apply_transition(Transition::ResetWith(registry::browser()));
        }
        PreviewEvent::Failed(msg) => {
            app.set_error(msg);
            app.apply_transition(Transition::Pop);
        }
    }
}
