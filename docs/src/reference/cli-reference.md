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

## Adapter Commands

```sh
hermesix adapter obs export-to-nix [CONFIG_DIR]
hermesix adapter obs plugin-inspect --source-dir DIR
hermesix adapter obs plugin-inspect verify --evidence FILE --source-dir DIR
```

OBS Studio is the first adapter. The Nix package installs only the `hermesix`
binary.
