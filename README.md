# Hermesix

Hermesix is a command line tool for Home Manager managed configuration
workflows. It can diff, sync, validate, and redact files described by a
versioned manifest, and it includes OBS Studio adapters for exporting existing
OBS configuration and inspecting plugin source trees.

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
hermesix obs export-to-nix ~/.config/obs-studio
hermesix obs plugin-inspect --source-dir ./plugin
```

Compatibility command names are also installed by the Nix package:
`hm-managed-config`, `obs-studio-sync`, and `obs-studio-export-to-nix`.

## Documentation

The documentation site is published at
<https://caniko.codeberg.page/hermesix/docs/>.

## Release Status

Hermesix is prepared for a crates.io `0.1.0` release, but publishing is a
separate manual step.
