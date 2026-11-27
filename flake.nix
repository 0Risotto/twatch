{
  description = "twatch — desktop and web torrent streaming client";

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
          twatchd = pkgs.callPackage ./nix/package.nix { };
          twatchElectron = pkgs.callPackage ./nix/electron.nix { inherit twatchd; };
        in
        {
          default = twatchElectron;
          inherit twatchd;
          twatch-electron = twatchElectron;
        }
      );

      apps = forAllSystems (
        pkgs:
        let
          packages = self.packages.${pkgs.stdenv.hostPlatform.system};
          twatchd = packages.twatchd;
          twatchElectron = packages.twatch-electron;
        in
        {
          default = {
            type = "app";
            program = "${twatchElectron}/bin/twatch-electron";
            meta = twatchElectron.meta;
          };
          twatchd = {
            type = "app";
            program = "${twatchd}/bin/twatchd";
            meta = twatchd.meta;
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
            ffmpeg
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
