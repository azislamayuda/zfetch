# zfetch v0.6.0: Snapshot and Compare design

Status: **Draft for owner review. Nothing here is final until the owner approves it.**
Phase 1 is design only. No code, `Cargo.toml`, or workflow is changed by this document.

Code references were checked against the working tree at commit `a957407` (v0.5.0).

## 1. Purpose and scope

A snapshot is a small JSON file that records the **stable configuration** of one machine:
OS, kernel, desktop stack, package counts, hardware models, and capacities. A later
`zfetch --compare <file>` answers one question: *what changed on this machine since then?*
Typical uses: before and after a distro upgrade, a kernel update, or a hardware swap,
and attaching a machine description to a bug report.

**In a snapshot**

- The `system` modules from `SYSTEM_MODULES` (`src/config.rs`).
- The stable part of the `hardware` modules from `HARDWARE_MODULES`: models, capacities,
  display list, disk layout, battery presence.
- Identity (`username`, `hostname`), redacted under `--privacy`.
- Metadata: snapshot schema version, zfetch version, creation time, privacy flag, module list.

**Deliberately left out**

| Left out | Why |
| --- | --- |
| Logo, colors, theme, layout | Presentation, not machine state |
| Memory used, swap used, disk used | Change on every run; would make every compare noisy |
| `uptime` | Changes every minute |
| `local_ip` | Changes with the network, and is personal data |
| Battery percent, state, time estimate | Change every minute |
| `--hardware-details` telemetry (`cpu_details`, `gpu_details`, `battery_details`, `display_details`) | Frequencies change every second; the stable parts are an open question (section 11) |
| Raw config file contents, config path | Personal, and not machine state |
| Environment variables, process list, logs | Out of scope and a privacy risk |

**Recommendation:** a fixed module set, independent of the user's config, so two snapshots
of the same machine always cover the same fields.
**Alternative:** honor `--only` / `--exclude` and the config's module list. Trade-off: smaller
snapshots on request, but two snapshots taken with different selections become hard to
compare, and every compare needs "not recorded" handling for whole modules.

## 2. Format

**Recommendation:** a separate JSON document with its own top-level `snapshot_schema_version: 1`,
and **no** `schema_version` key.

```json
{
  "snapshot_schema_version": 1,
  "kind": "zfetch-snapshot",
  "zfetch_version": "0.6.0",
  "created_unix": 1794650400,
  "privacy": false,
  "modules": ["os", "kernel", "...", "battery"],
  "identity": {"username": "...", "hostname": "..."},
  "system": {"os": "...", "kernel": "...", "packages": {"dpkg": 2134, "flatpak": 12}},
  "hardware": {"cpu": "...", "memory": {"total_bytes": 16000000000}, "disk": []}
}
```

Why this does not break `--format json` schema v1:

- The `--format json` path in `main.rs` (`run`, `Action::Run` arm) and
  `advanced::json_document` are not touched. Schema v1 keys and value formats stay as they are.
- The snapshot is a different document type, built by its own function. Reusing `schema_version`
  would invite tools to parse a snapshot as a v1 report and fail on the structured values below.
- A reader can tell the two apart from the top-level key alone: `schema_version` means a report,
  `snapshot_schema_version` means a snapshot. Compare uses this to give a clear error (section 8).

**Alternative:** store the `--format json` document as-is and diff it. Trade-off: no new
serializer, but values such as `"8.07 GiB / 14.86 GiB"` and `"2134 (dpkg), 12 (flatpak)"` are
display strings. Compare would have to parse them back, and used-memory noise would be in every
diff.

## 3. Fields

Types: `string|null` means the detector returned `"Unknown"` (or an empty value) and the snapshot
stores `null`, so compare can say "not detected" instead of comparing the word `Unknown`.

### Metadata (top level)

| Field | Source | Type | Redacted by `--privacy` |
| --- | --- | --- | --- |
| `snapshot_schema_version` | constant `1` | integer | no |
| `kind` | constant `"zfetch-snapshot"` | string | no |
| `zfetch_version` | `env!("CARGO_PKG_VERSION")` | string | no |
| `created_unix` | `SystemTime::now()` since `UNIX_EPOCH`, seconds | integer | no |
| `privacy` | `--privacy` given | boolean | no |
| `modules` | the fixed module list that was collected | array of strings | no |

