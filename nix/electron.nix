{
  lib,
  stdenv,
  makeWrapper,
  makeDesktopItem,
  electron,
  twatchd,
}:
let
  version = "0.2.6";
  desktopItem = makeDesktopItem {
    name = "twatch";
    desktopName = "twatch";
    comment = "Desktop torrent streaming client";
    exec = "twatch-electron";
    icon = "twatch";
    terminal = false;
    categories = [
      "Network"
      "FileTransfer"
    ];
    startupNotify = true;
  };
in
stdenv.mkDerivation {
  pname = "twatch-electron";
  inherit version;
  src = lib.cleanSourceWith {
    src = ../electron;
    filter =
      path: type:
      let
        base = baseNameOf (toString path);
      in
      !(builtins.elem base [
        "node_modules"
        "dist"
        "resources"
      ]);
  };

  nativeBuildInputs = [ makeWrapper ];

  dontBuild = true;

  installPhase = ''
    runHook preInstall
    install -d $out/bin $out/share/twatch-electron
    cp -r . $out/share/twatch-electron/
    makeWrapper ${electron}/bin/electron $out/bin/twatch-electron \
      --add-flags "--no-sandbox" \
      --add-flags "$out/share/twatch-electron/main.js" \
      --set TWATCHD_BIN ${twatchd}/bin/twatchd
    install -Dm644 ${../electron/build/icons/512x512.png} $out/share/icons/hicolor/512x512/apps/twatch.png
    install -Dm644 ${desktopItem}/share/applications/twatch.desktop $out/share/applications/twatch.desktop
    runHook postInstall
  '';

  meta = {
    description = "Electron desktop application for twatch";
    homepage = "https://github.com/0Risotto/twatch";
    license = lib.licenses.asl20;
    mainProgram = "twatch-electron";
    platforms = lib.platforms.linux;
  };
}
