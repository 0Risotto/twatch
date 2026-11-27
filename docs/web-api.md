# twatch web API

`twatchd` is a local-only Rust gateway. The SPA is static; every operation goes
through this API.

Default address: `http://127.0.0.1:7878`

## Authentication

- Bind loopback by default. Non-loopback binds require `--allow-remote` and
  print the token.
- `GET /` sets an `HttpOnly`, `SameSite=Strict` `twatch_token` cookie.
- API requests accept either the cookie or `Authorization: Bearer <token>`.
- `GET /api/health` is public. Everything else under `/api` requires the token.
- `Host` and `Origin` are checked to block DNS rebinding and cross-origin
  mutations.

Token file: `$XDG_CONFIG_HOME/twatch/server-token` (mode `0600`).

## REST

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/health` | Public liveness probe |
| GET | `/api/bootstrap` | First paint: config, session, downloads, themes |
| GET | `/api/session` | Current torrent session or `null` |
| POST | `/api/session/preview` | `{url}` — fetch metadata and make it current |
| POST | `/api/session/close` | Close the browse session (downloads keep running) |
| POST | `/api/session/files/select` | `{indices, selected}` — toggle file selection |
| POST | `/api/session/files/watch` | `{index}` — stream one file (video/audio/image) |
| POST | `/api/session/files/download` | `{indices}` — download selected files |
| PATCH | `/api/session/files/rename` | `{index, name}` — rename on disk + history |
| DELETE | `/api/session/files` | `{indices}` — delete files from disk |
| GET | `/api/downloads` | Active and recent downloads |
| POST | `/api/downloads/{id}/pause` | Pause |
| POST | `/api/downloads/{id}/resume` | Resume (or re-add a restored transfer) |
| DELETE | `/api/downloads/{id}` | Remove torrent from the engine |
| GET | `/api/library` | Indexed local media from the cache and downloads roots |
| POST | `/api/library/rescan` | Rebuild the local media index |
| GET | `/api/library/{id}/stream` | Byte-range stream for a local file |
| GET | `/api/search?q=&page=` | Search the configured JSON API |
| GET/PUT | `/api/search/config` | Read/write the search endpoint and field map |
| GET | `/api/history` | History entries |
| PATCH | `/api/history` | `{url, name}` — rename a history entry |
| DELETE | `/api/history` | `{url}` — remove a history entry |
| POST | `/api/history/open` | `{url}` — re-preview a history entry |
| PUT | `/api/history/progress` | `{url, file, position, duration}` — persist a resume point |
| GET | `/api/themes` | 56 built-in theme variants (28 bases × dark/light) |
| POST | `/api/covers` | `{items:[{title,kind?}]}` — batched movie/anime/book cover lookup |
| GET/PATCH | `/api/config` | Web theme preference |
| GET | `/api/stream/{torrentId}/{fileIndex}` | Byte-range media stream |
| GET | `/api/transcode/download/{id}/{file}` | On-demand HLS playlist/segments for a download |
| GET | `/api/transcode/library/{id}/{file}` | On-demand HLS playlist/segments for a library file |
| GET | `/api/subtitles/{torrentId}/{fileIndex}` | SRT/ASS/SSA/VTT as WebVTT (or `?format=raw`) |

All response bodies are camelCase JSON. Errors use:

```json
{ "error": { "code": 400, "message": "No active torrent session" } }
```

## Cover art

`POST /api/covers` accepts up to 60 items and returns one `Cover` per item:

```json
{
  "items": [
    { "title": "Jujutsu Kaisen S03E01 1080p", "kind": "tv" },
    { "title": "Perfume - The Story of a Murderer (2006)", "kind": "movie" }
  ]
}
```

```json
[
  {
    "query": "Jujutsu Kaisen S03E01 1080p",
    "title": "Jujutsu Kaisen",
    "url": "https://m.media-amazon.com/images/M/....jpg",
    "provider": "imdb",
    "year": 2020,
    "kind": "tv"
  }
]
```

Providers are keyless and best-effort: IMDb suggestions, AniList (anime),
TVmaze (TV fallback), and Open Library (books). Results are cached for 30 days
in `covers.json`; misses are cached for one hour. A provider failure returns
`url: null`, and the SPA falls back to the file-type icon. External poster URLs
are allowed by the page CSP via `img-src https:` and loaded with
`referrerPolicy="no-referrer"`.

## Subtitles

`GET /api/subtitles/{torrentId}/{fileIndex}` reads a subtitle file **already in
the torrent** and returns `text/vtt; charset=utf-8`:

- `.vtt` is passed through (with a `WEBVTT` header added if missing)
- `.srt` gets timestamps converted from comma to dot and a `WEBVTT` header
- `.ass` / `.ssa` dialogue is flattened to plain WebVTT cues; styling and
  positioning are intentionally dropped
- other subtitle formats return `415`

Add `?format=raw` to receive the original bytes with a format-specific
`Content-Type` (`text/x-ssa` for ASS/SSA, `text/plain` for SRT, `text/vtt` for
VTT). The SPA feeds ASS/SSA directly to libass via JASSUB for styled rendering
and keeps the default WebVTT form as a native `<track>` fallback. SRT and VTT
use the native WebVTT track path.

No external subtitle provider is contacted. `DownloadSnapshot.subtitles` lists
the torrent's convertible subtitle files, and the SPA matches them to the
playing video by episode tag or base name, then attaches them as native
`<track kind="subtitles">` elements. The player has an Off / track selector,
per-torrent track memory, a ±0.1 s / ±1 s delay control, and three cue sizes.
Delay is applied to the loaded VTT cues, so it works with the native renderer.

## Download persistence

`twatchd` writes active transfer metadata to `downloads.json` in the config
directory (mode `0600`). On startup entries are restored as **paused**; the
torrent is not re-added until `POST /api/downloads/{id}/resume`, which spawns
the stream or folder download again. Completed batch downloads are restored as
completed. IDs, queues, subtitle tracks and file indices are preserved so the
Downloads page and player can resume the same item.

The SPA separately stores the active player download in
`localStorage["twatch.active-download"]`. After a page reload it restores that
stream as a paused miniplayer at the last persisted position. If the daemon
restarted and the stream no longer exists, the key is cleared and the restored
download card is shown instead.

## Playback progress

`PUT /api/history/progress` stores the resume point for one file:

```json
{ "url": "magnet:?xt=...", "file": "Show.S01E02.mkv", "position": 742.5, "duration": 1440 }
```

Positions are clamped to the duration and invalid/non-finite values are ignored.
`GET /api/history` includes the saved points:

```json
{
  "url": "magnet:?xt=...",
  "progress": {
    "Show.S01E02.mkv": { "position": 742.5, "duration": 1440, "updatedAt": 1795280663 }
  }
}
```

Starting a stream returns the saved point as `DownloadSnapshot.resumePosition`
and the player seeks to it after metadata loads. The player writes progress
every five seconds, on pause, on tab hide, and on unmount. `DownloadSnapshot`
also carries the current `fileIndex` and a `queue` of playable files so the
player can provide next/previous/autoplay without re-fetching the session.

## Local library

The library scans two roots:

- `Config.session_path` (default `$XDG_CACHE_HOME/twatch/session`)
- `Config.download_dir` (default `$XDG_DOWNLOAD_DIR/twatch`)

Only non-empty video and audio files are indexed. Hidden directories, symlinks,
and temporary/partial files without a media extension are skipped. The scan is
recursive, capped at 10,000 files, and returns opaque ids. `/api/library/{id}/stream`
only resolves ids from that index; the request never supplies a filesystem path.
Responses use the same single-range semantics as torrent streams, and the
The browser player opens the local range stream directly.

## Transcoding

The SPA uses the direct byte-range player for MP4, M4V, WebM, and audio.
Other video containers (MKV/HEVC, and files with unsupported audio) request an
H.264/AAC HLS event playlist from `/api/transcode/...`.

The daemon probes the source with `ffprobe`. H.264 video is stream-copied;
other video codecs are transcoded with libx264. Audio is transcoded to AAC.
The first playlist and segment are produced before the playlist request
returns, so the browser does not need a separate prepare call. Temporary
segments live under `$XDG_CONFIG_HOME/twatch/transcode/` and are cleaned up on
session expiry and daemon restart. The web player uses hls.js, so it remains a
single browser `<video>` element with the existing controls and subtitles.

## Streaming

`GET|HEAD /api/stream/{torrentId}/{fileIndex}` supports a single byte range:

```http
Range: bytes=100-199
```

- `200` full response with `Accept-Ranges: bytes`, `Content-Length`,
  `Content-Type` and `Content-Disposition: inline`.
- `206` for a range with `Content-Range` and the requested `Content-Length`.
- `416` + `Content-Range: bytes */<len>` for an invalid range.
- `503` if the streaming engine is unavailable (tests), the swarm has no peers,
  or initialization exceeds 60 seconds.
- Stream opening polls progress every 250 ms. It keeps waiting while downloaded
  bytes advance, and fails after 8 seconds of no progress with zero peers.
- Files are served only from active torrents; there is no path input.

## WebSocket

`GET /api/ws` upgrades to a live-state socket. The first frame is always a full
snapshot:

```json
{
  "type": "snapshot",
  "revision": 12,
  "session": { "...": "SessionSnapshot or null" },
  "downloads": ["DownloadSnapshot..."]
}
```

Subsequent frames:

```json
{ "type": "session", "revision": 13, "session": { "...": "or null" } }
{ "type": "downloads", "revision": 14, "downloads": ["..."] }
{ "type": "toast", "revision": 16, "level": "success", "message": "Download complete" }
```

Rules:

- `revision` is monotonic. Ignore frames with an older revision.
- On a broadcast lag the server sends a fresh `snapshot` instead of replaying
  deltas.
- Progress is coalesced at **4 Hz**. The UI interpolates between ticks.
- The client reconnects with exponential backoff and refetches `/api/bootstrap`
  if the socket is unavailable.

## File preview types

The UI decides what to do with a streamed file:

| Kind | UI |
|---|---|
| `movie`, `tv` | custom `<video>` player with seek-preview thumbnails, subtitles, queue |
| `audio` | custom player with cover art and the same controls |
| `image` | `<img>` with open-original link |
| PDF/ebook/archive/code/subtitle/app/game/other | Open file or download only |

`POST /api/session/files/watch` is the generic "open one file" endpoint; the
frontend picks the presentation from the file extension.
