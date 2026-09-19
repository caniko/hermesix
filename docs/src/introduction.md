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

## Home Manager OBS integration

Home Manager owns the declarative OBS Studio module. Hermesix can be layered on
top as a separate flake module to install the companion CLI:

```nix
{
  imports = [
    inputs.hermesix.homeManagerModules.obs-studio
  ];

  programs.obs-studio.enable = true;
}
```

The Home Manager module generates OBS configuration. Hermesix provides
export/sync/diff/validate/redact tooling for users who want those workflows.

Source code is hosted at <https://github.com/caniko/hermesix>.
