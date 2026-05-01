+++
title = "Hermesix"

[extra]
tagline = "Managed configuration, carried safely."
subtitle = "Hermesix gives Home Manager modules a small, hash-checked CLI for diffing, syncing, validating, and redacting generated configuration files."

[[extra.features]]
title = "Manifest diff and sync"
description = "Compare generated files with live configuration and apply missing or changed files only when requested."

[[extra.features]]
title = "Hash-checked writes"
description = "Verify source file hashes before applying managed files to a user configuration directory."

[[extra.features]]
title = "Validation"
description = "Reject unsafe manifest paths, unsupported versions, mismatched targets, bad hashes, and non-portable values."

[[extra.features]]
title = "Redaction"
description = "Remove sensitive fields, runtime state, and machine-local paths from JSON and INI files."

[[extra.features]]
title = "OBS export"
description = "Export existing OBS Studio profiles, scene collections, and plugin configuration into Nix-friendly output."

[[extra.features]]
title = "Plugin inspection"
description = "Scan OBS plugin source trees for source IDs, registrations, defaults, and property settings."

[[extra.features]]
title = "Compatibility aliases"
description = "Keep existing Home Manager and OBS command names while exposing the Hermesix CLI."
+++
