# zfetch v0.6.0: `--update` APT sandbox warning fix

Status: **Draft for owner review. Nothing here is final until the owner approves it.**
Code references were checked against the working tree at commit `a957407` (v0.5.0).

## 1. Root cause

Observed during `zfetch --update` on a `.deb` install:

```text
N: Download is performed unsandboxed as root as file '/tmp/zfetch-update-.../zfetch_..._amd64.deb'
   couldn't be accessed by user '_apt'. - pkgAcquire::Run (13: Permission denied)
```

What the code does (`src/update.rs`):

1. `TemporaryDirectory::new` creates `env::temp_dir()/zfetch-update-<pid>-<nanos>-<attempt>`
   with `fs::DirBuilder` and `builder.mode(0o700)`.
2. `run` downloads the `.deb` into that directory with `curl_bytes(..., Some(&deb_path))`.
3. `run` installs with `sudo apt-get install -- <deb_path>`.

APT reads local `.deb` files in its fetch step as the sandbox user `_apt`. Mode `0700` gives
**no search (x) permission** to anyone except the owner, so `_apt` cannot traverse into the
directory. APT falls back to reading the file as root and prints the notice. The install
itself still succeeds; the notice is noise, not a failure.

**Where my reading differs from the brief:** the brief says the directory's owner is root.
In the code, the directory is created by the zfetch process itself, and only `apt-get` runs
under `sudo`. With a normal `zfetch --update`, the owner is therefore **the invoking user**,
not root. If someone runs `sudo zfetch --update`, the owner is root. Either way the cause is
the same: mode `0700` blocks `_apt`, whoever the owner is.

A second, smaller factor: `curl --output` creates the `.deb` with the process umask. With a
umask of `077` the file is `0600`, and `_apt` could not read it even if the directory allowed
traversal. The fix must cover both.

## 2. Options

| Option | What changes | Pro | Con |
| --- | --- | --- | --- |
| **A. Directory `0711`, file `0644` (recommended)** | `builder.mode(0o711)`, then `set_permissions(0o711)`, because the umask filters `DirBuilder::mode`; after the download and before the checksum check, `fs::set_permissions(deb_path, 0o644)` | Two lines; `_apt` can traverse and read; others cannot list the directory; keeps `apt-get` and its dependency handling | The `.deb` is readable by local users who guess the exact name. It is a public release asset, so nothing is exposed |
| B. Install with `sudo dpkg -i` | replace `apt-get install` | no `_apt` step at all | loses APT dependency resolution and the APT log entry; changes the user-visible flow |
| C. Disable the sandbox: `-o APT::Sandbox::User=root` | one extra argument | hides the notice | turns off an APT safety feature on purpose. Rejected |
| D. Copy into `/var/cache/apt/archives` | `sudo cp` first | APT's own directory | extra privileged write outside zfetch's directory, plus cleanup there. Rejected |

**Recommendation: A.** Why `0711` and not `0755`: `_apt` needs only to pass through the directory
to a known file name. Without `r` on the directory, other users cannot list it, so they do not
learn the file name from the directory.

Write access stays owner-only in every option. No other user can replace the `.deb` between
the checksum check and APT reading it. `/tmp` stays sticky (`1777`), so another user cannot
rename or delete the directory either.

## 3. Cleanup

Current behavior (`TemporaryDirectory` in `update.rs`, `impl Drop` calls `fs::remove_dir_all`):

| Situation | Removed today? | Why |
| --- | --- | --- |
| Success | yes | `workspace` goes out of scope at the end of `run` |
| Any error returned with `?` (network, size, checksum, metadata, apt failure) | yes | normal unwinding of the `Result` path drops `workspace` |
| Panic | **no** | `Cargo.toml` sets `panic = "abort"` in `[profile.release]`, and `compact` inherits it, so `Drop` does not run |
| Ctrl-C | **no** | default `SIGINT` kills the process without running `Drop`. Ctrl-C during `curl` or `apt-get` reaches the whole foreground process group, so zfetch dies too |
| `SIGKILL`, power loss | no | cannot be handled in-process |

**Recommendation for Phase 2**

