# zfetch

**Fast. Lightweight. Rust-powered.**

A fast, lightweight, and customizable Linux system information tool written in Rust.

zfetch is an open-source command-line application that displays essential system and hardware information with minimal overhead. Built as an alternative to Fastfetch, zfetch focuses on native Linux detection, startup performance, efficient memory usage, and a customizable terminal experience.

**Version:** 0.1.0  
**Platform:** Linux  
**Language:** Rust (Edition 2024)  
**License:** MIT  
**Author:** Azis Lamayuda

---

## Overview

zfetch provides detailed information about your Linux system in a clean terminal layout, with optional ASCII artwork, configurable colors, and customizable information modules.

Designed with speed and simplicity in mind, zfetch uses native Linux system interfaces wherever practical and avoids unnecessary subprocesses.

### Highlights

- **Fast startup** — approximately 4.6 ms with the full default configuration on the development machine.
- **Lightweight** — approximately 3.6 MiB of peak memory usage.
- **Small executable** — approximately 1.52 MB with the Compact build profile.
- **Native Rust** — built using Rust without Electron, Python, or Node.js runtimes.
- **Customizable** — configure ASCII logos, colors, module visibility, and ordering.
- **JSONC configuration** — supports comments and human-readable configuration.
- **Lazy detection** — disabled modules are not unnecessarily detected.
- **Parallel detection** — selected system modules run concurrently.
- **Linux integration** — supports Linux hardware information and desktop environment detection.

---

## Features

### System Information

| Module | Description |
|---|---|
| OS | Linux distribution and version |
| Kernel | Running Linux kernel version |
| Architecture | System architecture |
| Shell | Shell name and version |
| Desktop Environment | Desktop environment and version |
| Session | Display session type |
| Window Manager | Window manager |
| Terminal | Terminal emulator and version |
| Packages | Installed package counts |

### Hardware Information

| Module | Description |
|---|---|
| Host | Manufacturer and device model |
| BIOS | BIOS vendor, version, and release date |
| Display | Active display resolution, refresh rate, and connector |
| CPU | Processor model |
| GPU | Graphics processor |
| Memory | Used and total system memory |
| Swap | Used and total swap memory |
| Disk | Filesystem usage and mount points |
| Local IP | Local IPv4 address and network interface |
| Battery | Battery percentage and charging status |
| Uptime | System uptime |

### Package Detection

On supported Linux environments, zfetch can report installed package counts from:

- DPKG
- Flatpak
- AppImage

Detection availability depends on the distribution and installed package systems.

### Desktop Integration

zfetch supports desktop and display information detection through Linux system interfaces.

On supported GNOME environments, it uses D-Bus for desktop version and display information, with fallback mechanisms where available.

---

## Preview

Example output using `zfetch --no-logo`:

```text
lamayuda@lamayuda
zfetch v0.1.0

SYSTEM INFO ────────────────────────────
OS          Ubuntu 26.04.1 LTS
Kernel      7.0.0-38-generic
Arch        x86_64
Shell       bash 5.3.9
DE          GNOME 50.1
Session     Wayland
WM          Mutter (Wayland)
Terminal    Ptyxis 50.1
Packages    13 (appimage), 3387 (dpkg), 26 (flatpak)

HARDWARE INFO ────────────────────────────
Host        Dell Inc. Latitude 3320
BIOS        Dell Inc. 1.46.0 (08/25/2026)
Display     1920x1080 @ 60 Hz [Built-in] (eDP-1)
CPU         11th Gen Intel(R) Core(TM) i5-1145G7 @ 2.60GHz
GPU         Intel Iris Xe Graphics
Memory      8.58 GiB / 14.86 GiB (58%)
Swap        0.00 GiB / 15.63 GiB (0%)
Local IP    192.168.1.10/24 (wlp0s20f3)
Battery     79% (Discharging)
Uptime      0h 29m
```

This is an example from the development machine. Actual output varies according to hardware, system configuration, enabled modules, and operating environment.

By default, zfetch can display an ASCII logo alongside system information.

---

## Performance

zfetch is developed with a focus on low startup latency, efficient memory usage, and accurate system information.

### Development Benchmarks

