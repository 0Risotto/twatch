# twatch

Terminal and web torrent streaming client. Browse, preview, stream, and
download torrents from your terminal or a local browser console.

The backend is Rust: `librqbit` for the engine, `tokio` for async work, and an
`axum` gateway for the web client. There is no Node server at runtime.

## Prerequisites

- **Rust** 1.88+ (edition 2024)
- **Node.js** 22+ only when building the web UI
- **mpv** or **vlc** for external player hand-off
- A C compiler (gcc/clang) for `librqbit` native dependencies

## Quick Start

### NixOS / Nix

The default package is the Electron desktop app. It uses Chromium for the
browser player and bundles the Rust gateway as a local sidecar:

```bash
# Run the desktop app from a checkout
nix run

# Or build it and run the result
nix build
./result/bin/twatch-electron
```

The flake also exposes the terminal client, the headless web daemon, and the
older Tauri shell:

```bash
nix run .#twatch
nix run .#twatchd -- --open
nix run .#twatch-app
```

On Wayland, the desktop app follows the desktop convention: GNOME-family
desktops and Niri get the frameless in-app header, while KDE, XFCE, tiling
window managers, and X11 keep the native window frame.

To add it declaratively to a flake-based NixOS configuration, add the flake as
an input:

```nix
{
  inputs.twatch.url = "github:0Risotto/twatch";

  outputs = { self, nixpkgs, twatch, ... }: {
    nixosConfigurations.host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ({ pkgs, ... }: {
          environment.systemPackages = [
            twatch.packages.${pkgs.stdenv.hostPlatform.system}.default
          ];
        })
      ];
    };
  };
}
```

### Terminal client

```bash
git clone https://github.com/0Risotto/twatch.git
cd twatch
cargo build --release
./target/release/twatch
```

### Web console

```bash
cd web
npm install
npm run build
cd ..

# Embeds web/dist into the binary:
cargo build --release -p twatchd --features embed-web
./target/release/twatchd --open
```

Open <http://127.0.0.1:7878> if the browser does not launch. The daemon sets an
HttpOnly auth cookie on the first page load; `server-token` lives under
`$XDG_CONFIG_HOME/twatch/`.

`twatchd` is single-instance per config directory. Starting it again opens the
running console instead of creating a second daemon.

For UI development, run the daemon and Vite separately:

```bash
# terminal 1
cargo run -p twatchd -- --port 7878 --web-dir web/dist

# terminal 2
cd web
npm run dev
# http://localhost:5173 — /api and /api/ws proxy to twatchd
```

`twatchd --help` lists bind/port/token/web-dir options. It is loopback-only by
default; `--allow-remote` is required to expose it beyond localhost.

## Development

### Rust

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo test --workspace
```

### Web

```bash
cd web
npm run typecheck
npm test
npm run build
```

### Run all checks at once

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo test --workspace
(cd web && npm run typecheck && npm test && npm run build)
```

## Before Submitting a PR

