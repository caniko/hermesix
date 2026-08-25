# Hermesix

<!-- simit:badges:start -->

[![CI](https://img.shields.io/badge/CI-drift-2088ff)](.forgejo/workflows/ci.yaml) [![Nix](https://img.shields.io/badge/Nix-managed-5277c3)](flake.nix) [![docs](https://img.shields.io/badge/docs-enabled-6f42c1)](docs) [![crates.io](https://img.shields.io/badge/crates.io-ready-f46623)](https://crates.io/crates/hermesix)

<!-- simit:badges:end -->

Hermesix is a generic command line tool for managed configuration workflows.
It can diff, sync, validate, and redact files described by a versioned
manifest. Application-specific helpers live under adapters; OBS Studio is the
first adapter.

## Install

With Nix:

```sh
nix run codeberg:caniko/hermesix -- --help
```

With Home Manager and the OBS Studio module:

```nix
{
  inputs.hermesix.url = "git+https://github.com/caniko/hermesix";

  outputs =
    {
      home-manager,
      hermesix,
      ...
    }:
    {
      homeConfigurations.example = home-manager.lib.homeManagerConfiguration {
        modules = [
          hermesix.homeManagerModules.obs-studio
          {
            programs.obs-studio.enable = true;
          }
        ];
      };
    };
}
```

Home Manager provides the declarative OBS Studio configuration module. Hermesix
provides the companion CLI, including export, sync, diff, validation, and
redaction commands.

From source:

```sh
cargo install --git https://github.com/caniko/hermesix
```

## Commands

```sh
hermesix diff --manifest manifest.json --config-dir "$XDG_CONFIG_HOME/example"
hermesix sync --manifest manifest.json --config-dir "$XDG_CONFIG_HOME/example" --apply
hermesix validate --manifest manifest.json --config-dir "$XDG_CONFIG_HOME/example"
hermesix redact config.json --format json
hermesix adapter obs export-to-nix ~/.config/obs-studio
hermesix adapter obs plugin-inspect --source-dir ./plugin
hermesix adapter goxlr capture --output-dir ./goxlr-capture --json
```

The GoXLR adapter copies the live Utility XDG tree into a deterministic source
tree and emits `goxlr-config-manifest.json`. It does not mutate the device or
runtime files; pass that manifest to the `goxlr-config` plan/apply/verify
commands supplied by the goxlr-nexus flake. This capture path is deterministic
and usable from scripts or CI without an LLM.

## Documentation

The documentation site is published at
<https://caniko.codeberg.page/hermesix/docs/>.

## Release Status

Hermesix is prepared for a crates.io `0.1.0` release, but publishing is a
separate manual step.
