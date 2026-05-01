# CLI Reference

## Generic Commands

```sh
hermesix diff --manifest PATH --config-dir DIR [--json]
hermesix sync --manifest PATH --config-dir DIR [--apply] [--json]
hermesix validate --manifest PATH --config-dir DIR
hermesix redact FILE --format auto|json|ini
```

`diff` exits with status `1` when managed files are missing or changed.

`sync` is a dry run unless `--apply` is passed. With `--apply`, Hermesix checks
manifest shape and source hashes before copying files.

`validate` checks manifest shape, source hashes, parseability for known file
kinds, and portable configuration policy.

`redact` prints a sanitized copy of a JSON, INI, or raw file.

## OBS Commands

```sh
hermesix obs export-to-nix [CONFIG_DIR]
hermesix obs plugin-inspect --source-dir DIR
hermesix obs plugin-inspect verify --evidence FILE --source-dir DIR
```

The Nix package also installs compatibility command names:

- `hm-managed-config`
- `obs-studio-sync`
- `obs-studio-export-to-nix`
