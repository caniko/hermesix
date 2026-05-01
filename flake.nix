{
  description = "Home Manager managed configuration utilities";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          lib = pkgs.lib;
          hermesix = pkgs.callPackage ./package.nix { };

          website = pkgs.stdenv.mkDerivation {
            pname = "hermesix-website";
            version = "0.1.0";
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.maybeMissing ./website;
            };
            nativeBuildInputs = [ pkgs.zola ];
            phases = [
              "buildPhase"
              "installPhase"
            ];
            buildPhase = ''
              cp -r --no-preserve=mode $src/website site
              cd site
              zola build
            '';
            installPhase = ''
              cp -r public $out
            '';
          };

          docs = pkgs.stdenv.mkDerivation {
            pname = "hermesix-docs";
            version = "0.1.0";
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.maybeMissing ./docs;
            };
            nativeBuildInputs = [ pkgs.mdbook ];
            buildPhase = ''
              mdbook build docs
            '';
            installPhase = ''
              cp -r docs/book $out
            '';
          };

          site = pkgs.runCommand "hermesix-site" { } ''
            mkdir -p $out
            cp -r ${website}/* $out/
            mkdir -p $out/docs
            cp -r ${docs}/* $out/docs/
          '';
        in
        {
          inherit
            docs
            hermesix
            site
            website
            ;
          default = hermesix;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${nixpkgs.legacyPackages.${system}.callPackage ./package.nix { }}/bin/hermesix";
        };
        hermesix = {
          type = "app";
          program = "${nixpkgs.legacyPackages.${system}.callPackage ./package.nix { }}/bin/hermesix";
        };
      });

      checks = forAllSystems (system: {
        hermesix = nixpkgs.legacyPackages.${system}.callPackage ./package.nix { };
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.clippy
              pkgs.gcc
              pkgs.mdbook
              pkgs.rustc
              pkgs.rustfmt
              pkgs.zola
            ];
            shellHook = ''
              echo "Website: cd website && zola serve"
              echo "Documentation: cd docs && mdbook serve"
              echo "Release dry-run: cargo publish --dry-run"
            '';
          };
        }
      );
    };
}
