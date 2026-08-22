//! Shared session state used by [`super::App`] and the screens.

use crate::domain::torrent::{FileStatus, FileTree, TorrentFile};
use crate::ui::Theme;

/// The current torrent session: the torrent being browsed plus the
/// per-file lifecycle lists.  Owned by the app shell, shared by the
/// browser/player/rename screens and the background flows.
#[derive(Debug, Clone, Default)]
pub struct TorrentSession {
    pub torrent_name: Option<String>,
    pub pending_url: Option<String>,
    pub selected_files: Vec<bool>,
    pub watched_files: Vec<String>,
    pub downloaded_files: Vec<String>,
    pub downloading_files: Vec<String>,
    pub tree: FileTree,
}

impl TorrentSession {
    /// Load a freshly previewed torrent into the session.
    pub fn set_torrent(&mut self, name: String, url: String, files: Vec<TorrentFile>) {
        self.torrent_name = Some(name);
        self.pending_url = Some(url);
        self.selected_files = vec![false; files.len()];
        self.tree = FileTree::new(files);
    }

    /// Drop all session state (used when leaving the browser flow).
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Number of currently toggled files.
    #[must_use]
    pub fn selected_count(&self) -> usize {
        self.selected_files.iter().filter(|&&s| s).count()
    }

    /// Indices of currently toggled files.
    #[must_use]
    pub fn selected_indices(&self) -> Vec<usize> {
        self.selected_files.iter().enumerate().filter(|(_, s)| **s).map(|(i, _)| i).collect()
    }

    /// Toggle the selection flag of the file at `file_index`.
    pub fn toggle_selection(&mut self, file_index: usize) {
        if let Some(sel) = self.selected_files.get_mut(file_index) {
            *sel = !*sel;
        }
    }

    /// Rename a file inside the tree.  Returns `false` when the index
    /// does not exist.
    pub fn rename_file(&mut self, file_index: usize, new_name: &str) -> bool {
        self.tree.rename_file(file_index, new_name)
    }

    /// The lifecycle status of a file, aggregated from the watched /
    /// downloaded / downloading lists.  Single source of truth for
    /// UI badges.
    #[must_use]
    pub fn file_status(&self, file_name: &str) -> FileStatus {
        FileStatus {
            watched: self.watched_files.iter().any(|f| f == file_name),
            downloaded: self.downloaded_files.iter().any(|f| f == file_name),
            downloading: self.downloading_files.iter().any(|f| f == file_name),
        }
    }

    /// The root subfolder downloads land in, mirroring librqbit's
    /// default behaviour: multi-file torrents get `<torrent name>/`,
    /// single-file torrents go directly into the download directory.
    #[must_use]
    pub fn download_root(&self) -> Option<&str> {
        if self.tree.files().len() > 1 { self.torrent_name.as_deref() } else { None }
    }

    /// The directory (inside `download_dir`) where this torrent's
    /// files are written.
    #[must_use]
    pub fn download_root_dir(&self, download_dir: &std::path::Path) -> std::path::PathBuf {
        match self.download_root() {
            Some(root) => download_dir.join(root),
            None => download_dir.to_path_buf(),
        }
    }

    /// The absolute path of a file within the download directory.
    #[must_use]
    pub fn download_path(
        &self,
        download_dir: &std::path::Path,
        file_name: &str,
    ) -> std::path::PathBuf {
        self.download_root_dir(download_dir).join(file_name)
    }
}

/// Text editing state with cursor tracking.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub value: String,
    pub cursor: usize,
}

/// Maximum length for user-edited text inputs.
pub const MAX_INPUT_LEN: usize = 8192;

impl InputState {
    /// Create an input pre-filled with `value`, cursor at the end.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let mut input = Self { value, cursor: 0 };
        input.cursor = input.value.len();
        input
    }

    /// Insert a character at the cursor position.
    pub fn push_char(&mut self, c: char) {
        if self.value.len() + c.len_utf8() > MAX_INPUT_LEN {
            return;
        }
        let at = self.clamped_cursor();
        self.value.insert(at, c);
        self.cursor = at + c.len_utf8();
    }

    /// Delete the character before the cursor.
    pub fn backspace(&mut self) {
        let at = self.clamped_cursor();
        if at == 0 {
            return;
        }
        let prev = self.prev_char_boundary(at);
        self.value.remove(prev);
        self.cursor = prev;
    }

    /// Move the cursor one character left.
    pub fn cursor_left(&mut self) {
        let at = self.clamped_cursor();
        self.cursor = self.prev_char_boundary(at);
    }

    /// Move the cursor one character right.
    pub fn cursor_right(&mut self) {
        let at = self.clamped_cursor();
        self.cursor = self.next_char_boundary(at);
    }

    /// The cursor clamped to a valid char boundary in `value`.
    fn clamped_cursor(&self) -> usize {
        self.snap_forward(self.cursor.min(self.value.len()))
    }

    /// Start byte of the character immediately before boundary `at`.
    fn prev_char_boundary(&self, at: usize) -> usize {
        match at.checked_sub(1) {
            None => 0,
            Some(step) => self.snap_backward(step),
        }
    }

    /// End byte of the character starting at boundary `at`.
    fn next_char_boundary(&self, at: usize) -> usize {
        if at >= self.value.len() {
            return self.value.len();
        }
        self.snap_forward(at + 1)
    }

    /// Nearest char boundary at or after `at`.  Uses `is_char_boundary`
    /// rather than slicing, which would itself panic off-boundary.
    fn snap_forward(&self, mut at: usize) -> usize {
        while at < self.value.len() && !self.value.is_char_boundary(at) {
            at += 1;
        }
        at
    }

    /// Nearest char boundary at or before `at`.
    fn snap_backward(&self, mut at: usize) -> usize {
        while at > 0 && !self.value.is_char_boundary(at) {
            at -= 1;
        }
        at
    }
}

/// The active theme.
#[derive(Debug, Clone, Default)]
pub struct ThemeState {
    pub theme: Theme,
}
