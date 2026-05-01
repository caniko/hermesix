# Quick Start

Create a manifest that describes a managed file:

```json
{
  "version": 1,
  "module": "programs.example",
  "files": [
    {
      "path": "settings.json",
      "source": "/nix/store/example-settings.json",
      "target": "/home/alice/.config/example/settings.json",
      "sha256": "hex-encoded-sha256",
      "kind": "json",
      "origin": "settings"
    }
  ]
}
```

Compare live configuration with the manifest:

```sh
hermesix diff --manifest manifest.json --config-dir ~/.config/example
```

Apply managed files:

```sh
hermesix sync --manifest manifest.json --config-dir ~/.config/example --apply
```

Validate the manifest and source files:

```sh
hermesix validate --manifest manifest.json --config-dir ~/.config/example
```
