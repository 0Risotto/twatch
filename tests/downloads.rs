#![allow(clippy::unwrap_used)]

//! Download silo tests: per-download channels, task metadata, and
//! concurrent downloads of multiple files from the same torrent.

use shaku::HasComponent;
use std::path::PathBuf;
use std::sync::Arc;
use twatch::app::App;
use twatch::app::screens::browser::BrowserScreen;
use twatch::app::screens::loading::LoadingScreen;
use twatch::app::screens::registry;
use twatch::application::background::download::{DownloadEvent, DownloadTask, apply};
use twatch::application::downloads::DownloadTracker;
use twatch::config::Config;
use twatch::domain::common::spawner::TaskSpawner;
use twatch::domain::history::ports::HistoryPort;
use twatch::domain::player::ports::PlayerPort;
use twatch::domain::search::SearchConfig;
use twatch::domain::torrent::TorrentFile;
use twatch::domain::torrent::TorrentId;
use twatch::domain::torrent::TorrentStats;
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
        Config { download_dir: PathBuf::from("."), ..Config::default() },
        SearchConfig::default(),
    )
}

fn file(index: usize, name: &str) -> TorrentFile {
    TorrentFile { index, name: name.to_string(), size: 10 }
}

fn open_torrent(app: &mut App) {
    app.torrent.set_torrent(
        "Torrent T".into(),
        "magnet:x".into(),
        vec![file(0, "a.mkv"), file(1, "b.mkv"), file(2, "c.mkv")],
    );
    app.push_screen(registry::browser());
}

fn task(id: u64, is_watch: bool, file_names: &[&str]) -> DownloadTask {
    DownloadTask {
        id,
        url: "magnet:x".into(),
        torrent_name: "Torrent T".into(),
        label: file_names.join(", "),
        file_names: file_names.iter().map(|s| (*s).to_string()).collect(),
        is_watch,
    }
}

#[test]
fn batch_download_marks_each_file_downloading() {
    let mut app = mock_app();
    open_torrent(&mut app);
    app.torrent.selected_files = vec![true, true, false];

    let id = app.enqueue_download_batch(
        "magnet:x".into(),
        vec![file(0, "a.mkv"), file(1, "b.mkv")],
        PathBuf::from("."),
    );
    assert_eq!(id, 0);

    let events = app.download_flow.drain();
    assert_eq!(events.len(), 1, "mock spawner completes the task inline");
    apply(&mut app, events.into_iter().next().unwrap());

    assert!(app.torrent.downloading_files.contains(&"a.mkv".to_string()));
    assert!(app.torrent.downloading_files.contains(&"b.mkv".to_string()));
    assert!(!app.torrent.downloading_files.contains(&"c.mkv".to_string()));

    let dl = &app.active_downloads[0];
    assert_eq!(dl.file_names, vec!["a.mkv", "b.mkv"]);
    assert_eq!(dl.file_name, "2 files");
    assert_eq!(dl.url, "magnet:x", "task carries its own url, not session state");
}

#[test]
fn concurrent_batches_have_isolated_channels() {
    let mut app = mock_app();
    open_torrent(&mut app);

    // Two batches from the same torrent, each with its own channel.
    let id1 =
        app.enqueue_download_batch("magnet:x".into(), vec![file(0, "a.mkv")], PathBuf::from("."));
    let id2 =
        app.enqueue_download_batch("magnet:x".into(), vec![file(1, "b.mkv")], PathBuf::from("."));
    assert_ne!(id1, id2);

    let events = app.download_flow.drain();
    assert_eq!(events.len(), 2);

    // Both downloads land in the sidebar.
    for event in events {
        apply(&mut app, event);
    }
    assert_eq!(app.active_downloads.len(), 2);
}

#[test]
fn finished_download_only_pops_its_own_loading_screen() {
    let mut app = mock_app();
    open_torrent(&mut app);

    // User started two batches; the second loading screen is on top.
    let id1 = 7;
    let id2 = 9;
    app.push_screen(registry::loading_task(id1));
    app.push_screen(registry::loading_task(id2));
    assert_eq!(app.top_as::<LoadingScreen>().map(|l| l.task()), Some(Some(id2)));

    // The first batch finishes: its loading screen is extracted from
    // below the top without touching batch 2's screen.
    apply(
        &mut app,
        DownloadEvent::Ready {
            task: task(id1, false, &["a.mkv"]),
            torrent: TorrentId::new(0),
            stream_url: String::new(),
        },
    );
    assert_eq!(
        app.top_as::<LoadingScreen>().map(|l| l.task()),
        Some(Some(id2)),
        "batch 1 finishing must not dismiss batch 2's loading screen"
    );

    // The second batch finishes: its screen pops, and no stale
    // loading screens remain anywhere in the stack.
    apply(
        &mut app,
        DownloadEvent::Ready {
            task: task(id2, false, &["b.mkv"]),
            torrent: TorrentId::new(0),
            stream_url: String::new(),
        },
    );
    assert_eq!(
        app.top_as::<LoadingScreen>().map(|l| l.task()),
        None,
        "both loading screens must be gone; browser is revealed"
    );
    assert!(app.screens().all(|s| s.as_any().downcast_ref::<LoadingScreen>().is_none()));
}

