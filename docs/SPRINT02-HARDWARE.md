# zfetch 0.3.0 — Sprint 02: Optional Hardware Information v2

This source patch adds **opt-in** hardware telemetry to the existing `--hardware-details` option introduced in Sprint 01. It does not change the app version, the updater, ASCII logo database, or the JSONC configuration format.

## Scope

- CPU: read `base_frequency` and `cpuinfo_max_freq` from Linux cpufreq when available, rendered as Base / Max GHz. The existing Sprint 01 core/thread detection remains unchanged.
- GPU: conservative integrated/discrete labels for recognizable GPUs. Read **Intel DRM** current/max GT clocks only if there is exactly one Intel DRM card and valid sysfs clock data. Skip unspecified GPUs and unknown clocks.
- Battery: estimate remaining time (discharging) or time until full (charging) from `energy_now/energy_full/power_now` or the matching `charge_*/current_now` unit family. Skip zero, malformed, ambiguous, excessive, or unavailable values. This is an estimate, not guaranteed battery life.
- Displays: decode physical dimensions from EDID detailed timing descriptors (millimeters), with a fallback to EDID centimeter fields. Show a diagonal in inches for connectors already detected by zfetch; skip missing or ambiguous connectors.
- JSON: when `--format json --hardware-details` is used, add available structured `cpu_details`, `gpu_details`, `battery_details`, `display_details` fields within `hardware` without changing `schema_version: 1` or removing existing strings.

## Performance and safeguards

- No new crates and no subprocesses.
- Normal zfetch invocation remains unchanged and performs **no** Sprint 02 extra detection.
- Only selected modules are read (`--only`, `--exclude` and JSONC module selection are respected).
- Missing kernel/driver support is handled by omission rather than fabricated values.
- No new network calls; updater behavior is untouched.

## Validate

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --profile compact
./target/compact/zfetch --no-logo --hardware-details
./target/compact/zfetch --format json --only cpu,gpu,battery,display --hardware-details | python3 -m json.tool
```

**Note:** EDID sizes, GPU clocks, and battery ETA may not appear if drivers or device data are missing. CPU maximum frequency exposed by cpufreq is a driver-reported maximum, not necessarily the current boost clock. For fair benchmarks compare `--hardware-details` to itself on two versions, and compare ordinary zfetch to ordinary zfetch.

## Source changes

- New: `src/hardware_ext.rs` (pure parsers, sysfs readers and unit tests).
- Modified: `src/main.rs` only, to call `hardware_ext::enrich(...)` after the Sprint 01 topology enrichment and augment structured JSON fields before serialization.
- New: this document.

## Commit workflow

After passing all checks, use GitHub Desktop: Changes -> Commit to main -> Push origin.

Suggested commit: `feat: enrich CPU GPU battery and display details (v0.3.0 sprint 02)`

Do not publish v0.3.0 yet; the planned display-layout, quality, and release validation sprints remain.
