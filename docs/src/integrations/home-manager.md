# Home Manager Integration

Hermesix is being developed alongside Home Manager integration work.

The current upstream draft pull requests are:

- <https://github.com/nix-community/home-manager/pull/9227> for adding Hermesix
  as managed configuration tooling
- <https://github.com/nix-community/home-manager/pull/9228> for adding
  declarative OBS Studio configuration

The standalone Hermesix repository contains the reusable CLI and documentation.
The Home Manager module implementation remains in the Home Manager pull
requests.

Home Manager modules that generate a Hermesix manifest should write JSON with:

- `version = 1`
- a non-empty `module` identifier such as `programs.example`
- `files[]` entries with safe relative paths and source hashes

Application modules choose the managed configuration root and write a manifest
under Home Manager state. The OBS module is one current producer of this
generic manifest format.
