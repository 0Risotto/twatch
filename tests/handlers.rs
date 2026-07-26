#![allow(clippy::unwrap_used)]

use crossterm::event::KeyCode;
use twatch::app::App;
use twatch::config::Config;
use twatch::model::{DisplayEntry, Screen, TorrentFile};
use twatch::module::AppModule;
use twatch::traits::{PlayerService, StorageService, TorrentService};

use std::fs;
use std::path::PathBuf;

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

    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter.into());
    assert_eq!(app.screen, Screen::Input);
}

#[test]
fn welcome_enter_on_quit_stops_running() {
    let mut app = mock_app();
    twatch::app::handlers::handle_key(&mut app, KeyCode::Down.into());
    twatch::app::handlers::handle_key(&mut app, KeyCode::Down.into());
    twatch::app::handlers::handle_key(&mut app, KeyCode::Down.into());
    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter.into());
    assert!(!app.running);
}

#[test]
fn welcome_q_stops_running() {
    let mut app = mock_app();
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('q').into());
    assert!(!app.running);
}

#[test]
fn input_esc_returns_to_welcome() {
    let mut app = mock_app();
    app.screen = Screen::Input;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc.into());
    assert_eq!(app.screen, Screen::Welcome);
}

#[test]
fn input_typing_builds_value() {
    let mut app = mock_app();
    app.screen = Screen::Input;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('h').into());
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('i').into());
    assert_eq!(app.input.value, "hi");
}

#[test]
fn input_enter_on_non_empty_url_starts_loading() {
    let mut app = mock_app();
    app.screen = Screen::Input;
    app.input.value = "magnet:?xt=urn:btih:abc".into();
    app.input.cursor = app.input.value.len();

    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter.into());
    assert_eq!(app.screen, Screen::Loading);
}

#[test]
fn input_enter_on_empty_url_is_ignored() {
    let mut app = mock_app();
    app.screen = Screen::Input;
    app.input.value.clear();
    app.input.cursor = 0;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter.into());
    assert_eq!(app.screen, Screen::Input);
}

#[test]
fn browser_toggle_selection() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a.mkv".into(), size: 100 }];
    app.selected_files = vec![false];
    app.rebuild_entries();

    twatch::app::handlers::handle_key(&mut app, KeyCode::Char(' ').into());
    assert!(app.selected_files[0]);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char(' ').into());
    assert!(!app.selected_files[0]);
}

#[test]
fn browser_esc_returns_to_welcome_and_clears_files() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a".into(), size: 1 }];
    app.selected_files = vec![false];
    app.rebuild_entries();

    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc.into());
    assert_eq!(app.screen, Screen::Welcome);
    assert!(app.files.is_empty());
    assert!(app.selected_files.is_empty());
    assert!(app.display_entries.is_empty());
}

#[test]
fn browser_folder_expand_collapse() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![
        TorrentFile { index: 0, name: "dir/file1.mkv".into(), size: 100 },
        TorrentFile { index: 1, name: "dir/file2.mkv".into(), size: 200 },
        TorrentFile { index: 2, name: "README.txt".into(), size: 50 },
    ];
    app.selected_files = vec![false; 3];
    app.rebuild_entries();

    // Tree should have: folder "dir" (depth 0), file "README.txt" (depth 0)
    assert_eq!(app.display_entries.len(), 2);
    assert!(matches!(app.display_entries[0], DisplayEntry::Folder { .. }));
    assert!(matches!(app.display_entries[1], DisplayEntry::File { .. }));

    // Enter on folder expands it.
    app.selected_file = 0;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter.into());
    assert_eq!(app.display_entries.len(), 4, "should show 2 files inside dir");
    assert!(!app.expanded_paths.is_empty(), "dir/ should be in expanded_paths");

    // Enter again collapses.
    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter.into());
    assert_eq!(app.display_entries.len(), 2, "should be back to 2 entries");
    assert!(app.expanded_paths.is_empty());
}

