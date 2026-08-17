//! `FileBadge` registry tests: a new file attribute is one badge +
//! one registration row.

use twatch::domain::torrent::{FileStatus, TorrentFile};
use twatch::ui::Theme;
use twatch::ui::badges::{badge_for, registered_badges};

fn file() -> TorrentFile {
    TorrentFile { index: 0, name: "a.mkv".into(), size: 1 }
}

#[test]
fn no_status_yields_no_badge() {
    assert!(badge_for(&file(), &FileStatus::default(), &Theme::default()).is_none());
}

#[test]
fn downloaded_status_yields_downloaded_badge() {
    let status = FileStatus { downloaded: true, ..Default::default() };
    assert!(badge_for(&file(), &status, &Theme::default()).is_some());
}

#[test]
fn watched_takes_priority_over_downloaded() {
    let status = FileStatus { watched: true, downloaded: true, ..Default::default() };
    let span = badge_for(&file(), &status, &Theme::default());
    assert!(span.is_some_and(|s| s.content.contains("watched")));
}

#[test]
fn registry_is_non_empty() {
    assert!(!registered_badges().is_empty());
}
