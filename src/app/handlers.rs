//! Keyboard event handlers for each screen.

use crate::app::App;
use crate::model::{DisplayEntry, InputState, MenuEntry, Screen, validate_torrent_input};
use crate::traits::{PlayerService, StorageService};
use crossterm::event::KeyCode;
use shaku::HasComponent;
use std::sync::Arc;

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

// ── Welcome ──

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
                app.display_entries.clear();
                app.expanded_paths.clear();
                app.torrent_id = None;
                app.clear_pending_url();
                app.status_message = "Enter a magnet link or torrent URL".into();
            }
            MenuEntry::History => {
                app.screen = Screen::History;
                app.history_selected = 0;
                app.is_searching = false;
                app.search_query.clear();
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

// ── Browser (with collapsible folder tree + search) ──

fn browser(app: &mut App, code: KeyCode) {
    if app.is_searching {
        return browser_search(app, code);
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.screen = Screen::Welcome;
            app.files.clear();
            app.selected_files.clear();
            app.display_entries.clear();
            app.expanded_paths.clear();
            app.selected_file = 0;
            app.torrent_id = None;
            app.clear_pending_url();
            app.status_message.clear();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let visible = app.visible_entries();
            if !visible.is_empty() && app.selected_file > 0 {
                app.selected_file -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let visible = app.visible_entries();
            if app.selected_file < visible.len().saturating_sub(1) {
                app.selected_file += 1;
            }
        }
        KeyCode::Enter => browser_enter(app),
        KeyCode::Char('w') => watch_file(app),
        KeyCode::Char('d') => download_batch(app),
        KeyCode::Char(' ') => toggle_selection(app),
        KeyCode::Char('r') => start_rename(app),
        KeyCode::Char('/') => {
            app.is_searching = true;
            app.search_query.clear();
        }
        _ => {}
    }
}

fn browser_search(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.is_searching = false;
            app.search_query.clear();
            app.clamp_selection();
        }
        KeyCode::Char('q') => {
            app.is_searching = false;
            app.search_query.clear();
            app.screen = Screen::Welcome;
            app.files.clear();
            app.selected_files.clear();
            app.display_entries.clear();
            app.expanded_paths.clear();
            app.selected_file = 0;
            app.torrent_id = None;
            app.clear_pending_url();
            app.status_message.clear();
        }
        KeyCode::Enter => {
            app.is_searching = false;
            let visible = app.visible_entries();
            if !visible.is_empty() {
                let (_, entry) = &visible[app.selected_file];
                let name = match entry {
                    DisplayEntry::Folder { name, .. } => name.clone(),
                    DisplayEntry::File { file, .. } => file.name.clone(),
                };
                app.search_query = name;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let visible = app.visible_entries();
            if !visible.is_empty() && app.selected_file > 0 {
                app.selected_file -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let visible = app.visible_entries();
            if app.selected_file < visible.len().saturating_sub(1) {
                app.selected_file += 1;
            }
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.selected_file = 0;
        }
        KeyCode::Char(c) if app.search_query.len() < 256 => {
            app.search_query.push(c);
            app.selected_file = 0;
        }
        _ => {}
    }
}

fn browser_enter(app: &mut App) {
    let Some(entry) = app.display_entries.get(app.selected_file).cloned() else {
        return;
    };
    match entry {
        DisplayEntry::Folder { name, depth, expanded: _ } => {
            let full = folder_path(app, depth, &name);
            if app.expanded_paths.contains(&full) {
                app.expanded_paths.remove(&full);
            } else {
                app.expanded_paths.insert(full);
            }
            app.rebuild_entries();
        }
        DisplayEntry::File { .. } => {
            watch_file(app);
        }
    }
}

fn folder_path(app: &App, depth: usize, name: &str) -> String {
    let mut ancestors: Vec<&str> = Vec::with_capacity(depth);
    for i in (0..app.selected_file).rev() {
        if let Some(DisplayEntry::Folder { name: n, depth: d, .. }) = app.display_entries.get(i) {
            if *d < depth && ancestors.len() < depth {
                ancestors.push(n.as_str());
            }
        }
    }
    ancestors.reverse();
    ancestors.push(name);

    let mut full = String::new();
    for seg in ancestors {
        full.push_str(seg);
        full.push('/');
    }
    full
}

fn watch_file(app: &mut App) {
    let visible = app.visible_entries();
    let file = match visible.get(app.selected_file).map(|(_, e)| e) {
        Some(DisplayEntry::File { file, .. }) => file.clone(),
        _ => {
            app.set_error("Select a file to watch, not a folder.");
            return;
        }
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

fn toggle_selection(app: &mut App) {
    let visible = app.visible_entries();
    let idx = match visible.get(app.selected_file).map(|(_, e)| e) {
        Some(DisplayEntry::File { file, .. }) => file.index,
        _ => return,
    };
    if let Some(sel) = app.selected_files.get_mut(idx) {
        *sel = !*sel;
    }
}

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

fn start_rename(app: &mut App) {
    let file = {
        let visible = app.visible_entries();
        match visible.get(app.selected_file).map(|(_, e)| *e) {
            Some(DisplayEntry::File { file, .. }) => file.clone(),
            _ => return,
        }
    };
    app.renaming = true;
    app.rename_input = InputState { value: file.name.clone(), cursor: file.name.len() };
    app.status_message = format!("Renaming: {} (Enter to confirm, Esc to cancel)", file.name);
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
            if app.history_selected > 0 {
                app.history_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.history_selected < visible.len().saturating_sub(1) {
                app.history_selected += 1;
            }
        }
        KeyCode::Enter => {
            if let Some((_, entry)) = visible.get(app.history_selected) {
                let url = entry.url.clone();
                app.screen = Screen::Loading;
                app.status_message = "Fetching metadata...".to_string();
                app.task_busy = true;
                app.enqueue_preview(url);
            }
        }
        KeyCode::Char('r') => {
            if let Some((orig_idx, entry)) = visible.get(app.history_selected) {
                app.history_orig_index = *orig_idx;
                let name = entry.custom_name.as_deref().unwrap_or(&entry.torrent_name);
                app.renaming = true;
                app.rename_input = InputState { value: name.to_string(), cursor: name.len() };
                app.status_message = format!("Renaming: {name} (Enter to confirm, Esc to cancel)");
            }
        }
        KeyCode::Char('d') => {
            if let Some((orig_idx, _)) = visible.get(app.history_selected) {
                let idx = *orig_idx;
                let storage: Arc<dyn StorageService> = app.module.resolve();
                if let Err(e) = storage.remove_entry(idx) {
                    app.set_error(format!("Failed to delete: {e}"));
                }
                // Re-fetch after deletion.
                let all = storage.history();
                if app.history_selected >= all.len().saturating_sub(1) {
                    app.history_selected = app.history_selected.saturating_sub(1);
                }
            }
        }
        KeyCode::Char('/') => {
            app.is_searching = true;
            app.history_selected = 0;
            app.search_query.clear();
        }
        _ => {}
    }
}

fn history_search(app: &mut App, code: KeyCode) {
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
            app.history_selected = 0;
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
            if app.history_selected > 0 {
                app.history_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.history_selected < visible.len().saturating_sub(1) {
                app.history_selected += 1;
            }
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.history_selected = 0;
        }
        KeyCode::Char(c) if app.search_query.len() < 256 => {
            app.search_query.push(c);
            app.history_selected = 0;
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
                    let idx = app.history_orig_index;
                    if let Err(e) = storage.rename_entry(idx, &new_name) {
                        app.set_error(format!("Failed to rename: {e}"));
                    }
                    app.status_message = format!("Renamed to: {new_name}");
                }
                Screen::Browser => {
                    let visible = app.visible_entries();
                    let idx = match visible.get(app.selected_file).map(|(_, e)| e) {
                        Some(DisplayEntry::File { file, .. }) => file.index,
                        _ => usize::MAX,
                    };
                    if let Some(file) = app.files.get_mut(idx) {
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