#[test]
fn browser_q_in_search_exits_to_welcome() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a.mkv".into(), size: 100 }];
    app.selected_files = vec![false];
    app.rebuild_entries();

    // Enter search mode
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('/').into());
    assert!(app.is_searching);

    // q exits search + goes to welcome
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('q').into());
    assert_eq!(app.screen, Screen::Welcome);
    assert!(!app.is_searching);
    assert!(app.display_entries.is_empty());
}

#[test]
fn browser_search_esc_cancels_search_only() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a.mkv".into(), size: 100 }];
    app.selected_files = vec![false];
    app.rebuild_entries();

    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('/').into());
    assert!(app.is_searching);

    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc.into());
    assert!(!app.is_searching);
    assert_eq!(app.screen, Screen::Browser, "should stay on Browser");
    assert!(!app.display_entries.is_empty());
}

#[test]
fn player_esc_kills_and_returns_to_browser() {
    let mut app = mock_app();
    app.screen = Screen::Player;

    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc.into());
    assert_eq!(app.screen, Screen::Browser);
    assert!(app.status_message.contains("stopped"));
}

#[test]
fn history_esc_returns_to_welcome() {
    let mut app = mock_app();
    app.screen = Screen::History;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc.into());
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

    twatch::app::handlers::handle_key(&mut app, KeyCode::Down.into());
    assert_eq!(app.history_selected, 1);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Up.into());
    assert_eq!(app.history_selected, 0);
}

#[test]
fn history_entries_show_watched_tag() {
    use shaku::HasComponent;
    use std::sync::Arc;

    let mut app = mock_app();
    app.screen = Screen::History;

    let storage: Arc<dyn StorageService> = app.module.resolve();
    storage.add_entry("magnet:a", "Torrent A");
    storage.mark_watched("magnet:a", "file1.mkv");
    storage.add_entry("magnet:b", "Torrent B");

    let entries = storage.history();
    assert!(entries.iter().any(|e| e.url == "magnet:a" && e.watched));
    assert!(entries.iter().any(|e| e.url == "magnet:b" && !e.watched));
}

#[test]
fn browser_shows_watched_and_downloaded_tags() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.set_pending_url("magnet:x".into());
    app.files = vec![
        TorrentFile { index: 0, name: "watched.mkv".into(), size: 100 },
        TorrentFile { index: 1, name: "downloaded.mkv".into(), size: 200 },
    ];
    app.selected_files = vec![false, false];
    app.watched_files = vec!["watched.mkv".into()];
    app.downloaded_files = vec!["downloaded.mkv".into()];
    app.rebuild_entries();

    use shaku::HasComponent;
    use std::sync::Arc;
    let storage: Arc<dyn StorageService> = app.module.resolve();
    storage.add_entry("magnet:x", "Torrent X");
    storage.mark_watched("magnet:x", "watched.mkv");
    storage.mark_downloaded("magnet:x", "downloaded.mkv");

    let entries = storage.history();
    let entry = entries.iter().find(|e| e.url == "magnet:x").unwrap();
    assert!(entry.watched);
    assert!(entry.watched_files.contains(&"watched.mkv".to_string()));
    assert!(entry.downloaded);
    assert!(entry.downloaded_files.contains(&"downloaded.mkv".to_string()));
}

#[test]
fn t_key_opens_theme_picker() {
    let mut app = mock_app();
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('t').into());
    assert!(app.theme_picker);
}

#[test]
fn theme_picker_enter_selects_theme() {
    let mut app = mock_app();
    let original = app.theme;
    // Open picker, move to next theme, select it
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('t').into());
    assert!(app.theme_picker);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Down.into());
    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter.into());
    assert!(!app.theme_picker);
    assert_ne!(app.theme, original);
}

#[test]
fn theme_picker_esc_closes_without_change() {
    let mut app = mock_app();
    let original = app.theme;
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('t').into());
    assert!(app.theme_picker);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc.into());
    assert!(!app.theme_picker);
    assert_eq!(app.theme, original);
}

#[test]
fn theme_picker_filter_narrows_list() {
    let mut app = mock_app();
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('t').into());
    // Type 'nord'
    for c in "nord".chars() {
        twatch::app::handlers::handle_key(&mut app, KeyCode::Char(c).into());
    }
    assert_eq!(app.theme_picker_filter, "nord");
    assert!(app.theme_picker); // still open
    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc.into());
}

