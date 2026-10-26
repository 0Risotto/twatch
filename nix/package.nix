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
    exec = "twatchd --open";
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
  pname = "twatch";
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
    go
  ];

  preBuild = ''
    mkdir -p web/dist
    cp -r ${web}/. web/dist/
  '';

  cargoBuildFlags = [
    "-p"
    "twatch"
    "-p"
    "twatchd"
    "--features"
    "twatchd/embed-web"
  ];

  doCheck = false;

  postInstall = ''
    wrapProgram $out/bin/twatch --prefix PATH : ${
      lib.makeBinPath [
        mpv
        xdg-utils
      ]
    }
    wrapProgram $out/bin/twatchd --prefix PATH : ${
      lib.makeBinPath [
        mpv
        xdg-utils
      ]
    }
    install -Dm644 ${../web/public/favicon.svg} $out/share/icons/hicolor/scalable/apps/twatch.svg
    install -Dm644 ${desktopItem}/share/applications/twatch.desktop $out/share/applications/twatch.desktop
  '';

  meta = {
    description = "Terminal and web torrent streaming client";
    homepage = "https://github.com/0Risotto/twatch";
    license = lib.licenses.asl20;
    mainProgram = "twatchd";
    platforms = lib.platforms.linux;
  };
}
