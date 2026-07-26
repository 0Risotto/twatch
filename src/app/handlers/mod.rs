//! Keyboard event handlers for each screen.

mod browser;
mod history;
mod input;
mod player;
mod rename;
mod search;
mod text;
mod theme;
mod welcome;

use crate::app::App;
use crate::model::Screen;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let code = key.code;

    if app.renaming {
        return rename::rename_input(app, code);
    }

    if app.theme_state.picker {
        return theme::theme_picker_input(app, code);
    }

    if app.search_popup.open {
        return search::search_input(app, key);
    }

    if app.task_busy && !matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
        return;
    }

    if code == KeyCode::Char('t') {
        theme::open_theme_picker(app);
        return;
    }

    if code == KeyCode::Char('s') {
        search::open_search(app);
        return;
    }

    match app.screen {
        Screen::Welcome => welcome::welcome(app, code),
        Screen::Input => input::input(app, code),
        Screen::Loading => input::loading(app, code),
        Screen::Browser => browser::browser(app, code),
        Screen::Player => player::player(app, code),
        Screen::History => history::history(app, code),
    }
}
