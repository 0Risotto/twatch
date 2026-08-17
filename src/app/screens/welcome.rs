//! The welcome menu screen.

use super::registry;
use super::{Screen, ScreenViewModel, Transition};
use crate::app::App;
use crate::model::MenuEntry;
use crossterm::event::{KeyCode, KeyEvent};
use std::any::Any;

/// View model for [`WelcomeScreen`].
#[derive(Debug, Clone, Copy)]
pub struct ViewModel {
    pub menu_selected: usize,
}

/// The root menu: add, search, history, quit.
#[derive(Debug, Clone, Copy, Default)]
pub struct WelcomeScreen {
    menu_selected: usize,
}

impl WelcomeScreen {
    fn open_input(&self, app: &mut App) -> Transition {
        app.torrent.clear();
        app.torrent_id = None;
        app.status_message = "Enter a magnet link or torrent URL".into();
        Transition::Push(registry::input(String::new()))
    }

    fn open_history(&self, app: &mut App) -> Transition {
        app.status_message = "History — Enter: re-add, r: rename, d: delete".into();
        Transition::Push(registry::history())
    }
}

impl Screen for WelcomeScreen {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu_selected = self.menu_selected.saturating_sub(1);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = MenuEntry::ALL.len() - 1;
                if self.menu_selected < max {
                    self.menu_selected += 1;
                }
                None
            }
            KeyCode::Enter => match MenuEntry::ALL[self.menu_selected] {
                MenuEntry::Search => {
                    app.status_message.clear();
                    Some(Transition::Push(registry::search(app.search_config.clone())))
                }
                MenuEntry::Add => Some(self.open_input(app)),
                MenuEntry::History => Some(self.open_history(app)),
                MenuEntry::Quit => Some(Transition::Quit),
            },
            KeyCode::Char('q') => Some(Transition::Quit),
            KeyCode::Char('a') => Some(self.open_input(app)),
            KeyCode::Char('h') => Some(self.open_history(app)),
            _ => None,
        }
    }

    fn view<'a>(&'a self, _app: &'a App) -> ScreenViewModel<'a> {
        ScreenViewModel::Welcome(ViewModel { menu_selected: self.menu_selected })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
