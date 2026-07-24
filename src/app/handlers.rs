//! Keyboard event handlers for each screen.
//!
//! Every screen has a dedicated handler function.  Services are
//! resolved from the DI module via [`HasComponent::resolve`](shaku::HasComponent::resolve).

use crate::app::App;
use crate::model::{InputState, MenuEntry, Screen, validate_torrent_input};
use crate::traits::{PlayerService, StorageService};
use crossterm::event::KeyCode;
use shaku::HasComponent;
use std::sync::Arc;

/// Top-level dispatch: routes a key press to the handler for the
/// active screen (or to the rename overlay if active).
pub fn handle_key(app: &mut App, code: KeyCode) {
    if app.renaming {
        return rename_input(app, code);
    }

    if app.task_busy && !matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
        return;
    }

    match app.screen {
        Screen::Welcome => welcome(app, code),
        Screen::Input => input(app, code),
        Screen::Loading => loading(app, code),
        Screen::Browser => browser(app, code),
        Screen::Player => player(app, code),
        Screen::History => history(app, code),
    }
}

// ── Welcome (selectable menu) ──

fn welcome(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.menu_selected > 0 {
                app.menu_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let max = MenuEntry::ALL.len() - 1;
            if app.menu_selected < max {
                app.menu_selected += 1;
            }
        }
        KeyCode::Enter => match MenuEntry::ALL[app.menu_selected] {
            MenuEntry::Add => {
                app.screen = Screen::Input;
                app.input = InputState::default();
                app.files.clear();
                app.selected_files.clear();
                app.torrent_id = None;
                app.clear_pending_url();
                app.status_message = "Enter a magnet link or torrent URL".into();
            }
            MenuEntry::History => {
                app.screen = Screen::History;
                app.history_selected = 0;
                app.status_message = "History — Enter: re-add, r: rename, d: delete".into();
            }
            MenuEntry::Quit => app.running = false,
        },
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

// ── Input ──

fn input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.screen = Screen::Welcome;
            app.status_message.clear();
        }
        KeyCode::Enter => {
            let url = app.input.value.trim().to_string();
            if app.task_busy {
                return;
            }

            if let Err(msg) = validate_torrent_input(&url) {
                app.status_message = msg.to_string();
                return;
            }

            app.screen = Screen::Loading;
            app.status_message = "Fetching metadata...".to_string();
            app.task_busy = true;
            app.enqueue_preview(url);
        }
        KeyCode::Char(c) => push_char(&mut app.input, c),
        KeyCode::Backspace => backspace(&mut app.input),
        KeyCode::Left => cursor_left(&mut app.input),
        KeyCode::Right if app.input.cursor < app.input.value.len() => {
            app.input.cursor += 1;
        }
        _ => {}
    }
}

// ── Loading ──

fn loading(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = Screen::Welcome;
            app.status_message.clear();
        }
        _ => {}
    }
}

// ── Browser (w: watch, Space: toggle, d: download) ──

fn browser(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.screen = Screen::Welcome;
            app.files.clear();
            app.selected_files.clear();
            app.selected_file = 0;
            app.torrent_id = None;
            app.clear_pending_url();
            app.status_message.clear();
        }
        KeyCode::Char('w') => watch_file(app),
        KeyCode::Char('d') => download_batch(app),
        KeyCode::Char(' ') => toggle_selection(app),
        KeyCode::Char('r') => start_rename(app),
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_file > 0 {
                app.selected_file -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j')
            if app.selected_file < app.files.len().saturating_sub(1) =>
        {
            app.selected_file += 1;
        }
        _ => {}
    }
}

/// Enqueue the currently selected file for streaming.
fn watch_file(app: &mut App) {
    let Some(file) = app.files.get(app.selected_file).cloned() else {
        return;
    };
    let Some(url) = app.pending_url().map(|s| s.to_string()) else {
        app.set_error("No pending URL");
        return;
    };

    app.screen = Screen::Loading;
    app.status_message = format!("Streaming: {}", file.name);
    app.task_busy = true;
    app.enqueue_watch(url, file.index, file.name);
}

/// Toggle the selection checkbox for the highlighted file.
fn toggle_selection(app: &mut App) {
    if let Some(sel) = app.selected_files.get_mut(app.selected_file) {
        *sel = !*sel;
    }
}