`created_unix` is an integer to avoid adding a date crate. The terminal shows an age
("created 3 days ago"), computed from the current time.

### identity

| Field | Source | Type | Redacted |
| --- | --- | --- | --- |
| `identity.username` | `SystemInfo.username` (`$USER`) | string | yes, `"hidden"` |
| `identity.hostname` | `SystemInfo.hostname` (`/proc/sys/kernel/hostname`) | string | yes, `"hidden"` |

### system

| Field | Source (`src/system/`) | Type | Redacted |
| --- | --- | --- | --- |
| `system.os` | `os::name` | string\|null | scrubbed text |
| `system.kernel` | `os::kernel` | string\|null | scrubbed text |
| `system.arch` | `env::consts::ARCH` | string | scrubbed text |
| `system.shell` | `shell::name` | string\|null | scrubbed text |
| `system.de` | `desktop::name` | string\|null | scrubbed text |
| `system.session` | `session::name` | string\|null | scrubbed text |
| `system.wm` | `wm::name` | string\|null | scrubbed text |
| `system.terminal` | `terminal::name` | string\|null | scrubbed text |
| `system.packages` | per-manager counts from `packages.rs` | object `{manager: integer}` | no |

"Scrubbed text" means `Sensitive::scrub` in `report.rs`. It removes the username, hostname,
IP, MAC, and IPv6 tokens and replaces personal mount paths. Normal values pass through unchanged.

`system.packages` needs a small Phase 2 refactor. Today `packages::summary` returns one joined
string (`"2134 (dpkg), 12 (flatpak)"`). The plan is to split it into a function that returns
`(manager, count)` pairs, with `summary` joining them, so the terminal and JSON output stay
byte-identical.

### hardware

| Field | Source (`src/system/`) | Type | Redacted |
| --- | --- | --- | --- |
| `hardware.host` | `host::name` | string\|null | scrubbed text |
| `hardware.bios` | `bios::info` | string\|null | scrubbed text |
| `hardware.cpu` | `cpu::name(false)` | string\|null | scrubbed text |
| `hardware.gpu` | `gpu::name` | string\|null | scrubbed text |
| `hardware.display` | `display::active` | array of strings | scrubbed text |
| `hardware.memory.total_bytes` | `memory::usage` (`Usage.total`) | integer\|null | no |
| `hardware.swap.total_bytes` | `swap::usage` (`Usage.total`, `0` = disabled) | integer\|null | no |
| `hardware.disk[]` | `disk::all`, `/boot/efi` included | array of objects | see below |
| `hardware.disk[].mount_point` | `DiskInfo.mount_point` | string | personal mounts become `"[private mount]"` |
| `hardware.disk[].filesystem` | `DiskInfo.filesystem` | string | no |
| `hardware.disk[].total_bytes` | `DiskInfo.usage.total` | integer | no |
| `hardware.battery.present` | `battery::status().is_some()` | boolean | no |

Redaction runs in the same two steps as the report path in `main.rs`: `report::protect_info` on
`SystemInfo`, then `report::protect_json` on the finished document. A snapshot therefore cannot
redact less than `--format json --privacy`.

## 4. Stability rules

Within `snapshot_schema_version: 1`:

1. **Add:** a new field may be added at any time. It must be optional for readers.
2. **Readers ignore unknown fields.** Parse into `serde_json::Value`, never with
   `deny_unknown_fields`. When compare meets fields it does not know, it prints one line:
   "N fields from a newer zfetch were not compared".
3. **Never change a field's type or meaning.** `total_bytes` stays bytes, `packages` stays an
   object of integers.
4. **Rename = add new + keep old.** The old field is still written for at least one minor release
   and is marked deprecated in the CLI doc. Readers accept either.
5. **Deprecate:** keep writing the field. Removing it needs a schema bump.
6. **Breaking change:** bump to `snapshot_schema_version: 2`. The new reader keeps a v1 reading
   path (a `v1 -> v2` upgrade function) for at least two minor releases.

