//! In-torrent subtitle tracks converted to WebVTT.
//!
//! Browsers only understand WebVTT in `<track>`. SRT is a near-direct
//! conversion; ASS/SSA is flattened to plain dialogue cues (styles, positioning
//! and karaoke effects are intentionally dropped). No external subtitle service
//! is involved — only files that are already inside the torrent.

use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tokio::io::AsyncReadExt;

use twatch_core::domain::torrent::TorrentId;

use crate::state::AppState;

const MAX_SUBTITLE_BYTES: u64 = 4 * 1024 * 1024;

/// `GET /api/subtitles/{torrentId}/{fileIndex}`.
pub async fn subtitles(
    State(state): State<AppState>,
    Path((torrent_id, file_idx)): Path<(usize, usize)>,
) -> Response {
    let Some(stream_service) = &state.stream else {
        return (StatusCode::SERVICE_UNAVAILABLE, "Stream engine unavailable").into_response();
    };
    let torrent = TorrentId::new(torrent_id);
    let Some(name) = stream_service.file_name(torrent, file_idx) else {
        return (StatusCode::NOT_FOUND, "Subtitle file not found").into_response();
    };
    let extension = name.rsplit('.').next().unwrap_or_default().to_ascii_lowercase();

    let stream = match stream_service.open(torrent, file_idx) {
        Ok(stream) => stream,
        Err(error) => {
            tracing::warn!("Subtitle open failed for {torrent_id}/{file_idx}: {error:#}");
            return (StatusCode::NOT_FOUND, "Subtitle file not found").into_response();
        }
    };
    let mut data = Vec::new();
    if let Err(error) = stream.take(MAX_SUBTITLE_BYTES).read_to_end(&mut data).await {
        tracing::warn!("Subtitle read failed for {torrent_id}/{file_idx}: {error:#}");
        return (StatusCode::BAD_GATEWAY, "Could not read subtitle").into_response();
    }
    let text = String::from_utf8_lossy(&data);

    let vtt = match extension.as_str() {
        "vtt" => ensure_vtt(&text),
        "srt" => srt_to_vtt(&text),
        "ass" | "ssa" => ass_to_vtt(&text),
        _ => {
            return (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Only SRT, ASS, SSA and VTT subtitles can be previewed",
            )
                .into_response();
        }
    };

    let mut response = vtt.into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/vtt; charset=utf-8"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("inline"));
    response
}

fn ensure_vtt(text: &str) -> String {
    let text = text.trim_start_matches('\u{feff}');
    if text.trim_start().starts_with("WEBVTT") {
        text.to_owned()
    } else {
        format!("WEBVTT\n\n{text}")
    }
}

fn srt_to_vtt(srt: &str) -> String {
    let normalized = srt.replace("\r\n", "\n").replace('\r', "\n");
    let mut output = String::from("WEBVTT\n\n");
    for line in normalized.trim_start_matches('\u{feff}').lines() {
        if line.contains("-->") {
            output.push_str(&line.replace(',', "."));
        } else {
            output.push_str(&strip_braces(line));
        }
        output.push('\n');
    }
    output
}

fn ass_to_vtt(ass: &str) -> String {
    let mut output = String::from("WEBVTT\n\n");
    let mut in_events = false;
    let mut fields: Vec<String> = default_ass_fields();

    for raw_line in ass.lines() {
        let line = raw_line.trim_end();
        if line.starts_with('[') && line.ends_with(']') {
            in_events = line.eq_ignore_ascii_case("[Events]");
            continue;
        }
        if !in_events {
            continue;
        }
        if let Some(format) = line.strip_prefix("Format:") {
            let parsed: Vec<String> =
                format.split(',').map(|field| field.trim().to_owned()).collect();
            if !parsed.is_empty() {
                fields = parsed;
            }
            continue;
        }
        let Some(dialogue) = line.strip_prefix("Dialogue:") else {
            continue;
        };

        let parts: Vec<&str> = dialogue.splitn(fields.len().max(1), ',').collect();
        let value = |name: &str| -> &str {
            fields
                .iter()
                .position(|field| field.eq_ignore_ascii_case(name))
                .and_then(|index| parts.get(index))
                .copied()
                .unwrap_or_default()
                .trim()
        };

        let Some(start) = ass_time(value("Start")) else {
            continue;
        };
        let Some(end) = ass_time(value("End")) else {
            continue;
        };
        let text = value("Text").replace("\\N", "\n").replace("\\n", "\n").replace("\\h", " ");
        let text = strip_braces(&text);
        if text.trim().is_empty() {
            continue;
        }

        output.push_str(&start);
        output.push_str(" --> ");
        output.push_str(&end);
        output.push('\n');
        output.push_str(text.trim());
        output.push_str("\n\n");
    }
    output
}

fn default_ass_fields() -> Vec<String> {
    ["Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect", "Text"]
        .iter()
        .map(|field| (*field).to_owned())
        .collect()
}

fn ass_time(value: &str) -> Option<String> {
    let mut pieces = value.split(':');
    let hours: u32 = pieces.next()?.trim().parse().ok()?;
    let minutes: u32 = pieces.next()?.trim().parse().ok()?;
    let seconds_part = pieces.next()?.trim();
    if pieces.next().is_some() {
        return None;
    }
    let (seconds, fraction) = seconds_part.split_once('.')?;
    let seconds: u32 = seconds.parse().ok()?;
    let mut fraction = fraction.to_owned();
    // ASS uses centiseconds; SSA may use milliseconds. Normalize both.
    fraction.truncate(3);
    while fraction.len() < 3 {
        fraction.push('0');
    }
    if fraction.len() == 2 {
        fraction.push('0');
    }
    Some(format!("{hours:02}:{minutes:02}:{seconds:02}.{fraction}"))
}

/// Strip SRT/ASS override blocks (`{\\an8}`, `{\\i1}`) without touching visible text.
fn strip_braces(line: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let mut inside = false;
    for character in line.chars() {
        match character {
            '{' => inside = true,
            '}' if inside => inside = false,
            _ if !inside => output.push(character),
            _ => {}
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{ass_time, ass_to_vtt, srt_to_vtt};

    #[test]
    fn converts_srt() {
        let srt = "1\r\n00:00:01,000 --> 00:00:02,500\r\nHello\r\n\r\n2\r\n00:00:03,000 --> 00:00:04,000\r\nWorld\r\n";
        let vtt = srt_to_vtt(srt);
        assert!(vtt.starts_with("WEBVTT"));
        assert!(vtt.contains("00:00:01.000 --> 00:00:02.500"));
        assert!(vtt.contains("Hello"));
    }

    #[test]
    fn converts_ass_dialogue() {
        let ass = "[Script Info]\nTitle: test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:02.50,Default,,0,0,0,,Hello\\Nworld\nDialogue: 0,0:00:03.00,0:00:04.00,Default,,0,0,0,,{\\i1}Styled{\\i0} text\n";
        let vtt = ass_to_vtt(ass);
        assert!(vtt.starts_with("WEBVTT"));
        assert!(vtt.contains("00:00:01.000 --> 00:00:02.500"));
        assert!(vtt.contains("Hello\nworld"));
        assert!(vtt.contains("Styled text"));
    }

    #[test]
    fn parses_ass_times() {
        assert_eq!(ass_time("0:00:01.00").as_deref(), Some("00:00:01.000"));
        assert_eq!(ass_time("1:02:03.456").as_deref(), Some("01:02:03.456"));
        assert!(ass_time("bad").is_none());
    }
}
