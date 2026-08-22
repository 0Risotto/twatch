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
use ratatui::{Frame, layout::Rect};
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

/// Where a matching screen sits in the navigation stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenPosition {
    /// The screen is the current top.
    Top,
    /// The screen sits below the top.
    Below,
    /// No matching screen exists.
    Absent,
}

/// A UI screen: private state + key handling + rendering delegation.
///
/// Rendering is delegated to a pure renderer in [`crate::ui`]; the
/// screen itself only builds the view model.  Adding a screen means
/// adding its module and one constructor row in [`registry`] —
/// nothing else in the codebase needs to change.
pub trait Screen: Any {
    /// Handle a key press.  Return a navigation transition if any.
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition>;

    /// Render this screen into `area`, delegating to its pure
    /// presentation function in [`crate::ui`].
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);

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