| Metric | Measured Result |
|---|---|
| Full startup | ~4.6 ms |
| Minimal startup | ~0.65 ms |
| Peak memory usage | ~3.6 MiB |
| Compact binary size | ~1.52 MB |
| Debian package size | ~533 KiB |

### Benchmark Environment

| Component | Specification |
|---|---|
| Device | Dell Latitude 3320 |
| Processor | Intel Core i5-1145G7 |
| Operating System | Ubuntu 26.04.1 LTS |
| Desktop | GNOME 50.1 |
| Display Session | Wayland |
| Architecture | x86_64 |
| Build Profile | Compact |

Startup measurements were obtained with Hyperfine using repeated runs.

Example benchmark command:

```bash
hyperfine --shell=none --warmup 20 --runs 100 \
  './target/compact/zfetch --no-color --no-logo'
```

**Note:** Benchmark results are specific to the tested environment. Performance may vary depending on enabled modules, system load, hardware, storage, Linux distribution, and desktop environment.

The minimal configuration uses fewer detection modules than the full configuration and therefore is not a like-for-like comparison of feature-complete execution.

---

## Installation

### Option 1 — Debian / Ubuntu Package

The initial Debian package is available in the project's release build.

When the GitHub Release is published, download the appropriate `.deb` asset from:

