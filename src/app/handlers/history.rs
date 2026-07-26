use crate::app::App;
use crate::model::{InputState, Screen};
use crate::traits::StorageService;
use crossterm::event::KeyCode;
use shaku::HasComponent;
use std::sync::Arc;
pub(crate) fn history(app: &mut App, code: KeyCode) {
    if app.is_searching {
        return history_search(app, code);
    }

    let storage: Arc<dyn StorageService> = app.module.resolve();
    let all = storage.history();
    let visible: Vec<(usize, _)> = if app.search_query.is_empty() {
        all.iter().enumerate().collect()
    } else {
        all.iter()
            .enumerate()
            .filter(|(_, e)| {
                let name = e.custom_name.as_deref().unwrap_or(&e.torrent_name);
                crate::model::fuzzy_match(&app.search_query, name)
                    || crate::model::fuzzy_match(&app.search_query, &e.url)
            })
            .collect()
    };

    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.screen = Screen::Welcome;
            app.is_searching = false;
            app.search_query.clear();
            app.status_message.clear();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.history.selected > 0 {
                app.history.selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.history.selected < visible.len().saturating_sub(1) {
                app.history.selected += 1;
            }
        }
        KeyCode::Enter => {
            if let Some((_, entry)) = visible.get(app.history.selected) {
                let url = entry.url.clone();
                app.screen = Screen::Loading;
                app.status_message = "Fetching metadata...".to_string();
                app.task_busy = true;
                app.enqueue_preview(url);
            }
        }
        KeyCode::Char('r') => {
            if let Some((orig_idx, entry)) = visible.get(app.history.selected) {
                app.history.orig_index = *orig_idx;
                let name = entry.custom_name.as_deref().unwrap_or(&entry.torrent_name);
                app.renaming = true;
                app.rename_input = InputState { value: name.to_string(), cursor: name.len() };
                app.status_message = format!("Renaming: {name} (Enter to confirm, Esc to cancel)");
            }
        }
        KeyCode::Char('d') => {
            if let Some((orig_idx, _)) = visible.get(app.history.selected) {
                let idx = *orig_idx;
                let storage: Arc<dyn StorageService> = app.module.resolve();
                if let Err(e) = storage.remove_entry(idx) {
                    app.set_error(format!("Failed to delete: {e}"));
                }
                // Re-fetch after deletion.
                let all = storage.history();
                if app.history.selected >= all.len().saturating_sub(1) {
                    app.history.selected = app.history.selected.saturating_sub(1);
                }
            }
        }
        KeyCode::Char('/') => {
            app.is_searching = true;
            app.history.selected = 0;
            app.search_query.clear();
        }
        _ => {}
    }
}

pub(crate) fn history_search(app: &mut App, code: KeyCode) {
    let storage: Arc<dyn StorageService> = app.module.resolve();
    let all = storage.history();
    let visible: Vec<(usize, _)> = if app.search_query.is_empty() {
        all.iter().enumerate().collect()
    } else {
        all.iter()
            .enumerate()
            .filter(|(_, e)| {
                let name = e.custom_name.as_deref().unwrap_or(&e.torrent_name);
                crate::model::fuzzy_match(&app.search_query, name)
                    || crate::model::fuzzy_match(&app.search_query, &e.url)
            })
            .collect()
    };

    match code {
        KeyCode::Esc => {
            app.is_searching = false;
            app.search_query.clear();
            app.history.selected = 0;
        }
        KeyCode::Char('q') => {
            app.is_searching = false;
            app.search_query.clear();
            app.screen = Screen::Welcome;
            app.status_message.clear();
        }
        KeyCode::Enter => {
            app.is_searching = false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.history.selected > 0 {
                app.history.selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.history.selected < visible.len().saturating_sub(1) {
                app.history.selected += 1;
            }
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.history.selected = 0;
        }
        KeyCode::Char(c) if app.search_query.len() < 256 => {
            app.search_query.push(c);
            app.history.selected = 0;
        }
        _ => {}
    }
}
