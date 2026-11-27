//! Local media discovery for the cache and completed-download roots.

use super::*;
use std::time::UNIX_EPOCH;

const MAX_DEPTH: usize = 8;
const MAX_ENTRIES: usize = 10_000;

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "mkv", "avi", "mov", "webm", "wmv", "flv", "mpg", "mpeg", "m2ts", "ts", "vob",
    "ogv", "rmvb",
];
const AUDIO_EXTENSIONS: &[&str] =
    &["mp3", "flac", "wav", "aac", "m4a", "mka", "ogg", "opus", "wma", "aiff", "alac", "ape"];

#[derive(Debug, Clone)]
pub(super) struct LibraryFile {
    pub(super) entry: LibraryEntry,
    pub(super) path: PathBuf,
}

impl Gateway {
    /// Cached local media.  The first call scans both roots; later calls
    /// reuse the result until [`Self::rescan_library`] is called.
    #[must_use]
    pub fn library(&self) -> Vec<LibraryEntry> {
        self.library_entries(false)
    }

    /// Re-scan the cache and downloads roots.
    #[must_use]
    pub fn rescan_library(&self) -> Vec<LibraryEntry> {
        self.library_entries(true)
    }

    /// Resolve an opaque library id to a file path.  The id can only come
    /// from the scan, so callers cannot request arbitrary filesystem paths.
    #[must_use]
    pub fn library_file_path(&self, id: &str) -> Option<PathBuf> {
        self.find_library_file(id).map(|file| file.path)
    }

    /// Hand a library stream to the configured native player.
    pub fn open_library_file(&self, id: &str, base_url: &str) -> GatewayResult<PlayerSnapshot> {
        self.open_library_file_with_token(id, base_url, None)
    }

    /// Open a local library file in a native player with a media query token.
    pub fn open_library_file_with_token(
        &self,
        id: &str,
        base_url: &str,
        token: Option<&str>,
    ) -> GatewayResult<PlayerSnapshot> {
        let file = self.find_library_file(id).ok_or(GatewayError::LibraryNotFound)?;
        let url = stream_url(base_url, &file.entry.stream_path, token);

        self.player
            .play(&url, &file.entry.name)
            .map_err(|error| GatewayError::PlayerUnavailable(format!("{error:#}")))?;
        {
            let mut state = self.player_state.lock().unwrap_or_else(|e| e.into_inner());
            state.title = Some(file.entry.name);
            state.stream_path = Some(file.entry.stream_path);
            state.download_id = None;
        }
        self.emit(GatewayEvent::PlayerChanged);
        Ok(self.player())
    }

    fn library_entries(&self, force: bool) -> Vec<LibraryEntry> {
        if force {
            let files = scan_library(&self.config());
            let entries = files.iter().map(|file| file.entry.clone()).collect();
            *self.library.write().unwrap_or_else(|e| e.into_inner()) = Some(files);
            return entries;
        }

        self.ensure_library();
        self.library
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .map(|files| files.iter().map(|file| file.entry.clone()).collect())
            .unwrap_or_default()
    }

    fn ensure_library(&self) {
        let loaded = self.library.read().unwrap_or_else(|e| e.into_inner()).is_some();
        if loaded {
            return;
        }

        let files = scan_library(&self.config());
        let mut library = self.library.write().unwrap_or_else(|e| e.into_inner());
        if library.is_none() {
            *library = Some(files);
        }
    }

    fn find_library_file(&self, id: &str) -> Option<LibraryFile> {
        self.ensure_library();
        self.library
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .and_then(|files| files.iter().find(|file| file.entry.id == id).cloned())
    }
}

fn scan_library(config: &Config) -> Vec<LibraryFile> {
    scan_roots(&[
        (LibraryRoot::Cache, config.session_path.as_path()),
        (LibraryRoot::Downloads, config.download_dir.as_path()),
    ])
}

