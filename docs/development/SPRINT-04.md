# zfetch v0.4.0 — Sprint 04: Privacy Mode & Report Export

Development-only feature, while Cargo package version remains 0.3.0 until the final release sprint.

## CLI additions

- `zfetch --privacy`: show a redacted terminal report (username, hostname, local IP, personal mount points).
- `zfetch --privacy --format json`: redact both established v0.3 JSON fields and known sensitive fields in optional telemetry.
- `zfetch --export report.txt`: export *plain-text classic-layout* report (ANSI-free; no ASCII logo). Uses selected modules and existing display configuration.
- `zfetch --format json --export report.json`: export v0.3 structured JSON.
- `zfetch --privacy --format json --export public.json`: write a redacted structured report.
- Existing `--only`, `--exclude`, `--preset`, `--theme`, `--layout`, and `--hardware-details` remain available; the plain-text export uses the classic data layout regardless of terminal layout preference.

## Security boundaries

- **Privacy Mode is optional.** Normal display and export retain the original system fields. Always use `--privacy` when posting reports publicly.
- Known privacy-sensitive fields are replaced before rendering, and an additional JSON pass handles keys such as `username`, `hostname`, `local_ip`, `ssid`, `mac_address`, and `serial_number`.
- Mount points under `/home/`, `/media/`, `/run/media/`, and `/mnt/` are hidden in privacy mode. Root and system mounts remain visible.
- This is a privacy convenience, **not a comprehensive anonymizer**. Rare detector strings, custom logos, hardware serials embedded under unexpected keys, or unique machine data may still identify a system. Inspect output before publishing.
- Export only creates new files (`create_new`), refuses to overwrite existing paths/symlinks, sets `0600` permissions, and writes data locally with no network traffic.
- Errors return exit code 1. Failed writes attempt to remove incomplete output.
- `--export` without `--privacy` may store username, hostname, IP, and personal paths: use it only for private diagnostic reports.

## Local QA checklist

```bash
cargo fmt --all
cargo fmt --all --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --profile compact
./target/compact/zfetch --help
./target/compact/zfetch --privacy --no-logo
./target/compact/zfetch --privacy --format json
./target/compact/zfetch --privacy --export /tmp/zfetch-sprint04.txt
./target/compact/zfetch --privacy --format json --export /tmp/zfetch-sprint04.json
stat -c '%a %n' /tmp/zfetch-sprint04.txt /tmp/zfetch-sprint04.json
python3 -m json.tool /tmp/zfetch-sprint04.json >/dev/null
```

Use **unique export filenames** or remove the test files between runs: overwrites are rejected intentionally. Validate that the output no longer contains your actual username, hostname, local IP, or personal mount paths. Test CLI compatibility with `--only os,cpu`, `--layout compact` for terminal rendering, and `--format json --hardware-details`.

## Packaging and release

- No Cargo version bump yet.
- No `.deb` installed or Git tags created.
- No commit or push from the application script.
- Review changes and commit using GitHub Desktop:
  `feat(privacy): add redacted output and safe report exports`
- The GitHub Actions ARM64 packaging workflow from Sprint 03 still needs independent verification.
