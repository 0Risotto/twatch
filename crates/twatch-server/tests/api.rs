//! Server contract tests: auth, static fallback, and DTO casing.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use twatch_core::config::Config;
use twatch_core::domain::common::spawner::TaskSpawner;
use twatch_core::domain::history::HistoryEntry;
use twatch_core::domain::history::ports::HistoryPort;
use twatch_core::domain::player::ports::PlayerPort;
use twatch_core::domain::search::ports::SearchPort;
use twatch_core::domain::search::{SearchConfig, SearchResult};
use twatch_core::domain::torrent::ports::TorrentPort;
use twatch_core::domain::torrent::{TorrentFile, TorrentId, TorrentInfo, TorrentStats};
use twatch_core::gateway::Gateway;
use twatch_engine::covers::CoverService;
use twatch_server::state::{AppState, ServerConfig};

struct MockTorrent;
#[async_trait]
impl TorrentPort for MockTorrent {
    async fn preview(&self, _url: &str) -> Result<TorrentInfo> {
        Ok(TorrentInfo {
            name: "Test".into(),
            files: vec![TorrentFile { index: 0, name: "test.mkv".into(), size: 10 }],
        })
    }
    async fn download(&self, _url: &str, _file_idx: usize) -> Result<TorrentId> {
        Ok(TorrentId::new(1))
    }
    async fn download_to_folder(
        &self,
        _url: &str,
        _indices: &[usize],
        _dir: &Path,
    ) -> Result<TorrentId> {
        Ok(TorrentId::new(1))
    }
    fn get_stats(&self, _id: &TorrentId) -> Result<TorrentStats> {
        Ok(TorrentStats {
            progress: 0.0,
            download_speed: 0,
            upload_speed: 0,
            peers: 0,
            total_size: 10,
            downloaded: 0,
        })
    }
}

struct MockHistory(Mutex<Vec<HistoryEntry>>);
impl Default for MockHistory {
    fn default() -> Self {
        Self(Mutex::new(Vec::new()))
    }
}
impl HistoryPort for MockHistory {
    fn history(&self) -> Vec<HistoryEntry> {
        self.0.lock().unwrap().clone()
    }
    fn add_entry(&self, _url: &str, _torrent_name: &str) {}
    fn rename_entry(&self, _index: usize, _new_name: &str) -> Result<()> {
        Ok(())
    }
    fn mark_watched(&self, _url: &str, _file_name: &str) {}
    fn mark_downloaded(&self, _url: &str, _file_name: &str) {}
    fn mark_deleted(&self, _url: &str, _file_name: &str) {}
    fn rename_file(&self, _url: &str, _old_name: &str, _new_name: &str) {}
    fn remove_entry(&self, _index: usize) -> Result<()> {
        Ok(())
    }
}

struct MockSearch;
#[async_trait]
impl SearchPort for MockSearch {
    async fn search(
        &self,
        _query: &str,
        _config: &SearchConfig,
        _page: u32,
    ) -> Result<Vec<SearchResult>> {
        Ok(Vec::new())
    }
}

struct MockPlayer;
impl PlayerPort for MockPlayer {
    fn play(&self, _url: &str, _title: &str) {}
    fn is_running(&self) -> bool {
        false
    }
    fn kill(&self) {}
}

struct MockSpawner;
impl TaskSpawner for MockSpawner {
    fn spawn(&self, _future: Pin<Box<dyn Future<Output = ()> + Send>>) {}
}

fn temp_dir() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("twatch_server_{nanos}"))
}

fn test_state() -> AppState {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), "<html><body>twatch</body></html>").unwrap();

    let config = Config {
        config_dir: dir.join("config"),
        download_dir: dir.join("downloads"),
        ..Config::default()
    };
    let gateway = Gateway::new(
        config,
        SearchConfig::default(),
        Arc::new(MockTorrent),
        Arc::new(MockHistory::default()),
        Arc::new(MockSearch),
        Arc::new(MockPlayer),
        Arc::new(MockSpawner),
    );
    let server = ServerConfig {
        bind: "127.0.0.1:0".parse().unwrap(),
        web_dir: Some(dir),
        token: "test-token".into(),
        allow_remote: false,
    };
    AppState::new(gateway, None, Arc::new(CoverService::ephemeral()), &server)
}

fn app() -> axum::Router {
    twatch_server::router(test_state())
}

fn request(path: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().uri(path).method("GET");
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    builder.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn health_is_public() {
    let response = app().oneshot(request("/api/health", None)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn api_requires_the_token() {
    let response = app().oneshot(request("/api/bootstrap", None)).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_accepts_the_bearer_token() {
    let response = app().oneshot(request("/api/bootstrap", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["config"]["theme"], "emerald-dark");
    assert_eq!(json["themes"][0]["name"], "tokyonight-dark");
}

#[tokio::test]
async fn themes_expose_the_full_catalogue() {
    let response = app().oneshot(request("/api/themes", Some("test-token"))).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let themes: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(themes.len(), 54);
    assert!(themes.iter().any(|theme| theme["name"] == "orchid-light"));
    assert!(themes.iter().any(|theme| theme["name"] == "amethyst-oled-dark"));
    assert!(themes.iter().any(|theme| theme["name"] == "kanagawa-dark"));
}

#[tokio::test]
async fn history_is_camel_case() {
    let response = app().oneshot(request("/api/history", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn spa_fallback_serves_index() {
    let response = app().oneshot(request("/downloads", None)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/html");
}

#[tokio::test]
async fn covers_batch_is_available() {
    let request = Request::builder()
        .uri("/api/covers")
        .method("POST")
        .header("authorization", "Bearer test-token")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"items":[]}"#))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn stream_without_engine_is_a_clean_503() {
    let response = app().oneshot(request("/api/stream/1/0", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}
