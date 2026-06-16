# Redaction

Hermesix redaction is designed for exporting configuration into a portable
declarative form.

By default, redaction removes:

- fields with sensitive names such as token, key, password, secret, auth, or
  cookie
- runtime fields such as window geometry and last update checks
- local absolute paths when the key looks path-like

Use these flags when you intentionally want to keep those values:

```sh
--include-sensitive
--include-runtime
--include-local-paths
```

The same redaction flags are used by `hermesix redact`, `hermesix validate`,
and `hermesix adapter obs export-to-nix`.
