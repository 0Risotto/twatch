# twatch

Terminal torrent streaming client. Browse, stream, and download torrents from your terminal.

## Prerequisites

- **Rust** 1.85+ (edition 2024)
- **mpv** or **vlc** for streaming playback
- A C compiler (gcc/clang) for librqbit native dependencies

## Quick Start

```bash
git clone https://github.com/0Risotto/twatch.git
cd twatch
cargo build --release
./target/release/twatch
```

## Development

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test --all-features
```

### Format

```bash
cargo fmt --all -- --check    # check only
cargo fmt --all               # auto-fix
```

### Lint

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Run all checks at once

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Before Submitting a PR

1. Run `cargo fmt --all` and ensure no diffs.
2. Run `cargo clippy --all-targets --all-features -- -D warnings`. Zero warnings.
3. Run `cargo test --all-features`. All tests pass.
4. Write a [conventional commit](https://www.conventionalcommits.org/) message: `feat:`, `fix:`, `refactor:`, `chore:`, `docs:`.
5. Keep PRs small. One concern per PR.

## Architecture

```
src/
├── domain/          Pure domain layer — entities, value objects, ports (no I/O, no UI deps)
│   ├── torrent/       TorrentId (opaque), FileTree, FileStatus, TorrentPort, StreamPort
│   ├── history/       HistoryEntry, HistoryPort
│   ├── player/        PlayerPort
│   ├── search/        SearchResult, SearchPort
│   └── common/        fuzzy_match, validation, paging, TaskSpawner
├── application/     Business rules — use cases, background flows, observers
│   ├── background/    Per-feature flows (each owns its channel; no central event enum)
│   ├── downloads.rs   DownloadTracker + completion observers
│   ├── files.rs       Delete/rename use cases
│   └── theme.rs       Theme selection use case
├── app/             Application shell + screens
│   ├── state.rs       TorrentSession, InputState, ThemeState
│   └── screens/       Self-contained screen state machines (Screen trait + stack)
│       └── registry.rs  Single closed point enumerating every screen constructor
├── service/         Infrastructure adapters (librqbit, mpv/vlc, JSON storage, HTTP search, tokio)
├── config/          Application + search API configuration (persisted; no runtime state)
├── model/           Session types + re-exports of domain types
├── module/          shaku DI container wiring
└── ui/              Presentation only — pure renderers over view models
    └── badges.rs      FileBadge registry (new file attribute = 1 badge + 1 registration row)
tests/
├── common/           Mock implementations of every port
├── handlers.rs       Screen navigation + stack tests (34)
├── application.rs    Use-case and observer tests (5)
├── player.rs         Player unit + mock tests (4)
├── storage.rs        Storage CRUD + mock tests (5)
├── theme.rs          Theme tests (15)
├── validation.rs     Input validation tests (14)
└── integration.rs    Binary smoke tests (2)
```

**Key design points**

- **Navigation stack**: every screen (including popups like search, theme picker, and
  rename) is a `Screen` pushed on a stack. Esc/q pops back to where you came from.
- **Open/Closed**: adding a screen = one new module + one row in `screens/registry.rs`;
  adding a background flow = one new module; adding a file attribute = one badge +
  one registration row; adding a completion side effect = one observer.
- **Separation of concerns**: UI files never touch services or business logic — they
  render pure view models produced by the screens.
- **Swappable tools**: everything behind ports (`TorrentPort`, `StreamPort`,
  `HistoryPort`, `PlayerPort`, `SearchPort`, `TaskSpawner`). Swapping an engine or
  player = one new adapter + one line in `src/main.rs`.

Dependency injection uses [shaku](https://crates.io/crates/shaku). Compile-time DI with component overrides for testing.

## Data

- History:    `$XDG_CONFIG_HOME/twatch/history.json`
- Session:    `$XDG_CACHE_HOME/twatch/session/`
- Logs:       `$XDG_STATE_HOME/twatch/twatch.log`
- Downloads:  `$XDG_DOWNLOAD_DIR/twatch/`

## License

MIT
