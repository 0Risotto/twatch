//! Collapsible folder-tree projection of a torrent's file list.
//!
//! Owns the file list, folder-expansion state, and the flat
//! `display_entries` projection derived from them.

use super::{DisplayEntry, TorrentFile};
use crate::domain::common::fuzzy_match;
use std::collections::{BTreeSet, HashSet};

/// The tree of files within a torrent, projected into a flat,
/// expandable folder view.
#[derive(Debug, Clone, Default)]
pub struct FileTree {
    files: Vec<TorrentFile>,
    expanded: HashSet<String>,
    entries: Vec<DisplayEntry>,
}

impl FileTree {
    /// Create a tree over `files` with no folders expanded.
    #[must_use]
    pub fn new(files: Vec<TorrentFile>) -> Self {
        Self::from_expanded(files, HashSet::new())
    }

    /// Create a tree over `files` with the given folders expanded.
    #[must_use]
    pub fn from_expanded(files: Vec<TorrentFile>, expanded: HashSet<String>) -> Self {
        let mut tree = Self { files, expanded, entries: Vec::new() };
        tree.rebuild();
        tree
    }

    /// The raw torrent file list.
    #[must_use]
    pub fn files(&self) -> &[TorrentFile] {
        &self.files
    }

    /// Currently expanded folder paths (trailing-slash form).
    #[must_use]
    pub const fn expanded_paths(&self) -> &HashSet<String> {
        &self.expanded
    }

    /// The flat projection: folders first, then files, per depth.
    #[must_use]
    pub fn entries(&self) -> &[DisplayEntry] {
        &self.entries
    }

    /// Consume the tree, returning the projected entries.
    #[must_use]
    pub fn into_entries(self) -> Vec<DisplayEntry> {
        self.entries
    }

    /// Recompute `entries` from `files` + `expanded`.
    pub fn rebuild(&mut self) {
        self.entries.clear();
        rebuild_level(&self.files, "", 0, &self.expanded, &mut self.entries);
    }

    /// Rename a file by its torrent file index, keeping the current
    /// folder-expansion state.  Returns `false` when the index does
    /// not exist.
    pub fn rename_file(&mut self, file_index: usize, new_name: &str) -> bool {
        let Some(file) = self.files.iter_mut().find(|f| f.index == file_index) else {
            return false;
        };
        file.name = new_name.to_string();
        self.rebuild();
        true
    }

    /// Toggle expansion of the folder displayed at `entry_idx`.
    ///
    /// No-op when the entry is not a folder.
    pub fn toggle_folder_at(&mut self, entry_idx: usize) {
        let Some(path) = self.folder_path_at(entry_idx) else {
            return;
        };
        if self.expanded.contains(&path) {
            self.expanded.remove(&path);
        } else {
            self.expanded.insert(path);
        }
        self.rebuild();
    }

    /// Full path (trailing-slash form) of the folder displayed at
    /// `entry_idx`, reconstructed from its ancestor folders.
    ///
    /// `None` when the entry is not a folder.
    #[must_use]
    pub fn folder_path_at(&self, entry_idx: usize) -> Option<String> {
        let DisplayEntry::Folder { name, depth, .. } = self.entries.get(entry_idx)? else {
            return None;
        };
        let depth = *depth;

        let mut ancestors: Vec<&str> = Vec::with_capacity(depth);
        for i in (0..entry_idx).rev() {
            if let Some(DisplayEntry::Folder { name: n, depth: d, .. }) = self.entries.get(i) {
                if *d < depth && ancestors.len() < depth {
                    ancestors.push(n.as_str());
                }
            }
        }
        ancestors.reverse();
        ancestors.push(name.as_str());

        let mut full = String::new();
        for seg in ancestors {
            full.push_str(seg);
            full.push('/');
        }
        Some(full)
    }
}

/// True when `query` fuzzy-matches an entry's visible text.
#[must_use]
pub fn entry_matches(query: &str, entry: &DisplayEntry) -> bool {
    match entry {
        DisplayEntry::Folder { name, .. } => fuzzy_match(query, name),
        DisplayEntry::File { file, .. } => fuzzy_match(query, &file.name),
    }
}

fn rebuild_level(
    files: &[TorrentFile],
    prefix: &str,
    depth: usize,
    expanded: &HashSet<String>,
    entries: &mut Vec<DisplayEntry>,
) {
    let mut folders = BTreeSet::new();
    let mut file_indices = Vec::new();

    for (i, file) in files.iter().enumerate() {
        if !file.name.starts_with(prefix) {
            continue;
        }
        let rest = &file.name[prefix.len()..];
        if let Some(slash_idx) = rest.find('/') {
            let folder_name = &rest[..slash_idx];
            folders.insert(folder_name.to_string());
        } else {
            file_indices.push(i);
        }
    }

    for folder_name in &folders {
        let full = format!("{prefix}{folder_name}/");
        let is_expanded = expanded.contains(&full);

        entries.push(DisplayEntry::Folder {
            name: folder_name.clone(),
            depth,
            expanded: is_expanded,
        });

        if is_expanded {
            rebuild_level(files, &full, depth + 1, expanded, entries);
        }
    }

    for i in &file_indices {
        entries.push(DisplayEntry::File { file: files[*i].clone(), depth });
    }
}