/// Enqueue all toggled files for batch download.
fn download_batch(app: &mut App) {
    let indices: Vec<usize> =
        app.selected_files.iter().enumerate().filter(|(_, s)| **s).map(|(i, _)| i).collect();

    if indices.is_empty() {
        app.set_error("No files selected. Press Space to toggle selection.");
        return;
    }

    let Some(url) = app.pending_url().map(|s| s.to_string()) else {
        app.set_error("No pending URL");
        return;
    };

    let label = if indices.len() == 1 {
        app.files.get(indices[0]).map(|f| f.name.clone()).unwrap_or_default()
    } else {
        format!("{} files", indices.len())
    };

    app.screen = Screen::Loading;
    app.status_message = format!("Downloading: {label}");
    app.task_busy = true;
    app.enqueue_download_batch(url, indices, label, app.config.download_dir.clone());
}

/// Enter rename mode for the highlighted file.
fn start_rename(app: &mut App) {
    if let Some(file) = app.files.get(app.selected_file) {
        app.renaming = true;
        app.rename_input = InputState { value: file.name.clone(), cursor: file.name.len() };
        app.status_message = format!("Renaming: {} (Enter to confirm, Esc to cancel)", file.name);
    }
}

// ── Player ──

fn player(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            let player: Arc<dyn PlayerService> = app.module.resolve();
            player.kill();
            app.screen = Screen::Browser;
            app.status_message = "Playback stopped".into();
        }
        _ => {}
    }
}

// ── History ──

fn history(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.screen = Screen::Welcome;
            app.status_message.clear();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.history_selected > 0 {
                app.history_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let storage: Arc<dyn StorageService> = app.module.resolve();
            let len = storage.history().len();
            if app.history_selected < len.saturating_sub(1) {
                app.history_selected += 1;
            }
        }
        KeyCode::Enter => {
            let storage: Arc<dyn StorageService> = app.module.resolve();
            let history = storage.history();
            if let Some(entry) = history.get(app.history_selected) {
                let url = entry.url.clone();
                app.screen = Screen::Loading;
                app.status_message = "Fetching metadata...".to_string();
                app.task_busy = true;
                app.enqueue_preview(url);
            }
        }
        KeyCode::Char('r') => {
            let storage: Arc<dyn StorageService> = app.module.resolve();
            let history = storage.history();
            if let Some(entry) = history.get(app.history_selected) {
                let name = entry.custom_name.as_deref().unwrap_or(&entry.torrent_name);
                app.renaming = true;
                app.rename_input = InputState { value: name.to_string(), cursor: name.len() };
                app.status_message = format!("Renaming: {name} (Enter to confirm, Esc to cancel)");
            }
        }
        KeyCode::Char('d') => {
            let storage: Arc<dyn StorageService> = app.module.resolve();
            let idx = app.history_selected;
            if let Err(e) = storage.remove_entry(idx) {
                app.set_error(format!("Failed to delete: {e}"));
            }
            let history = storage.history();
            if app.history_selected >= history.len().saturating_sub(1) {
                app.history_selected = app.history_selected.saturating_sub(1);
            }
        }
        _ => {}
    }
}

// ── Rename mode ──

fn rename_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.renaming = false;
            app.status_message.clear();
            if app.screen == Screen::Browser {
                app.status_message = "Rename cancelled".into();
            }
        }
        KeyCode::Enter => {
            let new_name = app.rename_input.value.trim().to_string();
            if new_name.is_empty() || new_name.len() > 256 {
                return;
            }

            match app.screen {
                Screen::History => {
                    let storage: Arc<dyn StorageService> = app.module.resolve();
                    let idx = app.history_selected;
                    if let Err(e) = storage.rename_entry(idx, &new_name) {
                        app.set_error(format!("Failed to rename: {e}"));
                    }
                    app.status_message = format!("Renamed to: {new_name}");
                }
                Screen::Browser => {
                    if let Some(file) = app.files.get_mut(app.selected_file) {
                        file.name = new_name.clone();
                    }
                    app.status_message = format!("Renamed to: {new_name}");
                }
                _ => {}
            }

            app.renaming = false;
        }
        KeyCode::Char(c) => push_char(&mut app.rename_input, c),
        KeyCode::Backspace => backspace(&mut app.rename_input),
        KeyCode::Left => cursor_left(&mut app.rename_input),
        KeyCode::Right if app.rename_input.cursor < app.rename_input.value.len() => {
            app.rename_input.cursor += 1;
        }
        _ => {}
    }
}

// ── Text-editing helpers ──

/// Maximum length for user-supplied text (URL input and rename).
const MAX_INPUT_LEN: usize = 8192;

fn push_char(input: &mut InputState, c: char) {
    if input.value.len() >= MAX_INPUT_LEN {
        return;
    }
    input.value.push(c);
    input.cursor += 1;
}

fn backspace(input: &mut InputState) {
    if input.cursor > 0 {
        input.value.remove(input.cursor - 1);
        input.cursor -= 1;
    }
}

const fn cursor_left(input: &mut InputState) {
    if input.cursor > 0 {
        input.cursor -= 1;
    }
}
