use super::text::{backspace, cursor_left, push_char};
use crate::app::App;
use crate::model::{Screen, validate_torrent_input};
use crossterm::event::KeyCode;
pub(crate) fn input(app: &mut App, code: KeyCode) {
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

pub(crate) fn loading(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = Screen::Welcome;
            app.status_message.clear();
        }
        _ => {}
    }
}
