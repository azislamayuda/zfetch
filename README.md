# zfetch

**A fast, lightweight, and customizable Linux system information tool written in Rust.**

zfetch is an open-source command-line application designed to display essential system and hardware information with minimal overhead. It is an alternative to Fastfetch, focused on performance, simplicity, and customization.

**Current version:** v0.1.0  
**Platform:** Linux  
**Language:** Rust  
**License:** MIT

## Features

- Fast system information detection.
- Lightweight native Rust executable.
- Configurable ASCII logo and alignment.
- Customizable terminal colors.
- JSONC configuration support.
- Configurable module visibility and ordering.
- Lazy detection to avoid unnecessary operations.
- Parallel detection for selected system modules.
- Multi-display information support.
- Compact optimized release builds.

### System Information

- Operating System
- Kernel
- Architecture
- Shell
- Desktop Environment
- Display Session
- Window Manager
- Terminal
- Installed Packages

### Hardware Information

- Host / Device Model
- BIOS
- Display
- CPU
- GPU
- Memory
- Swap
- Disk Usage
- Local IP Address
- Battery
- System Uptime

Package detection currently includes DPKG, Flatpak, and AppImage on supported Linux environments.

## Performance

zfetch is designed to minimize startup latency and memory consumption.

Measurements from the development machine:

| Metric | Result |
|---|---|
| Full system startup | ~4.6 ms |
| Minimal configuration | ~0.645 ms |
| Peak memory usage | ~3.6 MiB |
| Compact binary size | ~1.52 MB |

**Test environment:** Dell Latitude 3320, Intel Core i5-1145G7, Ubuntu 26.04.1 LTS.

These measurements are development benchmarks, not guaranteed results. Performance varies depending on hardware, operating system, enabled modules, and system load.

## Installation

### Build from source

Install the stable Rust toolchain and Git, then run:

```bash
git clone https://github.com/azislamayuda/zfetch.git
cd zfetch

cargo build --profile compact
```

Run zfetch:

```bash
./target/compact/zfetch
```

Alternatively, build the standard release:

```bash
cargo build --release
./target/release/zfetch
```

### Debian / Ubuntu packages

Prebuilt `.deb` packaging is planned for the first GitHub release.

Until that package is published, use the source build instructions above.

## Usage

Display system information:

```bash
zfetch
```

Disable the ASCII logo:

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
zfetch --logo-align center
```

Supported logo alignments: `top`, `center`, `bottom`.

Use a custom configuration:

```bash
zfetch --config /path/to/config.jsonc
```

Show help or version:

```bash
zfetch --help
zfetch --version
```

If zfetch is not installed system-wide, replace `zfetch` with `./target/compact/zfetch` when running it from the project directory.

## Configuration

zfetch supports JSONC-style configuration using the JSON5 parser.

Default configuration location:

```text
~/.config/zfetch/config.jsonc
```

If `XDG_CONFIG_HOME` is set, zfetch uses:

```text
$XDG_CONFIG_HOME/zfetch/config.jsonc
```

A missing default configuration file is allowed; zfetch uses its built-in defaults.

### Example configuration

```jsonc
{
  "logo": {
    "enabled": true,
    "source": "~/.config/zfetch/logo.txt",
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

### Available colors

`red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray`.

### Module customization

Remove a module from its corresponding array to disable detection.

For example:

```jsonc
{
  "modules": {
    "system": ["os", "kernel", "shell"],
    "hardware": ["cpu", "gpu", "memory"]
  }
}
```

Module ordering follows the configuration arrays. Unsupported or duplicate module names are rejected.

## Development

zfetch uses the Rust stable toolchain.

Run formatting checks:

```bash
cargo fmt --all -- --check
```

Run the compiler checks:

```bash
cargo check
```

Run Clippy:

```bash
cargo clippy --all-targets -- -D warnings
```

Run unit tests:

```bash
cargo test
```

Build the compact release:

```bash
cargo build --profile compact
```

For development profiling:

```bash
ZFETCH_PROFILE=1 ./target/compact/zfetch
```

## Compatibility

zfetch is currently developed and tested primarily on Ubuntu Linux.

Some hardware and desktop detection functionality depends on Linux system interfaces, installed utilities, and desktop environment services. In particular, GNOME-specific D-Bus interfaces are used for selected desktop and display information.

Compatibility across other Linux distributions and desktop environments requires further testing.

## Roadmap

- [x] Rust-based system information collector
- [x] Native Linux hardware detection
- [x] JSONC configuration
- [x] Custom ASCII logo
- [x] Custom terminal colors
- [x] Lazy detection and parallel processing
- [x] Compact binary optimization
- [ ] Debian `.deb` packaging
- [ ] GitHub Releases
- [ ] Broader Linux distribution testing
- [ ] Automated release builds

## Contributing

Contributions, bug reports, and feature suggestions are welcome.

Please open a GitHub Issue to report problems or propose improvements.

Repository: https://github.com/azislamayuda/zfetch

## License

zfetch is released under the MIT License. See the `LICENSE` file for details.

## Author

**Azis Lamayuda**

GitHub: https://github.com/azislamayuda

---

**zfetch — Fast. Lightweight. Rust-powered.**
