//! Embedded twatch gateway for the desktop application.
//!
//! The Tauri window talks to this loopback server, so the desktop app and the
//! browser console share the same React UI and API without spawning a second
//! `twatchd` process.

use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Context, Result};
use shaku::HasComponent;
use tokio::net::TcpListener;

use twatch_core::config::Config;
use twatch_core::domain::history::ports::HistoryPort;
use twatch_core::domain::player::ports::PlayerPort;
use twatch_core::domain::search::SearchConfig;
use twatch_core::domain::search::ports::SearchPort;
use twatch_core::domain::torrent::ports::TorrentPort;
use twatch_core::gateway::Gateway;
use twatch_engine::covers::CoverService;
use twatch_engine::module::AppModule;
use twatch_engine::service::storage::{RealStorage, RealStorageParameters, StorageState};
use twatch_engine::service::task::{TokioTaskSpawner, TokioTaskSpawnerParameters};
use twatch_engine::service::torrent::{
    StreamUrlProvider, StreamUrlProviderParameters, TorrentEngineImpl, TorrentEngineImplParameters,
};
use twatch_engine::stream::StreamService;
use twatch_server::state::ServerConfig;

fn load_or_create_token(config_dir: &Path) -> Result<String> {
    std::fs::create_dir_all(config_dir).context("Failed to create config directory")?;
    let path = config_dir.join("server-token");
    if let Ok(token) = std::fs::read_to_string(&path) {
        let token = token.trim().to_owned();
        if !token.is_empty() {
            return Ok(token);
        }
    }
    let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(&path).context("Failed to create token file")?;
    file.write_all(token.as_bytes()).context("Failed to write token file")?;
    Ok(token)
}

/// Start the loopback gateway and return the URL the Tauri window should load.
pub async fn start() -> Result<String> {
    let config = Config::load();
    for dir in [&config.config_dir, &config.session_path, &config.download_dir] {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory {}", dir.display()))?;
    }
    // Keep the desktop session separate from a headless twatchd session.
    let session_path = config.session_path.join("desktop");
    std::fs::create_dir_all(&session_path)
        .with_context(|| format!("Failed to create directory {}", session_path.display()))?;

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .context("Failed to bind the desktop gateway")?;
    let local_addr = listener.local_addr()?;
    let port = local_addr.port();
    let token = load_or_create_token(&config.config_dir)?;

    let session = librqbit::Session::new_with_opts(
        session_path.clone(),
        librqbit::SessionOptions {
            // Keep the desktop app independent from any headless twatchd that
            // may still be using the normal session DHT file.
            dht_config: Some(librqbit::dht::PersistentDhtConfig {
                config_filename: Some(session_path.join("dht.json")),
                ..Default::default()
            }),
            ..Default::default()
        },
    )
    .await
    .context("Failed to create librqbit session")?;
    let stream = StreamService::new(session.clone());

    let storage = StorageState::new(config.config_dir.clone(), &config.history_filename)?;
    let search_config = SearchConfig::load(&config.config_dir);

    let module = AppModule::builder()
        .with_component_parameters::<TorrentEngineImpl>(TorrentEngineImplParameters {
            session: Some(session),
        })
        .with_component_parameters::<StreamUrlProvider>(StreamUrlProviderParameters {
            http_port: Some(port),
        })
        .with_component_parameters::<TokioTaskSpawner>(TokioTaskSpawnerParameters {
            rt: None,
            handle: Some(tokio::runtime::Handle::current()),
        })
        .with_component_parameters::<RealStorage>(RealStorageParameters {
            state: Some(Mutex::new(storage)),
        })
        .build();

    let torrent: std::sync::Arc<dyn TorrentPort> = module.resolve();
    let history: std::sync::Arc<dyn HistoryPort> = module.resolve();
    let search: std::sync::Arc<dyn SearchPort> = module.resolve();
    let player: std::sync::Arc<dyn PlayerPort> = module.resolve();
    let spawner: std::sync::Arc<dyn twatch_core::domain::common::spawner::TaskSpawner> =
        module.resolve();
    let covers = std::sync::Arc::new(CoverService::new(config.config_dir.join("covers.json")));
    let gateway =
        Gateway::new(config.clone(), search_config, torrent, history, search, player, spawner);

    let ticker_gateway = gateway.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(250));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            ticker_gateway.tick();
        }
    });

    let server_config =
        ServerConfig { bind: local_addr, web_dir: None, token, allow_remote: false };
    tokio::spawn(async move {
        if let Err(error) =
            twatch_server::serve(listener, gateway, stream, covers, server_config).await
        {
            eprintln!("twatch-app gateway stopped: {error:#}");
        }
    });

    Ok(format!("http://{local_addr}"))
}
