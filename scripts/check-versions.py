#!/usr/bin/env python3
"""Fail when the release version drifts between manifests."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VERSION_RE = re.compile(r'^version\s*=\s*"([^"]+)"', re.MULTILINE)


def cargo_version(path: Path) -> str:
    match = VERSION_RE.search(path.read_text())
    if not match:
        raise SystemExit(f"no version in {path}")
    return match.group(1)


def json_version(path: Path) -> str:
    return json.loads(path.read_text())["version"]


def nix_version(path: Path) -> str:
    match = re.search(r'^\s*version\s*=\s*"([^"]+)"', path.read_text(), re.MULTILINE)
    if not match:
        raise SystemExit(f"no version in {path}")
    return match.group(1)


def main() -> int:
    versions = {
        **{
            str(path.relative_to(ROOT)): cargo_version(path)
            for path in sorted((ROOT / "crates").glob("*/Cargo.toml"))
        },
        "electron/package.json": json_version(ROOT / "electron/package.json"),
        "web/package.json": json_version(ROOT / "web/package.json"),
        "crates/twatch-app/tauri.conf.json": json_version(
            ROOT / "crates/twatch-app/tauri.conf.json"
        ),
        **{
            f"nix/{name}.nix": nix_version(ROOT / "nix" / f"{name}.nix")
            for name in ("electron", "package", "tauri")
        },
    }
    unique = sorted(set(versions.values()))
    width = max(len(name) for name in versions)
    for name, version in versions.items():
        print(f"{name:<{width}}  {version}")
    if len(unique) != 1:
        print(f"\nversion mismatch: {', '.join(unique)}", file=sys.stderr)
        return 1
    print(f"\nall manifests use {unique[0]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
