//! The loading screen shown while a background task runs.

use super::{Screen, ScreenViewModel, Transition};
use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use std::any::Any;

/// View model for [`LoadingScreen`].
#[derive(Debug, Clone, Copy)]
pub struct ViewModel<'a> {
    pub status: &'a str,
}

/// "Please wait" screen.  Esc goes back to wherever the user came
/// from.  When `task` is set, the screen belongs to one specific
/// background task (see download silos) and is dismissed only when
/// that task finishes.
pub struct LoadingScreen {
    task: Option<u64>,
}

impl LoadingScreen {
    /// Create a loading screen for `task`, or a task-agnostic one
    /// when `task` is `None`.
    #[must_use]
    pub const fn new(task: Option<u64>) -> Self {
        Self { task }
    }

    /// The task this screen belongs to, if any.
    #[must_use]
    pub const fn task(&self) -> Option<u64> {
        self.task
    }
}

impl Screen for LoadingScreen {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.status_message.clear();
                Some(Transition::Pop)
            }
            _ => None,
        }
    }

    fn view<'a>(&'a self, app: &'a App) -> ScreenViewModel<'a> {
        ScreenViewModel::Loading(ViewModel { status: &app.status_message })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
