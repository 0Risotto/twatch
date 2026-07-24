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
use twatch::module::AppModule;
use twatch::service::storage::{RealStorage, RealStorageParameters, StorageState};
use twatch::service::torrent::{
    TorrentEngineImpl, TorrentEngineImplParameters, start_stream_server,
};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Build the DI container, the terminal, and run the application.
fn try_main() -> anyhow::Result<()> {
    let config = Config::default();

    // Ensure the log file's parent directory exists.
    if let Some(parent) = config.log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

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
            rt: Some(rt),
            session: Some(session),
            http_port: Some(http_port),
        })
        .with_component_parameters::<RealStorage>(RealStorageParameters {
            state: Some(Mutex::new(storage_state)),
        })
        .build();

    let mut terminal = twatch::ui::init()?;
    let result = twatch::app::run(&mut terminal, module, config);
    twatch::ui::restore()?;
    result
}
