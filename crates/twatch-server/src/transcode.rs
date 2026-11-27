//! On-demand HLS transcoding for browser playback.
//!
//! Browser-friendly files still use the direct range stream.  Files that
//! Chromium cannot decode reliably (MKV/HEVC, DTS, …) are fed to a local
//! ffmpeg process and exposed as an HLS event playlist.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use tokio::process::{Child, Command};
use tokio::sync::Mutex;

const START_TIMEOUT: Duration = Duration::from_secs(45);
const POLL_INTERVAL: Duration = Duration::from_millis(250);
const SESSION_TTL: Duration = Duration::from_secs(2 * 60 * 60);

struct Session {
    dir: PathBuf,
    child: Child,
    last_access: Instant,
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

/// Owns ffmpeg sessions and their temporary HLS output.
pub struct TranscodeService {
    root: PathBuf,
    sessions: Mutex<HashMap<String, Session>>,
}

impl TranscodeService {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        // Stale segments from a previous daemon are never reusable.
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::create_dir_all(&root);
        Self { root, sessions: Mutex::new(HashMap::new()) }
    }

    /// Start or reuse a transcode session and wait for the first HLS segment.
    pub async fn ensure(
        &self,
        key: &str,
        input_url: &str,
        token: Option<&str>,
    ) -> Result<PathBuf, String> {
        if !valid_key(key) {
            return Err("Invalid transcode key".into());
        }
        let dir = self.root.join(key);
        let mut sessions = self.sessions.lock().await;
        let stale: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.last_access.elapsed() >= SESSION_TTL)
            .map(|(key, _)| key.clone())
            .collect();
        for stale_key in stale {
            if let Some(mut session) = sessions.remove(&stale_key) {
                let _ = session.child.start_kill();
                let _ = tokio::fs::remove_dir_all(&session.dir).await;
            }
        }

