# Sprint 05 review — zfetch v0.5.0 (uncommitted working tree)

Reviewed 2026-11-08. HEAD `a8b2f18` (`main...origin/main`, nothing staged).
`Cargo.toml` version is `0.5.0`.

## 1. State snapshot

`git diff --stat` (tracked files, after the review fixes below):

```
 Cargo.lock             | 776 +------------------------------------------------
 Cargo.toml             |   3 +-
 README.md              |  36 +++++++++----
 src/advanced.rs        | 167 +++--------
 src/cli.rs             | 153 ++++++----
 src/config.rs          |  53 ++--
 src/display/layout.rs  | 174 +++--------
 src/display/logo.rs    |  27 +-
 src/display/mod.rs     |   5 +-
 src/hardware_ext.rs    |  34 +--
 src/main.rs            | 117 ++++----
 src/report.rs          | 221 ++++++++++----
 src/system/battery.rs  |  70 +++--
 src/system/bios.rs     |  36 +--
 src/system/cpu.rs      |  67 ++++-
 src/system/desktop.rs  |  26 +-
 src/system/disk.rs     |  26 +-
 src/system/display.rs  | 194 +++++--------
 src/system/host.rs     |   9 +-
 src/system/memory.rs   |  46 +--
 src/system/mod.rs      | 142 +++++----
 src/system/network.rs  |   2 +-
 src/system/os.rs       |  20 +-
 src/system/packages.rs | 136 ++++++++++++
 src/system/swap.rs     |  54 +---
 src/system/wm.rs       |  79 ++++-
 src/update.rs          | 162 +++++++++--
 27 files changed, 1140 insertions(+), 1695 deletions(-)
```

Untracked: `src/doctor.rs`, `src/system/util.rs`, `tests/cli.rs`, `scripts/measure.py`,
`docs/{CLI-v0.5.0,RELEASE-NOTES-v0.5.0,V0.5.0}.md`, `docs/development/*` (baseline, review, verify files).
`git diff --check`: clean.

Leftover `0.4.0` strings: README lines 13 and 212 were fixed (see section 4). The README release-asset
table, the install commands, and "What's new in v0.4.0" still say 0.4.0 **on purpose**: they name
files that exist on the v0.4.0 release, and no 0.5.0 assets exist yet (finding m7). `src/update.rs` uses
0.4.0 only as test fixture data. `RELEASE-NOTES-v0.5.0.md` and `CLI-v0.5.0.md` contain no 0.4.0.
`V0.5.0.md` mentions 0.4.0 only as the comparison baseline.

## 2. Findings

