#![allow(clippy::unwrap_used)]

use crossterm::event::KeyCode;
use twatch::app::App;
use twatch::config::Config;
use twatch::model::{Screen, TorrentFile};
use twatch::module::AppModule;
use twatch::traits::{PlayerService, StorageService, TorrentService};

mod common;

fn mock_app() -> App {
    let module = AppModule::builder()
        .with_component_override::<dyn TorrentService>(Box::new(
            common::torrent::MockTorrentService::new(),
        ))
        .with_component_override::<dyn PlayerService>(Box::new(
            common::player::MockPlayerService::new(),
        ))
        .with_component_override::<dyn StorageService>(Box::new(
            common::storage::MockStorageService::new(),
        ))
        .build();
    App::new(module, Config::default()).unwrap()
}

#[test]
fn welcome_enter_on_add_navigates_to_input() {
    let mut app = mock_app();
    assert_eq!(app.screen, Screen::Welcome);

    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, Screen::Input);
}

#[test]
fn welcome_enter_on_quit_stops_running() {
    let mut app = mock_app();
    twatch::app::handlers::handle_key(&mut app, KeyCode::Down);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Down);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter);
    assert!(!app.running);
}

#[test]
fn welcome_q_stops_running() {
    let mut app = mock_app();
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('q'));
    assert!(!app.running);
}

#[test]
fn input_esc_returns_to_welcome() {
    let mut app = mock_app();
    app.screen = Screen::Input;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, Screen::Welcome);
}

#[test]
fn input_typing_builds_value() {
    let mut app = mock_app();
    app.screen = Screen::Input;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('h'));
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('i'));
    assert_eq!(app.input.value, "hi");
}

#[test]
fn input_enter_on_non_empty_url_starts_loading() {
    let mut app = mock_app();
    app.screen = Screen::Input;
    app.input.value = "magnet:?xt=urn:btih:abc".into();
    app.input.cursor = app.input.value.len();

    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, Screen::Loading);
}

#[test]
fn input_enter_on_empty_url_is_ignored() {
    let mut app = mock_app();
    app.screen = Screen::Input;
    app.input.value.clear();
    app.input.cursor = 0;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, Screen::Input);
}

#[test]
fn browser_toggle_selection() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a.mkv".into(), size: 100 }];
    app.selected_files = vec![false];

    twatch::app::handlers::handle_key(&mut app, KeyCode::Char(' '));
    assert!(app.selected_files[0]);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char(' '));
    assert!(!app.selected_files[0]);
}

#[test]
fn browser_esc_returns_to_welcome_and_clears_files() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a".into(), size: 1 }];
    app.selected_files = vec![false];

    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, Screen::Welcome);
    assert!(app.files.is_empty());
    assert!(app.selected_files.is_empty());
}

#[test]
fn player_esc_kills_and_returns_to_browser() {
    let mut app = mock_app();
    app.screen = Screen::Player;

    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, Screen::Browser);
    assert!(app.status_message.contains("stopped"));
}

#[test]
fn history_esc_returns_to_welcome() {
    let mut app = mock_app();
    app.screen = Screen::History;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, Screen::Welcome);
}

#[test]
fn history_down_moves_selection() {
    let mut app = mock_app();
    app.screen = Screen::History;

    use shaku::HasComponent;
    use std::sync::Arc;
    let storage: Arc<dyn StorageService> = app.module.resolve();
    storage.add_entry("magnet:a", "A");
    storage.add_entry("magnet:b", "B");

    twatch::app::handlers::handle_key(&mut app, KeyCode::Down);
    assert_eq!(app.history_selected, 1);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Up);
    assert_eq!(app.history_selected, 0);
}
