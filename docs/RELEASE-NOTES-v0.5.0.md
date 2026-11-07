# zfetch v0.5.0 — Release Notes

zfetch v0.5.0 is a smaller, lighter release. It adds diagnostics, stronger privacy, and a
safer update manager. Normal runs remain offline.

## Highlights

- **`zfetch --doctor`**: a read-only OK / WARN / INFO checklist covering the config file,
  logo source, `/etc/os-release`, `/proc`, `/sys/class/power_supply`, `/sys/class/drm`,
  package databases, `XDG_CURRENT_DESKTOP` / `XDG_SESSION_TYPE`, and the optional tools
  `gdctl` and `curl`. Combine it with `--privacy` before you paste the output publicly.
- **Privacy Mode 2.0**: `--privacy` now also hides MAC addresses, IPv6 addresses, DMI
  serial numbers, and the username and hostname wherever they appear. Terminal output, JSON,
  `--export`, and `--doctor` share one redaction path. Privacy Mode is **not** a guarantee
  of anonymity, so review every report before you share it.
- **Universal update manager**: `--check-update` shows how zfetch was installed (`.deb`,
  Arch package, RPM, or portable/source), detected from files only. `--update` still
  installs only verified `.deb` packages after you confirm. For every other method it
  prints a safe manual command and changes nothing.
- **More package managers**: Alpine APK, Gentoo Portage, and Void XBPS.
- **More window managers**: Hyprland, Sway, niri, and i3.
- **`zfetch --benchmark`**: a per-module detection time table.

## Smaller and faster

The `zbus` dependency, together with its async runtime, is removed: 76 fewer locked crates.
The compact binary is about 53% smaller and peak RSS is about 18% lower. A default run makes
no D-Bus connections and starts no new subprocesses.

## Behavior changes

- **Display list**: displays now come from DRM connectors and EDID in `/sys/class/drm`.
  The line shows the panel's *preferred* mode. If you set a different resolution or refresh
  rate in your desktop settings, zfetch still shows the preferred one. Connector names
  follow the kernel naming, for example `HDMI-A-1`. `gdctl` is used only when no EDID can be read.
- **GNOME version**: read from the installed `gnome-shell` package (dpkg) or from
  `gnome-shell --version`. After an upgrade without a re-login, it can show the new
  version while the old shell is still running.
- **Swap percentage**: now rounded like memory and disk (previously truncated), so
  it can differ by 1%.
- **WM detection**: uses the same exact-token matching as DE detection.
- **`--update` on non-`.deb` installs**: now exits with status 0 and prints manual
  steps, instead of exiting with an error.
- **Help**: the description column is aligned for `--hardware-details`, `--privacy`, and `--export`.
- The updater's user agent now carries the real version (`zfetch-updater/0.5.0`).

## Compatibility

- JSON `schema_version` stays `1`, and every existing field is unchanged.
- Every existing CLI flag keeps its behavior. The new flags are `--doctor` and `--benchmark`,
  and `--doctor` accepts `--privacy` and `--config`.
- Nix package counting is not included, because profile generations do not map reliably to
  installed package counts.

## Postponed to 0.6.0

Snapshots and comparison (`--snapshot`, `--compare`), and image or animation rendering.