| Severity | File:line | Description | Suggested fix |
| --- | --- | --- | --- |
| **major** | `src/system/display.rs` (`drm_displays`, `active`) | The display list no longer uses `gdctl show` as its primary source, as the Sprint 05 prompt specified. It reads DRM/EDID from sysfs, and `gdctl` runs only if no EDID is readable. This was chosen because `gdctl` costs ~108 ms on every default run (startup gate). Your answer to the question about this never arrived, so it is **not approved**. Effects: it shows the EDID *preferred* mode, not a changed or VRR current mode. Connector names follow the kernel (`HDMI-A-1`, where Mutter may say `HDMI-1`). It also lists physically connected panels with no GNOME session (seen in the container test: `Display 1920x1080 @ 60 Hz` appeared inside podman). | Owner decides: (a) accept as is; (b) use `gdctl show` as primary, as specified (~+100 ms default startup); (c) sysfs by default and exact `gdctl` with `--hardware-details`. |
| **major** | `src/system/desktop.rs:79-81` | The GNOME version now comes from the dpkg status file first, then `gnome-shell --version`. On **non-Debian GNOME** (Fedora, Arch) the default path gains a new ~30 ms subprocess where v0.4.0 used D-Bus. That conflicts with "no new subprocesses on the default path". On Debian/Ubuntu there is no subprocess (verified with strace). dpkg also reports the *installed* version, not the running one, until the next login. | Owner decides. File-only options: read `/var/lib/pacman/local/gnome-shell-*/desc` (Arch) and `/usr/share/gnome/gnome-version.xml` where present, and fall back to the subprocess only after those. RPM has no file-only source. |
| minor | `src/display/mod.rs:85-91` | Privacy: a custom `--logo` or `logo.source` file is printed unredacted under `--privacy`. A logo containing the username is shown verbatim (verified). This predates v0.5.0 (v0.4.0 identical). Text and JSON export do not include the logo, so only the terminal is affected. | Proposed diff P1 below (needs approval: output change). |
| minor | `src/main.rs:17-19, 56` | Privacy: error messages on stderr are not scrubbed. `zfetch --privacy --config /home/<user>/x.jsonc` prints the path with the username (verified). Predates v0.5.0. | Proposed diff P2 below (needs approval). |
| minor | `src/system/packages.rs:270-272` | The new APK, Portage and XBPS sources use `include_zero = true`. An empty `/lib/apk/db/installed` or an empty `/var/db/pkg` on another distro would print `0 (apk)` / `0 (portage)`. | Proposed diff P3: `include_zero = false` for these three (needs approval). |
| minor | `src/system/packages.rs` (`count_xbps_at`) | The XBPS parser was tested only against a hand-made fixture. It assumes proplib puts `<string>installed</string>` on its own line. If several `pkgdb-*.plist` files exist, the first `read_dir` entry wins. | Owner: verify in a `voidlinux/voidlinux` container before claiming Void support. |
| minor | `src/system/display.rs` (`drm_displays`) | A connected output whose EDID is unreadable is skipped silently when another output has EDID, and the `gdctl` fallback runs only when *all* outputs fail. | Part of the display decision above. |
| minor | `src/update.rs` (`install_method`) | Without running `rpm`, RPM ownership is assumed when the binary is `/usr/bin/zfetch` and an rpmdb exists. A hand-copied binary on Debian with `rpm` installed would be called "RPM package". The impact is only wrong manual instructions; nothing is installed. | Accept, or document. |
| minor | `docs/development/baseline-v0.4.0/` | The raw v0.4.0 outputs contain the username, hostname, local IP and personal mount paths. | Resolved at commit time: the directory is left untracked (local only). The verify logs were committed with paths shown as `<repo>` / `<tmp>` and the IP as `<local-ip>`. |
| minor | `README.md:70-130` | The release-asset table and install commands name 0.4.0 files, which are correct until the 0.5.0 assets exist. | Update to 0.5.0 file names when the release assets are final (checklist item 9). |
| nit | `src/system/util.rs` (`Usage::percent`), `src/system/battery.rs` | Rounding is unified: swap was floored before and is now rounded, and memory is computed from raw bytes instead of the 2-decimal strings. Each can differ by 1%. With percentages hidden and battery telemetry present, the battery shows `Charging \| ~1h` instead of `(Charging) \| ~1h`. The swap change is in the release notes; the battery one is not. Identical in every comparison run here. | Add the battery line to the release notes. |
| nit | `src/system/packages.rs` (`count_apk_at`) | A malformed `P:` line with an empty name still counts as a package. | Accept, or count only `P:` lines with a non-empty name. |
| nit | `src/doctor.rs:127` | The text "(DE/session show Unknown)" is printed for both variables. For `XDG_CURRENT_DESKTOP` it is DE and WM that show Unknown. | Wording change in output; needs approval. |
| nit | `src/system/{packages,terminal,desktop}.rs` | `/var/lib/dpkg/status` can be read up to three times in parallel threads. Measured startup is still about 10% faster than v0.4.0. | Accept. |
| nit | `src/config.rs`, `src/advanced.rs` (pre-existing) | "Unknown module 'nope' in modules.system" does not name the file, and a JSON5 syntax error reads "EOF parsing identifier at line 1 column 1". The messages are identical to v0.4.0. | Later: prefix with the config path. |
| info | `src/system/mod.rs:135`, `src/display/logo.rs:44` | The only `expect()` calls outside tests: a thread-join panic re-raise, and an embedded logo that always exists. Both predate v0.5.0 and are not reachable from input. | None. |
| info | `src/report.rs` | A previous summary claimed "`--update` expects `SHA256SUMS` but CI only makes `SHA256SUMS-<arch>`; existing bug". That was **wrong**: the v0.4.0 release has a combined `SHA256SUMS` asset (`gh release view v0.4.0`). `docs/V0.5.0.md` is corrected. | Attach a combined `SHA256SUMS` to v0.5.0 (checklist item 7). |

