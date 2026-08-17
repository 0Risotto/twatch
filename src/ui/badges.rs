//! File status badge registry.
//!
//! Each badge provider declares the condition under which it applies
//! and how it renders.  The registry is the single closed point where
//! badges are registered in display-priority order: adding a new file
//! attribute means adding one provider struct plus one row here —
//! nothing else in the codebase changes.

use crate::domain::torrent::{FileStatus, TorrentFile};
use crate::ui::Theme;
use ratatui::text::Span;

/// A status badge provider for browser file rows.
pub trait FileBadge: Send + Sync {
    /// The badge span to append to the row, if this badge applies.
    fn badge(
        &self,
        file: &TorrentFile,
        status: &FileStatus,
        theme: &Theme,
    ) -> Option<Span<'static>>;
}

/// All registered badges, in display-priority order (first match wins).
#[must_use]
pub fn registered_badges() -> &'static [&'static dyn FileBadge] {
    &[&WatchedBadge, &DownloadingBadge, &DownloadedBadge]
}

/// `[watched]` — the file has been streamed before.
pub struct WatchedBadge;

impl FileBadge for WatchedBadge {
    fn badge(
        &self,
        _file: &TorrentFile,
        status: &FileStatus,
        theme: &Theme,
    ) -> Option<Span<'static>> {
        status.watched.then(|| Span::styled(" [watched]", theme.badge_stream_style()))
    }
}

/// `[downloading]` — the file is currently being downloaded.
pub struct DownloadingBadge;

impl FileBadge for DownloadingBadge {
    fn badge(
        &self,
        _file: &TorrentFile,
        status: &FileStatus,
        theme: &Theme,
    ) -> Option<Span<'static>> {
        status.downloading.then(|| Span::styled(" [downloading]", theme.warning_style()))
    }
}

/// `[downloaded]` — the file has been downloaded before.
pub struct DownloadedBadge;

impl FileBadge for DownloadedBadge {
    fn badge(
        &self,
        _file: &TorrentFile,
        status: &FileStatus,
        theme: &Theme,
    ) -> Option<Span<'static>> {
        status.downloaded.then(|| Span::styled(" [downloaded]", theme.badge_dl_style()))
    }
}

/// The first badge that applies to `status`, if any.
#[must_use]
pub fn badge_for(file: &TorrentFile, status: &FileStatus, theme: &Theme) -> Option<Span<'static>> {
    registered_badges().iter().find_map(|badge| badge.badge(file, status, theme))
}
