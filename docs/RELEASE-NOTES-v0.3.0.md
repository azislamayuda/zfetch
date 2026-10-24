# zfetch v0.3.0 — Advanced CLI & Hardware Insights

zfetch v0.3.0 expands the Rust-powered Linux system information tool with machine-readable output, targeted module selection, optional hardware details, and customizable terminal layouts.

## What's New

- JSON output with `--format json` and schema version 1.
- CLI module selection through `--only`, `--exclude`, and `--list-modules`.
- Three terminal layouts: `compact`, `classic`, and `detailed`.
- Minimal, default, and full module presets.
- Built-in ANSI color themes with terminal previews.
- JSONC configuration initialization and validation.
- Optional hardware insights for CPU topology and frequencies, GPU classification and clocks when supported, display physical size from EDID, and battery time estimates when possible.
- Preserved custom ASCII logos, automatic distro logos, and GitHub update checking introduced in v0.2.0.
- Additional regression tests and CI validation.

## Installation

Download `zfetch_0.3.0-1_amd64.deb` and `SHA256SUMS` from the release assets.

```bash
sha256sum -c SHA256SUMS
sudo apt install ./zfetch_0.3.0-1_amd64.deb
zfetch --version
```

## Examples

```bash
zfetch
zfetch --layout compact --preset minimal
zfetch --layout detailed --only cpu,gpu,display,battery
zfetch --theme nord
zfetch --format json --only os,kernel,cpu,memory
zfetch --check-update
```

## Compatibility

The provided package targets Debian-compatible `amd64` systems. Availability of hardware metadata depends on kernel, drivers and devices. The development environment is Ubuntu 26.04.1 LTS; other distributions require separate validation.

## Validation

Final benchmark results, tests, installation verification and CI status must be confirmed on the actual v0.3.0 release candidate before publishing. The interactive successful-update path from an older published release is not fully exercised by offline tests.

## License

zfetch is MIT licensed. Bundled third-party ASCII logo artwork retains its applicable MIT attribution files.

Repository: https://github.com/azislamayuda/zfetch
