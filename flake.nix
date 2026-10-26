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
        in
        {
          default = twatch;
          inherit twatch;
        }
      );

      apps = forAllSystems (
        pkgs:
        let
          twatch = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        in
        {
          default = {
            type = "app";
            program = "${twatch}/bin/twatchd";
            meta = twatch.meta;
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
            cmake
            perl
            pkg-config
            mpv
            xdg-utils
          ];
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
