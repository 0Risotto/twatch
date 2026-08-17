#![allow(clippy::unwrap_used)]

//! Screen navigation and stack tests.

use crossterm::event::KeyCode;
use twatch::app::App;
use twatch::app::screens::browser::BrowserScreen;
use twatch::app::screens::history::HistoryScreen;
use twatch::app::screens::input::InputScreen;
use twatch::app::screens::loading::LoadingScreen;
use twatch::app::screens::registry;
use twatch::app::screens::rename::RenameScreen;
use twatch::app::screens::search::SearchScreen;
use twatch::app::screens::theme_picker::ThemePickerScreen;
use twatch::app::screens::welcome::WelcomeScreen;
use twatch::app::screens::{ScreenViewModel, Transition};
use twatch::application::background::preview::{PreviewEvent, apply};
use twatch::config::Config;
use twatch::config::search::SearchConfig;
use twatch::domain::common::spawner::TaskSpawner;
use twatch::domain::history::ports::HistoryPort;
use twatch::domain::player::ports::PlayerPort;
use twatch::domain::torrent::ports::{StreamPort, TorrentPort};
use twatch::domain::torrent::{DisplayEntry, TorrentFile};
use twatch::module::AppModule;

use std::fs;
use std::path::PathBuf;

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

fn mock_app_with_download_dir(dir: PathBuf) -> App {
    let config = Config { download_dir: dir, ..Config::default() };
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
        config,
        SearchConfig::default(),
    )
}

fn key(code: KeyCode) -> crossterm::event::KeyEvent {
    code.into()
}

fn top_is<T: twatch::app::screens::Screen + 'static>(app: &App) -> bool {
    app.top().is_some_and(|s| s.as_any().is::<T>())
}

fn stack_len(app: &App) -> usize {
    app.screens().count()
}

/// Push a browser screen with the given files loaded into the session.
fn open_browser(app: &mut App, files: Vec<TorrentFile>) {
    app.torrent.set_torrent("Torrent T".into(), "magnet:x".into(), files);
    app.push_screen(registry::browser());
}

// ── Welcome ──

#[test]
fn welcome_enter_on_add_navigates_to_input() {
    let mut app = mock_app();
    assert!(top_is::<WelcomeScreen>(&app));

    app.handle_key(key(KeyCode::Enter));
    assert!(top_is::<InputScreen>(&app));
}

#[test]
fn welcome_enter_on_quit_stops_running() {
    let mut app = mock_app();
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert!(!app.running);
}

#[test]
fn welcome_q_stops_running() {
    let mut app = mock_app();
    app.handle_key(key(KeyCode::Char('q')));
    assert!(!app.running);
}

#[test]
fn welcome_h_opens_history() {
    let mut app = mock_app();
    app.handle_key(key(KeyCode::Char('h')));
    assert!(top_is::<HistoryScreen>(&app));
}

// ── Input ──

#[test]
fn input_esc_returns_to_welcome() {
    let mut app = mock_app();
    app.handle_key(key(KeyCode::Enter));
    assert!(top_is::<InputScreen>(&app));

    app.handle_key(key(KeyCode::Esc));
    assert!(top_is::<WelcomeScreen>(&app));
}

#[test]
fn input_typing_builds_value() {
    let mut app = mock_app();
    app.push_screen(registry::input(String::new()));

    app.handle_key(key(KeyCode::Char('h')));
    app.handle_key(key(KeyCode::Char('i')));

    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::Input(vm) => assert_eq!(vm.value, "hi"),
        _ => panic!("expected input screen"),
    }
}

#[test]
fn input_enter_on_non_empty_url_starts_loading() {
    let mut app = mock_app();
    app.push_screen(registry::input("magnet:?xt=urn:btih:abc"));

    app.handle_key(key(KeyCode::Enter));
    assert!(top_is::<LoadingScreen>(&app));
}

#[test]
fn input_enter_on_empty_url_is_ignored() {
    let mut app = mock_app();
    app.push_screen(registry::input(String::new()));

    app.handle_key(key(KeyCode::Enter));
    assert!(top_is::<InputScreen>(&app));
}

// ── Browser ──

fn browser_files() -> Vec<TorrentFile> {
    vec![TorrentFile { index: 0, name: "a.mkv".into(), size: 100 }]
}

#[test]
fn browser_toggle_selection() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());

    app.handle_key(key(KeyCode::Char(' ')));
    assert!(app.torrent.selected_files[0]);
    app.handle_key(key(KeyCode::Char(' ')));
    assert!(!app.torrent.selected_files[0]);
}

