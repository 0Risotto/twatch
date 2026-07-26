use crate::app::App;
use crate::model::Screen;
use crate::traits::PlayerService;
use crossterm::event::KeyCode;
use shaku::HasComponent;
use std::sync::Arc;
pub(crate) fn player(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            let player: Arc<dyn PlayerService> = app.module.resolve();
            player.kill();
            app.screen = Screen::Browser;
            app.status_message = "Playback stopped".into();
        }
        _ => {}
    }
}
