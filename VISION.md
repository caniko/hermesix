# Hermesix vision

Hermesix is a generic command line tool for managed configuration workflows. It
diffs, syncs, validates, and redacts files described by a versioned manifest, so
configuration that a declarative tool generated can be inspected, exported, and
moved without hand-copying files or leaking machine-specific values.
Application-specific knowledge lives in adapters, starting with OBS Studio.

## Problem

Declarative configuration tools generate files, but once written they are just
files. Users want to compare them with the live directory, move an export to
another machine, or commit a portable form of them — and a naive copy either
misses a changed source or carries secrets, runtime state, and absolute local
paths into the export. An ad-hoc script per application also duplicates the same
comparison and redaction logic.

## Principles

- The manifest is the contract: it carries a version, a module name, and per-file
  source, target, SHA-256, kind, and origin, so every operation works from
  recorded facts rather than from whatever happens to be on disk.
- Hash-checked writes. `sync --apply` verifies source hashes before writing, and
  validation rejects unsupported versions, empty module names, absolute managed
  paths, and `..` components.
- Redaction is the default, not an option. Sensitive names, runtime fields, and
  local absolute paths are removed unless the caller explicitly asks for them,
  and the same flags apply to `redact`, `validate`, and adapter exports.
- Keep the core generic. Adapter-specific behaviour stays under `adapters`, so
  the diff/sync/validate/redact engine has no OBS or audio-interface concepts.
- Layered, not replacing: Home Manager still owns the declarative OBS Studio
  module; Hermesix adds export, sync, diff, validate, and redact tooling beside
  it through a separate module.
- Reproducible delivery: the flake builds the binary and the mdBook docs, and the
  release notes and publishing checklist are part of the documentation set.

## Scope

The `diff`, `sync`, `validate`, and `redact` commands over JSON and INI files;
the versioned manifest format; the OBS Studio adapter (export-to-Nix and plugin
inspection) with its plugin schemas; the Home Manager `obs-studio` module; the
Nix package and docs site.

## Non-goals

Hermesix is not a configuration management system and does not replace Home
Manager or the upstream modules that declare a program's settings. It does not
try to infer arbitrary application semantics outside its adapters.

## Direction

The published 0.1.0 release candidate covers the manifest engine, redaction, and
the OBS Studio adapter. No roadmap beyond that release is documented.

## Place in the fleet

Canix pins the hermesix flake and enables `programs.hermesix` on personal
machines, which installs the CLI alongside the Home Manager OBS Studio module.
