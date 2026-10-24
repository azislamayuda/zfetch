# zfetch v0.4.0 — Sprint 03: Secure Updates and Debian Packaging

## Scope

- Architecture-aware `.deb` selection for x86_64/amd64 and aarch64/arm64.
- Prevent source builds or shadowed executables from modifying installed DPKG packages.
- Validate the full asset download URL belongs to the selected official tag.
- Restrict package download to 64 MiB and checksum file to 1 MiB.
- Check package metadata, SHA-256 and size before requesting elevated installation.
- Build AMD64 and ARM64 `.deb` natively in GitHub Actions.
- No automatic network use in regular `zfetch`.

## Limits

SHA256SUMS hosted in the *same* GitHub Release verifies download consistency, **not publisher provenance** if a GitHub account/release is compromised. Artifact signing / attestations remain recommended future work.

GitHub Actions only uploads workflow artifacts. Publishing a Release asset, aggregating both `.deb` checksum lines into one `SHA256SUMS`, and end-to-end APT installation remain separate release steps. Do not announce multi-arch releases until the ARM64 runner succeeds and an ARM device verifies functionality.

Do not change version in Cargo.toml yet: retain v0.3.0 during sprint development.

## Local verification

```
cargo fmt --all
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --profile compact
./target/compact/zfetch --check-update
```

`--check-update` invokes GitHub; standard zfetch remains offline. Do not run `--update` as a smoke test against unreviewed release assets.

## GitHub Desktop commit message

`feat(update): validate architecture and release assets; add Debian CI`
