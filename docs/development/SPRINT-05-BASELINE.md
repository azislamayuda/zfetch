# Sprint 05 baseline and results (v0.4.0 → v0.5.0)

Measured on 2026-11-08, Ubuntu 26.04.1 LTS, GNOME 50.1 Wayland, Intel i5-1145G7,
Rust/cargo 1.99.1. Starting commit: `a8b2f18` (`main`, clean tree).

## Baseline (v0.4.0, before any change)

| Metric | Value | Command |
| --- | --- | --- |
| Crates (`cargo tree` lines) | 145 | `cargo tree -e normal \| sort -u \| wc -l` |
| Compact binary size | 1,657,296 bytes | `stat -c %s target/compact/zfetch` |
| Peak RSS, avg of 30 runs | ~4,075 KiB | `python3 scripts/measure.py <bin> 30` (`/usr/bin/time -f %M`) |
| Median startup, 30 runs | ~4.95 ms | `python3 scripts/measure.py <bin> 30` (`--no-logo --no-color`) |
| Unit tests | 80 passed | `cargo test --locked` |

Saved baseline output, for later comparison: `docs/development/baseline-v0.4.0/zfetch-no-color.txt`
and `zfetch.json`. These files contain the owner's username, hostname and local IP;
remove them or replace them with `--privacy` output before committing if that is a concern.

Note: the first RSS attempt read `ru_maxrss` through Python's `os.wait4`. It reported
~13.8 MiB for **both** binaries, because the value includes the forked Python interpreter.
`scripts/measure.py` now takes RSS from `/usr/bin/time -f %M` and timing from `posix_spawn`.

## After (v0.5.0 working tree)

| Metric | v0.4.0 | v0.5.0 | Change |
| --- | --- | --- | --- |
| Crates (`cargo tree` lines) | 145 | 21 | −124 |
| `Cargo.lock` packages removed | — | 76 | zbus and its async stack |
| Compact binary | 1,657,296 B | ~771,900 B | −53% |
| Peak RSS (paired, 2×30 runs) | 4,087 / 4,063 KiB | 3,344 / 3,349 KiB | −18% |
| Median startup (paired, 2×30 runs) | 4.94 / 4.97 ms | 4.47 / 4.41 ms | −10% |
| D-Bus `connect()` on a default run | 2 | 0 | `strace -f -e trace=connect,execve` |
| Subprocesses on a default run | `bash --version` | `bash --version` | unchanged |

Paired command: `python3 scripts/measure.py <baseline-bin>,target/compact/zfetch 30`.

## Dependency changes

| Crate | Before | After | Note |
| --- | --- | --- | --- |
| zbus | 5.19.0 | removed | Replaced by DRM/EDID sysfs (displays) and dpkg status / `gnome-shell --version` (GNOME version) |
| json5 | 1.3.2 | 1.3.7 | `cargo update`, semver-compatible; still used for JSONC config |
| serde | 1.0.229 | 1.0.229 | already latest; `derive` needed |
| serde_json | 1.0.154 | 1.0.154 | already latest; now also parses the GitHub release JSON |
| libc | 0.2.191 | 0.2.191 | latest stable (1.0 is alpha only); used by `statvfs` and `getifaddrs` |

- `cargo tree -d`: no duplicate versions.
- `cargo tree -i zbus`: "did not match any packages".
- `cargo-outdated` and `cargo-machete` were not installed. Latest versions were checked with
  `cargo search`, and each dependency was confirmed in use with `grep -rln <crate> src`.
- No feature trimming was possible: `serde/derive` is required, and the other crates use only their defaults.