Blockers: **0**. Major: **2**. Minor: **8**. Nit: **5**. Info: 2.

### Checks with no finding

- **zbus**: absent from `Cargo.toml`, `Cargo.lock` (`grep -c zbus` = 0) and `src/`; `cargo tree -i zbus` matches nothing.
- **Usage refactor**: the old and new binaries produce byte-identical output for `--no-color`, `--no-logo`, `--layout compact|detailed`, `--preset minimal|full`, `--only memory,swap,disk,battery`, `show_percentage: false` + `show_efi: true`, and **colored TTY output** (via `script`, which checks the color thresholds). See `SPRINT-05-VERIFY-10`. There is no `--no-percent` flag; the only switch is the config option `display.show_percentage`.
- **Privacy**: terminal, JSON, and txt/json export are identical to v0.4.0 for this machine, and no username or hostname appears in any of them. `--doctor --privacy` replaces `$HOME` with `~` and scrubs the username and hostname (integration test plus the container run). MAC, IPv6 and serial keys are covered by unit tests.
- **Update flow**: on non-`.deb` installs it prints manual steps and changes nothing (unit-tested; end-to-end impossible while v0.4.0 is the latest release, see checklist item 10). The `.deb` path still requires `[y/N]`, a size check, a SHA-256 check, and `dpkg-deb` name/version/arch checks. The user agent is `concat!("zfetch-updater/", env!("CARGO_PKG_VERSION"))`. Install-method detection was verified in a container: `.deb (dpkg)` for the installed binary, `portable or source build` for a copy.
- **Unsafe**: 9 `unsafe` uses, all in `disk.rs` (`statvfs`) and `network.rs` (`getifaddrs`). All predate v0.5.0, and each has a `// SAFETY:` comment that is still accurate. The only change is `&mut head` → `&raw mut head`, which passes the same pointer.
- **Dead code**: `cargo build` shows no warnings. `cargo machete` is not installed; a manual grep confirms json5, libc, serde, and serde_json are each used.
- **Parsers**: APK, Portage and XBPS return `None` (omitted) for missing paths, unreadable or non-UTF-8 files, and a file where a directory is expected. Empty input gives `Some(0)`. The new test `new_parsers_handle_missing_empty_and_malformed_input` covers this. WM detection treats empty socket variables as unset (`wm::tests`). Nix was not added.

## 3. Proposed diffs (NOT applied; they change behavior and need owner approval)

P1 — redact the custom logo under `--privacy` (terminal only):
```diff
--- src/main.rs
-            } else {
-                display::render(&info, &options, &config);
+            } else {
+                display::render(&info, &options, &config, sensitive.as_ref());
--- src/display/mod.rs
-pub fn render(info: &SystemInfo, options: &Options, config: &Config) {
+pub fn render(info: &SystemInfo, options: &Options, config: &Config, sensitive: Option<&Sensitive>) {
-    let logo_lines = logo::load(logo_path, no_logo);
+    let mut logo_lines = logo::load(logo_path, no_logo);
+    if let Some(s) = sensitive {
+        logo_lines.iter_mut().for_each(|line| *line = s.scrub(line));
+    }
```

P2 — scrub stderr messages under `--privacy`: `run()` would return the parsed `privacy`
flag with the error, and `main` would scrub with `Sensitive::new($USER, hostname, "")` plus the `$HOME` → `~`
replacement already used by `--doctor`.

