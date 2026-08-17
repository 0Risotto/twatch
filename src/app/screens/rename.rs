//! The rename popup screen.

use super::{Screen, ScreenViewModel, Transition};
use crate::app::App;
use crate::app::state::InputState;
use crate::domain::history::ports::HistoryPort;
use crossterm::event::{KeyCode, KeyEvent};
use shaku::HasComponent;
use std::any::Any;
use std::sync::Arc;

/// What the rename applies to.
#[derive(Debug, Clone, Copy)]
pub enum RenameTarget {
    History { index: usize },
    BrowserFile { file_index: usize },
}

/// View model for [`RenameScreen`].
pub struct ViewModel<'a> {
    pub title: &'static str,
    pub value: &'a str,
    pub cursor: usize,
}

/// Modal text input that renames a history entry or a browser file.
pub struct RenameScreen {
    input: InputState,
    target: RenameTarget,
}

impl RenameScreen {
    #[must_use]
    pub fn new(target: RenameTarget, initial: impl Into<String>) -> Self {
        Self { input: InputState::new(initial), target }
    }
}

impl Screen for RenameScreen {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        match key.code {
            KeyCode::Esc => {
                app.status_message.clear();
                Some(Transition::Pop)
            }
            KeyCode::Enter => {
                let new_name = self.input.value.trim().to_string();
                if new_name.is_empty() || new_name.len() > 256 {
                    return None;
                }
                match self.target {
                    RenameTarget::History { index } => {
                        let storage: Arc<dyn HistoryPort> = app.module.resolve();
                        if let Err(e) = storage.rename_entry(index, &new_name) {
                            app.set_error(format!("Failed to rename: {e}"));
                        }
                    }
                    RenameTarget::BrowserFile { file_index } => {
                        app.torrent.rename_file(file_index, &new_name);
                    }
                }
                app.status_message = format!("Renamed to: {new_name}");
                Some(Transition::Pop)
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
        let title = match self.target {
            RenameTarget::History { .. } => "Rename entry",
            RenameTarget::BrowserFile { .. } => "Rename file",
        };
        ScreenViewModel::Rename(ViewModel {
            title,
            value: &self.input.value,
            cursor: self.input.cursor,
        })
    }

    fn is_modal(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
