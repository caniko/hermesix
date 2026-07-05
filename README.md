# Hermesix

<!-- simit:badges:start -->
[![CI](https://img.shields.io/badge/CI-managed+extra-2088ff)](.forgejo/workflows/ci.yaml) [![Nix](https://img.shields.io/badge/Nix-managed-5277c3)](flake.nix) [![docs](https://img.shields.io/badge/docs-enabled-6f42c1)](docs) [![crates.io](https://img.shields.io/badge/crates.io-ready-f46623)](https://crates.io/crates/hermesix)
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

From source:

```sh
cargo install --git https://codeberg.org/caniko/hermesix
```

## Commands

```sh
hermesix diff --manifest manifest.json --config-dir "$XDG_CONFIG_HOME/example"
hermesix sync --manifest manifest.json --config-dir "$XDG_CONFIG_HOME/example" --apply
hermesix validate --manifest manifest.json --config-dir "$XDG_CONFIG_HOME/example"
hermesix redact config.json --format json
hermesix adapter obs export-to-nix ~/.config/obs-studio
hermesix adapter obs plugin-inspect --source-dir ./plugin
```

## Documentation

The documentation site is published at
<https://caniko.codeberg.page/hermesix/docs/>.

## Release Status

Hermesix is prepared for a crates.io `0.1.0` release, but publishing is a
separate manual step.
