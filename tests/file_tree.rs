//! `FileTree` tests: folder projection, expansion, and fuzzy matching.

use twatch::domain::torrent::DisplayEntry;
use twatch::domain::torrent::TorrentFile;
use twatch::domain::torrent::file_tree::{FileTree, entry_matches};

fn file(index: usize, name: &str) -> TorrentFile {
    TorrentFile { index, name: name.to_string(), size: 10 }
}

#[test]
fn folders_sort_before_files() {
    let tree =
        FileTree::new(vec![file(0, "README.txt"), file(1, "dir/a.mkv"), file(2, "dir/b.mkv")]);
    let entries = tree.entries();
    assert!(matches!(entries[0], DisplayEntry::Folder { .. }));
    assert!(matches!(entries[1], DisplayEntry::File { .. }));
    assert_eq!(entries.len(), 2);
}

#[test]
fn toggle_expands_and_collapses() {
    let mut tree = FileTree::new(vec![file(0, "dir/a.mkv"), file(1, "dir/b.mkv")]);
    assert_eq!(tree.entries().len(), 1);
    tree.toggle_folder_at(0);
    assert_eq!(tree.entries().len(), 3);
    tree.toggle_folder_at(0);
    assert_eq!(tree.entries().len(), 1);
}

#[test]
fn folder_path_reconstructs_ancestors() {
    let mut tree = FileTree::new(vec![file(0, "a/b/c.mkv"), file(1, "a/other.mkv")]);
    // Expand "a/"; the nested folder "b" is then entry 1.
    tree.toggle_folder_at(0);
    assert_eq!(tree.folder_path_at(1), Some("a/b/".to_string()));
}

#[test]
fn folder_path_none_for_file_entries() {
    let tree = FileTree::new(vec![file(0, "a.mkv")]);
    assert_eq!(tree.folder_path_at(0), None);
}

#[test]
fn entry_matches_fuzzy_on_names() {
    let folder = DisplayEntry::Folder { name: "Season 1".into(), depth: 0, expanded: false };
    let entry = DisplayEntry::File { file: file(0, "episode.mkv"), depth: 0 };
    assert!(entry_matches("s1", &folder));
    assert!(entry_matches("epi", &entry));
    assert!(!entry_matches("xyz", &entry));
}
