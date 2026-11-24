# Conventions

## Rust

- Keep domain logic in `twatch-core`, adapters in `twatch-engine`, transport in
  `twatch-server`, and process wiring in `twatchd`/`twatch-app`.
- Production modules should stay focused. Split by feature before a file grows
  past roughly 400 lines.
- Public gateway operations return user-facing errors. Internal adapters may use
  `anyhow`; map errors to HTTP in `twatch-server` only.
- Use typed IDs (`DownloadId`, `FileIndex`, `TorrentId`) at boundaries instead of
  raw `u64`/`usize`.
- Do not block an async worker with filesystem or process work. Use
  `spawn_blocking` or an adapter that owns blocking work.
- Keep locks scoped. Never hold a `std::sync` guard across `.await`.
- Tests live with their crate. Shared test ports and fixtures go in a support
  module, not production code.
- `unsafe_code` is forbidden. Production code should not use `unwrap`/`expect`;
  tests may.

## TypeScript

- Keep API types generated from Rust; do not hand-maintain duplicate DTOs.
- Use one query-key catalogue and one mutation/error/toast pattern.
- Components own rendering. Data loading and playback logic belong in hooks or
  feature modules.
- Prefer explicit narrowing over `as`; avoid `any`.
- Every page must handle loading, empty, error, and success states.
- UI behavior is covered by Vitest for logic and Playwright for flows.

## Nix and CI

- Use the shared web build and source filter helpers.
- Keep one release version across every manifest; CI checks it.
- Every commit must pass format, lint, tests, web checks, and Nix evaluation.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):
`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`. Add `!` for breaking
changes. Keep refactors behaviour-preserving unless the commit is explicitly a
fix.
