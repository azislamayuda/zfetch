# zfetch v0.3.0 — CLI Reference

## Normal terminal output

| Command | What it does |
| --- | --- |
| `zfetch` | Classic display, honors `~/.config/zfetch/config.jsonc` and custom logo |
| `zfetch --layout compact --preset minimal` | Simple vertical list, small module selection |
| `zfetch --layout detailed` | Classic view with opt-in hardware details |
| `zfetch --no-logo --no-color` | Plain text suitable for a log |
| `zfetch --only os,kernel,cpu,memory` | Detect only named modules |
| `zfetch --exclude disk,battery` | Exclude named modules |
| `zfetch --list-modules` | List available module identifiers |
| `zfetch --theme nord --layout classic` | Apply an ANSI named-color theme |
| `zfetch --list-themes` | List theme names and color previews |
| `zfetch --init-config` | Create configuration if none exists; never overwrite |
| `zfetch --validate-config` | Validate the current JSONC configuration |

## JSON

```bash
zfetch --format json --only os,kernel,cpu,memory | python3 -m json.tool
zfetch --format json --only cpu,gpu,display,battery --hardware-details
```

- JSON schema is version `1`, with top-level `schema_version`, `zfetch_version`, `identity`, `system`, and `hardware` fields.
- `--hardware-details` makes best-effort, opt-in sysfs queries. Metadata such as `cpu_details`, `gpu_details`, `display_details`, or `battery_details` may be missing when drivers cannot provide them.
- Unknown or unavailable hardware details must never be invented.
- `--layout compact` and `--layout detailed` are **terminal-only** and are intentionally rejected when combined with `--format json`.
- JSON is kept free of ASCII logos and terminal escape codes.
- Numeric memory byte fields are not yet part of the v0.3.0 JSON contract. The current `memory` value remains a formatted string.

## Update Manager

| Command | Network? | Does it install anything? |
| --- | --- | --- |
| `zfetch` | No update network calls | No |
| `zfetch --check-update` | Yes, official GitHub Releases | No |
| `zfetch --update` | Yes, official GitHub Releases | Can offer an interactive, checksum-verified compatible `.deb` upgrade |

Never treat an `--update` prompt as evidence the release has passed security review. Check the download source, Debian package version, dependency changes and SHA-256 verification before accepting.
