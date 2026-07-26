use crate::app::App;
use crate::model::{InputState, Screen, SearchResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
pub(crate) fn open_search(app: &mut App) {
    app.search_popup.open = true;
    app.search_popup.query.clear();
    app.search_popup.fetched_query.clear();
    app.search_popup.all_results.clear();
    app.search_popup.selected = 0;
    app.search_popup.page = 0;
    app.search_popup.busy = false;
}

pub(crate) fn search_input(app: &mut App, key: KeyEvent) {
    if app.search_popup.config_open {
        return search_config_input(app, key.code);
    }

    let code = key.code;

    match code {
        KeyCode::Esc => {
            app.search_popup.open = false;
        }
        KeyCode::Enter => {
            let select_result = !current_page_results(app).is_empty()
                && app.search_popup.query == app.search_popup.fetched_query;

            if select_result {
                let name = current_page_results(app)
                    .get(app.search_popup.selected)
                    .map(|r| (r.info_hash.clone(), r.name.clone()));
                if let Some((hash, name)) = name {
                    let magnet = format!("magnet:?xt=urn:btih:{hash}&dn={name}");
                    app.search_popup.open = false;
                    app.input = InputState { value: magnet.clone(), cursor: magnet.len() };
                    app.screen = Screen::Input;
                    app.status_message = format!("Ready: {name}");
                }
            } else {
                let query = app.search_popup.query.clone();
                if !query.is_empty() && !app.search_popup.busy {
                    app.search_popup.busy = true;
                    app.search_popup.page = 0;
                    app.search_popup.fetched_query = query.clone();
                    app.search_popup.all_results.clear();
                    app.search_popup.selected = 0;
                    app.enqueue_search(query, 0);
                }
            }
        }
        KeyCode::Up => {
            let len = current_page_results(app).len();
            if len == 0 {
                return;
            }
            if app.search_popup.selected > 0 {
                app.search_popup.selected -= 1;
            } else {
                app.search_popup.selected = len - 1;
            }
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let len = current_page_results(app).len();
            if len == 0 {
                return;
            }
            if app.search_popup.selected > 0 {
                app.search_popup.selected -= 1;
            } else {
                app.search_popup.selected = len - 1;
            }
        }
        KeyCode::Down => {
            let len = current_page_results(app).len();
            if len == 0 {
                return;
            }
            if app.search_popup.selected < len - 1 {
                app.search_popup.selected += 1;
            } else {
                app.search_popup.selected = 0;
            }
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let len = current_page_results(app).len();
            if len == 0 {
                return;
            }
            if app.search_popup.selected < len - 1 {
                app.search_popup.selected += 1;
            } else {
                app.search_popup.selected = 0;
            }
        }
        KeyCode::Left => {
            if app.search_popup.page > 0 {
                app.search_popup.page -= 1;
                app.search_popup.selected = 0;
            }
        }
        KeyCode::Right => {
            let total_pages = app.search_popup.all_results.len().saturating_add(20) / 21;
            if (app.search_popup.page as usize) < total_pages.saturating_sub(1) {
                app.search_popup.page += 1;
                app.search_popup.selected = 0;
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.search_popup.query.clear();
            app.search_popup.fetched_query.clear();
            app.search_popup.all_results.clear();
            app.search_popup.selected = 0;
            app.search_popup.page = 0;
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.search_popup.config_open = true;
            app.search_popup.config_input = app.search_popup.config.url.clone();
        }
        KeyCode::Backspace => {
            app.search_popup.query.pop();
            clamp_search_selection(app);
        }
        KeyCode::Char(ch) if app.search_popup.query.len() < 256 => {
            app.search_popup.query.push(ch);
            clamp_search_selection(app);
        }
        _ => {}
    }
}

pub(crate) fn current_page_results(app: &App) -> &[SearchResult] {
    let start = app.search_popup.page as usize * 21;
    let all = &app.search_popup.all_results;
    if start >= all.len() {
        return &[];
    }
    let end = (start + 21).min(all.len());
    &all[start..end]
}

pub(crate) fn clamp_search_selection(app: &mut App) {
    let max = current_page_results(app).len().saturating_sub(1);
    if app.search_popup.selected > max {
        app.search_popup.selected = max;
    }
}

pub(crate) fn search_config_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.search_popup.config_open = false;
        }
        KeyCode::Enter => {
            let url = app.search_popup.config_input.trim().to_string();
            app.search_popup.config.url = url;
            app.search_popup.config_open = false;
            let _ = app.search_popup.config.save(&app.config.config_dir);
            app.status_message = "Search API configured".into();
        }
        KeyCode::Backspace => {
            app.search_popup.config_input.pop();
        }
        KeyCode::Char(c) if app.search_popup.config_input.len() < 2048 => {
            app.search_popup.config_input.push(c);
        }
        _ => {}
    }
}
