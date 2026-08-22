//! Config load/merge behaviour: user-set fields win, empty fields fall
//! back to built-in defaults, corrupted files degrade gracefully.

#![allow(clippy::unwrap_used)]

use std::fs;
use twatch::config::Config;

fn tmp_dir() -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("twatch-config-{}-{n}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn missing_file_yields_defaults() {
    let dir = tmp_dir();
    let config = Config::load_from(&dir);
    let defaults = Config::default();
    assert_eq!(config.download_dir, defaults.download_dir);
    assert_eq!(config.log_level, defaults.log_level);
    assert_eq!(config.history_filename, defaults.history_filename);
}

#[test]
fn user_fields_are_honored() {
    let dir = tmp_dir();
    fs::write(
        dir.join("config.json"),
        r#"{
            "download_dir": "/custom/downloads",
            "history_filename": "custom.json",
            "theme": "nord-dark",
            "log_level": "debug"
        }"#,
    )
    .unwrap();

    let config = Config::load_from(&dir);
    assert_eq!(config.download_dir, std::path::PathBuf::from("/custom/downloads"));
    assert_eq!(config.history_filename, "custom.json");
    assert_eq!(config.theme, "nord-dark");
    assert_eq!(config.log_level, "debug");
    // Unset path fields keep the built-in defaults.
    assert_eq!(config.session_path, Config::default().session_path);
}

#[test]
fn empty_fields_fall_back_to_defaults() {
    let dir = tmp_dir();
    fs::write(
        dir.join("config.json"),
        r#"{
            "download_dir": "",
            "history_filename": "",
            "theme": ""
        }"#,
    )
    .unwrap();

    let defaults = Config::default();
    let config = Config::load_from(&dir);
    assert_eq!(config.download_dir, defaults.download_dir);
    assert_eq!(config.history_filename, defaults.history_filename);
    assert_eq!(config.theme, defaults.theme);
}

#[test]
fn corrupted_file_yields_defaults() {
    let dir = tmp_dir();
    fs::write(dir.join("config.json"), "{not json").unwrap();
    let config = Config::load_from(&dir);
    assert_eq!(config.download_dir, Config::default().download_dir);
}

#[test]
fn save_and_reload_round_trips() {
    let dir = tmp_dir();
    let mut config = Config::load_from(&dir);
    config.config_dir = dir.clone();
    config.download_dir = std::path::PathBuf::from("/custom/saved");
    config.history_filename = "saved.json".into();
    config.save().unwrap();

    let reloaded = Config::load_from(&dir);
    assert_eq!(reloaded.download_dir, std::path::PathBuf::from("/custom/saved"));
    assert_eq!(reloaded.history_filename, "saved.json");
}
