//! `TorrentSession` download-path tests: files must land in
//! `<download_dir>/<torrent name>/<internal path>`, not flat.

use twatch::domain::torrent::TorrentFile;

fn file(index: usize, name: &str) -> TorrentFile {
    TorrentFile { index, name: name.to_string(), size: 10 }
}

fn session(files: Vec<TorrentFile>) -> twatch::app::state::TorrentSession {
    let mut s = twatch::app::state::TorrentSession::default();
    s.set_torrent("Torrent T".into(), "magnet:x".into(), files);
    s
}

#[test]
fn multi_file_torrent_downloads_under_torrent_name_folder() {
    let s = session(vec![file(0, "a.mkv"), file(1, "b.mkv")]);
    let root = s.download_root_dir(std::path::Path::new("/downloads"));
    assert_eq!(root, std::path::Path::new("/downloads/Torrent T"));
    assert_eq!(
        s.download_path(std::path::Path::new("/downloads"), "a.mkv"),
        std::path::Path::new("/downloads/Torrent T/a.mkv")
    );
}

#[test]
fn single_file_torrent_downloads_directly() {
    let s = session(vec![file(0, "movie.mkv")]);
    assert_eq!(s.download_root(), None);
    assert_eq!(
        s.download_path(std::path::Path::new("/downloads"), "movie.mkv"),
        std::path::Path::new("/downloads/movie.mkv")
    );
}

#[test]
fn internal_folders_are_preserved_under_the_root() {
    let s = session(vec![file(0, "Season 1/episode.mkv"), file(1, "Season 2/episode.mkv")]);
    assert_eq!(
        s.download_path(std::path::Path::new("/downloads"), "Season 1/episode.mkv"),
        std::path::Path::new("/downloads/Torrent T/Season 1/episode.mkv")
    );
}
