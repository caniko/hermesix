# Hermesix

Hermesix provides generic command line utilities for managed configuration
workflows. It operates on a versioned manifest that describes generated files,
their source paths, target paths, content hashes, file kinds, and origin labels.

The CLI can:

- compare managed files with the live configuration directory
- sync missing or changed files using hash-checked writes
- validate manifest shape, source hashes, and portable configuration policy
- redact sensitive, runtime, and local-path values from JSON and INI files
- run application-specific adapters, starting with OBS Studio export and plugin
  inspection helpers

Source code is hosted at <https://codeberg.org/caniko/hermesix>.
