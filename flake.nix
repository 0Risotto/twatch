{
  description = "twatch — terminal and web torrent streaming client";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          twatch = pkgs.callPackage ./nix/package.nix { };
          twatchApp = pkgs.callPackage ./nix/tauri.nix { };
          twatchElectron = pkgs.callPackage ./nix/electron.nix { inherit twatch; };
        in
        {
          default = twatchElectron;
          inherit twatch;
          twatch-app = twatchApp;
          twatch-electron = twatchElectron;
        }
      );

      apps = forAllSystems (
        pkgs:
        let
          packages = self.packages.${pkgs.stdenv.hostPlatform.system};
          twatch = packages.twatch;
          twatchApp = packages.twatch-app;
          twatchElectron = packages.twatch-electron;
        in
        {
          default = {
            type = "app";
            program = "${twatchElectron}/bin/twatch-electron";
            meta = twatchElectron.meta;
          };
          twatch = {
            type = "app";
            program = "${twatch}/bin/twatch";
            meta = twatch.meta;
          };
          twatchd = {
            type = "app";
            program = "${twatch}/bin/twatchd";
            meta = twatch.meta;
          };
          twatch-app = {
            type = "app";
            program = "${twatchApp}/bin/twatch-app";
            meta = twatchApp.meta;
          };
          twatch-electron = {
            type = "app";
            program = "${twatchElectron}/bin/twatch-electron";
            meta = twatchElectron.meta;
          };
        }
      );

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            nodejs_22
            electron
            cmake
            perl
            pkg-config
            mpv
            xdg-utils
            go
            wrapGAppsHook3
            webkitgtk_4_1
            libsoup_3
            gtk3
            librsvg
            openssl
            dbus
          ];
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
