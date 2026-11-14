# zfetch v0.6.0: CLI Reference

Status: **Draft. v0.6.0 is not released yet.**

Everything in the [v0.5.0 CLI reference](CLI-v0.5.0.md) still applies. This page lists
what is new and repeats the tables that changed.

## Snapshots and comparison (new in 0.6.0)

| Command | What it does |
| --- | --- |
| `zfetch --snapshot FILE` | Save this machine's stable configuration to a new JSON file |
| `zfetch --snapshot FILE --privacy` | Same, with identity, addresses, and personal mount paths redacted |
| `zfetch --compare FILE` | Show what changed since the snapshot |
| `zfetch --compare FILE --format json` | Same, as a JSON document |
| `zfetch --compare FILE --no-color` | Terminal diff without colors |
| `zfetch --compare FILE --privacy` | Redact both the snapshot's values and the current values in the output |

**What a snapshot contains:** the OS, kernel, architecture, shell, desktop, session, window
manager, terminal, per-manager package counts, host, BIOS, CPU, GPU, display list, total
memory, total swap, disks (mount point, filesystem, total size), whether a battery is present,
and the username and hostname.

**What it leaves out:** used memory, used swap, used disk space, uptime, local IP address,
battery level and state, the logo, the theme, and your config file. These change from run to
run or are not machine state.

**Fixed module set:** `--snapshot` and `--compare` always use the same modules. They ignore
`--only`, `--exclude`, presets, and the module lists in your config, so two snapshots of the
same machine always cover the same fields.

**Writing:** `--snapshot` creates a new file with mode `0600`. It refuses to overwrite an
existing file or follow an existing symlink, and exits with status 1 if the file exists.

**Reading:** `--compare` refuses symlinks, non-regular files (FIFOs, devices, directories),
files over 1 MiB, invalid JSON, a normal `--format json` report, and snapshots from a newer
snapshot schema. Each case prints a message and exits with status 1.

**Output:**

```text
Comparing with before.json (zfetch 0.6.0, created 3 days ago)

changed  system.kernel                      7.0.0-40-generic -> 7.0.0-43-generic
changed  system.packages.dpkg               3825 -> 3831 (+6)
added    hardware.disk[/data]               ext4, 931.51 GiB
removed  system.packages.snap               12

4 differences, 21 fields unchanged, 0 skipped.
```

Fields are matched by name (disks by mount point, displays by their full line), never by
position. Redacted fields are reported as `skipped`, never as `changed`. If the snapshot was
made with `--privacy`, the current values are redacted too, even without `--privacy`.

**Exit status:** `0` whether or not there are differences. `1` means the file could not be
used, and `2` means the command line was wrong. Scripts should read the `changed`, `added`,
and `removed` arrays of `--format json`.

### Allowed combinations

| | `--privacy` | `--format` | `--no-color` | `--config` | other display options |
| --- | --- | --- | --- | --- | --- |
| `--snapshot FILE` | yes | no | no | no | no |
| `--compare FILE` | yes | yes | yes | no | no |

`--snapshot` and `--compare` are action flags. They cannot be combined with each other or
with any other action flag.

### Document versions

| Document | Version key | Value |
| --- | --- | --- |
| `--format json` report | `schema_version` | `1` (unchanged) |
| Snapshot file | `snapshot_schema_version` | `1` |
| `--compare --format json` | `compare_schema_version` | `1` |

A snapshot has no `schema_version` key, so it cannot be mistaken for a report. New fields can
be added to snapshot schema 1. Fields this version does not know are counted as
"not compared" and are not reported as removed.

## Update Manager (changed in 0.6.0)

`--update` on a `.deb` install now downloads into a directory with mode `0711` and makes the
package file `0644`, so APT's `_apt` sandbox user can read it. The
`N: Download is performed unsandboxed as root` notice no longer appears. All checks are
unchanged: HTTPS-only download, size, SHA-256 against `SHA256SUMS`, package name, version,
architecture, and the `[y/N]` confirmation. Ctrl-C during an update removes the downloaded
files and exits with status 130.

`--update` does not verify the Sigstore signature in v0.6.0.

## Verifying a release

**These commands are not confirmed yet.** They will be checked against the first signed
release, and this section will be updated.

Build provenance for a package (needs a `gh` version with the `attestation` command; the
minimum version is to be confirmed, and Ubuntu's `gh` 2.46.0 does not have it):

```bash
gh attestation verify zfetch_0.6.0-1_amd64.deb --repo azislamayuda/zfetch \
  --signer-workflow azislamayuda/zfetch/.github/workflows/release.yml
```

Signature on `SHA256SUMS` (needs `cosign`). The certificate identity below is to be
confirmed on the first real run:

```bash
cosign verify-blob SHA256SUMS --bundle SHA256SUMS.sigstore.json \
  --certificate-identity https://github.com/azislamayuda/zfetch/.github/workflows/release.yml@refs/heads/main \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
sha256sum -c --ignore-missing SHA256SUMS
```
