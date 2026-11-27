//! Shared media classification used by downloads and the local library.

use super::*;

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "mkv", "webm", "avi", "mov", "wmv", "flv", "mpg", "mpeg", "m2ts", "ts", "vob",
    "ogv", "rmvb",
];
const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "aac", "m4a", "mka", "ogg", "oga", "opus", "wma", "aiff", "alac", "ape",
];
const IMAGE_EXTENSIONS: &[&str] =
    &["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg", "avif", "heic", "tiff", "tif"];
const DIRECT_VIDEO_EXTENSIONS: &[&str] = &["mp4", "m4v", "webm", "ogv"];

fn extension(name: &str) -> String {
    name.rsplit('.').next().unwrap_or_default().to_ascii_lowercase()
}

#[must_use]
pub(crate) fn media_kind(name: &str) -> MediaKind {
    let ext = extension(name);
    if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        MediaKind::Video
    } else if AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        MediaKind::Audio
    } else if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        MediaKind::Image
    } else {
        MediaKind::Other
    }
}

#[must_use]
pub(crate) fn playback_mode(name: &str) -> PlaybackMode {
    let ext = extension(name);
    if media_kind(name) == MediaKind::Video && !DIRECT_VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        PlaybackMode::Hls
    } else {
        PlaybackMode::Direct
    }
}

#[must_use]
pub(crate) fn descriptor(
    id: MediaId,
    title: impl Into<String>,
    filename: impl Into<String>,
    size: u64,
    direct_stream_path: Option<String>,
) -> MediaDescriptor {
    let filename = filename.into();
    let kind = media_kind(&filename);
    let playback = playback_mode(&filename);
    let stream_path = direct_stream_path.map(|direct| match playback {
        PlaybackMode::Direct => direct,
        PlaybackMode::Hls => match &id {
            MediaId::Download(download) => {
                format!("/api/transcode/download/{download}/index.m3u8")
            }
            MediaId::Library(library) => {
                format!("/api/transcode/library/{library}/index.m3u8")
            }
        },
    });
    MediaDescriptor { id, title: title.into(), filename, kind, playback, stream_path, size }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_media_and_playback_modes() {
        assert_eq!(media_kind("movie.MKV"), MediaKind::Video);
        assert_eq!(media_kind("track.flac"), MediaKind::Audio);
        assert_eq!(media_kind("cover.jpg"), MediaKind::Image);
        assert_eq!(media_kind("notes.txt"), MediaKind::Other);
        assert_eq!(playback_mode("movie.mkv"), PlaybackMode::Hls);
        assert_eq!(playback_mode("movie.mp4"), PlaybackMode::Direct);
        assert_eq!(playback_mode("track.flac"), PlaybackMode::Direct);
        assert_eq!(playback_mode("book.pdf"), PlaybackMode::Direct);
        assert_eq!(media_kind("track.oga"), MediaKind::Audio);
    }

    #[test]
    fn descriptor_owns_the_playback_url_decision() {
        let direct = descriptor(
            MediaId::Download(7),
            "Movie",
            "Movie.mp4",
            10,
            Some("/api/stream/7/0".into()),
        );
        assert_eq!(direct.playback, PlaybackMode::Direct);
        assert_eq!(direct.stream_path.as_deref(), Some("/api/stream/7/0"));

        let transcoded = descriptor(
            MediaId::Download(7),
            "Movie",
            "Movie.mkv",
            10,
            Some("/api/stream/7/0".into()),
        );
        assert_eq!(transcoded.playback, PlaybackMode::Hls);
        assert_eq!(transcoded.stream_path.as_deref(), Some("/api/transcode/download/7/index.m3u8"));

        let library = descriptor(
            MediaId::Library("cache-abc".into()),
            "Movie",
            "Movie.mkv",
            10,
            Some("/api/library/cache-abc/stream".into()),
        );
        assert_eq!(
            library.stream_path.as_deref(),
            Some("/api/transcode/library/cache-abc/index.m3u8")
        );

        let unavailable = descriptor(MediaId::Download(7), "Movie", "Movie.mkv", 10, None);
        assert_eq!(unavailable.stream_path, None);
    }

    #[test]
    fn media_ids_serialize_for_the_typescript_contract() {
        assert_eq!(
            serde_json::to_value(MediaId::Download(7)).unwrap(),
            serde_json::json!({ "kind": "download", "value": 7 })
        );
        assert_eq!(
            serde_json::to_value(MediaId::Library("cache-abc".into())).unwrap(),
            serde_json::json!({ "kind": "library", "value": "cache-abc" })
        );
    }
}
