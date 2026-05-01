# Installation

## Nix

Run Hermesix directly from the flake:

```sh
nix run codeberg:caniko/hermesix -- --help
```

Install it into a profile:

```sh
nix profile install codeberg:caniko/hermesix
```

## Cargo

Until the first crates.io release is published, install from the Codeberg
repository:

```sh
cargo install --git https://codeberg.org/caniko/hermesix
```

After publication, the intended install command is:

```sh
cargo install hermesix
```
