# twatch

Desktop and web torrent streaming client. Browse, preview, stream, and download
torrents from the Electron app or a local browser console. Completed downloads
and movies already in the twatch cache are indexed in a local library. Browser
formats stream directly; MKV/HEVC files are prepared locally with ffmpeg and
played in the same web player.

The backend is Rust: `librqbit` for the engine, `tokio` for async work, and
`axum` for the local web API. No Node server runs at runtime.

## Install

### NixOS / Nix

```bash
# Run the Electron app
nix run github:0Risotto/twatch

# Or install it into your profile
nix profile install github:0Risotto/twatch#twatch-electron
```

Declarative NixOS and Home Manager examples are in
[`docs/development.md`](docs/development.md#nixos).

### Releases

Download the latest Windows installer or Linux package from
[GitHub Releases](https://github.com/0Risotto/twatch/releases/latest).

- Windows: `twatch-*-setup.exe` (recommended) or `twatch-*-portable.exe`
- Linux: AppImage or deb

The builds are currently unsigned, so Windows SmartScreen may show
“Windows protected your PC”. Use **More info → Run anyway** if you trust the
download.

## Headless web console

```bash
nix run github:0Risotto/twatch#twatchd -- --open
```

The daemon listens on `127.0.0.1:7878` by default, opens the browser, and stores
its auth token in `$XDG_CONFIG_HOME/twatch/server-token`. It is loopback-only
unless `--allow-remote` is set.

## Development

```bash
nix develop
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo test --workspace

cd web
npm ci
npm run format:check
npm run lint
npm run typecheck
npm test
npm run build
```

See [`docs/development.md`](docs/development.md) for prerequisites, desktop
packaging, releases, and NixOS configuration.

## Docs

- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [Desktop packaging](docs/desktop.md)
- [Conventions](docs/conventions.md)
- [Web API](docs/web-api.md)

## License

Apache License 2.0. See [`LICENSE`](LICENSE).
