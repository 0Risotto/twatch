#![allow(clippy::unwrap_used)]

//! Rendering regression tests: what actually gets drawn on screen.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use shaku::HasComponent;
use std::sync::Arc;
use twatch::app::App;
use twatch::app::screens::registry;
use twatch::config::Config;
use twatch::config::search::SearchConfig;
use twatch::domain::common::spawner::TaskSpawner;
use twatch::domain::history::ports::HistoryPort;
use twatch::domain::player::ports::PlayerPort;
use twatch::domain::torrent::TorrentFile;
use twatch::domain::torrent::ports::{StreamPort, TorrentPort};
use twatch::module::AppModule;

mod common;

fn mock_app() -> App {
    App::new(
        AppModule::builder()
            .with_component_override::<dyn TorrentPort>(Box::new(
                common::torrent::MockTorrentService::new(),
            ))
            .with_component_override::<dyn StreamPort>(Box::new(
                common::torrent::MockStreamPort::new(),
            ))
            .with_component_override::<dyn TaskSpawner>(Box::new(
                common::task::MockTaskSpawner::new(),
            ))
            .with_component_override::<dyn PlayerPort>(Box::new(
                common::player::MockPlayerService::new(),
            ))
            .with_component_override::<dyn HistoryPort>(Box::new(
                common::storage::MockStorageService::new(),
            ))
            .build(),
        Config::default(),
        SearchConfig::default(),
    )
}

/// Render `app` into a test buffer and return its text content.
fn render(app: &App) -> String {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| twatch::ui::draw(f, app)).unwrap();
    let buf = terminal.backend().buffer();
    let mut text = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let idx = (y * buf.area.width + x) as usize;
            let cell = buf.content.get(idx).map(|c| c.symbol()).unwrap_or(" ");
            text.push_str(cell);
        }
        text.push('\n');
    }
    text
}

#[test]
fn history_screen_renders_entries() {
    let mut app = mock_app();
    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    storage.add_entry("magnet:a", "Torrent A");
    storage.add_entry("magnet:b", "Torrent B");
    app.push_screen(registry::history());

    let text = render(&app);
    assert!(text.contains("Torrent A"), "first entry should be visible");
    assert!(text.contains("Torrent B"), "second entry should be visible");
}

#[test]
fn history_screen_hides_welcome_behind_it() {
    let mut app = mock_app();
    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    storage.add_entry("magnet:a", "Torrent A");
    app.push_screen(registry::history());

    let text = render(&app);
    assert!(text.contains("Torrent A"));
    assert!(
        !text.contains("Terminal torrent streaming client"),
        "welcome screen must not bleed through history"
    );
}

#[test]
fn browser_screen_renders_files_with_badges() {
    let mut app = mock_app();
    app.torrent.set_torrent(
        "Torrent T".into(),
        "magnet:x".into(),
        vec![
            TorrentFile { index: 0, name: "watched.mkv".into(), size: 100 },
            TorrentFile { index: 1, name: "plain.mkv".into(), size: 200 },
        ],
    );
    app.torrent.watched_files = vec!["watched.mkv".into()];
    app.push_screen(registry::browser());

    let text = render(&app);
    assert!(text.contains("watched.mkv"));
    assert!(text.contains("[watched]"));
    assert!(text.contains("plain.mkv"));
}
