# Architecture

twatch is a Rust daemon plus a React SPA. The Electron shell runs `twatchd` as
a sidecar and hosts the web UI.

## Crates

```text
crates/
├── twatch-core/      Pure domain, config, themes, headless Gateway
├── twatch-engine/    librqbit, JSON history, search, player, covers, streams
├── twatch-server/    axum REST/WebSocket API, auth, static SPA, Range streaming
└── twatchd/          Headless daemon binary
electron/             Electron shell and packaging
web/                  React 19 + TypeScript + Vite SPA
```

## Request path

```text
Browser / Electron
        │ HTTP + WebSocket
        ▼
twatch-server
        │ snapshots, commands, events
        ▼
twatch-core::gateway::Gateway
        │ ports
        ▼
twatch-engine ── librqbit, filesystem, mpv/vlc, search, covers
```

The gateway owns the active browse session, transfers, history, player state,
and event stream. It is independent of HTTP and desktop UI code.

`TorrentPort`, `StreamPort`, `HistoryPort`, `PlayerPort`, `SearchPort`, and
`TaskSpawner` are the adapter boundaries. Tests replace them with in-memory
implementations.

## Behaviour worth knowing

- The SPA receives one bootstrap snapshot, then coalesced updates over
  `/api/ws` at roughly 4 Hz. The UI interpolates between updates.
- Transfers persist to `downloads.json`. After a restart they return paused and
  resume only when the user asks.
- Streams support `HEAD`, `Range`, `206`, `416`, MIME detection, and byte-range
  seeking.
- Playback is owned by a global dock. Leaving `/player` keeps audio/video alive
  in the miniplayer.
- In-torrent subtitles are converted to WebVTT; ASS/SSA is rendered by libass
  through JASSUB.
- Posters are resolved server-side from keyless providers and cached locally.
- A title pasted into Home becomes a search; a magnet or `.torrent` URL opens
  Preview.

## Data

| Data | Path |
| --- | --- |
| History | `$XDG_CONFIG_HOME/twatch/history.json` |
| Session cache | `$XDG_CACHE_HOME/twatch/session/` |
| Downloads | `$XDG_DOWNLOAD_DIR/twatch/` |
| Web token | `$XDG_CONFIG_HOME/twatch/server-token` |
| Web theme | `web_theme` in `$XDG_CONFIG_HOME/twatch/config.json` |
| Covers | `$XDG_CONFIG_HOME/twatch/covers.json` |
| Daemon logs | stdout, or `twatchd.log` in the Electron logs directory |

## API

See [`web-api.md`](web-api.md) for REST endpoints, WebSocket frames, auth rules,
and streaming semantics.
