# zfetch v0.6.0: Release Notes

Status: **Draft for owner review.**

zfetch v0.6.0 adds machine snapshots and comparison, fixes the APT notice during
`--update`, and signs release files. Normal runs are unchanged and remain offline.

## Highlights

- **`zfetch --snapshot <FILE>`**: saves the stable configuration of this machine to a new
  JSON file: OS, kernel, desktop stack, package counts per manager, hardware models, display
  list, and memory, swap, and disk sizes. Values that change from run to run (used memory and
  disk, uptime, local IP, battery level) are not stored. The file is created with mode `0600`
  and is never overwritten. Add `--privacy` before you share it.
- **`zfetch --compare <FILE>`**: shows what changed since a snapshot. Fields are matched by
  name, so reordered disks or displays are not reported as changes. `--format json` gives a
  machine-readable result. The exit status is 0 whether or not there are differences.
- **`--update` APT notice fixed**: the download directory is now readable by APT's `_apt`
  sandbox user, so `N: Download is performed unsandboxed as root` no longer appears. Ctrl-C
  during an update now removes the downloaded files. Every existing check is unchanged.
- **Signed releases**: each of the nine packages gets GitHub build provenance, and
  `SHA256SUMS` gets a keyless Sigstore signature, published as `SHA256SUMS.sigstore.json`.

## Compatibility

- JSON `schema_version` stays `1`, and every existing field and value is unchanged.
- Snapshots use their own key, `snapshot_schema_version: 1`, and have no `schema_version`.
  `--compare` refuses a normal `--format json` report with a clear message.
- Every existing CLI flag keeps its behavior. The new flags are `--snapshot` and `--compare`.
- The `packages` line is byte-identical to v0.5.0. It is now built from per-manager counts.
- The release has one more asset: `SHA256SUMS.sigstore.json`.

## Privacy

`--snapshot --privacy` uses the same redaction as every other output. Comparing against a
private snapshot always redacts the current values as well. Privacy Mode is **not** a
guarantee of anonymity: disk sizes, package counts, and the hardware list together can still
identify a machine. Review a snapshot before you share it.

## Known limitations

- **`gh attestation verify`:** the minimum `gh` version is to be confirmed. Ubuntu's packaged
  `gh` 2.46.0 does not have the `attestation` command.
- **Signer identity:** the cosign certificate identity for the release workflow is to be
  confirmed on the first real run. The verification commands in `docs/CLI-v0.6.0.md` are not
  confirmed until then.
- **`artifact-metadata: write`:** kept in the `attest` job because the `actions/attest` README
  lists it as required. It will be removed if a real run shows it is unused.
- **Release workflow:** the new `attest` job has not run on GitHub yet.
- `--update` does not verify the Sigstore signature in v0.6.0.
- Ctrl-C cleanup covers the downloaded files. A crash, `SIGKILL`, or power loss can still
  leave a `zfetch-update-*` directory in the temporary directory.
- `--compare` reports every byte change in a disk's total size. Resizing a filesystem shows
  up as a change.
