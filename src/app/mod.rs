//! Application shell: session state, screen stack, and the main loop.
//!
//! Business logic lives in [`crate::application`]; screens own their
//! state; the loop here only orchestrates: drain background flows,
//! poll input, tick the download tracker, render.

pub mod screens;
pub mod state;

use crate::app::screens::{Screen, Transition, registry};
use crate::app::state::{ThemeState, TorrentSession};
use crate::application::background::download::DownloadFlow;
use crate::application::background::preview::PreviewFlow;
use crate::application::background::search::SearchFlow;
use crate::application::downloads::DownloadTracker;
use crate::domain::search::SearchConfig;
use crate::domain::torrent::TorrentId;
use crate::model::{ActiveDownload, TorrentStats};
use crate::ui::Theme;
use crate::{config::Config, module::AppModule, ui};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::Stdout;
use std::time::Duration;

/// The top-level application state.
///
/// Owns the DI module, shared session state, the navigation stack, and
/// the per-feature background flows.  Services are retrieved from
/// `module` through [`shaku::HasComponent::resolve`].
pub struct App {
    pub module: AppModule,
    pub config: Config,
    pub search_config: SearchConfig,
    pub running: bool,
    pub status_message: String,
    pub theme_state: ThemeState,
    pub torrent: TorrentSession,
    pub active_downloads: Vec<ActiveDownload>,
    pub torrent_id: Option<TorrentId>,
    pub stats: Option<TorrentStats>,
    pub browser_query: String,
    pub history_query: String,
    pub preview_flow: PreviewFlow,
    pub download_flow: DownloadFlow,
    pub search_flow: SearchFlow,
    stack: Vec<Box<dyn Screen>>,
    error_message: Option<String>,
    task_busy: bool,
}

impl App {
    /// Create a new application with the given DI module, configuration,
    /// and search configuration.
    ///
    /// Pure state assembly — all I/O (directory creation, file loading)
    /// happens in the composition root before this is called.
    #[must_use]
    pub fn new(module: AppModule, config: Config, search_config: SearchConfig) -> Self {
        let theme = Theme::from_name(&config.theme).unwrap_or_default();

        Self {
            module,
            config,
            search_config,
            running: true,
            status_message: String::new(),
            theme_state: ThemeState { theme },
            torrent: TorrentSession::default(),
            active_downloads: Vec::new(),
            torrent_id: None,
            stats: None,
            browser_query: String::new(),
            history_query: String::new(),
            preview_flow: PreviewFlow::new(),
            download_flow: DownloadFlow::new(),
            search_flow: SearchFlow::new(),
            stack: vec![registry::welcome()],
            error_message: None,
            task_busy: false,
        }
    }

