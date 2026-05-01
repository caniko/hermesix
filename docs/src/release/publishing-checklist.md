# Publishing Checklist

Hermesix is prepared for crates.io publication, but publishing is manual.

Before publishing:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo package --list
cargo publish --dry-run
```

If all checks pass and the package contents are correct:

```sh
cargo publish
git tag v0.1.0
git push origin v0.1.0
```

Do not publish from an unreviewed or dirty working tree.