#[test]
fn browser_esc_returns_to_welcome_and_clears_files() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());

    app.handle_key(key(KeyCode::Esc));
    assert!(top_is::<WelcomeScreen>(&app));
    assert!(app.torrent.torrent_name.is_none());
    assert!(app.torrent.tree.entries().is_empty());
}

#[test]
fn browser_folder_expand_collapse() {
    let mut app = mock_app();
    open_browser(
        &mut app,
        vec![
            TorrentFile { index: 0, name: "dir/file1.mkv".into(), size: 100 },
            TorrentFile { index: 1, name: "dir/file2.mkv".into(), size: 200 },
            TorrentFile { index: 2, name: "README.txt".into(), size: 50 },
        ],
    );

    // Tree should have: folder "dir" (depth 0), file "README.txt" (depth 0)
    assert_eq!(app.torrent.tree.entries().len(), 2);
    assert!(matches!(app.torrent.tree.entries()[0], DisplayEntry::Folder { .. }));
    assert!(matches!(app.torrent.tree.entries()[1], DisplayEntry::File { .. }));

    // Enter on folder expands it.
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.torrent.tree.entries().len(), 4, "should show 2 files inside dir");
    assert!(!app.torrent.tree.expanded_paths().is_empty(), "dir/ should be in expanded_paths");

    // Enter again collapses.
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.torrent.tree.entries().len(), 2, "should be back to 2 entries");
    assert!(app.torrent.tree.expanded_paths().is_empty());
}

#[test]
fn browser_q_in_search_exits_to_welcome() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());

    // Enter search mode
    app.handle_key(key(KeyCode::Char('/')));
    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::Browser(vm) => assert!(vm.searching.is_some()),
        _ => panic!("expected browser screen"),
    }

    // q exits search + goes to welcome
    app.handle_key(key(KeyCode::Char('q')));
    assert!(top_is::<WelcomeScreen>(&app));
    assert!(app.torrent.tree.entries().is_empty());
}

#[test]
fn browser_search_esc_cancels_search_only() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());

    app.handle_key(key(KeyCode::Char('/')));
    app.handle_key(key(KeyCode::Esc));
    assert!(top_is::<BrowserScreen>(&app), "should stay on Browser");
    assert!(!app.torrent.tree.entries().is_empty());
}

// ── Player ──

#[test]
fn player_esc_kills_and_returns_to_browser() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());
    app.push_screen(registry::player());

    app.handle_key(key(KeyCode::Esc));
    assert!(top_is::<BrowserScreen>(&app));
    assert!(app.status_message.contains("stopped"));
}

// ── History ──

#[test]
fn history_esc_returns_to_welcome() {
    let mut app = mock_app();
    app.push_screen(registry::history());

    app.handle_key(key(KeyCode::Esc));
    assert!(top_is::<WelcomeScreen>(&app));
}

#[test]
fn history_down_moves_selection() {
    use shaku::HasComponent;
    use std::sync::Arc;

    let mut app = mock_app();
    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    storage.add_entry("magnet:a", "A");
    storage.add_entry("magnet:b", "B");
    app.push_screen(registry::history());

    app.handle_key(key(KeyCode::Down));
    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::History(vm) => assert_eq!(vm.selected, 1),
        _ => panic!("expected history screen"),
    }

    app.handle_key(key(KeyCode::Up));
    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::History(vm) => assert_eq!(vm.selected, 0),
        _ => panic!("expected history screen"),
    }
}

#[test]
fn history_entries_show_watched_tag() {
    use shaku::HasComponent;
    use std::sync::Arc;

    let app = mock_app();
    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    storage.add_entry("magnet:a", "Torrent A");
    storage.mark_watched("magnet:a", "file1.mkv");
    storage.add_entry("magnet:b", "Torrent B");

    let entries = storage.history();
    assert!(entries.iter().any(|e| e.url == "magnet:a" && e.watched));
    assert!(entries.iter().any(|e| e.url == "magnet:b" && !e.watched));
}