1. Keep the existing `Drop` guard. It already covers success and every error path.
2. For Ctrl-C: from the creation of the workspace until `run` returns, install a `SIGINT` (and
   `SIGTERM`) **handler** that only sets an `AtomicBool`. Use `libc::sigaction`; `libc` is
   already a dependency.
   - Use a handler, not `SIG_IGN`. An ignored signal stays ignored in child processes after
     `exec`, so `curl` and `apt-get` would ignore Ctrl-C. A caught signal is reset to the default
     on `exec`, so the children still stop on Ctrl-C.
   - After each child process returns, check the flag. If it is set, return
     `Err("update interrupted; no changes were made by zfetch")`. The guard then removes the
     directory.
   - Restore the previous handlers before `run` returns.
3. Accepted limit, documented in code as a `shortcut:` comment: `SIGKILL`, power loss, and an
   abort-panic can leave a `zfetch-update-*` directory behind. systemd-tmpfiles cleans `/tmp`
   on most distros. A startup sweep of old directories is **not** recommended: it adds work and
   risk to every `--update` run.

**Alternative:** keep the current behavior and document that Ctrl-C can leave a directory.
Trade-off: no new code, but the `.deb` stays in `/tmp` until the next cleanup.

Interrupting `apt-get` partway is APT's own concern. zfetch does not try to roll back dpkg state.

## 4. Security

The fix changes two permission bits and adds a signal flag. Each existing check stays exactly
where it is:

| Check | Where (`update.rs`) | Preserved how |
| --- | --- | --- |
| HTTPS only, exact asset URL | `curl_bytes` (`--proto =https`), `trusted_asset_url` | untouched |
| Size matches release metadata | `run`, `actual_size != deb.size` | untouched |
| SHA-256 against `SHA256SUMS` | `expected_checksum`, `verify_checksum` | untouched; `set_permissions` runs **before** the check, so the verified bytes are the bytes APT reads, and only the owner can write to the directory |
| Package name, version, architecture | `verify_deb_package` (`dpkg-deb --field`) | untouched |
| Native arch only | `native_deb_arch` | untouched |
| Confirmation prompt `[y/N]` | `run`, before `TemporaryDirectory::new` | untouched; nothing is downloaded before "y" |
| Never overwrite package-managed files of other managers | `install_method`; non-`.deb` returns manual steps | untouched; the fix is inside the `.deb` branch only |

The `SIGINT` flag can only make `run` return earlier with an error. It never skips a check.

Out of scope here, and covered by `PACKAGE-VERIFICATION-v0.6.0.md`: `SHA256SUMS` comes from the
same release as the `.deb`, so it proves integrity, not who published it.

## 5. Test plan (Phase 2, not written yet)

Unit tests in `update.rs` (no network, no root):

Tests 1 and 6 need `TemporaryDirectory::new` to take its base directory as an argument
(`new_in(base)`, with `run` passing `env::temp_dir()`). Changing `TMPDIR` or the umask inside a
test affects the whole test process, and tests run in parallel. In edition 2024,
`env::set_var` is also `unsafe`.

1. **Directory mode:** `TemporaryDirectory::new_in(base)` gives mode `0o711`. `DirBuilder::mode`
   is still filtered by the umask, so Phase 2 also calls `set_permissions(0o711)` after creating
   the directory. The test checks the final mode, not the requested one.
2. **File mode:** a file created with mode `0o600` is `0o644` after the permission step.
3. **Drop cleanup:** a `TemporaryDirectory` with a file inside is removed on drop, and also when
   dropped through an early `Err` return from a helper.
4. **Checksum mismatch:** `verify_checksum` on a temp file with a wrong digest returns the
   `SHA-256 MISMATCH` error (it calls the `sha256sum` binary, which CI images have).
5. **Non-`.deb` file:** `verify_deb_package` on a text file returns
   `downloaded .deb has invalid metadata` (needs `dpkg-deb`; skip with a message when it is absent,
   as on Arch and Rocky runners).
6. **Unreadable temp directory:** with a base directory of mode `0500`,
   `TemporaryDirectory::new_in(base)` returns `cannot create update directory: ...`, not a panic.
   (Run as non-root; root ignores the mode. Skip when `geteuid() == 0`.)
7. **Network failure:** `curl_bytes("https://127.0.0.1:9/", None)` (closed port) returns
   `network request failed: ...` quickly. It uses only loopback, no internet.
8. **Interrupt flag:** with the flag set, the post-child check returns the interrupted error,
   and the guard removes the directory.

Manual test on a `.deb` system (owner, after Phase 2):

9. Install v0.5.0 `.deb`, build v0.6.0 into a local test release, run `zfetch --update`: the
   `N: Download is performed unsandboxed` line no longer appears.
10. Press Ctrl-C during the download: `ls -d /tmp/zfetch-update-*` finds nothing.
