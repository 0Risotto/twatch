{
  lib,
  rustPlatform,
  buildNpmPackage,
  cmake,
  perl,
  pkg-config,
  makeWrapper,
  makeDesktopItem,
  mpv,
  xdg-utils,
  go,
  wrapGAppsHook3,
  webkitgtk_4_1,
  libsoup_3,
  gtk3,
  glib,
  cairo,
  pango,
  atk,
  gdk-pixbuf,
  librsvg,
  openssl,
  dbus,
  gst_all_1,
  runtimeShell,
}:
let
  version = "0.1.0";

  sourceFilter =
    path: type:
    let
      base = baseNameOf (toString path);
    in
    !(builtins.elem base [
      ".git"
      ".playwright-mcp"
      ".vite"
      "coverage"
      "dist"
      "node_modules"
      "target"
      "web-dist"
    ]);

  web = buildNpmPackage {
    pname = "twatch-web";
    inherit version;
    src = lib.cleanSourceWith {
      src = ../web;
      filter = sourceFilter;
    };
    npmDepsHash = "sha256-cFfGyCWrGmbahPoBz5W4/Hv3fuueCs8wSQZz/NFkzgc=";
    installPhase = ''
      runHook preInstall
      mkdir -p $out
      cp -r dist/. $out/
      runHook postInstall
    '';
  };

  desktopItem = makeDesktopItem {
    name = "twatch";
    desktopName = "twatch";
    comment = "Terminal and web torrent streaming client";
    exec = "twatch-app";
    icon = "twatch";
    terminal = false;
    categories = [
      "Network"
      "FileTransfer"
    ];
    startupNotify = true;
  };
in
rustPlatform.buildRustPackage {
  pname = "twatch-app";
  inherit version;

  src = lib.cleanSourceWith {
    src = ../.;
    filter = sourceFilter;
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    cmake
    perl
    pkg-config
    makeWrapper
    wrapGAppsHook3
    go
  ];

  buildInputs = [
    webkitgtk_4_1
    libsoup_3
    gtk3
    glib
    cairo
    pango
    atk
    gdk-pixbuf
    librsvg
    openssl
    dbus
    gst_all_1.gstreamer.out
    gst_all_1.gst-plugins-base.out
    gst_all_1.gst-plugins-good.out
    gst_all_1.gst-plugins-bad.out
    gst_all_1.gst-plugins-ugly.out
    gst_all_1.gst-libav.out
  ];

  preBuild = ''
    mkdir -p web/dist
    cp -r ${web}/. web/dist/
  '';

  cargoBuildFlags = [
    "-p"
    "twatch-app"
  ];

  doCheck = false;

  postInstall = ''
        mv $out/bin/twatch-app $out/bin/twatch-app-real
        cat > $out/bin/twatch-app <<EOF
    #!${runtimeShell}
    export PATH="${
      lib.makeBinPath [
        mpv
        xdg-utils
      ]
    }:\$PATH"
    export GST_PLUGIN_SYSTEM_PATH_1_0='${
      lib.makeSearchPath "lib/gstreamer-1.0" [
        gst_all_1.gstreamer.out
        gst_all_1.gst-plugins-base.out
        gst_all_1.gst-plugins-good.out
        gst_all_1.gst-plugins-bad.out
        gst_all_1.gst-plugins-ugly.out
        gst_all_1.gst-libav.out
      ]
    }'
    export GST_PLUGIN_SCANNER='${gst_all_1.gstreamer.out}/libexec/gstreamer-1.0/gst-plugin-scanner'
    exec "$out/bin/twatch-app-real" "\$@"
    EOF
        chmod +x $out/bin/twatch-app
        install -Dm644 ${../web/public/favicon.svg} $out/share/icons/hicolor/scalable/apps/twatch.svg
        install -Dm644 ${desktopItem}/share/applications/twatch.desktop $out/share/applications/twatch.desktop
  '';

  meta = {
    description = "Tauri desktop application for twatch";
    homepage = "https://github.com/0Risotto/twatch";
    license = lib.licenses.asl20;
    mainProgram = "twatch-app";
    platforms = lib.platforms.linux;
  };
}
