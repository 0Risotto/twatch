//! Screen registry — the single closed point that enumerates every
//! screen constructor.
//!
//! Adding a screen = one new module + one row here.  Nothing else in
//! the codebase needs to change.

use super::Screen;
use super::browser::BrowserScreen;
use super::history::HistoryScreen;
use super::input::InputScreen;
use super::loading::LoadingScreen;
use super::player::PlayerScreen;
use super::rename::{RenameScreen, RenameTarget};
use super::search::SearchScreen;
use super::theme_picker::ThemePickerScreen;
use super::welcome::WelcomeScreen;
use crate::config::search::SearchConfig;
use crate::ui::Theme;

/// The root welcome screen.
#[must_use]
pub fn welcome() -> Box<dyn Screen> {
    Box::new(WelcomeScreen::default())
}

/// The magnet/URL input screen, optionally pre-filled.
#[must_use]
pub fn input(initial: impl Into<String>) -> Box<dyn Screen> {
    Box::new(InputScreen::new(initial))
}

/// The loading screen shown while a background task runs.
#[must_use]
pub fn loading() -> Box<dyn Screen> {
    Box::new(LoadingScreen::new(None))
}

/// A loading screen bound to one specific background task (download
/// silo).  It is dismissed only when that task finishes.
#[must_use]
pub fn loading_task(task: u64) -> Box<dyn Screen> {
    Box::new(LoadingScreen::new(Some(task)))
}

/// The torrent file browser for the current session.
#[must_use]
pub fn browser() -> Box<dyn Screen> {
    Box::new(BrowserScreen::default())
}

/// The now-playing screen.
#[must_use]
pub fn player() -> Box<dyn Screen> {
    Box::new(PlayerScreen)
}

/// The history list screen.
#[must_use]
pub fn history() -> Box<dyn Screen> {
    Box::new(HistoryScreen::default())
}

/// The search popup.
#[must_use]
pub fn search(config: SearchConfig) -> Box<dyn Screen> {
    Box::new(SearchScreen::new(config))
}

/// The theme picker popup.
#[must_use]
pub fn theme_picker(current: Theme) -> Box<dyn Screen> {
    Box::new(ThemePickerScreen::new(current))
}

/// The rename popup targeting a history entry or browser file.
#[must_use]
pub fn rename(target: RenameTarget, initial: impl Into<String>) -> Box<dyn Screen> {
    Box::new(RenameScreen::new(target, initial))
}
