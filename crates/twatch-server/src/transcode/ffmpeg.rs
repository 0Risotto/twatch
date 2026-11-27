use std::path::Path;
use std::time::Duration;

use tokio::process::Command;

pub(super) async fn playlist_ready(dir: &Path) -> bool {
    let playlist = match tokio::fs::read_to_string(dir.join("index.m3u8")).await {
        Ok(playlist) => playlist,
        Err(_) => return false,
    };
    if !playlist.contains("#EXTINF") {
        return false;
    }
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(_) => return false,
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry.file_name().to_string_lossy().ends_with(".ts") {
            return true;
        }
    }
    false
}

pub(super) async fn probe_video_codec(
    input_url: &str,
    auth_header: Option<&str>,
) -> Option<String> {
    let output = tokio::time::timeout(Duration::from_secs(15), async {
        let mut command = Command::new("ffprobe");
        command.args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "default=nw=1:nk=1",
        ]);
        if let Some(header) = auth_header {
            command.args(["-headers", header]);
        }
        command.arg(input_url).output().await
    })
    .await
    .ok()?
    .ok()?;
    if !output.status.success() {
        return None;
    }
    let codec = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!codec.is_empty()).then_some(codec)
}