#[test]
fn dismissed_loading_screen_is_still_cleaned_up_on_watch_ready() {
    let mut app = mock_app();
    open_torrent(&mut app);

    // User starts a watch, then Esc's past the loading screen back to
    // the browser.
    let id = 3;
    app.push_screen(registry::loading_task(id));
    app.handle_key(crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Esc));
    assert!(app.top_as::<LoadingScreen>().is_none());

    apply(
        &mut app,
        DownloadEvent::Ready {
            task: task(id, true, &["a.mkv"]),
            torrent: TorrentId::new(0),
            stream_url: "http://127.0.0.1:0/stream/0/0".into(),
        },
    );

    // The stale loading screen must be gone; playback starts, but the
    // user's current screen is not hijacked (they already moved on).
    assert!(app.screens().all(|s| s.as_any().downcast_ref::<LoadingScreen>().is_none()));
    assert!(app.top_as::<BrowserScreen>().is_some(), "browser must stay on top");
    assert!(app.status_message.contains("Watching"));
    let player: Arc<dyn PlayerPort> = app.module.resolve();
    assert!(player.is_running(), "stream must have been handed to the player");
}

#[test]
fn watch_ready_with_loading_on_top_replaces_it_with_player() {
    let mut app = mock_app();
    open_torrent(&mut app);

    let id = 4;
    app.push_screen(registry::loading_task(id));
    apply(
        &mut app,
        DownloadEvent::Ready {
            task: task(id, true, &["a.mkv"]),
            torrent: TorrentId::new(0),
            stream_url: "http://127.0.0.1:0/stream/0/0".into(),
        },
    );

    assert!(top_is_player(&app), "loading screen must be replaced by the player");
    assert_eq!(app.screens().count(), 3, "welcome + browser + player");
}

fn top_is_player(app: &App) -> bool {
    app.top().is_some_and(|s| s.as_any().is::<twatch::app::screens::player::PlayerScreen>())
}

#[test]
fn crashed_task_surfaces_as_failed_event() {
    let mut app = mock_app();
    open_torrent(&mut app);

    // Simulate a task that died without sending anything: the receiver
    // is dropped without a value.
    let (tx, rx) = tokio::sync::oneshot::channel::<DownloadEvent>();
    drop(tx);
    app.download_flow.pending.push(twatch::application::background::download::PendingDownload {
        task: task(1, false, &["a.mkv"]),
        rx,
    });

    let events = app.download_flow.drain();
    assert_eq!(events.len(), 1, "closed channel must yield a Failed event");
    match &events[0] {
        DownloadEvent::Failed { task, message } => {
            assert_eq!(task.id, 1);
            assert!(message.contains("unexpectedly"));
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[test]
fn failed_download_only_pops_its_own_loading_screen() {
    let mut app = mock_app();
    open_torrent(&mut app);
    app.push_screen(registry::loading_task(1));
    app.push_screen(registry::loading_task(2));

    apply(
        &mut app,
        DownloadEvent::Failed { task: task(1, false, &["a.mkv"]), message: "boom".into() },
    );
    assert_eq!(app.top_as::<LoadingScreen>().map(|l| l.task()), Some(Some(2)));
}

#[test]
fn batch_completion_marks_every_file() {
    let torrent = common::torrent::MockTorrentService::new();
    torrent.set_stats(TorrentStats {
        progress: 1.0,
        download_speed: 0,
        upload_speed: 0,
        peers: 0,
        total_size: 10,
        downloaded: 10,
    });

    let mut app = App::new(
        AppModule::builder()
            .with_component_override::<dyn TorrentPort>(Box::new(torrent))
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
        Config { download_dir: PathBuf::from("."), ..Config::default() },
        SearchConfig::default(),
    );
    open_torrent(&mut app);
    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    storage.add_entry("magnet:x", "Torrent T");
    app.torrent.downloading_files = vec!["a.mkv".into(), "b.mkv".into()];

    app.active_downloads.push(twatch::model::ActiveDownload {
        torrent_id: TorrentId::new(0),
        torrent_name: "Torrent T".into(),
        file_name: "2 files".into(),
        file_names: vec!["a.mkv".into(), "b.mkv".into()],
        url: "magnet:x".into(),
        progress: 0.0,
        download_speed: 0,
        total_size: 10,
        downloaded: 0,
        is_streaming: false,
    });

    DownloadTracker::new().tick(&mut app);

    assert!(app.torrent.downloading_files.is_empty());
    assert!(app.torrent.downloaded_files.contains(&"a.mkv".to_string()));
    assert!(app.torrent.downloaded_files.contains(&"b.mkv".to_string()));

    let entry = storage.history().into_iter().find(|e| e.url == "magnet:x").unwrap();
    assert!(entry.downloaded_files.contains(&"a.mkv".to_string()));
    assert!(entry.downloaded_files.contains(&"b.mkv".to_string()));
}
