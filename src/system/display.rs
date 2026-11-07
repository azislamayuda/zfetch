use std::{fs, path::Path, process::Command};

fn is_builtin(connector: &str) -> bool {
    connector.starts_with("eDP") || connector.starts_with("LVDS") || connector.starts_with("DSI")
}

fn format_display(width: i32, height: i32, refresh: f64, connector: &str, builtin: bool) -> String {
    let hz = if (refresh - refresh.round()).abs() < 0.01 {
        format!("{refresh:.0}")
    } else {
        format!("{refresh:.2}")
    };

    let label = if builtin { " [Built-in]" } else { "" };

    format!("{width}x{height} @ {hz} Hz{label} ({connector})")
}

// ============================================
// NATIVE DRM/EDID DETECTION (no subprocess)
// ============================================

/// Active size and refresh rate of the EDID preferred timing (first detailed descriptor).
fn edid_preferred_mode(edid: &[u8]) -> Option<(i32, i32, f64)> {
    const HEADER: [u8; 8] = [0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00];
    if edid.len() < 128 || edid[..8] != HEADER {
        return None;
    }
    let d = &edid[54..72];
    let clock_hz = f64::from(u16::from_le_bytes([d[0], d[1]])) * 10_000.0;
    let h_active = i32::from(d[2]) | (i32::from(d[4] >> 4) << 8);
    let h_blank = i32::from(d[3]) | (i32::from(d[4] & 0x0f) << 8);
    let v_active = i32::from(d[5]) | (i32::from(d[7] >> 4) << 8);
    let v_blank = i32::from(d[6]) | (i32::from(d[7] & 0x0f) << 8);
    let pixels = f64::from((h_active + h_blank) * (v_active + v_blank));
    if clock_hz == 0.0 || h_active == 0 || v_active == 0 || pixels == 0.0 {
        return None;
    }
    Some((h_active, v_active, clock_hz / pixels))
}

// shortcut: reports the EDID preferred mode, not a user-changed current mode;
// read the compositor's current mode if that difference matters.
fn drm_displays(root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut devices: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
    devices.sort();
    let mut displays = Vec::new();
    for device in devices {
        let Some((card, connector)) = device
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.split_once('-'))
        else {
            continue;
        };
        if !card.starts_with("card") || !card[4..].chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let read = |name| fs::read_to_string(device.join(name)).unwrap_or_default();
        if read("status").trim() != "connected" || read("enabled").trim() == "disabled" {
            continue;
        }
        let Some((width, height, refresh)) = fs::read(device.join("edid"))
            .ok()
            .and_then(|edid| edid_preferred_mode(&edid))
        else {
            continue;
        };
        displays.push(format_display(
            width,
            height,
            refresh,
            connector,
            is_builtin(connector),
        ));
    }
    displays
}

// ============================================
// GDCTL FALLBACK
// ============================================

fn parse_mode(line: &str) -> Option<(i32, i32, f64)> {
    let mode = line
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .split_whitespace()
        .next()?;

    let (resolution, refresh) = mode.split_once('@')?;
    let (width, height) = resolution.split_once('x')?;

    Some((
        width.parse().ok()?,
        height.parse().ok()?,
        refresh.parse().ok()?,
    ))
}

fn gdctl_fallback() -> Vec<String> {
    let Ok(output) = Command::new("gdctl").arg("show").output() else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&output.stdout);

    let mut displays = Vec::new();
    let mut connector = String::new();
    let mut expect_mode = false;

    for line in text.lines() {
        if let Some((_, name)) = line.split_once("Monitor ") {
            connector = name
                .split_whitespace()
                .next()
                .unwrap_or("Unknown")
                .to_string();

            expect_mode = false;
            continue;
        }

        if line.contains("Current mode") {
            expect_mode = true;
            continue;
        }

        if expect_mode {
            if let Some((width, height, refresh)) = parse_mode(line) {
                displays.push(format_display(
                    width,
                    height,
                    refresh,
                    &connector,
                    is_builtin(&connector),
                ));
            }

            expect_mode = false;
        }
    }

    displays
}

// ============================================
// PUBLIC DISPLAY DETECTOR
// ============================================

pub fn active() -> Vec<String> {
    let displays = drm_displays(Path::new("/sys/class/drm"));
    if displays.is_empty() {
        // Fallback for systems without readable EDID (e.g. some VMs).
        gdctl_fallback()
    } else {
        displays
    }
}

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_current_mode() {
        assert_eq!(
            parse_mode("│   └──1920x1080@60.000"),
            Some((1920, 1080, 60.0))
        );
    }

    #[test]
    fn format_builtin_display() {
        assert_eq!(
            format_display(1920, 1080, 60.0, "eDP-1", true),
            "1920x1080 @ 60 Hz [Built-in] (eDP-1)"
        );
    }

    #[test]
    fn format_external_display() {
        assert_eq!(
            format_display(2560, 1440, 59.94, "HDMI-1", false),
            "2560x1440 @ 59.94 Hz (HDMI-1)"
        );
    }

    #[test]
    fn edid_preferred_timing() {
        // 1920x1080, 141.0 MHz pixel clock, 2000x1175 total -> ~60.0 Hz.
        let mut edid = [0u8; 128];
        edid[..8].copy_from_slice(&[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]);
        edid[54..62].copy_from_slice(&[0x14, 0x37, 0x80, 0x50, 0x70, 0x38, 0x5f, 0x40]);
        let (w, h, hz) = edid_preferred_mode(&edid).unwrap();
        assert_eq!((w, h), (1920, 1080));
        assert_eq!(
            format_display(w, h, hz, "eDP-1", true),
            "1920x1080 @ 60 Hz [Built-in] (eDP-1)"
        );
        edid[0] = 1;
        assert_eq!(edid_preferred_mode(&edid), None);
    }

    #[test]
    fn builtin_connector_detection() {
        assert!(is_builtin("eDP-1"));
        assert!(is_builtin("LVDS-1"));
        assert!(!is_builtin("HDMI-1"));
        assert!(!is_builtin("DP-1"));
    }
}
