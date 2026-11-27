# Third-party notices

twatch itself is licensed under the Apache License 2.0. The packaged desktop
application includes the following third-party components.

| Component | License |
| --- | --- |
| Electron | MIT |
| Chromium | BSD-style and other licenses, see Electron's `LICENSES.chromium.html` |
| librqbit | MIT / Apache-2.0 |
| FFmpeg / ffprobe (bundled static binaries) | GPL-3.0-or-later |
| JASSUB (libass + Emscripten runtime) | LGPL-2.1-or-later, with libass/fontconfig/FreeType components under their respective licenses |
| React, React DOM | MIT |
| Lucide icons | ISC |
| Motion | MIT |
| Tailwind CSS | MIT |
| Radix UI | MIT |
| Space Grotesk | SIL Open Font License 1.1 |
| JetBrains Mono | SIL Open Font License 1.1 |
| Noto Sans / Liberation Sans (JASSUB fallback) | SIL Open Font License 1.1 / Liberation Font License |

The full license texts for npm dependencies are available in the corresponding
`node_modules/*/LICENSE*` files in the source tree. Electron's complete
third-party license file is bundled by Electron itself at
`resources/LICENSES.chromium.html` in a packaged build.

The bundled FFmpeg and ffprobe executables are distributed under
GPL-3.0-or-later. Their source is available from <https://ffmpeg.org/> and the
corresponding source package for the bundled build is available from
<https://github.com/eugeneware/ffmpeg-static>.

JASSUB is dynamically loaded as a WebAssembly/JavaScript component. Its source
is available at <https://github.com/ThaUnknown/jassub> under the LGPL-2.1-or-later
and the additional licenses listed in that project.

For the exact dependency versions, see `web/package-lock.json`,
`electron/package-lock.json`, and `Cargo.lock`.
