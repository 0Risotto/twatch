//! The torrent file browser screen.

use super::registry;
use super::rename::RenameTarget;
use super::{Screen, Transition};
use crate::app::App;
use crate::domain::torrent::file_tree::entry_matches;
use crate::domain::torrent::{DisplayEntry, FileStatus, TorrentFile};
use crossterm::event::{KeyCode, KeyEvent};
use std::any::Any;

/// The browser's input mode.
#[derive(Debug, Clone, Default)]
pub enum Mode {
    /// Normal file navigation.
    #[default]
    Normal,
    /// Inline file-name filtering (`/`).
    Searching,
    /// Delete confirmation dialog.
    ConfirmDelete { yes: bool },
}

/// View model for [`BrowserScreen`].
pub struct ViewModel<'a> {
    pub title: &'a str,
    pub selected_count: usize,
    pub cursor: usize,
    pub entries: Vec<EntryViewModel<'a>>,
    pub searching: Option<&'a str>,
    pub search_matches: Option<usize>,
    pub confirm: Option<ConfirmViewModel<'a>>,
}

/// One row of the file tree.
pub enum EntryViewModel<'a> {
    Folder { name: &'a str, depth: usize, expanded: bool },
    File { file: &'a TorrentFile, depth: usize, status: FileStatus, selected: bool },
}

/// The delete-confirmation dialog data.
pub struct ConfirmViewModel<'a> {
    pub files: Vec<&'a str>,
    pub total: usize,
    pub yes: bool,
}

/// Browses the current torrent session's file tree.
#[derive(Debug, Clone, Default)]
pub struct BrowserScreen {
    cursor: usize,
    mode: Mode,
}

impl BrowserScreen {
    /// The flat entries that pass the current search filter.
    fn visible_entries<'a>(&'a self, app: &'a App) -> Vec<(usize, &'a DisplayEntry)> {
        match &self.mode {
            Mode::Searching => app
                .torrent
                .tree
                .entries()
                .iter()
                .enumerate()
                .filter(|(_, e)| entry_matches(&app.browser_query, e))
                .collect(),
            _ => app.torrent.tree.entries().iter().enumerate().collect(),
        }
    }

    fn clamp_cursor(&mut self, app: &App) {
        let visible = self.visible_entries(app);
        if visible.is_empty() {
            self.cursor = 0;
            return;
        }
        if self.cursor >= visible.len() {
            self.cursor = visible.len() - 1;
        }
    }

    const fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_down(&mut self, app: &App) {
        if self.cursor < self.visible_entries(app).len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    /// The file under the cursor, if any.
    fn selected_file<'a>(&'a self, app: &'a App) -> Option<&'a TorrentFile> {
        let visible = self.visible_entries(app);
        match visible.get(self.cursor).map(|(_, e)| *e) {
            Some(DisplayEntry::File { file, .. }) => Some(file),
            _ => None,
        }
    }

    /// Expand or collapse the folder under the cursor.
    fn toggle_folder(&mut self, app: &mut App) {
        app.torrent.tree.toggle_folder_at(self.cursor);
        self.clamp_cursor(app);
    }

    /// Watch (stream) the file under the cursor.
    fn watch_selected(&mut self, app: &mut App) -> Option<Transition> {
        let Some(file) = self.selected_file(app).cloned() else {
            app.set_error("Select a file to watch, not a folder.");
            return None;
        };
        let Some(url) = app.torrent.pending_url.clone() else {
            app.set_error("No pending URL");
            return None;
        };
        app.status_message = format!("Streaming: {}", file.name);
        app.task_busy = true;
        let task_id = app.enqueue_watch(url, file.index, file.name);
        Some(Transition::Push(registry::loading_task(task_id)))
    }

    /// Download the toggled files to the download directory.
    fn download_selected(&mut self, app: &mut App) -> Option<Transition> {
        let indices = app.torrent.selected_indices();
        if indices.is_empty() {
            app.set_error("No files selected. Press Space to toggle selection.");
            return None;
        }
        let Some(url) = app.torrent.pending_url.clone() else {
            app.set_error("No pending URL");
            return None;
        };
        let files: Vec<TorrentFile> =
            indices.iter().filter_map(|i| app.torrent.tree.files().get(*i).cloned()).collect();
        let label =
            if files.len() == 1 { files[0].name.clone() } else { format!("{} files", files.len()) };
        app.status_message = format!("Downloading: {label}");
        let task_id = app.enqueue_download_batch(url, files, app.config.download_dir.clone());
        Some(Transition::Push(registry::loading_task(task_id)))
    }

    /// Toggle the selection flag of the file under the cursor.
    fn toggle_selection(&mut self, app: &mut App) {
        if let Some(file) = self.selected_file(app) {
            app.torrent.toggle_selection(file.index);
        }
    }

    /// Leave the browser: clear the session and go back to the root.
    fn leave(&self, app: &mut App) -> Transition {
        app.torrent.clear();
        app.torrent_id = None;
        app.status_message.clear();
        Transition::Pop
    }

    /// Start renaming the file under the cursor.
    fn start_rename(&self, app: &mut App) -> Option<Transition> {
        let file = self.selected_file(app).cloned()?;
        let target = RenameTarget::BrowserFile { file_index: file.index };
        app.status_message = format!("Renaming: {} (Enter to confirm, Esc to cancel)", file.name);
        Some(Transition::Push(registry::rename(target, file.name)))
    }

    fn handle_normal(&mut self, app: &mut App, code: KeyCode) -> Option<Transition> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => Some(self.leave(app)),
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down(app);
                None
            }
            KeyCode::Enter => {
                if matches!(
                    self.visible_entries(app).get(self.cursor).map(|(_, e)| *e),
                    Some(DisplayEntry::Folder { .. })
                ) {
                    self.toggle_folder(app);
                    None
                } else {
                    self.watch_selected(app)
                }
            }
            KeyCode::Char('w') => self.watch_selected(app),
            KeyCode::Char('d') => self.download_selected(app),
            KeyCode::Char('x') => {
                if app.torrent.selected_indices().is_empty() {
                    app.set_error("No files selected. Press Space to toggle selection.");
                    None
                } else {
                    self.mode = Mode::ConfirmDelete { yes: false };
                    None
                }
            }
            KeyCode::Char(' ') => {
                self.toggle_selection(app);
                None
            }
            KeyCode::Char('r') => self.start_rename(app),
            KeyCode::Char('/') => {
                app.browser_query.clear();
                self.mode = Mode::Searching;
                None
            }
            _ => None,
        }
    }

    fn handle_search(&mut self, app: &mut App, code: KeyCode) -> Option<Transition> {
        match code {
            KeyCode::Esc => {
                app.browser_query.clear();
                self.mode = Mode::Normal;
                self.clamp_cursor(app);
                None
            }
            KeyCode::Char('q') => {
                app.browser_query.clear();
                self.mode = Mode::Normal;
                Some(self.leave(app))
            }
            KeyCode::Enter => {
                self.mode = Mode::Normal;
                let visible = self.visible_entries(app);
                if !visible.is_empty() {
                    let (_, entry) = &visible[self.cursor.min(visible.len() - 1)];
                    let name = match entry {
                        DisplayEntry::Folder { name, .. } => name.clone(),
                        DisplayEntry::File { file, .. } => file.name.clone(),
                    };
                    app.browser_query = name;
                }
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down(app);
                None
            }
            KeyCode::Backspace => {
                app.browser_query.pop();
                self.cursor = 0;
                None
            }
            KeyCode::Char(c) if app.browser_query.len() < 256 => {
                app.browser_query.push(c);
                self.cursor = 0;
                None
            }
            _ => None,
        }
    }

    fn handle_confirm(&mut self, app: &mut App, code: KeyCode) -> Option<Transition> {
        let Mode::ConfirmDelete { yes } = &mut self.mode else {
            unreachable!("confirm mode expected");
        };
        match code {
            KeyCode::Char('y') => {
                let message = crate::application::files::delete_selected_files(app);
                app.status_message = message;
                self.mode = Mode::Normal;
                None
            }
            // Enter confirms the highlighted option: only delete when
            // "Yes" is selected, otherwise treat as cancel.
            KeyCode::Enter => {
                if *yes {
                    let message = crate::application::files::delete_selected_files(app);
                    app.status_message = message;
                } else {
                    app.status_message = "Delete cancelled".into();
                }
                self.mode = Mode::Normal;
                None
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('n') => {
                self.mode = Mode::Normal;
                None
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                *yes = !*yes;
                None
            }
            KeyCode::Char('h') => {
                *yes = false;
                None
            }
            KeyCode::Char('l') => {
                *yes = true;
                None
            }
            _ => None,
        }
    }
}

