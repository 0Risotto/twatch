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
├── traits/       Service interfaces (TorrentService, PlayerService, StorageService)
├── service/      Real implementations (librqbit, mpv/vlc, JSON storage)
├── module/       shaku DI container wiring (module! macro)
├── config/       Application configuration
├── app/          App state, event loop, keyboard handlers
├── ui/           Terminal UI: screens, sidebar, theme helpers
├── model/        Domain types and input validation
tests/
├── common/       Mock implementations shared across test files
├── handlers.rs   Screen navigation tests (12)
├── player.rs     Player unit + mock tests (4)
├── storage.rs    Storage CRUD + mock tests (5)
├── validation.rs Input validation tests (6)
└── integration.rs Binary smoke tests (2)
```

Dependency injection uses [shaku](https://crates.io/crates/shaku). Compile-time DI with component overrides for testing.

## Data

- History:    `$XDG_CONFIG_HOME/twatch/history.json`
- Session:    `$XDG_CACHE_HOME/twatch/session/`
- Logs:       `$XDG_STATE_HOME/twatch/twatch.log`
- Downloads:  `$XDG_DOWNLOAD_DIR/twatch/`

## License

MIT
