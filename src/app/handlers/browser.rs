use crate::app::App;
use crate::model::{DisplayEntry, InputState, Screen};
use crate::traits::StorageService;
use crossterm::event::KeyCode;
use shaku::HasComponent;
use std::fs;
use std::sync::Arc;
pub(crate) fn browser(app: &mut App, code: KeyCode) {
    if app.is_searching {
        return browser_search(app, code);
    }

    if app.browser.confirm_delete {
        return browser_confirm_delete(app, code);
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.screen = Screen::Welcome;
            app.browser.files.clear();
            app.browser.selected_files.clear();
            app.browser.display_entries.clear();
            app.browser.expanded_paths.clear();
            app.browser.selected_file = 0;
            app.torrent_id = None;
            app.clear_pending_url();
            app.status_message.clear();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let visible = app.visible_entries();
            if !visible.is_empty() && app.browser.selected_file > 0 {
                app.browser.selected_file -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let visible = app.visible_entries();
            if app.browser.selected_file < visible.len().saturating_sub(1) {
                app.browser.selected_file += 1;
            }
        }
        KeyCode::Enter => browser_enter(app),
        KeyCode::Char('w') => watch_file(app),
        KeyCode::Char('d') => download_batch(app),
        KeyCode::Char('x') => {
            let indices = app.selected_indices();
            if indices.is_empty() {
                app.set_error("No files selected. Press Space to toggle selection.");
            } else {
                app.browser.confirm_delete = true;
                app.browser.confirm_delete_yes = false;
            }
        }
        KeyCode::Char(' ') => toggle_selection(app),
        KeyCode::Char('r') => start_rename(app),
        KeyCode::Char('/') => {
            app.is_searching = true;
            app.search_query.clear();
        }
        _ => {}
    }
}

pub(crate) fn browser_search(app: &mut App, code: KeyCode) {
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
            app.browser.files.clear();
            app.browser.selected_files.clear();
            app.browser.display_entries.clear();
            app.browser.expanded_paths.clear();
            app.browser.selected_file = 0;
            app.torrent_id = None;
            app.clear_pending_url();
            app.status_message.clear();
        }
        KeyCode::Enter => {
            app.is_searching = false;
            let visible = app.visible_entries();
            if !visible.is_empty() {
                let (_, entry) = &visible[app.browser.selected_file];
                let name = match entry {
                    DisplayEntry::Folder { name, .. } => name.clone(),
                    DisplayEntry::File { file, .. } => file.name.clone(),
                };
                app.search_query = name;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let visible = app.visible_entries();
            if !visible.is_empty() && app.browser.selected_file > 0 {
                app.browser.selected_file -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let visible = app.visible_entries();
            if app.browser.selected_file < visible.len().saturating_sub(1) {
                app.browser.selected_file += 1;
            }
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.browser.selected_file = 0;
        }
        KeyCode::Char(c) if app.search_query.len() < 256 => {
            app.search_query.push(c);
            app.browser.selected_file = 0;
        }
        _ => {}
    }
}

pub(crate) fn browser_enter(app: &mut App) {
    let Some(entry) = app.browser.display_entries.get(app.browser.selected_file).cloned() else {
        return;
    };
    match entry {
        DisplayEntry::Folder { name, depth, expanded: _ } => {
            let full = folder_path(app, depth, &name);
            if app.browser.expanded_paths.contains(&full) {
                app.browser.expanded_paths.remove(&full);
            } else {
                app.browser.expanded_paths.insert(full);
            }
            app.rebuild_entries();
        }
        DisplayEntry::File { .. } => {
            watch_file(app);
        }
    }
}

pub(crate) fn folder_path(app: &App, depth: usize, name: &str) -> String {
    let mut ancestors: Vec<&str> = Vec::with_capacity(depth);
    for i in (0..app.browser.selected_file).rev() {
        if let Some(DisplayEntry::Folder { name: n, depth: d, .. }) =
            app.browser.display_entries.get(i)
        {
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

pub(crate) fn watch_file(app: &mut App) {
    let visible = app.visible_entries();
    let file = match visible.get(app.browser.selected_file).map(|(_, e)| e) {
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

pub(crate) fn browser_confirm_delete(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter | KeyCode::Char('y') => {
            if !app.browser.confirm_delete_yes {
                app.browser.confirm_delete_yes = true;
            }
            delete_selected_files(app);
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('n') => {
            app.browser.confirm_delete = false;
            app.browser.confirm_delete_yes = false;
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.browser.confirm_delete_yes = !app.browser.confirm_delete_yes;
        }
        KeyCode::Char('h') => {
            app.browser.confirm_delete_yes = false;
        }
        KeyCode::Char('l') => {
            app.browser.confirm_delete_yes = true;
        }
        _ => {}
    }
}

pub(crate) fn delete_selected_files(app: &mut App) {
    let indices = app.selected_indices();
    let download_dir = app.config.download_dir.clone();
    let url = app.pending_url().map(|s| s.to_string()).unwrap_or_default();
    let storage: Arc<dyn StorageService> = app.module.resolve();
    let mut deleted = 0;
    let mut failed: Vec<String> = Vec::new();

    for i in &indices {
        if let Some(f) = app.browser.files.get(*i) {
            let path = download_dir.join(&f.name);
            match fs::remove_file(&path) {
                Ok(()) => {
                    deleted += 1;
                    storage.mark_deleted(&url, &f.name);
                    app.browser.watched_files.retain(|wf| wf != &f.name);
                    app.browser.downloaded_files.retain(|df| df != &f.name);
                    app.browser.downloading_files.retain(|df| df != &f.name);
                }
                Err(_) => failed.push(f.name.clone()),
            }
        }
    }

    if failed.is_empty() {
        app.status_message = format!("Deleted {} file(s)", deleted);
    } else {
        app.status_message =
            format!("Deleted {} file(s), {} failed: {}", deleted, failed.len(), failed.join(", "));
    }
    app.browser.confirm_delete = false;
    app.browser.confirm_delete_yes = false;
}

pub(crate) fn toggle_selection(app: &mut App) {
    let visible = app.visible_entries();
    let idx = match visible.get(app.browser.selected_file).map(|(_, e)| e) {
        Some(DisplayEntry::File { file, .. }) => file.index,
        _ => return,
    };
    if let Some(sel) = app.browser.selected_files.get_mut(idx) {
        *sel = !*sel;
    }
}

pub(crate) fn download_batch(app: &mut App) {
    let indices = app.selected_indices();
    if indices.is_empty() {
        app.set_error("No files selected. Press Space to toggle selection.");
        return;
    }
    let Some(url) = app.pending_url().map(|s| s.to_string()) else {
        app.set_error("No pending URL");
        return;
    };
    let label = if indices.len() == 1 {
        app.browser.files.get(indices[0]).map(|f| f.name.clone()).unwrap_or_default()
    } else {
        format!("{} files", indices.len())
    };
    app.screen = Screen::Loading;
    app.status_message = format!("Downloading: {label}");
    app.task_busy = true;
    app.enqueue_download_batch(url, indices, label, app.config.download_dir.clone());
}

pub(crate) fn start_rename(app: &mut App) {
    let file = {
        let visible = app.visible_entries();
        match visible.get(app.browser.selected_file).map(|(_, e)| *e) {
            Some(DisplayEntry::File { file, .. }) => file.clone(),
            _ => return,
        }
    };
    app.renaming = true;
    app.rename_input = InputState { value: file.name.clone(), cursor: file.name.len() };
    app.status_message = format!("Renaming: {} (Enter to confirm, Esc to cancel)", file.name);
}
