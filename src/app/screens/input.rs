//! The magnet/URL input screen.

use super::registry;
use super::{Screen, ScreenViewModel, Transition};
use crate::app::App;
use crate::app::state::InputState;
use crate::domain::common::validate_torrent_input;
use crossterm::event::{KeyCode, KeyEvent};
use std::any::Any;

/// View model for [`InputScreen`].
#[derive(Debug, Clone, Copy)]
pub struct ViewModel<'a> {
    pub value: &'a str,
    pub cursor: usize,
}

/// Text entry for a magnet link or torrent URL.
pub struct InputScreen {
    input: InputState,
}

impl InputScreen {
    #[must_use]
    pub fn new(initial: impl Into<String>) -> Self {
        Self { input: InputState::new(initial) }
    }
}

impl Screen for InputScreen {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        match key.code {
            KeyCode::Esc => {
                app.status_message.clear();
                Some(Transition::Pop)
            }
            KeyCode::Enter => {
                if app.task_busy {
                    return None;
                }
                let url = self.input.value.trim().to_string();
                if let Err(msg) = validate_torrent_input(&url) {
                    app.status_message = msg.to_string();
                    return None;
                }
                app.status_message = "Fetching metadata...".to_string();
                app.task_busy = true;
                app.enqueue_preview(url);
                Some(Transition::Push(registry::loading()))
            }
            KeyCode::Char(c) => {
                self.input.push_char(c);
                None
            }
            KeyCode::Backspace => {
                self.input.backspace();
                None
            }
            KeyCode::Left => {
                self.input.cursor_left();
                None
            }
            KeyCode::Right => {
                self.input.cursor_right();
                None
            }
            _ => None,
        }
    }

    fn view<'a>(&'a self, _app: &'a App) -> ScreenViewModel<'a> {
        ScreenViewModel::Input(ViewModel { value: &self.input.value, cursor: self.input.cursor })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
