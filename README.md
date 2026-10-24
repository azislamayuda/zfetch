# zfetch

**Fast. Lightweight. Rust-powered.**

zfetch is an open-source Linux system information tool written in Rust. It displays operating system, desktop, hardware, disk, network, and package information in a customizable terminal view, or exports selected information as JSON for scripts and other tools.

Designed as a lightweight alternative to Neofetch and Fastfetch, zfetch favors native Linux interfaces, lazy module detection, and optional hardware telemetry.

[Releases](https://github.com/azislamayuda/zfetch/releases) · [Issues](https://github.com/azislamayuda/zfetch/issues) · [CLI Guide](docs/CLI-v0.3.0.md) · [License](LICENSE)

**Current version:** `v0.3.0`  
**Platform:** Linux (`amd64` Debian package available)  
**Language:** Rust, edition 2024  
**License:** MIT  
**Author:** [Azis Lamayuda](https://github.com/azislamayuda)

## What's new in v0.3.0

- **JSON output:** machine-readable system data with `--format json` (schema version 1).
- **CLI module selection:** `--only`, `--exclude`, and `--list-modules`; unselected detectors are skipped.
- **Three terminal layouts:** `compact`, `classic`, and `detailed`.
- **Module presets:** `minimal`, `default`, and `full`.
- **Six ANSI themes:** `default`, `monochrome`, `nord`, `gruvbox`, `rust`, and `high-contrast`.
- **Configuration tools:** `--init-config` and `--validate-config`.
- **Extended hardware information:** CPU topology and frequency, GPU classification and clocks when available, battery estimates, and physical display size via EDID.
- **Quality improvements:** 63 passing Rust unit tests, offline CLI smoke checks, CI workflow, and Debian release checks on the tested development build.

v0.3.0 retains the automatic distribution ASCII logo database and GitHub update checking introduced in v0.2.0. See the [v0.3.0 release notes](docs/RELEASE-NOTES-v0.3.0.md).

## Features

| Category | Information |
| --- | --- |
| System | Distribution, kernel, architecture, shell, desktop environment, session, window manager, terminal, package counts |
| Hardware | Host, BIOS, display, CPU, GPU, memory, swap, disks, battery, uptime |
| Network | Local IPv4 address and interface |
| Packages | DPKG, Flatpak, and AppImage counts where supported |
| Display | ASCII logo, configurable alignment and colors, layouts, themes, presets |
| Automation | JSON output, selected modules, configuration validation |
| Maintenance | Optional GitHub Release checking and interactive Debian package update |

Hardware details are best-effort. Availability depends on the kernel, device, driver, permissions, and desktop environment. Some values may be omitted if they cannot be reliably detected.

## Preview

An example of `zfetch --no-logo` on the development machine:

```text
lamayuda@lamayuda
zfetch v0.3.0

SYSTEM INFO ────────────────────────────
OS          Ubuntu 26.04.1 LTS
Kernel      7.0.0-38-generic
Arch        x86_64
Shell       bash 5.3.9
DE          GNOME 50.1
Session     Wayland
WM          Mutter (Wayland)
Terminal    Ptyxis 50.1
Packages    13 (appimage), 3381 (dpkg), 26 (flatpak)

HARDWARE INFO ────────────────────────────
Host        Dell Inc. Latitude 3320
Display     1920x1080 @ 60 Hz [Built-in] (eDP-1)
CPU         11th Gen Intel(R) Core(TM) i5-1145G7 @ 2.60GHz
GPU         Intel Iris Xe Graphics
Memory      9.07 GiB / 14.86 GiB (61%)
Battery     100% (Full)
```

This is an illustrative excerpt of actual development-machine output. Values change with system state and configuration; additional modules are normally displayed.

For extended hardware information:

```bash
zfetch --layout detailed
```

Example extended fields include `4 cores / 8 threads`, `Kernel Base 1.50 GHz`, `Max 4.40 GHz`, `integrated`, Intel GPU clock frequencies, and a `13.3"` display size on the tested machine. Not all hardware provides these fields.

## Installation

### Debian / Ubuntu (`amd64`)

Download these assets from the [GitHub Releases](https://github.com/azislamayuda/zfetch/releases) page for **v0.3.0**:

- `zfetch_0.3.0-1_amd64.deb`
- `SHA256SUMS`

In the download directory, verify the checksum and install or upgrade:

```bash
sha256sum -c SHA256SUMS
sudo apt install ./zfetch_0.3.0-1_amd64.deb
zfetch --version
zfetch
```

The Debian package installs the executable at `/usr/bin/zfetch`. The v0.3.0 `amd64` package declares dependencies on `apt`, `coreutils`, `curl`, `dpkg`, and `libc6 (>= 2.39)`. It does not require Rust or Cargo at runtime.

The package has been built and its contents, checksum, and simulated upgrade from v0.2.0 validated on Ubuntu 26.04.1 LTS. Wider distribution compatibility and the successful interactive remote-update installation path have not been comprehensively tested.

### Build from source

Install a compatible stable Rust toolchain and Git, then run:

```bash
git clone https://github.com/azislamayuda/zfetch.git
cd zfetch
cargo build --release
./target/release/zfetch
```

For the optimized **Compact** profile used for packaging:

```bash
cargo build --locked --profile compact
./target/compact/zfetch
```

To build a local Debian package (requires [`cargo-deb`](https://github.com/kornelski/cargo-deb)):

```bash
cargo install cargo-deb --locked
cargo deb
```

The resulting `.deb` is written under `target/debian/`. The release preparation script in this repository also verifies package metadata, the packaged binary, license notices, and a SHA-256 checksum:

```bash
bash scripts/build-deb-v0.3.0.sh
```

The script expects `cargo-deb` to be discoverable on `PATH` (commonly `$HOME/.cargo/bin`). It does not install the package or publish a release.

## Command-line usage

### Common commands

```bash
zfetch                                             # Default terminal display
zfetch --no-logo --no-color                        # Plain terminal output
zfetch --layout compact --preset minimal           # Small terminal view
zfetch --layout classic                            # Standard logo + information layout
zfetch --layout detailed                           # Extended hardware details
zfetch --only os,kernel,cpu,memory                 # Detect selected modules only
zfetch --exclude disk,battery                      # Skip selected modules
zfetch --theme nord                                # Apply a theme
zfetch --list-themes                               # Show available ANSI themes
zfetch --list-modules                              # Show module identifiers
zfetch --logo ~/.config/zfetch/my-logo.txt         # Custom ASCII artwork
zfetch --check-update                              # Query latest stable GitHub Release
```

### CLI reference

| Option | Description |
| --- | --- |
| `-h`, `--help` | Display command help |
| `-V`, `--version` | Display zfetch version |
| `--format terminal\|json` | Select terminal output (default) or JSON |
| `--only <MODULES>` | Detect and display only comma-separated module names |
| `--exclude <MODULES>` | Skip comma-separated module names |
| `--list-modules` | Show valid module names |
| `--layout compact\|classic\|detailed` | Select terminal layout (default: classic) |
| `--preset minimal\|default\|full` | Choose a module preset |
| `--theme <NAME>` | Select a built-in ANSI theme |
| `--list-themes` | Show themes with ANSI color previews |
| `--hardware-details` | Enable optional CPU, GPU, display, and battery details |
| `--logo <FILE>` | Use custom ASCII logo file |
| `--no-logo` | Disable ASCII logo |
| `--logo-align top\|center\|bottom` | Align the ASCII logo |
| `--no-color` | Disable ANSI color output |
| `--config <FILE>` | Load a custom JSONC configuration |
| `--init-config` | Create an example configuration without overwriting an existing file |
| `--validate-config` | Validate the default config or one supplied with `--config` |
| `--check-update` | Check the latest stable GitHub Release; no installation |
| `--update` | Check for a compatible update and offer an interactive `.deb` upgrade |

Use `zfetch --help` for the authoritative command syntax of the installed build.

`--only` and `--exclude` cannot be combined. `--logo` and `--no-logo` are mutually exclusive. `--layout compact` and `--layout detailed` are terminal-only and cannot be combined with `--format json`.

### JSON output

```bash
zfetch --format json --only os,kernel,cpu,memory | python3 -m json.tool
zfetch --format json --only cpu,gpu,display,battery --hardware-details
```

Example JSON structure (illustrative values):

```json
{
  "schema_version": 1,
  "zfetch_version": "0.3.0",
  "identity": {
    "username": "user",
    "hostname": "linux-pc"
  },
  "system": {
    "os": "Ubuntu 26.04.1 LTS",
    "kernel": "7.0.0-38-generic"
  },
  "hardware": {
    "cpu": "11th Gen Intel Core i5-1145G7",
    "memory": "8.36 GiB / 14.86 GiB"
  }
}
```

JSON output contains no ASCII logos or terminal color codes. It uses schema version `1`. Optional objects such as `cpu_details`, `gpu_details`, `display_details`, and `battery_details` are included when `--hardware-details` is enabled **and** the underlying system provides the data. Memory is currently a formatted string, while disk entries expose numeric byte counters. Do not rely on missing hardware fields being present.

JSON does not accept presentation-only detailed or compact layouts. For equivalent detail use `--format json --hardware-details` without `--layout detailed`.

## Linux distribution logos

zfetch detects the distribution using `/etc/os-release` and displays a matching built-in ASCII logo when no custom logo is configured. The embedded logo database includes recognizable artwork for popular distributions such as Ubuntu, Debian, Linux Mint, Fedora, Arch Linux, and others. Distribution-family fallback and a generic Linux fallback are available when appropriate.

Logo selection priority:

1. Explicit `--logo <FILE>` on the command line.
2. Custom logo specified in JSONC configuration.
3. Automatically detected distribution logo.
4. Generic Linux fallback.

`--no-logo` disables logos regardless of the selected artwork. Custom ASCII artwork is preserved when upgrading the Debian package because it lives in user configuration, not the packaged binary.

### Custom ASCII logo

```bash
mkdir -p ~/.config/zfetch
cp /path/to/my-logo.txt ~/.config/zfetch/my-logo.txt
zfetch --logo ~/.config/zfetch/my-logo.txt
```

To select a custom logo by default, set `logo.source` in `config.jsonc` as shown below. Omit `source` when you want automatic distro detection.

## JSONC configuration

zfetch reads configuration from `$XDG_CONFIG_HOME/zfetch/config.jsonc`, or `~/.config/zfetch/config.jsonc` when `XDG_CONFIG_HOME` is unset. If no config exists, built-in defaults are used.

Create a default file without overwriting an existing one:

```bash
zfetch --init-config
zfetch --validate-config
```

Example configuration:

```jsonc
{
  "logo": {
    "enabled": true,
    "source": "~/.config/zfetch/my-logo.txt",
    "align": "center"
  },
  "theme": {
    "color": true,
    "logo_color": "red",
    "heading_color": "cyan",
    "label_color": "white",
    "value_color": "gray"
  },
  "display": {
    "show_efi": false,
    "show_percentage": true
  },
  "modules": {
    "system": [
      "os", "kernel", "arch", "shell", "de", "session", "wm", "terminal", "packages"
    ],
    "hardware": [
      "host", "bios", "display", "cpu", "gpu", "memory", "swap",
      "disk", "local_ip", "battery", "uptime"
    ]
  }
}
```

Make sure the custom logo file exists before enabling `logo.source`. For **automatic distro logos**, remove the `"source"` property from the example. Supported color names are `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, and `gray`. Built-in themes use these eight named ANSI colors rather than a truecolor palette.

Configuration module arrays control visibility and ordering. Unknown or duplicate module names are rejected. CLI filters override the in-memory module selection for the current command without modifying the user's config file.

To validate another configuration file:

```bash
zfetch --validate-config --config /path/to/config.jsonc
```

## Update Manager

```bash
zfetch --check-update  # Check only
zfetch --update        # Check and offer update
```

The updater checks **official stable GitHub Releases on demand**. Ordinary `zfetch` runs do **not** contact GitHub or perform network update checks. If a newer compatible Debian package is available, `--update` can offer an interactive installation path, using release checksum verification and Debian package management. It must not be assumed to upgrade arbitrary installation types.

Review the release source, asset architecture, dependency changes, and checksum before accepting an update. A successful remote update of an older published version to v0.3.0 has not yet been validated end-to-end on every supported system.

## Performance

Measurements from the v0.3.0 Compact build on the development machine (Dell Latitude 3320, Ubuntu 26.04.1 LTS, x86_64):

| Metric | v0.3.0 measurement |
| --- | ---: |
| Startup, `--no-logo --no-color` (100 runs) | **8.1 ± 0.9 ms** |
| Startup with `--hardware-details` in a paired run (100 runs) | **10.3 ± 1.2 ms** |
| Default command in that paired run (100 runs) | 10.2 ± 2.8 ms |
| Average maximum resident set size (10 independent runs) | **4,130.8 KiB (~4.03 MiB)** |
| Compact binary size (measured build) | **1,635,920 bytes (~1.56 MiB)** |
| Debian package size (verified build) | **592,268 bytes (~578 KiB)** |

These measurements are environment-specific; the hardware-details benchmark was affected by scheduling variability, and the paired difference was not statistically meaningful. The results should not be compared directly with older releases measured under different conditions or configurations.

To reproduce the benchmark (requires `hyperfine` and GNU `time`):

```bash
cargo build --locked --profile compact
bash scripts/benchmark-v0.3.0.sh ./target/compact/zfetch
```

Benchmark output is stored under `target/benchmarks/`. Optional system detection profiling is also available with:

```bash
ZFETCH_PROFILE=1 ./target/compact/zfetch --no-logo
```

Profiling writes timings to standard error; parallel tasks may overlap.

## Development and testing

Recommended validation steps:

```bash
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --profile compact
python3 scripts/smoke-v0.3.0.py ./target/compact/zfetch
```

The v0.3.0 release-candidate source passed **63 unit tests** and **9 offline smoke-test groups** on the development machine. The repository also includes a GitHub Actions CI workflow to run automated checks; its remote status should be verified separately after pushing changes.

zfetch uses `/proc`, `/sys`, filesystem metadata, Linux interfaces, and selected D-Bus services on supported GNOME desktops. Detection modules can run concurrently; disabled modules are not collected unnecessarily. Optional telemetry is queried only when requested.

Additional resources: [CLI Reference](docs/CLI-v0.3.0.md), [Release Checklist](docs/SPRINT04-RELEASE.md), and [Release Notes](docs/RELEASE-NOTES-v0.3.0.md).

## Compatibility and limitations

- The distributed `.deb` currently targets **amd64** with Debian-compatible package management and `libc6 >= 2.39`.
- Primary testing has been on **Ubuntu 26.04.1 LTS**, GNOME 50.1, Wayland, and Intel Iris Xe graphics; broader Linux support remains a development goal.
- Some desktop, GPU, display, and battery fields depend on driver or sysfs/D-Bus availability and may be absent.
- The terminal layout can become wide when detailed CPU/GPU information is shown. Use compact mode or a wider terminal if necessary.
- JSON schema version 1 is not a full numeric hardware inventory: for example, memory usage remains a formatted string.
- Built-in themes use a limited ANSI named-color palette, not 24-bit RGB truecolor themes.

## Troubleshooting

**`zfetch: command not found`** — Check the installation and path:

```bash
dpkg-query -W zfetch
command -v zfetch
```

**Invalid JSONC configuration** — Validate it before running normal output:

```bash
zfetch --validate-config
```

**JSON parser receives no data** — Do not combine `--format json` with `--layout compact` or `--layout detailed`. Use `--hardware-details` to add hardware data to JSON.

**Missing hardware detail** — Verify that your Linux kernel and drivers expose the requested sysfs or EDID data; unavailable fields are intentionally omitted rather than invented.

**Release update unavailable** — Confirm that a newer stable GitHub Release contains a compatible Debian asset and a valid checksum file. Manually downloading from [Releases](https://github.com/azislamayuda/zfetch/releases) remains an option.

## Uninstallation

```bash
sudo apt remove zfetch
```

User-maintained files in `~/.config/zfetch/` are not removed by this command. They can be retained for later installations.

## Roadmap

Completed through v0.3.0: native system detection, JSONC configuration, automatic distro ASCII logos, custom artwork, interactive update checking, JSON output, CLI module selection, layouts, themes, optional hardware details, and release validation tooling.

Future priorities include broader distro/architecture support, improved terminal-width handling, more structured numeric JSON fields, expanded device testing, and continued performance optimization.

## Contributing

Contributions, compatibility reports, bug fixes, documentation improvements, and hardware detection enhancements are welcome. Please open an [issue](https://github.com/azislamayuda/zfetch/issues) or [pull request](https://github.com/azislamayuda/zfetch/pulls). Before submitting code, run Rustfmt, Clippy, the unit tests, and the smoke tests.

## License and acknowledgments

zfetch is distributed under the [MIT License](LICENSE).

Some built-in Linux distribution ASCII logos are adapted from Fastfetch artwork. Their MIT license and attribution are included in [Third-Party Logo Notices](docs/THIRD_PARTY_LOGOS.md) and [Fastfetch MIT License](docs/licenses/fastfetch-MIT.txt). Distribution identities remain associated with their respective projects.

**Created by [Azis Lamayuda](https://github.com/azislamayuda)**  
**Repository:** https://github.com/azislamayuda/zfetch