        if let Some(session) = sessions.get_mut(key) {
            if playlist_ready(&session.dir).await {
                session.last_access = Instant::now();
                return Ok(session.dir.clone());
            }
            if let Some(mut old) = sessions.remove(key) {
                let _ = old.child.kill().await;
            }
            let _ = tokio::fs::remove_dir_all(&dir).await;
        }

        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|error| format!("Could not create transcode directory: {error}"))?;

        let auth_header = token.map(|token| format!("Authorization: Bearer {token}\r\n"));
        let video_codec = probe_video_codec(input_url, auth_header.as_deref()).await;
        let log_path = dir.join("ffmpeg.log");
        let log = std::fs::File::create(&log_path)
            .map_err(|error| format!("Could not create ffmpeg log: {error}"))?;
        let mut command = Command::new("ffmpeg");
        command.arg("-hide_banner").arg("-loglevel").arg("warning").arg("-y");
        if let Some(header) = &auth_header {
            command.args(["-headers", header]);
        }
        command
            .arg("-i")
            .arg(input_url)
            .args(["-map", "0:v:0", "-map", "0:a:0?"])
            .args(["-sn", "-dn"]);

        if video_codec.as_deref() == Some("h264") {
            command.args(["-c:v", "copy"]);
        } else {
            command.args([
                "-c:v", "libx264", "-preset", "veryfast", "-crf", "22", "-pix_fmt", "yuv420p",
            ]);
        }

        command
            .args(["-c:a", "aac", "-b:a", "192k", "-ac", "2"])
            .args([
                "-f",
                "hls",
                "-hls_time",
                "4",
                "-hls_playlist_type",
                "event",
                "-hls_list_size",
                "0",
                "-hls_segment_filename",
            ])
            .arg(dir.join("seg%05d.ts"))
            .arg(dir.join("index.m3u8"))
            .stdout(Stdio::null())
            .stderr(Stdio::from(log));

        let child = command.spawn().map_err(|error| {
            let _ = std::fs::remove_dir_all(&dir);
            format!("Could not launch ffmpeg: {error}")
        })?;

        sessions.insert(
            key.to_owned(),
            Session { dir: dir.clone(), child, last_access: Instant::now() },
        );

        let started = Instant::now();
        loop {
            if playlist_ready(&dir).await {
                if let Some(session) = sessions.get_mut(key) {
                    session.last_access = Instant::now();
                }
                return Ok(dir);
            }

            if let Some(session) = sessions.get_mut(key) {
                match session.child.try_wait() {
                    Ok(Some(status)) => {
                        let log = tokio::fs::read_to_string(&log_path)
                            .await
                            .unwrap_or_else(|_| "ffmpeg exited without a log".into());
                        sessions.remove(key);
                        let _ = tokio::fs::remove_dir_all(&dir).await;
                        return Err(format!("ffmpeg exited with {status}: {log}"));
                    }
                    Ok(None) => {}
                    Err(error) => {
                        sessions.remove(key);
                        let _ = tokio::fs::remove_dir_all(&dir).await;
                        return Err(format!("Could not check ffmpeg: {error}"));
                    }
                }
            }

            if started.elapsed() >= START_TIMEOUT {
                sessions.remove(key);
                let _ = tokio::fs::remove_dir_all(&dir).await;
                return Err("Timed out waiting for ffmpeg HLS output".into());
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    /// Read one HLS file.  The key must already have an active session.
    pub async fn read(&self, key: &str, file: &str) -> Option<Vec<u8>> {
        if !valid_key(key) || !valid_file(file) {
            return None;
        }
        let mut sessions = self.sessions.lock().await;
        let session = sessions.get_mut(key)?;
        session.last_access = Instant::now();
        tokio::fs::read(session.dir.join(file)).await.ok()
    }

    /// Stop a session and remove its temporary segments.
    pub async fn stop(&self, key: &str) {
        if !valid_key(key) {
            return;
        }
        let mut sessions = self.sessions.lock().await;
        if let Some(mut session) = sessions.remove(key) {
            let _ = session.child.kill().await;
            let _ = tokio::fs::remove_dir_all(&session.dir).await;
        }
    }
}

async fn playlist_ready(dir: &Path) -> bool {
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

async fn probe_video_codec(input_url: &str, auth_header: Option<&str>) -> Option<String> {
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

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 160
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn valid_file(file: &str) -> bool {
    !file.is_empty()
        && !file.contains("..")
        && !file.contains('/')
        && !file.contains('\\')
        && (file.ends_with(".m3u8") || file.ends_with(".ts"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    async fn ffmpeg_available() -> bool {
        Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .is_ok_and(|status| status.success())
    }

    fn scratch_dir() -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        std::env::temp_dir().join(format!("twatch-transcode-{}-{nanos}", std::process::id()))
    }

    #[tokio::test]
    async fn produces_hls_from_a_generated_file() {
        if !ffmpeg_available().await {
            return;
        }
        let scratch = scratch_dir();
        std::fs::create_dir_all(&scratch).expect("create scratch");
        let source = scratch.join("sample.mkv");
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=160x120:rate=10",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=44100",
                "-t",
                "2",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-c:a",
                "aac",
            ])
            .arg(&source)
            .status()
            .await
            .expect("run ffmpeg fixture");
        if !status.success() {
            let _ = std::fs::remove_dir_all(&scratch);
            return;
        }

        let service = TranscodeService::new(scratch.join("hls"));
        let result = service.ensure("fixture", source.to_str().expect("utf8 path"), None).await;
        let dir = result.expect("transcode should produce HLS");
        let playlist = std::fs::read_to_string(dir.join("index.m3u8")).expect("playlist");
        assert!(playlist.contains("#EXTINF"));
        assert!(service.read("fixture", "seg00000.ts").await.is_some());

        service.stop("fixture").await;
        let _ = std::fs::remove_dir_all(scratch);
    }
}
