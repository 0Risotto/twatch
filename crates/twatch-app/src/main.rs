//! Tauri desktop shell for twatch.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend;
mod linux_frame;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn focus_existing(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("twatch=info")),
        )
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_existing(app);
        }))
        .setup(|app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        eprintln!("twatch-app: failed to create runtime: {error:#}");
                        handle.exit(1);
                        return;
                    }
                };

                runtime.block_on(async move {
                    match backend::start().await {
                        Ok(url) => {
                            let window_handle = handle.clone();
                            if let Err(error) = handle.run_on_main_thread(move || {
                                let Ok(parsed) = url.parse::<url::Url>() else {
                                    eprintln!("twatch-app: invalid gateway URL: {url}");
                                    window_handle.exit(1);
                                    return;
                                };
                                let frameless = linux_frame::prefers_frameless();
                                let window = WebviewWindowBuilder::new(
                                    &window_handle,
                                    "main",
                                    WebviewUrl::External(parsed),
                                )
                                .title("twatch")
                                .inner_size(1280.0, 820.0)
                                .min_inner_size(900.0, 600.0)
                                .resizable(true)
                                .decorations(!frameless)
                                .initialization_script(format!(
                                    "window.__TWATCH_FRAMELESS__ = {frameless};"
                                ))
                                .build();
                                if let Err(error) = window {
                                    eprintln!("twatch-app: could not open window: {error:#}");
                                    window_handle.exit(1);
                                }
                            }) {
                                eprintln!("twatch-app: could not schedule window: {error:#}");
                                handle.exit(1);
                                return;
                            }
                            // Keep the Tokio runtime alive for the embedded
                            // HTTP server until the Tauri process exits.
                            std::future::pending::<()>().await;
                        }
                        Err(error) => {
                            eprintln!("twatch-app: {error:#}");
                            handle.exit(1);
                        }
                    }
                });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running twatch");
}
