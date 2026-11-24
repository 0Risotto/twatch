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
cargo clippy --all-targets --all-features --workspace --exclude twatch-app -- -D warnings
cargo test --workspace --exclude twatch-app

cd web
npm ci
npm run format:check
npm run lint
npm run typecheck
npm test
npm run build
```

All checks also run in GitHub Actions. `twatch-app` is checked separately
because Tauri needs GTK/WebKit development libraries.

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

Available packages: `twatch-electron` (default), `twatchd`, `twatch-app`.

## Desktop packaging

See [`desktop.md`](desktop.md).

## Versions

All manifests must use the same version:

```bash
python3 scripts/check-versions.py
```

When bumping, update the five Cargo crates, `electron/package.json`,
`web/package.json`, `crates/twatch-app/tauri.conf.json`, and the three Nix
files. Tag releases as `vX.Y.Z`; the Desktop workflow builds the artifacts and
publishes the GitHub release.