    /// Queue an error that will be shown on the next frame.
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
    }

    /// The top screen, if any.
    #[must_use]
    pub fn top(&self) -> Option<&dyn Screen> {
        self.stack.last().map(|s| s.as_ref())
    }

    /// The top screen downcast to a concrete type.
    #[must_use]
    pub fn top_as<T: Screen>(&self) -> Option<&T> {
        self.stack.last().and_then(|s| s.as_any().downcast_ref::<T>())
    }

    /// Mutable downcast of the top screen.
    #[must_use]
    pub fn top_as_mut<T: Screen>(&mut self) -> Option<&mut T> {
        self.stack.last_mut().and_then(|s| s.as_any_mut().downcast_mut::<T>())
    }

    /// All screens, bottom to top.
    pub fn screens(&self) -> impl Iterator<Item = &dyn Screen> {
        self.stack.iter().map(|s| s.as_ref())
    }

    /// Push a screen onto the navigation stack.
    pub fn push_screen(&mut self, screen: Box<dyn Screen>) {
        self.stack.push(screen);
    }

    /// Apply a navigation transition.
    pub fn apply_transition(&mut self, transition: Transition) {
        match transition {
            Transition::Push(screen) => self.stack.push(screen),
            Transition::Pop => {
                if self.stack.len() > 1 {
                    self.stack.pop();
                }
            }
            Transition::Replace(screen) => {
                if let Some(top) = self.stack.last_mut() {
                    *top = screen;
                }
            }
            Transition::ResetWith(screen) => {
                self.stack.truncate(1);
                self.stack.push(screen);
            }
            Transition::Quit => self.running = false,
        }
    }

    /// Position of the topmost screen satisfying `pred` in the stack.
    pub fn screen_position(&self, pred: impl Fn(&dyn Screen) -> bool) -> screens::ScreenPosition {
        let Some(idx) = self.stack.iter().rposition(|s| pred(s.as_ref())) else {
            return screens::ScreenPosition::Absent;
        };
        if idx + 1 == self.stack.len() {
            screens::ScreenPosition::Top
        } else {
            screens::ScreenPosition::Below
        }
    }

    /// Remove the topmost screen satisfying `pred`, wherever it sits in
    /// the stack.  The root screen is never removed.
    pub fn remove_screen_if(&mut self, pred: impl Fn(&dyn Screen) -> bool) {
        let Some(idx) = self.stack.iter().rposition(|s| pred(s.as_ref())) else {
            return;
        };
        if idx == 0 && self.stack.len() == 1 {
            return;
        }
        self.stack.remove(idx);
    }

    /// Dispatch one key press through the screen stack: global
    /// shortcuts first, then the top screen.
    pub fn handle_key(&mut self, key: KeyEvent) {
        let top_is_modal = self.top().is_some_and(|s| s.is_modal());

        if !top_is_modal {
            if self.task_busy && !matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                return;
            }

            match key.code {
                KeyCode::Char('t') => {
                    let theme = self.theme_state.theme;
                    self.stack.push(registry::theme_picker(theme));
                    return;
                }
                KeyCode::Char('s') => {
                    let config = self.search_config.clone();
                    self.stack.push(registry::search(config));
                    return;
                }
                _ => {}
            }
        }

        let mut top = match self.stack.pop() {
            Some(top) => top,
            None => return,
        };
        let transition = top.handle_key(self, key);
        self.stack.push(top);
        if let Some(transition) = transition {
            self.apply_transition(transition);
        }
    }

    /// Spawn a background task that performs a search query.
    pub fn enqueue_search(&self, query: String, page: u32) {
        self.search_flow.spawn(self, query, page);
    }

    /// Spawn a background task that fetches torrent metadata for `url`.
    pub fn enqueue_preview(&self, url: String) {
        self.preview_flow.spawn(self, url);
    }

    /// Spawn a background task that downloads a single file for
    /// streaming, then resolves the stream URL.  Returns the silo id
    /// of the download.
    pub fn enqueue_watch(&mut self, url: String, file_index: usize, file_name: String) -> u64 {
        let mut flow = std::mem::take(&mut self.download_flow);
        let id = flow.spawn_watch(self, url, file_index, file_name);
        self.download_flow = flow;
        id
    }

    /// Spawn a background task that downloads the selected files to
    /// `output_dir`.  Returns the silo id of the download.
    pub fn enqueue_download_batch(
        &mut self,
        url: String,
        files: Vec<crate::domain::torrent::TorrentFile>,
        output_dir: std::path::PathBuf,
    ) -> u64 {
        let mut flow = std::mem::take(&mut self.download_flow);
        let id = flow.spawn_batch(self, url, files, output_dir);
        self.download_flow = flow;
        id
    }
}

// ── Run loop ──

/// Run the main application loop.
///
/// Drives the `TUI` with ratatui-crossterm, drains the background
/// flows, and dispatches keyboard input to the top screen.
pub fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    module: AppModule,
    config: Config,
    search_config: SearchConfig,
) -> Result<()> {
    let mut app = App::new(module, config, search_config);
    let tracker = DownloadTracker::new();

    while app.running {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Drain background flows, applying events in arrival batches.
        let mut handled = 0;
        for event in app.preview_flow.drain() {
            crate::application::background::preview::apply(&mut app, event);
            handled += 1;
        }
        for event in app.download_flow.drain() {
            crate::application::background::download::apply(&mut app, event);
            handled += 1;
        }
        for event in app.search_flow.drain() {
            crate::application::background::search::apply(&mut app, event);
            handled += 1;
        }
        if handled > 0 {
            app.task_busy = false;
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key(key);
                }
                _ => {}
            }
        }

        tracker.tick(&mut app);

        if let Some(msg) = app.error_message.take() {
            app.status_message = msg;
        }
    }

    Ok(())
}