v0.6.0 snapshots stay readable by v0.7.0: v0.7.0 either still writes v1 (rules 1 to 5) or reads
v1 through the upgrade path (rule 6). In the other direction, v0.6.0 reading a v0.7.0 snapshot
works if it is still v1 (extra fields are ignored). If it is v2, it is refused with a clear
message (section 8).

**Alternative:** one schema version shared with `--format json`. Trade-off: one number to track,
but every snapshot change would force a report schema bump, and the reverse. Rejected.

## 5. CLI surface

```text
zfetch --snapshot <FILE> [--privacy]
zfetch --compare <FILE> [--format terminal|json] [--no-color]
```

- `--snapshot <FILE>`: collect, build the snapshot, write it with `report::write_export`
  (`O_CREAT|O_EXCL`, mode `0600`, never overwrites, does not follow an existing symlink, removes
  a partial file on write error). It prints `Snapshot saved: <FILE>` to stdout, without the file
  contents.
- `--compare <FILE>`: read and validate the file, collect the same module list, compare, and
  print the result to stdout.

Both are **action flags** (the `ACTION_FLAGS` table in `cli.rs`), but unlike the current ones
they take a value. Parsing reuses the `--export` value check: missing value, empty value, or a
value starting with `--` is an error.

| Combination | Result | Reason |
| --- | --- | --- |
| `--snapshot F --privacy` | allowed | redacted, shareable snapshot |
| `--snapshot F --config C` | rejected | the module set is fixed; the config has no effect |
| `--snapshot F --format json` | rejected | a snapshot is always JSON |
| `--snapshot` + any display option (`--only`, `--exclude`, `--preset`, `--layout`, `--theme`, `--logo*`, `--no-logo`, `--no-color`, `--hardware-details`, `--export`) | rejected | existing rule: "cannot be combined with display options" |
| `--compare F --format json` | allowed | machine-readable diff |
| `--compare F --no-color` | allowed | the terminal diff uses color |
| `--compare F --privacy` | allowed | redacts the current side as well (section 7) |
| `--compare F --export E` | rejected | redirect stdout instead; keeps one write path |
| `--compare` + other display options | rejected | the module set is fixed |
| `--snapshot` + `--compare` | rejected | existing rule: "Only one action flag can be used at a time" |
| either + another action flag | rejected | same |

Implementation note: `parse_args` already rejects `--privacy` and `--config` for every action
except the allowed ones (`config_ok`, the `name != "doctor"` check). Phase 2 extends those two
checks. It also needs a separate rule for `--format` / `--no-color`, which currently count as
`ordinary_options`.

**Alternative:** subcommands (`zfetch snapshot save F`, `zfetch snapshot compare F`). Trade-off:
reads well, but zfetch has no subcommands today. It would be a second parsing style for two
features.

## 6. Compare semantics

### Matching

Both documents are flattened to **paths**. Values are matched by path, never by position:

| Data | Path key |
| --- | --- |
| Scalars | `system.kernel`, `hardware.memory.total_bytes` |
| Packages | `system.packages.<manager>` |
| Disks | `hardware.disk[<mount_point>]`, comparing `filesystem` and `total_bytes` |
| Displays | set comparison of the strings; each entry is either present or added/removed |

Privacy edge case: several disks can all become `"[private mount]"`. Entries with the same key
are compared as a list sorted by `(filesystem, total_bytes)`.

### Kinds of result

| Kind | Meaning |
| --- | --- |
| `changed` | path exists on both sides, values differ |
| `added` | path exists now, not in the snapshot (new disk, new package manager) |
| `removed` | path in the snapshot, not now |
| `unchanged` | counted, not listed |
| `skipped` | not comparable: redacted on either side, or module not in the snapshot's `modules` |

A detector value that becomes `null` is shown as `changed` with `not detected`, not as `removed`.

### What counts as a change

The snapshot stores only stable fields (section 1), so **every difference in a stored field is
reported**. The noise filter is the field selection itself. No thresholds or fuzzy matching:

- kernel `6.8.0-40` to `6.8.0-45`: reported
- memory used: not stored, so never reported
- `memory.total_bytes`: reported (RAM added, or reserved memory changed after a firmware update)
- package count `2134` to `2140`: reported with the delta `(+6)`