impl Screen for BrowserScreen {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        match self.mode.clone() {
            Mode::Normal => self.handle_normal(app, key.code),
            Mode::Searching => self.handle_search(app, key.code),
            Mode::ConfirmDelete { .. } => self.handle_confirm(app, key.code),
        }
    }

    fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
        crate::ui::browser::render(frame, area, app, self.view(app));
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl BrowserScreen {
    /// Build the view model rendered by the presentation layer.
    #[must_use]
    pub fn view<'a>(&'a self, app: &'a App) -> ViewModel<'a> {
        let selected_count = app.torrent.selected_count();
        let title = app.torrent.torrent_name.as_deref().unwrap_or("Torrent");

        let visible = self.visible_entries(app);
        let entries = visible
            .iter()
            .map(|(_, entry)| match entry {
                DisplayEntry::Folder { name, depth, expanded } => {
                    EntryViewModel::Folder { name, depth: *depth, expanded: *expanded }
                }
                DisplayEntry::File { file, depth } => EntryViewModel::File {
                    file,
                    depth: *depth,
                    selected: app.torrent.selected_files.get(file.index).copied().unwrap_or(false),
                    status: app.torrent.file_status(&file.name),
                },
            })
            .collect();

        let (searching, search_matches) = match &self.mode {
            Mode::Searching => (Some(app.browser_query.as_str()), Some(visible.len())),
            _ => (None, None),
        };

        let confirm = match &self.mode {
            Mode::ConfirmDelete { yes } => {
                let files: Vec<&str> = app
                    .torrent
                    .selected_indices()
                    .iter()
                    .filter_map(|i| app.torrent.tree.files().get(*i).map(|f| f.name.as_str()))
                    .collect();
                Some(ConfirmViewModel { total: files.len(), files, yes: *yes })
            }
            _ => None,
        };

        ViewModel {
            title,
            selected_count,
            cursor: self.cursor,
            entries,
            searching,
            search_matches,
            confirm,
        }
    }
}
