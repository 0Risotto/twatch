//! Torrent search background flow.

use crate::app::App;
use crate::app::screens::search::SearchScreen;
use crate::domain::common::spawner::TaskSpawner;
use crate::domain::search::SearchResult;
use crate::domain::search::ports::SearchPort;
use shaku::HasComponent;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Events produced by search background tasks.
pub enum SearchEvent {
    Results(Vec<SearchResult>),
    Failed(String),
}

/// Owns the search channel and background-task lifecycle.
pub struct SearchFlow {
    tx: mpsc::UnboundedSender<SearchEvent>,
    rx: mpsc::UnboundedReceiver<SearchEvent>,
}

impl SearchFlow {
    /// Create an idle flow with no queued events.
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }

    /// Spawn a background task that performs a search query.
    pub fn spawn(&self, app: &App, query: String, page: u32) {
        let searcher: Arc<dyn SearchPort> = app.module.resolve();
        let spawner: Arc<dyn TaskSpawner> = app.module.resolve();
        let config = app.search_config.clone();
        let tx = self.tx.clone();

        spawner.spawn(Box::pin(async move {
            match searcher.search(&query, &config, page).await {
                Ok(results) => {
                    let _ = tx.send(SearchEvent::Results(results));
                }
                Err(e) => {
                    let _ = tx.send(SearchEvent::Failed(format!("Search failed: {e}")));
                }
            }
        }));
    }

    /// Remove all currently queued events.
    pub fn drain(&mut self) -> Vec<SearchEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            events.push(event);
        }
        events
    }
}

impl Default for SearchFlow {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply a search event to the top search screen.
pub fn apply(app: &mut App, event: SearchEvent) {
    match event {
        SearchEvent::Results(results) => {
            let count = results.len();
            if let Some(screen) = app.top_as_mut::<SearchScreen>() {
                screen.apply_results(results);
                app.status_message = format!("{count} result(s) found");
            }
        }
        SearchEvent::Failed(message) => {
            if let Some(screen) = app.top_as_mut::<SearchScreen>() {
                screen.apply_results(Vec::new());
            }
            app.set_error(message);
        }
    }
}