Known noise: `disk.total_bytes` can shift by a few KiB when a filesystem is resized or
`statvfs` reports reserved blocks differently. This is accepted for v0.6.0 and listed as an
open question.

### Terminal output

```text
Comparing with snapshot.json (zfetch 0.6.0, created 3 days ago)

changed  system.kernel              6.8.0-40-generic -> 6.8.0-45-generic
changed  system.packages.dpkg       2134 -> 2140 (+6)
added    hardware.disk[/data]       ext4, 931.51 GiB
removed  system.packages.snap       12

4 differences, 23 fields unchanged, 0 skipped.
```

Byte values are shown with the existing GiB formatting (`util::Usage::formatted` style).
Colors follow the current rules: off with `--no-color`, with `NO_COLOR`, or when stdout is not
a terminal.

### JSON output (`--compare F --format json`)

A separate document with its own `compare_schema_version: 1`, not report schema v1:

```json
{
  "compare_schema_version": 1,
  "snapshot": {"zfetch_version": "0.6.0", "created_unix": 1794650400, "privacy": false},
  "changed": [{"path": "system.kernel", "old": "6.8.0-40-generic", "new": "6.8.0-45-generic"}],
  "added": [{"path": "hardware.disk[/data]", "new": {"filesystem": "ext4", "total_bytes": 1000204886016}}],
  "removed": [{"path": "system.packages.snap", "old": 12}],
  "skipped": [{"path": "identity.hostname", "reason": "redacted"}],
  "unchanged_count": 23
}
```

### Exit code

**Recommendation:** exit `0` whether or not there are differences. `1` = runtime or file error,
`2` = usage error, as in `main.rs` (`usage`, `failure`). Scripts check `changed`/`added`/`removed`
in the JSON.
**Alternative:** `diff(1)` style (`0` same, `1` different, `2` trouble). Trade-off: shell-friendly,
but `1` already means "runtime failure" in zfetch. A failed read and "found differences" would
look the same. Listed as an open question.

## 7. Privacy

Snapshots are made to be shared, so the redaction must match every other output:

- `--snapshot F --privacy` runs `report::protect_info` and then `report::protect_json` on the
  snapshot document, the same path as `--format json --privacy`. The file records `"privacy": true`.
- `--compare F` against a snapshot with `"privacy": true` **always** redacts the current side as
  well, even without `--privacy`. Otherwise the diff would print the real hostname next to
  `hidden`. Redacted paths are reported as `skipped (redacted)`, never as `changed`.
- `--compare F --privacy` against a non-private snapshot redacts the **output**: both old and new
  values go through `Sensitive::scrub` built from the current identity. Old identity values that
  differ from the current ones (for example, a hostname that has since changed) are redacted by
  the key rule for `identity.*`.
- The snapshot never contains the `local_ip` or serial numbers, so those are protected by
  omission, not only by redaction.

Wording for the CLI doc and release notes, same as v0.5.0: **Privacy Mode is not a guarantee of
anonymity.** Disk sizes, package counts, and the exact hardware list together can still identify
a machine. Review a snapshot before you share it.

**Alternative:** no `--privacy` on `--snapshot`; users redact by hand. Trade-off: less code, but
it breaks the v0.5.0 promise that every output shares one redaction path.

## 8. Failure modes

All errors print `zfetch snapshot: ...` or `zfetch compare: ...` to stderr and exit `1` (runtime,
the `failure` helper in `main.rs`). Parse errors in the arguments stay exit `2`.

