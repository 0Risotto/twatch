//! twatch daemon: the Rust gateway behind the web UI.
//!
//! Owns the librqbit session, the JSON history store, the search adapter, the
//! external player, and the axum server.  The browser is a thin client.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Context, Result};
use shaku::HasComponent;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

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

const DEFAULT_PORT: u16 = 7878;

struct Args {
    bind: IpAddr,
    port: u16,
    open: bool,
    allow_remote: bool,
    token: Option<String>,
    web_dir: Option<PathBuf>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: DEFAULT_PORT,
            open: false,
            allow_remote: false,
            token: None,
            web_dir: None,
        }
    }
}

fn parse_args() -> Result<Args> {
    let mut args = Args::default();
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--bind" => {
                let value = iter.next().context("--bind needs an IP address")?;
                args.bind =
                    value.parse().with_context(|| format!("Invalid --bind value: {value}"))?;
            }
            "--port" => {
                let value = iter.next().context("--port needs a number")?;
                args.port =
                    value.parse().with_context(|| format!("Invalid --port value: {value}"))?;
            }
            "--open" => args.open = true,
            "--allow-remote" => args.allow_remote = true,
            "--token" => args.token = Some(iter.next().context("--token needs a value")?),
            "--web-dir" => {
                args.web_dir = Some(PathBuf::from(iter.next().context("--web-dir needs a path")?));
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => anyhow::bail!("Unknown argument: {other}. Try --help."),
        }
    }
    Ok(args)
}

fn print_help() {
    println!(
        "twatchd — Rust gateway for the twatch web UI\n\n\
         Usage: twatchd [options]\n\n\
         Options:\n\
           --bind <ip>        Bind address (default: 127.0.0.1)\n\
           --port <port>      HTTP port (default: {DEFAULT_PORT})\n\
           --open             Open the browser after starting\n\
           --allow-remote     Allow non-loopback Host headers (requires the token)\n\
           --token <token>    Override the persisted auth token\n\
           --web-dir <path>   Serve the SPA from this directory\n\
           -h, --help         Show this help\n\n\
         The daemon prints the URL to open.  The web UI is served by Rust; no\n\
         Node process is needed at runtime."
    );
}

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

fn web_dir(arg: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(dir) = arg {
        return Some(dir);
    }
    let fallback = PathBuf::from("web/dist");
    fallback.join("index.html").exists().then_some(fallback)
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("twatchd: {error:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = parse_args()?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new("twatchd=info,twatch_server=info,twatch_core=info")
            }),
        )
        .with_target(false)
        .init();

    let config = Config::load();
    for dir in [&config.config_dir, &config.session_path, &config.download_dir] {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory {}", dir.display()))?;
    }

    let session = librqbit::Session::new(config.session_path.clone())
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
            http_port: Some(args.port),
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

    {
        let gateway = gateway.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(250));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                gateway.tick();
            }
        });
    }

    let token = match args.token {
        Some(token) => token,
        None => load_or_create_token(&config.config_dir)?,
    };

    let bind = SocketAddr::new(args.bind, args.port);
    let allow_remote = args.allow_remote || !args.bind.is_loopback();
    let listener =
        TcpListener::bind(bind).await.with_context(|| format!("Failed to bind {bind}"))?;
    let local_addr = listener.local_addr()?;
    let url = format!("http://{local_addr}");

    println!("twatchd listening on {url}");
    if allow_remote {
        println!("Remote access is enabled. Token: {token}");
    }
    println!("Open the URL above. Press Ctrl+C to stop.");

    if args.open {
        if let Err(error) = open::that(&url) {
            tracing::warn!("Could not open the browser: {error}");
        }
    }

    let server_config =
        ServerConfig { bind: local_addr, web_dir: web_dir(args.web_dir), token, allow_remote };

    twatch_server::serve(listener, gateway, stream, covers, server_config).await
}
