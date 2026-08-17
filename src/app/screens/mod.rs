//! Screens: self-contained UI state machines.
//!
//! Each screen owns its private state, translates key presses into
//! navigation transitions, and exposes a pure view model for the
//! presentation layer.  The [`registry`] module is the single closed
//! point that enumerates all screens.

pub mod browser;
pub mod history;
pub mod input;
pub mod loading;
pub mod player;
pub mod rename;
pub mod search;
pub mod theme_picker;
pub mod welcome;

pub mod registry;

use crate::app::App;
use crossterm::event::KeyEvent;
use std::any::Any;

/// A navigation transition produced by a screen.
pub enum Transition {
    /// Push a new screen on top of the stack.
    Push(Box<dyn Screen>),
    /// Pop the top screen (never pops the welcome root).
    Pop,
    /// Replace the top screen.
    Replace(Box<dyn Screen>),
    /// Pop to the welcome root, then push a new screen.
    ResetWith(Box<dyn Screen>),
    /// Leave the application.
    Quit,
}

/// A UI screen: private state + key handling + pure view model.
pub trait Screen: Any {
    /// Handle a key press.  Return a navigation transition if any.
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition>;

    /// Build the view model rendered by the presentation layer.
    fn view<'a>(&'a self, app: &'a App) -> ScreenViewModel<'a>;

    /// True for modal overlays: they consume global shortcuts and
    /// render over the full frame.
    fn is_modal(&self) -> bool {
        false
    }

    /// True when the screen renders over the full frame instead of
    /// the main pane.
    fn renders_full_frame(&self) -> bool {
        false
    }

    /// Type-erased access for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Type-erased mutable access for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Per-screen view models — plain data, no presentation logic.
pub enum ScreenViewModel<'a> {
    Welcome(welcome::ViewModel),
    Input(input::ViewModel<'a>),
    Loading(loading::ViewModel<'a>),
    Browser(browser::ViewModel<'a>),
    Player(player::ViewModel<'a>),
    History(history::ViewModel),
    Search(search::ViewModel<'a>),
    ThemePicker(theme_picker::ViewModel<'a>),
    Rename(rename::ViewModel<'a>),
}
