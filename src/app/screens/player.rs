//! The now-playing screen.

use super::{Screen, Transition};
use crate::app::App;
use crate::domain::player::ports::PlayerPort;
use crate::domain::torrent::TorrentStats;
use crossterm::event::{KeyCode, KeyEvent};
use shaku::HasComponent;
use std::any::Any;
use std::sync::Arc;

/// View model for [`PlayerScreen`].
#[derive(Debug, Clone, Copy)]
pub struct ViewModel<'a> {
    pub stats: Option<&'a TorrentStats>,
    pub status: &'a str,
}

/// Playback progress + stats.  Esc kills the player and returns to
/// the screen below (the browser).
pub struct PlayerScreen;

impl Screen for PlayerScreen {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                let player: Arc<dyn PlayerPort> = app.module.resolve();
                player.kill();
                app.status_message = "Playback stopped".into();
                Some(Transition::Pop)
            }
            _ => None,
        }
    }

    fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
        crate::ui::player::render(frame, area, app, self.view(app));
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl PlayerScreen {
    /// Build the view model rendered by the presentation layer.
    #[must_use]
    pub fn view<'a>(&'a self, app: &'a App) -> ViewModel<'a> {
        ViewModel { stats: app.stats.as_ref(), status: &app.status_message }
    }
}
