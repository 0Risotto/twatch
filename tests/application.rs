#![allow(clippy::unwrap_used)]

//! Unit tests for the application layer use cases.

use shaku::HasComponent;
use std::path::PathBuf;
use std::sync::Arc;
use twatch::app::App;
use twatch::application::downloads::{
    BrowserStatusObserver, CompletionObserver, DownloadTracker, HistorySyncObserver,
    StatusMessageObserver,
};
use twatch::config::Config;
use twatch::domain::common::spawner::TaskSpawner;
use twatch::domain::history::ports::HistoryPort;
use twatch::domain::player::ports::PlayerPort;
use twatch::domain::search::SearchConfig;
use twatch::domain::torrent::TorrentId;
use twatch::domain::torrent::TorrentStats;
use twatch::domain::torrent::ports::{StreamPort, TorrentPort};
use twatch::model::ActiveDownload;
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

fn completed_download() -> ActiveDownload {
    ActiveDownload {
        torrent_id: TorrentId::new(0),
        torrent_name: "Torrent T".into(),
        file_name: "a.mkv".into(),
        file_names: vec!["a.mkv".into()],
        url: "magnet:x".into(),
        progress: 1.0,
        download_speed: 0,
        total_size: 10,
        downloaded: 10,
        is_streaming: false,
    }
}

#[test]
fn history_sync_observer_marks_downloaded() {
    let mut app = mock_app();
    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    storage.add_entry("magnet:x", "Torrent T");

    CompletionObserver::on_completed(&HistorySyncObserver, &mut app, &completed_download());

    let entry = storage.history().into_iter().find(|e| e.url == "magnet:x").unwrap();
    assert!(entry.downloaded);
    assert!(entry.downloaded_files.contains(&"a.mkv".to_string()));
}

#[test]
fn browser_status_observer_updates_badges() {
    let mut app = mock_app();
    app.torrent.downloading_files.push("a.mkv".into());

    CompletionObserver::on_completed(&BrowserStatusObserver, &mut app, &completed_download());

    assert!(app.torrent.downloading_files.is_empty());
    assert!(app.torrent.downloaded_files.contains(&"a.mkv".to_string()));
}

#[test]
fn status_message_observer_sets_message() {
    let mut app = mock_app();
    CompletionObserver::on_completed(&StatusMessageObserver, &mut app, &completed_download());
    assert_eq!(app.status_message, "Download complete: a.mkv");
}

#[test]
fn tracker_tick_without_completion_keeps_download() {
    let mut app = mock_app();
    let mut dl = completed_download();
    dl.progress = 0.0;
    app.active_downloads.push(dl);

    DownloadTracker::new().tick(&mut app);

    // Mock engine reports 0% progress, so no completion should fire.
    assert!(app.active_downloads.iter().any(|d| d.file_name == "a.mkv"));
    assert!(app.status_message.is_empty());
}

#[test]
fn tracker_tick_detects_completion_and_prunes() {
    let torrent = common::torrent::MockTorrentService::new();
    torrent.set_stats(TorrentStats {
        progress: 1.0,
        download_speed: 0,
        upload_speed: 0,
        peers: 0,
        total_size: 10,
        downloaded: 10,
    });

    let config = Config { download_dir: PathBuf::from("."), ..Config::default() };
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
        config,
        SearchConfig::default(),
    );
    let storage: Arc<dyn HistoryPort> = app.module.resolve();
    storage.add_entry("magnet:x", "Torrent T");

    let mut dl = completed_download();
    dl.progress = 0.0;
    app.active_downloads.push(dl);
    app.torrent.downloading_files.push("a.mkv".into());
    app.torrent_id = Some(TorrentId::new(0));

    DownloadTracker::new().tick(&mut app);

    assert!(app.active_downloads.is_empty());
    assert_eq!(app.status_message, "Download complete: a.mkv");
    assert!(app.torrent.downloaded_files.contains(&"a.mkv".to_string()));
    assert!(app.torrent.downloading_files.is_empty());
    let entry = storage.history().into_iter().find(|e| e.url == "magnet:x").unwrap();
    assert!(entry.downloaded_files.contains(&"a.mkv".to_string()));
}
