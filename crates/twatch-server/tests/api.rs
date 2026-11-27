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
async fn csp_allows_wasm_subtitle_workers() {
    let response = app().oneshot(request("/api/bootstrap", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let csp = response.headers()["content-security-policy"].to_str().unwrap();
    assert!(csp.contains("'wasm-unsafe-eval'"));
    assert!(csp.contains("'unsafe-eval'"));
    assert!(csp.contains("worker-src 'self'"));
    assert!(csp.contains("media-src 'self' blob: data:"));
}

#[tokio::test]
async fn api_accepts_the_bearer_token() {
    let response = app().oneshot(request("/api/bootstrap", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["config"]["theme"], "mono-dark");
    assert_eq!(json["themes"][0]["name"], "tokyonight-dark");
}

#[tokio::test]
async fn themes_expose_the_full_catalogue() {
    let response = app().oneshot(request("/api/themes", Some("test-token"))).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let themes: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(themes.len(), 56);
    assert!(themes.iter().any(|theme| theme["name"] == "orchid-light"));
    assert!(themes.iter().any(|theme| theme["name"] == "mono-dark"));
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
async fn playback_progress_accepts_a_put() {
    let request = Request::builder()
        .uri("/api/history/progress")
        .method("PUT")
        .header("authorization", "Bearer test-token")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"url":"magnet:test","file":"episode.mkv","position":42.5,"duration":100}"#,
        ))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn stream_without_engine_is_a_clean_503() {
    let response = app().oneshot(request("/api/stream/1/0", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

fn json_request(method: &str, path: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .uri(path)
        .method(method)
        .header("authorization", "Bearer test-token")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn session_preview_select_and_close() {
    let app = app();
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/session/preview",
            serde_json::json!({ "url": "magnet:test" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "Test");
    assert_eq!(json["files"].as_array().unwrap().len(), 1);

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/session/files/select",
            serde_json::json!({ "indices": [0], "selected": true }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["selectedCount"], 1);

    let response = app.clone().oneshot(request("/api/session", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(json_request("POST", "/api/session/close", serde_json::json!({})))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = app.oneshot(request("/api/session", Some("test-token"))).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_null());
}

#[tokio::test]
async fn search_config_round_trips() {
    let app = app();
    let response =
        app.clone().oneshot(request("/api/search/config", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let config = serde_json::json!({
        "url": "https://example.test/search?q={query}",
        "headers": {},
        "fieldMap": {
            "name": "name",
            "infoHash": "infohash",
            "seeders": "seeders",
            "leechers": "leechers",
            "size": "size",
            "username": "username",
            "category": "category"
        },
        "blockedCategories": ["700-799", "blocked-tag"]
    });
    let response = app
        .clone()
        .oneshot(json_request("PUT", "/api/search/config", config.clone()))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["url"], config["url"]);
    assert_eq!(json["blockedCategories"], config["blockedCategories"]);

    let response = app
        .clone()
        .oneshot(request("/api/search?q=test&page=0", Some("test-token")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn player_and_config_endpoints_validate_input() {
    let app = app();
    let response = app.clone().oneshot(request("/api/player", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["running"], false);

    let response = app
        .clone()
        .oneshot(json_request("POST", "/api/player/open", serde_json::json!({ "downloadId": 99 })))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = app
        .clone()
        .oneshot(json_request("POST", "/api/player/stop", serde_json::json!({})))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            "/api/config",
            serde_json::json!({ "theme": "emerald-dark", "player": "mpv" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["theme"], "emerald-dark");
    assert_eq!(json["player"], "mpv");

    let response = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            "/api/config",
            serde_json::json!({ "theme": "missing-theme" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = app
        .oneshot(json_request(
            "PATCH",
            "/api/config",
            serde_json::json!({ "player": "missing-player" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn empty_download_and_delete_lists_are_well_formed() {
    let app = app();
    let response =
        app.clone().oneshot(request("/api/downloads", Some("test-token"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, serde_json::json!([]));

    let response = app
        .oneshot(json_request(
            "DELETE",
            "/api/session/files",
            serde_json::json!({ "indices": [0] }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["deleted"], 0);
    assert_eq!(json["failed"], serde_json::json!([]));
}