#[test]
fn browser_shows_watched_and_downloaded_tags() {
    use shaku::HasComponent;
    use std::sync::Arc;

    let mut app = mock_app();
    app.torrent.set_torrent(
        "Torrent X".into(),
        "magnet:x".into(),
        vec![
            TorrentFile { index: 0, name: "watched.mkv".into(), size: 100 },
            TorrentFile { index: 1, name: "downloaded.mkv".into(), size: 200 },
        ],
    );
    app.torrent.watched_files = vec!["watched.mkv".into()];
    app.torrent.downloaded_files = vec!["downloaded.mkv".into()];

    let storage: Arc<dyn HistoryPort> = app.module.resolve();
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

// ── Theme picker ──

#[test]
fn t_key_opens_theme_picker() {
    let mut app = mock_app();
    app.handle_key(key(KeyCode::Char('t')));
    assert!(top_is::<ThemePickerScreen>(&app));
}

#[test]
fn theme_picker_enter_selects_theme() {
    let mut app = mock_app_with_config_dir(temp_dir());
    let original = app.theme_state.theme;
    app.handle_key(key(KeyCode::Char('t')));
    assert!(top_is::<ThemePickerScreen>(&app));

    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert!(top_is::<WelcomeScreen>(&app));
    assert_ne!(app.theme_state.theme, original);
}

#[test]
fn theme_picker_esc_closes_without_change() {
    let mut app = mock_app();
    let original = app.theme_state.theme;
    app.handle_key(key(KeyCode::Char('t')));
    assert!(top_is::<ThemePickerScreen>(&app));

    app.handle_key(key(KeyCode::Esc));
    assert!(top_is::<WelcomeScreen>(&app));
    assert_eq!(app.theme_state.theme, original);
}

#[test]
fn theme_picker_filter_narrows_list() {
    let mut app = mock_app();
    app.handle_key(key(KeyCode::Char('t')));
    for c in "nord".chars() {
        app.handle_key(key(KeyCode::Char(c)));
    }

    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::ThemePicker(vm) => {
            assert_eq!(vm.filter, "nord");
            assert!(!vm.items.is_empty());
        }
        _ => panic!("expected theme picker screen"),
    }
    app.handle_key(key(KeyCode::Esc));
}

#[test]
fn theme_picker_backspace_clears_filter() {
    let mut app = mock_app();
    app.handle_key(key(KeyCode::Char('t')));
    app.handle_key(key(KeyCode::Char('x')));
    app.handle_key(key(KeyCode::Backspace));

    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::ThemePicker(vm) => assert_eq!(vm.filter, ""),
        _ => panic!("expected theme picker screen"),
    }
    app.handle_key(key(KeyCode::Esc));
}

// ── Delete ──

#[test]
fn delete_x_with_no_selection_shows_error() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());

    app.handle_key(key(KeyCode::Char('x')));
    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::Browser(vm) => assert!(vm.confirm.is_none()),
        _ => panic!("expected browser screen"),
    }
}

#[test]
fn delete_x_with_selection_shows_confirm() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());
    app.torrent.selected_files = vec![true];

    app.handle_key(key(KeyCode::Char('x')));
    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::Browser(vm) => assert!(vm.confirm.is_some()),
        _ => panic!("expected browser screen"),
    }
}

#[test]
fn delete_confirm_esc_cancels() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());
    app.torrent.selected_files = vec![true];

    app.handle_key(key(KeyCode::Char('x')));
    app.handle_key(key(KeyCode::Esc));
    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::Browser(vm) => assert!(vm.confirm.is_none()),
        _ => panic!("expected browser screen"),
    }
}

#[test]
fn delete_confirm_toggle_yes_no() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());
    app.torrent.selected_files = vec![true];

    app.handle_key(key(KeyCode::Char('x')));
    app.handle_key(key(KeyCode::Right));

    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::Browser(vm) => assert!(vm.confirm.unwrap().yes),
        _ => panic!("expected browser screen"),
    }

    app.handle_key(key(KeyCode::Left));
    let vm = app.top().unwrap().view(&app);
    match vm {
        ScreenViewModel::Browser(vm) => assert!(!vm.confirm.unwrap().yes),
        _ => panic!("expected browser screen"),
    }
}