**[GitHub Releases](https://github.com/azislamayuda/zfetch/releases)**

Package filename:

```text
zfetch_0.1.0-1_amd64.deb
```

Install using APT:

```bash
sudo apt install ./zfetch_0.1.0-1_amd64.deb
```

Verify the installation:

```bash
zfetch --version
```

Run:

```bash
zfetch
```

The installed executable is located at:

```text
/usr/bin/zfetch
```

#### Package Requirements

| Requirement | Description |
|---|---|
| OS | Compatible Linux distribution using Debian packages |
| Architecture | amd64 / x86_64 |
| libc6 | Version 2.39 or newer |
| Rust | Not required for prebuilt installation |
| Cargo | Not required for prebuilt installation |

The first `.deb` package has been built and successfully installed on Ubuntu 26.04.1 LTS.

Compatibility with other distributions and releases has not yet been comprehensively validated.

### Option 2 — Build from Source

Install the Rust stable toolchain and Git.

Clone the repository:

```bash
git clone https://github.com/azislamayuda/zfetch.git
cd zfetch
```

Build the standard release:

```bash
cargo build --release
```

Run:

```bash
./target/release/zfetch
```

### Build the Compact Version

For a smaller optimized executable:

```bash
cargo build --profile compact
```

Run:

```bash
./target/compact/zfetch
```

The Compact profile uses size-oriented optimizations and fat Link Time Optimization (LTO).

### Build the Debian Package Yourself

Install `cargo-deb`:

```bash
cargo install cargo-deb --locked
```

Build the package:

```bash
cargo deb
```

The generated package is placed in:

```text
target/debian/
```

For version 0.1.0, the expected filename is:

```text
zfetch_0.1.0-1_amd64.deb
```

The Debian packaging configuration uses the Compact build profile.

---

## Usage

Run with the default configuration:

```bash
zfetch
```

Display information without a logo:

```bash
zfetch --no-logo
```

Disable ANSI colors:

```bash
zfetch --no-color
```

Use a custom ASCII logo:

```bash
zfetch --logo /path/to/logo.txt
```

Change logo alignment:

```bash
zfetch --logo-align top
zfetch --logo-align center
zfetch --logo-align bottom
```

Load a custom configuration:

```bash
zfetch --config /path/to/config.jsonc
```

Display available options:

```bash
zfetch --help
```

Show the application version:

```bash
zfetch --version
```

### Command-Line Options

| Option | Description |
|---|---|
| `-h`, `--help` | Display help |
| `-V`, `--version` | Display version |
| `--no-color` | Disable ANSI colors |
| `--no-logo` | Hide the ASCII logo |
| `--logo <FILE>` | Load a custom ASCII logo |
| `--logo-align <POS>` | Set logo alignment |
| `--config <FILE>` | Load a custom configuration file |

Valid logo alignment values are `top`, `center`, and `bottom`.

`--logo` and `--no-logo` cannot be used together.

---

## Configuration

zfetch supports JSONC-style configuration through a JSON5-compatible parser.

### Default Configuration Location

```text
~/.config/zfetch/config.jsonc
```

When `XDG_CONFIG_HOME` is configured, the default path becomes:

```text
$XDG_CONFIG_HOME/zfetch/config.jsonc
```

If the default configuration file does not exist, zfetch automatically uses its built-in defaults.

### Create a Configuration Directory

```bash
mkdir -p ~/.config/zfetch
```

Create or edit your configuration:

```bash
nano ~/.config/zfetch/config.jsonc
```

### Complete Configuration Example

```jsonc
{
  // ASCII logo configuration
  "logo": {
    "enabled": true,
    "source": "~/.config/zfetch/logo.txt",
    "align": "center"
  },

  // Terminal theme
  "theme": {
    "color": true,
    "logo_color": "red",
    "heading_color": "cyan",
    "label_color": "white",
    "value_color": "gray"
  },

  // Display preferences
  "display": {
    "show_efi": false,
    "show_percentage": true
  },

  // Enabled information modules
  "modules": {
    "system": [
      "os",
      "kernel",
      "arch",
      "shell",
      "de",
      "session",
      "wm",
      "terminal",
      "packages"
    ],

    "hardware": [
      "host",
      "bios",
      "display",
      "cpu",
      "gpu",
      "memory",
      "swap",
      "disk",
      "local_ip",
      "battery",
      "uptime"
    ]
  }
}
```

If you use this example with `logo.source`, create the referenced ASCII logo file first or change the source to an existing file.

### Color Customization

Supported named colors:

| Color | Configuration Value |
|---|---|
| Red | `red` |
| Green | `green` |
| Yellow | `yellow` |
| Blue | `blue` |
| Magenta | `magenta` |
| Cyan | `cyan` |
| White | `white` |
| Gray | `gray` |

To disable colors globally:

```jsonc
{
  "theme": {
    "color": false
  }
}
```

### Hide the Logo

```jsonc
{
  "logo": {
    "enabled": false
  }
}
```

### Custom ASCII Logo

Create a text file containing your own ASCII artwork.

Example:

```bash
nano ~/.config/zfetch/logo.txt
```

Then reference it in the configuration:

```jsonc
{
  "logo": {
    "enabled": true,
    "source": "~/.config/zfetch/logo.txt",
    "align": "center"
  }
}
```

Custom logos can also be specified directly with the `--logo` command-line option.

### Control Information Modules

Remove modules you do not want to display.

Example lightweight configuration:

```jsonc
{
  "logo": {
    "enabled": false
  },

  "modules": {
    "system": [
      "os",
      "kernel",
      "shell"
    ],

    "hardware": [
      "cpu",
      "gpu",
      "memory"
    ]
  }
}
```

Only enabled modules are detected.

Module ordering follows the configuration arrays.

Unknown and duplicate module names are rejected during configuration validation.

### Disk and Percentage Display

To control EFI visibility and percentage formatting:

```jsonc
{
  "display": {
    "show_efi": true,
    "show_percentage": true
  }
}
```

---

## Architecture

zfetch is implemented in Rust and uses modular system information collectors.

### Core Components

| Component | Responsibility |
|---|---|
| CLI | Command-line parsing |
| Config | JSONC configuration and validation |
| System | Linux system and hardware detection |
| Display | Terminal output formatting |
| Logo | ASCII artwork rendering |
| Colors | ANSI color formatting |
| Layout | Output alignment and formatting |

### Detection Strategy

zfetch uses a combination of:

- Native Linux `/proc` interfaces
- Linux `/sys` interfaces
- Filesystem metadata
- POSIX/Linux system calls
- GNOME D-Bus interfaces where supported
- External command fallbacks when necessary

Selected detection modules run concurrently while smaller operations are performed on the main thread.

### Lazy Detection

Modules disabled in the configuration are skipped during data collection.

This reduces unnecessary operations, especially when only a small selection of system information is required.

---

## Development

### Requirements

- Linux development environment
- Rust stable toolchain with Rust 2024 edition support
- Cargo
- Rustfmt
- Clippy

The repository includes `rust-toolchain.toml` to select the stable toolchain and required development components.

### Formatting

```bash
cargo fmt --all
```

Check formatting:

```bash
cargo fmt --all -- --check
```

### Code Validation

```bash
cargo check
```

### Clippy

```bash
cargo clippy --all-targets -- -D warnings
```

### Unit Tests

```bash
cargo test
```

At the time of the first release preparation, zfetch passed all 36 available unit tests.

### Build

Standard optimized build:

```bash
cargo build --release
```

Compact optimized build:

```bash
cargo build --profile compact
```

### Performance Profiling

Enable internal module profiling:

```bash
ZFETCH_PROFILE=1 ./target/compact/zfetch --no-logo
```

This prints individual module timing information to standard error.

Profiling results may be influenced by parallel execution and system scheduling and should not be interpreted as an exact breakdown of total wall-clock startup time.

---

## Compatibility

zfetch is currently developed and tested primarily on Ubuntu Linux.

### Tested Environment

- Ubuntu 26.04.1 LTS
- Linux x86_64
- GNOME 50.1
- Wayland
- Dell Latitude 3320
- Intel Iris Xe Graphics

Other Linux distributions, desktop environments, hardware configurations, and package managers may produce different results or have limited detection support.

The availability of GNOME-specific information depends on the corresponding D-Bus services.

**The initial Debian package is not guaranteed to run on older Ubuntu or Debian releases**, particularly when their glibc versions do not satisfy the package requirement.

---

## Roadmap

### Completed

- [x] Rust-based Linux system information tool
- [x] Native system and hardware detection
- [x] CPU and GPU information
- [x] Memory, swap, and disk information
- [x] Local IPv4 detection
- [x] Battery and uptime detection
- [x] GNOME desktop and display detection
- [x] Package counting
- [x] JSONC configuration support
- [x] Custom ASCII logos
- [x] Customizable terminal colors
- [x] Configurable modules and ordering
- [x] Lazy detection
- [x] Parallel detection
- [x] Compact Rust build profile
- [x] Debian `.deb` packaging
- [x] Local Debian package installation testing

### Planned

- [ ] Broader Linux distribution compatibility
- [ ] Additional desktop environment testing
- [ ] Automated GitHub release builds
- [ ] Additional hardware detection improvements
- [ ] Further performance and memory optimizations
- [ ] Expanded documentation and examples

---

## Troubleshooting

### Command Not Found

If `zfetch` is not found after installing the Debian package, verify the installation:

```bash
dpkg -s zfetch
```

Check the executable location:

```bash
command -v zfetch
```

For an installed Debian package, the executable should normally be:

```text
/usr/bin/zfetch
```

### Configuration Errors

Validate your configuration syntax and supported module names.

You can test with a specific configuration:

```bash
zfetch --config ~/.config/zfetch/config.jsonc
```

To temporarily use the built-in defaults, rename your existing configuration file or use a separate valid configuration.

### Missing Hardware Information

Some information depends on the availability of Linux kernel interfaces, hardware drivers, desktop services, or supported detection backends.

Please include your Linux distribution, desktop environment, and relevant command output when reporting issues.

---

## Uninstallation

To remove the Debian package:

```bash
sudo apt remove zfetch
```

The package can also be removed using your preferred Debian-compatible package manager.

User-created configuration files in `~/.config/zfetch/` are not managed by the Debian package and can be retained for future installations.

---

## Contributing

Contributions are welcome.

You can help by:

- Reporting bugs
- Testing zfetch on different Linux distributions
- Improving hardware detection
- Suggesting new features
- Improving documentation
- Submitting pull requests

Please open an issue before proposing significant architectural changes.

**[Issues](https://github.com/azislamayuda/zfetch/issues)**  
**[Pull Requests](https://github.com/azislamayuda/zfetch/pulls)**

When contributing code, please ensure the project builds successfully and passes Rustfmt, Clippy, and unit tests.

---

## License

zfetch is free and open-source software distributed under the **MIT License**.

See [LICENSE](LICENSE) for the complete license text.

---

## Author

**Azis Lamayuda**

GitHub: [@azislamayuda](https://github.com/azislamayuda)

Repository: [github.com/azislamayuda/zfetch](https://github.com/azislamayuda/zfetch)

---

**zfetch — Fast. Lightweight. Rust-powered.**

Made for Linux, built with Rust.