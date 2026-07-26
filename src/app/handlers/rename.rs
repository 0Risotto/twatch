use super::text::{backspace, cursor_left, push_char};
use crate::app::App;
use crate::model::{DisplayEntry, Screen};
use crate::traits::StorageService;
use crossterm::event::KeyCode;
use shaku::HasComponent;
use std::sync::Arc;
pub(crate) fn rename_input(app: &mut App, code: KeyCode) {
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
