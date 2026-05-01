# Manifest Format

Hermesix manifests are JSON documents with a version and a list of managed
files.

```json
{
  "version": 1,
  "module": "programs.example",
  "files": []
}
```

Each file entry has these fields:

| Field | Description |
| --- | --- |
| `path` | Relative path below the command's `--config-dir`. |
| `source` | Source file path to copy from. |
| `target` | Expected absolute target path. |
| `sha256` | Hex-encoded SHA-256 of the source file. |
| `kind` | One of `ini`, `json`, or `raw`. |
| `origin` | Human-readable source option or generator label. |

`diff`, `sync`, and `validate` reject manifests with unsupported versions,
empty module names, absolute managed paths, or `..` path components. `sync
--apply` verifies source hashes before writing.
