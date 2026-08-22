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
fn folder_path_skips_preceding_sibling_folders() {
    // Regression: the parent chain must follow strictly-decreasing
    // depths, not "the nearest N lower-depth folders".  With a sibling
    // folder A before B, the old heuristic built "A/B/" for C.
    let mut tree =
        FileTree::new(vec![file(0, "P/A/a1.txt"), file(1, "P/B/C/c1.txt"), file(2, "P/B/b2.txt")]);
    tree.toggle_folder_at(0); // expand P -> [P, A, B, b2.txt]
    tree.toggle_folder_at(1); // expand A -> [P, A, a1.txt, B, b2.txt]
    tree.toggle_folder_at(3); // expand B -> [P, A, a1.txt, B, C, c1.txt, b2.txt]

    // Entries: 0=P(d0) 1=A(d1) 2=a1.txt(d2) 3=B(d1) 4=C(d2) 5=c1.txt(d3)
    assert_eq!(tree.folder_path_at(4), Some("P/B/C/".to_string()));
}

#[test]
fn toggle_deeply_nested_folder_with_sibling_parent() {
    // End-to-end: expanding and collapsing C must actually change the
    // projection (the old wrong path silently no-oped).
    let mut tree = FileTree::new(vec![file(0, "P/A/a1.txt"), file(1, "P/B/C/c1.txt")]);
    tree.toggle_folder_at(0); // P
    tree.toggle_folder_at(1); // A
    let b_idx = tree.entries().len() - 1; // B sits after A's subtree
    assert!(matches!(&tree.entries()[b_idx], DisplayEntry::Folder { name, .. } if name == "B"));
    tree.toggle_folder_at(b_idx); // B
    let c_idx = b_idx + 1;
    assert_eq!(
        tree.folder_path_at(c_idx),
        Some("P/B/C/".to_string()),
        "C must resolve to its true path P/B/C/"
    );

    let before = tree.entries().len();
    tree.toggle_folder_at(c_idx); // expand C via its true path
    assert_eq!(tree.entries().len(), before + 1, "expanding C must reveal c1.txt");
    tree.toggle_folder_at(c_idx); // collapse C again
    assert_eq!(tree.entries().len(), before, "collapsing C must hide c1.txt");
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
