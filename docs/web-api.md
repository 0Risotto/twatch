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
| GET | `/api/bootstrap` | First paint: config, session, downloads, player, themes |
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
| POST | `/api/downloads/{id}/resume` | Resume |
| DELETE | `/api/downloads/{id}` | Remove torrent from the engine |
| GET | `/api/search?q=&page=` | Search the configured JSON API |
| GET/PUT | `/api/search/config` | Read/write the search endpoint and field map |
| GET | `/api/history` | History entries |
| PATCH | `/api/history` | `{url, name}` — rename a history entry |
| DELETE | `/api/history` | `{url}` — remove a history entry |
| POST | `/api/history/open` | `{url}` — re-preview a history entry |
| GET | `/api/player` | External player state |
| POST | `/api/player/open` | `{downloadId}` — hand the stream to mpv/VLC |
| POST | `/api/player/stop` | Kill the external player |
| GET | `/api/themes` | 54 built-in theme variants (27 bases × dark/light) |
| POST | `/api/covers` | `{items:[{title,kind?}]}` — batched movie/anime/book cover lookup |
| GET/PATCH | `/api/config` | Web theme and external player preference |
| GET | `/api/stream/{torrentId}/{fileIndex}` | Byte-range media stream |
| GET | `/api/subtitles/{torrentId}/{fileIndex}` | SRT/ASS/SSA/VTT converted to WebVTT |

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

No external subtitle provider is contacted. `DownloadSnapshot.subtitles` lists
the torrent's convertible subtitle files, and the SPA matches them to the
playing video by episode tag or base name, then attaches them as native
`<track kind="subtitles">` elements. The player has an Off / track selector.

## Streaming

`GET|HEAD /api/stream/{torrentId}/{fileIndex}` supports a single byte range:

```http
Range: bytes=100-199
```

- `200` full response with `Accept-Ranges: bytes`, `Content-Length`,
  `Content-Type` and `Content-Disposition: inline`.
- `206` for a range with `Content-Range` and the requested `Content-Length`.
- `416` + `Content-Range: bytes */<len>` for an invalid range.
- `503` if the streaming engine is unavailable (tests).
- Files are served only from active torrents; there is no path input.

## WebSocket

`GET /api/ws` upgrades to a live-state socket. The first frame is always a full
snapshot:

```json
{
  "type": "snapshot",
  "revision": 12,
  "session": { "...": "SessionSnapshot or null" },
  "downloads": ["DownloadSnapshot..."],
  "player": { "...": "PlayerSnapshot" }
}
```

Subsequent frames:

```json
{ "type": "session", "revision": 13, "session": { "...": "or null" } }
{ "type": "downloads", "revision": 14, "downloads": ["..."] }
{ "type": "player", "revision": 15, "player": { "...": "..." } }
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
| `movie`, `tv` | `<video controls>` |
| `audio` | `<audio controls>` |
| `image` | `<img>` with open-original link |
| PDF/ebook/archive/code/subtitle/app/game/other | Open file or download only |

`POST /api/session/files/watch` is the generic "open one file" endpoint; the
frontend picks the presentation from the file extension.
