# zfetch v0.5.0 — CLI Reference

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

## Diagnostics (new in 0.5.0)

| Command | What it does |
| --- | --- |
| `zfetch --doctor` | Read-only OK / WARN / INFO checklist; no network, subprocesses, or writes |
| `zfetch --doctor --privacy` | Same checklist with username, hostname, and home paths redacted |
| `zfetch --doctor --config FILE` | Check a specific config file |
| `zfetch --benchmark` | Per-module detection time table (threaded modules overlap) |

`--doctor` and `--benchmark` are action flags: they cannot be combined with other action
flags or with display options. `--doctor` accepts `--privacy` and `--config`; `--benchmark`
accepts `--config`.

## Privacy

`--privacy` redacts the username, hostname, local IPv4, IPv6 and MAC addresses,
JSON keys such as `serial_number` / `mac_address` / `ssid`, and personal mount paths
(`/home/`, `/media/`, `/run/media/`, `/mnt/`). The same redaction applies to terminal
output, JSON, `--export`, and `--doctor`. It is **not** a guarantee of anonymity.

zfetch has no network-interface module, so MAC addresses are not normally shown. The
redaction acts as a guard for free text and future modules. DMI serial numbers are not
collected; `serial`-type JSON keys are redacted in case a future module adds them.

## JSON

```bash
zfetch --format json --only os,kernel,cpu,memory | python3 -m json.tool
zfetch --format json --only cpu,gpu,display,battery --hardware-details
```

- The schema is still version `1`. Every existing key and value format is unchanged.
- `--layout compact` and `--layout detailed` are terminal-only and are rejected with `--format json`.

## Update Manager

| Command | Network? | Does it install anything? |
| --- | --- | --- |
| `zfetch` | No | No |
| `zfetch --check-update` | Yes, official GitHub Releases | No; also prints the detected install method |
| `zfetch --update` on a `.deb` install | Yes | Only after `[y/N]` confirmation, with SHA-256, name, version, and architecture checks |
| `zfetch --update` on Arch, RPM, or portable | Yes (release lookup only) | No; prints the safe manual command |
