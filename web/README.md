# twatch web UI

React 19 + TypeScript + Vite + Tailwind 4 + Motion. The SPA talks to the Rust
`twatchd` gateway over REST and a single WebSocket. It is built as static files
and served by Rust in release builds.

## Develop

Start the daemon first:

```bash
# from the repository root
nix-shell -p cargo rustc pkg-config gcc --run \
  'cargo run -p twatchd -- --port 7878 --web-dir web/dist'
```

Then start Vite:

```bash
cd web
npm install
npm run dev
# http://localhost:5173 — /api and /api/ws proxy to 127.0.0.1:7878
```

The daemon serves the built SPA from `web/dist` when you use `--web-dir`.
Vite hot reload is used for UI work; the Rust process only needs restarting
when backend code changes.

## Build

```bash
npm run build        # tsc --noEmit && vite build -> web/dist
npm run typecheck
npm test
```

Release builds embed `web/dist` into `twatchd`:

```bash
npm run build
cd ..
cargo build --release -p twatchd --features embed-web
```

## Themes

The built-in catalogue lives in `src/styles/themes.css`: 27 base palettes, each
with a dark and light variant. Tokyo Night, Kanagawa, Nord, Gruvbox, Catppuccin,
Purple, Emerald and the rest are available from Settings → Appearance. The
default is Emerald dark (`emerald-dark`).

`src/lib/themes.ts` owns the base list and the id splitting. The server also
exposes the catalogue at `GET /api/themes` and stores the selected variant in
`web_theme` in `config.json`; the terminal client's own `theme` field is
separate.

## Cover art

Movie, TV, anime and book posters come from `POST /api/covers` (IMDb
suggestions, AniList, TVmaze, Open Library with a local 30-day cache). Posters
load lazily as rows approach the viewport via `CoverArt` and are purely
decorative: a provider miss falls back to the file-type icon.

## Subtitles

Only subtitles already inside the torrent are used — there is no external
subtitle download. `DownloadSnapshot.subtitles` carries the SRT/ASS/SSA/VTT
files; the player matches them to the playing episode and attaches native
`<track>` elements. The gateway converts SRT and ASS/SSA to WebVTT on demand at
`/api/subtitles/{torrentId}/{fileIndex}`.

## File previews

`src/lib/fileIcons.tsx` maps extensions and torrent titles to Lucide icons.
Preview is intentionally limited to what a browser can render:

- video (`movie`/`tv`) → `<video>`
- audio → `<audio>`
- images → `<img>`
- PDFs, ebooks, archives, code, subtitles, apps, games and unknown files →
  open/download only

The stream endpoint still serves every file with byte-range support; the UI
only exposes the Watch/Play/Preview actions for previewable types.

## API

See [`../docs/web-api.md`](../docs/web-api.md) for the endpoint, WebSocket
frame, auth and range-streaming contracts.
