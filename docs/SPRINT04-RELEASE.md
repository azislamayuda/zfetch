# zfetch v0.3.0 — Sprint 04: Quality and Release Readiness

## Scope

This patch supplies repeatable validation and release tooling **without modifying Rust application code**. It intentionally avoids introducing new renderer dependencies or last-minute startup work that could regress performance.

- GitHub Actions checks: Rust format, check, strict Clippy, unit tests, compact build, offline CLI smoke tests.
- Offline CLI regression suite for layouts, JSON schema, module filters, configuration safety, ASCII logos and rejected flag combinations.
- Reproducible local `hyperfine` startup and RSS report script, with optional v0.2.0 vs v0.3.0 comparison.
- Locally build `.deb` via `cargo deb`, verify package metadata, packaged executable and third-party artwork attribution files, then generate `SHA256SUMS`.
- CLI usage notes, release notes draft, and manual end-to-end QA checklist.

## Pre-release validation on Ubuntu

```bash
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --profile compact
python3 scripts/smoke-v0.3.0.py target/compact/zfetch
```

If `cargo --locked` reports a lockfile mismatch, regenerate the lock locally with `cargo check`, inspect the changes, and commit `Cargo.lock` using GitHub Desktop. Do **not** remove `--locked` from CI.

The smoke script uses a temporary `XDG_CONFIG_HOME` and never writes into `~/.config/zfetch`; it does not execute any network update commands.

## Performance baseline

```bash
sudo apt install hyperfine time
bash scripts/benchmark-v0.3.0.sh ./target/compact/zfetch
```

For a fair v0.2.0 comparison, use an **actual 0.2.0** executable as the second argument. `--no-logo --no-color` on both does not guarantee identical config/module selection; check those first. Save each result under ignored `target/benchmarks/`.

```bash
bash scripts/benchmark-v0.3.0.sh ./target/compact/zfetch /path/to/zfetch-v0.2.0
```

Do not use an old v0.1.0 benchmark as if it were a paired v0.3.0 run. `--hardware-details` intentionally increases sysfs readings and must be compared separately from default runs.

## Debian packaging — no automatic installation

```bash
cargo install cargo-deb --locked  # only if not installed
bash scripts/build-deb-v0.3.0.sh
```

Expected artifacts:

```
target/release-assets/v0.3.0/
├── zfetch_0.3.0-1_amd64.deb
└── SHA256SUMS
```

The release script verifies the `.deb` Package, Version, Architecture, extracted binary version and mandatory third-party artwork licensing files. If the package differs from `zfetch_0.3.0-1_amd64.deb`, stop and investigate rather than rename it. The script performs no `sudo`, APT install, Git tag, release publication or Git push.

Before installing:

```bash
sudo apt-get -s install ./target/debian/zfetch_0.3.0-1_amd64.deb
```

Check the simulation's install/remove lines. After approving the plan, **manually** install:

```bash
sudo apt install ./target/debian/zfetch_0.3.0-1_amd64.deb
zfetch --version
dpkg-query -W zfetch
zfetch --check-update
```

`--check-update` needs network and reads the latest public release. Before publishing v0.3.0, it may report v0.2.0 as the latest release. This is expected and must never prompt a downgrade.

## Interactive updater: external QA is still necessary

`zfetch --update` was designed to fetch an official `.deb` asset and verify `SHA256SUMS` before proposing installation, but the **positive update path** needs an actual newer, compatible GitHub release to test safely. This patch does not simulate network responses, bypass HTTPS, fake a GitHub Release, or automatically install anything.

Checklist for the next genuine update cycle:

1. On a supported test machine with v0.2.0, check v0.3.0 availability after publication.
2. Verify a decline (`N`) leaves v0.2.0 installed and configuration untouched.
3. Verify downloaded package SHA-256 and packaged Debian metadata are checked.
4. Approve installation only on a disposable/test machine after manual review of the upgrade plan.
5. Recheck `dpkg-query -W zfetch`, `zfetch --version`, logo selection, JSON output and config preservation.
6. Test unavailable GitHub, missing/incorrect checksum, missing asset, malformed release and unsupported architecture in separate controlled integration tests in a later sprint.

Do not claim that the update manager was fully end-to-end tested solely because `--check-update` succeeds.

## Terminal width

The Sprint 03 renderer may still produce long lines on narrow terminals, especially detailed CPU/GPU descriptions. Sprint 04 includes **manual regression coverage** but does not modify the renderer. Test at 80, 100 and 120 columns:

```bash
COLUMNS=80 ./target/compact/zfetch --layout detailed --only cpu,gpu,display,battery
COLUMNS=100 ./target/compact/zfetch --layout classic --no-logo
```

Note that `COLUMNS` only influences an app if the renderer actually reads it; use Ptyxis window resizing or `stty cols 80` in an appropriate interactive terminal to test actual behavior. Terminal-responsive hard wrapping or truncation is deferred until output behavior is specified and tested; silently cutting hardware information would be worse.

## Publish only after all gates pass

- [ ] `cargo fmt`, check, strict Clippy, test, compact build all pass locally.
- [ ] Offline smoke test passes and CI is green after GitHub Desktop push.
- [ ] Standard and detailed terminal output verified; MJ custom logo and Ubuntu auto logo verified.
- [ ] JSON parses with Python; `--layout detailed --format json` is correctly rejected.
- [ ] Default startup, RAM, and binary size measured and recorded with actual v0.3.0 data.
- [ ] Debian `.deb` metadata, executable and bundled license attribution verified.
- [ ] `apt-get -s install` upgrade plan reviewed; package installed and tested manually.
- [ ] Release assets include matching `SHA256SUMS`.
- [ ] All source changes committed and pushed in GitHub Desktop before tagging `v0.3.0`.

## GitHub Desktop commit

`chore: add v0.3.0 CI regression benchmarks and release tooling`

Use GitHub Desktop for Changes -> Commit to main -> Push origin. Create the GitHub Release manually only once release-gate verification succeeds.
