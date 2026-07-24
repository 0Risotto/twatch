#![allow(clippy::unwrap_used)]

use std::sync::Mutex;
use twatch::model::display_name;
use twatch::service::storage::{RealStorage, StorageState};
use twatch::traits::StorageService;

mod common;

fn test_storage(tmp: &std::path::Path, file: &str) -> RealStorage {
    let state = StorageState::new(tmp.to_path_buf(), file).unwrap();
    RealStorage { state: Mutex::new(state) }
}

#[test]
fn add_and_list_entries() {
    let tmp = std::env::temp_dir().join("twatch_test_storage_add");
    let _ = std::fs::remove_dir_all(&tmp);
    let s = test_storage(&tmp, "add.json");

    s.add_entry("magnet:?xt=urn:btih:abc", "Test Torrent");
    let history = s.history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].url, "magnet:?xt=urn:btih:abc");
    assert_eq!(history[0].torrent_name, "Test Torrent");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn duplicate_url_moves_to_front() {
    let tmp = std::env::temp_dir().join("twatch_test_storage_dup");
    let _ = std::fs::remove_dir_all(&tmp);
    let s = test_storage(&tmp, "dup.json");

    s.add_entry("magnet:a", "First");
    s.add_entry("magnet:b", "Second");
    s.add_entry("magnet:a", "First Again");
    let history = s.history();
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].url, "magnet:a");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn rename_persists() {
    let tmp = std::env::temp_dir().join("twatch_test_storage_rename");
    let _ = std::fs::remove_dir_all(&tmp);
    let s = test_storage(&tmp, "rename.json");

    s.add_entry("magnet:a", "Original");
    s.rename_entry(0, "Custom Name").unwrap();
    let history = s.history();
    assert_eq!(display_name(&history[0]), "Custom Name");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn remove_entry_shrinks_list() {
    let tmp = std::env::temp_dir().join("twatch_test_storage_remove");
    let _ = std::fs::remove_dir_all(&tmp);
    let s = test_storage(&tmp, "remove.json");

    s.add_entry("magnet:a", "A");
    s.add_entry("magnet:b", "B");
    s.remove_entry(0).unwrap();
    assert_eq!(s.history().len(), 1);
    assert_eq!(s.history()[0].url, "magnet:b");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn mock_storage_crud() {
    let s = common::storage::MockStorageService::new();
    assert!(s.history().is_empty());

    s.add_entry("magnet:a", "Alpha");
    s.add_entry("magnet:b", "Beta");
    assert_eq!(s.history().len(), 2);

    s.rename_entry(0, "Renamed").unwrap();
    assert_eq!(display_name(&s.history()[0]), "Renamed");

    s.remove_entry(1).unwrap();
    assert_eq!(s.history().len(), 1);
}
