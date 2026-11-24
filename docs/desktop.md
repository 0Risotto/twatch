# Desktop packaging

The Electron app bundles the release `twatchd` sidecar with the SPA embedded, so
an installed app needs neither Node nor a separate daemon.

## Linux

```bash
# NixOS / local development
nix build .#twatch-electron
./result/bin/twatch-electron

# AppImage + deb on a generic Linux host or CI
scripts/build-desktop.sh linux
```

On Wayland, GNOME-family desktops and Niri get the frameless in-app header.
KDE, XFCE, tiling window managers, and X11 keep the native window frame.

## Windows

```powershell
scripts\build-desktop.sh win
```

This produces a portable executable and an NSIS installer under `electron/dist`.
The GitHub **Desktop** workflow builds both on `windows-latest`.

For Bottles/Wine testing, add `--no-sandbox` if Chromium refuses to start, and
`--disable-gpu` if the window is blank. Daemon output is written to
`twatchd.log` in the Electron logs directory.

## CI and releases

The **Desktop** workflow runs on every push and uploads Linux and Windows
artifacts. Pushing a `v*` tag builds the same artifacts and publishes them to a
GitHub release with `SHA256SUMS.txt`.

The builds are unsigned. Windows SmartScreen reports an unknown publisher until
code signing is configured. See [`development.md`](development.md#versions) for
versioning.
