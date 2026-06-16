# OBS Adapter

Hermesix includes OBS Studio helpers because the original managed-config use
case was a Home Manager OBS module.

Export an existing OBS configuration:

```sh
hermesix adapter obs export-to-nix ~/.config/obs-studio
```

The exporter reads common OBS files such as `global.ini`, `user.ini`, profile
settings, encoder JSON files, scene collections, and plugin JSON config. It
prints Nix assignments that can be used as a starting point for declarative
configuration.

Inspect an OBS plugin source tree:

```sh
hermesix adapter obs plugin-inspect --source-dir ./my-plugin
```

The inspection command emits JSON evidence for OBS source IDs, filters,
outputs, encoders, registrations, default settings, and property settings.
