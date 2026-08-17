//! The torrent history screen.

use super::registry;
use super::rename::RenameTarget;
use super::{Screen, ScreenViewModel, Transition};
use crate::app::App;
use crate::domain::common::fuzzy_match;
use crate::domain::history::HistoryEntry;
use crate::domain::history::ports::HistoryPort;
use crossterm::event::{KeyCode, KeyEvent};
use shaku::HasComponent;
use std::any::Any;
use std::sync::Arc;

/// View model for [`HistoryScreen`].
pub struct ViewModel {
    pub entries: Vec<HistoryEntry>,
    pub selected: usize,
    pub searching: bool,
}

/// Lists persisted history entries.  Data is fetched fresh from the
/// history port each frame (mutations update the store immediately).
#[derive(Debug, Clone, Copy, Default)]
pub struct HistoryScreen {
    selected: usize,
    searching: bool,
}

impl HistoryScreen {
    /// Entries matching the current filter, with their index in the
    /// full list.
    fn visible_entries(&self, app: &App) -> Vec<(usize, HistoryEntry)> {
        let storage: Arc<dyn HistoryPort> = app.module.resolve();
        let all = storage.history();
        if app.history_query.is_empty() {
            return all.into_iter().enumerate().collect();
        }
        all.into_iter()
            .enumerate()
            .filter(|(_, e)| {
                let name = e.custom_name.as_deref().unwrap_or(&e.torrent_name);
                fuzzy_match(&app.history_query, name) || fuzzy_match(&app.history_query, &e.url)
            })
            .collect()
    }

    const fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn move_down(&mut self, app: &App) {
        if self.selected < self.visible_entries(app).len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    fn re_add(&mut self, app: &mut App) -> Option<Transition> {
        let entry = self.visible_entries(app).get(self.selected).map(|(_, e)| e.clone())?;
        app.status_message = "Fetching metadata...".to_string();
        app.task_busy = true;
        app.enqueue_preview(entry.url);
        Some(Transition::Push(registry::loading()))
    }

    fn start_rename(&self, app: &mut App) -> Option<Transition> {
        let (orig_index, entry) = self.visible_entries(app).get(self.selected)?.clone();
        let name = entry.custom_name.as_deref().unwrap_or(&entry.torrent_name).to_string();
        app.status_message = format!("Renaming: {name} (Enter to confirm, Esc to cancel)");
        Some(Transition::Push(registry::rename(RenameTarget::History { index: orig_index }, name)))
    }

    fn delete_selected(&mut self, app: &mut App) {
        let visible = self.visible_entries(app);
        let Some((orig_index, _)) = visible.get(self.selected) else {
            return;
        };
        let index = *orig_index;
        let storage: Arc<dyn HistoryPort> = app.module.resolve();
        if let Err(e) = storage.remove_entry(index) {
            app.set_error(format!("Failed to delete: {e}"));
        }
        let all = storage.history();
        if self.selected >= all.len().saturating_sub(1) {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    fn handle_normal(&mut self, app: &mut App, code: KeyCode) -> Option<Transition> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.searching = false;
                app.history_query.clear();
                app.status_message.clear();
                Some(Transition::Pop)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down(app);
                None
            }
            KeyCode::Enter => self.re_add(app),
            KeyCode::Char('r') => self.start_rename(app),
            KeyCode::Char('d') => {
                self.delete_selected(app);
                None
            }
            KeyCode::Char('/') => {
                self.selected = 0;
                app.history_query.clear();
                self.searching = true;
                None
            }
            _ => None,
        }
    }

    fn handle_search(&mut self, app: &mut App, code: KeyCode) -> Option<Transition> {
        match code {
            KeyCode::Esc => {
                self.searching = false;
                app.history_query.clear();
                self.selected = 0;
                None
            }
            KeyCode::Char('q') => {
                self.searching = false;
                app.history_query.clear();
                app.status_message.clear();
                Some(Transition::Pop)
            }
            KeyCode::Enter => {
                self.searching = false;
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
                app.history_query.pop();
                self.selected = 0;
                None
            }
            KeyCode::Char(c) if app.history_query.len() < 256 => {
                app.history_query.push(c);
                self.selected = 0;
                None
            }
            _ => None,
        }
    }
}

impl Screen for HistoryScreen {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        if self.searching {
            self.handle_search(app, key.code)
        } else {
            self.handle_normal(app, key.code)
        }
    }

    fn view<'a>(&'a self, app: &'a App) -> ScreenViewModel<'a> {
        let entries: Vec<HistoryEntry> =
            self.visible_entries(app).into_iter().map(|(_, e)| e).collect();
        ScreenViewModel::History(ViewModel {
            entries,
            selected: self.selected,
            searching: self.searching,
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
