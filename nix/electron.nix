{
  lib,
  stdenv,
  makeWrapper,
  makeDesktopItem,
  electron,
  twatch,
}:
let
  version = "0.1.0";
  desktopItem = makeDesktopItem {
    name = "twatch";
    desktopName = "twatch";
    comment = "Terminal and web torrent streaming client";
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
  src = ../electron;

  nativeBuildInputs = [ makeWrapper ];

  dontBuild = true;

  installPhase = ''
    runHook preInstall
    install -d $out/bin $out/share/twatch-electron
    cp -r . $out/share/twatch-electron/
    makeWrapper ${electron}/bin/electron $out/bin/twatch-electron \
      --add-flags "--no-sandbox" \
      --add-flags "$out/share/twatch-electron/main.js" \
      --set TWATCHD_BIN ${twatch}/bin/twatchd
    install -Dm644 ${../web/public/favicon.svg} $out/share/icons/hicolor/scalable/apps/twatch.svg
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
