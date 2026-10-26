//! twatch binary entry point.
//!
//! Initialises logging, constructs the DI module, and starts the
//! `TUI` event loop.

use anyhow::Context;
use std::sync::Mutex;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use twatch::config::Config;
use twatch::domain::search::SearchConfig;
use twatch::module::AppModule;
use twatch::service::storage::{RealStorage, RealStorageParameters, StorageState};
use twatch::service::stream_server::start_stream_server;
use twatch::service::task::{TokioTaskSpawner, TokioTaskSpawnerParameters};
use twatch::service::torrent::{
    StreamUrlProvider, StreamUrlProviderParameters, TorrentEngineImpl, TorrentEngineImplParameters,
};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Build the DI container, the terminal, and run the application.
fn try_main() -> anyhow::Result<()> {
    let config = Config::load();

    // Composition root: ensure runtime directories exist on disk.
    if let Some(parent) = config.log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::create_dir_all(&config.download_dir).with_context(|| {
        format!("Failed to create download directory {}", config.download_dir.display())
    })?;

    let log_file = std::fs::OpenOptions::new().create(true).append(true).open(&config.log_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        log_file.set_permissions(std::fs::Permissions::from_mode(0o600)).ok();
    }

    tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(log_file)
                .with_filter(tracing_subscriber::filter::EnvFilter::new(&config.log_level)),
        )
        .init();

    // Create the tokio runtime and boot the torrent engine + stream server.
    let rt = tokio::runtime::Runtime::new()?;
    let (session, http_port) = rt.block_on(async {
        let session = librqbit::Session::new(config.session_path.clone())
            .await
            .context("Failed to create librqbit session")?;
        let port = start_stream_server(session.clone()).await.context("Stream server error")?;
        tracing::info!("Stream HTTP server on port {port}");
        Ok::<(std::sync::Arc<librqbit::Session>, u16), anyhow::Error>((session, port))
    })?;

    // Load history from disk (or seed an empty store).
    let storage_state = StorageState::new(config.config_dir.clone(), &config.history_filename)?;

    // Wire up the shaku DI module.
    let module = AppModule::builder()
        .with_component_parameters::<TorrentEngineImpl>(TorrentEngineImplParameters {
            session: Some(session),
        })
        .with_component_parameters::<StreamUrlProvider>(StreamUrlProviderParameters {
            http_port: Some(http_port),
        })
        .with_component_parameters::<TokioTaskSpawner>(TokioTaskSpawnerParameters {
            rt: Some(rt),
            handle: None,
        })
        .with_component_parameters::<RealStorage>(RealStorageParameters {
            state: Some(Mutex::new(storage_state)),
        })
        .build();

    twatch::ui::install_panic_hook();
    let mut terminal = twatch::ui::init()?;
    let search_config = SearchConfig::load(&config.config_dir);
    let result = twatch::app::run(&mut terminal, module, config, search_config);
    twatch::ui::restore()?;
    result
}