P3 — omit zero counts for the new package sources:
```diff
--- src/system/packages.rs
-    add("apk", count_apk_at(Path::new("/lib/apk/db/installed")), true);
-    add("portage", count_portage_at(Path::new("/var/db/pkg")), true);
-    add("xbps", count_xbps_at(Path::new("/var/db/xbps")), true);
+    add("apk", count_apk_at(Path::new("/lib/apk/db/installed")), false);
+    add("portage", count_portage_at(Path::new("/var/db/pkg")), false);
+    add("xbps", count_xbps_at(Path::new("/var/db/xbps")), false);
```

## 4. Fixes applied during this review (allowed list)

| File | Fix | Category |
| --- | --- | --- |
| `src/system/packages.rs` | Added `new_parsers_handle_missing_empty_and_malformed_input` (APK/Portage/XBPS: missing, empty, malformed, non-UTF-8, file-instead-of-dir) | missing parser unit test |
| `src/system/packages.rs` | `cargo fmt` of that test | formatting |
| `README.md:13` | "Current source version" v0.4.0 → v0.5.0 | 0.4.0 version string |
| `README.md:212` | JSON example `zfetch_version` 0.4.0 → 0.5.0 | 0.4.0 version string |
| `README.md:295` | Test count 94 → 95, and removed the claim that GitHub Actions builds pass (they have not run on 0.5.0) | doc correction |
| `docs/V0.5.0.md:51-53` | Corrected the wrong `SHA256SUMS` claim | doc correction |

No behavior, output, dependency, version, or package-metadata change was made.

## 5. Verification results

| Step | Result | File |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS | `SPRINT-05-VERIFY-1-cargo-fmt.txt` |
| `cargo check --locked` | PASS | `SPRINT-05-VERIFY-2-cargo-check.txt` |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS | `SPRINT-05-VERIFY-3-cargo-clippy.txt` |
| `cargo test --locked` | PASS (95 unit + 3 integration) | `SPRINT-05-VERIFY-4-cargo-test.txt` |
| `cargo build --locked --profile compact` | PASS | `SPRINT-05-VERIFY-5-cargo-build.txt` |
| `zfetch --version` | PASS (`zfetch 0.5.0`) | `SPRINT-05-VERIFY-6-…version.txt` |
| smoke-v0.3.0.py | PASS (9/9) | `SPRINT-05-VERIFY-7-…smoke….txt` |
| `git diff --check` | PASS | `SPRINT-05-VERIFY-8-git-diff.txt` |
| CLI contract (35 valid, 24 invalid) | PASS 58/59; the 1 mismatch is a wrong expectation (export-overwrite exits 1 in v0.4.0 too) | `SPRINT-05-VERIFY-9-cli-contract.txt` |
| JSON schema, config, exports | PASS (no v0.4.0 key missing, schema 1; exports 0600 even with umask 000; overwrite and symlink refused) | `SPRINT-05-VERIFY-10-json-config-export.txt` |
| Offline (strace, `unshare -rn`) | PASS (0 `connect()`; 2 netlink `sendto` from `getifaddrs`, unchanged; runs without network) | `SPRINT-05-VERIFY-11-offline.txt` |
| Performance (3× paired 30 runs) | PASS (median 5.07–5.10 → 4.54–4.60 ms; RSS ~4,055 → ~3,354 KiB; 1,657,296 → 771,848 B) | `SPRINT-05-VERIFY-12-performance.txt` |
| Package metadata + `cargo deb --no-build` | PASS (assets exist, `zfetch_0.5.0-1_amd64.deb` contents correct). The local build depends on `libc6 >= 2.39`; CI builds on 22.04 for 2.35 | `SPRINT-05-VERIFY-13-package.txt` |
| Install in disposable container | PASS (podman ubuntu:24.04: install, `--version`, `--no-color`, `--doctor --privacy`, `--check-update`, remove) | `SPRINT-05-VERIFY-14-install-container.txt` |
