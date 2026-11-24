# Development

## Prerequisites

- Rust 1.88+ (edition 2024)
- Node.js 22+ for the web UI
- mpv or VLC for external player hand-off
- A C compiler and CMake for `librqbit`

The Nix dev shell provides all of the above:

```bash
nix develop
```

## Checks

```bash
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
npx playwright test
```

For Playwright on NixOS, point it at the Nix browser bundle:

```bash
PLAYWRIGHT_BROWSERS_PATH=$(nix eval --raw nixpkgs#playwright-driver.browsers) \
  nix develop -c npx playwright test
```

All checks also run in GitHub Actions.

## Web development

Run the daemon and Vite separately:

```bash
# terminal 1
cargo run -p twatchd -- --port 7878 --web-dir web/dist

# terminal 2
cd web
npm run dev
```

Vite serves <http://localhost:5173> and proxies `/api` and `/api/ws` to the
daemon. `twatchd --help` lists the bind, port, token, and web-dir options.

### Embedded build

```bash
cd web && npm run build && cd ..
cargo build --release -p twatchd --features embed-web
./target/release/twatchd --open
```

The daemon sets an HttpOnly auth cookie on the first page load. `server-token`
lives in `$XDG_CONFIG_HOME/twatch/server-token`. `twatchd` is single-instance per
config directory; a second launch opens the running console.

## NixOS

Run from the flake:

```bash
nix run github:0Risotto/twatch
nix run github:0Risotto/twatch#twatchd -- --open
```

Declarative flake input:

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

Home Manager:

```nix
home.packages = [
  inputs.twatch.packages.${pkgs.stdenv.hostPlatform.system}.twatch-electron
];
```

Available packages: `twatch-electron` (default) and `twatchd`.

## Desktop packaging

See [`desktop.md`](desktop.md).

## Coverage

Rust coverage uses `cargo-llvm-cov`; CI enforces the current non-regression
threshold. Raise the threshold with each batch of tests.

```bash
cargo llvm-cov --workspace --all-features --summary-only
```

## Versions

All manifests must use the same version:

```bash
python3 scripts/check-versions.py
```

When bumping, update the four Cargo crates, `electron/package.json`,
`web/package.json`, and the two Nix files. Tag releases as `vX.Y.Z`; the Desktop workflow builds the artifacts and
publishes the GitHub release.