#[test]
fn theme_picker_backspace_clears_filter() {
    let mut app = mock_app();
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('t').into());
    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('x').into());
    twatch::app::handlers::handle_key(&mut app, KeyCode::Backspace.into());
    assert_eq!(app.theme_picker_filter, "");
    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc.into());
}

#[test]
fn delete_x_with_no_selection_shows_error() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a.mkv".into(), size: 100 }];
    app.selected_files = vec![false];
    app.rebuild_entries();

    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('x').into());
    assert!(!app.confirm_delete);
}

#[test]
fn delete_x_with_selection_shows_confirm() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a.mkv".into(), size: 100 }];
    app.selected_files = vec![true];
    app.rebuild_entries();

    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('x').into());
    assert!(app.confirm_delete);
    assert!(!app.confirm_delete_yes);
}

#[test]
fn delete_confirm_esc_cancels() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a.mkv".into(), size: 100 }];
    app.selected_files = vec![true];
    app.rebuild_entries();

    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('x').into());
    assert!(app.confirm_delete);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Esc.into());
    assert!(!app.confirm_delete);
}

#[test]
fn delete_confirm_toggle_yes_no() {
    let mut app = mock_app();
    app.screen = Screen::Browser;
    app.files = vec![TorrentFile { index: 0, name: "a.mkv".into(), size: 100 }];
    app.selected_files = vec![true];
    app.rebuild_entries();

    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('x').into());
    assert!(!app.confirm_delete_yes);

    twatch::app::handlers::handle_key(&mut app, KeyCode::Right.into());
    assert!(app.confirm_delete_yes);

    twatch::app::handlers::handle_key(&mut app, KeyCode::Left.into());
    assert!(!app.confirm_delete_yes);
}

#[test]
fn delete_confirm_enter_on_yes_removes_files() {
    let tmp = temp_dir();
    let f1 = tmp.join("a.mkv");
    let f2 = tmp.join("b.mkv");
    fs::write(&f1, b"a").unwrap();
    fs::write(&f2, b"b").unwrap();

    let mut app = mock_app_with_download_dir(tmp.clone());
    app.screen = Screen::Browser;
    app.set_pending_url("magnet:d".into());
    app.files = vec![
        TorrentFile { index: 0, name: "a.mkv".into(), size: 100 },
        TorrentFile { index: 1, name: "b.mkv".into(), size: 200 },
    ];
    app.selected_files = vec![true, true];
    app.watched_files = vec!["a.mkv".into()];
    app.downloaded_files = vec!["a.mkv".into(), "b.mkv".into()];
    app.rebuild_entries();

    use shaku::HasComponent;
    use std::sync::Arc;
    let storage: Arc<dyn StorageService> = app.module.resolve();
    storage.add_entry("magnet:d", "Torrent D");
    storage.mark_watched("magnet:d", "a.mkv");
    storage.mark_downloaded("magnet:d", "a.mkv");
    storage.mark_downloaded("magnet:d", "b.mkv");

    twatch::app::handlers::handle_key(&mut app, KeyCode::Char('x').into());
    assert!(app.confirm_delete);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Right.into());
    assert!(app.confirm_delete_yes);
    twatch::app::handlers::handle_key(&mut app, KeyCode::Enter.into());

    assert!(!app.confirm_delete);
    assert!(!f1.exists());
    assert!(!f2.exists());
    assert!(app.status_message.contains("Deleted"));
    assert!(app.watched_files.is_empty());
    assert!(app.downloaded_files.is_empty());

    let entry = storage.history().into_iter().find(|e| e.url == "magnet:d").unwrap();
    assert!(!entry.watched);
    assert!(entry.watched_files.is_empty());
    assert!(!entry.downloaded);
    assert!(entry.downloaded_files.is_empty());
}

fn temp_dir() -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("twatch_test_{ts}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn mock_app_with_download_dir(dir: PathBuf) -> App {
    let config = Config { download_dir: dir, ..Config::default() };
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
    App::new(module, config).unwrap()
}