#[test]
fn delete_confirm_enter_on_yes_removes_files() {
    use shaku::HasComponent;
    use std::sync::Arc;

    let tmp = temp_dir();
    // Multi-file torrents download into <download_dir>/<torrent name>/.
    let torrent_dir = tmp.join("Torrent T");
    fs::create_dir_all(&torrent_dir).unwrap();
    let f1 = torrent_dir.join("a.mkv");
    let f2 = torrent_dir.join("b.mkv");
    fs::write(&f1, b"a").unwrap();
    fs::write(&f2, b"b").unwrap();

    let mut app = mock_app_with_download_dir(tmp);
    open_browser(
        &mut app,
        vec![
            TorrentFile { index: 0, name: "a.mkv".into(), size: 100 },
            TorrentFile { index: 1, name: "b.mkv".into(), size: 200 },
        ],
    );
    app.torrent.selected_files = vec![true, true];
    app.torrent.watched_files = vec!["a.mkv".into()];
    app.torrent.downloaded_files = vec!["a.mkv".into(), "b.mkv".into()];

    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    storage.add_entry("magnet:x", "Torrent D");
    storage.mark_watched("magnet:x", "a.mkv");
    storage.mark_downloaded("magnet:x", "a.mkv");
    storage.mark_downloaded("magnet:x", "b.mkv");

    app.handle_key(key(KeyCode::Char('x')));
    app.handle_key(key(KeyCode::Right));
    app.handle_key(key(KeyCode::Enter));

    assert!(!f1.exists());
    assert!(!f2.exists());
    assert!(app.status_message.contains("Deleted"));
    assert!(app.torrent.watched_files.is_empty());
    assert!(app.torrent.downloaded_files.is_empty());

    let entry = storage.history().into_iter().find(|e| e.url == "magnet:x").unwrap();
    assert!(!entry.watched);
    assert!(entry.watched_files.is_empty());
    assert!(!entry.downloaded);
    assert!(entry.downloaded_files.is_empty());
}

// ── Popups as stacked screens ──

#[test]
fn s_key_opens_search_popup_over_browser() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());

    app.handle_key(key(KeyCode::Char('s')));
    assert!(top_is::<SearchScreen>(&app));
    assert_eq!(stack_len(&app), 3, "welcome + browser + search popup");

    app.handle_key(key(KeyCode::Esc));
    assert!(top_is::<BrowserScreen>(&app));
    assert_eq!(stack_len(&app), 2);
}

#[test]
fn r_key_opens_rename_popup_over_browser() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());

    app.handle_key(key(KeyCode::Char('r')));
    assert!(top_is::<RenameScreen>(&app));

    app.handle_key(key(KeyCode::Esc));
    assert!(top_is::<BrowserScreen>(&app));
}

#[test]
fn rename_confirms_and_updates_session_file() {
    let mut app = mock_app();
    open_browser(&mut app, browser_files());

    app.handle_key(key(KeyCode::Char('r')));
    app.handle_key(key(KeyCode::Backspace));
    app.handle_key(key(KeyCode::Backspace));
    app.handle_key(key(KeyCode::Backspace));
    app.handle_key(key(KeyCode::Backspace));
    app.handle_key(key(KeyCode::Backspace));
    app.handle_key(key(KeyCode::Char('n')));
    app.handle_key(key(KeyCode::Enter));

    assert!(top_is::<BrowserScreen>(&app));
    assert_eq!(app.torrent.tree.files()[0].name, "n");
    assert!(app.status_message.contains("Renamed"));
}

// ── Navigation stack ──

#[test]
fn preview_ready_resets_stack_and_opens_browser() {
    let mut app = mock_app();
    app.push_screen(registry::history());
    app.push_screen(registry::loading());
    assert_eq!(stack_len(&app), 3);

    apply(
        &mut app,
        PreviewEvent::Ready {
            url: "magnet:x".into(),
            info: twatch::model::TorrentInfo { name: "Torrent T".into(), files: browser_files() },
        },
    );

    assert!(top_is::<BrowserScreen>(&app));
    assert_eq!(stack_len(&app), 2, "history/loading popped; browser sits on welcome");
}

#[test]
fn preview_failed_pops_back_to_origin() {
    let mut app = mock_app();
    app.push_screen(registry::history());
    app.push_screen(registry::loading());

    apply(&mut app, PreviewEvent::Failed("nope".into()));

    assert!(top_is::<HistoryScreen>(&app));
    assert_eq!(stack_len(&app), 2);
}

#[test]
fn transition_push_pop_replaces_top() {
    let mut app = mock_app();
    app.push_screen(registry::history());
    assert!(top_is::<HistoryScreen>(&app));

    app.apply_transition(Transition::Pop);
    assert!(top_is::<WelcomeScreen>(&app));

    app.apply_transition(Transition::Replace(registry::player()));
    assert!(top_is::<twatch::app::screens::player::PlayerScreen>(&app));
}

// ── Helpers ──

fn mock_app_with_config_dir(config_dir: PathBuf) -> App {
    let config = Config { config_dir, ..Config::default() };
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
        config,
        SearchConfig::default(),
    )
}

fn temp_dir() -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("twatch_test_{ts}"));
    fs::create_dir_all(&path).unwrap();
    path
}