fn scan_roots(roots: &[(LibraryRoot, &Path)]) -> Vec<LibraryFile> {
    let mut files = Vec::new();
    let mut seen = HashSet::new();

    for (root, path) in roots {
        if !path.is_dir() {
            continue;
        }
        let mut state =
            ScanState { root: *root, root_path: path, files: &mut files, seen: &mut seen };
        walk(path, 0, &mut state);
    }

    files.sort_by(|left, right| {
        left.entry
            .name
            .to_ascii_lowercase()
            .cmp(&right.entry.name.to_ascii_lowercase())
            .then_with(|| left.entry.relative_path.cmp(&right.entry.relative_path))
    });
    files
}

struct ScanState<'a> {
    root: LibraryRoot,
    root_path: &'a Path,
    files: &'a mut Vec<LibraryFile>,
    seen: &'a mut HashSet<PathBuf>,
}

fn walk(directory: &Path, depth: usize, state: &mut ScanState<'_>) {
    if depth > MAX_DEPTH || state.files.len() >= MAX_ENTRIES {
        return;
    }
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        if state.files.len() >= MAX_ENTRIES {
            return;
        }
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            walk(&path, depth + 1, state);
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let Some(media_kind) = media_kind(&path) else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.len() == 0 {
            continue;
        }
        let canonical = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if !state.seen.insert(canonical) {
            continue;
        }

        let relative_path = relative_display_path(&path, state.root_path);
        let id = library_id(state.root, &relative_path);
        let modified_ms = metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .and_then(|duration| u64::try_from(duration.as_millis()).ok());

        state.files.push(LibraryFile {
            path,
            entry: LibraryEntry {
                stream_path: format!("/api/library/{id}/stream"),
                id,
                name: name.to_string_lossy().into_owned(),
                relative_path,
                root: state.root,
                media_kind,
                size: metadata.len(),
                modified_ms,
            },
        });
    }
}

fn media_kind(path: &Path) -> Option<LibraryMediaKind> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    if VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        Some(LibraryMediaKind::Video)
    } else if AUDIO_EXTENSIONS.contains(&extension.as_str()) {
        Some(LibraryMediaKind::Audio)
    } else {
        None
    }
}

fn relative_display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}

fn library_id(root: LibraryRoot, relative_path: &str) -> String {
    let mut hex = String::with_capacity(relative_path.len() * 2);
    for byte in relative_path.as_bytes() {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    format!("{}-{hex}", root.key())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let path = std::env::temp_dir()
            .join(format!("twatch-library-{label}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).expect("create scratch dir");
        path
    }

    #[test]
    fn scans_media_recursively_and_skips_empty_or_hidden_files() {
        let root = scratch_dir("scan");
        let nested = root.join("Season 01");
        fs::create_dir_all(&nested).expect("create nested");
        fs::write(root.join("Movie.MKV"), b"movie").expect("write movie");
        fs::write(nested.join("Episode.mp4"), b"episode").expect("write episode");
        fs::write(root.join("empty.mp4"), b"").expect("write empty");
        fs::write(root.join(".hidden.mp4"), b"hidden").expect("write hidden");
        fs::write(root.join("notes.txt"), b"notes").expect("write notes");

        let files = scan_roots(&[(LibraryRoot::Cache, root.as_path())]);
        let names: Vec<_> = files.iter().map(|file| file.entry.name.as_str()).collect();
        assert_eq!(names, vec!["Episode.mp4", "Movie.MKV"]);
        assert_eq!(files[0].entry.relative_path, "Season 01/Episode.mp4");
        assert_eq!(files[1].entry.media_kind, LibraryMediaKind::Video);
        assert!(files.iter().all(|file| file.entry.stream_path.starts_with("/api/library/")));

        fs::remove_dir_all(root).expect("remove scratch");
    }

    #[test]
    fn ids_are_stable_and_root_qualified() {
        let cache = library_id(LibraryRoot::Cache, "Movie.mkv");
        let downloads = library_id(LibraryRoot::Downloads, "Movie.mkv");
        assert_eq!(cache, library_id(LibraryRoot::Cache, "Movie.mkv"));
        assert_ne!(cache, downloads);
        assert!(cache.starts_with("cache-"));
    }
}