| Case | Detection | Message (draft) |
| --- | --- | --- |
| Snapshot target exists | `write_export` `create_new` fails | `Cannot create new report F: File exists` (reword "report" to "file" in Phase 2) |
| Snapshot target is a symlink | `O_EXCL` refuses existing symlinks | same as above |
| Compare file missing / unreadable | `open` error | `cannot read F: <io error>` |
| Compare file is a symlink | `open` with `O_NOFOLLOW` gives `ELOOP` | `F is a symbolic link; refusing to read it` |
| Compare file is a FIFO, device, or directory | open with `O_NONBLOCK`, then `fstat` and require a regular file | `F is not a regular file` |
| Too large | `fstat` size > 1 MiB, and reading through `take(1 MiB + 1)` in case the file grows | `F is larger than 1 MiB; not a zfetch snapshot` |
| Not JSON / not UTF-8 | `serde_json` error | `F is not valid JSON: <line:col>` |
| A `--format json` report | top-level `schema_version` present, no `snapshot_schema_version` | `F is a --format json report, not a snapshot; create one with --snapshot` |
| Missing or wrong-type version | `snapshot_schema_version` absent or not an integer | `F is not a zfetch snapshot` |
| Newer schema | `snapshot_schema_version` > supported | `F was created by a newer zfetch (snapshot schema N); this version reads up to 1` |
| Field with the wrong type (e.g. `total_bytes: "x"`) | validation while flattening | `F: field hardware.memory.total_bytes has the wrong type` |
| Module missing on this system now | detector returns `Unknown` / `None` | **not an error**: `changed ... -> not detected` |
| Module not in the snapshot's `modules` | list check | **not an error**: `skipped (not recorded)` |

The 1 MiB limit is about 100 times a realistic snapshot (estimated at 2 to 10 KiB, **not
measured**). `O_NOFOLLOW` and `O_NONBLOCK` come from the `libc` crate, which is already a
dependency.

## 9. Performance

- The default `zfetch` path (`Action::Run` in `main.rs`) gets no new code. `--snapshot` and
  `--compare` are new `Action` variants, handled in their own `match` arms.
- `cli::parse_args` gains two match arms. That is string comparisons on arguments the default
  run does not have; the cost is negligible and is **not measured** as a separate number.
- The `packages` refactor (section 3) must keep `summary()` output byte-identical. The Sprint 05
  smoke and benchmark scripts are rerun in Phase 2 to show default-path time and RSS are unchanged.
- Binary size will grow a little (serializer plus diff). It is measured in Phase 2 against the
  v0.5.0 compact binary. No number is claimed here.

## 10. Test plan (Phase 2, not written yet)

Unit tests:

1. `cli`: `--snapshot` needs a value; empty and `--x` values are rejected; the allowed and
   rejected combinations in the section 5 table; `--snapshot` with `--compare` is rejected.
2. Snapshot builder: has `snapshot_schema_version: 1` and no `schema_version`; volatile fields
   (memory used, uptime, local_ip, battery percent) are absent; `Unknown` becomes `null`.
3. Snapshot privacy: with the `example_info()` fixture from `report.rs` tests, no username,
   hostname, IP, MAC, IPv6, or `/home/` text appears; `"privacy": true` is set.
4. `packages` split: `summary()` output is byte-identical to before for a fixture list.
5. Flatten and diff: changed / added / removed / unchanged by path; reordered arrays give no
   difference; disks are matched by mount point; duplicate `[private mount]` keys are handled.
6. Compare redaction: a private snapshot against non-private current data never prints the
   current hostname.
7. Reader: newer schema, report document, missing version, wrong field type, and unknown extra
   fields (ignored and counted).
8. File reading: symlink refused, FIFO refused without blocking, file over 1 MiB refused,
   non-UTF-8 refused.

Integration tests (`tests/cli.rs`, built binary):

9. `--snapshot F` creates `F` with mode `0600`; a second run fails, exit `1`, and leaves `F` unchanged.
10. `--snapshot F` then `--compare F` immediately reports `0 differences` (stable-field claim).
11. `--compare F --format json` is valid JSON with `compare_schema_version: 1`.
12. Every failure row in section 8 exits non-zero with its message.
13. Regression: `--format json` output still has `schema_version: 1` and no snapshot keys
    (the existing check in `scripts/smoke-v0.3.0.py`).

## 11. Open questions for the owner

1. Fixed module set (recommended) or honor `--only` / `--exclude` / config?
2. Compare exit code: always `0` (recommended), or `diff(1)` style?
3. Store `disk.used_bytes` with a change threshold, or leave it out (recommended for v0.6.0)?
4. Include the stable `--hardware-details` parts (display connector and physical size) in
   snapshot schema v1, or postpone them?
5. Include `identity.username` at all? The hostname is useful in a compare; the username rarely is.
6. Is a `kind` field useful next to `snapshot_schema_version`, or redundant?
7. Should compare ignore small `disk.total_bytes` changes (for example under 0.1%), or report every byte?
