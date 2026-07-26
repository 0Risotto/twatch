use super::search::open_search;
use crate::app::App;
use crate::model::{InputState, MenuEntry, Screen};
use crossterm::event::KeyCode;
pub(crate) fn welcome(app: &mut App, code: KeyCode) {
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
            MenuEntry::Search => {
                open_search(app);
            }
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
        KeyCode::Char('a') => {
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
        KeyCode::Char('h') => {
            app.screen = Screen::History;
            app.history_selected = 0;
            app.is_searching = false;
            app.search_query.clear();
            app.status_message = "History — Enter: re-add, r: rename, d: delete".into();
        }
        _ => {}
    }
}
