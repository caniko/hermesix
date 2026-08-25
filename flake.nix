{
  description = "Generic managed configuration utilities";

  inputs = {
    rs-harbor.url = "git+https://github.com/caniko/harbor-rs.git?ref=trunk&rev=05cc4f162b55fa904b687db1821e2463fa813e50";

    nixpkgs.follows = "rs-harbor/nixpkgs";
    rust-overlay.follows = "rs-harbor/rust-overlay";
    crane.follows = "rs-harbor/crane";
    flake-utils.url = "github:numtide/flake-utils";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    plinth = {
      url = "git+https://github.com/caniko/plinth.git?ref=refs/heads/trunk";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs =
    {
      self,
      advisory-db,
      nixpkgs,
      plinth,
      rs-harbor,
      rust-overlay,
      ...
    }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      lib = nixpkgs.lib;
      forAllSystems = lib.genAttrs systems;

      perSystem = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          toolchain = rs-harbor.lib.mkToolchain { inherit pkgs; };
          cross = rs-harbor.lib.mkCross { inherit pkgs system; };
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
                platforms = lib.platforms.unix;
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

          fmtCheck = craneLib.cargoFmt { src = cargoSrc; };

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
            hermesix = hermesix;
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
          };

          website = pkgs.stdenv.mkDerivation {
            pname = "hermesix-website";
            inherit version;
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
            inherit version;
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
        in
        {
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

            docs = rs-harbor.lib.mkDocsShell {
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
    in
    {
      packages = lib.mapAttrs (_: value: value.packages) perSystem;
      apps = lib.mapAttrs (_: value: value.apps) perSystem;
      checks = lib.mapAttrs (_: value: value.checks) perSystem;
      devShells = lib.mapAttrs (_: value: value.devShells) perSystem;
      homeManagerModules = rec {
        obs-studio = import ./modules/home-manager/obs-studio.nix { inherit self; };
        default = obs-studio;
      };
    };
}
