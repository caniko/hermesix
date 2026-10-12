{
  description = "Generic managed configuration utilities";

  inputs = {
    harbor.url = "git+https://github.com/caniko/harbor.git?ref=feat/harbor-monorepo-components&rev=e8b1996580e4ecd51dd402edf4a16d9df88e3a1b";

    nixpkgs.follows = "harbor/nixpkgs";
    rust-overlay.follows = "harbor/rust-overlay";
    crane.follows = "harbor/crane";
    treefmt-nix.follows = "harbor/treefmt-nix";
    flake-utils.url = "github:numtide/flake-utils";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    plinth = {
      url = "git+https://github.com/caniko/plinth.git?ref=migrate/harbor-combined-20261012-features&rev=534dfa6dd6217db97b7ccd313501120a3c34d912";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    advisory-db,
    nixpkgs,
    plinth,
    harbor,
    rust-overlay,
    treefmt-nix,
    ...
  }: let
    systems = [
      "aarch64-darwin"
      "aarch64-linux"
      "x86_64-darwin"
      "x86_64-linux"
    ];
    inherit (nixpkgs) lib;
    forAllSystems = lib.genAttrs systems;

    perSystem = forAllSystems (
      system: let
        pkgs =
          import (
            if system == "x86_64-darwin"
            then harbor.inputs.nixpkgs-darwin
            else nixpkgs
          ) {
            inherit system;
            overlays = [(import rust-overlay)];
          };
        toolchain = harbor.lib.rust.mkToolchain {
          inherit pkgs;
          toolchainProfile = "nightly";
        };
        cross = harbor.lib.rust.mkCross {inherit pkgs system;};
        inherit (toolchain) craneLib;
        version = "0.1.0";
        plinthProject = plinth.packages.${system}.plinth-project;

        cargoSrc = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./deny.toml
            ./LICENSE
            ./README.md
            ./plugin-schemas
            ./src
          ];
        };

        commonArgs = {
          pname = "hermesix";
          inherit version;
          src = cargoSrc;
          strictDeps = true;
          cargoExtraArgs = "--all-features";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        hermesix = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;

            meta = {
              description = "Generic managed configuration utilities";
              homepage = "https://github.com/caniko/hermesix";
              mainProgram = "hermesix";
              license = lib.licenses.mit;
              platforms = pkgs.lib.platforms.unix;
            };
          }
        );

        clippyCheck = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          }
        );

        treefmtEval = treefmt-nix.lib.evalModule pkgs (import ./nix/treefmt.nix {
          rustfmtPackage = toolchain.rustToolchain;
          edition = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.edition;
        });
        fmtCheck = treefmtEval.config.build.check self;

        nextestCheck = craneLib.cargoNextest (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

        docCheck = craneLib.cargoDoc (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoDocExtraArgs = "--no-deps";
          }
        );

        auditCheck = craneLib.cargoAudit {
          inherit advisory-db;
          src = cargoSrc;
        };

        denyCheck = craneLib.cargoDeny {
          src = cargoSrc;
        };

        projectChecks = {
          default = hermesix;
          inherit hermesix;
          hermesix-deps = cargoArtifacts;
          hermesix-clippy = clippyCheck;
          hermesix-fmt = fmtCheck;
          hermesix-nextest = nextestCheck;
          hermesix-doc = docCheck;
          hermesix-audit = auditCheck;
          hermesix-deny = denyCheck;
          clippy = clippyCheck;
          fmt = fmtCheck;
          nextest = nextestCheck;
          doc = docCheck;
          audit = auditCheck;
          deny = denyCheck;
          # Fail if flake inputs ever point at the retired Codeberg/Codefloe
          # mirrors again (fleet migrated to github.com/caniko/*).
          # sourceUrl package metadata is excluded: informational only, not fetched.
          host-pinning = let
            # Split across literals so this file never matches its own pattern.
            staleHosts = "cod" + "eberg|cod" + "efloe";
          in
            pkgs.runCommand "hermesix-host-pinning" {} ''
              if ${pkgs.lib.getExe pkgs.ripgrep} -v "sourceUrl" ${./flake.nix} ${./flake.lock} \
                | ${pkgs.lib.getExe pkgs.ripgrep} -q "${staleHosts}"; then
                echo "ERROR: retired forge host in flake inputs:" >&2
                ${pkgs.lib.getExe pkgs.ripgrep} -v "sourceUrl" ${./flake.nix} ${./flake.lock} \
                  | ${pkgs.lib.getExe pkgs.ripgrep} -n "${staleHosts}" >&2 || true
                exit 1
              fi
              touch $out
            '';
        };

        website = pkgs.stdenv.mkDerivation {
          pname = "hermesix-website";
          inherit version;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.maybeMissing ./website;
          };
          nativeBuildInputs = [pkgs.zola];
          # Zola initializes the data-loader client for local templates too.
          SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
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
          inherit version;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.maybeMissing ./docs;
          };
          nativeBuildInputs = [pkgs.mdbook];
          buildPhase = ''
            mdbook build docs
          '';
          installPhase = ''
            cp -r docs/book $out
          '';
        };

        projectSite = plinth.lib.${system}.mkProjectSite {
          pname = "hermesix-website";
          domain = "hermesix.tartanoglu.com";
          configPath = ./website/plinth-project.toml;
          staticPaths = [
            {
              source = website;
              target = ".";
            }
          ];
          docsPackage = docs;
        };
      in {
        formatter = treefmtEval.config.build.wrapper;
        packages = {
          inherit
            docs
            hermesix
            website
            ;
          default = hermesix;
          site = projectSite;
        };

        apps = {
          default = {
            type = "app";
            program = lib.getExe hermesix;
            meta.description = "Run Hermesix";
          };
          hermesix = {
            type = "app";
            program = lib.getExe hermesix;
            meta.description = "Run Hermesix";
          };
          deploy-pages = plinth.lib.${system}.mkDeployPagesApp {
            domain = "hermesix.tartanoglu.com";
          };
        };

        checks = projectChecks;

        devShells = {
          default = craneLib.devShell {
            checks = projectChecks;

            packages = [
              treefmtEval.config.build.wrapper
              harbor.packages.${system}.rust-harbor-ci
              pkgs.cargo-audit
              pkgs.cargo-deny
              pkgs.cargo-nextest
              pkgs.mdbook
              pkgs.rust-analyzer
              pkgs.zola
            ];

            shellHook = ''
              echo "Website: cd website && zola serve"
              echo "Documentation: cd docs && mdbook serve"
              echo "Release dry-run: cargo publish --dry-run"
            '';
          };

          docs = harbor.lib.rust.mkDocsShell {
            inherit pkgs cross;
            inherit (toolchain) craneLib;
            checks = projectChecks;
            packages = [
              pkgs.mdbook
              plinthProject
              pkgs.rust-analyzer
            ];
            extraShellHook = ''
              echo "Project site: plinth-project serve --config website/plinth-project.toml"
              echo "Documentation: mdbook serve docs"
            '';
          };
        };
      }
    );
  in {
    packages = lib.mapAttrs (_: value: value.packages) perSystem;
    apps = lib.mapAttrs (_: value: value.apps) perSystem;
    checks = lib.mapAttrs (_: value: value.checks) perSystem;
    devShells = lib.mapAttrs (_: value: value.devShells) perSystem;
    formatter = lib.mapAttrs (_: value: value.formatter) perSystem;
    homeManagerModules = rec {
      obs-studio = import ./modules/home-manager/obs-studio.nix {inherit self;};
      default = obs-studio;
    };
  };
}
