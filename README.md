# zfetch

**Fast. Lightweight. Rust-powered. Built for Linux.**

zfetch is an open-source system information CLI written in Rust. It shows your Linux distribution, kernel, desktop environment, hardware, memory, disks, network, battery, and installed package counts in a fast, customizable terminal view. It can also produce machine-readable JSON and export system reports.

Designed as a lightweight alternative to Neofetch and Fastfetch, zfetch uses native Linux interfaces, selective detection, and an embedded database of distribution ASCII logos.

[Releases](https://github.com/azislamayuda/zfetch/releases) · [Issues](https://github.com/azislamayuda/zfetch/issues) · [Documentation](docs/) · [License](LICENSE)

| Project | Details |
| --- | --- |
| Current source version | **v0.6.0** (draft, not released) |
| Platform | **Linux** — native packages and portable builds |
| CPU architectures | **x86_64 / amd64** and **aarch64 / arm64**, depending on format |
| Language | **Rust (edition 2024)** |
| License | **MIT** |
| Author | [Azis Lamayuda](https://github.com/azislamayuda) |

## What's new in v0.6.0 (draft)

- **`zfetch --snapshot <FILE>`:** save the stable configuration of this machine (OS, kernel, desktop, package counts, hardware models, capacities) to a new JSON file with `0600` permissions. Values that change every run (used memory and disk, uptime, local IP, battery level) are not stored. Combine with `--privacy` for a shareable snapshot.
- **`zfetch --compare <FILE>`:** show what changed since a snapshot: changed, added, and removed fields, matched by name. Add `--format json` for a machine-readable diff. The exit code is 0 whether or not there are differences.
- **`--update` APT notice fixed:** the download directory can now be read by APT's `_apt` sandbox user, so the "Download is performed unsandboxed as root" notice no longer appears. Ctrl-C during an update now removes the downloaded files.
- **Signed releases:** release packages get GitHub build provenance, and `SHA256SUMS` gets a keyless Sigstore signature (`SHA256SUMS.sigstore.json`).

## What's new in v0.5.0

- **`zfetch --doctor`:** read-only checklist (OK / WARN / INFO) for config, logo, os-release, `/proc` and `/sys` access, package databases, desktop session variables, and optional `gdctl` / `curl`.
- **Privacy Mode 2.0:** `--privacy` now also hides MAC and IPv6 addresses, DMI serial numbers, and the username/hostname wherever they appear — in terminal output, JSON, `--export`, and `--doctor`. It is still not a guarantee of anonymity.
- **Universal update manager:** `--check-update` reports how zfetch was installed (`.deb`, Arch package, RPM, or portable/source), detected from files only. `--update` installs only verified `.deb` updates after confirmation; other methods get safe manual commands and nothing is changed.
- **More package managers:** Alpine APK, Gentoo Portage, and Void XBPS, read directly from their databases (no subprocesses).
- **Window manager detection:** Hyprland, Sway, niri, and i3 via their IPC socket variables.
- **`zfetch --benchmark`:** per-module detection time table.
- **Smaller and lighter:** the `zbus` D-Bus dependency is gone. Displays are read from DRM/EDID in sysfs and the GNOME version from the package database: ~53% smaller binary, ~18% less peak RSS, no D-Bus connections on normal runs.

## What's new in v0.4.0

- **Expanded Linux packaging:** Debian/Ubuntu `.deb`, RPM-family `.rpm`, Arch Linux `.pkg.tar.zst`, and portable `.tar.gz` builds.
- **Multi-architecture releases:** amd64/x86_64 and arm64/aarch64 where supported by the package format.
- **Statically linked musl binaries:** Portable builds that do not depend on the host GLIBC version.
- **Wider GLIBC compatibility:** Debian builds target **GLIBC 2.35**; RPM and GNU portable builds target **GLIBC 2.34**.
- **Privacy Mode:** `--privacy` hides selected identity, network, and personal mount information.
- **Safe report export:** `--export` writes terminal or JSON reports with private file permissions and refuses to overwrite existing files.
- **Better hardware and desktop detection:** Improvements to CPU, GPU, ARM64, and desktop-environment fallbacks.
- **Reliable builds:** 80 passing Rust unit tests, offline CLI regression checks, and automated multi-architecture packaging pipelines.

## Features

| Area | Capabilities |
| --- | --- |
| System | Distribution, kernel, architecture, shell, DE, session, WM, terminal, packages |
| Hardware | Host, BIOS, display, CPU, GPU, RAM, swap, disks, battery, uptime |
| Network | Local IPv4 address and interface |
| Package counts | DPKG, RPM, Pacman, APK, Portage, XBPS, Flatpak, Snap, and AppImage where detectable |
| Terminal | Automatic distro ASCII logo, custom logo, ANSI colors, six themes, three layouts |
| Customization | JSONC configuration, module filtering, module presets, hardware details |
| Automation | Machine-readable JSON, private text/JSON report export |
| Maintenance | On-demand GitHub release checking; interactive `.deb` updates; `--doctor` diagnostics; `--benchmark` timings |

Hardware and desktop fields are detected on a best-effort basis. Some values require support from the kernel, drivers, desktop services, or package database.

## Linux support and downloads

**zfetch is designed for a broad range of Linux distributions, not just Debian.** Choose the native package when one is provided for your distribution, or use a portable build with a compatible Linux kernel and CPU architecture.

| Linux environment | Release format | Architectures | Notes |
| --- | --- | --- | --- |
| Debian, Ubuntu, Linux Mint, Pop!_OS, and derivatives | `.deb` | amd64, arm64 | Built on Ubuntu 22.04; GLIBC 2.35 target |
| Rocky Linux 9, AlmaLinux 9, Fedora, and compatible RPM systems | `.rpm` | x86_64, aarch64 | Built and smoke-tested on Rocky Linux 9; GLIBC 2.34 target |
| Arch Linux, Manjaro, EndeavourOS, and derivatives | `.pkg.tar.zst` | x86_64 | Built using Arch Linux tooling |
| Other compatible GNU/Linux distributions | GNU `.tar.gz` | x86_64, aarch64 | Requires compatible GLIBC (target 2.34) and shared libraries |
| Other Linux distributions, including musl-based systems | Static musl `.tar.gz` | x86_64, aarch64 | No host GLIBC requirement; depends on compatible kernel/architecture |

**Important:** A portable binary is not a guarantee of compatibility with every distribution, kernel, hardware platform, or security policy. The CI matrix validates builds and selected smoke tests; not every Linux distribution has been tested individually. Users of openSUSE, Gentoo, Void, Alpine, NixOS, and other distributions can try the appropriate portable build or compile from source.

### Release assets — v0.4.0

Download from [GitHub Releases](https://github.com/azislamayuda/zfetch/releases) after the release is published.

| Asset | Target |
| --- | --- |
| `zfetch_0.4.0-1_amd64.deb` | Debian/Ubuntu x86_64 |
| `zfetch_0.4.0-1_arm64.deb` | Debian/Ubuntu ARM64 |
| `zfetch-0.4.0-1.el9.x86_64.rpm` | RPM-based Linux x86_64 |
| `zfetch-0.4.0-1.el9.aarch64.rpm` | RPM-based Linux ARM64 |
| `zfetch-0.4.0-1-x86_64.pkg.tar.zst` | Arch Linux x86_64 |
| `zfetch-0.4.0-linux-x86_64-gnu.tar.gz` | GNU/Linux portable x86_64 |
| `zfetch-0.4.0-linux-aarch64-gnu.tar.gz` | GNU/Linux portable ARM64 |
| `zfetch-0.4.0-linux-x86_64-musl.tar.gz` | Static portable x86_64 |
| `zfetch-0.4.0-linux-aarch64-musl.tar.gz` | Static portable ARM64 |

The release also includes `SHA256SUMS` to verify downloaded files. On systems with GNU `sha256sum`, run this in the directory containing the checksum file and downloaded package(s):

```bash
sha256sum -c --ignore-missing SHA256SUMS
```

This verifies matching local files against the published checksums; verify that the expected package actually reports `OK`. SHA-256 checksums alone do not replace publisher-signature verification.

From v0.6.0, releases also carry build provenance and a signature on `SHA256SUMS`. The exact verification commands are in [docs/CLI-v0.6.0.md](docs/CLI-v0.6.0.md#verifying-a-release); they are not confirmed until the first signed release has been published.

## Installation

### Debian, Ubuntu, and derivatives

```bash
sudo apt install ./zfetch_0.4.0-1_amd64.deb
zfetch --version
```

For an ARM64 system, use `zfetch_0.4.0-1_arm64.deb` instead. APT checks the available runtime dependencies before installation.

### Fedora, Rocky Linux, and RPM-based systems

```bash
sudo dnf install ./zfetch-0.4.0-1.el9.x86_64.rpm
zfetch --version
```

Use the `aarch64` RPM on ARM64 and your distribution's supported RPM package manager. These RPMs are built against Rocky Linux 9, so compatibility with other RPM distributions should be checked rather than assumed.

### Arch Linux and derivatives

```bash
sudo pacman -U ./zfetch-0.4.0-1-x86_64.pkg.tar.zst
zfetch --version
```

A native Arch package is currently provided for x86_64 only.

### Portable static musl (recommended for other distributions)

The portable build can be executed without installing a system package:

```bash
tar -xzf zfetch-0.4.0-linux-x86_64-musl.tar.gz
./zfetch-0.4.0-linux-x86_64-musl/zfetch --version
./zfetch-0.4.0-linux-x86_64-musl/zfetch
```

Choose `aarch64-musl` for ARM64. The GNU portable archives can be used in the same way, but require compatible GLIBC and shared libraries. Portable archives include `zfetch`, `README.md`, and `LICENSE`.

### Build from source

Install a recent stable Rust toolchain (edition 2024 support) and Git:

```bash
git clone https://github.com/azislamayuda/zfetch.git
cd zfetch
cargo build --locked --profile compact
./target/compact/zfetch --version
./target/compact/zfetch
```

For a conventional release build, use `cargo build --release` and run `./target/release/zfetch`.

## Usage

```bash
zfetch                                       # Default terminal view
zfetch --no-logo --no-color                  # Plain output
zfetch --layout compact --preset minimal     # Minimal layout
zfetch --layout classic                      # Classic layout
zfetch --layout detailed --hardware-details  # Extended hardware details
zfetch --only os,kernel,cpu,memory           # Select modules
zfetch --exclude disk,battery                # Skip modules
zfetch --theme nord                          # Choose an ANSI theme
zfetch --list-themes                         # List available themes
zfetch --list-modules                        # List modules
zfetch --format json                         # Machine-readable JSON
zfetch --privacy                             # Redact selected personal information
zfetch --privacy --export report.txt         # Private text report
zfetch --privacy --format json --export report.json
zfetch --check-update                        # Check GitHub for a newer release
zfetch --update                              # Verified .deb update, or manual steps for other installs
zfetch --doctor                              # Read-only diagnostics
zfetch --doctor --privacy                    # Diagnostics safe to paste into an issue
zfetch --benchmark                           # Per-module detection time
zfetch --snapshot before.json                # Save a machine snapshot (no overwrite)
zfetch --compare before.json                 # Show what changed since the snapshot
zfetch --compare before.json --format json   # Same, as JSON
```

### Common CLI options

| Option | Description |
| --- | --- |
| `-h`, `--help` | Display help |
| `-V`, `--version` | Display version |
| `--format terminal\|json` | Choose terminal or JSON output |
| `--only <MODULES>` / `--exclude <MODULES>` | Select or skip comma-separated modules |
| `--list-modules` | List supported module names |
| `--layout compact\|classic\|detailed` | Select terminal layout |
| `--preset minimal\|default\|full` | Select module preset |
| `--theme <NAME>` / `--list-themes` | Apply/list ANSI themes |
| `--hardware-details` | Request extended hardware information |
| `--logo <FILE>` / `--no-logo` | Custom ASCII logo or no logo |
| `--logo-align top\|center\|bottom` | ASCII artwork alignment |
| `--no-color` | Disable terminal colors |
| `--config <FILE>` | Read a custom JSONC config |
| `--init-config` / `--validate-config` | Create/validate configuration |
| `--privacy` | Redact identity, IP/MAC addresses, serials, and personal mount paths |
| `--export <FILE>` | Export to a new private file (no overwrite) |
| `--check-update` / `--update` | On-demand release check / verified `.deb` update (manual steps elsewhere) |
| `--doctor` | Read-only diagnostics; combine with `--privacy` and `--config` |
| `--benchmark` | Per-module detection time table |
| `--snapshot <FILE>` | Save a new machine snapshot (JSON, `0600`, no overwrite); accepts `--privacy` |
| `--compare <FILE>` | Compare with a snapshot; accepts `--format json`, `--no-color`, `--privacy` |

Use `zfetch --help` for authoritative syntax. Some combinations are intentionally invalid, including `--only` with `--exclude`, `--logo` with `--no-logo`, and JSON output with presentation-only compact or detailed layouts.

## JSON output and report export

```bash
zfetch --format json --only os,kernel,cpu,memory | python3 -m json.tool
zfetch --format json --hardware-details
zfetch --privacy --format json --export my-system-report.json
```

Example JSON excerpt:

```json
{
  "schema_version": 1,
  "zfetch_version": "0.5.0",
  "identity": {
    "username": "hidden",
    "hostname": "hidden"
  },
  "system": {
    "os": "Linux distribution",
    "kernel": "Linux kernel version"
  }
}
```

The example is illustrative. JSON schema version 1 includes more fields, and optional hardware telemetry depends on hardware availability. For private reports, prefer `--privacy --export` rather than redirecting standard output: the built-in exporter creates a new file with `0600` permissions and refuses to overwrite an existing file or symlink. Privacy Mode redacts selected values; review reports before sharing because it is not a comprehensive anonymization guarantee.

## Automatic distro logos and customization

zfetch reads `/etc/os-release` and selects an embedded ASCII logo using the distribution ID, distribution-family fallback, or a generic Linux fallback. The logo database is compiled into the Rust binary, so normal startup does not depend on external logo files.

Selection priority: explicit `--logo` option, custom logo from JSONC, automatically detected distribution logo, then the generic fallback. `--no-logo` disables artwork entirely.

Your existing custom ASCII artwork can be kept under `~/.config/zfetch/`:

```bash
mkdir -p ~/.config/zfetch
cp /path/to/my-logo.txt ~/.config/zfetch/my-logo.txt
zfetch --logo ~/.config/zfetch/my-logo.txt
```

zfetch reads `$XDG_CONFIG_HOME/zfetch/config.jsonc`, or `~/.config/zfetch/config.jsonc` when `XDG_CONFIG_HOME` is unset. Generate or validate the configuration:

```bash
zfetch --init-config
zfetch --validate-config
```

Example JSONC settings:

```jsonc
{
  "logo": {
    "enabled": true,
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
  }
}
```

Without `logo.source`, distro auto-detection remains active. To use your own logo by default, add `"source": "~/.config/zfetch/my-logo.txt"` inside the `logo` object. Config modules also support choosing and ordering detected fields.

Six built-in ANSI themes: `default`, `monochrome`, `nord`, `gruvbox`, `rust`, and `high-contrast`. Layouts: `compact`, `classic`, and `detailed`.

## Updates

```bash
zfetch --check-update
zfetch --update
```

Release checks contact the official GitHub Releases API **only on request**. Normal `zfetch` runs do not perform update-network requests. `--check-update` shows the detected installation method (`.deb`, Arch package, RPM, or portable/source), read from package-manager files without running any command. The interactive `--update` installer handles only **Debian `.deb` installations**, with explicit confirmation plus SHA-256, package name, version, and architecture verification. For Arch, RPM, and portable installs it prints the safe manual command (`pacman -U`, `dnf install`, or checksum-verified download) and changes nothing. zfetch never overwrites files owned by a package manager.

## Build, tests, and performance

Run development checks from the repository root:

```bash
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --profile compact
./target/compact/zfetch --version
```

The v0.6.0 development build passes **109 Rust unit tests and 4 integration tests** and the offline CLI smoke checks. The Release workflow with the new `attest` job has not run on GitHub yet. Performance depends on system load, enabled modules, kernel interfaces, and configuration. Prefer paired, repeatable benchmarks when comparing against other tools.

## Compatibility notes

- **Debian `.deb`:** built against Ubuntu 22.04 with GLIBC 2.35 target. The package additionally declares runtime dependencies through Debian packaging metadata.
- **RPM and GNU portable:** built against Rocky Linux 9 with GLIBC 2.34 target. GNU portable builds dynamically link to system libraries.
- **Static musl:** no dynamic interpreter or required shared libraries in CI checks; a compatible CPU architecture and Linux kernel are still required.
- **Arch package:** built for x86_64 using Arch Linux packaging tools. Package availability does not automatically imply support for every Arch derivative or older system snapshot.
- **Runtime detection:** filesystem, `/proc`, `/sys`, drivers, and package databases differ by distro; unavailable fields may be omitted. Displays come from DRM/EDID (preferred mode); `gdctl` is used only when no EDID is readable.
- **Nix:** package counting is not supported. Profile generations do not map reliably to installed package counts.
- **No AppImage release:** zfetch is a CLI tool. It can count existing AppImage files on supported layouts, but AppImage is not a distribution format provided by this release.

## Troubleshooting and removal

Check the installed executable and configuration:

```bash
command -v zfetch
zfetch --version
zfetch --validate-config
```

If a package does not install, inspect architecture and runtime dependencies rather than forcing installation. If hardware information is missing, verify kernel and driver interfaces. If the update manager cannot find a compatible package, download the matching release asset manually.

Remove installations using the package manager that installed them:

```bash
sudo apt remove zfetch     # Debian and Ubuntu
sudo dnf remove zfetch     # RPM distributions using DNF
sudo pacman -R zfetch      # Arch Linux
```

For a portable installation, remove only the directory or binary you extracted. User-maintained configuration under `~/.config/zfetch/` is preserved unless you delete it yourself.

## Contributing

Bug reports, Linux distribution compatibility results, hardware detection improvements, documentation updates, and pull requests are welcome. Please run Rustfmt, Clippy, and the unit tests before submitting changes.

[Open an issue](https://github.com/azislamayuda/zfetch/issues) · [Pull requests](https://github.com/azislamayuda/zfetch/pulls)

## License and acknowledgments

zfetch is distributed under the [MIT License](LICENSE). Some embedded distribution ASCII logos are adapted from Fastfetch artwork; notices and attribution are included in [Third-Party Logo Notices](docs/THIRD_PARTY_LOGOS.md) and the [Fastfetch MIT License](docs/licenses/fastfetch-MIT.txt). Distribution identities remain associated with their respective projects.

**Created by [Azis Lamayuda](https://github.com/azislamayuda)** · [GitHub repository](https://github.com/azislamayuda/zfetch)