1. Run `cargo fmt --all` and ensure no diffs.
2. Run clippy with zero warnings.
3. Run all workspace tests.
4. Run the web typecheck, tests, and production build.
5. Write a [conventional commit](https://www.conventionalcommits.org/) message:
   `feat:`, `fix:`, `refactor:`, `chore:`, `docs:`.
6. Keep PRs small. One concern per PR.

## Architecture

```
src/                     TUI client (ratatui) + facade re-exports
crates/
├── twatch-core/         Pure domain, config, theme catalogue, headless Gateway
│   ├── domain/            entities, value objects, ports (no I/O/UI)
│   ├── gateway.rs         session, downloads, history, search, player, events
│   └── theme.rs           Built-in theme catalogue names/modes
├── twatch-engine/       Adapters
│   ├── service/           librqbit, JSON history, reqwest search, mpv/vlc
│   └── stream.rs          range-capable FileStream facade
├── twatch-server/       axum REST + WebSocket + static SPA + range streaming
├── twatch-app/          Tauri desktop shell with an in-process gateway
└── twatchd/             Web daemon binary
web/                     React 19 + TypeScript + Vite + Tailwind + Motion SPA
docs/web-api.md          Gateway endpoint, WebSocket, auth, and range contracts
```

The TUI still uses the layered domain/application/screen stack documented
below. The web daemon uses `twatch-core::gateway::Gateway`, which owns the
current browse session, the download registry, and the live event stream.

```
Browser (React + Motion) ── HTTP + WebSocket ──┐
TUI (ratatui) ── in-process ──────────────────┤
                                              ▼
                                   twatchd (Rust)
                                   ├─ SessionService
                                   ├─ DownloadService
                                   ├─ HistoryService
                                   ├─ SearchService
                                   ├─ PlayerService
                                   └─ librqbit + range stream
```

**TUI design points**

- **Navigation stack**: every screen (including popups) is a `Screen` pushed
  on a stack. Esc/q pops back to where you came from.
- **Open/Closed**: adding a screen = one new module (state machine + view model)
  plus one row in `screens/registry.rs`. Each screen implements
  `Screen::render`, which delegates to a pure renderer in `ui/`.
- **Separation of concerns**: UI files render pure view models produced by the
  screens; services live behind ports.
- **Swappable tools**: `TorrentPort`, `StreamPort`, `HistoryPort`,
  `PlayerPort`, `SearchPort`, and `TaskSpawner` keep engine/player/storage
  choices behind one adapter boundary.

**Web design points**

- The gateway is headless: `twatch-core` has no `ratatui`, `axum`, or screen
  dependency.
- Progress is coalesced at 4 Hz and the UI interpolates between ticks, instead
  of pushing 60 updates per second.
- Streams support `HEAD`, `Range`, `206`, `416`, MIME types, and byte-range
  seeking for video/audio/image preview.
- Preview actions are type-aware: video plays in `<video>`, audio in `<audio>`,
  images in `<img>`; PDFs, archives, code and other files are open/download only.
- In-torrent subtitles (SRT/ASS/SSA/VTT) are matched to the playing episode and
  attached as native `<track>` elements after on-the-fly WebVTT conversion. No
  external subtitle provider is contacted.
- Movie/TV/anime/book posters are resolved server-side from keyless providers
  (IMDb, AniList, TVmaze, Open Library) and cached locally; misses fall back to
  the matching file-type icon.
- Pasting a title into the home input routes to Search; only a magnet or
  `.torrent` URL goes through Preview.
- The built-in catalogue lives in `web/src/styles/themes.css`: 27 base themes,
  each with dark and light variants, including Tokyo Night, Kanagawa, Nord,
  Gruvbox, Catppuccin, Purple, Emerald, Ocean, Forest, Sunset, Sakura and Coffee.
  Default: Emerald dark. The TUI keeps its own palette in `src/ui/theme.rs`.
- Motion uses short durations and springs (120–320 ms), respects
  `prefers-reduced-motion`, and has a Settings override.

Dependency injection for the TUI uses [shaku](https://crates.io/crates/shaku).

## Data

- History:    `$XDG_CONFIG_HOME/twatch/history.json`
- Session:    `$XDG_CACHE_HOME/twatch/session/`
- Logs:       `$XDG_STATE_HOME/twatch/twatch.log`
- Downloads:  `$XDG_DOWNLOAD_DIR/twatch/`
- Web token:  `$XDG_CONFIG_HOME/twatch/server-token`
- Web theme:  `web_theme` in `$XDG_CONFIG_HOME/twatch/config.json`
- Covers:     `$XDG_CONFIG_HOME/twatch/covers.json` (30-day cache)

## API

See [`docs/web-api.md`](docs/web-api.md) for:
- REST endpoints and camelCase DTOs
- WebSocket snapshot/revision frames
- cookie/Bearer auth and Host/Origin rules
- `Range` streaming semantics

## License

Apache License 2.0. See [`LICENSE`](LICENSE).
